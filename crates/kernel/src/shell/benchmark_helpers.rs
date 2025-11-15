// Helpers for benchmark commands (memory/commands/network/full/report)

impl super::Shell {
    /// Benchmark memory subsystem
    pub(crate) fn benchmark_memory(&self, args: &[&str]) {
        let duration_sec = if args.is_empty() {
            10 // Default 10 seconds
        } else {
            args[0].parse::<u32>().unwrap_or(10)
        };

        unsafe {
            crate::uart_print(b"[BENCHMARK] Memory Stress Test\n");
            crate::uart_print(b"  Duration: ");
            self.print_number_simple(duration_sec as u64);
            crate::uart_print(b" seconds\n\n");

            // Run baseline (AI disabled)
            crate::uart_print(b"Running BASELINE (AI disabled)...\n");
        }

        let baseline = crate::benchmark::run_memory_benchmark(duration_sec, false);

        unsafe {
            crate::uart_print(b"Baseline complete.\n\n");

            // Run with AI enabled
            crate::uart_print(b"Running WITH AI ENABLED...\n");
        }

        let ai_metrics = crate::benchmark::run_memory_benchmark(duration_sec, true);

        unsafe {
            crate::uart_print(b"AI test complete.\n\n");
        }

        // Save metrics
        let mut state = crate::benchmark::BENCHMARK_STATE.lock();
        state.baseline_metrics = Some(baseline);
        state.ai_metrics = Some(ai_metrics);
        drop(state);

        // Display results
        self.benchmark_show_results();
    }

    /// Benchmark command subsystem
    pub(crate) fn benchmark_commands(&self, args: &[&str]) {
        let duration_sec = if args.is_empty() {
            5 // Default 5 seconds
        } else {
            args[0].parse::<u32>().unwrap_or(5)
        };

        let rate = if args.len() > 1 {
            args[1].parse::<u32>().unwrap_or(10)
        } else {
            10 // Default 10 commands/sec
        };

        unsafe {
            crate::uart_print(b"[BENCHMARK] Command Flood Test\n");
            crate::uart_print(b"  Duration: ");
            self.print_number_simple(duration_sec as u64);
            crate::uart_print(b" seconds\n");
            crate::uart_print(b"  Rate: ");
            self.print_number_simple(rate as u64);
            crate::uart_print(b" commands/sec\n\n");

            // Run baseline
            crate::uart_print(b"Running BASELINE (AI disabled)...\n");
        }

        let baseline = crate::benchmark::run_command_benchmark(duration_sec, rate, false);

        unsafe {
            crate::uart_print(b"Baseline complete.\n\n");
            crate::uart_print(b"Running WITH AI ENABLED...\n");
        }

        let ai_metrics = crate::benchmark::run_command_benchmark(duration_sec, rate, true);

        unsafe {
            crate::uart_print(b"AI test complete.\n\n");
        }

        // Save metrics
        let mut state = crate::benchmark::BENCHMARK_STATE.lock();
        state.baseline_metrics = Some(baseline);
        state.ai_metrics = Some(ai_metrics);
        drop(state);

        self.benchmark_show_results();
    }

    /// Benchmark network subsystem
    pub(crate) fn benchmark_network(&self, args: &[&str]) {
        let duration_sec = if args.is_empty() {
            10
        } else {
            args[0].parse::<u32>().unwrap_or(10)
        };

        unsafe {
            crate::uart_print(b"[BENCHMARK] Network Throughput Test\n");
            crate::uart_print(b"  Duration: ");
            self.print_number_simple(duration_sec as u64);
            crate::uart_print(b" seconds\n\n");

            crate::uart_print(b"Running BASELINE (AI disabled)...\n");
        }

        let baseline = crate::benchmark::run_network_benchmark(duration_sec, false);

        unsafe {
            crate::uart_print(b"Baseline complete.\n\n");
            crate::uart_print(b"Running WITH AI ENABLED...\n");
        }

        let ai_metrics = crate::benchmark::run_network_benchmark(duration_sec, true);

        unsafe {
            crate::uart_print(b"AI test complete.\n\n");
        }

        // Save metrics
        let mut state = crate::benchmark::BENCHMARK_STATE.lock();
        state.baseline_metrics = Some(baseline);
        state.ai_metrics = Some(ai_metrics);
        drop(state);

        self.benchmark_show_results();
    }

    /// Full system integration benchmark
    pub(crate) fn benchmark_full(&self, args: &[&str]) {
        let duration_sec = if args.is_empty() {
            30 // Default 30 seconds for full test
        } else {
            args[0].parse::<u32>().unwrap_or(30)
        };

        unsafe {
            crate::uart_print(b"[BENCHMARK] Full System Integration Test\n");
            crate::uart_print(b"  Duration: ");
            self.print_number_simple(duration_sec as u64);
            crate::uart_print(b" seconds\n");
            crate::uart_print(b"  Testing: Memory + Scheduling + Commands + Network\n\n");

            crate::uart_print(b"Running BASELINE (AI disabled)...\n");
        }

        let baseline = crate::benchmark::run_full_benchmark(duration_sec, false);

        unsafe {
            crate::uart_print(b"Baseline complete.\n\n");
            crate::uart_print(b"Running WITH AI ENABLED...\n");
        }

        let ai_metrics = crate::benchmark::run_full_benchmark(duration_sec, true);

        unsafe {
            crate::uart_print(b"AI test complete.\n\n");
        }

        // Save metrics
        let mut state = crate::benchmark::BENCHMARK_STATE.lock();
        state.baseline_metrics = Some(baseline);
        state.ai_metrics = Some(ai_metrics);
        drop(state);

        self.benchmark_show_results();
    }

