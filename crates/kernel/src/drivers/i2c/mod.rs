//! I2C (Inter-Integrated Circuit) Bus Driver for Raspberry Pi 5
//!
//! This module provides I2C communication for connecting sensors and peripherals
//! to the Raspberry Pi 5. I2C is a two-wire serial protocol commonly used in robotics.
//!
//! # Overview
//!
//! The RPi5 provides 6 I2C controllers through the RP1 I/O Hub, allowing connection
//! to multiple I2C buses simultaneously.
//!
//! # Common I2C Sensors for Robotics
//!
//! ## Inertial Measurement Units (IMU)
//! - **MPU6050** (0x68) - 6-axis gyro + accelerometer
//! - **MPU9250** (0x68) - 9-axis gyro + accel + magnetometer
//! - **BNO055** (0x28) - 9-DOF sensor fusion
//!
//! ## Environmental Sensors
//! - **BME280** (0x76/0x77) - Temperature, humidity, pressure
//! - **BMP280** (0x76/0x77) - Temperature, pressure
//!
//! ## Distance Sensors
//! - **VL53L0X** (0x29) - Time-of-Flight laser ranging
//! - **VL53L1X** (0x29) - Long distance ToF
//!
//! ## Current/Voltage Sensors
//! - **INA219** (0x40-0x4F) - Current, voltage, power
//!
//! # Usage Example
//!
//! ```rust
//! use crate::drivers::i2c;
//!
//! // Initialize I2C bus 0
//! i2c::initialize()?;
//!
//! // Scan for devices
//! let devices = i2c::scan(0)?;
//! for addr in devices {
//!     println!("Found device at 0x{:02X}", addr);
//! }
//!
//! // Read from MPU6050 WHO_AM_I register (0x75)
//! let mut who_am_i = [0u8; 1];
//! i2c::write_read(0, 0x68, 0x75, &mut who_am_i)?;
//! assert_eq!(who_am_i[0], 0x68); // MPU6050 ID
//!
//! // Write configuration
//! i2c::write(0, 0x68, &[0x6B, 0x00])?; // Wake up MPU6050
//! ```

pub mod bcm2712;

use crate::drivers::{DriverError, DriverResult};
use bcm2712::{Bcm2712I2c, I2cMode};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Once;

/// Maximum number of I2C controllers (RP1 provides 6)
pub const MAX_I2C_CONTROLLERS: usize = 6;

/// I2C subsystem state
struct I2cState {
    /// I2C controllers (indexed by bus number)
    controllers: [Bcm2712I2c; MAX_I2C_CONTROLLERS],

    /// Initialization complete
    initialized: AtomicBool,
}

/// Global I2C state
static I2C_STATE: Once<I2cState> = Once::new();

/// Initialize I2C subsystem
///
/// This function must be called after RP1 initialization to set up
/// all I2C controllers. Controllers are initialized in Fast mode (400 kHz)
/// by default.
///
/// # Returns
/// Ok(()) if initialization succeeds, or an error if:
/// - RP1 not initialized
/// - I2C controller initialization fails
pub fn initialize() -> DriverResult<()> {
    // Get RP1 driver
    let rp1 = crate::drivers::pcie::get_rp1()
        .ok_or(DriverError::NotInitialized)?;

    crate::info!("[I2C] Initializing I2C subsystem");

    // Get I2C controller base addresses from RP1
    let mut controllers = [
        Bcm2712I2c::new(0, 0),
        Bcm2712I2c::new(0, 1),
        Bcm2712I2c::new(0, 2),
        Bcm2712I2c::new(0, 3),
        Bcm2712I2c::new(0, 4),
        Bcm2712I2c::new(0, 5),
    ];

    // Initialize each controller
    for i in 0..MAX_I2C_CONTROLLERS {
        if let Some(base) = rp1.i2c_base(i as u8) {
            controllers[i] = Bcm2712I2c::new(base, i as u8);
            // Initialize in Fast mode (400 kHz) by default
            controllers[i].initialize(I2cMode::Fast)?;
        }
    }

    // Store global state
    I2C_STATE.call_once(|| I2cState {
        controllers,
        initialized: AtomicBool::new(true),
    });

    crate::info!("[I2C] I2C subsystem initialized");
    crate::info!("[I2C]   Controllers: {}", MAX_I2C_CONTROLLERS);
    crate::info!("[I2C]   Default mode: Fast (400 kHz)");

    Ok(())
}

