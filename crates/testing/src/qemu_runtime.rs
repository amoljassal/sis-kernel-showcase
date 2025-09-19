// SIS Kernel QEMU Runtime Integration
// Interfaces with QEMU instances running the SIS kernel for real testing

use crate::{TestSuiteConfig, TestError};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::collections::HashMap;
use tokio::process::{Command, Child};
use tokio::time::{sleep, Duration};
use std::path::{Path, PathBuf};
use tokio::fs;

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

#[derive(Debug)]
pub struct QEMURuntimeManager {
    _config: TestSuiteConfig,
    cluster: QEMUCluster,
    processes: HashMap<usize, Child>,
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

        Self { _config: config.clone(), cluster, processes: HashMap::new() }
    }

    pub async fn build_kernel(&self) -> Result<(), TestError> {
        log::info!("Building SIS kernel for QEMU testing");
        
        // Build the kernel in release mode for accurate performance testing
        let root = Self::workspace_root();
        // Build features: bringup + neon-optimized, with optional graph-autostats when SIS_GRAPH_STATS=1
        let mut features = "bringup,neon-optimized".to_string();
        if std::env::var("SIS_GRAPH_STATS").unwrap_or_default() == "1" {
            features.push_str(",graph-autostats");
        }
        let output = Command::new("cargo")
            .args([
                "+nightly",
                "build",
                "-p", "sis_kernel",
                "-Z", "build-std=core,alloc",
                "--target", "aarch64-unknown-none",
                "--features", &features
            ])
            .current_dir(&root)
            .env("RUSTFLAGS", format!(
                "-C link-arg=-T{}",
                root.join("crates/kernel/src/arch/aarch64/aarch64-qemu.ld").display()
            ))
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

        // Build the UEFI bootloader - run from workspace root
        let output = Command::new("cargo")
            .args([
                "build",
                "-p", "uefi-boot",
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
            "-smp".to_string(), "4".to_string(),  // Multi-core for realistic M-series behavior
            "-m".to_string(), "1G".to_string(),  // Increased memory for better performance
            "-nographic".to_string(),
            // Use chardev with socket for bidirectional serial communication + file logging
            "-chardev".to_string(), format!("socket,id=serial0,port={},host=localhost,server,nowait,logfile={}", 
                                           instance.serial_port, instance.serial_log_path),
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
        
        // Add cycle-accurate simulation for performance measurement
        qemu_args.extend([
            "-icount".to_string(), "shift=0".to_string(),  // Cycle-accurate for benchmarking
            "-object".to_string(), "memory-backend-ram,id=ram,size=1G,prealloc=on".to_string(),  // Preallocate memory
            "-numa".to_string(), "node,memdev=ram".to_string(),  // NUMA awareness for M-series simulation
        ]);
        
        // Add network device for distributed testing
        qemu_args.extend([
            "-netdev".to_string(), format!("user,id=net0,hostfwd=tcp::{}-:22", instance.network_port),
            "-device".to_string(), "virtio-net-pci,netdev=net0".to_string(),
        ]);
        
        log::debug!("QEMU command: qemu-system-aarch64 {}", qemu_args.join(" "));
        
        let qemu_process = Command::new("qemu-system-aarch64")
            .args(qemu_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to launch QEMU instance {}: {}", instance.node_id, e)
            })?;

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
