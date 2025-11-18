//! Shell command helpers for I2C control and diagnostics
//!
//! Provides interactive commands for testing and debugging I2C communication
//! with sensors and peripherals.

use crate::drivers::i2c;

impl super::Shell {
    /// I2C command handler
    ///
    /// Usage:
    ///   i2c                              - Show I2C status
    ///   i2c scan <bus>                   - Scan bus for devices
    ///   i2c read <bus> <addr> <count>    - Read bytes from device
    ///   i2c write <bus> <addr> <bytes...> - Write bytes to device
    ///   i2c readreg <bus> <addr> <reg> <count> - Read from register
    ///   i2c writereg <bus> <addr> <reg> <value> - Write to register
    ///   i2c devices                      - List common device addresses
    pub(crate) fn i2c_cmd(&self, args: &[&str]) {
        if !i2c::is_initialized() {
            unsafe {
                crate::uart_print(b"[I2C] Not initialized\n");
            }
            return;
        }

        if args.is_empty() {
            self.i2c_status_cmd();
            return;
        }

        match args[0] {
            "scan" => {
                if args.len() < 2 {
                    unsafe {
                        crate::uart_print(b"Usage: i2c scan <bus>\n");
                        crate::uart_print(b"Example: i2c scan 0\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                self.i2c_scan_cmd(bus);
            }
            "read" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: i2c read <bus> <addr> <count>\n");
                        crate::uart_print(b"Example: i2c read 0 0x68 6\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let addr = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;
                let count = self.parse_number(args[3].as_bytes()).unwrap_or(0) as usize;

                self.i2c_read_cmd(bus, addr, count);
            }
            "write" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: i2c write <bus> <addr> <byte1> [byte2...]\n");
                        crate::uart_print(b"Example: i2c write 0 0x68 0x6B 0x00\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let addr = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;

                // Parse data bytes
                let mut data = alloc::vec::Vec::new();
                for i in 3..args.len() {
                    if let Some(byte) = self.parse_number(args[i].as_bytes()) {
                        data.push(byte as u8);
                    }
                }

                self.i2c_write_cmd(bus, addr, &data);
            }
            "readreg" => {
                if args.len() < 5 {
                    unsafe {
                        crate::uart_print(b"Usage: i2c readreg <bus> <addr> <reg> <count>\n");
                        crate::uart_print(b"Example: i2c readreg 0 0x68 0x75 1  (MPU6050 WHO_AM_I)\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let addr = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;
                let reg = self.parse_number(args[3].as_bytes()).unwrap_or(255) as u8;
                let count = self.parse_number(args[4].as_bytes()).unwrap_or(0) as usize;

                self.i2c_readreg_cmd(bus, addr, reg, count);
            }
            "writereg" => {
                if args.len() < 5 {
                    unsafe {
                        crate::uart_print(b"Usage: i2c writereg <bus> <addr> <reg> <value>\n");
                        crate::uart_print(b"Example: i2c writereg 0 0x68 0x6B 0x00  (Wake MPU6050)\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let addr = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;
                let reg = self.parse_number(args[3].as_bytes()).unwrap_or(255) as u8;
                let value = self.parse_number(args[4].as_bytes()).unwrap_or(0) as u8;

                self.i2c_writereg_cmd(bus, addr, reg, value);
            }
            "devices" => {
                self.i2c_devices_cmd();
            }
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown i2c command: ");
                    crate::uart_print(args[0].as_bytes());
                    crate::uart_print(b"\n");
                    crate::uart_print(b"Available commands: scan, read, write, readreg, writereg, devices\n");
                }
            }
        }
    }

    /// Show I2C subsystem status
    fn i2c_status_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n=== I2C Subsystem Status ===\n\n");

            crate::uart_print(b"Controllers: ");
            self.print_number_simple(i2c::MAX_I2C_CONTROLLERS as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"Default mode: Fast (400 kHz)\n");
            crate::uart_print(b"Address range: 0x08 - 0x77 (7-bit)\n\n");

            crate::uart_print(b"Common Commands:\n");
            crate::uart_print(b"  i2c scan 0           - Scan bus 0 for devices\n");
            crate::uart_print(b"  i2c readreg 0 0x68 0x75 1  - Read WHO_AM_I from MPU6050\n");
            crate::uart_print(b"  i2c devices          - List common sensor addresses\n\n");
        }
    }

    /// Scan I2C bus for devices
    fn i2c_scan_cmd(&self, bus: u8) {
        unsafe {
            crate::uart_print(b"\n=== Scanning I2C Bus ");
            self.print_number_simple(bus as u64);
            crate::uart_print(b" ===\n\n");
        }

        match i2c::scan(bus) {
            Ok(devices) => {
                if devices.is_empty() {
                    unsafe {
                        crate::uart_print(b"No devices found\n");
                    }
                    return;
                }

                unsafe {
                    crate::uart_print(b"Found ");
                    self.print_number_simple(devices.len() as u64);
                    crate::uart_print(b" device(s):\n\n");
                }

                for addr in devices {
                    unsafe {
                        crate::uart_print(b"  0x");
                        self.print_i2c_hex(addr);
                        crate::uart_print(b"  (");
                        self.print_number_simple(addr as u64);
                        crate::uart_print(b")  ");
                        self.print_i2c_device_name(addr);
                        crate::uart_print(b"\n");
                    }
                }

                unsafe {
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[I2C] Error scanning bus\n");
                }
            }
        }
    }

    /// Read bytes from I2C device
    fn i2c_read_cmd(&self, bus: u8, addr: u8, count: usize) {
        if count == 0 || count > 256 {
            unsafe {
                crate::uart_print(b"[I2C] Invalid count (must be 1-256)\n");
            }
            return;
        }

        let mut buffer = alloc::vec![0u8; count];

        match i2c::read(bus, addr, &mut buffer) {
            Ok(bytes_read) => {
                unsafe {
                    crate::uart_print(b"[I2C] Read ");
                    self.print_number_simple(bytes_read as u64);
                    crate::uart_print(b" bytes from 0x");
                    self.print_i2c_hex(addr);
                    crate::uart_print(b":\n");

                    for (i, &byte) in buffer.iter().enumerate() {
                        if i % 16 == 0 {
                            crate::uart_print(b"  ");
                        }
                        self.print_i2c_hex(byte);
                        crate::uart_print(b" ");
                        if (i + 1) % 16 == 0 {
                            crate::uart_print(b"\n");
                        }
                    }
                    if bytes_read % 16 != 0 {
                        crate::uart_print(b"\n");
                    }
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[I2C] Error reading from device\n");
                }
            }
        }
    }

    /// Write bytes to I2C device
    fn i2c_write_cmd(&self, bus: u8, addr: u8, data: &[u8]) {
        if data.is_empty() {
            unsafe {
                crate::uart_print(b"[I2C] No data to write\n");
            }
            return;
        }

        match i2c::write(bus, addr, data) {
            Ok(bytes_written) => {
                unsafe {
                    crate::uart_print(b"[I2C] Wrote ");
                    self.print_number_simple(bytes_written as u64);
                    crate::uart_print(b" bytes to 0x");
                    self.print_i2c_hex(addr);
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[I2C] Error writing to device\n");
                }
            }
        }
    }

    /// Read from register
    fn i2c_readreg_cmd(&self, bus: u8, addr: u8, reg: u8, count: usize) {
        if count == 0 || count > 256 {
            unsafe {
                crate::uart_print(b"[I2C] Invalid count (must be 1-256)\n");
            }
            return;
        }

        let mut buffer = alloc::vec![0u8; count];

        match i2c::write_read(bus, addr, reg, &mut buffer) {
            Ok(bytes_read) => {
                unsafe {
                    crate::uart_print(b"[I2C] Read register 0x");
                    self.print_i2c_hex(reg);
                    crate::uart_print(b" from 0x");
                    self.print_i2c_hex(addr);
                    crate::uart_print(b":\n  ");

                    for &byte in buffer.iter() {
                        self.print_i2c_hex(byte);
                        crate::uart_print(b" ");
                    }
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[I2C] Error reading register\n");
                }
            }
        }
    }

    /// Write to register
    fn i2c_writereg_cmd(&self, bus: u8, addr: u8, reg: u8, value: u8) {
        match i2c::write(bus, addr, &[reg, value]) {
            Ok(_) => {
                unsafe {
                    crate::uart_print(b"[I2C] Wrote 0x");
                    self.print_i2c_hex(value);
                    crate::uart_print(b" to register 0x");
                    self.print_i2c_hex(reg);
                    crate::uart_print(b" at 0x");
                    self.print_i2c_hex(addr);
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[I2C] Error writing register\n");
                }
            }
        }
    }

    /// Show common I2C device addresses
    fn i2c_devices_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n=== Common I2C Device Addresses ===\n\n");

            crate::uart_print(b"IMU Sensors:\n");
            crate::uart_print(b"  0x68 (104)  MPU6050, MPU9250, DS1307, DS3231\n");
            crate::uart_print(b"  0x28 (40)   BNO055\n\n");

            crate::uart_print(b"Environmental Sensors:\n");
            crate::uart_print(b"  0x76 (118)  BME280, BMP280 (default)\n");
            crate::uart_print(b"  0x77 (119)  BME280, BMP280 (alternate)\n\n");

            crate::uart_print(b"Distance Sensors:\n");
            crate::uart_print(b"  0x29 (41)   VL53L0X, VL53L1X\n\n");

            crate::uart_print(b"Power Monitoring:\n");
            crate::uart_print(b"  0x40 (64)   INA219 (default)\n");
            crate::uart_print(b"  0x41-0x4F   INA219 (configurable)\n\n");

            crate::uart_print(b"Memory:\n");
            crate::uart_print(b"  0x50-0x57   AT24Cxx EEPROM\n\n");
        }
    }

    /// Print I2C device name if known
    fn print_i2c_device_name(&self, addr: u8) {
        let name = match addr {
            0x28 => "BNO055 IMU",
            0x29 => "VL53L0X/VL53L1X ToF",
            0x40 => "INA219 Power Monitor",
            0x50..=0x57 => "AT24Cxx EEPROM",
            0x68 => "MPU6050/MPU9250/DS1307/DS3231",
            0x76 | 0x77 => "BME280/BMP280",
            _ => "Unknown",
        };

        unsafe {
            crate::uart_print(name.as_bytes());
        }
    }

    /// Print 8-bit hex value (I2C helper)
    fn print_i2c_hex(&self, val: u8) {
        let hex_chars = b"0123456789abcdef";
        unsafe {
            crate::uart_print(&[hex_chars[(val >> 4) as usize]]);
            crate::uart_print(&[hex_chars[(val & 0xF) as usize]]);
        }
    }
}
