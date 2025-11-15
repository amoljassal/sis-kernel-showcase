// Helpers for mlctl commands (advanced ML control)

impl super::Shell {
    pub(crate) fn mlctl_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: mlctl <status|replay N|weights P W L|features>\n"); }
            return;
        }
        match args[0] {
            "status" => { crate::meta_agent::print_advanced_ml_status(); }
            "replay" => {
                if args.len() < 2 { unsafe { crate::uart_print(b"Usage: mlctl replay <batch_size>\n"); } return; }
                let batch_size = args[1].parse::<usize>().unwrap_or(10);
                unsafe { crate::uart_print(b"[ML] Training from replay buffer (batch_size="); self.print_number_simple(batch_size as u64); crate::uart_print(b")...\n"); }
                crate::meta_agent::train_from_replay(batch_size);
                let (count, capacity) = crate::meta_agent::get_replay_stats();
                unsafe { crate::uart_print(b"[ML] Replay buffer: "); self.print_number_simple(count as u64); crate::uart_print(b"/"); self.print_number_simple(capacity as u64); crate::uart_print(b" entries\n"); }
            }
            "weights" => {
                if args.len() < 4 { unsafe { crate::uart_print(b"Usage: mlctl weights <perf> <power> <latency>\n"); } return; }
                let perf = args[1].parse::<u8>().unwrap_or(40).min(100);
                let power = args[2].parse::<u8>().unwrap_or(30).min(100);
                let latency = args[3].parse::<u8>().unwrap_or(30).min(100);
                let mut config = crate::meta_agent::get_meta_config();
                config.performance_weight = perf; config.power_weight = power; config.latency_weight = latency; crate::meta_agent::set_meta_config(config);
                unsafe {
                    crate::uart_print(b"[ML] Reward weights updated:\n");
                    crate::uart_print(b"  Performance: "); self.print_number_simple(perf as u64); crate::uart_print(b"%\n");
                    crate::uart_print(b"  Power: "); self.print_number_simple(power as u64); crate::uart_print(b"%\n");
                    crate::uart_print(b"  Latency: "); self.print_number_simple(latency as u64); crate::uart_print(b"%\n");
                }
            }
            "features" => {
                let mut config = crate::meta_agent::get_meta_config();
                let mut i = 1;
                while i < args.len() {
                    match args[i] {
                        "--replay" if i + 1 < args.len() => { config.replay_enabled = args[i + 1] == "on"; i += 2; }
                        "--td" if i + 1 < args.len() => { config.td_learning_enabled = args[i + 1] == "on"; i += 2; }
                        "--topology" if i + 1 < args.len() => { config.topology_adapt_enabled = args[i + 1] == "on"; i += 2; }
                        _ => { i += 1; }
                    }
                }
                crate::meta_agent::set_meta_config(config);
                unsafe {
                    crate::uart_print(b"[ML] Feature configuration updated:\n");
                    crate::uart_print(b"  Experience Replay: "); crate::uart_print(if config.replay_enabled { b"ON\n" } else { b"OFF\n" });
                    crate::uart_print(b"  TD Learning: "); crate::uart_print(if config.td_learning_enabled { b"ON\n" } else { b"OFF\n" });
                    crate::uart_print(b"  Topology Adaptation: "); crate::uart_print(if config.topology_adapt_enabled { b"ON\n" } else { b"OFF\n" });
                }
            }
            _ => unsafe { crate::uart_print(b"Usage: mlctl <status|replay N|weights P W L|features>\n"); }
        }
    }
}

