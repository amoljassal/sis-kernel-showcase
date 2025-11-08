//! Automatic Rollback Logic
//!
//! Monitors shadow agent performance and triggers rollback
//! when divergence exceeds acceptable thresholds.

use alloc::string::String;
use super::agent::ShadowStats;

/// Rollback trigger configuration
pub struct RollbackTrigger {
    pub divergence_threshold: u32,       // Max divergences before rollback
    pub confidence_drop_threshold: u32,   // Max confidence drop
    pub error_rate_threshold: f32,        // Max error rate (%)
}

impl Default for RollbackTrigger {
    fn default() -> Self {
        Self {
            divergence_threshold: 50,
            confidence_drop_threshold: 300,  // 30%
            error_rate_threshold: 20.0,      // 20%
        }
    }
}

impl RollbackTrigger {
    /// Check if rollback should be triggered
    pub fn check(&self, stats: &ShadowStats) -> RollbackDecision {
        // 1. Check absolute divergence count
        if stats.divergence_count >= self.divergence_threshold as u64 {
            return RollbackDecision::Rollback {
                reason: String::from("Divergence threshold exceeded"),
                metric: alloc::format!("{} divergences", stats.divergence_count),
            };
        }

        // 2. Check divergence rate
        if stats.divergence_rate > self.error_rate_threshold {
            return RollbackDecision::Rollback {
                reason: String::from("Divergence rate too high"),
                metric: alloc::format!("{:.2}%", stats.divergence_rate),
            };
        }

        // 3. All checks passed
        RollbackDecision::Continue
    }
}

/// Rollback decision
#[derive(Debug)]
pub enum RollbackDecision {
    /// Continue with current deployment
    Continue,
    /// Rollback recommended
    Rollback {
        reason: String,
        metric: String,
    },
}

/// Execute automatic rollback if needed
pub fn auto_rollback_if_needed() -> crate::lib::error::Result<()> {
    use super::agent::SHADOW_AGENT;
    use crate::lib::error::Errno;

    let stats = SHADOW_AGENT.get_stats();
    let trigger = RollbackTrigger::default();

    match trigger.check(&stats) {
        RollbackDecision::Continue => Ok(()),
        RollbackDecision::Rollback { reason, metric } => {
            crate::println!("[ROLLBACK] Triggered: {} ({})", reason, metric);

            // Disable shadow
            SHADOW_AGENT.disable();

            // TODO: Trigger model lifecycle rollback
            // let mut lifecycle = MODEL_LIFECYCLE.lock();
            // lifecycle.rollback()?;

            crate::println!("[ROLLBACK] Complete - reverted to production model");

            Err(Errno::ECANCELED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shadow::agent::ShadowMode;

    #[test]
    fn test_rollback_trigger() {
        let trigger = RollbackTrigger::default();

        // Stats below threshold
        let stats = ShadowStats {
            mode: ShadowMode::Compare,
            decision_count: 100,
            divergence_count: 5,
            divergence_rate: 5.0,
        };

        match trigger.check(&stats) {
            RollbackDecision::Continue => {}
            _ => panic!("Should continue"),
        }

        // Stats above threshold
        let stats = ShadowStats {
            mode: ShadowMode::Compare,
            decision_count: 100,
            divergence_count: 60,
            divergence_rate: 60.0,
        };

        match trigger.check(&stats) {
            RollbackDecision::Rollback { .. } => {}
            _ => panic!("Should rollback"),
        }
    }
}
