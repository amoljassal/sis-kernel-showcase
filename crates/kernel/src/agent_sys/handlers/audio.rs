//! Audio control handlers for AgentSys

use crate::control::CtrlError;
use crate::agent_sys::{AgentId, policy, audit};
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
        audit().log_operation(agent_id, 0x36, false);
        return Err(CtrlError::AuthFailed);
    }

    // Create audio placeholder WAV file
    let filename = alloc::format!("/tmp/agentsys/audio_track_{}.wav", track_ref);

    // Ensure directory exists
    let _ = crate::vfs::mkdir("/tmp", 0o755);
    let _ = crate::vfs::mkdir("/tmp/agentsys", 0o755);

    // Minimal WAV header (1 second, 8kHz, mono, 8-bit)
    let mut wav_data = alloc::vec::Vec::new();
    wav_data.extend_from_slice(b"RIFF");  // ChunkID
    wav_data.extend_from_slice(&44u32.to_le_bytes());  // ChunkSize
    wav_data.extend_from_slice(b"WAVE");  // Format
    wav_data.extend_from_slice(b"fmt ");  // Subchunk1ID
    wav_data.extend_from_slice(&16u32.to_le_bytes());  // Subchunk1Size
    wav_data.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (PCM)
    wav_data.extend_from_slice(&1u16.to_le_bytes());  // NumChannels
    wav_data.extend_from_slice(&8000u32.to_le_bytes());  // SampleRate
    wav_data.extend_from_slice(&8000u32.to_le_bytes());  // ByteRate
    wav_data.extend_from_slice(&1u16.to_le_bytes());  // BlockAlign
    wav_data.extend_from_slice(&8u16.to_le_bytes());  // BitsPerSample
    wav_data.extend_from_slice(b"data");  // Subchunk2ID
    wav_data.extend_from_slice(&8u32.to_le_bytes());  // Subchunk2Size

    // Add 8 samples of silence
    for _ in 0..8 {
        wav_data.push(128);  // 8-bit PCM center
    }

    match crate::vfs::create(&filename, 0o644, crate::vfs::OpenFlags::O_WRONLY | crate::vfs::OpenFlags::O_CREAT) {
        Ok(file) => {
            match file.write(&wav_data) {
                Ok(_) => {
                    uart::print_str("[Audio] Playing track ");
                    uart::print_u32(track_ref);
                    uart::print_str(" -> ");
                    uart::print_str(&filename);
                    uart::print_str("\n");

                    audit().log_operation(agent_id, 0x36, true);
                    Ok(())
                }
                Err(_) => {
                    uart::print_str("[Audio] Write failed\n");
                    audit().log_operation(agent_id, 0x36, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(_) => {
            uart::print_str("[Audio] Failed to create file\n");
            audit().log_operation(agent_id, 0x36, false);
            Err(CtrlError::IoError)
        }
    }
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
