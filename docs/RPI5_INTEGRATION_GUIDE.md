# Raspberry Pi 5 Integration Guide

**Version:** 1.0
**Date:** 2025-11-15
**Milestones:** M0 (Foundation), M1 (Storage)

---

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Platform Integration](#platform-integration)
4. [Storage Integration](#storage-integration)
5. [Interrupt Management](#interrupt-management)
6. [Common Patterns](#common-patterns)
7. [Troubleshooting](#troubleshooting)
8. [Examples](#examples)

---

## 1. Overview

This guide provides practical examples for integrating Raspberry Pi 5 hardware support into your kernel code. It covers platform detection, storage access, interrupt handling, and common usage patterns.

### Prerequisites

- Raspberry Pi 5 hardware OR QEMU aarch64 virt machine
- SIS kernel with M0 and M1 implementations
- Device Tree Blob (DTB) from bootloader
- Understanding of bare-metal aarch64 programming

---

## 2. Quick Start

### 2.1 Minimal Initialization

```rust
// In your kernel's main initialization function
pub fn kernel_main(dtb_ptr: *const u8) -> ! {
    // 1. Initialize UART for early logging
    crate::drivers::uart::init();
    println!("SIS Kernel Starting...");

    // 2. Parse FDT and detect platform
    unsafe {
        if !crate::platform::override_with_dtb(dtb_ptr) {
            panic!("Failed to parse device tree");
        }
    }

    // 3. Log detected platform
    match crate::platform::detected_type() {
        PlatformType::RaspberryPi5 => {
            println!("Platform: Raspberry Pi 5 (BCM2712)");
        }
        PlatformType::QemuVirt => {
            println!("Platform: QEMU aarch64 virt");
        }
        _ => {
            println!("Platform: Unknown (using defaults)");
        }
    }

    // 4. Initialize GICv3 interrupt controller
    unsafe {
        crate::arch::aarch64::gicv3::init();
    }

    // 5. Initialize timer with 100ms interval
    unsafe {
        crate::arch::aarch64::timer::init_with_gic(100);
    }

    // 6. Initialize block devices
    unsafe {
        if let Err(e) = crate::drivers::block::init() {
            println!("Warning: Block device init failed: {:?}", e);
        }
    }

    // 7. Your kernel logic here
    kernel_loop();
}
```

### 2.2 Expected Boot Output

```
SIS Kernel Starting...
Platform: Parsing FDT at 0x40000000
Platform: Found 'uart@107d001000' compatible with 'arm,pl011'
Platform: Found 'interrupt-controller@107fef0000' compatible with 'arm,gic-v3'
Platform: Found 'mmc@1000fff0' compatible with 'arasan,sdhci-5.1'
Platform detected: RaspberryPi5
Platform: Raspberry Pi 5 (BCM2712)
Initializing Raspberry Pi 5 hardware
  SDHCI @ 0x1000fff0
GICv3: Initializing Distributor at 0x107fef0000
GICv3: Supports 128 SPIs (INTID 32-159)
GICv3: Distributor initialized
GICv3: Initializing Redistributor for CPU 0 at 0x107ff00000
GICv3: Redistributor initialized for CPU 0
GICv3: Initializing CPU Interface
GICv3: CPU Interface initialized
GICv3: Initialization complete
Timer: Frequency 54000000 Hz (54 MHz)
Timer: Interval 100 ms (5400000 ticks)
Timer: Enabled IRQ 30 in GIC
Timer: Initialization complete
Block: Initializing block devices
SDHCI: Initializing controller at 0x1000fff0
SDHCI: Version 3.0
SDHCI: Card detected
SDHCI: SD Card Version 2.0+
SDHCI: Card type: SDHC/SDXC
SDHCI: RCA = 0x1234
SDHCI: Capacity = 30535680 blocks
SDHCI: Clock set to 25000000 Hz (divisor 4)
SDHCI: Set 4-bit bus width
SDHCI: Initialization complete (30535680 blocks, 15 GB)
Block: Registered device 'mmcblk0'
Block: Initialized 1 block device(s)
```

---

## 3. Platform Integration

### 3.1 Checking Platform Type

```rust
use crate::platform::{detected_type, PlatformType};

fn platform_specific_init() {
    match detected_type() {
        PlatformType::RaspberryPi5 => {
            rpi5_specific_init();
        }
        PlatformType::QemuVirt => {
            qemu_specific_init();
        }
        PlatformType::Unknown => {
            println!("Warning: Unknown platform, using generic defaults");
        }
    }
}

fn rpi5_specific_init() {
    println!("Initializing RPi5-specific hardware...");

    // Access RPi5 platform instance
    use crate::platform::rpi5::INSTANCE;

    // Get SDHCI info
    if let Some(sdhci) = INSTANCE.sdhci_info() {
        println!("SDHCI at {:#x}", sdhci.base);
    }

    // Get PCIe info (if available)
    if let Some(pcie) = INSTANCE.pcie_info() {
        println!("PCIe controller at {:#x}", pcie.base);
    }

    // Get USB info (if available)
    if let Some(usb) = INSTANCE.usb_info() {
        println!("USB XHCI at {:#x}", usb.base);
    }
}
```

### 3.2 Accessing Platform Descriptors

```rust
use crate::platform::active;

fn get_hardware_info() {
    let platform = active();

    // UART configuration
    let uart = platform.uart();
    println!("UART: base={:#x}, clock={} Hz", uart.base, uart.clock_hz);

    // GIC configuration
    let gic = platform.gic();
    println!("GIC: GICD={:#x}, GICR={:#x}", gic.gicd, gic.gicr);

    // Timer configuration
    let timer = platform.timer();
    println!("Timer: {} Hz", timer.freq_hz);

    // Check PSCI availability
    if platform.psci_available() {
        println!("PSCI: Available for power management");
    }
}
```

---

## 4. Storage Integration

### 4.1 Basic Storage Access

```rust
use crate::drivers::block::{get_block_device, get_block_devices};
use crate::drivers::traits::BlockDevice;

fn basic_storage_example() -> Result<(), Errno> {
    // Get the SD card device
    let sd = get_block_device("mmcblk0")
        .ok_or(Errno::NoDevice)?;

    println!("SD Card Information:");
    println!("  Name: {}", sd.name());
    println!("  Block size: {} bytes", sd.block_size());
    println!("  Block count: {}", sd.block_count());
    println!("  Capacity: {} MB", sd.capacity() / 1_000_000);

    // Read first block
    let mut buffer = [0u8; 512];
    sd.read(0, &mut buffer)?;
    println!("First 16 bytes: {:02x?}", &buffer[0..16]);

    // Write test data to block 1000
    let test_data = [0xAA; 512];
    sd.write(1000, &test_data)?;

    // Read it back and verify
    sd.read(1000, &mut buffer)?;
    assert_eq!(&buffer[..], &test_data[..]);
    println!("Write/Read test: PASSED");

    Ok(())
}
```

### 4.2 Reading Multiple Blocks

```rust
fn read_multiple_blocks(
    device: &dyn BlockDevice,
    start_block: u64,
    num_blocks: usize,
) -> Result<Vec<u8>, Errno> {
    let block_size = device.block_size();
    let mut data = alloc::vec![0u8; block_size * num_blocks];

    for i in 0..num_blocks {
        let offset = i * block_size;
        device.read(start_block + i as u64, &mut data[offset..offset + block_size])?;
    }

    Ok(data)
}

// Usage
fn example_multi_block_read() -> Result<(), Errno> {
    let sd = get_block_device("mmcblk0").ok_or(Errno::NoDevice)?;

    // Read 10 blocks starting at block 100
    let data = read_multiple_blocks(&*sd, 100, 10)?;
    println!("Read {} bytes", data.len());

    Ok(())
}
```

### 4.3 Writing a Boot Sector

```rust
fn write_boot_sector(device: &dyn BlockDevice) -> Result<(), Errno> {
    // Create a simple boot sector
    let mut boot_sector = [0u8; 512];

    // Boot signature at bytes 510-511
    boot_sector[510] = 0x55;
    boot_sector[511] = 0xAA;

    // Write to block 0
    device.write(0, &boot_sector)?;
    device.sync()?;  // Ensure it's written

    println!("Boot sector written successfully");
    Ok(())
}
```

### 4.4 Enumerating All Block Devices

```rust
fn list_block_devices() {
    let devices = get_block_devices();

    println!("Block devices ({} found):", devices.len());
    for device in devices {
        println!("  {} - {} blocks ({} MB)",
            device.name(),
            device.block_count(),
            device.capacity() / 1_000_000
        );

        if device.is_readonly() {
            println!("    [READ-ONLY]");
        }
    }
}
```

---

## 5. Interrupt Management

### 5.1 Enabling Device Interrupts

```rust
use crate::arch::aarch64::gicv3;

fn enable_device_interrupts() {
    // Timer interrupt (PPI 30)
    gicv3::enable_irq(30);
    println!("Timer IRQ 30 enabled");

    // SDHCI interrupt (check device tree for actual IRQ number)
    // For RPi5, SDHCI is typically on a higher IRQ
    gicv3::enable_irq(48);  // Example
    println!("SDHCI IRQ 48 enabled");

    // USB XHCI interrupt
    gicv3::enable_irq(49);  // Example
    println!("USB IRQ 49 enabled");
}
```

### 5.2 IRQ Handler Example

```rust
use crate::arch::aarch64::gicv3;

#[no_mangle]
pub extern "C" fn irq_handler() {
    unsafe {
        // Acknowledge the interrupt and get its number
        let irq = gicv3::handle_irq();

        // Dispatch to appropriate handler
        match irq {
            30 => {
                // Timer interrupt
                handle_timer_irq();
            }
            48 => {
                // SDHCI interrupt
                handle_sdhci_irq();
            }
            49 => {
                // USB interrupt
                handle_usb_irq();
            }
            1020..=1023 => {
                // Spurious interrupt
                println!("Spurious IRQ: {}", irq);
            }
            _ => {
                println!("Unexpected IRQ: {}", irq);
            }
        }

        // Signal end of interrupt
        gicv3::eoi_irq(irq);
    }
}

fn handle_timer_irq() {
    unsafe {
        crate::arch::aarch64::timer::handle_timer_interrupt();
    }
    println!("Timer tick");
}

fn handle_sdhci_irq() {
    // Handle SDHCI completion or error
    println!("SDHCI interrupt");
}

fn handle_usb_irq() {
    // Handle USB event
    println!("USB interrupt");
}
```

### 5.3 Setting Interrupt Priorities

```rust
use crate::arch::aarch64::gicv3;

fn configure_irq_priorities() {
    // Lower value = higher priority
    // Range: 0-255

    gicv3::set_priority(30, 0x20);  // Timer (high priority)
    gicv3::set_priority(48, 0x40);  // SDHCI (medium priority)
    gicv3::set_priority(49, 0x60);  // USB (lower priority)

    println!("IRQ priorities configured");
}
```

---

## 6. Common Patterns

### 6.1 Delayed Initialization Pattern

```rust
use spin::Mutex;
use alloc::sync::Arc;

static STORAGE: Mutex<Option<Arc<dyn BlockDevice>>> = Mutex::new(None);

pub fn init_storage() -> Result<(), Errno> {
    // Get SD card
    let sd = get_block_device("mmcblk0").ok_or(Errno::NoDevice)?;

    // Store in global
    *STORAGE.lock() = Some(sd);

    println!("Storage subsystem initialized");
    Ok(())
}

pub fn get_storage() -> Option<Arc<dyn BlockDevice>> {
    STORAGE.lock().clone()
}

// Later in code
fn use_storage() -> Result<(), Errno> {
    let storage = get_storage().ok_or(Errno::NotReady)?;

    // Use storage
    let mut buf = [0u8; 512];
    storage.read(0, &mut buf)?;

    Ok(())
}
```

### 6.2 Error Recovery Pattern

```rust
fn robust_block_read(
    device: &dyn BlockDevice,
    block: u64,
    buffer: &mut [u8],
) -> Result<(), Errno> {
    const MAX_RETRIES: usize = 3;

    for attempt in 0..MAX_RETRIES {
        match device.read(block, buffer) {
            Ok(()) => return Ok(()),
            Err(e) => {
                println!("Read block {} failed (attempt {}): {:?}",
                    block, attempt + 1, e);

                if attempt < MAX_RETRIES - 1 {
                    // Wait a bit before retrying
                    crate::arch::aarch64::timer::busy_wait_ms(10);
                }
            }
        }
    }

    Err(Errno::IOError)
}
```

### 6.3 Async I/O Pattern (Polling)

```rust
pub struct BlockRequest {
    device: Arc<dyn BlockDevice>,
    block: u64,
    buffer: Vec<u8>,
    state: RequestState,
}

enum RequestState {
    Pending,
    InProgress,
    Complete(Result<(), Errno>),
}

impl BlockRequest {
    pub fn new(device: Arc<dyn BlockDevice>, block: u64, size: usize) -> Self {
        Self {
            device,
            block,
            buffer: alloc::vec![0u8; size],
            state: RequestState::Pending,
        }
    }

    pub fn poll(&mut self) -> Option<Result<(), Errno>> {
        match self.state {
            RequestState::Pending => {
                // Start the operation
                let result = self.device.read(self.block, &mut self.buffer);
                self.state = RequestState::Complete(result.clone());
                Some(result)
            }
            RequestState::Complete(ref result) => {
                Some(result.clone())
            }
            _ => None,
        }
    }
}
```

---

## 7. Troubleshooting

### 7.1 Platform Not Detected

**Symptom:** Platform detected as Unknown

**Causes:**
- FDT not parsed correctly
- FDT pointer is invalid
- Device tree is incomplete

**Solution:**
```rust
// Enable debug logging in platform detection
if !crate::platform::override_with_dtb(dtb_ptr) {
    println!("FDT parsing failed!");
    println!("DTB pointer: {:#x}", dtb_ptr as usize);

    // Try to read magic number
    unsafe {
        let magic = core::ptr::read_volatile(dtb_ptr as *const u32);
        println!("FDT magic: {:#x} (expected 0xd00dfeed)", u32::from_be(magic));
    }
}
```

### 7.2 SD Card Not Detected

**Symptom:** `SDHCI: No card detected`

**Causes:**
- No SD card inserted
- Card not properly seated
- SDHCI base address incorrect
- Card power not enabled

**Solution:**
```rust
// Check SDHCI present state register
unsafe {
    let base = 0x1000fff0; // From device tree
    let present = core::ptr::read_volatile((base + 0x24) as *const u32);

    println!("SDHCI Present State: {:#x}", present);
    println!("  Card inserted: {}", (present & (1 << 16)) != 0);
    println!("  Card stable: {}", (present & (1 << 17)) != 0);
    println!("  Card detect pin: {}", (present & (1 << 18)) != 0);
}
```

### 7.3 Timer Interrupts Not Firing

**Symptom:** No timer ticks, `handle_timer_irq` never called

**Causes:**
- GIC not initialized
- Timer IRQ not enabled
- IRQs masked in CPU
- Timer not configured correctly

**Solution:**
```rust
// Verify timer configuration
unsafe {
    let ctl: u64;
    core::arch::asm!("mrs {}, CNTP_CTL_EL0", out(reg) ctl);
    println!("Timer CTL: {:#x}", ctl);
    println!("  Enable: {}", (ctl & 1) != 0);
    println!("  Masked: {}", (ctl & 2) != 0);
    println!("  Status: {}", (ctl & 4) != 0);

    // Check if IRQs are enabled at CPU level
    let daif: u64;
    core::arch::asm!("mrs {}, DAIF", out(reg) daif);
    println!("DAIF: {:#x}", daif);
    println!("  IRQ masked: {}", (daif & (1 << 7)) != 0);
}
```

### 7.4 Block I/O Errors

**Symptom:** `Errno::IOError` on read/write operations

**Causes:**
- Block number out of range
- SD card removed
- Card not initialized
- Hardware error

**Solution:**
```rust
fn diagnose_block_io_error(device: &dyn BlockDevice, block: u64) {
    println!("Device: {}", device.name());
    println!("Block: {} (max: {})", block, device.block_count() - 1);

    if block >= device.block_count() {
        println!("ERROR: Block number out of range!");
        return;
    }

    // Try reading block 0 (should always work)
    let mut buf = [0u8; 512];
    match device.read(0, &mut buf) {
        Ok(()) => println!("Block 0 read: OK"),
        Err(e) => println!("Block 0 read: FAILED ({:?})", e),
    }

    // Try the problematic block
    match device.read(block, &mut buf) {
        Ok(()) => println!("Block {} read: OK", block),
        Err(e) => println!("Block {} read: FAILED ({:?})", block, e),
    }
}
```

---

## 8. Examples

### 8.1 Simple File System Bootloader

```rust
// Simple bootloader that reads kernel from SD card
fn load_kernel_from_sd() -> Result<(), Errno> {
    println!("Loading kernel from SD card...");

    // Get SD card
    let sd = get_block_device("mmcblk0").ok_or(Errno::NoDevice)?;

    // Read MBR (block 0)
    let mut mbr = [0u8; 512];
    sd.read(0, &mut mbr)?;

    // Check boot signature
    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        return Err(Errno::InvalidArgument);
    }

    println!("MBR signature valid");

    // Read kernel starting at block 2048 (1MB offset)
    const KERNEL_START: u64 = 2048;
    const KERNEL_SIZE: usize = 512 * 1024;  // 512KB
    const KERNEL_LOAD_ADDR: usize = 0x80000;

    let blocks_to_read = KERNEL_SIZE / 512;
    let kernel_ptr = KERNEL_LOAD_ADDR as *mut u8;

    for i in 0..blocks_to_read {
        let mut buf = [0u8; 512];
        sd.read(KERNEL_START + i as u64, &mut buf)?;

        unsafe {
            core::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                kernel_ptr.add(i * 512),
                512
            );
        }

        if (i % 100) == 0 {
            println!("  Loaded {} KB...", (i * 512) / 1024);
        }
    }

    println!("Kernel loaded at {:#x}", KERNEL_LOAD_ADDR);
    Ok(())
}
```

### 8.2 Block Device Benchmark

```rust
use crate::arch::aarch64::timer;

fn benchmark_storage(device: &dyn BlockDevice) {
    const NUM_BLOCKS: usize = 1000;
    let mut buffer = [0u8; 512];

    // Sequential read benchmark
    let start = timer::read_cntpct();
    for i in 0..NUM_BLOCKS {
        device.read(i as u64, &mut buffer).unwrap();
    }
    let end = timer::read_cntpct();

    let freq = timer::read_cntfrq();
    let elapsed_ms = ((end - start) * 1000) / freq;
    let throughput = (NUM_BLOCKS * 512) / (elapsed_ms as usize);  // KB/s

    println!("Sequential Read:");
    println!("  Blocks: {}", NUM_BLOCKS);
    println!("  Time: {} ms", elapsed_ms);
    println!("  Throughput: {} KB/s", throughput);

    // Sequential write benchmark
    let test_data = [0xAA; 512];
    let start = timer::read_cntpct();
    for i in 0..NUM_BLOCKS {
        device.write(10000 + i as u64, &test_data).unwrap();
    }
    let end = timer::read_cntpct();

    let elapsed_ms = ((end - start) * 1000) / freq;
    let throughput = (NUM_BLOCKS * 512) / (elapsed_ms as usize);  // KB/s

    println!("Sequential Write:");
    println!("  Blocks: {}", NUM_BLOCKS);
    println!("  Time: {} ms", elapsed_ms);
    println!("  Throughput: {} KB/s", throughput);
}
```

### 8.3 Partition Table Parser

```rust
#[repr(C, packed)]
struct MbrPartitionEntry {
    status: u8,
    first_chs: [u8; 3],
    partition_type: u8,
    last_chs: [u8; 3],
    first_lba: u32,
    num_sectors: u32,
}

fn parse_partitions(device: &dyn BlockDevice) -> Result<(), Errno> {
    let mut mbr = [0u8; 512];
    device.read(0, &mut mbr)?;

    // Check signature
    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        return Err(Errno::InvalidArgument);
    }

    println!("Partition table:");

    // Parse 4 partition entries (at offset 446)
    for i in 0..4 {
        let offset = 446 + (i * 16);
        let entry = unsafe {
            &*(mbr.as_ptr().add(offset) as *const MbrPartitionEntry)
        };

        if entry.partition_type != 0 {
            println!("  Partition {}:", i + 1);
            println!("    Type: {:#x}", entry.partition_type);
            println!("    Start LBA: {}", entry.first_lba);
            println!("    Sectors: {}", entry.num_sectors);
            println!("    Size: {} MB",
                (entry.num_sectors as u64 * 512) / 1_000_000);
        }
    }

    Ok(())
}
```

---

## Summary

This integration guide provides comprehensive examples for:
- ✅ Platform detection and initialization
- ✅ Block device access and management
- ✅ Interrupt handling with GICv3
- ✅ Common usage patterns and best practices
- ✅ Troubleshooting common issues
- ✅ Real-world examples (bootloader, benchmark, partition parser)

For more information, see:
- `docs/RPI5_HARDWARE_IMPLEMENTATION.md` - M0 Foundation details
- `docs/RPI5_M1_STORAGE.md` - SDHCI driver specifics
- `docs/RPI5_REVIEW_M0_M1.md` - Code review and quality assessment

---

**Happy Kernel Hacking!** 🚀
