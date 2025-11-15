//! # x86_64 Architecture Support
//!
//! This module provides x86_64 (Intel/AMD 64-bit) architecture support for the SIS kernel.
//! It implements the platform-specific functionality required for booting, exception handling,
//! memory management, and device interaction on x86_64 systems.
//!
//! ## Architecture Overview
//!
//! The x86_64 implementation follows these key principles:
//! - **UEFI-first boot**: Leverages UEFI firmware (OVMF in QEMU) for initialization
//! - **Modern features**: Uses APIC (not legacy PIC), SYSCALL/SYSRET (not INT 0x80)
//! - **Security-focused**: NX bit, SMEP/SMAP where available, no red zone
//! - **SMP-ready**: Multi-core support via INIT-SIPI-SIPI protocol
//!
//! ## Boot Flow
//!
//! ```text
//! OVMF UEFI Firmware
//!     ↓
//! UEFI Boot Application (crates/uefi-boot)
//!     ↓ (loads kernel ELF)
//! Kernel Entry (64-bit long mode, paging enabled)
//!     ↓
//! arch_early_init() - This module
//!     ├── Disable interrupts
//!     ├── Load GDT (Global Descriptor Table)
//!     ├── Load IDT (Interrupt Descriptor Table)
//!     ├── Enable CPU features (SSE, AVX, NX)
//!     ├── Initialize serial console (16550 UART)
//!     └── Load TSS (Task State Segment)
//!     ↓
//! Platform Detection (ACPI/CPUID)
//!     ↓
//! Core Init (APIC, Timer, Paging)
//!     ↓
//! Driver Init (PCI, VirtIO, Block, Net)
//!     ↓
//! Userspace Init (Shell, AgentSys)
//! ```
//!
//! ## Memory Layout
//!
//! Virtual address space uses 48-bit canonical addresses:
//!
//! ```text
//! 0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_F000   User Space (128 TB)
//! 0x0000_7FFF_FFFF_F000 - 0x0000_8000_0000_0000   Guard Page
//!
//! [Canonical Address Hole - Invalid addresses]
//!
//! 0xFFFF_8000_0000_0000 - 0xFFFF_8800_0000_0000   Kernel Image (512 GB)
//! 0xFFFF_8800_0000_0000 - 0xFFFF_9000_0000_0000   Kernel Heap (512 GB)
//! 0xFFFF_9000_0000_0000 - 0xFFFF_A000_0000_0000   Device MMIO (1 TB)
//! 0xFFFF_A000_0000_0000 - 0xFFFF_B000_0000_0000   PCI ECAM Space (1 TB)
//! 0xFFFF_B000_0000_0000 - 0xFFFF_C000_0000_0000   Per-CPU Data (1 TB)
//! 0xFFFF_C000_0000_0000 - 0xFFFF_FFFF_8000_0000   Reserved
//! 0xFFFF_FFFF_8000_0000 - 0xFFFF_FFFF_FFFF_FFFF   Direct Map (512 GB)
//! ```
//!
//! ## Module Organization
//!
//! - `cpu`: CPU initialization and feature detection (SSE, AVX, NX, FSGSBASE)
//! - `gdt`: Global Descriptor Table setup (kernel/user code/data segments, TSS)
//! - `idt`: Interrupt Descriptor Table (exceptions 0-31, hardware IRQs 32-255)
//! - `tss`: Task State Segment (privilege level switching, IST stacks)
//! - `serial`: 16550 UART driver for COM1/COM2 serial ports
//! - `pic`: Legacy 8259A PIC support (for early boot, disabled after APIC init)
//! - `apic`: Local APIC and I/O APIC (modern interrupt handling)
//! - `pit`: Programmable Interval Timer (8254 PIT for early timekeeping)
//! - `hpet`: High Precision Event Timer (accurate timekeeping)
//! - `tsc`: Time Stamp Counter (CPU cycle counter, calibrated)
//! - `paging`: 4-level page tables (PML4 → PDPT → PD → PT)
//! - `syscall`: SYSCALL/SYSRET fast system call entry
//! - `smp`: Symmetric Multi-Processing (AP startup via INIT-SIPI-SIPI)
//! - `percpu`: Per-CPU data structures (accessed via GS segment)
//! - `acpi`: ACPI table parsing (RSDP, MADT, HPET, MCFG, FADT)
//! - `power`: Power management (reset, shutdown via ACPI)
//!
//! ## Feature Support
//!
//! ### Required Features
//! - 64-bit long mode
//! - SSE2 (required for Rust floating-point)
//! - APIC (Local APIC for interrupts)
//!
//! ### Optional Features (detected via CPUID)
//! - SSE3, SSE4.1, SSE4.2, AVX, AVX2 (SIMD performance)
//! - Execute Disable (NX bit for security)
//! - FSGSBASE (fast per-CPU data access)
//! - x2APIC (MSR-based APIC, better than memory-mapped)
//! - PCID (Process-Context Identifiers for TLB efficiency)
//! - INVPCID (selective TLB invalidation)
//! - SMEP/SMAP (Supervisor Mode Execution/Access Prevention)
//!
//! ## Exception Handling
//!
//! All CPU exceptions (0-31) are handled:
//! - `#DE` Divide Error (0)
//! - `#DB` Debug (1)
//! - `NMI` Non-Maskable Interrupt (2)
//! - `#BP` Breakpoint (3)
//! - `#OF` Overflow (4)
//! - `#BR` Bound Range Exceeded (5)
//! - `#UD` Invalid Opcode (6)
//! - `#NM` Device Not Available (7)
//! - `#DF` Double Fault (8) - **uses dedicated IST stack**
//! - `#TS` Invalid TSS (10)
//! - `#NP` Segment Not Present (11)
//! - `#SS` Stack Segment Fault (12)
//! - `#GP` General Protection Fault (13)
//! - `#PF` Page Fault (14) - **primary memory management exception**
//! - `#MF` x87 FPU Error (16)
//! - `#AC` Alignment Check (17)
//! - `#MC` Machine Check (18)
//! - `#XM` SIMD Floating-Point Exception (19)
//! - `#VE` Virtualization Exception (20)
//!
//! ## Interrupt Handling
//!
//! Hardware interrupts (32-255):
//! - **32-47**: Legacy PIC IRQs (if PIC mode)
//! - **48-255**: APIC vectors (configurable)
//!   - Timer interrupt
//!   - Keyboard interrupt
//!   - Serial interrupt
//!   - PCI device interrupts (MSI/MSI-X)
//!   - IPIs (Inter-Processor Interrupts for SMP)
//!
//! ## System Call Interface
//!
//! Uses `SYSCALL/SYSRET` instructions (not `INT 0x80`):
//! - **Entry**: `SYSCALL` instruction from userspace
//! - **Exit**: `SYSRET` instruction back to userspace
//! - **Arguments**: System V ABI (RDI, RSI, RDX, R10, R8, R9)
//! - **Return**: RAX (signed, negative = error code)
//! - **Stack**: Kernel stack loaded from TSS.RSP0
//!
//! ## Safety Considerations
//!
//! This module contains extensive unsafe code for:
//! - CPU control register manipulation (CR0, CR3, CR4, EFER)
//! - Model-Specific Register (MSR) access
//! - Memory-mapped I/O to devices
//! - Assembly language integration
//! - Interrupt/exception handler registration
//!
//! All unsafe operations are carefully documented and follow these rules:
//! 1. Validate inputs before hardware operations
//! 2. Ensure proper ordering with memory barriers
//! 3. Disable interrupts during critical sections
//! 4. Use volatile operations for MMIO
//! 5. Document safety invariants clearly