/// Check if I2C subsystem is initialized
pub fn is_initialized() -> bool {
    I2C_STATE
        .get()
        .map(|state| state.initialized.load(Ordering::Acquire))
        .unwrap_or(false)
}

/// Get I2C controller by bus number
fn get_controller(bus: u8) -> DriverResult<&'static Bcm2712I2c> {
    let state = I2C_STATE.get().ok_or(DriverError::NotInitialized)?;

    if (bus as usize) >= MAX_I2C_CONTROLLERS {
        return Err(DriverError::InvalidParameter);
    }

    Ok(&state.controllers[bus as usize])
}

/// Write data to I2C device
///
/// # Arguments
/// * `bus` - I2C bus number (0-5)
/// * `addr` - 7-bit device address
/// * `data` - Data to write
///
/// # Returns
/// Number of bytes written
///
/// # Example
/// ```rust
/// // Write to register 0x6B with value 0x00
/// i2c::write(0, 0x68, &[0x6B, 0x00])?;
/// ```
pub fn write(bus: u8, addr: u8, data: &[u8]) -> DriverResult<usize> {
    let controller = get_controller(bus)?;
    controller.write(addr, data)
}

/// Read data from I2C device
///
/// # Arguments
/// * `bus` - I2C bus number (0-5)
/// * `addr` - 7-bit device address
/// * `buffer` - Buffer to receive data
///
/// # Returns
/// Number of bytes read
///
/// # Example
/// ```rust
/// let mut data = [0u8; 6];
/// i2c::read(0, 0x68, &mut data)?;
/// ```
pub fn read(bus: u8, addr: u8, buffer: &mut [u8]) -> DriverResult<usize> {
    let controller = get_controller(bus)?;
    controller.read(addr, buffer)
}

/// Write register address then read data (common I2C pattern)
///
/// This is the most common I2C transaction for reading sensor registers.
///
/// # Arguments
/// * `bus` - I2C bus number (0-5)
/// * `addr` - 7-bit device address
/// * `reg` - Register address to read from
/// * `buffer` - Buffer to receive data
///
/// # Returns
/// Number of bytes read
///
/// # Example
/// ```rust
/// // Read WHO_AM_I register (0x75) from MPU6050 (0x68)
/// let mut who_am_i = [0u8; 1];
/// i2c::write_read(0, 0x68, 0x75, &mut who_am_i)?;
/// ```
pub fn write_read(bus: u8, addr: u8, reg: u8, buffer: &mut [u8]) -> DriverResult<usize> {
    let controller = get_controller(bus)?;
    controller.write_read(addr, reg, buffer)
}

/// Scan I2C bus for devices
///
/// Scans all valid 7-bit addresses (0x08 to 0x77) and returns addresses
/// that respond with ACK.
///
/// # Arguments
/// * `bus` - I2C bus number (0-5)
///
/// # Returns
/// Vector of device addresses found
///
/// # Example
/// ```rust
/// let devices = i2c::scan(0)?;
/// for addr in devices {
///     println!("Found device at 0x{:02X}", addr);
/// }
/// ```
pub fn scan(bus: u8) -> DriverResult<alloc::vec::Vec<u8>> {
    let controller = get_controller(bus)?;
    Ok(controller.scan())
}

/// Get status of an I2C controller
///
/// # Arguments
/// * `bus` - I2C bus number (0-5)
pub fn get_status(bus: u8) -> DriverResult<u32> {
    let controller = get_controller(bus)?;
    Ok(controller.get_status())
}

/// Clear error flags on an I2C controller
///
/// # Arguments
/// * `bus` - I2C bus number (0-5)
pub fn clear_errors(bus: u8) -> DriverResult<()> {
    let controller = get_controller(bus)?;
    controller.clear_errors()
}

