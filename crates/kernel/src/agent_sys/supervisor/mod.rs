//! Agent Supervisor Module (ASM)
//!
//! The Agent Supervision Module provides comprehensive lifecycle management,
//! fault detection, and recovery for all agents in the system. This is a
//! kernel-resident service that extends the existing AgentSys infrastructure.
//!
//! # Architecture
//!
//! ASM is **NOT** a userland super-agent. It is implemented as kernel-resident
//! services to avoid single points of failure and maintain security boundaries.
//!
//! ## Key Components
//!
//! - **AgentSupervisor**: Lifecycle hooks, fault detection, recovery
//! - **PolicyController**: Dynamic policy updates, hot-patching, compliance
//! - **TelemetryAggregator**: Metrics collection, /proc export
//! - **FaultDetector**: Resource monitoring, violation detection
//!
//! ## Design Principles
//!
//! 1. **No Userland Super-Agent**: All supervision logic in kernel
//! 2. **Distributed Responsibilities**: Each subsystem handles its domain
//! 3. **Least Privilege**: Services have minimal needed capabilities
//! 4. **Fail-Safe**: Kernel services survive agent failures
//! 5. **Observable**: All actions logged and visible via /proc
//! 6. **Policy-Driven**: Runtime behavior controlled by policy engine
//!
//! # Usage
//!
//! ```rust
//! // Initialize during kernel boot
//! agent_sys::supervisor::init();
//!
//! // Lifecycle hooks are called automatically by process manager
//! // User code typically interacts via syscalls or /proc filesystem
//! ```

pub mod types;
pub mod lifecycle;
pub mod telemetry;
pub mod fault;
pub mod policy_controller;
pub mod hooks;
pub mod compliance;

#[cfg(test)]
mod tests;

pub use types::*;
pub use lifecycle::AgentSupervisor;
pub use telemetry::TelemetryAggregator;
pub use fault::{FaultDetector, Fault, FaultAction, RecoveryPolicy};
pub use policy_controller::PolicyController;
pub use compliance::{ComplianceTracker, ComplianceReport, RiskLevel, ComplianceEvent};

use spin::Mutex;

/// Global agent supervisor instance
pub static AGENT_SUPERVISOR: Mutex<Option<AgentSupervisor>> = Mutex::new(None);

/// Global telemetry aggregator instance
pub static TELEMETRY: Mutex<Option<TelemetryAggregator>> = Mutex::new(None);

/// Global fault detector instance
pub static FAULT_DETECTOR: Mutex<Option<FaultDetector>> = Mutex::new(None);

/// Global policy controller instance
pub static POLICY_CONTROLLER: Mutex<Option<PolicyController>> = Mutex::new(None);

/// Global compliance tracker instance
pub static COMPLIANCE_TRACKER: Mutex<Option<ComplianceTracker>> = Mutex::new(None);

/// Initialize the Agent Supervision Module
///
/// This must be called during kernel initialization, after AgentSys is initialized.
pub fn init() {
    *AGENT_SUPERVISOR.lock() = Some(AgentSupervisor::new());
    *TELEMETRY.lock() = Some(TelemetryAggregator::new());
    *FAULT_DETECTOR.lock() = Some(FaultDetector::new());
    *POLICY_CONTROLLER.lock() = Some(PolicyController::new());
    *COMPLIANCE_TRACKER.lock() = Some(ComplianceTracker::new());

    crate::uart::print_str("[ASM] Agent Supervision Module initialized\n");
    crate::uart::print_str("[ASM] EU AI Act compliance tracking enabled\n");
}

/// Check if ASM is initialized
pub fn is_initialized() -> bool {
    AGENT_SUPERVISOR.lock().is_some()
}
