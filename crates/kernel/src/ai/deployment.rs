//! # Enhanced Deployment Phase Manager
//!
//! Automated phase transitions based on performance metrics with safety checks.
//!
//! ## Phase Progression
//!
//! ```text
//! Phase A (Learning)
//!   ├─ Conservative: max 5 autonomous actions/hour
//!   ├─ Criteria: 100 decisions, 90% success, 48h uptime
//!   └─ Auto-advance to Phase B ✓
//!
//! Phase B (Validation)
//!   ├─ Moderate: max 20 autonomous actions/hour
//!   ├─ Criteria: 500 decisions, 92% success, 168h uptime
//!   └─ Auto-advance to Phase C ✓
//!
//! Phase C (Production)
//!   ├─ Aggressive: max 100 autonomous actions/hour
//!   ├─ Auto-rollback on drift/accuracy drop
//!   └─ No auto-advance (manual only)
//!
//! Phase D (Emergency)
//!   ├─ Manual only: 0 autonomous actions
//!   └─ Requires human intervention to exit
//! ```

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Deployment phase identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PhaseId {
    /// Phase A: Learning (conservative)
    A_Learning = 0,
    /// Phase B: Validation (moderate)
    B_Validation = 1,
    /// Phase C: Production (aggressive)
    C_Production = 2,
    /// Phase D: Emergency (manual only)
    D_Emergency = 3,
}

impl PhaseId {
    /// Get the next phase (for advancement)
    pub fn next(self) -> Option<PhaseId> {
        match self {
            PhaseId::A_Learning => Some(PhaseId::B_Validation),
            PhaseId::B_Validation => Some(PhaseId::C_Production),
            PhaseId::C_Production => None, // No auto-advance from production
            PhaseId::D_Emergency => None,  // Manual exit only
        }
    }

    /// Get the previous phase (for rollback)
    pub fn previous(self) -> Option<PhaseId> {
        match self {
            PhaseId::A_Learning => None,
            PhaseId::B_Validation => Some(PhaseId::A_Learning),
            PhaseId::C_Production => Some(PhaseId::B_Validation),
            PhaseId::D_Emergency => None, // Manual exit only
        }
    }

    /// Get human-readable name
    pub fn name(self) -> &'static str {
        match self {
            PhaseId::A_Learning => "Phase A: Learning",
            PhaseId::B_Validation => "Phase B: Validation",
            PhaseId::C_Production => "Phase C: Production",
            PhaseId::D_Emergency => "Phase D: Emergency",
        }
    }

    /// Get short name (A, B, C, D)
    pub fn short_name(self) -> &'static str {
        match self {
            PhaseId::A_Learning => "A",
            PhaseId::B_Validation => "B",
            PhaseId::C_Production => "C",
            PhaseId::D_Emergency => "D",
        }
    }
}

/// Constraints for a deployment phase
#[derive(Debug, Clone, Copy)]
pub struct PhaseConstraints {
    /// Maximum risk level (0-100)
    pub max_risk: u8,
    /// Maximum autonomous actions per hour
    pub max_autonomous_actions_per_hour: u32,
    /// Require human approval above this confidence threshold
    pub require_human_approval_above: f32,
    /// Maximum drift tolerance before rollback
    pub max_drift_tolerance: f32,
    /// Minimum accuracy before emergency rollback
    pub min_accuracy: f32,
}

/// Criteria for advancing to next phase
#[derive(Debug, Clone, Copy)]
pub struct AdvanceCriteria {
    /// Minimum decisions in current phase
    pub min_decisions: u32,
    /// Must achieve this success rate
    pub min_success_rate: f32,
    /// Stability requirement (hours)
    pub min_uptime_hours: u32,
    /// Safety requirement (max incidents)
    pub max_incidents: u32,
}

/// Criteria for rolling back to previous phase
#[derive(Debug, Clone, Copy)]
pub struct RollbackCriteria {
    /// Accuracy drops below this
    pub critical_accuracy: f32,
    /// Drift exceeds this
    pub critical_drift: f32,
    /// Too many incidents
    pub max_incidents_per_hour: u32,
}

/// Deployment phase definition
#[derive(Debug, Clone, Copy)]
pub struct DeploymentPhase {
    pub phase_id: PhaseId,
    pub constraints: PhaseConstraints,
    pub auto_advance_criteria: AdvanceCriteria,
    pub auto_rollback_criteria: RollbackCriteria,
}

