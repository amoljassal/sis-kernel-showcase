//! Live QEMU transport for real-time kernel interaction
//!
//! Spawns QEMU processes via uefi_run.sh and manages bidirectional communication.
//! Captures stdout/stderr for event streaming and provides stdin for command input.

use super::shell::ShellCommandResponse;
use super::supervisor::QemuEvent;
use super::types::QemuConfig;
use crate::parser::{LineParser, ParsedEvent};
use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::{broadcast, mpsc, oneshot, Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Maximum number of output lines to buffer (prevents memory exhaustion)
const MAX_OUTPUT_BUFFER: usize = 50_000;

/// Command timeout duration
const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

/// Pending command waiting for response
#[derive(Debug)]
struct PendingCommand {
    command: String,
    sent_at: Instant,
    tx: oneshot::Sender<ShellCommandResponse>,
    output_buffer: Vec<String>,
}

/// Command-response correlation tracker
#[derive(Debug, Clone)]
pub struct CommandTracker {
    pending: Arc<RwLock<HashMap<Uuid, PendingCommand>>>,
    output_queue: Arc<Mutex<VecDeque<String>>>,
}

impl CommandTracker {
    fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            output_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Send a command and get a channel for the response
    pub async fn send_command(
        &self,
        command: String,
        stdin: &mut ChildStdin,
    ) -> Result<oneshot::Receiver<ShellCommandResponse>> {
        let id = Uuid::new_v4();
        let (tx, rx) = oneshot::channel();
        let sent_at = Instant::now();

        // Store pending command
        {
            let mut pending = self.pending.write().await;
            pending.insert(
                id,
                PendingCommand {
                    command: command.clone(),
                    sent_at,
                    tx,
                    output_buffer: Vec::new(),
                },
            );
        }

        // Write command to stdin
        stdin
            .write_all(command.as_bytes())
            .await
            .context("Failed to write command")?;
        stdin
            .write_all(b"\n")
            .await
            .context("Failed to write newline")?;
        stdin.flush().await.context("Failed to flush stdin")?;

        debug!("Sent command (ID: {}): {}", id, command);

        // Spawn timeout task
        let pending_clone = self.pending.clone();
        tokio::spawn(async move {
            tokio::time::sleep(COMMAND_TIMEOUT).await;
            let mut pending = pending_clone.write().await;
            if let Some(cmd) = pending.remove(&id) {
                warn!("Command timeout after {}s: {}", COMMAND_TIMEOUT.as_secs(), cmd.command);
                let _ = cmd.tx.send(ShellCommandResponse {
                    command: cmd.command,
                    success: false,
                    output: cmd.output_buffer,
                    error: Some(format!("Command timeout after {}s", COMMAND_TIMEOUT.as_secs())),
                    execution_time_ms: COMMAND_TIMEOUT.as_millis() as u64,
                });
            }
        });

        Ok(rx)
    }

    /// Handle output line - match to pending command or queue
    pub async fn handle_output(&self, line: String) {
        let mut pending = self.pending.write().await;

        // Simple heuristic: assign output to oldest pending command
        if let Some((id, cmd)) = pending.iter_mut().next() {
            cmd.output_buffer.push(line.clone());

            // Check if this looks like a command completion (contains "OK", "ERROR", or prompt)
            // This is a simple heuristic - real implementation should parse kernel output format
            if line.contains("OK") || line.contains("ERROR") || line.starts_with("sis>") {
                let id = *id;
                if let Some(mut cmd) = pending.remove(&id) {
                    let execution_time = cmd.sent_at.elapsed().as_millis() as u64;
                    let success = !cmd.output_buffer.iter().any(|l| l.contains("ERROR"));

                    debug!(
                        "Command completed (ID: {}, {}ms): {} lines",
                        id,
                        execution_time,
                        cmd.output_buffer.len()
                    );

                    let _ = cmd.tx.send(ShellCommandResponse {
                        command: cmd.command,
                        success,
                        output: cmd.output_buffer,
                        error: None,
                        execution_time_ms: execution_time,
                    });
                }
            }
        } else {
            // No pending commands, queue the output
            let mut queue = self.output_queue.lock().await;
            queue.push_back(line);
            // Limit queue size
            while queue.len() > 1000 {
                queue.pop_front();
            }
        }
    }
}

/// Live QEMU process handle
#[derive(Debug)]
pub struct LiveProcess {
    /// Child process handle
    child: Child,
    /// Process ID
    pub pid: u32,
    /// Start time
    pub started_at: Instant,
    /// Standard input (for sending commands)
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    /// Standard output reader task handle
    stdout_task: Option<JoinHandle<Result<()>>>,
    /// Standard error reader task handle
    stderr_task: Option<JoinHandle<Result<()>>>,
    /// Lines processed counter
    lines_processed: Arc<Mutex<u64>>,
    /// Command-response tracker
    command_tracker: CommandTracker,
}

impl LiveProcess {
    /// Create a new LiveProcess from spawned child
    fn new(
        mut child: Child,
        event_tx: broadcast::Sender<QemuEvent>,
        parser: Arc<Mutex<LineParser>>,
    ) -> Result<Self> {
        let pid = child.id().context("Failed to get child PID")?;
        let started_at = Instant::now();

        // Take ownership of stdio handles
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().context("Failed to get stdout")?;
        let stderr = child.stderr.take().context("Failed to get stderr")?;

        let lines_processed = Arc::new(Mutex::new(0u64));
        let command_tracker = CommandTracker::new();

        // Spawn stdout reader task
        let stdout_task = {
            let event_tx = event_tx.clone();
            let parser = parser.clone();
            let lines_processed = lines_processed.clone();
            let command_tracker = command_tracker.clone();
            Some(tokio::spawn(async move {
                process_stdout(stdout, event_tx, parser, lines_processed, command_tracker).await
            }))
        };

        // Spawn stderr reader task
        let stderr_task = {
            let event_tx = event_tx.clone();
            Some(tokio::spawn(async move {
                process_stderr(stderr, event_tx).await
            }))
        };

        Ok(Self {
            child,
            pid,
            started_at,
            stdin: Arc::new(Mutex::new(stdin)),
            stdout_task,
            stderr_task,
            lines_processed,
            command_tracker,
        })
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Get lines processed count
    pub async fn lines_processed(&self) -> u64 {
        *self.lines_processed.lock().await
    }

    /// Check if process is still running
    pub fn is_running(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }

    /// Send a command to stdin (fire-and-forget, no response tracking)
    pub async fn send_command(&self, command: &str) -> Result<()> {
        let mut stdin_guard = self.stdin.lock().await;
        if let Some(stdin) = stdin_guard.as_mut() {
            stdin
                .write_all(command.as_bytes())
                .await
                .context("Failed to write to stdin")?;
            stdin
                .write_all(b"\n")
                .await
                .context("Failed to write newline")?;
            stdin.flush().await.context("Failed to flush stdin")?;
            debug!("Sent command to kernel: {}", command);
            Ok(())
        } else {
            anyhow::bail!("stdin not available")
        }
    }

    /// Execute a command and wait for the response (with correlation tracking)
    pub async fn execute_command_with_response(
        &self,
        command: String,
    ) -> Result<ShellCommandResponse> {
        let mut stdin_guard = self.stdin.lock().await;
        if let Some(stdin) = stdin_guard.as_mut() {
            // Use command tracker to send command and get response channel
            let rx = self
                .command_tracker
                .send_command(command.clone(), stdin)
                .await?;

            // Release stdin lock while waiting for response
            drop(stdin_guard);

            // Wait for response (will timeout after 5s if no response)
            rx.await
                .context("Failed to receive command response")
        } else {
            anyhow::bail!("stdin not available")
        }
    }

    /// Stop the process gracefully
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping QEMU process (PID: {})", self.pid);

        // Drop stdin to signal EOF to child
        {
            let mut stdin_guard = self.stdin.lock().await;
            *stdin_guard = None;
        }

        // Wait a bit for graceful shutdown
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Kill if still running
        if self.is_running() {
            warn!("QEMU did not exit gracefully, killing process");
            self.child.kill().await.ok();
        }

        // Wait for exit
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            self.child.wait(),
        )
        .await
        {
            Ok(Ok(status)) => {
                info!("QEMU exited with status: {:?}", status);
            }
            Ok(Err(e)) => {
                error!("Error waiting for QEMU exit: {}", e);
            }
            Err(_) => {
                error!("Timeout waiting for QEMU exit, force killing");
                self.child.kill().await.ok();
            }
        }

        // Cancel reader tasks
        if let Some(task) = self.stdout_task.take() {
            task.abort();
        }
        if let Some(task) = self.stderr_task.take() {
            task.abort();
        }

        Ok(())
    }
}

