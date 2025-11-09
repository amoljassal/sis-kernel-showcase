//! # Conflict Resolution Engine
//!
//! Priority-based conflict resolution when AI agents disagree on actions.
//!
//! ## Overview
//!
//! When multiple AI agents make conflicting recommendations, this module
//! resolves them using a priority table, synthesis strategies, or human escalation.
//!
//! ## Example
//!
//! ```text
//! Crash Predictor (priority=100, confidence=0.87):
//!   Action: Trigger compaction NOW
//!
//! Transformer Scheduler (priority=60, confidence=0.92):
//!   Action: Increase task priority (conflicts with compaction)
//!
//! Resolution: SafetyOverride
//!   - Crash predictor wins (priority 100 > 60)
//!   - Explanation: "High crash risk (87%) overrides performance optimization"
//! ```

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Type of AI agent making a decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Crash prediction (highest priority - safety)
    CrashPredictor,
    /// State inference engine (high confidence suggestions)
    StateInference,
    /// Transformer scheduler (performance optimization)
    TransformerScheduler,
    /// LLM fine-tuning (learning improvements)
    FineTuner,
    /// Metrics dashboard (monitoring only)
    Metrics,
}

impl AgentType {
    /// Get the base priority for this agent type
    pub fn base_priority(self) -> u8 {
        match self {
            AgentType::CrashPredictor => 100,    // Safety always wins
            AgentType::StateInference => 80,     // High-confidence suggestions
            AgentType::TransformerScheduler => 60, // Performance optimization
            AgentType::FineTuner => 40,          // Learning improvements
            AgentType::Metrics => 20,            // Monitoring (lowest priority)
        }
    }

    /// Get human-readable name
    pub fn name(self) -> &'static str {
        match self {
            AgentType::CrashPredictor => "Crash Predictor",
            AgentType::StateInference => "State Inference",
            AgentType::TransformerScheduler => "Transformer Scheduler",
            AgentType::FineTuner => "Fine-Tuner",
            AgentType::Metrics => "Metrics",
        }
    }
}

/// Action that an agent wants to take
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Trigger preventive memory compaction
    PreventiveCompaction,
    /// Increase task priority
    IncreasePriority,
    /// Compact memory
    CompactMemory,
    /// Continue normal operation
    ContinueNormal,
    /// Stop operation (safety)
    Stop,
    /// Trigger retraining
    TriggerRetraining,
    /// No action recommended
    NoAction,
}

impl Action {
    /// Check if two actions are compatible
    pub fn compatible_with(&self, other: &Action) -> bool {
        match (self, other) {
            // Same actions are compatible
            (a, b) if a == b => true,
            // NoAction is compatible with everything
            (Action::NoAction, _) | (_, Action::NoAction) => true,
            // ContinueNormal is compatible with non-disruptive actions
            (Action::ContinueNormal, Action::IncreasePriority) => true,
            (Action::IncreasePriority, Action::ContinueNormal) => true,
            // Compaction actions conflict with priority increases
            (Action::PreventiveCompaction, Action::IncreasePriority) => false,
            (Action::IncreasePriority, Action::PreventiveCompaction) => false,
            (Action::CompactMemory, Action::IncreasePriority) => false,
            (Action::IncreasePriority, Action::CompactMemory) => false,
            // Stop conflicts with most things
            (Action::Stop, Action::ContinueNormal) => false,
            (Action::ContinueNormal, Action::Stop) => false,
            // Default: compatible
            _ => true,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Action::PreventiveCompaction => "Trigger preventive compaction",
            Action::IncreasePriority => "Increase task priority",
            Action::CompactMemory => "Compact memory",
            Action::ContinueNormal => "Continue normal operation",
            Action::Stop => "Stop operation (safety)",
            Action::TriggerRetraining => "Trigger model retraining",
            Action::NoAction => "No action",
        }
    }
}

/// Decision made by an AI agent
#[derive(Debug, Clone)]
pub struct AgentDecision {
    /// Which agent made this decision
    pub agent: AgentType,
    /// What action the agent recommends
    pub action: Action,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Optional explanation
    pub explanation: Option<String>,
}

impl AgentDecision {
    /// Create a new agent decision
    pub fn new(agent: AgentType, action: Action, confidence: f32) -> Self {
        Self {
            agent,
            action,
            confidence,
            explanation: None,
        }
    }

    /// Add an explanation to the decision
    pub fn with_explanation(mut self, explanation: String) -> Self {
        self.explanation = Some(explanation);
        self
    }

    /// Get the effective priority (base priority * confidence)
    pub fn effective_priority(&self) -> f32 {
        self.agent.base_priority() as f32 * self.confidence
    }
}

