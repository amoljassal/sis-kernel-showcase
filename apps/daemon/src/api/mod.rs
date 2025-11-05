//! REST API and WebSocket handlers

pub mod autonomy_handlers;
pub mod graph_handlers;
pub mod handlers;
pub mod llm_handlers;
pub mod logs_handlers;
pub mod memory_handlers;
pub mod metrics_handlers;
pub mod middleware;
pub mod replay_handlers;
pub mod routes;
pub mod sched_handlers;
pub mod shell_handlers;
pub mod ws;

pub use routes::create_router;
