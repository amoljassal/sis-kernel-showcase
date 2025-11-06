/// Socket abstraction layer
///
/// Provides BSD sockets API on top of smoltcp

use crate::lib::error::{Result, Errno};
use alloc::vec::Vec;
use smoltcp::socket::{tcp, udp, icmp};
use smoltcp::wire::{IpEndpoint, IpAddress};

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

// Socket operations will be implemented with syscalls
