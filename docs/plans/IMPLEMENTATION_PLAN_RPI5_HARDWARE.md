# Implementation Plan: Raspberry Pi 5 Hardware Boot Support

**Status:** Planning
**Target Hardware:** Raspberry Pi 5 (BCM2712, Cortex-A76)
**Boot Method:** UEFI (EDK2 firmware) → EL1 kernel entry
**Primary Validation:** QEMU aarch64 virt with driver self-tests
**Secondary Validation:** Real RPi 5 hardware boot

---

## Executive Summary

This plan provides end-to-end hardware enablement for Raspberry Pi 5, maintaining compatibility with existing QEMU-based testing infrastructure. All drivers will be abstracted with FDT-based configuration, allowing QEMU validation before hardware testing. The implementation follows a staged approach (M0-M8) with clear acceptance criteria at each milestone.

**Key Constraints:**
- Must maintain QEMU boot compatibility throughout
- All MMIO addresses sourced from FDT (no hardcoded addresses)
- Driver self-tests must pass in QEMU before hardware validation
- Existing Phase 0-9 functionality must remain intact
- No regressions in stub-to-real implementations (P0-P7, P9)

---

## Architecture Overview

### Boot Flow
```
UEFI Firmware (EDK2 RPi)
  ↓
UEFI Boot Application (crates/uefi-boot)
  ↓ (loads kernel ELF)
Kernel Entry (EL1, MMU off)
  ↓
Platform Detection (FDT parsing)
  ↓
RPi5 Platform Init (UART, GIC, Timer)
  ↓
Driver Initialization (SDHCI, PCIe, USB, Net)
  ↓
Filesystem Mount (ext4 on SD/NVMe)
  ↓
Userspace Init (shell, agents)
```

### Platform Abstraction
```rust
// New module structure
crates/kernel/src/platform/
  ├── mod.rs          // Platform trait + detection logic
  ├── fdt.rs          // Device Tree parser
  ├── qemu_virt.rs    // Existing QEMU platform
  └── rpi5.rs         // NEW: RPi5 platform descriptor

pub trait Platform {
    fn name(&self) -> &str;
    fn uart_base(&self) -> usize;
    fn gic_dist_base(&self) -> usize;
    fn gic_redist_base(&self) -> usize;
    fn timer_frequency(&self) -> u64;
    fn detect_devices(&self, fdt: &[u8]) -> DeviceMap;
}
```

---

## Phase Breakdown

## **M0: Foundation (Platform + Console + Interrupts + Time)**

### Objectives
1. Boot on RPi5 via UEFI with FDT detection
2. Serial console output via PL011 UART
3. GICv3 interrupt handling
4. ARM Generic Timer operational at 1Hz
5. QEMU continues to boot with existing UART

### Components

#### M0.1: FDT Parser
**Files:**
- `crates/kernel/src/platform/fdt.rs` (NEW)

**Implementation:**
```rust
pub struct FdtParser<'a> {
    blob: &'a [u8],
}

impl<'a> FdtParser<'a> {
    /// Parse FDT header and validate magic
    pub fn new(blob: &'a [u8]) -> Result<Self, FdtError>;

    /// Find node by path (e.g., "/soc/serial@7d001000")
    pub fn find_node(&self, path: &str) -> Option<FdtNode>;

    /// Get property from node
    pub fn get_property(&self, node: &FdtNode, name: &str) -> Option<&[u8]>;

    /// Parse reg property to (base_addr, size)
    pub fn parse_reg(&self, prop: &[u8]) -> Vec<(u64, u64)>;

    /// Parse interrupts property
    pub fn parse_interrupts(&self, prop: &[u8]) -> Vec<u32>;

    /// Parse compatible string
    pub fn parse_compatible(&self, prop: &[u8]) -> Vec<&str>;
}

pub struct DeviceMap {
    pub uart: Option<UartInfo>,
    pub gic: Option<GicInfo>,
    pub timer: Option<TimerInfo>,
    pub sdhci: Option<SdhciInfo>,
    pub pcie: Option<PcieInfo>,
    pub usb: Option<UsbInfo>,
    pub ethernet: Option<EthInfo>,
}

#[derive(Debug, Clone)]
pub struct UartInfo {
    pub base: usize,
    pub irq: u32,
    pub clock: u32,
    pub compatible: &'static str, // "arm,pl011" or "brcm,bcm2835-aux-uart"
}

#[derive(Debug, Clone)]
pub struct GicInfo {
    pub dist_base: usize,
    pub redist_base: usize,
    pub version: u32, // 2 or 3
}

#[derive(Debug, Clone)]
pub struct TimerInfo {
    pub frequency: u64,
    pub irq_phys: u32,
    pub irq_virt: u32,
}

#[derive(Debug, Clone)]
pub struct SdhciInfo {
    pub base: usize,
    pub irq: u32,
    pub quirks: u32, // SDHCI_QUIRK_* flags
}
```

**FDT Parsing Strategy:**
1. Locate FDT blob address (passed by UEFI in x0 or via UEFI config table)
2. Validate magic (0xd00dfeed) and size
3. Walk structure blocks to find compatible devices
4. Extract reg/interrupts/clock-frequency properties
5. Populate DeviceMap for platform init

**Acceptance Criteria:**
- [ ] Parses QEMU virt FDT successfully (existing devices detected)
- [ ] Parses RPi5 FDT with PL011, GICv3, SDHCI nodes
- [ ] Handles missing nodes gracefully (returns None)
- [ ] Prints device map at boot: `[PLATFORM] Detected: PL011@0x7d001000, GICv3@0x...`

---

#### M0.2: Platform Detection & Selection
**Files:**
- `crates/kernel/src/platform/mod.rs` (MODIFY)
- `crates/kernel/src/platform/rpi5.rs` (NEW)
- `crates/kernel/src/main.rs` (MODIFY)

**Implementation:**
```rust
// platform/mod.rs
pub mod fdt;
pub mod qemu_virt;
pub mod rpi5;

pub trait Platform: Send + Sync {
    fn name(&self) -> &'static str;
    fn uart_base(&self) -> usize;
    fn gic_dist_base(&self) -> usize;
    fn gic_redist_base(&self) -> usize;
    fn timer_frequency(&self) -> u64;
    fn init(&self, fdt_blob: Option<&[u8]>) -> Result<(), PlatformError>;
}

pub enum PlatformType {
    QemuVirt,
    RaspberryPi5,
    Unknown,
}

/// Detect platform from FDT compatible string or fallback heuristics
pub fn detect_platform(fdt_blob: Option<&[u8]>) -> PlatformType {
    if let Some(fdt) = fdt_blob {
        let parser = FdtParser::new(fdt).ok()?;

        // Check root compatible for "raspberrypi,5-model-b"
        if let Some(root) = parser.find_node("/") {
            if let Some(compat) = parser.get_property(&root, "compatible") {
                let compat_str = parser.parse_compatible(compat);
                if compat_str.contains(&"raspberrypi,5-model-b") {
                    return PlatformType::RaspberryPi5;
                }
                if compat_str.contains(&"linux,dummy-virt") {
                    return PlatformType::QemuVirt;
                }
            }
        }
    }

    // Fallback: probe UART at known addresses
    PlatformType::Unknown
}

pub fn get_platform(ptype: PlatformType) -> &'static dyn Platform {
    match ptype {
        PlatformType::QemuVirt => &qemu_virt::QEMU_VIRT_PLATFORM,
        PlatformType::RaspberryPi5 => &rpi5::RPI5_PLATFORM,
        PlatformType::Unknown => &qemu_virt::QEMU_VIRT_PLATFORM, // safe default
    }
}
```

```rust
// platform/rpi5.rs
use super::{Platform, PlatformError};
use crate::platform::fdt::{FdtParser, DeviceMap};

pub struct Rpi5Platform {
    devices: spin::Once<DeviceMap>,
}

pub static RPI5_PLATFORM: Rpi5Platform = Rpi5Platform {
    devices: spin::Once::new(),
};

impl Platform for Rpi5Platform {
    fn name(&self) -> &'static str {
        "Raspberry Pi 5 (BCM2712)"
    }

    fn init(&self, fdt_blob: Option<&[u8]>) -> Result<(), PlatformError> {
        let fdt = fdt_blob.ok_or(PlatformError::NoDeviceTree)?;
        let parser = FdtParser::new(fdt)?;

        // Parse all devices
        let mut devmap = DeviceMap::default();

        // UART: look for "arm,pl011" at /soc/serial@...
        if let Some(uart_node) = parser.find_compatible("arm,pl011").next() {
            let reg = parser.get_property(&uart_node, "reg").unwrap();
            let (base, _size) = parser.parse_reg(reg)[0];
            let irq_prop = parser.get_property(&uart_node, "interrupts").unwrap();
            let irq = parser.parse_interrupts(irq_prop)[0];

            devmap.uart = Some(UartInfo {
                base: base as usize,
                irq,
                clock: 48_000_000, // 48MHz on RPi5
                compatible: "arm,pl011",
            });
        }

        // GICv3: look for "arm,gic-v3"
        if let Some(gic_node) = parser.find_compatible("arm,gic-v3").next() {
            let reg = parser.get_property(&gic_node, "reg").unwrap();
            let regs = parser.parse_reg(reg);
            devmap.gic = Some(GicInfo {
                dist_base: regs[0].0 as usize,
                redist_base: regs[1].0 as usize,
                version: 3,
            });
        }

        // SDHCI: look for "brcm,bcm2712-sdhci" or "arasan,sdhci-5.1"
        if let Some(sdhci_node) = parser.find_compatible("brcm,bcm2712-sdhci").next() {
            let reg = parser.get_property(&sdhci_node, "reg").unwrap();
            let (base, _size) = parser.parse_reg(reg)[0];
            let irq_prop = parser.get_property(&sdhci_node, "interrupts").unwrap();
            let irq = parser.parse_interrupts(irq_prop)[0];

            devmap.sdhci = Some(SdhciInfo {
                base: base as usize,
                irq,
                quirks: 0, // TODO: parse from device tree or apply known quirks
            });
        }

        self.devices.call_once(|| devmap);
        Ok(())
    }

    fn uart_base(&self) -> usize {
        self.devices.get()
            .and_then(|d| d.uart.as_ref())
            .map(|u| u.base)
            .unwrap_or(0x7d001000) // Default PL011 base on RPi5
    }

    fn gic_dist_base(&self) -> usize {
        self.devices.get()
            .and_then(|d| d.gic.as_ref())
            .map(|g| g.dist_base)
            .unwrap_or(0x107fef0000) // Default GIC DIST on RPi5
    }

    fn gic_redist_base(&self) -> usize {
        self.devices.get()
            .and_then(|d| d.gic.as_ref())
            .map(|g| g.redist_base)
            .unwrap_or(0x107ff00000) // Default GIC REDIST on RPi5
    }

    fn timer_frequency(&self) -> u64 {
        // Read CNTFRQ_EL0 register (set by firmware)
        unsafe {
            let freq: u64;
            core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
            freq
        }
    }
}
```

