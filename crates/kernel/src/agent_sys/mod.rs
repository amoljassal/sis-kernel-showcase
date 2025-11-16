//! AgentSys: Capability-based system call layer for user agents
//!
//! This module provides a secure, capability-gated interface for LLM-driven
//! agents to access kernel resources (files, audio, docs, etc.).
//!
//! Architecture:
//! - Messages use compact TLV encoding
//! - All operations checked against agent capabilities
//! - Full audit trail for security compliance
//! - Synchronous execution model (async in future phases)
//!
//! # Agent Supervision Module
//!
//! The supervisor submodule provides comprehensive lifecycle management,
//! fault detection, and recovery for all agents. See `supervisor` module
//! documentation for details.

use crate::control::CtrlError;
use crate::security::agent_policy::{PolicyEngine, Capability};
use crate::security::agent_audit::AuditLogger;
use crate::trace::metric_kv;

pub mod protocol;
pub mod handlers;
pub mod supervisor;

#[cfg(feature = "agentsys")]
pub mod cloud_gateway;

// Re-export for convenience
pub use protocol::*;
pub use crate::security::agent_policy::AgentId;

/// Global policy engine instance (static for Phase 1)
static mut POLICY_ENGINE: Option<PolicyEngine> = None;

/// Global audit logger
static mut AUDIT_LOGGER: Option<AuditLogger> = None;

/// Initialize AgentSys subsystem (call from kernel main)
pub fn init() {
    unsafe {
        POLICY_ENGINE = Some(PolicyEngine::new_default());
        AUDIT_LOGGER = Some(AuditLogger::new());
    }
    crate::uart::print_str("[AgentSys] Initialized (sync mode)\n");

    // Initialize Agent Supervision Module
    supervisor::init();

    // Initialize Cloud Gateway (if feature enabled)
    #[cfg(feature = "agentsys")]
    cloud_gateway::init();
}

/// Main dispatcher for AgentSys control frames
pub fn handle_frame(cmd: u8, token: u64, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_calls_total", 1);

    // Extract agent ID from token (upper 16 bits)
    let agent_id: AgentId = ((token >> 48) & 0xFFFF) as u32;

    // Dispatch to handler based on opcode (adjusted to 0x30-0x3F range)
    let result = match cmd {
        0x30 => handlers::fs::handle_list(agent_id, payload),
        0x31 => handlers::fs::handle_read(agent_id, payload),
        0x32 => handlers::fs::handle_write(agent_id, payload),
        0x33 => handlers::fs::handle_stat(agent_id, payload),
        0x34 => handlers::fs::handle_create(agent_id, payload),
        0x35 => handlers::fs::handle_delete(agent_id, payload),
        0x36 => handlers::audio::handle_play(agent_id, payload),
        0x37 => handlers::audio::handle_stop(agent_id, payload),
        0x38 => handlers::audio::handle_volume(agent_id, payload),
        0x39 => handlers::docs::handle_new(agent_id, payload),
        0x3A => handlers::docs::handle_edit(agent_id, payload),
        0x3B => handlers::docs::handle_save(agent_id, payload),
        0x3C => handlers::io::handle_screenshot(agent_id, payload),
        0x3D => handlers::io::handle_record(agent_id, payload),
        _ => Err(CtrlError::Unsupported),
    };

    // Audit result
    if let Some(logger) = unsafe { AUDIT_LOGGER.as_mut() } {
        logger.log_operation(agent_id, cmd, result.is_ok());
    }

    result
}

/// Get policy engine reference (for use by handlers)
pub(crate) fn policy() -> &'static PolicyEngine {
    unsafe { POLICY_ENGINE.as_ref().expect("AgentSys not initialized") }
}

/// Get audit logger reference (for use by handlers)
pub(crate) fn audit() -> &'static mut AuditLogger {
    unsafe { AUDIT_LOGGER.as_mut().expect("AgentSys not initialized") }
}
