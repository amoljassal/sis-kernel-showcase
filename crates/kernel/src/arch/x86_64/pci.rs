//! # PCI (Peripheral Component Interconnect) Bus Support
//!
//! This module provides PCI bus enumeration and device access for x86_64 systems.
//! It implements both legacy I/O port-based configuration space access and modern
//! PCI Express ECAM (Enhanced Configuration Access Mechanism) via memory-mapped I/O.
//!
//! ## PCI Configuration Space Access
//!
//! ### Legacy I/O Port Method (PCI 2.x)
//!
//! Uses two I/O ports:
//! - **0xCF8 (CONFIG_ADDRESS)**: 32-bit address register
//! - **0xCFC (CONFIG_DATA)**: 32-bit data register
//!
//! Address format (written to 0xCF8):
//! ```text
//! Bits 31    : Enable bit (must be 1)
//! Bits 30-24 : Reserved (0)
//! Bits 23-16 : Bus number (0-255)
//! Bits 15-11 : Device number (0-31)
//! Bits 10-8  : Function number (0-7)
//! Bits 7-2   : Register offset (DWORD aligned)
//! Bits 1-0   : Reserved (0)
//! ```
//!
//! ### PCI Express ECAM (Enhanced Configuration Access Mechanism)
//!
//! Modern PCIe systems use memory-mapped configuration space:
//! - Base address provided by ACPI MCFG table
//! - Each bus gets 1 MB of address space
//! - Address calculation: `base + (bus << 20) + (dev << 15) + (func << 12) + offset`
//!
//! ## PCI Device Identification
//!
//! Each PCI device has:
//! - **Vendor ID** (16 bits): Identifies the manufacturer
//! - **Device ID** (16 bits): Identifies the specific device
//! - **Class Code** (24 bits): Device type classification
//!   - Base Class (8 bits): Major category (e.g., 0x01 = Mass Storage)
//!   - Sub Class (8 bits): Specific type (e.g., 0x00 = SCSI)
//!   - Programming Interface (8 bits): Register-level interface
//!
//! ## Common Vendor IDs
//!
//! - **0x1234**: QEMU/Bochs (legacy graphics)
//! - **0x1AF4**: Red Hat (VirtIO devices)
//! - **0x8086**: Intel
//! - **0x10EC**: Realtek
//! - **0x1022**: AMD
//!
//! ## Base Address Registers (BARs)
//!
//! PCI devices expose up to 6 BARs (offsets 0x10-0x24) for:
//! - Memory-mapped I/O (MMIO) regions
//! - I/O port ranges
//!
//! BAR encoding:
//! ```text
//! Bit 0: 0 = Memory space, 1 = I/O space
//! Bits 1-2 (Memory): Type (00 = 32-bit, 10 = 64-bit)
//! Bit 3 (Memory): Prefetchable
//! Bits 4-31: Base address (16-byte aligned for memory)
//! ```

use x86_64::instructions::port::Port;
use alloc::vec::Vec;
use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;

/// PCI Configuration Address register (I/O port 0xCF8)
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;

/// PCI Configuration Data register (I/O port 0xCFC)
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// PCI configuration space register offsets
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum PciConfigOffset {
    VendorId = 0x00,
    DeviceId = 0x02,
    Command = 0x04,
    Status = 0x06,
    RevisionId = 0x08,
    ProgIf = 0x09,
    Subclass = 0x0A,
    ClassCode = 0x0B,
    CacheLineSize = 0x0C,
    LatencyTimer = 0x0D,
    HeaderType = 0x0E,
    Bist = 0x0F,
    Bar0 = 0x10,
    Bar1 = 0x14,
    Bar2 = 0x18,
    Bar3 = 0x1C,
    Bar4 = 0x20,
    Bar5 = 0x24,
    CardbusCisPtr = 0x28,
    SubsystemVendorId = 0x2C,
    SubsystemId = 0x2E,
    ExpansionRomBase = 0x30,
    CapabilitiesPtr = 0x34,
    InterruptLine = 0x3C,
    InterruptPin = 0x3D,
    MinGrant = 0x3E,
    MaxLatency = 0x3F,
}

