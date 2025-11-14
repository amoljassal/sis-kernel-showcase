//! I/O capture handlers (screenshot, audio recording) for AgentSys

use crate::control::CtrlError;
use crate::agent_sys::{AgentId, policy};
use crate::security::agent_policy::{Capability, Resource, PolicyDecision};
use crate::uart;
use crate::trace::metric_kv;

/// Handle SCREENSHOT (0x3C)
pub fn handle_screenshot(agent_id: AgentId, _payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_screenshot", 1);

    let decision = policy().check(
        agent_id,
        Capability::Screenshot,
        &Resource::NoResource,
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] SCREENSHOT\n");
    uart::print_str("[IO] Screenshot captured (simulated)\n");

    Ok(())
}

/// Handle AUDIO_RECORD (0x3D)
pub fn handle_record(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_audio_record", 1);

    if payload.len() < 2 {
        return Err(CtrlError::BadFrame);
    }

    let duration_secs = u16::from_le_bytes([payload[0], payload[1]]);

    let decision = policy().check(
        agent_id,
        Capability::Capture,
        &Resource::NoResource,
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] AUDIO_RECORD: duration=");
    uart::print_u16(duration_secs);
    uart::print_str("s\n");
    uart::print_str("[IO] Recording started (simulated)\n");

    Ok(())
}
