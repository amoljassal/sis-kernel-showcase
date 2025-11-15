//! # VirtIO Block Device Driver
//!
//! This module implements a VirtIO block device driver for x86_64 systems.
//! It provides block-level I/O operations (read/write sectors) over the
//! VirtIO-PCI transport.
//!
//! ## VirtIO Block Device
//!
//! Device type: 2 (VIRTIO_BLK_DEVICE_ID)
//! Device features:
//! - VIRTIO_BLK_F_SIZE_MAX (1): Maximum segment size
//! - VIRTIO_BLK_F_SEG_MAX (2): Maximum number of segments
//! - VIRTIO_BLK_F_RO (5): Read-only device
//! - VIRTIO_BLK_F_BLK_SIZE (6): Logical block size
//! - VIRTIO_BLK_F_FLUSH (9): Cache flush support
//!
//! ## Request Format
//!
//! Each block request consists of three parts:
//!
//! ```text
//! ┌────────────────────────────┐
//! │ Request Header (16 bytes)  │  Device-readable
//! │  - type: u32               │
//! │  - reserved: u32           │
//! │  - sector: u64             │
//! └────────────────────────────┘
//! ┌────────────────────────────┐
//! │ Data Buffer (N bytes)      │  Device-readable (write) or
//! │                            │  Device-writable (read)
//! └────────────────────────────┘
//! ┌────────────────────────────┐
//! │ Status Byte (1 byte)       │  Device-writable
//! │  - 0 = OK                  │
//! │  - 1 = IO Error            │
//! │  - 2 = Unsupported         │
//! └────────────────────────────┘
//! ```
//!
//! ## Request Types
//!
//! - VIRTIO_BLK_T_IN (0): Read from device
//! - VIRTIO_BLK_T_OUT (1): Write to device
//! - VIRTIO_BLK_T_FLUSH (4): Flush write cache
//!
//! ## Device Configuration
//!
//! ```text
//! Offset  Size  Field
//! ------  ----  -----
//! 0x00    8     capacity (512-byte sectors)
//! 0x08    4     size_max (maximum segment size)
//! 0x0C    4     seg_max (maximum segments)
//! 0x10    4     geometry (C/H/S)
//! 0x14    4     blk_size (logical block size)
//! ```

use crate::arch::x86_64::virtio_pci::{VirtioPciTransport, status};
use crate::arch::x86_64::virtqueue::Virtqueue;
use crate::arch::x86_64::pci::PciDevice;
use x86_64::PhysAddr;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use spin::Mutex;
use alloc::vec::Vec;

/// VirtIO block device feature bits
pub mod features {
    pub const SIZE_MAX: u64 = 1 << 1;      // Maximum segment size
    pub const SEG_MAX: u64 = 1 << 2;       // Maximum number of segments
    pub const GEOMETRY: u64 = 1 << 4;      // Disk geometry available
    pub const RO: u64 = 1 << 5;            // Device is read-only
    pub const BLK_SIZE: u64 = 1 << 6;      // Block size of disk
    pub const FLUSH: u64 = 1 << 9;         // Cache flush command support
    pub const TOPOLOGY: u64 = 1 << 10;     // Topology information
}

/// VirtIO block request types
pub mod request_type {
    pub const IN: u32 = 0;          // Read
    pub const OUT: u32 = 1;         // Write
    pub const FLUSH: u32 = 4;       // Flush
    pub const DISCARD: u32 = 11;    // Discard
    pub const WRITE_ZEROES: u32 = 13; // Write zeroes
}

/// VirtIO block request status
pub mod status_code {
    pub const OK: u8 = 0;           // Success
    pub const IOERR: u8 = 1;        // I/O error
    pub const UNSUPP: u8 = 2;       // Unsupported operation
}

/// Block request header (16 bytes, device-readable)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioBlkReq {
    /// Request type (IN, OUT, FLUSH, etc.)
    pub req_type: u32,
    /// Reserved (must be 0)
    pub reserved: u32,
    /// Starting sector number
    pub sector: u64,
}

/// Block device configuration space
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioBlkConfig {
    /// Capacity in 512-byte sectors
    pub capacity: u64,
    /// Maximum segment size (if SIZE_MAX feature)
    pub size_max: u32,
    /// Maximum number of segments (if SEG_MAX feature)
    pub seg_max: u32,
    /// Geometry (cylinders, heads, sectors)
    pub geometry: [u8; 4],
    /// Logical block size (if BLK_SIZE feature)
    pub blk_size: u32,
}

