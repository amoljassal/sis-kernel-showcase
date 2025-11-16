//! Policy controller for dynamic policy updates
//!
//! The PolicyController extends the existing PolicyEngine with support for
//! dynamic policy updates, hot-patching, and compliance export.

use super::types::*;
use crate::agent_sys::AgentId;
use crate::security::agent_policy::{AgentToken, Capability, PolicyDecision, Scope};
use crate::trace::metric_kv;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::String;

pub use super::POLICY_CONTROLLER;

/// Policy patch operation
#[derive(Debug, Clone)]
pub enum PolicyPatch {
    /// Add a capability
    AddCapability(Capability),

    /// Remove a capability
    RemoveCapability(Capability),

    /// Update scope restrictions
    UpdateScope(Scope),

    /// Enable auto-restart
    EnableAutoRestart { max_restarts: u32 },

    /// Disable auto-restart
    DisableAutoRestart,
}

impl PolicyPatch {
    /// Check if this patch is safe to apply (no privilege escalation)
    pub fn is_safe(&self) -> bool {
        match self {
            // Adding Admin capability would be a privilege escalation
            PolicyPatch::AddCapability(Capability::Admin) => false,
            // Other operations are safe
            _ => true,
        }
    }
}

/// Policy error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    /// Agent not found
    AgentNotFound,

    /// Insufficient permission to modify policy
    InsufficientPermission,

    /// Attempted privilege escalation
    PrivilegeEscalation,

    /// Invalid policy patch
    InvalidPatch,
}

/// Per-agent policy set
#[derive(Debug, Clone)]
pub struct PolicySet {
    /// Agent identifier
    pub agent_id: AgentId,

    /// Current capabilities
    pub capabilities: Vec<Capability>,

    /// Scope restrictions
    pub scope: Scope,

    /// Auto-restart configuration
    pub auto_restart: bool,

    /// Max restart attempts
    pub max_restarts: u32,

    /// Policy violations detected
    pub violations: Vec<PolicyViolation>,

    /// Audit trail of policy changes
    pub audit_trail: Vec<PolicyAuditEntry>,
}

impl PolicySet {
    fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            capabilities: Vec::new(),
            scope: Scope::UNRESTRICTED,
            auto_restart: false,
            max_restarts: 3,
            violations: Vec::new(),
            audit_trail: Vec::new(),
        }
    }

    /// Apply a policy patch
    fn apply(&mut self, patch: PolicyPatch) -> Result<(), PolicyError> {
        let now = current_timestamp();

        match patch.clone() {
            PolicyPatch::AddCapability(cap) => {
                if !self.capabilities.contains(&cap) {
                    self.capabilities.push(cap);
                }
            }
            PolicyPatch::RemoveCapability(cap) => {
                self.capabilities.retain(|c| *c != cap);
            }
            PolicyPatch::UpdateScope(scope) => {
                self.scope = scope;
            }
            PolicyPatch::EnableAutoRestart { max_restarts } => {
                self.auto_restart = true;
                self.max_restarts = max_restarts;
            }
            PolicyPatch::DisableAutoRestart => {
                self.auto_restart = false;
            }
        }

        // Record in audit trail
        self.audit_trail.push(PolicyAuditEntry {
            timestamp: now,
            patch,
        });

        Ok(())
    }

    /// Check if agent has a specific capability
    pub fn has_capability(&self, cap: Capability) -> bool {
        self.capabilities.contains(&cap)
    }

    /// Record a policy violation
    fn record_violation(&mut self, violation: PolicyViolation) {
        // Keep only last 100 violations
        if self.violations.len() >= 100 {
            self.violations.remove(0);
        }
        self.violations.push(violation);
    }
}

/// Policy violation record
#[derive(Debug, Clone)]
pub struct PolicyViolation {
    /// Timestamp of violation
    pub timestamp: Timestamp,

    /// Description of the violation
    pub description: String,

    /// Decision made
    pub decision: PolicyDecision,
}

/// Policy audit entry
#[derive(Debug, Clone)]
pub struct PolicyAuditEntry {
    /// Timestamp of change
    pub timestamp: Timestamp,

    /// Patch that was applied
    pub patch: PolicyPatch,
}

/// Compliance report entry
#[derive(Debug, Clone)]
pub struct ComplianceEntry {
    /// Agent identifier
    pub agent_id: AgentId,

    /// Current capabilities
    pub capabilities: Vec<Capability>,

    /// Policy violations
    pub violations: Vec<PolicyViolation>,

    /// Audit trail
    pub audit_trail: Vec<PolicyAuditEntry>,
}

/// Compliance report
#[derive(Debug, Clone)]
pub struct ComplianceReport {
    /// Report generation timestamp
    pub timestamp: Timestamp,

    /// Per-agent compliance entries
    pub agents: Vec<ComplianceEntry>,
}

/// Policy controller
///
/// Manages dynamic policy updates and maintains compliance records.
/// Thread-safe when accessed through the global POLICY_CONTROLLER mutex.
pub struct PolicyController {
    /// Current policy rules by agent ID
    policies: BTreeMap<AgentId, PolicySet>,

