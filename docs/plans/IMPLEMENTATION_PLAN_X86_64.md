# Implementation Plan: x86_64 Architecture Support

**Status:** Planning
**Target Platform:** x86_64 (Intel/AMD 64-bit)
**Boot Method:** UEFI (OVMF for QEMU)
**Primary Validation:** QEMU x86_64
**Secondary Validation:** Bare metal (future)
**Timeline:** 31 days (10 milestones)

---

## Executive Summary

This plan adds x86_64 architecture support to the SIS kernel alongside the existing AArch64 implementation. The approach maintains UEFI boot compatibility, creates a unified platform abstraction layer, and targets QEMU first for rapid development. The implementation is staged across 10 milestones (M0-M9), progressing from basic serial output to full SMP support with PCI/VirtIO drivers.

**Key Principles:**
- **UEFI-first:** Reuse existing UEFI boot flow, avoid legacy BIOS
- **Platform abstraction:** Unified trait-based interface for both architectures
- **QEMU validation:** All features tested in qemu-system-x86_64 before bare metal
- **Code reuse:** Maximum sharing between architectures via common modules
- **Incremental delivery:** Each milestone provides working functionality

---

## 1. Architecture Overview

### 1.1 Boot Flow

```
OVMF UEFI Firmware
    ↓
UEFI Boot Application (crates/uefi-boot)
    ↓ (loads kernel ELF)
Kernel Entry (64-bit long mode, paging enabled)
    ↓
Early Init (GDT, IDT, Serial)
    ↓
Platform Detection (ACPI/CPUID)
    ↓
Core Init (APIC, Timer, Paging)
    ↓
Driver Init (PCI, VirtIO, Block, Net)
    ↓
Userspace Init (Shell, AgentSys)
```

### 1.2 Platform Abstraction Layer

```rust
// crates/kernel/src/arch/common/platform.rs

pub trait Platform: Send + Sync {
    // Identification
    fn name(&self) -> &'static str;
    fn arch(&self) -> Architecture;

    // Console
    fn uart_init(&self, base: usize) -> Result<(), PlatformError>;
    fn uart_write(&self, bytes: &[u8]);
    fn uart_read(&self) -> Option<u8>;

    // Interrupts
    fn irq_init(&self) -> Result<(), PlatformError>;
    fn irq_enable(&self, irq: u32);
    fn irq_disable(&self, irq: u32);
    fn irq_ack(&self, irq: u32);
    fn irq_eoi(&self, irq: u32);

    // Timer
    fn timer_init(&self, frequency: u64) -> Result<(), PlatformError>;
    fn timer_read_counter(&self) -> u64;
    fn timer_frequency(&self) -> u64;

    // Memory
    fn memory_map(&self) -> &'static [MemoryRegion];
    fn kernel_base(&self) -> VirtAddr;
    fn phys_to_virt(&self, phys: PhysAddr) -> VirtAddr;
    fn virt_to_phys(&self, virt: VirtAddr) -> PhysAddr;

    // Optional: PCI
    fn pci_init(&self) -> Result<Option<PciController>, PlatformError>;

    // Optional: ACPI
    fn acpi_rsdp(&self) -> Option<PhysAddr>;
}

pub enum Architecture {
    X86_64,
    AArch64,
}
```

### 1.3 Memory Layout (x86_64)

```
Virtual Address Space (48-bit canonical addresses):

0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_F000   User Space (128 TB)
0x0000_7FFF_FFFF_F000 - 0x0000_8000_0000_0000   Guard Page

[Canonical Address Hole]

0xFFFF_8000_0000_0000 - 0xFFFF_8800_0000_0000   Kernel Image (512 GB)
0xFFFF_8800_0000_0000 - 0xFFFF_9000_0000_0000   Kernel Heap (512 GB)
0xFFFF_9000_0000_0000 - 0xFFFF_A000_0000_0000   Device MMIO (1 TB)
0xFFFF_A000_0000_0000 - 0xFFFF_B000_0000_0000   PCI ECAM Space (1 TB)
0xFFFF_B000_0000_0000 - 0xFFFF_C000_0000_0000   Per-CPU Data (1 TB)
0xFFFF_C000_0000_0000 - 0xFFFF_FFFF_8000_0000   Reserved
0xFFFF_FFFF_8000_0000 - 0xFFFF_FFFF_FFFF_FFFF   Direct Map (512 GB)

Physical Memory Map (typical):
0x0000_0000_0000_0000 - 0x0000_0000_000A_0000   Low Memory
0x0000_0000_0010_0000 - DRAM_END                 Main Memory
0x0000_0000_FEC0_0000 - 0x0000_0000_FEC0_1000   IOAPIC
0x0000_0000_FED0_0000 - 0x0000_0000_FED0_1000   HPET
0x0000_0000_FEE0_0000 - 0x0000_0000_FEE1_0000   Local APIC
0x0000_00E0_0000_0000 - 0x0000_00F0_0000_0000   PCI ECAM (256 buses)
```

---

## 2. Milestone M0: Skeleton Boot

### 2.1 Objectives
- UEFI handoff from OVMF
- Basic CPU initialization (GDT, IDT stubs)
- Early serial console (16550 UART)
- Minimal shell interaction

### 2.2 Components

#### 2.2.1 CPU Early Init

```rust
// crates/kernel/src/arch/x86_64/mod.rs

use x86_64::instructions::interrupts;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

pub fn arch_early_init() -> Result<(), &'static str> {
    // 1. Disable interrupts (safety first)
    interrupts::disable();

    // 2. Load GDT (required before any segment operations)
    gdt::init();

    // 3. Load empty IDT (prevents triple fault)
    idt::init_early();

    // 4. Enable CPU features required by Rust
    enable_cpu_features()?;

    // 5. Initialize early serial console
    serial::init_early(UART_COM1_BASE)?;

    // 6. Load TSS (for privilege level switching)
    tss::init();

    crate::info!("[x86_64] Early init complete");
    Ok(())
}

fn enable_cpu_features() -> Result<(), &'static str> {
    // Check for required features
    let cpuid = CpuId::new();
    let features = cpuid.get_feature_info()
        .ok_or("CPUID not supported")?;

    if !features.has_sse2() {
        return Err("SSE2 required for Rust floating-point operations");
    }

    // Enable SSE/SSE2 (required for Rust floats)
    unsafe {
        let mut cr0 = Cr0::read();
        cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
        cr0.remove(Cr0Flags::MONITOR_COPROCESSOR);
        Cr0::write(cr0);

        let mut cr4 = Cr4::read();
        cr4.insert(Cr4Flags::OSFXSR);
        cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
        Cr4::write(cr4);
    }

    // Enable AVX if available (optional, for performance)
    if features.has_avx() {
        unsafe {
            let mut cr4 = Cr4::read();
            cr4.insert(Cr4Flags::OSXSAVE);
            Cr4::write(cr4);

            // Enable AVX state in XCR0
            let xcr0 = _xgetbv(0);
            _xsetbv(0, xcr0 | 0x7); // x87, SSE, AVX states
        }
    }

    // Enable NX bit (for security)
    if features.has_execute_disable() {
        use x86_64::registers::model_specific::Efer;
        unsafe {
            Efer::update(|flags| {
                flags.insert(EferFlags::NO_EXECUTE_ENABLE);
            });
        }
    }

    // Enable FSGSBASE if available (faster TLS/per-CPU)
    if cpuid_has_fsgsbase() {
        unsafe {
            let mut cr4 = Cr4::read();
            cr4.insert(Cr4Flags::FSGSBASE);
            Cr4::write(cr4);
        }
    }

    Ok(())
}
```

#### 2.2.2 GDT Setup

