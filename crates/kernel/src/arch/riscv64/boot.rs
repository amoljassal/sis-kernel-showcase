//! RISC-V Boot Module
//!
//! This module includes the boot assembly and provides Rust interfaces

// Include the boot assembly file
core::arch::global_asm!(include_str!("boot.S"));

/// Boot-related constants
pub const KERNEL_LOAD_ADDR: usize = 0x8000_0000;
pub const STACK_SIZE_PER_HART: usize = 4096;

/// Boot information passed from assembly
#[repr(C)]
pub struct BootInfo {
    pub hart_id: usize,
    pub dtb_ptr: usize,
}

// Early hardware initialization is defined in boot.S assembly

/// External references to assembly symbols
extern "C" {
    pub static init_complete_flag: u64;
}