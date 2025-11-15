// Minimal test initramfs data (no BusyBox)
pub const INITRAMFS_DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/initramfs_minimal.cpio"));
