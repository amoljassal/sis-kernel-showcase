// AArch64 architecture-specific code
// Phase A0 - Exception handling and syscall infrastructure

pub mod trap;
pub mod timer;

// TODO: Add in later phases:
// pub mod psci;    // SMP CPU bring-up (Phase E)
// pub mod gicv3;   // Interrupt controller (Phase E)
// pub mod mmu;     // Page tables, TLB (Phase A1)

pub use trap::*;