/// Type of conflict between agents
#[derive(Debug, Clone)]
pub enum Conflict {
    /// Direct opposition between two agents
    DirectOpposition {
        agent_a: AgentDecision,
        agent_b: AgentDecision,
        incompatibility: String,
    },
    /// Multiple agents want the same resource
    ResourceContention {
        agents: Vec<AgentDecision>,
        contested_resource: Resource,
    },
    /// Large disparity in confidence levels
    ConfidenceDisparity {
        high_conf: AgentDecision,
        low_conf: AgentDecision,
        delta: f32,
    },
}

/// System resource that agents might contend for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Cpu,
    Memory,
    Io,
    Network,
}

/// Result of conflict resolution
#[derive(Debug, Clone)]
pub enum Resolution {
    /// Resolved by priority (higher priority agent wins)
    ByPriority {
        winner: AgentDecision,
        losers: Vec<AgentDecision>,
        explanation: String,
    },
    /// Resolved by synthesizing compatible actions
    BySynthesis {
        combined_action: Action,
        contributors: Vec<AgentType>,
        explanation: String,
    },
    /// Escalated to human for decision
    EscalatedToHuman {
        conflicting_decisions: Vec<AgentDecision>,
        reason: String,
    },
}

/// Conflict resolution engine
pub struct ConflictResolver {
    /// Total conflicts resolved
    conflicts_resolved: AtomicU64,
    /// Conflicts resolved by priority
    resolved_by_priority: AtomicU64,
    /// Conflicts resolved by synthesis
    resolved_by_synthesis: AtomicU64,
    /// Conflicts escalated to human
    escalated_to_human: AtomicU64,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub const fn new() -> Self {
        Self {
            conflicts_resolved: AtomicU64::new(0),
            resolved_by_priority: AtomicU64::new(0),
            resolved_by_synthesis: AtomicU64::new(0),
            escalated_to_human: AtomicU64::new(0),
        }
    }

    /// Detect conflicts between multiple agent decisions
    pub fn detect_conflicts(&self, decisions: &[AgentDecision]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        // Check for direct oppositions
        for i in 0..decisions.len() {
            for j in (i + 1)..decisions.len() {
                let a = &decisions[i];
                let b = &decisions[j];

                if !a.action.compatible_with(&b.action) {
                    conflicts.push(Conflict::DirectOpposition {
                        agent_a: a.clone(),
                        agent_b: b.clone(),
                        incompatibility: alloc::format!(
                            "{} vs {}",
                            a.action.description(),
                            b.action.description()
                        ),
                    });
                }

                // Check for confidence disparity
                let delta = (a.confidence - b.confidence).abs();
                if delta > 0.4 {
                    let (high, low) = if a.confidence > b.confidence {
                        (a.clone(), b.clone())
                    } else {
                        (b.clone(), a.clone())
                    };

                    conflicts.push(Conflict::ConfidenceDisparity {
                        high_conf: high,
                        low_conf: low,
                        delta,
                    });
                }
            }
        }

        conflicts
    }

    /// Resolve a conflict using priority table
    pub fn resolve_by_priority(&self, conflict: &Conflict) -> Resolution {
        self.conflicts_resolved.fetch_add(1, Ordering::Relaxed);
        self.resolved_by_priority.fetch_add(1, Ordering::Relaxed);

        match conflict {
            Conflict::DirectOpposition { agent_a, agent_b, incompatibility } => {
                let priority_a = agent_a.effective_priority();
                let priority_b = agent_b.effective_priority();

                let (winner, loser) = if priority_a > priority_b {
                    (agent_a.clone(), agent_b.clone())
                } else {
                    (agent_b.clone(), agent_a.clone())
                };

                Resolution::ByPriority {
                    winner: winner.clone(),
                    losers: alloc::vec![loser.clone()],
                    explanation: alloc::format!(
                        "{} (priority {:.1}) overrides {} (priority {:.1}): {}",
                        winner.agent.name(),
                        winner.effective_priority(),
                        loser.agent.name(),
                        loser.effective_priority(),
                        incompatibility
                    ),
                }
            }
            Conflict::ConfidenceDisparity { high_conf, low_conf, delta } => {
                Resolution::ByPriority {
                    winner: high_conf.clone(),
                    losers: alloc::vec![low_conf.clone()],
                    explanation: alloc::format!(
                        "{} has much higher confidence ({:.1}% vs {:.1}%, delta={:.1}%)",
                        high_conf.agent.name(),
                        high_conf.confidence * 100.0,
                        low_conf.confidence * 100.0,
                        delta * 100.0
                    ),
                }
            }
            Conflict::ResourceContention { agents, contested_resource } => {
                // Find highest priority agent
                let winner = agents
                    .iter()
                    .max_by(|a, b| {
                        a.effective_priority()
                            .partial_cmp(&b.effective_priority())
                            .unwrap_or(core::cmp::Ordering::Equal)
                    })
                    .cloned()
                    .unwrap();

                let losers: Vec<_> = agents
                    .iter()
                    .filter(|a| a.agent != winner.agent)
                    .cloned()
                    .collect();

                Resolution::ByPriority {
                    winner: winner.clone(),
                    losers,
                    explanation: alloc::format!(
                        "{} wins contention for {:?} resource",
                        winner.agent.name(),
                        contested_resource
                    ),
                }
            }
        }
    }

