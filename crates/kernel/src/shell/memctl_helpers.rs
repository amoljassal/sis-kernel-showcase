// Helpers for memctl commands (status/predict)

impl super::Shell {
    pub(crate) fn memctl_status(&self) {
        // Show memory agent status with telemetry and predictions
        crate::neural::print_memory_agent_status();
    }

    pub(crate) fn memctl_predict(&self, mode: Option<&str>) {
        if let Some("compaction") = mode {
            let (should_compact, pred_frag, conf) = crate::predictive_memory::evaluate_compaction_policy();
            unsafe { crate::uart_print(b"[PRED_MEM] Compaction Decision Preview:\n"); }
            unsafe { crate::uart_print(b"  Predicted fragmentation (5s ahead): "); }
            self.print_number_simple(pred_frag as u64);
            unsafe { crate::uart_print(b"%\n  Confidence: "); }
            self.print_number_simple(conf as u64);
            unsafe { crate::uart_print(b"/1000\n  Decision: "); }
            if should_compact { unsafe { crate::uart_print(b"COMPACT (threshold exceeded)\n"); } }
            else { unsafe { crate::uart_print(b"SKIP (below threshold)\n"); } }
        } else {
            let (conf, oom_risk, compact_needed) = crate::neural::predict_memory_health();
            unsafe { crate::uart_print(b"[MEM] Prediction:\n"); }
            unsafe { crate::uart_print(b"  Confidence: "); }
            self.print_number_simple(conf as u64);
            unsafe { crate::uart_print(b"/1000\n"); }
            unsafe { crate::uart_print(b"  OOM Risk: "); }
            if oom_risk { unsafe { crate::uart_print(b"YES (Low memory predicted)\n"); } }
            else { unsafe { crate::uart_print(b"NO (Memory healthy)\n"); } }
            unsafe { crate::uart_print(b"  Compaction Needed: "); }
            if compact_needed { unsafe { crate::uart_print(b"YES (Fragmentation detected)\n"); } }
            else { unsafe { crate::uart_print(b"NO (Memory compact)\n"); } }
        }
    }

    pub(crate) fn memctl_query_mode(&self, state: &str) {
        use core::sync::atomic::Ordering;
        match state {
            "on" => {
                crate::predictive_memory::MEMORY_QUERY_MODE.store(true, Ordering::Release);
                unsafe { crate::uart_print(b"[MEMCTL] Query mode: ENABLED\n"); }
                unsafe { crate::uart_print(b"  Memory operations will be predicted but NOT executed.\n"); }
                unsafe { crate::uart_print(b"  Use 'memctl query-mode off' to resume normal operation.\n"); }
            }
            "off" => {
                crate::predictive_memory::MEMORY_QUERY_MODE.store(false, Ordering::Release);
                unsafe { crate::uart_print(b"[MEMCTL] Query mode: DISABLED\n"); }
                unsafe { crate::uart_print(b"  Memory operations will execute normally.\n"); }
            }
            "status" => {
                let enabled = crate::predictive_memory::MEMORY_QUERY_MODE.load(Ordering::Acquire);
                unsafe { crate::uart_print(b"[MEMCTL] Query mode: "); }
                unsafe { crate::uart_print(if enabled { b"ENABLED (dry-run)\n" } else { b"DISABLED (normal)\n" }); }
            }
            _ => {
                unsafe { crate::uart_print(b"Usage: memctl query-mode <on|off|status>\n"); }
            }
        }
    }

    pub(crate) fn memctl_approval(&self, state: &str) {
        use core::sync::atomic::Ordering;
        match state {
            "on" => {
                crate::predictive_memory::MEMORY_APPROVAL_MODE.store(true, Ordering::Release);
                unsafe { crate::uart_print(b"[MEMCTL] Approval mode: ENABLED\n"); }
                unsafe { crate::uart_print(b"  Note: Approval flag is set. Full approve/deny workflow (pending ops queue)\n"); }
                unsafe { crate::uart_print(b"        is planned for future enhancement.\n"); }
            }
            "off" => {
                crate::predictive_memory::MEMORY_APPROVAL_MODE.store(false, Ordering::Release);
                unsafe { crate::uart_print(b"[MEMCTL] Approval mode: DISABLED\n"); }
                unsafe { crate::uart_print(b"  Memory operations will execute automatically.\n"); }
            }
            "status" => {
                let enabled = crate::predictive_memory::MEMORY_APPROVAL_MODE.load(Ordering::Acquire);
                unsafe { crate::uart_print(b"[MEMCTL] Approval mode: "); }
                unsafe { crate::uart_print(if enabled { b"ENABLED (requires approval)\n" } else { b"DISABLED (automatic)\n" }); }
            }
            _ => {
                unsafe { crate::uart_print(b"Usage: memctl approval <on|off|status>\n"); }
            }
        }
    }
}

