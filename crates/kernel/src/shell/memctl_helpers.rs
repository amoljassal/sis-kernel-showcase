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
}

