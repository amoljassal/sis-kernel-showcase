// Helpers for coordctl commands (cross-agent coordination)

impl super::Shell {
    pub(crate) fn coordctl_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: coordctl <process|stats>\n"); }
            return;
        }
        match args[0] {
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
            _ => unsafe { crate::uart_print(b"Usage: coordctl <process|stats>\n"); }
        }
    }
}

