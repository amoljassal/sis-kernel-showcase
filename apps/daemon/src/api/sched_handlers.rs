//! Scheduling control REST endpoints
//!
//! Wraps `schedctl` shell commands with REST API.

use super::handlers::{exec_and_parse, ErrorResponse};
use crate::qemu::{QemuEvent, QemuSupervisor, ReplayManager};
use axum::{
    extract::State,
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
pub struct Workload {
    pub pid: u32,
    pub name: String,
    pub prio: u8,
    pub cpu: u8,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetPriorityRequest {
    pub pid: u32,
    pub prio: u8,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetAffinityRequest {
    pub pid: u32,
    #[serde(rename = "cpuMask")]
    pub cpu_mask: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetFeatureRequest {
    pub name: String,
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SchedResponse {
    pub ok: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CircuitBreakerState {
    pub state: String, // "Closed" | "Open" | "HalfOpen"
    pub consecutive_failures: u32,
    pub failure_threshold: u32,
    pub reset_timeout_us: u64,
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Get list of workloads
#[utoipa::path(
    get,
    path = "/api/v1/sched/workloads",
    tag = "scheduling",
    responses(
        (status = 200, description = "List of workloads", body = Vec<Workload>),
        (status = 500, description = "Failed to get workloads", body = ErrorResponse)
    )
)]
pub async fn sched_workloads(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Response {
    exec_and_parse::<Vec<Workload>>(&supervisor, "schedctl workloads --json".to_string())
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

/// Set workload priority
#[utoipa::path(
    post,
    path = "/api/v1/sched/priorities",
    tag = "scheduling",
    request_body = SetPriorityRequest,
    responses(
        (status = 200, description = "Priority updated", body = SchedResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 404, description = "Process not found", body = ErrorResponse),
        (status = 500, description = "Failed to set priority", body = ErrorResponse)
    )
)]
pub async fn sched_set_priority(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<SetPriorityRequest>,
) -> Response {
    let cmd = format!(
        "schedctl set-priority --pid {} --prio {} --json",
        req.pid, req.prio
    );
    match exec_and_parse::<SchedResponse>(&supervisor, cmd).await {
        Ok(resp) => {
            // Emit SchedEvent
            let payload = serde_json::json!({ "pid": req.pid, "prio": req.prio });
            let event = QemuEvent::SchedEvent {
                event: "prio_change".to_string(),
                payload,
                ts: chrono::Utc::now().timestamp_millis(),
            };
            supervisor.broadcast_event(event);
            Json(resp).into_response()
        }
        Err(r) => r,
    }
}

/// Set workload CPU affinity
#[utoipa::path(
    post,
    path = "/api/v1/sched/affinity",
    tag = "scheduling",
    request_body = SetAffinityRequest,
    responses(
        (status = 200, description = "Affinity updated", body = SchedResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 404, description = "Process not found", body = ErrorResponse),
        (status = 500, description = "Failed to set affinity", body = ErrorResponse)
    )
)]
pub async fn sched_set_affinity(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<SetAffinityRequest>,
) -> Response {
    let cmd = format!(
        "schedctl set-affinity --pid {} --mask {} --json",
        req.pid, req.cpu_mask
    );
    match exec_and_parse::<SchedResponse>(&supervisor, cmd).await {
        Ok(resp) => {
            // Emit SchedEvent
            let payload = serde_json::json!({ "pid": req.pid, "cpuMask": req.cpu_mask });
            let event = QemuEvent::SchedEvent {
                event: "affinity_change".to_string(),
                payload,
                ts: chrono::Utc::now().timestamp_millis(),
            };
            supervisor.broadcast_event(event);
            Json(resp).into_response()
        }
        Err(r) => r,
    }
}

/// Toggle scheduling feature
#[utoipa::path(
    post,
    path = "/api/v1/sched/feature",
    tag = "scheduling",
    request_body = SetFeatureRequest,
    responses(
        (status = 200, description = "Feature toggled", body = SchedResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 500, description = "Failed to toggle feature", body = ErrorResponse)
    )
)]
pub async fn sched_set_feature(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<SetFeatureRequest>,
) -> Response {
    let action = if req.enable { "enable" } else { "disable" };
    let cmd = format!(
        "schedctl feature --name {} --action {} --json",
        req.name, action
    );
    match exec_and_parse::<SchedResponse>(&supervisor, cmd).await {
        Ok(resp) => {
            // Emit SchedEvent
            let payload = serde_json::json!({ "name": req.name, "enable": req.enable });
            let event = QemuEvent::SchedEvent {
                event: "feature_toggle".to_string(),
                payload,
                ts: chrono::Utc::now().timestamp_millis(),
            };
            supervisor.broadcast_event(event);
            Json(resp).into_response()
        }
        Err(r) => r,
    }
}

/// Get circuit breaker state
#[utoipa::path(
    get,
    path = "/api/v1/sched/circuit-breaker",
    tag = "scheduling",
    responses(
        (status = 200, description = "Circuit breaker state", body = CircuitBreakerState),
        (status = 500, description = "Failed to get circuit breaker state", body = ErrorResponse)
    )
)]
pub async fn sched_circuit_breaker_status(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Response {
    exec_and_parse::<CircuitBreakerState>(&supervisor, "schedctl circuit-breaker --json".to_string())
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

/// Reset circuit breaker
#[utoipa::path(
    post,
    path = "/api/v1/sched/circuit-breaker/reset",
    tag = "scheduling",
    responses(
        (status = 200, description = "Circuit breaker reset", body = SchedResponse),
        (status = 500, description = "Failed to reset circuit breaker", body = ErrorResponse)
    )
)]
pub async fn sched_circuit_breaker_reset(
    State((supervisor, _)): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Response {
    exec_and_parse::<SchedResponse>(&supervisor, "schedctl circuit-breaker --reset --json".to_string())
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}