**main.rs Integration:**
```rust
// In kernel_main() after MMU setup
pub fn kernel_main(fdt_addr: usize) -> ! {
    // ... existing setup ...

    // Get FDT blob (passed in x0 by UEFI or bootloader)
    let fdt_blob: Option<&[u8]> = if fdt_addr != 0 {
        Some(unsafe {
            core::slice::from_raw_parts(fdt_addr as *const u8, 0x10000) // max 64KB
        })
    } else {
        None
    };

    // Detect platform
    let ptype = platform::detect_platform(fdt_blob);
    let platform = platform::get_platform(ptype);

    crate::info!("[PLATFORM] Detected: {}", platform.name());

    // Initialize platform (parses FDT, sets up device map)
    platform.init(fdt_blob).expect("Platform init failed");

    // Initialize UART using platform-provided base
    crate::uart::init(platform.uart_base());

    // Initialize GIC using platform-provided bases
    crate::arch::aarch64::gicv3::init(
        platform.gic_dist_base(),
        platform.gic_redist_base(),
    );

    // Initialize timer
    crate::time::init(platform.timer_frequency());

    // ... rest of init ...
}
```

**UEFI Boot Changes:**
```rust
// crates/uefi-boot/src/main.rs
// Ensure FDT address is preserved and passed to kernel

// In efi_main() after loading kernel:
let fdt_addr = system_table
    .config_table()
    .iter()
    .find(|entry| entry.guid == DEVICE_TREE_GUID)
    .map(|entry| entry.address as usize)
    .unwrap_or(0);

// Jump to kernel with FDT address in x0
unsafe {
    asm!(
        "mov x0, {}",
        "br {}",
        in(reg) fdt_addr,
        in(reg) entry_point,
        options(noreturn)
    );
}
```

**Acceptance Criteria:**
- [ ] QEMU boot prints: `[PLATFORM] Detected: QEMU aarch64 virt`
- [ ] RPi5 boot prints: `[PLATFORM] Detected: Raspberry Pi 5 (BCM2712)`
- [ ] Platform init succeeds with valid device map
- [ ] Fallback to QEMU platform if FDT missing

---

#### M0.3: PL011 UART Driver
**Files:**
- `crates/kernel/src/drivers/char/pl011.rs` (NEW)
- `crates/kernel/src/uart.rs` (MODIFY to support multiple backends)

**Implementation:**
```rust
// drivers/char/pl011.rs
use core::ptr::{read_volatile, write_volatile};

const PL011_DR: usize = 0x00;      // Data register
const PL011_FR: usize = 0x18;      // Flag register
const PL011_IBRD: usize = 0x24;    // Integer baud rate divisor
const PL011_FBRD: usize = 0x28;    // Fractional baud rate divisor
const PL011_LCRH: usize = 0x2C;    // Line control register
const PL011_CR: usize = 0x30;      // Control register
const PL011_IMSC: usize = 0x38;    // Interrupt mask set/clear
const PL011_ICR: usize = 0x44;     // Interrupt clear register

const FR_TXFF: u32 = 1 << 5;       // Transmit FIFO full
const FR_RXFE: u32 = 1 << 4;       // Receive FIFO empty

const CR_UARTEN: u32 = 1 << 0;     // UART enable
const CR_TXE: u32 = 1 << 8;        // Transmit enable
const CR_RXE: u32 = 1 << 9;        // Receive enable

const LCRH_WLEN8: u32 = 0b11 << 5; // 8-bit word length
const LCRH_FEN: u32 = 1 << 4;      // Enable FIFOs

pub struct Pl011Uart {
    base: usize,
}

impl Pl011Uart {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    /// Initialize PL011 at given base address
    /// clock: UART reference clock in Hz (e.g., 48MHz for RPi5)
    /// baud: desired baud rate (e.g., 115200)
    pub unsafe fn init(&self, clock: u32, baud: u32) {
        // Disable UART
        self.write_reg(PL011_CR, 0);

        // Clear all interrupts
        self.write_reg(PL011_ICR, 0x7FF);

        // Set baud rate divisor
        // BAUDDIV = (FUARTCLK / (16 * Baud rate))
        let bauddiv = (clock * 4) / baud; // *4 for fractional part
        let ibrd = bauddiv >> 6;
        let fbrd = bauddiv & 0x3f;

        self.write_reg(PL011_IBRD, ibrd);
        self.write_reg(PL011_FBRD, fbrd);

        // 8N1, enable FIFOs
        self.write_reg(PL011_LCRH, LCRH_WLEN8 | LCRH_FEN);

        // Mask all interrupts (polled mode for now)
        self.write_reg(PL011_IMSC, 0);

        // Enable UART, TX, RX
        self.write_reg(PL011_CR, CR_UARTEN | CR_TXE | CR_RXE);
    }

    /// Write a byte (blocking if FIFO full)
    pub fn putc(&self, c: u8) {
        // Wait while TX FIFO is full
        while (self.read_reg(PL011_FR) & FR_TXFF) != 0 {
            core::hint::spin_loop();
        }
        self.write_reg(PL011_DR, c as u32);
    }

    /// Read a byte (non-blocking, returns None if FIFO empty)
    pub fn getc(&self) -> Option<u8> {
        if (self.read_reg(PL011_FR) & FR_RXFE) != 0 {
            None
        } else {
            Some((self.read_reg(PL011_DR) & 0xFF) as u8)
        }
    }

    #[inline]
    fn read_reg(&self, offset: usize) -> u32 {
        unsafe { read_volatile((self.base + offset) as *const u32) }
    }

    #[inline]
    fn write_reg(&self, offset: usize, value: u32) {
        unsafe { write_volatile((self.base + offset) as *mut u32, value) }
    }
}

// Global instance (initialized by platform)
static mut PL011_INSTANCE: Option<Pl011Uart> = None;

pub fn init_pl011(base: usize, clock: u32, baud: u32) {
    unsafe {
        let uart = Pl011Uart::new(base);
        uart.init(clock, baud);
        PL011_INSTANCE = Some(uart);
    }
}

pub fn pl011_putc(c: u8) {
    unsafe {
        if let Some(uart) = &PL011_INSTANCE {
            uart.putc(c);
        }
    }
}

pub fn pl011_getc() -> Option<u8> {
    unsafe {
        PL011_INSTANCE.as_ref().and_then(|uart| uart.getc())
    }
}
```

```rust
// uart.rs (modify to support multiple backends)
pub enum UartBackend {
    Pl011,
    Ns16550, // existing QEMU UART
}

static UART_BACKEND: AtomicU8 = AtomicU8::new(0); // 0=None, 1=Pl011, 2=Ns16550

pub fn init(base: usize) {
    // Detect UART type from platform
    let platform = crate::platform::get_current_platform();

    match platform.uart_type() {
        UartBackend::Pl011 => {
            crate::drivers::char::pl011::init_pl011(base, 48_000_000, 115200);
            UART_BACKEND.store(1, Ordering::Release);
        }
        UartBackend::Ns16550 => {
            // existing init
            UART_BACKEND.store(2, Ordering::Release);
        }
    }
}

pub fn uart_putc(c: u8) {
    match UART_BACKEND.load(Ordering::Acquire) {
        1 => crate::drivers::char::pl011::pl011_putc(c),
        2 => { /* existing ns16550 putc */ }
        _ => {}
    }
}
```

**Acceptance Criteria:**
- [ ] RPi5 boot prints via PL011 at 0x7d001000 (from FDT)
- [ ] QEMU continues to use existing UART without regression
- [ ] Console echo works: typed characters appear correctly
- [ ] No TX FIFO overruns or dropped characters at high volume

---

#### M0.4: GICv3 Extensions
**Files:**
- `crates/kernel/src/arch/aarch64/gicv3.rs` (EXTEND existing)

**Current State:**
Existing GICv3 code may have basic init for QEMU. Need to verify:
1. Distributor init (GICD_CTLR, enable groups)
2. Redistributor init per-CPU (GICR_WAKER, enable)
3. CPU interface init (ICC_*_EL1 registers)
4. IRQ routing and masking

