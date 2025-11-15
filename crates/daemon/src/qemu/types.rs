//! Types for QEMU configuration and status

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// QEMU operating mode
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum QemuMode {
    /// Live mode - real QEMU process
    Live {
        #[serde(skip_serializing_if = "Option::is_none")]
        pid: Option<u32>,
    },
    /// Replay mode - reading from log files
    Replay {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<String>,
    },
}

/// Default features for QEMU (llm + crypto-real for full functionality)
fn default_features() -> Vec<String> {
    vec!["llm".to_string(), "crypto-real".to_string()]
}

/// QEMU run configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QemuConfig {
    /// Feature flags to enable (e.g., "llm", "graph-demo", "perf-verbose")
    /// Defaults to ["llm", "crypto-real"] for full functionality
    #[serde(default = "default_features")]
    pub features: Vec<String>,

    /// Environment variable overrides
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Additional command-line arguments for QEMU
    #[serde(default)]
    pub args: Vec<String>,

    /// Working directory (defaults to repo root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}

impl Default for QemuConfig {
    fn default() -> Self {
        Self {
            features: default_features(),
            env: HashMap::new(),
            args: vec![],
            working_dir: None,
        }
    }
}

/// QEMU process state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QemuState {
    /// Not running
    Idle,
    /// Starting up
    Starting,
    /// Running and operational
    Running,
    /// Stopping
    Stopping,
    /// Crashed or exited unexpectedly
    Failed,
}

/// QEMU status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QemuStatus {
    /// Current state
    pub state: QemuState,

    /// Operating mode (live or replay)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<QemuMode>,

    /// Process ID (if running)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,

    /// Uptime in seconds (if running)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_secs: Option<u64>,

    /// Active features
    #[serde(default)]
    pub features: Vec<String>,

    /// Last error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Number of lines processed
    pub lines_processed: u64,

    /// Number of events emitted
    pub events_emitted: u64,
}

impl Default for QemuStatus {
    fn default() -> Self {
        Self {
            state: QemuState::Idle,
            mode: None,
            pid: None,
            uptime_secs: None,
            features: vec![],
            error: None,
            lines_processed: 0,
            events_emitted: 0,
        }
    }
}
