/// crashctl - Crash prediction and management commands
///
/// Provides commands to monitor and control crash prediction system.

impl super::Shell {
    /// Handle crashctl commands
    pub(crate) fn cmd_crashctl(&self, args: &[&str]) {
        if args.is_empty() {
            self.crashctl_help();
            return;
        }

        match args[0] {
            "status" => self.crashctl_status(),
            "history" => self.crashctl_history(&args[1..]),
            "tune" => self.crashctl_tune(&args[1..]),
            "help" | "-h" | "--help" => self.crashctl_help(),
            _ => {
                unsafe { crate::uart_print(b"Unknown crashctl command: "); }
                unsafe { crate::uart_print(args[0].as_bytes()); }
                unsafe { crate::uart_print(b"\nUse 'crashctl help' for usage\n"); }
            }
        }
    }

    /// Show crash prediction status
    fn crashctl_status(&self) {
        #[cfg(feature = "ai-ops")]
        {
            if let Some(status) = crate::ai_insights::get_crash_status() {
                unsafe { crate::uart_print(b"=== Crash Prediction Status ===\n"); }

                unsafe { crate::uart_print(b"Prediction Confidence: "); }
                self.print_number_simple((status.confidence * 100.0) as u64);
                unsafe { crate::uart_print(b"%\n"); }

                // Color-coded confidence level
                let level = if status.confidence >= 0.9 {
                    b"CRITICAL"
                } else if status.confidence >= 0.8 {
                    b"WARNING "  // Padded to 8 bytes
                } else if status.confidence >= 0.6 {
                    b"ELEVATED"
                } else {
                    b"NORMAL  "  // Padded to 8 bytes
                };
                unsafe { crate::uart_print(b"Risk Level: "); }
                unsafe { crate::uart_print(level); }
                unsafe { crate::uart_print(b"\n"); }

                unsafe { crate::uart_print(b"Free Pages: "); }
                self.print_number_simple(status.free_pages as u64);
                unsafe { crate::uart_print(b"\n"); }

                unsafe { crate::uart_print(b"Fragmentation: "); }
                self.print_number_simple((status.fragmentation * 100.0) as u64);
                unsafe { crate::uart_print(b"%\n"); }

                unsafe { crate::uart_print(b"Recent Failures: "); }
                self.print_number_simple(status.recent_failures as u64);
                unsafe { crate::uart_print(b"\n\nRecommendation: "); }
                unsafe { crate::uart_print(status.recommendation.as_bytes()); }
                unsafe { crate::uart_print(b"\n"); }
            } else {
                unsafe { crate::uart_print(b"Crash predictor not initialized\n"); }
            }
        }

        #[cfg(not(feature = "ai-ops"))]
        {
            unsafe { crate::uart_print(b"crashctl requires 'ai-ops' feature\n"); }
        }
    }

    /// Show crash prediction history
    fn crashctl_history(&self, args: &[&str]) {
        #[cfg(feature = "ai-ops")]
        {
            let limit = if !args.is_empty() {
                args[0].parse::<usize>().unwrap_or(10)
            } else {
                10
            };

            let history = crate::ai_insights::get_crash_history();
            if history.is_empty() {
                unsafe { crate::uart_print(b"No prediction history available\n"); }
                return;
            }

            unsafe { crate::uart_print(b"=== Crash Prediction History ===\n"); }
            unsafe { crate::uart_print(b"Time(ms)    Confidence  Free Pages  Fragmentation  Outcome\n"); }
            unsafe { crate::uart_print(b"---------------------------------------------------------------\n"); }

            let start = history.len().saturating_sub(limit);
            for record in &history[start..] {
                let outcome_str = match record.outcome {
                    crate::ai_insights::PredictionOutcome::Pending => b"Pending  ",
                    crate::ai_insights::PredictionOutcome::TruePositive => b"True+    ",
                    crate::ai_insights::PredictionOutcome::TrueNegative => b"True-    ",
                    crate::ai_insights::PredictionOutcome::FalsePositive => b"False+   ",
                    crate::ai_insights::PredictionOutcome::FalseNegative => b"False-   ",
                };

                self.print_number_simple(record.timestamp_ms);
                unsafe { crate::uart_print(b"    "); }
                self.print_number_simple((record.confidence * 100.0) as u64);
                unsafe { crate::uart_print(b"%       "); }
                self.print_number_simple(record.free_pages as u64);
                unsafe { crate::uart_print(b"       "); }
                self.print_number_simple((record.fragmentation * 100.0) as u64);
                unsafe { crate::uart_print(b"%          "); }
                unsafe { crate::uart_print(outcome_str); }
                unsafe { crate::uart_print(b"\n"); }
            }

            // Calculate accuracy if we have resolved predictions
            let mut resolved_count = 0;
            let mut correct_count = 0;
            for record in &history {
                if record.outcome != crate::ai_insights::PredictionOutcome::Pending {
                    resolved_count += 1;
                    if matches!(record.outcome,
                        crate::ai_insights::PredictionOutcome::TruePositive |
                        crate::ai_insights::PredictionOutcome::TrueNegative)
                    {
                        correct_count += 1;
                    }
                }
            }

            if resolved_count > 0 {
                let accuracy = (correct_count as f32 / resolved_count as f32) * 100.0;
                unsafe { crate::uart_print(b"\nAccuracy: "); }
                self.print_number_simple(accuracy as u64);
                unsafe { crate::uart_print(b"% ("); }
                self.print_number_simple(correct_count);
                unsafe { crate::uart_print(b"/"); }
                self.print_number_simple(resolved_count);
                unsafe { crate::uart_print(b" predictions)\n"); }
            }
        }

        #[cfg(not(feature = "ai-ops"))]
        {
            unsafe { crate::uart_print(b"crashctl requires 'ai-ops' feature\n"); }
        }
    }