```rust
// crates/kernel/src/arch/x86_64/gdt.rs

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const KERNEL_STACK_SIZE: usize = 16 * 1024; // 16 KiB

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // Null descriptor (required)
        let null = gdt.add_entry(Descriptor::kernel_null());

        // Kernel segments
        let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_data = gdt.add_entry(Descriptor::kernel_data_segment());

        // User segments (for future userspace)
        let user_data = gdt.add_entry(Descriptor::user_data_segment());
        let user_code = gdt.add_entry(Descriptor::user_code_segment());

        // TSS
        let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors {
            kernel_code,
            kernel_data,
            user_code,
            user_data,
            tss,
        })
    };
}

struct Selectors {
    kernel_code: SegmentSelector,
    kernel_data: SegmentSelector,
    user_code: SegmentSelector,
    user_data: SegmentSelector,
    tss: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, FS, GS, SS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        // Load kernel segments
        CS::set_reg(GDT.1.kernel_code);
        DS::set_reg(GDT.1.kernel_data);
        ES::set_reg(GDT.1.kernel_data);
        FS::set_reg(GDT.1.kernel_data);
        GS::set_reg(GDT.1.kernel_data);
        SS::set_reg(GDT.1.kernel_data);

        // Load TSS
        load_tss(GDT.1.tss);
    }
}
```

#### 2.2.3 Early Serial Console

```rust
// crates/kernel/src/arch/x86_64/serial.rs

use uart_16550::SerialPort;
use spin::Mutex;

pub const UART_COM1_BASE: u16 = 0x3F8;
pub const UART_COM2_BASE: u16 = 0x2F8;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(UART_COM1_BASE) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub fn init_early(base: u16) -> Result<(), &'static str> {
    // Force initialization of lazy_static
    let _ = SERIAL1.lock();
    Ok(())
}

pub fn write_byte(byte: u8) {
    SERIAL1.lock().send(byte);
}

pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_byte(byte);
    }
}

pub fn read_byte() -> Option<u8> {
    SERIAL1.lock().try_receive()
}

// Hook into kernel uart interface
impl crate::uart::UartOps for X86Serial {
    fn putc(&self, c: u8) {
        write_byte(c);
    }

    fn getc(&self) -> Option<u8> {
        read_byte()
    }

    fn flush(&self) {
        // 16550 doesn't need explicit flush
    }
}
```

#### 2.2.4 UEFI Handoff

```rust
// crates/uefi-boot/src/x86_64.rs

use uefi::prelude::*;
use uefi::table::boot::{MemoryDescriptor, MemoryType};
use uefi::proto::console::text::Output;

#[repr(C)]
pub struct BootInfo {
    pub memory_map: &'static [MemoryDescriptor],
    pub rsdp_addr: Option<u64>,
    pub kernel_image_base: u64,
    pub kernel_image_size: u64,
    pub framebuffer: Option<FramebufferInfo>,
}

pub unsafe fn jump_to_kernel(
    entry_point: u64,
    boot_info: *const BootInfo,
    stack_top: u64,
) -> ! {
    asm!(
        // Set up stack
        "mov rsp, {stack}",

        // Clear registers (ABI requirement)
        "xor rbp, rbp",
        "xor rbx, rbx",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",

        // Pass boot info in RDI (System V ABI)
        "mov rdi, {boot_info}",

        // Jump to kernel
        "jmp {entry}",

        stack = in(reg) stack_top,
        boot_info = in(reg) boot_info,
        entry = in(reg) entry_point,
        options(noreturn)
    );
}
```

### 2.3 Build Configuration

```toml
# crates/kernel/Cargo.toml
[target.'cfg(target_arch = "x86_64")'.dependencies]
x86_64 = "0.15"
uart_16550 = "0.3"
x86 = "0.52"
raw-cpuid = "11.0"

[target.x86_64-unknown-none]
runner = "scripts/run_x86_64.sh"

# .cargo/config.toml
[target.x86_64-unknown-none]
rustflags = [
    "-C", "target-feature=-red-zone",     # Disable red zone
    "-C", "target-feature=+soft-float",   # Use soft float initially
    "-C", "link-arg=-Tlinker_x86_64.ld", # Custom linker script
]

[build]
target = ["x86_64-unknown-none", "aarch64-unknown-none"]
```

### 2.4 QEMU Launch Script

```bash
#!/bin/bash
# scripts/run_x86_64.sh

OVMF_CODE="/usr/share/OVMF/OVMF_CODE.fd"
OVMF_VARS="/usr/share/OVMF/OVMF_VARS.fd"
KERNEL_ELF="target/x86_64-unknown-none/release/sis_kernel"
ESP_IMG="build/esp_x86_64.img"

# Build ESP image with kernel
mkdir -p build/esp/EFI/BOOT
cp crates/uefi-boot/target/x86_64-unknown-uefi/release/uefi-boot.efi \
   build/esp/EFI/BOOT/BOOTX64.EFI
cp $KERNEL_ELF build/esp/kernel.elf

# Create ESP image
dd if=/dev/zero of=$ESP_IMG bs=1M count=64
mkfs.vfat -F 32 $ESP_IMG
mcopy -i $ESP_IMG -s build/esp/* ::

# Run QEMU
qemu-system-x86_64 \
    -bios $OVMF_CODE \
    -drive file=$ESP_IMG,format=raw,if=virtio \
    -serial stdio \
    -m 1G \
    -cpu qemu64,+sse2,+sse3,+sse4.1,+sse4.2 \
    -machine q35 \
    -no-reboot \
    -no-shutdown \
    -d guest_errors,cpu_reset
```

### 2.5 Acceptance Criteria

- [ ] OVMF boots UEFI application successfully
- [ ] Kernel receives control in 64-bit long mode
- [ ] Serial console prints boot banner
- [ ] Basic shell commands work (help, echo)
- [ ] No triple faults or CPU resets

---

## 3. Milestone M1: Interrupts & Exceptions

### 3.1 Objectives
- Complete IDT with all exception handlers
- Basic interrupt handling infrastructure
- Legacy PIC initialization (8259A)
- PIT timer for early timekeeping

### 3.2 Components

#### 3.2.1 IDT Setup

```rust
// crates/kernel/src/arch/x86_64/idt.rs

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::idt::PageFaultErrorCode;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // CPU Exceptions (0-31)
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);

        // Double fault needs special stack
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        // x87 FPU exception
        idt.x87_floating_point.set_handler_fn(x87_fp_handler);

        // More exceptions...
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_fp_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);

        // Hardware interrupts (32-255)
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Serial.as_usize()]
            .set_handler_fn(serial_interrupt_handler);

        // IPI handlers for SMP
        idt[InterruptIndex::IpiReschedule.as_usize()]
            .set_handler_fn(ipi_reschedule_handler);
        idt[InterruptIndex::IpiTlbFlush.as_usize()]
            .set_handler_fn(ipi_tlb_flush_handler);

        idt
    };
}

pub fn init() {
    IDT.load();
}

// Exception handlers
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    let addr = Cr2::read();
    crate::error!("EXCEPTION: PAGE FAULT");
    crate::error!("  Accessed Address: {:#x}", addr);
    crate::error!("  Error Code: {:?}", error_code);
    crate::error!("  Stack Frame: {:#?}", stack_frame);

    // Check if it's a stack overflow
    let stack_bottom = crate::process::current_thread_stack_bottom();
    if addr.as_u64() < stack_bottom && addr.as_u64() > stack_bottom - 4096 {
        panic!("Stack overflow detected!");
    }

    panic!("Page fault at {:#x}", addr);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT (error_code: {})\n{:#?}",
           error_code, stack_frame);
}

extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::error!("EXCEPTION: GENERAL PROTECTION FAULT");
    crate::error!("  Error Code: {:#x}", error_code);
    crate::error!("  Stack Frame: {:#?}", stack_frame);

    if error_code != 0 {
        // Segment selector index is in bits 3-15
        let selector_index = (error_code >> 3) & 0x1FFF;
        let is_external = (error_code & 0x1) != 0;
        let in_idt = (error_code & 0x2) != 0;

        crate::error!("  Selector Index: {}", selector_index);
        crate::error!("  External: {}", is_external);
        crate::error!("  In IDT: {}", in_idt);
    }

    panic!("General protection fault");
}
```

