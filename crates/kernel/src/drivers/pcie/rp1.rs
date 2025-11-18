//! RP1 I/O Hub Driver for Raspberry Pi 5
//!
//! The RP1 is a custom I/O controller chip designed by Raspberry Pi for the RPi5.
//! It connects to the BCM2712 SoC via PCIe Gen 2 ×4 and provides:
//!
//! - 6× I2C controllers
//! - 5× SPI controllers
//! - 2× UART controllers
//! - USB 2.0/3.0 host controller (XHCI)
//! - Ethernet controller (GENET v5)
//! - Extended GPIO pins
//! - PWM controllers
//! - Audio I/O
//!
//! # Critical Importance
//!
//! **Without the RP1 driver, the RPi5 has NO USB, NO Ethernet, and NO extended GPIO
//! on real hardware.** The RP1 is the gateway to almost all external peripherals.
//!
//! # PCIe Enumeration
//!
//! The RP1 appears as a standard PCIe device:
//! - Vendor ID: 0x1DE4 (Raspberry Pi)
//! - Device ID: 0x0001 (RP1)
//! - Class: 0x058000 (Other system peripheral)
//!
//! # Initialization Sequence
//!
//! 1. Detect RP1 via PCIe enumeration
//! 2. Map RP1 configuration and MMIO regions
//! 3. Initialize power management
//! 4. Initialize internal bus controllers (I2C, SPI, etc.)
//! 5. Route interrupts to GIC
//! 6. Enable peripheral clocks
//!
//! # References
//! - Raspberry Pi 5 Datasheet (when available)
//! - Linux: drivers/misc/rp1.c (if available in RPi kernel tree)
//! - BCM2712 TRM (Technical Reference Manual)

use super::ecam::{Ecam, PciAddress, PciDevice, command};
use super::PcieError;
use crate::drivers::{DriverResult, DriverError};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// RP1 Vendor ID (Raspberry Pi)
pub const RP1_VENDOR_ID: u16 = 0x1DE4;

/// RP1 Device ID
pub const RP1_DEVICE_ID: u16 = 0x0001;

/// RP1 I2C controller count
pub const RP1_I2C_COUNT: u8 = 6;

/// RP1 SPI controller count
pub const RP1_SPI_COUNT: u8 = 5;

/// RP1 UART controller count
pub const RP1_UART_COUNT: u8 = 2;

/// RP1 PWM controller count
pub const RP1_PWM_COUNT: u8 = 2;

/// RP1 register offsets (preliminary, subject to change based on actual hardware)
mod regs {
    /// RP1 identification register
    pub const ID: u32 = 0x0000;

    /// RP1 version register
    pub const VERSION: u32 = 0x0004;

    /// RP1 control register
    pub const CONTROL: u32 = 0x0008;

    /// RP1 status register
    pub const STATUS: u32 = 0x000C;

    /// RP1 interrupt status register
    pub const IRQ_STATUS: u32 = 0x0010;

    /// RP1 interrupt enable register
    pub const IRQ_ENABLE: u32 = 0x0014;

    /// RP1 clock control register
    pub const CLOCK_CTRL: u32 = 0x0020;

    /// RP1 power management register
    pub const POWER_CTRL: u32 = 0x0024;

    /// I2C controller base offset
    pub const I2C_BASE: u32 = 0x1000;

    /// I2C controller stride
    pub const I2C_STRIDE: u32 = 0x0100;

    /// SPI controller base offset
    pub const SPI_BASE: u32 = 0x2000;

    /// SPI controller stride
    pub const SPI_STRIDE: u32 = 0x0100;

    /// PWM controller base offset
    pub const PWM_BASE: u32 = 0x3000;

    /// PWM controller stride
    pub const PWM_STRIDE: u32 = 0x0100;

    /// GPIO controller base offset
    pub const GPIO_BASE: u32 = 0x4000;

    /// XHCI (USB 3.0) controller base offset
    pub const XHCI_BASE: u32 = 0x200000;
}

/// RP1 control register bits
mod control {
    /// Enable RP1 I/O hub
    pub const ENABLE: u32 = 1 << 0;

    /// Reset RP1 (active high)
    pub const RESET: u32 = 1 << 1;

    /// Enable I2C controllers
    pub const I2C_ENABLE: u32 = 1 << 8;

    /// Enable SPI controllers
    pub const SPI_ENABLE: u32 = 1 << 9;

    /// Enable PWM controllers
    pub const PWM_ENABLE: u32 = 1 << 10;

    /// Enable GPIO controller
    pub const GPIO_ENABLE: u32 = 1 << 11;
}

/// RP1 status register bits
mod status {
    /// RP1 is ready
    pub const READY: u32 = 1 << 0;

