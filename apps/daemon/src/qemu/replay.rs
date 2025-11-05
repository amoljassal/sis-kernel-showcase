//! Replay transport for offline testing
//!
//! Reads pre-recorded log files and emits events as if they came from QEMU.
//! Useful for testing, demos, and development without running actual QEMU.

use super::supervisor::QemuEvent;
use crate::parser::{LineParser, ParsedEvent};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::{broadcast, RwLock};
use tokio::time::sleep;
use tracing::{debug, info};

/// Replay state
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReplayState {
    Idle,
    Running,
}

/// Replay status
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ReplayStatus {
    pub state: ReplayState,
    pub source: Option<String>,
    pub mode: Option<String>,
    pub progress: u8, // 0-100
}

/// Replay speed multiplier
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplaySpeed {
    /// Real-time (matches original timing if recorded)
    #[serde(rename = "realtime")]
    RealTime,
    /// Fast (10ms between lines)
    Fast,
    /// Instant (no delay)
    Instant,
}

impl ReplaySpeed {
    fn delay(&self) -> Option<Duration> {
        match self {
            Self::RealTime => Some(Duration::from_millis(100)), // Simulate realistic output
            Self::Fast => Some(Duration::from_millis(10)),
            Self::Instant => None,
        }
    }
}

/// Global replay manager
#[derive(Debug, Clone)]
pub struct ReplayManager {
    status: Arc<RwLock<ReplayStatus>>,
    cancel: Arc<AtomicBool>,
}

impl ReplayManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(ReplayStatus {
                state: ReplayState::Idle,
                source: None,
                mode: None,
                progress: 0,
            })),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn get_status(&self) -> ReplayStatus {
        self.status.read().await.clone()
    }

    pub async fn start(&self, source: String, mode: String) {
        let mut status = self.status.write().await;
        status.state = ReplayState::Running;
        status.source = Some(source);
        status.mode = Some(mode);
        status.progress = 0;
        self.cancel.store(false, Ordering::SeqCst);
    }

    pub async fn stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
        let mut status = self.status.write().await;
        status.state = ReplayState::Idle;
        status.source = None;
        status.mode = None;
        status.progress = 0;
    }

    pub fn is_canceled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    pub async fn update_progress(&self, progress: u8) {
        self.status.write().await.progress = progress.min(100);
    }

    pub async fn complete(&self) {
        let mut status = self.status.write().await;
        status.state = ReplayState::Idle;
        status.progress = 100;
    }
}

impl Default for ReplayManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Replay transport that reads from log files
pub struct ReplayTransport {
    event_tx: broadcast::Sender<QemuEvent>,
    speed: ReplaySpeed,
    manager: Option<Arc<ReplayManager>>,
}

impl ReplayTransport {
    /// Create a new replay transport
    pub fn new(event_tx: broadcast::Sender<QemuEvent>, speed: ReplaySpeed) -> Self {
        Self {
            event_tx,
            speed,
            manager: None,
        }
    }

    /// Create a new replay transport with manager for cancellation and progress
    pub fn with_manager(
        event_tx: broadcast::Sender<QemuEvent>,
        speed: ReplaySpeed,
        manager: Arc<ReplayManager>,
    ) -> Self {
        Self {
            event_tx,
            speed,
            manager: Some(manager),
        }
    }

    /// Replay a log file, emitting events line by line
    pub async fn replay_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Starting replay from: {}", path.display());

        let content = fs::read_to_string(path)
            .await
            .context(format!("Failed to read replay file: {}", path.display()))?;

        let mut parser = LineParser::new();
        let total_lines = content.lines().count();
        let mut line_count = 0;

        for line in content.lines() {
            // Check for cancellation
            if let Some(ref manager) = self.manager {
                if manager.is_canceled() {
                    info!("Replay canceled at line {}/{}", line_count, total_lines);
                    return Ok(());
                }
            }

            line_count += 1;

            // Emit raw line event
            let _ = self.event_tx.send(QemuEvent::RawLine {
                line: line.to_string(),
                timestamp: chrono::Utc::now(),
            });

            // Parse and emit parsed event
            if let Some(event) = parser.parse_line(line) {
                let _ = self.event_tx.send(QemuEvent::Parsed { event });
            }

            // Update progress every 10 lines (0-100%)
            if let Some(ref manager) = self.manager {
                if line_count % 10 == 0 || line_count == total_lines {
                    let progress = ((line_count as f64 / total_lines as f64) * 100.0) as u8;
                    manager.update_progress(progress).await;
                }
            }

            // Delay between lines
            if let Some(delay) = self.speed.delay() {
                sleep(delay).await;
            }
        }

        debug!("Replay complete: {} lines processed", line_count);
        Ok(())
    }

    /// Replay multiple files in sequence
    pub async fn replay_sequence(&self, paths: &[impl AsRef<Path>]) -> Result<()> {
        for path in paths {
            self.replay_file(path).await?;

            // Small delay between files
            if let Some(delay) = self.speed.delay() {
                sleep(delay * 5).await;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_replay_speed() {
        assert!(matches!(ReplaySpeed::RealTime.delay(), Some(_)));
        assert!(matches!(ReplaySpeed::Fast.delay(), Some(_)));
        assert!(matches!(ReplaySpeed::Instant.delay(), None));
    }
}