#### 3.2.2 TSS Setup

```rust
// crates/kernel/src/arch/x86_64/tss.rs

use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_STACK_SIZE: usize = 16 * 1024; // 16 KiB
pub const INTERRUPT_STACK_SIZE: usize = 16 * 1024;

#[repr(align(16))]
struct Stack([u8; DOUBLE_FAULT_STACK_SIZE]);

static mut DOUBLE_FAULT_STACK: Stack = Stack([0; DOUBLE_FAULT_STACK_SIZE]);
static mut INTERRUPT_STACKS: [Stack; 7] = [Stack([0; INTERRUPT_STACK_SIZE]); 7];

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Set up double fault stack (IST 0)
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(unsafe {
                &DOUBLE_FAULT_STACK as *const _
            });
            let stack_end = stack_start + DOUBLE_FAULT_STACK_SIZE;
            stack_end
        };

        // Set up privilege stack (for syscalls)
        tss.privilege_stack_table[0] = {
            // This will be updated per-thread
            VirtAddr::new(0)
        };

        tss
    };
}

pub fn init() {
    // TSS is loaded as part of GDT init
}

pub fn set_kernel_stack(stack_top: VirtAddr) {
    unsafe {
        // Update TSS with current kernel stack for syscalls
        TSS.privilege_stack_table[0] = stack_top;
    }
}
```

#### 3.2.3 Legacy PIC Setup

```rust
// crates/kernel/src/arch/x86_64/pic.rs

use pic8259::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

pub fn init() {
    unsafe {
        PICS.lock().initialize();
    }
}

pub fn disable() {
    // Disable all IRQs when switching to APIC
    unsafe {
        PICS.lock().disable();
    }
}

pub fn enable_irq(irq: u8) {
    unsafe {
        PICS.lock().enable_irq(irq);
    }
}

pub fn eoi(interrupt_id: u8) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(interrupt_id);
    }
}

// IRQ numbers
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Irq {
    Timer = 0,
    Keyboard = 1,
    Cascade = 2,
    Serial2 = 3,
    Serial1 = 4,
    Parallel2 = 5,
    Floppy = 6,
    Parallel1 = 7,
    RTC = 8,
    Free1 = 9,
    Free2 = 10,
    Free3 = 11,
    Mouse = 12,
    FPU = 13,
    PrimaryATA = 14,
    SecondaryATA = 15,
}
```

#### 3.2.4 PIT Timer

```rust
// crates/kernel/src/arch/x86_64/pit.rs

use x86_64::instructions::port::Port;

const PIT_FREQ: u32 = 1_193_182; // Hz
const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;

pub fn init(frequency: u32) {
    let divisor = PIT_FREQ / frequency;

    unsafe {
        // Channel 0, lobyte/hibyte, rate generator
        Port::<u8>::new(PIT_COMMAND).write(0x36);

        // Write frequency divisor
        Port::<u8>::new(PIT_CHANNEL0).write((divisor & 0xFF) as u8);
        Port::<u8>::new(PIT_CHANNEL0).write((divisor >> 8) as u8);
    }

    // Enable timer IRQ
    pic::enable_irq(Irq::Timer as u8);
}

pub fn calibrate_tsc() -> u64 {
    // Use PIT to calibrate TSC frequency
    let start_tsc = read_tsc();

    // Wait for 100ms using PIT
    pit_wait_ms(100);

    let end_tsc = read_tsc();
    let tsc_ticks = end_tsc - start_tsc;

    // Calculate TSC frequency
    tsc_ticks * 10 // Convert 100ms to 1 second
}

fn pit_wait_ms(ms: u32) {
    // Implementation using PIT one-shot mode
    // ...
}
```

### 3.3 Acceptance Criteria

- [ ] All CPU exceptions have handlers
- [ ] Page faults show CR2 and error code
- [ ] Double fault uses separate stack
- [ ] PIT timer ticks at configured rate
- [ ] Timer interrupt increments tick counter
- [ ] Serial interrupt handles received bytes

---

## 4. Milestone M2: APIC & High Precision Timer

### 4.1 Objectives
- Detect and initialize Local APIC (xAPIC/x2APIC)
- Replace PIC with APIC for interrupt handling
- HPET initialization for TSC calibration
- Accurate timekeeping infrastructure

### 4.2 Components

#### 4.2.1 APIC Detection & Init

```rust
// crates/kernel/src/arch/x86_64/apic.rs

use x86_64::registers::model_specific::Msr;

const IA32_APIC_BASE: u32 = 0x1B;
const APIC_BASE_ENABLE: u64 = 1 << 11;
const APIC_BASE_X2APIC: u64 = 1 << 10;
const APIC_DEFAULT_BASE: u64 = 0xFEE00000;

#[derive(Debug, Clone, Copy)]
pub enum ApicMode {
    Disabled,
    XApic,      // Memory-mapped at 0xFEE00000
    X2Apic,     // MSR-based access
}

pub struct LocalApic {
    mode: ApicMode,
    base_addr: Option<VirtAddr>,
}

impl LocalApic {
    pub fn new() -> Result<Self, &'static str> {
        let mode = detect_apic_mode()?;

        let base_addr = match mode {
            ApicMode::XApic => {
                let phys = PhysAddr::new(APIC_DEFAULT_BASE);
                Some(crate::mm::phys_to_virt(phys))
            }
            ApicMode::X2Apic => None, // MSR access, no mapping needed
            ApicMode::Disabled => return Err("APIC not available"),
        };

        let mut apic = Self { mode, base_addr };
        apic.init()?;
        Ok(apic)
    }

    fn init(&mut self) -> Result<(), &'static str> {
        match self.mode {
            ApicMode::XApic => self.init_xapic(),
            ApicMode::X2Apic => self.init_x2apic(),
            ApicMode::Disabled => Err("APIC disabled"),
        }
    }

    fn init_xapic(&self) -> Result<(), &'static str> {
        let base = self.base_addr.ok_or("No APIC base")?;

        unsafe {
            // Enable APIC
            self.write_xapic(0xF0, self.read_xapic(0xF0) | 0x100);

            // Set spurious interrupt vector
            self.write_xapic(0xF0, 0x100 | 0xFF);

            // Configure timer
            self.write_xapic(0x3E0, 0x3); // Divide by 16
            self.write_xapic(0x320, 0x10000); // Periodic mode, vector 32
            self.write_xapic(0x380, 10000); // Initial count

            // Configure LVT entries
            self.write_xapic(0x350, 0x10000); // LINT0 masked
            self.write_xapic(0x360, 0x10000); // LINT1 masked
            self.write_xapic(0x370, 0x10000); // Error masked
        }

        Ok(())
    }

    fn init_x2apic(&self) -> Result<(), &'static str> {
        unsafe {
            // Enable x2APIC mode
            let mut apic_base = rdmsr(IA32_APIC_BASE);
            apic_base |= APIC_BASE_ENABLE | APIC_BASE_X2APIC;
            wrmsr(IA32_APIC_BASE, apic_base);

            // Configure via MSRs
            wrmsr(0x80F, 0x100 | 0xFF); // Spurious interrupt
            wrmsr(0x832, 0x10000); // Timer, periodic, vector 32
            wrmsr(0x83E, 0x3); // Timer divide by 16
            wrmsr(0x838, 10000); // Initial count
        }

        Ok(())
    }

    pub fn eoi(&self) {
        match self.mode {
            ApicMode::XApic => unsafe {
                self.write_xapic(0xB0, 0);
            },
            ApicMode::X2Apic => unsafe {
                wrmsr(0x80B, 0);
            },
            _ => {}
        }
    }

    pub fn send_ipi(&self, dest: u32, vector: u8) {
        match self.mode {
            ApicMode::XApic => unsafe {
                self.write_xapic(0x310, (dest as u32) << 24);
                self.write_xapic(0x300, vector as u32);
            },
            ApicMode::X2Apic => unsafe {
                let icr = ((dest as u64) << 32) | (vector as u64);
                wrmsr(0x830, icr);
            },
            _ => {}
        }
    }

    unsafe fn read_xapic(&self, offset: u32) -> u32 {
        let addr = self.base_addr.unwrap().as_u64() + offset as u64;
        core::ptr::read_volatile(addr as *const u32)
    }

    unsafe fn write_xapic(&self, offset: u32, value: u32) {
        let addr = self.base_addr.unwrap().as_u64() + offset as u64;
        core::ptr::write_volatile(addr as *mut u32, value);
    }
}

fn detect_apic_mode() -> Result<ApicMode, &'static str> {
    let cpuid = CpuId::new();
    let features = cpuid.get_feature_info()
        .ok_or("No CPUID feature info")?;

    if !features.has_apic() {
        return Ok(ApicMode::Disabled);
    }

    // Check for x2APIC support
    let has_x2apic = features.has_x2apic();

    // Read APIC base MSR
    let apic_base = unsafe { rdmsr(IA32_APIC_BASE) };
    let x2apic_enabled = (apic_base & APIC_BASE_X2APIC) != 0;

    if has_x2apic && (x2apic_enabled || prefer_x2apic()) {
        Ok(ApicMode::X2Apic)
    } else {
        Ok(ApicMode::XApic)
    }
}

fn prefer_x2apic() -> bool {
    // Prefer x2APIC in VMs for performance
    is_running_in_vm()
}
```

