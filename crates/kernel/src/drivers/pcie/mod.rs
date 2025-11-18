//! PCIe Driver Infrastructure for Raspberry Pi 5
//!
//! This module provides PCIe support for the Raspberry Pi 5, including:
//! - ECAM (Enhanced Configuration Access Mechanism) for device enumeration
//! - RP1 I/O Hub driver for peripheral access
//! - Device scanning and initialization
//!
//! # Architecture
//!
//! The PCIe subsystem on RPi5 follows this hierarchy:
//!
//! ```text
//! BCM2712 SoC
//!   └─> PCIe Controller (Gen 2 ×4)
//!         └─> RP1 I/O Hub (vendor:device = 0x1DE4:0x0001)
//!               ├─> I2C Controllers (6×)
//!               ├─> SPI Controllers (5×)
//!               ├─> UART Controllers (2×)
//!               ├─> USB 3.0 Host (XHCI)
//!               ├─> Ethernet (GENET v5)
//!               ├─> GPIO Expander
//!               └─> PWM Controllers (2×)
//! ```
//!
//! # Initialization Sequence
//!
//! 1. Parse FDT to get PCIe ECAM base address
//! 2. Initialize ECAM accessor
//! 3. Scan PCIe bus 0 for devices
//! 4. Detect and initialize RP1 I/O Hub
//! 5. Make peripheral controllers available to other drivers
//!
//! # Usage
//!
//! ```rust
//! use crate::drivers::pcie;
//!
//! // Initialize PCIe subsystem (called during platform init)
//! pcie::initialize()?;
//!
//! // Access RP1 driver
//! if let Some(rp1) = pcie::get_rp1() {
//!     let i2c0_base = rp1.i2c_base(0);
//!     // Use I2C controller...
//! }
//! ```

pub mod ecam;
pub mod rp1;

use crate::drivers::{DriverError, DriverResult};
use crate::platform;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Once;

/// PCIe-specific errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PcieError {
    /// ECAM base address not available
    NoEcam,

    /// RP1 device not found on PCIe bus
    Rp1NotFound,

    /// Invalid PCIe device number
    InvalidDevice,

    /// Invalid PCIe function number
    InvalidFunction,

    /// Invalid configuration space offset
    InvalidOffset,

    /// Offset out of bounds
    OutOfBounds,

    /// Misaligned register access
    MisalignedAccess,

    /// No device at address
    NoDevice,

    /// Invalid BAR index or BAR not implemented
    InvalidBar,

    /// BAR is not a memory BAR
    NoBar,
}

impl From<PcieError> for DriverError {
    fn from(err: PcieError) -> Self {
        match err {
            PcieError::NoEcam => DriverError::DeviceNotFound,
            PcieError::Rp1NotFound => DriverError::DeviceNotFound,
            PcieError::InvalidDevice => DriverError::InvalidParameter,
            PcieError::InvalidFunction => DriverError::InvalidParameter,
            PcieError::InvalidOffset => DriverError::InvalidParameter,
            PcieError::OutOfBounds => DriverError::InvalidParameter,
            PcieError::MisalignedAccess => DriverError::InvalidParameter,
            PcieError::NoDevice => DriverError::DeviceNotFound,
            PcieError::InvalidBar => DriverError::DeviceNotFound,
            PcieError::NoBar => DriverError::DeviceNotFound,
        }
    }
}

/// Global PCIe state
struct PcieState {
    /// ECAM accessor
    ecam: ecam::Ecam,

    /// RP1 driver (if detected)
    rp1: Option<rp1::Rp1Driver>,

    /// Initialization complete flag
    initialized: AtomicBool,
}

/// Global PCIe state instance
static PCIE_STATE: Once<PcieState> = Once::new();