**Implementation Additions:**
```rust
// gicv3.rs
pub struct GicV3 {
    dist_base: usize,
    redist_base: usize,
}

static mut GIC_INSTANCE: Option<GicV3> = None;

pub fn init(dist_base: usize, redist_base: usize) {
    let gic = GicV3 { dist_base, redist_base };

    // Initialize distributor
    gic.init_distributor();

    // Initialize redistributor for CPU 0
    gic.init_redistributor(0);

    // Initialize CPU interface
    gic.init_cpu_interface();

    unsafe { GIC_INSTANCE = Some(gic); }

    crate::info!("[GIC] GICv3 initialized: DIST={:#x} REDIST={:#x}",
                 dist_base, redist_base);
}

impl GicV3 {
    fn init_distributor(&self) {
        unsafe {
            // Disable distributor
            write_volatile((self.dist_base + GICD_CTLR) as *mut u32, 0);

            // Configure all SPIs as Group 1 (non-secure)
            for i in 1..32 { // 32-1023 SPIs, 32 IRQs per register
                write_volatile(
                    (self.dist_base + GICD_IGROUPR + i * 4) as *mut u32,
                    0xFFFFFFFF
                );
            }

            // Set all SPIs to default priority
            for i in 32..1024 {
                write_volatile(
                    (self.dist_base + GICD_IPRIORITYR + i) as *mut u8,
                    0xA0
                );
            }

            // Enable distributor (Group 1)
            write_volatile(
                (self.dist_base + GICD_CTLR) as *mut u32,
                GICD_CTLR_ENABLE_G1
            );
        }
    }

    fn init_redistributor(&self, cpu: usize) {
        let redist_base = self.redist_base + (cpu * 0x20000);

        unsafe {
            // Wake up redistributor
            let waker = read_volatile((redist_base + GICR_WAKER) as *const u32);
            write_volatile(
                (redist_base + GICR_WAKER) as *mut u32,
                waker & !GICR_WAKER_PROCESSOR_SLEEP
            );

            // Wait for ChildrenAsleep to clear
            while (read_volatile((redist_base + GICR_WAKER) as *const u32)
                   & GICR_WAKER_CHILDREN_ASLEEP) != 0 {
                core::hint::spin_loop();
            }

            // Configure SGIs/PPIs as Group 1
            let sgi_base = redist_base + 0x10000; // SGI_base
            write_volatile((sgi_base + GICR_IGROUPR0) as *mut u32, 0xFFFFFFFF);

            // Set default priorities for SGIs/PPIs
            for i in 0..32 {
                write_volatile(
                    (sgi_base + GICR_IPRIORITYR + i) as *mut u8,
                    0xA0
                );
            }
        }
    }

    fn init_cpu_interface(&self) {
        unsafe {
            // Enable system register access
            core::arch::asm!(
                "msr ICC_SRE_EL1, {sre}",
                "isb",
                sre = in(reg) 0x7u64 // SRE + DIB + DFB
            );

            // Set priority mask (allow all priorities)
            core::arch::asm!("msr ICC_PMR_EL1, {pmr}", pmr = in(reg) 0xF0u64);

            // Set binary point (no priority grouping)
            core::arch::asm!("msr ICC_BPR1_EL1, {bpr}", bpr = in(reg) 0u64);

            // Enable Group 1 interrupts
            core::arch::asm!("msr ICC_IGRPEN1_EL1, {en}", en = in(reg) 1u64);
        }
    }

    pub fn enable_irq(&self, irq: u32) {
        if irq < 32 {
            // SGI/PPI: enable in redistributor
            let sgi_base = self.redist_base + 0x10000;
            unsafe {
                let reg = (sgi_base + GICR_ISENABLER0) as *mut u32;
                write_volatile(reg, 1 << irq);
            }
        } else {
            // SPI: enable in distributor
            let reg_offset = GICD_ISENABLER + ((irq / 32) * 4) as usize;
            unsafe {
                let reg = (self.dist_base + reg_offset) as *mut u32;
                write_volatile(reg, 1 << (irq % 32));
            }
        }
    }

    pub fn disable_irq(&self, irq: u32) {
        if irq < 32 {
            let sgi_base = self.redist_base + 0x10000;
            unsafe {
                let reg = (sgi_base + GICR_ICENABLER0) as *mut u32;
                write_volatile(reg, 1 << irq);
            }
        } else {
            let reg_offset = GICD_ICENABLER + ((irq / 32) * 4) as usize;
            unsafe {
                let reg = (self.dist_base + reg_offset) as *mut u32;
                write_volatile(reg, 1 << (irq % 32));
            }
        }
    }

    pub fn ack_irq(&self) -> u32 {
        let intid: u64;
        unsafe {
            core::arch::asm!("mrs {}, ICC_IAR1_EL1", out(reg) intid);
        }
        intid as u32
    }

    pub fn eoi_irq(&self, irq: u32) {
        unsafe {
            core::arch::asm!("msr ICC_EOIR1_EL1, {}", in(reg) irq as u64);
        }
    }
}

pub fn enable_irq(irq: u32) {
    unsafe {
        if let Some(gic) = &GIC_INSTANCE {
            gic.enable_irq(irq);
        }
    }
}

pub fn disable_irq(irq: u32) {
    unsafe {
        if let Some(gic) = &GIC_INSTANCE {
            gic.disable_irq(irq);
        }
    }
}

pub fn handle_irq() -> u32 {
    unsafe {
        GIC_INSTANCE.as_ref()
            .map(|gic| gic.ack_irq())
            .unwrap_or(1023) // Spurious IRQ
    }
}

pub fn eoi_irq(irq: u32) {
    unsafe {
        if let Some(gic) = &GIC_INSTANCE {
            gic.eoi_irq(irq);
        }
    }
}
```

**IRQ Handler Integration:**
```rust
// arch/aarch64/interrupts.rs
#[no_mangle]
pub extern "C" fn handle_irq_exception() {
    let irq = crate::arch::aarch64::gicv3::handle_irq();

    match irq {
        30 => { // ARM generic timer (PPI 30 = INTID 30)
            crate::time::handle_timer_interrupt();
        }
        _ if irq < 1020 => {
            // Route to device driver
            crate::drivers::handle_device_irq(irq);
        }
        _ => {
            // Spurious or invalid
        }
    }

    crate::arch::aarch64::gicv3::eoi_irq(irq);
}
```

**Acceptance Criteria:**
- [ ] GICv3 init succeeds on both QEMU and RPi5
- [ ] Distributor CTLR register shows enabled state
- [ ] Redistributor WAKER shows processor awake
- [ ] Timer IRQ (PPI 30) fires and is acknowledged
- [ ] No spurious IRQs logged during idle

---

#### M0.5: ARM Generic Timer Configuration
**Files:**
- `crates/kernel/src/time.rs` (MODIFY)
- `crates/kernel/src/arch/aarch64/timer.rs` (NEW or EXTEND)

**Implementation:**
```rust
// arch/aarch64/timer.rs
pub fn init_timer(frequency: u64) {
    unsafe {
        // Disable timer
        core::arch::asm!("msr CNTV_CTL_EL0, {}", in(reg) 0u64);

        // Set compare value for 1Hz tick (1 second intervals)
        let period = frequency;
        core::arch::asm!("msr CNTV_TVAL_EL0, {}", in(reg) period);

        // Enable timer with interrupt
        core::arch::asm!("msr CNTV_CTL_EL0, {}", in(reg) 1u64);

        crate::info!("[TIMER] ARM Generic Timer @ {}Hz, period={}",
                     frequency, period);
    }

    // Enable timer IRQ in GIC (PPI 27 or 30 depending on timer type)
    crate::arch::aarch64::gicv3::enable_irq(30); // Virtual timer = PPI 30
}

pub fn handle_timer_interrupt() {
    // Reload compare value for next tick
    unsafe {
        let freq: u64;
        core::arch::asm!("mrs {}, CNTFRQ_EL0", out(reg) freq);
        core::arch::asm!("msr CNTV_TVAL_EL0, {}", in(reg) freq);
    }

    // Update system time
    crate::time::tick();
}

pub fn read_counter() -> u64 {
    let cnt: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTVCT_EL0", out(reg) cnt);
    }
    cnt
}
```

```rust
// time.rs modifications
static TIMER_FREQUENCY: AtomicU64 = AtomicU64::new(0);
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn init(frequency: u64) {
    TIMER_FREQUENCY.store(frequency, Ordering::Release);
    crate::arch::aarch64::timer::init_timer(frequency);
}

pub fn tick() {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);

    // Call scheduler tick
    crate::process::scheduler::tick();
}

pub fn uptime_seconds() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}

pub fn timestamp_us() -> u64 {
    let cnt = crate::arch::aarch64::timer::read_counter();
    let freq = TIMER_FREQUENCY.load(Ordering::Acquire);
    if freq > 0 {
        (cnt * 1_000_000) / freq
    } else {
        0
    }
}
```

**Acceptance Criteria:**
- [ ] Timer IRQ fires at 1Hz (verified by tick counter)
- [ ] `uptime_seconds()` increments correctly
- [ ] `timestamp_us()` returns monotonic microsecond values
- [ ] No timer overruns or missed ticks under load

---

### M0 Integration & Testing

**Boot Sequence Validation:**
```
[BOOT] SIS Kernel starting...
[PLATFORM] Detected: Raspberry Pi 5 (BCM2712)
[PLATFORM] FDT parsed: UART@0x7d001000 GIC@0x107fef0000 TIMER@54MHz
[UART] PL011 initialized at 0x7d001000 (115200 baud)
[GIC] GICv3 initialized: DIST=0x107fef0000 REDIST=0x107ff00000
[TIMER] ARM Generic Timer @ 54000000Hz, period=54000000
[TICK] Timer IRQ received, tick=1
[TICK] Timer IRQ received, tick=2
...
```

**QEMU Regression Test:**
```bash
# Must still boot successfully with existing UART
SIS_FEATURES="default" ./scripts/uefi_run.sh build
# Should print: [PLATFORM] Detected: QEMU aarch64 virt
```

