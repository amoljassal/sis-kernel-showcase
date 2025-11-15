/// DHCP Client Implementation
///
/// Implements DHCP DORA (Discover, Offer, Request, Acknowledge) process

use crate::lib::error::{Result, Errno};
use smoltcp::socket::dhcpv4;
use smoltcp::time::Instant;
use crate::net::smoltcp_iface::{with_interface_and_sockets, with_socket_set};

/// DHCP configuration result
#[derive(Debug, Clone)]
pub struct DhcpConfig {
    pub ip_addr: [u8; 4],
    pub subnet_mask: u8,
    pub gateway: Option<[u8; 4]>,
    pub dns_servers: alloc::vec::Vec<[u8; 4]>,
    pub lease_time: u32,
}

/// DHCP client state
pub struct DhcpClient {
    socket_handle: Option<smoltcp::iface::SocketHandle>,
}

impl DhcpClient {
    /// Create a new DHCP client
    pub fn new() -> Self {
        Self {
            socket_handle: None,
        }
    }

    /// Start DHCP and acquire IP address with retry/backoff
    pub fn acquire_lease(&mut self) -> Result<DhcpConfig> {
        crate::info!("dhcp: Starting DHCP discovery...");

        // Create DHCP socket
        let dhcp_socket = dhcpv4::Socket::new();

        // Add socket to socket set
        let handle = with_socket_set(|sockets| {
            sockets.add(dhcp_socket)
        })?;

        self.socket_handle = Some(handle);

        // Poll until we get an IP address with exponential backoff between bursts
        let mut attempts: usize = 0;
        let mut backoff_iters: usize = 1_000; // initial small delay
        const MAX_ATTEMPTS: usize = 400; // ~4x previous
        const MAX_BACKOFF_ITERS: usize = 200_000; // clamp backoff

        loop {
            // Poll network
            crate::net::network_poll()?;

            // Check DHCP state
            let config = with_socket_set(|sockets| {
                let socket = sockets.get_mut::<dhcpv4::Socket>(handle);

                match socket.poll() {
                    Some(dhcpv4::Event::Configured(config)) => {
                        crate::info!("dhcp: Lease acquired!");
                        crate::info!("dhcp: IP address: {:?}", config.address);
                        if let Some(gw) = config.router {
                            crate::info!("dhcp: Gateway: {:?}", gw);
                        }
                        if !config.dns_servers.is_empty() {
                            crate::info!("dhcp: DNS servers: {:?}", config.dns_servers);
                        }

                        let addr = config.address.address();
                        let ip_octets = addr.as_bytes();
                        let ip_addr = [ip_octets[0], ip_octets[1], ip_octets[2], ip_octets[3]];
                        let subnet_mask = config.address.prefix_len();

                        let gateway = config.router.map(|gw| {
                            let gw_octets = gw.as_bytes();
                            [gw_octets[0], gw_octets[1], gw_octets[2], gw_octets[3]]
                        });

                        let dns_servers = config.dns_servers.iter().map(|dns| {
                            let octets = dns.as_bytes();
                            [octets[0], octets[1], octets[2], octets[3]]
                        }).collect();

                        Some(DhcpConfig {
                            ip_addr,
                            subnet_mask,
                            gateway,
                            dns_servers,
                            lease_time: 3600, // Default 1 hour
                        })
                    }
                    Some(dhcpv4::Event::Deconfigured) => {
                        crate::warn!("dhcp: Lease lost");
                        None
                    }
                    None => None,
                }
            })?;

            if let Some(config) = config {
                return Ok(config);
            }

            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                crate::warn!("dhcp: Timeout waiting for lease");
                return Err(Errno::ETIMEDOUT);
            }

            // Exponential backoff delay between polls
            for _ in 0..backoff_iters { core::hint::spin_loop(); }
            backoff_iters = (backoff_iters.saturating_mul(2)).min(MAX_BACKOFF_ITERS);
        }
    }

    /// Apply DHCP configuration to interface
    pub fn apply_config(&self, config: &DhcpConfig) -> Result<()> {
        // Set IP address
        crate::net::smoltcp_iface::set_ip_address(config.ip_addr, config.subnet_mask)?;

        // Set gateway if provided
        if let Some(gw) = config.gateway {
            crate::net::smoltcp_iface::set_gateway(gw)?;
        }

        crate::info!("dhcp: Configuration applied");
        Ok(())
    }
}