/// Helper functions for common sensor operations
pub mod sensors {
    use super::*;

    /// Read a single 8-bit register
    pub fn read_reg_u8(bus: u8, addr: u8, reg: u8) -> DriverResult<u8> {
        let mut data = [0u8; 1];
        write_read(bus, addr, reg, &mut data)?;
        Ok(data[0])
    }

    /// Read a 16-bit register (big-endian)
    pub fn read_reg_u16_be(bus: u8, addr: u8, reg: u8) -> DriverResult<u16> {
        let mut data = [0u8; 2];
        write_read(bus, addr, reg, &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    /// Read a 16-bit register (little-endian)
    pub fn read_reg_u16_le(bus: u8, addr: u8, reg: u8) -> DriverResult<u16> {
        let mut data = [0u8; 2];
        write_read(bus, addr, reg, &mut data)?;
        Ok(u16::from_le_bytes(data))
    }

    /// Write a single 8-bit register
    pub fn write_reg_u8(bus: u8, addr: u8, reg: u8, value: u8) -> DriverResult<()> {
        write(bus, addr, &[reg, value])?;
        Ok(())
    }

    /// Write a 16-bit register (big-endian)
    pub fn write_reg_u16_be(bus: u8, addr: u8, reg: u8, value: u16) -> DriverResult<()> {
        let bytes = value.to_be_bytes();
        write(bus, addr, &[reg, bytes[0], bytes[1]])?;
        Ok(())
    }

    /// Write a 16-bit register (little-endian)
    pub fn write_reg_u16_le(bus: u8, addr: u8, reg: u8, value: u16) -> DriverResult<()> {
        let bytes = value.to_le_bytes();
        write(bus, addr, &[reg, bytes[0], bytes[1]])?;
        Ok(())
    }

    /// Read multiple bytes from consecutive registers
    pub fn read_regs(bus: u8, addr: u8, start_reg: u8, buffer: &mut [u8]) -> DriverResult<usize> {
        write_read(bus, addr, start_reg, buffer)
    }
}

/// Common I2C device addresses
pub mod addresses {
    /// MPU6050 IMU (6-axis)
    pub const MPU6050: u8 = 0x68;

    /// MPU9250 IMU (9-axis)
    pub const MPU9250: u8 = 0x68;

    /// BNO055 IMU (9-DOF with sensor fusion)
    pub const BNO055: u8 = 0x28;

    /// BME280 environmental sensor (default)
    pub const BME280: u8 = 0x76;

    /// BME280 environmental sensor (alternate)
    pub const BME280_ALT: u8 = 0x77;

    /// BMP280 pressure sensor (default)
    pub const BMP280: u8 = 0x76;

    /// BMP280 pressure sensor (alternate)
    pub const BMP280_ALT: u8 = 0x77;

    /// VL53L0X ToF distance sensor
    pub const VL53L0X: u8 = 0x29;

    /// VL53L1X ToF distance sensor
    pub const VL53L1X: u8 = 0x29;

    /// INA219 current/voltage sensor (default)
    pub const INA219: u8 = 0x40;

    /// DS1307 RTC
    pub const DS1307: u8 = 0x68;

    /// DS3231 RTC
    pub const DS3231: u8 = 0x68;

    /// AT24C32 EEPROM (base address)
    pub const AT24C32: u8 = 0x50;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_validation() {
        // Valid bus numbers
        assert!(0 < MAX_I2C_CONTROLLERS);
        assert!(5 < MAX_I2C_CONTROLLERS);

        // Invalid bus number
        assert!(6 >= MAX_I2C_CONTROLLERS);
    }

    #[test]
    fn test_address_constants() {
        // Verify common addresses are in valid range
        assert!(addresses::MPU6050 <= 0x7F);
        assert!(addresses::BNO055 <= 0x7F);
        assert!(addresses::BME280 <= 0x7F);
        assert!(addresses::VL53L0X <= 0x7F);
    }
}
