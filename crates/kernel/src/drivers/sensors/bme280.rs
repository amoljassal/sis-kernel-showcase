//! BME280 Environmental Sensor Driver
//!
//! Provides access to temperature, humidity, and pressure data from the BME280
//! I2C sensor commonly used in weather stations and environmental monitoring.

use crate::drivers::{i2c, DriverError, DriverResult};
use core::sync::atomic::{AtomicBool, Ordering};

/// BME280 I2C addresses
pub const BME280_ADDR_LOW: u8 = 0x76;   // SDO to GND
pub const BME280_ADDR_HIGH: u8 = 0x77;  // SDO to VDD

/// BME280 Register addresses
const REG_ID: u8 = 0xD0;
const REG_RESET: u8 = 0xE0;
const REG_CTRL_HUM: u8 = 0xF2;
const REG_STATUS: u8 = 0xF3;
const REG_CTRL_MEAS: u8 = 0xF4;
const REG_CONFIG: u8 = 0xF5;
const REG_PRESS_MSB: u8 = 0xF7;
const REG_TEMP_MSB: u8 = 0xFA;
const REG_HUM_MSB: u8 = 0xFD;

// Calibration data registers
const REG_CALIB_00: u8 = 0x88;  // dig_T1 LSB
const REG_CALIB_26: u8 = 0xE1;  // dig_H2 LSB

/// Expected chip ID
const CHIP_ID_BME280: u8 = 0x60;

/// Oversampling settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oversampling {
    None = 0,
    X1 = 1,
    X2 = 2,
    X4 = 3,
    X8 = 4,
    X16 = 5,
}

/// Operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Sleep = 0,
    Forced = 1,
    Normal = 3,
}

/// Calibration data
#[derive(Debug, Clone, Copy)]
struct CalibrationData {
    // Temperature calibration
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,

    // Pressure calibration
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,

    // Humidity calibration
    dig_h1: u8,
    dig_h2: i16,
    dig_h3: u8,
    dig_h4: i16,
    dig_h5: i16,
    dig_h6: i8,
}

/// Environmental measurements
#[derive(Debug, Clone, Copy)]
pub struct Measurements {
    /// Temperature in °C
    pub temperature: f32,
    /// Pressure in hPa
    pub pressure: f32,
    /// Relative humidity in %
    pub humidity: f32,
}

/// BME280 sensor instance
pub struct Bme280 {
    bus: u8,
    addr: u8,
    calib: Option<CalibrationData>,
    t_fine: i32,
    initialized: AtomicBool,
}

impl Bme280 {
    /// Create a new BME280 instance
    ///
    /// # Arguments
    /// * `bus` - I2C bus number
    /// * `addr` - I2C device address (use BME280_ADDR_LOW or BME280_ADDR_HIGH)
    pub fn new(bus: u8, addr: u8) -> Self {
        Self {
            bus,
            addr,
            calib: None,
            t_fine: 0,
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the sensor
    pub fn initialize(&mut self) -> DriverResult<()> {
        // Check chip ID
        let chip_id = i2c::sensors::read_reg_u8(self.bus, self.addr, REG_ID)?;
        if chip_id != CHIP_ID_BME280 {
            return Err(DriverError::DeviceNotFound);
        }

        // Soft reset
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_RESET, 0xB6)?;
        crate::time::sleep_ms(10);

        // Read calibration data
        self.read_calibration()?;

        // Configure sensor
        // Humidity oversampling x1
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_CTRL_HUM, Oversampling::X1 as u8)?;

        // Temperature oversampling x1, pressure oversampling x1, normal mode
        let ctrl_meas = (Oversampling::X1 as u8) << 5
            | (Oversampling::X1 as u8) << 2
            | Mode::Normal as u8;
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_CTRL_MEAS, ctrl_meas)?;

        // Standby time 0.5ms, filter off
        i2c::sensors::write_reg_u8(self.bus, self.addr, REG_CONFIG, 0x00)?;

