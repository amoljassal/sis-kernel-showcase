//! # Interrupt Descriptor Table (IDT)
//!
//! The IDT is a table of interrupt and exception handlers used by the x86_64 CPU.
//! When an interrupt or exception occurs, the CPU uses the IDT to find the appropriate
//! handler function to execute.
//!
//! ## IDT Structure
//!
//! The IDT contains 256 entries (vectors 0-255):
//! - **0-31**: CPU exceptions (divide error, page fault, etc.)
//! - **32-255**: Hardware interrupts and software interrupts
//!
//! ## Exception Vectors (0-31)
//!
//! ```text
//! Vector  Mnemonic  Description                      Error Code
//! ------  --------  -------------------------------  ----------
//! 0       #DE       Divide Error                     No
//! 1       #DB       Debug Exception                  No
//! 2       NMI       Non-Maskable Interrupt           No
//! 3       #BP       Breakpoint                       No
//! 4       #OF       Overflow                         No
//! 5       #BR       Bound Range Exceeded             No
//! 6       #UD       Invalid Opcode                   No
//! 7       #NM       Device Not Available             No
//! 8       #DF       Double Fault                     Yes (always 0)
//! 9       ---       Coprocessor Segment Overrun      No (legacy)
//! 10      #TS       Invalid TSS                      Yes
//! 11      #NP       Segment Not Present              Yes
//! 12      #SS       Stack Segment Fault              Yes
//! 13      #GP       General Protection Fault         Yes
//! 14      #PF       Page Fault                       Yes
//! 15      ---       Reserved                         No
//! 16      #MF       x87 FPU Error                    No
//! 17      #AC       Alignment Check                  Yes (always 0)
//! 18      #MC       Machine Check                    No
//! 19      #XM       SIMD Floating-Point Exception    No
//! 20      #VE       Virtualization Exception         No
//! 21-31   ---       Reserved                         No
//! ```
//!
//! ## Interrupt Stack Frame
//!
//! When an interrupt/exception occurs, the CPU automatically pushes:
//! ```text
//! [High Address]
//! SS           (if privilege level changed)
//! RSP          (if privilege level changed)
//! RFLAGS
//! CS
//! RIP
//! Error Code   (for some exceptions)
//! [Low Address - RSP points here]
//! ```
//!
//! ## Double Fault Handler
//!
//! The double fault handler is special because it uses the IST (Interrupt Stack Table).
//! This ensures it has a valid stack even if the kernel stack is corrupted.
//!
//! ## Safety Considerations
//!
//! Exception handlers must:
//! 1. Not panic unless the system is truly unrecoverable
//! 2. Preserve all registers (handled by x86-interrupt ABI)
//! 3. Return via IRET instruction (handled by x86-interrupt ABI)
//! 4. Be careful with stack usage (especially double fault handler)

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use crate::arch::x86_64::tss::{DOUBLE_FAULT_IST_INDEX, NMI_IST_INDEX, MACHINE_CHECK_IST_INDEX};

lazy_static! {
    /// Global Interrupt Descriptor Table
    ///
    /// This table is initialized once during boot and contains handlers for all
    /// CPU exceptions and hardware interrupts.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // CPU Exceptions (0-31)
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);

        // Double fault - uses dedicated IST stack
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);

        // Hardware interrupts (32-255) will be added in M1

        idt
    };
}

/// Initialize the IDT (early boot version)
///
/// This function loads the IDT with basic exception handlers.
/// Full interrupt handling (hardware IRQs) will be added in later milestones.
///
/// # Safety
///
/// Must be called during early boot, after GDT/TSS are loaded.
pub unsafe fn init_idt_early() {
    IDT.load();
}

//
// Exception Handlers
//

/// Divide Error (#DE) - Vector 0
///
/// Triggered by DIV or IDIV instruction when:
/// - Divisor is zero
/// - Quotient is too large for destination
extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

/// Debug Exception (#DB) - Vector 1
///
/// Triggered by:
/// - Hardware breakpoints
/// - Single-step execution (TF flag)
extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

/// Non-Maskable Interrupt (NMI) - Vector 2
///
/// Cannot be disabled by CLI instruction.
/// Typically used for:
/// - Critical hardware errors
/// - Watchdog timers
/// - System profiling
extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NON-MASKABLE INTERRUPT (NMI)\n{:#?}", stack_frame);
}

/// Breakpoint (#BP) - Vector 3
///
/// Triggered by INT3 instruction.
/// Used by debuggers.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    // Breakpoint is recoverable, so we don't panic
    crate::serial_write(b"EXCEPTION: BREAKPOINT\n");
    // TODO: Integrate with debugger in future
}

/// Overflow (#OF) - Vector 4
///
/// Triggered by INTO instruction when OF flag is set.
extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

/// Bound Range Exceeded (#BR) - Vector 5
///
/// Triggered by BOUND instruction (legacy).
extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

/// Invalid Opcode (#UD) - Vector 6
///
/// Triggered by:
/// - Undefined or reserved opcode
/// - Instruction not supported by current CPU mode
extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

