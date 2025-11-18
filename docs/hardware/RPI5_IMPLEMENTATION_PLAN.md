# Raspberry Pi 5 Hardware Boot Implementation Plan

**Status:** 🚧 In Progress
**Target Hardware:** Raspberry Pi 5 (BCM2712 SoC)
**Current Platform:** Mac Mini M1 (development)
**Test Strategy:** QEMU validation → Real hardware deployment

---

## Executive Summary

This document outlines the comprehensive plan to enable SIS kernel boot on Raspberry Pi 5 hardware while maintaining full compatibility with the existing QEMU development workflow. The implementation uses feature flags to separate platform-specific code, allowing simultaneous support for both environments.

### Key Objectives

1. ✅ Enable RPi5 boot from UEFI firmware (EDK2)
2. ✅ Maintain QEMU development workflow on Mac Mini
3. ✅ Test RPi5 code paths in QEMU before hardware arrival
4. ✅ Minimize code duplication through platform abstraction
5. ✅ Document hardware-specific quirks and workarounds

---

## Current State Analysis

### What We Have ✅

| Component | Status | Location | Notes |
|-----------|--------|----------|-------|
| Platform Abstraction | ✅ Complete | `src/platform/mod.rs` | Trait-based, runtime detection |
| RPi5 Module | ✅ Complete | `src/platform/rpi5.rs` | 260 lines, BCM2712 support |
| Device Tree Parser | ✅ Complete | `src/platform/dt.rs` | FDT parsing, device detection |
| QEMU Module | ✅ Complete | `src/platform/qemu_virt.rs` | Baseline implementation |
| UEFI Boot Loader | ✅ Complete | `crates/uefi-boot/` | Generic UEFI entry point |
| USB Boot Scripts | ✅ Complete | `scripts/create_bootable_usb_mac.sh` | Mac-compatible |
| Hardware Build Script | ⚠️ Basic | `scripts/hw_build.sh` | Needs RPi5 enhancements |

### What Needs Implementation 🚧

| Component | Priority | Effort | Dependencies |
|-----------|----------|--------|--------------|
| RPi5-Specific Boot Flow | P0 | Medium | UEFI loader mods |
| RP1 I/O Hub Initialization | P0 | High | PCIe, USB drivers |
| SDHCI Driver (SD card) | P0 | High | Block device layer |
| PL011 UART Verification | P1 | Low | Existing driver |
| GICv3 Interrupt Routing | P1 | Medium | IRQ subsystem |
| USB XHCI Driver | P1 | High | USB stack |
| Ethernet Driver (GENET) | P2 | High | Network stack |
| GPIO Driver | P2 | Medium | RP1 access |
| Power Management | P3 | Medium | PSCI integration |
| Performance Tuning | P3 | Low | Benchmarking |

---

## Architecture Overview

### Platform Detection Flow

```text
┌─────────────────────────────────────────────────────────────────┐
│                      UEFI Firmware Boot                         │
│  (EDK2 for RPi5 or QEMU UEFI)                                  │
└──────────────────────┬──────────────────────────────────────────┘
                       │
                       │ Pass FDT address in x0
                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Kernel Entry (EL1)                           │
│  crates/uefi-boot/src/main.rs                                  │
└──────────────────────┬──────────────────────────────────────────┘
                       │
                       │ Parse FDT
                       ▼
┌─────────────────────────────────────────────────────────────────┐
│              Platform Detection (runtime)                       │
│  src/platform/mod.rs::override_with_dtb()                      │
│                                                                 │
│  Check FDT compatible strings:                                 │
│  - "raspberrypi,5-model-b" → RPi5                             │
│  - "brcm,bcm2712" → RPi5                                      │
│  - "qemu,virt" → QEMU                                         │
└──────────────────────┬──────────────────────────────────────────┘
                       │
                       │ Select platform implementation
                       ▼
        ┌──────────────┴──────────────┐
        │                             │
        ▼                             ▼
┌───────────────┐           ┌─────────────────┐
│  QEMU Path    │           │   RPi5 Path     │
│               │           │                 │
│ qemu_virt.rs  │           │   rpi5.rs       │
│               │           │                 │
│ PL011 @ 0x9.. │           │ PL011 @ 0x107.. │
│ GICv3 (virt)  │           │ GICv3 (BCM2712) │
│ VirtIO        │           │ RP1 I/O Hub     │
└───────────────┘           └─────────────────┘
```

