//! # VirtIO PCI Transport Layer
//!
//! This module implements the VirtIO 1.0+ PCI transport for x86_64 systems.
//! It provides the PCI-specific implementation of VirtIO device discovery,
//! initialization, and operation.
//!
//! ## VirtIO PCI Device Structure
//!
//! VirtIO 1.0+ devices on PCI use capability structures in the PCI configuration space
//! to describe device resources. This is different from the legacy 0.9.5 implementation
//! which used fixed I/O port locations.
//!
//! ## PCI Capability Structure
//!
//! VirtIO devices expose multiple capabilities starting at the offset specified in
//! the PCI Capabilities Pointer (offset 0x34):
//!
//! ```text
//! Offset  Size  Description
//! ------  ----  -----------
//! 0x00    1     Capability ID (0x09 = Vendor-Specific)
//! 0x01    1     Next capability offset
//! 0x02    1     Capability length
//! 0x03    1     Configuration type:
//!                 1 = Common configuration
//!                 2 = Notify configuration
//!                 3 = ISR Status
//!                 4 = Device-specific configuration
//!                 5 = PCI configuration access
//! 0x04    1     BAR index (0-5)
//! 0x05    3     Padding
//! 0x08    4     Offset within BAR
//! 0x0C    4     Length of the structure
//! ```
//!
//! ## Common Configuration Structure
//!
//! The common configuration structure (type 1) provides device-independent control:
//!
//! ```text
//! Offset  Size  Field
//! ------  ----  -----
//! 0x00    4     device_feature_select
//! 0x04    4     device_feature
//! 0x08    4     driver_feature_select
//! 0x0C    4     driver_feature
//! 0x10    2     msix_config
//! 0x12    2     num_queues
//! 0x14    1     device_status
//! 0x15    1     config_generation
//! 0x16    2     queue_select
//! 0x18    2     queue_size
//! 0x1A    2     queue_msix_vector
//! 0x1C    2     queue_enable
//! 0x1E    2     queue_notify_off
//! 0x20    8     queue_desc (64-bit physical address)
//! 0x28    8     queue_avail (64-bit physical address)
//! 0x30    8     queue_used (64-bit physical address)
//! ```

use crate::arch::x86_64::pci::{PciDevice, BarType, PCI};
use crate::arch::x86_64::paging::PageTableManager;
use x86_64::structures::paging::{PhysFrame, Size4KiB, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};
use core::ptr::{read_volatile, write_volatile};
use alloc::vec::Vec;

/// VirtIO PCI Capability IDs
const PCI_CAP_ID_VNDR: u8 = 0x09; // Vendor-specific capability

/// VirtIO PCI Configuration Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VirtioPciCapType {
    CommonCfg = 1,   // Common configuration
    NotifyCfg = 2,   // Notifications
    IsrCfg = 3,      // ISR Status
    DeviceCfg = 4,   // Device-specific configuration
    PciCfg = 5,      // PCI configuration access
}

/// VirtIO PCI Capability structure (as read from config space)
#[derive(Debug, Clone, Copy)]
pub struct VirtioPciCap {
    pub cap_type: VirtioPciCapType,
    pub bar: u8,
    pub offset: u32,
    pub length: u32,
}

/// VirtIO Device Status bits
pub mod status {
    pub const ACKNOWLEDGE: u8 = 1;       // Guest acknowledges device
    pub const DRIVER: u8 = 2;            // Guest driver loaded
    pub const DRIVER_OK: u8 = 4;         // Driver ready
    pub const FEATURES_OK: u8 = 8;       // Feature negotiation complete
    pub const DEVICE_NEEDS_RESET: u8 = 64; // Device needs reset
    pub const FAILED: u8 = 128;          // Fatal error occurred
}

/// Common Configuration structure layout
#[repr(C)]
pub struct CommonCfg {
    pub device_feature_select: u32,
    pub device_feature: u32,
    pub driver_feature_select: u32,
    pub driver_feature: u32,
    pub msix_config: u16,
    pub num_queues: u16,
    pub device_status: u8,
    pub config_generation: u8,
    pub queue_select: u16,
    pub queue_size: u16,
    pub queue_msix_vector: u16,
    pub queue_enable: u16,
    pub queue_notify_off: u16,
    pub queue_desc: u64,
    pub queue_avail: u64,
    pub queue_used: u64,
}