/// In-flight request tracking
struct PendingRequest {
    /// Request header physical address
    req_phys: PhysAddr,
    /// Request header virtual address
    req_virt: usize,
    /// Data buffer physical address
    data_phys: PhysAddr,
    /// Data buffer virtual address
    data_virt: usize,
    /// Status byte physical address
    status_phys: PhysAddr,
    /// Status byte virtual address
    status_virt: usize,
    /// Descriptor chain head
    desc_head: u16,
    /// Request completed flag
    completed: AtomicBool,
}

/// VirtIO block device
pub struct VirtioBlockDevice {
    /// PCI transport layer
    transport: VirtioPciTransport,
    /// Primary I/O virtqueue (queue 0)
    queue: Mutex<Virtqueue>,
    /// Device capacity in 512-byte sectors
    capacity: u64,
    /// Logical block size (usually 512)
    block_size: u32,
    /// Device is read-only
    read_only: bool,
    /// Pending requests (for async operation)
    pending: Mutex<Vec<PendingRequest>>,
}

impl VirtioBlockDevice {
    /// Create a new VirtIO block device
    ///
    /// # Arguments
    /// * `pci_device` - PCI device information
    ///
    /// # Safety
    /// Device must be a valid VirtIO block device
    pub unsafe fn new(pci_device: PciDevice) -> Result<Self, &'static str> {
        // Create VirtIO-PCI transport
        let transport = VirtioPciTransport::new(pci_device)?;

        // Perform VirtIO initialization handshake
        // 1. Reset device
        transport.reset();

        // 2. Set ACKNOWLEDGE status bit
        transport.write_device_status(status::ACKNOWLEDGE);

        // 3. Set DRIVER status bit
        transport.write_device_status(status::ACKNOWLEDGE | status::DRIVER);

        // 4. Read device features
        let device_features = transport.read_device_features();

        // 5. Negotiate features (accept all for now)
        let supported_features = device_features & (
            features::SIZE_MAX |
            features::SEG_MAX |
            features::RO |
            features::BLK_SIZE |
            features::FLUSH
        );
        transport.write_driver_features(supported_features);

        // 6. Set FEATURES_OK status bit
        transport.write_device_status(
            status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK
        );

        // 7. Verify FEATURES_OK is still set
        let device_status = transport.read_device_status();
        if device_status & status::FEATURES_OK == 0 {
            return Err("Device rejected feature negotiation");
        }

        // 8. Read device configuration
        let config: VirtioBlkConfig = transport.read_device_config(0)?;

        crate::arch::x86_64::serial::serial_write(b"[VirtIO-BLK] Device capacity: ");
        print_u64(config.capacity);
        crate::arch::x86_64::serial::serial_write(b" sectors (");
        print_u64(config.capacity * 512 / 1024 / 1024);
        crate::arch::x86_64::serial::serial_write(b" MB)\n");

        // 9. Set up virtqueue (queue 0)
        transport.select_queue(0);
        let queue_size = transport.get_queue_max_size();

        crate::arch::x86_64::serial::serial_write(b"[VirtIO-BLK] Queue size: ");
        print_u16(queue_size);
        crate::arch::x86_64::serial::serial_write(b"\n");

        // Use smaller queue size for simplicity (must be power of 2)
        let actual_queue_size = queue_size.min(128);

        // Allocate virtqueue memory
        let (queue_phys, queue_virt, _) =
            crate::arch::x86_64::virtqueue::alloc_virtqueue_memory(actual_queue_size)?;

        let queue = Virtqueue::new(actual_queue_size, queue_phys, queue_virt);

        // Configure queue addresses
        let (desc_phys, avail_phys, used_phys) = queue.get_physical_addresses();

        transport.set_queue_size(actual_queue_size);
        transport.set_queue_desc(desc_phys);
        transport.set_queue_avail(avail_phys);
        transport.set_queue_used(used_phys);

        // Enable queue
        transport.enable_queue();

        // Enable bus mastering for DMA
        transport.enable_bus_mastering();

        // 10. Set DRIVER_OK status bit
        transport.write_device_status(
            status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK | status::DRIVER_OK
        );

        crate::arch::x86_64::serial::serial_write(b"[VirtIO-BLK] Device initialized successfully\n");