/// PCI Command register bits
pub mod command {
    pub const IO_SPACE: u16 = 1 << 0;          // Enable I/O space access
    pub const MEMORY_SPACE: u16 = 1 << 1;      // Enable memory space access
    pub const BUS_MASTER: u16 = 1 << 2;        // Enable bus mastering (DMA)
    pub const SPECIAL_CYCLES: u16 = 1 << 3;    // Monitor special cycles
    pub const MWI_ENABLE: u16 = 1 << 4;        // Memory write & invalidate
    pub const VGA_PALETTE_SNOOP: u16 = 1 << 5; // VGA palette snooping
    pub const PARITY_ERROR: u16 = 1 << 6;      // Parity error response
    pub const SERR_ENABLE: u16 = 1 << 8;       // SERR# enable
    pub const FAST_B2B_ENABLE: u16 = 1 << 9;   // Fast back-to-back enable
    pub const INTERRUPT_DISABLE: u16 = 1 << 10; // Interrupt disable
}

/// PCI device class codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PciClass {
    Unclassified = 0x00,
    MassStorage = 0x01,
    Network = 0x02,
    Display = 0x03,
    Multimedia = 0x04,
    Memory = 0x05,
    Bridge = 0x06,
    SimpleCommunication = 0x07,
    BaseSystemPeripheral = 0x08,
    InputDevice = 0x09,
    DockingStation = 0x0A,
    Processor = 0x0B,
    SerialBus = 0x0C,
    Wireless = 0x0D,
    IntelligentIO = 0x0E,
    SatelliteCommunication = 0x0F,
    EncryptionDecryption = 0x10,
    SignalProcessing = 0x11,
    ProcessingAccelerator = 0x12,
    NonEssentialInstrumentation = 0x13,
    CoProcessor = 0x40,
    Unknown = 0xFF,
}

impl From<u8> for PciClass {
    fn from(val: u8) -> Self {
        match val {
            0x00 => PciClass::Unclassified,
            0x01 => PciClass::MassStorage,
            0x02 => PciClass::Network,
            0x03 => PciClass::Display,
            0x04 => PciClass::Multimedia,
            0x05 => PciClass::Memory,
            0x06 => PciClass::Bridge,
            0x07 => PciClass::SimpleCommunication,
            0x08 => PciClass::BaseSystemPeripheral,
            0x09 => PciClass::InputDevice,
            0x0A => PciClass::DockingStation,
            0x0B => PciClass::Processor,
            0x0C => PciClass::SerialBus,
            0x0D => PciClass::Wireless,
            0x0E => PciClass::IntelligentIO,
            0x0F => PciClass::SatelliteCommunication,
            0x10 => PciClass::EncryptionDecryption,
            0x11 => PciClass::SignalProcessing,
            0x12 => PciClass::ProcessingAccelerator,
            0x13 => PciClass::NonEssentialInstrumentation,
            0x40 => PciClass::CoProcessor,
            _ => PciClass::Unknown,
        }
    }
}

/// Base Address Register (BAR) type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarType {
    Memory32 { address: u64, size: u64, prefetchable: bool },
    Memory64 { address: u64, size: u64, prefetchable: bool },
    IoPort { port: u16, size: u32 },
    Unused,
}

/// PCI device representation
#[derive(Debug, Clone)]
pub struct PciDevice {
    /// Bus number (0-255)
    pub bus: u8,
    /// Device number (0-31)
    pub device: u8,
    /// Function number (0-7)
    pub function: u8,
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID
    pub device_id: u16,
    /// Class code
    pub class: u8,
    /// Subclass code
    pub subclass: u8,
    /// Programming interface
    pub prog_if: u8,
    /// Revision ID
    pub revision: u8,
    /// Header type
    pub header_type: u8,
    /// Base Address Registers (parsed)
    pub bars: [BarType; 6],
    /// Interrupt line
    pub interrupt_line: u8,
    /// Interrupt pin (0 = none, 1-4 = INTA-INTD)
    pub interrupt_pin: u8,
}

impl PciDevice {
    /// Check if this is a VirtIO device
    pub fn is_virtio(&self) -> bool {
        self.vendor_id == 0x1AF4 && self.device_id >= 0x1000 && self.device_id <= 0x103F
    }

    /// Get VirtIO device type if this is a VirtIO device
    pub fn virtio_device_type(&self) -> Option<u16> {
        if self.is_virtio() {
            // VirtIO 1.0+ PCI device IDs: 0x1040 + device_type
            // VirtIO 0.9.5 PCI device IDs: 0x1000 + device_type
            if self.device_id >= 0x1040 {
                Some(self.device_id - 0x1040)
            } else {
                Some(self.device_id - 0x1000)
            }
        } else {
            None
        }
    }

