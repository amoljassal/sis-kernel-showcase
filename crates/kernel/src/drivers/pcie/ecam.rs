//! PCIe Enhanced Configuration Access Mechanism (ECAM) Driver
//!
//! This module implements PCIe ECAM for memory-mapped configuration space access
//! on the Raspberry Pi 5 (BCM2712 SoC). ECAM provides direct memory access to
//! PCIe device configuration registers without the need for legacy I/O port access.
//!
//! # Overview
//!
//! ECAM maps the entire PCIe configuration space into system memory, allowing
//! software to access device configuration registers using standard load/store
//! instructions. Each device's configuration space is 4KB (standard) or 4KB extended.
//!
//! # Memory Layout
//!
//! ```text
//! ECAM Base + Bus[27:20] + Device[19:15] + Function[14:12] + Register[11:0]
//! ```
//!
//! For RPi5, the ECAM base address is typically at 0x1f00000000 (from FDT).
//!
//! # References
//! - PCIe Base Specification Rev 4.0
//! - PCI Express Enhanced Configuration Access Mechanism (ECAM)
//! - Linux: drivers/pci/controller/pcie-brcmstb.c

use super::PcieError;
use crate::drivers::DriverResult;
use core::ptr;

/// PCIe Vendor ID register offset
pub const PCI_VENDOR_ID: u16 = 0x00;

/// PCIe Device ID register offset
pub const PCI_DEVICE_ID: u16 = 0x02;

/// PCIe Command register offset
pub const PCI_COMMAND: u16 = 0x04;

/// PCIe Status register offset
pub const PCI_STATUS: u16 = 0x06;

/// PCIe Revision ID register offset
pub const PCI_REVISION_ID: u16 = 0x08;

/// PCIe Class Code register offset
pub const PCI_CLASS_CODE: u16 = 0x09;

/// PCIe Header Type register offset
pub const PCI_HEADER_TYPE: u16 = 0x0E;

/// PCIe Base Address Register 0 offset
pub const PCI_BAR0: u16 = 0x10;

/// PCIe Base Address Register 1 offset
pub const PCI_BAR1: u16 = 0x14;

/// PCIe Base Address Register 2 offset
pub const PCI_BAR2: u16 = 0x18;

/// PCIe Base Address Register 3 offset
pub const PCI_BAR3: u16 = 0x1C;

/// PCIe Base Address Register 4 offset
pub const PCI_BAR4: u16 = 0x20;

/// PCIe Base Address Register 5 offset
pub const PCI_BAR5: u16 = 0x24;

/// PCIe Subsystem Vendor ID offset
pub const PCI_SUBSYSTEM_VENDOR_ID: u16 = 0x2C;

/// PCIe Subsystem ID offset
pub const PCI_SUBSYSTEM_ID: u16 = 0x2E;

/// PCIe Interrupt Line register offset
pub const PCI_INTERRUPT_LINE: u16 = 0x3C;

/// PCIe Interrupt Pin register offset
pub const PCI_INTERRUPT_PIN: u16 = 0x3D;

/// PCIe configuration space size (4KB)
pub const PCI_CFG_SPACE_SIZE: usize = 0x1000;

/// Maximum number of buses
pub const PCI_MAX_BUS: u8 = 255;

/// Maximum number of devices per bus
pub const PCI_MAX_DEV: u8 = 32;

/// Maximum number of functions per device
pub const PCI_MAX_FUNC: u8 = 8;

/// Invalid vendor ID (indicates no device present)
pub const PCI_VENDOR_ID_INVALID: u16 = 0xFFFF;

/// PCIe Command register bits
pub mod command {
    /// Enable I/O space access
    pub const IO_ENABLE: u16 = 1 << 0;

    /// Enable memory space access
    pub const MEMORY_ENABLE: u16 = 1 << 1;

    /// Enable bus mastering
    pub const BUS_MASTER: u16 = 1 << 2;

    /// Enable interrupts
    pub const INTERRUPT_DISABLE: u16 = 1 << 10;
}

/// BAR (Base Address Register) type flags
pub mod bar {
    /// BAR is memory space (not I/O)
    pub const TYPE_MEMORY: u32 = 0 << 0;

    /// BAR is I/O space
    pub const TYPE_IO: u32 = 1 << 0;

    /// Memory BAR is 32-bit
    pub const MEM_TYPE_32BIT: u32 = 0 << 1;

    /// Memory BAR is 64-bit
    pub const MEM_TYPE_64BIT: u32 = 2 << 1;

