//! Shell command helpers for sensor control and reading
//!
//! Provides interactive commands for testing and reading data from sensors.

use crate::drivers::sensors::{bme280, mpu6050, vl53l0x};

impl super::Shell {
    /// Sensor command handler
    ///
    /// Usage:
    ///   sensor                    - Show available sensors
    ///   sensor mpu6050 <bus>      - Read MPU6050 IMU data
    ///   sensor bme280 <bus>       - Read BME280 environmental data
    ///   sensor vl53l0x <bus>      - Read VL53L0X distance
    ///   sensor scan <bus>         - Scan for known sensors
    pub(crate) fn sensor_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            self.sensor_help_cmd();
            return;
        }

        match args[0] {
            "mpu6050" => {
                if args.len() < 2 {
                    unsafe {
                        crate::uart_print(b"Usage: sensor mpu6050 <bus> [addr]\n");
                        crate::uart_print(b"Example: sensor mpu6050 1\n");
                        crate::uart_print(b"  Default addr: 0x68\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let addr = if args.len() >= 3 {
                    self.parse_number(args[2].as_bytes()).unwrap_or(mpu6050::MPU6050_ADDR_LOW as u32) as u8
                } else {
                    mpu6050::MPU6050_ADDR_LOW
                };

                self.sensor_mpu6050_cmd(bus, addr);
            }
            "bme280" => {
                if args.len() < 2 {
                    unsafe {
                        crate::uart_print(b"Usage: sensor bme280 <bus> [addr]\n");
                        crate::uart_print(b"Example: sensor bme280 1\n");
                        crate::uart_print(b"  Default addr: 0x76\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let addr = if args.len() >= 3 {
                    self.parse_number(args[2].as_bytes()).unwrap_or(bme280::BME280_ADDR_LOW as u32) as u8
                } else {
                    bme280::BME280_ADDR_LOW
                };

                self.sensor_bme280_cmd(bus, addr);
            }
            "vl53l0x" => {
                if args.len() < 2 {
                    unsafe {
                        crate::uart_print(b"Usage: sensor vl53l0x <bus> [addr]\n");
                        crate::uart_print(b"Example: sensor vl53l0x 1\n");
                        crate::uart_print(b"  Default addr: 0x29\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let addr = if args.len() >= 3 {
                    self.parse_number(args[2].as_bytes()).unwrap_or(vl53l0x::VL53L0X_ADDR as u32) as u8
                } else {
                    vl53l0x::VL53L0X_ADDR
                };

                self.sensor_vl53l0x_cmd(bus, addr);
            }
            "scan" => {
                if args.len() < 2 {
                    unsafe {
                        crate::uart_print(b"Usage: sensor scan <bus>\n");
                        crate::uart_print(b"Example: sensor scan 1\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                self.sensor_scan_cmd(bus);
            }
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown sensor: ");
                    crate::uart_print(args[0].as_bytes());
                    crate::uart_print(b"\n");
                    crate::uart_print(b"Available: mpu6050, bme280, vl53l0x, scan\n");
                }
            }
        }
    }

    /// Show sensor help
    fn sensor_help_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n=== Sensor Subsystem ===\n\n");
            crate::uart_print(b"Available Sensors:\n");
            crate::uart_print(b"  mpu6050  - MPU6050/6500/9250 6-axis IMU (accel + gyro)\n");
            crate::uart_print(b"  bme280   - BME280 environmental sensor (temp + humidity + pressure)\n");
            crate::uart_print(b"  vl53l0x  - VL53L0X time-of-flight distance sensor\n\n");

            crate::uart_print(b"Commands:\n");
            crate::uart_print(b"  sensor mpu6050 <bus>   - Read IMU data\n");
            crate::uart_print(b"  sensor bme280 <bus>    - Read environmental data\n");
            crate::uart_print(b"  sensor vl53l0x <bus>   - Read distance\n");
            crate::uart_print(b"  sensor scan <bus>      - Scan for known sensors\n\n");

            crate::uart_print(b"Common Addresses:\n");
            crate::uart_print(b"  MPU6050:  0x68 (AD0=0) or 0x69 (AD0=1)\n");
            crate::uart_print(b"  BME280:   0x76 (SDO=0) or 0x77 (SDO=1)\n");
            crate::uart_print(b"  VL53L0X:  0x29\n\n");
        }
    }

    /// Read MPU6050 sensor
    fn sensor_mpu6050_cmd(&self, bus: u8, addr: u8) {
        use crate::drivers::sensors::Mpu6050;

        let mut sensor = Mpu6050::new(bus, addr);

        unsafe {
            crate::uart_print(b"[MPU6050] Initializing sensor on bus ");
            self.print_number_simple(bus as u64);
            crate::uart_print(b" at address 0x");
            self.print_sensor_hex8(addr);
            crate::uart_print(b"...\n");
        }

        match sensor.initialize() {
            Ok(()) => {
                unsafe {
                    crate::uart_print(b"[MPU6050] Initialized successfully\n\n");
                }

                // Read accelerometer
                match sensor.read_accel() {
                    Ok(accel) => {
                        unsafe {
                            crate::uart_print(b"Accelerometer:\n");
                            crate::uart_print(b"  X: ");
                            self.print_sensor_float(accel.x_g, 2);
                            crate::uart_print(b" g\n");
                            crate::uart_print(b"  Y: ");
                            self.print_sensor_float(accel.y_g, 2);
                            crate::uart_print(b" g\n");
                            crate::uart_print(b"  Z: ");
                            self.print_sensor_float(accel.z_g, 2);
                            crate::uart_print(b" g\n\n");
                        }
                    }
                    Err(_) => {
                        unsafe {
                            crate::uart_print(b"[MPU6050] Failed to read accelerometer\n");
                        }
                    }
                }

                // Read gyroscope
                match sensor.read_gyro() {
                    Ok(gyro) => {
                        unsafe {
                            crate::uart_print(b"Gyroscope:\n");
                            crate::uart_print(b"  X: ");
                            self.print_sensor_float(gyro.x_dps, 2);
                            crate::uart_print(b" deg/s\n");
                            crate::uart_print(b"  Y: ");
                            self.print_sensor_float(gyro.y_dps, 2);
                            crate::uart_print(b" deg/s\n");
                            crate::uart_print(b"  Z: ");
                            self.print_sensor_float(gyro.z_dps, 2);
                            crate::uart_print(b" deg/s\n\n");
                        }
                    }
                    Err(_) => {
                        unsafe {
                            crate::uart_print(b"[MPU6050] Failed to read gyroscope\n");
                        }
                    }
                }

                // Read temperature
                match sensor.read_temperature() {
                    Ok(temp) => {
                        unsafe {
                            crate::uart_print(b"Temperature: ");
                            self.print_sensor_float(temp.celsius, 1);
                            crate::uart_print(b" C\n");
                        }
                    }
                    Err(_) => {
                        unsafe {
                            crate::uart_print(b"[MPU6050] Failed to read temperature\n");
                        }
                    }
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[MPU6050] Initialization failed\n");
                    crate::uart_print(b"  Check I2C connection and address\n");
                }
            }
        }
    }

    /// Read BME280 sensor
    fn sensor_bme280_cmd(&self, bus: u8, addr: u8) {
        use crate::drivers::sensors::Bme280;

        let mut sensor = Bme280::new(bus, addr);

        unsafe {
            crate::uart_print(b"[BME280] Initializing sensor on bus ");
            self.print_number_simple(bus as u64);
            crate::uart_print(b" at address 0x");
            self.print_sensor_hex8(addr);
            crate::uart_print(b"...\n");
        }

        match sensor.initialize() {
            Ok(()) => {
                unsafe {
                    crate::uart_print(b"[BME280] Initialized successfully\n\n");
                }

                match sensor.read_measurements() {
                    Ok(data) => {
                        unsafe {
                            crate::uart_print(b"Environmental Data:\n");
                            crate::uart_print(b"  Temperature: ");
                            self.print_sensor_float(data.temperature, 1);
                            crate::uart_print(b" C\n");
                            crate::uart_print(b"  Humidity:    ");
                            self.print_sensor_float(data.humidity, 1);
                            crate::uart_print(b" %\n");
                            crate::uart_print(b"  Pressure:    ");
                            self.print_sensor_float(data.pressure, 1);
                            crate::uart_print(b" hPa\n");
                        }
                    }
                    Err(_) => {
                        unsafe {
                            crate::uart_print(b"[BME280] Failed to read measurements\n");
                        }
                    }
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[BME280] Initialization failed\n");
                    crate::uart_print(b"  Check I2C connection and address\n");
                }
            }
        }
    }

    /// Read VL53L0X sensor
    fn sensor_vl53l0x_cmd(&self, bus: u8, addr: u8) {
        use crate::drivers::sensors::Vl53l0x;

        let sensor = Vl53l0x::new(bus, addr);

        unsafe {
            crate::uart_print(b"[VL53L0X] Initializing sensor on bus ");
            self.print_number_simple(bus as u64);
            crate::uart_print(b" at address 0x");
            self.print_sensor_hex8(addr);
            crate::uart_print(b"...\n");
        }

        match sensor.initialize() {
            Ok(()) => {
                unsafe {
                    crate::uart_print(b"[VL53L0X] Initialized successfully\n\n");
                }

                match sensor.read_range_mm() {
                    Ok(range) => {
                        unsafe {
                            crate::uart_print(b"Distance: ");
                            self.print_number_simple(range.distance_mm as u64);
                            crate::uart_print(b" mm (");
                            if range.valid {
                                crate::uart_print(b"valid");
                            } else {
                                crate::uart_print(b"invalid");
                            }
                            crate::uart_print(b")\n");
                        }
                    }
                    Err(_) => {
                        unsafe {
                            crate::uart_print(b"[VL53L0X] Failed to read range\n");
                        }
                    }
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[VL53L0X] Initialization failed\n");
                    crate::uart_print(b"  Check I2C connection and address\n");
                }
            }
        }
    }

    /// Scan for known sensors
    fn sensor_scan_cmd(&self, bus: u8) {
        unsafe {
            crate::uart_print(b"[SENSOR] Scanning I2C bus ");
            self.print_number_simple(bus as u64);
            crate::uart_print(b" for known sensors...\n\n");
        }

        // Check for MPU6050
        self.check_sensor(bus, 0x68, b"MPU6050/MPU9250 IMU");
        self.check_sensor(bus, 0x69, b"MPU6050/MPU9250 IMU (alt)");

        // Check for BME280
        self.check_sensor(bus, 0x76, b"BME280 Environmental");
        self.check_sensor(bus, 0x77, b"BME280 Environmental (alt)");

        // Check for VL53L0X
        self.check_sensor(bus, 0x29, b"VL53L0X Time-of-Flight");

        // Check for BNO055
        self.check_sensor(bus, 0x28, b"BNO055 9-DOF IMU");

        // Check for INA219
        for addr in 0x40..=0x4F {
            self.check_sensor(bus, addr, b"INA219 Power Monitor");
        }

        unsafe {
            crate::uart_print(b"\nScan complete\n");
        }
    }

    /// Check if a sensor is present at an address
    fn check_sensor(&self, bus: u8, addr: u8, name: &[u8]) {
        use crate::drivers::i2c;

        match i2c::sensors::read_reg_u8(bus, addr, 0x00) {
            Ok(_) => {
                unsafe {
                    crate::uart_print(b"  [0x");
                    self.print_sensor_hex8(addr);
                    crate::uart_print(b"] Found: ");
                    crate::uart_print(name);
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                // Sensor not found, skip
            }
        }
    }

    /// Print floating point number (simple implementation)
    fn print_sensor_float(&self, value: f32, decimals: u32) {
        let negative = value < 0.0;
        let abs_value = if negative { -value } else { value };

        if negative {
            unsafe {
                crate::uart_print(b"-");
            }
        }

        let integer_part = abs_value as u64;
        self.print_number_simple(integer_part);

        if decimals > 0 {
            unsafe {
                crate::uart_print(b".");
            }

            let mut fractional = abs_value - (integer_part as f32);
            for _ in 0..decimals {
                fractional *= 10.0;
                let digit = fractional as u64 % 10;
                self.print_number_simple(digit);
            }
        }
    }

    /// Print 8-bit hex value (sensor helper)
    fn print_sensor_hex8(&self, val: u8) {
        let hex_chars = b"0123456789abcdef";
        unsafe {
            crate::uart_print(&[hex_chars[(val >> 4) as usize]]);
            crate::uart_print(&[hex_chars[(val & 0xF) as usize]]);
        }
    }
}