#### 4.2.2 HPET Support

```rust
// crates/kernel/src/arch/x86_64/hpet.rs

use acpi::HpetInfo;

const HPET_REG_CAP: usize = 0x00;
const HPET_REG_CONFIG: usize = 0x10;
const HPET_REG_COUNTER: usize = 0xF0;

pub struct Hpet {
    base: VirtAddr,
    frequency: u64,
    period_fs: u64, // Femtoseconds
}

impl Hpet {
    pub fn new(hpet_info: &HpetInfo) -> Result<Self, &'static str> {
        let phys = PhysAddr::new(hpet_info.base_address as u64);
        let base = crate::mm::map_device(phys, 0x1000)?;

        let cap = unsafe {
            core::ptr::read_volatile((base.as_u64() + HPET_REG_CAP) as *const u64)
        };

        let period_fs = cap >> 32; // Upper 32 bits
        let frequency = 1_000_000_000_000_000 / period_fs; // Convert fs to Hz

        let mut hpet = Self {
            base,
            frequency,
            period_fs,
        };

        hpet.init();
        Ok(hpet)
    }

    fn init(&mut self) {
        unsafe {
            // Stop counter
            let config_addr = (self.base.as_u64() + HPET_REG_CONFIG) as *mut u64;
            let mut config = core::ptr::read_volatile(config_addr);
            config &= !1; // Clear ENABLE bit
            core::ptr::write_volatile(config_addr, config);

            // Reset counter
            let counter_addr = (self.base.as_u64() + HPET_REG_COUNTER) as *mut u64;
            core::ptr::write_volatile(counter_addr, 0);

            // Start counter
            config |= 1; // Set ENABLE bit
            core::ptr::write_volatile(config_addr, config);
        }
    }

    pub fn read_counter(&self) -> u64 {
        unsafe {
            core::ptr::read_volatile(
                (self.base.as_u64() + HPET_REG_COUNTER) as *const u64
            )
        }
    }

    pub fn calibrate_tsc(&self, duration_ms: u32) -> u64 {
        let start_hpet = self.read_counter();
        let start_tsc = read_tsc();

        // Wait for specified duration
        let target_ticks = (self.frequency * duration_ms as u64) / 1000;
        while (self.read_counter() - start_hpet) < target_ticks {
            core::hint::spin_loop();
        }

        let end_tsc = read_tsc();
        let elapsed_tsc = end_tsc - start_tsc;

        // Calculate TSC frequency
        (elapsed_tsc * 1000) / duration_ms as u64
    }
}
```

### 4.3 Acceptance Criteria

- [ ] APIC mode detected correctly (xAPIC/x2APIC)
- [ ] Local APIC timer generates interrupts
- [ ] EOI works correctly
- [ ] HPET counter increments
- [ ] TSC calibrated to actual frequency
- [ ] PIC disabled after APIC init

---

## 5. Milestone M3: Paging & Memory Management

### 5.1 Objectives
- 4-level page table management (PML4)
- Integration with existing memory manager
- Kernel/user address space separation
- Demand paging support

### 5.2 Components

#### 5.2.1 Page Table Structures

```rust
// crates/kernel/src/arch/x86_64/paging.rs

use x86_64::structures::paging::{
    PageTable, PageTableFlags, PageTableIndex,
    Page, PhysFrame, Size4KiB, Size2MiB, Size1GiB,
    Mapper, RecursivePageTable, OffsetPageTable,
};
use x86_64::{VirtAddr, PhysAddr};

pub struct PageTableManager {
    mapper: OffsetPageTable<'static>,
    frame_allocator: FrameAllocator,
}

impl PageTableManager {
    pub fn new() -> Result<Self, &'static str> {
        let level_4_table = unsafe { active_level_4_table() };
        let phys_offset = VirtAddr::new(PHYS_MEM_OFFSET);

        let mapper = unsafe {
            OffsetPageTable::new(level_4_table, phys_offset)
        };

        Ok(Self {
            mapper,
            frame_allocator: FrameAllocator::new(),
        })
    }

    pub fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: PageTableFlags,
    ) -> Result<(), MapError> {
        let page = Page::<Size4KiB>::containing_address(virt);
        let frame = PhysFrame::containing_address(phys);

        let flush = unsafe {
            self.mapper.map_to(page, frame, flags, &mut self.frame_allocator)?
        };

        flush.flush();
        Ok(())
    }

    pub fn map_region(
        &mut self,
        virt_start: VirtAddr,
        phys_start: PhysAddr,
        size: usize,
        flags: PageTableFlags,
    ) -> Result<(), MapError> {
        let pages = (size + 4095) / 4096;

        for i in 0..pages {
            let virt = virt_start + (i * 4096);
            let phys = phys_start + (i * 4096);
            self.map_page(virt, phys, flags)?;
        }

        Ok(())
    }

    pub fn map_kernel() -> Result<(), MapError> {
        let mut mgr = Self::new()?;

        // Map kernel code as read-execute
        mgr.map_region(
            VirtAddr::new(KERNEL_CODE_START),
            PhysAddr::new(KERNEL_PHYS_START),
            KERNEL_CODE_SIZE,
            PageTableFlags::PRESENT |
            PageTableFlags::GLOBAL |
            PageTableFlags::NO_EXECUTE.complement(),
        )?;

        // Map kernel data as read-write
        mgr.map_region(
            VirtAddr::new(KERNEL_DATA_START),
            PhysAddr::new(KERNEL_DATA_PHYS),
            KERNEL_DATA_SIZE,
            PageTableFlags::PRESENT |
            PageTableFlags::WRITABLE |
            PageTableFlags::GLOBAL |
            PageTableFlags::NO_EXECUTE,
        )?;

        // Map kernel heap
        mgr.map_region(
            VirtAddr::new(KERNEL_HEAP_START),
            PhysAddr::new(KERNEL_HEAP_PHYS),
            KERNEL_HEAP_SIZE,
            PageTableFlags::PRESENT |
            PageTableFlags::WRITABLE |
            PageTableFlags::GLOBAL |
            PageTableFlags::NO_EXECUTE,
        )?;

        Ok(())
    }
}

unsafe fn active_level_4_table() -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = phys.as_u64() + PHYS_MEM_OFFSET;
    let page_table_ptr: *mut PageTable = virt as *mut PageTable;

    &mut *page_table_ptr
}
```

