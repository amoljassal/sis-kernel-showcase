//! REST API and WebSocket handlers

pub mod handlers;
pub mod middleware;
pub mod replay_handlers;
pub mod routes;
pub mod shell_handlers;
pub mod ws;

pub use routes::create_router;
