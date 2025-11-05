// Process management
// Phase A1 - Full process model with fork/exec/wait

pub mod scheduler;
pub mod task;
pub mod pid;
pub mod wait;
pub mod current;
pub mod exec;
pub mod signal;

// Re-export commonly used types and functions
pub use task::{Pid, Task, ProcessState, Credentials, MemoryManager, Vma, VmaFlags};
pub use pid::{init_process_table, alloc_pid, insert_task, get_process_table};
pub use wait::{do_wait4, do_exit, WNOHANG, WUNTRACED, WCONTINUED};
pub use signal::{Signal, SignalQueue, SignalAction, send_signal, deliver_signals};

// Use scheduler's current_pid (returns Option<Pid>)
pub fn current_pid() -> Pid {
    scheduler::current_pid().unwrap_or(0)
}

pub fn set_current_pid(pid: Pid) {
    scheduler::set_current(pid);
}