### Feature Flag Strategy

We use **runtime detection** rather than compile-time feature flags for platform selection. This allows a single kernel binary to boot on both QEMU and RPi5.

**Conditional Compilation** (feature flags) is used for:
- Optional drivers (e.g., `rpi5-gpio`, `rpi5-mailbox`)
- Development/debug features
- Experimental functionality

**Runtime Selection** (FDT-based) is used for:
- Platform type (QEMU vs RPi5)
- Device addresses
- Interrupt mappings
- Clock frequencies

---

## Implementation Phases

### Phase 1: QEMU Validation (Week 1)

**Goal:** Test RPi5 boot path in QEMU without real hardware

#### 1.1 QEMU RPi5 Emulation Setup

```bash
# Install QEMU with RPi5 support (if available)
brew install qemu  # macOS

# Create RPi5 QEMU boot script
./scripts/uefi_run_rpi5_qemu.sh
```

**Script requirements:**
- Use `-machine raspi4` (closest to RPi5)
- Load UEFI firmware for ARM
- Pass device tree with RPi5 compatible strings
- Enable UART for console output

#### 1.2 Boot Loader Enhancements

**File:** `crates/uefi-boot/src/main.rs`

**Changes needed:**
```rust
// Enhanced FDT validation
fn validate_fdt(fdt_ptr: *const u8) -> Result<(), &'static str> {
    // Check magic number: 0xd00dfeed
    // Verify totalsize < 1MB (sanity check)
    // Validate structure offsets
}

// Platform-specific boot parameters
fn setup_rpi5_boot_params() -> BootParams {
    BootParams {
        // RPi5 specific:
        // - Disable D-cache early (RP1 I/O hub requirement)
        // - Set up PCIe BAR windows
        // - Configure RP1 clock gates
    }
}
```

#### 1.3 Early Console Verification

**File:** `src/platform/rpi5.rs`

Test PL011 UART at correct address:
```rust
pub const RPI5_UART_BASE: usize = 0x107d001000;
pub const RPI5_UART_CLOCK: u32 = 48_000_000;  // 48MHz

// Test function
pub fn test_uart_early() {
    let uart = unsafe { &mut *(RPI5_UART_BASE as *mut PL011) };
    uart.send_str("RPi5 boot: UART OK\r\n");
}
```

**Expected output in QEMU:**
```
[0.000] SIS Kernel v0.1.0 (aarch64)
[0.001] Platform detected: RaspberryPi5
[0.002] RPi5 boot: UART OK
[0.003] GICv3 initialization...
```

#### 1.4 Device Tree Validation

**File:** `src/platform/dt.rs`

Add RPi5-specific device extraction:
```rust
pub fn parse_rpi5_devices(fdt: &Fdt) -> DeviceMap {
    DeviceMap {
        uart: find_pl011(fdt),
        gic: find_gicv3(fdt),
        sdhci: find_sdhci_bcm2712(fdt),  // NEW
        pcie: find_pcie_rp1(fdt),        // NEW
        usb: find_xhci_rp1(fdt),         // NEW
        ethernet: find_genet(fdt),       // NEW
    }
}

// Example: SDHCI detection
fn find_sdhci_bcm2712(fdt: &Fdt) -> Option<SdhciInfo> {
    // Look for compatible = "brcm,bcm2712-sdhci"
    // Extract reg property for base address
    // Extract interrupts property
}
```

