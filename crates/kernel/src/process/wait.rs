/// Wait syscall implementation for zombie reaping
///
/// Implements wait4/waitpid to allow parent processes to reap
/// zombie children and retrieve their exit codes.

use super::task::{Pid, Task, ProcessState};
use super::pid::get_process_table;
use crate::lib::error::{KernelError, Errno};
use alloc::vec;

/// Options for wait4
pub const WNOHANG: i32 = 1;
pub const WUNTRACED: i32 = 2;
pub const WCONTINUED: i32 = 8;

/// Encode exit status
pub fn w_exitcode(exit_code: i32, signal: i32) -> i32 {
    (exit_code << 8) | (signal & 0x7f)
}

/// Wait for a child process to exit
///
/// Implements wait4 syscall:
/// - pid > 0: wait for specific child
/// - pid == -1: wait for any child
/// - pid == 0: wait for any child in same process group (not implemented yet)
/// - pid < -1: wait for any child in process group |pid| (not implemented yet)
///
/// Returns (child_pid, exit_status) or error
pub fn do_wait4(
    current_pid: Pid,
    pid: i32,
    wstatus: *mut i32,
    options: i32,
) -> Result<Pid, Errno> {
    let mut table = get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;

    // Get the current task
    let current = table.get(current_pid).ok_or(Errno::ESRCH)?;

    // Find children that match the wait criteria
    let children = if pid > 0 {
        // Wait for specific child
        vec![pid as Pid]
    } else if pid == -1 {
        // Wait for any child
        table.find_children(current_pid)
    } else {
        // Process groups not implemented yet
        return Err(Errno::EINVAL);
    };

    if children.is_empty() {
        // No children to wait for
        return Err(Errno::ECHILD);
    }

    // Look for zombie children
    for child_pid in children {
        if let Some(child) = table.get(child_pid) {
            if child.is_zombie() {
                let exit_code = child.exit_code;
                let child_pid = child.pid;

                // Write exit status to userspace if pointer is valid
                if !wstatus.is_null() {
                    unsafe {
                        // Validate userspace pointer (basic check)
                        if (wstatus as u64) < 0xFFFF_0000_0000_0000 {
                            *wstatus = w_exitcode(exit_code, 0);
                        } else {
                            return Err(Errno::EFAULT);
                        }
                    }
                }

                // Reap the zombie child
                table.remove(child_pid);

                return Ok(child_pid);
            }
        }
    }

    // No zombie children found
    if (options & WNOHANG) != 0 {
        // Non-blocking mode - return 0 to indicate no child ready
        return Ok(0);
    }

    // Blocking mode - should sleep and retry (not implemented yet)
    // For now, return EAGAIN to indicate would block
    Err(Errno::EAGAIN)
}

/// Exit current process
///
/// Marks the current process as zombie and performs cleanup:
/// - Reparent children to init
/// - Wake up parent (not implemented - needs wait queue)
/// - Schedule next task
pub fn do_exit(current_pid: Pid, exit_code: i32) -> ! {
    {
        let mut table = get_process_table();
        if let Some(ref mut tbl) = *table {
            // Mark current task as zombie
            if let Some(task) = tbl.get_mut(current_pid) {
                task.exit(exit_code);
                crate::info!("Process {} exited with code {}", current_pid, exit_code);
            }

            // Reparent children to init
            tbl.reparent_to_init(current_pid);
        }
    }

    // Notify ASM if this was an agent process
    #[cfg(feature = "agentsys")]
    {
        crate::agent_sys::supervisor::hooks::on_process_exit(current_pid, exit_code);
    }

    // TODO: Wake parent if it's waiting
    // TODO: Send SIGCHLD to parent

    // Schedule next task
    crate::process::scheduler::schedule();

    // Should never reach here
    loop {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfi", options(nostack, preserves_flags));
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("hlt", options(nostack, preserves_flags));
        }
    }
}
