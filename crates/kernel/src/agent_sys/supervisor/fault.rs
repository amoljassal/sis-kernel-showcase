//! Fault detection and recovery policies
//!
//! The FaultDetector monitors agent behavior and detects violations
//! of resource limits, policy violations, and other fault conditions.

use crate::agent_sys::AgentId;
use crate::security::agent_policy::Capability;
use crate::process::signal::Signal;
use alloc::string::String;

pub use super::FAULT_DETECTOR;

/// Fault types that can be detected
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Fault {
    /// CPU usage exceeded quota
    CpuQuotaExceeded {
        /// CPU time used in microseconds
        used: u64,
        /// Quota limit in microseconds
        quota: u64,
    },

    /// Memory usage exceeded limit
    MemoryExceeded {
        /// Memory used in bytes
        used: usize,
        /// Memory limit in bytes
        limit: usize,
    },

    /// Syscall rate limit exceeded
    SyscallFlood {
        /// Syscall rate (calls/second)
        rate: u64,
        /// Rate limit threshold
        threshold: u64,
    },

    /// Agent crashed (received signal)
    Crashed {
        /// Signal that caused the crash
        signal: u32,
    },

    /// Capability violation attempted
    CapabilityViolation {
        /// Capability that was attempted without permission
        attempted: Capability,
    },

    /// Agent became unresponsive (watchdog timeout)
    Unresponsive {
        /// Time since last activity in microseconds
        idle_time: u64,
        /// Watchdog timeout threshold
        threshold: u64,
    },

    /// Policy violation
    PolicyViolation {
        /// Description of the violation
        reason: String,
    },
}

impl Fault {
    /// Get a human-readable description of the fault
    pub fn description(&self) -> &'static str {
        match self {
            Fault::CpuQuotaExceeded { .. } => "CPU quota exceeded",
            Fault::MemoryExceeded { .. } => "Memory limit exceeded",
            Fault::SyscallFlood { .. } => "Syscall rate limit exceeded",
            Fault::Crashed { .. } => "Agent crashed",
            Fault::CapabilityViolation { .. } => "Capability violation",
            Fault::Unresponsive { .. } => "Agent unresponsive",
            Fault::PolicyViolation { .. } => "Policy violation",
        }
    }

    /// Get the severity level of the fault
    pub fn severity(&self) -> FaultSeverity {
        match self {
            Fault::Crashed { .. } => FaultSeverity::Critical,
            Fault::CapabilityViolation { .. } => FaultSeverity::Critical,
            Fault::PolicyViolation { .. } => FaultSeverity::High,
            Fault::CpuQuotaExceeded { .. } => FaultSeverity::Medium,
            Fault::MemoryExceeded { .. } => FaultSeverity::High,
            Fault::SyscallFlood { .. } => FaultSeverity::Medium,
            Fault::Unresponsive { .. } => FaultSeverity::High,
        }
    }
}

/// Fault severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultSeverity {
    /// Low severity - informational only
    Low = 1,

    /// Medium severity - throttle or warn
    Medium = 2,

    /// High severity - may require intervention
    High = 3,

    /// Critical severity - immediate action required
    Critical = 4,
}

/// Action to take in response to a fault
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultAction {
    /// Kill the agent immediately
    Kill,

    /// Throttle the agent's resources
    Throttle,

    /// Restart the agent
    Restart,

    /// Alert the administrator but continue running
    Alert,
}

/// Recovery policy configuration
#[derive(Debug, Clone, Copy)]
pub struct RecoveryPolicy {
    /// Action for CPU quota violations
    pub cpu_quota_action: FaultAction,

    /// Action for memory violations
    pub memory_action: FaultAction,

    /// Action for syscall floods
    pub syscall_flood_action: FaultAction,

    /// Action for crashes
    pub crash_action: FaultAction,

    /// Action for capability violations
    pub capability_violation_action: FaultAction,

    /// Action for unresponsive agents
    pub unresponsive_action: FaultAction,

    /// Action for policy violations
    pub policy_violation_action: FaultAction,
}

impl RecoveryPolicy {
    /// Create a default recovery policy
    pub fn default() -> Self {
        Self {
            cpu_quota_action: FaultAction::Throttle,
            memory_action: FaultAction::Kill,
            syscall_flood_action: FaultAction::Throttle,
            crash_action: FaultAction::Restart,
            capability_violation_action: FaultAction::Kill,
            unresponsive_action: FaultAction::Restart,
            policy_violation_action: FaultAction::Kill,
        }
    }

    /// Create a permissive recovery policy (for testing)
    pub fn permissive() -> Self {
        Self {
            cpu_quota_action: FaultAction::Alert,
            memory_action: FaultAction::Alert,
            syscall_flood_action: FaultAction::Alert,
            crash_action: FaultAction::Alert,
            capability_violation_action: FaultAction::Alert,
            unresponsive_action: FaultAction::Alert,
            policy_violation_action: FaultAction::Alert,
        }
    }

    /// Create a strict recovery policy
    pub fn strict() -> Self {
        Self {
            cpu_quota_action: FaultAction::Kill,
            memory_action: FaultAction::Kill,
            syscall_flood_action: FaultAction::Kill,
            crash_action: FaultAction::Kill,
            capability_violation_action: FaultAction::Kill,
            unresponsive_action: FaultAction::Kill,
            policy_violation_action: FaultAction::Kill,
        }
    }