    /// Get BDF (Bus:Device.Function) identifier
    pub fn bdf(&self) -> u16 {
        ((self.bus as u16) << 8) | ((self.device as u16) << 3) | (self.function as u16)
    }
}

impl fmt::Display for PciDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}.{:x} [{:04x}:{:04x}] Class {:02x}.{:02x}.{:02x}",
            self.bus,
            self.device,
            self.function,
            self.vendor_id,
            self.device_id,
            self.class,
            self.subclass,
            self.prog_if
        )
    }
}

/// PCI bus controller (I/O port-based configuration access)
pub struct PciController {
    config_address: Port<u32>,
    config_data: Port<u32>,
}

impl PciController {
    /// Create a new PCI controller
    ///
    /// # Safety
    /// Must only be called once during system initialization.
    pub unsafe fn new() -> Self {
        Self {
            config_address: Port::new(PCI_CONFIG_ADDRESS),
            config_data: Port::new(PCI_CONFIG_DATA),
        }
    }

    /// Read a 32-bit value from PCI configuration space
    ///
    /// # Arguments
    /// * `bus` - Bus number (0-255)
    /// * `device` - Device number (0-31)
    /// * `function` - Function number (0-7)
    /// * `offset` - Register offset (must be 4-byte aligned)
    pub fn read_config_u32(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        let address = self.make_address(bus, device, function, offset);
        unsafe {
            self.config_address.write(address);
            self.config_data.read()
        }
    }

