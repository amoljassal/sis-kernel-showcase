//! Shell commands for Agent Supervision Module
//!
//! Provides commands for inspecting and managing the Agent Supervision Module.

use crate::shell::Shell;
use crate::agent_sys::supervisor::{AGENT_SUPERVISOR, TELEMETRY, POLICY_CONTROLLER};
use crate::agent_sys::AgentId;

impl Shell {
    /// asmstatus command - Show ASM telemetry
    ///
    /// Usage: asmstatus
    pub fn cmd_asmstatus(&self) {
        unsafe {
            crate::uart_print(b"\n=== Agent Supervision Module Status ===\n\n");
        }

        let telemetry = TELEMETRY.lock();
        if let Some(ref telem) = *telemetry {
            let snapshot = telem.snapshot();

            // System metrics
            unsafe {
                crate::uart_print(b"System Metrics:\n");
                crate::uart_print(b"  Total Spawns:    ");
                self.print_number_simple(snapshot.system_metrics.total_spawns);
                crate::uart_print(b"\n  Total Exits:     ");
                self.print_number_simple(snapshot.system_metrics.total_exits);
                crate::uart_print(b"\n  Total Faults:    ");
                self.print_number_simple(snapshot.system_metrics.total_faults);
                crate::uart_print(b"\n  Total Restarts:  ");
                self.print_number_simple(snapshot.system_metrics.total_restarts);
                crate::uart_print(b"\n  Active Agents:   ");
                self.print_number_simple(snapshot.system_metrics.active_agents as u64);
                crate::uart_print(b"\n  Total Syscalls:  ");
                self.print_number_simple(snapshot.total_syscalls);
                crate::uart_print(b"\n\n");
            }

            // Per-agent metrics
            if !snapshot.agent_metrics.is_empty() {
                unsafe {
                    crate::uart_print(b"Per-Agent Metrics:\n");
                    crate::uart_print(b"  ID    Spawns Exits  Faults\n");
                    crate::uart_print(b"  ----  ------ -----  ------\n");
                }

                for (agent_id, metrics) in snapshot.agent_metrics.iter() {
                    unsafe {
                        crate::uart_print(b"  ");
                        self.print_number_padded(*agent_id as u64, 4);
                        crate::uart_print(b"  ");
                        self.print_number_padded(metrics.spawn_count, 6);
                        crate::uart_print(b" ");
                        self.print_number_padded(metrics.exit_count, 5);
                        crate::uart_print(b"  ");
                        self.print_number_padded(metrics.fault_count, 6);
                        crate::uart_print(b"\n");
                    }
                }
            } else {
                unsafe {
                    crate::uart_print(b"No active agents\n");
                }
            }

            unsafe {
                crate::uart_print(b"\n");
            }
        } else {
            unsafe {
                crate::uart_print(b"Agent Supervision Module not initialized\n");
            }
        }
    }

    /// asmlist command - List all active agents
    ///
    /// Usage: asmlist
    pub fn cmd_asmlist(&self) {
        unsafe {
            crate::uart_print(b"\n=== Active Agents ===\n\n");
        }

        let supervisor = AGENT_SUPERVISOR.lock();
        if let Some(ref sup) = *supervisor {
            let agents = sup.list_agents();

            if agents.is_empty() {
                unsafe {
                    crate::uart_print(b"No active agents\n");
                }
            } else {
                unsafe {
                    crate::uart_print(b"ID    PID   Name                 Restart Count\n");
                    crate::uart_print(b"----  ----  ------------------   -------------\n");
                }

                for metadata in agents {
                    unsafe {
                        // Print ID
                        self.print_number_padded(metadata.agent_id as u64, 4);
                        crate::uart_print(b"  ");

                        // Print PID
                        self.print_number_padded(metadata.pid as u64, 4);
                        crate::uart_print(b"  ");

                        // Print name (truncate to 18 chars)
                        let name_bytes = metadata.name.as_bytes();
                        let len = name_bytes.len().min(18);
                        crate::uart_print(&name_bytes[..len]);
                        for _ in 0..(20 - len) {
                            crate::uart_print(b" ");
                        }

                        // Print restart info
                        self.print_number_simple(metadata.restart_count as u64);
                        crate::uart_print(b"/");
                        self.print_number_simple(metadata.max_restarts as u64);

                        crate::uart_print(b"\n");
                    }
                }
            }

            unsafe {
                crate::uart_print(b"\n");
            }
        } else {
            unsafe {
                crate::uart_print(b"Agent Supervision Module not initialized\n");
            }
        }
    }

