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
        audit().log_operation(agent_id, 0x30, false);
        return Err(CtrlError::AuthFailed);
    }

    // REAL VFS OPERATION
    match crate::vfs::open(path, crate::vfs::OpenFlags::O_RDONLY | crate::vfs::OpenFlags::O_DIRECTORY) {
        Ok(file) => {
            match file.readdir() {
                Ok(entries) => {
                    uart::print_str("[FS] Entries: ");
                    for (i, entry) in entries.iter().enumerate() {
                        if i > 0 { uart::print_str(", "); }
                        uart::print_str(&entry.name);
                        if entry.itype == crate::vfs::InodeType::Directory {
                            uart::print_str("/");
                        }
                    }
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x30, true);
                    Ok(())
                }
                Err(e) => {
                    uart::print_str("[FS] Error: ");
                    uart::print_str(e.description());
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x30, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(e) => {
            uart::print_str("[FS] Open error: ");
            uart::print_str(e.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x30, false);
            Err(CtrlError::IoError)
        }
    }
}

/// Handle FS_READ (0x31): Read file contents
pub fn handle_read(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_read", 1);

    // Parse path
    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;
    let path_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;

    // Parse offset + length (if present, otherwise read from beginning)
    let (offset, len) = if payload.len() >= 2 + path_len + 12 {
        protocol::parse_offset_len(payload, path_len)
            .map_err(|_| CtrlError::BadFrame)?
    } else {
        (0, 512) // Default: read first 512 bytes
    };

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
        audit().log_operation(agent_id, 0x31, false);
        return Err(CtrlError::AuthFailed);
    }

    // REAL VFS READ
    match crate::vfs::open(path, crate::vfs::OpenFlags::O_RDONLY) {
        Ok(file) => {
            // Seek to offset if needed
            if offset > 0 {
                if let Err(e) = file.lseek(offset as i64, 0) {
                    uart::print_str("[FS] Seek error: ");
                    uart::print_str(e.description());
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x31, false);
                    return Err(CtrlError::IoError);
                }
            }

            // Read data
            let mut buffer = alloc::vec![0u8; core::cmp::min(len as usize, 512)];
            match file.read(&mut buffer) {
                Ok(bytes_read) => {
                    uart::print_str("[FS] Read ");
                    uart::print_u32(bytes_read as u32);
                    uart::print_str(" bytes\n");

                    // Print first 64 chars as preview
                    uart::print_str("[FS] Preview: ");
                    for i in 0..core::cmp::min(64, bytes_read) {
                        let c = buffer[i];
                        if c >= 32 && c < 127 {
                            unsafe { uart::write_byte(c); }
                        } else {
                            uart::print_str(".");
                        }
                    }
                    uart::print_str("\n");

                    audit().log_operation(agent_id, 0x31, true);
                    Ok(())
                }
                Err(e) => {
                    uart::print_str("[FS] Read error: ");
                    uart::print_str(e.description());
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x31, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(e) => {
            uart::print_str("[FS] Open error: ");
            uart::print_str(e.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x31, false);
            Err(CtrlError::IoError)
        }
    }
}

/// Handle FS_WRITE (0x32): Write file contents
pub fn handle_write(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_write", 1);

    // Parse path
    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;
    let path_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;

    // Parse offset + data (format: [offset:8][data:remaining])
    if payload.len() < 2 + path_len + 8 {
        return Err(CtrlError::BadFrame);
    }

    let offset_start = 2 + path_len;
    let offset = u64::from_le_bytes([
        payload[offset_start],
        payload[offset_start + 1],
        payload[offset_start + 2],
        payload[offset_start + 3],
        payload[offset_start + 4],
        payload[offset_start + 5],
        payload[offset_start + 6],
        payload[offset_start + 7],
    ]);

    let data = &payload[offset_start + 8..];

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
        audit().log_operation(agent_id, 0x32, false);
        return Err(CtrlError::AuthFailed);
    }

    // Check file size limit
    let size_decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FileSize(data.len()),
    );

    if let PolicyDecision::Deny { reason } = size_decision {
        uart::print_str("[AgentSys] FS_WRITE denied: ");
        uart::print_str(reason);
        uart::print_str("\n");
        audit().log_operation(agent_id, 0x32, false);
        return Err(CtrlError::AuthFailed);
    }

    // REAL VFS WRITE
    match crate::vfs::open(path, crate::vfs::OpenFlags::O_WRONLY | crate::vfs::OpenFlags::O_CREAT) {
        Ok(file) => {
            // Seek to offset if needed
            if offset > 0 {
                if let Err(e) = file.lseek(offset as i64, 0) {
                    uart::print_str("[FS] Seek error: ");
                    uart::print_str(e.description());
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x32, false);
                    return Err(CtrlError::IoError);
                }
            }

            match file.write(data) {
                Ok(bytes_written) => {
                    uart::print_str("[FS] Wrote ");
                    uart::print_u32(bytes_written as u32);
                    uart::print_str(" bytes to ");
                    uart::print_str(path);
                    uart::print_str("\n");

                    audit().log_operation(agent_id, 0x32, true);
                    Ok(())
                }
                Err(e) => {
                    uart::print_str("[FS] Write error: ");
                    uart::print_str(e.description());
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x32, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(e) => {
            uart::print_str("[FS] Open/Create error: ");
            uart::print_str(e.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x32, false);
            Err(CtrlError::IoError)
        }
    }
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
        uart::print_str("[AgentSys] FS_STAT denied: ");
        uart::print_str(reason);
        uart::print_str("\n");
        audit().log_operation(agent_id, 0x33, false);
        return Err(CtrlError::AuthFailed);
    }

    // REAL VFS STAT
    match crate::vfs::open(path, crate::vfs::OpenFlags::O_RDONLY) {
        Ok(file) => {
            match file.getattr() {
                Ok(meta) => {
                    uart::print_str("[FS] Stat: ");
                    uart::print_str(path);
                    uart::print_str(" - size=");
                    uart::print_u64(meta.size);
                    uart::print_str(" mode=");
                    uart::print_hex8((meta.mode & 0xFF) as u8);
                    uart::print_str(" type=");
                    uart::print_str(match meta.itype {
                        crate::vfs::InodeType::Directory => "dir",
                        crate::vfs::InodeType::Regular => "file",
                        crate::vfs::InodeType::CharDevice => "char",
                        crate::vfs::InodeType::BlockDevice => "block",
                        crate::vfs::InodeType::Symlink => "symlink",
                    });
                    uart::print_str("\n");

                    audit().log_operation(agent_id, 0x33, true);
                    Ok(())
                }
                Err(e) => {
                    uart::print_str("[FS] Getattr error: ");
                    uart::print_str(e.description());
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x33, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(e) => {
            uart::print_str("[FS] Stat error: ");
            uart::print_str(e.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x33, false);
            Err(CtrlError::IoError)
        }
    }
}

/// Handle FS_CREATE (0x34): Create file or directory
pub fn handle_create(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_fs_create", 1);

    if payload.len() < 3 {
        return Err(CtrlError::BadFrame);
    }

    let path = protocol::parse_path(payload).map_err(|_| CtrlError::BadFrame)?;
    let path_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;

    // Check if we have a kind byte
    let kind = if payload.len() >= 2 + path_len + 1 {
        payload[2 + path_len]
    } else {
        0 // Default to file
    };

    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if let PolicyDecision::Deny { reason } = decision {
        uart::print_str("[AgentSys] FS_CREATE denied: ");
        uart::print_str(reason);
        uart::print_str("\n");
        audit().log_operation(agent_id, 0x34, false);
        return Err(CtrlError::AuthFailed);
    }

    // REAL VFS CREATE
    let result = if kind == 0 {
        // Create file
        crate::vfs::create(path, 0o644, crate::vfs::OpenFlags::O_CREAT)
            .map(|_| ())
    } else {
        // Create directory
        crate::vfs::mkdir(path, 0o755)
    };

    match result {
        Ok(_) => {
            uart::print_str("[FS] Created: ");
            uart::print_str(path);
            uart::print_str(" (");
            uart::print_str(if kind == 0 { "file" } else { "dir" });
            uart::print_str(")\n");
            audit().log_operation(agent_id, 0x34, true);
            Ok(())
        }
        Err(e) => {
            uart::print_str("[FS] Create error: ");
            uart::print_str(e.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x34, false);
            Err(CtrlError::IoError)
        }
    }
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
        uart::print_str("[AgentSys] FS_DELETE denied: ");
        uart::print_str(reason);
        uart::print_str("\n");
        audit().log_operation(agent_id, 0x35, false);
        return Err(CtrlError::AuthFailed);
    }

    // REAL VFS DELETE
    // Try unlink first (for files), then rmdir (for directories)
    let result = crate::vfs::unlink(path)
        .or_else(|_| crate::vfs::rmdir(path));

    match result {
        Ok(_) => {
            uart::print_str("[FS] Deleted: ");
            uart::print_str(path);
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x35, true);
            Ok(())
        }
        Err(e) => {
            uart::print_str("[FS] Delete error: ");
            uart::print_str(e.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x35, false);
            Err(CtrlError::IoError)
        }
    }
}
