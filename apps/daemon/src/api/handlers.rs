//! API request handlers

use crate::config::DaemonConfig;
use crate::qemu::{QemuConfig, QemuStatus, QemuSupervisor};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// API error response (RFC 7807 problem+json format)
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// A URI reference that identifies the problem type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// A short, human-readable summary of the problem type
    pub title: String,

    /// The HTTP status code
    pub status: u16,

    /// A human-readable explanation specific to this occurrence
    pub detail: String,

    /// A URI reference that identifies the specific occurrence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    /// Request ID for tracing (X-Request-Id)
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestId")]
    pub request_id: Option<String>,

    /// Legacy error field for backward compatibility
    #[serde(skip)]
    pub error: String,
}

impl ErrorResponse {
    pub fn new(status: StatusCode, detail: String) -> Self {
        Self::with_type(status, detail, None)
    }

    pub fn with_type(status: StatusCode, detail: String, error_type: Option<String>) -> Self {
        Self {
            r#type: error_type,
            title: status.canonical_reason().unwrap_or("Error").to_string(),
            status: status.as_u16(),
            detail: detail.clone(),
            instance: None,
            request_id: None,
            error: detail,
        }
    }

    pub fn with_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }
}

/// API success response
#[derive(Debug, Serialize, ToSchema)]
pub struct SuccessResponse {
    pub message: String,
}

/// Health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 0, // TODO: track actual uptime
    })
}

/// Run QEMU with configuration
#[utoipa::path(
    post,
    path = "/api/v1/qemu/run",
    request_body = QemuConfig,
    responses(
        (status = 200, description = "QEMU started successfully", body = SuccessResponse),
        (status = 400, description = "Invalid configuration", body = ErrorResponse),
        (status = 500, description = "Failed to start QEMU", body = ErrorResponse)
    ),
    tag = "qemu"
)]
pub async fn qemu_run(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(config): Json<QemuConfig>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    supervisor
        .run(config)
        .await
        .map(|_| {
            Json(SuccessResponse {
                message: "QEMU started".to_string(),
            })
        })
        .map_err(|e| {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            (status, Json(ErrorResponse::new(status, e.to_string())))
        })
}

/// Stop QEMU
#[utoipa::path(
    post,
    path = "/api/v1/qemu/stop",
    responses(
        (status = 200, description = "QEMU stopped successfully", body = SuccessResponse),
        (status = 500, description = "Failed to stop QEMU", body = ErrorResponse)
    ),
    tag = "qemu"
)]
pub async fn qemu_stop(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    supervisor
        .stop()
        .await
        .map(|_| {
            Json(SuccessResponse {
                message: "QEMU stopped".to_string(),
            })
        })
        .map_err(|e| {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            (status, Json(ErrorResponse::new(status, e.to_string())))
        })
}

/// Get QEMU status
#[utoipa::path(
    get,
    path = "/api/v1/qemu/status",
    responses(
        (status = 200, description = "QEMU status retrieved", body = QemuStatus)
    ),
    tag = "qemu"
)]
pub async fn qemu_status(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Json<QemuStatus> {
    Json(supervisor.status().await)
}

/// Get daemon configuration
#[utoipa::path(
    get,
    path = "/api/v1/config",
    responses(
        (status = 200, description = "Configuration retrieved", body = DaemonConfig)
    ),
    tag = "config"
)]
pub async fn get_config() -> Json<DaemonConfig> {
    Json(DaemonConfig::from_env())
}
