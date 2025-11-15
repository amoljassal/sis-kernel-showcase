//! # VirtIO Network Device Driver (x86_64 PCI)
//!
//! This module implements a VirtIO network device driver for x86_64 systems using
//! the PCI transport layer. It provides network packet transmission and reception.
//!
//! ## VirtIO Network Device
//!
//! Device type: 1 (VIRTIO_NET_DEVICE_ID)
//! Device features:
//! - VIRTIO_NET_F_CSUM (0): Checksum offloading
//! - VIRTIO_NET_F_GUEST_CSUM (1): Guest checksum offload
//! - VIRTIO_NET_F_MAC (5): Device has MAC address
//! - VIRTIO_NET_F_GSO (6): Generic segmentation offload
//! - VIRTIO_NET_F_GUEST_TSO4 (7): Guest can receive TSO
//! - VIRTIO_NET_F_GUEST_UFO (10): Guest can receive UFO
//! - VIRTIO_NET_F_HOST_TSO4 (11): Host can receive TSO
//! - VIRTIO_NET_F_HOST_UFO (14): Host can receive UFO
//! - VIRTIO_NET_F_MRG_RXBUF (15): Merge RX buffers
//! - VIRTIO_NET_F_STATUS (16): Status field available
//! - VIRTIO_NET_F_CTRL_VQ (17): Control virtqueue
//! - VIRTIO_NET_F_CTRL_RX (18): RX mode control
//! - VIRTIO_NET_F_CTRL_VLAN (19): VLAN filtering
//! - VIRTIO_NET_F_MQ (22): Multiqueue support
//!
//! ## Virtqueues
//!
//! Standard configuration uses 2 queues:
//! - **Queue 0 (RX)**: Receive queue for incoming packets
//! - **Queue 1 (TX)**: Transmit queue for outgoing packets
//!
//! ## Packet Format
//!
//! Each packet consists of a header followed by data:
//!
//! ```text
//! ┌────────────────────────┐
//! │ VirtioNetHdr (12 bytes)│  Always present
//! │   flags: u8            │
//! │   gso_type: u8         │
//! │   hdr_len: u16         │
//! │   gso_size: u16        │
//! │   csum_start: u16      │
//! │   csum_offset: u16     │
//! │   num_buffers: u16     │  (if VIRTIO_NET_F_MRG_RXBUF)
//! └────────────────────────┘
//! ┌────────────────────────┐
//! │ Packet Data (N bytes)  │  Ethernet frame
//! └────────────────────────┘
//! ```
//!
//! ## Device Configuration
//!
//! ```text
//! Offset  Size  Field
//! ------  ----  -----
//! 0x00    6     mac (MAC address)
//! 0x06    2     status
//! 0x08    2     max_virtqueue_pairs
//! 0x0A    2     mtu
//! ```

use crate::arch::x86_64::virtio_pci::{VirtioPciTransport, status};
use crate::arch::x86_64::virtqueue::Virtqueue;
use crate::arch::x86_64::pci::PciDevice;
use x86_64::PhysAddr;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use spin::Mutex;
use alloc::vec::Vec;
use alloc::collections::VecDeque;

/// VirtIO network device feature bits
pub mod features {
    pub const CSUM: u64 = 1 << 0;           // Checksum offload
    pub const GUEST_CSUM: u64 = 1 << 1;     // Guest checksum
    pub const MAC: u64 = 1 << 5;            // Device has MAC address
    pub const GSO: u64 = 1 << 6;            // Generic segmentation offload
    pub const GUEST_TSO4: u64 = 1 << 7;     // Guest TSO for IPv4
    pub const GUEST_TSO6: u64 = 1 << 8;     // Guest TSO for IPv6
    pub const GUEST_UFO: u64 = 1 << 10;     // Guest UFO
    pub const HOST_TSO4: u64 = 1 << 11;     // Host TSO for IPv4
    pub const HOST_TSO6: u64 = 1 << 12;     // Host TSO for IPv6
    pub const HOST_UFO: u64 = 1 << 14;      // Host UFO
    pub const MRG_RXBUF: u64 = 1 << 15;     // Merge RX buffers
    pub const STATUS: u64 = 1 << 16;        // Status field
    pub const CTRL_VQ: u64 = 1 << 17;       // Control virtqueue
    pub const CTRL_RX: u64 = 1 << 18;       // RX mode control
    pub const CTRL_VLAN: u64 = 1 << 19;     // VLAN filtering
    pub const MQ: u64 = 1 << 22;            // Multiqueue
}

