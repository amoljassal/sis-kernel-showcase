// Process management
// Phase A0 - Minimal stubs, full implementation in Phase A1

pub mod scheduler;

// Process ID type
pub type Pid = u32;

// Process state (stubbed for Phase A0)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Ready,
    Sleeping,
    Stopped,
    Zombie,
    Dead,
}

// Stub: Get current process PID
// Phase A0: Always returns 1 (no process model yet)
// Phase A1: Will return actual current process PID
pub fn current_pid() -> Pid {
    1
}

// Stub: Get current process (will be real in Phase A1)
#[allow(dead_code)]
pub fn current_process() -> Pid {
    current_pid()
}
