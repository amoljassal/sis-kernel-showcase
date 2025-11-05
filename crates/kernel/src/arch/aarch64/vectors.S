// AArch64 Exception Vector Table
// Phase A0 - Complete exception vector implementation
//
// ARM Architecture Reference Manual ARMv8-A:
// - Vector table must be aligned to 2048 bytes (2^11)
// - Each vector entry is 128 bytes (32 instructions)
// - 16 total entries organized as:
//   * 4 for current EL with SP0
//   * 4 for current EL with SPx
//   * 4 for lower EL using AArch64
//   * 4 for lower EL using AArch32
//
// TrapFrame layout (defined in trap.rs):
// Offset | Field
// -------|-------
// 0x000  | x0-x1
// 0x010  | x2-x3
// 0x020  | x4-x5
// 0x030  | x6-x7
// 0x040  | x8-x9
// 0x050  | x10-x11
// 0x060  | x12-x13
// 0x070  | x14-x15
// 0x080  | x16-x17
// 0x090  | x18-x19
// 0x0A0  | x20-x21
// 0x0B0  | x22-x23
// 0x0C0  | x24-x25
// 0x0D0  | x26-x27
// 0x0E0  | x28-x29
// 0x0F0  | x30
// 0x0F8  | sp
// 0x100  | pc (ELR_EL1)
// 0x108  | pstate (SPSR_EL1)
// Total: 272 bytes (0x110)

.section .text
.global exception_vector_table

.align 11  // 2048-byte alignment required
exception_vector_table:

    //
    // Current EL with SP0 (vectors 0-3)
    //
    .org exception_vector_table + 0x000
    b exception_handler_invalid  // Sync

    .org exception_vector_table + 0x080
    b exception_handler_invalid  // IRQ

    .org exception_vector_table + 0x100
    b exception_handler_invalid  // FIQ

    .org exception_vector_table + 0x180
    b exception_handler_invalid  // SError

    //
    // Current EL with SPx (vectors 4-7)
    //
    .org exception_vector_table + 0x200
    b handle_sync_curr_el_spx

    .org exception_vector_table + 0x280
    b handle_irq_curr_el_spx

    .org exception_vector_table + 0x300
    b handle_fiq_curr_el_spx

    .org exception_vector_table + 0x380
    b handle_serr_curr_el_spx

    //
    // Lower EL using AArch64 (vectors 8-11) - SYSCALLS ENTER HERE
    //
    .org exception_vector_table + 0x400
    b handle_sync_lower_el_aarch64  // SVC instruction from EL0

    .org exception_vector_table + 0x480
    b handle_irq_lower_el_aarch64

    .org exception_vector_table + 0x500
    b handle_fiq_lower_el_aarch64

    .org exception_vector_table + 0x580
    b handle_serr_lower_el_aarch64

    //
    // Lower EL using AArch32 (vectors 12-15) - Unused
    //
    .org exception_vector_table + 0x600
    b exception_handler_invalid

    .org exception_vector_table + 0x680
    b exception_handler_invalid

    .org exception_vector_table + 0x700
    b exception_handler_invalid

    .org exception_vector_table + 0x780
    b exception_handler_invalid

//
// Handler implementations
//

// Invalid handler - should never be reached
exception_handler_invalid:
    // Infinite loop (will be caught during development)
    b exception_handler_invalid