/// Initialize PCIe subsystem
///
/// This function should be called once during platform initialization.
/// It performs the following steps:
/// 1. Gets PCIe ECAM information from FDT
/// 2. Creates ECAM accessor
/// 3. Scans for PCIe devices
/// 4. Initializes RP1 I/O Hub
///
/// # Returns
/// Ok(()) if initialization succeeds, or an error if:
/// - ECAM base address not available from FDT
/// - RP1 device not found
/// - RP1 initialization fails
pub fn initialize() -> DriverResult<()> {
    // Get PCIe info from FDT
    let pcie_info = platform::dt::get_device_map()
        .and_then(|devmap| devmap.pcie);

    let pcie_info = pcie_info.ok_or(PcieError::NoEcam)?;

    crate::info!("[PCIe] Initializing PCIe subsystem");
    crate::info!("[PCIe] ECAM base: {:#018x}, size: {:#x}", pcie_info.cfg_base, pcie_info.cfg_size);

    // Create ECAM accessor
    let ecam = ecam::Ecam::new(pcie_info.cfg_base, pcie_info.cfg_size);

    // Scan bus 0 for devices
    crate::info!("[PCIe] Scanning bus 0...");
    let devices = ecam.scan_bus(0);

    crate::info!("[PCIe] Found {} device(s)", devices.len());
    for dev in &devices {
        crate::info!(
            "[PCIe]   {:02x}:{:02x}.{} - [{:04x}:{:04x}] class={:06x}",
            dev.address.bus,
            dev.address.device,
            dev.address.function,
            dev.vendor_id,
            dev.device_id,
            dev.class_code
        );
    }

    // Initialize RP1 I/O Hub
    crate::info!("[PCIe] Initializing RP1 I/O Hub...");
    let rp1 = rp1::initialize_rp1(&ecam)?;

    // Store global state
    PCIE_STATE.call_once(|| PcieState {
        ecam,
        rp1: Some(rp1),
        initialized: AtomicBool::new(true),
    });

    crate::info!("[PCIe] PCIe subsystem initialized");

    Ok(())
}

/// Check if PCIe subsystem is initialized
pub fn is_initialized() -> bool {
    PCIE_STATE
        .get()
        .map(|state| state.initialized.load(Ordering::Acquire))
        .unwrap_or(false)
}

/// Get ECAM accessor
///
/// Returns a reference to the ECAM accessor if PCIe is initialized.
pub fn get_ecam() -> Option<&'static ecam::Ecam> {
    PCIE_STATE.get().map(|state| &state.ecam)
}

/// Get RP1 driver
///
/// Returns a reference to the RP1 driver if it was successfully initialized.
pub fn get_rp1() -> Option<&'static rp1::Rp1Driver> {
    PCIE_STATE
        .get()
        .and_then(|state| state.rp1.as_ref())
}

/// Scan PCIe bus for devices
///
/// Returns a list of all devices found on the specified bus.
///
/// # Arguments
/// * `bus` - Bus number to scan (typically 0 for root bus)
///
/// # Returns
/// Vector of PCIe device information, or error if PCIe not initialized
pub fn scan_bus(bus: u8) -> DriverResult<alloc::vec::Vec<ecam::PciDevice>> {
    let ecam = get_ecam().ok_or(DriverError::NotInitialized)?;
    Ok(ecam.scan_bus(bus))
}

/// Find devices by vendor and device ID
///
/// Scans the PCIe bus and returns all devices matching the specified
/// vendor and device IDs.
///
/// # Arguments
/// * `vendor_id` - PCIe vendor ID to match
/// * `device_id` - PCIe device ID to match
///
/// # Returns
/// Vector of matching devices, or error if PCIe not initialized
pub fn find_devices(vendor_id: u16, device_id: u16) -> DriverResult<alloc::vec::Vec<ecam::PciDevice>> {
    let devices = scan_bus(0)?;
    Ok(devices.into_iter().filter(|dev| dev.matches(vendor_id, device_id)).collect())
}

/// Get device information by address
///
/// Reads and returns device information for a specific PCIe address.
///
/// # Arguments
/// * `address` - PCIe address (bus/device/function)
///
/// # Returns
/// Device information, or error if device not found or PCIe not initialized
pub fn get_device_info(address: ecam::PciAddress) -> DriverResult<ecam::PciDevice> {
    let ecam = get_ecam().ok_or(DriverError::NotInitialized)?;
    ecam.read_device_info(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcie_error_conversion() {
        let err: DriverError = PcieError::NoEcam.into();
        assert!(matches!(err, DriverError::NotFound));

        let err: DriverError = PcieError::InvalidDevice.into();
        assert!(matches!(err, DriverError::InvalidParameter));
    }
}
