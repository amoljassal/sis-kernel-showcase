//! Crash capture and incident management REST endpoints
//!
//! Provides crash ingestion, querying, and incident tracking.

use super::handlers::ErrorResponse;
use crate::qemu::{QemuSupervisor, ReplayManager};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CrashLog {
    #[serde(rename = "crashId")]
    pub crash_id: String,
    pub ts: u64,
    pub panic_msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub severity: String, // critical | high | medium | low
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IngestCrashRequest {
    pub panic_msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default = "default_severity")]
    pub severity: String,
}

fn default_severity() -> String {
    "high".to_string()
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IngestCrashResponse {
    #[serde(rename = "crashId")]
    pub crash_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CrashListQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    50
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CrashListResponse {
    pub crashes: Vec<CrashLog>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Incident {
    #[serde(rename = "incidentId")]
    pub incident_id: String,
    #[serde(rename = "crashId")]
    pub crash_id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    #[serde(rename = "resolvedAt", skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<u64>,
    pub status: String, // open | investigating | resolved
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateIncidentRequest {
    #[serde(rename = "crashId")]
    pub crash_id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateIncidentResponse {
    #[serde(rename = "incidentId")]
    pub incident_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IncidentListQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct IncidentListResponse {
    pub incidents: Vec<Incident>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Ingest a crash report from the kernel
#[utoipa::path(
    post,
    path = "/api/v1/crash",
    tag = "crashes",
    request_body = IngestCrashRequest,
    responses(
        (status = 201, description = "Crash ingested", body = IngestCrashResponse),
        (status = 400, description = "Invalid crash data", body = ErrorResponse),
        (status = 500, description = "Failed to ingest crash", body = ErrorResponse)
    )
)]
pub async fn crash_ingest(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<IngestCrashRequest>,
) -> Response {
    // Validate severity
    if !matches!(req.severity.as_str(), "critical" | "high" | "medium" | "low") {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid severity level".to_string(),
            }),
        )
            .into_response();
    }

    // Generate crash ID
    let crash_id = uuid::Uuid::new_v4().to_string();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let crash = CrashLog {
        crash_id: crash_id.clone(),
        ts,
        panic_msg: req.panic_msg,
        stack_trace: req.stack_trace,
        registers: req.registers,
        run_id: req.run_id,
        severity: req.severity,
    };

    // In production, this would:
    // 1. Store crash in persistent storage (SQLite/PostgreSQL)
    // 2. Broadcast crash event via WebSocket
    // 3. Trigger auto-incident creation for critical crashes

    // TODO: Store crash in supervisor state or dedicated crash store
    // TODO: Broadcast via WebSocket

    (
        StatusCode::CREATED,
        Json(IngestCrashResponse { crash_id }),
    )
        .into_response()
}

/// List crashes with pagination and filters
#[utoipa::path(
    get,
    path = "/api/v1/crashes",
    tag = "crashes",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default 50)"),
        ("severity" = Option<String>, Query, description = "Filter by severity"),
        ("run_id" = Option<String>, Query, description = "Filter by run ID")
    ),
    responses(
        (status = 200, description = "List of crashes", body = CrashListResponse),
        (status = 500, description = "Failed to list crashes", body = ErrorResponse)
    )
)]
pub async fn crash_list(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Query(query): Query<CrashListQuery>,
) -> Response {
    // In production, this would:
    // 1. Query persistent storage with filters
    // 2. Apply pagination
    // 3. Return results

    // For now, return empty list
    let response = CrashListResponse {
        crashes: vec![],
        total: 0,
        page: query.page,
        page_size: query.page_size,
    };

    Json(response).into_response()
}

/// Create an incident from a crash
#[utoipa::path(
    post,
    path = "/api/v1/incidents",
    tag = "incidents",
    request_body = CreateIncidentRequest,
    responses(
        (status = 201, description = "Incident created", body = CreateIncidentResponse),
        (status = 400, description = "Invalid incident data", body = ErrorResponse),
        (status = 404, description = "Crash not found", body = ErrorResponse),
        (status = 500, description = "Failed to create incident", body = ErrorResponse)
    )
)]
pub async fn incident_create(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<CreateIncidentRequest>,
) -> Response {
    // Validate title and description
    if req.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Title cannot be empty".to_string(),
            }),
        )
            .into_response();
    }

    // Generate incident ID
    let incident_id = uuid::Uuid::new_v4().to_string();

    // In production, this would:
    // 1. Verify crash exists
    // 2. Create incident record in storage
    // 3. Link incident to crash
    // 4. Broadcast incident event via WebSocket

    (
        StatusCode::CREATED,
        Json(CreateIncidentResponse { incident_id }),
    )
        .into_response()
}

/// List incidents with pagination and filters
#[utoipa::path(
    get,
    path = "/api/v1/incidents",
    tag = "incidents",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<u32>, Query, description = "Items per page (default 50)"),
        ("status" = Option<String>, Query, description = "Filter by status (open/investigating/resolved)")
    ),
    responses(
        (status = 200, description = "List of incidents", body = IncidentListResponse),
        (status = 500, description = "Failed to list incidents", body = ErrorResponse)
    )
)]
pub async fn incident_list(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Query(query): Query<IncidentListQuery>,
) -> Response {
    // In production, this would:
    // 1. Query persistent storage with filters
    // 2. Apply pagination
    // 3. Return results

    // For now, return empty list
    let response = IncidentListResponse {
        incidents: vec![],
        total: 0,
        page: query.page,
        page_size: query.page_size,
    };

    Json(response).into_response()
}
