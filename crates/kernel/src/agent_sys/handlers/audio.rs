//! Audio control handlers for AgentSys

use crate::control::CtrlError;
use crate::agent_sys::{AgentId, policy};
use crate::security::agent_policy::{Capability, Resource, PolicyDecision};
use crate::uart;
use crate::trace::metric_kv;

/// Handle AUDIO_PLAY (0x36)
pub fn handle_play(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_audio_play", 1);

    if payload.len() < 4 {
        return Err(CtrlError::BadFrame);
    }

    let track_ref = u32::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3]
    ]);

    let decision = policy().check(
        agent_id,
        Capability::AudioControl,
        &Resource::AudioTrack(track_ref),
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] AUDIO_PLAY: track=");
    uart::print_u32(track_ref);
    uart::print_str("\n");
    uart::print_str("[AUDIO] Playing track (simulated)\n");

    Ok(())
}

/// Handle AUDIO_STOP (0x37)
pub fn handle_stop(agent_id: AgentId, _payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_audio_stop", 1);

    let decision = policy().check(
        agent_id,
        Capability::AudioControl,
        &Resource::NoResource,
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] AUDIO_STOP\n");
    uart::print_str("[AUDIO] Stopped\n");

    Ok(())
}

/// Handle AUDIO_VOLUME (0x38)
pub fn handle_volume(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_audio_volume", 1);

    if payload.len() < 1 {
        return Err(CtrlError::BadFrame);
    }

    let level = payload[0]; // 0-100

    let decision = policy().check(
        agent_id,
        Capability::AudioControl,
        &Resource::NoResource,
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] AUDIO_VOLUME: level=");
    uart::print_u8(level);
    uart::print_str("\n");
    uart::print_str("[AUDIO] Volume set\n");

    Ok(())
}