#### 5.2.2 Frame Allocator

```rust
// crates/kernel/src/arch/x86_64/frame_allocator.rs

use x86_64::structures::paging::{FrameAllocator as FrameAllocatorTrait, PhysFrame};
use x86_64::PhysAddr;

pub struct FrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
}

impl FrameAllocator {
    pub fn new() -> Self {
        Self {
            memory_map: get_memory_map(),
            next: 0,
        }
    }
}

unsafe impl FrameAllocatorTrait<Size4KiB> for FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

impl FrameAllocator {
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryType::Usable)
            .flat_map(|r| {
                let start_addr = r.start;
                let end_addr = r.start + r.size;
                let start_frame = PhysFrame::containing_address(PhysAddr::new(start_addr));
                let end_frame = PhysFrame::containing_address(PhysAddr::new(end_addr - 1));
                (start_frame.start_address().as_u64()..end_frame.start_address().as_u64())
                    .step_by(4096)
                    .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
            })
    }
}
```

### 5.3 Integration with Existing MM

```rust
// crates/kernel/src/mm/arch_x86_64.rs

impl crate::mm::ArchMM for X86_64MM {
    fn map_page(virt: VirtAddr, phys: PhysAddr, flags: PageFlags) -> Result<(), MMError> {
        let mut mgr = PAGE_TABLE_MANAGER.lock();

        let x86_flags = PageTableFlags::PRESENT;
        let x86_flags = if flags.contains(PageFlags::WRITABLE) {
            x86_flags | PageTableFlags::WRITABLE
        } else {
            x86_flags
        };
        let x86_flags = if flags.contains(PageFlags::USER) {
            x86_flags | PageTableFlags::USER_ACCESSIBLE
        } else {
            x86_flags
        };
        let x86_flags = if !flags.contains(PageFlags::EXECUTE) {
            x86_flags | PageTableFlags::NO_EXECUTE
        } else {
            x86_flags
        };

        mgr.map_page(virt.into(), phys.into(), x86_flags)
            .map_err(|_| MMError::MappingFailed)
    }

    fn unmap_page(virt: VirtAddr) -> Result<(), MMError> {
        let mut mgr = PAGE_TABLE_MANAGER.lock();
        let page = Page::<Size4KiB>::containing_address(virt.into());
        mgr.mapper.unmap(page)
            .map(|flush| flush.flush())
            .map_err(|_| MMError::UnmapFailed)
    }

    fn switch_address_space(page_table: PhysAddr) {
        use x86_64::registers::control::Cr3;
        let frame = PhysFrame::containing_address(page_table.into());
        unsafe {
            Cr3::write(frame, Cr3Flags::empty());
        }
    }
}
```

### 5.4 Acceptance Criteria

- [ ] Kernel pages mapped correctly
- [ ] Page fault handler shows faulting address
- [ ] Can allocate/free physical frames
- [ ] TLB flush works correctly
- [ ] No page table corruption under load

---

## 6. Milestone M4: Syscall Entry

### 6.1 Objectives
- SYSCALL/SYSRET fast path setup
- System call dispatcher
- Register preservation
- Integration with existing syscall table

### 6.2 Components

#### 6.2.1 SYSCALL MSR Setup

```rust
// crates/kernel/src/arch/x86_64/syscall.rs

use x86_64::registers::model_specific::{Efer, EferFlags, Star, LStar, SFMask};
use x86_64::registers::rflags::RFlags;

pub fn init_syscall() {
    // Set up segments for SYSCALL/SYSRET
    let star = Star::read();
    Star::write(
        star.syscall_cs_ss(),  // Keep existing kernel segments
        star.sysret_cs_ss(),    // Keep existing user segments
    );

    // Set syscall entry point
    LStar::write(VirtAddr::from_ptr(syscall_entry as *const ()));

    // Clear interrupts on syscall entry
    SFMask::write(RFlags::INTERRUPT_FLAG);

    // Enable SYSCALL instruction
    unsafe {
        Efer::update(|flags| {
            *flags |= EferFlags::SYSTEM_CALL_EXTENSIONS;
        });
    }
}

#[naked]
unsafe extern "C" fn syscall_entry() -> ! {
    asm!(
        // Save user stack pointer
        "mov gs:[offset_user_rsp], rsp",

        // Load kernel stack
        "mov rsp, gs:[offset_kernel_rsp]",

        // Save registers (System V ABI preserved)
        "push rcx",      // User RIP
        "push r11",      // User RFLAGS
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Arguments are already in correct registers:
        // rdi = syscall number
        // rsi = arg1
        // rdx = arg2
        // r10 = arg3 (not rcx!)
        // r8  = arg4
        // r9  = arg5

        // Call syscall handler
        "mov rcx, r10",  // Fix arg3 (SYSCALL uses r10 instead of rcx)
        "call syscall_handler",

        // Return value is in rax

        // Restore registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "pop r11",      // User RFLAGS
        "pop rcx",      // User RIP

        // Restore user stack
        "mov rsp, gs:[offset_user_rsp]",

        // Return to userspace
        "sysretq",

        options(noreturn)
    );
}

#[no_mangle]
pub extern "C" fn syscall_handler(
    syscall_num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> isize {
    // Build syscall frame
    let frame = SyscallFrame {
        syscall_num,
        args: [arg1, arg2, arg3, arg4, arg5, 0],
    };

    // Dispatch to common handler
    crate::syscall::handle_syscall(&frame)
}
```

#### 6.2.2 Alternative: INT 0x80 (Fallback)

```rust
// For simpler initial bring-up
extern "x86-interrupt" fn syscall_interrupt_handler(
    stack_frame: InterruptStackFrame,
) {
    // Read registers from stack
    let regs = unsafe {
        &*(stack_frame.as_mut().stack_pointer.as_u64() as *const SyscallRegs)
    };

    let result = crate::syscall::handle_syscall(&regs.to_frame());

    // Set return value in RAX
    unsafe {
        asm!("mov rax, {}", in(reg) result);
    }
}
```

### 6.3 Acceptance Criteria

- [ ] SYSCALL instruction works from userspace
- [ ] System calls preserve registers correctly
- [ ] Return values passed in RAX
- [ ] Error codes returned as negative values
- [ ] Basic syscalls work (write, read, exit)

---

## 7. Milestone M5: Serial/TTY Polish

### 7.1 Objectives
- Interrupt-driven serial I/O
- FIFO management
- Line discipline support
- /dev/ttyS0 device node

### 7.2 Components

