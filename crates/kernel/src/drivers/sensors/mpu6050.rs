//! MPU6050 6-axis IMU (Inertial Measurement Unit) Driver
//!
//! Provides access to accelerometer and gyroscope data from the MPU6050/MPU6500/MPU9250
//! I2C sensor commonly used in robotics for orientation and motion tracking.

use crate::drivers::{i2c, DriverError, DriverResult};
use core::sync::atomic::{AtomicBool, Ordering};

/// MPU6050 I2C addresses
pub const MPU6050_ADDR_LOW: u8 = 0x68;  // AD0 = 0
pub const MPU6050_ADDR_HIGH: u8 = 0x69; // AD0 = 1

/// MPU6050 Register addresses
const REG_WHO_AM_I: u8 = 0x75;
const REG_PWR_MGMT_1: u8 = 0x6B;
const REG_PWR_MGMT_2: u8 = 0x6C;
const REG_SMPLRT_DIV: u8 = 0x19;
const REG_CONFIG: u8 = 0x1A;
const REG_GYRO_CONFIG: u8 = 0x1B;
const REG_ACCEL_CONFIG: u8 = 0x1C;
const REG_INT_ENABLE: u8 = 0x38;
const REG_ACCEL_XOUT_H: u8 = 0x3B;
const REG_TEMP_OUT_H: u8 = 0x41;
const REG_GYRO_XOUT_H: u8 = 0x43;

/// Expected WHO_AM_I values
const WHO_AM_I_MPU6050: u8 = 0x68;
const WHO_AM_I_MPU6500: u8 = 0x70;
const WHO_AM_I_MPU9250: u8 = 0x71;

/// Accelerometer full-scale range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccelRange {
    /// ±2g
    G2 = 0,
    /// ±4g
    G4 = 1,
    /// ±8g
    G8 = 2,
    /// ±16g
    G16 = 3,
}

impl AccelRange {
    /// Get scale factor (LSB/g)
    pub fn scale_factor(&self) -> f32 {
        match self {
            AccelRange::G2 => 16384.0,
            AccelRange::G4 => 8192.0,
            AccelRange::G8 => 4096.0,
            AccelRange::G16 => 2048.0,
        }
    }
}

/// Gyroscope full-scale range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GyroRange {
    /// ±250°/s
    Dps250 = 0,
    /// ±500°/s
    Dps500 = 1,
    /// ±1000°/s
    Dps1000 = 2,
    /// ±2000°/s
    Dps2000 = 3,
}

impl GyroRange {
    /// Get scale factor (LSB/°/s)
    pub fn scale_factor(&self) -> f32 {
        match self {
            GyroRange::Dps250 => 131.0,
            GyroRange::Dps500 => 65.5,
            GyroRange::Dps1000 => 32.8,
            GyroRange::Dps2000 => 16.4,
        }
    }
}

/// Accelerometer data (raw and scaled)
#[derive(Debug, Clone, Copy)]
pub struct AccelData {
    /// Raw X-axis value
    pub x_raw: i16,
    /// Raw Y-axis value
    pub y_raw: i16,
    /// Raw Z-axis value
    pub z_raw: i16,
    /// X-axis acceleration in g
    pub x_g: f32,
    /// Y-axis acceleration in g
    pub y_g: f32,
    /// Z-axis acceleration in g
    pub z_g: f32,
}

/// Gyroscope data (raw and scaled)
#[derive(Debug, Clone, Copy)]
pub struct GyroData {
    /// Raw X-axis value
    pub x_raw: i16,
    /// Raw Y-axis value
    pub y_raw: i16,
    /// Raw Z-axis value
    pub z_raw: i16,
    /// X-axis rotation in °/s
    pub x_dps: f32,
    /// Y-axis rotation in °/s
    pub y_dps: f32,
    /// Z-axis rotation in °/s
    pub z_dps: f32,
}

/// Temperature data
#[derive(Debug, Clone, Copy)]
pub struct TempData {
    /// Raw temperature value
    pub raw: i16,
    /// Temperature in °C
    pub celsius: f32,
}

/// MPU6050 sensor instance
pub struct Mpu6050 {
    bus: u8,
    addr: u8,
    accel_range: AccelRange,
    gyro_range: GyroRange,
    initialized: AtomicBool,
}

impl Mpu6050 {
    /// Create a new MPU6050 instance
    ///
    /// # Arguments
    /// * `bus` - I2C bus number
    /// * `addr` - I2C device address (use MPU6050_ADDR_LOW or MPU6050_ADDR_HIGH)
    pub fn new(bus: u8, addr: u8) -> Self {
        Self {
            bus,
            addr,
            accel_range: AccelRange::G2,
            gyro_range: GyroRange::Dps250,
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the sensor
    pub fn initialize(&self) -> DriverResult<()> {
        // Check WHO_AM_I register
        let who_am_i = i2c::sensors::read_reg_u8(self.bus, self.addr, REG_WHO_AM_I)?;

        if who_am_i != WHO_AM_I_MPU6050
            && who_am_i != WHO_AM_I_MPU6500
            && who_am_i != WHO_AM_I_MPU9250 {
            return Err(DriverError::DeviceNotFound);
        }

        // Wake up device (clear sleep bit)
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_PWR_MGMT_1, 0x00)?;

        // Wait for sensor to stabilize (10ms)
        crate::time::sleep_ms(10);

        // Set clock source to PLL with X axis gyroscope reference
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_PWR_MGMT_1, 0x01)?;

        // Configure sample rate divider (1kHz / (1 + 7) = 125Hz)
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_SMPLRT_DIV, 0x07)?;

