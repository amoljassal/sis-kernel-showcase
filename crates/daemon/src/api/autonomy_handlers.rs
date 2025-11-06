//! Autonomy control API handlers
//!
//! These endpoints wrap `autoctl` shell commands for autonomy management

use crate::qemu::{QemuSupervisor, ShellCommandRequest};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;
use utoipa::ToSchema;

use super::handlers::{exec_and_parse, ErrorResponse};

/// Autonomy status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AutonomyStatus {
    pub enabled: bool,
    pub mode: String, // "active", "safe_mode", "learning_frozen"
    pub interval_ms: u64,
    pub conf_threshold: f64,
    pub total_decisions: u64,
    pub accepted: u64,
    pub deferred: u64,
    pub watchdog_resets: u64,
}

/// Autonomy decision entry for audit log
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AutonomyDecision {
    pub id: String,
    pub timestamp: i64,
    pub action: String,
    pub confidence: f64,
    pub reward: Option<f64>,
    pub executed: bool,
    pub reason: Option<String>,
}

/// Explain response with attention weights
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExplainResponse {
    pub id: String,
    pub action: String,
    pub confidence: f64,
    pub attention: Vec<AttentionWeight>,
    pub reasoning: String,
}

/// Attention weight for a feature
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttentionWeight {
    pub feature: String,
    pub weight: f64,
    pub value: String,
}

/// Preview request
#[derive(Debug, Deserialize, ToSchema)]
pub struct PreviewRequest {
    pub count: Option<u32>, // Default 10
}

/// Preview response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PreviewResponse {
    pub directives: Vec<String>,
    pub confidence: f64,
    pub would_execute: bool,
    pub warnings: Vec<String>,
}

/// What-if scenario request
#[derive(Debug, Deserialize, ToSchema)]
pub struct WhatIfRequest {
    pub overrides: serde_json::Value, // JSON object with scenario overrides
}

/// What-if response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WhatIfResponse {
    pub baseline: PreviewResponse,
    pub scenario: PreviewResponse,
    pub diff: Vec<String>,
}

/// Query parameters for audit log
#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub last: Option<u32>, // Default 100
}

/// Query parameters for explain
#[derive(Debug, Deserialize)]
pub struct ExplainQuery {
    pub id: String,
}

/// Turn autonomy on
#[utoipa::path(
    post,
    path = "/api/v1/autonomy/on",
    responses(
        (status = 200, description = "Autonomy enabled", body = AutonomyStatus),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_on(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    let (supervisor, _) = &state;
    debug!("Enabling autonomy");
    exec_and_parse::<AutonomyStatus>(supervisor, "autoctl on --json".to_string())
        .await
        .map(|status| Json(status).into_response())
        .unwrap_or_else(|r| r)
}

/// Turn autonomy off
#[utoipa::path(
    post,
    path = "/api/v1/autonomy/off",
    responses(
        (status = 200, description = "Autonomy disabled", body = AutonomyStatus),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_off(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Disabling autonomy");
    exec_and_parse::<AutonomyStatus>(&supervisor, "autoctl off --json".to_string())
        .await
        .map(|status| Json(status).into_response())
        .unwrap_or_else(|r| r)
}

/// Reset autonomy state
#[utoipa::path(
    post,
    path = "/api/v1/autonomy/reset",
    responses(
        (status = 200, description = "Autonomy reset", body = AutonomyStatus),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_reset(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Resetting autonomy");
    exec_and_parse::<AutonomyStatus>(&supervisor, "autoctl reset --json".to_string())
        .await
        .map(|status| Json(status).into_response())
        .unwrap_or_else(|r| r)
}

/// Set autonomy interval
#[utoipa::path(
    post,
    path = "/api/v1/autonomy/interval",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Interval updated", body = AutonomyStatus),
        (status = 400, description = "Invalid interval", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_set_interval(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let (supervisor, _) = &state;
    let interval_ms = match payload.get("interval_ms").and_then(|v| v.as_u64()) {
        Some(i) => i,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::with_type(
                    StatusCode::BAD_REQUEST,
                    "Missing or invalid interval_ms field".to_string(),
                    Some("/errors/invalid-interval".to_string()),
                )),
            )
                .into_response()
        }
    };

    debug!("Setting autonomy interval to {}ms", interval_ms);
    exec_and_parse::<AutonomyStatus>(
        &supervisor,
        format!("autoctl interval {} --json", interval_ms),
    )
    .await
    .map(|status| Json(status).into_response())
    .unwrap_or_else(|r| r)
}

/// Set confidence threshold
#[utoipa::path(
    post,
    path = "/api/v1/autonomy/conf-threshold",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Threshold updated", body = AutonomyStatus),
        (status = 400, description = "Invalid threshold", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_set_threshold(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let (supervisor, _) = &state;
    let threshold = match payload.get("threshold").and_then(|v| v.as_f64()) {
        Some(t) => t,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::with_type(
                    StatusCode::BAD_REQUEST,
                    "Missing or invalid threshold field".to_string(),
                    Some("/errors/invalid-threshold".to_string()),
                )),
            )
                .into_response()
        }
    };

    debug!("Setting confidence threshold to {}", threshold);
    exec_and_parse::<AutonomyStatus>(
        &supervisor,
        format!("autoctl conf-threshold {} --json", threshold),
    )
    .await
    .map(|status| Json(status).into_response())
    .unwrap_or_else(|r| r)
}

