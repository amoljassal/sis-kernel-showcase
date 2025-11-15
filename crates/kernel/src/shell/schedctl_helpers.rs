// Helpers for schedctl commands (workload/priorities/affinity/shadow/feature)

use crate::shell::print_number_signed;

impl super::Shell {
    /// Show current workload classification, policy, and quantum
    pub(crate) fn schedctl_workload(&self) {
        let state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
        unsafe { crate::uart_print(b"[PRED_SCHED] Current Workload Class: "); }
        unsafe { crate::uart_print(state.current_workload.as_str().as_bytes()); }
        unsafe { crate::uart_print(b"\n"); }
        unsafe { crate::uart_print(b"  Scheduling Policy: "); }
        unsafe { crate::uart_print(state.current_policy.as_str().as_bytes()); }
        unsafe { crate::uart_print(b"\n"); }
        unsafe { crate::uart_print(b"  Quantum: "); }
        self.print_number_simple(state.current_quantum_us);
        unsafe { crate::uart_print(b" us\n"); }
        unsafe { crate::uart_print(b"  Total Classifications: "); }
        self.print_number_simple(state.total_classifications as u64);
        unsafe { crate::uart_print(b"\n"); }
    }

    /// Show operator priority adjustments and registered operators
    pub(crate) fn schedctl_priorities(&self) {
        let state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
        unsafe { crate::uart_print(b"[PRED_SCHED] Priority Adjustments:\n"); }
        unsafe { crate::uart_print(b"  Total: "); }
        self.print_number_simple(state.total_adjustments as u64);
        unsafe { crate::uart_print(b"\n  Misses Prevented: "); }
        self.print_number_simple(state.misses_prevented as u64);
        unsafe { crate::uart_print(b"\n  Unnecessary: "); }
        self.print_number_simple(state.unnecessary_adjustments as u64);
        unsafe { crate::uart_print(b"\n"); }

        // Show registered operators
        unsafe { crate::uart_print(b"\nRegistered Operators: "); }
        self.print_number_simple(state.operators.len() as u64);
        unsafe { crate::uart_print(b"\n"); }
        for (i, op) in state.operators.iter().enumerate().take(10) {
            unsafe { crate::uart_print(b"  ["); }
            self.print_number_simple(i as u64);
            unsafe { crate::uart_print(b"] ID="); }
            self.print_number_simple(op.id as u64);
            unsafe { crate::uart_print(b" Priority="); }
            print_number_signed(op.priority as i64);
            unsafe { crate::uart_print(b" Misses="); }
            self.print_number_simple(op.miss_count as u64);
            unsafe { crate::uart_print(b"\n"); }
        }
    }

    /// Show learned operator affinities (>70%)
    pub(crate) fn schedctl_affinity(&self) {
        let state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
        let groups = state.get_affinity_groups();
        unsafe { crate::uart_print(b"[PRED_SCHED] Operator Affinity Groups (>70%):\n"); }
        if groups.is_empty() {
            unsafe { crate::uart_print(b"  No high-affinity groups found\n"); }
        } else {
            for (a, b, score) in groups {
                unsafe { crate::uart_print(b"  OP "); }
                self.print_number_simple(a as u64);
                unsafe { crate::uart_print(b" <-> OP "); }
                self.print_number_simple(b as u64);
                unsafe { crate::uart_print(b" (affinity: "); }
                self.print_number_simple(score as u64);
                unsafe { crate::uart_print(b"/1000)\n"); }
            }
        }
    }

