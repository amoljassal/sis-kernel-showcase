//! Cloud Gateway for Multi-Provider LLM Routing
//!
//! This module provides kernel-resident cloud API routing for agent LLM requests.
//! It implements multi-provider support with intelligent fallback, rate limiting,
//! and comprehensive telemetry.
//!
//! # Architecture
//!
//! ```
//! Agent → syscall(LLM_REQUEST) → CloudGateway → [Rate Limit Check]
//!                                      ↓
//!                              [Provider Selection]
//!                                      ↓
//!                         Primary Backend (e.g., Claude)
//!                                      ↓
//!                              [Success / Failure]
//!                                      ↓
//!                    [Fallback Chain if failure]
//!                                      ↓
//!                         Response → Agent
//! ```
//!
//! # Design Principles
//!
//! 1. **Multi-Provider**: Support Claude, GPT-4, Gemini, local fallback
//! 2. **Intelligent Fallback**: Automatic failover on errors/timeouts
//! 3. **Rate Limiting**: Per-agent token bucket rate limiting
//! 4. **Observable**: Comprehensive metrics and audit logging
//! 5. **Secure**: Capability-based access control
//! 6. **Efficient**: Minimize syscall overhead with batching
//!
//! # Usage
//!
//! ```rust
//! // From agent code (userspace)
//! let req = LLMRequest {
//!     agent_id: current_agent_id(),
//!     prompt: "Analyze this file...".to_string(),
//!     max_tokens: 1000,
//!     temperature: 0.7,
//! };
//!
//! // Make syscall
//! let response = sys_llm_request(&req)?;
//! ```

pub mod types;
pub mod backend;
pub mod rate_limit;
pub mod fallback;
pub mod gateway;

#[cfg(test)]
mod tests;

pub use types::*;
pub use backend::{CloudBackend, BackendError};
pub use rate_limit::RateLimiter;
pub use fallback::{FallbackPolicy, FallbackChain};
pub use gateway::CloudGateway;

use spin::Mutex;

/// Global Cloud Gateway instance
pub static CLOUD_GATEWAY: Mutex<Option<CloudGateway>> = Mutex::new(None);

/// Initialize the Cloud Gateway
///
/// This must be called during kernel initialization, after networking is available.
pub fn init() {
    *CLOUD_GATEWAY.lock() = Some(CloudGateway::new());
    crate::uart::print_str("[ASM] Cloud Gateway initialized\n");
}

/// Check if Cloud Gateway is initialized
pub fn is_initialized() -> bool {
    CLOUD_GATEWAY.lock().is_some()
}