// Submodules
pub mod cpu;      // CPU initialization and features
pub mod gdt;      // Global Descriptor Table
pub mod idt;      // Interrupt Descriptor Table
pub mod tss;      // Task State Segment
pub mod serial;   // 16550 UART driver
pub mod boot;     // Boot sequence and early init

// M1: Interrupts & Exceptions (COMPLETE)
pub mod pic;      // Legacy 8259A PIC
pub mod pit;      // Programmable Interval Timer

// M2: APIC & High Precision Timer (COMPLETE)
pub mod apic;     // Local APIC (xAPIC/x2APIC)
pub mod hpet;     // High Precision Event Timer

#[cfg(feature = "m3-complete")]
pub mod paging;   // 4-level page tables (M3: Paging)

#[cfg(feature = "m4-complete")]
pub mod syscall;  // SYSCALL/SYSRET entry (M4: Syscalls)

#[cfg(feature = "m8-complete")]
pub mod smp;      // SMP support (M8: SMP)
#[cfg(feature = "m8-complete")]
pub mod percpu;   // Per-CPU data (M8: SMP)

#[cfg(feature = "m9-complete")]
pub mod acpi;     // ACPI tables (M9: ACPI)
#[cfg(feature = "m9-complete")]
pub mod power;    // Power management (M9: ACPI)

