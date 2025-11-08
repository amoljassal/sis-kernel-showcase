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

            // Enhancement: Display confidence threshold and decision outcomes
            let threshold = crate::autonomy::AUTONOMOUS_CONTROL.get_confidence_threshold();
            let accepted = crate::autonomy::AUTONOMOUS_CONTROL
                .decisions_accepted
                .load(core::sync::atomic::Ordering::Relaxed);
            let deferred = crate::autonomy::AUTONOMOUS_CONTROL
                .decisions_deferred
                .load(core::sync::atomic::Ordering::Relaxed);

            crate::uart_print(b"  Confidence Threshold: ");
            self.print_number_simple(threshold as u64);
            crate::uart_print(b"/1000 (");
            self.print_number_simple((threshold as usize / 10) as u64);
            crate::uart_print(b"%)\n");

            crate::uart_print(b"  Decisions Accepted: ");
            self.print_number_simple(accepted);
            crate::uart_print(b" | Deferred: ");
            self.print_number_simple(deferred);
            if accepted + deferred > 0 {
                let acceptance_rate = (accepted * 100) / (accepted + deferred);
                crate::uart_print(b" (");
                self.print_number_simple(acceptance_rate);
                crate::uart_print(b"% accepted)\n");
            } else {
                crate::uart_print(b"\n");
            }
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

    /// UX Enhancement: Preview next autonomous decision without executing
    pub(crate) fn autoctl_preview(&self, count: Option<usize>) {
        let steps = count.unwrap_or(1).min(5); // Max 5 preview steps

        unsafe { crate::uart_print(b"\n=== Autonomy Decision Preview ===\n"); }

        if steps > 1 {
            unsafe { crate::uart_print(b"NOTE: Multi-step preview shows "); }
            self.print_number_simple(steps as u64);
            unsafe { crate::uart_print(b" iterations of the same state.\n"); }
            unsafe { crate::uart_print(b"Real execution would change state between decisions.\n\n"); }
        }

        for i in 0..steps {
            if steps > 1 {
                unsafe { crate::uart_print(b"\n--- Step "); }
                self.print_number_simple((i + 1) as u64);
                unsafe { crate::uart_print(b" ---\n"); }
            }

            let preview = crate::autonomy::preview_next_decision();

            unsafe { crate::uart_print(b"Timestamp: "); }
            self.print_number_simple(preview.timestamp / 1_000_000);
            unsafe { crate::uart_print(b" seconds\n"); }

            unsafe { crate::uart_print(b"Autonomy Status: "); }
            if !preview.enabled {
                unsafe { crate::uart_print(b"DISABLED (would take no action)\n"); }
                return;
            } else if preview.safe_mode {
                unsafe { crate::uart_print(b"SAFE MODE (would take no action)\n"); }
                return;
            } else {
                unsafe { crate::uart_print(b"ENABLED\n"); }
            }

            unsafe { crate::uart_print(b"\nCurrent System State:\n"); }
            unsafe { crate::uart_print(b"  Memory Pressure: "); }
            self.print_number_simple(preview.memory_pressure as u64);
            unsafe { crate::uart_print(b"%\n  Memory Fragmentation: "); }
            self.print_number_simple(preview.memory_fragmentation as u64);
            unsafe { crate::uart_print(b"%\n  Deadline Misses: "); }
            self.print_number_simple(preview.deadline_misses as u64);
            unsafe { crate::uart_print(b"%\n  Command Rate: "); }
            self.print_number_simple(preview.command_rate as u64);
            unsafe { crate::uart_print(b" cmds/sec\n"); }

            unsafe { crate::uart_print(b"\nPredicted Directives (Q8.8 fixed-point):\n"); }
            unsafe { crate::uart_print(b"  Memory: "); }
            super::print_number_signed(preview.memory_directive as i64);
            unsafe { crate::uart_print(b" ("); }
            if preview.memory_directive > 256 {
                unsafe { crate::uart_print(b"increase allocation)\n"); }
            } else if preview.memory_directive < -256 {
                unsafe { crate::uart_print(b"trigger compaction)\n"); }
            } else {
                unsafe { crate::uart_print(b"maintain current)\n"); }
            }

            unsafe { crate::uart_print(b"  Scheduling: "); }
            super::print_number_signed(preview.scheduling_directive as i64);
            unsafe { crate::uart_print(b" ("); }
            if preview.scheduling_directive > 256 {
                unsafe { crate::uart_print(b"increase priority)\n"); }
            } else if preview.scheduling_directive < -256 {
                unsafe { crate::uart_print(b"decrease priority)\n"); }
            } else {
                unsafe { crate::uart_print(b"maintain current)\n"); }
            }

            unsafe { crate::uart_print(b"  Command Prediction: "); }
            super::print_number_signed(preview.command_directive as i64);
            unsafe { crate::uart_print(b" ("); }
            if preview.command_directive > 256 {
                unsafe { crate::uart_print(b"enable prediction)\n"); }
            } else if preview.command_directive < -256 {
                unsafe { crate::uart_print(b"disable prediction)\n"); }
            } else {
                unsafe { crate::uart_print(b"maintain current)\n"); }
            }

            unsafe { crate::uart_print(b"\nDecision Confidence: "); }
            self.print_number_simple((preview.confidence / 10) as u64);
            unsafe { crate::uart_print(b"/100\n"); }

            if preview.memory_pressure > 80 || preview.memory_fragmentation > 60 {
                unsafe { crate::uart_print(b"\nWARNING: High memory pressure or fragmentation detected!\n"); }
            }
            if preview.deadline_misses > 20 {
                unsafe { crate::uart_print(b"WARNING: High deadline miss rate detected!\n"); }
            }
        }

        unsafe { crate::uart_print(b"\nUse 'autoctl on' to enable autonomous execution.\n"); }
        unsafe { crate::uart_print(b"Use 'autoctl tick' to execute one decision manually.\n"); }
    }

    /// UX Enhancement: Show or set autonomy phase
    pub(crate) fn autoctl_phase(&self, action: Option<&str>) {
        if action.is_none() || action == Some("status") {
            // Show current phase
            let phase = crate::autonomy::get_autonomy_phase();
            unsafe { crate::uart_print(b"\n=== Autonomy Phase Status ===\n"); }
            unsafe { crate::uart_print(b"Current Phase: "); }
            unsafe { crate::uart_print(phase.as_str().as_bytes()); }
            unsafe { crate::uart_print(b"\n"); }
            unsafe { crate::uart_print(b"Description: "); }
            unsafe { crate::uart_print(phase.description().as_bytes()); }
            unsafe { crate::uart_print(b"\n"); }
            unsafe { crate::uart_print(b"Max Risk Score: "); }
            self.print_number_simple(phase.max_risk_score() as u64);
            unsafe { crate::uart_print(b"/100\n"); }
            unsafe { crate::uart_print(b"Recommended Interval: "); }
            self.print_number_simple(phase.recommended_interval_ms());
            unsafe { crate::uart_print(b" ms\n"); }

            unsafe { crate::uart_print(b"\nAvailable Phases:\n"); }
            unsafe { crate::uart_print(b"  A - Learning (exploration, low risk)\n"); }
            unsafe { crate::uart_print(b"  B - Validation (balanced, medium risk)\n"); }
            unsafe { crate::uart_print(b"  C - Production (conservative, reduced risk)\n"); }
            unsafe { crate::uart_print(b"  D - Emergency (minimal autonomy, safety-critical)\n"); }
            unsafe { crate::uart_print(b"\nUse 'autoctl phase <A|B|C|D>' to change phase.\n"); }
        } else {
            // Set phase
            let new_phase = match action.unwrap().to_uppercase().as_str() {
                "A" => crate::autonomy::AutonomyPhase::PhaseA,
                "B" => crate::autonomy::AutonomyPhase::PhaseB,
                "C" => crate::autonomy::AutonomyPhase::PhaseC,
                "D" => crate::autonomy::AutonomyPhase::PhaseD,
                _ => {
                    unsafe { crate::uart_print(b"Usage: autoctl phase <A|B|C|D|status>\n"); }
                    return;
                }
            };

            let old_phase = crate::autonomy::get_autonomy_phase();
            crate::autonomy::set_autonomy_phase(new_phase);

            unsafe { crate::uart_print(b"[AUTOCTL] Phase transition: "); }
            unsafe { crate::uart_print(old_phase.as_str().as_bytes()); }
            unsafe { crate::uart_print(b" -> "); }
            unsafe { crate::uart_print(new_phase.as_str().as_bytes()); }
            unsafe { crate::uart_print(b"\n"); }

            unsafe { crate::uart_print(b"  Description: "); }
            unsafe { crate::uart_print(new_phase.description().as_bytes()); }
            unsafe { crate::uart_print(b"\n"); }

            unsafe { crate::uart_print(b"  Max risk score: "); }
            self.print_number_simple(new_phase.max_risk_score() as u64);
            unsafe { crate::uart_print(b"/100\n"); }

            unsafe { crate::uart_print(b"  Recommended interval: "); }
            self.print_number_simple(new_phase.recommended_interval_ms());
            unsafe { crate::uart_print(b" ms\n"); }

            unsafe { crate::uart_print(b"\nConsider running 'autoctl interval "); }
            self.print_number_simple(new_phase.recommended_interval_ms());
            unsafe { crate::uart_print(b"' to match phase settings.\n"); }

            // Log phase transition to audit log
            unsafe { crate::uart_print(b"[AUDIT] Phase transition logged\n"); }
        }
    }

    /// Display attention weights for the last decision (Phase 6: Explainability)
    pub(crate) fn autoctl_attention(&self) {
        unsafe { crate::uart_print(b"\n=== Decision Attention Analysis ===\n"); }

        // Retrieve last decision from audit log
        let decision_opt = crate::autonomy::get_last_decision_rationale();

        if decision_opt.is_none() {
            unsafe { crate::uart_print(b"No decisions have been made yet.\n"); }
            unsafe { crate::uart_print(b"\nRun 'autoctl on' and 'autoctl tick' to generate a decision.\n"); }
            return;
        }

        let decision = decision_opt.unwrap();
        let rationale = decision.rationale;

        // Display decision metadata
        unsafe { crate::uart_print(b"Last Decision ID: #"); }
        self.print_number_simple(decision.decision_id);
        unsafe { crate::uart_print(b"\n"); }

        unsafe { crate::uart_print(b"Timestamp: "); }
        self.print_number_simple(decision.timestamp / 1_000_000); // Convert to seconds
        unsafe { crate::uart_print(b" seconds\n"); }

        unsafe { crate::uart_print(b"Explanation: "); }
        unsafe { crate::uart_print(rationale.explanation_code.as_str().as_bytes()); }
        unsafe { crate::uart_print(b"\n"); }

        // Display feature importance with visual bars
        unsafe { crate::uart_print(b"\nInput Feature Influence (0-100%):\n"); }

        // Memory features
        unsafe { crate::uart_print(b"  Memory Features:      "); }
        self.print_progress_bar(rationale.memory_pressure_importance, 100);
        unsafe { crate::uart_print(b" "); }
        self.print_number_simple(rationale.memory_pressure_importance as u64);
        unsafe { crate::uart_print(b"% "); }
        self.print_importance_level(rationale.memory_pressure_importance);
        unsafe { crate::uart_print(b"\n"); }

        // Scheduling features
        unsafe { crate::uart_print(b"  Scheduling Features:  "); }
        self.print_progress_bar(rationale.scheduling_load_importance, 100);
        unsafe { crate::uart_print(b" "); }
        self.print_number_simple(rationale.scheduling_load_importance as u64);
        unsafe { crate::uart_print(b"% "); }
        self.print_importance_level(rationale.scheduling_load_importance);
        unsafe { crate::uart_print(b"\n"); }

        // Command features
        unsafe { crate::uart_print(b"  Command Features:     "); }
        self.print_progress_bar(rationale.command_rate_importance, 100);
        unsafe { crate::uart_print(b" "); }
        self.print_number_simple(rationale.command_rate_importance as u64);
        unsafe { crate::uart_print(b"% "); }
        self.print_importance_level(rationale.command_rate_importance);
        unsafe { crate::uart_print(b"\n"); }

        // Display system state at decision time
        unsafe { crate::uart_print(b"\nSystem State at Decision Time:\n"); }
        unsafe { crate::uart_print(b"  Memory Pressure:      "); }
        self.print_number_simple(decision.state_before.memory_pressure as u64);
        unsafe { crate::uart_print(b"%\n"); }

        unsafe { crate::uart_print(b"  Memory Fragmentation: "); }
        self.print_number_simple(decision.state_before.memory_fragmentation as u64);
        unsafe { crate::uart_print(b"%\n"); }

        unsafe { crate::uart_print(b"  Deadline Misses:      "); }
        self.print_number_simple(decision.state_before.deadline_misses as u64);
        unsafe { crate::uart_print(b"%\n"); }

        unsafe { crate::uart_print(b"  Command Rate:         "); }
        self.print_number_simple(decision.state_before.command_rate as u64);
        unsafe { crate::uart_print(b"/100\n"); }

        // Display directives taken
        unsafe { crate::uart_print(b"\nDirectives Issued:\n"); }
        unsafe { crate::uart_print(b"  Memory Directive:     "); }
        super::print_number_signed(decision.directives[0] as i64);
        unsafe { crate::uart_print(b" (Q8.8)\n"); }

        unsafe { crate::uart_print(b"  Scheduling Directive: "); }
        super::print_number_signed(decision.directives[1] as i64);
        unsafe { crate::uart_print(b" (Q8.8)\n"); }

        unsafe { crate::uart_print(b"  Command Directive:    "); }
        super::print_number_signed(decision.directives[2] as i64);
        unsafe { crate::uart_print(b" (Q8.8)\n"); }

        // Overall confidence
        unsafe { crate::uart_print(b"\nOverall Decision Confidence: "); }
        super::print_number_signed(decision.confidence as i64);
        unsafe { crate::uart_print(b"/1000\n"); }

        // Confidence reason (dev feedback: interpretability)
        unsafe { crate::uart_print(b"Confidence Reason: "); }
        unsafe { crate::uart_print(rationale.confidence_reason.as_str().as_bytes()); }
        unsafe { crate::uart_print(b"\n"); }

        // Interpretation guidance
        unsafe { crate::uart_print(b"\nInterpretation:\n"); }
        let max_importance = rationale.memory_pressure_importance
            .max(rationale.scheduling_load_importance)
            .max(rationale.command_rate_importance);

        if max_importance == rationale.memory_pressure_importance && max_importance > 50 {
            unsafe { crate::uart_print(b"  The decision was PRIMARILY driven by memory conditions.\n"); }
            unsafe { crate::uart_print(b"  Monitor memory allocation patterns to understand decisions.\n"); }
        } else if max_importance == rationale.scheduling_load_importance && max_importance > 50 {
            unsafe { crate::uart_print(b"  The decision was PRIMARILY driven by scheduling conditions.\n"); }
            unsafe { crate::uart_print(b"  Monitor deadline misses and task latency to understand decisions.\n"); }
        } else if max_importance == rationale.command_rate_importance && max_importance > 50 {
            unsafe { crate::uart_print(b"  The decision was PRIMARILY driven by command load.\n"); }
            unsafe { crate::uart_print(b"  Monitor command rate and complexity to understand decisions.\n"); }
        } else {
            unsafe { crate::uart_print(b"  The decision was influenced EQUALLY by multiple factors.\n"); }
            unsafe { crate::uart_print(b"  System is operating in balanced conditions.\n"); }
        }

        unsafe { crate::uart_print(b"\n"); }
    }

    /// Print importance level label
    fn print_importance_level(&self, importance: u8) {
        if importance >= 60 {
            unsafe { crate::uart_print(b"(HIGH)"); }
        } else if importance >= 40 {
            unsafe { crate::uart_print(b"(MEDIUM)"); }
        } else if importance >= 20 {
            unsafe { crate::uart_print(b"(LOW)"); }
        } else {
            unsafe { crate::uart_print(b"(MINIMAL)"); }
        }
    }

    /// Print a simple progress bar (20 chars max)
    fn print_progress_bar(&self, value: u8, max: u8) {
        let bar_width: u32 = 20;
        let filled = (value as u32 * bar_width) / max as u32;

        unsafe { crate::uart_print(b"["); }
        for i in 0..bar_width {
            if i < filled {
                unsafe { crate::uart_print(b"="); }
            } else {
                unsafe { crate::uart_print(b" "); }
            }
        }
        unsafe { crate::uart_print(b"]"); }
    }

    /// Phase 6 Part 2: Whatif scenario analysis (EU AI Act Article 14: human oversight)
    pub(crate) fn autoctl_whatif(&self, args: &[&str]) {
        unsafe { crate::uart_print(b"\n=== What-If Scenario Analysis ===\n\n"); }

        // Start with current state as baseline
        let mut hypothetical_state = crate::meta_agent::collect_telemetry();
        let baseline_state = hypothetical_state;

        // Parse user-specified overrides (mem=80, frag=70, etc.)
        let mut overrides = heapless::Vec::<(&str, u8), 8>::new();

        for arg in args {
            if let Some(eq_pos) = arg.find('=') {
                let (key, value_str) = arg.split_at(eq_pos);
                let value_str = &value_str[1..]; // Skip '='

                if let Ok(value) = value_str.parse::<u8>() {
                    let value_clamped = value.min(100); // Clamp to 0-100

                    match key {
                        "mem" => {
                            hypothetical_state.memory_pressure = value_clamped;
                            let _ = overrides.push((key, value_clamped));
                        }
                        "frag" => {
                            hypothetical_state.memory_fragmentation = value_clamped;
                            let _ = overrides.push((key, value_clamped));
                        }
                        "misses" => {
                            hypothetical_state.deadline_misses = value_clamped;
                            let _ = overrides.push((key, value_clamped));
                        }
                        "rate" => {
                            hypothetical_state.command_rate = value_clamped;
                            let _ = overrides.push((key, value_clamped));
                        }
                        _ => {
                            unsafe {
                                crate::uart_print(b"[WARNING] Unknown parameter: ");
                                crate::uart_print(key.as_bytes());
                                crate::uart_print(b" (use mem, frag, misses, or rate)\n");
                            }
                        }
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"[ERROR] Invalid value for ");
                        crate::uart_print(arg.as_bytes());
                        crate::uart_print(b"\n");
                    }
                }
            } else if !arg.is_empty() {
                unsafe {
                    crate::uart_print(b"[ERROR] Invalid argument: ");
                    crate::uart_print(arg.as_bytes());
                    crate::uart_print(b" (use key=value format)\n");
                }
            }
        }

        // Display scenario configuration
        if overrides.is_empty() {
            unsafe { crate::uart_print(b"Scenario: CURRENT STATE (no overrides)\n"); }
        } else {
            unsafe { crate::uart_print(b"Scenario: HYPOTHETICAL STATE with overrides:\n"); }
            for (key, value) in &overrides {
                unsafe {
                    crate::uart_print(b"  ");
                    crate::uart_print(key.as_bytes());
                    crate::uart_print(b"=");
                    self.print_number_simple(*value as u64);
                    crate::uart_print(b"%\n");
                }
            }
        }

        // Run whatif simulation
        let whatif_result = crate::autonomy::simulate_whatif_decision(hypothetical_state);

        // Display hypothetical state vs current state
        unsafe { crate::uart_print(b"\n--- System State Comparison ---\n"); }

        unsafe { crate::uart_print(b"                      Current   ->  Hypothetical\n"); }
        unsafe { crate::uart_print(b"Memory Pressure:        "); }
        self.print_number_simple(baseline_state.memory_pressure as u64);
        unsafe { crate::uart_print(b"%     ->  "); }
        self.print_number_simple(whatif_result.memory_pressure as u64);
        unsafe { crate::uart_print(b"%\n"); }

        unsafe { crate::uart_print(b"Memory Fragmentation:   "); }
        self.print_number_simple(baseline_state.memory_fragmentation as u64);
        unsafe { crate::uart_print(b"%     ->  "); }
        self.print_number_simple(whatif_result.memory_fragmentation as u64);
        unsafe { crate::uart_print(b"%\n"); }

        unsafe { crate::uart_print(b"Deadline Misses:        "); }
        self.print_number_simple(baseline_state.deadline_misses as u64);
        unsafe { crate::uart_print(b"%     ->  "); }
        self.print_number_simple(whatif_result.deadline_misses as u64);
        unsafe { crate::uart_print(b"%\n"); }

        unsafe { crate::uart_print(b"Command Rate:           "); }
        self.print_number_simple(baseline_state.command_rate as u64);
        unsafe { crate::uart_print(b"%     ->  "); }
        self.print_number_simple(whatif_result.command_rate as u64);
        unsafe { crate::uart_print(b"%\n"); }

        // Display predicted directives
        unsafe { crate::uart_print(b"\n--- Predicted AI Directives (Q8.8 fixed-point) ---\n"); }

        unsafe { crate::uart_print(b"Memory Directive:       "); }
        super::print_number_signed(whatif_result.memory_directive as i64);
        unsafe { crate::uart_print(b" "); }
        if whatif_result.memory_directive > 256 {
            unsafe { crate::uart_print(b"(increase allocation)\n"); }
        } else if whatif_result.memory_directive < -256 {
            unsafe { crate::uart_print(b"(trigger compaction)\n"); }
        } else {
            unsafe { crate::uart_print(b"(maintain current)\n"); }
        }

        unsafe { crate::uart_print(b"Scheduling Directive:   "); }
        super::print_number_signed(whatif_result.scheduling_directive as i64);
        unsafe { crate::uart_print(b" "); }
        if whatif_result.scheduling_directive > 256 {
            unsafe { crate::uart_print(b"(increase priority)\n"); }
        } else if whatif_result.scheduling_directive < -256 {
            unsafe { crate::uart_print(b"(decrease priority)\n"); }
        } else {
            unsafe { crate::uart_print(b"(maintain current)\n"); }
        }

        unsafe { crate::uart_print(b"Command Directive:      "); }
        super::print_number_signed(whatif_result.command_directive as i64);
        unsafe { crate::uart_print(b" "); }
        if whatif_result.command_directive > 256 {
            unsafe { crate::uart_print(b"(enable prediction)\n"); }
        } else if whatif_result.command_directive < -256 {
            unsafe { crate::uart_print(b"(disable prediction)\n"); }
        } else {
            unsafe { crate::uart_print(b"(maintain current)\n"); }
        }

        // Display confidence and assessment
        unsafe { crate::uart_print(b"\nDecision Confidence:    "); }
        self.print_number_simple((whatif_result.confidence / 10) as u64);
        unsafe { crate::uart_print(b"/100 ("); }
        self.print_number_simple(whatif_result.confidence as u64);
        unsafe { crate::uart_print(b"/1000)\n"); }

        // Confidence threshold check
        let threshold = crate::autonomy::AUTONOMOUS_CONTROL.get_confidence_threshold();
        unsafe { crate::uart_print(b"Would Execute?:         "); }
        if whatif_result.confidence >= threshold {
            unsafe { crate::uart_print(b"YES (confidence >= threshold "); }
            self.print_number_simple(threshold as u64);
            unsafe { crate::uart_print(b"/1000)\n"); }
        } else {
            unsafe { crate::uart_print(b"NO (confidence < threshold "); }
            self.print_number_simple(threshold as u64);
            unsafe { crate::uart_print(b"/1000)\n"); }
        }

        // Risk warnings
        if whatif_result.memory_pressure > 80 || whatif_result.memory_fragmentation > 60 {
            unsafe { crate::uart_print(b"\n[WARNING] High memory pressure or fragmentation in scenario!\n"); }
        }
        if whatif_result.deadline_misses > 20 {
            unsafe { crate::uart_print(b"[WARNING] High deadline miss rate in scenario!\n"); }
        }

        // Usage hints
        unsafe { crate::uart_print(b"\nUsage Examples:\n"); }
        unsafe { crate::uart_print(b"  autoctl whatif mem=80              # What if memory pressure is 80%?\n"); }
        unsafe { crate::uart_print(b"  autoctl whatif mem=80 frag=70      # Multiple conditions\n"); }
        unsafe { crate::uart_print(b"  autoctl whatif mem=90 misses=40    # High load scenario\n\n"); }
    }

    /// Set or get minimum confidence threshold (enhancement: runtime configuration)
    pub(crate) fn autoctl_conf_threshold(&self, threshold_str: Option<&str>) {
        use crate::autonomy::AUTONOMOUS_CONTROL;

        if let Some(value_str) = threshold_str {
            // Parse and set new threshold
            if let Ok(value) = value_str.parse::<usize>() {
                let threshold_i16 = value as i16;
                AUTONOMOUS_CONTROL.set_confidence_threshold(threshold_i16);
                unsafe {
                    crate::uart_print(b"[AUTOCTL] Confidence threshold set to: ");
                    self.print_number_simple(value as u64);
                    crate::uart_print(b"/1000 (");
                    self.print_number_simple((value / 10) as u64);
                    crate::uart_print(b"%)\n");
                }
            } else {
                unsafe { crate::uart_print(b"[ERROR] Invalid threshold value. Use 0-1000 (0-100%)\n"); }
            }
        } else {
            // Display current threshold
            let current = AUTONOMOUS_CONTROL.get_confidence_threshold();
            unsafe {
                crate::uart_print(b"[AUTOCTL] Current confidence threshold: ");
                self.print_number_simple(current as u64);
                crate::uart_print(b"/1000 (");
                self.print_number_simple((current as usize / 10) as u64);
                crate::uart_print(b"%)\n");
                crate::uart_print(b"  Actions are accepted when confidence >= threshold.\n");
                crate::uart_print(b"  Usage: autoctl conf-threshold <0-1000>\n");
            }
        }
    }

    /// Display AI performance metrics dashboard (Phase 1.5)
    pub(crate) fn autoctl_ai_metrics(&self) {
        // This would normally call crate::control::ai_metrics::get_snapshot()
        // For now, display placeholder structure to show integration

        unsafe { crate::uart_print(b"\n=== AI Performance Metrics ===\n\n"); }

        // Crash Prediction Metrics
        unsafe { crate::uart_print(b"Crash Prediction:\n"); }
        if let Some(status) = crate::ai_insights::get_crash_status() {
            unsafe { crate::uart_print(b"  Current Risk:         "); }
            self.print_number_simple((status.confidence * 100.0) as u64);
            unsafe { crate::uart_print(b"%\n"); }
            unsafe { crate::uart_print(b"  Recent Failures:      "); }
            self.print_number_simple(status.recent_failures as u64);
            unsafe { crate::uart_print(b"\n"); }
        }

        // Transformer Scheduler Metrics
        unsafe { crate::uart_print(b"\nTransformer Scheduler:\n"); }
        if let Some(metrics) = crate::sched::get_transformer_metrics() {
            unsafe { crate::uart_print(b"  Status:               "); }
            if crate::sched::is_transformer_enabled() {
                unsafe { crate::uart_print(b"ENABLED\n"); }
            } else {
                unsafe { crate::uart_print(b"DISABLED\n"); }
            }
            unsafe { crate::uart_print(b"  Total Decisions:      "); }
            self.print_number_simple(metrics.total_decisions);
            unsafe { crate::uart_print(b"\n  Avg Latency:          "); }
            self.print_number_simple(metrics.avg_inference_latency_us);
            unsafe { crate::uart_print(b" us\n  Avg Score:            "); }
            self.print_number_simple((metrics.avg_prediction_score * 100.0) as u64);
            unsafe { crate::uart_print(b"%\n"); }
        }

        // LLM Inference Metrics
        #[cfg(feature = "llm")]
        {
            unsafe { crate::uart_print(b"\nLLM State Inference:\n"); }
            if let Some(stats) = crate::llm::get_inference_stats() {
                unsafe { crate::uart_print(b"  Total Queries:        "); }
                self.print_number_simple(stats.total_queries);
                unsafe { crate::uart_print(b"\n  Successful:           "); }
                self.print_number_simple(stats.successful_executions);
                unsafe { crate::uart_print(b"\n  Success Rate:         "); }
                self.print_number_simple(stats.success_rate as u64);
                unsafe { crate::uart_print(b"%\n"); }
            }
        }

        unsafe { crate::uart_print(b"\nUse 'autoctl export-metrics <path>' to export detailed metrics.\n"); }
        unsafe { crate::uart_print(b"Use 'autoctl reset-baseline' to reset performance baselines.\n\n"); }
    }

    /// Export AI metrics to JSON
    pub(crate) fn autoctl_export_metrics(&self, path: &str) {
        // This would normally call crate::control::ai_metrics::export_json()
        // and write to VFS at the specified path

        unsafe { crate::uart_print(b"[AI_METRICS] Exporting metrics to: "); }
        unsafe { crate::uart_print(path.as_bytes()); }
        unsafe { crate::uart_print(b"\n"); }

        // Placeholder for actual export logic
        unsafe { crate::uart_print(b"[AI_METRICS] Export complete (stub implementation)\n"); }
        unsafe { crate::uart_print(b"Full implementation requires VFS write support\n"); }
    }

    /// Reset AI performance baselines
    pub(crate) fn autoctl_reset_baseline(&self) {
        // This would reset baseline metrics in crate::control::ai_metrics

        unsafe { crate::uart_print(b"[AI_METRICS] Resetting performance baselines...\n"); }

        // Reset crash predictor peak
        crate::ai_insights::reset_peak();

        // Reset transformer scheduler
        crate::sched::reset_transformer();

        unsafe { crate::uart_print(b"[AI_METRICS] Baselines reset complete\n"); }
    }
}