**Shell Commands:**
```
sis> platform
Platform: Raspberry Pi 5 (BCM2712)
UART: PL011 @ 0x7d001000 (115200 baud)
GIC: GICv3 DIST=0x107fef0000 REDIST=0x107ff00000
Timer: 54MHz (ARM Generic Timer)

sis> uptime
Uptime: 42 seconds (42 ticks)

sis> irqstats
IRQ  Count  Handler
30   42     ARM Timer (Virtual)
```

**Acceptance Criteria for M0:**
- [ ] Boots on QEMU with no regressions
- [ ] Boots on RPi5 hardware via UEFI (with PL011 output)
- [ ] Platform detection works correctly on both
- [ ] Serial console functional and reliable
- [ ] 1Hz timer tick active and stable
- [ ] GICv3 IRQ delivery working
- [ ] Shell accessible and responsive

---

## **M1: Storage (SDHCI + SD Card + ext4)**

### Objectives
1. SDHCI Arasan host controller driver
2. SD/MMC card initialization and protocol
3. Block device abstraction
4. ext4 filesystem mount from SD card
5. File I/O operations (create, read, write, delete)

### Components

#### M1.1: SDHCI Arasan Driver
**Files:**
- `crates/kernel/src/drivers/block/sdhci_arasan.rs` (NEW)
- `crates/kernel/src/drivers/block/sdhci.rs` (NEW - generic SDHCI layer)
- `crates/kernel/src/drivers/block/mod.rs` (MODIFY)

**Implementation:**
```rust
// drivers/block/sdhci.rs (generic SDHCI controller)
pub const SDHCI_DMA_ADDRESS: usize = 0x00;
pub const SDHCI_BLOCK_SIZE: usize = 0x04;
pub const SDHCI_BLOCK_COUNT: usize = 0x06;
pub const SDHCI_ARGUMENT: usize = 0x08;
pub const SDHCI_TRANSFER_MODE: usize = 0x0C;
pub const SDHCI_COMMAND: usize = 0x0E;
pub const SDHCI_RESPONSE: usize = 0x10;
pub const SDHCI_BUFFER: usize = 0x20;
pub const SDHCI_PRESENT_STATE: usize = 0x24;
pub const SDHCI_HOST_CONTROL: usize = 0x28;
pub const SDHCI_POWER_CONTROL: usize = 0x29;
pub const SDHCI_CLOCK_CONTROL: usize = 0x2C;
pub const SDHCI_TIMEOUT_CONTROL: usize = 0x2E;
pub const SDHCI_SOFTWARE_RESET: usize = 0x2F;
pub const SDHCI_INT_STATUS: usize = 0x30;
pub const SDHCI_INT_ENABLE: usize = 0x34;
pub const SDHCI_CAPABILITIES: usize = 0x40;

pub const PRESENT_STATE_CMD_INHIBIT: u32 = 1 << 0;
pub const PRESENT_STATE_DAT_INHIBIT: u32 = 1 << 1;
pub const PRESENT_STATE_CARD_INSERTED: u32 = 1 << 16;

pub const INT_CMD_COMPLETE: u32 = 1 << 0;
pub const INT_TRANSFER_COMPLETE: u32 = 1 << 1;
pub const INT_ERROR: u32 = 1 << 15;

pub struct SdhciHost {
    base: usize,
    quirks: u32,
    version: u32,
    caps: u64,
}

impl SdhciHost {
    pub fn new(base: usize, quirks: u32) -> Self {
        Self {
            base,
            quirks,
            version: 0,
            caps: 0,
        }
    }

    pub fn init(&mut self) -> Result<(), SdhciError> {
        // Read version and capabilities
        self.version = self.read_reg_8(SDHCI_HOST_CONTROL + 1);
        self.caps = self.read_reg_32(SDHCI_CAPABILITIES) as u64;
        self.caps |= (self.read_reg_32(SDHCI_CAPABILITIES + 4) as u64) << 32;

        // Software reset (all)
        self.write_reg_8(SDHCI_SOFTWARE_RESET, 0x07);
        self.wait_reset(0x07)?;

        // Power on
        self.write_reg_8(SDHCI_POWER_CONTROL, 0x0F); // 3.3V
        crate::time::udelay(10000); // 10ms delay

        // Set clock to 400kHz for initialization
        self.set_clock(400_000)?;

        // Enable interrupts
        self.write_reg_32(SDHCI_INT_ENABLE, 0xFFFF_FFFF);

        Ok(())
    }

    fn set_clock(&self, freq: u32) -> Result<(), SdhciError> {
        // Disable clock
        self.write_reg_16(SDHCI_CLOCK_CONTROL, 0);

        // Calculate divisor (base clock from capabilities)
        let base_clock = ((self.caps >> 8) & 0xFF) as u32 * 1_000_000;
        let mut div = 1;
        while base_clock / (div * 2) > freq && div < 256 {
            div *= 2;
        }

        // Set divisor and enable internal clock
        let clk = ((div / 2) << 8) | 0x01;
        self.write_reg_16(SDHCI_CLOCK_CONTROL, clk as u16);

        // Wait for clock stable
        let timeout = 1000;
        for _ in 0..timeout {
            if (self.read_reg_16(SDHCI_CLOCK_CONTROL) & 0x02) != 0 {
                break;
            }
            crate::time::udelay(1000);
        }

        // Enable clock to card
        self.write_reg_16(SDHCI_CLOCK_CONTROL, (clk | 0x04) as u16);
        Ok(())
    }

    pub fn send_command(&self, cmd: u32, arg: u32, flags: u32)
        -> Result<[u32; 4], SdhciError>
    {
        // Wait for command line ready
        let timeout = 1000;
        for _ in 0..timeout {
            let state = self.read_reg_32(SDHCI_PRESENT_STATE);
            if (state & PRESENT_STATE_CMD_INHIBIT) == 0 {
                break;
            }
            crate::time::udelay(1000);
        }

        // Clear interrupts
        self.write_reg_32(SDHCI_INT_STATUS, 0xFFFF_FFFF);

        // Set argument
        self.write_reg_32(SDHCI_ARGUMENT, arg);

        // Send command
        let cmd_reg = ((cmd & 0x3F) << 8) | (flags & 0xFF);
        self.write_reg_16(SDHCI_COMMAND, cmd_reg as u16);

        // Wait for command complete
        for _ in 0..timeout {
            let status = self.read_reg_32(SDHCI_INT_STATUS);
            if (status & INT_CMD_COMPLETE) != 0 {
                self.write_reg_32(SDHCI_INT_STATUS, INT_CMD_COMPLETE);
                break;
            }
            if (status & INT_ERROR) != 0 {
                return Err(SdhciError::CommandFailed);
            }
            crate::time::udelay(1000);
        }

        // Read response
        let mut resp = [0u32; 4];
        if (flags & 0x01) != 0 { // Response present
            resp[0] = self.read_reg_32(SDHCI_RESPONSE);
            if (flags & 0x02) != 0 { // Long response
                resp[1] = self.read_reg_32(SDHCI_RESPONSE + 4);
                resp[2] = self.read_reg_32(SDHCI_RESPONSE + 8);
                resp[3] = self.read_reg_32(SDHCI_RESPONSE + 12);
            }
        }

        Ok(resp)
    }

    pub fn read_block(&self, block: u32, buffer: &mut [u8])
        -> Result<(), SdhciError>
    {
        if buffer.len() != 512 {
            return Err(SdhciError::InvalidBlockSize);
        }

        // Set block size and count
        self.write_reg_16(SDHCI_BLOCK_SIZE, 512);
        self.write_reg_16(SDHCI_BLOCK_COUNT, 1);

        // CMD17 (READ_SINGLE_BLOCK)
        let resp = self.send_command(17, block, 0x3A)?; // Data + R1 response

        // Wait for transfer complete
        let timeout = 10000;
        for _ in 0..timeout {
            let status = self.read_reg_32(SDHCI_INT_STATUS);
            if (status & INT_TRANSFER_COMPLETE) != 0 {
                self.write_reg_32(SDHCI_INT_STATUS, INT_TRANSFER_COMPLETE);
                break;
            }
            if (status & INT_ERROR) != 0 {
                return Err(SdhciError::TransferFailed);
            }
            crate::time::udelay(100);
        }

        // Read data from buffer register (PIO mode)
        for i in (0..512).step_by(4) {
            let word = self.read_reg_32(SDHCI_BUFFER);
            buffer[i..i+4].copy_from_slice(&word.to_le_bytes());
        }

        Ok(())
    }

    pub fn write_block(&self, block: u32, buffer: &[u8])
        -> Result<(), SdhciError>
    {
        if buffer.len() != 512 {
            return Err(SdhciError::InvalidBlockSize);
        }

        // Set block size and count
        self.write_reg_16(SDHCI_BLOCK_SIZE, 512);
        self.write_reg_16(SDHCI_BLOCK_COUNT, 1);

        // CMD24 (WRITE_SINGLE_BLOCK)
        let resp = self.send_command(24, block, 0x3A)?;

        // Write data to buffer register (PIO mode)
        for i in (0..512).step_by(4) {
            let word = u32::from_le_bytes([
                buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]
            ]);
            self.write_reg_32(SDHCI_BUFFER, word);
        }

        // Wait for transfer complete
        let timeout = 10000;
        for _ in 0..timeout {
            let status = self.read_reg_32(SDHCI_INT_STATUS);
            if (status & INT_TRANSFER_COMPLETE) != 0 {
                self.write_reg_32(SDHCI_INT_STATUS, INT_TRANSFER_COMPLETE);
                return Ok(());
            }
            if (status & INT_ERROR) != 0 {
                return Err(SdhciError::TransferFailed);
            }
            crate::time::udelay(100);
        }

        Err(SdhciError::Timeout)
    }

    #[inline]
    fn read_reg_8(&self, offset: usize) -> u8 {
        unsafe { core::ptr::read_volatile((self.base + offset) as *const u8) }
    }

    #[inline]
    fn write_reg_8(&self, offset: usize, value: u8) {
        unsafe { core::ptr::write_volatile((self.base + offset) as *mut u8, value) }
    }

    #[inline]
    fn read_reg_16(&self, offset: usize) -> u16 {
        unsafe { core::ptr::read_volatile((self.base + offset) as *const u16) }
    }

    #[inline]
    fn write_reg_16(&self, offset: usize, value: u16) {
        unsafe { core::ptr::write_volatile((self.base + offset) as *mut u16, value) }
    }

    #[inline]
    fn read_reg_32(&self, offset: usize) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + offset) as *const u32) }
    }

    #[inline]
    fn write_reg_32(&self, offset: usize, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + offset) as *mut u32, value) }
    }

    fn wait_reset(&self, mask: u8) -> Result<(), SdhciError> {
        let timeout = 100;
        for _ in 0..timeout {
            if (self.read_reg_8(SDHCI_SOFTWARE_RESET) & mask) == 0 {
                return Ok(());
            }
            crate::time::udelay(1000);
        }
        Err(SdhciError::ResetTimeout)
    }
}

#[derive(Debug)]
pub enum SdhciError {
    ResetTimeout,
    CommandFailed,
    TransferFailed,
    InvalidBlockSize,
    Timeout,
}
```