    /// Memory is prefetchable
    pub const MEM_PREFETCHABLE: u32 = 1 << 3;

    /// Mask for extracting address from BAR
    pub const ADDR_MASK: u32 = 0xFFFF_FFF0;
}

/// PCIe device address (BDF - Bus/Device/Function)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PciAddress {
    /// Bus number (0-255)
    pub bus: u8,

    /// Device number (0-31)
    pub device: u8,

    /// Function number (0-7)
    pub function: u8,
}

impl PciAddress {
    /// Create a new PCIe address
    pub const fn new(bus: u8, device: u8, function: u8) -> Self {
        Self { bus, device, function }
    }

    /// Calculate ECAM offset for this address
    #[inline]
    pub fn ecam_offset(&self) -> usize {
        ((self.bus as usize) << 20)
            | ((self.device as usize) << 15)
            | ((self.function as usize) << 12)
    }
}

impl core::fmt::Display for PciAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02x}:{:02x}.{}", self.bus, self.device, self.function)
    }
}

/// Base Address Register (BAR) information
#[derive(Debug, Copy, Clone)]
pub struct BarInfo {
    /// BAR index (0-5)
    pub index: u8,

    /// Physical base address
    pub base: u64,

    /// Size in bytes
    pub size: u64,

    /// Is this a memory BAR (vs I/O)?
    pub is_memory: bool,

    /// Is this 64-bit? (only for memory BARs)
    pub is_64bit: bool,

    /// Is this prefetchable? (only for memory BARs)
    pub is_prefetchable: bool,
}

/// PCIe device information
#[derive(Debug, Copy, Clone)]
pub struct PciDevice {
    /// Device address (BDF)
    pub address: PciAddress,

    /// Vendor ID
    pub vendor_id: u16,

    /// Device ID
    pub device_id: u16,

    /// Revision ID
    pub revision_id: u8,

    /// Class code (3 bytes: base class, sub-class, programming interface)
    pub class_code: u32,

    /// Subsystem vendor ID
    pub subsystem_vendor: u16,

    /// Subsystem ID
    pub subsystem_id: u16,
}

impl PciDevice {
    /// Check if this is a specific vendor/device
    pub fn matches(&self, vendor: u16, device: u16) -> bool {
        self.vendor_id == vendor && self.device_id == device
    }

    /// Get base class code
    pub fn base_class(&self) -> u8 {
        ((self.class_code >> 16) & 0xFF) as u8
    }

    /// Get sub-class code
    pub fn sub_class(&self) -> u8 {
        ((self.class_code >> 8) & 0xFF) as u8
    }

    /// Get programming interface
    pub fn prog_interface(&self) -> u8 {
        (self.class_code & 0xFF) as u8
    }
}

/// ECAM configuration space accessor
pub struct Ecam {
    /// Base address of ECAM region
    base: usize,

    /// Size of ECAM region
    size: usize,
}

impl Ecam {
    /// Create a new ECAM accessor
    ///
    /// # Arguments
    /// * `base` - Physical base address of ECAM region
    /// * `size` - Size of ECAM region in bytes
    ///
    /// # Safety
    /// Caller must ensure the ECAM region is properly mapped and accessible.
    pub const fn new(base: usize, size: usize) -> Self {
        Self { base, size }
    }

    /// Get ECAM base address
    pub fn base(&self) -> usize {
        self.base
    }

    /// Get ECAM size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Calculate configuration space address for a device
    #[inline]
    fn config_addr(&self, addr: PciAddress, offset: u16) -> DriverResult<*mut u8> {
        if addr.device >= PCI_MAX_DEV {
            return Err(PcieError::InvalidDevice.into());
        }
        if addr.function >= PCI_MAX_FUNC {
            return Err(PcieError::InvalidFunction.into());
        }
        if (offset as usize) >= PCI_CFG_SPACE_SIZE {
            return Err(PcieError::InvalidOffset.into());
        }

        let ecam_offset = addr.ecam_offset() + (offset as usize);
        if ecam_offset >= self.size {
            return Err(PcieError::OutOfBounds.into());
        }

        Ok((self.base + ecam_offset) as *mut u8)
    }

    /// Read 8-bit value from configuration space
    pub fn read_u8(&self, addr: PciAddress, offset: u16) -> DriverResult<u8> {
        let ptr = self.config_addr(addr, offset)?;
        Ok(unsafe { ptr::read_volatile(ptr) })
    }

