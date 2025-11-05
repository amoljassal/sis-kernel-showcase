//! Daemon configuration

use serde::Serialize;
use utoipa::ToSchema;

/// Daemon configuration
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DaemonConfig {
    /// Prompt pattern regex for shell detection
    pub prompt_pattern: String,

    /// Maximum output bytes per command
    pub max_output_bytes: u64,

    /// Retry-After seconds for 409 Conflict responses
    pub retry_after_seconds: u64,

    /// High-resolution metrics retention in milliseconds
    pub metrics_high_res_retention_ms: u64,

    /// Downsampled metrics retention in milliseconds
    pub metrics_downsample_retention_ms: u64,

    /// Maximum number of unique metric series
    pub metrics_cardinality_limit: usize,

    /// Path to QEMU run script
    pub run_script: String,

    /// Default features to enable
    pub default_features: Vec<String>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            prompt_pattern: r"(?m)^\s*sis>\s*$".to_string(),
            max_output_bytes: 1_000_000, // 1 MB
            retry_after_seconds: 5,
            metrics_high_res_retention_ms: 5 * 60 * 1000, // 5 minutes
            metrics_downsample_retention_ms: 60 * 60 * 1000, // 1 hour
            metrics_cardinality_limit: 256,
            run_script: std::env::var("SIS_RUN_SCRIPT")
                .unwrap_or_else(|_| "./scripts/uefi_run.sh".to_string()),
            default_features: std::env::var("SIS_FEATURES")
                .ok()
                .map(|s| s.split(',').map(|f| f.trim().to_string()).collect())
                .unwrap_or_default(),
        }
    }
}

impl DaemonConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override with environment variables if present
        if let Ok(pattern) = std::env::var("SIS_PROMPT_PATTERN") {
            config.prompt_pattern = pattern;
        }

        if let Ok(bytes) = std::env::var("SIS_MAX_OUTPUT_BYTES") {
            if let Ok(val) = bytes.parse() {
                config.max_output_bytes = val;
            }
        }

        if let Ok(seconds) = std::env::var("SIS_RETRY_AFTER_SECONDS") {
            if let Ok(val) = seconds.parse() {
                config.retry_after_seconds = val;
            }
        }

        if let Ok(ms) = std::env::var("METRICS_HIGH_RES_RETENTION_MS") {
            if let Ok(val) = ms.parse() {
                config.metrics_high_res_retention_ms = val;
            }
        }

        if let Ok(ms) = std::env::var("METRICS_DOWNSAMPLE_RETENTION_MS") {
            if let Ok(val) = ms.parse() {
                config.metrics_downsample_retention_ms = val;
            }
        }

        if let Ok(limit) = std::env::var("METRICS_MAX_POINTS") {
            if let Ok(val) = limit.parse() {
                config.metrics_cardinality_limit = val;
            }
        }

        config
    }
}