#### M1.2: SD Card Initialization
**Files:**
- `crates/kernel/src/drivers/block/sd_card.rs` (NEW)

**Implementation:**
```rust
// drivers/block/sd_card.rs
use super::sdhci::{SdhciHost, SdhciError};

pub struct SdCard {
    host: SdhciHost,
    rca: u16,        // Relative Card Address
    capacity: u64,   // Capacity in blocks
    block_size: u32, // Should be 512
}

impl SdCard {
    pub fn new(base: usize, quirks: u32) -> Self {
        Self {
            host: SdhciHost::new(base, quirks),
            rca: 0,
            capacity: 0,
            block_size: 512,
        }
    }

    pub fn init(&mut self) -> Result<(), SdhciError> {
        // Initialize host controller
        self.host.init()?;

        // CMD0: GO_IDLE_STATE
        self.host.send_command(0, 0, 0)?;
        crate::time::udelay(1000);

        // CMD8: SEND_IF_COND (voltage check for SD 2.0+)
        let resp = self.host.send_command(8, 0x1AA, 0x1A)?; // R7 response

        // ACMD41: SD_SEND_OP_COND (repeatedly until ready)
        let mut ocr = 0u32;
        for _ in 0..1000 {
            // CMD55: APP_CMD (prefix for ACMD)
            self.host.send_command(55, 0, 0x1A)?; // R1

            // ACMD41: voltage range + HCS (high capacity support)
            let resp = self.host.send_command(41, 0x40FF8000, 0x02)?; // R3
            ocr = resp[0];

            if (ocr & 0x80000000) != 0 {
                // Card ready
                break;
            }
            crate::time::udelay(10000);
        }

        if (ocr & 0x80000000) == 0 {
            return Err(SdhciError::Timeout);
        }

        let is_sdhc = (ocr & 0x40000000) != 0;

        // CMD2: ALL_SEND_CID
        self.host.send_command(2, 0, 0x09)?; // R2 (long)

        // CMD3: SEND_RELATIVE_ADDR
        let resp = self.host.send_command(3, 0, 0x1A)?; // R6
        self.rca = (resp[0] >> 16) as u16;

        // CMD9: SEND_CSD (get capacity)
        self.host.send_command(9, (self.rca as u32) << 16, 0x09)?;
        // TODO: parse CSD register to get capacity

        // CMD7: SELECT_CARD
        self.host.send_command(7, (self.rca as u32) << 16, 0x1B)?; // R1b

        // Set block length to 512 (for non-SDHC cards)
        if !is_sdhc {
            self.host.send_command(16, 512, 0x1A)?;
        }

        // ACMD6: SET_BUS_WIDTH (4-bit mode)
        self.host.send_command(55, (self.rca as u32) << 16, 0x1A)?;
        self.host.send_command(6, 2, 0x1A)?; // 2 = 4-bit

        // TODO: Set host controller to 4-bit mode
        // TODO: Increase clock to 25MHz or 50MHz

        crate::info!("[SD] Card initialized: RCA={:#x} SDHC={}",
                     self.rca, is_sdhc);

        Ok(())
    }

    pub fn read_block(&self, block: u32, buffer: &mut [u8])
        -> Result<(), SdhciError>
    {
        self.host.read_block(block, buffer)
    }

    pub fn write_block(&self, block: u32, buffer: &[u8])
        -> Result<(), SdhciError>
    {
        self.host.write_block(block, buffer)
    }
}

// Global SD card instance
static mut SD_CARD: Option<SdCard> = None;

pub fn init_sd_card(base: usize, quirks: u32) -> Result<(), SdhciError> {
    let mut card = SdCard::new(base, quirks);
    card.init()?;

    unsafe {
        SD_CARD = Some(card);
    }

    Ok(())
}

pub fn get_sd_card() -> Option<&'static SdCard> {
    unsafe { SD_CARD.as_ref() }
}
```

#### M1.3: Block Device Integration
**Files:**
- `crates/kernel/src/drivers/block/mod.rs` (MODIFY)
- `crates/kernel/src/vfs/mount.rs` (MODIFY for SD mount)

**Implementation:**
```rust
// drivers/block/mod.rs
pub mod sdhci;
pub mod sd_card;

pub trait BlockDevice: Send + Sync {
    fn read_block(&self, block: u64, buffer: &mut [u8]) -> Result<(), BlockError>;
    fn write_block(&self, block: u64, buffer: &[u8]) -> Result<(), BlockError>;
    fn block_size(&self) -> u32;
    fn block_count(&self) -> u64;
}

#[derive(Debug)]
pub enum BlockError {
    IoError,
    InvalidBlock,
    NotReady,
}

// Wrapper for SD card as block device
pub struct SdBlockDevice;

impl BlockDevice for SdBlockDevice {
    fn read_block(&self, block: u64, buffer: &mut [u8]) -> Result<(), BlockError> {
        let card = sd_card::get_sd_card().ok_or(BlockError::NotReady)?;
        card.read_block(block as u32, buffer)
            .map_err(|_| BlockError::IoError)
    }

    fn write_block(&self, block: u64, buffer: &[u8]) -> Result<(), BlockError> {
        let card = sd_card::get_sd_card().ok_or(BlockError::NotReady)?;
        card.write_block(block as u32, buffer)
            .map_err(|_| BlockError::IoError)
    }

    fn block_size(&self) -> u32 { 512 }

    fn block_count(&self) -> u64 {
        // TODO: return actual capacity from CSD
        0
    }
}
```

#### M1.4: ext4 Mount from SD
**Files:**
- `crates/kernel/src/vfs/ext4.rs` (EXTEND existing)

**Integration:**
```rust
// In main.rs initialization
pub fn init_storage() -> Result<(), &'static str> {
    let platform = platform::get_current_platform();

    if let Some(sdhci_info) = platform.get_sdhci_info() {
        // Initialize SD card
        drivers::block::sd_card::init_sd_card(
            sdhci_info.base,
            sdhci_info.quirks
        ).map_err(|_| "SD card init failed")?;

        // Create block device wrapper
        let sd_blk = Arc::new(drivers::block::SdBlockDevice);

        // Mount ext4 filesystem from SD card
        vfs::mount_ext4("/sd", sd_blk)
            .map_err(|_| "Failed to mount SD ext4")?;

        crate::info!("[STORAGE] SD card mounted at /sd");
        Ok(())
    } else {
        Err("No SDHCI controller found")
    }
}
```

**Acceptance Criteria for M1:**
- [ ] SDHCI controller initializes without errors
- [ ] SD card detected and initialized (CMD sequence completes)
- [ ] Can read partition table from SD card block 0
- [ ] ext4 superblock parsed successfully
- [ ] Can mount ext4 filesystem from SD partition
- [ ] `ls /sd` shows files from SD card
- [ ] Can create new file on SD: `touch /sd/test.txt`
- [ ] Can write to file: `echo "hello" > /sd/test.txt`
- [ ] Can read back: `cat /sd/test.txt` → "hello"
- [ ] File persists across reboot

---

## **M2: PSCI + Power Management**

### Objectives
1. Implement PSCI (Power State Coordination Interface) calls
2. System reset and power off functionality
3. Optional watchdog driver

### Components

#### M2.1: PSCI Implementation
**Files:**
- `crates/kernel/src/arch/aarch64/psci.rs` (NEW or EXTEND)