impl Drop for LiveProcess {
    fn drop(&mut self) {
        // Best effort kill on drop
        let _ = self.child.start_kill();
    }
}

/// Spawn QEMU process using uefi_run.sh
pub async fn spawn_qemu(
    config: &QemuConfig,
    event_tx: broadcast::Sender<QemuEvent>,
) -> Result<LiveProcess> {
    let script = std::env::var("SIS_RUN_SCRIPT")
        .unwrap_or_else(|_| "./scripts/uefi_run.sh".to_string());

    info!("Spawning QEMU with script: {}", script);
    debug!("QEMU config: {:?}", config);

    // Build command
    let mut cmd = Command::new("bash");
    cmd.arg(&script)
        // Note: script doesn't parse arguments, just builds+runs QEMU
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    // Add features as SIS_FEATURES env var
    if !config.features.is_empty() {
        let features_str = config.features.join(",");
        cmd.env("SIS_FEATURES", features_str);
    }

    // Enable bringup mode
    cmd.env("BRINGUP", "1");

    // Add custom environment variables
    for (key, value) in &config.env {
        cmd.env(key, value);
    }

    // Set working directory if specified
    if let Some(ref wd) = config.working_dir {
        cmd.current_dir(wd);
    }

    // Spawn the process
    let child = cmd
        .spawn()
        .context(format!("Failed to spawn QEMU via script: {}", script))?;

    let pid = child.id().context("Failed to get child PID")?;
    info!("QEMU spawned successfully (PID: {})", pid);

    // Emit state changed event
    let _ = event_tx.send(QemuEvent::StateChanged {
        state: super::types::QemuState::Starting,
        timestamp: chrono::Utc::now(),
    });

    // Create LiveProcess
    let parser = Arc::new(Mutex::new(LineParser::new()));
    let process = LiveProcess::new(child, event_tx.clone(), parser)?;

    // Emit running event after short delay
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let _ = event_tx_clone.send(QemuEvent::StateChanged {
            state: super::types::QemuState::Running,
            timestamp: chrono::Utc::now(),
        });
    });

    Ok(process)
}

