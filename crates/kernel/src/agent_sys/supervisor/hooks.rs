//! Process manager integration for Agent Supervision Module
//!
//! This module provides integration hooks that are called by the process
//! manager during agent lifecycle events (spawn, exit, etc.).

use super::types::*;
use super::{AGENT_SUPERVISOR, TELEMETRY, FAULT_DETECTOR, COMPLIANCE_TRACKER};
use super::compliance::{ComplianceEvent, RiskLevel};
use crate::process::Pid;
use crate::agent_sys::AgentId;

/// Check if a process is an agent and return its AgentId
///
/// This function checks if a given PID corresponds to an agent process
/// by looking it up in the supervisor's registry.
pub fn is_agent_process(pid: Pid) -> Option<AgentId> {
    let supervisor = AGENT_SUPERVISOR.lock();
    if let Some(ref sup) = *supervisor {
        if let Some(metadata) = sup.get_agent_by_pid(pid) {
            return Some(metadata.agent_id);
        }
    }
    None
}

/// Process spawn hook for agents
///
/// Call this from the process manager when an agent process is spawned.
/// This function will register the agent with the supervisor and start
/// tracking its lifecycle.
///
/// # Arguments
///
/// * `pid` - Process ID of the newly spawned process
/// * `spec` - Agent specification with capabilities and configuration
///
/// # Returns
///
/// The assigned AgentId for the new agent
///
/// # Example
///
/// ```rust
/// // In process manager after spawning an agent process
/// if is_agent {
///     let agent_id = agent_sys::supervisor::hooks::on_process_spawn(pid, spec);
///     // Agent is now tracked by ASM
/// }
/// ```
pub fn on_process_spawn(pid: Pid, spec: AgentSpec) -> AgentId {
    let agent_id = spec.agent_id;
    let name = spec.name.clone();

    let mut supervisor = AGENT_SUPERVISOR.lock();
    if let Some(ref mut sup) = *supervisor {
        sup.on_agent_spawn(pid, spec);

        // Log compliance event for agent spawn
        drop(supervisor); // Release lock before acquiring compliance lock
        let mut compliance = COMPLIANCE_TRACKER.lock();
        if let Some(ref mut tracker) = *compliance {
            // Classify risk level based on agent capabilities
            // For now, use Limited as default (can be enhanced later)
            let event = ComplianceEvent::AgentSpawned {
                agent_id,
                risk_level: RiskLevel::Limited,
                purpose: alloc::format!("Agent: {}", name),
            };
            tracker.log_event(event);
        }

        agent_id
    } else {
        crate::uart::print_str("[ASM] Warning: Supervisor not initialized during spawn\n");
        agent_id // Return the spec's ID if supervisor not ready
    }
}

/// Process exit hook for agents
///
/// Call this from the process manager when a process exits.
/// If the process is an agent, this will update telemetry and
/// potentially trigger auto-restart.
///
/// # Arguments
///
/// * `pid` - Process ID that is exiting
/// * `exit_code` - Exit status code
///
/// # Returns
///
/// `true` if the process was an agent, `false` otherwise
///
/// # Example
///
/// ```rust
/// // In process manager exit path
/// agent_sys::supervisor::hooks::on_process_exit(pid, exit_code);
/// ```
pub fn on_process_exit(pid: Pid, exit_code: i32) -> bool {
    let mut supervisor = AGENT_SUPERVISOR.lock();
    if let Some(ref mut sup) = *supervisor {
        // Check if this PID is an agent
        if let Some(metadata) = sup.get_agent_by_pid(pid) {
            let agent_id = metadata.agent_id;

            // Get operations count from telemetry before exit
            let operations_count = {
                let telemetry = TELEMETRY.lock();
                if let Some(ref t) = *telemetry {
                    if let Some(agent_metrics) = t.get_agent_metrics(agent_id) {
                        agent_metrics.operations_count
                    } else {
                        0
                    }
                } else {
                    0
                }
            };

            // Update supervisor
            sup.on_agent_exit(pid, exit_code);

            // Log compliance event for agent exit
            drop(supervisor); // Release lock before acquiring compliance lock
            let mut compliance = COMPLIANCE_TRACKER.lock();
            if let Some(ref mut tracker) = *compliance {
                let event = ComplianceEvent::AgentExited {
                    agent_id,
                    exit_code,
                    operations_count,
                };
                tracker.log_event(event);
            }
            drop(compliance);

            // Clean up Cloud Gateway rate limiter for this agent
            #[cfg(feature = "agentsys")]
            {
                let mut gateway = crate::agent_sys::cloud_gateway::CLOUD_GATEWAY.lock();
                if let Some(ref mut gw) = *gateway {
                    gw.remove_agent(agent_id);
                }
            }

            return true;
        }
    }
    false
}