**Implementation:**
```rust
// arch/aarch64/psci.rs
const PSCI_VERSION: u32 = 0x84000000;
const PSCI_CPU_ON: u32 = 0xC4000003;
const PSCI_SYSTEM_OFF: u32 = 0x84000008;
const PSCI_SYSTEM_RESET: u32 = 0x84000009;

#[derive(Copy, Clone)]
pub enum PsciConduit {
    Hvc,  // Hypervisor Call
    Smc,  // Secure Monitor Call
}

static PSCI_CONDUIT: AtomicU8 = AtomicU8::new(0); // 0=unknown, 1=HVC, 2=SMC

pub fn detect_psci_conduit() -> PsciConduit {
    // Try HVC first (common in UEFI environments)
    let version = psci_call_hvc(PSCI_VERSION, 0, 0, 0);
    if version != 0xFFFFFFFF && version >= 0x00010000 {
        PSCI_CONDUIT.store(1, Ordering::Release);
        crate::info!("[PSCI] Using HVC conduit, version {:#x}", version);
        return PsciConduit::Hvc;
    }

    // Try SMC fallback
    let version = psci_call_smc(PSCI_VERSION, 0, 0, 0);
    if version != 0xFFFFFFFF {
        PSCI_CONDUIT.store(2, Ordering::Release);
        crate::info!("[PSCI] Using SMC conduit, version {:#x}", version);
        return PsciConduit::Smc;
    }

    crate::warn!("[PSCI] No valid conduit found");
    PsciConduit::Hvc // default
}

fn psci_call(func: u32, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    match PSCI_CONDUIT.load(Ordering::Acquire) {
        1 => psci_call_hvc(func, arg0, arg1, arg2),
        2 => psci_call_smc(func, arg0, arg1, arg2),
        _ => 0xFFFFFFFF,
    }
}

fn psci_call_hvc(func: u32, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let result: u64;
    unsafe {
        core::arch::asm!(
            "mov x0, {func}",
            "mov x1, {arg0}",
            "mov x2, {arg1}",
            "mov x3, {arg2}",
            "hvc #0",
            "mov {result}, x0",
            func = in(reg) func as u64,
            arg0 = in(reg) arg0,
            arg1 = in(reg) arg1,
            arg2 = in(reg) arg2,
            result = out(reg) result,
            options(nostack)
        );
    }
    result
}

fn psci_call_smc(func: u32, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let result: u64;
    unsafe {
        core::arch::asm!(
            "mov x0, {func}",
            "mov x1, {arg0}",
            "mov x2, {arg1}",
            "mov x3, {arg2}",
            "smc #0",
            "mov {result}, x0",
            func = in(reg) func as u64,
            arg0 = in(reg) arg0,
            arg1 = in(reg) arg1,
            arg2 = in(reg) arg2,
            result = out(reg) result,
            options(nostack)
        );
    }
    result
}

pub fn system_reset() -> ! {
    crate::info!("[PSCI] System reset requested");
    psci_call(PSCI_SYSTEM_RESET, 0, 0, 0);

    // Fallback: infinite loop
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

pub fn system_off() -> ! {
    crate::info!("[PSCI] System poweroff requested");
    psci_call(PSCI_SYSTEM_OFF, 0, 0, 0);

    // Fallback: infinite loop
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

pub fn cpu_on(cpu_id: u64, entry_point: u64, context: u64) -> Result<(), u32> {
    let result = psci_call(PSCI_CPU_ON, cpu_id, entry_point, context);
    if result == 0 {
        Ok(())
    } else {
        Err(result as u32)
    }
}
```

#### M2.2: Shell Commands
**Files:**
- `crates/kernel/src/shell/mod.rs` (ADD reboot/poweroff commands)

**Implementation:**
```rust
"reboot" => {
    crate::uart_print(b"[SYS] Rebooting...\n");
    crate::arch::aarch64::psci::system_reset();
}

"poweroff" => {
    crate::uart_print(b"[SYS] Powering off...\n");
    crate::arch::aarch64::psci::system_off();
}
```

**Acceptance Criteria for M2:**
- [ ] PSCI conduit detected (HVC or SMC)
- [ ] `reboot` shell command resets the board
- [ ] `poweroff` shell command powers off (or halts in QEMU)
- [ ] No kernel panics during reset/poweroff

---

## **M3: SMP (Multi-Core Support)**

### Objectives
1. Bring up secondary CPUs using PSCI
2. Per-CPU GIC redistributor initialization
3. Per-CPU timer initialization
4. Scheduler load balancing across cores

### Components

#### M3.1: SMP Bringup
**Files:**
- `crates/kernel/src/arch/aarch64/smp.rs` (NEW or EXTEND)
- `crates/kernel/src/process/scheduler_smp.rs` (EXTEND)

**Implementation:**
```rust
// arch/aarch64/smp.rs
use crate::arch::aarch64::psci;
use core::sync::atomic::{AtomicU32, Ordering};

static NUM_CPUS: AtomicU32 = AtomicU32::new(1);
static SECONDARY_BOOT_FLAG: [AtomicU32; 4] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

pub fn init_smp(max_cpus: u32) {
    let num_cpus = max_cpus.min(4); // RPi5 has 4 cores

    for cpu in 1..num_cpus {
        crate::info!("[SMP] Bringing up CPU {}", cpu);

        let entry = secondary_entry as usize as u64;
        let context = cpu as u64;

        match psci::cpu_on(cpu as u64, entry, context) {
            Ok(()) => {
                // Wait for secondary CPU to signal ready
                let timeout = 1000;
                for _ in 0..timeout {
                    if SECONDARY_BOOT_FLAG[cpu as usize].load(Ordering::Acquire) == 1 {
                        NUM_CPUS.fetch_add(1, Ordering::Release);
                        crate::info!("[SMP] CPU {} online", cpu);
                        break;
                    }
                    crate::time::udelay(1000);
                }
            }
            Err(e) => {
                crate::warn!("[SMP] Failed to start CPU {}: error {:#x}", cpu, e);
            }
        }
    }

    crate::info!("[SMP] {} CPUs online", NUM_CPUS.load(Ordering::Acquire));
}

#[no_mangle]
extern "C" fn secondary_entry(cpu_id: u64) -> ! {
    let cpu = cpu_id as usize;

    // Initialize per-CPU GIC redistributor
    crate::arch::aarch64::gicv3::init_cpu(cpu);

    // Initialize per-CPU timer
    crate::arch::aarch64::timer::init_cpu();

    // Enable interrupts
    unsafe {
        core::arch::asm!(
            "msr DAIFClr, #2", // Clear IRQ mask
            options(nomem, nostack)
        );
    }

    // Signal that this CPU is ready
    SECONDARY_BOOT_FLAG[cpu].store(1, Ordering::Release);

    // Enter scheduler idle loop
    crate::process::scheduler::idle_loop(cpu);
}

pub fn num_cpus() -> u32 {
    NUM_CPUS.load(Ordering::Acquire)
}

pub fn current_cpu() -> u32 {
    // Read MPIDR_EL1 to get current CPU ID
    let mpidr: u64;
    unsafe {
        core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) mpidr);
    }
    (mpidr & 0xFF) as u32
}
```

#### M3.2: Per-CPU Data Structures
**Files:**
- `crates/kernel/src/process/scheduler_smp.rs` (EXTEND)

**Implementation:**
```rust
// process/scheduler_smp.rs
use alloc::collections::VecDeque;

pub struct PerCpuData {
    pub cpu_id: u32,
    pub run_queue: VecDeque<Arc<Process>>,
    pub idle_time: u64,
    pub busy_time: u64,
}

static PER_CPU_DATA: [spin::Mutex<Option<PerCpuData>>; 4] = [
    spin::Mutex::new(None),
    spin::Mutex::new(None),
    spin::Mutex::new(None),
    spin::Mutex::new(None),
];

pub fn init_cpu(cpu_id: u32) {
    let data = PerCpuData {
        cpu_id,
        run_queue: VecDeque::new(),
        idle_time: 0,
        busy_time: 0,
    };

    *PER_CPU_DATA[cpu_id as usize].lock() = Some(data);
}

pub fn idle_loop(cpu_id: u32) -> ! {
    loop {
        // Try to get work from run queue
        let process = {
            let mut data = PER_CPU_DATA[cpu_id as usize].lock();
            data.as_mut().and_then(|d| d.run_queue.pop_front())
        };

        if let Some(proc) = process {
            // Run process
            crate::process::run_process(proc);
        } else {
            // No work: WFI (wait for interrupt)
            unsafe {
                core::arch::asm!("wfi", options(nomem, nostack));
            }
        }
    }
}
```

**Acceptance Criteria for M3:**
- [ ] Secondary CPUs boot successfully
- [ ] `smp` shell command shows all CPUs online
- [ ] Per-CPU timer ticks work independently
- [ ] Processes scheduled across multiple cores
- [ ] No deadlocks or race conditions under SMP stress

---

## **M4: Networking (Optional)**

### Objectives
1. Choose one network path: USB CDC ECM or PCIe NIC
2. Integrate with smoltcp stack
3. DHCP client with fallback IP
4. Basic connectivity test (UDP ping)

### Path Options

**Option A: USB CDC ECM (Recommended First)**
- **Pros:** Simple protocol, common on USB NICs, no PCIe complexity
- **Cons:** Requires XHCI driver first
- **Files:** `drivers/usb/xhci/mod.rs`, `drivers/net/cdc_ecm.rs`

**Option B: PCIe NIC (e.g., RTL8168)**
- **Pros:** Faster, more standard for embedded boards
- **Cons:** Requires PCIe host controller (Synopsys DWC)
- **Files:** `drivers/pci/dwc.rs`, `drivers/net/rtl8168.rs`

**Deferred to M6:** Implement after core functionality stable

---

## **M5: PMU & Performance Monitoring**

### Objectives
1. ARM PMU (Performance Monitoring Unit) initialization
2. Cycle counter and event counter configuration
3. `pmu` shell command to display stats

### Components

#### M5.1: PMU Driver
**Files:**
- `crates/kernel/src/pmu.rs` (EXTEND existing)

**Implementation:**
```rust
// pmu.rs
pub fn init_pmu() {
    unsafe {
        // Enable user-mode access to PMU (optional)
        core::arch::asm!("msr PMUSERENR_EL0, {}", in(reg) 0x0Fu64);

        // Reset all counters
        core::arch::asm!("msr PMCR_EL0, {}", in(reg) 0x07u64); // E + P + C bits

        // Enable cycle counter
        core::arch::asm!("msr PMCNTENSET_EL0, {}", in(reg) 0x80000000u64);

        crate::info!("[PMU] Performance monitoring enabled");
    }
}

pub fn read_cycle_counter() -> u64 {
    let cnt: u64;
    unsafe {
        core::arch::asm!("mrs {}, PMCCNTR_EL0", out(reg) cnt);
    }
    cnt
}

pub fn read_event_counter(idx: u32) -> u64 {
    let cnt: u64;
    unsafe {
        core::arch::asm!(
            "msr PMSELR_EL0, {}",
            "isb",
            "mrs {}, PMXEVCNTR_EL0",
            in(reg) idx as u64,
            out(reg) cnt
        );
    }
    cnt
}
```

