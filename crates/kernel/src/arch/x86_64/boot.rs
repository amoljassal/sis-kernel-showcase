//! # x86_64 Boot Sequence
//!
//! This module handles the early boot sequence for x86_64 systems.
//! It coordinates the initialization of all architecture-specific components
//! before handing control to the platform-independent kernel code.
//!
//! ## Boot Flow
//!
//! ```text
//! UEFI Firmware (OVMF)
//!     ↓
//! UEFI Boot Application (uefi-boot crate)
//!     ↓ Loads kernel ELF into memory
//!     ↓ Sets up initial page tables
//!     ↓ Exits boot services
//!     ↓ Jumps to kernel entry point
//! Kernel Entry Point (_start)
//!     ↓
//! arch_early_init() [THIS MODULE]
//!     ├── 1. Disable interrupts
//!     ├── 2. Load GDT
//!     ├── 3. Load TSS
//!     ├── 4. Load IDT
//!     ├── 5. Enable CPU features (SSE, AVX, NX)
//!     ├── 6. Initialize serial console
//!     └── 7. Initialize TSC
//!     ↓
//! Platform Init (kernel/main.rs)
//!     ├── Initialize heap allocator
//!     ├── Initialize memory management
//!     ├── Initialize process subsystem
//!     └── Initialize drivers
//!     ↓
//! Userspace Init
//! ```
//!
//! ## Memory State on Entry
//!
//! When the kernel entry point is called:
//! - CPU is in 64-bit long mode
//! - Paging is enabled (identity-mapped by bootloader)
//! - Interrupts are disabled
//! - Stack is valid and properly aligned
//! - Bootloader has provided memory map and other boot information
//!
//! ## Responsibilities
//!
//! This module is responsible for:
//! 1. Setting up CPU execution environment (GDT, IDT, TSS)
//! 2. Enabling required CPU features
//! 3. Initializing early console for debugging
//! 4. Validating hardware compatibility
//! 5. Transitioning to platform-independent code
//!
//! ## Safety Considerations
//!
//! Boot code runs in a very constrained environment:
//! - No heap allocation available yet
//! - No interrupts (must use polling for I/O)
//! - No exception handling (triple fault = reset)
//! - Limited stack space
//! - Must not use floating-point until SSE is enabled

use crate::arch::x86_64::{gdt, idt, tss, cpu, serial, tsc};

