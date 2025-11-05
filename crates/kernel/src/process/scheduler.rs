// Process scheduler
// Phase A0 - Minimal stubs
// Phase A1 - Full preemptive scheduler with round-robin

use core::arch::asm;

/// Schedule next process
///
/// Phase A0: No-op (no process model yet)
/// Phase A1: Will implement round-robin scheduling with runqueue
pub fn schedule() {
    // Phase A0: No process model, nothing to schedule
}

/// Voluntarily yield CPU
///
/// Phase A0: Just wait for interrupt
/// Phase A1: Will call schedule() to switch to next process
pub fn yield_now() {
    unsafe {
        // Wait for interrupt
        asm!("wfi");
    }
}

/// Handle timer tick (called from IRQ handler)
///
/// Phase A0: No-op
/// Phase A1: Will check timeslice and preempt if needed
pub fn timer_tick() {
    // Phase A0: Timer ticks but no scheduling yet
}

/// Block current process
///
/// Phase A0: No-op
/// Phase A1: Mark process as Sleeping and schedule()
pub fn block_current() {
    // Phase A0: No process blocking
}

/// Wake a process by PID
///
/// Phase A0: No-op
/// Phase A1: Mark process as Ready
pub fn wake_process(_pid: super::Pid) {
    // Phase A0: No process waking
}
