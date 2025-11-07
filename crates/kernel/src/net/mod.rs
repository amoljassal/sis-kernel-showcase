/// Network subsystem (Phase C)
///
/// Provides TCP/IP networking via smoltcp integration

pub mod smoltcp_iface;
pub mod socket;
pub mod dhcp;

pub use smoltcp_iface::{init_network, get_interface, network_poll};
pub use socket::{Socket, SocketHandle};
