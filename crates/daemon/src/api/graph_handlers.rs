//! Graph control REST endpoints
//!
//! Wraps `graphctl` shell commands with REST API.

use super::handlers::{exec_and_parse, ErrorResponse};
use crate::qemu::supervisor::{
    GraphChannel as EventGraphChannel, GraphOperator as EventGraphOperator,
    GraphOperatorStats, GraphStateData,
};
use crate::qemu::{QemuEvent, QemuSupervisor, ReplayManager};
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateGraphResponse {
    #[serde(rename = "graphId")]
    pub graph_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddChannelRequest {
    #[serde(rename = "graphId")]
    pub graph_id: String,
    pub cap: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddChannelResponse {
    #[serde(rename = "channelId")]
    pub channel_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddOperatorRequest {
    #[serde(rename = "graphId")]
    pub graph_id: String,
    #[serde(rename = "opId")]
    pub op_id: String,
    #[serde(rename = "in", skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(rename = "out", skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prio: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(rename = "inSchema", skip_serializing_if = "Option::is_none")]
    pub in_schema: Option<String>,
    #[serde(rename = "outSchema", skip_serializing_if = "Option::is_none")]
    pub out_schema: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddOperatorResponse {
    #[serde(rename = "operatorId")]
    pub operator_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StartGraphRequest {
    #[serde(rename = "graphId")]
    pub graph_id: String,
    pub steps: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StartGraphResponse {
    pub started: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PredictRequest {
    #[serde(rename = "opId")]
    pub op_id: String,
    pub latency_us: u64,
    pub depth: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prio: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PredictResponse {
    pub predicted: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conf: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FeedbackRequest {
    #[serde(rename = "opId")]
    pub op_id: String,
    pub verdict: String, // 'helpful' | 'not_helpful' | 'expected'
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FeedbackResponse {
    pub recorded: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GraphStateQuery {
    #[serde(rename = "graphId")]
    pub graph_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GraphOperator {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prio: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GraphChannel {
    pub id: String,
    pub cap: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GraphStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_executions: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GraphState {
    pub operators: Vec<GraphOperator>,
    pub channels: Vec<GraphChannel>,
    pub stats: GraphStats,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExportGraphRequest {
    #[serde(rename = "graphId")]
    pub graph_id: String,
    pub format: String, // 'json'
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExportGraphResponse {
    pub json: String,
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Create a new graph
#[utoipa::path(
    post,
    path = "/api/v1/graph/create",
    tag = "graph",
    responses(
        (status = 200, description = "Graph created", body = CreateGraphResponse),
        (status = 500, description = "Failed to create graph", body = ErrorResponse)
    )
)]
pub async fn graph_create(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    match exec_and_parse::<CreateGraphResponse>(supervisor, "graphctl create --json".to_string())
        .await
    {
        Ok(resp) => {
            // Emit GraphState event
            let graph_id = resp.graph_id.clone();
            let supervisor_clone = Arc::clone(supervisor);
            tokio::spawn(async move {
                emit_graph_state_event(&supervisor_clone, graph_id).await;
            });
            Json(resp).into_response()
        }
        Err(r) => r,
    }
}

/// Add a channel to a graph
#[utoipa::path(
    post,
    path = "/api/v1/graph/add-channel",
    tag = "graph",
    request_body = AddChannelRequest,
    responses(
        (status = 200, description = "Channel added", body = AddChannelResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 500, description = "Failed to add channel", body = ErrorResponse)
    )
)]
pub async fn graph_add_channel(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<AddChannelRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let cmd = format!(
        "graphctl add-channel --graph {} --cap {} --json",
        req.graph_id, req.cap
    );
    let graph_id = req.graph_id.clone();
    match exec_and_parse::<AddChannelResponse>(&supervisor, cmd).await {
        Ok(resp) => {
            // Emit GraphState event
            let supervisor_clone = Arc::clone(&supervisor);
            tokio::spawn(async move {
                emit_graph_state_event(&supervisor_clone, graph_id).await;
            });
            Json(resp).into_response()
        }
        Err(r) => r,
    }
}

/// Add an operator to a graph
#[utoipa::path(
    post,
    path = "/api/v1/graph/add-operator",
    tag = "graph",
    request_body = AddOperatorRequest,
    responses(
        (status = 200, description = "Operator added", body = AddOperatorResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 500, description = "Failed to add operator", body = ErrorResponse)
    )
)]
pub async fn graph_add_operator(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<AddOperatorRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let mut cmd = format!(
        "graphctl add-operator --graph {} --op {}",
        req.graph_id, req.op_id
    );

    if let Some(input) = &req.input {
        cmd.push_str(&format!(" --in {}", input));
    }
    if let Some(output) = &req.output {
        cmd.push_str(&format!(" --out {}", output));
    }
    if let Some(prio) = req.prio {
        cmd.push_str(&format!(" --prio {}", prio));
    }
    if let Some(stage) = &req.stage {
        cmd.push_str(&format!(" --stage {}", stage));
    }
    if let Some(in_schema) = &req.in_schema {
        cmd.push_str(&format!(" --in-schema {}", in_schema));
    }
    if let Some(out_schema) = &req.out_schema {
        cmd.push_str(&format!(" --out-schema {}", out_schema));
    }

    cmd.push_str(" --json");

    let graph_id = req.graph_id.clone();
    match exec_and_parse::<AddOperatorResponse>(&supervisor, cmd).await {
        Ok(resp) => {
            // Emit GraphState event
            let supervisor_clone = Arc::clone(&supervisor);
            tokio::spawn(async move {
                emit_graph_state_event(&supervisor_clone, graph_id).await;
            });
            Json(resp).into_response()
        }
        Err(r) => r,
    }
}

/// Start a graph execution
#[utoipa::path(
    post,
    path = "/api/v1/graph/start",
    tag = "graph",
    request_body = StartGraphRequest,
    responses(
        (status = 200, description = "Graph started", body = StartGraphResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 409, description = "Busy", body = ErrorResponse),
        (status = 500, description = "Failed to start graph", body = ErrorResponse)
    )
)]
pub async fn graph_start(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<StartGraphRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let cmd = format!(
        "graphctl start --graph {} --steps {} --json",
        req.graph_id, req.steps
    );
    let graph_id = req.graph_id.clone();
    match exec_and_parse::<StartGraphResponse>(&supervisor, cmd).await {
        Ok(resp) => {
            // Emit GraphState event
            let supervisor_clone = Arc::clone(&supervisor);
            tokio::spawn(async move {
                emit_graph_state_event(&supervisor_clone, graph_id).await;
            });
            Json(resp).into_response()
        }
        Err(r) => r,
    }
}

/// Predict operator performance
#[utoipa::path(
    post,
    path = "/api/v1/graph/predict",
    tag = "graph",
    request_body = PredictRequest,
    responses(
        (status = 200, description = "Prediction result", body = PredictResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 500, description = "Failed to predict", body = ErrorResponse)
    )
)]
pub async fn graph_predict(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<PredictRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let mut cmd = format!(
        "graphctl predict --op {} --latency {} --depth {}",
        req.op_id, req.latency_us, req.depth
    );

    if let Some(prio) = req.prio {
        cmd.push_str(&format!(" --prio {}", prio));
    }

    cmd.push_str(" --json");

    // Predict doesn't have graphId in request, so we don't emit event
    exec_and_parse::<PredictResponse>(&supervisor, cmd)
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

/// Submit feedback for operator
#[utoipa::path(
    post,
    path = "/api/v1/graph/feedback",
    tag = "graph",
    request_body = FeedbackRequest,
    responses(
        (status = 200, description = "Feedback recorded", body = FeedbackResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 500, description = "Failed to record feedback", body = ErrorResponse)
    )
)]
pub async fn graph_feedback(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<FeedbackRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let cmd = format!(
        "graphctl feedback --op {} --verdict {} --json",
        req.op_id, req.verdict
    );
    // Feedback doesn't have graphId in request, so we don't emit event
    exec_and_parse::<FeedbackResponse>(&supervisor, cmd)
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

/// Get graph state
#[utoipa::path(
    get,
    path = "/api/v1/graph/state",
    tag = "graph",
    params(
        ("graphId" = String, Query, description = "Graph ID")
    ),
    responses(
        (status = 200, description = "Graph state", body = GraphState),
        (status = 404, description = "Graph not found", body = ErrorResponse),
        (status = 500, description = "Failed to get graph state", body = ErrorResponse)
    )
)]
pub async fn graph_state(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Query(params): Query<GraphStateQuery>,
) -> Response {
    let (supervisor, _) = &state;
    let cmd = format!("graphctl state --graph {} --json", params.graph_id);
    exec_and_parse::<GraphState>(&supervisor, cmd)
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

/// Export graph as JSON
#[utoipa::path(
    post,
    path = "/api/v1/graph/export",
    tag = "graph",
    request_body = ExportGraphRequest,
    responses(
        (status = 200, description = "Graph exported", body = ExportGraphResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 500, description = "Failed to export graph", body = ErrorResponse)
    )
)]
pub async fn graph_export(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<ExportGraphRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let cmd = format!(
        "graphctl export --graph {} --format {} --json",
        req.graph_id, req.format
    );
    exec_and_parse::<ExportGraphResponse>(&supervisor, cmd)
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

// ============================================================================
// Event Emission Helpers
// ============================================================================

/// Emit GraphState event after a mutation
async fn emit_graph_state_event(supervisor: &Arc<QemuSupervisor>, graph_id: String) {
    // Fetch current graph state
    let cmd = format!("graphctl state --graph {} --json", graph_id);
    if let Ok(state) = exec_and_parse::<GraphState>(supervisor, cmd).await {
        // Convert REST types to event types
        let operators = state
            .operators
            .into_iter()
            .map(|op| EventGraphOperator {
                id: op.id,
                name: None,
                prio: op.prio.map(|p| p as u32),
                stage: op.stage,
                stats: None, // TODO: Add stats if available
            })
            .collect();

        let channels = state
            .channels
            .into_iter()
            .map(|ch| EventGraphChannel {
                id: ch.id,
                cap: ch.cap,
                depth: ch.depth,
            })
            .collect();

        let event_data = GraphStateData {
            operators,
            channels,
        };

        let event = QemuEvent::GraphState {
            graph_id,
            state: event_data,
            ts: chrono::Utc::now().timestamp_millis(),
        };

        supervisor.broadcast_event(event);
    }
}