/// Phase configuration table
const PHASES: [DeploymentPhase; 4] = [
    // Phase A: Learning (conservative)
    DeploymentPhase {
        phase_id: PhaseId::A_Learning,
        constraints: PhaseConstraints {
            max_risk: 30,
            max_autonomous_actions_per_hour: 5,
            require_human_approval_above: 0.6,
            max_drift_tolerance: 0.05,
            min_accuracy: 0.85,
        },
        auto_advance_criteria: AdvanceCriteria {
            min_decisions: 100,
            min_success_rate: 0.90,
            min_uptime_hours: 48,
            max_incidents: 2,
        },
        auto_rollback_criteria: RollbackCriteria {
            critical_accuracy: 0.80,
            critical_drift: 0.10,
            max_incidents_per_hour: 3,
        },
    },
    // Phase B: Validation (moderate)
    DeploymentPhase {
        phase_id: PhaseId::B_Validation,
        constraints: PhaseConstraints {
            max_risk: 60,
            max_autonomous_actions_per_hour: 20,
            require_human_approval_above: 0.8,
            max_drift_tolerance: 0.10,
            min_accuracy: 0.80,
        },
        auto_advance_criteria: AdvanceCriteria {
            min_decisions: 500,
            min_success_rate: 0.92,
            min_uptime_hours: 168, // 1 week
            max_incidents: 5,
        },
        auto_rollback_criteria: RollbackCriteria {
            critical_accuracy: 0.75,
            critical_drift: 0.15,
            max_incidents_per_hour: 5,
        },
    },
    // Phase C: Production (aggressive)
    DeploymentPhase {
        phase_id: PhaseId::C_Production,
        constraints: PhaseConstraints {
            max_risk: 40,
            max_autonomous_actions_per_hour: 100,
            require_human_approval_above: 0.9,
            max_drift_tolerance: 0.15,
            min_accuracy: 0.75,
        },
        auto_advance_criteria: AdvanceCriteria {
            min_decisions: 0, // No auto-advance from production
            min_success_rate: 0.0,
            min_uptime_hours: 0,
            max_incidents: 0,
        },
        auto_rollback_criteria: RollbackCriteria {
            critical_accuracy: 0.70,
            critical_drift: 0.20,
            max_incidents_per_hour: 10,
        },
    },
    // Phase D: Emergency (manual only)
    DeploymentPhase {
        phase_id: PhaseId::D_Emergency,
        constraints: PhaseConstraints {
            max_risk: 10,
            max_autonomous_actions_per_hour: 0,
            require_human_approval_above: 1.0,
            max_drift_tolerance: 0.0,
            min_accuracy: 0.0,
        },
        auto_advance_criteria: AdvanceCriteria {
            min_decisions: 0,
            min_success_rate: 0.0,
            min_uptime_hours: 0,
            max_incidents: 0,
        },
        auto_rollback_criteria: RollbackCriteria {
            critical_accuracy: 0.0,
            critical_drift: 0.0,
            max_incidents_per_hour: 0,
        },
    },
];

/// Metrics for current phase
#[derive(Debug, Clone, Copy)]
pub struct PhaseMetrics {
    /// Number of decisions made in this phase
    pub decisions: u32,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f32,
    /// Uptime in hours
    pub uptime_hours: u32,
    /// Number of incidents
    pub incidents: u32,
    /// Current accuracy
    pub accuracy: f32,
    /// Current drift
    pub drift: f32,
}

/// Phase transition event
#[derive(Debug, Clone)]
pub enum PhaseTransition {
    /// Stay in current phase
    Stay {
        current_metrics: PhaseMetrics,
    },
    /// Advance to next phase
    Advance {
        from: PhaseId,
        to: PhaseId,
        reason: String,
        metrics: PhaseMetrics,
    },
    /// Rollback to previous phase
    Rollback {
        from: PhaseId,
        to: PhaseId,
        reason: String,
        metrics: PhaseMetrics,
    },
}

/// Deployment phase manager
pub struct DeploymentManager {
    /// Current phase
    current_phase: AtomicU32,
    /// Auto-advance enabled
    auto_advance_enabled: AtomicU32,
    /// Auto-rollback enabled
    auto_rollback_enabled: AtomicU32,
    /// Phase start time (nanoseconds)
    phase_start_time: AtomicU64,
    /// Decisions in current phase
    phase_decisions: AtomicU32,
    /// Successful decisions in current phase
    phase_successes: AtomicU32,
    /// Incidents in current phase
    phase_incidents: AtomicU32,
    /// Total phase transitions
    total_transitions: AtomicU64,
    /// Auto-advances
    auto_advances: AtomicU64,
    /// Auto-rollbacks
    auto_rollbacks: AtomicU64,
}