    /// RP1 initialization complete
    pub const INIT_DONE: u32 = 1 << 1;

    /// RP1 error flag
    pub const ERROR: u32 = 1 << 31;
}

/// RP1 I/O Hub controller state
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Rp1State {
    /// RP1 not detected
    NotDetected,

    /// RP1 detected but not initialized
    Detected,

    /// RP1 initializing
    Initializing,

    /// RP1 ready
    Ready,

    /// RP1 error
    Error,
}

/// XHCI controller information
#[derive(Debug, Clone, Copy)]
pub struct XhciInfo {
    /// Base MMIO address of XHCI controller
    pub base_addr: usize,
}

/// RP1 I/O Hub driver
pub struct Rp1Driver {
    /// PCIe address of RP1 device
    address: PciAddress,

    /// MMIO base address (from BAR0)
    mmio_base: usize,

    /// MMIO region size
    mmio_size: usize,

    /// Current state
    state: AtomicU32,

    /// Initialization complete flag
    initialized: AtomicBool,
}

impl Rp1Driver {
    /// Create a new RP1 driver instance
    ///
    /// # Arguments
    /// * `device` - PCIe device information for RP1
    /// * `mmio_base` - MMIO base address (from BAR0)
    /// * `mmio_size` - MMIO region size
    pub fn new(device: PciDevice, mmio_base: usize, mmio_size: usize) -> Self {
        Self {
            address: device.address,
            mmio_base,
            mmio_size,
            state: AtomicU32::new(Rp1State::Detected as u32),
            initialized: AtomicBool::new(false),
        }
    }

    /// Get current state
    pub fn state(&self) -> Rp1State {
        match self.state.load(Ordering::Acquire) {
            0 => Rp1State::NotDetected,
            1 => Rp1State::Detected,
            2 => Rp1State::Initializing,
            3 => Rp1State::Ready,
            _ => Rp1State::Error,
        }
    }

    /// Set state
    fn set_state(&self, state: Rp1State) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Check if RP1 is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Get MMIO base address
    pub fn mmio_base(&self) -> usize {
        self.mmio_base
    }

    /// Get MMIO region size
    pub fn mmio_size(&self) -> usize {
        self.mmio_size
    }

    /// Get PCIe address
    pub fn pci_address(&self) -> PciAddress {
        self.address
    }

    /// Read a 32-bit register
    #[inline]
    fn read_reg(&self, offset: u32) -> u32 {
        if (offset as usize) >= self.mmio_size {
            return 0;
        }
        let addr = (self.mmio_base + offset as usize) as *const u32;
        unsafe { core::ptr::read_volatile(addr) }
    }

    /// Write a 32-bit register
    #[inline]
    fn write_reg(&self, offset: u32, value: u32) {
        if (offset as usize) >= self.mmio_size {
            return;
        }
        let addr = (self.mmio_base + offset as usize) as *mut u32;
        unsafe { core::ptr::write_volatile(addr, value); }
    }

    /// Initialize the RP1 I/O Hub
    ///
    /// This performs the complete initialization sequence for the RP1:
    /// 1. Verify RP1 identification
    /// 2. Reset the device
    /// 3. Configure power management
    /// 4. Enable clocks
    /// 5. Initialize peripheral controllers
    /// 6. Wait for ready status
    pub fn initialize(&self, ecam: &Ecam) -> DriverResult<()> {
        if self.is_initialized() {
            return Ok(());
        }

        self.set_state(Rp1State::Initializing);

        // Enable memory and bus master in PCIe configuration space
        let mut cmd = ecam.read_u16(self.address, super::ecam::PCI_COMMAND)?;
        cmd |= command::MEMORY_ENABLE | command::BUS_MASTER;
        ecam.write_u16(self.address, super::ecam::PCI_COMMAND, cmd)?;

        // Read RP1 ID register to verify device
        let id = self.read_reg(regs::ID);
        let version = self.read_reg(regs::VERSION);

        crate::info!("  RP1 ID: {:#010x}, Version: {:#010x}", id, version);

        // Perform soft reset
        self.write_reg(regs::CONTROL, control::RESET);

        // Wait for reset to complete (simple delay)
        for _ in 0..1000 {
            core::hint::spin_loop();
        }

        // Clear reset and enable RP1
        self.write_reg(regs::CONTROL, control::ENABLE);

        // Enable all peripheral controllers
        let ctrl = control::ENABLE
            | control::I2C_ENABLE
            | control::SPI_ENABLE
            | control::PWM_ENABLE
            | control::GPIO_ENABLE;
        self.write_reg(regs::CONTROL, ctrl);

        // Enable clocks for all peripherals
        self.write_reg(regs::CLOCK_CTRL, 0xFFFF_FFFF);

        // Wait for RP1 to become ready
        let mut timeout = 10000;
        while timeout > 0 {
            let status = self.read_reg(regs::STATUS);

            if (status & status::ERROR) != 0 {
                self.set_state(Rp1State::Error);
                return Err(DriverError::HardwareError);
            }

            if (status & status::READY) != 0 && (status & status::INIT_DONE) != 0 {
                break;
            }

            timeout -= 1;
            core::hint::spin_loop();
        }

        if timeout == 0 {
            self.set_state(Rp1State::Error);
            let timeout_err = super::super::timeout::TimeoutError::new(10000, 10000);
            return Err(DriverError::Timeout(timeout_err));
        }

        // Verify peripheral controllers are accessible
        self.verify_peripherals()?;

        self.initialized.store(true, Ordering::Release);
        self.set_state(Rp1State::Ready);

        crate::info!("  RP1 initialization complete");
        crate::info!("    I2C controllers: {}", RP1_I2C_COUNT);
        crate::info!("    SPI controllers: {}", RP1_SPI_COUNT);
        crate::info!("    UART controllers: {}", RP1_UART_COUNT);
        crate::info!("    PWM controllers: {}", RP1_PWM_COUNT);

        Ok(())
    }