    /// Global default policy
    default_policy: PolicySet,
}

impl PolicyController {
    /// Create a new policy controller
    pub fn new() -> Self {
        Self {
            policies: BTreeMap::new(),
            default_policy: PolicySet::new(0),
        }
    }

    /// Update agent policy with a patch
    pub fn update_policy(
        &mut self,
        agent_id: AgentId,
        patch: PolicyPatch,
    ) -> Result<(), PolicyError> {
        // Validate patch safety
        if !patch.is_safe() {
            metric_kv("policy_privilege_escalation_blocked", 1);
            return Err(PolicyError::PrivilegeEscalation);
        }

        // Get or create policy set
        let policy = self
            .policies
            .entry(agent_id)
            .or_insert_with(|| PolicySet::new(agent_id));

        // Apply patch
        policy.apply(patch)?;

        // Notify supervisor of policy change
        if let Some(supervisor) = super::AGENT_SUPERVISOR.lock().as_mut() {
            // Update supervisor's agent metadata if it exists
            if let Some(metadata) = supervisor.get_agent_mut(agent_id) {
                metadata.capabilities = policy.capabilities.clone();
                metadata.scope = policy.scope;
                metadata.auto_restart = policy.auto_restart;
                metadata.max_restarts = policy.max_restarts;
            }
        }

        // Record telemetry
        if let Some(telemetry) = super::TELEMETRY.lock().as_mut() {
            telemetry.record_policy_change(agent_id);
        }

        metric_kv("policy_update_success", 1);
        Ok(())
    }

    /// Get policy for an agent
    pub fn get_policy(&self, agent_id: AgentId) -> Option<&PolicySet> {
        self.policies.get(&agent_id)
    }

    /// Get mutable policy for an agent
    pub fn get_policy_mut(&mut self, agent_id: AgentId) -> Option<&mut PolicySet> {
        self.policies.get_mut(&agent_id)
    }

    /// Record a policy violation
    pub fn record_violation(
        &mut self,
        agent_id: AgentId,
        description: String,
        decision: PolicyDecision,
    ) {
        let violation = PolicyViolation {
            timestamp: current_timestamp(),
            description,
            decision,
        };

        let policy = self
            .policies
            .entry(agent_id)
            .or_insert_with(|| PolicySet::new(agent_id));

        policy.record_violation(violation);
        metric_kv("policy_violation_recorded", 1);
    }

    /// Export compliance report
    pub fn export_compliance(&self) -> ComplianceReport {
        let entries: Vec<ComplianceEntry> = self
            .policies
            .iter()
            .map(|(agent_id, policy)| ComplianceEntry {
                agent_id: *agent_id,
                capabilities: policy.capabilities.clone(),
                violations: policy.violations.clone(),
                audit_trail: policy.audit_trail.clone(),
            })
            .collect();

        ComplianceReport {
            timestamp: current_timestamp(),
            agents: entries,
        }
    }

    /// Export EU AI Act compliance report
    ///
    /// Generates a compliance report following EU AI Act requirements
    /// (Articles 13-16: Transparency and provision of information)
    pub fn export_eu_ai_act_report(&self) -> ComplianceReport {
        // For now, same as regular compliance report
        // In a full implementation, this would include additional
        // EU AI Act specific fields
        self.export_compliance()
    }

    /// Initialize policy from agent token
    pub fn init_from_token(&mut self, token: &AgentToken) {
        let mut policy = PolicySet::new(token.agent_id);
        policy.capabilities = token.capabilities.to_vec();
        policy.scope = token.scope;

        self.policies.insert(token.agent_id, policy);
    }

    /// Get count of managed policies
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }
}

impl Default for PolicyController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_patch_safety() {
        let patch = PolicyPatch::AddCapability(Capability::FsBasic);
        assert!(patch.is_safe());

        let patch = PolicyPatch::AddCapability(Capability::Admin);
        assert!(!patch.is_safe());
    }

    #[test]
    fn test_update_policy() {
        let mut controller = PolicyController::new();

        let patch = PolicyPatch::AddCapability(Capability::FsBasic);
        let result = controller.update_policy(100, patch);
        assert!(result.is_ok());

        let policy = controller.get_policy(100).unwrap();
        assert!(policy.has_capability(Capability::FsBasic));
    }

    #[test]
    fn test_privilege_escalation_blocked() {
        let mut controller = PolicyController::new();

        let patch = PolicyPatch::AddCapability(Capability::Admin);
        let result = controller.update_policy(100, patch);
        assert_eq!(result, Err(PolicyError::PrivilegeEscalation));
    }

    #[test]
    fn test_compliance_export() {
        let mut controller = PolicyController::new();

        controller.update_policy(100, PolicyPatch::AddCapability(Capability::FsBasic)).ok();
        controller.update_policy(101, PolicyPatch::AddCapability(Capability::AudioControl)).ok();

        let report = controller.export_compliance();
        assert_eq!(report.agents.len(), 2);
    }
}