/// Network device configuration space
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioNetConfig {
    /// MAC address (6 bytes)
    pub mac: [u8; 6],
    /// Status flags
    pub status: u16,
    /// Maximum virtqueue pairs
    pub max_virtqueue_pairs: u16,
    /// Maximum transmission unit
    pub mtu: u16,
}

/// VirtIO network packet header (12 bytes without MRG_RXBUF, 14 bytes with)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioNetHdr {
    /// Flags (NEEDS_CSUM, DATA_VALID, RSC_INFO)
    pub flags: u8,
    /// GSO type (NONE, TCPV4, UDP, TCPV6, ECN)
    pub gso_type: u8,
    /// Header length
    pub hdr_len: u16,
    /// GSO size
    pub gso_size: u16,
    /// Checksum start offset
    pub csum_start: u16,
    /// Checksum offset
    pub csum_offset: u16,
    /// Number of buffers (only if MRG_RXBUF feature)
    pub num_buffers: u16,
}

impl VirtioNetHdr {
    /// Create a default header for simple packets
    pub const fn new() -> Self {
        Self {
            flags: 0,
            gso_type: 0,
            hdr_len: 0,
            gso_size: 0,
            csum_start: 0,
            csum_offset: 0,
            num_buffers: 0,
        }
    }
}

/// Received packet
pub struct RxPacket {
    /// Packet data (without VirtioNetHdr)
    pub data: Vec<u8>,
}

/// VirtIO network device
pub struct VirtioNetDevice {
    /// PCI transport layer
    transport: VirtioPciTransport,
    /// Receive virtqueue (queue 0)
    rx_queue: Mutex<Virtqueue>,
    /// Transmit virtqueue (queue 1)
    tx_queue: Mutex<Virtqueue>,
    /// MAC address
    mac: [u8; 6],
    /// Maximum transmission unit
    mtu: u16,
    /// Device supports MRG_RXBUF
    has_mrg_rxbuf: bool,
    /// Received packets waiting to be processed
    rx_pending: Mutex<VecDeque<RxPacket>>,
}

const NET_HDR_SIZE: usize = core::mem::size_of::<VirtioNetHdr>();
const MAX_PACKET_SIZE: usize = 1514; // Ethernet MTU (1500) + header (14)
const RX_BUFFER_SIZE: usize = NET_HDR_SIZE + MAX_PACKET_SIZE;

impl VirtioNetDevice {
    /// Create a new VirtIO network device
    ///
    /// # Arguments
    /// * `pci_device` - PCI device information
    ///
    /// # Safety
    /// Device must be a valid VirtIO network device
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

        // 5. Negotiate features
        // We support: MAC, STATUS, and MRG_RXBUF (optional)
        let mut supported_features = features::MAC | features::STATUS;
        if device_features & features::MRG_RXBUF != 0 {
            supported_features |= features::MRG_RXBUF;
        }

        let negotiated_features = device_features & supported_features;
        transport.write_driver_features(negotiated_features);

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
        let config: VirtioNetConfig = transport.read_device_config(0)?;

        crate::arch::x86_64::serial::serial_write(b"[VirtIO-NET] MAC address: ");
        for (i, &byte) in config.mac.iter().enumerate() {
            print_hex_u8(byte);
            if i < 5 {
                crate::arch::x86_64::serial::serial_write(b":");
            }
        }
        crate::arch::x86_64::serial::serial_write(b"\n");

