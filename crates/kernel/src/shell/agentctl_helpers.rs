// Helpers for agentctl commands (agent message bus)

impl super::Shell {
    pub(crate) fn agentctl_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: agentctl <bus|stats|clear>\n"); }
            return;
        }
        match args[0] {
            "bus" => {
                let messages = crate::agent_bus::get_all_messages();
                unsafe { crate::uart_print(b"[AGENT BUS] Messages ("); }
                self.print_number_simple(messages.len() as u64);
                unsafe { crate::uart_print(b" total):\n"); }

                if messages.is_empty() {
                    unsafe { crate::uart_print(b"  (no messages)\n"); }
                } else {
                    for msg in messages.iter() { crate::agent_bus::print_message(msg); }
                }
            }
            "stats" => {
                crate::agent_bus::print_bus_stats();
            }
            "clear" => {
                crate::agent_bus::clear_message_bus();
                unsafe { crate::uart_print(b"[AGENT BUS] Cleared all messages\n"); }
            }
            _ => unsafe { crate::uart_print(b"Usage: agentctl <bus|stats|clear>\n"); }
        }
    }
}

