//! Minimal AArch64 cooperative context switching for benchmarking
//! Saves/restores callee-saved registers and SP between two contexts.

#![allow(dead_code)]

#[repr(C)]
pub struct A64Context {
    pub sp: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub x30: u64, // LR
}

impl A64Context {
    pub const fn new() -> Self {
        Self {
            sp: 0,
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            x29: 0,
            x30: 0,
        }
    }
}

/// Initialize a context to start executing at the given `entry` function
/// with the provided stack buffer. The stack pointer is set to the top of
/// the buffer, aligned to 16 bytes. On first switch into this context, the
/// CPU will `ret` to `entry`.
pub unsafe fn init_context(ctx: *mut A64Context, stack_ptr: *mut u8, stack_len: usize, entry: extern "C" fn() -> !) {
    let base = stack_ptr as usize;
    let size = stack_len;
    let mut sp_top = base + size;
    sp_top &= !0xF; // 16-byte alignment
    (*ctx).sp = sp_top as u64;
    (*ctx).x19 = 0;
    (*ctx).x20 = 0;
    (*ctx).x21 = 0;
    (*ctx).x22 = 0;
    (*ctx).x23 = 0;
    (*ctx).x24 = 0;
    (*ctx).x25 = 0;
    (*ctx).x26 = 0;
    (*ctx).x27 = 0;
    (*ctx).x28 = 0;
    (*ctx).x29 = 0;
    (*ctx).x30 = entry as u64;
}

extern "C" {
    /// Switch from `old` to `new`, saving callee-saved registers and SP into `old`
    /// and restoring them from `new`. Returns into the restored context via LR.
    pub fn aarch64_context_switch(old: *mut A64Context, new: *const A64Context);
}

core::arch::global_asm!(
    r#"
    .text
    .global aarch64_context_switch
aarch64_context_switch:
    // x0 = old, x1 = new
    // Save callee-saved regs and SP into *old
    mov     x2, sp
    str     x2, [x0, #0]
    stp     x19, x20, [x0, #8]
    stp     x21, x22, [x0, #24]
    stp     x23, x24, [x0, #40]
    stp     x25, x26, [x0, #56]
    stp     x27, x28, [x0, #72]
    stp     x29, x30, [x0, #88]

    // Restore from *new
    ldr     x2, [x1, #0]
    mov     sp, x2
    ldp     x19, x20, [x1, #8]
    ldp     x21, x22, [x1, #24]
    ldp     x23, x24, [x1, #40]
    ldp     x25, x26, [x1, #56]
    ldp     x27, x28, [x1, #72]
    ldp     x29, x30, [x1, #88]

    ret
    "#
);
