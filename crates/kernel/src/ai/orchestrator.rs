//! # Multi-Agent Orchestrator
//!
//! Central coordinator that queries all Phase 1 AI components, aggregates their
//! recommendations, and resolves conflicts via priority table.
//!
//! ## Architecture
//!
//! ```text
//! System Event
//!     ↓
//! Orchestrator.coordinate()
//!     ├→ Query Crash Predictor
//!     ├→ Query State Inference
//!     ├→ Query Transformer Scheduler
//!     └→ Aggregate + Resolve Conflicts
//!         ↓
//! Coordinated Decision (with audit trail)
//! ```
//!
//! ## Example
//!
//! ```ignore
//! let decision = orchestrator.coordinate(&system_state)?;
//!
//! match decision {
//!     CoordinatedDecision::Unanimous { action, .. } => {
//!         // All agents agree - safe to proceed
//!     }
//!     CoordinatedDecision::SafetyOverride { action, .. } => {
//!         // Safety agent overrode others - critical
//!     }
//!     _ => {}
//! }
//! ```

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::conflict::{AgentDecision, AgentType, Action, ConflictResolver, Resolution};

/// Ring buffer for decision history
const DECISION_HISTORY_SIZE: usize = 1000;

/// Result of coordination between agents
#[derive(Debug, Clone)]
pub enum CoordinatedDecision {
    /// All agents unanimously agree
    Unanimous {
        action: Action,
        confidence: f32,
        agents: Vec<AgentType>,
    },
    /// Majority of agents agree
    Majority {
        action: Action,
        agree: Vec<AgentType>,
        disagree: Vec<AgentType>,
    },
    /// Safety agent overrode others
    SafetyOverride {
        action: Action,
        overridden_by: AgentType,
        reason: String,
        overridden_agents: Vec<AgentType>,
    },
    /// No consensus reached
    NoConsensus {
        defer_to_human: bool,
        conflicting_actions: Vec<(AgentType, Action)>,
    },
}

impl CoordinatedDecision {
    /// Get the action to execute (if any)
    pub fn action(&self) -> Option<&Action> {
        match self {
            CoordinatedDecision::Unanimous { action, .. } => Some(action),
            CoordinatedDecision::Majority { action, .. } => Some(action),
            CoordinatedDecision::SafetyOverride { action, .. } => Some(action),
            CoordinatedDecision::NoConsensus { .. } => None,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            CoordinatedDecision::Unanimous { action, confidence, agents } => {
                alloc::format!(
                    "Unanimous: {} (confidence: {:.1}%, {} agents)",
                    action.description(),
                    confidence * 100.0,
                    agents.len()
                )
            }
            CoordinatedDecision::Majority { action, agree, disagree } => {
                alloc::format!(
                    "Majority: {} ({} agree, {} disagree)",
                    action.description(),
                    agree.len(),
                    disagree.len()
                )
            }
            CoordinatedDecision::SafetyOverride { action, overridden_by, reason, .. } => {
                alloc::format!(
                    "Safety Override: {} by {} - {}",
                    action.description(),
                    overridden_by.name(),
                    reason
                )
            }
            CoordinatedDecision::NoConsensus { defer_to_human, conflicting_actions } => {
                if *defer_to_human {
                    alloc::format!(
                        "No Consensus: Deferred to human ({} conflicting actions)",
                        conflicting_actions.len()
                    )
                } else {
                    alloc::format!(
                        "No Consensus: Default action ({} conflicts)",
                        conflicting_actions.len()
                    )
                }
            }
        }
    }
}

/// Error during orchestration
#[derive(Debug, Clone)]
pub enum OrchestrationError {
    /// No agents provided any decisions
    NoDecisions,
    /// Internal error
    InternalError(String),
}

/// Statistics about orchestration
#[derive(Debug, Clone, Copy)]
pub struct OrchestrationStats {
    /// Total decisions coordinated
    pub total_decisions: u64,
    /// Unanimous decisions
    pub unanimous: u64,
    /// Majority decisions
    pub majority: u64,
    /// Safety overrides
    pub safety_overrides: u64,
    /// No consensus
    pub no_consensus: u64,
    /// Average latency in microseconds
    pub avg_latency_us: u64,
}

/// Multi-agent orchestrator
pub struct AgentOrchestrator {
    /// Conflict resolver
    conflict_resolver: ConflictResolver,
    /// Total decisions made
    total_decisions: AtomicU64,
    /// Unanimous decisions
    unanimous: AtomicU64,
    /// Majority decisions
    majority: AtomicU64,
    /// Safety overrides
    safety_overrides: AtomicU64,
    /// No consensus
    no_consensus: AtomicU64,
    /// Total latency in microseconds
    total_latency_us: AtomicU64,
}