// Always compile TSC support (used for timekeeping)
pub mod tsc;      // Time Stamp Counter

// Re-exports for common use
pub use cpu::*;
pub use gdt::init_gdt;
pub use idt::init_idt_early;
pub use tss::init_tss;
pub use serial::{init_serial, serial_write, serial_read};

use x86_64::instructions::interrupts;

/// CPU context for context switching
/// Contains callee-saved registers that must be preserved across function calls
/// according to the System V AMD64 ABI
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    // Callee-saved general-purpose registers
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Stack pointer
    pub rsp: u64,

    // Instruction pointer (return address)
    pub rip: u64,

    // RFLAGS (processor flags)
    pub rflags: u64,

    // FS and GS base (for TLS and per-CPU data)
    pub fs_base: u64,
    pub gs_base: u64,
}

impl CpuContext {
    /// Create a new empty context
    pub const fn new() -> Self {
        Self {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp: 0,
            rip: 0,
            rflags: 0,
            fs_base: 0,
            gs_base: 0,
        }
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

extern "C" {
    /// Context switch function (will be implemented in switch.S)
    /// Saves current context to prev, restores from next
    pub fn switch_to(prev: *mut CpuContext, next: *const CpuContext);
}

/// Architecture early initialization (Milestone M0)
///
/// This function is called very early in the boot process, immediately after
/// the UEFI bootloader hands off control to the kernel. At this point:
/// - CPU is in 64-bit long mode
/// - Paging is enabled (identity-mapped by bootloader)
/// - Interrupts are disabled
/// - Stack is valid
///
/// # Safety
/// Must be called exactly once during boot, before any other kernel code runs.
pub unsafe fn arch_early_init() -> Result<(), &'static str> {
    // 1. Ensure interrupts are disabled (safety first)
    interrupts::disable();

    // 2. Load GDT (required before any segment operations)
    gdt::init_gdt();

    // 3. Load TSS (required for privilege level switching and IST)
    tss::init_tss();

    // 4. Load empty IDT (prevents triple fault on exceptions)
    idt::init_idt_early();

    // 5. Enable CPU features required by Rust and kernel
    cpu::enable_cpu_features()?;

    // 6. Initialize early serial console
    serial::init_serial()?;

    // Print boot banner
    serial_write(b"\n");
    serial_write(b"========================================\n");
    serial_write(b"  SIS Kernel - x86_64 Architecture\n");
    serial_write(b"========================================\n");
    serial_write(b"[x86_64] Early initialization complete\n");
    serial_write(b"[x86_64] GDT loaded\n");
    serial_write(b"[x86_64] TSS loaded\n");
    serial_write(b"[x86_64] IDT initialized\n");
    serial_write(b"[x86_64] CPU features enabled\n");
    serial_write(b"[x86_64] Serial console ready\n");

    Ok(())
}

/// Halt the CPU until the next interrupt
#[inline]
pub fn halt() {
    x86_64::instructions::hlt();
}

/// Halt the CPU forever (no interrupts)
#[inline]
pub fn halt_loop() -> ! {
    loop {
        interrupts::disable();
        halt();
    }
}

/// Read the current CPU cycle counter (TSC)
#[inline]
pub fn read_tsc() -> u64 {
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
}

/// Read Model-Specific Register
#[inline]
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let (high, low): (u32, u32);
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}

/// Write Model-Specific Register
#[inline]
pub unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack, preserves_flags)
    );
}

/// Invalidate TLB entry for a specific virtual address
#[inline]
pub fn invlpg(addr: u64) {
    unsafe {
        core::arch::asm!(
            "invlpg [{}]",
            in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}

/// Flush entire TLB by reloading CR3
#[inline]
pub fn flush_tlb() {
    use x86_64::registers::control::Cr3;
    let (frame, flags) = Cr3::read();
    unsafe {
        Cr3::write(frame, flags);
    }
}