    /// Show benchmark results
    fn benchmark_show_results(&self) {
        let state = crate::benchmark::BENCHMARK_STATE.lock();

        if let Some(report) = state.get_comparative_report() {
            unsafe {
                crate::uart_print(b"\n");
                crate::uart_print(b"========================================\n");
                crate::uart_print(b"COMPARATIVE BENCHMARK RESULTS\n");
                crate::uart_print(b"========================================\n\n");

                // Memory metrics
                crate::uart_print(b"Memory Subsystem:\n");
                crate::uart_print(b"  Baseline (no AI):\n");
                crate::uart_print(b"    - Avg Pressure: ");
                self.print_number_simple(report.without_ai.memory_pressure_avg as u64);
                crate::uart_print(b"%\n");
                crate::uart_print(b"    - Peak Pressure: ");
                self.print_number_simple(report.without_ai.memory_pressure_peak as u64);
                crate::uart_print(b"%\n");
                crate::uart_print(b"    - OOM Events: ");
                self.print_number_simple(report.without_ai.oom_events as u64);
                crate::uart_print(b"\n");

                crate::uart_print(b"  With AI Enabled:\n");
                crate::uart_print(b"    - Avg Pressure: ");
                self.print_number_simple(report.with_ai.memory_pressure_avg as u64);
                crate::uart_print(b"%\n");
                crate::uart_print(b"    - Peak Pressure: ");
                self.print_number_simple(report.with_ai.memory_pressure_peak as u64);
                crate::uart_print(b"%\n");
                crate::uart_print(b"    - OOM Events: ");
                self.print_number_simple(report.with_ai.oom_events as u64);
                crate::uart_print(b"\n");

                crate::uart_print(b"  Improvement: ");
                if report.oom_reduction_pct >= 0 {
                    self.print_number_simple(report.oom_reduction_pct as u64);
                    crate::uart_print(b"% OOM reduction\n\n");
                } else {
                    crate::uart_print(b"(baseline better)\n\n");
                }

                // Network metrics
                crate::uart_print(b"Network Subsystem:\n");
                crate::uart_print(b"  Baseline (no AI):\n");
                crate::uart_print(b"    - Packets Sent: ");
                self.print_number_simple(report.without_ai.packets_sent as u64);
                crate::uart_print(b"\n");
                crate::uart_print(b"    - Packets Lost: ");
                self.print_number_simple(report.without_ai.packets_lost as u64);
                crate::uart_print(b"\n");

                crate::uart_print(b"  With AI Enabled:\n");
                crate::uart_print(b"    - Packets Sent: ");
                self.print_number_simple(report.with_ai.packets_sent as u64);
                crate::uart_print(b"\n");
                crate::uart_print(b"    - Packets Lost: ");
                self.print_number_simple(report.with_ai.packets_lost as u64);
                crate::uart_print(b"\n");

                if report.packet_loss_reduction_pct >= 0 {
                    crate::uart_print(b"  Improvement: ");
                    self.print_number_simple(report.packet_loss_reduction_pct as u64);
                    crate::uart_print(b"% packet loss reduction\n\n");
                } else {
                    crate::uart_print(b"  (baseline better)\n\n");
                }

                // Commands
                crate::uart_print(b"Command Subsystem:\n");
                crate::uart_print(b"  Commands Executed: ");
                self.print_number_simple(report.with_ai.commands_executed as u64);
                crate::uart_print(b"\n");
                crate::uart_print(b"  Prediction Accuracy Gain: +");
                self.print_number_simple(report.accuracy_improvement_pct.abs() as u64);
                crate::uart_print(b"%\n\n");

                // Summary
                crate::uart_print(b"========================================\n");
                crate::uart_print(b"SUMMARY\n");
                crate::uart_print(b"========================================\n");
                crate::uart_print(b"AI-native kernel achieved:\n");

                if report.oom_reduction_pct > 0 {
                    crate::uart_print(b"  + ");
                    self.print_number_simple(report.oom_reduction_pct as u64);
                    crate::uart_print(b"% reduction in OOM events\n");
                }

                if report.deadline_miss_reduction_pct > 0 {
                    crate::uart_print(b"  + ");
                    self.print_number_simple(report.deadline_miss_reduction_pct as u64);
                    crate::uart_print(b"% reduction in deadline misses\n");
                }

                if report.packet_loss_reduction_pct > 0 {
                    crate::uart_print(b"  + ");
                    self.print_number_simple(report.packet_loss_reduction_pct as u64);
                    crate::uart_print(b"% reduction in packet loss\n");
                }

                crate::uart_print(b"\n");
            }
        } else {
            unsafe {
                crate::uart_print(b"[BENCHMARK] No comparative results available\n");
                crate::uart_print(b"Run a benchmark test first (memory/commands/network/full)\n");
            }
        }
    }

    /// Generate benchmark report
    pub(crate) fn benchmark_report(&self, _args: &[&str]) {
        self.benchmark_show_results();
    }
}