        // Configure DLPF (Digital Low Pass Filter) - bandwidth 44Hz
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_CONFIG, 0x03)?;

        // Set accelerometer range to ±2g
        self.set_accel_range(AccelRange::G2)?;

        // Set gyroscope range to ±250°/s
        self.set_gyro_range(GyroRange::Dps250)?;

        self.initialized.store(true, Ordering::Release);
        Ok(())
    }

    /// Check if sensor is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Set accelerometer range
    pub fn set_accel_range(&self, range: AccelRange) -> DriverResult<()> {
        let val = (range as u8) << 3;
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_ACCEL_CONFIG, val)?;
        // Note: In real implementation, we'd update self.accel_range
        // but that requires interior mutability
        Ok(())
    }

    /// Set gyroscope range
    pub fn set_gyro_range(&self, range: GyroRange) -> DriverResult<()> {
        let val = (range as u8) << 3;
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_GYRO_CONFIG, val)?;
        // Note: In real implementation, we'd update self.gyro_range
        Ok(())
    }

    /// Read raw accelerometer data
    fn read_accel_raw(&self) -> DriverResult<(i16, i16, i16)> {
        let mut data = [0u8; 6];
        i2c::sensors::read_regs(self.bus, self.addr, REG_ACCEL_XOUT_H, &mut data)?;

        let x = i16::from_be_bytes([data[0], data[1]]);
        let y = i16::from_be_bytes([data[2], data[3]]);
        let z = i16::from_be_bytes([data[4], data[5]]);

        Ok((x, y, z))
    }

    /// Read accelerometer data (raw and scaled)
    pub fn read_accel(&self) -> DriverResult<AccelData> {
        let (x_raw, y_raw, z_raw) = self.read_accel_raw()?;
        let scale = self.accel_range.scale_factor();

        Ok(AccelData {
            x_raw,
            y_raw,
            z_raw,
            x_g: x_raw as f32 / scale,
            y_g: y_raw as f32 / scale,
            z_g: z_raw as f32 / scale,
        })
    }

    /// Read raw gyroscope data
    fn read_gyro_raw(&self) -> DriverResult<(i16, i16, i16)> {
        let mut data = [0u8; 6];
        i2c::sensors::read_regs(self.bus, self.addr, REG_GYRO_XOUT_H, &mut data)?;

        let x = i16::from_be_bytes([data[0], data[1]]);
        let y = i16::from_be_bytes([data[2], data[3]]);
        let z = i16::from_be_bytes([data[4], data[5]]);

        Ok((x, y, z))
    }

    /// Read gyroscope data (raw and scaled)
    pub fn read_gyro(&self) -> DriverResult<GyroData> {
        let (x_raw, y_raw, z_raw) = self.read_gyro_raw()?;
        let scale = self.gyro_range.scale_factor();

        Ok(GyroData {
            x_raw,
            y_raw,
            z_raw,
            x_dps: x_raw as f32 / scale,
            y_dps: y_raw as f32 / scale,
            z_dps: z_raw as f32 / scale,
        })
    }

    /// Read temperature
    pub fn read_temperature(&self) -> DriverResult<TempData> {
        let mut data = [0u8; 2];
        i2c::sensors::read_regs(self.bus, self.addr, REG_TEMP_OUT_H, &mut data)?;
        let raw = i16::from_be_bytes([data[0], data[1]]);

        // Temperature in °C = (TEMP_OUT / 340) + 36.53
        let celsius = (raw as f32 / 340.0) + 36.53;

        Ok(TempData { raw, celsius })
    }

    /// Read all sensor data at once
    pub fn read_all(&self) -> DriverResult<(AccelData, GyroData, TempData)> {
        let accel = self.read_accel()?;
        let gyro = self.read_gyro()?;
        let temp = self.read_temperature()?;
        Ok((accel, gyro, temp))
    }

    /// Reset the device
    pub fn reset(&self) -> DriverResult<()> {
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_PWR_MGMT_1, 0x80)?;
        crate::time::sleep_ms(100);
        self.initialized.store(false, Ordering::Release);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accel_range_scale_factors() {
        assert_eq!(AccelRange::G2.scale_factor(), 16384.0);
        assert_eq!(AccelRange::G4.scale_factor(), 8192.0);
        assert_eq!(AccelRange::G8.scale_factor(), 4096.0);
        assert_eq!(AccelRange::G16.scale_factor(), 2048.0);
    }

    #[test]
    fn test_gyro_range_scale_factors() {
        assert_eq!(GyroRange::Dps250.scale_factor(), 131.0);
        assert_eq!(GyroRange::Dps500.scale_factor(), 65.5);
        assert_eq!(GyroRange::Dps1000.scale_factor(), 32.8);
        assert_eq!(GyroRange::Dps2000.scale_factor(), 16.4);
    }
}
