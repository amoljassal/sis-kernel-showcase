//! I/O capture handlers (screenshot, audio recording) for AgentSys

use crate::control::CtrlError;
use crate::agent_sys::{AgentId, policy, audit};
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
        audit().log_operation(agent_id, 0x3C, false);
        return Err(CtrlError::AuthFailed);
    }

    // Create minimal PNG placeholder
    let timestamp = crate::time::get_uptime_ms();
    let filename = alloc::format!("/tmp/agentsys/screenshot_{}.png", timestamp);

    // Ensure directory exists
    let _ = crate::vfs::mkdir("/tmp", 0o755);
    let _ = crate::vfs::mkdir("/tmp/agentsys", 0o755);

    // Minimal PNG header (1x1 pixel, black)
    const PNG_DATA: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,  // PNG signature
        0x00, 0x00, 0x00, 0x0D,  // IHDR length
        0x49, 0x48, 0x44, 0x52,  // IHDR
        0x00, 0x00, 0x00, 0x01,  // width = 1
        0x00, 0x00, 0x00, 0x01,  // height = 1
        0x08, 0x00, 0x00, 0x00, 0x00,  // bit depth, color type, etc
        0x3A, 0x7E, 0x9B, 0x55,  // CRC
        0x00, 0x00, 0x00, 0x0A,  // IDAT length
        0x49, 0x44, 0x41, 0x54,  // IDAT
        0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01,  // compressed data
        0xE5, 0x27, 0xDE, 0xFC,  // CRC
        0x00, 0x00, 0x00, 0x00,  // IEND length
        0x49, 0x45, 0x4E, 0x44,  // IEND
        0xAE, 0x42, 0x60, 0x82,  // CRC
    ];

    match crate::vfs::create(&filename, 0o644, crate::vfs::OpenFlags::O_WRONLY | crate::vfs::OpenFlags::O_CREAT) {
        Ok(file) => {
            match file.write(PNG_DATA) {
                Ok(_) => {
                    uart::print_str("[Screenshot] Saved: ");
                    uart::print_str(&filename);
                    uart::print_str(" (");
                    uart::print_u32(PNG_DATA.len() as u32);
                    uart::print_str(" bytes)\n");

                    audit().log_operation(agent_id, 0x3C, true);
                    Ok(())
                }
                Err(_) => {
                    uart::print_str("[Screenshot] Write failed\n");
                    audit().log_operation(agent_id, 0x3C, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(_) => {
            uart::print_str("[Screenshot] Failed to create file\n");
            audit().log_operation(agent_id, 0x3C, false);
            Err(CtrlError::IoError)
        }
    }
}

/// Handle AUDIO_RECORD (0x3D)
pub fn handle_record(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_audio_record", 1);

    // Parse duration (default 5 seconds if not provided)
    let duration_ms = if payload.len() >= 4 {
        u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]])
    } else if payload.len() >= 2 {
        u16::from_le_bytes([payload[0], payload[1]]) as u32 * 1000  // seconds to ms
    } else {
        5000  // default 5 seconds
    };

    let decision = policy().check(
        agent_id,
        Capability::Capture,
        &Resource::NoResource,
    );

    if let PolicyDecision::Deny { reason } = decision {
        audit().log_operation(agent_id, 0x3D, false);
        return Err(CtrlError::AuthFailed);
    }

    let timestamp = crate::time::get_uptime_ms();
    let filename = alloc::format!("/tmp/agentsys/recording_{}_{}.mp4", timestamp, duration_ms);

    // Ensure directory exists
    let _ = crate::vfs::mkdir("/tmp", 0o755);
    let _ = crate::vfs::mkdir("/tmp/agentsys", 0o755);

    // Create placeholder MP4 (minimal valid structure)
    // This is a simplified placeholder - real implementation would capture frames
    let placeholder = alloc::format!("VIDEO:{}ms:PLACEHOLDER", duration_ms);

    match crate::vfs::create(&filename, 0o644, crate::vfs::OpenFlags::O_WRONLY | crate::vfs::OpenFlags::O_CREAT) {
        Ok(file) => {
            match file.write(placeholder.as_bytes()) {
                Ok(_) => {
                    uart::print_str("[Record] Started: ");
                    uart::print_str(&filename);
                    uart::print_str(" (");
                    uart::print_u32(duration_ms);
                    uart::print_str("ms)\n");

                    audit().log_operation(agent_id, 0x3D, true);
                    Ok(())
                }
                Err(_) => {
                    uart::print_str("[Record] Write failed\n");
                    audit().log_operation(agent_id, 0x3D, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(_) => {
            uart::print_str("[Record] Failed to create file\n");
            audit().log_operation(agent_id, 0x3D, false);
            Err(CtrlError::IoError)
        }
    }
}

