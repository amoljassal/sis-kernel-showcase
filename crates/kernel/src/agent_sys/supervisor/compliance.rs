//! EU AI Act Compliance Reporting
//!
//! This module implements compliance tracking and reporting for the EU AI Act
//! and other regulatory frameworks. It provides transparent logging, risk
//! assessment, and comprehensive audit trails.
//!
//! # Regulatory Requirements
//!
//! The EU AI Act requires:
//! - Transparency: All AI system decisions must be logged and explainable
//! - Risk Assessment: Systems must be classified and monitored
//! - Human Oversight: Mechanisms for human intervention
//! - Accuracy & Robustness: Monitoring for failures and biases
//! - Audit Trails: Complete records of all operations
//!
//! # Architecture
//!
//! ```
//! Agent Operation → Compliance Logger → Risk Assessor
//!                         ↓                    ↓
//!                   Audit Trail         Risk Score
//!                         ↓                    ↓
//!                   Compliance Report (EU AI Act Format)
//! ```

use crate::agent_sys::AgentId;
use crate::time::get_timestamp_us;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Risk level classification per EU AI Act
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RiskLevel {
    /// Minimal risk - Limited transparency obligations
    Minimal,
    /// Limited risk - Transparency obligations apply
    Limited,
    /// High risk - Strict requirements apply
    High,
    /// Unacceptable risk - Prohibited
    Unacceptable,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Minimal => "Minimal",
            RiskLevel::Limited => "Limited",
            RiskLevel::High => "High",
            RiskLevel::Unacceptable => "Unacceptable",
        }
    }
}

/// Compliance event types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ComplianceEvent {
    /// Agent was spawned
    AgentSpawned {
        agent_id: AgentId,
        risk_level: RiskLevel,
        purpose: String,
    },

    /// Agent made a decision
    DecisionMade {
        agent_id: AgentId,
        decision_type: String,
        confidence: f32,
        human_reviewed: bool,
    },

    /// Agent accessed sensitive data
    SensitiveDataAccess {
        agent_id: AgentId,
        data_category: String,
        access_type: String, // read/write/delete
    },

    /// Policy violation detected
    PolicyViolation {
        agent_id: AgentId,
        violation_type: String,
        severity: RiskLevel,
    },

    /// Human oversight performed
    HumanOversight {
        agent_id: AgentId,
        reviewer_id: String,
        action: String,
    },

    /// Agent exited
    AgentExited {
        agent_id: AgentId,
        exit_code: i32,
        operations_count: u64,
    },
}

impl ComplianceEvent {
    /// Get the agent ID for this event
    pub fn agent_id(&self) -> AgentId {
        match self {
            ComplianceEvent::AgentSpawned { agent_id, .. } => *agent_id,
            ComplianceEvent::DecisionMade { agent_id, .. } => *agent_id,
            ComplianceEvent::SensitiveDataAccess { agent_id, .. } => *agent_id,
            ComplianceEvent::PolicyViolation { agent_id, .. } => *agent_id,
            ComplianceEvent::HumanOversight { agent_id, .. } => *agent_id,
            ComplianceEvent::AgentExited { agent_id, .. } => *agent_id,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            ComplianceEvent::AgentSpawned { purpose, .. } => {
                alloc::format!("Agent spawned: {}", purpose)
            }
            ComplianceEvent::DecisionMade { decision_type, confidence, .. } => {
                alloc::format!("Decision: {} (confidence: {:.1}%)", decision_type, confidence * 100.0)
            }
            ComplianceEvent::SensitiveDataAccess { data_category, access_type, .. } => {
                alloc::format!("Data access: {} ({})", data_category, access_type)
            }
            ComplianceEvent::PolicyViolation { violation_type, severity, .. } => {
                alloc::format!("Policy violation: {} ({})", violation_type, severity.as_str())
            }
            ComplianceEvent::HumanOversight { action, .. } => {
                alloc::format!("Human oversight: {}", action)
            }
            ComplianceEvent::AgentExited { exit_code, operations_count, .. } => {
                alloc::format!("Agent exited: code={}, ops={}", exit_code, operations_count)
            }
        }
    }
}

/// Per-agent compliance record
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AgentComplianceRecord {
    pub agent_id: AgentId,
    pub risk_level: RiskLevel,
    pub purpose: String,
    pub spawn_time: u64,
    pub exit_time: Option<u64>,
    pub decisions_made: u64,
    pub human_reviews: u64,
    pub policy_violations: u64,
    pub sensitive_data_accesses: u64,
    pub total_operations: u64,
}

impl AgentComplianceRecord {
    pub fn new(agent_id: AgentId, risk_level: RiskLevel, purpose: String) -> Self {
        Self {
            agent_id,
            risk_level,
            purpose,
            spawn_time: get_timestamp_us(),
            exit_time: None,
            decisions_made: 0,
            human_reviews: 0,
            policy_violations: 0,
            sensitive_data_accesses: 0,
            total_operations: 0,
        }
    }

