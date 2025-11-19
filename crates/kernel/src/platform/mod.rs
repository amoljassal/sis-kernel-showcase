//! Platform abstraction layer for hardware-neutral bring-up.
//! Provides device descriptors and memory ranges. Default implementation targets QEMU virt.

#![allow(dead_code)]

/// UART descriptor
#[derive(Debug, Copy, Clone)]
pub struct UartDesc {
    pub base: usize,
    pub clock_hz: u32,
}

/// GICv3 descriptor
#[derive(Debug, Copy, Clone)]
pub struct GicDesc {
    pub gicd: usize,
    pub gicr: usize,
}

/// Generic timer descriptor
#[derive(Debug, Copy, Clone)]
pub struct TimerDesc {
    pub freq_hz: u64,
}

/// MMIO range descriptor
#[derive(Copy, Clone)]
pub struct MmioRange {
    pub start: usize,
    pub size: usize,
    pub device: bool,
}

/// RAM range descriptor
#[derive(Copy, Clone)]
pub struct RamRange {
    pub start: usize,
    pub size: usize,
}

/// Platform trait provides device descriptors and ranges.
pub trait Platform: Sync {
    fn uart(&self) -> UartDesc;
    fn gic(&self) -> GicDesc;
    fn timer(&self) -> TimerDesc;
    fn mmio_ranges(&self) -> &'static [MmioRange];
    fn ram_ranges(&self) -> &'static [RamRange];
    fn psci_available(&self) -> bool { false }
    /// Optional hint for VirtIO MMIO layout: (base, per-device size, irq_base)
    fn virtio_mmio_hint(&self) -> Option<(usize, usize, u32)> { None }
}

pub mod qemu_virt;
pub mod dt;
pub mod rpi5;

/// Platform type enumeration
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PlatformType {
    /// QEMU aarch64 virt machine
    QemuVirt = 0,
    /// Raspberry Pi 5 (BCM2712)
    RaspberryPi5 = 1,
    /// Unknown/generic platform
    Unknown = 2,
}

use core::sync::atomic::{AtomicU8, Ordering};
use spin::Once;

/// Return the active platform implementation
static ACTIVE_PLATFORM: Once<&'static dyn Platform> = Once::new();
static DETECTED_PLATFORM: AtomicU8 = AtomicU8::new(PlatformType::Unknown as u8);

pub fn active() -> &'static dyn Platform {
    ACTIVE_PLATFORM.get().copied().unwrap_or(&qemu_virt::INSTANCE)
}

/// Get the detected platform type
pub fn detected_type() -> PlatformType {
    match DETECTED_PLATFORM.load(Ordering::Acquire) {
        0 => PlatformType::QemuVirt,
        1 => PlatformType::RaspberryPi5,
        2 => PlatformType::Unknown,
        _ => PlatformType::Unknown,
    }
}

/// Try to override the active platform by parsing a provided DTB pointer.
/// Returns true on success. Safe to call multiple times; subsequent calls are ignored once set.
///
/// This function:
/// 1. Parses the FDT to extract device information
/// 2. Detects the platform type (QEMU virt vs RPi5)
/// 3. Selects the appropriate platform implementation
///
/// # Safety
/// Must be called with a valid FDT pointer during early boot, before the platform is used.
pub unsafe fn override_with_dtb(dtb_ptr: *const u8) -> bool {
    if ACTIVE_PLATFORM.get().is_some() { return true; }

    // Parse the FDT first
    if let Some(p) = dt::from_dtb(dtb_ptr) {
        // Detect platform type based on FDT contents
        let platform_type = detect_platform_from_fdt();

        crate::info!("Platform detected: {:?}", platform_type);

        // Set the detected platform type atomically
        DETECTED_PLATFORM.store(platform_type as u8, Ordering::Release);

        // Select the appropriate platform implementation
        match platform_type {
            PlatformType::RaspberryPi5 => {
                // Use the FDT-based platform for RPi5
                // This gives us access to all the parsed device information
                ACTIVE_PLATFORM.call_once(|| p);
                rpi5::init_hardware();
            }
            PlatformType::QemuVirt => {
                // Use the FDT-based platform for QEMU as well
                ACTIVE_PLATFORM.call_once(|| p);
            }
            PlatformType::Unknown => {
                // Default to FDT-based platform
                ACTIVE_PLATFORM.call_once(|| p);
                crate::warn!("Unknown platform, using FDT-based configuration");
            }
        }

        true
    } else {
        crate::warn!("Failed to parse FDT, using default platform");
        DETECTED_PLATFORM.store(PlatformType::QemuVirt as u8, Ordering::Release);
        false
    }
}

/// Detect platform type from FDT device map
///
/// This function examines the parsed device map to determine which platform we're running on.
/// It looks for platform-specific device signatures.
fn detect_platform_from_fdt() -> PlatformType {
    if let Some(devmap) = dt::get_device_map() {
        // Check for RPi5-specific devices
        // RPi5 has SDHCI, PCIe controller, and specific device addresses
        if devmap.sdhci.is_some() {
            // If we have SDHCI with a BCM2712-specific address range, it's likely RPi5
            if let Some(sdhci) = devmap.sdhci {
                // RPi5 SDHCI is typically in the RP1 I/O hub region or VC peripheral region
                if sdhci.base > 0x1000_0000 {
                    return PlatformType::RaspberryPi5;
                }
            }
        }

        // Check for PCIe - RPi5 has RP1 PCIe
        if devmap.pcie.is_some() {
            return PlatformType::RaspberryPi5;
        }

        // Check UART base address
        if let Some(uart) = devmap.uart {
            // QEMU typically has UART at 0x0900_0000
            // RPi5 has UART at higher addresses (0x107d001000 or similar)
            if uart.base >= 0x0900_0000 && uart.base < 0x0A00_0000 {
                // Likely QEMU virt
                return PlatformType::QemuVirt;
            } else if uart.base > 0x1000_0000 {
                // Likely RPi5
                return PlatformType::RaspberryPi5;
            }
        }
    }

    // Default to QEMU if we can't determine
    PlatformType::QemuVirt
}
