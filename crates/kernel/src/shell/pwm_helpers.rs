//! Shell command helpers for PWM control
//!
//! Provides interactive commands for testing and controlling PWM channels,
//! including servo control, motor speed control, and LED dimming.

use crate::drivers::pwm;

impl super::Shell {
    /// PWM command handler
    ///
    /// Usage:
    ///   pwm                            - Show PWM status
    ///   pwm enable <ctrl> <ch>        - Enable PWM channel
    ///   pwm disable <ctrl> <ch>       - Disable PWM channel
    ///   pwm freq <ctrl> <ch> <hz>     - Set frequency in Hz
    ///   pwm duty <ctrl> <ch> <percent> - Set duty cycle (0-100%)
    ///   pwm pulse <ctrl> <ch> <us>    - Set pulse width in microseconds
    ///   pwm servo <ctrl> <ch> <cmd>   - Servo control commands
    pub(crate) fn pwm_cmd(&self, args: &[&str]) {
        if !pwm::is_initialized() {
            unsafe {
                crate::uart_print(b"[PWM] Not initialized\n");
            }
            return;
        }

        if args.is_empty() {
            self.pwm_status_cmd();
            return;
        }

        match args[0] {
            "enable" => {
                if args.len() < 3 {
                    unsafe {
                        crate::uart_print(b"Usage: pwm enable <controller> <channel>\n");
                        crate::uart_print(b"Example: pwm enable 0 0\n");
                    }
                    return;
                }

                let ctrl = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let ch = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;

                match pwm::enable(ctrl, ch) {
                    Ok(()) => unsafe {
                        crate::uart_print(b"[PWM] PWM");
                        self.print_number_simple(ctrl as u64);
                        crate::uart_print(b" Channel ");
                        self.print_number_simple(ch as u64);
                        crate::uart_print(b" enabled\n");
                    },
                    Err(_) => unsafe {
                        crate::uart_print(b"[PWM] Error enabling channel\n");
                    },
                }
            }
            "disable" => {
                if args.len() < 3 {
                    unsafe {
                        crate::uart_print(b"Usage: pwm disable <controller> <channel>\n");
                    }
                    return;
                }

                let ctrl = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let ch = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;

                match pwm::disable(ctrl, ch) {
                    Ok(()) => unsafe {
                        crate::uart_print(b"[PWM] PWM");
                        self.print_number_simple(ctrl as u64);
                        crate::uart_print(b" Channel ");
                        self.print_number_simple(ch as u64);
                        crate::uart_print(b" disabled\n");
                    },
                    Err(_) => unsafe {
                        crate::uart_print(b"[PWM] Error disabling channel\n");
                    },
                }
            }
            "freq" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: pwm freq <controller> <channel> <hz>\n");
                        crate::uart_print(b"Example: pwm freq 0 0 50    (for servo)\n");
                        crate::uart_print(b"Example: pwm freq 0 0 1000  (for motor/LED)\n");
                    }
                    return;
                }

                let ctrl = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let ch = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;
                let freq = self.parse_number(args[3].as_bytes()).unwrap_or(0) as u32;