    /// Shadow mode control (on/off/compare)
    pub(crate) fn schedctl_shadow(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: schedctl shadow <on|off|compare> [version]\n"); }
            return;
        }
        match args[0] {
            "on" => {
                let version = if args.len() > 1 {
                    args[1].parse::<u32>().unwrap_or(1)
                } else {
                    1
                };
                let mut state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
                state.enable_shadow_mode(version);
                unsafe { crate::uart_print(b"[PRED_SCHED] Shadow mode ENABLED (version "); }
                self.print_number_simple(version as u64);
                unsafe { crate::uart_print(b")\n"); }
            }
            "off" => {
                let mut state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
                state.disable_shadow_mode();
                unsafe { crate::uart_print(b"[PRED_SCHED] Shadow mode DISABLED\n"); }
            }
            "compare" => {
                let state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
                unsafe { crate::uart_print(b"[PRED_SCHED] Shadow Mode Comparison:\n"); }
                unsafe { crate::uart_print(b"  Enabled: "); }
                unsafe { crate::uart_print(if state.shadow_config.enabled { b"YES\n" } else { b"NO\n" }); }
                unsafe { crate::uart_print(b"  Comparisons: "); }
                self.print_number_simple(state.shadow_config.comparison_count as u64);
                unsafe { crate::uart_print(b"\n  Disagreements: "); }
                self.print_number_simple(state.shadow_config.disagreement_count as u64);
                unsafe { crate::uart_print(b"\n  Primary Better: "); }
                self.print_number_simple(state.shadow_config.primary_better_count as u64);
                unsafe { crate::uart_print(b"\n  Shadow Better: "); }
                self.print_number_simple(state.shadow_config.shadow_better_count as u64);
                unsafe { crate::uart_print(b"\n"); }
            }
            _ => unsafe { crate::uart_print(b"Usage: schedctl shadow <on|off|compare> [version]\n"); }
        }
    }

    /// Feature flag control (enable/disable/list)
    pub(crate) fn schedctl_feature(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: schedctl feature <enable|disable|list> [NAME]\n"); }
            return;
        }
        match args[0] {
            "enable" => {
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: schedctl feature enable <autonomous-scheduling|workload-classification|affinity-learning|shadow-mode>\n"); }
                    return;
                }
                let mut state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
                match args[1] {
                    "autonomous-scheduling" => {
                        state.features.autonomous_scheduling = true;
                        unsafe { crate::uart_print(b"[PRED_SCHED] Autonomous scheduling ENABLED\n"); }
                    }
                    "workload-classification" => {
                        state.features.workload_classification = true;
                        unsafe { crate::uart_print(b"[PRED_SCHED] Workload classification ENABLED\n"); }
                    }
                    "affinity-learning" => {
                        state.features.affinity_learning = true;
                        unsafe { crate::uart_print(b"[PRED_SCHED] Affinity learning ENABLED\n"); }
                    }
                    "shadow-mode" => {
                        state.features.shadow_mode = true;
                        unsafe { crate::uart_print(b"[PRED_SCHED] Shadow mode ENABLED\n"); }
                    }
                    _ => unsafe { crate::uart_print(b"Unknown feature\n"); }
                }
            }
            "disable" => {
                if args.len() < 2 {
                    unsafe { crate::uart_print(b"Usage: schedctl feature disable <autonomous-scheduling|workload-classification|affinity-learning|shadow-mode>\n"); }
                    return;
                }
                let mut state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
                match args[1] {
                    "autonomous-scheduling" => {
                        state.features.autonomous_scheduling = false;
                        unsafe { crate::uart_print(b"[PRED_SCHED] Autonomous scheduling DISABLED\n"); }
                    }
                    "workload-classification" => {
                        state.features.workload_classification = false;
                        unsafe { crate::uart_print(b"[PRED_SCHED] Workload classification DISABLED\n"); }
                    }
                    "affinity-learning" => {
                        state.features.affinity_learning = false;
                        unsafe { crate::uart_print(b"[PRED_SCHED] Affinity learning DISABLED\n"); }
                    }
                    "shadow-mode" => {
                        state.features.shadow_mode = false;
                        unsafe { crate::uart_print(b"[PRED_SCHED] Shadow mode DISABLED\n"); }
                    }
                    _ => unsafe { crate::uart_print(b"Unknown feature\n"); }
                }
            }
            "list" => {
                let state = crate::predictive_scheduling::PREDICTIVE_SCHEDULING.lock();
                unsafe { crate::uart_print(b"[PRED_SCHED] Feature Flags:\n"); }
                unsafe { crate::uart_print(b"  autonomous-scheduling: "); }
                unsafe { crate::uart_print(if state.features.autonomous_scheduling { b"ENABLED\n" } else { b"DISABLED\n" }); }
                unsafe { crate::uart_print(b"  workload-classification: "); }
                unsafe { crate::uart_print(if state.features.workload_classification { b"ENABLED\n" } else { b"DISABLED\n" }); }
                unsafe { crate::uart_print(b"  affinity-learning: "); }
                unsafe { crate::uart_print(if state.features.affinity_learning { b"ENABLED\n" } else { b"DISABLED\n" }); }
                unsafe { crate::uart_print(b"  shadow-mode: "); }
                unsafe { crate::uart_print(if state.features.shadow_mode { b"ENABLED\n" } else { b"DISABLED\n" }); }
            }
            _ => unsafe { crate::uart_print(b"Usage: schedctl feature <enable|disable|list> [NAME]\n"); }
        }
    }

    /// Handle transformer scheduler commands
    pub(crate) fn schedctl_transformer(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: schedctl transformer <on|off|stats|reset>\n"); }
            return;
        }

        match args[0] {
            "on" => {
                crate::sched::set_transformer_enabled(true);
                unsafe { crate::uart_print(b"Transformer scheduler: ENABLED\n"); }
            }
            "off" => {
                crate::sched::set_transformer_enabled(false);
                unsafe { crate::uart_print(b"Transformer scheduler: DISABLED\n"); }
            }
            "stats" => {
                self.schedctl_transformer_stats();
            }
            "reset" => {
                crate::sched::reset_transformer();
                unsafe { crate::uart_print(b"Transformer scheduler: RESET\n"); }
            }
            _ => {
                unsafe { crate::uart_print(b"Unknown transformer command\n"); }
                unsafe { crate::uart_print(b"Usage: schedctl transformer <on|off|stats|reset>\n"); }
            }
        }
    }

    /// Show transformer scheduler statistics
    fn schedctl_transformer_stats(&self) {
        if let Some(metrics) = crate::sched::get_transformer_metrics() {
            unsafe { crate::uart_print(b"=== Transformer Scheduler Statistics ===\n"); }

            unsafe { crate::uart_print(b"Status: "); }
            if crate::sched::is_transformer_enabled() {
                unsafe { crate::uart_print(b"ENABLED\n"); }
            } else {
                unsafe { crate::uart_print(b"DISABLED\n"); }
            }

            unsafe { crate::uart_print(b"Total Decisions: "); }
            self.print_number_simple(metrics.total_decisions);
            unsafe { crate::uart_print(b"\n"); }

            unsafe { crate::uart_print(b"Avg Prediction Score: "); }
            self.print_number_simple((metrics.avg_prediction_score * 100.0) as u64);
            unsafe { crate::uart_print(b"%\n"); }

            unsafe { crate::uart_print(b"Avg Inference Latency: "); }
            self.print_number_simple(metrics.avg_inference_latency_us);
            unsafe { crate::uart_print(b" us\n"); }

            unsafe { crate::uart_print(b"Context Switches Saved: "); }
            self.print_number_simple(metrics.context_switches_saved);
            unsafe { crate::uart_print(b"\n"); }
        } else {
            unsafe { crate::uart_print(b"Transformer scheduler not initialized\n"); }
        }
    }
}
