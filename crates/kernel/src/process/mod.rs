// Process management
// Phase A1 - Full process model with fork/exec/wait

pub mod scheduler;
pub mod task;
pub mod pid;
pub mod wait;
pub mod current;
pub mod exec;

// Re-export commonly used types and functions
pub use task::{Pid, Task, ProcessState, Credentials, MemoryManager, Vma, VmaFlags};
pub use pid::{init_process_table, alloc_pid, insert_task, get_process_table};
pub use current::{current_pid, set_current_pid, switch_to};
pub use wait::{do_wait4, do_exit, WNOHANG, WUNTRACED, WCONTINUED};
