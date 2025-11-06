//! LLM control REST endpoints (feature-gated)
//!
//! Wraps `llmctl` shell commands with REST API.

use super::handlers::{exec_and_parse, ErrorResponse};
use crate::qemu::{QemuSupervisor, ReplayManager};
use axum::{
    response::{IntoResponse, Response},
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoadModelRequest {
    #[serde(rename = "modelId")]
    pub model_id: String,
    #[serde(rename = "wcetCycles", skip_serializing_if = "Option::is_none")]
    pub wcet_cycles: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocab: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoadModelResponse {
    pub loaded: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InferRequest {
    pub text: String,
    #[serde(rename = "maxTokens", skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InferResponse {
    #[serde(rename = "requestId")]
    pub request_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuditEntry {
    pub id: String,
    #[serde(rename = "modelId")]
    pub model_id: String,
    pub tokens: u32,
    pub done: bool,
    pub ts: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LlmStatus {
    pub budget: u64,
    #[serde(rename = "wcetCycles")]
    pub wcet_cycles: u64,
    #[serde(rename = "periodNs")]
    pub period_ns: u64,
    #[serde(rename = "maxTokensPerPeriod")]
    pub max_tokens_per_period: u32,
    #[serde(rename = "queueDepth")]
    pub queue_depth: u32,
    #[serde(rename = "lastInferUs")]
    pub last_infer_us: u64,
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Load LLM model
#[utoipa::path(
    post,
    path = "/api/v1/llm/load",
    tag = "llm",
    request_body = LoadModelRequest,
    responses(
        (status = 200, description = "Model loaded", body = LoadModelResponse),
        (status = 400, description = "Invalid model", body = ErrorResponse),
        (status = 500, description = "Failed to load model", body = ErrorResponse)
    )
)]
pub async fn llm_load(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<LoadModelRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let mut cmd = format!("llmctl load --model {}", req.model_id);

    if let Some(wcet) = req.wcet_cycles {
        cmd.push_str(&format!(" --wcet {}", wcet));
    }
    if let Some(ctx) = req.ctx {
        cmd.push_str(&format!(" --ctx {}", ctx));
    }
    if let Some(vocab) = req.vocab {
        cmd.push_str(&format!(" --vocab {}", vocab));
    }
    if let Some(quant) = &req.quant {
        cmd.push_str(&format!(" --quant {}", quant));
    }
    if let Some(hash) = &req.hash {
        cmd.push_str(&format!(" --hash {}", hash));
    }
    if let Some(sig) = &req.sig {
        cmd.push_str(&format!(" --sig {}", sig));
    }

    cmd.push_str(" --json");

    exec_and_parse::<LoadModelResponse>(&supervisor, cmd)
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

/// Submit inference request
#[utoipa::path(
    post,
    path = "/api/v1/llm/infer",
    tag = "llm",
    request_body = InferRequest,
    responses(
        (status = 200, description = "Inference started", body = InferResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Model not loaded", body = ErrorResponse),
        (status = 408, description = "Timeout", body = ErrorResponse),
        (status = 500, description = "Failed to start inference", body = ErrorResponse)
    )
)]
pub async fn llm_infer(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
    Json(req): Json<InferRequest>,
) -> Response {
    let (supervisor, _) = &state;
    let mut cmd = format!("llmctl infer --text '{}'", req.text.replace('\'', "'\\''"));

    if let Some(max_tokens) = req.max_tokens {
        cmd.push_str(&format!(" --max-tokens {}", max_tokens));
    }

    cmd.push_str(" --json");

    // TODO: Implement streaming LlmTokens event emission
    // The actual llmctl wrapper should read output line-by-line and emit:
    // - QemuEvent::LlmTokens { request_id, chunk, done: false, ts } for each token chunk
    // - QemuEvent::LlmTokens { request_id, chunk: "", done: true, ts } when complete

    exec_and_parse::<InferResponse>(&supervisor, cmd)
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

/// Get inference audit log
#[utoipa::path(
    get,
    path = "/api/v1/llm/audit",
    tag = "llm",
    responses(
        (status = 200, description = "Audit log entries", body = Vec<AuditEntry>),
        (status = 500, description = "Failed to get audit log", body = ErrorResponse)
    )
)]
pub async fn llm_audit(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    exec_and_parse::<Vec<AuditEntry>>(&supervisor, "llmctl audit --json".to_string())
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}

/// Get LLM status
#[utoipa::path(
    get,
    path = "/api/v1/llm/status",
    tag = "llm",
    responses(
        (status = 200, description = "LLM status", body = LlmStatus),
        (status = 500, description = "Failed to get LLM status", body = ErrorResponse)
    )
)]
pub async fn llm_status(
    State(state): State<(Arc<QemuSupervisor>, Arc<ReplayManager>)>,
) -> Response {
    let (supervisor, _) = &state;
    exec_and_parse::<LlmStatus>(&supervisor, "llmctl status --json".to_string())
        .await
        .map(|resp| Json(resp).into_response())
        .unwrap_or_else(|r| r)
}
