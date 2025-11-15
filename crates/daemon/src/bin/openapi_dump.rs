//! OpenAPI schema generator
//!
//! Generates the OpenAPI JSON spec without starting the daemon or touching external networks.
//! This allows us to freeze the API schema in the repository for CI and client generation.

use utoipa::OpenApi;

// Re-export the ApiDoc from routes
#[derive(OpenApi)]
#[openapi(
    paths(
        sisctl::api::handlers::health,
        sisctl::api::handlers::get_config,
        sisctl::api::handlers::qemu_run,
        sisctl::api::handlers::qemu_stop,
        sisctl::api::handlers::qemu_status,
        sisctl::api::shell_handlers::shell_exec,
        sisctl::api::shell_handlers::shell_selfcheck,
        sisctl::api::shell_handlers::shell_selfcheck_cancel,
        sisctl::api::replay_handlers::replay_start,
        sisctl::api::replay_handlers::replay_stop,
        sisctl::api::replay_handlers::replay_status,
        sisctl::api::metrics_handlers::list_streams,
        sisctl::api::metrics_handlers::query_series,
        sisctl::api::autonomy_handlers::autonomy_on,
        sisctl::api::autonomy_handlers::autonomy_off,
        sisctl::api::autonomy_handlers::autonomy_reset,
        sisctl::api::autonomy_handlers::autonomy_set_interval,
        sisctl::api::autonomy_handlers::autonomy_set_threshold,
        sisctl::api::autonomy_handlers::autonomy_status,
        sisctl::api::autonomy_handlers::autonomy_audit,
        sisctl::api::autonomy_handlers::autonomy_explain,
        sisctl::api::autonomy_handlers::autonomy_preview,
        sisctl::api::autonomy_handlers::autonomy_whatif,
        sisctl::api::memory_handlers::mem_get_approvals,
        sisctl::api::memory_handlers::mem_approval_toggle,
        sisctl::api::memory_handlers::mem_approve,
        sisctl::api::memory_handlers::mem_reject,
    ),
    components(
        schemas(
            sisctl::config::DaemonConfig,
            sisctl::qemu::QemuConfig,
            sisctl::qemu::QemuStatus,
            sisctl::qemu::QemuState,
            sisctl::qemu::ReplayStatus,
            sisctl::qemu::ReplayState,
            sisctl::qemu::ShellCommandRequest,
            sisctl::qemu::ShellCommandResponse,
            sisctl::qemu::SelfCheckResponse,
            sisctl::qemu::TestResultEntry,
            sisctl::metrics::MetricPoint,
            sisctl::metrics::SeriesStats,
            sisctl::metrics::store::SeriesMetadata,
            sisctl::metrics::store::QueryResult,
            sisctl::api::handlers::ErrorResponse,
            sisctl::api::handlers::SuccessResponse,
            sisctl::api::handlers::HealthResponse,
            sisctl::api::replay_handlers::ReplayRequest,
            sisctl::api::replay_handlers::ReplayResponse,
            sisctl::api::autonomy_handlers::AutonomyStatus,
            sisctl::api::autonomy_handlers::AutonomyDecision,
            sisctl::api::autonomy_handlers::ExplainResponse,
            sisctl::api::autonomy_handlers::AttentionWeight,
            sisctl::api::autonomy_handlers::PreviewRequest,
            sisctl::api::autonomy_handlers::PreviewResponse,
            sisctl::api::autonomy_handlers::WhatIfRequest,
            sisctl::api::autonomy_handlers::WhatIfResponse,
            sisctl::api::memory_handlers::MemoryApprovalStatus,
            sisctl::api::memory_handlers::PendingOperation,
            sisctl::api::memory_handlers::ApproveRequest,
            sisctl::api::memory_handlers::RejectRequest,
            sisctl::api::memory_handlers::ApprovalToggleRequest,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "config", description = "Configuration endpoints"),
        (name = "qemu", description = "QEMU control endpoints"),
        (name = "shell", description = "Shell command execution"),
        (name = "replay", description = "Replay log files for offline testing"),
        (name = "metrics", description = "Metrics collection and querying"),
        (name = "autonomy", description = "Autonomy control and decision management"),
        (name = "memory", description = "Memory approval management")
    ),
    info(
        title = "SIS Kernel Control Daemon (sisctl)",
        version = "0.1.0",
        description = "REST API for managing SIS kernel QEMU instances",
    )
)]
struct ApiDoc;

fn main() {
    // Generate OpenAPI spec
    let openapi = ApiDoc::openapi();

    // Serialize to JSON with pretty formatting
    let json = serde_json::to_string_pretty(&openapi)
        .expect("Failed to serialize OpenAPI spec");

    // Determine output path (repo root)
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "openapi.json".to_string());

    // Write to file
    std::fs::write(&output_path, json)
        .expect(&format!("Failed to write OpenAPI spec to {}", output_path));

    eprintln!("✓ OpenAPI spec written to {}", output_path);
}
