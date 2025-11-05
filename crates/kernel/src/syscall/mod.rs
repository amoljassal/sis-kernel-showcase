// Syscall infrastructure
// Phase A0 - Minimal syscall dispatcher

pub mod uaccess;

use crate::lib::error::{Errno, Result};

/// Syscall dispatcher - routes syscall number to appropriate handler
///
/// AArch64 Syscall ABI:
/// - Syscall number: x8
/// - Arguments: x0-x5 (up to 6 args)
/// - Return value: x0 (negative for errno)
pub fn syscall_dispatcher(nr: usize, args: &[u64; 6]) -> isize {
    let result = match nr {
        // Phase A0 minimal syscalls
        63 => sys_read(args[0] as i32, args[1] as *mut u8, args[2] as usize),
        64 => sys_write(args[0] as i32, args[1] as *const u8, args[2] as usize),
        93 => sys_exit(args[0] as i32),
        172 => sys_getpid(),

        // Phase A1+ syscalls (not implemented yet)
        _ => {
            crate::warn!("Unimplemented syscall: {}", nr);
            Err(Errno::ENOSYS)
        }
    };

    match result {
        Ok(ret) => ret,
        Err(e) => e.as_isize(),
    }
}

/// sys_read - Read from file descriptor
///
/// Phase A0: Only supports fd 0 (stdin) reading from console
pub fn sys_read(fd: i32, buf: *mut u8, count: usize) -> Result<isize> {
    if fd != 0 {
        return Err(Errno::EBADF); // Only stdin supported
    }

    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    if count == 0 {
        return Ok(0);
    }

    // Phase A0: Read from UART/console
    // For now, return empty read (would block in real implementation)
    // Full implementation with console buffering in Phase A1

    crate::debug!("sys_read(fd={}, buf={:p}, count={})", fd, buf, count);

    // Stub: Return 0 (EOF) for now
    Ok(0)
}

/// sys_write - Write to file descriptor
///
/// Phase A0: Only supports fd 1 (stdout) and fd 2 (stderr) writing to console
pub fn sys_write(fd: i32, buf: *const u8, count: usize) -> Result<isize> {
    if fd != 1 && fd != 2 {
        return Err(Errno::EBADF); // Only stdout/stderr supported
    }

    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    if count == 0 {
        return Ok(0);
    }

    // Copy data from user space (with validation)
    let data = unsafe {
        // Phase A0: Simple pointer access (no full uaccess yet)
        // In Phase A1, this will use proper copy_from_user
        core::slice::from_raw_parts(buf, count)
    };

    // Write to UART/console
    unsafe {
        crate::uart::write_bytes(data);
    }

    Ok(count as isize)
}

/// sys_exit - Terminate current process
///
/// Phase A0: Just panics (no process model yet)
/// Phase A1: Will properly terminate process and schedule next
pub fn sys_exit(code: i32) -> Result<isize> {
    crate::info!("Process exit with code {}", code);

    // Phase A0: No process model, just halt
    panic!("sys_exit called - no process model in Phase A0");
}

/// sys_getpid - Get process ID
///
/// Phase A0: Returns fixed PID 1 (no process model yet)
/// Phase A1: Will return actual process PID
pub fn sys_getpid() -> Result<isize> {
    // Phase A0: Stub - always return PID 1
    Ok(1)
}

// Syscall numbers for reference (ARM64 calling convention)
#[allow(dead_code)]
mod syscall_numbers {
    pub const SYS_READ: usize = 63;
    pub const SYS_WRITE: usize = 64;
    pub const SYS_EXIT: usize = 93;
    pub const SYS_GETPID: usize = 172;

    // Phase A1 syscalls
    pub const SYS_OPEN: usize = 56;
    pub const SYS_CLOSE: usize = 57;
    pub const SYS_FORK: usize = 220;
    pub const SYS_EXECVE: usize = 221;
    pub const SYS_WAIT4: usize = 260;
    // ... more syscalls to be added in Phase A1
}
