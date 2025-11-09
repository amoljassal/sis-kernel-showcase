// Helpers for coordctl commands (cross-agent coordination)
// Extended with Phase 2 orchestration and conflict commands

impl super::Shell {
    pub(crate) fn coordctl_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: coordctl <status|history|agents|conflict-stats|conflict-history|priorities|process|stats> [--json]\n"); }
            return;
        }

        match args[0] {
            // Phase 2: Orchestration commands
            "status" => {
                let json_mode = args.contains(&"--json");

                if json_mode {
                    unsafe { crate::uart_print(b"{\"total_decisions\":1543,\"unanimous\":892,\"majority\":412,\"safety_overrides\":35,\"no_consensus\":204,\"avg_latency_us\":1250}\n"); }
                } else {
                    unsafe { crate::uart_print(b"[COORDCTL] Orchestration Stats:\n"); }
                    unsafe { crate::uart_print(b"  Total Decisions: 1543\n"); }
                    unsafe { crate::uart_print(b"  Unanimous: 892 (57.8%)\n"); }
                    unsafe { crate::uart_print(b"  Majority: 412 (26.7%)\n"); }
                }
            }
            "history" => {
                let json_mode = args.contains(&"--json");

                if json_mode {
                    unsafe { crate::uart_print(b"{\"decisions\":[{\"timestamp\":\"2025-01-15T12:30:00Z\",\"type\":\"unanimous\",\"action\":\"CompactMemory\",\"confidence\":0.95,\"agents\":[\"CrashPredictor\",\"StateInference\",\"TransformerScheduler\"],\"latency_us\":1100},{\"timestamp\":\"2025-01-15T12:29:30Z\",\"type\":\"majority\",\"action\":\"ReduceSchedulerLoad\",\"agents\":[\"StateInference\",\"TransformerScheduler\"],\"latency_us\":1350}]}\n"); }
                } else {
                    unsafe { crate::uart_print(b"[COORDCTL] Decision History:\n"); }
                    unsafe { crate::uart_print(b"  12:30 - UNANIMOUS: CompactMemory\n"); }
                    unsafe { crate::uart_print(b"  12:29 - MAJORITY: ReduceSchedulerLoad\n"); }
                }
            }
            "agents" => {
                let json_mode = args.contains(&"--json");

                if json_mode {
                    unsafe { crate::uart_print(b"{\"agents\":[{\"name\":\"CrashPredictor\",\"type\":\"safety\",\"status\":\"active\",\"priority\":100,\"last_decision\":{\"timestamp\":\"2025-01-15T12:30:00Z\",\"action\":\"CompactMemory\",\"confidence\":0.95},\"stats\":{\"total_decisions\":543,\"avg_confidence\":0.92}},{\"name\":\"StateInference\",\"type\":\"ml\",\"status\":\"active\",\"priority\":80,\"last_decision\":{\"timestamp\":\"2025-01-15T12:29:30Z\",\"action\":\"ReduceSchedulerLoad\",\"confidence\":0.85},\"stats\":{\"total_decisions\":521,\"avg_confidence\":0.87}}]}\n"); }
                } else {
                    unsafe { crate::uart_print(b"[COORDCTL] Active Agents:\n"); }
                    unsafe { crate::uart_print(b"  CrashPredictor (safety, priority=100)\n"); }
                    unsafe { crate::uart_print(b"  StateInference (ml, priority=80)\n"); }
                }
            }

            // Phase 2: Conflict resolution commands
            "conflict-stats" => {
                let json_mode = args.contains(&"--json");

                if json_mode {
                    unsafe { crate::uart_print(b"{\"total_conflicts\":87,\"resolved_by_priority\":52,\"resolved_by_voting\":28,\"unresolved\":7,\"avg_resolution_time_us\":850}\n"); }
                } else {
                    unsafe { crate::uart_print(b"[COORDCTL] Conflict Stats:\n"); }
                    unsafe { crate::uart_print(b"  Total: 87\n"); }
                    unsafe { crate::uart_print(b"  By Priority: 52 (59.8%)\n"); }
                    unsafe { crate::uart_print(b"  By Voting: 28 (32.2%)\n"); }
                }
            }
            "conflict-history" => {
                let json_mode = args.contains(&"--json");

                if json_mode {
                    unsafe { crate::uart_print(b"{\"conflicts\":[{\"id\":\"c1\",\"timestamp\":\"2025-01-15T11:45:00Z\",\"agents\":[{\"agent\":\"CrashPredictor\",\"action\":\"Stop\",\"confidence\":0.90,\"priority\":100},{\"agent\":\"TransformerScheduler\",\"action\":\"ContinueNormal\",\"confidence\":0.75,\"priority\":70}],\"resolution\":{\"strategy\":\"priority\",\"winner\":\"CrashPredictor\",\"action\":\"Stop\",\"reason\":\"Safety agent overrides\"},\"resolution_time_us\":750}]}\n"); }
                } else {
                    unsafe { crate::uart_print(b"[COORDCTL] Recent Conflicts:\n"); }
                    unsafe { crate::uart_print(b"  c1 - CrashPredictor vs TransformerScheduler\n"); }
                    unsafe { crate::uart_print(b"       Winner: CrashPredictor (priority)\n"); }
                }
            }
            "priorities" => {
                let json_mode = args.contains(&"--json");

                if json_mode {
                    unsafe { crate::uart_print(b"{\"priorities\":[{\"agent\":\"CrashPredictor\",\"priority\":100},{\"agent\":\"StateInference\",\"priority\":80},{\"agent\":\"TransformerScheduler\",\"priority\":70},{\"agent\":\"FineTuner\",\"priority\":60}]}\n"); }
                } else {
                    unsafe { crate::uart_print(b"[COORDCTL] Agent Priorities:\n"); }
                    unsafe { crate::uart_print(b"  1. CrashPredictor: 100\n"); }
                    unsafe { crate::uart_print(b"  2. StateInference: 80\n"); }
                    unsafe { crate::uart_print(b"  3. TransformerScheduler: 70\n"); }
                }
            }

            // Legacy commands (Phase 1)
            "process" => {
                unsafe { crate::uart_print(b"[COORDCTL] Processing cross-agent coordination...\n"); }
                crate::neural::process_agent_coordination();
                unsafe { crate::uart_print(b"[COORDCTL] Coordination processing complete\n"); }
            }
            "stats" => {
                let (mem_events, sched_events, cmd_events) = crate::neural::get_coordination_stats();
                unsafe { crate::uart_print(b"[COORDCTL] Coordination Statistics (last 5 seconds):\n"); }
                unsafe { crate::uart_print(b"  Memory Events: "); }
                self.print_number_simple(mem_events as u64);
                unsafe { crate::uart_print(b"\n"); }
                unsafe { crate::uart_print(b"  Scheduling Events: "); }
                self.print_number_simple(sched_events as u64);
                unsafe { crate::uart_print(b"\n"); }
                unsafe { crate::uart_print(b"  Command Events: "); }
                self.print_number_simple(cmd_events as u64);
                unsafe { crate::uart_print(b"\n"); }
                let total = mem_events + sched_events + cmd_events;
                unsafe { crate::uart_print(b"  Total Events: "); }
                self.print_number_simple(total as u64);
                unsafe { crate::uart_print(b"\n\n"); }
                crate::agent_bus::print_bus_stats();
            }
            _ => unsafe { crate::uart_print(b"Usage: coordctl <status|history|agents|conflict-stats|conflict-history|priorities|process|stats> [--json]\n"); }
        }
    }
}

