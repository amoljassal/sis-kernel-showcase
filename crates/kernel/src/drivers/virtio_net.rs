/// virtio-net Driver (Phase C)
///
/// Implements VirtIO network device driver for packet transmission and reception.
/// Spec: VirtIO 1.0, Device ID 1

use crate::lib::error::{Result, Errno};
use crate::virtio::{VirtIOMMIOTransport, VirtIODeviceType, VirtIOMMIOOffset};
use crate::virtio::virtqueue::VirtQueue;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::VecDeque;
use spin::Mutex;
use core::mem::size_of;

/// VirtIO-net device configuration space
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioNetConfig {
    mac: [u8; 6],
    status: u16,
    max_virtqueue_pairs: u16,
    mtu: u16,
}

/// VirtIO-net packet header (prepended to all packets)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioNetHdr {
    flags: u8,
    gso_type: u8,
    hdr_len: u16,
    gso_size: u16,
    csum_start: u16,
    csum_offset: u16,
    num_buffers: u16,
}

impl Default for VirtioNetHdr {
    fn default() -> Self {
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

/// VirtIO-net device
pub struct VirtioNetDevice {
    transport: Arc<Mutex<VirtIOMMIOTransport>>,
    rx_queue: Arc<Mutex<VirtQueue>>,
    tx_queue: Arc<Mutex<VirtQueue>>,
    mac: [u8; 6],
    mtu: u16,
    rx_buffers: Mutex<VecDeque<Vec<u8>>>,
}

impl VirtioNetDevice {
    /// Create and initialize a new virtio-net device
    pub fn new(transport: VirtIOMMIOTransport, name: String) -> Result<Self> {
        let mut transport_ref = transport;

        // Verify device type
        if transport_ref.device_type() != VirtIODeviceType::NetworkCard {
            return Err(Errno::EINVAL);
        }

        // Check VirtIO MMIO version
        let mmio_version = transport_ref.read_reg(VirtIOMMIOOffset::Version);
        let is_legacy = mmio_version == 1;
        crate::warn!("virtio-net: MMIO version = {} ({})", mmio_version,
                     if is_legacy { "legacy" } else { "modern" });

        // Log device features for visibility
        let device_features_low = transport_ref.read_reg(VirtIOMMIOOffset::DeviceFeatures);
        crate::warn!("virtio-net: device features[0:31] = 0x{:08x}", device_features_low);

        // Enable required features:
        // - VIRTIO_NET_F_MAC (bit 5) - Device has MAC address
        // - VIRTIO_NET_F_MRG_RXBUF (bit 15) - Merged RX buffers (try enabling for RX)
        // - VIRTIO_NET_F_STATUS (bit 16) - Device has status field
        // Note: Legacy (v1) devices don't support VIRTIO_F_VERSION_1
        let driver_features: u32 = (1 << 5) | (1 << 15) | (1 << 16);  // MAC + MRG_RXBUF + STATUS
        crate::warn!("virtio-net: requesting features = 0x{:08x}", driver_features);

        // For modern devices (v2+), also enable VIRTIO_F_VERSION_1
        if !is_legacy {
            transport_ref.write_reg(VirtIOMMIOOffset::DriverFeaturesSel, 1);
            transport_ref.write_reg(VirtIOMMIOOffset::DriverFeatures, 1);  // VIRTIO_F_VERSION_1
            transport_ref.write_reg(VirtIOMMIOOffset::DriverFeaturesSel, 0);
        }

        transport_ref
            .init_device(driver_features)
            .map_err(|e| {
                crate::warn!("virtio-net: init_device failed: {:?}", e);
                Errno::EIO
            })?;

        let status_after = transport_ref.read_reg(VirtIOMMIOOffset::Status);
        crate::warn!("virtio-net: status after init = 0x{:x}", status_after);

        // Read configuration space for MAC address
        let mac = Self::read_mac(&transport_ref);
        crate::info!("virtio-net: MAC address = {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                     mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

        let mtu = 1514; // Standard Ethernet MTU + header

        // Initialize virtqueues (use contiguous allocation for legacy devices)
        // RX queue (index 0)
        let mut rx_queue = if is_legacy {
            VirtQueue::new_contiguous(0, 256)?
        } else {
            VirtQueue::new(0, 256)?
        };
        let (rx_desc, rx_avail, rx_used) = rx_queue.get_addresses();
        crate::warn!("virtio-net: RX queue addrs: desc=0x{:x} avail=0x{:x} used=0x{:x}", rx_desc, rx_avail, rx_used);
        if is_legacy {
            transport_ref.setup_queue_legacy(&rx_queue).map_err(|_| Errno::EIO)?;
        } else {
            transport_ref.setup_queue(&mut rx_queue).map_err(|_| Errno::EIO)?;
        }

        // TX queue (index 1)
        let mut tx_queue = if is_legacy {
            VirtQueue::new_contiguous(1, 256)?
        } else {
            VirtQueue::new(1, 256)?
        };
        let (tx_desc, tx_avail, tx_used) = tx_queue.get_addresses();
        crate::warn!("virtio-net: TX queue addrs: desc=0x{:x} avail=0x{:x} used=0x{:x}", tx_desc, tx_avail, tx_used);
        if is_legacy {
            transport_ref.setup_queue_legacy(&tx_queue).map_err(|_| Errno::EIO)?;
        } else {
            transport_ref.setup_queue(&mut tx_queue).map_err(|_| Errno::EIO)?;
        }

        // Create device structure (but don't set DRIVER_OK yet)
        let device = Self {
            transport: Arc::new(Mutex::new(transport_ref)),
            rx_queue: Arc::new(Mutex::new(rx_queue)),
            tx_queue: Arc::new(Mutex::new(tx_queue)),
            mac,
            mtu,
            rx_buffers: Mutex::new(VecDeque::new()),
        };

        // Pre-fill RX queue with buffers BEFORE setting DRIVER_OK
        device.refill_rx_buffers()?;

        // Now set device status to DRIVER_OK
        device.transport.lock().driver_ready();

        crate::info!("virtio-net: initialized device {}", name);
        Ok(device)
    }

    /// Read MAC address from device configuration space
    fn read_mac(transport: &VirtIOMMIOTransport) -> [u8; 6] {
        let mut mac = [0u8; 6];

        // Read MAC address from config space (offset 0)
        for i in 0..6 {
            mac[i] = transport.read_config_u8(i);
        }

        mac
    }

    /// Pre-fill RX queue with empty buffers
    fn refill_rx_buffers(&self) -> Result<()> {
        let mut rx_queue = self.rx_queue.lock();
        let transport = self.transport.lock();

        use core::sync::atomic::{AtomicUsize, Ordering};
        static REFILL_COUNT: AtomicUsize = AtomicUsize::new(0);
        let count = REFILL_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        if count <= 2 {
            crate::warn!("RX refill #{}: adding 128 buffers", count);
        }

        // Add 128 RX buffers
        let mut added = 0;
        for _ in 0..128 {
            // Allocate buffer (header + max packet size)
            let buffer_size = size_of::<VirtioNetHdr>() + self.mtu as usize;
            let buffer = vec![0u8; buffer_size];
            let buffer_addr = buffer.as_ptr() as u64;

            // Add to virtqueue as writable (device writes received packets here)
            let buffers = vec![(buffer_addr, buffer_size as u32, true)];
            if let Ok(_desc_id) = rx_queue.add_buf(&buffers) {
                // Store buffer for later retrieval
                self.rx_buffers.lock().push_back(buffer);
                added += 1;
            } else {
                break; // Queue full
            }
        }

        if count <= 2 {
            crate::warn!("RX refill #{}: added {} buffers, notifying queue 0", count, added);
            // Check available ring index
            unsafe {
                let avail_idx_ptr = (rx_queue.avail_ring_addr() + 2) as *const u16;
                let avail_idx = core::ptr::read_volatile(avail_idx_ptr);
                crate::warn!("RX queue: avail_idx={} (should be {})", avail_idx, added);
            }
        }

        // Notify device that RX queue has buffers
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);

        Ok(())
    }

    /// Transmit a packet
    pub fn transmit(&self, packet: &[u8]) -> Result<()> {
        if packet.len() > self.mtu as usize {
            crate::warn!("TX: packet too large {} > {}", packet.len(), self.mtu);
            return Err(Errno::EMSGSIZE);
        }

        let mut tx_queue = self.tx_queue.lock();
        let transport = self.transport.lock();

        // Allocate TX buffer (header + packet data)
        let header = VirtioNetHdr::default();
        let header_bytes = unsafe {
            core::slice::from_raw_parts(
                &header as *const VirtioNetHdr as *const u8,
                size_of::<VirtioNetHdr>()
            )
        };

        let mut tx_buffer = Vec::with_capacity(size_of::<VirtioNetHdr>() + packet.len());
        tx_buffer.extend_from_slice(header_bytes);
        tx_buffer.extend_from_slice(packet);

        let buffer_addr = tx_buffer.as_ptr() as u64;
        let buffer_len = tx_buffer.len() as u32;

        // Add to TX queue (readable by device)
        let buffers = vec![(buffer_addr, buffer_len, false)];
        let desc_id = tx_queue.add_buf(&buffers)?;

        // Check device status before notify
        let dev_status = transport.read_reg(VirtIOMMIOOffset::Status);

        // Notify device
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 1);

        // Log packet details (first 3 packets for debugging)
        use core::sync::atomic::{AtomicUsize, Ordering};
        static TX_COUNT: AtomicUsize = AtomicUsize::new(0);
        let tx_count = TX_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        if tx_count <= 3 {
            crate::warn!("TX #{}: addr=0x{:x} len={} desc={} dev_status=0x{:x}",
                       tx_count, buffer_addr, buffer_len, desc_id, dev_status);
        }

        // Wait for completion (bounded spin to avoid deadlock if device not consuming)
        let mut spins: usize = 0;
        let mut last_int_status = 0;
        let mut checked_used = false;
        loop {
            // Check and acknowledge any pending interrupts
            let int_status = transport.read_reg(VirtIOMMIOOffset::InterruptStatus);
            if int_status != 0 && int_status != last_int_status {
                crate::warn!("TX: INTERRUPT! status=0x{:x} at spin={}", int_status, spins);
                transport.write_reg(VirtIOMMIOOffset::InterruptACK, int_status);
                last_int_status = int_status;
            }

            if let Some((completed_id, _len)) = tx_queue.get_used_buf() {
                if completed_id == desc_id {
                    crate::warn!("TX: SUCCESS after {} spins", spins);
                    break;
                } else if !checked_used {
                    crate::warn!("TX: got used buf {} but waiting for {}", completed_id, desc_id);
                    checked_used = true;
                }
            }
            core::hint::spin_loop();
            spins = spins.wrapping_add(1);
            if spins > 5_000_000 {
                crate::warn!("TX: TIMEOUT after {} spins, int_status=0x{:x}, dev_status=0x{:x}",
                          spins, last_int_status, transport.read_reg(VirtIOMMIOOffset::Status));
                return Err(Errno::ETIMEDOUT);
            }
        }

        Ok(())
    }

    /// Receive a packet (non-blocking)
    pub fn receive(&self) -> Option<Vec<u8>> {
        let transport = self.transport.lock();
        let mut rx_queue = self.rx_queue.lock();

        // Check for completed RX descriptors
        use core::sync::atomic::{AtomicUsize, Ordering};
        static RX_CHECKS: AtomicUsize = AtomicUsize::new(0);
        let rx_check_count = RX_CHECKS.fetch_add(1, Ordering::Relaxed) + 1;
        if rx_check_count <= 10 {
            // Check interrupt status
            let int_status = transport.read_reg(VirtIOMMIOOffset::InterruptStatus);
            crate::warn!("RX check #{}: has_used={} int_status=0x{:x}",
                       rx_check_count, rx_queue.has_used_buf(), int_status);
            // Acknowledge any pending interrupts
            if int_status != 0 {
                transport.write_reg(VirtIOMMIOOffset::InterruptACK, int_status);
                crate::warn!("RX: ACKed interrupt 0x{:x}", int_status);
            }
        }

        if let Some((desc_id, len)) = rx_queue.get_used_buf() {
            crate::warn!("RX: got packet desc={} len={}", desc_id, len);
            // Retrieve the buffer
            let mut rx_buffers = self.rx_buffers.lock();
            if let Some(mut buffer) = rx_buffers.pop_front() {
                // Extract packet data (skip virtio_net_hdr)
                let hdr_size = size_of::<VirtioNetHdr>();
                if len as usize > hdr_size {
                    let packet_len = len as usize - hdr_size;
                    let packet = buffer[hdr_size..hdr_size + packet_len].to_vec();

                    // Refill RX buffer (must drop all locks first!)
                    drop(rx_buffers);
                    drop(rx_queue);
                    drop(transport);  // Drop transport lock to avoid deadlock
                    let _ = self.refill_rx_buffers();

                    return Some(packet);
                }
            }
        }

        // Drop locks explicitly
        drop(rx_queue);
        drop(transport);

        None
    }

    /// Get MAC address
    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    /// Get MTU
    pub fn mtu(&self) -> u16 {
        self.mtu
    }
}

/// Global virtio-net device (for now, only one)
static VIRTIO_NET_DEVICE: Mutex<Option<Arc<VirtioNetDevice>>> = Mutex::new(None);

/// Register a virtio-net device
pub fn register_virtio_net(transport: VirtIOMMIOTransport, name: String) -> Result<Arc<VirtioNetDevice>> {
    let device = Arc::new(VirtioNetDevice::new(transport, name.clone())?);

    *VIRTIO_NET_DEVICE.lock() = Some(device.clone());

    crate::info!("virtio-net: registered device {}", name);
    Ok(device)
}

/// Get the global virtio-net device
pub fn get_virtio_net_device() -> Option<Arc<VirtioNetDevice>> {
    VIRTIO_NET_DEVICE.lock().clone()
}

/// Transmit a packet on the global device
pub fn transmit(packet: &[u8]) -> Result<()> {
    let device = get_virtio_net_device().ok_or(Errno::ENODEV)?;
    device.transmit(packet)
}

/// Receive a packet from the global device
pub fn receive() -> Option<Vec<u8>> {
    let device = get_virtio_net_device()?;
    device.receive()
}

/// Get MAC address of the global device
pub fn mac_address() -> Result<[u8; 6]> {
    let device = get_virtio_net_device().ok_or(Errno::ENODEV)?;
    Ok(device.mac_address())
}
