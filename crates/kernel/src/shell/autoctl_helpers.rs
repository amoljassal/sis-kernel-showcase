// Helper for autoctl status printing

impl super::Shell {
    pub(crate) fn print_autoctl_status(&self) {
        let enabled = crate::autonomy::AUTONOMOUS_CONTROL.is_enabled();
        let safe_mode = crate::autonomy::AUTONOMOUS_CONTROL.is_safe_mode();
        let ready = crate::autonomy::is_ready();
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
            crate::uart_print(b"  Ready Flag: ");
            crate::uart_print(if ready { b"SET (timer will call tick)\n" } else { b"NOT SET (timer will NOT call tick)\n" });
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

    /// Show canary rollout status
    pub(crate) fn autoctl_rollout_status(&self) {
        let rollout = crate::command_predictor::CANARY_ROLLOUT.lock();
        unsafe {
            crate::uart_print(b"[CANARY_ROLLOUT] Status:\n");
            crate::uart_print(b"  Current percentage: ");
            crate::uart_print(rollout.percentage.as_str().as_bytes());
            crate::uart_print(b"\n  Total decisions: ");
            self.print_number_simple(rollout.decisions_made as u64);
            crate::uart_print(b"\n  Autonomous decisions: ");
            self.print_number_simple(rollout.decisions_autonomous as u64);
            crate::uart_print(b"\n  Baseline reward: ");
            self.print_number_simple((rollout.baseline_reward / 256) as u64);
            crate::uart_print(b".");
            let frac = ((rollout.baseline_reward % 256) * 100) / 256;
            self.print_number_simple(frac.abs() as u64);
            crate::uart_print(b"\n  Auto-rollback threshold: ");
            self.print_number_simple((rollout.auto_rollback_threshold.abs() / 256) as u64);
            crate::uart_print(b".");
            let frac = ((rollout.auto_rollback_threshold.abs() % 256) * 100) / 256;
            self.print_number_simple(frac as u64);
            crate::uart_print(b"\n");
        }
    }

    /// Set canary rollout percentage
    pub(crate) fn autoctl_rollout_set(&self, percentage_str: &str) {
        let mut rollout = crate::command_predictor::CANARY_ROLLOUT.lock();

        use crate::command_predictor::RolloutPercentage;
        let new_percentage = match percentage_str {
            "0" => RolloutPercentage::Disabled,
            "1" => RolloutPercentage::OnePercent,
            "5" => RolloutPercentage::FivePercent,
            "10" => RolloutPercentage::TenPercent,
            "50" => RolloutPercentage::FiftyPercent,
            "100" => RolloutPercentage::Full,
            "advance" => {
                rollout.advance();
                unsafe { crate::uart_print(b"[CANARY_ROLLOUT] Advanced to next stage: "); }
                unsafe { crate::uart_print(rollout.percentage.as_str().as_bytes()); }
                unsafe { crate::uart_print(b"\n"); }
                return;
            }
            "rollback" => {
                rollout.rollback();
                unsafe { crate::uart_print(b"[CANARY_ROLLOUT] Rolled back to previous stage: "); }
                unsafe { crate::uart_print(rollout.percentage.as_str().as_bytes()); }
                unsafe { crate::uart_print(b"\n"); }
                return;
            }
            _ => {
                unsafe { crate::uart_print(b"Usage: autoctl rollout <0|1|5|10|50|100|advance|rollback|status>\n"); }
                return;
            }
        };

        rollout.percentage = new_percentage;
        unsafe { crate::uart_print(b"[CANARY_ROLLOUT] Set to "); }
        unsafe { crate::uart_print(rollout.percentage.as_str().as_bytes()); }
        unsafe { crate::uart_print(b"\n"); }
    }

    /// Show circuit breaker status
    pub(crate) fn autoctl_circuit_breaker_status(&self) {
        let breaker = crate::command_predictor::CIRCUIT_BREAKER.lock();
        unsafe {
            crate::uart_print(b"[CIRCUIT_BREAKER] Status:\n");
            crate::uart_print(b"  State: ");
            crate::uart_print(breaker.state.as_str().as_bytes());
            crate::uart_print(b"\n  Consecutive failures: ");
            self.print_number_simple(breaker.consecutive_failures as u64);
            crate::uart_print(b"/");
            self.print_number_simple(breaker.failure_threshold as u64);
            crate::uart_print(b"\n  Success count (half-open): ");
            self.print_number_simple(breaker.success_count as u64);
            crate::uart_print(b"/");
            self.print_number_simple(breaker.test_threshold as u64);
            crate::uart_print(b"\n  Total trips: ");
            self.print_number_simple(breaker.total_trips as u64);
            crate::uart_print(b"\n  Reset timeout: ");
            self.print_number_simple(breaker.reset_timeout_us / 1_000_000);
            crate::uart_print(b" seconds\n");
            crate::uart_print(b"  Autonomous allowed: ");
            crate::uart_print(if breaker.is_autonomous_allowed() { b"YES\n" } else { b"NO\n" });
        }
    }

    /// Reset circuit breaker
    pub(crate) fn autoctl_circuit_breaker_reset(&self) {
        let mut breaker = crate::command_predictor::CIRCUIT_BREAKER.lock();
        breaker.state = crate::command_predictor::CircuitState::Closed;
        breaker.consecutive_failures = 0;
        breaker.success_count = 0;
        unsafe { crate::uart_print(b"[CIRCUIT_BREAKER] Manually reset to CLOSED\n"); }
    }
}

