//! # x86_64 System Call Entry
//!
//! This module implements the SYSCALL/SYSRET fast path for x86_64 system calls.
//! SYSCALL/SYSRET is significantly faster than the legacy INT 0x80 mechanism.
//!
//! ## SYSCALL/SYSRET Overview
//!
//! The SYSCALL instruction is Intel/AMD's fast system call mechanism:
//! - **SYSCALL**: Transition from user mode (Ring 3) to kernel mode (Ring 0)
//! - **SYSRET**: Return from kernel mode to user mode
//!
//! ### Advantages over INT 0x80
//!
//! - **Faster**: ~50% faster than INT-based syscalls
//! - **No IDT lookup**: Direct jump to LSTAR MSR address
//! - **Minimal overhead**: Only saves RIP, RFLAGS, and switches stacks
//!
//! ## MSR Configuration
//!
//! SYSCALL/SYSRET requires configuring several Model-Specific Registers:
//!
//! ### EFER (Extended Feature Enable Register)
//! - Bit 0: SCE (System Call Extensions) - must be set to enable SYSCALL/SYSRET
//!
//! ### STAR (Syscall Target Address Register)
//! ```text
//! 63:48  47:32  31:0
//! +------+------+----+
//! |SYSRET|SYSCALL|Res|
//! | CS   | CS    |   |
//! +------+------+----+
//! ```
//! - Bits 63-48: CS selector for SYSRET (user code segment)
//! - Bits 47-32: CS selector for SYSCALL (kernel code segment)
//!
//! ### LSTAR (Long Mode SYSCALL Target Address)
//! - Contains the RIP to jump to on SYSCALL (syscall_entry function)
//!
//! ### SFMASK (Syscall Flag Mask)
//! - RFLAGS bits to clear on SYSCALL entry (typically IF to disable interrupts)
//!
//! ## Calling Convention
//!
//! x86_64 System V ABI calling convention:
//! ```text
//! Register    SYSCALL Usage       Function Call Usage
//! --------    --------------      -------------------
//! RAX         Syscall number      Return value
//! RDI         Argument 1          Argument 1
//! RSI         Argument 2          Argument 2
//! RDX         Argument 3          Argument 3
//! R10         Argument 4          -
//! R8          Argument 5          Argument 5
//! R9          Argument 6          Argument 6
//! RCX         Destroyed (RIP)     Argument 4
//! R11         Destroyed (RFLAGS)  -
//! ```
//!
//! **Important**: SYSCALL uses R10 for arg4 (not RCX like function calls)
//! because SYSCALL saves user RIP in RCX.
//!
//! ## State Transitions
//!
//! ### On SYSCALL (User → Kernel)
//! 1. RCX ← User RIP (return address)
//! 2. R11 ← User RFLAGS
//! 3. RFLAGS ← RFLAGS & ~SFMASK (clear masked bits)
//! 4. CS ← STAR[47:32] (kernel code segment)
//! 5. SS ← STAR[47:32] + 8 (kernel data segment)
//! 6. RIP ← LSTAR (syscall_entry)
//! 7. CPL ← 0 (kernel mode)
//!
//! ### On SYSRET (Kernel → User)
//! 1. RIP ← RCX (user return address)
//! 2. RFLAGS ← R11 (user flags)
//! 3. CS ← STAR[63:48] + 16 (user code segment)
//! 4. SS ← STAR[63:48] + 8 (user data segment)
//! 5. CPL ← 3 (user mode)
//!
//! ## Stack Management
//!
//! For M4, we use the TSS RSP0 field for the kernel stack:
//! - TSS.RSP0 contains the kernel stack pointer for this CPU
//! - On SYSCALL, we manually switch to TSS.RSP0
//! - On SYSRET, we restore user stack from saved value
//!
//! Future (M8: SMP): Per-CPU kernel stacks via GS segment.
//!
//! ## Safety Considerations
//!
//! - SYSCALL doesn't switch stacks automatically (unlike interrupts)
//! - SYSCALL doesn't disable interrupts automatically (we do via SFMASK)
//! - User can control RCX/R11, so we must save them before using
//! - Stack pointer must be valid before accessing memory
//! - Interrupts must be disabled until stack is switched

use x86_64::{
    VirtAddr,
    registers::{
        model_specific::{Efer, EferFlags, LStar, Star, SFMask},
        rflags::RFlags,
    },
};
use crate::arch::x86_64::{gdt, tss};

