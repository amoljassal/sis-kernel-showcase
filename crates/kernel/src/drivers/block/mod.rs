//! Block device drivers
//!
//! This module provides drivers for block storage devices including:
//! - SDHCI (SD Host Controller Interface) for SD/MMC cards
//! - Future: NVMe, SATA, eMMC, etc.

pub mod sdhci;

pub use sdhci::SdhciController;

use crate::drivers::traits::BlockDevice;
use crate::lib::error::Result;
use crate::platform::dt::{SdhciInfo, get_device_map};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

/// Global block device registry
static BLOCK_DEVICES: Mutex<Vec<Arc<dyn BlockDevice>>> = Mutex::new(Vec::new());

/// Register a block device
pub fn register_block_device(device: Arc<dyn BlockDevice>) -> Result<()> {
    let mut devices = BLOCK_DEVICES.lock();
    crate::info!("Block: Registered device '{}'", device.name());
    devices.push(device);
    Ok(())
}

/// Get all registered block devices
pub fn get_block_devices() -> Vec<Arc<dyn BlockDevice>> {
    BLOCK_DEVICES.lock().clone()
}

/// Get block device by name
pub fn get_block_device(name: &str) -> Option<Arc<dyn BlockDevice>> {
    let devices = BLOCK_DEVICES.lock();
    devices.iter().find(|d| d.name() == name).cloned()
}

/// Initialize SDHCI controller from device tree
///
/// This function:
/// 1. Reads SDHCI information from the device tree
/// 2. Creates an SDHCI controller instance
/// 3. Initializes the controller and SD card
/// 4. Registers the device
pub unsafe fn init_sdhci_from_dt() -> Result<()> {
    // Get SDHCI info from device tree
    let devmap = get_device_map().ok_or(crate::lib::error::Errno::ENODEV)?;
    let sdhci_info = devmap.sdhci.ok_or(crate::lib::error::Errno::ENODEV)?;

    crate::info!("Block: Initializing SDHCI at {:#x}", sdhci_info.base);

    // Create SDHCI controller
    let mut controller = Box::new(SdhciController::new(
        sdhci_info.base,
        alloc::format!("mmcblk0"),
    ));

    // Initialize controller and card
    controller.init()?;

    // Register as block device
    let controller: alloc::sync::Arc<sdhci::SdhciController> = alloc::sync::Arc::from(controller);
    let device: alloc::sync::Arc<dyn BlockDevice> = controller as alloc::sync::Arc<dyn BlockDevice>;
    register_block_device(device)?;

    Ok(())
}

/// Initialize all block devices
///
/// This should be called during kernel initialization after the platform
/// has been detected and the device tree parsed.
pub unsafe fn init() -> Result<()> {
    crate::info!("Block: Initializing block devices");

    // Try to initialize SDHCI (RPi5)
    if let Err(e) = init_sdhci_from_dt() {
        crate::warn!("Block: Failed to initialize SDHCI: {:?}", e);
    }

    let device_count = BLOCK_DEVICES.lock().len();
    crate::info!("Block: Initialized {} block device(s)", device_count);

    Ok(())
}