/// Report a fault for an agent
///
/// Call this when the scheduler or other kernel subsystems detect
/// that an agent has violated resource limits or policies.
///
/// # Arguments
///
/// * `agent_id` - Agent that faulted
/// * `fault` - Type of fault detected
///
/// # Returns
///
/// The action taken in response to the fault
///
/// # Example
///
/// ```rust
/// // In scheduler health check
/// use agent_sys::supervisor::fault::Fault;
///
/// if cpu_time > quota {
///     let fault = Fault::CpuQuotaExceeded { used: cpu_time, quota };
///     agent_sys::supervisor::hooks::report_agent_fault(agent_id, fault);
/// }
/// ```
pub fn report_agent_fault(agent_id: AgentId, fault: super::fault::Fault) -> super::fault::FaultAction {
    // Log compliance event for policy violation
    let violation_type = match &fault {
        super::fault::Fault::CpuQuotaExceeded { .. } => "CPU Quota Exceeded",
        super::fault::Fault::MemoryExceeded { .. } => "Memory Limit Exceeded",
        super::fault::Fault::SyscallFlood { .. } => "Syscall Flood",
        super::fault::Fault::WatchdogTimeout { .. } => "Watchdog Timeout",
        super::fault::Fault::Crashed { .. } => "Agent Crashed",
    };

    let severity = match &fault {
        super::fault::Fault::Crashed { .. } => RiskLevel::High,
        super::fault::Fault::MemoryExceeded { .. } => RiskLevel::High,
        super::fault::Fault::SyscallFlood { .. } => RiskLevel::Limited,
        _ => RiskLevel::Limited,
    };

    let mut compliance = COMPLIANCE_TRACKER.lock();
    if let Some(ref mut tracker) = *compliance {
        let event = ComplianceEvent::PolicyViolation {
            agent_id,
            violation_type: violation_type.to_string(),
            severity,
        };
        tracker.log_event(event);
    }
    drop(compliance);

    // Handle fault
    let mut supervisor = AGENT_SUPERVISOR.lock();
    if let Some(ref mut sup) = *supervisor {
        sup.on_fault(agent_id, fault)
    } else {
        crate::uart::print_str("[ASM] Warning: Supervisor not initialized during fault\n");
        super::fault::FaultAction::Alert
    }
}

/// Health check for all agents
///
/// This function should be called periodically by the scheduler (e.g., every timer tick)
/// to check agent health and detect faults.
///
/// # Returns
///
/// Number of faults detected and handled
///
/// # Example
///
/// ```rust
/// // In scheduler timer interrupt
/// pub fn timer_tick() {
///     // ... scheduler logic ...
///
///     // Check agent health
///     agent_sys::supervisor::hooks::periodic_health_check();
/// }
/// ```
pub fn periodic_health_check() -> usize {
    let supervisor = AGENT_SUPERVISOR.lock();
    let fault_detector = FAULT_DETECTOR.lock();

    if let (Some(ref sup), Some(ref detector)) = (&*supervisor, &*fault_detector) {
        let mut fault_count = 0;

        // Check each active agent
        for metadata in sup.list_agents() {
            let agent_id = metadata.agent_id;

            // Check if agent is responsive (watchdog)
            let idle_time = current_timestamp().saturating_sub(metadata.last_activity);
            if let Some(fault) = detector.check_watchdog(agent_id, idle_time) {
                drop(supervisor);
                drop(fault_detector);
                report_agent_fault(agent_id, fault);
                fault_count += 1;
                return fault_count; // Early return to avoid lock issues
            }

            // TODO: Check CPU quota (requires integration with scheduler)
            // TODO: Check memory limit (requires integration with memory manager)
        }

        fault_count
    } else {
        0
    }
}

/// Get telemetry snapshot for userland tools
///
/// This provides a safe interface for userland tools to access
/// telemetry data through syscalls.
pub fn get_telemetry_snapshot() -> Option<super::telemetry::TelemetrySnapshot> {
    let telemetry = TELEMETRY.lock();
    telemetry.as_ref().map(|t| t.snapshot())
}

/// Update agent last activity timestamp
///
/// Call this whenever an agent performs a syscall or other activity
/// to update its watchdog timer.
pub fn touch_agent(agent_id: AgentId) {
    let mut supervisor = AGENT_SUPERVISOR.lock();
    if let Some(ref mut sup) = *supervisor {
        sup.touch_agent(agent_id);
    }
}

/// Touch agent by PID
///
/// Convenience function that looks up the agent by PID
pub fn touch_agent_by_pid(pid: Pid) {
    let supervisor = AGENT_SUPERVISOR.lock();
    if let Some(ref sup) = *supervisor {
        if let Some(metadata) = sup.get_agent_by_pid(pid) {
            let agent_id = metadata.agent_id;
            drop(supervisor);
            touch_agent(agent_id);
        }
    }
}

/// Get compliance report for all agents
///
/// This provides a comprehensive EU AI Act compliance report
/// for use by syscalls and /proc filesystem.
pub fn get_compliance_report() -> Option<super::compliance::ComplianceReport> {
    let compliance = COMPLIANCE_TRACKER.lock();
    compliance.as_ref().map(|t| t.generate_report())
}

/// Log a compliance event (for use by syscall handlers)
///
/// This allows handlers to log compliance events when agents
/// access sensitive data or make decisions.
pub fn log_compliance_event(event: ComplianceEvent) {
    let mut compliance = COMPLIANCE_TRACKER.lock();
    if let Some(ref mut tracker) = *compliance {
        tracker.log_event(event);
    }
}

/// Get compliance score for an agent
///
/// Returns compliance score (0.0 - 1.0) for an agent
pub fn get_agent_compliance_score(agent_id: AgentId) -> Option<f32> {
    let compliance = COMPLIANCE_TRACKER.lock();
    compliance.as_ref()?.get_agent_record(agent_id).map(|r| r.compliance_score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_process_spawn_hook() {
        // Initialize supervisor
        super::super::init();

        let spec = AgentSpec::new(100, "test_agent".to_string())
            .with_capability(crate::security::agent_policy::Capability::FsBasic);

        let agent_id = on_process_spawn(42, spec);
        assert_eq!(agent_id, 100);

        // Verify agent is tracked
        assert_eq!(is_agent_process(42), Some(100));
    }

    #[test]
    fn test_process_exit_hook() {
        // Initialize supervisor
        super::super::init();

        let spec = AgentSpec::new(101, "test_agent".to_string());
        on_process_spawn(43, spec);

        // Exit the agent
        let was_agent = on_process_exit(43, 0);
        assert!(was_agent);

        // Verify agent is no longer tracked
        assert_eq!(is_agent_process(43), None);
    }
}