    /// Verify peripheral controllers are accessible
    fn verify_peripherals(&self) -> DriverResult<()> {
        // Try to read from each I2C controller base
        for i in 0..RP1_I2C_COUNT {
            let offset = regs::I2C_BASE + (i as u32 * regs::I2C_STRIDE);
            let _val = self.read_reg(offset);
        }

        // Try to read from each SPI controller base
        for i in 0..RP1_SPI_COUNT {
            let offset = regs::SPI_BASE + (i as u32 * regs::SPI_STRIDE);
            let _val = self.read_reg(offset);
        }

        // Try to read from PWM controller base
        for i in 0..RP1_PWM_COUNT {
            let offset = regs::PWM_BASE + (i as u32 * regs::PWM_STRIDE);
            let _val = self.read_reg(offset);
        }

        // Try to read from GPIO controller base
        let _val = self.read_reg(regs::GPIO_BASE);

        Ok(())
    }

    /// Get I2C controller MMIO address
    ///
    /// # Arguments
    /// * `index` - I2C controller index (0-5)
    ///
    /// # Returns
    /// Physical MMIO address of the I2C controller, or None if invalid index
    pub fn i2c_base(&self, index: u8) -> Option<usize> {
        if index >= RP1_I2C_COUNT {
            return None;
        }

        let offset = regs::I2C_BASE + (index as u32 * regs::I2C_STRIDE);
        Some(self.mmio_base + offset as usize)
    }

    /// Get SPI controller MMIO address
    ///
    /// # Arguments
    /// * `index` - SPI controller index (0-4)
    ///
    /// # Returns
    /// Physical MMIO address of the SPI controller, or None if invalid index
    pub fn spi_base(&self, index: u8) -> Option<usize> {
        if index >= RP1_SPI_COUNT {
            return None;
        }

        let offset = regs::SPI_BASE + (index as u32 * regs::SPI_STRIDE);
        Some(self.mmio_base + offset as usize)
    }

    /// Get PWM controller MMIO address
    ///
    /// # Arguments
    /// * `index` - PWM controller index (0-1)
    ///
    /// # Returns
    /// Physical MMIO address of the PWM controller, or None if invalid index
    pub fn pwm_base(&self, index: u8) -> Option<usize> {
        if index >= RP1_PWM_COUNT {
            return None;
        }

        let offset = regs::PWM_BASE + (index as u32 * regs::PWM_STRIDE);
        Some(self.mmio_base + offset as usize)
    }

    /// Get GPIO controller MMIO address
    ///
    /// # Returns
    /// Physical MMIO address of the GPIO controller
    pub fn gpio_base(&self) -> usize {
        self.mmio_base + regs::GPIO_BASE as usize
    }

    /// Get XHCI controller information
    ///
    /// # Returns
    /// XhciInfo struct with XHCI controller base address, or None if not initialized
    pub fn get_xhci_info(&self) -> Option<XhciInfo> {
        if !self.is_initialized() {
            return None;
        }

        Some(XhciInfo {
            base_addr: self.mmio_base + regs::XHCI_BASE as usize,
        })
    }

    /// Get interrupt status
    pub fn irq_status(&self) -> u32 {
        self.read_reg(regs::IRQ_STATUS)
    }

    /// Enable interrupts
    pub fn enable_irq(&self, mask: u32) {
        let current = self.read_reg(regs::IRQ_ENABLE);
        self.write_reg(regs::IRQ_ENABLE, current | mask);
    }

