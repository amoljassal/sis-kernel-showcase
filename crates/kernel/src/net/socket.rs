/// Socket abstraction layer
///
/// Provides BSD sockets API on top of smoltcp

use crate::lib::error::Result;
use alloc::vec::Vec;
use smoltcp::socket::{tcp, udp, icmp};
use smoltcp::wire::{IpEndpoint, IpAddress};
use crate::net::smoltcp_iface::with_socket_set;

/// Socket handle (wraps smoltcp socket handle)
pub type SocketHandle = smoltcp::iface::SocketHandle;

/// Address family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFamily {
    Inet,   // AF_INET (IPv4)
    Inet6,  // AF_INET6 (IPv6)
}

/// Socket type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Stream,   // SOCK_STREAM (TCP)
    Dgram,    // SOCK_DGRAM (UDP)
    Raw,      // SOCK_RAW
}

/// Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,    // IPPROTO_TCP
    Udp,    // IPPROTO_UDP
    Icmp,   // IPPROTO_ICMP
    Raw,    // IPPROTO_RAW
}

/// Socket state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Unbound,
    Bound,
    Listening,
    Connecting,
    Connected,
    Closing,
    Closed,
}

/// Socket structure
pub struct Socket {
    pub domain: AddressFamily,
    pub sock_type: SocketType,
    pub protocol: Protocol,
    pub state: SocketState,
    pub smoltcp_handle: Option<SocketHandle>,
}

impl Socket {
    /// Create a new socket
    pub fn new(domain: AddressFamily, sock_type: SocketType, protocol: Protocol) -> Self {
        Self {
            domain,
            sock_type,
            protocol,
            state: SocketState::Unbound,
            smoltcp_handle: None,
        }
    }
}

/// Create and register a UDP socket with default buffers
pub fn udp_create() -> Result<SocketHandle> {
    // Allocate simple rx/tx buffers
    let rx_meta = alloc::vec![udp::PacketMetadata::EMPTY; 16];
    let rx_buffer = alloc::vec![0u8; 2048];
    let tx_meta = alloc::vec![udp::PacketMetadata::EMPTY; 16];
    let tx_buffer = alloc::vec![0u8; 2048];

    let udp_socket = udp::Socket::new(
        udp::PacketBuffer::new(rx_meta, rx_buffer),
        udp::PacketBuffer::new(tx_meta, tx_buffer),
    );

    with_socket_set(|sockets| sockets.add(udp_socket))
}

/// Bind UDP socket to endpoint
pub fn udp_bind(handle: SocketHandle, ip: [u8;4], port: u16) -> Result<()> {
    // Return true from closure if bind succeeded
    let ok = with_socket_set(|sockets| {
        let socket = sockets.get_mut::<udp::Socket>(handle);
        let endpoint = IpEndpoint::new(IpAddress::v4(ip[0], ip[1], ip[2], ip[3]), port);
        socket.bind(endpoint).is_ok()
    })?;
    if ok { Ok(()) } else { Err(crate::lib::error::Errno::EINVAL) }
}

/// Send a UDP packet to endpoint
pub fn udp_sendto(handle: SocketHandle, buf: &[u8], dst_ip: [u8;4], dst_port: u16) -> Result<usize> {
    // Try to send; if not ready, poll network and retry briefly
    for _ in 0..8 {
        let sent: bool = with_socket_set(|sockets| {
            let socket = sockets.get_mut::<udp::Socket>(handle);
            let endpoint = IpEndpoint::new(IpAddress::v4(dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3]), dst_port);
            socket.send_slice(buf, endpoint).is_ok()
        })?;
        if sent {
            let _ = crate::net::network_poll();
            return Ok(buf.len());
        }
        let _ = crate::net::network_poll();
    }
    Err(crate::lib::error::Errno::EAGAIN)
}

/// Receive a UDP packet; returns (len, src_ip, src_port)
pub fn udp_recvfrom(handle: SocketHandle, out: &mut [u8]) -> Result<(usize, [u8;4], u16)> {
    // Try a few times with polling
    for _ in 0..8 {
        let got: Option<(usize, smoltcp::socket::udp::UdpMetadata)> = with_socket_set(|sockets| {
            let socket = sockets.get_mut::<udp::Socket>(handle);
            if let Ok((data, endpoint)) = socket.recv() {
                let n = core::cmp::min(data.len(), out.len());
                out[..n].copy_from_slice(&data[..n]);
                return Some((n, endpoint));
            }
            None
        })?;
        if let Some((n, ep)) = got {
            let ip = match ep.endpoint.addr {
                IpAddress::Ipv4(v4) => v4.0,
                _ => [0,0,0,0],
            };
            return Ok((n, ip, ep.endpoint.port));
        }
        let _ = crate::net::network_poll();
    }
    Err(crate::lib::error::Errno::EAGAIN)
}
