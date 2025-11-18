//! Shell command helpers for SPI control and diagnostics
//!
//! Provides interactive commands for testing and debugging SPI communication
//! with peripherals and devices.

use crate::drivers::spi;

impl super::Shell {
    /// SPI command handler
    ///
    /// Usage:
    ///   spi                                  - Show SPI status
    ///   spi config <bus> <mode> <speed_mhz>  - Configure bus
    ///   spi transfer <bus> <cs> <hex_bytes...> - Transfer data
    ///   spi write <bus> <cs> <hex_bytes...>  - Write data
    ///   spi read <bus> <cs> <count>          - Read data
    pub(crate) fn spi_cmd(&self, args: &[&str]) {
        if !spi::is_initialized() {
            unsafe {
                crate::uart_print(b"[SPI] Not initialized\n");
            }
            return;
        }

        if args.is_empty() {
            self.spi_status_cmd();
            return;
        }

        match args[0] {
            "config" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: spi config <bus> <mode> <speed_mhz>\n");
                        crate::uart_print(b"Example: spi config 0 0 10\n");
                        crate::uart_print(b"  Modes: 0=CPOL0/CPHA0, 1=CPOL0/CPHA1, 2=CPOL1/CPHA0, 3=CPOL1/CPHA1\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let mode_num = self.parse_number(args[2].as_bytes()).unwrap_or(255);
                let speed_mhz = self.parse_number(args[3].as_bytes()).unwrap_or(1);

                let mode = match mode_num {
                    0 => spi::Mode::Mode0,
                    1 => spi::Mode::Mode1,
                    2 => spi::Mode::Mode2,
                    3 => spi::Mode::Mode3,
                    _ => {
                        unsafe {
                            crate::uart_print(b"[SPI] Invalid mode (must be 0-3)\n");
                        }
                        return;
                    }
                };

                self.spi_config_cmd(bus, mode, (speed_mhz * 1_000_000) as u32);
            }
            "transfer" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: spi transfer <bus> <cs> <hex_byte1> [hex_byte2...]\n");
                        crate::uart_print(b"Example: spi transfer 0 0 0x9F 0x00 0x00 0x00\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let cs_num = self.parse_number(args[2].as_bytes()).unwrap_or(255);

                let cs = match cs_num {
                    0 => spi::Cs::Cs0,
                    1 => spi::Cs::Cs1,
                    2 => spi::Cs::Cs2,
                    _ => {
                        unsafe {
                            crate::uart_print(b"[SPI] Invalid CS (must be 0-2)\n");
                        }
                        return;
                    }
                };

                // Parse data bytes
                let mut data = alloc::vec::Vec::new();
                for i in 3..args.len() {
                    if let Some(byte) = self.parse_number(args[i].as_bytes()) {
                        data.push(byte as u8);
                    }
                }

                self.spi_transfer_cmd(bus, cs, &data);
            }
            "write" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: spi write <bus> <cs> <hex_byte1> [hex_byte2...]\n");
                        crate::uart_print(b"Example: spi write 0 0 0x2A 0x00 0x00 0x00 0xEF\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let cs_num = self.parse_number(args[2].as_bytes()).unwrap_or(255);

                let cs = match cs_num {
                    0 => spi::Cs::Cs0,
                    1 => spi::Cs::Cs1,
                    2 => spi::Cs::Cs2,
                    _ => {
                        unsafe {
                            crate::uart_print(b"[SPI] Invalid CS (must be 0-2)\n");
                        }
                        return;
                    }
                };

                // Parse data bytes
                let mut data = alloc::vec::Vec::new();
                for i in 3..args.len() {
                    if let Some(byte) = self.parse_number(args[i].as_bytes()) {
                        data.push(byte as u8);
                    }
                }

                self.spi_write_cmd(bus, cs, &data);
            }
            "read" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: spi read <bus> <cs> <count>\n");
                        crate::uart_print(b"Example: spi read 0 0 16\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let cs_num = self.parse_number(args[2].as_bytes()).unwrap_or(255);
                let count = self.parse_number(args[3].as_bytes()).unwrap_or(0) as usize;

                let cs = match cs_num {
                    0 => spi::Cs::Cs0,
                    1 => spi::Cs::Cs1,
                    2 => spi::Cs::Cs2,
                    _ => {
                        unsafe {
                            crate::uart_print(b"[SPI] Invalid CS (must be 0-2)\n");
                        }
                        return;
                    }
                };

                self.spi_read_cmd(bus, cs, count);
            }
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown spi command: ");
                    crate::uart_print(args[0].as_bytes());
                    crate::uart_print(b"\n");
                    crate::uart_print(b"Available commands: config, transfer, write, read\n");
                }
            }
        }
    }

    /// Show SPI subsystem status
    fn spi_status_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n=== SPI Subsystem Status ===\n\n");

            crate::uart_print(b"Controllers: ");
            self.print_number_simple(spi::MAX_SPI_CONTROLLERS as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"Default mode: Mode 0 (CPOL=0, CPHA=0)\n");
            crate::uart_print(b"Default speed: 1 MHz\n");
            crate::uart_print(b"Max speed: 125 MHz\n");
            crate::uart_print(b"Chip selects: 0-2 per bus\n\n");

            crate::uart_print(b"Common Commands:\n");
            crate::uart_print(b"  spi config 0 0 10       - Configure bus 0, Mode 0, 10MHz\n");
            crate::uart_print(b"  spi transfer 0 0 0x9F 0x00 0x00 0x00  - Read device ID\n");
            crate::uart_print(b"  spi write 0 0 0x2A ...  - Write command\n");
            crate::uart_print(b"  spi read 0 0 16         - Read 16 bytes\n\n");

            crate::uart_print(b"SPI Modes:\n");
            crate::uart_print(b"  0: CPOL=0, CPHA=0  (clock idles low, sample on leading edge)\n");
            crate::uart_print(b"  1: CPOL=0, CPHA=1  (clock idles low, sample on trailing edge)\n");
            crate::uart_print(b"  2: CPOL=1, CPHA=0  (clock idles high, sample on leading edge)\n");
            crate::uart_print(b"  3: CPOL=1, CPHA=1  (clock idles high, sample on trailing edge)\n\n");
        }
    }

    /// Configure SPI bus
    fn spi_config_cmd(&self, bus: u8, mode: spi::Mode, speed_hz: u32) {
        match spi::configure(bus, mode, speed_hz) {
            Ok(actual_speed) => {
                unsafe {
                    crate::uart_print(b"[SPI] Configured bus ");
                    self.print_number_simple(bus as u64);
                    crate::uart_print(b"\n");

                    crate::uart_print(b"  Mode: ");
                    let mode_str = match mode {
                        spi::Mode::Mode0 => b"0 (CPOL=0, CPHA=0)",
                        spi::Mode::Mode1 => b"1 (CPOL=0, CPHA=1)",
                        spi::Mode::Mode2 => b"2 (CPOL=1, CPHA=0)",
                        spi::Mode::Mode3 => b"3 (CPOL=1, CPHA=1)",
                    };
                    crate::uart_print(mode_str);
                    crate::uart_print(b"\n");

                    crate::uart_print(b"  Requested speed: ");
                    self.print_number_simple(speed_hz as u64 / 1000);
                    crate::uart_print(b" kHz\n");

                    crate::uart_print(b"  Actual speed: ");
                    self.print_number_simple(actual_speed as u64 / 1000);
                    crate::uart_print(b" kHz\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[SPI] Error configuring bus\n");
                }
            }
        }
    }

    /// Transfer data (full duplex)
    fn spi_transfer_cmd(&self, bus: u8, cs: spi::Cs, data: &[u8]) {
        if data.is_empty() {
            unsafe {
                crate::uart_print(b"[SPI] No data to transfer\n");
            }
            return;
        }

        let mut rx_data = alloc::vec![0u8; data.len()];

        match spi::transfer(bus, cs, data, &mut rx_data) {
            Ok(bytes_transferred) => {
                unsafe {
                    crate::uart_print(b"[SPI] Transferred ");
                    self.print_number_simple(bytes_transferred as u64);
                    crate::uart_print(b" bytes\n");

                    crate::uart_print(b"TX: ");
                    for &byte in data.iter() {
                        self.print_spi_hex(byte);
                        crate::uart_print(b" ");
                    }
                    crate::uart_print(b"\n");

                    crate::uart_print(b"RX: ");
                    for &byte in rx_data.iter() {
                        self.print_spi_hex(byte);
                        crate::uart_print(b" ");
                    }
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[SPI] Error during transfer\n");
                }
            }
        }
    }

    /// Write data
    fn spi_write_cmd(&self, bus: u8, cs: spi::Cs, data: &[u8]) {
        if data.is_empty() {
            unsafe {
                crate::uart_print(b"[SPI] No data to write\n");
            }
            return;
        }

        match spi::write(bus, cs, data) {
            Ok(bytes_written) => {
                unsafe {
                    crate::uart_print(b"[SPI] Wrote ");
                    self.print_number_simple(bytes_written as u64);
                    crate::uart_print(b" bytes: ");
                    for &byte in data.iter() {
                        self.print_spi_hex(byte);
                        crate::uart_print(b" ");
                    }
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[SPI] Error writing data\n");
                }
            }
        }
    }

    /// Read data
    fn spi_read_cmd(&self, bus: u8, cs: spi::Cs, count: usize) {
        if count == 0 || count > 256 {
            unsafe {
                crate::uart_print(b"[SPI] Invalid count (must be 1-256)\n");
            }
            return;
        }

        let mut buffer = alloc::vec![0u8; count];

        match spi::read(bus, cs, &mut buffer) {
            Ok(bytes_read) => {
                unsafe {
                    crate::uart_print(b"[SPI] Read ");
                    self.print_number_simple(bytes_read as u64);
                    crate::uart_print(b" bytes:\n  ");

                    for (i, &byte) in buffer.iter().enumerate() {
                        self.print_spi_hex(byte);
                        crate::uart_print(b" ");
                        if (i + 1) % 16 == 0 && i + 1 < bytes_read {
                            crate::uart_print(b"\n  ");
                        }
                    }
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[SPI] Error reading data\n");
                }
            }
        }
    }

    /// Print 8-bit hex value (SPI helper)
    fn print_spi_hex(&self, val: u8) {
        let hex_chars = b"0123456789abcdef";
        unsafe {
            crate::uart_print(&[hex_chars[(val >> 4) as usize]]);
            crate::uart_print(&[hex_chars[(val & 0xF) as usize]]);
        }
    }
}