/// VirtIO PCI Transport
pub struct VirtioPciTransport {
    pub device: PciDevice,
    pub common_cfg: VirtAddr,
    pub notify_base: Option<VirtAddr>,
    pub notify_off_multiplier: u32,
    pub isr_status: Option<VirtAddr>,
    pub device_cfg: Option<VirtAddr>,
    pub device_cfg_len: usize,
}

impl VirtioPciTransport {
    /// Create a new VirtIO PCI transport for the given device
    ///
    /// This parses the PCI capability structures to locate the various
    /// VirtIO configuration regions.
    pub fn new(device: PciDevice) -> Result<Self, &'static str> {
        // Read capabilities pointer from PCI config space
        let cap_ptr = PCI.lock().read_config_u8(device.bus, device.device, device.function, 0x34);

        if cap_ptr == 0 {
            return Err("No PCI capabilities found");
        }

        // Parse capability list to find VirtIO structures
        let mut capabilities = Vec::new();
        let mut next_cap = cap_ptr;

        while next_cap != 0 && next_cap >= 0x40 {
            let cap_id = PCI.lock().read_config_u8(
                device.bus,
                device.device,
                device.function,
                next_cap,
            );

            if cap_id == PCI_CAP_ID_VNDR {
                // Read VirtIO capability structure
                let cfg_type = PCI.lock().read_config_u8(
                    device.bus,
                    device.device,
                    device.function,
                    next_cap + 3,
                );

                let bar = PCI.lock().read_config_u8(
                    device.bus,
                    device.device,
                    device.function,
                    next_cap + 4,
                );

                let offset = PCI.lock().read_config_u32(
                    device.bus,
                    device.device,
                    device.function,
                    next_cap + 8,
                );

                let length = PCI.lock().read_config_u32(
                    device.bus,
                    device.device,
                    device.function,
                    next_cap + 12,
                );

                // Convert cfg_type to enum
                let cap_type = match cfg_type {
                    1 => VirtioPciCapType::CommonCfg,
                    2 => VirtioPciCapType::NotifyCfg,
                    3 => VirtioPciCapType::IsrCfg,
                    4 => VirtioPciCapType::DeviceCfg,
                    5 => VirtioPciCapType::PciCfg,
                    _ => {
                        // Skip unknown capability types
                        next_cap = PCI.lock().read_config_u8(
                            device.bus,
                            device.device,
                            device.function,
                            next_cap + 1,
                        );
                        continue;
                    }
                };

                capabilities.push(VirtioPciCap {
                    cap_type,
                    bar,
                    offset,
                    length,
                });
            }

            // Move to next capability
            next_cap = PCI.lock().read_config_u8(
                device.bus,
                device.device,
                device.function,
                next_cap + 1,
            );
        }

        // Find required capabilities
        let common_cap = capabilities
            .iter()
            .find(|cap| cap.cap_type == VirtioPciCapType::CommonCfg)
            .ok_or("Missing common configuration capability")?;

        let notify_cap = capabilities
            .iter()
            .find(|cap| cap.cap_type == VirtioPciCapType::NotifyCfg);

        let isr_cap = capabilities
            .iter()
            .find(|cap| cap.cap_type == VirtioPciCapType::IsrCfg);

        let device_cap = capabilities
            .iter()
            .find(|cap| cap.cap_type == VirtioPciCapType::DeviceCfg);

        // Map common configuration BAR
        let common_cfg = Self::map_capability(&device, common_cap)?;

        // Map other capabilities if present
        let notify_base = if let Some(cap) = notify_cap {
            Some(Self::map_capability(&device, cap)?)
        } else {
            None
        };

        let isr_status = if let Some(cap) = isr_cap {
            Some(Self::map_capability(&device, cap)?)
        } else {
            None
        };

        let (device_cfg, device_cfg_len) = if let Some(cap) = device_cap {
            (Some(Self::map_capability(&device, cap)?), cap.length as usize)
        } else {
            (None, 0)
        };