        crate::arch::x86_64::serial::serial_write(b"[VirtIO-NET] MTU: ");
        print_u16(config.mtu);
        crate::arch::x86_64::serial::serial_write(b"\n");

        // 9. Set up virtqueues
        // RX queue (queue 0)
        transport.select_queue(0);
        let rx_queue_size = transport.get_queue_max_size().min(128);

        let (rx_queue_phys, rx_queue_virt, _) =
            crate::arch::x86_64::virtqueue::alloc_virtqueue_memory(rx_queue_size)?;
        let rx_queue = Virtqueue::new(rx_queue_size, rx_queue_phys, rx_queue_virt);

        let (rx_desc_phys, rx_avail_phys, rx_used_phys) = rx_queue.get_physical_addresses();
        transport.set_queue_size(rx_queue_size);
        transport.set_queue_desc(rx_desc_phys);
        transport.set_queue_avail(rx_avail_phys);
        transport.set_queue_used(rx_used_phys);
        transport.enable_queue();

        // TX queue (queue 1)
        transport.select_queue(1);
        let tx_queue_size = transport.get_queue_max_size().min(128);

        let (tx_queue_phys, tx_queue_virt, _) =
            crate::arch::x86_64::virtqueue::alloc_virtqueue_memory(tx_queue_size)?;
        let tx_queue = Virtqueue::new(tx_queue_size, tx_queue_phys, tx_queue_virt);

        let (tx_desc_phys, tx_avail_phys, tx_used_phys) = tx_queue.get_physical_addresses();
        transport.set_queue_size(tx_queue_size);
        transport.set_queue_desc(tx_desc_phys);
        transport.set_queue_avail(tx_avail_phys);
        transport.set_queue_used(tx_used_phys);
        transport.enable_queue();

        // Enable bus mastering for DMA
        transport.enable_bus_mastering();

        // 10. Set DRIVER_OK status bit
        transport.write_device_status(
            status::ACKNOWLEDGE | status::DRIVER | status::FEATURES_OK | status::DRIVER_OK
        );

        crate::arch::x86_64::serial::serial_write(b"[VirtIO-NET] Device initialized successfully\n");
        crate::arch::x86_64::serial::serial_write(b"[VirtIO-NET] RX queue size: ");
        print_u16(rx_queue_size);
        crate::arch::x86_64::serial::serial_write(b", TX queue size: ");
        print_u16(tx_queue_size);
        crate::arch::x86_64::serial::serial_write(b"\n");

        let has_mrg_rxbuf = negotiated_features & features::MRG_RXBUF != 0;

        let mut device = Self {
            transport,
            rx_queue: Mutex::new(rx_queue),
            tx_queue: Mutex::new(tx_queue),
            mac: config.mac,
            mtu: config.mtu,
            has_mrg_rxbuf,
            rx_pending: Mutex::new(VecDeque::new()),
        };

        // Pre-fill RX queue with buffers
        device.refill_rx_queue()?;