    /// Tune crash prediction threshold
    fn crashctl_tune(&self, args: &[&str]) {
        #[cfg(feature = "ai-ops")]
        {
            if args.is_empty() {
                unsafe { crate::uart_print(b"Usage: crashctl tune <threshold>\n"); }
                unsafe { crate::uart_print(b"Threshold must be between 0.0 and 1.0\n"); }
                return;
            }

            // Simple float parsing (supports basic format like "0.85")
            let threshold_str = args[0];
            let mut value: f32 = 0.0;
            let mut after_dot = false;
            let mut decimal_places = 0;

            for ch in threshold_str.chars() {
                if ch == '.' {
                    after_dot = true;
                } else if ch.is_ascii_digit() {
                    let digit = ch as u32 - '0' as u32;
                    if after_dot {
                        decimal_places += 1;
                        value = value + (digit as f32) / (10u32.pow(decimal_places) as f32);
                    } else {
                        value = value * 10.0 + digit as f32;
                    }
                } else {
                    unsafe { crate::uart_print(b"Invalid threshold value\n"); }
                    return;
                }
            }

            if value < 0.0 || value > 1.0 {
                unsafe { crate::uart_print(b"Threshold must be between 0.0 and 1.0\n"); }
                return;
            }

            crate::ai_insights::set_crash_threshold(value);

            unsafe { crate::uart_print(b"Crash prediction threshold set to "); }
            self.print_number_simple((value * 100.0) as u64);
            unsafe { crate::uart_print(b"%\n"); }
        }

        #[cfg(not(feature = "ai-ops"))]
        {
            unsafe { crate::uart_print(b"crashctl requires 'ai-ops' feature\n"); }
        }
    }

    /// Show crashctl help
    fn crashctl_help(&self) {
        unsafe { crate::uart_print(b"crashctl - Crash Prediction Management\n\n"); }
        unsafe { crate::uart_print(b"USAGE:\n"); }
        unsafe { crate::uart_print(b"  crashctl <command> [args]\n\n"); }
        unsafe { crate::uart_print(b"COMMANDS:\n"); }
        unsafe { crate::uart_print(b"  status              Show current crash prediction status\n"); }
        unsafe { crate::uart_print(b"  history [N]         Show last N predictions (default: 10)\n"); }
        unsafe { crate::uart_print(b"  tune <threshold>    Set prediction threshold (0.0-1.0)\n"); }
        unsafe { crate::uart_print(b"  help                Show this help message\n\n"); }
        unsafe { crate::uart_print(b"EXAMPLES:\n"); }
        unsafe { crate::uart_print(b"  crashctl status               # Check crash risk\n"); }
        unsafe { crate::uart_print(b"  crashctl history 20           # Show last 20 predictions\n"); }
        unsafe { crate::uart_print(b"  crashctl tune 0.85            # Set 85% confidence threshold\n\n"); }
        unsafe { crate::uart_print(b"NOTES:\n"); }
        unsafe { crate::uart_print(b"  - Confidence levels:\n"); }
        unsafe { crate::uart_print(b"    0-60%:   NORMAL\n"); }
        unsafe { crate::uart_print(b"    60-80%:  ELEVATED\n"); }
        unsafe { crate::uart_print(b"    80-90%:  WARNING\n"); }
        unsafe { crate::uart_print(b"    90-100%: CRITICAL (auto-compaction may trigger)\n"); }
    }
}