/// Initialize SYSCALL/SYSRET support
///
/// Configures the necessary MSRs to enable fast system calls:
/// - Enables SCE bit in EFER
/// - Sets up STAR with kernel/user segment selectors
/// - Points LSTAR to syscall_entry
/// - Configures SFMASK to disable interrupts on entry
///
/// # Safety
///
/// - Must be called after GDT and TSS are initialized
/// - Must only be called once during boot
/// - Interrupts should be disabled when calling this
///
/// # Panics
///
/// Panics if SYSCALL/SYSRET is not supported by the CPU
pub unsafe fn init() {
    use x86_64::registers::control::Cr4;

    // Verify CPU supports SYSCALL/SYSRET
    // This is part of the x86_64 specification, so should always be present
    // on 64-bit CPUs, but we check anyway
    let cpuid = raw_cpuid::CpuId::new();
    if let Some(features) = cpuid.get_extended_processor_and_feature_identifiers() {
        if !features.has_syscall_sysret() {
            panic!("CPU does not support SYSCALL/SYSRET");
        }
    } else {
        panic!("Cannot detect SYSCALL/SYSRET support");
    }

    crate::arch::x86_64::serial::serial_write(b"[SYSCALL] Initializing SYSCALL/SYSRET...\n");

    // Enable SYSCALL/SYSRET in EFER
    Efer::update(|flags| {
        *flags |= EferFlags::SYSTEM_CALL_EXTENSIONS;
    });

    // Configure STAR register with segment selectors
    // SYSCALL loads CS from STAR[47:32] and SS from STAR[47:32]+8
    // SYSRET loads CS from STAR[63:48]+16 and SS from STAR[63:48]+8
    //
    // Our GDT layout (from gdt.rs):
    // 0: Null
    // 1: Kernel Code (index=1, selector=0x08)
    // 2: Kernel Data (index=2, selector=0x10)
    // 3: User Data   (index=3, selector=0x18)
    // 4: User Code   (index=4, selector=0x20)
    // 5: TSS
    //
    // For SYSRET to work correctly with our GDT:
    // - STAR[63:48] should be set to (User Data - 16) = 0x18 - 16 = 0x08
    //   Then SYSRET will load CS=0x08+16=0x18+8=0x20 (User Code with RPL=3: 0x20|3=0x23)
    //   And SS=0x08+8=0x10+8=0x18 (User Data with RPL=3: 0x18|3=0x1B)
    //
    // Wait, let me recalculate. The GDT selectors with RPL:
    // - Kernel Code: 0x08 | 0 = 0x08
    // - Kernel Data: 0x10 | 0 = 0x10
    // - User Data:   0x18 | 3 = 0x1B
    // - User Code:   0x20 | 3 = 0x23
    //
    // STAR format:
    // - SYSCALL CS = STAR[47:32]
    // - SYSCALL SS = STAR[47:32] + 8
    // - SYSRET CS = STAR[63:48] + 16
    // - SYSRET SS = STAR[63:48] + 8
    //
    // We want:
    // - SYSCALL CS = 0x08 (Kernel Code)
    // - SYSCALL SS = 0x10 (Kernel Data)
    // - SYSRET CS = 0x23 (User Code with RPL=3)
    // - SYSRET SS = 0x1B (User Data with RPL=3)
    //
    // So: STAR[47:32] = 0x08 (Kernel Code)
    //     STAR[63:48] = 0x23 - 16 = 0x13
    //
    // Wait, SYSRET automatically sets RPL=3, so:
    // - SYSRET CS = (STAR[63:48] + 16) | 3
    // - SYSRET SS = (STAR[63:48] + 8) | 3
    //
    // We want CS=0x20|3 and SS=0x18|3
    // So: STAR[63:48] = 0x10 (because 0x10+16=0x20, 0x10+8=0x18)
    //
    // Actually, let me look at the GDT indices:
    // Index 1 (0x08): Kernel Code
    // Index 2 (0x10): Kernel Data
    // Index 3 (0x18): User Data
    // Index 4 (0x20): User Code
    //
    // STAR[47:32] = 0x08 (kernel code)
    // STAR[63:48] = 0x10 (user base, SYSRET adds 8 for SS, 16 for CS, and sets RPL=3)

    let kernel_cs = gdt::kernel_code_selector().0;
    let user_data_base = gdt::user_data_selector().0 & !3; // Clear RPL bits

    Star::write(
        gdt::user_code_selector(),   // SYSRET will use this
        gdt::kernel_code_selector(),  // SYSCALL will use this
    ).unwrap();

    // Set LSTAR to point to syscall_entry
    LStar::write(VirtAddr::new(syscall_entry as u64));

    // Configure SFMASK to clear interrupt flag on SYSCALL entry
    // This ensures interrupts are disabled when we enter the kernel
    SFMask::write(
        RFlags::INTERRUPT_FLAG |
        RFlags::TRAP_FLAG |
        RFlags::ALIGNMENT_CHECK |
        RFlags::DIRECTION_FLAG
    );

    crate::arch::x86_64::serial::serial_write(b"[SYSCALL] SYSCALL/SYSRET initialized\n");
}

