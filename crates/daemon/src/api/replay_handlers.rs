//! Replay API handlers for testing without QEMU

use crate::qemu::{QemuSupervisor, ReplayManager, ReplaySpeed, ReplayStatus, ReplayTransport};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tracing::{debug, error};
use utoipa::ToSchema;

// Re-export problem+json ErrorResponse from handlers
pub use super::handlers::ErrorResponse;

/// Replay request
#[derive(Debug, serde::Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct ReplayRequest {
    /// Log source mode: "sample" for built-in samples or "upload" for custom file
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Log source identifier (sample name or file path)
    #[serde(rename = "logSource", default)]
    pub log_source: Option<String>,

    /// Base64-encoded log file content (only for mode=upload)
    #[serde(default)]
    pub file: Option<String>,

    /// Replay speed (realtime, fast, instant)
    #[serde(default = "default_speed")]
    pub speed: String,

    /// DEPRECATED: Use logSource with mode=sample instead
    #[serde(default)]
    pub sample: Option<String>,
}

fn default_mode() -> String {
    "sample".to_string()
}

fn default_speed() -> String {
    "realtime".to_string()
}

/// Replay response
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ReplayResponse {
    pub message: String,
    pub lines_processed: usize,
}

/// Replay a log file
#[utoipa::path(
    post,
    path = "/api/v1/replay",
    request_body = ReplayRequest,
    responses(
        (status = 200, description = "Replay started", body = ReplayResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 409, description = "Replay already running", body = ErrorResponse),
        (status = 500, description = "Replay failed", body = ErrorResponse)
    ),
    tag = "replay"
)]
pub async fn replay_start(
    State((supervisor, replay_manager)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(request): Json<ReplayRequest>,
) -> Result<Json<ReplayResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if replay already running
    let status = replay_manager.get_status().await;
    if status.state == crate::qemu::ReplayState::Running {
        let status = StatusCode::CONFLICT;
        return Err((
            status,
            Json(ErrorResponse::with_type(
                status,
                "Replay already running. Stop current replay first.".to_string(),
                Some("/errors/replay-busy".to_string()),
            )),
        ));
    }

    // Parse speed
    let speed = match request.speed.as_str() {
        "instant" => ReplaySpeed::Instant,
        "fast" => ReplaySpeed::Fast,
        _ => ReplaySpeed::RealTime,
    };

    // Determine log source
    let log_source = request.log_source.clone().or(request.sample.clone());
    let mode = request.mode.clone();

    let (path, source_display) = match mode.as_str() {
        "sample" => {
            let sample = log_source.ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new(
                        StatusCode::BAD_REQUEST,
                        "Missing logSource or sample field for mode=sample".to_string(),
                    )),
                )
            })?;

            let path = match sample.as_str() {
                "boot_minimal" => "samples/boot_minimal.log",
                "boot_with_metrics" => "samples/boot_with_metrics.log",
                "self_check" => "samples/self_check.log",
                // Additional samples for advanced panels and logs
                "logs_mixed" => "samples/logs_mixed.log",
                "boot_llm" => "samples/boot_llm.log",
                "boot_graph" => "samples/boot_graph.log",
                "boot_sched" => "samples/boot_sched.log",
                _ => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse::new(
                            StatusCode::BAD_REQUEST,
                            format!("Unknown sample: {}", sample),
                        )),
                    ))
                }
            };
            (path.to_string(), sample)
        }
        "upload" => {
            // TODO: Handle base64-decoded file content
            // For now, return not implemented
            return Err((
                StatusCode::NOT_IMPLEMENTED,
                Json(ErrorResponse::new(
                    StatusCode::NOT_IMPLEMENTED,
                    "File upload mode not yet implemented".to_string(),
                )),
            ));
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    StatusCode::BAD_REQUEST,
                    format!("Invalid mode: {}. Must be 'sample' or 'upload'", mode),
                )),
            ))
        }
    };

    debug!("Starting replay: mode={}, source={}, speed={:?}", mode, source_display, speed);

    // Mark replay as running
    replay_manager.start(source_display.clone(), mode.clone()).await;

    // Create replay transport using supervisor's event broadcaster
    let event_tx = supervisor.event_broadcaster();
    let replay = ReplayTransport::with_manager(event_tx, speed, replay_manager.clone());

    // Spawn replay task with progress updates
    let manager_clone = replay_manager.clone();
    tokio::spawn(async move {
        match replay.replay_file(&path).await {
            Ok(_) => {
                manager_clone.complete().await;
                debug!("Replay completed successfully");
            }
            Err(e) => {
                manager_clone.stop().await;
                error!("Replay failed: {}", e);
            }
        }
    });

    Ok(Json(ReplayResponse {
        message: format!("Replay started: {} (mode: {}, speed: {})", source_display, mode, request.speed),
        lines_processed: 0, // Updated as replay progresses via WebSocket events
    }))
}

/// Stop a running replay
#[utoipa::path(
    post,
    path = "/api/v1/replay/stop",
    responses(
        (status = 200, description = "Replay stopped", body = super::handlers::SuccessResponse),
        (status = 404, description = "No replay running", body = ErrorResponse),
    ),
    tag = "replay"
)]
pub async fn replay_stop(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Result<Json<super::handlers::SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (_, replay_manager) = &state;
    let status = replay_manager.get_status().await;
    if status.state != crate::qemu::ReplayState::Running {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::with_type(
                StatusCode::NOT_FOUND,
                "No replay currently running".to_string(),
                Some("/errors/not-found".to_string()),
            )),
        ));
    }

    replay_manager.stop().await;
    debug!("Replay stopped by user request");

    Ok(Json(super::handlers::SuccessResponse {
        message: "Replay stopped".to_string(),
    }))
}

/// Get replay status
#[utoipa::path(
    get,
    path = "/api/v1/replay/status",
    responses(
        (status = 200, description = "Replay status retrieved", body = ReplayStatus),
    ),
    tag = "replay"
)]
pub async fn replay_status(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Json<ReplayStatus> {
    let (_, replay_manager) = &state;
    Json(replay_manager.get_status().await)
}
