//! Drift Detection API handlers
//!
//! These endpoints wrap Phase 2 model drift detection shell commands

use crate::qemu::QemuSupervisor;
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

/// Drift detection status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriftStatus {
    pub baseline_accuracy: f64,
    pub current_accuracy: f64,
    pub accuracy_delta: f64,
    pub drift_level: String, // "normal", "warning", "critical"
    pub sample_window_size: u32,
    pub samples_analyzed: u64,
    pub last_retrain: String,
    pub auto_retrain_enabled: bool,
    pub auto_retrain_threshold: f64,
}

/// Drift sample data point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriftSample {
    pub timestamp: String,
    pub accuracy: f64,
    pub drift_level: String, // "normal", "warning", "critical"
    pub accuracy_delta: f64,
    pub sample_count: u64,
}

/// Drift history response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriftHistoryResponse {
    pub samples: Vec<DriftSample>,
}

/// Retrain request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct RetrainRequest {
    #[serde(default = "default_training_examples")]
    pub training_examples: u32,
    #[serde(default = "default_epochs")]
    pub epochs: u32,
}

fn default_training_examples() -> u32 {
    1000
}

fn default_epochs() -> u32 {
    10
}

/// Retrain response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RetrainResponse {
    pub success: bool,
    pub training_started: bool,
    pub timestamp: String,
    pub estimated_duration_ms: u64,
}

/// Reset baseline response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResetBaselineResponse {
    pub success: bool,
    pub old_baseline: f64,
    pub new_baseline: f64,
    pub timestamp: String,
}

/// Query parameters for drift history
#[derive(Debug, Deserialize)]
pub struct DriftHistoryQuery {
    pub limit: Option<u32>,
    pub time_range: Option<u64>, // in seconds
}

/// Get drift detection status
#[utoipa::path(
    get,
    path = "/api/v1/drift/status",
    responses(
        (status = 200, description = "Drift detection status", body = DriftStatus),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "drift"
)]
pub async fn get_drift_status(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Getting drift status");
    exec_and_parse::<DriftStatus>(supervisor, "driftctl status --json".to_string())
        .await
        .map(|status| Json(status).into_response())
        .unwrap_or_else(|r| r)
}

/// Get drift detection history
#[utoipa::path(
    get,
    path = "/api/v1/drift/history",
    params(
        ("limit" = Option<u32>, Query, description = "Max entries to return (default: 100)"),
        ("time_range" = Option<u64>, Query, description = "Time range in seconds (default: 86400 = 24h)")
    ),
    responses(
        (status = 200, description = "Drift history", body = DriftHistoryResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "drift"
)]
pub async fn get_drift_history(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<DriftHistoryQuery>,
) -> Response {
    let (supervisor, _) = &state;
    let limit = params.limit.unwrap_or(100);
    let time_range = params.time_range.unwrap_or(86400); // default 24h
    debug!(
        "Getting drift history (limit: {}, time_range: {}s)",
        limit, time_range
    );

    exec_and_parse::<DriftHistoryResponse>(
        supervisor,
        format!("driftctl history --limit {} --json", limit),
    )
    .await
    .map(|history| Json(history).into_response())
    .unwrap_or_else(|r| r)
}

/// Manually trigger model retraining
#[utoipa::path(
    post,
    path = "/api/v1/drift/retrain",
    request_body = RetrainRequest,
    responses(
        (status = 200, description = "Retraining started", body = RetrainResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "drift"
)]
pub async fn trigger_retrain(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(req): Json<RetrainRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!(
        "Triggering retrain (examples: {}, epochs: {})",
        req.training_examples, req.epochs
    );

    exec_and_parse::<RetrainResponse>(
        supervisor,
        format!(
            "driftctl retrain --examples={} --epochs={} --json",
            req.training_examples, req.epochs
        ),
    )
    .await
    .map(|response| Json(response).into_response())
    .unwrap_or_else(|r| r)
}

/// Reset baseline accuracy to current accuracy
#[utoipa::path(
    post,
    path = "/api/v1/drift/reset-baseline",
    responses(
        (status = 200, description = "Baseline reset", body = ResetBaselineResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "drift"
)]
pub async fn reset_baseline(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Resetting drift baseline");
    exec_and_parse::<ResetBaselineResponse>(supervisor, "driftctl reset-baseline --json".to_string())
        .await
        .map(|response| Json(response).into_response())
        .unwrap_or_else(|r| r)
}
