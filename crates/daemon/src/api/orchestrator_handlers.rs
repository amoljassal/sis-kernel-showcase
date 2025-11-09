//! Orchestrator API handlers
//!
//! These endpoints wrap Phase 2 multi-agent orchestration shell commands

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

/// Orchestration statistics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OrchestrationStats {
    pub total_decisions: u64,
    pub unanimous: u64,
    pub majority: u64,
    pub safety_overrides: u64,
    pub no_consensus: u64,
    pub avg_latency_us: u64,
}

/// Coordinated decision entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CoordinatedDecision {
    pub timestamp: String,
    #[serde(rename = "type")]
    pub decision_type: String, // "unanimous", "majority", "safety_override", "no_consensus"
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overridden_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overridden_agents: Option<Vec<String>>,
    pub latency_us: u64,
}

/// Agent status information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub status: String, // "active", "inactive", "error"
    pub priority: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_decision: Option<AgentLastDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<AgentStats>,
}

/// Agent's last decision
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentLastDecision {
    pub timestamp: String,
    pub action: String,
    pub confidence: f64,
}

/// Agent statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentStats {
    pub total_decisions: u64,
    pub avg_confidence: f64,
}

/// Decisions list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DecisionsResponse {
    pub decisions: Vec<CoordinatedDecision>,
}

/// Agents list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentsResponse {
    pub agents: Vec<AgentInfo>,
}

/// Query parameters for decisions endpoint
#[derive(Debug, Deserialize)]
pub struct DecisionsQuery {
    pub limit: Option<u32>,
    #[serde(rename = "type")]
    pub decision_type: Option<String>,
}

/// Get orchestration statistics
#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/stats",
    responses(
        (status = 200, description = "Orchestration statistics", body = OrchestrationStats),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "orchestrator"
)]
pub async fn get_orchestrator_stats(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Getting orchestrator stats");
    exec_and_parse::<OrchestrationStats>(supervisor, "coordctl status --json".to_string())
        .await
        .map(|stats| Json(stats).into_response())
        .unwrap_or_else(|r| r)
}

/// Get recent coordinated decisions
#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/decisions",
    params(
        ("limit" = Option<u32>, Query, description = "Max decisions to return (default: 100, max: 1000)"),
        ("type" = Option<String>, Query, description = "Filter by decision type")
    ),
    responses(
        (status = 200, description = "Recent decisions", body = DecisionsResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "orchestrator"
)]
pub async fn get_orchestrator_decisions(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<DecisionsQuery>,
) -> Response {
    let (supervisor, _) = &state;
    let limit = params.limit.unwrap_or(100).min(1000);
    debug!("Getting orchestrator decisions (limit: {})", limit);

    let mut cmd = format!("coordctl history --limit {} --json", limit);
    if let Some(dtype) = params.decision_type {
        cmd = format!("{} --type {}", cmd, dtype);
    }

    exec_and_parse::<DecisionsResponse>(supervisor, cmd)
        .await
        .map(|decisions| Json(decisions).into_response())
        .unwrap_or_else(|r| r)
}

/// Get agent status
#[utoipa::path(
    get,
    path = "/api/v1/orchestrator/agents",
    responses(
        (status = 200, description = "Agent status list", body = AgentsResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "orchestrator"
)]
pub async fn get_orchestrator_agents(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Getting orchestrator agents");
    exec_and_parse::<AgentsResponse>(supervisor, "agentctl list --json".to_string())
        .await
        .map(|agents| Json(agents).into_response())
        .unwrap_or_else(|r| r)
}
