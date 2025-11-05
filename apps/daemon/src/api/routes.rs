//! API routing

use super::{handlers, ws};
use crate::qemu::QemuSupervisor;
use axum::{
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
        handlers::qemu_run,
        handlers::qemu_stop,
        handlers::qemu_status,
    ),
    components(
        schemas(
            crate::qemu::QemuConfig,
            crate::qemu::QemuStatus,
            crate::qemu::QemuState,
            handlers::ErrorResponse,
            handlers::SuccessResponse,
            handlers::HealthResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "qemu", description = "QEMU control endpoints")
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

    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // QEMU control endpoints
        .route("/api/v1/qemu/run", post(handlers::qemu_run))
        .route("/api/v1/qemu/stop", post(handlers::qemu_stop))
        .route("/api/v1/qemu/status", get(handlers::qemu_status))
        // WebSocket events
        .route("/events", get(ws::events_handler))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        // State
        .with_state(supervisor)
        // CORS for local development
        .layer(CorsLayer::permissive())
}