**Shell Command:**
```rust
"pmu" => {
    let cycles = crate::pmu::read_cycle_counter();
    crate::uart_print(b"[PMU] Cycle counter: ");
    self.print_number_simple(cycles);
    crate::uart_print(b"\n");
}
```

**Acceptance Criteria for M5:**
- [ ] PMU initialized without errors
- [ ] Cycle counter increments during workload
- [ ] `pmu` command shows non-zero values
- [ ] Event counters configurable (optional)

---

## **M6: Optional I/O (GPIO, Mailbox, Watchdog)**

### Components

#### M6.1: GPIO Driver
**Files:**
- `crates/kernel/src/drivers/gpio/bcm2xxx.rs` (NEW)

**Implementation Sketch:**
```rust
// Simple GPIO driver for LED blink
pub struct BcmGpio {
    base: usize,
}

impl BcmGpio {
    pub fn set_function(&self, pin: u32, func: u32) {
        // Set GPIO function (input, output, alt0-5)
    }

    pub fn set_pin(&self, pin: u32) {
        // Set pin high
    }

    pub fn clear_pin(&self, pin: u32) {
        // Set pin low
    }
}

// LED blink task
pub fn led_heartbeat() {
    let gpio = BcmGpio::new(GPIO_BASE);
    gpio.set_function(ACT_LED_PIN, GPIO_OUTPUT);

    loop {
        gpio.set_pin(ACT_LED_PIN);
        crate::time::msleep(500);
        gpio.clear_pin(ACT_LED_PIN);
        crate::time::msleep(500);
    }
}
```

#### M6.2: Mailbox Properties
**Files:**
- `crates/kernel/src/drivers/firmware/mailbox.rs` (NEW)

**Implementation Sketch:**
```rust
// VC mailbox for firmware properties
pub fn get_temperature() -> Result<u32, MailboxError> {
    let mut msg = [0u32; 8];
    msg[0] = 8 * 4; // size
    msg[1] = 0; // request
    msg[2] = 0x00030006; // get temp tag
    msg[3] = 8; // value size
    msg[4] = 0; // request
    msg[5] = 0; // temperature ID
    msg[6] = 0; // response
    msg[7] = 0; // end tag

    mailbox_call(&mut msg)?;

    Ok(msg[6]) // temperature in millidegrees
}
```

**Acceptance Criteria for M6:**
- [ ] GPIO pin control works (LED blinks at 1Hz)
- [ ] Mailbox returns valid temperature reading
- [ ] Optional: watchdog barks on test timeout

---

## **M7: Full Hardware Validation**

### Test Suite
1. **Boot Test:** Power on → UEFI → Kernel → Shell prompt
2. **Console Test:** Type commands, see responses
3. **Timer Test:** `uptime` increments correctly
4. **IRQ Test:** Timer ticks logged, no spurious IRQs
5. **Storage Test:** Mount SD, create file, reboot, verify persistence
6. **SMP Test:** All cores online, load distributed
7. **Reset Test:** `reboot` command cleanly resets board
8. **Stress Test:** Idle for 10 minutes, no panics or hangs

### Debug Checklist
- [ ] Serial output clear and stable
- [ ] No MMU faults or data aborts
- [ ] GIC IRQ delivery reliable
- [ ] SD card I/O consistent (no corruption)
- [ ] SMP synchronization correct (no race conditions)
- [ ] Power management PSCI calls succeed

---

## **M8: Driver Hardening**

### Tasks
1. **Error Handling:** All I/O operations return proper errors
2. **Timeouts:** All hardware waits have timeout limits
3. **IRQ Affinity:** Route device IRQs to specific CPUs
4. **DMA Safety:** Ensure all DMA buffers aligned and coherent
5. **Logging:** Disable verbose debug logs in release builds
6. **Testing:** Driver self-tests pass in both QEMU and hardware

**Example Hardening:**
```rust
// Add timeout to all SDHCI waits
const SDHCI_TIMEOUT_MS: u64 = 1000;

fn wait_for_cmd_ready(&self) -> Result<(), SdhciError> {
    let start = crate::time::timestamp_us();
    loop {
        if (self.read_reg_32(SDHCI_PRESENT_STATE) & PRESENT_STATE_CMD_INHIBIT) == 0 {
            return Ok(());
        }
        if (crate::time::timestamp_us() - start) > (SDHCI_TIMEOUT_MS * 1000) {
            return Err(SdhciError::Timeout);
        }
        crate::time::udelay(10);
    }
}
```

---

## QEMU Validation Strategy

### Driver Self-Tests

All drivers implement self-test mode that runs without real hardware:

```rust
// Example: SDHCI self-test with mock registers
#[cfg(test)]
pub fn sdhci_selftest() {
    let mock_base = alloc_mock_mmio_region(0x1000);

    // Setup mock register values
    mock_write_reg(mock_base + SDHCI_CAPABILITIES, 0x01E00000);
    mock_write_reg(mock_base + SDHCI_PRESENT_STATE, PRESENT_STATE_CARD_INSERTED);

    // Initialize driver with mock base
    let mut host = SdhciHost::new(mock_base, 0);
    assert!(host.init().is_ok());

    // Verify register writes
    assert_eq!(mock_read_reg(mock_base + SDHCI_POWER_CONTROL), 0x0F);

    crate::info!("[TEST] SDHCI self-test PASSED");
}
```

**Shell Commands for Self-Tests:**
```
sis> selftest sdhci
[TEST] SDHCI self-test PASSED

sis> selftest all
[TEST] PL011 self-test PASSED
[TEST] GICv3 self-test PASSED
[TEST] SDHCI self-test PASSED
[TEST] All driver self-tests PASSED (4/4)
```

### QEMU Build Flag
```rust
#[cfg(feature = "qemu-virt")]
const ENABLE_SELFTEST: bool = true;

#[cfg(not(feature = "qemu-virt"))]
const ENABLE_SELFTEST: bool = false;
```

**Build Commands:**
```bash
# QEMU with self-tests
SIS_FEATURES="qemu-virt,selftest" ./scripts/uefi_run.sh build

# RPi5 production (no self-tests)
SIS_FEATURES="rpi5,default" cargo build --target aarch64-unknown-none --release
```

---

## Risk Mitigation

### Risk: SDHCI Tuning Failures
**Mitigation:**
- Start with conservative 25MHz clock, 1-bit mode
- Implement auto-tuning retry with fallback
- Log detailed tuning errors for debug

### Risk: RP1 I/O Hub Quirks
**Mitigation:**
- Use FDT to detect RP1 vs SoC devices
- Implement quirk flags per device
- Test with known-good SD card first

### Risk: DMA Coherency Issues
**Mitigation:**
- Use PIO mode initially (slower but reliable)
- Implement bounce buffers for DMA
- Add cache flush/invalidate helpers

### Risk: Device Tree Variations
**Mitigation:**
- Don't hardcode any MMIO addresses
- Provide sensible defaults if FDT parsing fails
- Log all FDT-derived values for debug

### Risk: SMP Race Conditions
**Mitigation:**
- Use proper spinlocks for all shared data
- Implement per-CPU data structures
- Add lockdep-style debugging (optional)

---

## File Checklist

### New Files (Estimated ~2500 LOC)
```
crates/kernel/src/
├── platform/
│   ├── fdt.rs                      (~400 LOC)
│   ├── rpi5.rs                     (~200 LOC)
│   └── mod.rs (modify)             (+50 LOC)
├── drivers/
│   ├── char/
│   │   └── pl011.rs                (~150 LOC)
│   ├── block/
│   │   ├── sdhci.rs                (~500 LOC)
│   │   ├── sd_card.rs              (~300 LOC)
│   │   └── mod.rs (modify)         (+100 LOC)
│   ├── net/ (optional M4)
│   │   ├── cdc_ecm.rs              (~400 LOC)
│   │   └── rtl8168.rs              (~600 LOC)
│   ├── usb/ (optional M4)
│   │   └── xhci/...                (~800 LOC)
│   ├── pci/ (optional M4)
│   │   └── dwc.rs                  (~500 LOC)
│   ├── gpio/
│   │   └── bcm2xxx.rs              (~150 LOC)
│   └── firmware/
│       └── mailbox.rs              (~200 LOC)
├── arch/aarch64/
│   ├── gicv3.rs (extend)           (+300 LOC)
│   ├── timer.rs (extend)           (+100 LOC)
│   ├── psci.rs                     (~150 LOC)
│   └── smp.rs                      (~200 LOC)
├── vfs/
│   └── ext4.rs (extend)            (+50 LOC)
├── process/
│   └── scheduler_smp.rs (extend)   (+200 LOC)
├── pmu.rs (extend)                 (+50 LOC)
├── time.rs (extend)                (+50 LOC)
└── main.rs (modify)                (+100 LOC)

crates/uefi-boot/src/
└── main.rs (modify)                (+50 LOC)

docs/
└── IMPLEMENTATION_PLAN_RPI5_HARDWARE.md (this file)
```

### Modified Files
- `crates/kernel/src/main.rs`: Platform detection, init sequence
- `crates/kernel/src/uart.rs`: Multi-backend support
- `crates/kernel/src/arch/aarch64/gicv3.rs`: Extended init
- `crates/kernel/src/time.rs`: Timer integration
- `crates/kernel/src/shell/mod.rs`: New commands (reboot, pmu, selftest)
- `crates/uefi-boot/src/main.rs`: FDT address passing