**Validation checklist:**
- [ ] UART detected and functional
- [ ] GIC distributor/redistributor addresses correct
- [ ] SDHCI device found (even if driver not ready)
- [ ] PCIe controller detected
- [ ] USB XHCI controller detected
- [ ] Ethernet controller detected

---

### Phase 2: Core Driver Implementation (Week 2-3)

#### 2.1 SDHCI Driver (SD Card)

**Priority:** P0 (required for storage)

**File:** `src/drivers/sdhci_bcm2712.rs` (new)

**Capabilities needed:**
- Block device read/write
- DMA transfers (if supported)
- Error handling
- Partition table parsing (GPT)

**Implementation notes:**
- BCM2712 SDHCI is Arasan SDHCI 5.1 controller
- Base address from FDT (typically `0x1000fff0`)
- Supports SDMA and ADMA2
- Clock gating via RP1 I/O hub

**Testing in QEMU:**
```bash
# Create virtual SD card image
dd if=/dev/zero of=sd.img bs=1M count=512
mkfs.ext4 sd.img

# Boot QEMU with SD card
qemu-system-aarch64 -machine raspi4 \
  -sd sd.img \
  -kernel kernel.elf \
  -dtb bcm2712-rpi-5-b.dtb
```

**Expected behavior:**
```
[0.150] SDHCI: Initializing BCM2712 controller @ 0x1000fff0
[0.151] SDHCI: Version 5.1 detected
[0.152] SDHCI: Card detected, capacity: 512MB
[0.153] SDHCI: Partition table (GPT) read OK
[0.154] SDHCI: Mounted ext4 filesystem on /
```

#### 2.2 RP1 I/O Hub Initialization

**Priority:** P0 (critical for USB, Ethernet, GPIO)

**File:** `src/drivers/rp1_io_hub.rs` (new)

**What is RP1?**
- Custom I/O controller chip on RPi5
- Connected via PCIe Gen 2 ×4
- Provides USB, Ethernet, GPIO, SPI, I2C
- Manages peripherals that were on SoC in RPi4

**Initialization sequence:**
```rust
pub struct Rp1IoHub {
    pcie_base: usize,
    config_space: usize,
}

impl Rp1IoHub {
    pub fn init() -> Result<Self, &'static str> {
        // 1. Initialize PCIe controller
        pcie_init()?;

        // 2. Enumerate PCIe devices
        let devices = pcie_enumerate()?;

        // 3. Find RP1 (vendor ID: 0x1de4, device ID: 0x0001)
        let rp1 = devices.find_rp1()?;

        // 4. Map RP1 BAR0 (configuration space)
        let config_space = map_bar(rp1.bar0)?;

        // 5. Initialize RP1 subsystems
        rp1_usb_init(config_space)?;
        rp1_eth_init(config_space)?;
        rp1_gpio_init(config_space)?;

        Ok(Self { pcie_base, config_space })
    }
}
```

**PCIe Requirements:**
- PCIe root complex initialization
- Configuration space access (ECAM or legacy)
- BAR mapping and address translation
- MSI/MSI-X interrupt setup

#### 2.3 USB XHCI Driver

**Priority:** P1 (for USB keyboard, mass storage)

**File:** `src/drivers/usb_xhci_rp1.rs` (new)

**Dependencies:**
- RP1 I/O hub initialized
- PCIe BAR mapped
- Interrupt routing configured

**Features:**
- USB 2.0 and 3.0 support
- Multiple root hub ports (RPi5 has 2× USB 3.0 + 2× USB 2.0)
- Mass storage class (for USB boot)
- HID class (keyboard/mouse)

**Testing:**
```
[0.200] RP1: USB XHCI controller initialized
[0.201] XHCI: 4 ports detected
[0.202] XHCI: Port 1: USB 3.0 device connected
[0.203] USB: Mass storage device detected (SanDisk Ultra 32GB)
```

#### 2.4 Ethernet Driver (GENET)