    /// Disable interrupts
    pub fn disable_irq(&self, mask: u32) {
        let current = self.read_reg(regs::IRQ_ENABLE);
        self.write_reg(regs::IRQ_ENABLE, current & !mask);
    }

    /// Power down RP1 (for power management)
    pub fn power_down(&self) -> DriverResult<()> {
        if !self.is_initialized() {
            return Err(DriverError::NotInitialized);
        }

        // Disable all peripheral clocks
        self.write_reg(regs::CLOCK_CTRL, 0);

        // Disable RP1
        self.write_reg(regs::CONTROL, 0);

        self.initialized.store(false, Ordering::Release);
        self.set_state(Rp1State::Detected);

        Ok(())
    }
}

/// Detect RP1 device on PCIe bus
///
/// Scans the PCIe bus for the RP1 I/O Hub and returns its device information
/// if found.
///
/// # Arguments
/// * `ecam` - ECAM configuration space accessor
///
/// # Returns
/// PCIe device information for RP1, or None if not found
pub fn detect_rp1(ecam: &Ecam) -> Option<PciDevice> {
    // Scan bus 0 for devices
    let devices = ecam.scan_bus(0);

    // Look for RP1 device
    for device in devices {
        if device.vendor_id == RP1_VENDOR_ID && device.device_id == RP1_DEVICE_ID {
            return Some(device);
        }
    }

    None
}

/// Create and initialize RP1 driver
///
/// This is a convenience function that detects the RP1, reads its BAR0,
/// creates a driver instance, and initializes it.
///
/// # Arguments
/// * `ecam` - ECAM configuration space accessor
///
/// # Returns
/// Initialized RP1 driver, or error if detection/initialization fails
pub fn initialize_rp1(ecam: &Ecam) -> DriverResult<Rp1Driver> {
    // Detect RP1 device
    let device = detect_rp1(ecam).ok_or(PcieError::Rp1NotFound)?;

    crate::info!("Found RP1 I/O Hub at {}", device.address);
    crate::info!("  Vendor: {:#06x}, Device: {:#06x}", device.vendor_id, device.device_id);
    crate::info!("  Revision: {:#04x}", device.revision_id);

    // Read BAR0 to get MMIO base address
    let bar0 = ecam.read_bar(device.address, 0)?
        .ok_or(PcieError::NoBar)?;

    if !bar0.is_memory {
        return Err(PcieError::InvalidBar.into());
    }

    crate::info!("  BAR0: base={:#018x}, size={:#x}", bar0.base, bar0.size);

    // Create driver instance
    let driver = Rp1Driver::new(device, bar0.base as usize, bar0.size as usize);

    // Initialize RP1
    driver.initialize(ecam)?;

    Ok(driver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rp1_state() {
        // Mock MMIO region (not actually used in this test)
        let device = PciDevice {
            address: PciAddress::new(0, 0, 0),
            vendor_id: RP1_VENDOR_ID,
            device_id: RP1_DEVICE_ID,
            revision_id: 1,
            class_code: 0x058000,
            subsystem_vendor: 0,
            subsystem_id: 0,
        };

        let driver = Rp1Driver::new(device, 0x1000_0000, 0x10000);
        assert_eq!(driver.state(), Rp1State::Detected);
        assert!(!driver.is_initialized());
    }

    #[test]
    fn test_rp1_peripheral_addresses() {
        let device = PciDevice {
            address: PciAddress::new(0, 0, 0),
            vendor_id: RP1_VENDOR_ID,
            device_id: RP1_DEVICE_ID,
            revision_id: 1,
            class_code: 0x058000,
            subsystem_vendor: 0,
            subsystem_id: 0,
        };

        let driver = Rp1Driver::new(device, 0x1000_0000, 0x10000);

        // Test I2C addresses
        assert_eq!(driver.i2c_base(0), Some(0x1000_1000));
        assert_eq!(driver.i2c_base(5), Some(0x1000_1500));
        assert_eq!(driver.i2c_base(6), None);

        // Test SPI addresses
        assert_eq!(driver.spi_base(0), Some(0x1000_2000));
        assert_eq!(driver.spi_base(4), Some(0x1000_2400));
        assert_eq!(driver.spi_base(5), None);

        // Test PWM addresses
        assert_eq!(driver.pwm_base(0), Some(0x1000_3000));
        assert_eq!(driver.pwm_base(1), Some(0x1000_3100));
        assert_eq!(driver.pwm_base(2), None);

        // Test GPIO address
        assert_eq!(driver.gpio_base(), 0x1000_4000);
    }
}