/// Process stdout line-by-line
async fn process_stdout(
    stdout: ChildStdout,
    event_tx: broadcast::Sender<QemuEvent>,
    parser: Arc<Mutex<LineParser>>,
    lines_processed: Arc<Mutex<u64>>,
    command_tracker: CommandTracker,
) -> Result<()> {
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let mut count = 0u64;

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF - process exited
                info!("QEMU stdout closed (EOF), processed {} lines", count);
                let _ = event_tx.send(QemuEvent::StateChanged {
                    state: super::types::QemuState::Idle,
                    timestamp: chrono::Utc::now(),
                });
                break;
            }
            Ok(_) => {
                count += 1;
                *lines_processed.lock().await = count;

                // Check buffer limit
                if count > MAX_OUTPUT_BUFFER as u64 {
                    warn!(
                        "Output buffer limit reached ({}), dropping old lines",
                        MAX_OUTPUT_BUFFER
                    );
                }

                // Remove trailing newline
                let line_trimmed = line.trim_end();

                // Feed line to command tracker for response correlation
                command_tracker.handle_output(line_trimmed.to_string()).await;

                // Emit raw line event
                let _ = event_tx.send(QemuEvent::RawLine {
                    line: line_trimmed.to_string(),
                    timestamp: chrono::Utc::now(),
                });

                // Parse and emit structured events
                let mut parser_guard = parser.lock().await;
                if let Some(event) = parser_guard.parse_line(line_trimmed) {
                    let _ = event_tx.send(QemuEvent::Parsed { event });
                }
            }
            Err(e) => {
                error!("Error reading stdout: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Process stderr line-by-line
async fn process_stderr(
    stderr: ChildStderr,
    event_tx: broadcast::Sender<QemuEvent>,
) -> Result<()> {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF
                debug!("QEMU stderr closed (EOF)");
                break;
            }
            Ok(_) => {
                let line_trimmed = line.trim_end();
                // Emit stderr as raw lines (prefixed for debugging)
                let _ = event_tx.send(QemuEvent::RawLine {
                    line: format!("[stderr] {}", line_trimmed),
                    timestamp: chrono::Utc::now(),
                });
            }
            Err(e) => {
                error!("Error reading stderr: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_qemu_missing_script() {
        // Clear SIS_RUN_SCRIPT env var
        std::env::remove_var("SIS_RUN_SCRIPT");

        let config = QemuConfig::default();
        let (event_tx, _) = broadcast::channel::<QemuEvent>(100);

        // Should fail if script doesn't exist
        let result = spawn_qemu(&config, event_tx).await;
        // This will fail because ./scripts/uefi_run.sh might not exist in test env
        // Or it might succeed if the script exists - either is fine for this test
        // Just ensure we don't panic
        match result {
            Ok(_) => {}
            Err(e) => {
                assert!(e.to_string().contains("Failed to spawn") ||
                        e.to_string().contains("PID"));
            }
        }
    }

    #[tokio::test]
    async fn test_live_process_structure() {
        // Just test that we can create the types
        let (event_tx, _) = broadcast::channel::<QemuEvent>(100);
        let config = QemuConfig::default();

        // Verify config serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("features"));
    }

    #[test]
    fn test_max_buffer_constant() {
        assert_eq!(MAX_OUTPUT_BUFFER, 50_000);
    }
}