/// Device Not Available (#NM) - Vector 7
///
/// Triggered by:
/// - FPU instruction when FPU disabled (CR0.TS=1)
/// - WAIT instruction when CR0.MP=1 and CR0.TS=1
extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}

/// Double Fault (#DF) - Vector 8
///
/// Triggered when an exception occurs while handling another exception.
///
/// This is a critical exception that indicates something is seriously wrong
/// with exception handling. Common causes:
/// - Invalid IDT entry
/// - Invalid exception handler address
/// - Stack overflow during exception handling
/// - Exception during exception handler execution
///
/// This handler uses a dedicated IST stack to ensure it can always execute.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT (error_code: {:#x})\n{:#?}",
        error_code, stack_frame
    );
}

/// Invalid TSS (#TS) - Vector 10
///
/// Triggered by:
/// - Invalid TSS descriptor
/// - TSS limit violation
extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: INVALID TSS (error_code: {:#x})\n{:#?}",
        error_code, stack_frame
    );
}

/// Segment Not Present (#NP) - Vector 11
///
/// Triggered when accessing a segment with P (present) bit = 0.
extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: SEGMENT NOT PRESENT (error_code: {:#x})\n{:#?}",
        error_code, stack_frame
    );
}

/// Stack Segment Fault (#SS) - Vector 12
///
/// Triggered by:
/// - Stack limit violation
/// - Loading invalid SS selector
extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT (error_code: {:#x})\n{:#?}",
        error_code, stack_frame
    );
}

/// General Protection Fault (#GP) - Vector 13
///
/// Triggered by various protection violations:
/// - Segment limit violation
/// - Invalid descriptor type
/// - Privilege level violation
/// - Writing to read-only segment
///
/// This is a very common exception during kernel development!
extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // Decode error code for better diagnostics
    let selector_index = (error_code >> 3) & 0x1FFF;
    let is_external = (error_code & 0x1) != 0;
    let in_idt = (error_code & 0x2) != 0;

    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n\
         Error Code: {:#x}\n\
         Selector Index: {}\n\
         External: {}\n\
         In IDT: {}\n\
         {:#?}",
        error_code, selector_index, is_external, in_idt, stack_frame
    );
}

/// Page Fault (#PF) - Vector 14
///
/// Triggered by:
/// - Accessing non-present page (P=0)
/// - Writing to read-only page
/// - Executing non-executable page (NX=1)
/// - Reserved bit set in page table
/// - Instruction fetch from non-executable page
///
/// Error code format:
/// - Bit 0 (P): 0 = non-present page, 1 = protection violation
/// - Bit 1 (W/R): 0 = read, 1 = write
/// - Bit 2 (U/S): 0 = kernel mode, 1 = user mode
/// - Bit 3 (RSVD): 1 = reserved bit violation
/// - Bit 4 (I/D): 1 = instruction fetch
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    // CR2 contains the virtual address that caused the page fault
    let fault_addr = Cr2::read();

    panic!(
        "EXCEPTION: PAGE FAULT\n\
         Accessed Address: {:#x}\n\
         Error Code: {:?}\n\
         {:#?}",
        fault_addr, error_code, stack_frame
    );
}

/// x87 FPU Error (#MF) - Vector 16
///
/// Triggered by x87 FPU exceptions (divide by zero, overflow, etc.)
extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: x87 FLOATING POINT ERROR\n{:#?}", stack_frame);
}

/// Alignment Check (#AC) - Vector 17
///
/// Triggered when:
/// - Alignment checking is enabled (CR0.AM=1, RFLAGS.AC=1)
/// - Unaligned memory access is performed
extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: ALIGNMENT CHECK (error_code: {:#x})\n{:#?}",
        error_code, stack_frame
    );
}

/// Machine Check (#MC) - Vector 18
///
/// Triggered by serious hardware errors:
/// - Bus errors
/// - Cache errors
/// - Memory errors
extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("EXCEPTION: MACHINE CHECK (HARDWARE ERROR)\n{:#?}", stack_frame);
}

/// SIMD Floating-Point Exception (#XM) - Vector 19
///
/// Triggered by SSE/AVX floating-point exceptions
extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SIMD FLOATING POINT ERROR\n{:#?}", stack_frame);
}

/// Virtualization Exception (#VE) - Vector 20
///
/// Triggered by EPT violations in virtualized environments
extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: VIRTUALIZATION EXCEPTION\n{:#?}", stack_frame);
}

// Temporary helper to write to serial port
// Will be replaced with proper logging infrastructure
fn crate_serial_write(bytes: &[u8]) {
    // TODO: Use proper serial driver
    // For now, this is just a placeholder
    #[cfg(feature = "early-serial")]
    {
        for &byte in bytes {
            unsafe {
                core::arch::asm!(
                    "out dx, al",
                    in("dx") 0x3F8u16, // COM1
                    in("al") byte,
                    options(nomem, nostack, preserves_flags)
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idt_loaded() {
        // Verify IDT was created successfully
        // In a real test, we would also verify it's loaded into the CPU
        let _ = &*IDT;
    }
}