        Ok(device)
    }

    /// Refill RX queue with empty buffers
    fn refill_rx_queue(&self) -> Result<(), &'static str> {
        let mut rx_queue = self.rx_queue.lock();

        // Fill up to 75% of queue capacity
        let target_buffers = (rx_queue.size() as usize * 3) / 4;
        let current_free = rx_queue.free_count();

        for _ in 0..(target_buffers.saturating_sub(rx_queue.size() as usize - current_free)) {
            // Allocate buffer for header + packet
            let (buf_phys, buf_virt) = Self::alloc_dma_buffer(RX_BUFFER_SIZE)?;

            // Add to RX queue (device will write to this buffer)
            if rx_queue.add_buffer(buf_phys, RX_BUFFER_SIZE as u32, true).is_none() {
                unsafe {
                    Self::free_dma_buffer(buf_phys, buf_virt, RX_BUFFER_SIZE);
                }
                break;
            }
        }

        // Notify device
        self.transport.notify_queue(0);

        Ok(())
    }

    /// Transmit a packet (synchronous)
    ///
    /// # Arguments
    /// * `packet_data` - Ethernet frame to transmit
    ///
    /// # Returns
    /// Ok(()) on success, Err on failure
    pub fn transmit(&self, packet_data: &[u8]) -> Result<(), &'static str> {
        if packet_data.len() > MAX_PACKET_SIZE {
            return Err("Packet too large");
        }

        // Allocate buffers: header + data
        let (hdr_phys, hdr_virt) = Self::alloc_dma_buffer(NET_HDR_SIZE)?;
        let (data_phys, data_virt) = Self::alloc_dma_buffer(packet_data.len())?;

        // Build packet header
        unsafe {
            let hdr = hdr_virt as *mut VirtioNetHdr;
            write_volatile(hdr, VirtioNetHdr::new());

            // Copy packet data
            core::ptr::copy_nonoverlapping(
                packet_data.as_ptr(),
                data_virt as *mut u8,
                packet_data.len(),
            );
        }

        // Add to TX queue
        let desc_head = {
            let mut tx_queue = self.tx_queue.lock();
            tx_queue.add_buffer_chain(&[
                (hdr_phys, NET_HDR_SIZE as u32, false),        // Header (device reads)
                (data_phys, packet_data.len() as u32, false),  // Data (device reads)
            ]).ok_or("TX queue full")?
        };

        // Notify device
        self.transport.notify_queue(1);

        // Wait for completion (polling)
        let mut timeout = 1000000;
        loop {
            let mut tx_queue = self.tx_queue.lock();
            if let Some((head, _bytes)) = tx_queue.get_used() {
                if head == desc_head {
                    tx_queue.reclaim_chain(head);
                    break;
                }
            }
            drop(tx_queue);

            timeout -= 1;
            if timeout == 0 {
                unsafe {
                    Self::free_dma_buffer(hdr_phys, hdr_virt, NET_HDR_SIZE);
                    Self::free_dma_buffer(data_phys, data_virt, packet_data.len());
                }
                return Err("TX timeout");
            }

            core::hint::spin_loop();
        }

        // Free buffers
        unsafe {
            Self::free_dma_buffer(hdr_phys, hdr_virt, NET_HDR_SIZE);
            Self::free_dma_buffer(data_phys, data_virt, packet_data.len());
        }

        Ok(())
    }

    /// Check for received packets
    ///
    /// Returns number of packets received
    pub fn poll_rx(&self) -> Result<usize, &'static str> {
        let mut count = 0;

        loop {
            let (desc_head, bytes_written) = {
                let mut rx_queue = self.rx_queue.lock();
                if let Some(used) = rx_queue.get_used() {
                    used
                } else {
                    break;
                }
            };

            // Process received packet
            // Note: In real implementation, we need to track which buffer corresponds to which descriptor
            // For now, we'll skip actual packet processing and just reclaim the descriptor

            {
                let mut rx_queue = self.rx_queue.lock();
                rx_queue.reclaim_chain(desc_head);
            }

            count += 1;
        }

        // Refill RX queue
        if count > 0 {
            self.refill_rx_queue()?;
        }

        Ok(count)
    }

    /// Get MAC address
    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    /// Get MTU
    pub fn mtu(&self) -> u16 {
        self.mtu
    }

    /// Allocate DMA-capable buffer
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
}

// Helper functions for debug output
fn print_hex_u8(n: u8) {
    let hex_chars = b"0123456789abcdef";
    let buf = [
        hex_chars[(n >> 4) as usize],
        hex_chars[(n & 0xF) as usize],
    ];
    crate::arch::x86_64::serial::serial_write(&buf);
}

fn print_u16(n: u16) {
    print_u64(n as u64);
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_hdr_size() {
        assert_eq!(core::mem::size_of::<VirtioNetHdr>(), 14);
    }

    #[test]
    fn test_net_config_size() {
        assert_eq!(core::mem::size_of::<VirtioNetConfig>(), 12);
    }
}
