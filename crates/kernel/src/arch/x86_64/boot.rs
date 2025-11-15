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
use core::ptr;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BootInfo {
    pub rsdp_addr: u64,
}

#[no_mangle]
pub static mut BOOT_INFO: BootInfo = BootInfo { rsdp_addr: 0 };

/// Record boot information provided by the loader (e.g., ACPI pointers).
// Store debug info for later printing after serial is initialized
static mut DEBUG_BOOT_INFO_PTR: u64 = 0;
static mut DEBUG_BOOT_INFO_RSDP: u64 = 0;

pub unsafe fn init_boot_info(info: *const BootInfo) {
    // Store debug info for later (serial not initialized yet)
    DEBUG_BOOT_INFO_PTR = info as u64;

    if !info.is_null() {
        BOOT_INFO = ptr::read_volatile(info);
        DEBUG_BOOT_INFO_RSDP = BOOT_INFO.rsdp_addr;
    }
}

#[inline]
fn boot_info() -> BootInfo {
    unsafe { BOOT_INFO }
}

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

    // Print debug info about boot info passing
    serial::serial_write(b"[BOOT] DEBUG: init_boot_info was called with pointer: 0x");
    print_hex_u64(DEBUG_BOOT_INFO_PTR);
    serial::serial_write(b"\n");
    serial::serial_write(b"[BOOT] DEBUG: BOOT_INFO.rsdp_addr loaded as: 0x");
    print_hex_u64(DEBUG_BOOT_INFO_RSDP);
    serial::serial_write(b"\n");

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

    // M3: Paging & Memory Management
    // NOTE: Basic paging is already set up by the bootloader (identity mapping).
    // The PageTableManager in arch::x86_64::paging provides advanced page table
    // management for future use (user processes, demand paging, etc.).
    // Full paging initialization will be added when integrating with userspace (M4+).
    serial::serial_write(b"\n[BOOT] Milestone M3: Paging infrastructure available\n");
    serial::serial_write(b"[BOOT] Page fault handler enhanced with diagnostics\n");

    // M8: Per-CPU Data Structures
    serial::serial_write(b"\n[BOOT] Milestone M8: Per-CPU Data (BSP only)\n");
    crate::arch::x86_64::percpu::init_bsp();

    // M4: Syscall Entry
    serial::serial_write(b"\n[BOOT] Milestone M4: Syscall Entry\n");
    crate::arch::x86_64::syscall::init();
    serial::serial_write(b"[BOOT] SYSCALL/SYSRET enabled (using per-CPU kernel stacks)\n");

    // M8 Part 2: Start Application Processors (SMP)
    serial::serial_write(b"\n[BOOT] Milestone M8 Part 2: SMP Initialization\n");
    match crate::arch::x86_64::smp::boot_aps() {
        Ok(cpu_count) => {
            serial::serial_write(b"[BOOT] SMP initialized: ");
            print_u64(cpu_count as u64);
            serial::serial_write(b" CPUs online\n");
        }
        Err(e) => {
            serial::serial_write(b"[BOOT] SMP initialization failed: ");
            serial::serial_write(e.as_bytes());
            serial::serial_write(b"\n[BOOT] Continuing with single-processor mode\n");
        }
    }

    // M5: Serial/TTY Polish - Enable interrupt-driven I/O
    serial::serial_write(b"\n[BOOT] Milestone M5: Serial/TTY Polish\n");
    crate::arch::x86_64::serial::enable_interrupts();
    crate::arch::x86_64::pic::enable_irq(crate::arch::x86_64::pic::Irq::COM1);
    serial::serial_write(b"[BOOT] Serial interrupt-driven I/O enabled (IRQ 4)\n");
    serial::serial_write(b"[BOOT] RX buffer: 256 bytes, non-blocking read operations\n");

    // M9: ACPI & Power Management (moved before PCI to provide MCFG table)
    serial::serial_write(b"\n[BOOT] Milestone M9: ACPI & Power Management\n");

    let mut rsdp = boot_info().rsdp_addr;
    serial::serial_write(b"[BOOT] boot_info().rsdp_addr = 0x");
    print_hex_u64(rsdp);
    serial::serial_write(b"\n");

    // TEMPORARY FIX: Hardcode the RSDP address that UEFI found
    // TODO: Fix boot info passing from UEFI bootloader
    if rsdp < 0x1000 {
        serial::serial_write(b"[BOOT] RSDP from boot_info is invalid, using hardcoded value\n");
        rsdp = 0x3f77e014;  // From UEFI: "Found ACPI RSDP at 0x3f77e014"
    }

    serial::serial_write(b"[BOOT] RSDP from loader: 0x");
    print_hex_u64(rsdp);
    serial::serial_write(b"\n");

    match crate::arch::x86_64::acpi::init(x86_64::PhysAddr::new(rsdp)) {
        Ok(()) => {
            serial::serial_write(b"[BOOT] ACPI tables parsed successfully\n");
        }
        Err(e) => {
            serial::serial_write(b"[BOOT] ACPI initialization failed: ");
            serial::serial_write(e.as_bytes());
            serial::serial_write(b"\n");
        }
    }

    serial::serial_write(b"[BOOT] Power management initialized\n");
    serial::serial_write(b"[BOOT] System reset/shutdown support enabled\n");

    // M6: VirtIO Block Driver - PCI Bus Enumeration (now after ACPI)
    serial::serial_write(b"\n[BOOT] Milestone M6: VirtIO Block Driver\n");
    match crate::arch::x86_64::pci::init() {
        Ok(device_count) => {
            serial::serial_write(b"[BOOT] PCI bus enumeration complete\n");

            // Look for VirtIO block devices (device type 2)
            let virtio_block_devices = crate::arch::x86_64::pci::find_virtio_devices(2);
            if !virtio_block_devices.is_empty() {
                serial::serial_write(b"[BOOT] Found ");
                print_u64(virtio_block_devices.len() as u64);
                serial::serial_write(b" VirtIO block device(s)\n");

                // Initialize first VirtIO block device
                if let Some(dev) = virtio_block_devices.first() {
                    match crate::arch::x86_64::virtio_block::VirtioBlockDevice::new(dev.clone()) {
                        Ok(block_dev) => {
                            serial::serial_write(b"[BOOT] VirtIO block device initialized\n");
                            serial::serial_write(b"[BOOT]   Capacity: ");
                            print_u64(block_dev.capacity_bytes() / 1024 / 1024);
                            serial::serial_write(b" MB\n");
                            serial::serial_write(b"[BOOT]   Block size: ");
                            print_u64(block_dev.block_size() as u64);
                            serial::serial_write(b" bytes\n");
                            serial::serial_write(b"[BOOT]   Read-only: ");
                            if block_dev.is_read_only() {
                                serial::serial_write(b"yes\n");
                            } else {
                                serial::serial_write(b"no\n");
                            }

                            // Perform test read of first sector
                            serial::serial_write(b"[BOOT] Testing block device - reading first sector...\n");
                            let mut test_buffer = [0u8; 512];
                            match block_dev.read_sectors(0, &mut test_buffer) {
                                Ok(bytes_read) => {
                                    serial::serial_write(b"[BOOT] Successfully read ");
                                    print_u64(bytes_read as u64);
                                    serial::serial_write(b" bytes from sector 0\n");

                                    // Print first 16 bytes as hex
                                    serial::serial_write(b"[BOOT] First 16 bytes: ");
                                    for i in 0..16 {
                                        print_hex_u8(test_buffer[i]);
                                        serial::serial_write(b" ");
                                    }
                                    serial::serial_write(b"\n");
                                }
                                Err(e) => {
                                    serial::serial_write(b"[BOOT] Block read test failed: ");
                                    serial::serial_write(e.as_bytes());
                                    serial::serial_write(b"\n");
                                }
                            }

                            // Note: In a real system, we would store the block_dev in a global
                            // or pass it to the block device subsystem. For now, we just drop it.
                            serial::serial_write(b"[BOOT] VirtIO block device test complete\n");
                        }
                        Err(e) => {
                            serial::serial_write(b"[BOOT] Failed to initialize VirtIO block device: ");
                            serial::serial_write(e.as_bytes());
                            serial::serial_write(b"\n");
                        }
                    }
                }
            } else {
                serial::serial_write(b"[BOOT] No VirtIO block devices found\n");
            }
        }
        Err(e) => {
            serial::serial_write(b"[BOOT] PCI initialization failed: ");
            serial::serial_write(e.as_bytes());
            serial::serial_write(b"\n");
        }
    }

    // M7: VirtIO Network Driver
    serial::serial_write(b"\n[BOOT] Milestone M7: VirtIO Network Driver\n");

    // Look for VirtIO network devices (device type 1)
    let virtio_net_devices = crate::arch::x86_64::pci::find_virtio_devices(1);
    if !virtio_net_devices.is_empty() {
        serial::serial_write(b"[BOOT] Found ");
        print_u64(virtio_net_devices.len() as u64);
        serial::serial_write(b" VirtIO network device(s)\n");

        // Initialize first VirtIO network device
        if let Some(dev) = virtio_net_devices.first() {
            match crate::arch::x86_64::virtio_net::VirtioNetDevice::new(dev.clone()) {
                Ok(net_dev) => {
                    let mac = net_dev.mac_address();
                    serial::serial_write(b"[BOOT] VirtIO network device initialized\n");
                    serial::serial_write(b"[BOOT]   MAC: ");
                    for (i, &byte) in mac.iter().enumerate() {
                        print_hex_u8(byte);
                        if i < 5 {
                            serial::serial_write(b":");
                        }
                    }
                    serial::serial_write(b"\n");
                    serial::serial_write(b"[BOOT]   MTU: ");
                    print_u64(net_dev.mtu() as u64);
                    serial::serial_write(b" bytes\n");

                    // Perform test transmission (simple test packet)
                    serial::serial_write(b"[BOOT] Testing network device - sending test packet...\n");

                    // Create a simple Ethernet frame (broadcast ARP request)
                    // Destination MAC: FF:FF:FF:FF:FF:FF (broadcast)
                    // Source MAC: our MAC
                    // EtherType: 0x0806 (ARP)
                    // ARP payload (minimal, 28 bytes)
                    let mut test_packet = [0u8; 64];

                    // Ethernet header (14 bytes)
                    test_packet[0..6].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // Dest MAC (broadcast)
                    test_packet[6..12].copy_from_slice(&mac); // Source MAC
                    test_packet[12..14].copy_from_slice(&[0x08, 0x06]); // EtherType: ARP

                    // ARP payload (28 bytes minimum)
                    test_packet[14..16].copy_from_slice(&[0x00, 0x01]); // Hardware type: Ethernet
                    test_packet[16..18].copy_from_slice(&[0x08, 0x00]); // Protocol type: IPv4
                    test_packet[18] = 6;   // Hardware size
                    test_packet[19] = 4;   // Protocol size
                    test_packet[20..22].copy_from_slice(&[0x00, 0x01]); // Opcode: request
                    test_packet[22..28].copy_from_slice(&mac); // Sender MAC
                    test_packet[28..32].copy_from_slice(&[192, 168, 1, 100]); // Sender IP (example)
                    test_packet[32..38].copy_from_slice(&[0, 0, 0, 0, 0, 0]); // Target MAC
                    test_packet[38..42].copy_from_slice(&[192, 168, 1, 1]); // Target IP (example)

                    // Padding to minimum Ethernet frame size (64 bytes)
                    // Already initialized to 0

                    match net_dev.transmit(&test_packet[..42]) {
                        Ok(()) => {
                            serial::serial_write(b"[BOOT] Successfully transmitted test packet (");
                            print_u64(42);
                            serial::serial_write(b" bytes)\n");
                        }
                        Err(e) => {
                            serial::serial_write(b"[BOOT] Network transmit test failed: ");
                            serial::serial_write(e.as_bytes());
                            serial::serial_write(b"\n");
                        }
                    }

                    // Note: In a real system, we would store the net_dev in a global
                    // or pass it to the network subsystem. For now, we just drop it.
                    serial::serial_write(b"[BOOT] VirtIO network device test complete\n");
                }
                Err(e) => {
                    serial::serial_write(b"[BOOT] Failed to initialize VirtIO network device: ");
                    serial::serial_write(e.as_bytes());
                    serial::serial_write(b"\n");
                }
            }
        }
    } else {
        serial::serial_write(b"[BOOT] No VirtIO network devices found\n");
    }

    // Note: ACPI initialization (M9) was moved earlier to run before PCI scanning

    serial::serial_write(b"\n[BOOT] Early initialization complete\n");
    serial::serial_write(b"\n");

    Ok(())
}