**Priority:** P2 (for network boot, NFS)

**File:** `src/drivers/ethernet_genet.rs` (new)

**Controller:** Broadcom GENET (Gigabit Ethernet)

**Features:**
- 10/100/1000 Mbps
- RGMII interface
- Hardware checksum offload
- Wake-on-LAN

**Use cases:**
- Network boot (PXE/TFTP)
- NFS root filesystem
- Remote debugging (GDB over Ethernet)

---

### Phase 3: USB Boot Image Creation (Week 3)

#### 3.1 Bootable USB Layout

```
/dev/sdX (USB drive)
├── Partition 1 (ESP - EFI System Partition)
│   ├── /EFI/
│   │   ├── BOOT/
│   │   │   └── BOOTAA64.EFI  ← UEFI boot loader
│   │   └── SIS/
│   │       └── KERNEL.ELF     ← SIS kernel
│   └── /models/                ← Optional: LLM models
│       └── tinyllama-1.1b.gguf
└── Partition 2 (Root filesystem - ext4)
    ├── /bin/
    ├── /etc/
    └── /lib/
```

#### 3.2 Enhanced Boot Script

**File:** `scripts/create_rpi5_usb.sh` (new)

```bash
#!/bin/bash
# Create bootable USB for Raspberry Pi 5
set -e

USB_DEVICE="${1:-/dev/sdX}"
ESP_SIZE=512M
ROOT_SIZE=remainder

echo "Creating RPi5 bootable USB on $USB_DEVICE"

# 1. Partition
parted -s $USB_DEVICE mklabel gpt
parted -s $USB_DEVICE mkpart ESP fat32 1MiB ${ESP_SIZE}
parted -s $USB_DEVICE set 1 esp on
parted -s $USB_DEVICE mkpart ROOT ext4 ${ESP_SIZE} 100%

# 2. Format
mkfs.vfat -F32 -n ESP ${USB_DEVICE}1
mkfs.ext4 -L ROOT ${USB_DEVICE}2

# 3. Mount and copy
mkdir -p /tmp/usb_esp /tmp/usb_root
mount ${USB_DEVICE}1 /tmp/usb_esp
mount ${USB_DEVICE}2 /tmp/usb_root

# Copy UEFI boot files
cp target/aarch64-unknown-uefi/release/uefi-boot.efi /tmp/usb_esp/EFI/BOOT/BOOTAA64.EFI
cp target/aarch64-unknown-none/debug/sis_kernel /tmp/usb_esp/EFI/SIS/KERNEL.ELF

# Copy optional files (models, config)
if [ -f models/tinyllama-1.1b.gguf ]; then
    mkdir -p /tmp/usb_esp/models
    cp models/*.gguf /tmp/usb_esp/models/
fi

# Sync and unmount
sync
umount /tmp/usb_esp /tmp/usb_root
```

#### 3.3 UEFI Firmware for RPi5

RPi5 uses **EDK2-based UEFI firmware**:

**Option 1:** Use official RPi5 UEFI firmware
```bash
# Download from Raspberry Pi Foundation
wget https://github.com/pftf/RPi5/releases/latest/download/RPi5_UEFI.zip
unzip RPi5_UEFI.zip
# Copy RPI_EFI.fd to SD card
```

**Option 2:** Build from source
```bash
git clone https://github.com/pftf/RPi5.git
cd RPi5
./build.sh
```

**SD Card Setup (for UEFI boot):**
```
RPi5 SD Card:
├── bootcode.bin      ← GPU bootloader
├── config.txt        ← RPi config
├── RPI_EFI.fd        ← UEFI firmware
└── overlays/
```

**config.txt for UEFI:**
```ini
[pi5]
kernel=RPI_EFI.fd
arm_64bit=1
disable_overscan=1

# Optional: UART enable
enable_uart=1
uart_2ndstage=1
```

---

### Phase 4: Real Hardware Boot (Week 4)

#### 4.1 Initial Boot Checklist

