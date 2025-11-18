//! Sensor Drivers for Robotics Applications
//!
//! This module provides drivers for common I2C sensors used in robotics:
//! - IMU sensors (MPU6050) for orientation and motion tracking
//! - Environmental sensors (BME280) for weather monitoring
//! - Distance sensors (VL53L0X) for obstacle detection and navigation
//!
//! # Common Sensor Usage Patterns
//!
//! ## IMU Sensor (MPU6050)
//! ```rust
//! use crate::drivers::sensors::{mpu6050, Mpu6050};
//!
//! let mut imu = Mpu6050::new(1, mpu6050::MPU6050_ADDR_LOW);
//! imu.initialize()?;
//!
//! // Read accelerometer data
//! let accel = imu.read_accel()?;
//! println!("Accel: x={:.2}g y={:.2}g z={:.2}g",
//!          accel.x_g, accel.y_g, accel.z_g);
//!
//! // Read gyroscope data
//! let gyro = imu.read_gyro()?;
//! println!("Gyro: x={:.2}°/s y={:.2}°/s z={:.2}°/s",
//!          gyro.x_dps, gyro.y_dps, gyro.z_dps);
//! ```
//!
//! ## Environmental Sensor (BME280)
//! ```rust
//! use crate::drivers::sensors::{bme280, Bme280};
//!
//! let mut env = Bme280::new(1, bme280::BME280_ADDR_LOW);
//! env.initialize()?;
//!
//! let data = env.read_measurements()?;
//! println!("Temp: {:.1}°C  Humidity: {:.1}%  Pressure: {:.1}hPa",
//!          data.temperature, data.humidity, data.pressure);
//! ```
//!
//! ## Distance Sensor (VL53L0X)
//! ```rust
//! use crate::drivers::sensors::{vl53l0x, Vl53l0x};
//!
//! let tof = Vl53l0x::new(1, vl53l0x::VL53L0X_ADDR);
//! tof.initialize()?;
//!
//! let range = tof.read_range_mm()?;
//! if range.valid {
//!     println!("Distance: {}mm", range.distance_mm);
//! }
//! ```

pub mod mpu6050;
pub mod bme280;
pub mod vl53l0x;

// Re-export sensor types for convenience
pub use mpu6050::Mpu6050;
pub use bme280::Bme280;
pub use vl53l0x::Vl53l0x;

/// Common sensor addresses for quick reference
pub mod addresses {
    /// MPU6050/MPU6500/MPU9250 IMU
    pub const MPU6050_LOW: u8 = 0x68;
    pub const MPU6050_HIGH: u8 = 0x69;

    /// BME280 Environmental Sensor
    pub const BME280_LOW: u8 = 0x76;
    pub const BME280_HIGH: u8 = 0x77;

    /// VL53L0X Time-of-Flight
    pub const VL53L0X: u8 = 0x29;

    /// BNO055 9-DOF IMU
    pub const BNO055_A: u8 = 0x28;
    pub const BNO055_B: u8 = 0x29;

    /// INA219 Power Monitor
    pub const INA219_BASE: u8 = 0x40;

    /// AT24C EEPROM (range 0x50-0x57)
    pub const EEPROM_BASE: u8 = 0x50;
}

#[cfg(test)]
mod tests {
    use super::addresses::*;

    #[test]
    fn test_sensor_addresses() {
        assert_eq!(MPU6050_LOW, 0x68);
        assert_eq!(BME280_LOW, 0x76);
        assert_eq!(VL53L0X, 0x29);
    }
}