    /// Record a compliance event
    pub fn record_event(&mut self, event: &ComplianceEvent) {
        match event {
            ComplianceEvent::DecisionMade { .. } => {
                self.decisions_made += 1;
                self.total_operations += 1;
            }
            ComplianceEvent::SensitiveDataAccess { .. } => {
                self.sensitive_data_accesses += 1;
                self.total_operations += 1;
            }
            ComplianceEvent::PolicyViolation { .. } => {
                self.policy_violations += 1;
            }
            ComplianceEvent::HumanOversight { .. } => {
                self.human_reviews += 1;
            }
            ComplianceEvent::AgentExited { operations_count, .. } => {
                self.exit_time = Some(get_timestamp_us());
                self.total_operations = *operations_count;
            }
            _ => {}
        }
    }

    /// Check if agent is compliant with EU AI Act
    pub fn is_compliant(&self) -> bool {
        match self.risk_level {
            RiskLevel::Unacceptable => false, // Should never be allowed
            RiskLevel::High => {
                // High-risk systems require human oversight
                if self.decisions_made > 0 && self.human_reviews == 0 {
                    return false;
                }
                // Must have no unresolved policy violations
                self.policy_violations == 0
            }
            RiskLevel::Limited => {
                // Limited risk requires transparency
                self.policy_violations < 3 // Allow some minor violations
            }
            RiskLevel::Minimal => {
                // Minimal risk has few requirements
                self.policy_violations < 10
            }
        }
    }

    /// Calculate compliance score (0.0 - 1.0)
    pub fn compliance_score(&self) -> f32 {
        let mut score = 1.0;

        // Penalty for policy violations
        score -= (self.policy_violations as f32) * 0.1;

        // Bonus for human oversight on high-risk systems
        if self.risk_level >= RiskLevel::High && self.human_reviews > 0 {
            score += 0.1;
        }

        // Cap between 0.0 and 1.0
        score.max(0.0).min(1.0)
    }
}

/// Compliance tracker aggregates events across all agents
#[derive(Debug)]
pub struct ComplianceTracker {
    /// Per-agent compliance records
    records: BTreeMap<AgentId, AgentComplianceRecord>,

    /// Recent events (ring buffer)
    events: Vec<(u64, ComplianceEvent)>, // (timestamp, event)
    event_capacity: usize,
    event_head: usize,

    /// System-wide statistics
    total_events: u64,
    total_violations: u64,
    total_high_risk_agents: u64,
}

impl ComplianceTracker {
    const DEFAULT_EVENT_CAPACITY: usize = 1024;

    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
            events: Vec::with_capacity(Self::DEFAULT_EVENT_CAPACITY),
            event_capacity: Self::DEFAULT_EVENT_CAPACITY,
            event_head: 0,
            total_events: 0,
            total_violations: 0,
            total_high_risk_agents: 0,
        }
    }

    /// Register a new agent for compliance tracking
    pub fn register_agent(&mut self, agent_id: AgentId, risk_level: RiskLevel, purpose: String) {
        let record = AgentComplianceRecord::new(agent_id, risk_level, purpose.clone());
        self.records.insert(agent_id, record);

        if risk_level >= RiskLevel::High {
            self.total_high_risk_agents += 1;
        }

        // Log spawn event
        let event = ComplianceEvent::AgentSpawned {
            agent_id,
            risk_level,
            purpose,
        };
        self.log_event(event);
    }

    /// Log a compliance event
    pub fn log_event(&mut self, event: ComplianceEvent) {
        let timestamp = get_timestamp_us();
        let agent_id = event.agent_id();

        // Update per-agent record
        if let Some(record) = self.records.get_mut(&agent_id) {
            record.record_event(&event);

            // Track violations
            if matches!(event, ComplianceEvent::PolicyViolation { .. }) {
                self.total_violations += 1;
            }
        }

        // Add to ring buffer
        if self.events.len() < self.event_capacity {
            self.events.push((timestamp, event));
        } else {
            self.events[self.event_head] = (timestamp, event);
            self.event_head = (self.event_head + 1) % self.event_capacity;
        }

        self.total_events += 1;
    }

    /// Remove agent from tracking
    pub fn unregister_agent(&mut self, agent_id: AgentId) {
        if let Some(record) = self.records.get(&agent_id) {
            if record.risk_level >= RiskLevel::High {
                self.total_high_risk_agents = self.total_high_risk_agents.saturating_sub(1);
            }
        }
        self.records.remove(&agent_id);
    }

    /// Get compliance record for an agent
    pub fn get_record(&self, agent_id: AgentId) -> Option<&AgentComplianceRecord> {
        self.records.get(&agent_id)
    }

    /// Get all compliance records
    pub fn all_records(&self) -> &BTreeMap<AgentId, AgentComplianceRecord> {
        &self.records
    }

    /// Get recent events
    pub fn recent_events(&self, count: usize) -> Vec<(u64, ComplianceEvent)> {
        let count = count.min(self.events.len());
        let mut events = Vec::with_capacity(count);

        if self.events.len() < self.event_capacity {
            // Buffer not full yet, take from end
            let start = self.events.len().saturating_sub(count);
            for i in start..self.events.len() {
                events.push(self.events[i].clone());
            }
        } else {
            // Ring buffer is full, take most recent
            for i in 0..count {
                let idx = (self.event_head + self.events.len() - count + i) % self.events.len();
                events.push(self.events[idx].clone());
            }
        }

        events
    }

    /// Generate EU AI Act compliance report
    pub fn generate_report(&self) -> ComplianceReport {
        let mut compliant_agents = 0;
        let mut non_compliant_agents = 0;

        for record in self.records.values() {
            if record.is_compliant() {
                compliant_agents += 1;
            } else {
                non_compliant_agents += 1;
            }
        }

        ComplianceReport {
            generated_at: get_timestamp_us(),
            total_agents: self.records.len(),
            compliant_agents,
            non_compliant_agents,
            total_events: self.total_events,
            total_violations: self.total_violations,
            high_risk_agents: self.total_high_risk_agents,
            agent_records: self.records.values().cloned().collect(),
        }
    }

    /// Get system compliance score (0.0 - 1.0)
    pub fn system_compliance_score(&self) -> f32 {
        if self.records.is_empty() {
            return 1.0;
        }

        let total_score: f32 = self.records.values()
            .map(|r| r.compliance_score())
            .sum();

        total_score / self.records.len() as f32
    }
}

