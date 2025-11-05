//! QEMU supervisor implementation

use super::shell::{ShellCommandRequest, ShellCommandResponse};
use super::shell_executor::ShellExecutor;
use super::types::{QemuConfig, QemuState, QemuStatus};
use crate::parser::{BootStatus, LineParser, ParsedEvent};
use anyhow::{Context, Result};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_EVENT_SUBSCRIBERS: usize = 100;
const MAX_PARSED_EVENT_BUFFER: usize = 1000;
const MAX_OUTPUT_LINES: u64 = 50_000; // Backpressure: cap at 50k lines

/// Event broadcast to subscribers
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QemuEvent {
    /// State changed
    StateChanged {
        state: QemuState,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Parsed event from output
    Parsed {
        event: ParsedEvent,
    },
    /// Raw line (for debugging/logging)
    RawLine {
        line: String,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Self-check started
    SelfCheckStarted {
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Self-check test result
    SelfCheckTest {
        name: String,
        passed: bool,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Self-check completed
    SelfCheckCompleted {
        total: usize,
        passed: usize,
        failed: usize,
        success: bool,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Shared QEMU supervisor state
#[derive(Debug)]
struct SupervisorState {
    state: QemuState,
    config: Option<QemuConfig>,
    child: Option<Child>,
    start_time: Option<Instant>,
    lines_processed: u64,
    events_emitted: u64,
    boot_status: BootStatus,
    last_error: Option<String>,
}

impl Default for SupervisorState {
    fn default() -> Self {
        Self {
            state: QemuState::Idle,
            config: None,
            child: None,
            start_time: None,
            lines_processed: 0,
            events_emitted: 0,
            boot_status: BootStatus::new(),
            last_error: None,
        }
    }
}

/// QEMU supervisor manages QEMU process lifecycle
#[derive(Debug, Clone)]
pub struct QemuSupervisor {
    state: Arc<RwLock<SupervisorState>>,
    event_tx: broadcast::Sender<QemuEvent>,
    shell_executor: Arc<Mutex<Option<ShellExecutor>>>,
    busy: Arc<AtomicBool>,
}

impl QemuSupervisor {
    /// Create a new QEMU supervisor
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(MAX_EVENT_SUBSCRIBERS);
        Self {
            state: Arc::new(RwLock::new(SupervisorState::default())),
            event_tx,
            shell_executor: Arc::new(Mutex::new(None)),
            busy: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Subscribe to QEMU events
    pub fn subscribe(&self) -> broadcast::Receiver<QemuEvent> {
        self.event_tx.subscribe()
    }

    /// Get current status
    pub async fn status(&self) -> QemuStatus {
        let state = self.state.read().await;
        QemuStatus {
            state: state.state,
            pid: state.child.as_ref().and_then(|c| c.id()),
            uptime_secs: state.start_time.map(|t| t.elapsed().as_secs()),
            features: state
                .config
                .as_ref()
                .map(|c| c.features.clone())
                .unwrap_or_default(),
            error: state.last_error.clone(),
            lines_processed: state.lines_processed,
            events_emitted: state.events_emitted,
        }
    }

    /// Start QEMU with given configuration
    #[tracing::instrument(skip(self, config), fields(features = ?config.features, qemu_pid = tracing::field::Empty))]
    pub async fn run(&self, config: QemuConfig) -> Result<()> {
        let mut state = self.state.write().await;

        // Check if already running
        if state.state != QemuState::Idle {
            anyhow::bail!("QEMU already running or in transition");
        }

        info!("Starting QEMU with features: {:?}", config.features);

        // Build environment variables
        let mut env_vars = config.env.clone();

        // Set SIS_FEATURES if features are specified (or from environment)
        if !config.features.is_empty() {
            env_vars.insert("SIS_FEATURES".to_string(), config.features.join(","));
        } else if let Ok(env_features) = std::env::var("SIS_FEATURES") {
            env_vars.insert("SIS_FEATURES".to_string(), env_features);
        }

        // Default to bringup mode
        env_vars.entry("BRINGUP".to_string()).or_insert("1".to_string());

        // Find uefi_run.sh script (allow environment override)
        let script_path = std::env::var("SIS_RUN_SCRIPT").ok().or_else(|| {
            config
                .working_dir
                .as_ref()
                .map(|d| format!("{}/scripts/uefi_run.sh", d))
        }).unwrap_or_else(|| "./scripts/uefi_run.sh".to_string());

        // Launch QEMU via uefi_run.sh
        let mut cmd = Command::new("bash");
        cmd.arg(&script_path)
            .envs(env_vars)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        debug!("Executing: bash {}", script_path);

        let mut child = cmd
            .spawn()
            .context("Failed to spawn QEMU process")?;

        let pid = child.id();
        info!("QEMU started with PID: {:?}", pid);
        tracing::Span::current().record("qemu_pid", pid.map(|p| p.to_string()).unwrap_or_default());

        // Capture stdin, stdout and stderr
        let stdin = child.stdin.take().context("Failed to capture stdin")?;
        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        // Create channel for parsed events to shell executor
        let (parsed_event_tx, parsed_event_rx) = mpsc::unbounded_channel();

        // Update state
        state.state = QemuState::Starting;
        state.config = Some(config);
        state.child = Some(child);
        state.start_time = Some(Instant::now());
        state.lines_processed = 0;
        state.events_emitted = 0;
        state.boot_status = BootStatus::new();
        state.last_error = None;

        // Emit state change event
        let _ = self.event_tx.send(QemuEvent::StateChanged {
            state: QemuState::Starting,
            timestamp: chrono::Utc::now(),
        });

        // Drop write lock before spawning tasks
        drop(state);

        // Create shell executor
        let shell_exec = ShellExecutor::new(stdin, parsed_event_rx);
        *self.shell_executor.lock().await = Some(shell_exec);

        // Spawn output processing tasks (with parsed event sender)
        self.spawn_output_processor(stdout, false, Some(parsed_event_tx.clone()));
        self.spawn_output_processor(stderr, true, Some(parsed_event_tx));

        // Spawn process monitor
        self.spawn_process_monitor();

        Ok(())
    }

    /// Stop QEMU
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if state.state == QemuState::Idle {
            warn!("QEMU not running");
            return Ok(());
        }

        info!("Stopping QEMU");
        state.state = QemuState::Stopping;

        // Emit state change
        let _ = self.event_tx.send(QemuEvent::StateChanged {
            state: QemuState::Stopping,
            timestamp: chrono::Utc::now(),
        });

        // Kill child process
        if let Some(mut child) = state.child.take() {
            if let Some(pid) = child.id() {
                debug!("Killing QEMU process {}", pid);
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }

        state.state = QemuState::Idle;
        state.config = None;
        state.start_time = None;

        // Emit final state change
        let _ = self.event_tx.send(QemuEvent::StateChanged {
            state: QemuState::Idle,
            timestamp: chrono::Utc::now(),
        });

        // Clear shell executor
        *self.shell_executor.lock().await = None;

        Ok(())
    }

    /// Spawn output processor for stdout or stderr
    fn spawn_output_processor(
        &self,
        output: impl tokio::io::AsyncRead + Unpin + Send + 'static,
        is_stderr: bool,
        parsed_event_tx: Option<mpsc::UnboundedSender<ParsedEvent>>,
    ) {
        let state = Arc::clone(&self.state);
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(output);
            let mut lines = reader.lines();
            let mut parser = LineParser::new();

            while let Ok(Some(line)) = lines.next_line().await {
                // Check backpressure limit
                let lines_processed = {
                    let mut s = state.write().await;
                    s.lines_processed += 1;

                    // Transition to Running on first output
                    if s.state == QemuState::Starting {
                        s.state = QemuState::Running;
                        let _ = event_tx.send(QemuEvent::StateChanged {
                            state: QemuState::Running,
                            timestamp: chrono::Utc::now(),
                        });
                    }

                    s.lines_processed
                };

                // Apply backpressure: stop processing if limit exceeded
                if lines_processed > MAX_OUTPUT_LINES {
                    warn!(
                        "Output line limit reached ({} lines), dropping further output",
                        MAX_OUTPUT_LINES
                    );
                    break;
                }

                // Emit raw line event (for terminal display)
                let _ = event_tx.send(QemuEvent::RawLine {
                    line: line.clone(),
                    timestamp: chrono::Utc::now(),
                });

                // Parse line
                if let Some(event) = parser.parse_line(&line) {
                    // Update boot status if it's a marker
                    if let ParsedEvent::Marker { marker, .. } = &event {
                        let mut s = state.write().await;
                        s.boot_status.mark_seen(*marker);
                    }

                    // Emit parsed event
                    {
                        let mut s = state.write().await;
                        s.events_emitted += 1;
                    }

                    // Send to shell executor if available
                    if let Some(ref tx) = parsed_event_tx {
                        let _ = tx.send(event.clone());
                    }

                    let _ = event_tx.send(QemuEvent::Parsed { event });
                }
            }

            if is_stderr {
                debug!("QEMU stderr stream ended");
            } else {
                debug!("QEMU stdout stream ended");
            }
        });
    }

    /// Spawn process monitor to detect crashes
    fn spawn_process_monitor(&self) {
        let state = Arc::clone(&self.state);
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let mut s = state.write().await;

                // Check if process has exited
                if let Some(child) = s.child.as_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            error!("QEMU process exited: {:?}", status);
                            s.state = QemuState::Failed;
                            s.last_error = Some(format!("Process exited: {:?}", status));
                            s.child = None;

                            let _ = event_tx.send(QemuEvent::StateChanged {
                                state: QemuState::Failed,
                                timestamp: chrono::Utc::now(),
                            });
                            break;
                        }
                        Ok(None) => {
                            // Still running
                        }
                        Err(e) => {
                            error!("Failed to check process status: {}", e);
                            s.state = QemuState::Failed;
                            s.last_error = Some(format!("Process check failed: {}", e));
                            s.child = None;

                            let _ = event_tx.send(QemuEvent::StateChanged {
                                state: QemuState::Failed,
                                timestamp: chrono::Utc::now(),
                            });
                            break;
                        }
                    }
                } else {
                    // No child process, exit monitor
                    break;
                }
            }
        });
    }

    /// Execute a shell command
    #[tracing::instrument(skip(self), fields(command = %request.command, timeout_ms = request.timeout_ms))]
    pub async fn execute_command(&self, request: ShellCommandRequest) -> Result<ShellCommandResponse> {
        // Check if busy (e.g., running self-check)
        if self.busy.load(Ordering::SeqCst) {
            anyhow::bail!("System busy: another operation is in progress");
        }

        let executor = self.shell_executor.lock().await;
        match executor.as_ref() {
            Some(exec) => exec.execute(request).await,
            None => Err(anyhow::anyhow!("Shell not ready or QEMU not running")),
        }
    }

    /// Check if shell is ready for commands
    pub async fn is_shell_ready(&self) -> bool {
        let executor = self.shell_executor.lock().await;
        executor.as_ref().map(|e| e.is_available()).unwrap_or(false)
    }

    /// Get event broadcaster for replay transport
    pub fn event_broadcaster(&self) -> broadcast::Sender<QemuEvent> {
        self.event_tx.clone()
    }

    /// Run self-check tests
    #[tracing::instrument(skip(self))]
    pub async fn run_self_check(&self) -> Result<ShellCommandResponse> {
        // Set busy flag
        if self.busy.swap(true, Ordering::SeqCst) {
            anyhow::bail!("System busy: another operation is in progress");
        }

        // Emit started event
        let _ = self.event_tx.send(QemuEvent::SelfCheckStarted {
            timestamp: chrono::Utc::now(),
        });

        // Execute self-check command
        let request = ShellCommandRequest {
            command: "self_check".to_string(),
            timeout_ms: 60000, // 60 seconds for self-check
        };

        let result = {
            let executor = self.shell_executor.lock().await;
            match executor.as_ref() {
                Some(exec) => exec.execute(request).await,
                None => Err(anyhow::anyhow!("Shell not ready or QEMU not running")),
            }
        };

        // Parse and emit test results
        if let Ok(ref response) = result {
            let mut total = 0;
            let mut passed_count = 0;

            for line in &response.output {
                if line.contains("[PASS]") {
                    let test_name = line.replace("[PASS]", "").trim().to_string();
                    total += 1;
                    passed_count += 1;
                    let _ = self.event_tx.send(QemuEvent::SelfCheckTest {
                        name: test_name,
                        passed: true,
                        timestamp: chrono::Utc::now(),
                    });
                } else if line.contains("[FAIL]") {
                    let test_name = line.replace("[FAIL]", "").trim().to_string();
                    total += 1;
                    let _ = self.event_tx.send(QemuEvent::SelfCheckTest {
                        name: test_name,
                        passed: false,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }

            // Emit completed event
            let _ = self.event_tx.send(QemuEvent::SelfCheckCompleted {
                total,
                passed: passed_count,
                failed: total - passed_count,
                success: total > 0 && passed_count == total,
                timestamp: chrono::Utc::now(),
            });
        }

        // Clear busy flag
        self.busy.store(false, Ordering::SeqCst);

        result
    }
}

impl Default for QemuSupervisor {
    fn default() -> Self {
        Self::new()
    }
}
