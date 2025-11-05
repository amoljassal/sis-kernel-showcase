//! Logs and troubleshooting REST endpoints
//!
//! Provides log tailing, run history, and troubleshooting tools.

use super::handlers::ErrorResponse;
use crate::qemu::{QemuSupervisor, ReplayManager};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LogTailQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>, // daemon | qemu | kernel
}

fn default_limit() -> u32 {
    1000
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LogEntry {
    pub ts: u64,
    pub level: String,
    pub source: String,
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RunProfile {
    pub features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bringup: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StartRunRequest {
    pub profile: RunProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StartRunResponse {
    #[serde(rename = "runId")]
    pub run_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StopRunResponse {
    pub ok: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RunHistoryEntry {
    #[serde(rename = "runId")]
    pub run_id: String,
    pub profile: RunProfile,
    #[serde(rename = "startedAt")]
    pub started_at: u64,
    #[serde(rename = "stoppedAt", skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<u64>,
    pub markers: Vec<String>,
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Tail logs with optional filters
#[utoipa::path(
    get,
    path = "/api/v1/logs/tail",
    tag = "logs",
    params(
        ("limit" = Option<u32>, Query, description = "Max number of log entries (default 1000)"),
        ("level" = Option<String>, Query, description = "Filter by log level"),
        ("source" = Option<String>, Query, description = "Filter by source (daemon/qemu/kernel)")
    ),
    responses(
        (status = 200, description = "Log entries", body = Vec<LogEntry>),
        (status = 500, description = "Failed to get logs", body = ErrorResponse)
    )
)]
pub async fn logs_tail(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Query(query): Query<LogTailQuery>,
) -> Response {
    // For now, return empty logs as this would need actual log buffer implementation
    // In production, this would read from a circular buffer or log file
    let logs: Vec<LogEntry> = vec![];
    Json(logs).into_response()
}

/// Start a new run
#[utoipa::path(
    post,
    path = "/api/v1/runs/start",
    tag = "runs",
    request_body = StartRunRequest,
    responses(
        (status = 200, description = "Run started", body = StartRunResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 500, description = "Failed to start run", body = ErrorResponse)
    )
)]
pub async fn runs_start(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<StartRunRequest>,
) -> Response {
    // Generate run ID
    let run_id = uuid::Uuid::new_v4().to_string();

    // In production, this would:
    // 1. Store run metadata in run history
    // 2. Configure QEMU with specified features
    // 3. Start logging to run-specific buffer

    Json(StartRunResponse { run_id }).into_response()
}

/// Stop current run
#[utoipa::path(
    post,
    path = "/api/v1/runs/stop",
    tag = "runs",
    responses(
        (status = 200, description = "Run stopped", body = StopRunResponse),
        (status = 500, description = "Failed to stop run", body = ErrorResponse)
    )
)]
pub async fn runs_stop(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Response {
    // In production, this would:
    // 1. Mark current run as stopped
    // 2. Finalize logs and metrics
    // 3. Update run history

    Json(StopRunResponse { ok: true }).into_response()
}

/// List run history
#[utoipa::path(
    get,
    path = "/api/v1/runs/list",
    tag = "runs",
    responses(
        (status = 200, description = "Run history", body = Vec<RunHistoryEntry>),
        (status = 500, description = "Failed to get run history", body = ErrorResponse)
    )
)]
pub async fn runs_list(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Response {
    // For now, return empty history
    // In production, this would read from persistent run history store
    let runs: Vec<RunHistoryEntry> = vec![];
    Json(runs).into_response()
}

/// Export run logs and metrics
#[utoipa::path(
    get,
    path = "/api/v1/runs/{runId}/export",
    tag = "runs",
    params(
        ("runId" = String, Path, description = "Run ID")
    ),
    responses(
        (status = 200, description = "Run snapshot exported", content_type = "application/json"),
        (status = 404, description = "Run not found", body = ErrorResponse),
        (status = 500, description = "Failed to export run", body = ErrorResponse)
    )
)]
pub async fn runs_export(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Path(run_id): Path<String>,
) -> Response {
    // For now, return empty export
    // In production, this would:
    // 1. Collect logs for specified run
    // 2. Collect metrics for specified run
    // 3. Bundle into JSON snapshot
    // 4. Stream as downloadable file

    let export = serde_json::json!({
        "runId": run_id,
        "logs": [],
        "metrics": [],
        "markers": []
    });

    Json(export).into_response()
}
