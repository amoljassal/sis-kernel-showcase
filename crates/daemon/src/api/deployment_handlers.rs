//! Deployment Management API handlers
//!
//! These endpoints wrap Phase 2 deployment management shell commands

use crate::qemu::QemuSupervisor;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;
use utoipa::ToSchema;

use super::handlers::exec_and_parse;

/// Current deployment phase details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CurrentPhase {
    pub id: String, // "A", "B", "C", "D"
    pub name: String,
    pub description: String,
    pub entered_at: String,
    pub min_duration_ms: u64,
    pub elapsed_ms: u64,
    pub can_advance: bool,
    pub traffic_percentage: u8,
    pub error_rate: f64,
    pub success_rate: f64,
}

/// Deployment status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeploymentStatus {
    pub current_phase: CurrentPhase,
    pub auto_advance_enabled: bool,
    pub auto_rollback_enabled: bool,
    pub rollback_count: u32,
    pub max_rollbacks: u32,
}

/// Metrics snapshot at phase transition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MetricsSnapshot {
    pub error_rate: f64,
    pub success_rate: f64,
    pub uptime_hours: f64,
}

/// Phase transition entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhaseTransition {
    pub timestamp: String,
    pub from_phase: String,
    pub to_phase: String,
    pub trigger: String, // "auto_advance", "auto_rollback", "manual_advance", "manual_rollback"
    pub reason: String,
    pub metrics_snapshot: MetricsSnapshot,
}

/// Deployment history response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeploymentHistoryResponse {
    pub transitions: Vec<PhaseTransition>,
}

/// Advance request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdvanceRequest {
    #[serde(default)]
    pub force: bool,
}

/// Rollback request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct RollbackRequest {
    pub reason: String,
}

/// Config update request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeploymentConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_advance_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_rollback_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_rate_threshold: Option<f64>,
}

/// Advance/rollback response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhaseChangeResponse {
    pub success: bool,
    pub old_phase: String,
    pub new_phase: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_count: Option<u32>,
}

/// Config update response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigUpdateResponse {
    pub success: bool,
    pub config: DeploymentConfig,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeploymentConfig {
    pub auto_advance_enabled: bool,
    pub auto_rollback_enabled: bool,
    pub error_rate_threshold: f64,
}

/// Query parameters for deployment history
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u32>,
}

/// Get deployment status
#[utoipa::path(
    get,
    path = "/api/v1/deployment/status",
    responses(
        (status = 200, description = "Deployment status", body = DeploymentStatus),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "deployment"
)]
pub async fn get_deployment_status(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Getting deployment status");
    exec_and_parse::<DeploymentStatus>(supervisor, "deployctl status --json".to_string())
        .await
        .map(|status| Json(status).into_response())
        .unwrap_or_else(|r| r)
}

/// Get deployment history
#[utoipa::path(
    get,
    path = "/api/v1/deployment/history",
    params(
        ("limit" = Option<u32>, Query, description = "Max transitions to return (default: 50)")
    ),
    responses(
        (status = 200, description = "Deployment history", body = DeploymentHistoryResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "deployment"
)]
pub async fn get_deployment_history(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<HistoryQuery>,
) -> Response {
    let (supervisor, _) = &state;
    let limit = params.limit.unwrap_or(50);
    debug!("Getting deployment history (limit: {})", limit);

    exec_and_parse::<DeploymentHistoryResponse>(
        supervisor,
        format!("deployctl history --limit {} --json", limit),
    )
    .await
    .map(|history| Json(history).into_response())
    .unwrap_or_else(|r| r)
}

/// Manually advance to next phase
#[utoipa::path(
    post,
    path = "/api/v1/deployment/advance",
    request_body = AdvanceRequest,
    responses(
        (status = 200, description = "Phase advanced", body = PhaseChangeResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "deployment"
)]
pub async fn advance_deployment(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(req): Json<AdvanceRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Advancing deployment phase (force: {})", req.force);

    let cmd = if req.force {
        "deployctl advance --force --json".to_string()
    } else {
        "deployctl advance --json".to_string()
    };

    exec_and_parse::<PhaseChangeResponse>(supervisor, cmd)
        .await
        .map(|response| Json(response).into_response())
        .unwrap_or_else(|r| r)
}

/// Manually rollback to previous phase
#[utoipa::path(
    post,
    path = "/api/v1/deployment/rollback",
    request_body = RollbackRequest,
    responses(
        (status = 200, description = "Phase rolled back", body = PhaseChangeResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "deployment"
)]
pub async fn rollback_deployment(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(req): Json<RollbackRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Rolling back deployment: {}", req.reason);

    exec_and_parse::<PhaseChangeResponse>(supervisor, "deployctl rollback --json".to_string())
        .await
        .map(|response| Json(response).into_response())
        .unwrap_or_else(|r| r)
}

/// Update deployment configuration
#[utoipa::path(
    post,
    path = "/api/v1/deployment/config",
    request_body = DeploymentConfigRequest,
    responses(
        (status = 200, description = "Configuration updated", body = ConfigUpdateResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "deployment"
)]
pub async fn update_deployment_config(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(req): Json<DeploymentConfigRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Updating deployment config");

    let mut cmd = "deployctl config".to_string();
    if let Some(auto_advance) = req.auto_advance_enabled {
        cmd = format!("{} --auto-advance={}", cmd, if auto_advance { "on" } else { "off" });
    }
    if let Some(auto_rollback) = req.auto_rollback_enabled {
        cmd = format!("{} --auto-rollback={}", cmd, if auto_rollback { "on" } else { "off" });
    }
    if let Some(threshold) = req.error_rate_threshold {
        cmd = format!("{} --error-threshold={}", cmd, threshold);
    }
    cmd = format!("{} --json", cmd);

    exec_and_parse::<ConfigUpdateResponse>(supervisor, cmd)
        .await
        .map(|response| Json(response).into_response())
        .unwrap_or_else(|r| r)
}
