//! API routing

use super::{handlers, metrics_handlers, middleware, replay_handlers, shell_handlers, ws};
use crate::qemu::{QemuSupervisor, ReplayManager};
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::get_config,
        handlers::qemu_run,
        handlers::qemu_stop,
        handlers::qemu_status,
        shell_handlers::shell_exec,
        shell_handlers::shell_selfcheck,
        shell_handlers::shell_selfcheck_cancel,
        replay_handlers::replay_start,
        replay_handlers::replay_stop,
        replay_handlers::replay_status,
        metrics_handlers::list_streams,
        metrics_handlers::query_series,
    ),
    components(
        schemas(
            crate::config::DaemonConfig,
            crate::qemu::QemuConfig,
            crate::qemu::QemuStatus,
            crate::qemu::QemuState,
            crate::qemu::ReplayStatus,
            crate::qemu::ReplayState,
            crate::qemu::ShellCommandRequest,
            crate::qemu::ShellCommandResponse,
            crate::qemu::SelfCheckResponse,
            crate::qemu::TestResultEntry,
            crate::metrics::MetricPoint,
            crate::metrics::SeriesStats,
            crate::metrics::store::SeriesMetadata,
            crate::metrics::store::QueryResult,
            handlers::ErrorResponse,
            handlers::SuccessResponse,
            handlers::HealthResponse,
            replay_handlers::ReplayRequest,
            replay_handlers::ReplayResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "config", description = "Configuration endpoints"),
        (name = "qemu", description = "QEMU control endpoints"),
        (name = "shell", description = "Shell command execution"),
        (name = "replay", description = "Replay log files for offline testing"),
        (name = "metrics", description = "Metrics collection and querying")
    ),
    info(
        title = "SIS Kernel Control Daemon (sisctl)",
        version = "0.1.0",
        description = "REST API for managing SIS kernel QEMU instances",
    )
)]
struct ApiDoc;

/// Create the API router
pub fn create_router(supervisor: Arc<QemuSupervisor>, replay_manager: Arc<ReplayManager>) -> Router {
    // Create OpenAPI documentation
    let openapi = ApiDoc::openapi();

    // Shared state tuple for all handlers
    let state = (supervisor, replay_manager);

    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Configuration
        .route("/api/v1/config", get(handlers::get_config))
        // QEMU control endpoints
        .route("/api/v1/qemu/run", post(handlers::qemu_run))
        .route("/api/v1/qemu/stop", post(handlers::qemu_stop))
        .route("/api/v1/qemu/status", get(handlers::qemu_status))
        // Shell command endpoints
        .route("/api/v1/shell/exec", post(shell_handlers::shell_exec))
        .route(
            "/api/v1/shell/selfcheck",
            post(shell_handlers::shell_selfcheck),
        )
        .route(
            "/api/v1/shell/selfcheck/cancel",
            post(shell_handlers::shell_selfcheck_cancel),
        )
        // Replay endpoints for offline testing
        .route("/api/v1/replay", post(replay_handlers::replay_start))
        .route("/api/v1/replay/stop", post(replay_handlers::replay_stop))
        .route("/api/v1/replay/status", get(replay_handlers::replay_status))
        // Metrics endpoints
        .route("/api/v1/metrics/streams", get(metrics_handlers::list_streams))
        .route("/api/v1/metrics/query", get(metrics_handlers::query_series))
        // WebSocket events
        .route("/events", get(ws::events_handler))
        // State
        .with_state(state)
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        // Request ID middleware
        .layer(axum_middleware::from_fn(middleware::request_id_middleware))
        // CORS for local development
        .layer(CorsLayer::permissive())
}