        self.initialized.store(true, Ordering::Release);
        Ok(())
    }

    /// Read calibration data from sensor
    fn read_calibration(&mut self) -> DriverResult<()> {
        // Read temperature and pressure calibration (registers 0x88-0x9F)
        let mut tp_calib = [0u8; 24];
        i2c::sensors::read_regs(self.bus, self.addr, REG_CALIB_00, &mut tp_calib)?;

        // Read humidity calibration part 1 (register 0xA1)
        let h1 = i2c::sensors::read_reg_u8(self.bus, self.addr, 0xA1)?;

        // Read humidity calibration part 2 (registers 0xE1-0xE7)
        let mut h_calib = [0u8; 7];
        i2c::sensors::read_regs(self.bus, self.addr, REG_CALIB_26, &mut h_calib)?;

        // Parse calibration data
        let dig_h4 = ((h_calib[3] as i16) << 4) | ((h_calib[4] as i16) & 0x0F);
        let dig_h5 = ((h_calib[5] as i16) << 4) | ((h_calib[4] as i16) >> 4);

        self.calib = Some(CalibrationData {
            dig_t1: u16::from_le_bytes([tp_calib[0], tp_calib[1]]),
            dig_t2: i16::from_le_bytes([tp_calib[2], tp_calib[3]]),
            dig_t3: i16::from_le_bytes([tp_calib[4], tp_calib[5]]),

            dig_p1: u16::from_le_bytes([tp_calib[6], tp_calib[7]]),
            dig_p2: i16::from_le_bytes([tp_calib[8], tp_calib[9]]),
            dig_p3: i16::from_le_bytes([tp_calib[10], tp_calib[11]]),
            dig_p4: i16::from_le_bytes([tp_calib[12], tp_calib[13]]),
            dig_p5: i16::from_le_bytes([tp_calib[14], tp_calib[15]]),
            dig_p6: i16::from_le_bytes([tp_calib[16], tp_calib[17]]),
            dig_p7: i16::from_le_bytes([tp_calib[18], tp_calib[19]]),
            dig_p8: i16::from_le_bytes([tp_calib[20], tp_calib[21]]),
            dig_p9: i16::from_le_bytes([tp_calib[22], tp_calib[23]]),

            dig_h1: h1,
            dig_h2: i16::from_le_bytes([h_calib[0], h_calib[1]]),
            dig_h3: h_calib[2],
            dig_h4,
            dig_h5,
            dig_h6: h_calib[6] as i8,
        });

        Ok(())
    }

    /// Check if sensor is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Compensate temperature (also calculates t_fine for pressure/humidity)
    fn compensate_temperature(&mut self, adc_t: i32) -> f32 {
        let calib = self.calib.as_ref().unwrap();

        let var1 = ((adc_t >> 3) - ((calib.dig_t1 as i32) << 1))
            * (calib.dig_t2 as i32) >> 11;
        let var2 = (((((adc_t >> 4) - (calib.dig_t1 as i32))
            * ((adc_t >> 4) - (calib.dig_t1 as i32))) >> 12)
            * (calib.dig_t3 as i32)) >> 14;

        self.t_fine = var1 + var2;
        let t = (self.t_fine * 5 + 128) >> 8;
        t as f32 / 100.0
    }

    /// Compensate pressure
    fn compensate_pressure(&self, adc_p: i32) -> f32 {
        let calib = self.calib.as_ref().unwrap();

        let mut var1 = (self.t_fine as i64) - 128000;
        let mut var2 = var1 * var1 * (calib.dig_p6 as i64);
        var2 = var2 + ((var1 * (calib.dig_p5 as i64)) << 17);
        var2 = var2 + ((calib.dig_p4 as i64) << 35);
        var1 = ((var1 * var1 * (calib.dig_p3 as i64)) >> 8)
            + ((var1 * (calib.dig_p2 as i64)) << 12);
        var1 = ((1i64 << 47) + var1) * (calib.dig_p1 as i64) >> 33;

        if var1 == 0 {
            return 0.0;
        }

        let mut p = 1048576 - adc_p as i64;
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = ((calib.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        var2 = ((calib.dig_p8 as i64) * p) >> 19;
        p = ((p + var1 + var2) >> 8) + ((calib.dig_p7 as i64) << 4);

        (p as f32) / 25600.0
    }

    /// Compensate humidity
    fn compensate_humidity(&self, adc_h: i32) -> f32 {
        let calib = self.calib.as_ref().unwrap();

        let mut v_x1 = self.t_fine - 76800;
        v_x1 = ((((adc_h << 14) - ((calib.dig_h4 as i32) << 20)
            - ((calib.dig_h5 as i32) * v_x1)) + 16384) >> 15)
            * (((((((v_x1 * (calib.dig_h6 as i32)) >> 10)
                * (((v_x1 * (calib.dig_h3 as i32)) >> 11) + 32768)) >> 10)
                + 2097152) * (calib.dig_h2 as i32) + 8192) >> 14);

        v_x1 = v_x1 - (((((v_x1 >> 15) * (v_x1 >> 15)) >> 7)
            * (calib.dig_h1 as i32)) >> 4);

        v_x1 = if v_x1 < 0 { 0 } else { v_x1 };
        v_x1 = if v_x1 > 419430400 { 419430400 } else { v_x1 };

        (v_x1 >> 12) as f32 / 1024.0
    }

    /// Read all measurements
    pub fn read_measurements(&mut self) -> DriverResult<Measurements> {
        if self.calib.is_none() {
            return Err(DriverError::NotInitialized);
        }

        // Read all data registers (0xF7-0xFE)
        let mut data = [0u8; 8];
        i2c::sensors::read_regs(self.bus, self.addr, REG_PRESS_MSB, &mut data)?;

        // Parse raw values
        let adc_p = ((data[0] as i32) << 12) | ((data[1] as i32) << 4) | ((data[2] as i32) >> 4);
        let adc_t = ((data[3] as i32) << 12) | ((data[4] as i32) << 4) | ((data[5] as i32) >> 4);
        let adc_h = ((data[6] as i32) << 8) | (data[7] as i32);

        // Compensate values
        let temperature = self.compensate_temperature(adc_t);
        let pressure = self.compensate_pressure(adc_p);
        let humidity = self.compensate_humidity(adc_h);

        Ok(Measurements {
            temperature,
            pressure,
            humidity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oversampling_values() {
        assert_eq!(Oversampling::None as u8, 0);
        assert_eq!(Oversampling::X1 as u8, 1);
        assert_eq!(Oversampling::X16 as u8, 5);
    }

    #[test]
    fn test_mode_values() {
        assert_eq!(Mode::Sleep as u8, 0);
        assert_eq!(Mode::Forced as u8, 1);
        assert_eq!(Mode::Normal as u8, 3);
    }
}
