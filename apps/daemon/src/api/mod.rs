//! REST API and WebSocket handlers

pub mod handlers;
pub mod routes;
pub mod ws;

pub use routes::create_router;