/// Early architecture initialization
///
/// This is the first Rust function called after the UEFI bootloader hands
/// control to the kernel. It sets up the minimal execution environment needed
/// for the rest of the kernel to function.
///
/// # Initialization Steps
///
/// 1. **Disable Interrupts**: Ensure no interrupts occur during setup
/// 2. **Load GDT**: Set up segmentation (required even in long mode)
/// 3. **Load TSS**: Enable privilege level transitions
/// 4. **Load IDT**: Set up exception handlers (prevent triple fault)
/// 5. **Enable CPU Features**: SSE, AVX, NX, SMEP, SMAP, etc.
/// 6. **Initialize Serial**: Set up COM1 for early logging
/// 7. **Initialize TSC**: Calibrate time stamp counter
///
/// # Returns
///
/// - `Ok(())` if initialization succeeded
/// - `Err(&str)` with error message if critical failure occurred
///
/// # Safety
///
/// This function must be called exactly once during boot, before any other
/// kernel code runs. It must be called with:
/// - Interrupts disabled
/// - Valid stack
/// - CPU in 64-bit long mode
/// - Paging enabled
///
/// # Panics
///
/// Will panic if:
/// - Required CPU features are missing (SSE2, APIC)
/// - Hardware initialization fails
/// - Validation checks fail
pub unsafe fn early_init() -> Result<(), &'static str> {
    // Step 1: Ensure interrupts are disabled
    // This is critical - we can't handle interrupts until IDT is set up
    x86_64::instructions::interrupts::disable();

    // Step 2: Load Global Descriptor Table (GDT)
    // The GDT defines memory segments. Even though segmentation is mostly
    // legacy in 64-bit mode, we still need valid GDT entries for:
    // - Code segment (CS)
    // - Data segments (DS, ES, SS)
    // - TSS (for privilege transitions)
    gdt::init_gdt();

    // Step 3: Load Task State Segment (TSS)
    // The TSS is required for:
    // - Switching between privilege levels (user ↔ kernel)
    // - Providing dedicated stacks for critical exceptions (double fault, NMI)
    tss::init_tss();

    // Step 4: Load Interrupt Descriptor Table (IDT)
    // The IDT defines handlers for all exceptions and interrupts.
    // Without this, any exception would cause a triple fault (CPU reset).
    idt::init_idt_early();

    // Step 5: Enable CPU features
    // This enables required features (SSE2, etc.) and optional features
    // (AVX, NX, SMEP, SMAP) if available.
    cpu::enable_cpu_features()?;

    // Step 6: Initialize serial console
    // COM1 (0x3F8) is used for early kernel logging and debugging.
    // This must come after CPU init (to enable any required features).
    serial::init_serial()?;

    // Print boot banner
    serial::serial_write(b"\n");
    serial::serial_write(b"================================================================================\n");
    serial::serial_write(b"                         SIS Kernel - x86_64 Architecture\n");
    serial::serial_write(b"================================================================================\n");
    serial::serial_write(b"\n");
    serial::serial_write(b"[BOOT] Early initialization started\n");
    serial::serial_write(b"[BOOT] GDT loaded\n");
    serial::serial_write(b"[BOOT] TSS loaded\n");
    serial::serial_write(b"[BOOT] IDT loaded\n");
    serial::serial_write(b"[BOOT] CPU features enabled\n");
    serial::serial_write(b"[BOOT] Serial console initialized\n");

    // Step 7: Print CPU information
    cpu::print_cpu_info();

    // Step 8: Initialize HPET (High Precision Event Timer)
    // Try to initialize HPET for high-precision timing
    // HPET is optional - system will fall back to PIT if not available
    let hpet_available = match crate::arch::x86_64::hpet::init() {
        Ok(()) => {
            serial::serial_write(b"[BOOT] HPET initialized successfully\\n");
            true
        }
        Err(e) => {
            serial::serial_write(b"[BOOT] HPET not available: ");
            serial::serial_write(e.as_bytes());
            serial::serial_write(b"\\n");
            false
        }
    };

    // Step 9: Initialize Time Stamp Counter (TSC)
    // Calibrate TSC for accurate timekeeping
    // TSC calibration will use HPET if available, otherwise PIT
    tsc::init_tsc();

    // Validate TSS configuration (debug builds only)
    #[cfg(debug_assertions)]
    {
        tss::validate_tss()?;
        serial::serial_write(b"[BOOT] TSS validation passed\n");
    }

    // M1: Initialize interrupt handling
    serial::serial_write(b"\n[BOOT] Milestone M1: Interrupt Handling\n");

    // Step 10: Initialize legacy PIC (8259A)
    // Remap PIC to vectors 32-47 to avoid conflicts with CPU exceptions
    crate::arch::x86_64::pic::init();

    // Step 11: Initialize PIT (Programmable Interval Timer)
    // Configure for 1000 Hz (1 ms per tick)
    crate::arch::x86_64::pit::init(1000);

    // M2: Initialize APIC & High Precision Timer
    serial::serial_write(b"\n[BOOT] Milestone M2: APIC & High Precision Timer\n");

    // Step 12: Initialize Local APIC
    // Try to initialize APIC for modern interrupt handling
    // APIC is preferred over PIC but system will fall back if not available
    let apic_available = match crate::arch::x86_64::apic::init() {
        Ok(()) => {
            serial::serial_write(b"[BOOT] Local APIC initialized successfully\n");
            true
        }
        Err(e) => {
            serial::serial_write(b"[BOOT] Local APIC not available: ");
            serial::serial_write(e.as_bytes());
            serial::serial_write(b"\n[BOOT] Falling back to legacy PIC\n");
            false
        }
    };

    // Step 13: Configure timer interrupt
    if apic_available {
        // Use APIC timer (future work - for now still use PIT)
        // APIC timer will be configured in future milestones
        serial::serial_write(b"[BOOT] Using PIT timer with APIC (APIC timer not yet configured)\n");
        crate::arch::x86_64::pic::enable_irq(crate::arch::x86_64::pic::Irq::Timer);
    } else {
        // Use PIT with PIC
        crate::arch::x86_64::pic::enable_irq(crate::arch::x86_64::pic::Irq::Timer);
    }

    // Step 14: Enable interrupts globally
    serial::serial_write(b"[BOOT] Enabling interrupts...\n");
    x86_64::instructions::interrupts::enable();

    serial::serial_write(b"[BOOT] Interrupts enabled\n");
    serial::serial_write(b"[BOOT] Early initialization complete\n");
    serial::serial_write(b"\n");

    Ok(())
}