    /// Read 16-bit value from configuration space
    pub fn read_u16(&self, addr: PciAddress, offset: u16) -> DriverResult<u16> {
        if offset & 1 != 0 {
            return Err(PcieError::MisalignedAccess.into());
        }
        let ptr = self.config_addr(addr, offset)?;
        Ok(unsafe { ptr::read_volatile(ptr as *const u16) })
    }

    /// Read 32-bit value from configuration space
    pub fn read_u32(&self, addr: PciAddress, offset: u16) -> DriverResult<u32> {
        if offset & 3 != 0 {
            return Err(PcieError::MisalignedAccess.into());
        }
        let ptr = self.config_addr(addr, offset)?;
        Ok(unsafe { ptr::read_volatile(ptr as *const u32) })
    }

    /// Write 8-bit value to configuration space
    pub fn write_u8(&self, addr: PciAddress, offset: u16, value: u8) -> DriverResult<()> {
        let ptr = self.config_addr(addr, offset)?;
        unsafe { ptr::write_volatile(ptr, value); }
        Ok(())
    }

    /// Write 16-bit value to configuration space
    pub fn write_u16(&self, addr: PciAddress, offset: u16, value: u16) -> DriverResult<()> {
        if offset & 1 != 0 {
            return Err(PcieError::MisalignedAccess.into());
        }
        let ptr = self.config_addr(addr, offset)?;
        unsafe { ptr::write_volatile(ptr as *mut u16, value); }
        Ok(())
    }

    /// Write 32-bit value to configuration space
    pub fn write_u32(&self, addr: PciAddress, offset: u16, value: u32) -> DriverResult<()> {
        if offset & 3 != 0 {
            return Err(PcieError::MisalignedAccess.into());
        }
        let ptr = self.config_addr(addr, offset)?;
        unsafe { ptr::write_volatile(ptr as *mut u32, value); }
        Ok(())
    }

    /// Check if a device exists at the given address
    pub fn device_exists(&self, addr: PciAddress) -> bool {
        self.read_u16(addr, PCI_VENDOR_ID)
            .map(|vid| vid != PCI_VENDOR_ID_INVALID && vid != 0)
            .unwrap_or(false)
    }

    /// Read device information
    pub fn read_device_info(&self, addr: PciAddress) -> DriverResult<PciDevice> {
        let vendor_id = self.read_u16(addr, PCI_VENDOR_ID)?;
        if vendor_id == PCI_VENDOR_ID_INVALID || vendor_id == 0 {
            return Err(PcieError::NoDevice.into());
        }

        let device_id = self.read_u16(addr, PCI_DEVICE_ID)?;
        let revision_id = self.read_u8(addr, PCI_REVISION_ID)?;

        // Read class code (3 bytes)
        let class_prog = self.read_u8(addr, PCI_CLASS_CODE)? as u32;
        let class_sub = self.read_u8(addr, PCI_CLASS_CODE + 1)? as u32;
        let class_base = self.read_u8(addr, PCI_CLASS_CODE + 2)? as u32;
        let class_code = (class_base << 16) | (class_sub << 8) | class_prog;

        let subsystem_vendor = self.read_u16(addr, PCI_SUBSYSTEM_VENDOR_ID).unwrap_or(0);
        let subsystem_id = self.read_u16(addr, PCI_SUBSYSTEM_ID).unwrap_or(0);

        Ok(PciDevice {
            address: addr,
            vendor_id,
            device_id,
            revision_id,
            class_code,
            subsystem_vendor,
            subsystem_id,
        })
    }

