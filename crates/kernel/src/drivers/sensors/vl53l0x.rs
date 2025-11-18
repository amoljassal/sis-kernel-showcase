//! VL53L0X Time-of-Flight Distance Sensor Driver
//!
//! Provides ranging measurements from the VL53L0X laser-based ToF sensor
//! for precise distance measurement up to 2 meters.

use crate::drivers::{i2c, DriverError, DriverResult};
use core::sync::atomic::{AtomicBool, Ordering};

/// VL53L0X default I2C address
pub const VL53L0X_ADDR: u8 = 0x29;

/// VL53L0X Register addresses
const REG_IDENTIFICATION_MODEL_ID: u8 = 0xC0;
const REG_VHV_CONFIG_PAD_SCL_SDA: u8 = 0x89;
const REG_SYSRANGE_START: u8 = 0x00;
const REG_RESULT_INTERRUPT_STATUS: u8 = 0x13;
const REG_RESULT_RANGE_STATUS: u8 = 0x14;

/// Expected model ID
const MODEL_ID_VL53L0X: u8 = 0xEE;

/// Range profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeProfile {
    /// Default mode (up to 1.2m)
    Default,
    /// High accuracy mode (slower, up to 1.2m)
    HighAccuracy,
    /// Long range mode (up to 2m, lower accuracy)
    LongRange,
    /// High speed mode (faster, up to 1.2m)
    HighSpeed,
}

/// Range measurement
#[derive(Debug, Clone, Copy)]
pub struct RangeMeasurement {
    /// Distance in millimeters
    pub distance_mm: u16,
    /// Measurement valid
    pub valid: bool,
}

/// VL53L0X sensor instance
pub struct Vl53l0x {
    bus: u8,
    addr: u8,
    initialized: AtomicBool,
}

impl Vl53l0x {
    /// Create a new VL53L0X instance
    ///
    /// # Arguments
    /// * `bus` - I2C bus number
    /// * `addr` - I2C device address (usually VL53L0X_ADDR)
    pub fn new(bus: u8, addr: u8) -> Self {
        Self {
            bus,
            addr,
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the sensor
    pub fn initialize(&self) -> DriverResult<()> {
        // Check model ID
        let model_id = i2c::sensors::read_reg_u8(self.bus, self.addr, REG_IDENTIFICATION_MODEL_ID)?;
        if model_id != MODEL_ID_VL53L0X {
            return Err(DriverError::DeviceNotFound);
        }

        // Basic initialization sequence
        // Note: Full VL53L0X initialization is very complex and requires
        // loading calibration data, tuning parameters, etc.
        // This is a simplified version for basic ranging.

        // Set I2C standard mode
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x88, 0x00)?;

        // Set 2V8 mode
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x80, 0x01)?;
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0xFF, 0x01)?;
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x00, 0x00)?;
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0xFF, 0x00)?;
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x80, 0x00)?;

        // Disable SIGNAL_RATE_MSRC and SIGNAL_RATE_PRE_RANGE limit checks
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x60, 0x00)?;

        // Set signal rate limit to 0.25 MCPS
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x44, 0x00)?;
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x45, 0x20)?;

        // Set timing budget
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x46, 0x01)?;
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x47, 0xFF)?;

        self.initialized.store(true, Ordering::Release);
        Ok(())
    }

    /// Check if sensor is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Start a single range measurement
    pub fn start_range(&self) -> DriverResult<()> {
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_SYSRANGE_START, 0x01)?;
        Ok(())
    }

    /// Check if range measurement is ready
    pub fn is_range_complete(&self) -> DriverResult<bool> {
        let status = i2c::sensors::read_reg_u8(self.bus, self.addr, REG_RESULT_INTERRUPT_STATUS)?;
        Ok((status & 0x07) != 0)
    }

    /// Read range measurement (blocking)
    pub fn read_range_mm(&self) -> DriverResult<RangeMeasurement> {
        // Start measurement
        self.start_range()?;

        // Wait for measurement to complete (with timeout)
        let mut timeout = 1000;
        while timeout > 0 {
            if self.is_range_complete()? {
                break;
            }
            crate::time::sleep_ms(1);
            timeout -= 1;
        }

        if timeout == 0 {
            let timeout_err = crate::drivers::timeout::TimeoutError::new(1000, 1000);
            return Err(DriverError::Timeout(timeout_err));
        }

        // Read range value (mm)
        let mut range_data = [0u8; 2];
        i2c::sensors::read_regs(self.bus, self.addr, REG_RESULT_RANGE_STATUS + 10, &mut range_data)?;
        let distance_mm = u16::from_be_bytes([range_data[0], range_data[1]]);

        // Clear interrupt
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x0B, 0x01)?;

        // Check if measurement is valid
        let range_status = i2c::sensors::read_reg_u8(self.bus, self.addr, REG_RESULT_RANGE_STATUS)?;
        let valid = (range_status & 0x78) == 0;

        Ok(RangeMeasurement {
            distance_mm,
            valid,
        })
    }

    /// Read range in continuous mode (non-blocking)
    pub fn read_range_continuous(&self) -> DriverResult<Option<RangeMeasurement>> {
        if !self.is_range_complete()? {
            return Ok(None);
        }

        // Read range value
        let mut range_data = [0u8; 2];
        i2c::sensors::read_regs(self.bus, self.addr, REG_RESULT_RANGE_STATUS + 10, &mut range_data)?;
        let distance_mm = u16::from_be_bytes([range_data[0], range_data[1]]);

        // Check if measurement is valid
        let range_status = i2c::sensors::read_reg_u8(self.bus, self.addr, REG_RESULT_RANGE_STATUS)?;
        let valid = (range_status & 0x78) == 0;

        // Clear interrupt
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x0B, 0x01)?;

        Ok(Some(RangeMeasurement {
            distance_mm,
            valid,
        }))
    }

    /// Start continuous ranging mode
    pub fn start_continuous(&self, period_ms: u32) -> DriverResult<()> {
        // Set inter-measurement period
        let period_bytes = period_ms.to_be_bytes();
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x04, period_bytes[2])?;
        i2c::sensors::write_reg_u8(self.bus, self.addr, 0x05, period_bytes[3])?;

        // Start continuous mode
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_SYSRANGE_START, 0x02)?;

        Ok(())
    }

    /// Stop continuous ranging mode
    pub fn stop_continuous(&self) -> DriverResult<()> {
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_SYSRANGE_START, 0x01)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vl53l0x_constants() {
        assert_eq!(VL53L0X_ADDR, 0x29);
        assert_eq!(MODEL_ID_VL53L0X, 0xEE);
    }
}
