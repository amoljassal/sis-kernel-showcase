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
        // I/O syscalls
        63 => sys_read(args[0] as i32, args[1] as *mut u8, args[2] as usize),
        64 => sys_write(args[0] as i32, args[1] as *const u8, args[2] as usize),

        // Process management
        93 => sys_exit(args[0] as i32),
        172 => sys_getpid(),
        220 => sys_fork(),
        221 => sys_execve(args[0] as *const u8, args[1] as *const *const u8, args[2] as *const *const u8),
        260 => sys_wait4(args[0] as i32, args[1] as *mut i32, args[2] as i32, args[3] as *mut u8),

        // Memory management
        214 => sys_brk(args[0] as *const u8),
        222 => sys_mmap(args[0] as *mut u8, args[1] as usize, args[2] as i32, args[3] as i32, args[4] as i32, args[5] as i64),
        215 => sys_munmap(args[0] as *mut u8, args[1] as usize),

        // Unimplemented
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
pub fn sys_exit(code: i32) -> Result<isize> {
    let pid = crate::process::current_pid();
    crate::info!("Process {} exit with code {}", pid, code);

    // Call do_exit which never returns
    crate::process::do_exit(pid, code);
}

/// sys_getpid - Get process ID
pub fn sys_getpid() -> Result<isize> {
    let pid = crate::process::current_pid();
    Ok(pid as isize)
}

/// sys_fork - Create a child process
pub fn sys_fork() -> Result<isize> {
    let parent_pid = crate::process::current_pid();

    // Allocate new PID for child
    let child_pid = crate::process::alloc_pid()
        .map_err(|_| Errno::EAGAIN)?;

    crate::info!("fork: parent={}, child={}", parent_pid, child_pid);

    // Get parent task and create child
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;

    let parent = table.get(parent_pid).ok_or(Errno::ESRCH)?;
    let mut child = crate::process::Task::fork_from(parent, child_pid);

    // Set up COW for parent and child
    crate::mm::setup_cow_for_fork(&mut child.mm)
        .map_err(|_| Errno::ENOMEM)?;

    // Insert child into process table
    drop(table); // Release lock before inserting
    crate::process::insert_task(child)
        .map_err(|_| Errno::ENOMEM)?;

    // TODO: Copy trap frame and set child's return value to 0
    // TODO: Mark child as runnable in scheduler

    // Parent returns child PID
    Ok(child_pid as isize)
}

/// sys_execve - Execute a program
pub fn sys_execve(
    pathname: *const u8,
    argv: *const *const u8,
    envp: *const *const u8,
) -> Result<isize> {
    // Stub for Phase A1
    crate::warn!("sys_execve not yet implemented");
    Err(Errno::ENOSYS)
}

/// sys_wait4 - Wait for process to change state
pub fn sys_wait4(
    pid: i32,
    wstatus: *mut i32,
    options: i32,
    rusage: *mut u8,
) -> Result<isize> {
    let current_pid = crate::process::current_pid();

    let child_pid = crate::process::do_wait4(current_pid, pid, wstatus, options)?;

    Ok(child_pid as isize)
}

/// sys_brk - Change data segment size
pub fn sys_brk(addr: *const u8) -> Result<isize> {
    let new_brk = addr as u64;
    let pid = crate::process::current_pid();

    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    let result_brk = task.mm.do_brk(new_brk)?;
    Ok(result_brk as isize)
}

/// sys_mmap - Map memory
pub fn sys_mmap(
    addr: *mut u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> Result<isize> {
    let pid = crate::process::current_pid();

    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    let result_addr = task.mm.do_mmap(addr as u64, length as u64, prot, flags)?;
    Ok(result_addr as isize)
}

/// sys_munmap - Unmap memory
pub fn sys_munmap(addr: *mut u8, length: usize) -> Result<isize> {
    let pid = crate::process::current_pid();

    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    task.mm.do_munmap(addr as u64, length as u64)?;
    Ok(0)
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
