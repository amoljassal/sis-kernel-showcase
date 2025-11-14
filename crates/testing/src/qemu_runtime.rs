// SIS Kernel QEMU Runtime Integration
// Interfaces with QEMU instances running the SIS kernel for real testing

use crate::{TestSuiteConfig, TestError};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::{Command, Child};
use tokio::time::{sleep, Duration};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::fs::OpenOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QEMUInstance {
    pub node_id: usize,
    pub serial_port: u16,
    pub monitor_port: u16,
    pub network_port: u16,
    pub esp_directory: String,
    pub serial_log_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QEMUCluster {
    pub instances: Vec<QEMUInstance>,
    pub base_port: u16,
    pub total_nodes: usize,
}

pub struct QEMURuntimeManager {
    _config: TestSuiteConfig,
    cluster: QEMUCluster,
    processes: HashMap<usize, Child>,
    serial_writers: Arc<Mutex<HashMap<usize, tokio::fs::File>>>,  // PTY write file descriptors
}

impl QEMURuntimeManager {
    fn workspace_root() -> PathBuf {
        // crates/testing -> crates -> workspace root
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
    }
    /// Detect if running on Apple Silicon for HVF acceleration
    #[allow(dead_code)]
    async fn is_apple_silicon() -> bool {
        if cfg!(target_os = "macos") {
            // Check if we're on Apple Silicon by looking for the "Apple" brand in CPU info
            match Command::new("sysctl")
                .args(["-n", "machdep.cpu.brand_string"])
                .output()
                .await
            {
                Ok(output) => {
                    let cpu_info = String::from_utf8_lossy(&output.stdout);
                    cpu_info.contains("Apple")
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn new(config: &TestSuiteConfig) -> Self {
        let base_port = 7000;
        let instances = (0..config.qemu_nodes)
            .map(|node_id| QEMUInstance {
                node_id,
                serial_port: base_port + node_id as u16,
                monitor_port: base_port + 100 + node_id as u16,
                network_port: base_port + 200 + node_id as u16,
                esp_directory: format!("target/testing/esp-node{}", node_id),
                serial_log_path: format!("target/testing/serial-node{}.log", node_id),
            })
            .collect();

        let cluster = QEMUCluster {
            instances,
            base_port,
            total_nodes: config.qemu_nodes,
        };

        Self {
            _config: config.clone(),
            cluster,
            processes: HashMap::new(),
            serial_writers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn build_kernel(&self) -> Result<(), TestError> {
        log::info!("Building SIS kernel for QEMU testing");

        let root = Self::workspace_root();

        // Build features: allow override via SIS_TEST_FEATURES; default includes Phase 7/8 features
        let features_str = if let Ok(features) = std::env::var("SIS_TEST_FEATURES") {
            // Use the specified features directly (replace, don't append)
            features
        } else {
            // Default: All production features for comprehensive testing
            "bringup,graphctl-framed,deterministic,ai-ops,crypto-real,agentsys,llm,otel,decision-traces,model-lifecycle,shadow-mode".to_string()
        };

        // Build UEFI bootloader first
        log::info!("Building UEFI bootloader...");
        let output = Command::new("cargo")
            .args([
                "build",
                "--manifest-path", "crates/uefi-boot/Cargo.toml",
                "--release",
                "--target", "aarch64-unknown-uefi"
            ])
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to run UEFI build: {}", e)
            })?;

        if !output.status.success() {
            return Err(TestError::QEMUError {
                message: format!("UEFI build failed: {}", String::from_utf8_lossy(&output.stderr))
            });
        }

        // Build kernel using --manifest-path (not -p) to avoid workspace feature errors
        log::info!("Building kernel with features: {}", features_str);
        let output = Command::new("cargo")
            .args([
                "+nightly",
                "build",
                "--manifest-path", "crates/kernel/Cargo.toml",
                "-Z", "build-std=core,alloc",
                "--target", "aarch64-unknown-none",
                "--features", &features_str
            ])
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to run kernel build: {}", e)
            })?;

        if !output.status.success() {
            return Err(TestError::QEMUError {
                message: format!("Kernel build failed: {}", String::from_utf8_lossy(&output.stderr))
            });
        }

        log::info!("SIS kernel and UEFI bootloader built successfully");
        Ok(())
    }

    pub async fn prepare_esp_directories(&self) -> Result<(), TestError> {
        log::info!("Preparing ESP directories for {} QEMU instances", self.cluster.total_nodes);

        for instance in &self.cluster.instances {
            // Create ESP directory structure
            let esp_dir = &instance.esp_directory;
            let efi_boot_dir = format!("{}/EFI/BOOT", esp_dir);
            let efi_sis_dir = format!("{}/EFI/SIS", esp_dir);

            std::fs::create_dir_all(&efi_boot_dir)
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to create ESP directory {}: {}", efi_boot_dir, e)
                })?;

            std::fs::create_dir_all(&efi_sis_dir)
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to create ESP directory {}: {}", efi_sis_dir, e)
                })?;

            // Copy UEFI and kernel binaries
            let uefi_source = Self::workspace_root().join("target/aarch64-unknown-uefi/release/uefi-boot.efi");
            let kernel_source = Self::workspace_root().join("target/aarch64-unknown-none/debug/sis_kernel");   // Use debug build for stability
            let uefi_dest = format!("{}/BOOTAA64.EFI", efi_boot_dir);
            let kernel_dest = format!("{}/KERNEL.ELF", efi_sis_dir);

            std::fs::copy(&uefi_source, &uefi_dest)
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to copy UEFI binary from {} to {}: {}", uefi_source.display(), uefi_dest, e)
                })?;

            std::fs::copy(&kernel_source, &kernel_dest)
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to copy kernel binary from {} to {}: {}", kernel_source.display(), kernel_dest, e)
                })?;
        }

        log::info!("ESP directories prepared for all instances");
        Ok(())
    }

    pub async fn launch_cluster(&mut self) -> Result<(), TestError> {
        log::info!("Launching QEMU cluster with {} nodes", self.cluster.total_nodes);

        let instances = self.cluster.instances.clone();
        for instance in instances {
            self.launch_instance(&instance).await?;
            sleep(Duration::from_secs(3)).await; // Stagger launches
        }

        log::info!("All QEMU instances launched successfully");
        Ok(())
    }

    async fn launch_instance(&mut self, instance: &QEMUInstance) -> Result<(), TestError> {
        log::info!("Launching QEMU instance {} on ports {}/{}/{}", 
                  instance.node_id, instance.serial_port, instance.monitor_port, instance.network_port);
        
        // Use a consistent emulated CPU like the manual runner
        // (HVF/host model can behave differently for bare-metal UEFI)
        let cpu_type = "cortex-a72,pmu=on";
        
        // Optimize QEMU configuration for Apple Silicon development
        let firmware_path = "/opt/homebrew/share/qemu/edk2-aarch64-code.fd";
        
        // Ensure parent directory for logs exists
        if let Some(parent) = Path::new(&instance.serial_log_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| TestError::QEMUError {
                    message: format!("Failed to create log directory {}: {}", parent.display(), e)
                })?;
            }
        }

        // Clear previous log if exists
        let _ = fs::remove_file(&instance.serial_log_path).await;

        let mut qemu_args = vec![
            "-name".to_string(), format!("sis-node{}", instance.node_id),
            "-M".to_string(), "virt,gic-version=3,highmem=on,secure=off".to_string(),  // Enable highmem for M-series simulation
            "-cpu".to_string(), cpu_type.to_string(),
            "-smp".to_string(), "2".to_string(),  // 2 cores for testing
            "-m".to_string(), "512M".to_string(),  // Match manual test setup
            "-nographic".to_string(),
            // Use PTY for bidirectional serial communication
            "-chardev".to_string(), "pty,id=serial0".to_string(),
            "-serial".to_string(), "chardev:serial0".to_string(),
            "-monitor".to_string(), format!("tcp:localhost:{},server,nowait", instance.monitor_port),
            "-bios".to_string(), firmware_path.to_string(),
            "-drive".to_string(), format!("if=none,id=esp,format=raw,file=fat:rw:{}", instance.esp_directory),
            "-device".to_string(), "virtio-blk-pci,drive=esp".to_string(),
            "-device".to_string(), "virtio-rng-pci".to_string(),
            "-no-reboot".to_string(),
            // These -append kernel params are for Linux; left out for bare-metal kernel
            "-d".to_string(), "unimp,guest_errors".to_string(),
        ];

        // Note: HVF is intentionally disabled for stability in bare‑metal bring‑up tests

        // Add network device for distributed testing (use device variant for ARM virt)
        qemu_args.extend([
            "-netdev".to_string(), "user,id=n0".to_string(),
            "-device".to_string(), "virtio-net-device,netdev=n0".to_string(),
        ]);

        // Add GPU device (needed for full boot)
        qemu_args.extend([
            "-device".to_string(), "virtio-gpu-device".to_string(),
        ]);

        // Add RTC and other options from manual script
        qemu_args.extend([
            "-rtc".to_string(), "base=utc".to_string(),
        ]);

        log::debug!("QEMU command: qemu-system-aarch64 {}", qemu_args.join(" "));

        let mut qemu_process = Command::new("qemu-system-aarch64")
            .args(qemu_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to launch QEMU instance {}: {}", instance.node_id, e)
            })?;

        // Read stdout to get PTY path (QEMU prints it to stdout, not stderr)
        let stdout = qemu_process.stdout.take().expect("Failed to get stdout");
        let stderr = qemu_process.stderr.take().expect("Failed to get stderr");

        use tokio::io::AsyncBufReadExt;
        let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
        let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

        // Wait for PTY path from QEMU output (format: "char device redirected to /dev/ttysXXX")
        let mut pty_path = None;
        let mut all_lines = Vec::new();
        let mut attempts = 0;
        while attempts < 50 && pty_path.is_none() {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    if let Ok(Some(line)) = line {
                        log::debug!("QEMU stdout: {}", line);
                        all_lines.push(line.clone());
                        if line.contains("char device redirected to") {
                            // Extract PTY path from line like: "char device redirected to /dev/ttys004 (label serial0)"
                            if let Some(start) = line.find("/dev/") {
                                if let Some(end) = line[start..].find(" ") {
                                    pty_path = Some(line[start..start+end].to_string());
                                } else {
                                    // No space after path (end of line)
                                    pty_path = Some(line[start..].trim_end_matches(')').to_string());
                                }
                            }
                        }
                    }
                }
                line = stderr_reader.next_line() => {
                    if let Ok(Some(line)) = line {
                        log::debug!("QEMU stderr: {}", line);
                        all_lines.push(line.clone());
                        if line.contains("char device redirected to") {
                            // Extract PTY path from line like: "char device redirected to /dev/ttys004 (label serial0)"
                            if let Some(start) = line.find("/dev/") {
                                if let Some(end) = line[start..].find(" ") {
                                    pty_path = Some(line[start..start+end].to_string());
                                } else {
                                    // No space after path (end of line)
                                    pty_path = Some(line[start..].trim_end_matches(')').to_string());
                                }
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    attempts += 1;
                }
            }
        }

        if pty_path.is_none() {
            log::error!("Failed to find PTY path. Captured {} lines:", all_lines.len());
            for line in &all_lines {
                log::error!("  {}", line);
            }
        }

        let pty_path = pty_path.ok_or_else(|| TestError::QEMUError {
            message: format!("Failed to get PTY path from QEMU output for instance {}", instance.node_id)
        })?;

        log::info!("Instance {} using PTY: {}", instance.node_id, pty_path);

        // Spawn background tasks to drain QEMU's stdout/stderr to prevent pipe blocking
        // This prevents QEMU output from interfering with PTY communication
        tokio::spawn(async move {
            while let Ok(Some(_line)) = stdout_reader.next_line().await {
                // Discard QEMU stdout after PTY path extraction
            }
        });
        tokio::spawn(async move {
            while let Ok(Some(_line)) = stderr_reader.next_line().await {
                // Discard QEMU stderr after PTY path extraction
            }
        });

        // Open PTY device separately for reading and writing to avoid contention
        // Using tokio::io::split on a single file causes blocking issues with PTY devices
        let pty_read = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&pty_path)
            .await
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to open PTY {} for reading: {}", pty_path, e)
            })?;

        let pty_write = tokio::fs::OpenOptions::new()
            .write(true)
            .open(&pty_path)
            .await
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to open PTY {} for writing: {}", pty_path, e)
            })?;

        // Configure PTY in raw mode for proper character transmission
        // This is critical for capturing kernel output properly
        #[allow(unsafe_code)] // Required for FFI termios operations via nix crate
        {
            use std::os::unix::io::{AsRawFd, BorrowedFd};
            use nix::sys::termios::{self, SetArg};
            let fd = pty_read.as_raw_fd();
            // SAFETY: pty_read owns the fd and remains valid for the duration of this block
            let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
            match termios::tcgetattr(&borrowed_fd) {
                Ok(mut termios_attrs) => {
                    termios::cfmakeraw(&mut termios_attrs);
                    if let Err(e) = termios::tcsetattr(&borrowed_fd, SetArg::TCSANOW, &termios_attrs) {
                        log::warn!("Failed to configure PTY {} in raw mode: {}", pty_path, e);
                    } else {
                        log::debug!("Configured PTY {} in raw mode", pty_path);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get PTY {} attributes: {}", pty_path, e);
                }
            }
        }

        // Use separate file descriptors for reading and writing
        let read_half = pty_read;
        let write_half = pty_write;

        // Spawn background task to capture serial output from read half and write to log file
        let serial_log_path = instance.serial_log_path.clone();

        tokio::spawn(async move {
            // Create or truncate the log file
            let mut log_file = match OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&serial_log_path)
                .await
            {
                Ok(file) => file,
                Err(e) => {
                    log::error!("Failed to create serial log file {}: {}", serial_log_path, e);
                    return;
                }
            };

            // Read from PTY in raw chunks to capture partial lines (like shell prompts)
            use tokio::io::AsyncReadExt;
            let mut read_half = read_half;
            let mut buffer = vec![0u8; 4096];
            let mut line_buffer = Vec::new();

            loop {
                // Use timeout to periodically flush partial lines
                match tokio::time::timeout(
                    tokio::time::Duration::from_millis(100),
                    read_half.read(&mut buffer)
                ).await {
                    Ok(Ok(0)) => break, // EOF
                    Ok(Ok(n)) => {
                        // Process the chunk byte by byte, writing complete lines and buffering partial ones
                        for &byte in &buffer[..n] {
                            if byte == b'\n' {
                                // Complete line - write with prefix
                                let _ = log_file.write_all(b"[QEMU-OUT] ").await;
                                let _ = log_file.write_all(&line_buffer).await;
                                let _ = log_file.write_all(b"\n").await;
                                let _ = log_file.flush().await;
                                line_buffer.clear();
                            } else if byte >= 32 || byte == b'\t' || byte == b'\r' {
                                // Printable character or tab/CR
                                line_buffer.push(byte);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        log::error!("PTY read error: {}", e);
                        break;
                    }
                    Err(_) => {
                        // Timeout - flush partial line if any
                        if !line_buffer.is_empty() {
                            let _ = log_file.write_all(b"[QEMU-OUT] ").await;
                            let _ = log_file.write_all(&line_buffer).await;
                            let _ = log_file.write_all(b"\n").await;
                            let _ = log_file.flush().await;
                            line_buffer.clear();
                        }
                    }
                }
            }

            log::debug!("Serial log capture task finished for {}", serial_log_path);
        });

        // Store write half for command injection
        self.serial_writers.lock().await.insert(instance.node_id, write_half);
        self.processes.insert(instance.node_id, qemu_process);

        log::info!("Instance {} launched (serial log: {})",
                  instance.node_id, instance.serial_log_path);
        Ok(())
    }

    pub async fn shutdown_cluster(&mut self) -> Result<(), TestError> {
        log::info!("Shutting down QEMU cluster");

        for (node_id, mut process) in self.processes.drain() {
            log::info!("Terminating QEMU instance {}", node_id);
            let _ = process.kill().await;
        }

        // Clean up any remaining QEMU processes
        let _ = Command::new("pkill")
            .args(["-f", "qemu-system-aarch64.*sis-node"])
            .output()
            .await;

        log::info!("QEMU cluster shutdown complete");
        Ok(())
    }

    pub async fn read_boot_output(&self, node_id: usize) -> Result<String, TestError> {
        // Read current contents of the serial log file
        if let Some(instance) = self.cluster.instances.iter().find(|i| i.node_id == node_id) {
            match fs::read(&instance.serial_log_path).await {
                Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).to_string()),
                Err(e) => Err(TestError::QEMUError { message: format!(
                    "Failed to read serial log {}: {}",
                    instance.serial_log_path, e
                )}),
            }
        } else {
            Err(TestError::QEMUError {
                message: format!("Instance {} not found in cluster", node_id)
            })
        }
    }

    /// Write a command to the serial console of a specific node via persistent socket connection
    pub async fn write_command(&self, node_id: usize, command: &str) -> Result<(), TestError> {
        let command_with_newline = format!("{}\n", command);

        let mut writers = self.serial_writers.lock().await;
        if let Some(writer) = writers.get_mut(&node_id) {
            use tokio::io::AsyncWriteExt;
            writer.write_all(command_with_newline.as_bytes()).await
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to write command to PTY for node {}: {}", node_id, e)
                })?;
            writer.flush().await
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to flush PTY for node {}: {}", node_id, e)
                })?;

            log::debug!("Sent command to node {} via PTY: {}", node_id, command);
            Ok(())
        } else {
            Err(TestError::QEMUError {
                message: format!("No serial writer found for node {}", node_id)
            })
        }
    }

    pub fn get_cluster_info(&self) -> &QEMUCluster {
        &self.cluster
    }

    pub fn get_serial_log_path(&self, node_id: usize) -> Option<String> {
        self.cluster
            .instances
            .iter()
            .find(|i| i.node_id == node_id)
            .map(|i| i.serial_log_path.clone())
    }

    pub fn get_monitor_port(&self, node_id: usize) -> u16 {
        self.cluster
            .instances
            .iter()
            .find(|i| i.node_id == node_id)
            .map(|i| i.monitor_port)
            .unwrap_or(7100) // Default monitor port
    }

    pub async fn wait_for_boot(&self, node_id: usize, timeout_secs: u64) -> Result<bool, TestError> {
        log::info!("Waiting for instance {} to boot (timeout: {}s)", node_id, timeout_secs);
        
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(timeout_secs);

        while start_time.elapsed() < timeout_duration {
            match self.read_boot_output(node_id).await {
                Ok(output) => {
                    // Look for robust boot markers printed by the kernel
                    let booted = output.contains("!KERNEL(U)")
                        || output.contains("BOOT-ARM64 (UEFI)")
                        || output.contains("SIS UEFI loader v2")
                        || output.contains("MMU ON")
                        || output.contains("HEAP: READY")
                        || output.contains("LAUNCHING SHELL")
                        || output.contains("sis>");
                    if booted {
                        log::info!("Instance {} booted successfully (detected via serial log)", node_id);
                        return Ok(true);
                    }
                    if !output.is_empty() {
                        let sample: String = output.chars().rev().take(200).collect::<String>().chars().rev().collect();
                        log::info!("Instance {} boot output (tail): {}", node_id, sample);
                    }
                }
                Err(e) => {
                    log::debug!("Instance {} serial log not ready: {}", node_id, e);
                }
            }
            sleep(Duration::from_secs(2)).await;
        }

        log::warn!("Instance {} failed to boot within {} seconds", node_id, timeout_secs);
        Ok(false)
    }
}

impl Drop for QEMURuntimeManager {
    fn drop(&mut self) {
        // Ensure cleanup in case shutdown_cluster wasn't called
        for (node_id, mut process) in self.processes.drain() {
            log::debug!("Force terminating QEMU instance {} in drop", node_id);
            let _ = process.start_kill();
        }
    }
}