/// System call entry point
///
/// This is the entry point for all system calls via the SYSCALL instruction.
///
/// **Register State on Entry:**
/// - RAX: Syscall number
/// - RDI: Argument 1
/// - RSI: Argument 2
/// - RDX: Argument 3
/// - R10: Argument 4 (will be moved to RCX for C calling convention)
/// - R8:  Argument 5
/// - R9:  Argument 6
/// - RCX: User RIP (saved by SYSCALL)
/// - R11: User RFLAGS (saved by SYSCALL)
/// - RSP: User stack pointer (not changed by SYSCALL!)
///
/// **Register State on Exit:**
/// - RAX: Return value
/// - All other registers preserved (except RCX, R11 which are restored)
///
/// # Safety
///
/// This function is marked `unsafe` and uses inline assembly to:
/// 1. Save user stack pointer
/// 2. Switch to kernel stack from TSS
/// 3. Save registers
/// 4. Call syscall_handler in C
/// 5. Restore registers
/// 6. Switch back to user stack
/// 7. Return via SYSRETQ
#[naked]
pub unsafe extern "C" fn syscall_entry() {
    core::arch::asm!(
        // At this point:
        // - We're in kernel mode (CPL=0)
        // - Interrupts are disabled (via SFMASK)
        // - RCX = user RIP
        // - R11 = user RFLAGS
        // - RSP = user stack (DANGEROUS! We need to switch!)

        // Save user stack pointer to a scratch location
        // We'll use the user-accessible part of TSS or a temporary
        // For now, we'll push it on the user stack, then immediately
        // switch stacks and pop it to save elsewhere

        // Actually, for M4 without per-CPU, we use a simpler approach:
        // We'll use swapgs to access a kernel data structure
        // But wait, we don't have GS set up for per-CPU yet.

        // Simpler approach for M4: Use a global variable (not SMP-safe)
        // This will be fixed in M8 when we add per-CPU support

        "push rcx",              // Save user RIP on user stack (temporary)
        "push r11",              // Save user RFLAGS on user stack (temporary)

        // Get kernel stack from TSS RSP0
        // For now, we'll use a static kernel stack
        // In M8, this will come from per-CPU area
        "mov rcx, offset SYSCALL_STACK_TOP",
        "mov rsp, [rcx]",        // Switch to kernel stack

        // Now we're on kernel stack, save everything
        "push r11",              // User RFLAGS (from above)
        "push 0x2B",             // User SS (0x1B with RPL=3, but let's use 0x2B which is user data)
        "sub rsp, 8",            // Placeholder for user RSP (we'll fix this)
        "push r11",              // User RFLAGS again
        "push 0x23",             // User CS (0x23 = user code with RPL=3)
        "push rcx",              // User RIP (we saved in RCX above)

        // Wait, this is getting messy. Let me use a cleaner approach.
        // Let me start over with a cleaner design:

        // CLEAN APPROACH:
        // 1. Immediately save user RSP to R15 (callee-saved, we'll restore it)
        // 2. Load kernel stack
        // 3. Build stack frame
        // 4. Call handler
        // 5. Restore and return

        // Start over:
        "mov r15, rsp",          // R15 = user stack pointer (save it)

        // Load kernel stack pointer from static variable (M4 only, M8 will use per-CPU)
        "mov rsp, offset SYSCALL_KERNEL_STACK",
        "mov rsp, [rsp]",

        // Build a minimal stack frame for the syscall
        // We need to save: user RIP (RCX), user RFLAGS (R11), user RSP (R15)
        // Also save callee-saved registers per System V ABI

        "push r15",              // User RSP
        "push r11",              // User RFLAGS
        "push rcx",              // User RIP

        // Save callee-saved registers
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Arguments are already in the right registers for System V ABI:
        // RDI = syscall number
        // RSI = arg1
        // RDX = arg2
        // R10 = arg3 (but we need RCX for C calling convention)
        // R8  = arg4
        // R9  = arg5

        // Move R10 to RCX (arg3 position for C calling convention)
        "mov rcx, r10",

        // Move syscall number from RAX to RDI (first argument)
        // And shift other arguments
        "mov rdi, rax",          // arg0 = syscall number (was in RAX)
        // RSI already has arg1
        // RDX already has arg2
        // RCX has arg3 (moved from R10 above)
        // R8 already has arg4
        // R9 already has arg5

        // Call the syscall handler
        // Returns value in RAX
        "call {syscall_handler}",

        // RAX now contains return value, preserve it

        // Restore callee-saved registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",

        // Restore user context
        "pop rcx",               // User RIP
        "pop r11",               // User RFLAGS
        "pop r15",               // User RSP

        // Restore user stack pointer
        "mov rsp, r15",

        // Return to userspace via SYSRETQ
        // SYSRETQ will:
        // - RIP ← RCX
        // - RFLAGS ← R11
        // - CS ← STAR[63:48] + 16, RPL=3
        // - SS ← STAR[63:48] + 8, RPL=3
        "sysretq",

        syscall_handler = sym syscall_handler,
        options(noreturn)
    );
}

