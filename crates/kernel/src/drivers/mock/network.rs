// Mock Network Device
// Phase 6 - Production Readiness Plan

use crate::drivers::traits::{NetworkDevice, NetworkStats};
use crate::lib::error::{Errno, Result};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};

/// Mock network device for testing
pub struct MockNetworkDevice {
    name: String,
    mac_addr: [u8; 6],
    mtu: usize,
    link_up: AtomicBool,

    // Packet queues (simplified - real impl would use proper sync)
    tx_queue: alloc::sync::Arc<spin::Mutex<VecDeque<Vec<u8>>>>,
    rx_queue: alloc::sync::Arc<spin::Mutex<VecDeque<Vec<u8>>>>,

    // Chaos/failure injection
    packet_loss_rate: AtomicU32,  // Packet loss rate 0-100%
    delay_micros: AtomicU32,       // Artificial delay

    // Statistics
    stats: NetworkStatsAtomic,
}

struct NetworkStatsAtomic {
    rx_packets: AtomicU64,
    tx_packets: AtomicU64,
    rx_bytes: AtomicU64,
    tx_bytes: AtomicU64,
    rx_errors: AtomicU64,
    tx_errors: AtomicU64,
    rx_dropped: AtomicU64,
    tx_dropped: AtomicU64,
}

impl MockNetworkDevice {
    /// Create a new mock network device
    pub fn new(name: &str, mac_addr: [u8; 6]) -> Self {
        Self {
            name: String::from(name),
            mac_addr,
            mtu: 1500,
            link_up: AtomicBool::new(true),
            tx_queue: alloc::sync::Arc::new(spin::Mutex::new(VecDeque::new())),
            rx_queue: alloc::sync::Arc::new(spin::Mutex::new(VecDeque::new())),
            packet_loss_rate: AtomicU32::new(0),
            delay_micros: AtomicU32::new(0),
            stats: NetworkStatsAtomic {
                rx_packets: AtomicU64::new(0),
                tx_packets: AtomicU64::new(0),
                rx_bytes: AtomicU64::new(0),
                tx_bytes: AtomicU64::new(0),
                rx_errors: AtomicU64::new(0),
                tx_errors: AtomicU64::new(0),
                rx_dropped: AtomicU64::new(0),
                tx_dropped: AtomicU64::new(0),
            },
        }
    }

    /// Set link state
    pub fn set_link_up(&self, up: bool) {
        self.link_up.store(up, Ordering::Relaxed);
    }

    /// Set packet loss rate (0-100%)
    pub fn set_packet_loss_rate(&self, rate: u32) {
        self.packet_loss_rate.store(rate.min(100), Ordering::Relaxed);
    }

    /// Set artificial delay in microseconds
    pub fn set_delay(&self, micros: u32) {
        self.delay_micros.store(micros, Ordering::Relaxed);
    }

    /// Inject a packet into receive queue (for testing)
    pub fn inject_packet(&self, packet: Vec<u8>) {
        let mut rx_queue = self.rx_queue.lock();
        rx_queue.push_back(packet);
    }

    /// Get transmitted packets (for testing)
    pub fn get_tx_packets(&self) -> Vec<Vec<u8>> {
        let mut tx_queue = self.tx_queue.lock();
        tx_queue.drain(..).collect()
    }

    /// Check if packet should be dropped (based on loss rate)
    fn should_drop_packet(&self) -> bool {
        let rate = self.packet_loss_rate.load(Ordering::Relaxed);
        if rate == 0 {
            return false;
        }

        // Simple PRNG
        static SEED: AtomicU64 = AtomicU64::new(0x123456789abcdef0);
        let mut seed = SEED.load(Ordering::Relaxed);
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        SEED.store(seed, Ordering::Relaxed);

        (seed % 100) < rate as u64
    }