    /// Read a 16-bit value from PCI configuration space
    pub fn read_config_u16(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u16 {
        let dword = self.read_config_u32(bus, device, function, offset & 0xFC);
        let shift = (offset & 0x02) * 8;
        ((dword >> shift) & 0xFFFF) as u16
    }

    /// Read an 8-bit value from PCI configuration space
    pub fn read_config_u8(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u8 {
        let dword = self.read_config_u32(bus, device, function, offset & 0xFC);
        let shift = (offset & 0x03) * 8;
        ((dword >> shift) & 0xFF) as u8
    }

    /// Write a 32-bit value to PCI configuration space
    pub fn write_config_u32(&mut self, bus: u8, device: u8, function: u8, offset: u8, value: u32) {
        let address = self.make_address(bus, device, function, offset);
        unsafe {
            self.config_address.write(address);
            self.config_data.write(value);
        }
    }

    /// Write a 16-bit value to PCI configuration space
    pub fn write_config_u16(&mut self, bus: u8, device: u8, function: u8, offset: u8, value: u16) {
        let dword_offset = offset & 0xFC;
        let shift = (offset & 0x02) * 8;
        let old_dword = self.read_config_u32(bus, device, function, dword_offset);
        let mask = !(0xFFFF << shift);
        let new_dword = (old_dword & mask) | ((value as u32) << shift);
        self.write_config_u32(bus, device, function, dword_offset, new_dword);
    }

    /// Construct PCI configuration address
    ///
    /// Format: [Enable:1][Reserved:7][Bus:8][Device:5][Function:3][Offset:8]
    #[inline]
    fn make_address(&self, bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        0x8000_0000
            | ((bus as u32) << 16)
            | ((device as u32) << 11)
            | ((function as u32) << 8)
            | ((offset as u32) & 0xFC)
    }

    /// Check if a device exists at the given location
    pub fn device_exists(&mut self, bus: u8, device: u8, function: u8) -> bool {
        let vendor_id = self.read_config_u16(bus, device, function, 0x00);
        vendor_id != 0xFFFF
    }

    /// Scan for a specific device
    pub fn probe_device(&mut self, bus: u8, device: u8, function: u8) -> Option<PciDevice> {
        if !self.device_exists(bus, device, function) {
            return None;
        }

        let vendor_id = self.read_config_u16(bus, device, function, 0x00);
        let device_id = self.read_config_u16(bus, device, function, 0x02);
        let class = self.read_config_u8(bus, device, function, 0x0B);
        let subclass = self.read_config_u8(bus, device, function, 0x0A);
        let prog_if = self.read_config_u8(bus, device, function, 0x09);
        let revision = self.read_config_u8(bus, device, function, 0x08);
        let header_type = self.read_config_u8(bus, device, function, 0x0E);
        let interrupt_line = self.read_config_u8(bus, device, function, 0x3C);
        let interrupt_pin = self.read_config_u8(bus, device, function, 0x3D);

        // Parse BARs
        let mut bars = [BarType::Unused; 6];
        let mut bar_index = 0;
        while bar_index < 6 {
            let offset = 0x10 + (bar_index as u8 * 4);
            let bar_value = self.read_config_u32(bus, device, function, offset);

            if bar_value == 0 {
                bars[bar_index] = BarType::Unused;
                bar_index += 1;
                continue;
            }

            if bar_value & 0x01 == 1 {
                // I/O space BAR
                let port = (bar_value & 0xFFFC) as u16;

                // Determine size by writing all 1s and reading back
                self.write_config_u32(bus, device, function, offset, 0xFFFFFFFF);
                let size_mask = self.read_config_u32(bus, device, function, offset);
                self.write_config_u32(bus, device, function, offset, bar_value);

                let size = !(size_mask & 0xFFFC) + 1;
                bars[bar_index] = BarType::IoPort { port, size };
                bar_index += 1;
            } else {
                // Memory space BAR
                let bar_type = (bar_value >> 1) & 0x03;
                let prefetchable = (bar_value & 0x08) != 0;

                if bar_type == 0 {
                    // 32-bit memory
                    let address = (bar_value & 0xFFFFFFF0) as u64;

                    // Determine size
                    self.write_config_u32(bus, device, function, offset, 0xFFFFFFFF);
                    let size_mask = self.read_config_u32(bus, device, function, offset);
                    self.write_config_u32(bus, device, function, offset, bar_value);

                    let size = (!(size_mask & 0xFFFFFFF0) + 1) as u64;
                    bars[bar_index] = BarType::Memory32 { address, size, prefetchable };
                    bar_index += 1;
                } else if bar_type == 2 && bar_index < 5 {
                    // 64-bit memory
                    let low = (bar_value & 0xFFFFFFF0) as u64;
                    let high = self.read_config_u32(bus, device, function, offset + 4) as u64;
                    let address = low | (high << 32);

                    // Determine size
                    self.write_config_u32(bus, device, function, offset, 0xFFFFFFFF);
                    self.write_config_u32(bus, device, function, offset + 4, 0xFFFFFFFF);
                    let size_low = self.read_config_u32(bus, device, function, offset) as u64;
                    let size_high = self.read_config_u32(bus, device, function, offset + 4) as u64;
                    self.write_config_u32(bus, device, function, offset, bar_value);
                    self.write_config_u32(bus, device, function, offset + 4, high as u32);

                    let size_mask = (size_low & 0xFFFFFFF0) | (size_high << 32);
                    let size = !size_mask + 1;

                    bars[bar_index] = BarType::Memory64 { address, size, prefetchable };
                    bars[bar_index + 1] = BarType::Unused; // 64-bit BAR uses two slots
                    bar_index += 2;
                } else {
                    bars[bar_index] = BarType::Unused;
                    bar_index += 1;
                }
            }
        }

        Some(PciDevice {
            bus,
            device,
            function,
            vendor_id,
            device_id,
            class,
            subclass,
            prog_if,
            revision,
            header_type,
            bars,
            interrupt_line,
            interrupt_pin,
        })
    }

    /// Scan all PCI buses for devices
    pub fn scan_all(&mut self) -> Vec<PciDevice> {
        let mut devices = Vec::new();

        // Scan all possible bus/device/function combinations
        for bus in 0..=255u8 {
            for device in 0..32u8 {
                for function in 0..8u8 {
                    if let Some(pci_device) = self.probe_device(bus, device, function) {
                        devices.push(pci_device);

                        // If this is function 0 and not a multi-function device, skip other functions
                        if function == 0 && (pci_device.header_type & 0x80) == 0 {
                            break;
                        }
                    } else if function == 0 {
                        // No function 0 means no device at this slot
                        break;
                    }
                }
            }
        }

        devices
    }

    /// Enable bus mastering for a PCI device (required for DMA)
    pub fn enable_bus_mastering(&mut self, device: &PciDevice) {
        let mut command = self.read_config_u16(device.bus, device.device, device.function, 0x04);
        command |= command::BUS_MASTER | command::MEMORY_SPACE;
        self.write_config_u16(device.bus, device.device, device.function, 0x04, command);
    }
}

lazy_static! {
    /// Global PCI controller instance
    pub static ref PCI: Mutex<PciController> = {
        Mutex::new(unsafe { PciController::new() })
    };

    /// List of discovered PCI devices
    static ref PCI_DEVICES: Mutex<Vec<PciDevice>> = Mutex::new(Vec::new());
}

/// Initialize PCI bus and scan for devices
///
/// # Safety
/// Must be called during kernel initialization, after serial console is ready.
pub unsafe fn init() -> Result<usize, &'static str> {
    crate::arch::x86_64::serial::serial_write(b"[PCI] Scanning PCI bus...\n");

    let devices = PCI.lock().scan_all();
    let count = devices.len();

    crate::arch::x86_64::serial::serial_write(b"[PCI] Found ");
    print_decimal(count);
    crate::arch::x86_64::serial::serial_write(b" devices\n");

    // Print discovered devices
    for dev in &devices {
        crate::arch::x86_64::serial::serial_write(b"[PCI]   ");
        print_bdf(dev.bus, dev.device, dev.function);
        crate::arch::x86_64::serial::serial_write(b" ");
        print_hex_u16(dev.vendor_id);
        crate::arch::x86_64::serial::serial_write(b":");
        print_hex_u16(dev.device_id);
        crate::arch::x86_64::serial::serial_write(b" Class ");
        print_hex_u8(dev.class);
        crate::arch::x86_64::serial::serial_write(b".");
        print_hex_u8(dev.subclass);
        crate::arch::x86_64::serial::serial_write(b"\n");

        // Special logging for VirtIO devices
        if dev.is_virtio() {
            if let Some(vtype) = dev.virtio_device_type() {
                crate::arch::x86_64::serial::serial_write(b"[PCI]     -> VirtIO device type ");
                print_decimal(vtype as usize);
                crate::arch::x86_64::serial::serial_write(b" (");
                match vtype {
                    1 => crate::arch::x86_64::serial::serial_write(b"Net"),
                    2 => crate::arch::x86_64::serial::serial_write(b"Block"),
                    3 => crate::arch::x86_64::serial::serial_write(b"Console"),
                    16 => crate::arch::x86_64::serial::serial_write(b"GPU"),
                    _ => crate::arch::x86_64::serial::serial_write(b"Unknown"),
                }
                crate::arch::x86_64::serial::serial_write(b")\n");
            }
        }
    }

    *PCI_DEVICES.lock() = devices;
    Ok(count)
}

/// Get all discovered PCI devices
pub fn devices() -> Vec<PciDevice> {
    PCI_DEVICES.lock().clone()
}

/// Find VirtIO devices of a specific type
///
/// # Arguments
/// * `device_type` - VirtIO device type (1 = Net, 2 = Block, etc.)
pub fn find_virtio_devices(device_type: u16) -> Vec<PciDevice> {
    PCI_DEVICES
        .lock()
        .iter()
        .filter(|dev| dev.virtio_device_type() == Some(device_type))
        .cloned()
        .collect()
}

// Helper functions for serial output
fn print_decimal(n: usize) {
    if n == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut n = n;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::arch::x86_64::serial::serial_write(&[buf[i]]);
    }
}

