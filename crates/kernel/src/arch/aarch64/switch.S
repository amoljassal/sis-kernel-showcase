// Context switch assembly for AArch64
//
// void switch_to(CpuContext *prev, const CpuContext *next)
//
// Saves callee-saved registers from current context and restores from next context.
// This is the core of the context switch mechanism.
//
// CpuContext layout (in order):
// offset 0:  x19
// offset 8:  x20
// offset 16: x21
// offset 24: x22
// offset 32: x23
// offset 40: x24
// offset 48: x25
// offset 56: x26
// offset 64: x27
// offset 72: x28
// offset 80: x29 (FP)
// offset 88: x30 (LR)
// offset 96: sp

.section .text
.global switch_to
.type switch_to, @function

switch_to:
    // x0 = prev (CpuContext *prev)
    // x1 = next (const CpuContext *next)

    // Save callee-saved registers to prev
    stp x19, x20, [x0, #0]
    stp x21, x22, [x0, #16]
    stp x23, x24, [x0, #32]
    stp x25, x26, [x0, #48]
    stp x27, x28, [x0, #64]
    stp x29, x30, [x0, #80]    // FP and LR
    mov x9, sp
    str x9, [x0, #96]          // Save SP

    // Restore callee-saved registers from next
    ldp x19, x20, [x1, #0]
    ldp x21, x22, [x1, #16]
    ldp x23, x24, [x1, #32]
    ldp x25, x26, [x1, #48]
    ldp x27, x28, [x1, #64]
    ldp x29, x30, [x1, #80]    // FP and LR
    ldr x9, [x1, #96]
    mov sp, x9                  // Restore SP

    // Return to next context (via LR we just loaded)
    ret

.size switch_to, .-switch_to
