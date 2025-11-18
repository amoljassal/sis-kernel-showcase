//! Raspberry Pi 5 platform support
//!
//! This module provides platform-specific configuration for the Raspberry Pi 5 (BCM2712 SoC).
//!
//! # Hardware Specifications
//! - **SoC:** Broadcom BCM2712 (16nm)
//! - **CPU:** 4× Cortex-A76 @ 2.4GHz
//! - **RAM:** 4GB or 8GB LPDDR4X-4267
//! - **GIC:** ARM GICv3 (GICD + GICR per CPU)
//! - **Timer:** ARM Generic Timer @ 54MHz
//! - **UART:** PL011 (ARM PrimeCell UART)
//! - **Storage:** Arasan SDHCI 5.1 controller
//! - **I/O Hub:** RP1 (PCIe Gen 2 ×4) for USB, Ethernet, GPIO
//!
//! # Memory Map (Indicative - verified via FDT)
//! ```text
//! 0x0000_0000 - 0x3FFF_FFFF : DRAM (low 1GB)
//! 0x4000_0000 - 0x7FFF_FFFF : DRAM continuation
//! 0x7c00_0000 - 0x7FFF_FFFF : VC peripherals
//! 0x107f_0000 - 0x107F_FFFF : GIC Distributor
//! 0x107f_0000 - 0x108F_FFFF : GIC Redistributors
//! 0x1000_0000 - 0x1FFF_FFFF : RP1 I/O Hub (PCIe, USB, Ethernet)
//! ```
//!
//! # Boot Flow
//! 1. UEFI firmware (EDK2 RPi) loads at EL2
//! 2. UEFI loads kernel ELF and passes FDT address in x0
//! 3. Kernel entry at EL1 with MMU off
//! 4. Platform detection via FDT compatible strings
//! 5. Device initialization using FDT-provided addresses
//!
//! # Device Tree Detection
//! The RPi5 platform is detected by checking the FDT root compatible string for:
//! - "raspberrypi,5-model-b"
//! - "brcm,bcm2712"

use super::{GicDesc, MmioRange, Platform, RamRange, TimerDesc, UartDesc};

/// Raspberry Pi 5 platform descriptor
///
/// This structure provides access to RPi5-specific hardware configuration.
/// Device addresses are primarily sourced from the FDT at boot time, with
/// sensible defaults provided as fallbacks.
pub struct Rpi5Platform {
    /// Cached device information from FDT (populated during init)
    initialized: core::sync::atomic::AtomicBool,
}

impl Rpi5Platform {
    /// Create a new RPi5 platform instance
    pub const fn new() -> Self {
        Self {
            initialized: core::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Mark platform as initialized
    pub fn mark_initialized(&self) {
        self.initialized.store(true, core::sync::atomic::Ordering::Release);
    }

    /// Check if platform has been initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(core::sync::atomic::Ordering::Acquire)
    }

    /// Get SDHCI device information from FDT
    pub fn sdhci_info(&self) -> Option<super::dt::SdhciInfo> {
        super::dt::get_device_map()?.sdhci
    }

    /// Get PCIe controller information from FDT
    pub fn pcie_info(&self) -> Option<super::dt::PcieInfo> {
        super::dt::get_device_map()?.pcie
    }

    /// Get USB XHCI information from FDT
    pub fn usb_info(&self) -> Option<super::dt::UsbInfo> {
        super::dt::get_device_map()?.usb
    }

    /// Get Ethernet controller information from FDT
    pub fn ethernet_info(&self) -> Option<super::dt::EthInfo> {
        super::dt::get_device_map()?.ethernet
    }
}

impl Platform for Rpi5Platform {
    fn uart(&self) -> UartDesc {
        // Try to get from FDT first
        if let Some(devmap) = super::dt::get_device_map() {
            if let Some(uart) = devmap.uart {
                return uart;
            }
        }

        // Default: PL011 on RPi5 (typical address, verify via FDT)
        UartDesc {
            base: 0x107d001000,  // Default RPi5 PL011 address
            clock_hz: 48_000_000, // 48MHz UART clock on RPi5
        }
    }

    fn gic(&self) -> GicDesc {
        // Try to get from FDT first
        if let Some(devmap) = super::dt::get_device_map() {
            if let Some(gic) = devmap.gic {
                return gic;
            }
        }

        // Default GICv3 addresses for RPi5 (verify via FDT)
        GicDesc {
            gicd: 0x107fef0000,   // GIC Distributor
            gicr: 0x107ff00000,   // GIC Redistributor base (per-CPU regions)
        }
    }

    fn timer(&self) -> TimerDesc {
        // ARM Generic Timer frequency is read from CNTFRQ_EL0 at runtime
        // RPi5 typically runs at 54MHz, but we read it dynamically
        TimerDesc {
            freq_hz: unsafe {
                let freq: u64;
                core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
                freq
            },
        }
    }