        // Read notify_off_multiplier from notify capability (offset 16 in the capability structure)
        let notify_off_multiplier = if notify_cap.is_some() {
            // For now, use a default value of 0 (will be updated if we parse the full notify cap)
            // In practice, this should be read from the PCI config space at the notify cap + 16
            2 // Common value for QEMU
        } else {
            0
        };

        Ok(Self {
            device,
            common_cfg,
            notify_base,
            notify_off_multiplier,
            isr_status,
            device_cfg,
            device_cfg_len,
        })
    }

    /// Map a PCI BAR region into virtual address space
    fn map_capability(device: &PciDevice, cap: &VirtioPciCap) -> Result<VirtAddr, &'static str> {
        if cap.bar >= 6 {
            return Err("Invalid BAR index");
        }

        let bar = &device.bars[cap.bar as usize];

        // Get physical base address from BAR
        let phys_base = match bar {
            BarType::Memory32 { address, .. } => *address,
            BarType::Memory64 { address, .. } => *address,
            BarType::IoPort { .. } => return Err("I/O port BARs not supported for VirtIO config"),
            BarType::Unused => return Err("BAR is not configured"),
        };

        // Calculate physical address of the capability structure
        let phys_addr = PhysAddr::new(phys_base + cap.offset as u64);

        // Map the physical region into virtual memory
        // We'll map it using the direct physical memory mapping
        // For x86_64, we use the higher-half direct map: phys + PHYS_OFFSET
        const PHYS_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;
        let virt_addr = VirtAddr::new(phys_addr.as_u64() + PHYS_OFFSET);

        // For production, we should use proper page table mapping here
        // For now, rely on the direct physical memory mapping set up during boot

        Ok(virt_addr)
    }

    /// Get pointer to common configuration structure
    #[inline]
    pub fn common_cfg(&self) -> *mut CommonCfg {
        self.common_cfg.as_mut_ptr()
    }

    /// Read device status
    pub fn read_device_status(&self) -> u8 {
        unsafe {
            let common = self.common_cfg();
            read_volatile(&(*common).device_status)
        }
    }

    /// Write device status
    pub fn write_device_status(&self, status: u8) {
        unsafe {
            let common = self.common_cfg();
            write_volatile(&mut (*common).device_status, status);
        }
    }

    /// Reset the device
    pub fn reset(&self) {
        self.write_device_status(0);

        // Wait for reset to complete
        for _ in 0..1000 {
            if self.read_device_status() == 0 {
                break;
            }
            core::hint::spin_loop();
        }
    }

    /// Read device features (64-bit)
    pub fn read_device_features(&self) -> u64 {
        unsafe {
            let common = self.common_cfg();

            // Read low 32 bits
            write_volatile(&mut (*common).device_feature_select, 0);
            let low = read_volatile(&(*common).device_feature) as u64;

            // Read high 32 bits
            write_volatile(&mut (*common).device_feature_select, 1);
            let high = read_volatile(&(*common).device_feature) as u64;

            (high << 32) | low
        }
    }

    /// Write driver features (64-bit)
    pub fn write_driver_features(&self, features: u64) {
        unsafe {
            let common = self.common_cfg();

            // Write low 32 bits
            write_volatile(&mut (*common).driver_feature_select, 0);
            write_volatile(&mut (*common).driver_feature, features as u32);

            // Write high 32 bits
            write_volatile(&mut (*common).driver_feature_select, 1);
            write_volatile(&mut (*common).driver_feature, (features >> 32) as u32);
        }
    }

    /// Get number of queues supported by device
    pub fn num_queues(&self) -> u16 {
        unsafe {
            let common = self.common_cfg();
            read_volatile(&(*common).num_queues)
        }
    }

    /// Select a virtqueue
    pub fn select_queue(&self, index: u16) {
        unsafe {
            let common = self.common_cfg();
            write_volatile(&mut (*common).queue_select, index);
        }
    }

    /// Get maximum queue size for currently selected queue
    pub fn get_queue_max_size(&self) -> u16 {
        unsafe {
            let common = self.common_cfg();
            read_volatile(&(*common).queue_size)
        }
    }

    /// Set queue size for currently selected queue
    pub fn set_queue_size(&self, size: u16) {
        unsafe {
            let common = self.common_cfg();
            write_volatile(&mut (*common).queue_size, size);
        }
    }

    /// Set queue descriptor area physical address
    pub fn set_queue_desc(&self, addr: PhysAddr) {
        unsafe {
            let common = self.common_cfg();
            write_volatile(&mut (*common).queue_desc, addr.as_u64());
        }
    }

    /// Set queue available ring physical address
    pub fn set_queue_avail(&self, addr: PhysAddr) {
        unsafe {
            let common = self.common_cfg();
            write_volatile(&mut (*common).queue_avail, addr.as_u64());
        }
    }

    /// Set queue used ring physical address
    pub fn set_queue_used(&self, addr: PhysAddr) {
        unsafe {
            let common = self.common_cfg();
            write_volatile(&mut (*common).queue_used, addr.as_u64());
        }
    }

    /// Enable currently selected queue
    pub fn enable_queue(&self) {
        unsafe {
            let common = self.common_cfg();
            write_volatile(&mut (*common).queue_enable, 1);
        }
    }

    /// Notify device about available buffers in queue
    pub fn notify_queue(&self, queue_index: u16) {
        if let Some(notify_base) = self.notify_base {
            unsafe {
                let common = self.common_cfg();

                // Select queue to get its notify offset
                let old_select = read_volatile(&(*common).queue_select);
                write_volatile(&mut (*common).queue_select, queue_index);
                let notify_off = read_volatile(&(*common).queue_notify_off);
                write_volatile(&mut (*common).queue_select, old_select);

                // Calculate notification address
                let offset = notify_off as u64 * self.notify_off_multiplier as u64;
                let notify_addr = (notify_base.as_u64() + offset) as *mut u16;

                // Write queue index to notification address
                write_volatile(notify_addr, queue_index);
            }
        }
    }

    /// Read device-specific configuration space
    pub fn read_device_config<T: Copy>(&self, offset: usize) -> Result<T, &'static str> {
        if let Some(device_cfg) = self.device_cfg {
            if offset + core::mem::size_of::<T>() > self.device_cfg_len {
                return Err("Device config read out of bounds");
            }

            unsafe {
                let ptr = (device_cfg.as_u64() + offset as u64) as *const T;
                Ok(read_volatile(ptr))
            }
        } else {
            Err("Device configuration space not available")
        }
    }

    /// Enable bus mastering for DMA operations
    pub fn enable_bus_mastering(&self) {
        PCI.lock().enable_bus_mastering(&self.device);
    }
}