                match pwm::set_frequency(ctrl, ch, freq) {
                    Ok(actual_freq) => unsafe {
                        crate::uart_print(b"[PWM] Frequency set to ");
                        self.print_number_simple(actual_freq as u64);
                        crate::uart_print(b" Hz\n");
                    },
                    Err(_) => unsafe {
                        crate::uart_print(b"[PWM] Error setting frequency\n");
                    },
                }
            }
            "duty" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: pwm duty <controller> <channel> <percent>\n");
                        crate::uart_print(b"Example: pwm duty 0 0 75    (75% duty cycle)\n");
                    }
                    return;
                }

                let ctrl = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let ch = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;

                // Parse as integer, convert to float
                if let Some(duty_int) = self.parse_number(args[3].as_bytes()) {
                    let duty = duty_int as f32;

                    match pwm::set_duty_cycle(ctrl, ch, duty) {
                        Ok(()) => unsafe {
                            crate::uart_print(b"[PWM] Duty cycle set to ");
                            self.print_number_simple(duty_int as u64);
                            crate::uart_print(b"%\n");
                        },
                        Err(_) => unsafe {
                            crate::uart_print(b"[PWM] Error setting duty cycle\n");
                        },
                    }
                }
            }
            "pulse" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: pwm pulse <controller> <channel> <microseconds>\n");
                        crate::uart_print(b"Example: pwm pulse 0 0 1500  (1.5ms for servo center)\n");
                    }
                    return;
                }

                let ctrl = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let ch = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;
                let pulse_us = self.parse_number(args[3].as_bytes()).unwrap_or(0) as u32;

                match pwm::set_pulse_width_us(ctrl, ch, pulse_us) {
                    Ok(()) => unsafe {
                        crate::uart_print(b"[PWM] Pulse width set to ");
                        self.print_number_simple(pulse_us as u64);
                        crate::uart_print(b" us\n");
                    },
                    Err(_) => unsafe {
                        crate::uart_print(b"[PWM] Error setting pulse width\n");
                    },
                }
            }
            "servo" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: pwm servo <controller> <channel> <command>\n");
                        crate::uart_print(b"Commands:\n");
                        crate::uart_print(b"  init     - Initialize for servo control (50Hz)\n");
                        crate::uart_print(b"  center   - Move to center position\n");
                        crate::uart_print(b"  min      - Move to minimum position\n");
                        crate::uart_print(b"  max      - Move to maximum position\n");
                        crate::uart_print(b"  angle <degrees>  - Set angle (-90 to +90)\n");
                    }
                    return;
                }

                let ctrl = self.parse_number(args[1].as_bytes()).unwrap_or(255) as u8;
                let ch = self.parse_number(args[2].as_bytes()).unwrap_or(255) as u8;

                match args[3] {
                    "init" => {
                        match pwm::servo::init(ctrl, ch) {
                            Ok(()) => unsafe {
                                crate::uart_print(b"[PWM] Servo initialized (50Hz)\n");
                            },
                            Err(_) => unsafe {
                                crate::uart_print(b"[PWM] Error initializing servo\n");
                            },
                        }
                    }
                    "center" => {
                        match pwm::servo::center(ctrl, ch) {
                            Ok(()) => unsafe {
                                crate::uart_print(b"[PWM] Servo moved to center\n");
                            },
                            Err(_) => unsafe {
                                crate::uart_print(b"[PWM] Error moving servo\n");
                            },
                        }
                    }
                    "min" => {
                        match pwm::servo::min(ctrl, ch) {
                            Ok(()) => unsafe {
                                crate::uart_print(b"[PWM] Servo moved to minimum\n");
                            },
                            Err(_) => unsafe {
                                crate::uart_print(b"[PWM] Error moving servo\n");
                            },
                        }
                    }
                    "max" => {
                        match pwm::servo::max(ctrl, ch) {
                            Ok(()) => unsafe {
                                crate::uart_print(b"[PWM] Servo moved to maximum\n");
                            },
                            Err(_) => unsafe {
                                crate::uart_print(b"[PWM] Error moving servo\n");
                            },
                        }
                    }
                    "angle" => {
                        if args.len() < 5 {
                            unsafe {
                                crate::uart_print(b"Usage: pwm servo <ctrl> <ch> angle <degrees>\n");
                            }
                            return;
                        }

                        if let Some(angle) = self.parse_number(args[4].as_bytes()) {
                            let angle_i32 = angle as i32;
                            match pwm::servo::set_angle(ctrl, ch, angle_i32) {
                                Ok(()) => unsafe {
                                    crate::uart_print(b"[PWM] Servo angle set to ");
                                    self.print_number_simple(angle as u64);
                                    crate::uart_print(b" degrees\n");
                                },
                                Err(_) => unsafe {
                                    crate::uart_print(b"[PWM] Error setting servo angle\n");
                                },
                            }
                        }
                    }
                    _ => unsafe {
                        crate::uart_print(b"Unknown servo command: ");
                        crate::uart_print(args[3].as_bytes());
                        crate::uart_print(b"\n");
                    },
                }
            }
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown pwm command: ");
                    crate::uart_print(args[0].as_bytes());
                    crate::uart_print(b"\n");
                    crate::uart_print(b"Available commands: enable, disable, freq, duty, pulse, servo\n");
                }
            }
        }
    }

    /// Show PWM subsystem status
    fn pwm_status_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n=== PWM Subsystem Status ===\n\n");

            crate::uart_print(b"Controllers: ");
            self.print_number_simple(pwm::MAX_PWM_CONTROLLERS as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"Channels per controller: ");
            self.print_number_simple(pwm::CHANNELS_PER_CONTROLLER as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"Total channels: ");
            self.print_number_simple((pwm::MAX_PWM_CONTROLLERS * pwm::CHANNELS_PER_CONTROLLER) as u64);
            crate::uart_print(b"\n\n");

            // Show status of each channel
            for ctrl in 0..pwm::MAX_PWM_CONTROLLERS as u8 {
                for ch in 0..pwm::CHANNELS_PER_CONTROLLER as u8 {
                    crate::uart_print(b"PWM");
                    self.print_number_simple(ctrl as u64);
                    crate::uart_print(b" Ch");
                    self.print_number_simple(ch as u64);
                    crate::uart_print(b": ");

                    if pwm::is_enabled(ctrl, ch) {
                        crate::uart_print(b"ENABLED");

                        if let Ok(freq) = pwm::get_frequency(ctrl, ch) {
                            crate::uart_print(b" @ ");
                            self.print_number_simple(freq as u64);
                            crate::uart_print(b" Hz");
                        }
                    } else {
                        crate::uart_print(b"disabled");
                    }

                    crate::uart_print(b"\n");
                }
            }

            crate::uart_print(b"\nCommon Frequencies:\n");
            crate::uart_print(b"  Servo:  50 Hz (20ms period, 1-2ms pulse)\n");
            crate::uart_print(b"  Motor:  1-20 kHz\n");
            crate::uart_print(b"  LED:    100-1000 Hz\n");
            crate::uart_print(b"  Buzzer: 20-20000 Hz\n\n");
        }
    }
}