impl AgentOrchestrator {
    /// Create a new orchestrator
    pub const fn new() -> Self {
        Self {
            conflict_resolver: ConflictResolver::new(),
            total_decisions: AtomicU64::new(0),
            unanimous: AtomicU64::new(0),
            majority: AtomicU64::new(0),
            safety_overrides: AtomicU64::new(0),
            no_consensus: AtomicU64::new(0),
            total_latency_us: AtomicU64::new(0),
        }
    }

    /// Coordinate multiple agent decisions into a single decision
    pub fn coordinate(&self, decisions: &[AgentDecision]) -> Result<CoordinatedDecision, OrchestrationError> {
        let start = crate::time::time_ns();

        if decisions.is_empty() {
            return Err(OrchestrationError::NoDecisions);
        }

        self.total_decisions.fetch_add(1, Ordering::Relaxed);

        // Check for conflicts
        let conflicts = self.conflict_resolver.detect_conflicts(decisions);

        let result = if conflicts.is_empty() {
            // No conflicts - check for unanimity
            self.resolve_unanimous(decisions)
        } else {
            // Conflicts detected - resolve them
            self.resolve_conflicts(decisions, &conflicts)
        };

        let end = crate::time::time_ns();
        let latency_us = (end - start) / 1000;
        self.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);

