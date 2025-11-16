//! Agent Supervision Module syscall interface
//!
//! This module provides syscalls for userland tools to interact with ASM:
//! - Query telemetry
//! - Update policies
//! - Manage agent lifecycle

use crate::lib::error::{Errno, Result};
use crate::agent_sys::supervisor::{TELEMETRY, POLICY_CONTROLLER};
use crate::agent_sys::supervisor::policy_controller::{PolicyPatch, PolicyError};
use crate::agent_sys::AgentId;
use alloc::vec::Vec;

/// Syscall numbers for ASM (500-509 range)
pub const SYS_ASM_GET_TELEMETRY: usize = 500;
pub const SYS_ASM_UPDATE_POLICY: usize = 501;
pub const SYS_ASM_GET_AGENT_INFO: usize = 502;
pub const SYS_LLM_REQUEST: usize = 503;

/// sys_asm_get_telemetry - Get telemetry snapshot
///
/// Copies a JSON-serialized telemetry snapshot to userland buffer.
///
/// # Arguments
///
/// * `buf` - Userland buffer to write JSON data
/// * `len` - Size of buffer
///
/// # Returns
///
/// Number of bytes written, or negative errno
///
/// # Example
///
/// ```c
/// char buf[4096];
/// ssize_t ret = syscall(500, buf, sizeof(buf));
/// if (ret > 0) {
///     printf("Telemetry: %.*s\n", (int)ret, buf);
/// }
/// ```
pub fn sys_asm_get_telemetry(buf: *mut u8, len: usize) -> Result<isize> {
    // Validate buffer pointer
    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    // Basic userspace pointer validation
    if (buf as u64) < 0x1000 || (buf as u64) >= 0xFFFF_0000_0000_0000 {
        return Err(Errno::EFAULT);
    }

    // Get telemetry snapshot
    let snapshot = crate::agent_sys::supervisor::hooks::get_telemetry_snapshot()
        .ok_or(Errno::EAGAIN)?;

    // Serialize to JSON
    let json = serde_json::to_string(&snapshot)
        .map_err(|_| Errno::EINVAL)?;

    let bytes = json.as_bytes();
    let to_copy = bytes.len().min(len);

    // Copy to userspace
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, to_copy);
    }

    Ok(to_copy as isize)
}

/// sys_asm_update_policy - Update agent policy
///
/// Applies a policy patch to an agent.
///
/// # Arguments
///
/// * `agent_id` - Agent to update
/// * `patch_type` - Type of patch (see PolicyPatchType)
/// * `arg` - Patch-specific argument
///
/// # Returns
///
/// 0 on success, negative errno on failure
///
/// # PolicyPatchType values
///
/// - 0: AddCapability (arg = capability)
/// - 1: RemoveCapability (arg = capability)
/// - 2: EnableAutoRestart (arg = max_restarts)
/// - 3: DisableAutoRestart
///
/// # Example
///
/// ```c
/// // Enable auto-restart with 5 attempts
/// int ret = syscall(501, agent_id, 2, 5);
/// if (ret < 0) {
///     perror("Failed to update policy");
/// }
/// ```
pub fn sys_asm_update_policy(agent_id: AgentId, patch_type: u32, arg: u64) -> Result<isize> {
    use crate::security::agent_policy::Capability;

    // Determine patch from type and arg
    let patch = match patch_type {
        0 => {
            // AddCapability
            let cap = match arg {
                0 => Capability::FsBasic,
                1 => Capability::AudioControl,
                2 => Capability::DocBasic,
                3 => Capability::Capture,
                4 => Capability::Screenshot,
                5 => Capability::Admin,
                _ => return Err(Errno::EINVAL),
            };
            PolicyPatch::AddCapability(cap)
        }
        1 => {
            // RemoveCapability
            let cap = match arg {
                0 => Capability::FsBasic,
                1 => Capability::AudioControl,
                2 => Capability::DocBasic,
                3 => Capability::Capture,
                4 => Capability::Screenshot,
                5 => Capability::Admin,
                _ => return Err(Errno::EINVAL),
            };
            PolicyPatch::RemoveCapability(cap)
        }
        2 => {
            // EnableAutoRestart
            PolicyPatch::EnableAutoRestart {
                max_restarts: arg as u32,
            }
        }
        3 => {
            // DisableAutoRestart
            PolicyPatch::DisableAutoRestart
        }
        _ => return Err(Errno::EINVAL),
    };

    // Apply policy update
    let mut controller = POLICY_CONTROLLER.lock();
    if let Some(ref mut ctrl) = *controller {
        ctrl.update_policy(agent_id, patch)
            .map_err(|e| match e {
                PolicyError::AgentNotFound => Errno::ESRCH,
                PolicyError::InsufficientPermission => Errno::EPERM,
                PolicyError::PrivilegeEscalation => Errno::EPERM,
                PolicyError::InvalidPatch => Errno::EINVAL,
            })?;
        Ok(0)
    } else {
        Err(Errno::EAGAIN)
    }
}