    /// Simulate network delay
    fn simulate_delay(&self) {
        let delay = self.delay_micros.load(Ordering::Relaxed);
        if delay > 0 {
            for _ in 0..delay {
                core::hint::spin_loop();
            }
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.stats.rx_packets.store(0, Ordering::Relaxed);
        self.stats.tx_packets.store(0, Ordering::Relaxed);
        self.stats.rx_bytes.store(0, Ordering::Relaxed);
        self.stats.tx_bytes.store(0, Ordering::Relaxed);
        self.stats.rx_errors.store(0, Ordering::Relaxed);
        self.stats.tx_errors.store(0, Ordering::Relaxed);
        self.stats.rx_dropped.store(0, Ordering::Relaxed);
        self.stats.tx_dropped.store(0, Ordering::Relaxed);
    }
}

impl NetworkDevice for MockNetworkDevice {
    fn send(&self, packet: &[u8]) -> Result<()> {
        // Check link state
        if !self.link_up.load(Ordering::Relaxed) {
            self.stats.tx_errors.fetch_add(1, Ordering::Relaxed);
            return Err(Errno::ENETDOWN);
        }

        // Validate packet size
        if packet.len() > self.mtu {
            self.stats.tx_errors.fetch_add(1, Ordering::Relaxed);
            return Err(Errno::EMSGSIZE);
        }

        if packet.is_empty() {
            self.stats.tx_errors.fetch_add(1, Ordering::Relaxed);
            return Err(Errno::EINVAL);
        }

        // Simulate delay
        self.simulate_delay();

        // Check for packet loss
        if self.should_drop_packet() {
            self.stats.tx_dropped.fetch_add(1, Ordering::Relaxed);
            return Ok(()); // Silently drop (as real network would)
        }

        // Add to TX queue
        let mut tx_queue = self.tx_queue.lock();
        tx_queue.push_back(packet.to_vec());

        // Update statistics
        self.stats.tx_packets.fetch_add(1, Ordering::Relaxed);
        self.stats.tx_bytes.fetch_add(packet.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        // Check link state
        if !self.link_up.load(Ordering::Relaxed) {
            return Err(Errno::ENETDOWN);
        }

        // Simulate delay
        self.simulate_delay();

        // Get packet from RX queue
        let mut rx_queue = self.rx_queue.lock();
        if let Some(packet) = rx_queue.pop_front() {
            let len = packet.len().min(buf.len());
            buf[..len].copy_from_slice(&packet[..len]);

            // Update statistics
            self.stats.rx_packets.fetch_add(1, Ordering::Relaxed);
            self.stats.rx_bytes.fetch_add(len as u64, Ordering::Relaxed);

            Ok(len)
        } else {
            Err(Errno::EAGAIN) // No packet available (would block)
        }
    }

    fn mac_address(&self) -> [u8; 6] {
        self.mac_addr
    }

    fn mtu(&self) -> usize {
        self.mtu
    }

    fn link_up(&self) -> bool {
        self.link_up.load(Ordering::Relaxed)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn stats(&self) -> NetworkStats {
        NetworkStats {
            rx_packets: self.stats.rx_packets.load(Ordering::Relaxed),
            tx_packets: self.stats.tx_packets.load(Ordering::Relaxed),
            rx_bytes: self.stats.rx_bytes.load(Ordering::Relaxed),
            tx_bytes: self.stats.tx_bytes.load(Ordering::Relaxed),
            rx_errors: self.stats.rx_errors.load(Ordering::Relaxed),
            tx_errors: self.stats.tx_errors.load(Ordering::Relaxed),
            rx_dropped: self.stats.rx_dropped.load(Ordering::Relaxed),
            tx_dropped: self.stats.tx_dropped.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_network_device_creation() {
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let dev = MockNetworkDevice::new("eth0", mac);

        assert_eq!(dev.mac_address(), mac);
        assert_eq!(dev.mtu(), 1500);
        assert!(dev.link_up());
    }

    #[test]
    fn test_mock_network_send_recv() {
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let dev = MockNetworkDevice::new("eth0", mac);

        // Send a packet
        let packet = vec![0x01, 0x02, 0x03, 0x04];
        dev.send(&packet).unwrap();

        // Get transmitted packets
        let tx_packets = dev.get_tx_packets();
        assert_eq!(tx_packets.len(), 1);
        assert_eq!(tx_packets[0], packet);
    }

    #[test]
    fn test_mock_network_link_down() {
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let dev = MockNetworkDevice::new("eth0", mac);

        // Set link down
        dev.set_link_up(false);
        assert!(!dev.link_up());

        // Try to send - should fail
        let packet = vec![0x01, 0x02, 0x03, 0x04];
        let result = dev.send(&packet);
        assert_eq!(result, Err(Errno::ENETDOWN));
    }

    #[test]
    fn test_mock_network_packet_loss() {
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let dev = MockNetworkDevice::new("eth0", mac);

        // Set 100% packet loss
        dev.set_packet_loss_rate(100);

        // Send packets - they should be dropped
        for _ in 0..10 {
            let packet = vec![0x01, 0x02, 0x03, 0x04];
            dev.send(&packet).unwrap(); // Returns Ok but packet is dropped
        }

        // No packets should be in TX queue
        let tx_packets = dev.get_tx_packets();
        assert_eq!(tx_packets.len(), 0);

        // Check statistics
        let stats = dev.stats();
        assert_eq!(stats.tx_dropped, 10);
    }
}