        Ok(result)
    }

    /// Resolve unanimous decisions
    fn resolve_unanimous(&self, decisions: &[AgentDecision]) -> CoordinatedDecision {
        // Check if all actions are the same
        let first_action = &decisions[0].action;
        let all_same = decisions.iter().all(|d| &d.action == first_action);

        if all_same {
            self.unanimous.fetch_add(1, Ordering::Relaxed);

            let avg_confidence = decisions.iter().map(|d| d.confidence).sum::<f32>() / decisions.len() as f32;
            let agents: Vec<_> = decisions.iter().map(|d| d.agent).collect();

            CoordinatedDecision::Unanimous {
                action: first_action.clone(),
                confidence: avg_confidence,
                agents,
            }
        } else {
            // Not unanimous - find majority
            self.resolve_majority(decisions)
        }
    }

    /// Resolve majority decisions
    fn resolve_majority(&self, decisions: &[AgentDecision]) -> CoordinatedDecision {
        // Count votes for each action
        let mut action_votes: alloc::collections::BTreeMap<String, Vec<AgentType>> = alloc::collections::BTreeMap::new();

        for decision in decisions {
            let key = alloc::format!("{:?}", decision.action);
            action_votes.entry(key).or_default().push(decision.agent);
        }

        // Find action with most votes
        let (majority_action_str, majority_agents) = action_votes
            .iter()
            .max_by_key(|(_, agents)| agents.len())
            .map(|(action, agents)| (action.clone(), agents.clone()))
            .unwrap();

        // Get the actual action
        let majority_action = decisions
            .iter()
            .find(|d| alloc::format!("{:?}", d.action) == majority_action_str)
            .map(|d| d.action.clone())
            .unwrap();

        // Find dissenters
        let disagree: Vec<_> = decisions
            .iter()
            .filter(|d| !majority_agents.contains(&d.agent))
            .map(|d| d.agent)
            .collect();

        if majority_agents.len() > decisions.len() / 2 {
            self.majority.fetch_add(1, Ordering::Relaxed);

            CoordinatedDecision::Majority {
                action: majority_action,
                agree: majority_agents,
                disagree,
            }
        } else {
            self.no_consensus.fetch_add(1, Ordering::Relaxed);

            let conflicting: Vec<_> = decisions
                .iter()
                .map(|d| (d.agent, d.action.clone()))
                .collect();

            CoordinatedDecision::NoConsensus {
                defer_to_human: true,
                conflicting_actions: conflicting,
            }
        }
    }

    /// Resolve conflicts using conflict resolver
    fn resolve_conflicts(
        &self,
        decisions: &[AgentDecision],
        conflicts: &[super::conflict::Conflict],
    ) -> CoordinatedDecision {
        // Check for safety override (crash predictor with high confidence)
        if let Some(crash_decision) = decisions
            .iter()
            .find(|d| d.agent == AgentType::CrashPredictor && d.confidence > 0.8)
        {
            self.safety_overrides.fetch_add(1, Ordering::Relaxed);

            let overridden: Vec<_> = decisions
                .iter()
                .filter(|d| d.agent != AgentType::CrashPredictor)
                .map(|d| d.agent)
                .collect();

            return CoordinatedDecision::SafetyOverride {
                action: crash_decision.action.clone(),
                overridden_by: AgentType::CrashPredictor,
                reason: crash_decision
                    .explanation
                    .clone()
                    .unwrap_or_else(|| "High crash risk detected".to_string()),
                overridden_agents: overridden,
            };
        }

        // Try to resolve by priority
        if let Some(conflict) = conflicts.first() {
            match self.conflict_resolver.resolve_by_priority(conflict) {
                Resolution::ByPriority { winner, .. } => {
                    self.safety_overrides.fetch_add(1, Ordering::Relaxed);

                    let overridden: Vec<_> = decisions
                        .iter()
                        .filter(|d| d.agent != winner.agent)
                        .map(|d| d.agent)
                        .collect();

                    CoordinatedDecision::SafetyOverride {
                        action: winner.action.clone(),
                        overridden_by: winner.agent,
                        reason: winner.explanation.unwrap_or_else(|| "Priority-based resolution".to_string()),
                        overridden_agents: overridden,
                    }
                }
                _ => {
                    self.no_consensus.fetch_add(1, Ordering::Relaxed);

                    let conflicting: Vec<_> = decisions
                        .iter()
                        .map(|d| (d.agent, d.action.clone()))
                        .collect();

                    CoordinatedDecision::NoConsensus {
                        defer_to_human: true,
                        conflicting_actions: conflicting,
                    }
                }
            }
        } else {
            // No conflicts - shouldn't reach here
            self.resolve_unanimous(decisions)
        }
    }

    /// Get orchestration statistics
    pub fn get_stats(&self) -> OrchestrationStats {
        let total = self.total_decisions.load(Ordering::Relaxed);
        let total_latency = self.total_latency_us.load(Ordering::Relaxed);

        OrchestrationStats {
            total_decisions: total,
            unanimous: self.unanimous.load(Ordering::Relaxed),
            majority: self.majority.load(Ordering::Relaxed),
            safety_overrides: self.safety_overrides.load(Ordering::Relaxed),
            no_consensus: self.no_consensus.load(Ordering::Relaxed),
            avg_latency_us: if total > 0 {
                total_latency / total
            } else {
                0
            },
        }
    }

    /// Get conflict resolver stats
    pub fn get_conflict_stats(&self) -> super::conflict::ConflictStats {
        self.conflict_resolver.get_stats()
    }
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unanimous_decision() {
        let orchestrator = AgentOrchestrator::new();

        let decisions = alloc::vec![
            AgentDecision::new(AgentType::CrashPredictor, Action::CompactMemory, 0.9),
            AgentDecision::new(AgentType::StateInference, Action::CompactMemory, 0.85),
        ];

        let result = orchestrator.coordinate(&decisions).unwrap();

        match result {
            CoordinatedDecision::Unanimous { action, .. } => {
                assert_eq!(action, Action::CompactMemory);
            }
            _ => panic!("Expected unanimous decision"),
        }
    }

    #[test]
    fn test_safety_override() {
        let orchestrator = AgentOrchestrator::new();

        let decisions = alloc::vec![
            AgentDecision::new(AgentType::CrashPredictor, Action::Stop, 0.95),
            AgentDecision::new(AgentType::TransformerScheduler, Action::ContinueNormal, 0.8),
        ];

        let result = orchestrator.coordinate(&decisions).unwrap();

        match result {
            CoordinatedDecision::SafetyOverride { overridden_by, .. } => {
                assert_eq!(overridden_by, AgentType::CrashPredictor);
            }
            _ => panic!("Expected safety override"),
        }
    }

    #[test]
    fn test_majority_decision() {
        let orchestrator = AgentOrchestrator::new();

        let decisions = alloc::vec![
            AgentDecision::new(AgentType::StateInference, Action::CompactMemory, 0.8),
            AgentDecision::new(AgentType::TransformerScheduler, Action::CompactMemory, 0.7),
            AgentDecision::new(AgentType::FineTuner, Action::NoAction, 0.6),
        ];

        let result = orchestrator.coordinate(&decisions).unwrap();

        match result {
            CoordinatedDecision::Majority { action, .. } => {
                assert_eq!(action, Action::CompactMemory);
            }
            _ => panic!("Expected majority decision"),
        }
    }
}
