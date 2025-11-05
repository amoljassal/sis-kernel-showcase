//! Replay transport for offline testing
//!
//! Reads pre-recorded log files and emits events as if they came from QEMU.
//! Useful for testing, demos, and development without running actual QEMU.

use super::supervisor::QemuEvent;
use crate::parser::{LineParser, ParsedEvent};
use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, info};

/// Replay speed multiplier
#[derive(Debug, Clone, Copy)]
pub enum ReplaySpeed {
    /// Real-time (matches original timing if recorded)
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

/// Replay transport that reads from log files
pub struct ReplayTransport {
    event_tx: broadcast::Sender<QemuEvent>,
    speed: ReplaySpeed,
}

impl ReplayTransport {
    /// Create a new replay transport
    pub fn new(event_tx: broadcast::Sender<QemuEvent>, speed: ReplaySpeed) -> Self {
        Self { event_tx, speed }
    }

    /// Replay a log file, emitting events line by line
    pub async fn replay_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Starting replay from: {}", path.display());

        let content = fs::read_to_string(path)
            .await
            .context(format!("Failed to read replay file: {}", path.display()))?;

        let mut parser = LineParser::new();
        let mut line_count = 0;

        for line in content.lines() {
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