**Hardware Setup:**
- [ ] Raspberry Pi 5 (4GB or 8GB model)
- [ ] USB-C power supply (5V 5A, 27W)
- [ ] USB-to-Serial adapter (for UART debugging)
- [ ] Bootable USB drive (from Phase 3)
- [ ] SD card with UEFI firmware
- [ ] HDMI display (optional, for visual confirmation)

**Connection Diagram:**
```
┌─────────────────────────────────────────┐
│      Raspberry Pi 5                     │
│                                         │
│  [SD Card Slot] ← UEFI firmware SD     │
│  [USB Port 1]   ← Bootable USB drive   │
│  [USB Port 2]   ← Keyboard (optional)  │
│  [GPIO Header]  ← UART (pins 8,10,GND) │
│  [HDMI]         ← Display              │
│  [USB-C]        ← 5V 5A Power          │
└─────────────────────────────────────────┘
         │
         │ UART (115200 8N1)
         ▼
┌─────────────────┐
│  Mac Mini M1    │
│  (screen/cu)    │
└─────────────────┘
```

**UART Connection (GPIO Header):**
```
RPi5 GPIO       USB-to-Serial
Pin 8  (TX)  →  RX
Pin 10 (RX)  ←  TX
Pin 6  (GND) →  GND
```

#### 4.2 First Boot Procedure

**Terminal setup (Mac):**
```bash
# Find USB-to-Serial device
ls /dev/tty.usb*

# Connect to UART
screen /dev/tty.usbserial-XXXXX 115200

# Or use cu
cu -l /dev/tty.usbserial-XXXXX -s 115200
```

**Power on sequence:**
1. Insert SD card (UEFI firmware)
2. Insert USB drive (SIS kernel)
3. Connect UART
4. Connect power
5. Watch console output

**Expected boot sequence:**
```
[GPU Bootloader]
Raspberry Pi Bootcode
UEFI Firmware v1.0

[UEFI]
Scanning boot devices...
Found: USB Mass Storage (SanDisk)
Loading: \EFI\BOOT\BOOTAA64.EFI

[SIS Kernel]
[0.000] SIS Kernel v0.1.0 (aarch64)
[0.001] UEFI Boot: FDT @ 0x08000000
[0.002] Platform detected: RaspberryPi5 (BCM2712)
[0.003] CPU: 4× Cortex-A76 @ 2.4GHz
[0.004] RAM: 8GB LPDDR4X-4267
[0.005] GICv3 initialization @ 0x107fef0000
[0.006] Timer: ARM Generic Timer @ 54MHz
[0.007] UART: PL011 @ 0x107d001000 (48MHz)
[0.008] MMU: 4KB pages, 48-bit VA
[0.100] SDHCI: BCM2712 controller initialized
[0.150] RP1: PCIe I/O Hub detected
[0.200] USB: XHCI controller initialized (4 ports)
[0.250] Shell ready.
sis>
```

#### 4.3 Debugging Failed Boot

**Scenario 1: No UART output**
- Check UART connections (TX/RX swapped?)
- Verify baud rate (115200)
- Check GPIO pin numbers (BCM vs physical)
- Try different terminal program

**Scenario 2: UEFI boots, kernel doesn't**
```
# UEFI shell commands to debug
fs0:
ls \EFI\BOOT\
# Should see BOOTAA64.EFI

ls \EFI\SIS\
# Should see KERNEL.ELF

# Try manual load
\EFI\BOOT\BOOTAA64.EFI
```

**Scenario 3: Kernel panics early**
```
# Common causes:
# - FDT not passed correctly (x0 corrupted)
# - MMU mapping failure (incorrect page tables)
# - GIC initialization failure (wrong addresses)
# - Stack overflow (stack too small)

# Enable debug output in kernel
SIS_FEATURES="bringup,debug-verbose" ./scripts/hw_build.sh
```