---

## Milestone Dependencies

```
M0 (Foundation)
  ├── FDT parser must complete first
  ├── Platform detection depends on FDT
  ├── UART/GIC/Timer init depends on platform
  └── PREREQUISITE for all other milestones

M1 (Storage)
  ├── Requires M0 (platform init)
  ├── SDHCI driver → SD card init → Block device → ext4 mount
  └── CRITICAL PATH for persistent storage

M2 (PSCI/Power)
  ├── Requires M0 (platform init)
  ├── Independent of M1 (can be parallel)
  └── PREREQUISITE for M3 (SMP needs PSCI CPU_ON)

M3 (SMP)
  ├── Requires M0 (GIC/Timer)
  ├── Requires M2 (PSCI CPU_ON)
  └── Can be done after M1 or in parallel

M4 (Networking - OPTIONAL)
  ├── Requires M0 (interrupts)
  ├── Independent of M1/M2/M3
  └── DEFERRED until M0-M3 stable

M5 (PMU)
  ├── Requires M0 (platform init)
  ├── Independent of all others
  └── LOW PRIORITY (debug/profiling only)

M6 (GPIO/Mailbox - OPTIONAL)
  ├── Requires M0 (platform init)
  ├── Independent of all others
  └── LOW PRIORITY (nice-to-have)

M7 (Validation)
  ├── Requires M0-M3 complete
  ├── Optional: M4-M6 if implemented
  └── FINAL GATE before M8

M8 (Hardening)
  ├── Requires M7 (validation results)
  └── PRODUCTION READINESS
```

**Critical Path:** M0 → M1 → M7 → M8
**Parallel Path:** M0 → M2 → M3 → M7 → M8
**Optional:** M4, M5, M6 (add after M0-M3 stable)

---

## Development Workflow

### Phase 1: Mac Mini Development (No Hardware)
1. Implement FDT parser with test data
2. Add RPi5 platform descriptor (with FDT defaults)
3. Implement PL011 driver (test with QEMU if PL011 available)
4. Extend GICv3 for RPi5 address space
5. Add SDHCI driver with self-test mode
6. Implement PSCI calls (test in QEMU with -machine virt,secure=on)
7. Write driver self-tests for all components
8. Verify QEMU boot with no regressions

**Validation:**
```bash
# Build with all new drivers
SIS_FEATURES="rpi5,selftest" cargo build --target aarch64-unknown-none

# Run self-tests in QEMU
./scripts/uefi_run.sh run
sis> selftest all
[TEST] All driver self-tests PASSED (8/8)
```

### Phase 2: Initial Hardware Bring-up
1. Flash UEFI firmware to RPi5
2. Copy kernel ELF to SD card EFI partition
3. Boot and capture serial output
4. Verify platform detection
5. Test UART echo
6. Verify timer ticks
7. Test SD card detection (read block 0)

**First Boot Checklist:**
- [ ] Serial output appears on PL011
- [ ] Platform banner shows "Raspberry Pi 5"
- [ ] FDT parsed successfully
- [ ] GIC initialized without hangs
- [ ] Timer IRQ fires

### Phase 3: Storage & Filesystem
1. Mount SD card ext4 partition
2. Test file creation
3. Reboot and verify persistence
4. Stress test: create 1000 files

### Phase 4: SMP & Stability
1. Bring up secondary cores
2. Run multi-threaded workload
3. Idle stability test (10 minutes)
4. Stress test under load

### Phase 5: Optional Features
1. Choose networking path (USB vs PCIe)
2. Implement driver
3. Test DHCP and connectivity
4. Add PMU stats
5. Add GPIO LED heartbeat

### Phase 6: Hardening
1. Add all timeouts
2. Improve error messages
3. Disable debug logs
4. Final validation suite
5. Production build

---

## Testing Matrix

| Feature | QEMU Self-Test | QEMU Boot | RPi5 Boot | RPi5 Stress |
|---------|----------------|-----------|-----------|-------------|
| FDT Parser | ✓ (mock data) | ✓ | ✓ | - |
| Platform Detection | ✓ | ✓ | ✓ | - |
| PL011 UART | ✓ (mock MMIO) | N/A (no PL011 in virt) | ✓ | ✓ |
| GICv3 | ✓ (mock) | ✓ | ✓ | ✓ |
| ARM Timer | ✓ (mock) | ✓ | ✓ | ✓ |
| SDHCI Driver | ✓ (mock regs) | N/A | ✓ | ✓ |
| SD Card Init | ✓ (mock) | N/A | ✓ | ✓ |
| ext4 Mount | N/A | ✓ (virtio-blk) | ✓ | ✓ |
| PSCI Reset | ✓ (mock) | ✓ | ✓ | - |
| SMP | ✓ (mock) | ✓ | ✓ | ✓ |
| PMU | ✓ (mock) | ✓ | ✓ | - |

---

## Success Criteria (Final Gate)

### Minimum Viable Product (MVP)
- [ ] Boots on RPi5 via UEFI (serial output via PL011)
- [ ] Platform detection works (FDT parsed)
- [ ] Shell prompt accessible
- [ ] Timer ticks at 1Hz
- [ ] SD card detected and mounted (ext4)
- [ ] Can create/read/write files on SD
- [ ] `reboot` command works
- [ ] Stable for 10 minutes idle

### Stretch Goals
- [ ] All 4 CPU cores online (SMP)
- [ ] Network connectivity (DHCP + ping)
- [ ] PMU stats available
- [ ] GPIO LED heartbeat
- [ ] Watchdog functional

### Production Readiness
- [ ] All timeouts implemented
- [ ] Error handling comprehensive
- [ ] No verbose debug logs in release
- [ ] Driver self-tests pass
- [ ] No known panics or hangs
- [ ] Documentation complete

---

## Timeline Estimates

**Assuming 1 developer, full-time:**

| Milestone | Duration | Dependencies |
|-----------|----------|--------------|
| M0 (Foundation) | 5 days | None |
| M1 (Storage) | 7 days | M0 |
| M2 (PSCI/Power) | 2 days | M0 |
| M3 (SMP) | 4 days | M0, M2 |
| M4 (Networking) | 10 days | M0 (optional) |
| M5 (PMU) | 1 day | M0 (optional) |
| M6 (GPIO/etc) | 2 days | M0 (optional) |
| M7 (Validation) | 3 days | M0-M3 |
| M8 (Hardening) | 3 days | M7 |

**Total (Critical Path):** M0→M1→M7→M8 = 18 days
**Total (with SMP):** M0→M2→M3→M7→M8 = 17 days
**Total (with all optional):** M0→M1→M2→M3→M4→M5→M6→M7→M8 = 37 days

**Recommended Phasing:**
- **Week 1-2:** M0 + M1 (Foundation + Storage) → First file I/O on SD
- **Week 3:** M2 + M3 (PSCI + SMP) → Multi-core operational
- **Week 4:** M7 + M8 (Validation + Hardening) → Production ready
- **Week 5+ (optional):** M4/M5/M6 (Networking, PMU, GPIO)

---

## Next Steps

1. **Review this plan** with stakeholders
2. **Create feature branch:** `feat/rpi5-hardware-support`
3. **Start with M0.1:** Implement FDT parser
4. **Validate incrementally:** Each component passes self-test before moving on
5. **QEMU validation:** Ensure no regressions in existing QEMU boot
6. **Hardware validation:** Test on real RPi5 at M0 completion
7. **Iterate:** M1→M2→M3→M7→M8

**First Commit Target:** FDT parser + platform detection (M0.1 + M0.2)
**First Hardware Test:** After M0 complete (PL011 output + timer ticks)
**First Production Milestone:** After M0 + M1 + M7 (bootable with persistent storage)

---

## Appendix: Hardware Reference

### Raspberry Pi 5 Specifications
- **SoC:** Broadcom BCM2712 (16nm)
- **CPU:** 4× Cortex-A76 @ 2.4GHz
- **RAM:** 4GB or 8GB LPDDR4X-4267
- **GIC:** GICv3 (GICD + GICR)
- **Timer:** ARM Generic Timer @ 54MHz
- **UART:** PL011 @ 0x7d001000 (default, verify via FDT)
- **SDHCI:** Arasan SDHCI 5.1 controller
- **PCIe:** RP1 I/O hub (PCIe Gen 2 ×4)
- **USB:** RP1 XHCI (USB 3.0)
- **Ethernet:** RP1 Ethernet MAC (1Gbps)

### Memory Map (Indicative, always verify via FDT)
```
0x0000_0000 - 0x3FFF_FFFF : DRAM (1GB low)
0x4000_0000 - 0x7FFF_FFFF : Reserved
0x7d00_0000 - 0x7dFF_FFFF : VC peripherals (UART, etc)
0x107f_0000 - 0x107F_FFFF : GIC Distributor
0x107F_0000 - 0x108F_FFFF : GIC Redistributors
0x1000_0000 - 0x1FFF_FFFF : RP1 peripherals (PCIe, USB, Ethernet)
```

### Boot Firmware
- **Recommended:** Raspberry Pi UEFI firmware (EDK2)
- **Download:** https://github.com/pftf/RPi4/releases (use RPi5 variant)
- **Installation:** Copy to SD FAT32 EFI partition
  - `RPI_EFI.fd` → SD card root
  - Kernel ELF → `/EFI/BOOT/BOOTAA64.EFI` or via GRUB

### Serial Console
- **GPIO Pins:** GPIO14 (TXD), GPIO15 (RXD)
- **Baud Rate:** 115200
- **Connection:** USB-to-serial adapter (3.3V TTL)

---

**End of Implementation Plan**