    /// Determine the appropriate action for a given fault
    pub fn action_for(&self, fault: &Fault) -> FaultAction {
        match fault {
            Fault::CpuQuotaExceeded { .. } => self.cpu_quota_action,
            Fault::MemoryExceeded { .. } => self.memory_action,
            Fault::SyscallFlood { .. } => self.syscall_flood_action,
            Fault::Crashed { .. } => self.crash_action,
            Fault::CapabilityViolation { .. } => self.capability_violation_action,
            Fault::Unresponsive { .. } => self.unresponsive_action,
            Fault::PolicyViolation { .. } => self.policy_violation_action,
        }
    }
}

/// Resource limits for an agent
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    /// Maximum CPU time in microseconds per time window
    pub cpu_quota_us: Option<u64>,

    /// Maximum memory usage in bytes
    pub memory_limit_bytes: Option<usize>,

    /// Maximum syscalls per second
    pub syscall_rate_limit: Option<u64>,

    /// Watchdog timeout in microseconds
    pub watchdog_timeout_us: Option<u64>,
}

impl ResourceLimits {
    /// Create unlimited resource limits
    pub const fn unlimited() -> Self {
        Self {
            cpu_quota_us: None,
            memory_limit_bytes: None,
            syscall_rate_limit: None,
            watchdog_timeout_us: None,
        }
    }

    /// Create conservative resource limits
    pub const fn conservative() -> Self {
        Self {
            cpu_quota_us: Some(1_000_000), // 1 second per window
            memory_limit_bytes: Some(100 * 1024 * 1024), // 100 MB
            syscall_rate_limit: Some(1000), // 1000 syscalls/sec
            watchdog_timeout_us: Some(30_000_000), // 30 seconds
        }
    }
}

/// Fault detector
///
/// Monitors agent behavior and detects fault conditions.
/// Thread-safe when accessed through the global FAULT_DETECTOR mutex.
pub struct FaultDetector {
    /// Recovery policy
    recovery_policy: RecoveryPolicy,

    /// Default resource limits for new agents
    default_limits: ResourceLimits,
}

impl FaultDetector {
    /// Create a new fault detector
    pub fn new() -> Self {
        Self {
            recovery_policy: RecoveryPolicy::default(),
            default_limits: ResourceLimits::conservative(),
        }
    }

    /// Get the current recovery policy
    pub fn get_recovery_policy(&self) -> &RecoveryPolicy {
        &self.recovery_policy
    }

    /// Set the recovery policy
    pub fn set_recovery_policy(&mut self, policy: RecoveryPolicy) {
        self.recovery_policy = policy;
    }

    /// Get default resource limits
    pub fn get_default_limits(&self) -> &ResourceLimits {
        &self.default_limits
    }

    /// Set default resource limits
    pub fn set_default_limits(&mut self, limits: ResourceLimits) {
        self.default_limits = limits;
    }

    /// Check if an agent has exceeded CPU quota
    pub fn check_cpu_quota(&self, agent_id: AgentId, cpu_time_us: u64) -> Option<Fault> {
        if let Some(quota) = self.default_limits.cpu_quota_us {
            if cpu_time_us > quota {
                return Some(Fault::CpuQuotaExceeded {
                    used: cpu_time_us,
                    quota,
                });
            }
        }
        None
    }

    /// Check if an agent has exceeded memory limit
    pub fn check_memory_limit(&self, agent_id: AgentId, memory_bytes: usize) -> Option<Fault> {
        if let Some(limit) = self.default_limits.memory_limit_bytes {
            if memory_bytes > limit {
                return Some(Fault::MemoryExceeded {
                    used: memory_bytes,
                    limit,
                });
            }
        }
        None
    }

    /// Check if an agent is flooding syscalls
    pub fn check_syscall_rate(&self, agent_id: AgentId, rate: u64) -> Option<Fault> {
        if let Some(threshold) = self.default_limits.syscall_rate_limit {
            if rate > threshold {
                return Some(Fault::SyscallFlood { rate, threshold });
            }
        }
        None
    }

    /// Check if an agent is unresponsive
    pub fn check_watchdog(&self, agent_id: AgentId, idle_time: u64) -> Option<Fault> {
        if let Some(threshold) = self.default_limits.watchdog_timeout_us {
            if idle_time > threshold {
                return Some(Fault::Unresponsive {
                    idle_time,
                    threshold,
                });
            }
        }
        None
    }

    /// Report a crash
    pub fn report_crash(&self, agent_id: AgentId, signal: Signal) -> Fault {
        Fault::Crashed {
            signal: signal as u32,
        }
    }

    /// Report a capability violation
    pub fn report_capability_violation(&self, agent_id: AgentId, cap: Capability) -> Fault {
        Fault::CapabilityViolation { attempted: cap }
    }

    /// Report a policy violation
    pub fn report_policy_violation(&self, agent_id: AgentId, reason: String) -> Fault {
        Fault::PolicyViolation { reason }
    }
}

impl Default for FaultDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_severity() {
        assert_eq!(
            Fault::Crashed { signal: 11 }.severity(),
            FaultSeverity::Critical
        );
        assert_eq!(
            Fault::CpuQuotaExceeded {
                used: 100,
                quota: 50
            }
            .severity(),
            FaultSeverity::Medium
        );
    }

    #[test]
    fn test_recovery_policy() {
        let policy = RecoveryPolicy::default();

        let fault = Fault::Crashed { signal: 11 };
        assert_eq!(policy.action_for(&fault), FaultAction::Restart);

        let fault = Fault::CapabilityViolation {
            attempted: Capability::Admin,
        };
        assert_eq!(policy.action_for(&fault), FaultAction::Kill);
    }

    #[test]
    fn test_fault_detection() {
        let detector = FaultDetector::new();

        let fault = detector.check_memory_limit(100, 200 * 1024 * 1024);
        assert!(fault.is_some());
        assert!(matches!(fault.unwrap(), Fault::MemoryExceeded { .. }));
    }
}