```rust
// crates/kernel/src/drivers/serial_16550.rs

pub struct Serial16550 {
    port: SerialPort,
    rx_buffer: ArrayDeque<[u8; 256]>,
    tx_buffer: ArrayDeque<[u8; 256]>,
}

impl Serial16550 {
    pub fn init_interrupt(&mut self) {
        // Enable receive interrupt
        self.port.enable_interrupt(InterruptType::Received);
    }

    pub fn handle_interrupt(&mut self) {
        while let Some(byte) = self.port.try_receive() {
            self.rx_buffer.push_back(byte);

            // Wake any waiting readers
            self.wake_readers();
        }
    }
}

impl crate::drivers::tty::TtyDevice for Serial16550 {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, TtyError> {
        let mut count = 0;
        while count < buf.len() && !self.rx_buffer.is_empty() {
            buf[count] = self.rx_buffer.pop_front().unwrap();
            count += 1;
        }
        Ok(count)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, TtyError> {
        for &byte in buf {
            self.port.send(byte);
        }
        Ok(buf.len())
    }
}
```

### 7.3 Acceptance Criteria

- [ ] Serial receives trigger interrupts
- [ ] No character loss at 115200 baud
- [ ] Line editing works (backspace, etc.)
- [ ] /dev/ttyS0 accessible via VFS

---

## 8. Milestone M6-M7: Block Storage & PCI

### 8.1 Objectives
- PCI bus enumeration
- VirtIO-blk over PCI transport
- Block device abstraction
- ext4 filesystem mounting

### 8.2 PCI Bus Scanner

```rust
// crates/kernel/src/drivers/pci/mod.rs

use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

pub struct PciController {
    config_address: Port<u32>,
    config_data: Port<u32>,
    devices: Vec<PciDevice>,
}

impl PciController {
    pub fn new() -> Self {
        Self {
            config_address: Port::new(PCI_CONFIG_ADDRESS),
            config_data: Port::new(PCI_CONFIG_DATA),
            devices: Vec::new(),
        }
    }

    pub fn scan_bus(&mut self) {
        for bus in 0..256u8 {
            for device in 0..32u8 {
                for function in 0..8u8 {
                    let vendor_id = self.read_config(bus, device, function, 0x00);
                    if vendor_id & 0xFFFF == 0xFFFF {
                        continue; // No device
                    }

                    let device_id = (vendor_id >> 16) & 0xFFFF;
                    let vendor_id = vendor_id & 0xFFFF;

                    let class_code = self.read_config(bus, device, function, 0x08);

                    self.devices.push(PciDevice {
                        bus,
                        device,
                        function,
                        vendor_id: vendor_id as u16,
                        device_id: device_id as u16,
                        class: (class_code >> 24) as u8,
                        subclass: (class_code >> 16) as u8,
                    });

                    crate::info!("[PCI] Found device {:04x}:{:04x} at {:02x}:{:02x}.{:x}",
                                vendor_id, device_id, bus, device, function);
                }
            }
        }
    }

    fn read_config(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        let address = 0x80000000
            | ((bus as u32) << 16)
            | ((device as u32) << 11)
            | ((function as u32) << 8)
            | ((offset as u32) & 0xFC);

        unsafe {
            self.config_address.write(address);
            self.config_data.read()
        }
    }
}

// ECAM (Memory-mapped config) alternative
pub struct PciEcam {
    base: VirtAddr,
}

impl PciEcam {
    pub fn new(base: PhysAddr) -> Result<Self, &'static str> {
        let virt = crate::mm::map_device(base, 256 * 1024 * 1024)?; // 256MB for 256 buses
        Ok(Self { base: virt })
    }

    pub fn read_config(&self, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        let addr = self.base.as_u64()
            + ((bus as u64) << 20)
            + ((device as u64) << 15)
            + ((function as u64) << 12)
            + (offset as u64);

        unsafe {
            core::ptr::read_volatile(addr as *const u32)
        }
    }
}
```

### 8.3 VirtIO-PCI Transport

```rust
// crates/kernel/src/drivers/virtio/pci.rs

pub struct VirtioPciTransport {
    device: PciDevice,
    common_cfg: VirtAddr,
    notify_cfg: VirtAddr,
    device_cfg: VirtAddr,
}

impl VirtioPciTransport {
    pub fn new(device: PciDevice) -> Result<Self, VirtioError> {
        // Find capability structures in PCI config space
        let cap_pointer = read_config_u8(device, 0x34);

        let mut common_cfg = None;
        let mut notify_cfg = None;
        let mut device_cfg = None;

        let mut cap_offset = cap_pointer;
        while cap_offset != 0 {
            let cap_id = read_config_u8(device, cap_offset);

            if cap_id == 0x09 { // Vendor-specific capability
                let cfg_type = read_config_u8(device, cap_offset + 3);
                let bar = read_config_u8(device, cap_offset + 4);
                let offset = read_config_u32(device, cap_offset + 8);
                let length = read_config_u32(device, cap_offset + 12);

                let bar_addr = get_bar_address(device, bar)?;
                let cfg_addr = bar_addr + offset as u64;

                match cfg_type {
                    1 => common_cfg = Some(map_mmio(cfg_addr, length)?),
                    2 => notify_cfg = Some(map_mmio(cfg_addr, length)?),
                    4 => device_cfg = Some(map_mmio(cfg_addr, length)?),
                    _ => {}
                }
            }

            cap_offset = read_config_u8(device, cap_offset + 1);
        }

        Ok(Self {
            device,
            common_cfg: common_cfg.ok_or(VirtioError::MissingCapability)?,
            notify_cfg: notify_cfg.ok_or(VirtioError::MissingCapability)?,
            device_cfg,
        })
    }
}

impl VirtioTransport for VirtioPciTransport {
    fn read_device_features(&self) -> u64 {
        // Implementation
    }

    fn write_driver_features(&self, features: u64) {
        // Implementation
    }

    // ... other transport methods
}
```

### 8.4 Acceptance Criteria

- [ ] PCI bus enumeration finds devices
- [ ] VirtIO devices detected
- [ ] Block device reads/writes work
- [ ] ext4 mounts from VirtIO disk
- [ ] File operations work correctly

---

## 9. Milestone M8: SMP Support

### 9.1 Objectives
- Secondary CPU bring-up via INIT-SIPI-SIPI
- Per-CPU data structures
- CPU-local APIC initialization
- Load balancing scheduler

### 9.2 AP Startup

```rust
// crates/kernel/src/arch/x86_64/smp.rs

const AP_BOOT_CODE: &[u8] = include_bytes!("ap_boot.bin");
const AP_BOOT_ADDR: u64 = 0x8000; // Low memory trampoline

pub fn boot_aps() -> Result<usize, SmpError> {
    // Copy AP boot code to low memory
    unsafe {
        core::ptr::copy_nonoverlapping(
            AP_BOOT_CODE.as_ptr(),
            AP_BOOT_ADDR as *mut u8,
            AP_BOOT_CODE.len()
        );
    }

    // Parse MADT to find APIC IDs
    let madt = acpi::find_table::<Madt>()
        .ok_or(SmpError::NoMadt)?;

    let mut cpu_count = 1; // BSP already running

    for entry in madt.entries() {
        match entry {
            MadtEntry::LocalApic(apic) => {
                if apic.processor_id == 0 {
                    continue; // Skip BSP
                }

                if !apic.flags.contains(ApicFlags::ENABLED) {
                    continue;
                }

                // Send INIT IPI
                LOCAL_APIC.send_ipi(apic.apic_id, IpiType::Init);
                delay_us(10000); // 10ms

                // Send SIPI twice
                for _ in 0..2 {
                    LOCAL_APIC.send_ipi(
                        apic.apic_id,
                        IpiType::Startup(AP_BOOT_ADDR >> 12)
                    );
                    delay_us(200); // 200us
                }

                // Wait for AP to signal ready
                let timeout = 50000; // 50ms
                let mut ready = false;
                for _ in 0..timeout {
                    if AP_READY[apic.processor_id as usize].load(Ordering::Acquire) {
                        ready = true;
                        break;
                    }
                    delay_us(1);
                }

                if ready {
                    cpu_count += 1;
                    crate::info!("[SMP] CPU {} online (APIC ID {})",
                                apic.processor_id, apic.apic_id);
                } else {
                    crate::warn!("[SMP] CPU {} failed to start", apic.processor_id);
                }
            }
            _ => {}
        }
    }

    Ok(cpu_count)
}

// AP entry point (called from ap_boot.S after entering long mode)
#[no_mangle]
pub extern "C" fn ap_main(processor_id: u32) -> ! {
    // Initialize per-CPU GDT
    gdt::init_ap();

    // Initialize IDT
    idt::init();

    // Initialize local APIC
    apic::init_local();

    // Set up per-CPU data
    let cpu_data = CpuLocal {
        cpu_id: processor_id,
        kernel_stack: allocate_kernel_stack(),
        tss: TaskStateSegment::new(),
        current_task: None,
    };

    set_cpu_local(cpu_data);

    // Signal ready
    AP_READY[processor_id as usize].store(true, Ordering::Release);

    // Enter scheduler
    crate::process::scheduler::idle_loop();
}
```