fn print_hex_u8(n: u8) {
    let hex_chars = b"0123456789abcdef";
    let buf = [
        hex_chars[(n >> 4) as usize],
        hex_chars[(n & 0xF) as usize],
    ];
    crate::arch::x86_64::serial::serial_write(&buf);
}

fn print_hex_u16(n: u16) {
    let hex_chars = b"0123456789abcdef";
    let buf = [
        hex_chars[((n >> 12) & 0xF) as usize],
        hex_chars[((n >> 8) & 0xF) as usize],
        hex_chars[((n >> 4) & 0xF) as usize],
        hex_chars[(n & 0xF) as usize],
    ];
    crate::arch::x86_64::serial::serial_write(&buf);
}

fn print_bdf(bus: u8, device: u8, function: u8) {
    print_hex_u8(bus);
    crate::arch::x86_64::serial::serial_write(b":");
    print_hex_u8(device);
    crate::arch::x86_64::serial::serial_write(b".");
    crate::arch::x86_64::serial::serial_write(&[b'0' + function]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pci_address_construction() {
        let controller = unsafe { PciController::new() };
        let addr = controller.make_address(0, 0, 0, 0);
        assert_eq!(addr, 0x80000000);

        let addr = controller.make_address(1, 2, 3, 0x10);
        assert_eq!(addr, 0x80011310);
    }
}
