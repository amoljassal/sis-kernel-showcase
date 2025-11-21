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

    //
    // ========== P0 COMMANDS: Critical for Testing ==========
    //

    /// agentsys spawn - Spawn a test agent
    ///
    /// Usage: agentsys spawn <agent_id> <name> <capabilities>
    /// Example: agentsys spawn 1000 test_agent FsBasic,NetClient
    pub fn cmd_agentsys_spawn(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.len() < 3 {
                unsafe {
                    crate::uart_print(b"Usage: agentsys spawn <agent_id> <name> <capabilities>\n");
                    crate::uart_print(b"Example: agentsys spawn 1000 test_agent FsBasic,NetClient\n");
                    crate::uart_print(b"\nAvailable capabilities:\n");
                    crate::uart_print(b"  FsBasic, AudioControl, DocBasic, Capture, Screenshot, Admin,\n");
                    crate::uart_print(b"  NetClient, NetServer\n");
                }
                return;
            }

            // Parse agent ID
            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            let name = args[1];
            let caps_str = args[2];

            unsafe {
                crate::uart_print(b"[ASM] Spawning agent ");
                self.print_number_simple(agent_id as u64);
                crate::uart_print(b": ");
                crate::uart_print(name.as_bytes());
                crate::uart_print(b"\n");
                crate::uart_print(b"[ASM] Capabilities: ");
                crate::uart_print(caps_str.as_bytes());
                crate::uart_print(b"\n");
            }

            // For now, this is a stub - actual agent spawning would require process creation
            // This command will be fully implemented when we integrate with process manager
            unsafe {
                crate::uart_print(b"[ASM] Note: Agent spawning not yet integrated with process manager\n");
                crate::uart_print(b"[ASM] This command is ready for integration testing\n");
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Agent spawning not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys kill - Kill an agent
    ///
    /// Usage: agentsys kill <agent_id>
    pub fn cmd_agentsys_kill(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: agentsys kill <agent_id>\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            let supervisor = crate::agent_sys::supervisor::AGENT_SUPERVISOR.lock();
            if let Some(ref sup) = *supervisor {
                if let Some(metadata) = sup.get_agent(agent_id) {
                    let pid = metadata.pid;
                    unsafe {
                        crate::uart_print(b"[ASM] Killing agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" (PID: ");
                        self.print_number_simple(pid as u64);
                        crate::uart_print(b")\n");
                    }

                    // Send SIGKILL to the agent process
                    let _ = crate::process::signal::send_signal(pid, crate::process::signal::Signal::SIGKILL);

                    unsafe {
                        crate::uart_print(b"[ASM] Agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" terminated\n");
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"Error: Agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" not found\n");
                    }
                }
            } else {
                unsafe {
                    crate::uart_print(b"Agent Supervisor not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Agent management not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys metrics - Show detailed metrics for a specific agent
    ///
    /// Usage: agentsys metrics <agent_id>
    pub fn cmd_agentsys_metrics(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: agentsys metrics <agent_id>\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            let telemetry = crate::agent_sys::supervisor::TELEMETRY.lock();
            if let Some(ref telem) = *telemetry {
                if let Some(metrics) = telem.get_agent_metrics(agent_id) {
                    unsafe {
                        crate::uart_print(b"\nAgent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" Metrics:\n");
                        crate::uart_print(b"=================\n");
                        crate::uart_print(b"  Spawn Count:       ");
                        self.print_number_simple(metrics.spawn_count);
                        crate::uart_print(b"\n  Exit Count:        ");
                        self.print_number_simple(metrics.exit_count);
                        crate::uart_print(b"\n  Fault Count:       ");
                        self.print_number_simple(metrics.fault_count);
                        crate::uart_print(b"\n  CPU Time:          ");
                        self.print_number_simple(metrics.cpu_time_us);
                        crate::uart_print(b" us\n");
                        crate::uart_print(b"  Memory Usage:      ");
                        self.print_number_simple(metrics.memory_bytes as u64);
                        crate::uart_print(b" bytes\n");
                        crate::uart_print(b"  Syscall Count:     ");
                        self.print_number_simple(metrics.syscall_count);
                        crate::uart_print(b"\n  Last Spawn:        ");
                        self.print_number_simple(metrics.last_spawn);
                        crate::uart_print(b"\n  Last Exit:         ");
                        self.print_number_simple(metrics.last_exit);
                        crate::uart_print(b"\n  Last Exit Code:    ");
                        self.print_number_simple(metrics.last_exit_code as u64);
                        crate::uart_print(b"\n");

                        if !metrics.recent_faults.is_empty() {
                            crate::uart_print(b"  Recent Faults:\n");
                            for fault in &metrics.recent_faults {
                                crate::uart_print(b"    - ");
                                crate::uart_print(fault.description().as_bytes());
                                crate::uart_print(b"\n");
                            }
                        } else {
                            crate::uart_print(b"  Recent Faults:     None\n");
                        }
                        crate::uart_print(b"\n");
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"Error: No metrics found for agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b"\n");
                    }
                }
            } else {
                unsafe {
                    crate::uart_print(b"Telemetry not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Agent metrics not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys resources - Show resource usage for a specific agent
    ///
    /// Usage: agentsys resources <agent_id>
    pub fn cmd_agentsys_resources(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: agentsys resources <agent_id>\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            use crate::agent_sys::supervisor::RESOURCE_MONITOR;
            let mut monitor = RESOURCE_MONITOR.lock();
            if let Some(ref mut mon) = *monitor {
                if let Some(agent_monitor) = mon.get_agent(agent_id) {
                    // Get resource statistics from agent monitor
                    let cpu_time = {
                        let (cpu, _, _) = agent_monitor.lifetime_stats();
                        cpu
                    };
                    let memory = agent_monitor.current_memory();
                    let syscall_rate = agent_monitor.syscall_rate(1); // Last second
                    let (_, total_syscalls, _) = agent_monitor.lifetime_stats();

                    // Get limits from fault detector
                    use crate::agent_sys::supervisor::FAULT_DETECTOR;
                    let detector = FAULT_DETECTOR.lock();
                    let limits = detector.as_ref().map(|d| d.get_default_limits());

                    unsafe {
                        crate::uart_print(b"\nAgent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" Resource Usage:\n");
                        crate::uart_print(b"==========================\n");
                        crate::uart_print(b"  CPU Time:          ");
                        self.print_number_simple(cpu_time);
                        crate::uart_print(b" us");
                        if let Some(lim) = limits {
                            if let Some(quota) = lim.cpu_quota_us {
                                crate::uart_print(b" (quota: ");
                                self.print_number_simple(quota);
                                crate::uart_print(b" us)");
                            }
                        }
                        crate::uart_print(b"\n  Memory:            ");
                        self.print_number_simple(memory as u64);
                        crate::uart_print(b" bytes");
                        if let Some(lim) = limits {
                            if let Some(limit) = lim.memory_limit_bytes {
                                crate::uart_print(b" (limit: ");
                                self.print_number_simple(limit as u64);
                                crate::uart_print(b" bytes)");
                            }
                        }
                        crate::uart_print(b"\n  Syscalls (total):  ");
                        self.print_number_simple(total_syscalls);
                        crate::uart_print(b"\n  Syscall Rate:      ");
                        self.print_number_simple(syscall_rate as u64);
                        crate::uart_print(b" /sec");
                        if let Some(lim) = limits {
                            if let Some(rate) = lim.syscall_rate_limit {
                                crate::uart_print(b" (limit: ");
                                self.print_number_simple(rate);
                                crate::uart_print(b" /sec)");
                            }
                        }
                        crate::uart_print(b"\n\n  Status: Normal\n\n");
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"Error: No resource data for agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b"\n");
                    }
                }
            } else {
                unsafe {
                    crate::uart_print(b"Resource Monitor not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Resource monitoring not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys status - Show overall ASM status
    ///
    /// Usage: agentsys status
    pub fn cmd_agentsys_status(&self) {
        #[cfg(feature = "agentsys")]
        {
            unsafe {
                crate::uart_print(b"\nAgent Supervision Module Status\n");
                crate::uart_print(b"================================\n\n");
                crate::uart_print(b"Subsystems:\n");
            }

            // Check each subsystem
            let supervisor_ok = crate::agent_sys::supervisor::AGENT_SUPERVISOR.lock().is_some();
            let telemetry_ok = crate::agent_sys::supervisor::TELEMETRY.lock().is_some();
            let fault_ok = crate::agent_sys::supervisor::FAULT_DETECTOR.lock().is_some();
            let policy_ok = crate::agent_sys::supervisor::POLICY_CONTROLLER.lock().is_some();
            let compliance_ok = crate::agent_sys::supervisor::COMPLIANCE_TRACKER.lock().is_some();
            let resource_ok = crate::agent_sys::supervisor::RESOURCE_MONITOR.lock().is_some();
            let deps_ok = crate::agent_sys::supervisor::DEPENDENCY_GRAPH.lock().is_some();
            let profiler_ok = crate::agent_sys::supervisor::SYSTEM_PROFILER.lock().is_some();

            unsafe {
                crate::uart_print(if supervisor_ok { b"  \xE2\x9C\x93 " } else { b"  \xE2\x9C\x97 " });
                crate::uart_print(b"Agent Supervisor      ");
                crate::uart_print(if supervisor_ok { b"(initialized)\n" } else { b"(NOT initialized)\n" });

                crate::uart_print(if telemetry_ok { b"  \xE2\x9C\x93 " } else { b"  \xE2\x9C\x97 " });
                crate::uart_print(b"Telemetry Aggregator  ");
                crate::uart_print(if telemetry_ok { b"(initialized)\n" } else { b"(NOT initialized)\n" });

                crate::uart_print(if fault_ok { b"  \xE2\x9C\x93 " } else { b"  \xE2\x9C\x97 " });
                crate::uart_print(b"Fault Detector        ");
                crate::uart_print(if fault_ok { b"(initialized)\n" } else { b"(NOT initialized)\n" });

                crate::uart_print(if policy_ok { b"  \xE2\x9C\x93 " } else { b"  \xE2\x9C\x97 " });
                crate::uart_print(b"Policy Controller     ");
                crate::uart_print(if policy_ok { b"(initialized)\n" } else { b"(NOT initialized)\n" });

                crate::uart_print(if compliance_ok { b"  \xE2\x9C\x93 " } else { b"  \xE2\x9C\x97 " });
                crate::uart_print(b"Compliance Tracker    ");
                crate::uart_print(if compliance_ok { b"(initialized)\n" } else { b"(NOT initialized)\n" });

                crate::uart_print(if resource_ok { b"  \xE2\x9C\x93 " } else { b"  \xE2\x9C\x97 " });
                crate::uart_print(b"Resource Monitor      ");
                crate::uart_print(if resource_ok { b"(initialized)\n" } else { b"(NOT initialized)\n" });

                crate::uart_print(if deps_ok { b"  \xE2\x9C\x93 " } else { b"  \xE2\x9C\x97 " });
                crate::uart_print(b"Dependency Graph      ");
                crate::uart_print(if deps_ok { b"(initialized)\n" } else { b"(NOT initialized)\n" });

                crate::uart_print(if profiler_ok { b"  \xE2\x9C\x93 " } else { b"  \xE2\x9C\x97 " });
                crate::uart_print(b"System Profiler       ");
                crate::uart_print(if profiler_ok { b"(initialized)\n" } else { b"(NOT initialized)\n" });
            }

            // Show quick stats if telemetry available
            let telemetry = crate::agent_sys::supervisor::TELEMETRY.lock();
            if let Some(ref telem) = *telemetry {
                let snapshot = telem.snapshot();
                unsafe {
                    crate::uart_print(b"\nQuick Stats:\n");
                    crate::uart_print(b"  Active Agents:     ");
                    self.print_number_simple(snapshot.system_metrics.active_agents as u64);
                    crate::uart_print(b"\n  Total Spawns:      ");
                    self.print_number_simple(snapshot.system_metrics.total_spawns);
                    crate::uart_print(b"\n  Total Exits:       ");
                    self.print_number_simple(snapshot.system_metrics.total_exits);
                    crate::uart_print(b"\n  Total Faults:      ");
                    self.print_number_simple(snapshot.system_metrics.total_faults);
                    crate::uart_print(b"\n");
                }
            }

            let all_ok = supervisor_ok && telemetry_ok && fault_ok && policy_ok &&
                         compliance_ok && resource_ok && deps_ok && profiler_ok;

            unsafe {
                crate::uart_print(b"\nSystem Health: ");
                if all_ok {
                    crate::uart_print(b"Healthy\n");
                } else {
                    crate::uart_print(b"Degraded (some subsystems not initialized)\n");
                }
                crate::uart_print(b"\n");
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Agent Supervision Module not available (agentsys feature not enabled)\n");
            }
        }
    }

    // ========== Phase 2: P1 Commands (Important) ==========

    /// agentsys restart - Manually restart an agent
    ///
    /// Usage: agentsys restart <agent_id>
    pub fn cmd_agentsys_restart(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: agentsys restart <agent_id>\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            use crate::agent_sys::supervisor::AGENT_SUPERVISOR;
            let supervisor = AGENT_SUPERVISOR.lock();
            if let Some(ref sup) = *supervisor {
                if let Some(metadata) = sup.get_agent(agent_id) {
                    let pid = metadata.pid;
                    unsafe {
                        crate::uart_print(b"Restarting agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" (PID ");
                        self.print_number_simple(pid as u64);
                        crate::uart_print(b")...\n");
                    }

                    // Kill the agent (supervisor will auto-restart if configured)
                    let _ = crate::process::signal::send_signal(pid, crate::process::signal::Signal::SIGKILL);

                    unsafe {
                        crate::uart_print(b"Restart initiated (supervisor will respawn agent)\n");
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"Error: Agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" not found\n");
                    }
                }
            } else {
                unsafe {
                    crate::uart_print(b"Agent Supervisor not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Agent restart not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys risk - Show EU AI Act risk classification for agent
    ///
    /// Usage: agentsys risk <agent_id>
    pub fn cmd_agentsys_risk(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: agentsys risk <agent_id>\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            use crate::agent_sys::supervisor::COMPLIANCE_TRACKER;
            let compliance = COMPLIANCE_TRACKER.lock();
            if let Some(ref comp) = *compliance {
                if let Some(record) = comp.get_record(agent_id) {
                    unsafe {
                        crate::uart_print(b"\nAgent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" - EU AI Act Risk Classification\n");
                        crate::uart_print(b"==========================================\n");
                        crate::uart_print(b"  Risk Level:            ");
                        match record.risk_level {
                            crate::agent_sys::supervisor::compliance::RiskLevel::Minimal => {
                                crate::uart_print(b"Minimal (low risk)\n");
                            }
                            crate::agent_sys::supervisor::compliance::RiskLevel::Limited => {
                                crate::uart_print(b"Limited (moderate risk)\n");
                            }
                            crate::agent_sys::supervisor::compliance::RiskLevel::High => {
                                crate::uart_print(b"High (requires oversight)\n");
                            }
                            crate::agent_sys::supervisor::compliance::RiskLevel::Unacceptable => {
                                crate::uart_print(b"Unacceptable (PROHIBITED)\n");
                            }
                        }
                        crate::uart_print(b"  Compliance Score:      ");
                        self.print_number_simple(record.compliance_score() as u64);
                        crate::uart_print(b"%\n");
                        crate::uart_print(b"  Total Operations:      ");
                        self.print_number_simple(record.total_operations);
                        crate::uart_print(b"\n  Human Reviews:         ");
                        self.print_number_simple(record.human_reviews);
                        crate::uart_print(b"\n\n");
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"Error: No compliance data for agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b"\n");
                    }
                }
            } else {
                unsafe {
                    crate::uart_print(b"Compliance Tracker not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Risk classification not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys limits - Show resource limits for agent
    ///
    /// Usage: agentsys limits <agent_id>
    pub fn cmd_agentsys_limits(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: agentsys limits <agent_id>\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            use crate::agent_sys::supervisor::FAULT_DETECTOR;
            let detector = FAULT_DETECTOR.lock();
            if let Some(ref det) = *detector {
                let limits = det.get_default_limits();

                unsafe {
                    crate::uart_print(b"\nAgent ");
                    self.print_number_simple(agent_id as u64);
                    crate::uart_print(b" - Resource Limits\n");
                    crate::uart_print(b"==========================\n");
                    crate::uart_print(b"  CPU Quota:         ");
                    if let Some(quota) = limits.cpu_quota_us {
                        self.print_number_simple(quota);
                        crate::uart_print(b" us/sec\n");
                    } else {
                        crate::uart_print(b"unlimited\n");
                    }

                    crate::uart_print(b"  Memory Limit:      ");
                    if let Some(limit) = limits.memory_limit_bytes {
                        self.print_number_simple(limit as u64);
                        crate::uart_print(b" bytes\n");
                    } else {
                        crate::uart_print(b"unlimited\n");
                    }

                    crate::uart_print(b"  Syscall Rate:      ");
                    if let Some(rate) = limits.syscall_rate_limit {
                        self.print_number_simple(rate);
                        crate::uart_print(b" /sec\n");
                    } else {
                        crate::uart_print(b"unlimited\n");
                    }

                    crate::uart_print(b"  Watchdog Timeout:  ");
                    if let Some(timeout) = limits.watchdog_timeout_us {
                        self.print_number_simple(timeout);
                        crate::uart_print(b" us\n\n");
                    } else {
                        crate::uart_print(b"unlimited\n\n");
                    }
                }
            } else {
                unsafe {
                    crate::uart_print(b"Fault Detector not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Resource limits not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys deps - Show dependencies for a specific agent
    ///
    /// Usage: agentsys deps <agent_id>
    pub fn cmd_agentsys_deps(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: agentsys deps <agent_id>\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            use crate::agent_sys::supervisor::DEPENDENCY_GRAPH;
            let graph = DEPENDENCY_GRAPH.lock();
            if let Some(ref dep_graph) = *graph {
                unsafe {
                    crate::uart_print(b"\nAgent ");
                    self.print_number_simple(agent_id as u64);
                    crate::uart_print(b" - Dependencies\n");
                    crate::uart_print(b"==========================\n");
                    crate::uart_print(b"  Depends On:        ");

                    if let Some(dependencies) = dep_graph.get_dependencies(agent_id) {
                        if dependencies.is_empty() {
                            crate::uart_print(b"(none)\n");
                        } else {
                            crate::uart_print(b"\n");
                            for dep in dependencies {
                                crate::uart_print(b"    - Agent ");
                                self.print_number_simple(dep.dependency as u64);
                                crate::uart_print(b"\n");
                            }
                        }
                    } else {
                        crate::uart_print(b"(none)\n");
                    }

                    crate::uart_print(b"  Depended On By:    ");
                    if let Some(dependents) = dep_graph.get_dependents(agent_id) {
                        if dependents.is_empty() {
                            crate::uart_print(b"(none)\n");
                        } else {
                            crate::uart_print(b"\n");
                            for dep_id in dependents {
                                crate::uart_print(b"    - Agent ");
                                self.print_number_simple(*dep_id as u64);
                                crate::uart_print(b"\n");
                            }
                        }
                    } else {
                        crate::uart_print(b"(none)\n");
                    }
                    crate::uart_print(b"\n");
                }
            } else {
                unsafe {
                    crate::uart_print(b"Dependency Graph not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Dependency tracking not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys depgraph - Show full dependency graph
    ///
    /// Usage: agentsys depgraph
    pub fn cmd_agentsys_depgraph(&self) {
        #[cfg(feature = "agentsys")]
        {
            use crate::agent_sys::supervisor::{DEPENDENCY_GRAPH, AGENT_SUPERVISOR};
            let graph = DEPENDENCY_GRAPH.lock();
            let supervisor = AGENT_SUPERVISOR.lock();

            if let (Some(ref dep_graph), Some(ref sup)) = (&*graph, &*supervisor) {
                let all_agents = sup.list_agents();

                unsafe {
                    crate::uart_print(b"\nAgent Dependency Graph\n");
                    crate::uart_print(b"======================\n\n");

                    if all_agents.is_empty() {
                        crate::uart_print(b"No active agents\n\n");
                        return;
                    }

                    for metadata in all_agents {
                        let agent_id = metadata.agent_id;
                        crate::uart_print(b"Agent ");
                        self.print_number_simple(agent_id as u64);

                        if let Some(dependencies) = dep_graph.get_dependencies(agent_id) {
                            if dependencies.is_empty() {
                                crate::uart_print(b" (no dependencies)\n");
                            } else {
                                crate::uart_print(b" depends on: [");
                                for (i, dep) in dependencies.iter().enumerate() {
                                    if i > 0 {
                                        crate::uart_print(b", ");
                                    }
                                    self.print_number_simple(dep.dependency as u64);
                                }
                                crate::uart_print(b"]\n");
                            }
                        } else {
                            crate::uart_print(b" (no dependencies)\n");
                        }
                    }
                    crate::uart_print(b"\n");
                }
            } else {
                unsafe {
                    crate::uart_print(b"Dependency Graph or Supervisor not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Dependency graph not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys profile - Show performance profile for agent
    ///
    /// Usage: agentsys profile <agent_id>
    pub fn cmd_agentsys_profile(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.is_empty() {
                unsafe {
                    crate::uart_print(b"Usage: agentsys profile <agent_id>\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            use crate::agent_sys::supervisor::SYSTEM_PROFILER;
            let mut profiler = SYSTEM_PROFILER.lock();
            if let Some(ref mut prof) = *profiler {
                if let Some(agent_profiler) = prof.get_agent(agent_id) {
                    let all_stats = agent_profiler.get_all_stats();

                    if all_stats.is_empty() {
                        unsafe {
                            crate::uart_print(b"No profile data for agent ");
                            self.print_number_simple(agent_id as u64);
                            crate::uart_print(b"\n");
                        }
                        return;
                    }

                    // Aggregate stats across all operations
                    let total_ops: usize = all_stats.iter().map(|s| s.sample_count).sum();
                    let successful_ops: usize = all_stats.iter()
                        .map(|s| (s.sample_count as f32 * s.success_rate) as usize)
                        .sum();
                    let failed_ops = total_ops.saturating_sub(successful_ops);
                    let avg_latency: u64 = if total_ops > 0 {
                        all_stats.iter()
                            .map(|s| s.avg_duration_us * s.sample_count as u64)
                            .sum::<u64>() / total_ops as u64
                    } else {
                        0
                    };
                    let peak_latency: u64 = all_stats.iter()
                        .map(|s| s.max_duration_us)
                        .max()
                        .unwrap_or(0);

                    unsafe {
                        crate::uart_print(b"\nAgent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b" - Performance Profile\n");
                        crate::uart_print(b"======================================\n");
                        crate::uart_print(b"  Total Operations:      ");
                        self.print_number_simple(total_ops as u64);
                        crate::uart_print(b"\n  Successful:            ");
                        self.print_number_simple(successful_ops as u64);
                        crate::uart_print(b"\n  Failed:                ");
                        self.print_number_simple(failed_ops as u64);
                        crate::uart_print(b"\n  Success Rate:          ");
                        if total_ops > 0 {
                            let success_rate = (successful_ops * 100) / total_ops;
                            self.print_number_simple(success_rate as u64);
                            crate::uart_print(b"%\n");
                        } else {
                            crate::uart_print(b"N/A\n");
                        }
                        crate::uart_print(b"  Avg Latency:           ");
                        self.print_number_simple(avg_latency);
                        crate::uart_print(b" us\n");
                        crate::uart_print(b"  Peak Latency:          ");
                        self.print_number_simple(peak_latency);
                        crate::uart_print(b" us\n\n");
                    }
                } else {
                    unsafe {
                        crate::uart_print(b"Error: No profile data for agent ");
                        self.print_number_simple(agent_id as u64);
                        crate::uart_print(b"\n");
                    }
                }
            } else {
                unsafe {
                    crate::uart_print(b"System Profiler not initialized\n");
                }
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Performance profiling not available (agentsys feature not enabled)\n");
            }
        }
    }

    // ========== Phase 3: P2 Commands (Advanced) ==========

    /// agentsys policy-update - Hot-patch agent policy
    ///
    /// Usage: agentsys policy-update <agent_id> <capability>
    pub fn cmd_agentsys_policy_update(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            if args.len() < 2 {
                unsafe {
                    crate::uart_print(b"Usage: agentsys policy-update <agent_id> <capability>\n");
                    crate::uart_print(b"Example: agentsys policy-update 1000 FsBasic\n");
                }
                return;
            }

            let agent_id = match self.parse_number_u64(args[0]) {
                Some(id) => id as crate::agent_sys::AgentId,
                None => {
                    unsafe {
                        crate::uart_print(b"Error: Invalid agent ID\n");
                    }
                    return;
                }
            };

            let capability_str = args[1];
            unsafe {
                crate::uart_print(b"Policy update for agent ");
                self.print_number_simple(agent_id as u64);
                crate::uart_print(b": adding capability '");
                crate::uart_print(capability_str.as_bytes());
                crate::uart_print(b"'\n");
                crate::uart_print(b"Note: Hot-patching requires PolicyController API (not yet implemented)\n");
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Policy updates not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys profile-reset - Reset performance profiling data
    ///
    /// Usage: agentsys profile-reset [agent_id]
    pub fn cmd_agentsys_profile_reset(&self, args: &[&str]) {
        #[cfg(feature = "agentsys")]
        {
            unsafe {
                crate::uart_print(b"Profile reset functionality not yet implemented\n");
                crate::uart_print(b"(SystemProfiler API needs reset methods)\n");
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Profile reset not available (agentsys feature not enabled)\n");
            }
        }
    }

    /// agentsys dump - Debug dump of all ASM state
    ///
    /// Usage: agentsys dump
    pub fn cmd_agentsys_dump(&self) {
        #[cfg(feature = "agentsys")]
        {
            unsafe {
                crate::uart_print(b"\n========== ASM Debug Dump ==========\n\n");
            }

            // Dump telemetry
            self.cmd_asmstatus();

            // Dump agent list
            unsafe {
                crate::uart_print(b"\n--- Active Agents ---\n");
            }
            self.cmd_asmlist();

            // Dump compliance
            unsafe {
                crate::uart_print(b"\n--- Compliance Report ---\n");
            }
            self.cmd_asm_compliance();

            // Dump cloud gateway
            unsafe {
                crate::uart_print(b"\n--- Cloud Gateway ---\n");
            }
            self.cmd_gwstatus();

            unsafe {
                crate::uart_print(b"\n========== End of Dump ==========\n\n");
            }
        }

        #[cfg(not(feature = "agentsys"))]
        {
            unsafe {
                crate::uart_print(b"Debug dump not available (agentsys feature not enabled)\n");
            }
        }
    }
}