    /// Read BAR (Base Address Register) information
    pub fn read_bar(&self, addr: PciAddress, bar_index: u8) -> DriverResult<Option<BarInfo>> {
        if bar_index >= 6 {
            return Err(PcieError::InvalidBar.into());
        }

        let bar_offset = PCI_BAR0 + (bar_index as u16 * 4);
        let bar_value = self.read_u32(addr, bar_offset)?;

        // Check if BAR is implemented
        if bar_value == 0 {
            return Ok(None);
        }

        let is_memory = (bar_value & bar::TYPE_IO) == 0;

        if !is_memory {
            // I/O BAR
            let base = (bar_value & 0xFFFF_FFFC) as u64;

            // Determine size by writing all 1s and reading back
            self.write_u32(addr, bar_offset, 0xFFFF_FFFF)?;
            let size_mask = self.read_u32(addr, bar_offset)?;
            self.write_u32(addr, bar_offset, bar_value)?; // Restore original

            let size = (!(size_mask & 0xFFFF_FFFC) + 1) as u64;

            return Ok(Some(BarInfo {
                index: bar_index,
                base,
                size,
                is_memory: false,
                is_64bit: false,
                is_prefetchable: false,
            }));
        }

        // Memory BAR
        let is_64bit = (bar_value & bar::MEM_TYPE_64BIT) == bar::MEM_TYPE_64BIT;
        let is_prefetchable = (bar_value & bar::MEM_PREFETCHABLE) != 0;

        // Determine size by writing all 1s and reading back
        self.write_u32(addr, bar_offset, 0xFFFF_FFFF)?;
        let size_mask = self.read_u32(addr, bar_offset)?;
        self.write_u32(addr, bar_offset, bar_value)?; // Restore original

        let base: u64;
        let size: u64;

        if is_64bit {
            // 64-bit BAR spans two consecutive BARs
            if bar_index >= 5 {
                return Err(PcieError::InvalidBar.into());
            }

            let bar_hi_offset = bar_offset + 4;
            let bar_hi_value = self.read_u32(addr, bar_hi_offset)?;

            base = ((bar_hi_value as u64) << 32) | ((bar_value & bar::ADDR_MASK) as u64);

            // Get size high part
            self.write_u32(addr, bar_hi_offset, 0xFFFF_FFFF)?;
            let size_mask_hi = self.read_u32(addr, bar_hi_offset)?;
            self.write_u32(addr, bar_hi_offset, bar_hi_value)?; // Restore original

            let size_mask_64 = ((size_mask_hi as u64) << 32) | ((size_mask & bar::ADDR_MASK) as u64);
            size = !size_mask_64 + 1;
        } else {
            // 32-bit BAR
            base = (bar_value & bar::ADDR_MASK) as u64;
            size = (!(size_mask & bar::ADDR_MASK) + 1) as u64;
        }

        Ok(Some(BarInfo {
            index: bar_index,
            base,
            size,
            is_memory,
            is_64bit,
            is_prefetchable,
        }))
    }

    /// Enable memory and I/O access for a device
    pub fn enable_device(&self, addr: PciAddress) -> DriverResult<()> {
        let mut cmd = self.read_u16(addr, PCI_COMMAND)?;
        cmd |= command::MEMORY_ENABLE | command::IO_ENABLE | command::BUS_MASTER;
        self.write_u16(addr, PCI_COMMAND, cmd)?;
        Ok(())
    }

    /// Scan a specific bus for devices
    pub fn scan_bus(&self, bus: u8) -> alloc::vec::Vec<PciDevice> {
        let mut devices = alloc::vec::Vec::new();

        for device in 0..PCI_MAX_DEV {
            let addr = PciAddress::new(bus, device, 0);

            if !self.device_exists(addr) {
                continue;
            }

            if let Ok(dev_info) = self.read_device_info(addr) {
                devices.push(dev_info);

                // Check for multi-function device
                if let Ok(header_type) = self.read_u8(addr, PCI_HEADER_TYPE) {
                    if (header_type & 0x80) != 0 {
                        // Multi-function device, scan remaining functions
                        for func in 1..PCI_MAX_FUNC {
                            let func_addr = PciAddress::new(bus, device, func);
                            if self.device_exists(func_addr) {
                                if let Ok(func_info) = self.read_device_info(func_addr) {
                                    devices.push(func_info);
                                }
                            }
                        }
                    }
                }
            }
        }

        devices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pci_address() {
        let addr = PciAddress::new(0, 0, 0);
        assert_eq!(addr.ecam_offset(), 0);

        let addr = PciAddress::new(1, 0, 0);
        assert_eq!(addr.ecam_offset(), 1 << 20);

        let addr = PciAddress::new(0, 1, 0);
        assert_eq!(addr.ecam_offset(), 1 << 15);

        let addr = PciAddress::new(0, 0, 1);
        assert_eq!(addr.ecam_offset(), 1 << 12);
    }

    #[test]
    fn test_pci_device_class() {
        let dev = PciDevice {
            address: PciAddress::new(0, 0, 0),
            vendor_id: 0x1234,
            device_id: 0x5678,
            revision_id: 1,
            class_code: 0x060400, // PCI-to-PCI bridge
            subsystem_vendor: 0,
            subsystem_id: 0,
        };

        assert_eq!(dev.base_class(), 0x06);
        assert_eq!(dev.sub_class(), 0x04);
        assert_eq!(dev.prog_interface(), 0x00);
    }
}
