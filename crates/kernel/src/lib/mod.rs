// Core library utilities for the kernel

// Include AArch64 assembly files directly using global_asm!
// This ensures they are properly linked into the binary for bare-metal targets
#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(include_str!("../arch/aarch64/vectors.S"));
#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(include_str!("../arch/aarch64/switch.S"));

pub mod error;
pub mod printk;
pub mod ringbuf;
pub mod debug;
pub mod panic;