    /// Attempt to resolve by synthesizing compatible actions
    pub fn resolve_by_synthesis(&self, decisions: &[AgentDecision]) -> Option<Resolution> {
        self.conflicts_resolved.fetch_add(1, Ordering::Relaxed);
        self.resolved_by_synthesis.fetch_add(1, Ordering::Relaxed);

        // Check if all actions are compatible
        let all_compatible = decisions.iter().enumerate().all(|(i, a)| {
            decisions.iter().enumerate().all(|(j, b)| {
                i == j || a.action.compatible_with(&b.action)
            })
        });

        if !all_compatible {
            return None;
        }

        // If all decisions agree, synthesize them
        let contributors: Vec<_> = decisions.iter().map(|d| d.agent).collect();

        // Choose the most confident action
        let best = decisions
            .iter()
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })?;

        Some(Resolution::BySynthesis {
            combined_action: best.action.clone(),
            contributors,
            explanation: alloc::format!(
                "All agents agree on compatible actions, using highest confidence recommendation"
            ),
        })
    }

    /// Escalate severe conflict to human
    pub fn escalate_to_human(&self, decisions: Vec<AgentDecision>, reason: String) -> Resolution {
        self.conflicts_resolved.fetch_add(1, Ordering::Relaxed);
        self.escalated_to_human.fetch_add(1, Ordering::Relaxed);

        Resolution::EscalatedToHuman {
            conflicting_decisions: decisions,
            reason,
        }
    }

    /// Get conflict resolution statistics
    pub fn get_stats(&self) -> ConflictStats {
        ConflictStats {
            total_conflicts: self.conflicts_resolved.load(Ordering::Relaxed),
            resolved_by_priority: self.resolved_by_priority.load(Ordering::Relaxed),
            resolved_by_synthesis: self.resolved_by_synthesis.load(Ordering::Relaxed),
            escalated_to_human: self.escalated_to_human.load(Ordering::Relaxed),
        }
    }
}

/// Statistics about conflict resolution
#[derive(Debug, Clone, Copy)]
pub struct ConflictStats {
    pub total_conflicts: u64,
    pub resolved_by_priority: u64,
    pub resolved_by_synthesis: u64,
    pub escalated_to_human: u64,
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert_eq!(AgentType::CrashPredictor.base_priority(), 100);
        assert_eq!(AgentType::StateInference.base_priority(), 80);
        assert_eq!(AgentType::TransformerScheduler.base_priority(), 60);
    }

    #[test]
    fn test_action_compatibility() {
        assert!(Action::ContinueNormal.compatible_with(&Action::IncreasePriority));
        assert!(!Action::PreventiveCompaction.compatible_with(&Action::IncreasePriority));
        assert!(Action::NoAction.compatible_with(&Action::Stop));
    }

    #[test]
    fn test_conflict_detection() {
        let resolver = ConflictResolver::new();

        let decisions = alloc::vec![
            AgentDecision::new(AgentType::CrashPredictor, Action::PreventiveCompaction, 0.9),
            AgentDecision::new(AgentType::TransformerScheduler, Action::IncreasePriority, 0.8),
        ];

        let conflicts = resolver.detect_conflicts(&decisions);
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_priority_resolution() {
        let resolver = ConflictResolver::new();

        let conflict = Conflict::DirectOpposition {
            agent_a: AgentDecision::new(AgentType::CrashPredictor, Action::Stop, 0.9),
            agent_b: AgentDecision::new(AgentType::TransformerScheduler, Action::ContinueNormal, 0.8),
            incompatibility: "Safety vs Performance".to_string(),
        };

        let resolution = resolver.resolve_by_priority(&conflict);

        match resolution {
            Resolution::ByPriority { winner, .. } => {
                assert_eq!(winner.agent, AgentType::CrashPredictor);
            }
            _ => panic!("Expected ByPriority resolution"),
        }
    }
}
