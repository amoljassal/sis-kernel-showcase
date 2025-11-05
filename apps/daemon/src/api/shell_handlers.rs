//! Shell command API handlers

use crate::qemu::{
    QemuSupervisor, SelfCheckResponse, ShellCommandRequest, ShellCommandResponse, TestResultEntry,
};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tracing::debug;
use utoipa::ToSchema;

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
        (status = 409, description = "System busy", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse),
        (status = 500, description = "Execution failed", body = ErrorResponse)
    ),
    tag = "shell"
)]
pub async fn shell_exec(
    State(supervisor): State<Arc<QemuSupervisor>>,
    Json(request): Json<ShellCommandRequest>,
) -> Result<Json<ShellCommandResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Executing shell command: {}", request.command);

    // Execute command via supervisor
    match supervisor.execute_command(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let error_msg = e.to_string();
            let status_code = if error_msg.contains("busy") {
                StatusCode::CONFLICT
            } else if error_msg.contains("not ready") || error_msg.contains("not running") {
                StatusCode::SERVICE_UNAVAILABLE
            } else if error_msg.contains("timed out") {
                StatusCode::GATEWAY_TIMEOUT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status_code,
                Json(ErrorResponse { error: error_msg }),
            ))
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
    State(supervisor): State<Arc<QemuSupervisor>>,
) -> Result<Json<SelfCheckResponse>, (StatusCode, Json<ErrorResponse>)> {
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

            Ok(Json(SelfCheckResponse {
                tests,
                total: tests.len(),
                passed,
                failed,
                success: failed == 0,
                execution_time_ms,
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            let status_code = if error_msg.contains("busy") {
                StatusCode::CONFLICT
            } else if error_msg.contains("not ready") || error_msg.contains("not running") {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status_code,
                Json(ErrorResponse { error: error_msg }),
            ))
        }
    }
}
