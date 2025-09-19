//! RISC-V Boot Assembly for SIS Kernel
//! 
//! OpenSBI-compatible boot implementation following research-backed design.
//! Conforms to RISC-V Privileged Specification v20231002.
//!
//! Research Basis:
//! - RISC-V Privileged Spec v20231002
//! - OpenSBI v0.2+ HSM extensions
//! - Device tree validation patterns

.section .text.boot
.global _start

//! Boot entry point - called by OpenSBI
//! a0: hart ID
//! a1: device tree blob pointer
_start:
    // Save hart ID and DTB pointer for later use
    mv tp, a0                    // Store hart ID in tp (thread pointer)
    mv s0, a1                    // Store DTB pointer in s0 (preserved register)
    
    // Only hart 0 initializes BSS and performs early setup
    bnez a0, wait_for_initialization
    
    // Hart 0: Clear BSS section
    la t0, _bss_start
    la t1, _bss_end
clear_bss_loop:
    bge t0, t1, bss_cleared
    sd zero, (t0)
    addi t0, t0, 8
    j clear_bss_loop

bss_cleared:
    // Hart 0: Set up initial stack
    la sp, _stack_top
    
    // Hart 0: Initialize critical hardware
    call early_hardware_init
    
    // Hart 0: Signal other harts that initialization is complete
    la t0, init_complete_flag
    li t1, 1
    sd t1, (t0)
    fence w,w                    // Ensure write is visible to other harts
    
    // Hart 0: Jump to main kernel initialization
    mv a0, tp                    // hart ID
    mv a1, s0                    // DTB pointer
    call riscv64_main
    
    // Should never return, but halt if it does
    j halt_hart

wait_for_initialization:
    // Non-zero harts: Wait for hart 0 to complete initialization
    la t0, init_complete_flag
wait_loop:
    ld t1, (t0)
    beqz t1, wait_loop
    fence r,r                    // Ensure read is synchronized
    
    // Set up per-hart stack
    // Stack size is 4KB (4096 bytes) per hart
    li t0, 4096
    mul t0, t0, tp               // hart_id * stack_size
    la sp, _stack_top
    sub sp, sp, t0               // sp = stack_top - (hart_id * stack_size)
    
    // Jump to secondary hart initialization
    mv a0, tp                    // hart ID
    mv a1, s0                    // DTB pointer
    call riscv64_secondary_main
    
    // Halt this hart
    j halt_hart

//! Early hardware initialization
//! Sets up essential hardware features before full kernel initialization
early_hardware_init:
    // Enable supervisor timer interrupt
    li t0, 0x20                  // STIE bit
    csrs sie, t0
    
    // Set up supervisor trap vector
    la t0, trap_vector
    csrw stvec, t0
    
    // Configure supervisor status
    li t0, 0x2                   // SPP = 0 (user mode), SPIE = 0
    csrw sstatus, t0
    
    ret

//! Halt hart - enter low power state
halt_hart:
    // Disable interrupts
    csrci sstatus, 0x2           // Clear SIE
    
    // Enter WFI loop
halt_loop:
    wfi                          // Wait for interrupt
    j halt_loop

//! Trap vector - minimal handler for early boot
//! Full interrupt handling is initialized later in Rust code
.align 4
trap_vector:
    // Save minimal context
    csrw sscratch, sp
    
    // Check if this is a timer interrupt
    csrr t0, scause
    li t1, 0x8000000000000005    // Supervisor timer interrupt
    beq t0, t1, handle_timer
    
    // For other traps during boot, halt
    j halt_hart

handle_timer:
    // Simple timer acknowledgment - full handling in Rust
    li t0, 0x20
    csrc sip, t0                 // Clear timer interrupt pending
    
    // Restore context and return
    csrr sp, sscratch
    sret

.section .data
//! Initialization synchronization flag
.align 8
init_complete_flag:
    .quad 0

.section .bss
//! Stack allocation
//! 16KB total: 4KB per hart (supports up to 4 harts)
.align 16
_stack_bottom:
    .space 16384                 // 16KB stack space
_stack_top:

//! BSS section markers
.align 8
_bss_start:
_bss_end: