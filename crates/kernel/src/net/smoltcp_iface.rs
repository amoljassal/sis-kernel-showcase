/// smoltcp Network Interface Integration
///
/// Provides PHY adapter between virtio-net driver and smoltcp TCP/IP stack

use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken, Medium};
use smoltcp::time::Instant;
use smoltcp::iface::{Interface, Config, SocketSet};
use smoltcp::wire::{HardwareAddress, EthernetAddress, IpCidr, IpAddress, Ipv4Address};
use crate::drivers::virtio_net;
use crate::lib::error::{Result, Errno};
use alloc::vec;
use alloc::vec::Vec;
use spin::Mutex;
use alloc::sync::Arc;

/// VirtIO-net PHY adapter for smoltcp
pub struct VirtioNetPhy;

/// RX token for receiving packets
pub struct VirtioRxToken {
    packet: Vec<u8>,
}

/// TX token for transmitting packets
pub struct VirtioTxToken;

impl Device for VirtioNetPhy {
    type RxToken<'a> = VirtioRxToken where Self: 'a;
    type TxToken<'a> = VirtioTxToken where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // Try to receive a packet from virtio-net
        if let Some(packet) = virtio_net::receive() {
            Some((
                VirtioRxToken { packet },
                VirtioTxToken,
            ))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        // Always allow transmit (virtio-net handles queuing)
        Some(VirtioTxToken)
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;  // Ethernet MTU + header
        caps.max_burst_size = Some(256);
        caps.medium = Medium::Ethernet;
        caps
    }
}

impl RxToken for VirtioRxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.packet)
    }
}

impl TxToken for VirtioTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0u8; len];
        let result = f(&mut buffer);

        // Transmit packet via virtio-net
        if let Err(e) = virtio_net::transmit(&buffer) {
            crate::warn!("net: TX failed: {:?}", e);
        }

        result
    }
}

/// Global network interface
static NETWORK_INTERFACE: Mutex<Option<Interface>> = Mutex::new(None);
static SOCKET_SET: Mutex<Option<SocketSet<'static>>> = Mutex::new(None);

/// Initialize network interface
pub fn init_network() -> Result<()> {
    // Get MAC address from virtio-net device
    let mac = virtio_net::mac_address()?;
    let hw_addr = HardwareAddress::Ethernet(EthernetAddress(mac));

    crate::info!("net: Initializing interface with MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                 mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

    // Create interface config
    let config = Config::new(hw_addr);

    // Create interface
    let mut iface = Interface::new(config, &mut VirtioNetPhy, Instant::from_millis(0));

    // Start with no IP address (will be configured by DHCP)
    iface.update_ip_addrs(|addrs| {
        addrs.push(IpCidr::new(IpAddress::v4(0, 0, 0, 0), 0)).ok();
    });

    // Create socket set
    let socket_set = SocketSet::new(vec![]);

    *NETWORK_INTERFACE.lock() = Some(iface);
    *SOCKET_SET.lock() = Some(socket_set);

    crate::info!("net: Interface initialized (no IP yet, use DHCP)");

    Ok(())
}

/// Get reference to network interface
pub fn get_interface() -> Result<()> {
    let iface = NETWORK_INTERFACE.lock();
    if iface.is_none() {
        return Err(Errno::ENODEV);
    }
    Ok(())
}

/// Poll network interface (should be called periodically)
pub fn network_poll() -> Result<usize> {
    let mut iface_lock = NETWORK_INTERFACE.lock();
    let mut socket_lock = SOCKET_SET.lock();

    match (iface_lock.as_mut(), socket_lock.as_mut()) {
        (Some(iface), Some(sockets)) => {
            // Current time in milliseconds (use uptime)
            let ts_ms = crate::time::get_uptime_ms() as i64;
            let timestamp = Instant::from_millis(ts_ms);
            let mut device = VirtioNetPhy;

            let processed = iface.poll(timestamp, &mut device, sockets);
            Ok(if processed { 1 } else { 0 })
        }
        _ => Err(Errno::ENODEV),
    }
}

/// Set IP address manually (for testing before DHCP)
pub fn set_ip_address(ip: [u8; 4], prefix_len: u8) -> Result<()> {
    let mut iface_lock = NETWORK_INTERFACE.lock();

    if let Some(iface) = iface_lock.as_mut() {
        let ip_addr = IpAddress::v4(ip[0], ip[1], ip[2], ip[3]);
        let cidr = IpCidr::new(ip_addr, prefix_len);

        iface.update_ip_addrs(|addrs| {
            addrs.clear();
            addrs.push(cidr).ok();
        });

        crate::info!("net: IP address set to {}.{}.{}.{}/{}",
                     ip[0], ip[1], ip[2], ip[3], prefix_len);
        Ok(())
    } else {
        Err(Errno::ENODEV)
    }
}

/// Set default gateway
pub fn set_gateway(gw: [u8; 4]) -> Result<()> {
    let mut iface_lock = NETWORK_INTERFACE.lock();

    if let Some(iface) = iface_lock.as_mut() {
        let gw_addr = Ipv4Address::new(gw[0], gw[1], gw[2], gw[3]);

        iface.routes_mut().add_default_ipv4_route(gw_addr).map_err(|_| Errno::EINVAL)?;

        crate::info!("net: Default gateway set to {}.{}.{}.{}",
                     gw[0], gw[1], gw[2], gw[3]);
        Ok(())
    } else {
        Err(Errno::ENODEV)
    }
}

/// Get current IP address
pub fn get_ip_address() -> Result<[u8; 4]> {
    let iface_lock = NETWORK_INTERFACE.lock();

    if let Some(iface) = iface_lock.as_ref() {
        for addr in iface.ip_addrs() {
            let IpAddress::Ipv4(ipv4) = addr.address() else { continue };
            let octets = ipv4.as_bytes();
            return Ok([octets[0], octets[1], octets[2], octets[3]]);
        }
        Err(Errno::EADDRNOTAVAIL)
    } else {
        Err(Errno::ENODEV)
    }
}

/// Get socket set for socket operations
pub fn with_socket_set<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut SocketSet) -> R,
{
    let mut socket_lock = SOCKET_SET.lock();

    if let Some(sockets) = socket_lock.as_mut() {
        Ok(f(sockets))
    } else {
        Err(Errno::ENODEV)
    }
}

/// Access interface and socket set together
pub fn with_interface_and_sockets<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut Interface, &mut SocketSet) -> R,
{
    let mut iface_lock = NETWORK_INTERFACE.lock();
    let mut socket_lock = SOCKET_SET.lock();

    match (iface_lock.as_mut(), socket_lock.as_mut()) {
        (Some(iface), Some(sockets)) => Ok(f(iface, sockets)),
        _ => Err(Errno::ENODEV),
    }
}
