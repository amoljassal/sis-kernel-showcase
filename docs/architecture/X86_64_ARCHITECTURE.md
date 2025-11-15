# x86_64 Architecture Implementation

**Status:** Milestone M2 Complete (APIC & High Precision Timer)
**Last Updated:** 2025-11-15
**Architecture:** Intel/AMD 64-bit (x86_64)
**Boot Method:** UEFI
**Target Platform:** QEMU x86_64 (Primary), Bare Metal (Future)

---

## Executive Summary

This document describes the x86_64 architecture implementation for the SIS kernel. The implementation follows a milestone-based approach (M0-M9) as outlined in `IMPLEMENTATION_PLAN_X86_64.md`. As of this update, **Milestones M0 (Skeleton Boot), M1 (Interrupts & Exceptions), and M2 (APIC & High Precision Timer)** have been completed, providing a fully functional boot environment with modern APIC-based interrupt handling, high-precision timing via HPET, and improved TSC calibration.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Milestone M0: Skeleton Boot](#milestone-m0-skeleton-boot)
3. [Milestone M1: Interrupts & Exceptions](#milestone-m1-interrupts--exceptions)
4. [Milestone M2: APIC & High Precision Timer](#milestone-m2-apic--high-precision-timer)
5. [Module Organization](#module-organization)
6. [Memory Layout](#memory-layout)
7. [Boot Sequence](#boot-sequence)
8. [CPU Feature Management](#cpu-feature-management)
9. [Exception Handling](#exception-handling)
10. [Interrupt Handling](#interrupt-handling)
11. [Serial Console](#serial-console)
12. [Time Keeping](#time-keeping)
13. [Future Milestones](#future-milestones)
14. [Testing](#testing)
15. [References](#references)

---

## Architecture Overview

### Platform Characteristics

- **Architecture:** x86_64 (Intel/AMD 64-bit)
- **Execution Mode:** 64-bit Long Mode
- **Page Size:** 4 KiB (default), 2 MiB, 1 GiB (huge pages)
- **Address Space:** 48-bit virtual addresses (256 TB)
- **Privilege Levels:** Ring 0 (kernel), Ring 3 (user)
- **Interrupt Controller:** APIC (Legacy PIC for early boot)
- **Timer:** TSC (Time Stamp Counter), HPET, PIT (Legacy)

### Key Design Decisions

1. **UEFI-First Boot**: Leverages UEFI firmware for initial setup, avoids legacy BIOS
2. **Modern Features Only**: Targets CPUs with APIC, SSE2, and NX support
3. **Security-Focused**: Enables NX, SMEP, SMAP when available
4. **Platform Abstraction**: Unified interface for both x86_64 and AArch64
5. **Incremental Development**: 10 milestones from basic boot to full SMP

---

## Milestone M0: Skeleton Boot

**Status:** ✅ **COMPLETE**
**Duration:** 3 days (estimated)
**Completion Date:** 2025-11-15

### Objectives

Milestone M0 establishes the minimal viable boot environment:

- ✅ UEFI handoff and kernel entry
- ✅ CPU initialization (GDT, IDT, TSS)
- ✅ Serial console (16550 UART)
- ✅ Exception handlers (basic)
- ✅ CPU feature detection and enablement
- ✅ TSC initialization

### Components Implemented

#### 1. Global Descriptor Table (GDT)

**File:** `crates/kernel/src/arch/x86_64/gdt.rs`

The GDT defines memory segments for x86_64. While segmentation is largely legacy in 64-bit mode, the GDT is still required for:
- Code/data segment selectors (CS, DS, ES, SS)
- Task State Segment (TSS) descriptor
- SYSCALL/SYSRET instruction operation

**GDT Structure:**
```
Index  Segment          DPL   Type      Usage
-----  ---------------  ---   -------   ---------------------------
0      Null Descriptor  -     -         Required by CPU
1      Kernel Code      0     Code      Kernel mode execution
2      Kernel Data      0     Data      Kernel mode data access
3      User Data        3     Data      User mode data access
4      User Code        3     Code      User mode execution
5      TSS              0     System    Task State Segment
```

**Key Functions:**
- `init_gdt()`: Loads GDT and sets segment registers
- Selector getters for kernel/user segments

#### 2. Task State Segment (TSS)

**File:** `crates/kernel/src/arch/x86_64/tss.rs`

The TSS provides:
- **Privilege Stack Table (RSP0)**: Kernel stack for syscalls/interrupts
- **Interrupt Stack Table (IST)**: Dedicated stacks for critical exceptions

**IST Assignments:**
- IST[0]: Double Fault (#DF) - 16 KiB stack
- IST[1]: NMI (Non-Maskable Interrupt) - 16 KiB stack
- IST[2]: Machine Check (#MC) - 16 KiB stack

**Key Functions:**
- `init_tss()`: Initializes TSS with dedicated stacks
- `set_kernel_stack()`: Updates kernel stack pointer (for context switching)
- `validate_tss()`: Debug validation of TSS configuration

#### 3. Interrupt Descriptor Table (IDT)

**File:** `crates/kernel/src/arch/x86_64/idt.rs`

The IDT contains handlers for all CPU exceptions (0-31). Each exception has a dedicated handler that:
- Logs exception details
- Displays register state
- Panics (for M0, recovery will be added in later milestones)

**Exception Handlers Implemented:**
- Divide Error (#DE)
- Debug (#DB)
- Non-Maskable Interrupt (NMI)
- Breakpoint (#BP)
- Overflow (#OF)
- Bound Range Exceeded (#BR)
- Invalid Opcode (#UD)
- Device Not Available (#NM)
- **Double Fault (#DF)** - Uses IST stack
- Invalid TSS (#TS)
- Segment Not Present (#NP)
- Stack Segment Fault (#SS)
- **General Protection Fault (#GP)** - Detailed error decoding
- **Page Fault (#PF)** - Shows CR2 (faulting address) and error code
- x87 FPU Error (#MF)
- Alignment Check (#AC)
- Machine Check (#MC)
- SIMD Floating-Point (#XM)
- Virtualization Exception (#VE)

**Key Functions:**
- `init_idt_early()`: Loads IDT with exception handlers

#### 4. CPU Initialization

**File:** `crates/kernel/src/arch/x86_64/cpu.rs`

Handles CPU feature detection and enablement using CPUID instruction.

**Required Features (Validated):**
- SSE2 (required for Rust floating-point)
- APIC (required for interrupt handling)
- TSC (required for timekeeping)

**Optional Features (Enabled if Available):**
- **SIMD:** SSE3, SSE4.1, SSE4.2, AVX, AVX2
- **Security:** NX (No-Execute), SMEP, SMAP
- **Performance:** FSGSBASE, x2APIC, PCID, INVPCID
- **Timekeeping:** Invariant TSC

**Control Registers Modified:**
- **CR0:** FPU control (EM=0, MP=1)
- **CR4:** SSE (OSFXSR, OSXMMEXCPT), AVX (OSXSAVE), SMEP, SMAP, PCID, PGE, FSGSBASE
- **EFER (MSR):** NX bit (No-Execute Enable)
- **XCR0:** AVX state management (x87, SSE, AVX)

**Key Functions:**
- `detect_cpu_features()`: Returns CpuFeatures struct with capability flags
- `enable_cpu_features()`: Enables all required and optional features
- `print_cpu_info()`: Displays vendor, model, and feature list

#### 5. Serial Console (16550 UART)

**File:** `crates/kernel/src/arch/x86_64/serial.rs`

Provides early boot console via COM1 (0x3F8) serial port.

**Configuration:**
- **Baud Rate:** 115200
- **Data Bits:** 8
- **Parity:** None
- **Stop Bits:** 1
- **FIFO:** Enabled

**Key Functions:**
- `init_serial()`: Initializes COM1
- `serial_write()`: Writes byte array to serial port
- `serial_print!()`: Formatted print macro
- `serial_println!()`: Print with newline

**Usage:**
```rust
serial_println!("Boot complete: {} MHz", cpu_freq_mhz);
```

#### 6. Time Stamp Counter (TSC)

**File:** `crates/kernel/src/arch/x86_64/tsc.rs`

The TSC provides high-resolution CPU cycle counting.

**Calibration Methods (in priority order):**
1. **CPUID.15H** - TSC frequency from CPU
2. **MSR 0xCE** - Platform info (Intel)
3. **HPET** - Hardware calibration (M2)
4. **PIT** - Legacy timer calibration (M2)
5. **Fallback** - Assume 1 GHz (inaccurate)

**Key Functions:**
- `read_tsc()`: Reads TSC counter (non-serializing)
- `read_tsc_serialized()`: Reads TSC with CPUID fence (accurate timing)
- `read_tscp()`: Reads TSC + CPU ID
- `tsc_to_ns()`: Converts TSC ticks to nanoseconds
- `ns_to_tsc()`: Converts nanoseconds to TSC ticks
- `init_tsc()`: Calibrates TSC frequency

#### 7. Boot Sequence

**File:** `crates/kernel/src/arch/x86_64/boot.rs`

Coordinates early boot initialization.

**Boot Flow:**
```
UEFI Firmware (OVMF)
    ↓
UEFI Boot Application
    ↓ Loads kernel ELF
    ↓ Exits boot services
    ↓ Jumps to _start
_start (main.rs)
    ↓
early_init() [boot.rs]
    ├── Disable interrupts
    ├── Load GDT
    ├── Load TSS
    ├── Load IDT
    ├── Enable CPU features
    ├── Initialize serial
    └── Initialize TSC
    ↓
print_boot_info()
    ↓
Idle loop (HLT instruction)
```

**Key Functions:**
- `early_init()`: Orchestrates all early boot steps
- `validate_hardware()`: Checks for required CPU features
- `print_boot_info()`: Displays system configuration
- `halt_forever()`: Fatal error handler

#### 8. Module Structure

**File:** `crates/kernel/src/arch/x86_64/mod.rs`

Main architecture module with comprehensive documentation.

**Key Types:**
- `CpuContext`: Register state for context switching (rbx, rbp, r12-r15, rsp, rip, rflags, fs_base, gs_base)

**Utility Functions:**
- `halt()`: HLT instruction
- `halt_loop()`: Infinite HLT loop
- `read_tsc()`: TSC shortcut
- `rdmsr()`/`wrmsr()`: Model-Specific Register access
- `invlpg()`: Invalidate TLB entry
- `flush_tlb()`: Flush entire TLB

### Acceptance Criteria

All M0 acceptance criteria have been met:

- ✅ UEFI boots UEFI application successfully
- ✅ Kernel receives control in 64-bit long mode
- ✅ Serial console prints boot banner
- ✅ GDT, TSS, IDT loaded correctly
- ✅ CPU features enabled (SSE2, NX, etc.)
- ✅ TSC calibrated
- ✅ No triple faults or CPU resets during boot
- ✅ Exception handlers handle faults gracefully

---

## Milestone M1: Interrupts & Exceptions

**Status:** ✅ **COMPLETE**
**Duration:** 2 days (estimated)
**Completion Date:** 2025-11-15

### Objectives

Milestone M1 implements hardware interrupt support and timer-based interrupts:

- ✅ Legacy PIC (8259A) initialization and configuration
- ✅ PIT (Programmable Interval Timer) driver
- ✅ Timer interrupt handling (IRQ 0)
- ✅ Keyboard interrupt stub (IRQ 1)
- ✅ EOI (End of Interrupt) handling
- ✅ Spurious interrupt detection

### Components Implemented

#### 1. Legacy PIC (8259A)

**File:** `crates/kernel/src/arch/x86_64/pic.rs`

The Programmable Interrupt Controller manages hardware interrupts (IRQs) in cascaded dual-PIC configuration.

**Features:**
- Vector remapping: IRQ 0-15 → vectors 32-47 (avoids conflict with CPU exceptions)
- Master PIC: IRQ 0-7 (vectors 32-39)
- Slave PIC: IRQ 8-15 (vectors 40-47)
- Individual IRQ masking
- EOI acknowledgment
- Spurious interrupt detection

**Key Functions:**
- `init()`: Initialize and remap PIC
- `enable_irq(irq)`: Unmask specific IRQ
- `disable_irq(irq)`: Mask specific IRQ
- `end_of_interrupt(vector)`: Send EOI

#### 2. PIT (Programmable Interval Timer)

**File:** `crates/kernel/src/arch/x86_64/pit.rs`

The 8253/8254 PIT provides timer interrupts at configurable frequencies.

**Features:**
- Configured for 1000 Hz (1 ms per tick)
- Atomic tick counter for uptime tracking
- TSC calibration support
- Microsecond-precision busy-wait delays

**Key Functions:**
- `init(freq_hz)`: Initialize PIT to specified frequency
- `tick()`: Increment tick counter (called from IRQ handler)
- `ticks()`: Get current tick count
- `uptime_secs()`: Get uptime in seconds
- `calibrate_tsc(duration_ms)`: Calibrate TSC using PIT
- `udelay(us)`: Busy-wait microsecond delay

#### 3. IDT Interrupt Handlers

**File:** `crates/kernel/src/arch/x86_64/idt.rs` (updated)

Added hardware interrupt handlers for:
- **Timer (IRQ 0 / Vector 32)**: Increments PIT tick counter, sends EOI
- **Keyboard (IRQ 1 / Vector 33)**: Reads scancode from port 0x60, sends EOI
- **Serial (IRQ 4 / Vector 36)**: Stub for future serial driver
- **Spurious (IRQ 7/15 / Vectors 39/47)**: Handles spurious PIC interrupts

#### 4. Boot Sequence Integration

**File:** `crates/kernel/src/arch/x86_64/boot.rs` (updated)

Added M1 initialization steps:
1. Initialize PIC and remap to vectors 32-47
2. Initialize PIT at 1000 Hz
3. Enable timer interrupt (IRQ 0)
4. Enable interrupts globally (STI)

### Acceptance Criteria

All M1 acceptance criteria have been met:

- ✅ PIC initialized and remapped correctly
- ✅ PIT generates timer interrupts at 1000 Hz
- ✅ Timer interrupt handler executes successfully
- ✅ Tick counter increments accurately
- ✅ EOI sent correctly to PIC
- ✅ No spurious interrupts or interrupt storms
- ✅ Keyboard interrupts acknowledged (scancode read)
- ✅ System remains stable with interrupts enabled

---

## Milestone M2: APIC & High Precision Timer

**Status:** ✅ **COMPLETE**
**Duration:** 2 days (estimated)
**Completion Date:** 2025-11-15

### Objectives

Milestone M2 implements modern APIC-based interrupt handling and high-precision timing:

- ✅ Local APIC (xAPIC and x2APIC) detection and initialization
- ✅ HPET (High Precision Event Timer) driver
- ✅ Improved TSC calibration using HPET
- ✅ Dynamic EOI routing (APIC or PIC)
- ✅ Boot sequence integration

### Components Implemented

#### 1. Local APIC (Advanced Programmable Interrupt Controller)

**File:** `crates/kernel/src/arch/x86_64/apic.rs`

The Local APIC is the modern replacement for the legacy PIC, providing per-CPU interrupt control.

**Supported Modes:**
- **xAPIC**: Memory-mapped I/O at physical address 0xFEE00000
- **x2APIC**: MSR-based (faster, supports more CPUs)

**Features:**
- APIC detection via CPUID
- Mode selection (xAPIC vs x2APIC)
- APIC initialization and enabling
- Software enable/disable
- EOI handling
- IPI (Inter-Processor Interrupt) support (for future SMP)
- Timer configuration support (for future use)

**Key Functions:**
- `init()`: Detect and initialize Local APIC
- `get()`: Get reference to global APIC instance
- `eoi()`: Send End-of-Interrupt signal
- `send_ipi()`: Send IPI to other CPUs (future SMP)

**APIC Registers:**
- APIC ID: CPU identification
- Task Priority Register (TPR): Interrupt priority
- EOI Register: Acknowledge interrupts
- Spurious Interrupt Vector Register: APIC enable/disable
- Interrupt Command Register (ICR): Send IPIs

#### 2. HPET (High Precision Event Timer)

**File:** `crates/kernel/src/arch/x86_64/hpet.rs`

The HPET is a high-resolution hardware timer that replaces the legacy PIT for precise timing.

**Features:**
- 64-bit counter with femtosecond precision
- Frequency specified in ACPI (no calibration needed)
- Multiple timer comparators (typically 3-32)
- Memory-mapped registers
- TSC calibration support

**Key Functions:**
- `init()`: Initialize HPET at default address (0xFED00000)
- `get()`: Get reference to global HPET instance
- `read_counter()`: Read current 64-bit counter value
- `frequency()`: Get HPET frequency in Hz
- `ns_to_ticks()`: Convert nanoseconds to HPET ticks
- `ticks_to_ns()`: Convert HPET ticks to nanoseconds
- `calibrate_tsc()`: Calibrate TSC using HPET
- `delay_ns()`: Busy-wait nanosecond delay

**HPET Characteristics:**
- Default base address: 0xFED00000 (standard location)
- Typical frequency: ~14.3 MHz
- Counter period: Specified in capabilities register (femtoseconds)
- Never overflows (64-bit counter takes ~58,000 years at 14 MHz)

#### 3. Improved TSC Calibration

**File:** `crates/kernel/src/arch/x86_64/tsc.rs` (updated)

Enhanced TSC calibration with multi-tier fallback strategy:

**Calibration Priority:**
1. **CPUID.15H**: TSC frequency from CPUID (most accurate, if available)
2. **MSR 0xCE**: Intel IA32_PLATFORM_INFO (Intel CPUs only)
3. **HPET**: Calibrate against HPET (NEW in M2, accurate)
4. **PIT**: Calibrate against PIT (M1, less accurate)
5. **Fallback**: Assume 1 GHz (very inaccurate)

The HPET-based calibration provides significantly better accuracy than PIT while being more widely available than CPUID-based methods.

#### 4. Dynamic EOI Handling

**File:** `crates/kernel/src/arch/x86_64/idt.rs` (updated)

Implemented `send_eoi()` helper function that automatically routes EOI to the correct interrupt controller:

```rust
unsafe fn send_eoi(vector: u8) {
    if let Some(_apic) = apic::get() {
        apic::eoi();  // Use APIC EOI
    } else {
        pic::end_of_interrupt(vector);  // Fall back to PIC
    }
}
```

All interrupt handlers updated to use `send_eoi()` instead of calling PIC directly.

#### 5. Boot Sequence Integration

**File:** `crates/kernel/src/arch/x86_64/boot.rs` (updated)

Added M2 initialization steps:
1. Initialize HPET (before TSC calibration)
2. Initialize TSC (now uses HPET if available)
3. Initialize Local APIC (after PIC, before enabling interrupts)
4. Enable interrupts globally

The system gracefully falls back to PIC/PIT if APIC/HPET are not available.

### Acceptance Criteria

All M2 acceptance criteria have been met:

- ✅ APIC detected and initialized correctly (xAPIC or x2APIC)
- ✅ HPET detected and initialized at 0xFED00000
- ✅ TSC calibration uses HPET (if available) or falls back to PIT
- ✅ EOI routing works with both APIC and PIC
- ✅ Timer interrupts still work after APIC initialization
- ✅ System boots successfully with APIC enabled
- ✅ System falls back gracefully if APIC/HPET unavailable
- ✅ No interrupt delivery issues or system hangs

---

## Module Organization

```
crates/kernel/src/arch/x86_64/
├── mod.rs          # Main module, architecture interface
├── boot.rs         # Boot sequence orchestration
├── cpu.rs          # CPU initialization and feature detection
├── gdt.rs          # Global Descriptor Table
├── idt.rs          # Interrupt Descriptor Table
├── tss.rs          # Task State Segment
├── serial.rs       # 16550 UART serial driver
├── tsc.rs          # Time Stamp Counter
├── pic.rs          # Legacy 8259A PIC (M1) ✅
├── pit.rs          # Programmable Interval Timer (M1) ✅
├── apic.rs         # Local APIC (M2) ✅
└── hpet.rs         # High Precision Event Timer (M2) ✅

Future modules (M3-M9):
├── paging.rs       # 4-level page tables (M3)
├── syscall.rs      # SYSCALL/SYSRET entry (M4)
├── smp.rs          # SMP support (M8)
├── percpu.rs       # Per-CPU data (M8)
├── ioapic.rs       # I/O APIC (M2/M8)
├── acpi.rs         # ACPI tables (M9)
└── power.rs        # Power management (M9)
```

---

## Memory Layout

### Virtual Address Space (48-bit)

```
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_F000   User Space (128 TB)
0x0000_7FFF_FFFF_F000 - 0x0000_8000_0000_0000   Guard Page

[Canonical Address Hole - Invalid Addresses]

0xFFFF_8000_0000_0000 - 0xFFFF_8800_0000_0000   Kernel Image (512 GB)
0xFFFF_8800_0000_0000 - 0xFFFF_9000_0000_0000   Kernel Heap (512 GB)
0xFFFF_9000_0000_0000 - 0xFFFF_A000_0000_0000   Device MMIO (1 TB)
0xFFFF_A000_0000_0000 - 0xFFFF_B000_0000_0000   PCI ECAM Space (1 TB)
0xFFFF_B000_0000_0000 - 0xFFFF_C000_0000_0000   Per-CPU Data (1 TB)
0xFFFF_C000_0000_0000 - 0xFFFF_FFFF_8000_0000   Reserved
0xFFFF_FFFF_8000_0000 - 0xFFFF_FFFF_FFFF_FFFF   Direct Map (512 GB)
```

### Physical Memory (Typical QEMU Setup)

```
0x0000_0000_0000_0000 - 0x0000_0000_000A_0000   Low Memory
0x0000_0000_0010_0000 - DRAM_END                Main Memory (1-4 GB)
0x0000_0000_FEC0_0000 - 0x0000_0000_FEC0_1000   IOAPIC MMIO
0x0000_0000_FED0_0000 - 0x0000_0000_FED0_1000   HPET MMIO
0x0000_0000_FEE0_0000 - 0x0000_0000_FEE1_0000   Local APIC MMIO
0x0000_00E0_0000_0000 - 0x0000_00F0_0000_0000   PCI ECAM (256 buses)
```

---

## Boot Sequence

### UEFI to Kernel Handoff

1. **OVMF UEFI Firmware** starts
2. **UEFI Boot Application** (`crates/uefi-boot`) loads kernel ELF
3. **Exit Boot Services** - UEFI hands control to kernel
4. **Jump to _start** in kernel

### Kernel Entry Point (`_start`)

**File:** `crates/kernel/src/main.rs:200-224`

```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        arch::boot::early_init().expect("Boot failed");
    }
    arch::boot::print_boot_info();
    loop { x86_64::instructions::hlt(); }
}
```

### Early Initialization (`early_init()`)

**File:** `crates/kernel/src/arch/x86_64/boot.rs`

1. **Disable Interrupts** - CLI instruction
2. **Load GDT** - Set up segmentation
3. **Load TSS** - Enable privilege transitions
4. **Load IDT** - Set up exception handlers
5. **Enable CPU Features** - SSE, AVX, NX, SMEP, SMAP
6. **Initialize Serial** - COM1 for logging
7. **Print Boot Banner** - Display kernel info
8. **Print CPU Info** - Vendor, model, features
9. **Initialize TSC** - Calibrate timer
10. **Validate TSS** - Debug check (debug builds only)

---

## CPU Feature Management

### Feature Detection

Uses CPUID instruction to query CPU capabilities:

```rust
let features = detect_cpu_features();
if features.has_avx2 {
    // Enable AVX2 optimizations
}
```

### Enabled Features

| Feature | Purpose | Required | Status |
|---------|---------|----------|--------|
| SSE2 | Rust floating-point | Yes | ✅ Enabled |
| SSE3/4.1/4.2 | SIMD performance | No | ✅ Enabled if available |
| AVX/AVX2 | Advanced SIMD | No | ✅ Enabled if available |
| NX | No-Execute pages | No | ✅ Enabled if available |
| SMEP | Kernel can't execute user code | No | ✅ Enabled if available |
| SMAP | Kernel can't access user data | No | ✅ Enabled if available |
| FSGSBASE | Fast TLS/per-CPU access | No | ✅ Enabled if available |
| PCID | TLB efficiency | No | ✅ Enabled if available |
| APIC | Interrupt handling | Yes | ✅ Validated |
| TSC | Timekeeping | Yes | ✅ Validated |

---

## Exception Handling

### Exception Strategy

- **Exceptions 0-31**: CPU-defined exceptions (all handled)
- **Vectors 32-255**: Hardware interrupts (M1+)

### Critical Exceptions

#### Double Fault (#DF)

**Dedicated IST Stack**: IST[0] = 16 KiB

Prevents triple fault when exception handling fails.

#### Page Fault (#PF)

Shows CR2 (faulting address) and error code bits:
- **P**: Present (0) or protection violation (1)
- **W/R**: Read (0) or write (1)
- **U/S**: Kernel (0) or user (1)
- **RSVD**: Reserved bit violation
- **I/D**: Instruction fetch

#### General Protection Fault (#GP)

Decodes error code:
- Selector index (bits 3-15)
- External event (bit 0)
- IDT reference (bit 2)

---

## Serial Console

### Hardware Configuration

- **Port**: COM1 (0x3F8)
- **IRQ**: 4 (not used in M0, polling only)
- **Baud**: 115200
- **Format**: 8N1 (8 data, no parity, 1 stop)

### Usage

```rust
// Basic output
serial_write(b"Hello, world!\n");

// Formatted output
serial_println!("CPU frequency: {} MHz", freq_mhz);
```

### QEMU Integration

QEMU maps serial port to stdio:
```bash
qemu-system-x86_64 -serial stdio ...
```

All `serial_write()` output appears in terminal.

---

## Time Keeping

### TSC Calibration

**Priority Order:**
1. CPUID.15H (most accurate)
2. MSR IA32_PLATFORM_INFO (Intel)
3. HPET (M2)
4. PIT (M2)
5. Fallback: 1 GHz

**Current Status:**
- M0: CPUID and MSR calibration implemented
- M2: HPET/PIT calibration will be added

### Time Conversion

```rust
let start = read_tsc();
// ... work ...
let end = read_tsc();
let elapsed_ns = tsc_to_ns(end - start);
```

---

## Future Milestones

### M1: Interrupts & Exceptions (2 days)

- ✅ Complete IDT with all exception handlers
- ⏳ Basic interrupt handling infrastructure
- ⏳ Legacy PIC initialization (8259A)
- ⏳ PIT timer for early timekeeping

### M2: APIC & High Precision Timer (3 days)

- ⏳ Detect and initialize Local APIC
- ⏳ Replace PIC with APIC
- ⏳ HPET initialization
- ⏳ Accurate TSC calibration

### M3: Paging & Memory Management (4 days)

- ⏳ 4-level page table management (PML4)
- ⏳ Integration with existing memory manager
- ⏳ Kernel/user address space separation
- ⏳ Demand paging support

### M4: Syscall Entry (2 days)

- ⏳ SYSCALL/SYSRET fast path
- ⏳ System call dispatcher
- ⏳ Integration with existing syscall table

### M5: Serial/TTY Polish (1 day)

- ⏳ Interrupt-driven serial I/O
- ⏳ FIFO management
- ⏳ /dev/ttyS0 device node

### M6-M7: Block Storage & PCI (7 days)

- ⏳ PCI bus enumeration
- ⏳ VirtIO-blk over PCI
- ⏳ ext4 filesystem mounting

### M8: SMP Support (5 days)

- ⏳ Secondary CPU bring-up (INIT-SIPI-SIPI)
- ⏳ Per-CPU data structures
- ⏳ Load balancing scheduler

### M9: ACPI & Testing (4 days)

- ⏳ ACPI table parsing
- ⏳ Power management
- ⏳ Comprehensive testing

**Total Timeline:** 31 days

---

## Testing

### Test Strategy

1. **Unit Tests** - Individual module tests
2. **Integration Tests** - Boot sequence tests
3. **QEMU Tests** - Full system tests in emulator
4. **Bare Metal** - Real hardware validation (future)

### Current Test Status

**M0 Tests:**
- ✅ GDT selector validation
- ✅ TSS stack alignment
- ✅ TSS initialization
- ✅ CPU feature detection
- ✅ Serial write operations
- ✅ TSC monotonicity
- ✅ TSC conversion (ns ↔ ticks)

### QEMU Test Command

```bash
qemu-system-x86_64 \
    -bios /usr/share/OVMF/OVMF_CODE.fd \
    -drive file=esp_x86_64.img,format=raw,if=virtio \
    -serial stdio \
    -m 1G \
    -cpu qemu64,+sse2,+sse3,+sse4.1,+sse4.2 \
    -machine q35 \
    -no-reboot
```

---

## References

### Intel Manuals

- [Intel® 64 and IA-32 Architectures Software Developer's Manual Volume 3](https://www.intel.com/content/www/us/en/architecture-and-technology/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.html)
- [Intel® 64 and IA-32 Architectures Optimization Reference Manual](https://www.intel.com/content/www/us/en/architecture-and-technology/64-ia-32-architectures-optimization-manual.html)

### AMD Manuals

- [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://www.amd.com/system/files/TechDocs/24593.pdf)

### UEFI Specification

- [UEFI Specification 2.10](https://uefi.org/specifications)

### x86_64 Crate

- [x86_64 Rust crate documentation](https://docs.rs/x86_64/)

### Related Documents

- `docs/plans/IMPLEMENTATION_PLAN_X86_64.md` - Full implementation plan
- `docs/architecture/MEMORY_LAYOUT.md` - Memory layout details
- `docs/architecture/BOOT_SEQUENCE.md` - Boot process documentation

---

## Change Log

| Date | Milestone | Changes |
|------|-----------|---------|
| 2025-11-15 | M0 | Initial x86_64 implementation complete |
| 2025-11-15 | M0 | All core modules implemented and documented |
| TBD | M1 | Interrupt handling (planned) |

---

## Contact / Maintainers

This implementation is part of the SIS kernel showcase project.

For questions or issues, please refer to the main project documentation.

---

**Document Version:** 1.0
**Last Updated:** 2025-11-15
**Status:** Milestone M0 Complete ✅
