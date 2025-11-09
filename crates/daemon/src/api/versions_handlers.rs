//! Version Control API handlers
//!
//! These endpoints wrap Phase 2 adapter version control shell commands

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

/// Adapter version metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VersionMetadata {
    pub training_examples: u32,
    pub training_duration_ms: u64,
    pub final_loss: f64,
    pub accuracy_improvement: f64,
    pub environment_tag: String,
}

/// Adapter version entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdapterVersion {
    pub version_id: u32,
    pub parent_version: Option<u32>,
    pub timestamp: String,
    pub description: String,
    pub metadata: VersionMetadata,
    pub hash: String,
    pub storage_path: String,
    pub tags: Vec<String>,
}

/// Version list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VersionListResponse {
    pub current_version: u32,
    pub versions: Vec<AdapterVersion>,
}

/// Version diff comparison
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VersionDiff {
    pub version_a: u32,
    pub version_b: u32,
    pub accuracy_delta: f64,
    pub param_changes: u32,
    pub time_delta_hours: f64,
}

/// Commit request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct CommitRequest {
    pub description: String,
    pub environment_tag: String,
    pub metadata: serde_json::Value,
}

/// Commit response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CommitResponse {
    pub success: bool,
    pub version_id: u32,
    pub parent_version: u32,
    pub timestamp: String,
}

/// Rollback request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct VersionRollbackRequest {
    pub version_id: u32,
}

/// Rollback response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VersionRollbackResponse {
    pub success: bool,
    pub old_version: u32,
    pub new_version: u32,
    pub timestamp: String,
}

/// Tag request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct TagRequest {
    pub version_id: u32,
    pub tag: String,
}

/// Tag response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TagResponse {
    pub success: bool,
    pub version_id: u32,
    pub tag: String,
    pub timestamp: String,
}

/// Garbage collection request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct GarbageCollectRequest {
    #[serde(default = "default_keep_count")]
    pub keep_count: u32,
}

fn default_keep_count() -> u32 {
    10
}

/// Garbage collection response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GarbageCollectResponse {
    pub success: bool,
    pub removed_count: u32,
    pub kept_count: u32,
    pub timestamp: String,
}

/// Query parameters for version list
#[derive(Debug, Deserialize)]
pub struct VersionListQuery {
    pub limit: Option<u32>,
}

/// Query parameters for version diff
#[derive(Debug, Deserialize)]
pub struct VersionDiffQuery {
    pub v1: u32,
    pub v2: u32,
}

/// Get adapter version history
#[utoipa::path(
    get,
    path = "/api/v1/versions/list",
    params(
        ("limit" = Option<u32>, Query, description = "Max versions to return (default: 10)")
    ),
    responses(
        (status = 200, description = "Version history", body = VersionListResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "versions"
)]
pub async fn get_version_list(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<VersionListQuery>,
) -> Response {
    let (supervisor, _) = &state;
    let limit = params.limit.unwrap_or(10);
    debug!("Getting version list (limit: {})", limit);

    exec_and_parse::<VersionListResponse>(
        supervisor,
        format!("versionctl list --limit {} --json", limit),
    )
    .await
    .map(|response| Json(response).into_response())
    .unwrap_or_else(|r| r)
}

/// Commit current adapter as new version
#[utoipa::path(
    post,
    path = "/api/v1/versions/commit",
    request_body = CommitRequest,
    responses(
        (status = 200, description = "Version committed", body = CommitResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "versions"
)]
pub async fn commit_version(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(req): Json<CommitRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Committing new version: {}", req.description);

    exec_and_parse::<CommitResponse>(
        supervisor,
        format!(
            "versionctl commit -m \"{}\" --env={} --json",
            req.description.replace("\"", "\\\""),
            req.environment_tag
        ),
    )
    .await
    .map(|response| Json(response).into_response())
    .unwrap_or_else(|r| r)
}

/// Rollback to previous version
#[utoipa::path(
    post,
    path = "/api/v1/versions/rollback",
    request_body = VersionRollbackRequest,
    responses(
        (status = 200, description = "Rolled back to version", body = VersionRollbackResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "versions"
)]
pub async fn rollback_version(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(req): Json<VersionRollbackRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Rolling back to version: {}", req.version_id);

    exec_and_parse::<VersionRollbackResponse>(
        supervisor,
        format!("versionctl rollback {} --json", req.version_id),
    )
    .await
    .map(|response| Json(response).into_response())
    .unwrap_or_else(|r| r)
}

/// Compare two adapter versions
#[utoipa::path(
    get,
    path = "/api/v1/versions/diff",
    params(
        ("v1" = u32, Query, description = "First version ID"),
        ("v2" = u32, Query, description = "Second version ID")
    ),
    responses(
        (status = 200, description = "Version comparison", body = VersionDiff),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "versions"
)]
pub async fn get_version_diff(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Query(params): Query<VersionDiffQuery>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Getting version diff: {} vs {}", params.v1, params.v2);

    exec_and_parse::<VersionDiff>(
        supervisor,
        format!("versionctl diff {} {} --json", params.v1, params.v2),
    )
    .await
    .map(|response| Json(response).into_response())
    .unwrap_or_else(|r| r)
}

/// Tag a version
#[utoipa::path(
    post,
    path = "/api/v1/versions/tag",
    request_body = TagRequest,
    responses(
        (status = 200, description = "Version tagged", body = TagResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "versions"
)]
pub async fn tag_version(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(req): Json<TagRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Tagging version {} with tag: {}", req.version_id, req.tag);

    exec_and_parse::<TagResponse>(
        supervisor,
        format!("versionctl tag {} {} --json", req.version_id, req.tag),
    )
    .await
    .map(|response| Json(response).into_response())
    .unwrap_or_else(|r| r)
}

/// Garbage collect old versions
#[utoipa::path(
    post,
    path = "/api/v1/versions/gc",
    request_body = GarbageCollectRequest,
    responses(
        (status = 200, description = "Garbage collection complete", body = GarbageCollectResponse),
        (status = 503, description = "Shell not ready", body = ErrorResponse)
    ),
    tag = "versions"
)]
pub async fn garbage_collect_versions(
    State(state): State<(Arc<QemuSupervisor>, Arc<crate::qemu::ReplayManager>)>,
    Json(req): Json<GarbageCollectRequest>,
) -> Response {
    let (supervisor, _) = &state;
    debug!("Garbage collecting versions (keep: {})", req.keep_count);

    exec_and_parse::<GarbageCollectResponse>(
        supervisor,
        format!("versionctl gc --keep={} --json", req.keep_count),
    )
    .await
    .map(|response| Json(response).into_response())
    .unwrap_or_else(|r| r)
}
