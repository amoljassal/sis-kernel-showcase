//! Phase 5: Network stack initialization
//!
//! This phase initializes the network stack:
//! - Network interface (smoltcp)
//! - DHCP client
//! - Optional SNTP time synchronization

use super::{InitError, InitResult};

/// Initialize network stack and configure networking
///
/// # Safety
/// Must be called after network device drivers (Phase 4)
/// Must be called before SMP/security init (Phase 6)
pub unsafe fn init_network() -> InitResult<()> {
    // Initialize network interface
    if let Err(_) = crate::net::init_network() {
        // Network init failed - not fatal, skip network configuration
        return Ok(());
    }

    // Configure network with DHCP or static fallback
    configure_network()?;

    // Optional SNTP sync
    sync_time_sntp()?;

    Ok(())
}

/// Configure network with DHCP or static fallback
unsafe fn configure_network() -> InitResult<()> {
    // Try DHCP configuration (with retry/backoff)
    let mut dhcp_client = crate::net::dhcp::DhcpClient::new();
    match dhcp_client.acquire_lease() {
        Ok(config) => {
            if let Err(e) = dhcp_client.apply_config(&config) {
                crate::warn!("net: Failed to apply DHCP config: {:?}", e);
                // Fall through to static config
            } else {
                return Ok(());
            }
        }
        Err(e) => {
            crate::warn!("net: DHCP failed: {:?}", e);
            // Fall through to static config
        }
    }

    // Static fallback (QEMU user networking default)
    let _ = crate::net::smoltcp_iface::set_ip_address([10, 0, 2, 15], 24);
    let _ = crate::net::smoltcp_iface::set_gateway([10, 0, 2, 2]);

    Ok(())
}

/// Synchronize time with SNTP (optional, best-effort)
unsafe fn sync_time_sntp() -> InitResult<()> {
    #[cfg(feature = "sntp")]
    {
        if let Ok(secs) = crate::net::sntp::sntp_query([10, 0, 2, 2]) {
            crate::info!("sntp: time (unix secs) = {}", secs);
        } else {
            crate::warn!("sntp: query failed");
        }
    }

    Ok(())
}
