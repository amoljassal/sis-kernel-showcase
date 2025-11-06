/// virtio-net Driver (Phase C)
///
/// Implements VirtIO network device driver for packet transmission and reception.
/// Spec: VirtIO 1.0, Device ID 1

use crate::lib::error::{Result, Errno};
use crate::virtio::{VirtIOMMIOTransport, VirtIODeviceType, VirtIOMMIOOffset};
use crate::virtio::virtqueue::VirtQueue;
use alloc::sync::Arc;
use alloc::vec::{self, Vec};
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

        // Negotiate features
        let device_features = transport_ref.read_reg(VirtIOMMIOOffset::DeviceFeatures);
        crate::info!("virtio-net: device features = 0x{:08x}", device_features);

        // For now, we only require basic features
        let driver_features = 0; // No special features yet
        transport_ref.write_reg(VirtIOMMIOOffset::DriverFeatures, driver_features);

        // Read configuration space for MAC address
        let mac = Self::read_mac(&transport_ref);
        crate::info!("virtio-net: MAC address = {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                     mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

        let mtu = 1514; // Standard Ethernet MTU + header

        // Initialize virtqueues
        // RX queue (index 0)
        let mut rx_queue = VirtQueue::new(0, 256)?;
        transport_ref.setup_queue(&mut rx_queue)?;

        // TX queue (index 1)
        let mut tx_queue = VirtQueue::new(1, 256)?;
        transport_ref.setup_queue(&mut tx_queue)?;

        // Set device status to DRIVER_OK
        let status = transport_ref.read_reg(VirtIOMMIOOffset::Status);
        transport_ref.write_reg(VirtIOMMIOOffset::Status, status | 0x4); // DRIVER_OK

        let device = Self {
            transport: Arc::new(Mutex::new(transport_ref)),
            rx_queue: Arc::new(Mutex::new(rx_queue)),
            tx_queue: Arc::new(Mutex::new(tx_queue)),
            mac,
            mtu,
            rx_buffers: Mutex::new(VecDeque::new()),
        };

        // Pre-fill RX queue with buffers
        device.refill_rx_buffers()?;

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

        // Add 128 RX buffers
        for _ in 0..128 {
            // Allocate buffer (header + max packet size)
            let buffer_size = size_of::<VirtioNetHdr>() + self.mtu as usize;
            let buffer = vec![0u8; buffer_size];
            let buffer_addr = buffer.as_ptr() as u64;

            // Add to virtqueue as writable (device writes received packets here)
            let buffers = vec![(buffer_addr, buffer_size as u32, true)];
            rx_queue.add_buf(&buffers)?;

            // Store buffer for later retrieval
            self.rx_buffers.lock().push_back(buffer);
        }

        // Notify device that RX queue has buffers
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);

        Ok(())
    }

    /// Transmit a packet
    pub fn transmit(&self, packet: &[u8]) -> Result<()> {
        if packet.len() > self.mtu as usize {
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

        // Notify device
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 1);

        // Wait for completion (synchronous for now)
        loop {
            if let Some((completed_id, _len)) = tx_queue.get_used_buf() {
                if completed_id == desc_id {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Receive a packet (non-blocking)
    pub fn receive(&self) -> Option<Vec<u8>> {
        let mut rx_queue = self.rx_queue.lock();

        // Check for completed RX descriptors
        if let Some((desc_id, len)) = rx_queue.get_used_buf() {
            // Retrieve the buffer
            let mut rx_buffers = self.rx_buffers.lock();
            if let Some(mut buffer) = rx_buffers.pop_front() {
                // Extract packet data (skip virtio_net_hdr)
                let hdr_size = size_of::<VirtioNetHdr>();
                if len as usize > hdr_size {
                    let packet_len = len as usize - hdr_size;
                    let packet = buffer[hdr_size..hdr_size + packet_len].to_vec();

                    // Refill RX buffer
                    drop(rx_buffers);
                    drop(rx_queue);
                    let _ = self.refill_rx_buffers();

                    return Some(packet);
                }
            }
        }

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
