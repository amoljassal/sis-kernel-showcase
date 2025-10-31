// Helper for autoctl status printing

impl super::Shell {
    pub(crate) fn print_autoctl_status(&self) {
        let enabled = crate::autonomy::AUTONOMOUS_CONTROL.is_enabled();
        let safe_mode = crate::autonomy::AUTONOMOUS_CONTROL.is_safe_mode();
        let total_decisions = crate::autonomy::AUTONOMOUS_CONTROL
            .total_decisions
            .load(core::sync::atomic::Ordering::Relaxed);
        let interval_ms = crate::autonomy::AUTONOMOUS_CONTROL
            .decision_interval_ms
            .load(core::sync::atomic::Ordering::Relaxed);
        let learning_frozen = crate::autonomy::AUTONOMOUS_CONTROL
            .learning_frozen
            .load(core::sync::atomic::Ordering::Relaxed);

        unsafe {
            crate::uart_print(b"\n=== Autonomous Control Status ===\n");
            crate::uart_print(b"  Mode: ");
            crate::uart_print(if enabled { b"ENABLED\n" } else { b"DISABLED\n" });
            crate::uart_print(b"  Safe Mode: ");
            crate::uart_print(if safe_mode { b"ACTIVE\n" } else { b"INACTIVE\n" });
            crate::uart_print(b"  Learning: ");
            crate::uart_print(if learning_frozen { b"FROZEN\n" } else { b"ACTIVE\n" });
            crate::uart_print(b"  Decision Interval: ");
            self.print_number_simple(interval_ms);
            crate::uart_print(b" ms\n");
            crate::uart_print(b"  Total Decisions: ");
            self.print_number_simple(total_decisions);
            crate::uart_print(b"\n");
        }

        let audit_log = crate::autonomy::get_audit_log();
        unsafe {
            crate::uart_print(b"  Audit Log: ");
            self.print_number_simple(audit_log.len() as u64);
            crate::uart_print(b"/1000 entries\n");
        }
        drop(audit_log);

        // Prediction accuracy trend (last 100/500)
        {
            let (correct_100, total_100) = crate::prediction_tracker::compute_accuracy(100);
            let (correct_500, total_500) = crate::prediction_tracker::compute_accuracy(500);
            unsafe {
                crate::uart_print(b"  Accuracy (last 100): ");
                if total_100 > 0 {
                    self.print_number_simple((correct_100 * 100 / total_100) as u64);
                    crate::uart_print(b"%\n");
                } else {
                    crate::uart_print(b"N/A\n");
                }
                crate::uart_print(b"  Accuracy (last 500): ");
                if total_500 > 0 {
                    self.print_number_simple((correct_500 * 100 / total_500) as u64);
                    crate::uart_print(b"%\n");
                } else {
                    crate::uart_print(b"N/A\n");
                }
            }
        }

        let watchdog = crate::autonomy::get_watchdog();
        unsafe {
            crate::uart_print(b"  Watchdog Triggers: ");
            self.print_number_simple(watchdog.consecutive_low_rewards as u64);
            crate::uart_print(b" low rewards, ");
            self.print_number_simple(watchdog.consecutive_high_td_errors as u64);
            crate::uart_print(b" high TD errors\n\n");
        }
        drop(watchdog);
    }
}