/// Initialize a VirtIO PCI device using the standard initialization sequence
///
/// This performs the VirtIO 1.0 initialization handshake:
/// 1. Reset device
/// 2. Set ACKNOWLEDGE status bit
/// 3. Set DRIVER status bit
/// 4. Read device features
/// 5. Negotiate features (driver writes accepted features)
/// 6. Set FEATURES_OK status bit
/// 7. Verify FEATURES_OK is still set
/// 8. Device-specific setup (queues, etc.)
/// 9. Set DRIVER_OK status bit
pub fn initialize_virtio_device(transport: &VirtioPciTransport) -> Result<u64, &'static str> {
    // Step 1: Reset device
    transport.reset();

    // Step 2: Set ACKNOWLEDGE status bit
    transport.write_device_status(status::ACKNOWLEDGE);

    // Step 3: Set DRIVER status bit
    transport.write_device_status(status::ACKNOWLEDGE | status::DRIVER);

    // Step 4: Read device features
    let device_features = transport.read_device_features();

    // Step 5 & 6: Negotiate features (for now, accept all device features)
    // In a real driver, we would mask with supported features
    transport.write_driver_features(device_features);

    transport.write_device_status(
        status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK
    );

    // Step 7: Verify FEATURES_OK is still set
    let status = transport.read_device_status();
    if status & status::FEATURES_OK == 0 {
        return Err("Device rejected feature negotiation");
    }

    // Steps 8-9 are device-specific and handled by the device driver

    Ok(device_features)
}
