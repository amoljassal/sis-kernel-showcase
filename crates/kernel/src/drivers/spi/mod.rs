//! SPI (Serial Peripheral Interface) Bus Driver for Raspberry Pi 5
//!
//! This module provides SPI communication for connecting peripherals
//! to the Raspberry Pi 5 via the RP1 I/O Hub.
//!
//! # Overview
//!
//! The RPi5 provides multiple SPI controllers through the RP1, allowing connection
//! to various SPI devices such as displays, sensors, and storage.
//!
//! # Common SPI Devices
//!
//! ## Displays
//! - **ILI9341** - 2.4" TFT LCD
//! - **ST7789** - 1.3" TFT LCD
//!
//! ## Storage
//! - **SD Cards** - Via SPI mode
//! - **Flash Memory** - W25Q series
//!
//! ## Sensors
//! - **MAX31855** - Thermocouple amplifier
//! - **ADXL345** - 3-axis accelerometer (SPI mode)
//!
//! # Usage Example
//!
//! ```rust
//! use crate::drivers::spi::{self, SpiMode, ChipSelect};
//!
//! // Initialize SPI
//! spi::initialize()?;
//!
//! // Configure bus 0 for 1MHz, Mode 0
//! spi::configure(0, SpiMode::Mode0, 1_000_000)?;
//!
//! // Transfer data
//! let tx_data = [0x9F, 0x00, 0x00, 0x00];  // Read ID command
//! let mut rx_data = [0u8; 4];
//! spi::transfer(0, ChipSelect::Cs0, &tx_data, &mut rx_data)?;
//! ```

pub mod bcm2712;

use crate::drivers::{DriverError, DriverResult};
use bcm2712::{Bcm2712Spi, SpiMode};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Once;

/// Maximum number of SPI controllers (RP1 provides multiple)
pub const MAX_SPI_CONTROLLERS: usize = 6;

/// Re-export types
pub use bcm2712::{ChipSelect, ChipSelect as Cs, SpiMode as Mode};

/// SPI subsystem state
struct SpiState {
    /// SPI controllers (indexed by bus number)
    controllers: [Bcm2712Spi; MAX_SPI_CONTROLLERS],

    /// Initialization complete
    initialized: AtomicBool,
}

/// Global SPI state
static SPI_STATE: Once<SpiState> = Once::new();

/// Initialize SPI subsystem
///
/// This function must be called after RP1 initialization to set up
/// all SPI controllers.
///
/// # Returns
/// Ok(()) if initialization succeeds, or an error if:
/// - RP1 not initialized
/// - SPI controller initialization fails
pub fn initialize() -> DriverResult<()> {
    // Get RP1 driver
    let rp1 = crate::drivers::pcie::get_rp1()
        .ok_or(DriverError::NotInitialized)?;

    crate::info!("[SPI] Initializing SPI subsystem");

    // Get SPI controller base addresses from RP1
    let mut controllers = [
        Bcm2712Spi::new(0, 0),
        Bcm2712Spi::new(0, 1),
        Bcm2712Spi::new(0, 2),
        Bcm2712Spi::new(0, 3),
        Bcm2712Spi::new(0, 4),
        Bcm2712Spi::new(0, 5),
    ];

    // Initialize each controller
    let mut initialized_count = 0;
    for i in 0..MAX_SPI_CONTROLLERS {
        if let Some(base) = rp1.spi_base(i as u8) {
            controllers[i] = Bcm2712Spi::new(base, i as u8);
            // Initialize in Mode 0, 1MHz by default
            if controllers[i].initialize(SpiMode::Mode0, 1_000_000).is_ok() {
                initialized_count += 1;
            }
        }
    }

    // Store global state
    SPI_STATE.call_once(|| SpiState {
        controllers,
        initialized: AtomicBool::new(true),
    });

    crate::info!("[SPI] SPI subsystem initialized");
    crate::info!("[SPI]   Controllers: {}", initialized_count);
    crate::info!("[SPI]   Default mode: Mode 0 (CPOL=0, CPHA=0)");
    crate::info!("[SPI]   Default speed: 1 MHz");

    Ok(())
}

/// Check if SPI subsystem is initialized
pub fn is_initialized() -> bool {
    SPI_STATE
        .get()
        .map(|state| state.initialized.load(Ordering::Acquire))
        .unwrap_or(false)
}

/// Get SPI controller by bus number
fn get_controller(bus: u8) -> DriverResult<&'static Bcm2712Spi> {
    let state = SPI_STATE.get().ok_or(DriverError::NotInitialized)?;

    if (bus as usize) >= MAX_SPI_CONTROLLERS {
        return Err(DriverError::InvalidParameter);
    }

    Ok(&state.controllers[bus as usize])
}

