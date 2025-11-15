//! Minimal trap frame representation for x86_64.
//!
//! This is a placeholder structure so higher-level scheduler/process code can
//! compile. It will be expanded with the full register set as the port matures.

#[derive(Debug, Clone, Copy, Default)]
pub struct TrapFrame {
    /// Instruction pointer (RIP) to return to.
    pub pc: u64,
    /// Saved user-space RFLAGS/SPSR equivalent.
    pub pstate: u64,
    /// Saved stack pointer (RSP) for user mode.
    pub sp: u64,
    /// General-purpose register placeholder for syscall return value (matches AArch64 x0 usage).
    pub x0: u64,
}

impl TrapFrame {
    pub const fn new(pc: u64, sp: u64, pstate: u64) -> Self {
        Self { pc, sp, pstate, x0: 0 }
    }
}