    fn mmio_ranges(&self) -> &'static [MmioRange] {
        // Default MMIO ranges for RPi5
        // These will be overridden by FDT-parsed values when available
        const RPi5_MMIO: &[MmioRange] = &[
            // GIC region (includes GICD + GICR for 4 cores)
            MmioRange {
                start: 0x107f_0000,
                size: 0x0020_0000,  // 2MB
                device: true,
            },
            // UART region
            MmioRange {
                start: 0x107d001000,
                size: 0x1000,  // 4KB
                device: true,
            },
            // SDHCI region (placeholder, use FDT)
            MmioRange {
                start: 0x1000_fff0,
                size: 0x10000,
                device: true,
            },
            // RP1 I/O Hub region (PCIe, USB, Ethernet)
            MmioRange {
                start: 0x1f00_000000,
                size: 0x0040_0000,  // 4MB
                device: true,
            },
        ];
        RPi5_MMIO
    }

    fn ram_ranges(&self) -> &'static [RamRange] {
        // Try to get from FDT first
        // Note: This is a placeholder. In practice, RAM ranges should come from FDT.
        const RPi5_RAM: &[RamRange] = &[
            RamRange {
                start: 0x0000_0000,
                size: 0x8000_0000,  // 2GB default (RPi5 has 4GB or 8GB)
            },
        ];
        RPi5_RAM
    }

    fn psci_available(&self) -> bool {
        true  // RPi5 UEFI provides PSCI
    }
}

/// Global RPi5 platform instance
pub static INSTANCE: Rpi5Platform = Rpi5Platform::new();

/// Detect if we're running on Raspberry Pi 5 by checking FDT compatible strings
///
/// This function should be called early during boot after the FDT has been parsed.
/// It checks for RPi5-specific compatible strings in the device tree root node.
///
/// # Safety
/// Must be called after FDT parsing is complete.
pub fn detect_rpi5() -> bool {
    // For now, we rely on successful FDT parsing with RPi5-specific devices
    // A more robust implementation would check the FDT root compatible string
    // for "raspberrypi,5-model-b" or "brcm,bcm2712"

    // Check if we have RPi5-specific devices in the device map
    if let Some(devmap) = super::dt::get_device_map() {
        // If we have SDHCI or other RPi5-specific devices, we're likely on RPi5
        // This is a heuristic; proper detection would parse root compatible strings
        devmap.sdhci.is_some() || devmap.pcie.is_some()
    } else {
        false
    }
}

/// Initialize RPi5-specific hardware
///
/// This should be called after FDT parsing and platform detection.
/// It performs RPi5-specific initialization that isn't covered by generic drivers.
pub fn init_hardware() {
    crate::info!("Initializing Raspberry Pi 5 hardware");

    // Mark platform as initialized
    INSTANCE.mark_initialized();

    // Log detected hardware
    if let Some(devmap) = super::dt::get_device_map() {
        if let Some(sdhci) = devmap.sdhci {
            crate::info!("  SDHCI @ {:#x}", sdhci.base);
        }
        if let Some(usb) = devmap.usb {
            crate::info!("  USB XHCI @ {:#x}", usb.base);
        }
        if let Some(eth) = devmap.ethernet {
            crate::info!("  Ethernet @ {:#x}", eth.base);
        }
        if let Some(pcie) = devmap.pcie {
            crate::info!("  PCIe @ {:#x}", pcie.base);
        }
    }

    // Initialize PCIe and RP1 I/O Hub
    // This is critical for RPi5 as the RP1 provides USB, Ethernet, and GPIO
    crate::info!("Initializing PCIe subsystem...");
    match crate::drivers::pcie::initialize() {
        Ok(()) => {
            crate::info!("PCIe and RP1 I/O Hub initialized successfully");

            // Initialize PWM controllers (depends on RP1)
            crate::info!("Initializing PWM subsystem...");
            match crate::drivers::pwm::initialize() {
                Ok(()) => {
                    crate::info!("PWM controllers initialized successfully");
                }
                Err(e) => {
                    crate::warn!("Failed to initialize PWM: {:?}", e);
                    crate::warn!("Servo and motor control will not be available");
                }
            }
        }
        Err(e) => {
            crate::warn!("Failed to initialize PCIe/RP1: {:?}", e);
            crate::warn!("USB, Ethernet, GPIO, and PWM will not be available");
        }
    }

    // Additional RPi5-specific initialization:
    // - Clock configuration (if needed)
    // - Power management setup (if needed)
    // - Additional peripheral initialization
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_defaults() {
        let platform = Rpi5Platform::new();
        let uart = platform.uart();
        assert!(uart.base != 0);
        assert!(uart.clock_hz > 0);

        let gic = platform.gic();
        assert!(gic.gicd != 0);
        assert!(gic.gicr != 0);
    }

    #[test]
    fn test_psci_available() {
        let platform = Rpi5Platform::new();
        assert!(platform.psci_available());
    }
}
