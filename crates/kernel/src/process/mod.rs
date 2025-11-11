// Process management
// Phase A1 - Full process model with fork/exec/wait

pub mod scheduler;
pub mod scheduler_smp; // Phase E: SMP scheduler
#[cfg(feature = "deterministic")]
pub mod sched_glue;   // Phase 8: CBS+EDF unified scheduler glue
pub mod task;
pub mod pid;
pub mod wait;
pub mod current;
pub mod exec;
pub mod signal;
pub mod fork;  // Phase 8: Fork scaffolding

// Re-export commonly used types and functions
pub use task::{Pid, Task, ProcessState, Credentials, MemoryManager, Vma, VmaFlags};
pub use pid::{init_process_table, alloc_pid, insert_task, get_process_table};
pub use wait::{do_wait4, do_exit, WNOHANG, WUNTRACED, WCONTINUED};
pub use signal::{Signal, SignalQueue, SignalAction, send_signal, deliver_signals};
pub use fork::{do_fork, do_exec, get_fork_stats, ForkStats};

// Use SMP scheduler if multiple CPUs are online, otherwise use simple scheduler
pub fn current_pid() -> Pid {
    if crate::smp::num_cpus() > 1 {
        scheduler_smp::current_pid().unwrap_or(0)
    } else {
        scheduler::current_pid().unwrap_or(0)
    }
}

pub fn set_current_pid(pid: Pid) {
    if crate::smp::num_cpus() > 1 {
        scheduler_smp::set_current(pid);
    } else {
        scheduler::set_current(pid);
    }
}

pub fn count_processes() -> usize {
    let table = get_process_table();
    table.as_ref().map(|t| t.len()).unwrap_or(0)
}