    /// asminfo command - Show detailed info about an agent
    ///
    /// Usage: asminfo <agent_id>
    pub fn cmd_asminfo(&self, args: &[&str]) {
        if args.len() < 2 {
            unsafe {
                crate::uart_print(b"Usage: asminfo <agent_id>\n");
            }
            return;
        }

        // Parse agent ID
        let agent_id: AgentId = match self.parse_number(args[1]) {
            Some(id) => id as AgentId,
            None => {
                unsafe {
                    crate::uart_print(b"Invalid agent ID\n");
                }
                return;
            }
        };

        unsafe {
            crate::uart_print(b"\n=== Agent Information ===\n\n");
        }

        let supervisor = AGENT_SUPERVISOR.lock();
        if let Some(ref sup) = *supervisor {
            if let Some(metadata) = sup.get_agent(agent_id) {
                unsafe {
                    crate::uart_print(b"Agent ID:        ");
                    self.print_number_simple(metadata.agent_id as u64);
                    crate::uart_print(b"\nPID:             ");
                    self.print_number_simple(metadata.pid as u64);
                    crate::uart_print(b"\nName:            ");
                    crate::uart_print(metadata.name.as_bytes());
                    crate::uart_print(b"\nActive:          ");
                    crate::uart_print(if metadata.active { b"Yes" } else { b"No" });
                    crate::uart_print(b"\nAuto-restart:    ");
                    crate::uart_print(if metadata.auto_restart { b"Enabled" } else { b"Disabled" });
                    crate::uart_print(b"\nRestart count:   ");
                    self.print_number_simple(metadata.restart_count as u64);
                    crate::uart_print(b"/");
                    self.print_number_simple(metadata.max_restarts as u64);
                    crate::uart_print(b"\nUptime:          ");
                    self.print_number_simple(metadata.uptime());
                    crate::uart_print(b" us\nCapabilities:    ");
                    self.print_number_simple(metadata.capabilities.len() as u64);
                    crate::uart_print(b" total\n\n");
                }
            } else {
                unsafe {
                    crate::uart_print(b"Agent not found\n");
                }
            }
        } else {
            unsafe {
                crate::uart_print(b"Agent Supervision Module not initialized\n");
            }
        }
    }

    /// asmpolicy command - Show agent policy
    ///
    /// Usage: asmpolicy <agent_id>
    pub fn cmd_asmpolicy(&self, args: &[&str]) {
        if args.len() < 2 {
            unsafe {
                crate::uart_print(b"Usage: asmpolicy <agent_id>\n");
            }
            return;
        }

        let agent_id: AgentId = match self.parse_number(args[1]) {
            Some(id) => id as AgentId,
            None => {
                unsafe {
                    crate::uart_print(b"Invalid agent ID\n");
                }
                return;
            }
        };

        unsafe {
            crate::uart_print(b"\n=== Agent Policy ===\n\n");
        }

        let controller = POLICY_CONTROLLER.lock();
        if let Some(ref ctrl) = *controller {
            if let Some(policy) = ctrl.get_policy(agent_id) {
                unsafe {
                    crate::uart_print(b"Agent ID:        ");
                    self.print_number_simple(agent_id as u64);
                    crate::uart_print(b"\nCapabilities:    ");
                    self.print_number_simple(policy.capabilities.len() as u64);
                    crate::uart_print(b"\nAuto-restart:    ");
                    crate::uart_print(if policy.auto_restart { b"Enabled" } else { b"Disabled" });
                    crate::uart_print(b"\nMax restarts:    ");
                    self.print_number_simple(policy.max_restarts as u64);
                    crate::uart_print(b"\nViolations:      ");
                    self.print_number_simple(policy.violations.len() as u64);
                    crate::uart_print(b"\nPolicy changes:  ");
                    self.print_number_simple(policy.audit_trail.len() as u64);
                    crate::uart_print(b"\n\n");
                }
            } else {
                unsafe {
                    crate::uart_print(b"No policy found for agent ");
                    self.print_number_simple(agent_id as u64);
                    crate::uart_print(b"\n");
                }
            }
        } else {
            unsafe {
                crate::uart_print(b"Agent Supervision Module not initialized\n");
            }
        }
    }

    /// Helper: Print number with padding
    fn print_number_padded(&self, num: u64, width: usize) {
        let mut buf = [b' '; 20];
        let mut n = num;
        let mut i = 19;

        if n == 0 {
            buf[19] = b'0';
            i = 18;
        } else {
            while n > 0 && i > 0 {
                buf[i] = b'0' + (n % 10) as u8;
                n /= 10;
                i -= 1;
            }
            i += 1;
        }

        let start = if (20 - i) < width { 20 - width } else { i };
        unsafe {
            crate::uart_print(&buf[start..20]);
        }
    }

    /// Helper: Parse number from string
    fn parse_number(&self, s: &str) -> Option<u64> {
        let mut result: u64 = 0;
        for ch in s.chars() {
            if !ch.is_ascii_digit() {
                return None;
            }
            result = result.checked_mul(10)?;
            result = result.checked_add((ch as u64) - ('0' as u64))?;
        }
        Some(result)
    }
}