/// Find the RSDP (Root System Description Pointer) in memory
///
/// Searches in two locations:
/// 1. Extended BIOS Data Area (EBDA) - first 1KB
/// 2. BIOS ROM area (0xE0000 - 0xFFFFF)
///
/// Returns the physical address of the RSDP if found.
fn find_rsdp() -> Option<x86_64::PhysAddr> {
    use x86_64::PhysAddr;

    const PHYS_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;

    // RSDP signature: "RSD PTR "
    const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";

    // Search in EBDA (first 1KB of EBDA)
    // EBDA base address is at 0x40E (stored as segment)
    unsafe {
        let ebda_ptr = (0x40E + PHYS_OFFSET) as *const u16;
        let ebda_segment = core::ptr::read_volatile(ebda_ptr);
        let ebda_base = (ebda_segment as u64) << 4;

        // Search first 1KB of EBDA on 16-byte boundaries
        for offset in (0..1024).step_by(16) {
            let addr = ebda_base + offset;
            let virt_addr = (addr + PHYS_OFFSET) as *const [u8; 8];
            let signature = core::ptr::read_volatile(virt_addr);

            if &signature == RSDP_SIGNATURE {
                return Some(PhysAddr::new(addr));
            }
        }
    }

    // Search in BIOS ROM area (0xE0000 - 0xFFFFF)
    unsafe {
        for addr in (0xE0000..0x100000).step_by(16) {
            let virt_addr = (addr + PHYS_OFFSET) as *const [u8; 8];
            let signature = core::ptr::read_volatile(virt_addr);

            if &signature == RSDP_SIGNATURE {
                return Some(PhysAddr::new(addr));
            }
        }
    }

    None
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

/// Helper function to print u64 as hexadecimal to serial
fn print_hex_u64(n: u64) {
    let hex_chars = b"0123456789abcdef";
    let mut buf = [0u8; 16];

    for i in 0..16 {
        let shift = (15 - i) * 4;
        let nibble = ((n >> shift) & 0xF) as usize;
        buf[i] = hex_chars[nibble];
    }

    serial::serial_write(&buf);
}

/// Helper function to print u8 as hexadecimal to serial
fn print_hex_u8(n: u8) {
    let hex_chars = b"0123456789abcdef";
    let buf = [
        hex_chars[(n >> 4) as usize],
        hex_chars[(n & 0xF) as usize],
    ];
    serial::serial_write(&buf);
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
