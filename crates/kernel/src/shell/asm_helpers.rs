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
        let agent_id: AgentId = match self.parse_number_u64(args[1]) {
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

        let agent_id: AgentId = match self.parse_number_u64(args[1]) {
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

    /// Helper: Parse number from string (u64 version for ASM commands)
    fn parse_number_u64(&self, s: &str) -> Option<u64> {
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

    /// gwstatus command - Show Cloud Gateway status
    ///
    /// Usage: gwstatus
    pub fn cmd_gwstatus(&self) {
        #[cfg(feature = "agentsys")]
        {
            use crate::agent_sys::cloud_gateway::CLOUD_GATEWAY;

            unsafe {
                crate::uart_print(b"\n=== Cloud Gateway Status ===\n\n");
            }

            let gateway_guard = CLOUD_GATEWAY.lock();
            if let Some(ref gateway) = *gateway_guard {
                let metrics = gateway.metrics();
                let health = gateway.all_backend_health();

                // Request statistics
                unsafe {
                    crate::uart_print(b"Request Statistics:\n");
                    crate::uart_print(b"  Total Requests:    ");
                    self.print_number_simple(metrics.total_requests);
                    crate::uart_print(b"\n  Successful:        ");
                    self.print_number_simple(metrics.successful_requests);
                    crate::uart_print(b"\n  Failed:            ");
                    self.print_number_simple(metrics.failed_requests);
                    crate::uart_print(b"\n  Rate Limited:      ");
                    self.print_number_simple(metrics.rate_limited_requests);
                    crate::uart_print(b"\n  Fallback Used:     ");
                    self.print_number_simple(metrics.fallback_requests);
                    crate::uart_print(b"\n\n");

                    // Provider statistics
                    crate::uart_print(b"Provider Statistics:\n");
                    crate::uart_print(b"  Provider    Success  Failures  Health\n");
                    crate::uart_print(b"  ----------  -------  --------  --------\n");
                }

                // Claude
                unsafe {
                    crate::uart_print(b"  Claude      ");
                }
                self.print_number_padded(metrics.claude_successes, 7);
                unsafe { crate::uart_print(b"  "); }
                self.print_number_padded(metrics.claude_failures, 8);
                unsafe { crate::uart_print(b"  "); }
                if let Some(h) = health.get(&crate::agent_sys::cloud_gateway::Provider::Claude) {
                    let pct = (h * 100.0) as u64;
                    self.print_number_simple(pct);
                    unsafe { crate::uart_print(b"%\n"); }
                }

                // GPT-4
                unsafe {
                    crate::uart_print(b"  GPT-4       ");
                }
                self.print_number_padded(metrics.gpt4_successes, 7);
                unsafe { crate::uart_print(b"  "); }
                self.print_number_padded(metrics.gpt4_failures, 8);
                unsafe { crate::uart_print(b"  "); }
                if let Some(h) = health.get(&crate::agent_sys::cloud_gateway::Provider::GPT4) {
                    let pct = (h * 100.0) as u64;
                    self.print_number_simple(pct);
                    unsafe { crate::uart_print(b"%\n"); }
                }

                // Gemini
                unsafe {
                    crate::uart_print(b"  Gemini      ");
                }
                self.print_number_padded(metrics.gemini_successes, 7);
                unsafe { crate::uart_print(b"  "); }
                self.print_number_padded(metrics.gemini_failures, 8);
                unsafe { crate::uart_print(b"  "); }
                if let Some(h) = health.get(&crate::agent_sys::cloud_gateway::Provider::Gemini) {
                    let pct = (h * 100.0) as u64;
                    self.print_number_simple(pct);
                    unsafe { crate::uart_print(b"%\n"); }
                }

                // Local
                unsafe {
                    crate::uart_print(b"  Local       ");
                }
                self.print_number_padded(metrics.local_successes, 7);
                unsafe { crate::uart_print(b"  "); }
                self.print_number_padded(metrics.local_failures, 8);
                unsafe { crate::uart_print(b"  "); }
                if let Some(h) = health.get(&crate::agent_sys::cloud_gateway::Provider::LocalFallback) {
                    let pct = (h * 100.0) as u64;
                    self.print_number_simple(pct);
                    unsafe { crate::uart_print(b"%\n"); }
                }

                // Performance
                unsafe {
                    crate::uart_print(b"\nPerformance:\n");
                    crate::uart_print(b"  Total Tokens:       ");
                    self.print_number_simple(metrics.total_tokens);
                    crate::uart_print(b"\n  Avg Response Time:  ");
                    self.print_number_simple(metrics.avg_response_time_us);
                    crate::uart_print(b" us\n");

                    crate::uart_print(b"\nActive Agents: ");
                    self.print_number_simple(gateway.active_agents() as u64);
                    crate::uart_print(b"\n");
                }
            } else {
                unsafe {
                    crate::uart_print(b"Cloud Gateway not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Cloud Gateway not available (feature not enabled)\n");
            }
        }
    }

    /// Show compliance report (ASM-specific version)
    pub fn cmd_asm_compliance(&self) {
        #[cfg(feature = "agentsys")]
        {
            use crate::agent_sys::supervisor::hooks::get_compliance_report;

            unsafe {
                crate::uart_print(b"\nEU AI Act Compliance Report\n");
                crate::uart_print(b"===========================\n\n");
            }

            if let Some(report) = get_compliance_report() {
                // Calculate system compliance score from agent records
                let system_compliance_score = if report.agent_records.is_empty() {
                    1.0
                } else {
                    let total_score: f32 = report.agent_records.iter()
                        .map(|r| r.compliance_score())
                        .sum();
                    total_score / report.agent_records.len() as f32
                };

                // Calculate risk level distribution
                use crate::agent_sys::supervisor::compliance::RiskLevel;
                let minimal_risk = report.agent_records.iter().filter(|r| r.risk_level == RiskLevel::Minimal).count();
                let limited_risk = report.agent_records.iter().filter(|r| r.risk_level == RiskLevel::Limited).count();
                let high_risk = report.agent_records.iter().filter(|r| r.risk_level == RiskLevel::High).count();
                let unacceptable_risk = report.agent_records.iter().filter(|r| r.risk_level == RiskLevel::Unacceptable).count();

                // System-wide compliance
                unsafe {
                    crate::uart_print(b"Timestamp:          ");
                    self.print_number_simple(report.generated_at);
                    crate::uart_print(b"\nTotal Agents:       ");
                    self.print_number_simple(report.total_agents as u64);
                    crate::uart_print(b"\nTotal Events:       ");
                    self.print_number_simple(report.total_events as u64);
                    crate::uart_print(b"\nPolicy Violations:  ");
                    self.print_number_simple(report.total_violations as u64);
                    crate::uart_print(b"\nSystem Compliance:  ");
                }

                let compliance_pct = (system_compliance_score * 100.0) as u64;
                self.print_number_simple(compliance_pct);
                unsafe { crate::uart_print(b"%\n\n"); }

                // Risk level distribution
                unsafe {
                    crate::uart_print(b"Risk Level Distribution:\n");
                    crate::uart_print(b"  Minimal:          ");
                    self.print_number_simple(minimal_risk as u64);
                    crate::uart_print(b"\n  Limited:          ");
                    self.print_number_simple(limited_risk as u64);
                    crate::uart_print(b"\n  High:             ");
                    self.print_number_simple(high_risk as u64);
                    crate::uart_print(b"\n  Unacceptable:     ");
                    self.print_number_simple(unacceptable_risk as u64);
                    crate::uart_print(b"\n\n");
                }

                // Per-agent compliance
                unsafe {
                    crate::uart_print(b"Agent Compliance Details:\n");
                    crate::uart_print(b"-------------------------\n");
                }

                for agent_record in &report.agent_records {
                    unsafe {
                        crate::uart_print(b"\nAgent ID: ");
                        self.print_number_simple(agent_record.agent_id as u64);
                        crate::uart_print(b"\n  Risk Level:       ");
                        crate::uart_print(agent_record.risk_level.as_str().as_bytes());
                        crate::uart_print(b"\n  Events Logged:    ");
                        self.print_number_simple(agent_record.total_operations as u64);
                        crate::uart_print(b"\n  Violations:       ");
                        self.print_number_simple(agent_record.policy_violations as u64);
                        crate::uart_print(b"\n  Human Oversight:  ");
                        self.print_number_simple(agent_record.human_reviews as u64);
                        crate::uart_print(b"\n  Compliance Score: ");
                    }

                    let score_pct = (agent_record.compliance_score() * 100.0) as u64;
                    self.print_number_simple(score_pct);
                    unsafe {
                        crate::uart_print(b"%\n  Status:           ");

                        if agent_record.compliance_score() >= 0.9 {
                            crate::uart_print(b"COMPLIANT\n");
                        } else if agent_record.compliance_score() >= 0.7 {
                            crate::uart_print(b"REVIEW_NEEDED\n");
                        } else {
                            crate::uart_print(b"NON_COMPLIANT\n");
                        }
                    }
                }

                unsafe {
                    crate::uart_print(b"\nCompliance Requirements (EU AI Act):\n");
                    crate::uart_print(b"- Transparency: All operations logged\n");
                    crate::uart_print(b"- Risk Assessment: Agents classified by risk level\n");
                    crate::uart_print(b"- Human Oversight: Available via compliance events\n");
                    crate::uart_print(b"- Audit Trail: Complete event history maintained\n");
                    crate::uart_print(b"- Robustness: Fault detection and recovery active\n");
                }
            } else {
                unsafe {
                    crate::uart_print(b"Compliance tracking not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Compliance tracking not available (feature not enabled)\n");
            }
        }
    }
}