/// sys_asm_get_agent_info - Get information about a specific agent
///
/// Writes agent metadata to userland buffer in text format.
///
/// # Arguments
///
/// * `agent_id` - Agent to query
/// * `buf` - Buffer for output
/// * `len` - Buffer size
///
/// # Returns
///
/// Number of bytes written, or negative errno
pub fn sys_asm_get_agent_info(agent_id: AgentId, buf: *mut u8, len: usize) -> Result<isize> {
    use core::fmt::Write;

    // Validate buffer
    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    if (buf as u64) < 0x1000 || (buf as u64) >= 0xFFFF_0000_0000_0000 {
        return Err(Errno::EFAULT);
    }

    // Get agent metadata
    let supervisor = crate::agent_sys::supervisor::AGENT_SUPERVISOR.lock();
    if let Some(ref sup) = *supervisor {
        if let Some(metadata) = sup.get_agent(agent_id) {
            // Format agent info
            let mut output = Vec::with_capacity(len);
            let mut writer = VecWriter(&mut output);

            let _ = writeln!(writer, "Agent ID: {}", metadata.agent_id);
            let _ = writeln!(writer, "PID: {}", metadata.pid);
            let _ = writeln!(writer, "Name: {}", metadata.name);
            let _ = writeln!(writer, "Active: {}", metadata.active);
            let _ = writeln!(writer, "Auto-restart: {}", metadata.auto_restart);
            let _ = writeln!(writer, "Restart count: {}/{}", metadata.restart_count, metadata.max_restarts);
            let _ = writeln!(writer, "Uptime: {} us", metadata.uptime());
            let _ = writeln!(writer, "Capabilities: {} total", metadata.capabilities.len());

            let to_copy = output.len().min(len);
            unsafe {
                core::ptr::copy_nonoverlapping(output.as_ptr(), buf, to_copy);
            }

            Ok(to_copy as isize)
        } else {
            Err(Errno::ESRCH)
        }
    } else {
        Err(Errno::EAGAIN)
    }
}

/// sys_llm_request - Make an LLM API request
///
/// Routes an LLM request through the Cloud Gateway with automatic fallback,
/// rate limiting, and multi-provider support.
///
/// # Arguments
///
/// * `req_buf` - Pointer to JSON-encoded LLMRequest
/// * `req_len` - Length of request buffer
/// * `resp_buf` - Pointer to buffer for JSON response
/// * `resp_len` - Size of response buffer
///
/// # Returns
///
/// Number of bytes written to resp_buf, or negative errno
///
/// # Request Format (JSON)
///
/// ```json
/// {
///   "agent_id": 100,
///   "prompt": "Your prompt here",
///   "max_tokens": 1000,
///   "temperature": 0.7,
///   "preferred_provider": "claude"  // optional
/// }
/// ```
///
/// # Response Format (JSON)
///
/// ```json
/// {
///   "provider": "claude",
///   "text": "Response text...",
///   "tokens_used": 450,
///   "duration_us": 123456,
///   "was_fallback": false
/// }
/// ```
///
/// # Example
///
/// ```c
/// const char *req = "{\"agent_id\":100,\"prompt\":\"Hello\",\"max_tokens\":100}";
/// char resp[4096];
///
/// ssize_t ret = syscall(503, req, strlen(req), resp, sizeof(resp));
/// if (ret > 0) {
///     printf("Response: %.*s\n", (int)ret, resp);
/// }
/// ```
#[cfg(feature = "agentsys")]
pub fn sys_llm_request(
    req_buf: *const u8,
    req_len: usize,
    resp_buf: *mut u8,
    resp_len: usize,
) -> Result<isize> {
    use crate::agent_sys::cloud_gateway::{CLOUD_GATEWAY, LLMRequest};

    // Validate pointers
    if req_buf.is_null() || resp_buf.is_null() {
        return Err(Errno::EFAULT);
    }

    if (req_buf as u64) < 0x1000 || (resp_buf as u64) < 0x1000 {
        return Err(Errno::EFAULT);
    }

    // Read request from userspace
    let req_slice = unsafe {
        core::slice::from_raw_parts(req_buf, req_len)
    };

    // Parse JSON request
    let request: LLMRequest = serde_json::from_slice(req_slice)
        .map_err(|_| Errno::EINVAL)?;

    // TODO: Check if caller has LLM_ACCESS capability
    // For now, allow all requests

    // Route through gateway
    let mut gateway = CLOUD_GATEWAY.lock();
    let gateway_ref = gateway.as_mut()
        .ok_or(Errno::EAGAIN)?;

    let response = gateway_ref.route_request(&request)
        .map_err(|e| match e {
            crate::agent_sys::cloud_gateway::GatewayError::RateLimitExceeded => Errno::EAGAIN,
            crate::agent_sys::cloud_gateway::GatewayError::PermissionDenied => Errno::EPERM,
            crate::agent_sys::cloud_gateway::GatewayError::Timeout => Errno::ETIMEDOUT,
            _ => Errno::EIO,
        })?;

    // Serialize response to JSON
    let json = serde_json::to_string(&response)
        .map_err(|_| Errno::EINVAL)?;

    let bytes = json.as_bytes();
    let to_copy = bytes.len().min(resp_len);

    // Copy to userspace
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), resp_buf, to_copy);
    }

    Ok(to_copy as isize)
}

#[cfg(not(feature = "agentsys"))]
pub fn sys_llm_request(
    _req_buf: *const u8,
    _req_len: usize,
    _resp_buf: *mut u8,
    _resp_len: usize,
) -> Result<isize> {
    Err(Errno::ENOSYS)
}

/// Helper for writing to Vec<u8>
struct VecWriter<'a>(&'a mut Vec<u8>);

impl<'a> Write for VecWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_constants() {
        // Ensure no conflicts with existing syscalls
        assert!(SYS_ASM_GET_TELEMETRY >= 500);
        assert!(SYS_ASM_UPDATE_POLICY >= 500);
        assert!(SYS_ASM_GET_AGENT_INFO >= 500);
    }
}