**Scenario 4: Hangs during device init**
```
# Likely causes:
# - Waiting for hardware that doesn't exist
# - Infinite loop in driver
# - Interrupt storm
# - Deadlock in synchronization

# Add timeout to device probing
pub fn init_with_timeout(timeout_ms: u64) -> Result<(), &'static str> {
    let start = timer::get_ticks();
    loop {
        if device_ready() { return Ok(()); }
        if timer::elapsed_ms(start) > timeout_ms {
            return Err("Device init timeout");
        }
    }
}
```

#### 4.4 Performance Validation

**Benchmarks to run:**
```rust
// Boot time
[0.000] → [0.500] = 500ms (target: <1s)

// UART throughput
Transmit: 115200 baud = 14.4 KB/s

// SD card read speed
Sequential: ~80 MB/s (UHS-I)
Random 4K: ~10 MB/s

// USB 3.0 throughput
Theoretical: 5 Gbps = 625 MB/s
Practical: 400 MB/s

// Network throughput (Gigabit Ethernet)
Theoretical: 1000 Mbps = 125 MB/s
Practical: 900-950 Mbps
```

---

## Feature Flag Design

### Cargo.toml Features

```toml
[features]
default = ["bringup"]

# Platform targets
rpi5 = ["rpi5-gpio", "rpi5-mailbox", "sdhci-bcm2712"]
qemu-virt = []

# RPi5-specific drivers
rpi5-gpio = []
rpi5-mailbox = []
sdhci-bcm2712 = []
rp1-io-hub = []

# Optional functionality
usb-xhci = []
ethernet-genet = []
pcie-support = []

# Build modes
hw-minimal = []  # Minimal build for real hardware
debug-verbose = []  # Extra logging
```

### Build Commands

```bash
# Development (QEMU on Mac)
BRINGUP=1 ./scripts/uefi_run.sh build

# RPi5 in QEMU (testing)
SIS_FEATURES="rpi5" BRINGUP=1 ./scripts/uefi_run_rpi5_qemu.sh

# RPi5 real hardware
SIS_FEATURES="rpi5,hw-minimal" ./scripts/hw_build.sh

# Full RPi5 with all drivers
SIS_FEATURES="rpi5,usb-xhci,ethernet-genet,pcie-support" ./scripts/hw_build.sh
```

### Conditional Compilation Examples

```rust
// Platform-specific initialization
#[cfg(feature = "rpi5")]
fn init_platform() {
    rp1::init_io_hub();
    sdhci::init_bcm2712();
}

#[cfg(not(feature = "rpi5"))]
fn init_platform() {
    virtio::init();
}

// Optional drivers
#[cfg(feature = "usb-xhci")]
mod usb {
    pub fn init() { /* ... */ }
}

// Runtime checks (preferred for platform detection)
match platform::detected_type() {
    PlatformType::RaspberryPi5 => {
        info!("Running on RPi5, enabling SDHCI");
        sdhci::init();
    }
    PlatformType::QemuVirt => {
        info!("Running on QEMU, enabling VirtIO");
        virtio::init();
    }
    _ => {
        warn!("Unknown platform, minimal init");
    }
}
```

---

## Testing Strategy

### QEMU Testing Matrix

| Test | QEMU Machine | Boot Method | Expected Result |
|------|--------------|-------------|-----------------|
| Baseline | `-machine virt` | EFI | ✅ Boot to shell |
| RPi5 Emulation | `-machine raspi4` | EFI | ✅ Detect RPi5 platform |
| RPi5 with FDT | `-machine raspi4 -dtb bcm2712.dtb` | EFI | ✅ Parse BCM2712 devices |
| USB Boot | Add `-usb -device usb-storage,drive=usbdrive` | EFI | ✅ Detect USB mass storage |

### Hardware Testing Checklist

**Phase 4.1: Minimal Boot**
- [ ] Power on, see GPU bootloader
- [ ] UEFI firmware loads
- [ ] Kernel starts, prints banner
- [ ] UART console works
- [ ] Panic handler works (test with deliberate panic)