/// EU AI Act compliance report
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplianceReport {
    pub generated_at: u64,
    pub total_agents: usize,
    pub compliant_agents: usize,
    pub non_compliant_agents: usize,
    pub total_events: u64,
    pub total_violations: u64,
    pub high_risk_agents: u64,
    pub agent_records: Vec<AgentComplianceRecord>,
}

impl ComplianceReport {
    /// Export as human-readable text
    pub fn to_text(&self) -> String {
        let mut output = String::from("EU AI Act Compliance Report\n");
        output.push_str("===========================\n\n");

        output.push_str(&alloc::format!("Generated: {} μs since boot\n", self.generated_at));
        output.push_str(&alloc::format!("Total Agents: {}\n", self.total_agents));
        output.push_str(&alloc::format!("Compliant: {}\n", self.compliant_agents));
        output.push_str(&alloc::format!("Non-Compliant: {}\n", self.non_compliant_agents));
        output.push_str(&alloc::format!("Total Events: {}\n", self.total_events));
        output.push_str(&alloc::format!("Total Violations: {}\n", self.total_violations));
        output.push_str(&alloc::format!("High-Risk Agents: {}\n\n", self.high_risk_agents));

        if !self.agent_records.is_empty() {
            output.push_str("Agent Details:\n");
            output.push_str("ID    Risk      Decisions  Violations  Score\n");
            output.push_str("----  --------  ---------  ----------  -----\n");

            for record in &self.agent_records {
                output.push_str(&alloc::format!(
                    "{:<4}  {:<8}  {:<9}  {:<10}  {:.1}%\n",
                    record.agent_id,
                    record.risk_level.as_str(),
                    record.decisions_made,
                    record.policy_violations,
                    record.compliance_score() * 100.0
                ));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_tracker_creation() {
        let tracker = ComplianceTracker::new();
        assert_eq!(tracker.records.len(), 0);
        assert_eq!(tracker.total_events, 0);
    }

    #[test]
    fn test_register_agent() {
        let mut tracker = ComplianceTracker::new();
        tracker.register_agent(100, RiskLevel::High, "Test Agent".to_string());

        assert_eq!(tracker.records.len(), 1);
        assert_eq!(tracker.total_high_risk_agents, 1);

        let record = tracker.get_record(100).unwrap();
        assert_eq!(record.agent_id, 100);
        assert_eq!(record.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_log_event() {
        let mut tracker = ComplianceTracker::new();
        tracker.register_agent(100, RiskLevel::High, "Test".to_string());

        tracker.log_event(ComplianceEvent::DecisionMade {
            agent_id: 100,
            decision_type: "classification".to_string(),
            confidence: 0.95,
            human_reviewed: true,
        });

        let record = tracker.get_record(100).unwrap();
        assert_eq!(record.decisions_made, 1);
        assert_eq!(tracker.total_events, 2); // spawn + decision
    }

    #[test]
    fn test_compliance_score() {
        let mut record = AgentComplianceRecord::new(100, RiskLevel::High, "Test".to_string());
        assert_eq!(record.compliance_score(), 1.0);

        // Add violations
        record.policy_violations = 2;
        assert!(record.compliance_score() < 1.0);
    }

    #[test]
    fn test_high_risk_compliance() {
        let mut record = AgentComplianceRecord::new(100, RiskLevel::High, "Test".to_string());
        record.decisions_made = 5;

        // High-risk without human oversight should not be compliant
        assert!(!record.is_compliant());

        // Add human oversight
        record.human_reviews = 1;
        assert!(record.is_compliant());
    }

    #[test]
    fn test_generate_report() {
        let mut tracker = ComplianceTracker::new();
        tracker.register_agent(100, RiskLevel::High, "Agent1".to_string());
        tracker.register_agent(101, RiskLevel::Minimal, "Agent2".to_string());

        let report = tracker.generate_report();
        assert_eq!(report.total_agents, 2);
        assert_eq!(report.high_risk_agents, 1);
    }
}
