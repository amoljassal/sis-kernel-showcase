//! QEMU supervisor implementation

use super::live::{spawn_qemu, LiveProcess};
use super::shell::{ShellCommandRequest, ShellCommandResponse};
use super::shell_executor::ShellExecutor;
use super::types::{QemuConfig, QemuMode, QemuState, QemuStatus};
use crate::metrics::MetricsStore;
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
const METRIC_BATCH_INTERVAL_MS: u64 = 100; // Batch metrics every 100ms
const MAX_METRICS_PER_BATCH: usize = 1000; // Max points per batch

/// Batched metric point for WebSocket streaming
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchedMetricPoint {
    pub name: String,
    pub ts: i64,
    pub value: i64,
}

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
    /// Batched metrics (emitted every 100ms)
    MetricBatch {
        points: Vec<BatchedMetricPoint>,
        #[serde(skip_serializing_if = "Option::is_none")]
        dropped_count: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        seq: Option<u64>,
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
    /// Self-check canceled
    SelfCheckCanceled {
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// QEMU process exited unexpectedly
    QemuExited {
        code: Option<i32>,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// M4: Graph state update
    GraphState {
        #[serde(rename = "graphId")]
        graph_id: String,
        state: GraphStateData,
        ts: i64,
    },
    /// M4: Scheduling event
    SchedEvent {
        event: String,
        payload: serde_json::Value,
        ts: i64,
    },
    /// M4: LLM token chunk
    LlmTokens {
        #[serde(rename = "requestId")]
        request_id: String,
        chunk: String,
        done: bool,
        ts: i64,
    },
    /// M4: Log line
    LogLine {
        level: String,
        source: String,
        msg: String,
        ts: i64,
        #[serde(skip_serializing_if = "Option::is_none", rename = "requestId")]
        request_id: Option<String>,
    },
    /// M5: Crash captured
    Crash {
        #[serde(rename = "crashId")]
        crash_id: String,
        #[serde(rename = "panicMsg")]
        panic_msg: String,
        #[serde(skip_serializing_if = "Option::is_none", rename = "stackTrace")]
        stack_trace: Option<Vec<String>>,
        severity: String,
        ts: i64,
    },
}

/// Graph state data for GraphState event
#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphStateData {
    pub operators: Vec<GraphOperator>,
    pub channels: Vec<GraphChannel>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphOperator {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prio: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<GraphOperatorStats>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphOperatorStats {
    #[serde(rename = "execCount")]
    pub exec_count: u64,
    #[serde(rename = "avgUs")]
    pub avg_us: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GraphChannel {
    pub id: String,
    pub cap: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
}

/// Shared QEMU supervisor state
#[derive(Debug)]
struct SupervisorState {
    state: QemuState,
    mode: Option<QemuMode>,
    config: Option<QemuConfig>,
    child: Option<Child>,
    live_process: Option<LiveProcess>,
    start_time: Option<Instant>,
    lines_processed: u64,
    events_emitted: u64,
    boot_status: BootStatus,
    last_error: Option<String>,
    run_id: Option<String>,
    transport: String,
    profile: String,
}

impl Default for SupervisorState {
    fn default() -> Self {
        Self {
            state: QemuState::Idle,
            mode: None,
            config: None,
            child: None,
            live_process: None,
            start_time: None,
            lines_processed: 0,
            events_emitted: 0,
            boot_status: BootStatus::new(),
            last_error: None,
            run_id: None,
            transport: "none".to_string(),
            profile: std::env::var("SIS_PROFILE").unwrap_or_else(|_| "default".to_string()),
        }
    }
}

/// Reason for busy state
#[derive(Debug, Clone)]
enum BusyReason {
    SelfCheck,
    Command(String),
}

/// QEMU supervisor manages QEMU process lifecycle
#[derive(Debug, Clone)]
pub struct QemuSupervisor {
    state: Arc<RwLock<SupervisorState>>,
    event_tx: broadcast::Sender<QemuEvent>,
    shell_executor: Arc<Mutex<Option<ShellExecutor>>>,
    busy: Arc<AtomicBool>,
    busy_reason: Arc<Mutex<Option<BusyReason>>>,
    cancel_self_check: Arc<AtomicBool>,
    metrics: Arc<MetricsStore>,
}

impl QemuSupervisor {
    /// Create a new QEMU supervisor
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(MAX_EVENT_SUBSCRIBERS);

        // Load metrics config from daemon config
        let daemon_config = crate::config::DaemonConfig::from_env();
        let metrics_config = crate::metrics::MetricsConfig {
            high_res_retention_ms: daemon_config.metrics_high_res_retention_ms,
            downsample_retention_ms: daemon_config.metrics_downsample_retention_ms,
            cardinality_limit: daemon_config.metrics_cardinality_limit,
            memory_budget_bytes: 64 * 1024 * 1024, // 64MB
        };

        Self {
            state: Arc::new(RwLock::new(SupervisorState::default())),
            event_tx,
            shell_executor: Arc::new(Mutex::new(None)),
            busy: Arc::new(AtomicBool::new(false)),
            busy_reason: Arc::new(Mutex::new(None)),
            cancel_self_check: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(MetricsStore::new(metrics_config)),
        }
    }

    /// Subscribe to QEMU events
    pub fn subscribe(&self) -> broadcast::Receiver<QemuEvent> {
        self.event_tx.subscribe()
    }

    /// Broadcast an event to all subscribers (non-blocking)
    pub fn broadcast_event(&self, event: QemuEvent) {
        // Send returns Err if no receivers, which is fine
        let _ = self.event_tx.send(event);
    }

    /// Get metrics store reference
    pub fn metrics(&self) -> Arc<MetricsStore> {
        Arc::clone(&self.metrics)
    }

    /// Get current status
    pub async fn status(&self) -> QemuStatus {
        let state = self.state.read().await;

        // Get PID from either live_process or child
        let pid = state
            .live_process
            .as_ref()
            .map(|p| p.pid())
            .or_else(|| state.child.as_ref().and_then(|c| c.id()));

        // Get live count from LiveProcess if available, otherwise use supervisor's count
        let lines_processed = if let Some(ref live) = state.live_process {
            live.lines_processed().await
        } else {
            state.lines_processed
        };

        QemuStatus {
            state: state.state,
            mode: state.mode.clone(),
            pid,
            uptime_secs: state.start_time.map(|t| t.elapsed().as_secs()),
            features: state
                .config
                .as_ref()
                .map(|c| c.features.clone())
                .unwrap_or_default(),
            error: state.last_error.clone(),
            lines_processed,
            events_emitted: state.events_emitted,
        }
    }

    /// Start QEMU with given configuration
    #[tracing::instrument(
        skip(self, config),
        fields(
            run_id = tracing::field::Empty,
            transport = "qemu",
            qemu_pid = tracing::field::Empty,
            features = ?config.features,
            profile = tracing::field::Empty,
        )
    )]
    pub async fn run(&self, config: QemuConfig) -> Result<()> {
        let mut state = self.state.write().await;

        // Check if already running
        if state.state != QemuState::Idle {
            anyhow::bail!("QEMU already running or in transition");
        }

        // Generate run_id and set transport
        let run_id = uuid::Uuid::new_v4().to_string();
        state.run_id = Some(run_id.clone());
        state.transport = "qemu".to_string();

        // Record run_id and profile in span
        tracing::Span::current().record("run_id", &run_id);
        tracing::Span::current().record("profile", &state.profile);

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

        // Spawn metrics batch emitter
        self.spawn_metrics_emitter();

        Ok(())
    }

    /// Start QEMU in live mode (using LiveProcess)
    #[tracing::instrument(
        skip(self, config),
        fields(
            run_id = tracing::field::Empty,
            transport = "qemu-live",
            qemu_pid = tracing::field::Empty,
            features = ?config.features,
            profile = tracing::field::Empty,
        )
    )]
    pub async fn run_live(&self, config: QemuConfig) -> Result<()> {
        let mut state = self.state.write().await;

        // Check if already running
        if state.state != QemuState::Idle {
            anyhow::bail!("QEMU already running or in transition");
        }

        // Generate run_id and set transport
        let run_id = uuid::Uuid::new_v4().to_string();
        state.run_id = Some(run_id.clone());
        state.transport = "qemu-live".to_string();

        // Record run_id and profile in span
        tracing::Span::current().record("run_id", &run_id);
        tracing::Span::current().record("profile", &state.profile);

        info!("Starting QEMU in live mode with features: {:?}", config.features);

        // Store config
        state.config = Some(config.clone());
        state.start_time = Some(Instant::now());
        state.state = QemuState::Starting;

        // Drop the write lock before spawning
        drop(state);

        // Spawn QEMU process
        let live_process = spawn_qemu(&config, self.event_tx.clone()).await?;
        let pid = live_process.pid();

        info!("QEMU live process spawned with PID: {}", pid);
        tracing::Span::current().record("qemu_pid", pid);

        // Update state with live process
        let mut state = self.state.write().await;
        state.live_process = Some(live_process);
        state.mode = Some(QemuMode::Live { pid: Some(pid) });
        state.state = QemuState::Running;

        // Emit state changed event (already emitted by spawn_qemu, but redundant is fine)
        let _ = self.event_tx.send(QemuEvent::StateChanged {
            state: QemuState::Running,
            timestamp: chrono::Utc::now(),
        });

        info!("QEMU live mode started successfully");

        // Spawn metrics batch emitter
        self.spawn_metrics_emitter();

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

        // Stop live process if present
        if let Some(mut live_process) = state.live_process.take() {
            debug!("Stopping live QEMU process {}", live_process.pid());
            drop(state); // Release lock while stopping
            let _ = live_process.stop().await;
            state = self.state.write().await; // Reacquire lock
        }

        // Kill child process if present (legacy mode)
        if let Some(mut child) = state.child.take() {
            if let Some(pid) = child.id() {
                debug!("Killing QEMU process {}", pid);
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }

        state.state = QemuState::Idle;
        state.mode = None;
        state.config = None;
        state.start_time = None;
        state.run_id = None;
        state.transport = "none".to_string();

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
        let metrics = Arc::clone(&self.metrics);

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

                    // Record metric if it's a metric event
                    if let ParsedEvent::Metric { name, value, timestamp } = &event {
                        let ts_ms = timestamp.timestamp_millis();
                        if let Err(e) = metrics.record(name.clone(), *value, ts_ms).await {
                            // Log but don't fail - cardinality limit may be exceeded
                            debug!("Failed to record metric {}: {}", name, e);
                        }
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
        let shell_executor = Arc::clone(&self.shell_executor);
        let busy = Arc::clone(&self.busy);
        let busy_reason = Arc::clone(&self.busy_reason);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let mut s = state.write().await;

                // Check if process has exited
                if let Some(child) = s.child.as_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let exit_code = status.code();
                            error!("QEMU process exited unexpectedly: code={:?}", exit_code);

                            // Emit QemuExited event
                            let _ = event_tx.send(QemuEvent::QemuExited {
                                code: exit_code,
                                timestamp: chrono::Utc::now(),
                            });

                            // Update state
                            s.state = QemuState::Failed;
                            s.last_error = Some(format!(
                                "QEMU exited unexpectedly (code: {})",
                                exit_code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
                            ));
                            s.child = None;
                            s.run_id = None;
                            s.transport = "none".to_string();

                            // Clear shell executor
                            *shell_executor.lock().await = None;

                            // Clear busy state
                            busy.store(false, std::sync::atomic::Ordering::SeqCst);
                            *busy_reason.lock().await = None;

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

    /// Spawn metrics batch emitter (every 100ms)
    fn spawn_metrics_emitter(&self) {
        let state = Arc::clone(&self.state);
        let event_tx = self.event_tx.clone();
        let metrics = Arc::clone(&self.metrics);

        tokio::spawn(async move {
            let mut seq = 0_u64;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(METRIC_BATCH_INTERVAL_MS));

            loop {
                interval.tick().await;

                // Check if QEMU is still running
                {
                    let s = state.read().await;
                    if s.state != QemuState::Running {
                        // Stop emitting when not running
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }

                // Get all series
                let series_list = metrics.list_series().await;
                if series_list.is_empty() {
                    continue;
                }

                // Collect recent points from all series
                let mut batch: Vec<BatchedMetricPoint> = Vec::new();
                let now_ms = chrono::Utc::now().timestamp_millis();
                let window_start = now_ms - (METRIC_BATCH_INTERVAL_MS * 2) as i64; // 2x interval window

                for series_meta in series_list {
                    // Query last few points from each series
                    if let Ok(result) = metrics.query(&series_meta.name, window_start, now_ms, 100).await {
                        for point in result.points {
                            // Only include recent points
                            if point.ts >= window_start {
                                batch.push(BatchedMetricPoint {
                                    name: series_meta.name.clone(),
                                    ts: point.ts,
                                    value: point.value,
                                });
                            }
                        }
                    }
                }

                if batch.is_empty() {
                    continue;
                }

                // Apply backpressure: limit to MAX_METRICS_PER_BATCH
                let (points, dropped_count) = if batch.len() > MAX_METRICS_PER_BATCH {
                    let dropped = batch.len() - MAX_METRICS_PER_BATCH;
                    // Drop oldest points (beginning of vec)
                    batch.drain(0..dropped);
                    (batch, Some(dropped))
                } else {
                    (batch, None)
                };

                seq += 1;

                // Emit batch
                let _ = event_tx.send(QemuEvent::MetricBatch {
                    points,
                    dropped_count,
                    seq: Some(seq),
                });
            }
        });
    }

    /// Check if system is busy and return reason
    pub async fn check_busy(&self) -> Option<String> {
        if self.busy.load(Ordering::SeqCst) {
            let reason = self.busy_reason.lock().await;
            Some(match reason.as_ref() {
                Some(BusyReason::SelfCheck) => "self-check is currently running".to_string(),
                Some(BusyReason::Command(cmd)) => format!("command '{}' is currently executing", cmd),
                None => "another operation is in progress".to_string(),
            })
        } else {
            None
        }
    }

    /// Execute a shell command
    #[tracing::instrument(
        skip(self),
        fields(
            command = %request.command,
            timeout_ms = request.timeout_ms,
            run_id = tracing::field::Empty,
            transport = tracing::field::Empty,
            profile = tracing::field::Empty,
        )
    )]
    pub async fn execute_command(&self, request: ShellCommandRequest) -> Result<ShellCommandResponse> {
        // Record context from state
        {
            let state = self.state.read().await;
            if let Some(ref run_id) = state.run_id {
                tracing::Span::current().record("run_id", run_id);
            }
            tracing::Span::current().record("transport", &state.transport);
            tracing::Span::current().record("profile", &state.profile);
        }

        // Check if busy (e.g., running self-check)
        if let Some(reason) = self.check_busy().await {
            anyhow::bail!("System busy: {}", reason);
        }

        // Route to appropriate executor based on mode
        let state = self.state.read().await;
        match &state.mode {
            Some(QemuMode::Live { .. }) => {
                // Use LiveProcess for direct command execution
                if let Some(ref live_process) = state.live_process {
                    live_process.execute_command_with_response(request.command).await
                } else {
                    Err(anyhow::anyhow!("Live mode enabled but process not available"))
                }
            }
            Some(QemuMode::Replay { .. }) | None => {
                // Use ShellExecutor for replay mode
                drop(state); // Release read lock
                let executor = self.shell_executor.lock().await;
                match executor.as_ref() {
                    Some(exec) => exec.execute(request).await,
                    None => Err(anyhow::anyhow!("Shell not ready or QEMU not running")),
                }
            }
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

    /// Cancel running self-check
    pub async fn cancel_self_check(&self) -> Result<()> {
        // Check if self-check is running
        if !self.busy.load(Ordering::SeqCst) {
            anyhow::bail!("No self-check is currently running");
        }

        let reason = self.busy_reason.lock().await;
        if !matches!(reason.as_ref(), Some(BusyReason::SelfCheck)) {
            anyhow::bail!("No self-check is currently running");
        }
        drop(reason);

        // Set cancel flag
        self.cancel_self_check.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Run self-check tests
    #[tracing::instrument(
        skip(self),
        fields(
            run_id = tracing::field::Empty,
            transport = tracing::field::Empty,
            profile = tracing::field::Empty,
        )
    )]
    pub async fn run_self_check(&self) -> Result<ShellCommandResponse> {
        // Record context from state
        {
            let state = self.state.read().await;
            if let Some(ref run_id) = state.run_id {
                tracing::Span::current().record("run_id", run_id);
            }
            tracing::Span::current().record("transport", &state.transport);
            tracing::Span::current().record("profile", &state.profile);
        }

        // Set busy flag
        if self.busy.swap(true, Ordering::SeqCst) {
            if let Some(reason) = self.check_busy().await {
                anyhow::bail!("System busy: {}", reason);
            }
            anyhow::bail!("System busy: another operation is in progress");
        }

        // Clear cancel flag and set busy reason
        self.cancel_self_check.store(false, Ordering::SeqCst);
        *self.busy_reason.lock().await = Some(BusyReason::SelfCheck);

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

        // Check if canceled
        let was_canceled = self.cancel_self_check.load(Ordering::SeqCst);

        // Parse and emit test results (only if not canceled)
        if !was_canceled {
            if let Ok(ref response) = result {
                let mut total = 0;
                let mut passed_count = 0;

                for line in &response.output {
                    // Check cancellation during result processing
                    if self.cancel_self_check.load(Ordering::SeqCst) {
                        break;
                    }

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

                // Emit completed event only if not canceled mid-processing
                if !self.cancel_self_check.load(Ordering::SeqCst) {
                    let _ = self.event_tx.send(QemuEvent::SelfCheckCompleted {
                        total,
                        passed: passed_count,
                        failed: total - passed_count,
                        success: total > 0 && passed_count == total,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }

        // Emit canceled event if flagged
        if was_canceled || self.cancel_self_check.load(Ordering::SeqCst) {
            let _ = self.event_tx.send(QemuEvent::SelfCheckCanceled {
                timestamp: chrono::Utc::now(),
            });
        }

        // Clear busy flag, reason, and cancel flag
        self.busy.store(false, Ordering::SeqCst);
        *self.busy_reason.lock().await = None;
        self.cancel_self_check.store(false, Ordering::SeqCst);

        // Return error if was canceled
        if was_canceled {
            anyhow::bail!("Self-check was canceled");
        }

        result
    }
}

impl Default for QemuSupervisor {
    fn default() -> Self {
        Self::new()
    }
}