### 9.3 Per-CPU Data

```rust
// crates/kernel/src/arch/x86_64/percpu.rs

use x86_64::registers::segmentation::{Segment, GS};

#[repr(C)]
pub struct CpuLocal {
    self_ptr: *const CpuLocal,  // Must be first field
    pub cpu_id: u32,
    pub apic_id: u32,
    pub kernel_stack: VirtAddr,
    pub user_stack: VirtAddr,
    pub current_task: Option<Arc<Process>>,
    pub idle_task: Arc<Process>,
    pub tss: TaskStateSegment,
    pub stats: CpuStats,
}

pub struct CpuStats {
    pub idle_time: u64,
    pub busy_time: u64,
    pub interrupts: u64,
    pub syscalls: u64,
}

impl CpuLocal {
    pub fn current() -> &'static Self {
        unsafe {
            let ptr: *const CpuLocal;
            asm!("mov {}, gs:0", out(reg) ptr);
            &*ptr
        }
    }

    pub fn current_mut() -> &'static mut Self {
        unsafe {
            let ptr: *mut CpuLocal;
            asm!("mov {}, gs:0", out(reg) ptr);
            &mut *ptr
        }
    }
}

pub fn init_percpu(cpu_id: u32) {
    // Allocate per-CPU data
    let cpu_data = Box::leak(Box::new(CpuLocal {
        self_ptr: core::ptr::null(),
        cpu_id,
        apic_id: get_apic_id(),
        kernel_stack: allocate_kernel_stack(),
        user_stack: VirtAddr::zero(),
        current_task: None,
        idle_task: create_idle_task(cpu_id),
        tss: TaskStateSegment::new(),
        stats: CpuStats::default(),
    }));

    // Set self pointer
    cpu_data.self_ptr = cpu_data as *const _;

    // Load GS with per-CPU data pointer
    unsafe {
        if cpu_has_fsgsbase() {
            asm!("wrgsbase {}", in(reg) cpu_data as *const _ as u64);
        } else {
            // Use MSR
            wrmsr(IA32_GS_BASE, cpu_data as *const _ as u64);
        }
    }
}
```

### 9.4 Acceptance Criteria

- [ ] All CPUs detected from MADT
- [ ] Secondary CPUs boot successfully
- [ ] Per-CPU data accessible via GS
- [ ] Scheduler distributes load
- [ ] IPIs work between CPUs

---

## 10. Milestone M9: ACPI & Testing

### 10.1 ACPI Table Parsing

```rust
// crates/kernel/src/arch/x86_64/acpi.rs

use acpi::{AcpiTables, AcpiHandler};

struct KernelAcpiHandler;

impl AcpiHandler for KernelAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virt = crate::mm::map_device(
            PhysAddr::new(physical_address as u64),
            size
        ).expect("ACPI mapping failed");

        PhysicalMapping::new(
            physical_address,
            NonNull::new(virt.as_mut_ptr()).unwrap(),
            size,
            size,
            Self,
        )
    }

    fn unmap_physical_region<T>(&self, _region: &PhysicalMapping<Self, T>) {
        // Mappings are permanent in kernel
    }
}

pub fn init(rsdp_addr: PhysAddr) -> Result<(), AcpiError> {
    let handler = KernelAcpiHandler;
    let tables = unsafe {
        AcpiTables::from_rsdp(handler, rsdp_addr.as_u64() as usize)?
    };

    // Process important tables
    if let Ok(madt) = tables.find_table::<Madt>() {
        process_madt(&madt)?;
    }

    if let Ok(hpet) = tables.find_table::<Hpet>() {
        crate::arch::x86_64::hpet::init(&hpet)?;
    }

    if let Ok(mcfg) = tables.find_table::<Mcfg>() {
        init_pcie_ecam(&mcfg)?;
    }

    if let Ok(fadt) = tables.find_table::<Fadt>() {
        init_power_management(&fadt)?;
    }

    Ok(())
}
```

### 10.2 Power Management

```rust
// crates/kernel/src/arch/x86_64/power.rs

pub fn system_reset() -> ! {
    // Try ACPI reset first
    if let Some(reset_reg) = ACPI_RESET_REG.get() {
        unsafe {
            reset_reg.write(ACPI_RESET_VALUE.get());
        }
        delay_ms(1000);
    }

    // Try keyboard controller reset
    unsafe {
        Port::<u8>::new(0x64).write(0xFE);
    }
    delay_ms(1000);

    // Try triple fault
    unsafe {
        asm!("lidt [0]", "int3");
    }

    // Halt
    loop {
        unsafe { asm!("hlt") }
    }
}

pub fn system_poweroff() -> ! {
    // Try ACPI S5 state
    if let Some(pm1a_control) = PM1A_CONTROL.get() {
        let s5_value = SLP_TYP_S5.get() | SLP_EN;
        unsafe {
            pm1a_control.write(s5_value);
        }
    }

    // Fall back to halt
    loop {
        unsafe { asm!("hlt") }
    }
}
```

### 10.3 Test Infrastructure

```rust
// crates/testing/src/x86_64.rs

pub fn run_tests() {
    test_exceptions();
    test_interrupts();
    test_paging();
    test_syscalls();
    test_smp();
}

fn test_exceptions() {
    // Test page fault handling
    let result = catch_unwind(|| {
        unsafe {
            let ptr = 0xDEADBEEF as *const u8;
            let _val = *ptr; // Should page fault
        }
    });
    assert!(result.is_err());

    // Test divide by zero
    let result = catch_unwind(|| {
        let _val = 1 / 0;
    });
    assert!(result.is_err());
}

fn test_interrupts() {
    let start_ticks = TIMER_TICKS.load(Ordering::Relaxed);
    delay_ms(1100);
    let end_ticks = TIMER_TICKS.load(Ordering::Relaxed);

    // Should have ~1000 ticks in 1 second
    assert!(end_ticks - start_ticks > 900 && end_ticks - start_ticks < 1100);
}
```

### 10.4 Acceptance Criteria

- [ ] ACPI tables parsed correctly
- [ ] System reset works
- [ ] Power off works
- [ ] All architecture tests pass
- [ ] CI integration complete

---

## 11. Cross-Architecture Abstraction

### 11.1 Unified Platform Interface