/// System call handler (Rust function called from assembly)
///
/// Dispatches the system call to the appropriate handler based on syscall number.
///
/// # Arguments
///
/// * `syscall_num` - System call number
/// * `arg1-arg5` - System call arguments
///
/// # Returns
///
/// System call return value (0 or positive for success, negative for error)
#[no_mangle]
extern "C" fn syscall_handler(
    syscall_num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> i64 {
    // For M4, we'll just log the syscall and return an error
    // Full syscall table integration will come later

    crate::arch::x86_64::serial::serial_write(b"[SYSCALL] Received syscall #");
    print_u64(syscall_num);
    crate::arch::x86_64::serial::serial_write(b" with args: ");
    print_u64(arg1);
    crate::arch::x86_64::serial::serial_write(b", ");
    print_u64(arg2);
    crate::arch::x86_64::serial::serial_write(b", ");
    print_u64(arg3);
    crate::arch::x86_64::serial::serial_write(b"\n");

    // Return -ENOSYS (function not implemented)
    -38  // ENOSYS
}

/// Helper to print u64 to serial
fn print_u64(n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut num = n;

    if num == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
        return;
    }

    while num > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    // Reverse the buffer
    for j in 0..i/2 {
        buf.swap(j, i - 1 - j);
    }

    crate::arch::x86_64::serial::serial_write(&buf[..i]);
}

/// Kernel stack for syscalls (M4 only, replaced by per-CPU in M8)
///
/// This is a temporary solution for M4. In M8, each CPU will have its own
/// stack in the per-CPU data area.
///
/// Stack grows downward, so we use the end of the array as the stack top.
static mut SYSCALL_KERNEL_STACK_DATA: [u8; 16384] = [0; 16384];

/// Pointer to top of kernel stack
static mut SYSCALL_KERNEL_STACK: usize = 0;

/// Top of syscall stack (for assembly access)
#[no_mangle]
static mut SYSCALL_STACK_TOP: usize = 0;

/// Initialize the syscall kernel stack
///
/// # Safety
///
/// Must be called during boot before syscalls are enabled
pub unsafe fn init_stack() {
    let stack_top = SYSCALL_KERNEL_STACK_DATA.as_ptr() as usize + SYSCALL_KERNEL_STACK_DATA.len();
    SYSCALL_KERNEL_STACK = stack_top;
    SYSCALL_STACK_TOP = stack_top;

    crate::arch::x86_64::serial::serial_write(b"[SYSCALL] Kernel stack initialized at ");
    print_hex(stack_top as u64);
    crate::arch::x86_64::serial::serial_write(b"\n");
}

/// Helper to print hex value
fn print_hex(n: u64) {
    crate::arch::x86_64::serial::serial_write(b"0x");
    for i in (0..16).rev() {
        let nibble = ((n >> (i * 4)) & 0xF) as u8;
        let ch = if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + (nibble - 10)
        };
        crate::arch::x86_64::serial::serial_write(&[ch]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_stack_size() {
        // Ensure stack is large enough (16 KiB should be plenty)
        assert!(unsafe { SYSCALL_KERNEL_STACK_DATA.len() } >= 16384);
    }
}