//
// MACRO: Save full context to TrapFrame on stack
//
.macro SAVE_CONTEXT
    // Allocate TrapFrame on stack (272 bytes = 0x110)
    sub sp, sp, #0x110

    // Save general purpose registers x0-x30
    stp x0,  x1,  [sp, #(0x00)]
    stp x2,  x3,  [sp, #(0x10)]
    stp x4,  x5,  [sp, #(0x20)]
    stp x6,  x7,  [sp, #(0x30)]
    stp x8,  x9,  [sp, #(0x40)]
    stp x10, x11, [sp, #(0x50)]
    stp x12, x13, [sp, #(0x60)]
    stp x14, x15, [sp, #(0x70)]
    stp x16, x17, [sp, #(0x80)]
    stp x18, x19, [sp, #(0x90)]
    stp x20, x21, [sp, #(0xA0)]
    stp x22, x23, [sp, #(0xB0)]
    stp x24, x25, [sp, #(0xC0)]
    stp x26, x27, [sp, #(0xD0)]
    stp x28, x29, [sp, #(0xE0)]
    str x30,      [sp, #(0xF0)]

    // Save user stack pointer (SP_EL0)
    mrs x0, sp_el0
    str x0, [sp, #(0xF8)]

    // Save program counter (ELR_EL1)
    mrs x0, elr_el1
    str x0, [sp, #(0x100)]

    // Save processor state (SPSR_EL1)
    mrs x0, spsr_el1
    str x0, [sp, #(0x108)]
.endm

//
// MACRO: Restore full context from TrapFrame on stack
//
.macro RESTORE_CONTEXT
    // Restore processor state
    ldr x0, [sp, #(0x108)]
    msr spsr_el1, x0

    // Restore program counter
    ldr x0, [sp, #(0x100)]
    msr elr_el1, x0

    // Restore user stack pointer
    ldr x0, [sp, #(0xF8)]
    msr sp_el0, x0

    // Restore general purpose registers x0-x30
    ldp x28, x29, [sp, #(0xE0)]
    ldp x26, x27, [sp, #(0xD0)]
    ldp x24, x25, [sp, #(0xC0)]
    ldp x22, x23, [sp, #(0xB0)]
    ldp x20, x21, [sp, #(0xA0)]
    ldp x18, x19, [sp, #(0x90)]
    ldp x16, x17, [sp, #(0x80)]
    ldp x14, x15, [sp, #(0x70)]
    ldp x12, x13, [sp, #(0x60)]
    ldp x10, x11, [sp, #(0x50)]
    ldp x8,  x9,  [sp, #(0x40)]
    ldp x6,  x7,  [sp, #(0x30)]
    ldp x4,  x5,  [sp, #(0x20)]
    ldp x2,  x3,  [sp, #(0x10)]
    ldr x30,      [sp, #(0xF0)]
    ldp x0,  x1,  [sp, #(0x00)]

    // Deallocate TrapFrame
    add sp, sp, #0x110
.endm

//
// Lower EL AArch64 Synchronous Exception Handler
// This is called for syscalls (SVC) and page faults
//
handle_sync_lower_el_aarch64:
    SAVE_CONTEXT

    // Call Rust handler: fn handle_sync_exception(frame: &mut TrapFrame)
    mov x0, sp
    bl handle_sync_exception

    RESTORE_CONTEXT
    eret

//
// Lower EL AArch64 IRQ Handler
//
handle_irq_lower_el_aarch64:
    SAVE_CONTEXT

    // Call Rust handler: fn handle_irq(frame: &mut TrapFrame)
    mov x0, sp
    bl handle_irq

    RESTORE_CONTEXT
    eret

//
// Lower EL AArch64 FIQ Handler
//
handle_fiq_lower_el_aarch64:
    SAVE_CONTEXT

    // Call Rust handler: fn handle_fiq(frame: &mut TrapFrame)
    mov x0, sp
    bl handle_fiq

    RESTORE_CONTEXT
    eret

//
// Lower EL AArch64 SError Handler
//
handle_serr_lower_el_aarch64:
    SAVE_CONTEXT

    // Call Rust handler: fn handle_serror(frame: &mut TrapFrame)
    mov x0, sp
    bl handle_serror

    RESTORE_CONTEXT
    eret

//
// Current EL SPx Synchronous Exception Handler
//
handle_sync_curr_el_spx:
    SAVE_CONTEXT

    mov x0, sp
    bl handle_sync_curr_el

    RESTORE_CONTEXT
    eret

//
// Current EL SPx IRQ Handler
//
handle_irq_curr_el_spx:
    SAVE_CONTEXT

    mov x0, sp
    bl handle_irq

    RESTORE_CONTEXT
    eret

//
// Current EL SPx FIQ Handler
//
handle_fiq_curr_el_spx:
    SAVE_CONTEXT

    mov x0, sp
    bl handle_fiq

    RESTORE_CONTEXT
    eret

//
// Current EL SPx SError Handler
//
handle_serr_curr_el_spx:
    SAVE_CONTEXT

    mov x0, sp
    bl handle_serror

    RESTORE_CONTEXT
    eret
