//! Decision Trace Data Structures
//!
//! Captures comprehensive context for every autonomous decision:
//! - Input telemetry and system state
//! - Model predictions and confidence
//! - Policy checks and alternatives
//! - Execution outcome

use alloc::vec::Vec;
use alloc::string::String;
use serde::{Serialize, Deserialize};

/// Complete decision trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTrace {
    // Context
    pub trace_id: u64,                   // Unique trace ID
    pub timestamp_us: u64,               // UNIX microseconds
    pub model_version: String,           // "v1.2.3"
    pub model_hash: [u8; 32],           // SHA-256 of model

    // Inputs
    pub telemetry: Telemetry,
    pub features: Vec<f32>,              // Extracted features
    pub system_state: SystemState,

    // Processing
    pub hidden_activations: Vec<Vec<f32>>,  // NN layer outputs
    pub policy_checks: Vec<PolicyCheck>,     // Safety checks

    // Outputs
    pub predictions: Vec<f32>,           // All output neurons
    pub chosen_action: usize,            // Index of chosen action
    pub confidence: u32,                 // 0-1000 (0-100.0%)
    pub alternatives: Vec<Alternative>,  // Top 3 alternatives

    // Outcome
    pub was_executed: bool,
    pub was_overridden: bool,
    pub override_reason: Option<String>,
}

/// System telemetry at decision time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    pub mem_pressure: u32,
    pub deadline_misses: u32,
    pub cpu_usage: u32,
    pub network_latency: u32,
}

/// System state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub uptime_ms: u64,
    pub heap_used: usize,
    pub processes_running: usize,
}

/// Policy safety check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheck {
    pub check_name: String,
    pub passed: bool,
    pub value: f32,
    pub threshold: f32,
}

/// Alternative action considered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub action_idx: usize,
    pub confidence: u32,
}

impl DecisionTrace {
    /// Create new decision trace with minimal fields
    pub fn new(trace_id: u64) -> Self {
        Self {
            trace_id,
            timestamp_us: crate::time::get_uptime_ms() * 1000,
            model_version: String::from("unknown"),
            model_hash: [0u8; 32],
            telemetry: Telemetry {
                mem_pressure: 0,
                deadline_misses: 0,
                cpu_usage: 0,
                network_latency: 0,
            },
            features: Vec::new(),
            system_state: SystemState {
                uptime_ms: crate::time::get_uptime_ms(),
            heap_used: 0,
            processes_running: 0,
        },
            hidden_activations: Vec::new(),
            policy_checks: Vec::new(),
            predictions: Vec::new(),
            chosen_action: 0,
            confidence: 0,
            alternatives: Vec::new(),
            was_executed: false,
            was_overridden: false,
            override_reason: None,
        }
    }

    /// Check if trace represents a successful decision
    pub fn is_successful(&self) -> bool {
        self.was_executed && !self.was_overridden
    }

    /// Check if decision had high confidence (> 80%)
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 800
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        alloc::format!(
            "Trace {} @ {}μs: model={}, action={}, conf={}/1000, executed={}",
            self.trace_id,
            self.timestamp_us,
            self.model_version,
            self.chosen_action,
            self.confidence,
            if self.was_executed { "Y" } else { "N" }
        )
    }
}

impl Default for DecisionTrace {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_trace_new() {
        let trace = DecisionTrace::new(123);
        assert_eq!(trace.trace_id, 123);
        assert!(!trace.is_successful());
    }

    #[test]
    fn test_high_confidence() {
        let mut trace = DecisionTrace::new(1);
        trace.confidence = 900;
        assert!(trace.is_high_confidence());

        trace.confidence = 500;
        assert!(!trace.is_high_confidence());
    }
}