/// Get autonomy status
#[utoipa::path(
    get,
    path = "/api/v1/autonomy/status",
    responses(
        (status = 200, description = "Autonomy status", body = AutonomyStatus),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_status(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Getting autonomy status");
    exec_and_parse::<AutonomyStatus>(&supervisor, "autoctl status --json".to_string())
        .await
        .map(|status| Json(status).into_response())
        .unwrap_or_else(|r| r)
}

/// Get audit log
#[utoipa::path(
    get,
    path = "/api/v1/autonomy/audit",
    params(
        ("last" = Option<u32>, Query, description = "Number of recent decisions to return (default 100)")
    ),
    responses(
        (status = 200, description = "Audit log", body = Vec<AutonomyDecision>),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_audit(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<AuditQuery>,
) -> Response {
    let (supervisor, _) = &state;
    let last = params.last.unwrap_or(100);
    debug!("Getting last {} audit entries", last);

    exec_and_parse::<Vec<AutonomyDecision>>(
        &supervisor,
        format!("autoctl audit --last {} --json", last),
    )
    .await
    .map(|decisions| Json(decisions).into_response())
    .unwrap_or_else(|r| r)
}

/// Get explanation for a decision
#[utoipa::path(
    get,
    path = "/api/v1/autonomy/explain",
    params(
        ("id" = String, Query, description = "Decision ID to explain")
    ),
    responses(
        (status = 200, description = "Explanation", body = ExplainResponse),
        (status = 404, description = "Decision not found", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_explain(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<ExplainQuery>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Explaining decision {}", params.id);

    exec_and_parse::<ExplainResponse>(
        &supervisor,
        format!("autoctl explain --id {} --json", params.id),
    )
    .await
    .map(|explain| Json(explain).into_response())
    .unwrap_or_else(|r| r)
}

/// Preview next N decisions
#[utoipa::path(
    post,
    path = "/api/v1/autonomy/preview",
    request_body = PreviewRequest,
    responses(
        (status = 200, description = "Preview", body = PreviewResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_preview(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(request): Json<PreviewRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let count = request.count.unwrap_or(10);
    debug!("Previewing next {} decisions", count);

    exec_and_parse::<PreviewResponse>(
        &supervisor,
        format!("autoctl preview --count {} --json", count),
    )
    .await
    .map(|preview| Json(preview).into_response())
    .unwrap_or_else(|r| r)
}

/// Run what-if scenario
#[utoipa::path(
    post,
    path = "/api/v1/autonomy/whatif",
    request_body = WhatIfRequest,
    responses(
        (status = 200, description = "What-if comparison", body = WhatIfResponse),
        (status = 400, description = "Invalid scenario", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "autonomy"
)]
pub async fn autonomy_whatif(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(request): Json<WhatIfRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Running what-if scenario");

    // Serialize overrides as JSON string for command
    let overrides_json = match serde_json::to_string(&request.overrides) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::with_type(
                    StatusCode::BAD_REQUEST,
                    format!("Failed to serialize overrides: {}", e),
                    Some("/errors/invalid-scenario".to_string()),
                )),
            )
                .into_response()
        }
    };

    exec_and_parse::<WhatIfResponse>(
        &supervisor,
        format!("autoctl whatif '{}' --json", overrides_json),
    )
    .await
    .map(|whatif| Json(whatif).into_response())
    .unwrap_or_else(|r| r)
}