        Ok(Self {
            transport,
            queue: Mutex::new(queue),
            capacity: config.capacity,
            block_size: if supported_features & features::BLK_SIZE != 0 {
                config.blk_size
            } else {
                512
            },
            read_only: supported_features & features::RO != 0,
            pending: Mutex::new(Vec::new()),
        })
    }

    /// Read sectors from the device (synchronous)
    ///
    /// # Arguments
    /// * `sector` - Starting sector number
    /// * `buffer` - Buffer to read into (must be multiple of block size)
    ///
    /// # Returns
    /// Number of bytes read, or error
    pub fn read_sectors(&self, sector: u64, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if self.read_only {
            // Reading is allowed on read-only devices
        }

        if buffer.len() % self.block_size as usize != 0 {
            return Err("Buffer size must be multiple of block size");
        }

        let sectors = buffer.len() / self.block_size as usize;
        if sector + sectors as u64 > self.capacity {
            return Err("Read beyond end of device");
        }

        // Allocate request structures
        let (req_phys, req_virt) = Self::alloc_dma_buffer(16)?;
        let (data_phys, data_virt) = Self::alloc_dma_buffer(buffer.len())?;
        let (status_phys, status_virt) = Self::alloc_dma_buffer(1)?;

        // Build request header
        unsafe {
            let req = req_virt as *mut VirtioBlkReq;
            write_volatile(&mut (*req).req_type, request_type::IN);
            write_volatile(&mut (*req).reserved, 0);
            write_volatile(&mut (*req).sector, sector);

            // Initialize status to error (will be overwritten by device)
            write_volatile(status_virt as *mut u8, status_code::IOERR);
        }

        // Add buffers to virtqueue
        let desc_head = {
            let mut queue = self.queue.lock();
            queue.add_buffer_chain(&[
                (req_phys, 16, false),           // Request header (device reads)
                (data_phys, buffer.len() as u32, true),  // Data buffer (device writes)
                (status_phys, 1, true),          // Status byte (device writes)
            ]).ok_or("Virtqueue full")?
        };

        // Notify device
        self.transport.notify_queue(0);

        // Wait for completion (polling for now)
        let mut timeout = 1000000;
        loop {
            let mut queue = self.queue.lock();
            if let Some((head, _bytes)) = queue.get_used() {
                if head == desc_head {
                    // Reclaim descriptors
                    queue.reclaim_chain(head);
                    break;
                }
            }
            drop(queue);

            timeout -= 1;
            if timeout == 0 {
                // Clean up
                unsafe {
                    Self::free_dma_buffer(req_phys, req_virt, 16);
                    Self::free_dma_buffer(data_phys, data_virt, buffer.len());
                    Self::free_dma_buffer(status_phys, status_virt, 1);
                }
                return Err("Read timeout");
            }

            core::hint::spin_loop();
        }

        // Check status
        let status = unsafe { read_volatile(status_virt as *const u8) };
        if status != status_code::OK {
            unsafe {
                Self::free_dma_buffer(req_phys, req_virt, 16);
                Self::free_dma_buffer(data_phys, data_virt, buffer.len());
                Self::free_dma_buffer(status_phys, status_virt, 1);
            }
            return Err("Read I/O error");
        }

        // Copy data to user buffer
        unsafe {
            core::ptr::copy_nonoverlapping(
                data_virt as *const u8,
                buffer.as_mut_ptr(),
                buffer.len(),
            );
        }

        // Free DMA buffers
        unsafe {
            Self::free_dma_buffer(req_phys, req_virt, 16);
            Self::free_dma_buffer(data_phys, data_virt, buffer.len());
            Self::free_dma_buffer(status_phys, status_virt, 1);
        }

        Ok(buffer.len())
    }

    /// Write sectors to the device (synchronous)
    ///
    /// # Arguments
    /// * `sector` - Starting sector number
    /// * `buffer` - Buffer to write from (must be multiple of block size)
    ///
    /// # Returns
    /// Number of bytes written, or error
    pub fn write_sectors(&self, sector: u64, buffer: &[u8]) -> Result<usize, &'static str> {
        if self.read_only {
            return Err("Device is read-only");
        }

        if buffer.len() % self.block_size as usize != 0 {
            return Err("Buffer size must be multiple of block size");
        }

        let sectors = buffer.len() / self.block_size as usize;
        if sector + sectors as u64 > self.capacity {
            return Err("Write beyond end of device");
        }

        // Allocate request structures
        let (req_phys, req_virt) = Self::alloc_dma_buffer(16)?;
        let (data_phys, data_virt) = Self::alloc_dma_buffer(buffer.len())?;
        let (status_phys, status_virt) = Self::alloc_dma_buffer(1)?;

        // Build request header
        unsafe {
            let req = req_virt as *mut VirtioBlkReq;
            write_volatile(&mut (*req).req_type, request_type::OUT);
            write_volatile(&mut (*req).reserved, 0);
            write_volatile(&mut (*req).sector, sector);

            // Copy data to DMA buffer
            core::ptr::copy_nonoverlapping(
                buffer.as_ptr(),
                data_virt as *mut u8,
                buffer.len(),
            );

            // Initialize status
            write_volatile(status_virt as *mut u8, status_code::IOERR);
        }

        // Add buffers to virtqueue
        let desc_head = {
            let mut queue = self.queue.lock();
            queue.add_buffer_chain(&[
                (req_phys, 16, false),           // Request header
                (data_phys, buffer.len() as u32, false),  // Data buffer (device reads)
                (status_phys, 1, true),          // Status byte
            ]).ok_or("Virtqueue full")?
        };

        // Notify device
        self.transport.notify_queue(0);

        // Wait for completion
        let mut timeout = 1000000;
        loop {
            let mut queue = self.queue.lock();
            if let Some((head, _bytes)) = queue.get_used() {
                if head == desc_head {
                    queue.reclaim_chain(head);
                    break;
                }
            }
            drop(queue);

            timeout -= 1;
            if timeout == 0 {
                unsafe {
                    Self::free_dma_buffer(req_phys, req_virt, 16);
                    Self::free_dma_buffer(data_phys, data_virt, buffer.len());
                    Self::free_dma_buffer(status_phys, status_virt, 1);
                }
                return Err("Write timeout");
            }

            core::hint::spin_loop();
        }

        // Check status
        let status = unsafe { read_volatile(status_virt as *const u8) };
        if status != status_code::OK {
            unsafe {
                Self::free_dma_buffer(req_phys, req_virt, 16);
                Self::free_dma_buffer(data_phys, data_virt, buffer.len());
                Self::free_dma_buffer(status_phys, status_virt, 1);
            }
            return Err("Write I/O error");
        }

        // Free DMA buffers
        unsafe {
            Self::free_dma_buffer(req_phys, req_virt, 16);
            Self::free_dma_buffer(data_phys, data_virt, buffer.len());
            Self::free_dma_buffer(status_phys, status_virt, 1);
        }

        Ok(buffer.len())
    }

    /// Allocate DMA-capable buffer
    ///
    /// Returns (physical_address, virtual_address)
    fn alloc_dma_buffer(size: usize) -> Result<(PhysAddr, usize), &'static str> {
        let mut pages = (size + 4095) / 4096;
        if pages == 0 {
            pages = 1;
        }
        let pow2 = pages.next_power_of_two();
        let order = pow2.trailing_zeros() as u8;
        let phys = crate::mm::alloc_pages(order)
            .ok_or("Failed to allocate DMA buffer")?;

        const PHYS_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;
        let virt = (phys + PHYS_OFFSET) as usize;

        // Zero the buffer
        unsafe {
            core::ptr::write_bytes(virt as *mut u8, 0, (1usize << order) * 4096);
        }

        Ok((PhysAddr::new(phys), virt))
    }

    /// Free DMA buffer
    unsafe fn free_dma_buffer(phys: PhysAddr, _virt: usize, size: usize) {
        let mut pages = (size + 4095) / 4096;
        if pages == 0 {
            pages = 1;
        }
        let order = pages.next_power_of_two().trailing_zeros() as u8;
        crate::mm::free_pages(phys.as_u64(), order);
    }

    /// Get device capacity in sectors
    pub fn capacity_sectors(&self) -> u64 {
        self.capacity
    }

    /// Get device capacity in bytes
    pub fn capacity_bytes(&self) -> u64 {
        self.capacity * self.block_size as u64
    }

    /// Get logical block size
    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    /// Check if device is read-only
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
}

// Helper functions for debug output
fn print_u64(mut n: u64) {
    if n == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
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
        crate::arch::x86_64::serial::serial_write_byte(buf[i]);
    }
}

fn print_u16(n: u16) {
    print_u64(n as u64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_type_constants() {
        assert_eq!(request_type::IN, 0);
        assert_eq!(request_type::OUT, 1);
        assert_eq!(request_type::FLUSH, 4);
    }

    #[test]
    fn test_status_constants() {
        assert_eq!(status_code::OK, 0);
        assert_eq!(status_code::IOERR, 1);
        assert_eq!(status_code::UNSUPP, 2);
    }
}