**Phase 4.2: Core Functionality**
- [ ] GICv3 interrupts work (timer tick)
- [ ] UART transmit/receive
- [ ] MMU enables successfully
- [ ] Exception handling (test with SVC call)
- [ ] Shell accepts input

**Phase 4.3: Storage**
- [ ] SDHCI controller detected
- [ ] SD card detected
- [ ] Read sector 0 (MBR/GPT)
- [ ] Mount root filesystem

**Phase 4.4: USB**
- [ ] RP1 I/O hub detected
- [ ] PCIe enumeration works
- [ ] XHCI controller initialized
- [ ] USB device hotplug (insert USB stick)
- [ ] Read USB mass storage

**Phase 4.5: Network** (if time permits)
- [ ] Ethernet controller detected
- [ ] Link up (LED activity)
- [ ] DHCP request
- [ ] Ping test
- [ ] HTTP request

---

## Risk Assessment

### High Risk Items

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| UEFI firmware incompatibility | High | Medium | Test multiple firmware versions |
| RP1 I/O hub documentation sparse | High | High | Reverse engineer Linux driver |
| PCIe enumeration fails | High | Medium | Use minimal BAR setup, test in QEMU |
| SDHCI driver bugs (data corruption) | Critical | Low | Extensive testing, write protection |
| GIC interrupt routing errors | High | Medium | Test with timer first, then others |

### Medium Risk Items

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Device tree incompatibility | Medium | Medium | Support multiple FDT versions |
| Clock frequency misdetection | Medium | Low | Read from registers, not hardcode |
| USB hotplug race conditions | Medium | Medium | Proper locking, state machine |
| Ethernet PHY initialization | Low | High | Copy Linux sequence exactly |

---

## Documentation Requirements

### Files to Create

1. **`docs/hardware/RPI5_HARDWARE_GUIDE.md`**
   - Detailed BCM2712 register maps
   - RP1 I/O hub architecture
   - Clock tree diagram
   - Power domains

2. **`docs/hardware/RPI5_BOOT_FLOW.md`**
   - Step-by-step boot sequence
   - Firmware hand-off protocol
   - Memory map at each stage

3. **`docs/hardware/RPI5_TROUBLESHOOTING.md`**
   - Common boot failures
   - UART debugging guide
   - JTAG setup (if available)

4. **`docs/drivers/SDHCI_BCM2712.md`**
   - Driver architecture
   - DMA setup
   - Error handling

5. **`docs/drivers/RP1_IO_HUB.md`**
   - PCIe initialization
   - BAR mapping
   - Subsystem initialization

### Code Documentation

All RPi5-specific code must have:
- Module-level documentation (`//!`)
- Function documentation for public APIs
- Inline comments for hardware-specific quirks
- References to datasheets/errata

Example:
```rust
//! SDHCI BCM2712 Driver
//!
//! This module implements support for the Arasan SDHCI 5.1 controller
//! found on the Broadcom BCM2712 SoC (Raspberry Pi 5).
//!
//! # Hardware Reference
//! - BCM2712 Datasheet: Section 8.3 "SD Host Controller"
//! - SDHCI Spec 5.1: https://www.sdcard.org/downloads/pls/
//!
//! # Known Issues
//! - DMA transfers >64KB require chaining (SDMA limitation)
//! - Clock gating must be disabled during init (RP1 requirement)
//!
//! # Example
//! ```no_run
//! let sdhci = SdhciBcm2712::init(0x1000fff0)?;
//! sdhci.read_block(0, &mut buffer)?;
//! ```

