// Helpers for metaclassctl commands (meta-agent control)

impl super::Shell {
    pub(crate) fn metaclassctl_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: metaclassctl <status|force|config|on|off>\n"); }
            return;
        }
        match args[0] {
            "status" => {
                crate::meta_agent::print_meta_status();
            }
            "force" => {
                unsafe { crate::uart_print(b"[META] Forcing immediate decision...\n"); }
                let state = crate::meta_agent::collect_telemetry();
                unsafe { crate::uart_print(b"[META] Telemetry collected\n"); }
                let decision = crate::meta_agent::force_meta_decision();
                unsafe { crate::uart_print(b"[META] Decision executed:\n"); }
                unsafe { crate::uart_print(b"  Memory directive: "); }
                self.print_number_simple(decision.memory_directive.abs() as u64);
                unsafe { crate::uart_print(if decision.memory_directive < 0 { b" (negative)\n" } else { b" (positive)\n" }); }
                unsafe { crate::uart_print(b"  Scheduling directive: "); }
                self.print_number_simple(decision.scheduling_directive.abs() as u64);
                unsafe { crate::uart_print(if decision.scheduling_directive < 0 { b" (negative)\n" } else { b" (positive)\n" }); }
                unsafe { crate::uart_print(b"  Command directive: "); }
                self.print_number_simple(decision.command_directive.abs() as u64);
                unsafe { crate::uart_print(if decision.command_directive < 0 { b" (negative)\n" } else { b" (positive)\n" }); }
                unsafe { crate::uart_print(b"  Confidence: "); }
                self.print_number_simple(decision.confidence as u64);
                unsafe { crate::uart_print(b"/1000\n"); }
                unsafe { crate::uart_print(b"\n[META] State at decision time:\n"); }
                unsafe { crate::uart_print(b"  Memory pressure: "); }
                self.print_number_simple(state.memory_pressure as u64);
                unsafe { crate::uart_print(b"%\n"); }
            }
            "config" => {
                let mut config = crate::meta_agent::get_meta_config();
                let mut i = 1;
                while i < args.len() {
                    match args[i] {
                        "--interval" if i + 1 < args.len() => {
                            if let Ok(ms) = args[i + 1].parse::<u64>() { config.decision_interval_us = ms * 1000; i += 2; }
                            else { unsafe { crate::uart_print(b"Invalid interval value\n"); } return; }
                        }
                        "--threshold" if i + 1 < args.len() => {
                            if let Ok(thresh) = args[i + 1].parse::<u16>() { config.confidence_threshold = thresh.min(1000); i += 2; }
                            else { unsafe { crate::uart_print(b"Invalid threshold value\n"); } return; }
                        }
                        _ => { unsafe { crate::uart_print(b"Unknown config option\n"); } return; }
                    }
                }
                crate::meta_agent::set_meta_config(config);
                unsafe { crate::uart_print(b"[META] Configuration updated\n"); }
                unsafe { crate::uart_print(b"  Interval: "); }
                self.print_number_simple((config.decision_interval_us / 1000) as u64);
                unsafe { crate::uart_print(b" ms\n"); }
                unsafe { crate::uart_print(b"  Threshold: "); }
                self.print_number_simple(config.confidence_threshold as u64);
                unsafe { crate::uart_print(b"/1000\n"); }
            }
            "on" => {
                let mut config = crate::meta_agent::get_meta_config();
                config.enabled = true;
                crate::meta_agent::set_meta_config(config);
                unsafe { crate::uart_print(b"[META] Meta-agent enabled\n"); }
            }
            "off" => {
                let mut config = crate::meta_agent::get_meta_config();
                config.enabled = false;
                crate::meta_agent::set_meta_config(config);
                unsafe { crate::uart_print(b"[META] Meta-agent disabled\n"); }
            }
            _ => unsafe { crate::uart_print(b"Usage: metaclassctl <status|force|config|on|off>\n"); }
        }
    }
}

