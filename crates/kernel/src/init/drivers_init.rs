//! Phase 4: Device drivers initialization
//!
//! This phase initializes all device drivers in the correct dependency order:
//! - Block devices (virtio-blk)
//! - Network devices (virtio-net)
//! - Block driver framework
//! - Watchdog
//! - Optional GPIO/Mailbox (platform-specific)
//! - Optional ext4/ext2 mount at /models
//! - Optional driver framework and device discovery

use super::{InitError, InitResult};

/// Initialize all device drivers in the correct dependency order
///
/// # Safety
/// Must be called after VFS initialization (Phase 3)
/// Must be called before network stack (Phase 5)
pub unsafe fn init_drivers() -> InitResult<()> {
    // Initialize block devices (virtio-blk)
    init_block_devices()?;

    // Try to mount ext4/ext2 at /models if block device is available
    mount_models_filesystem()?;

    // Initialize network devices (virtio-net)
    init_network_devices()?;

    // Initialize block driver framework
    init_block_driver()?;

    // Initialize watchdog
    init_watchdog()?;

    // Initialize platform-specific drivers (GPIO, mailbox)
    init_platform_drivers()?;

    // Optionally initialize driver framework and discover devices
    init_driver_framework()?;

    Ok(())
}

/// Initialize block devices (virtio-blk)
unsafe fn init_block_devices() -> InitResult<()> {
    #[cfg(target_arch = "aarch64")]
    crate::arch::aarch64::init_virtio_blk();

    Ok(())
}

/// Try to mount ext4/ext2 filesystem at /models if block device exists
unsafe fn mount_models_filesystem() -> InitResult<()> {
    use crate::block::list_block_devices;
    use crate::vfs::get_root;

    let devs = list_block_devices();
    if devs.is_empty() {
        return Ok(());
    }

    // Get root filesystem
    let root = get_root().ok_or(InitError::MountFailed)?;

    // Create mountpoint
    let _ = root.create("models", crate::vfs::S_IFDIR | 0o755);

    // Try each device until one mounts
    for dev in devs {
        // Try ext4 first (with journal replay fixes)
        if let Ok(ext4_root) = crate::vfs::ext4::mount_ext4(dev.clone()) {
            let _ = crate::vfs::mount("ext4", ext4_root, "/models");
            return Ok(());
        }

        // Fallback to ext2 read-only
        if let Ok(ext2_root) = crate::vfs::ext2::mount_ext2(dev.clone()) {
            let _ = crate::vfs::mount("ext2", ext2_root, "/models");
            return Ok(());
        }
    }

    // Not an error if no compatible filesystem found
    Ok(())
}

/// Initialize network devices (virtio-net)
unsafe fn init_network_devices() -> InitResult<()> {
    #[cfg(target_arch = "aarch64")]
    crate::arch::aarch64::init_virtio_net();

    Ok(())
}

/// Initialize block driver framework
unsafe fn init_block_driver() -> InitResult<()> {
    crate::drivers::block::init()
        .map_err(|_| InitError::BlockDriverFailed)?;
    Ok(())
}

/// Initialize watchdog
unsafe fn init_watchdog() -> InitResult<()> {
    let _ = crate::drivers::watchdog::init();
    Ok(())
}

/// Initialize platform-specific drivers (GPIO, mailbox)
unsafe fn init_platform_drivers() -> InitResult<()> {
    // Initialize GPIO (General Purpose I/O)
    #[cfg(feature = "rpi5-gpio")]
    {
        let gpio_base = 0x107d508500usize;  // BCM2712 GPIO base (from FDT)
        crate::drivers::gpio::bcm2xxx::init(gpio_base);
    }

    // Initialize Mailbox (Firmware interface)
    #[cfg(feature = "rpi5-mailbox")]
    {
        let mailbox_base = 0x107c013880usize;  // BCM2712 mailbox base (from FDT)
        crate::drivers::firmware::mailbox::init(mailbox_base);
    }

    Ok(())
}

/// Initialize driver framework and discover devices (optional, feature-gated)
unsafe fn init_driver_framework() -> InitResult<()> {
    #[cfg(feature = "virtio-console")]
    {
        crate::driver::init_driver_framework()
            .map_err(|_| InitError::BlockDriverFailed)?;

        // Register VirtIO console driver
        crate::driver::register_driver(crate::virtio_console::get_virtio_console_driver())
            .map_err(|_| InitError::BlockDriverFailed)?;

        // Discover devices
        if let Some(registry) = crate::driver::get_driver_registry() {
            let _ = registry.discover_devices();
        }
    }

    Ok(())
}