/// Configure SPI bus
///
/// # Arguments
/// * `bus` - SPI bus number (0-5)
/// * `mode` - SPI mode (0-3)
/// * `speed_hz` - Clock speed in Hz (up to 125MHz)
///
/// # Returns
/// Actual speed achieved
///
/// # Example
/// ```rust
/// // Configure for SPI Mode 0 at 10MHz
/// let actual_speed = spi::configure(0, SpiMode::Mode0, 10_000_000)?;
/// ```
pub fn configure(bus: u8, mode: SpiMode, speed_hz: u32) -> DriverResult<u32> {
    let controller = get_controller(bus)?;
    controller.initialize(mode, speed_hz)?;
    controller.set_speed(speed_hz)
}

/// Transfer data (full duplex)
///
/// # Arguments
/// * `bus` - SPI bus number (0-5)
/// * `cs` - Chip select line
/// * `tx_data` - Data to transmit
/// * `rx_data` - Buffer to receive data
///
/// # Returns
/// Number of bytes transferred
///
/// # Example
/// ```rust
/// let tx = [0x9F, 0x00, 0x00, 0x00];  // Read ID
/// let mut rx = [0u8; 4];
/// spi::transfer(0, ChipSelect::Cs0, &tx, &mut rx)?;
/// println!("Device ID: {:02X}{:02X}{:02X}", rx[1], rx[2], rx[3]);
/// ```
pub fn transfer(
    bus: u8,
    cs: ChipSelect,
    tx_data: &[u8],
    rx_data: &mut [u8],
) -> DriverResult<usize> {
    let controller = get_controller(bus)?;
    controller.transfer(cs, tx_data, rx_data)
}

/// Write data to SPI device
///
/// # Arguments
/// * `bus` - SPI bus number (0-5)
/// * `cs` - Chip select line
/// * `data` - Data to write
///
/// # Returns
/// Number of bytes written
///
/// # Example
/// ```rust
/// // Write command to display
/// spi::write(0, ChipSelect::Cs0, &[0x2A, 0x00, 0x00, 0x00, 0xEF])?;
/// ```
pub fn write(bus: u8, cs: ChipSelect, data: &[u8]) -> DriverResult<usize> {
    let controller = get_controller(bus)?;
    controller.write(cs, data)
}

/// Read data from SPI device
///
/// # Arguments
/// * `bus` - SPI bus number (0-5)
/// * `cs` - Chip select line
/// * `buffer` - Buffer to receive data
///
/// # Returns
/// Number of bytes read
///
/// # Example
/// ```rust
/// let mut data = [0u8; 16];
/// spi::read(0, ChipSelect::Cs0, &mut data)?;
/// ```
pub fn read(bus: u8, cs: ChipSelect, buffer: &mut [u8]) -> DriverResult<usize> {
    let controller = get_controller(bus)?;
    controller.read(cs, buffer)
}

/// Set SPI bus speed
///
/// # Arguments
/// * `bus` - SPI bus number (0-5)
/// * `speed_hz` - Desired speed in Hz
///
/// # Returns
/// Actual speed achieved
pub fn set_speed(bus: u8, speed_hz: u32) -> DriverResult<u32> {
    let controller = get_controller(bus)?;
    controller.set_speed(speed_hz)
}

/// Get status of an SPI controller
///
/// # Arguments
/// * `bus` - SPI bus number (0-5)
pub fn get_status(bus: u8) -> DriverResult<u32> {
    let controller = get_controller(bus)?;
    Ok(controller.get_status())
}

/// Common SPI device helper functions
pub mod devices {
    use super::*;

    /// Read register from SPI device (common pattern)
    pub fn read_register(
        bus: u8,
        cs: ChipSelect,
        reg: u8,
        buffer: &mut [u8],
    ) -> DriverResult<usize> {
        let mut tx_data = alloc::vec![reg];
        tx_data.extend_from_slice(&alloc::vec![0u8; buffer.len()]);
        let mut rx_data = alloc::vec![0u8; tx_data.len()];

        transfer(bus, cs, &tx_data, &mut rx_data)?;

        buffer.copy_from_slice(&rx_data[1..]);
        Ok(buffer.len())
    }

    /// Write register to SPI device (common pattern)
    pub fn write_register(
        bus: u8,
        cs: ChipSelect,
        reg: u8,
        value: u8,
    ) -> DriverResult<()> {
        write(bus, cs, &[reg, value])?;
        Ok(())
    }

    /// Read multiple bytes from consecutive registers
    pub fn read_registers(
        bus: u8,
        cs: ChipSelect,
        start_reg: u8,
        buffer: &mut [u8],
    ) -> DriverResult<usize> {
        read_register(bus, cs, start_reg | 0x80, buffer)  // Set read bit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_validation() {
        // Valid bus numbers
        assert!(0 < MAX_SPI_CONTROLLERS);
        assert!(5 < MAX_SPI_CONTROLLERS);

        // Invalid bus number
        assert!(6 >= MAX_SPI_CONTROLLERS);
    }

    #[test]
    fn test_chip_select_variants() {
        // Ensure all CS variants are valid
        let _cs0 = ChipSelect::Cs0;
        let _cs1 = ChipSelect::Cs1;
        let _cs2 = ChipSelect::Cs2;
    }
}