impl DeploymentManager {
    /// Create a new deployment manager
    pub const fn new() -> Self {
        Self {
            current_phase: AtomicU32::new(PhaseId::A_Learning as u32),
            auto_advance_enabled: AtomicU32::new(1), // Enabled by default
            auto_rollback_enabled: AtomicU32::new(1), // Enabled by default
            phase_start_time: AtomicU64::new(0),
            phase_decisions: AtomicU32::new(0),
            phase_successes: AtomicU32::new(0),
            phase_incidents: AtomicU32::new(0),
            total_transitions: AtomicU64::new(0),
            auto_advances: AtomicU64::new(0),
            auto_rollbacks: AtomicU64::new(0),
        }
    }

    /// Get current phase ID
    pub fn current_phase(&self) -> PhaseId {
        let phase_num = self.current_phase.load(Ordering::Relaxed);
        match phase_num {
            0 => PhaseId::A_Learning,
            1 => PhaseId::B_Validation,
            2 => PhaseId::C_Production,
            3 => PhaseId::D_Emergency,
            _ => PhaseId::A_Learning,
        }
    }

    /// Get current phase definition
    pub fn get_current_phase(&self) -> &'static DeploymentPhase {
        let phase_id = self.current_phase();
        &PHASES[phase_id as usize]
    }

    /// Set current phase (manual override)
    pub fn set_phase(&self, phase: PhaseId) {
        self.current_phase.store(phase as u32, Ordering::Relaxed);
        self.phase_start_time.store(crate::time::get_timestamp_us() * 1000, Ordering::Relaxed); // Convert μs to ns
        self.phase_decisions.store(0, Ordering::Relaxed);
        self.phase_successes.store(0, Ordering::Relaxed);
        self.phase_incidents.store(0, Ordering::Relaxed);
        self.total_transitions.fetch_add(1, Ordering::Relaxed);
    }

    /// Enable/disable auto-advance
    pub fn set_auto_advance(&self, enabled: bool) {
        self.auto_advance_enabled.store(enabled as u32, Ordering::Relaxed);
    }

    /// Enable/disable auto-rollback
    pub fn set_auto_rollback(&self, enabled: bool) {
        self.auto_rollback_enabled.store(enabled as u32, Ordering::Relaxed);
    }

    /// Record a decision
    pub fn record_decision(&self, success: bool) {
        self.phase_decisions.fetch_add(1, Ordering::Relaxed);
        if success {
            self.phase_successes.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record an incident
    pub fn record_incident(&self) {
        self.phase_incidents.fetch_add(1, Ordering::Relaxed);
    }

    /// Collect current phase metrics
    pub fn collect_phase_metrics(&self) -> PhaseMetrics {
        let decisions = self.phase_decisions.load(Ordering::Relaxed);
        let successes = self.phase_successes.load(Ordering::Relaxed);
        let incidents = self.phase_incidents.load(Ordering::Relaxed);

        let start_time = self.phase_start_time.load(Ordering::Relaxed);
        let current_time = crate::time::get_timestamp_us() * 1000; // Convert μs to ns
        let uptime_hours = if start_time > 0 {
            ((current_time - start_time) / 1_000_000_000 / 3600) as u32
        } else {
            0
        };

        let success_rate = if decisions > 0 {
            successes as f32 / decisions as f32
        } else {
            0.0
        };

        PhaseMetrics {
            decisions,
            success_rate,
            uptime_hours,
            incidents,
            accuracy: success_rate, // Simplified: use success rate as accuracy
            drift: 0.0,             // TODO: Get from drift detector
        }
    }

    /// Check if phase should auto-advance
    pub fn check_auto_advance(&self) -> PhaseTransition {
        if self.auto_advance_enabled.load(Ordering::Relaxed) == 0 {
            return PhaseTransition::Stay {
                current_metrics: self.collect_phase_metrics(),
            };
        }

        let current = self.get_current_phase();
        let metrics = self.collect_phase_metrics();

        // Check if we can advance
        if let Some(next_phase) = current.phase_id.next() {
            let criteria = &current.auto_advance_criteria;

            if metrics.decisions >= criteria.min_decisions
                && metrics.success_rate >= criteria.min_success_rate
                && metrics.uptime_hours >= criteria.min_uptime_hours
                && metrics.incidents <= criteria.max_incidents
            {
                self.auto_advances.fetch_add(1, Ordering::Relaxed);

                return PhaseTransition::Advance {
                    from: current.phase_id,
                    to: next_phase,
                    reason: "All advancement criteria met".to_string(),
                    metrics,
                };
            }
        }

        PhaseTransition::Stay {
            current_metrics: metrics,
        }
    }

    /// Check if phase should auto-rollback
    pub fn check_auto_rollback(&self) -> PhaseTransition {
        if self.auto_rollback_enabled.load(Ordering::Relaxed) == 0 {
            return PhaseTransition::Stay {
                current_metrics: self.collect_phase_metrics(),
            };
        }

        let current = self.get_current_phase();
        let metrics = self.collect_phase_metrics();

        // Check for critical accuracy
        if metrics.accuracy < current.auto_rollback_criteria.critical_accuracy {
            self.auto_rollbacks.fetch_add(1, Ordering::Relaxed);

            return PhaseTransition::Rollback {
                from: current.phase_id,
                to: PhaseId::D_Emergency,
                reason: alloc::format!(
                    "Accuracy critically low: {:.1}% < {:.1}%",
                    metrics.accuracy * 100.0,
                    current.auto_rollback_criteria.critical_accuracy * 100.0
                ),
                metrics,
            };
        }

        // Check for high drift
        if metrics.drift > current.auto_rollback_criteria.critical_drift {
            self.auto_rollbacks.fetch_add(1, Ordering::Relaxed);

            if let Some(prev_phase) = current.phase_id.previous() {
                return PhaseTransition::Rollback {
                    from: current.phase_id,
                    to: prev_phase,
                    reason: alloc::format!(
                        "Model drift exceeds tolerance: {:.1}% > {:.1}%",
                        metrics.drift * 100.0,
                        current.auto_rollback_criteria.critical_drift * 100.0
                    ),
                    metrics,
                };
            }
        }

        PhaseTransition::Stay {
            current_metrics: metrics,
        }
    }

    /// Apply a phase transition
    pub fn apply_transition(&self, transition: &PhaseTransition) -> bool {
        match transition {
            PhaseTransition::Advance { to, .. } | PhaseTransition::Rollback { to, .. } => {
                self.set_phase(*to);
                true
            }
            PhaseTransition::Stay { .. } => false,
        }
    }

    /// Get deployment statistics
    pub fn get_stats(&self) -> DeploymentStats {
        DeploymentStats {
            current_phase: self.current_phase(),
            auto_advance_enabled: self.auto_advance_enabled.load(Ordering::Relaxed) != 0,
            auto_rollback_enabled: self.auto_rollback_enabled.load(Ordering::Relaxed) != 0,
            total_transitions: self.total_transitions.load(Ordering::Relaxed),
            auto_advances: self.auto_advances.load(Ordering::Relaxed),
            auto_rollbacks: self.auto_rollbacks.load(Ordering::Relaxed),
            current_metrics: self.collect_phase_metrics(),
        }
    }
}

/// Statistics about deployment phases
#[derive(Debug, Clone, Copy)]
pub struct DeploymentStats {
    pub current_phase: PhaseId,
    pub auto_advance_enabled: bool,
    pub auto_rollback_enabled: bool,
    pub total_transitions: u64,
    pub auto_advances: u64,
    pub auto_rollbacks: u64,
    pub current_metrics: PhaseMetrics,
}

impl Default for DeploymentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_progression() {
        assert_eq!(PhaseId::A_Learning.next(), Some(PhaseId::B_Validation));
        assert_eq!(PhaseId::B_Validation.next(), Some(PhaseId::C_Production));
        assert_eq!(PhaseId::C_Production.next(), None);
    }

    #[test]
    fn test_phase_rollback() {
        assert_eq!(PhaseId::C_Production.previous(), Some(PhaseId::B_Validation));
        assert_eq!(PhaseId::B_Validation.previous(), Some(PhaseId::A_Learning));
        assert_eq!(PhaseId::A_Learning.previous(), None);
    }

    #[test]
    fn test_phase_constraints() {
        let phase_a = &PHASES[PhaseId::A_Learning as usize];
        assert_eq!(phase_a.constraints.max_autonomous_actions_per_hour, 5);

        let phase_c = &PHASES[PhaseId::C_Production as usize];
        assert_eq!(phase_c.constraints.max_autonomous_actions_per_hour, 100);
    }

    #[test]
    fn test_deployment_manager() {
        let manager = DeploymentManager::new();

        assert_eq!(manager.current_phase(), PhaseId::A_Learning);

        // Record some decisions
        for _ in 0..100 {
            manager.record_decision(true);
        }

        let metrics = manager.collect_phase_metrics();
        assert_eq!(metrics.decisions, 100);
        assert_eq!(metrics.success_rate, 1.0);
    }
}
