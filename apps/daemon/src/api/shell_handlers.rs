//! Shell command API handlers

use crate::qemu::{
    SelfCheckResponse, ShellCommandRequest, ShellCommandResponse, TestResultEntry,
};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use tracing::{debug, warn};
use utoipa::ToSchema;

/// Shell executor shared state
pub struct ShellState {
    /// Whether shell is ready for commands
    shell_ready: AtomicBool,
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            shell_ready: AtomicBool::new(false),
        }
    }

    pub fn set_ready(&self, ready: bool) {
        self.shell_ready.store(ready, Ordering::Relaxed);
    }

    pub fn is_ready(&self) -> bool {
        self.shell_ready.load(Ordering::Relaxed)
    }
}

impl Default for ShellState {
    fn default() -> Self {
        Self::new()
    }
}

/// Error response
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// Execute a shell command
#[utoipa::path(
    post,
    path = "/api/v1/shell/exec",
    request_body = ShellCommandRequest,
    responses(
        (status = 200, description = "Command executed successfully", body = ShellCommandResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse),
        (status = 500, description = "Execution failed", body = ErrorResponse)
    ),
    tag = "shell"
)]
pub async fn shell_exec(
    State(shell_state): State<Arc<RwLock<ShellState>>>,
    Json(request): Json<ShellCommandRequest>,
) -> Result<Json<ShellCommandResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if shell is ready
    let is_ready = shell_state.read().await.is_ready();
    if !is_ready {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Shell not ready. Please wait for QEMU to boot.".to_string(),
            }),
        ));
    }

    debug!("Executing shell command: {}", request.command);

    // TODO: Implement actual command execution via stdin
    // For now, return a placeholder response
    warn!("Shell command execution not yet fully implemented");

    Ok(Json(ShellCommandResponse {
        command: request.command.clone(),
        output: vec![
            "Command execution coming soon!".to_string(),
            format!("Would execute: {}", request.command),
        ],
        success: true,
        error: None,
        execution_time_ms: 0,
    }))
}

/// Run self-check tests
#[utoipa::path(
    post,
    path = "/api/v1/shell/selfcheck",
    responses(
        (status = 200, description = "Self-check completed", body = SelfCheckResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse),
        (status = 500, description = "Self-check failed", body = ErrorResponse)
    ),
    tag = "shell"
)]
pub async fn shell_selfcheck(
    State(shell_state): State<Arc<RwLock<ShellState>>>,
) -> Result<Json<SelfCheckResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if shell is ready
    let is_ready = shell_state.read().await.is_ready();
    if !is_ready {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Shell not ready. Please wait for QEMU to boot.".to_string(),
            }),
        ));
    }

    debug!("Running self-check");

    // TODO: Implement actual self-check execution
    // For now, return a placeholder response
    warn!("Self-check execution not yet fully implemented");

    Ok(Json(SelfCheckResponse {
        tests: vec![
            TestResultEntry {
                name: "Example test".to_string(),
                passed: true,
                timestamp: chrono::Utc::now(),
            },
        ],
        total: 1,
        passed: 1,
        failed: 0,
        success: true,
        execution_time_ms: 0,
    }))
}
