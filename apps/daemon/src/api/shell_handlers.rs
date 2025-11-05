//! Shell command API handlers

use crate::qemu::{
    QemuSupervisor, SelfCheckResponse, ShellCommandRequest, ShellCommandResponse, TestResultEntry,
};
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tracing::debug;
use utoipa::ToSchema;

// Re-export problem+json ErrorResponse from handlers
pub use super::handlers::ErrorResponse;

/// Helper to create error response with proper headers
fn error_response(status: StatusCode, detail: String, error_type: Option<String>) -> Response {
    let mut headers = HeaderMap::new();

    // Add Retry-After header for 409 Conflict
    if status == StatusCode::CONFLICT {
        headers.insert(header::RETRY_AFTER, "5".parse().unwrap());
    }

    let error = ErrorResponse::with_type(status, detail, error_type);
    (status, headers, Json(error)).into_response()
}

/// Execute a shell command
#[utoipa::path(
    post,
    path = "/api/v1/shell/exec",
    request_body = ShellCommandRequest,
    responses(
        (status = 200, description = "Command executed successfully", body = ShellCommandResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 409, description = "System busy", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse),
        (status = 500, description = "Execution failed", body = ErrorResponse)
    ),
    tag = "shell"
)]
pub async fn shell_exec(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(request): Json<ShellCommandRequest>,
) -> Response {
    debug!("Executing shell command: {}", request.command);

    // Execute command via supervisor
    match supervisor.execute_command(request).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            let (status_code, error_type) = if error_msg.contains("busy") {
                (StatusCode::CONFLICT, Some("/errors/busy".to_string()))
            } else if error_msg.contains("not ready") || error_msg.contains("not running") {
                (StatusCode::SERVICE_UNAVAILABLE, Some("/errors/shell-not-ready".to_string()))
            } else if error_msg.contains("timed out") {
                (StatusCode::GATEWAY_TIMEOUT, Some("/errors/timeout".to_string()))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Some("/errors/internal".to_string()))
            };

            error_response(status_code, error_msg, error_type)
        }
    }
}

/// Run self-check tests
#[utoipa::path(
    post,
    path = "/api/v1/shell/selfcheck",
    responses(
        (status = 200, description = "Self-check completed", body = SelfCheckResponse),
        (status = 409, description = "System busy", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse),
        (status = 500, description = "Self-check failed", body = ErrorResponse)
    ),
    tag = "shell"
)]
pub async fn shell_selfcheck(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    debug!("Running self-check");

    let start = std::time::Instant::now();

    // Execute self-check command via supervisor (sets busy flag)
    match supervisor.run_self_check().await {
        Ok(response) => {
            // Parse test results from output
            // Looking for lines like "[PASS] Test name" or "[FAIL] Test name"
            let mut tests = Vec::new();
            for line in &response.output {
                if line.contains("[PASS]") {
                    let test_name = line.replace("[PASS]", "").trim().to_string();
                    tests.push(TestResultEntry {
                        name: test_name,
                        passed: true,
                        timestamp: chrono::Utc::now(),
                    });
                } else if line.contains("[FAIL]") {
                    let test_name = line.replace("[FAIL]", "").trim().to_string();
                    tests.push(TestResultEntry {
                        name: test_name,
                        passed: false,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }

            let passed = tests.iter().filter(|t| t.passed).count();
            let failed = tests.len() - passed;
            let execution_time_ms = start.elapsed().as_millis() as u64;

            Json(SelfCheckResponse {
                tests,
                total: tests.len(),
                passed,
                failed,
                success: failed == 0,
                execution_time_ms,
            }).into_response()
        }
        Err(e) => {
            let error_msg = e.to_string();
            let (status_code, error_type) = if error_msg.contains("busy") {
                (StatusCode::CONFLICT, Some("/errors/busy".to_string()))
            } else if error_msg.contains("not ready") || error_msg.contains("not running") {
                (StatusCode::SERVICE_UNAVAILABLE, Some("/errors/shell-not-ready".to_string()))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Some("/errors/internal".to_string()))
            };

            error_response(status_code, error_msg, error_type)
        }
    }
}

/// Cancel running self-check
#[utoipa::path(
    post,
    path = "/api/v1/shell/selfcheck/cancel",
    responses(
        (status = 200, description = "Self-check canceled", body = SuccessResponse),
        (status = 404, description = "No self-check running", body = ErrorResponse),
        (status = 500, description = "Cancel failed", body = ErrorResponse)
    ),
    tag = "shell"
)]
pub async fn shell_selfcheck_cancel(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    debug!("Canceling self-check");

    match supervisor.cancel_self_check().await {
        Ok(_) => Json(crate::api::handlers::SuccessResponse {
            message: "Self-check canceled".to_string(),
        }).into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            let (status_code, error_type) = if error_msg.contains("No self-check") {
                (StatusCode::NOT_FOUND, Some("/errors/not-found".to_string()))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Some("/errors/internal".to_string()))
            };

            error_response(status_code, error_msg, error_type)
        }
    }
}
