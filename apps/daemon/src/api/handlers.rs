//! API request handlers

use crate::qemu::{QemuConfig, QemuStatus, QemuSupervisor};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// API error response
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
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
    State(supervisor): State<Arc<QemuSupervisor>>,
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
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
    State(supervisor): State<Arc<QemuSupervisor>>,
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
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
    State(supervisor): State<Arc<QemuSupervisor>>,
) -> Json<QemuStatus> {
    Json(supervisor.status().await)
}
