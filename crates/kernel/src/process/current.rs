/// Current task pointer management
///
/// Provides access to the currently running task.
/// For single-CPU systems, uses a global variable.
/// For SMP systems, will use per-CPU storage.

use super::task::Pid;
use core::sync::atomic::{AtomicU32, Ordering};

/// Current task PID (global for single-CPU, will be per-CPU for SMP)
static CURRENT_PID: AtomicU32 = AtomicU32::new(1); // Start with PID 1 (init)

/// Get the current task PID
pub fn current_pid() -> Pid {
    CURRENT_PID.load(Ordering::Acquire)
}

/// Set the current task PID
pub fn set_current_pid(pid: Pid) {
    CURRENT_PID.store(pid, Ordering::Release);
}

/// Switch to a new task
///
/// This is called by the scheduler to switch context to a new task.
/// In a full implementation, this would:
/// 1. Save current task state
/// 2. Load new task state
/// 3. Switch page tables (TTBR0_EL0)
/// 4. Update current PID
///
/// For Phase A1, we do a minimal implementation.
pub fn switch_to(new_pid: Pid) {
    set_current_pid(new_pid);
    // TODO: Switch page tables
    // TODO: Load task's trap frame
}
