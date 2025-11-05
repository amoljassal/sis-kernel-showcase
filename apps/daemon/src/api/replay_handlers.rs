//! Replay API handlers for testing without QEMU

use crate::qemu::{QemuSupervisor, ReplaySpeed, ReplayTransport};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tracing::{debug, error};
use utoipa::ToSchema;

/// Error response
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// Replay request
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct ReplayRequest {
    /// Sample file to replay (boot_minimal, boot_with_metrics, self_check)
    pub sample: String,
    /// Replay speed (realtime, fast, instant)
    #[serde(default)]
    pub speed: String,
}

/// Replay response
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ReplayResponse {
    pub message: String,
    pub lines_processed: usize,
}

/// Replay a sample log file
#[utoipa::path(
    post,
    path = "/api/v1/replay",
    request_body = ReplayRequest,
    responses(
        (status = 200, description = "Replay started", body = ReplayResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 500, description = "Replay failed", body = ErrorResponse)
    ),
    tag = "replay"
)]
pub async fn replay_sample(
    State(supervisor): State<Arc<QemuSupervisor>>,
    Json(request): Json<ReplayRequest>,
) -> Result<Json<ReplayResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Replaying sample: {} at speed: {}", request.sample, request.speed);

    // Parse speed
    let speed = match request.speed.as_str() {
        "instant" => ReplaySpeed::Instant,
        "fast" => ReplaySpeed::Fast,
        _ => ReplaySpeed::RealTime,
    };

    // Get sample file path
    let sample_path = match request.sample.as_str() {
        "boot_minimal" => "samples/boot_minimal.log",
        "boot_with_metrics" => "samples/boot_with_metrics.log",
        "self_check" => "samples/self_check.log",
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown sample: {}", request.sample),
                }),
            ));
        }
    };

    // Create replay transport using supervisor's event broadcaster
    let event_tx = supervisor.event_broadcaster();
    let replay = ReplayTransport::new(event_tx, speed);

    // Spawn replay task
    let path = sample_path.to_string();
    tokio::spawn(async move {
        if let Err(e) = replay.replay_file(&path).await {
            error!("Replay failed: {}", e);
        }
    });

    Ok(Json(ReplayResponse {
        message: format!("Replay started: {} (speed: {})", request.sample, request.speed),
        lines_processed: 0, // Updated as replay progresses via WebSocket events
    }))
}
