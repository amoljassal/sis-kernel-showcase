//! SIS Kernel Control Daemon (sisctl)
//!
//! Standalone service that:
//! - Launches and supervises QEMU instances
//! - Parses kernel output (metrics, banners, markers)
//! - Exposes REST API and WebSocket events
//!
//! Default bind: 127.0.0.1:8871
//! WebSocket: /events
//! REST API: /api/v1/*

mod api;
mod config;
mod metrics;
mod parser;
mod qemu;
mod tracing_layer;

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Create QEMU supervisor first (needed for tracing layer)
    let supervisor = Arc::new(qemu::QemuSupervisor::new());

    // Initialize tracing with structured fields and WebSocket layer
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .with(tracing_layer::WebSocketLayer::new(Arc::clone(&supervisor)))
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,sisctl=debug")),
        )
        .init();

    info!("Starting SIS Kernel Control Daemon (sisctl)");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Parse bind address from env or use default
    let bind_addr = std::env::var("SISCTL_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8871".to_string())
        .parse::<SocketAddr>()?;

    info!("Binding to {}", bind_addr);

    // Create replay manager
    let replay_manager = Arc::new(qemu::ReplayManager::new());

    // Create API router
    let app = api::create_router(supervisor, replay_manager);

    // Create server
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!("sisctl listening on http://{}", bind_addr);
    info!("WebSocket events: ws://{}/events", bind_addr);
    info!("Swagger UI: http://{}/swagger-ui", bind_addr);

    // Run server
    axum::serve(listener, app).await?;

    Ok(())
}
