// Chaos engineering shell commands
// Phase 3.1 - Production Readiness Plan

impl super::Shell {
    /// Control chaos engineering mode
    pub(crate) fn cmd_chaos(&self, args: &[&str]) {
        #[cfg(feature = "chaos")]
        {
            use crate::chaos::{ChaosMode, get_mode, set_mode, set_failure_rate, get_stats, reset_stats};

            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: chaos <mode|rate|stats|reset>\n");
                    crate::uart_print(b"\n");
                    crate::uart_print(b"Modes:\n");
                    crate::uart_print(b"  none            - Disable chaos injection\n");
                    crate::uart_print(b"  disk_full       - Inject ENOSPC errors\n");
                    crate::uart_print(b"  disk_fail       - Inject EIO errors\n");
                    crate::uart_print(b"  network_fail    - Inject ENETDOWN errors\n");
                    crate::uart_print(b"  memory_pressure - Inject ENOMEM errors\n");
                    crate::uart_print(b"  random_panic    - Inject random panics\n");
                    crate::uart_print(b"  slow_io         - Inject I/O delays\n");
                    crate::uart_print(b"\n");
                    crate::uart_print(b"Commands:\n");
                    crate::uart_print(b"  chaos mode <mode>   - Set chaos mode\n");
                    crate::uart_print(b"  chaos rate <0-100>  - Set failure rate percentage\n");
                    crate::uart_print(b"  chaos stats         - Show chaos statistics\n");
                    crate::uart_print(b"  chaos reset         - Reset statistics\n");
                }
                return;
            }

            match args[0] {
                "mode" => {
                    if args.len() < 2 {
                        let mode = get_mode();
                        unsafe {
                            crate::uart_print(b"Current mode: ");
                            crate::uart_print(mode.as_str().as_bytes());
                            crate::uart_print(b"\n");
                        }
                        return;
                    }

                    let mode = match args[1] {
                        "none" => ChaosMode::None,
                        "disk_full" => ChaosMode::DiskFull,
                        "disk_fail" => ChaosMode::DiskFail,
                        "network_fail" => ChaosMode::NetworkFail,
                        "memory_pressure" => ChaosMode::MemoryPressure,
                        "random_panic" => ChaosMode::RandomPanic,
                        "slow_io" => ChaosMode::SlowIo,
                        _ => {
                            unsafe { crate::uart_print(b"Unknown mode. Use 'chaos' for help.\n"); }
                            return;
                        }
                    };

                    set_mode(mode);
                    unsafe {
                        crate::uart_print(b"Chaos mode set to: ");
                        crate::uart_print(mode.as_str().as_bytes());
                        crate::uart_print(b"\n");
                    }
                }

                "rate" => {
                    if args.len() < 2 {
                        unsafe { crate::uart_print(b"Usage: chaos rate <0-100>\n"); }
                        return;
                    }

                    // Parse rate
                    let rate_str = args[1];
                    let mut rate = 0u32;
                    for c in rate_str.bytes() {
                        if c >= b'0' && c <= b'9' {
                            rate = rate * 10 + (c - b'0') as u32;
                        } else {
                            unsafe { crate::uart_print(b"Invalid rate. Use 0-100.\n"); }
                            return;
                        }
                    }

                    if rate > 100 {
                        unsafe { crate::uart_print(b"Rate must be 0-100.\n"); }
                        return;
                    }

                    set_failure_rate(rate);
                    unsafe {
                        crate::uart_print(b"Failure rate set to ");
                        crate::trace::print_usize(rate as usize);
                        crate::uart_print(b"%\n");
                    }
                }

                "stats" => {
                    let stats = get_stats();
                    unsafe {
                        crate::uart_print(b"Chaos Statistics:\n");
                        crate::uart_print(b"  Mode:              ");
                        crate::uart_print(stats.mode.as_str().as_bytes());
                        crate::uart_print(b"\n");
                        crate::uart_print(b"  Failure rate:      ");
                        crate::trace::print_usize(stats.failure_rate as usize);
                        crate::uart_print(b"%\n");
                        crate::uart_print(b"  Disk full:         ");
                        crate::trace::print_usize(stats.disk_full_count as usize);
                        crate::uart_print(b"\n");
                        crate::uart_print(b"  Disk failures:     ");
                        crate::trace::print_usize(stats.disk_fail_count as usize);
                        crate::uart_print(b"\n");
                        crate::uart_print(b"  Network failures:  ");
                        crate::trace::print_usize(stats.network_fail_count as usize);
                        crate::uart_print(b"\n");
                        crate::uart_print(b"  Alloc failures:    ");
                        crate::trace::print_usize(stats.alloc_fail_count as usize);
                        crate::uart_print(b"\n");
                    }
                }

                "reset" => {
                    reset_stats();
                    unsafe { crate::uart_print(b"Chaos statistics reset.\n"); }
                }

                _ => {
                    unsafe { crate::uart_print(b"Unknown command. Use 'chaos' for help.\n"); }
                }
            }
        }

        #[cfg(not(feature = "chaos"))]
        {
            unsafe {
                crate::uart_print(b"Chaos engineering not enabled.\n");
                crate::uart_print(b"Rebuild with --features chaos to enable.\n");
            }
        }
    }
}