/// Initialize the SDHCI controller
///
/// # Arguments
/// * `base` - Physical base address from FDT
///
/// # Returns
/// Initialized controller instance or error
///
/// # Safety
/// Must be called after MMU initialization and RP1 I/O hub setup
pub fn init(base: usize) -> Result<Self, &'static str> {
    // Implementation...
}
```

---

## Success Criteria

### Minimal Success (MVP)

**Definition:** Kernel boots to shell on real RPi5 hardware

**Requirements:**
- ✅ Power on → UART output visible
- ✅ Platform detected as RPi5
- ✅ GICv3 initialized, timer ticking
- ✅ MMU enabled, virtual memory working
- ✅ Shell prompt appears
- ✅ Basic commands work (`help`, `info`)

### Functional Success

**Definition:** Core I/O works

**Additional requirements:**
- ✅ SD card detected and readable
- ✅ USB device detection
- ✅ Filesystem mounting
- ✅ LLM model loading from SD card

### Full Success

**Definition:** All major subsystems operational

**Additional requirements:**
- ✅ Ethernet link up and working
- ✅ USB mass storage read/write
- ✅ GPIO control (LED blink test)
- ✅ Performance meets targets (boot <1s)

---

## Timeline Estimates

### Conservative Timeline (4 weeks)

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1 | QEMU validation | RPi5 boot in QEMU, FDT parsing |
| 2 | Core drivers | SDHCI driver, RP1 init |
| 3 | USB and integration | XHCI driver, USB boot image |
| 4 | Hardware testing | Real RPi5 boot, debugging |

### Aggressive Timeline (2 weeks)

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1 | QEMU + basic drivers | Boot in QEMU, SDHCI stub |
| 2 | Hardware bringup | Real RPi5 boot, minimal I/O |

**Recommended:** Conservative timeline for production quality

---

## Open Questions

1. **UEFI Firmware Version**
   - Q: Which EDK2 firmware version for RPi5?
   - A: Use latest stable from pftf/RPi5 releases

2. **RP1 Documentation**
   - Q: Official RP1 I/O hub datasheet available?
   - A: No public datasheet, reverse engineer from Linux driver

3. **SD Card Compatibility**
   - Q: Which SD card brands/models tested?
   - A: Test with SanDisk Ultra/Extreme, Samsung EVO

4. **Performance Targets**
   - Q: What's acceptable boot time?
   - A: <1s to shell (UEFI → shell prompt)

5. **Network Stack**
   - Q: TCP/IP stack implementation?
   - A: Phase 2 work, use smoltcp (already in codebase)

---

## References

### Official Documentation
- [Raspberry Pi 5 Product Brief](https://www.raspberrypi.com/products/raspberry-pi-5/)
- [BCM2712 SoC Overview](https://datasheets.raspberrypi.com/bcm2712/bcm2712-peripherals.pdf)
- [RPi5 UEFI Firmware](https://github.com/pftf/RPi5)

### Kernel Development
- [ARM GICv3 Architecture](https://developer.arm.com/documentation/198123/0302)
- [UEFI Specification 2.10](https://uefi.org/specs/UEFI/2.10/)
- [Device Tree Specification](https://www.devicetree.org/specifications/)

### Similar Projects
- [Tock OS on RPi5](https://github.com/tock/tock/tree/master/boards/raspberry_pi_pico)
- [seL4 on RPi5](https://github.com/seL4/seL4/tree/master/src/plat/bcm2711)
- [Zephyr RTOS RPi Support](https://docs.zephyrproject.org/latest/boards/raspberrypi/index.html)

### Driver References
- [Linux SDHCI Driver](https://github.com/torvalds/linux/blob/master/drivers/mmc/host/sdhci-brcmstb.c)
- [Linux RP1 Driver](https://github.com/raspberrypi/linux/tree/rpi-6.1.y/drivers/misc/rp1)
- [U-Boot RPi5 Support](https://github.com/u-boot/u-boot/tree/master/board/raspberrypi/rpi)

---

**Last Updated:** 2025-11-17
**Status:** 🚧 Ready for Phase 1 (QEMU validation)
**Next Action:** Create `scripts/uefi_run_rpi5_qemu.sh` and test boot flow
