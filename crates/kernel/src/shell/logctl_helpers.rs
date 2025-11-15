// Logging control commands
//
// M8 Production Readiness - Logging Management

use crate::log::{LogLevel, get_level, set_level, policy};

impl super::Shell {
    /// Logging control command
    pub(crate) fn logctl_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            self.logctl_status();
            return;
        }

        match args[0] {
            "status" => self.logctl_status(),
            "level" => {
                if args.len() < 2 {
                    unsafe {
                        crate::uart_print(b"Usage: logctl level <error|warn|info|debug|trace>\n");
                    }
                    return;
                }
                self.logctl_set_level(args[1]);
            }
            "production" | "prod" => {
                policy::set_production();
                self.logctl_status();
            }
            "development" | "dev" => {
                policy::set_development();
                self.logctl_status();
            }
            "testing" | "test" => {
                policy::set_testing();
                self.logctl_status();
            }
            "demo" => self.logctl_demo(),
            _ => unsafe {
                crate::uart_print(b"Usage: logctl [status|level <LEVEL>|production|development|testing|demo]\n");
            },
        }
    }

    fn logctl_status(&self) {
        let level = get_level();

        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"=== Logging Configuration ===\n");
            crate::uart_print(b"Current Level: ");
            crate::uart_print(level.as_str().as_bytes());
            crate::uart_print(b"\n\n");

            crate::uart_print(b"Available Levels:\n");
            crate::uart_print(b"  ERROR - Critical errors only\n");
            crate::uart_print(b"  WARN  - Warnings and errors\n");
            crate::uart_print(b"  INFO  - Normal operation (production default)\n");
            crate::uart_print(b"  DEBUG - Debugging information\n");
            crate::uart_print(b"  TRACE - Detailed trace information\n\n");

            crate::uart_print(b"Log Policies:\n");
            crate::uart_print(b"  production  - WARN level (minimal logging)\n");
            crate::uart_print(b"  development - DEBUG level (detailed logging)\n");
            crate::uart_print(b"  testing     - TRACE level (maximum logging)\n\n");

            crate::uart_print(b"Enabled Levels: ");
            if level >= LogLevel::Error {
                crate::uart_print(b"ERROR ");
            }
            if level >= LogLevel::Warn {
                crate::uart_print(b"WARN ");
            }
            if level >= LogLevel::Info {
                crate::uart_print(b"INFO ");
            }
            if level >= LogLevel::Debug {
                crate::uart_print(b"DEBUG ");
            }
            if level >= LogLevel::Trace {
                crate::uart_print(b"TRACE");
            }
            crate::uart_print(b"\n\n");
        }
    }

    fn logctl_set_level(&self, level_str: &str) {
        let level = match level_str {
            "error" | "ERROR" | "0" => LogLevel::Error,
            "warn" | "WARN" | "1" => LogLevel::Warn,
            "info" | "INFO" | "2" => LogLevel::Info,
            "debug" | "DEBUG" | "3" => LogLevel::Debug,
            "trace" | "TRACE" | "4" => LogLevel::Trace,
            _ => {
                unsafe {
                    crate::uart_print(b"Invalid log level. Use: error, warn, info, debug, or trace\n");
                }
                return;
            }
        };

        set_level(level);

        unsafe {
            crate::uart_print(b"Log level set to: ");
            crate::uart_print(level.as_str().as_bytes());
            crate::uart_print(b"\n");
        }
    }

    fn logctl_demo(&self) {
        unsafe {
            crate::uart_print(b"\n=== Logging Demo ===\n\n");
        }

        // Test all log levels
        crate::log::error("DEMO", "This is an ERROR message");
        crate::log::warn("DEMO", "This is a WARN message");
        crate::log::info("DEMO", "This is an INFO message");
        crate::log::debug("DEMO", "This is a DEBUG message");
        crate::log::trace("DEMO", "This is a TRACE message");

        unsafe {
            crate::uart_print(b"\n");
        }

        // Test context logging
        crate::log::info_ctx("DEMO", "GPIO pin set", &[("pin", 27)]);
        crate::log::warn_ctx("DEMO", "Device temperature high", &[("temp", 75), ("threshold", 70)]);

        unsafe {
            crate::uart_print(b"\n");
        }

        // Test driver error logging
        let error = crate::drivers::DriverError::InvalidParameter;
        crate::log::log_driver_error("DEMO", "pin_set", &error);

        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"Demo complete. Only messages at or above current log level are displayed.\n");
            crate::uart_print(b"Current level: ");
            crate::uart_print(get_level().as_str().as_bytes());
            crate::uart_print(b"\n\n");
        }
    }
}
