//! Shell command helpers for mailbox/firmware interface
//!
//! Provides interactive commands for querying firmware information.
//! Part of M6 (GPIO/Mailbox) implementation.

impl super::Shell {
    /// Mailbox command handler
    ///
    /// Usage:
    ///   mailbox                 - Show mailbox status
    ///   mailbox temp            - Get SoC temperature
    ///   mailbox info            - Get board info
    ///   mailbox serial          - Get board serial number
    ///   mailbox fw              - Get firmware version
    ///   mailbox mem             - Get memory info
    ///   mailbox all             - Show all information
    pub(crate) fn mailbox_cmd(&self, args: &[&str]) {
        if !crate::drivers::firmware::mailbox::is_initialized() {
            unsafe {
                crate::uart_print(b"[MAILBOX] Not initialized\n");
            }
            return;
        }

        if args.is_empty() {
            self.mailbox_status_cmd();
            return;
        }

        match args[0] {
            "temp" | "temperature" => {
                self.mailbox_temp_cmd();
            }
            "info" | "board" => {
                self.mailbox_board_info_cmd();
            }
            "serial" => {
                self.mailbox_serial_cmd();
            }
            "fw" | "firmware" => {
                self.mailbox_firmware_cmd();
            }
            "mem" | "memory" => {
                self.mailbox_memory_cmd();
            }
            "all" => {
                self.mailbox_board_info_cmd();
                self.mailbox_serial_cmd();
                self.mailbox_firmware_cmd();
                self.mailbox_memory_cmd();
                self.mailbox_temp_cmd();
            }
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown mailbox command. Available:\n");
                    crate::uart_print(b"  mailbox temp      - Get SoC temperature\n");
                    crate::uart_print(b"  mailbox info      - Get board info\n");
                    crate::uart_print(b"  mailbox serial    - Get board serial\n");
                    crate::uart_print(b"  mailbox fw        - Get firmware version\n");
                    crate::uart_print(b"  mailbox mem       - Get memory info\n");
                    crate::uart_print(b"  mailbox all       - Show all info\n");
                }
            }
        }
    }

    /// Show mailbox status
    fn mailbox_status_cmd(&self) {
        unsafe {
            crate::uart_print(b"[MAILBOX] VideoCore Firmware Interface\n");
            crate::uart_print(b"Status: ");
            if crate::drivers::firmware::mailbox::is_initialized() {
                crate::uart_print(b"Initialized\n");
            } else {
                crate::uart_print(b"Not initialized\n");
            }
            crate::uart_print(b"\nUsage: mailbox <command>\n");
            crate::uart_print(b"Commands: temp, info, serial, fw, mem, all\n");
        }
    }

    /// Get and display temperature
    fn mailbox_temp_cmd(&self) {
        match crate::drivers::firmware::mailbox::get_temperature() {
            Ok(temp_millidegrees) => {
                let temp_degrees = temp_millidegrees / 1000;
                let temp_frac = (temp_millidegrees % 1000) / 100;

                unsafe {
                    crate::uart_print(b"[MAILBOX] SoC Temperature: ");
                    self.print_number_simple(temp_degrees as u64);
                    crate::uart_print(b".");
                    self.print_number_simple(temp_frac as u64);
                    crate::uart_print(b" C\n");
                }

                // Also try to get max temperature
                if let Ok(max_temp) = crate::drivers::firmware::mailbox::get_max_temperature() {
                    let max_degrees = max_temp / 1000;
                    let max_frac = (max_temp % 1000) / 100;
                    unsafe {
                        crate::uart_print(b"[MAILBOX] Max Temperature: ");
                        self.print_number_simple(max_degrees as u64);
                        crate::uart_print(b".");
                        self.print_number_simple(max_frac as u64);
                        crate::uart_print(b" C\n");
                    }
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[MAILBOX] Failed to get temperature\n");
                }
            }
        }
    }

    /// Get and display board info
    fn mailbox_board_info_cmd(&self) {
        unsafe {
            crate::uart_print(b"[MAILBOX] Board Information\n");
        }

        // Get board model
        if let Ok(model) = crate::drivers::firmware::mailbox::get_board_model() {
            unsafe {
                crate::uart_print(b"  Model:    ");
                self.print_hex_simple(model as u64);
                crate::uart_print(b"\n");
            }
        }

        // Get board revision
        if let Ok(revision) = crate::drivers::firmware::mailbox::get_board_revision() {
            unsafe {
                crate::uart_print(b"  Revision: ");
                self.print_hex_simple(revision as u64);
                crate::uart_print(b"\n");
            }
        }
    }

    /// Get and display board serial number
    fn mailbox_serial_cmd(&self) {
        match crate::drivers::firmware::mailbox::get_board_serial() {
            Ok(serial) => {
                unsafe {
                    crate::uart_print(b"[MAILBOX] Board Serial: ");
                    self.print_hex_simple(serial);
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[MAILBOX] Failed to get serial number\n");
                }
            }
        }
    }

    /// Get and display firmware version
    fn mailbox_firmware_cmd(&self) {
        match crate::drivers::firmware::mailbox::get_firmware_revision() {
            Ok(fw_rev) => {
                unsafe {
                    crate::uart_print(b"[MAILBOX] Firmware Revision: ");
                    self.print_hex_simple(fw_rev as u64);
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[MAILBOX] Failed to get firmware revision\n");
                }
            }
        }
    }

    /// Get and display memory info
    fn mailbox_memory_cmd(&self) {
        match crate::drivers::firmware::mailbox::get_arm_memory() {
            Ok((base, size)) => {
                unsafe {
                    crate::uart_print(b"[MAILBOX] ARM Memory\n");
                    crate::uart_print(b"  Base: ");
                    self.print_hex_simple(base as u64);
                    crate::uart_print(b"\n  Size: ");
                    self.print_number_simple(size as u64);
                    crate::uart_print(b" bytes (");
                    self.print_number_simple((size / (1024 * 1024)) as u64);
                    crate::uart_print(b" MB)\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[MAILBOX] Failed to get memory info\n");
                }
            }
        }
    }

    /// Print hex number with 0x prefix
    fn print_hex_simple(&self, value: u64) {
        unsafe {
            crate::uart_print(b"0x");
        }

        // Print in hex (simple implementation)
        let mut val = value;
        let mut digits = [0u8; 16];
        let mut count = 0;

        if val == 0 {
            unsafe { crate::uart_print(b"0"); }
            return;
        }

        while val > 0 {
            let digit = (val % 16) as u8;
            digits[count] = if digit < 10 {
                b'0' + digit
            } else {
                b'A' + (digit - 10)
            };
            count += 1;
            val /= 16;
        }

        // Print digits in reverse
        for i in (0..count).rev() {
            unsafe {
                let ch = [digits[i]];
                crate::uart_print(&ch);
            }
        }
    }
}