/// Validate hardware compatibility
///
/// Checks that the system has all required hardware features for the kernel
/// to function correctly.
///
/// # Returns
///
/// - `Ok(())` if all required features are present
/// - `Err(&str)` with description of missing feature
pub fn validate_hardware() -> Result<(), &'static str> {
    let features = cpu::detect_cpu_features();

    // Check required features
    if !features.has_sse2 {
        return Err("CPU does not support SSE2 (required for Rust)");
    }

    if !features.has_apic {
        return Err("CPU does not support APIC (required for interrupts)");
    }

    if !features.has_tsc {
        return Err("CPU does not support TSC (required for timekeeping)");
    }

    Ok(())
}

/// Print boot information
///
/// Displays useful information about the system configuration.
pub fn print_boot_info() {
    serial::serial_write(b"[BOOT] Boot Information:\n");

    // Print memory layout
    serial::serial_write(b"[BOOT] Memory Layout:\n");
    serial::serial_write(b"       Kernel Image:  0xFFFF_8000_0000_0000 - 0xFFFF_8800_0000_0000 (512 GB)\n");
    serial::serial_write(b"       Kernel Heap:   0xFFFF_8800_0000_0000 - 0xFFFF_9000_0000_0000 (512 GB)\n");
    serial::serial_write(b"       Device MMIO:   0xFFFF_9000_0000_0000 - 0xFFFF_A000_0000_0000 (1 TB)\n");
    serial::serial_write(b"       PCI ECAM:      0xFFFF_A000_0000_0000 - 0xFFFF_B000_0000_0000 (1 TB)\n");
    serial::serial_write(b"       Per-CPU Data:  0xFFFF_B000_0000_0000 - 0xFFFF_C000_0000_0000 (1 TB)\n");
    serial::serial_write(b"       Direct Map:    0xFFFF_FFFF_8000_0000 - 0xFFFF_FFFF_FFFF_FFFF (512 GB)\n");
    serial::serial_write(b"\n");

    // Print TSC frequency
    let tsc_freq = tsc::get_tsc_frequency();
    if tsc_freq > 0 {
        serial::serial_write(b"[BOOT] TSC Frequency: ");
        print_u64(tsc_freq / 1_000_000);
        serial::serial_write(b" MHz\n");
    }

    serial::serial_write(b"\n");
}

/// Helper function to print u64 to serial (temporary)
fn print_u64(mut n: u64) {
    if n == 0 {
        serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        serial::serial_write_byte(buf[i]);
    }
}

/// Halt the CPU forever
///
/// Used when a critical error occurs during boot and recovery is not possible.
pub fn halt_forever() -> ! {
    serial::serial_write(b"\n[BOOT] FATAL ERROR - System halted\n");

    loop {
        x86_64::instructions::interrupts::disable();
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hardware() {
        // Should pass on any x86_64 system
        assert!(validate_hardware().is_ok());
    }
}
