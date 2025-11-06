//! Memory approvals API handlers
//!
//! These endpoints wrap `memctl` shell commands for memory approval management

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

/// Memory approval status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryApprovalStatus {
    pub enabled: bool,
    pub query_mode: bool, // dry-run mode
    pub pending_count: usize,
    pub total_approved: u64,
    pub total_rejected: u64,
}

/// Pending memory operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PendingOperation {
    pub id: String,
    pub op_type: String, // "alloc", "free", "remap"
    pub confidence: f64,
    pub risk: String, // "low", "medium", "high"
    pub reason: String,
    pub timestamp: i64,
    pub size_bytes: Option<u64>,
    pub address: Option<String>,
}

/// Approve request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApproveRequest {
    pub n: Option<u32>, // Number of operations to approve (default 1)
}

/// Reject request
#[derive(Debug, Deserialize, ToSchema)]
pub struct RejectRequest {
    pub id: Option<String>, // If None, reject all
}

/// Approval toggle request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApprovalToggleRequest {
    pub action: String, // "on", "off", or "status"
}

/// Query parameters for approvals list
#[derive(Debug, Deserialize)]
pub struct ApprovalsQuery {
    pub limit: Option<u32>, // Max pending to return (default 100)
}

/// Get list of pending memory approvals
#[utoipa::path(
    get,
    path = "/api/v1/mem/approvals",
    params(
        ("limit" = Option<u32>, Query, description = "Max pending operations to return (default 100)")
    ),
    responses(
        (status = 200, description = "Pending operations", body = Vec<PendingOperation>),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "memory"
)]
pub async fn mem_get_approvals(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<ApprovalsQuery>,
) -> Response {
    let (supervisor, _) = &state;
    let limit = params.limit.unwrap_or(100);
    debug!("Getting up to {} pending approvals", limit);

    exec_and_parse::<Vec<PendingOperation>>(
        &supervisor,
        format!("memctl approvals --limit {} --json", limit),
    )
    .await
    .map(|ops| Json(ops).into_response())
    .unwrap_or_else(|r| r)
}

/// Toggle approval mode or get status
#[utoipa::path(
    post,
    path = "/api/v1/mem/approval",
    request_body = ApprovalToggleRequest,
    responses(
        (status = 200, description = "Approval status updated", body = MemoryApprovalStatus),
        (status = 400, description = "Invalid action", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "memory"
)]
pub async fn mem_approval_toggle(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(request): Json<ApprovalToggleRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let action = request.action.as_str();
    debug!("Memory approval action: {}", action);

    if !["on", "off", "status"].contains(&action) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_type(
                StatusCode::BAD_REQUEST,
                format!("Action must be 'on', 'off', or 'status', got '{}'", action),
                Some("/errors/invalid-action".to_string()),
            )),
        )
            .into_response();
    }

    exec_and_parse::<MemoryApprovalStatus>(
        &supervisor,
        format!("memctl approval {} --json", action),
    )
    .await
    .map(|status| Json(status).into_response())
    .unwrap_or_else(|r| r)
}

/// Approve N pending operations
#[utoipa::path(
    post,
    path = "/api/v1/mem/approve",
    request_body = ApproveRequest,
    responses(
        (status = 200, description = "Operations approved", body = MemoryApprovalStatus),
        (status = 400, description = "Invalid count", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "memory"
)]
pub async fn mem_approve(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(request): Json<ApproveRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let n = request.n.unwrap_or(1);
    debug!("Approving {} operations", n);

    exec_and_parse::<MemoryApprovalStatus>(
        &supervisor,
        format!("memctl approve --n {} --json", n),
    )
    .await
    .map(|status| Json(status).into_response())
    .unwrap_or_else(|r| r)
}

/// Reject operation(s)
#[utoipa::path(
    post,
    path = "/api/v1/mem/reject",
    request_body = RejectRequest,
    responses(
        (status = 200, description = "Operations rejected", body = MemoryApprovalStatus),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "memory"
)]
pub async fn mem_reject(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(request): Json<RejectRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let command = if let Some(id) = request.id {
        debug!("Rejecting operation {}", id);
        format!("memctl reject --id {} --json", id)
    } else {
        debug!("Rejecting all pending operations");
        "memctl reject --all --json".to_string()
    };

    exec_and_parse::<MemoryApprovalStatus>(&supervisor, command)
        .await
        .map(|status| Json(status).into_response())
        .unwrap_or_else(|r| r)
}