```rust
// crates/kernel/src/platform/mod.rs

pub trait Platform: Send + Sync {
    fn name(&self) -> &'static str;
    fn arch(&self) -> Architecture;

    // Console operations
    fn console_write(&self, bytes: &[u8]);
    fn console_read(&self) -> Option<u8>;

    // Interrupt operations
    fn irq_init(&self) -> Result<(), PlatformError>;
    fn irq_enable(&self, irq: u32);
    fn irq_disable(&self, irq: u32);
    fn irq_register(&self, irq: u32, handler: IrqHandler);

    // Timer operations
    fn timer_init(&self) -> Result<(), PlatformError>;
    fn timer_read_counter(&self) -> u64;
    fn timer_frequency(&self) -> u64;

    // Memory operations
    fn memory_map(&self) -> &[MemoryRegion];
    fn map_device(&self, phys: PhysAddr, size: usize) -> Result<VirtAddr, PlatformError>;

    // Optional: PCI
    fn pci_controller(&self) -> Option<&dyn PciController>;
}

// Platform detection
pub fn detect_platform() -> Box<dyn Platform> {
    #[cfg(target_arch = "x86_64")]
    {
        Box::new(X86_64Platform::new())
    }

    #[cfg(target_arch = "aarch64")]
    {
        Box::new(AArch64Platform::new())
    }
}
```

### 11.2 Build System

```toml
# Cargo.toml
[features]
default = ["arch-auto"]
arch-auto = []
arch-x86_64 = ["x86_64", "uart_16550", "pic8259", "acpi"]
arch-aarch64 = ["cortex-a"]

[[bin]]
name = "sis_kernel_x86_64"
path = "src/main.rs"
required-features = ["arch-x86_64"]

[[bin]]
name = "sis_kernel_aarch64"
path = "src/main.rs"
required-features = ["arch-aarch64"]
```

```bash
#!/bin/bash
# scripts/build.sh

ARCH="${SIS_ARCH:-aarch64}"

case "$ARCH" in
    x86_64)
        TARGET="x86_64-unknown-none"
        UEFI_TARGET="x86_64-unknown-uefi"
        FEATURES="arch-x86_64"
        ;;
    aarch64)
        TARGET="aarch64-unknown-none"
        UEFI_TARGET="aarch64-unknown-uefi"
        FEATURES="arch-aarch64"
        ;;
    *)
        echo "Unknown architecture: $ARCH"
        exit 1
        ;;
esac

# Build kernel
cargo build \
    --target=$TARGET \
    --features=$FEATURES \
    --release

# Build UEFI boot app
cargo build \
    --target=$UEFI_TARGET \
    --package=uefi-boot \
    --release
```

---

## 12. Risk Analysis & Mitigation

### 12.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| UEFI handoff issues | High | Medium | Test with multiple OVMF versions |
| SSE/AVX requirements | High | Low | Enable early, handle missing features |
| APIC initialization failure | High | Medium | Fall back to PIC, add diagnostics |
| TSC calibration inaccuracy | Medium | Medium | Use HPET, add invariant TSC detection |
| Page table corruption | High | Low | Validate mappings, add guards |
| SMP race conditions | High | Medium | Extensive testing, lock validation |
| PCI enumeration issues | Medium | Low | Support both I/O and ECAM access |
| VirtIO version mismatch | Medium | Low | Support legacy and modern |

### 12.2 Schedule Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Debugging without hardware | Medium | Maximum QEMU validation first |
| Complex ACPI parsing | Low | Use acpi crate, minimal subset |
| Driver compatibility | Medium | Focus on VirtIO, known working |
| Testing coverage | Medium | Automated tests from day one |

---

## 13. Testing Strategy

### 13.1 Unit Tests
- Page table operations
- Interrupt handling
- GDT/IDT setup
- Frame allocator

### 13.2 Integration Tests
- Boot sequence
- Exception handling
- System calls
- Driver initialization

### 13.3 QEMU Test Matrix

```yaml
test_configurations:
  - name: "basic"
    machine: "q35"
    cpu: "qemu64"
    memory: "512M"

  - name: "smp"
    machine: "q35"
    cpu: "qemu64"
    memory: "1G"
    smp: 4

  - name: "full"
    machine: "q35"
    cpu: "host,+sse4.2,+avx2"
    memory: "2G"
    smp: 4
    devices:
      - "virtio-blk-pci"
      - "virtio-net-pci"
```

---

## 14. Timeline & Milestones

| Milestone | Duration | Dependencies | Deliverable |
|-----------|----------|--------------|-------------|
| M0: Boot | 3 days | None | Serial output |
| M1: IDT | 2 days | M0 | Exception handling |
| M2: APIC | 3 days | M1 | Timer interrupts |
| M3: Paging | 4 days | M0 | Memory management |
| M4: Syscalls | 2 days | M1, M3 | System call entry |
| M5: Serial | 1 day | M1 | Polished UART |
| M6-7: Storage/PCI | 7 days | M2, M3 | Block device, ext4 |
| M8: SMP | 5 days | M2, M3 | Multi-core support |
| M9: ACPI/Test | 4 days | All | Power, testing |

**Total: 31 days**

---

## 15. Success Criteria

### Core Requirements
- [ ] Boots in QEMU x86_64 with OVMF
- [ ] Serial console functional
- [ ] Exceptions handled gracefully
- [ ] Timer interrupts working
- [ ] Memory management integrated
- [ ] System calls functional
- [ ] Block device operational
- [ ] ext4 filesystem mounts
- [ ] SMP with 2+ cores
- [ ] Power management works

### Integration Requirements
- [ ] Existing kernel features work
- [ ] AgentSys unchanged
- [ ] VFS operations normal
- [ ] Metrics/OTel functional
- [ ] LLM inference works (if available)
- [ ] Test suite passes

### Performance Requirements
- [ ] Boot time < 2 seconds
- [ ] Interrupt latency < 10μs
- [ ] Context switch < 5μs
- [ ] Syscall overhead < 1μs

---

## 16. File Structure

```
crates/kernel/src/
├── arch/
│   ├── common/
│   │   ├── platform.rs      # Platform trait
│   │   └── irq.rs          # IRQ abstraction
│   ├── x86_64/
│   │   ├── mod.rs          # Architecture entry
│   │   ├── boot.rs         # Boot sequence
│   │   ├── cpu.rs          # CPU features
│   │   ├── gdt.rs          # Global descriptor table
│   │   ├── idt.rs          # Interrupt descriptor table
│   │   ├── tss.rs          # Task state segment
│   │   ├── paging.rs       # Page tables
│   │   ├── serial.rs       # 16550 UART
│   │   ├── pic.rs          # Legacy 8259 PIC
│   │   ├── apic.rs         # Local APIC
│   │   ├── pit.rs          # 8254 PIT
│   │   ├── hpet.rs         # HPET timer
│   │   ├── tsc.rs          # TSC support
│   │   ├── syscall.rs      # System call entry
│   │   ├── smp.rs          # Multi-processor
│   │   ├── percpu.rs       # Per-CPU data
│   │   ├── acpi.rs         # ACPI tables
│   │   └── power.rs        # Power management
│   └── aarch64/            # Existing ARM code
│       └── ...
├── drivers/
│   ├── pci/
│   │   ├── mod.rs          # PCI bus scanner
│   │   ├── ecam.rs         # MMCONFIG access
│   │   └── legacy.rs       # I/O port access
│   ├── virtio/
│   │   ├── pci.rs          # PCI transport
│   │   └── ...
│   └── serial/
│       └── uart16550.rs    # 16550 driver
└── platform/
    ├── mod.rs              # Platform abstraction
    ├── x86_64.rs          # x86_64 platform
    └── aarch64.rs         # AArch64 platform

crates/uefi-boot/src/
├── main.rs                # Common UEFI app
├── x86_64.rs             # x86_64 specific
└── aarch64.rs            # AArch64 specific

scripts/
├── build_x86_64.sh       # Build script
├── run_x86_64.sh         # QEMU runner
└── test_x86_64.sh        # Test runner
```

---

**End of Implementation Plan**