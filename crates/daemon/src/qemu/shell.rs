//! Shell command execution types

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Shell command request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShellCommandRequest {
    /// Command to execute
    pub command: String,

    /// Timeout in milliseconds (default: 30000)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    30000
}

/// Shell command response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShellCommandResponse {
    /// Command that was executed
    pub command: String,

    /// Output lines collected
    pub output: Vec<String>,

    /// Whether command completed successfully
    pub success: bool,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Self-check test result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TestResultEntry {
    /// Test name
    pub name: String,

    /// Pass or fail
    pub passed: bool,

    /// Timestamp
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Self-check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SelfCheckResponse {
    /// All test results
    pub tests: Vec<TestResultEntry>,

    /// Total number of tests
    pub total: usize,

    /// Number of passed tests
    pub passed: usize,

    /// Number of failed tests
    pub failed: usize,

    /// Overall success
    pub success: bool,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}
