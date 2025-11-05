//! API routing

use super::{handlers, shell_handlers, ws};
use crate::qemu::QemuSupervisor;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::qemu_run,
        handlers::qemu_stop,
        handlers::qemu_status,
        shell_handlers::shell_exec,
        shell_handlers::shell_selfcheck,
    ),
    components(
        schemas(
            crate::qemu::QemuConfig,
            crate::qemu::QemuStatus,
            crate::qemu::QemuState,
            crate::qemu::ShellCommandRequest,
            crate::qemu::ShellCommandResponse,
            crate::qemu::SelfCheckResponse,
            crate::qemu::TestResultEntry,
            handlers::ErrorResponse,
            handlers::SuccessResponse,
            handlers::HealthResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "qemu", description = "QEMU control endpoints"),
        (name = "shell", description = "Shell command execution")
    ),
    info(
        title = "SIS Kernel Control Daemon (sisctl)",
        version = "0.1.0",
        description = "REST API for managing SIS kernel QEMU instances",
    )
)]
struct ApiDoc;

/// Create the API router
pub fn create_router(supervisor: Arc<QemuSupervisor>) -> Router {
    // Create OpenAPI documentation
    let openapi = ApiDoc::openapi();

    // Create shell state
    let shell_state = Arc::new(RwLock::new(shell_handlers::ShellState::new()));

    // Shell command router with shell state
    let shell_router = Router::new()
        .route("/api/v1/shell/exec", post(shell_handlers::shell_exec))
        .route(
            "/api/v1/shell/selfcheck",
            post(shell_handlers::shell_selfcheck),
        )
        .with_state(shell_state);

    // Main router with supervisor state
    let main_router = Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // QEMU control endpoints
        .route("/api/v1/qemu/run", post(handlers::qemu_run))
        .route("/api/v1/qemu/stop", post(handlers::qemu_stop))
        .route("/api/v1/qemu/status", get(handlers::qemu_status))
        // WebSocket events
        .route("/events", get(ws::events_handler))
        .with_state(supervisor);

    // Merge routers and add Swagger UI
    Router::new()
        .merge(main_router)
        .merge(shell_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .layer(CorsLayer::permissive())
}
