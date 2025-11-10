// Helpers for stresstest commands

impl super::Shell {
    pub(crate) fn stresstest_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe {
                crate::uart_print(b"Usage: stresstest <memory|commands|multi|learning|redteam|chaos|compare|report> [options]\n");
                crate::uart_print(b"  memory: --duration MS --target-pressure PCT (default: 50%) --oom-probability PCT\n");
                crate::uart_print(b"  chaos:  --duration MS --failure-rate PCT\n");
            }
            return;
        }
        match args[0] {
            "memory" => {
                let mut duration_ms: u64 = 10_000; let mut target_pressure: u8 = 50; let mut oom_probability: u8 = 0;
                let mut i = 1; while i + 1 < args.len() { match args[i] { "--duration" => { duration_ms = args[i+1].parse::<u64>().unwrap_or(duration_ms); i+=2; }, "--target-pressure" => { let v = args[i+1].parse::<u16>().unwrap_or(target_pressure as u16); target_pressure = v.min(100) as u8; i+=2; }, "--oom-probability" => { let v = args[i+1].parse::<u16>().unwrap_or(oom_probability as u16); oom_probability = v.min(100) as u8; i+=2; }, _ => { i+=1; } } }
                let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Memory); cfg.duration_ms = duration_ms; cfg.target_pressure = target_pressure; cfg.oom_probability = oom_probability; if oom_probability > 0 { cfg.expect_failures = true; } let metrics = crate::stress_test::run_memory_stress(cfg);
                unsafe { crate::uart_print(b"\n[STRESSTEST] Memory completed: peak_pressure="); self.print_number_simple(metrics.peak_memory_pressure as u64); crate::uart_print(b"% oom_events="); self.print_number_simple(metrics.oom_events as u64); crate::uart_print(b" compactions="); self.print_number_simple(metrics.compaction_triggers as u64); crate::uart_print(b" duration_ms="); self.print_number_simple(metrics.test_duration_ms); crate::uart_print(b"\n"); }
            }
            "commands" => {
                let mut duration_ms: u64 = 10_000; let mut rate: u32 = 50; let mut i = 1; while i + 1 < args.len() { match args[i] { "--duration" => { duration_ms = args[i+1].parse::<u64>().unwrap_or(duration_ms); i+=2; }, "--rate" => { rate = args[i+1].parse::<u32>().unwrap_or(rate); i+=2; }, _ => { i+=1; } } }
                let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Commands); cfg.duration_ms = duration_ms; cfg.command_rate = rate; let metrics = crate::stress_test::run_command_stress(cfg);
                unsafe { crate::uart_print(b"\n[STRESSTEST] Commands completed: actions="); self.print_number_simple(metrics.actions_taken as u64); crate::uart_print(b" duration_ms="); self.print_number_simple(metrics.test_duration_ms); crate::uart_print(b"\n"); }
            }
            "multi" => {
                let mut duration_ms: u64 = 10_000; let mut i = 1; while i + 1 < args.len() { match args[i] { "--duration" => { duration_ms = args[i+1].parse::<u64>().unwrap_or(duration_ms); i+=2; }, _ => { i+=1; } } }
                let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::MultiSubsystem); cfg.duration_ms = duration_ms; let _metrics = crate::stress_test::run_multi_stress(cfg);
            }
            "learning" => {
                let mut episodes: u32 = 10; let mut i = 1; while i + 1 < args.len() { match args[i] { "--episodes" => { episodes = args[i+1].parse::<u32>().unwrap_or(episodes); i+=2; }, _ => { i+=1; } } }
                let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Learning); cfg.episodes = episodes; let metrics = crate::stress_test::run_learning_stress(cfg);
                unsafe { crate::uart_print(b"\n[STRESSTEST] Learning completed: total_rewards="); self.print_number_simple(metrics.total_rewards as u64); crate::uart_print(b" decisions="); self.print_number_simple(metrics.decisions_made as u64); crate::uart_print(b" avg_reward="); self.print_number_simple(metrics.avg_reward_per_decision as u64); crate::uart_print(b"\n"); }
            }
            "redteam" => {
                let mut duration_ms: u64 = 10_000; let mut i = 1; while i + 1 < args.len() { match args[i] { "--duration" => { duration_ms = args[i+1].parse::<u64>().unwrap_or(duration_ms); i+=2; }, _ => { i+=1; } } }
                let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::RedTeam); cfg.duration_ms = duration_ms; let metrics = crate::stress_test::run_redteam_stress(cfg);
                unsafe { crate::uart_print(b"\n[STRESSTEST] Red team completed: attacks_survived="); self.print_number_simple(metrics.actions_taken as u64); crate::uart_print(b" duration_ms="); self.print_number_simple(metrics.test_duration_ms); crate::uart_print(b"\n"); }
            }
            "chaos" => {
                let mut duration_ms: u64 = 10_000;
                let mut failure_rate: u8 = 0;
                let mut i = 1;
                while i + 1 < args.len() {
                    match args[i] {
                        "--duration" => {
                            duration_ms = args[i+1].parse::<u64>().unwrap_or(duration_ms);
                            i+=2;
                        },
                        "--failure-rate" => {
                            let v = args[i+1].parse::<u16>().unwrap_or(failure_rate as u16);
                            failure_rate = v.min(100) as u8;
                            i+=2;
                        },
                        _ => { i+=1; }
                    }
                }
                let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Chaos);
                cfg.duration_ms = duration_ms;
                cfg.fail_rate_percent = failure_rate;
                if failure_rate > 0 { cfg.expect_failures = true; }
                let _metrics = crate::stress_test::run_chaos_stress(cfg);
            }
            "compare" => {
                if args.len() < 2 { unsafe { crate::uart_print(b"Usage: stresstest compare <memory|commands|multi> [--duration MS] [--target-pressure PCT] [--rate RPS]\n"); } return; }
                let mut duration_ms: u64 = 10_000; let mut target_pressure: u8 = 50; let mut rate: u32 = 50; let mut i = 2; while i + 1 < args.len() { match args[i] { "--duration" => { duration_ms = args[i+1].parse::<u64>().unwrap_or(duration_ms); i+=2; }, "--target-pressure" => { let v = args[i+1].parse::<u16>().unwrap_or(target_pressure as u16); target_pressure = v.min(100) as u8; i+=2; }, "--rate" => { rate = args[i+1].parse::<u32>().unwrap_or(rate); i+=2; }, _ => { i+=1; } } }

                let was_enabled = crate::autonomy::AUTONOMOUS_CONTROL.is_enabled();
                let which = args[1];

                // Run with autonomy DISABLED
                unsafe { crate::uart_print(b"\n[COMPARE] Running with autonomy DISABLED...\n"); }
                crate::autonomy::AUTONOMOUS_CONTROL.disable();
                crate::autonomy_metrics::AUTONOMY_METRICS.reset();

                let metrics_off = match which {
                    "memory" => { let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Memory); cfg.duration_ms = duration_ms; cfg.target_pressure = target_pressure; crate::stress_test::run_memory_stress(cfg) },
                    "commands" => { let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Commands); cfg.duration_ms = duration_ms; cfg.command_rate = rate; crate::stress_test::run_command_stress(cfg) },
                    "multi" => { let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::MultiSubsystem); cfg.duration_ms = duration_ms; crate::stress_test::run_multi_stress(cfg) },
                    _ => { unsafe { crate::uart_print(b"[ERROR] Unknown test type\n"); } return; }
                };
                let autonomy_off = crate::autonomy_metrics::AUTONOMY_METRICS.snapshot();

                // Run with autonomy ENABLED
                unsafe { crate::uart_print(b"\n[COMPARE] Running with autonomy ENABLED...\n"); }
                crate::autonomy::AUTONOMOUS_CONTROL.enable();
                crate::autonomy_metrics::AUTONOMY_METRICS.reset();

                #[cfg(target_arch = "aarch64")]
                unsafe { let mut frq: u64; core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq); let interval_ms = crate::autonomy::AUTONOMOUS_CONTROL.decision_interval_ms.load(core::sync::atomic::Ordering::Relaxed).clamp(100, 60_000); let cycles = if frq > 0 { (frq / 1000).saturating_mul(interval_ms) } else { (62_500u64).saturating_mul(interval_ms) }; core::arch::asm!("msr cntv_tval_el0, {x}", x = in(reg) cycles); let ctl: u64 = 1; core::arch::asm!("msr cntv_ctl_el0, {x}", x = in(reg) ctl); }

                let metrics_on = match which {
                    "memory" => { let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Memory); cfg.duration_ms = duration_ms; cfg.target_pressure = target_pressure; crate::stress_test::run_memory_stress(cfg) },
                    "commands" => { let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Commands); cfg.duration_ms = duration_ms; cfg.command_rate = rate; crate::stress_test::run_command_stress(cfg) },
                    "multi" => { let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::MultiSubsystem); cfg.duration_ms = duration_ms; crate::stress_test::run_multi_stress(cfg) },
                    _ => unreachable!(),
                };
                let autonomy_on = crate::autonomy_metrics::AUTONOMY_METRICS.snapshot();

                // Restore autonomy state
                if !was_enabled { crate::autonomy::AUTONOMOUS_CONTROL.disable(); }

                // Enhanced comparative output with autonomy metrics
                unsafe {
                    crate::uart_print(b"\n=== Comparative Results ===\n\n");
                    crate::uart_print(b"Autonomy OFF:\n");
                    match which {
                        "memory" => {
                            crate::uart_print(b"  Peak pressure: "); self.print_number_simple(metrics_off.peak_memory_pressure as u64); crate::uart_print(b"%\n");
                            crate::uart_print(b"  Avg pressure: "); self.print_number_simple(metrics_off.avg_memory_pressure as u64); crate::uart_print(b"%\n");
                            crate::uart_print(b"  OOM events: "); self.print_number_simple(metrics_off.oom_events as u64); crate::uart_print(b"\n");
                        },
                        "commands" | "multi" => {
                            crate::uart_print(b"  Actions: "); self.print_number_simple(metrics_off.actions_taken as u64); crate::uart_print(b"\n");
                        },
                        _ => {}
                    }
                    crate::uart_print(b"  Duration: "); self.print_number_simple(metrics_off.test_duration_ms); crate::uart_print(b" ms\n");
                    crate::uart_print(b"  AI interventions: "); self.print_number_simple(autonomy_off.total_interventions as u64); crate::uart_print(b"\n");

                    crate::uart_print(b"\nAutonomy ON:\n");
                    match which {
                        "memory" => {
                            crate::uart_print(b"  Peak pressure: "); self.print_number_simple(metrics_on.peak_memory_pressure as u64); crate::uart_print(b"%\n");
                            crate::uart_print(b"  Avg pressure: "); self.print_number_simple(metrics_on.avg_memory_pressure as u64); crate::uart_print(b"%\n");
                            crate::uart_print(b"  OOM events: "); self.print_number_simple(metrics_on.oom_events as u64); crate::uart_print(b"\n");
                        },
                        "commands" | "multi" => {
                            crate::uart_print(b"  Actions: "); self.print_number_simple(metrics_on.actions_taken as u64); crate::uart_print(b"\n");
                        },
                        _ => {}
                    }
                    crate::uart_print(b"  Duration: "); self.print_number_simple(metrics_on.test_duration_ms); crate::uart_print(b" ms\n");
                    crate::uart_print(b"  AI interventions: "); self.print_number_simple(autonomy_on.total_interventions as u64); crate::uart_print(b"\n");

                    if autonomy_on.total_interventions > 0 {
                        crate::uart_print(b"    - OOM preventions: "); self.print_number_simple(autonomy_on.preemptive_oom_preventions as u64); crate::uart_print(b"\n");
                        crate::uart_print(b"    - Memory predictions: "); self.print_number_simple(autonomy_on.memory_pressure_predictions as u64); crate::uart_print(b"\n");
                        crate::uart_print(b"    - Proactive compactions: "); self.print_number_simple(autonomy_on.proactive_compactions as u64); crate::uart_print(b"\n");
                    }

                    // Calculate improvements (Enhanced for broader impact visibility)
                    crate::uart_print(b"\nImpact:\n");
                    if which == "memory" {
                        let peak_diff = metrics_off.peak_memory_pressure as i32 - metrics_on.peak_memory_pressure as i32;
                        if peak_diff > 0 {
                            crate::uart_print(b"  [+] Peak pressure reduced by "); self.print_number_simple(peak_diff as u64);
                            crate::uart_print(b"% ("); self.print_number_simple(metrics_off.peak_memory_pressure as u64);
                            crate::uart_print(b"% -> "); self.print_number_simple(metrics_on.peak_memory_pressure as u64);
                            crate::uart_print(b"%)\n");
                        } else if peak_diff < 0 {
                            crate::uart_print(b"  [-] Peak pressure increased by "); self.print_number_simple((-peak_diff) as u64); crate::uart_print(b"%\n");
                        } else {
                            crate::uart_print(b"  - Peak pressure unchanged\n");
                        }

                        let avg_diff = metrics_off.avg_memory_pressure as i32 - metrics_on.avg_memory_pressure as i32;
                        if avg_diff > 0 {
                            crate::uart_print(b"  [+] Avg pressure reduced by "); self.print_number_simple(avg_diff as u64);
                            crate::uart_print(b"% ("); self.print_number_simple(metrics_off.avg_memory_pressure as u64);
                            crate::uart_print(b"% -> "); self.print_number_simple(metrics_on.avg_memory_pressure as u64);
                            crate::uart_print(b"%)\n");
                        } else if avg_diff < 0 {
                            crate::uart_print(b"  [-] Avg pressure increased by "); self.print_number_simple((-avg_diff) as u64); crate::uart_print(b"%\n");
                        } else {
                            crate::uart_print(b"  - Avg pressure unchanged\n");
                        }

                        if autonomy_on.proactive_compactions > 0 {
                            crate::uart_print(b"  [+] Proactive compactions: "); self.print_number_simple(autonomy_on.proactive_compactions as u64); crate::uart_print(b"\n");
                        }

                        let oom_diff = metrics_off.oom_events as i32 - metrics_on.oom_events as i32;
                        if oom_diff > 0 {
                            crate::uart_print(b"  [+] OOM events reduced by "); self.print_number_simple(oom_diff as u64);
                            crate::uart_print(b" ("); self.print_number_simple(metrics_off.oom_events as u64); crate::uart_print(b" -> ");
                            self.print_number_simple(metrics_on.oom_events as u64); crate::uart_print(b")\n");
                        } else if oom_diff < 0 {
                            crate::uart_print(b"  [-] OOM events increased\n");
                        } else {
                            crate::uart_print(b"  - No change in OOM events\n");
                        }
                    }

                    if autonomy_on.total_interventions > 0 {
                        crate::uart_print(b"  [+] "); self.print_number_simple(autonomy_on.total_interventions as u64);
                        crate::uart_print(b" AI interventions (");
                        self.print_number_simple(autonomy_on.proactive_compactions as u64);
                        crate::uart_print(b" early, ");
                        let reactive = autonomy_on.total_interventions.saturating_sub(autonomy_on.proactive_compactions);
                        self.print_number_simple(reactive as u64);
                        crate::uart_print(b" reactive)\n");
                    } else {
                        crate::uart_print(b"  [!] No AI interventions (autonomy may not be active)\n");
                    }
                    crate::uart_print(b"\n");
                }
            }
            "report" => {
                let hist = crate::stress_test::get_history(); unsafe { crate::uart_print(b"\n=== Stress Test History (last 16) ===\n"); }
                let mut any = false; for rec in hist.iter() { any = true; unsafe { crate::uart_print(b"  Type: "); match rec.test_type { crate::stress_test::StressTestType::Memory => crate::uart_print(b"memory"), crate::stress_test::StressTestType::Commands => crate::uart_print(b"commands"), crate::stress_test::StressTestType::MultiSubsystem => crate::uart_print(b"multi"), crate::stress_test::StressTestType::Learning => crate::uart_print(b"learning"), crate::stress_test::StressTestType::RedTeam => crate::uart_print(b"redteam"), crate::stress_test::StressTestType::Chaos => crate::uart_print(b"chaos"), } crate::uart_print(b" | Duration: "); self.print_number_simple(rec.metrics.test_duration_ms as u64); crate::uart_print(b" ms | Actions: "); self.print_number_simple(rec.metrics.actions_taken as u64); crate::uart_print(b" | OOM: "); self.print_number_simple(rec.metrics.oom_events as u64); crate::uart_print(b"\n"); } }
                if !any { unsafe { crate::uart_print(b"  (no history)\n"); } }
            }
            _ => unsafe { crate::uart_print(b"Usage: stresstest <memory|commands|multi|learning|redteam|chaos|compare|report> [options]\n"); }
        }
    }
}
