//! Shell command helpers for GPIO control
//!
//! Provides interactive commands for testing and controlling GPIO pins.
//! Part of M6 (GPIO/Mailbox) implementation.
//! M8 Hardening: Error handling for all GPIO operations

use crate::drivers::gpio::{GpioFunction, GpioPull};
use crate::drivers::DriverError;

impl super::Shell {
    /// GPIO command handler
    ///
    /// Usage:
    ///   gpio                    - Show GPIO status
    ///   gpio set <pin>          - Set pin high
    ///   gpio clear <pin>        - Set pin low
    ///   gpio toggle <pin>       - Toggle pin
    ///   gpio read <pin>         - Read pin state
    ///   gpio output <pin>       - Set pin as output
    ///   gpio input <pin>        - Set pin as input
    pub(crate) fn gpio_cmd(&self, args: &[&str]) {
        if !crate::drivers::gpio::is_initialized() {
            unsafe {
                crate::uart_print(b"[GPIO] Not initialized\n");
            }
            return;
        }

        if args.is_empty() {
            self.gpio_status_cmd();
            return;
        }

        match args[0] {
            "set" => {
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: gpio set <pin>\n"); }
                    return;
                }
                if let Some(pin) = self.parse_number(args[1].as_bytes()) {
                    match crate::drivers::gpio::set_pin(pin as u32) {
                        Ok(()) => unsafe {
                            crate::uart_print(b"[GPIO] Pin ");
                            self.print_number_simple(pin);
                            crate::uart_print(b" set HIGH\n");
                        },
                        Err(e) => self.print_gpio_error(e),
                    }
                }
            }
            "clear" => {
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: gpio clear <pin>\n"); }
                    return;
                }
                if let Some(pin) = self.parse_number(args[1].as_bytes()) {
                    match crate::drivers::gpio::clear_pin(pin as u32) {
                        Ok(()) => unsafe {
                            crate::uart_print(b"[GPIO] Pin ");
                            self.print_number_simple(pin);
                            crate::uart_print(b" set LOW\n");
                        },
                        Err(e) => self.print_gpio_error(e),
                    }
                }
            }
            "toggle" => {
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: gpio toggle <pin>\n"); }
                    return;
                }
                if let Some(pin) = self.parse_number(args[1].as_bytes()) {
                    match crate::drivers::gpio::toggle_pin(pin as u32) {
                        Ok(()) => unsafe {
                            crate::uart_print(b"[GPIO] Pin ");
                            self.print_number_simple(pin);
                            crate::uart_print(b" toggled\n");
                        },
                        Err(e) => self.print_gpio_error(e),
                    }
                }
            }
            "read" => {
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: gpio read <pin>\n"); }
                    return;
                }
                if let Some(pin) = self.parse_number(args[1].as_bytes()) {
                    match crate::drivers::gpio::read_pin(pin as u32) {
                        Ok(level) => unsafe {
                            crate::uart_print(b"[GPIO] Pin ");
                            self.print_number_simple(pin);
                            crate::uart_print(b" = ");
                            if level {
                                crate::uart_print(b"HIGH\n");
                            } else {
                                crate::uart_print(b"LOW\n");
                            }
                        },
                        Err(e) => self.print_gpio_error(e),
                    }
                }
            }
            "output" => {
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: gpio output <pin>\n"); }
                    return;
                }
                if let Some(pin) = self.parse_number(args[1].as_bytes()) {
                    match crate::drivers::gpio::set_function(pin as u32, GpioFunction::Output) {
                        Ok(()) => unsafe {
                            crate::uart_print(b"[GPIO] Pin ");
                            self.print_number_simple(pin);
                            crate::uart_print(b" configured as OUTPUT\n");
                        },
                        Err(e) => self.print_gpio_error(e),
                    }
                }
            }
            "input" => {
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: gpio input <pin>\n"); }
                    return;
                }
                if let Some(pin) = self.parse_number(args[1].as_bytes()) {
                    match crate::drivers::gpio::set_function(pin as u32, GpioFunction::Input) {
                        Ok(()) => unsafe {
                            crate::uart_print(b"[GPIO] Pin ");
                            self.print_number_simple(pin);
                            crate::uart_print(b" configured as INPUT\n");
                        },
                        Err(e) => self.print_gpio_error(e),
                    }
                }
            }
            "blink" => {
                // LED blink demo
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: gpio blink <pin> [count]\n"); }
                    return;
                }
                if let Some(pin) = self.parse_number(args[1].as_bytes()) {
                    let count = if args.len() >= 3 {
                        self.parse_number(args[2].as_bytes()).unwrap_or(5)
                    } else {
                        5
                    };

                    unsafe {
                        crate::uart_print(b"[GPIO] Blinking pin ");
                        self.print_number_simple(pin);
                        crate::uart_print(b" ");
                        self.print_number_simple(count);
                        crate::uart_print(b" times...\n");
                    }

                    // Configure as output
                    if let Err(e) = crate::drivers::gpio::set_function(pin as u32, GpioFunction::Output) {
                        self.print_gpio_error(e);
                        return;
                    }

                    // Blink
                    for i in 0..count {
                        if let Err(e) = crate::drivers::gpio::set_pin(pin as u32) {
                            self.print_gpio_error(e);
                            return;
                        }
                        crate::time::sleep_ms(500);

                        if let Err(e) = crate::drivers::gpio::clear_pin(pin as u32) {
                            self.print_gpio_error(e);
                            return;
                        }
                        crate::time::sleep_ms(500);

                        unsafe {
                            crate::uart_print(b".");
                        }
                    }

                    unsafe {
                        crate::uart_print(b"\n[GPIO] Blink complete\n");
                    }
                }
            }
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown gpio command. Available:\n");
                    crate::uart_print(b"  gpio set <pin>       - Set pin high\n");
                    crate::uart_print(b"  gpio clear <pin>     - Set pin low\n");
                    crate::uart_print(b"  gpio toggle <pin>    - Toggle pin\n");
                    crate::uart_print(b"  gpio read <pin>      - Read pin state\n");
                    crate::uart_print(b"  gpio output <pin>    - Configure as output\n");
                    crate::uart_print(b"  gpio input <pin>     - Configure as input\n");
                    crate::uart_print(b"  gpio blink <pin> [n] - Blink LED n times\n");
                }
            }
        }
    }

    /// Show GPIO status
    fn gpio_status_cmd(&self) {
        unsafe {
            crate::uart_print(b"[GPIO] BCM2xxx GPIO Controller\n");
            crate::uart_print(b"Status: ");
            if crate::drivers::gpio::is_initialized() {
                crate::uart_print(b"Initialized\n");
            } else {
                crate::uart_print(b"Not initialized\n");
            }
            crate::uart_print(b"\nUsage: gpio <command> [args]\n");
            crate::uart_print(b"Commands: set, clear, toggle, read, output, input, blink\n");
        }
    }

    /// Print GPIO error message
    /// M8 Hardening: Comprehensive error reporting
    fn print_gpio_error(&self, error: DriverError) {
        unsafe {
            crate::uart_print(b"[GPIO ERROR] ");
            match error {
                DriverError::InvalidParameter => crate::uart_print(b"Invalid pin number (valid: 0-53)\n"),
                DriverError::NotInitialized => crate::uart_print(b"GPIO not initialized\n"),
                DriverError::HardwareError => crate::uart_print(b"Hardware error\n"),
                DriverError::Timeout(_) => crate::uart_print(b"Operation timed out\n"),
                _ => {
                    crate::uart_print(b"Error code ");
                    self.print_number_simple(error.code() as u64);
                    crate::uart_print(b"\n");
                }
            }
        }
    }
}
