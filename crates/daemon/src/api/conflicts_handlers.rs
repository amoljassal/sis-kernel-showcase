//! Conflict Resolution API handlers
//!
//! These endpoints wrap Phase 2 conflict resolution shell commands

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

/// Conflict resolution statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictStats {
    pub total_conflicts: u64,
    pub resolved_by_priority: u64,
    pub resolved_by_voting: u64,
    pub unresolved: u64,
    pub avg_resolution_time_us: u64,
}

/// Agent in a conflict
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictAgent {
    pub agent: String,
    pub action: String,
    pub confidence: f64,
    pub priority: u32,
}

/// Conflict resolution details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictResolution {
    pub strategy: String, // "priority", "voting", "unresolved"
    pub winner: String,
    pub action: String,
    pub reason: String,
}

/// Conflict entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Conflict {
    pub id: String,
    pub timestamp: String,
    pub agents: Vec<ConflictAgent>,
    pub resolution: ConflictResolution,
    pub resolution_time_us: u64,
}

/// Conflicts history response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConflictsResponse {
    pub conflicts: Vec<Conflict>,
}

/// Agent priority entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PriorityEntry {
    pub agent: String,
    pub priority: u32,
}

/// Priority table response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PriorityTableResponse {
    pub priorities: Vec<PriorityEntry>,
}

/// Query parameters for conflicts history
#[derive(Debug, Deserialize)]
pub struct ConflictsQuery {
    pub limit: Option<u32>,
    pub resolved: Option<bool>,
}

/// Get conflict resolution statistics
#[utoipa::path(
    get,
    path = "/api/v1/conflicts/stats",
    responses(
        (status = 200, description = "Conflict resolution statistics", body = ConflictStats),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "conflicts"
)]
pub async fn get_conflict_stats(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Getting conflict stats");
    exec_and_parse::<ConflictStats>(supervisor, "coordctl conflict-stats --json".to_string())
        .await
        .map(|stats| Json(stats).into_response())
        .unwrap_or_else(|r| r)
}

/// Get conflict resolution history
#[utoipa::path(
    get,
    path = "/api/v1/conflicts/history",
    params(
        ("limit" = Option<u32>, Query, description = "Max conflicts to return (default: 100)"),
        ("resolved" = Option<bool>, Query, description = "Filter by resolution status")
    ),
    responses(
        (status = 200, description = "Conflict history", body = ConflictsResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "conflicts"
)]
pub async fn get_conflict_history(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<ConflictsQuery>,
) -> Response {
    let (supervisor, _) = &state;
    let limit = params.limit.unwrap_or(100);
    debug!("Getting conflict history (limit: {})", limit);

    let mut cmd = format!("coordctl conflict-history --limit {} --json", limit);
    if let Some(resolved) = params.resolved {
        cmd = format!("{} --resolved={}", cmd, resolved);
    }

    exec_and_parse::<ConflictsResponse>(supervisor, cmd)
        .await
        .map(|conflicts| Json(conflicts).into_response())
        .unwrap_or_else(|r| r)
}

/// Get agent priority table
#[utoipa::path(
    get,
    path = "/api/v1/conflicts/priority-table",
    responses(
        (status = 200, description = "Agent priority table", body = PriorityTableResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "conflicts"
)]
pub async fn get_priority_table(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Getting priority table");
    exec_and_parse::<PriorityTableResponse>(supervisor, "coordctl priorities --json".to_string())
        .await
        .map(|priorities| Json(priorities).into_response())
        .unwrap_or_else(|r| r)
}
