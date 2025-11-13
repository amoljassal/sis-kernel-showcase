//! Filesystem handlers for AgentSys

use crate::control::CtrlError;
use crate::agent_sys::{protocol, AgentId, policy, audit};
use crate::security::agent_policy::{Capability, Resource, PolicyDecision};
use crate::uart;
use crate::trace::metric_kv;

/// Handle FS_LIST (0x30): List directory contents
pub fn handle_list(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_list", 1);

    // Parse path
    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;

    // Check capability
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if let PolicyDecision::Deny { reason } = decision {
        uart::print_str("[AgentSys] FS_LIST denied: ");
        uart::print_str(reason);
        uart::print_str("\n");
        return Err(CtrlError::AuthFailed);
    }

    // Execute operation
    // TODO: Actual VFS integration - for now, simulate
    uart::print_str("[AgentSys] FS_LIST: ");
    uart::print_str(path);
    uart::print_str("\n");

    // Simulate directory listing
    if path == "/tmp/" || path == "/tmp" {
        uart::print_str("[FS] Entries: files/, docs/, test.txt\n");
    } else {
        uart::print_str("[FS] Entries: (empty)\n");
    }

    Ok(())
}

/// Handle FS_READ (0x31): Read file contents
pub fn handle_read(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_read", 1);

    // Parse path
    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;
    let path_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;

    // Parse offset + length
    let (offset, len) = protocol::parse_offset_len(payload, path_len)
        .map_err(|_| CtrlError::BadFrame)?;

    // Check capability
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if let PolicyDecision::Deny { reason } = decision {
        uart::print_str("[AgentSys] FS_READ denied: ");
        uart::print_str(reason);
        uart::print_str("\n");
        return Err(CtrlError::AuthFailed);
    }

    // Execute operation
    uart::print_str("[AgentSys] FS_READ: ");
    uart::print_str(path);
    uart::print_str(" offset=");
    uart::print_u64(offset);
    uart::print_str(" len=");
    uart::print_u32(len);
    uart::print_str("\n");

    // TODO: Actual VFS read
    uart::print_str("[FS] Data: (simulated read)\n");

    Ok(())
}

/// Handle FS_WRITE (0x32): Write file contents
pub fn handle_write(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_write", 1);

    // Parse path
    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;
    let path_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;

    // Parse offset + data length
    let (offset, data_len) = protocol::parse_offset_len(payload, path_len)
        .map_err(|_| CtrlError::BadFrame)?;

    // Check capability
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if let PolicyDecision::Deny { reason } = decision {
        uart::print_str("[AgentSys] FS_WRITE denied: ");
        uart::print_str(reason);
        uart::print_str("\n");
        return Err(CtrlError::AuthFailed);
    }

    // Check file size limit
    let size_decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FileSize(data_len as usize),
    );

    if let PolicyDecision::Deny { reason } = size_decision {
        uart::print_str("[AgentSys] FS_WRITE denied: ");
        uart::print_str(reason);
        uart::print_str("\n");
        return Err(CtrlError::AuthFailed);
    }

    // Execute operation
    uart::print_str("[AgentSys] FS_WRITE: ");
    uart::print_str(path);
    uart::print_str(" offset=");
    uart::print_u64(offset);
    uart::print_str(" len=");
    uart::print_u32(data_len);
    uart::print_str("\n");

    // TODO: Actual VFS write
    uart::print_str("[FS] Written: ");
    uart::print_u32(data_len);
    uart::print_str(" bytes\n");

    Ok(())
}

/// Handle FS_STAT (0x33): Get file metadata
pub fn handle_stat(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_stat", 1);

    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;

    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] FS_STAT: ");
    uart::print_str(path);
    uart::print_str("\n");
    uart::print_str("[FS] Stat: size=1024 mode=0644\n");

    Ok(())
}

/// Handle FS_CREATE (0x34): Create file or directory
pub fn handle_create(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_create", 1);

    if payload.len() < 3 {
        return Err(CtrlError::BadFrame);
    }

    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;
    let path_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    let kind = payload[2 + path_len]; // 0=file, 1=dir

    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    let kind_str = if kind == 0 { "file" } else { "directory" };
    uart::print_str("[AgentSys] FS_CREATE: ");
    uart::print_str(path);
    uart::print_str(" kind=");
    uart::print_str(kind_str);
    uart::print_str("\n");
    uart::print_str("[FS] Created\n");

    Ok(())
}

/// Handle FS_DELETE (0x35): Delete file or directory
pub fn handle_delete(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_delete", 1);

    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;

    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] FS_DELETE: ");
    uart::print_str(path);
    uart::print_str("\n");
    uart::print_str("[FS] Deleted\n");

    Ok(())
}
