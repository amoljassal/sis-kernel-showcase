//! Minimal SNTP client using smoltcp UDP

use crate::lib::error::{Result, Errno};
use alloc::vec;
use smoltcp::wire::{IpEndpoint, Ipv4Address};
use crate::net::smoltcp_iface::with_socket_set;

const NTP_PORT: u16 = 123;

/// Query an SNTP server and return the transmit timestamp seconds if available
pub fn sntp_query(server_ip: [u8; 4]) -> Result<u64> {
    // Create UDP socket
    let handle = with_socket_set(|sockets| {
        let udp = smoltcp::socket::udp::Socket::new(
            smoltcp::socket::udp::PacketBuffer::new(vec![smoltcp::socket::udp::PacketMetadata::EMPTY; 4], vec![0; 576]),
            smoltcp::socket::udp::PacketBuffer::new(vec![smoltcp::socket::udp::PacketMetadata::EMPTY; 4], vec![0; 576])
        );
        sockets.add(udp)
    })?;

    // Bind ephemeral port
    with_socket_set(|sockets| {
        let socket = sockets.get_mut::<smoltcp::socket::udp::Socket>(handle);
        socket.bind(IpEndpoint::new(smoltcp::wire::IpAddress::v4(0,0,0,0), 0)).ok();
    })?;

    // Build minimal SNTP request (48 bytes, LI=0 VN=4 Mode=3)
    let mut req = [0u8; 48];
    req[0] = 0b00_100_011; // LI=0, VN=4, Mode=3 (client)

    let dest = IpEndpoint::new(smoltcp::wire::IpAddress::Ipv4(Ipv4Address::from_bytes(&server_ip)), NTP_PORT);

    // Send request
    with_socket_set(|sockets| {
        let socket = sockets.get_mut::<smoltcp::socket::udp::Socket>(handle);
        socket.send_slice(&req, dest).map(|_| ()).map_err(|_| Errno::EIO)
    })?;

    // Poll network and wait for a reply up to a bounded number of iterations
    let mut spins = 0usize;
    let mut buf = [0u8; 96];
    loop {
        crate::net::network_poll()?;
        let got = with_socket_set(|sockets| {
            let socket = sockets.get_mut::<smoltcp::socket::udp::Socket>(handle);
            if let Ok((n, _ep)) = socket.recv_slice(&mut buf) {
                Some(n)
            } else {
                None
            }
        })?;
        if let Some(n) = got {
            if n >= 48 {
                // Transmit timestamp seconds is at bytes 40..44
                let secs = u32::from_be_bytes([buf[40], buf[41], buf[42], buf[43]]) as u64;
                // NTP epoch (1900) to Unix epoch (1970) offset
                const NTP_UNIX_DELTA: u64 = 2_208_988_800;
                if secs >= NTP_UNIX_DELTA {
                    return Ok(secs - NTP_UNIX_DELTA);
                } else {
                    return Err(Errno::EIO);
                }
            } else {
                return Err(Errno::EIO);
            }
        }
        spins = spins.wrapping_add(1);
        if spins > 2_000_000 { return Err(Errno::ETIMEDOUT); }
        core::hint::spin_loop();
    }
}
