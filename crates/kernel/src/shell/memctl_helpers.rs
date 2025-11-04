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

    /// List pending operations (approval workflow)
    pub(crate) fn memctl_approvals(&self) {
        let pending = crate::predictive_memory::PENDING_OPERATIONS.lock();

        unsafe {
            crate::uart_print(b"\n=== Pending Memory Operations ===\n");
            crate::uart_print(b"  Total: ");
            self.print_number_simple(pending.len() as u64);
            crate::uart_print(b"\n\n");
        }

        if pending.len() == 0 {
            unsafe { crate::uart_print(b"  No pending operations.\n\n"); }
            return;
        }

        unsafe {
            crate::uart_print(b"ID   | Type            | Confidence | Risk | Reason\n");
            crate::uart_print(b"-----|-----------------|------------|------|--------------------------------------------------\n");
        }

        for i in 0..pending.len() {
            if let Some(op) = pending.get(i) {
                unsafe {
                    // ID (padded to 4 chars)
                    self.print_number_simple(op.id as u64);
                    let id_len = if op.id < 10 { 1 } else if op.id < 100 { 2 } else { 3 };
                    for _ in 0..(4 - id_len) { crate::uart_print(b" "); }
                    crate::uart_print(b" | ");

                    // Type (padded to 15 chars)
                    let type_str = op.operation_type.as_str();
                    crate::uart_print(type_str.as_bytes());
                    for _ in 0..(15 - type_str.len()) { crate::uart_print(b" "); }
                    crate::uart_print(b" | ");

                    // Confidence (padded to 10 chars)
                    self.print_number_simple(op.confidence as u64);
                    crate::uart_print(b"/1000  | ");

                    // Risk (padded to 4 chars)
                    self.print_number_simple(op.risk_score as u64);
                    let risk_len = if op.risk_score < 10 { 1 } else { 2 };
                    for _ in 0..(4 - risk_len) { crate::uart_print(b" "); }
                    crate::uart_print(b" | ");

                    // Reason (truncated to 50 chars)
                    let reason_bytes = op.reason.as_bytes();
                    let max_len = core::cmp::min(50, reason_bytes.len());
                    crate::uart_print(&reason_bytes[..max_len]);
                    if reason_bytes.len() > 50 {
                        crate::uart_print(b"...");
                    }
                    crate::uart_print(b"\n");
                }
            }
        }

        unsafe { crate::uart_print(b"\n"); }
    }

    /// Approve pending operations
    pub(crate) fn memctl_approve(&self, count_str: Option<&str>) {
        let mut pending = crate::predictive_memory::PENDING_OPERATIONS.lock();

        if pending.len() == 0 {
            unsafe { crate::uart_print(b"[MEMCTL] No pending operations to approve.\n"); }
            return;
        }

        let operations = if let Some(n_str) = count_str {
            if let Ok(n) = n_str.parse::<usize>() {
                pending.approve_n(n)
            } else {
                unsafe { crate::uart_print(b"[ERROR] Invalid count. Usage: memctl approve [N]\n"); }
                return;
            }
        } else {
            pending.approve_all()
        };

        drop(pending);  // Release lock before executing

        let count = operations.len();
        unsafe {
            crate::uart_print(b"[MEMCTL] Approving ");
            self.print_number_simple(count as u64);
            crate::uart_print(b" operation(s)...\n\n");
        }

        let (executed, _failed) = crate::predictive_memory::execute_approved_operations(operations);

        unsafe {
            crate::uart_print(b"\n[MEMCTL] Completed: ");
            self.print_number_simple(executed as u64);
            crate::uart_print(b" operation(s) executed successfully.\n");
        }
    }

    /// Reject pending operations
    pub(crate) fn memctl_reject(&self, target: &str) {
        let mut pending = crate::predictive_memory::PENDING_OPERATIONS.lock();

        if target == "all" {
            let count = pending.len();
            pending.reject_all();
            unsafe {
                crate::uart_print(b"[MEMCTL] Rejected ");
                self.print_number_simple(count as u64);
                crate::uart_print(b" pending operation(s).\n");
            }
        } else {
            if let Ok(id) = target.parse::<usize>() {
                if pending.reject_by_id(id) {
                    unsafe {
                        crate::uart_print(b"[MEMCTL] Rejected operation ");
                        self.print_number_simple(id as u64);
                        crate::uart_print(b".\n");
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"[ERROR] Operation ID ");
                        self.print_number_simple(id as u64);
                        crate::uart_print(b" not found.\n");
                    }
                }
            } else {
                unsafe { crate::uart_print(b"[ERROR] Invalid ID. Usage: memctl reject <ID|all>\n"); }
            }
        }
    }
}

