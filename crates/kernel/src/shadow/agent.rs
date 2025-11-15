//! Shadow Agent Implementation
//!
//! Runs alternative models in parallel with production
//! to detect divergence before full deployment.

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use spin::Mutex;
use crate::lib::error::Result;
use crate::model_lifecycle::lifecycle::Model;

/// Shadow deployment mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowMode {
    Disabled,          // No shadow agent
    LogOnly,           // Shadow runs, logs only
    Compare,           // Shadow runs, compares with production
    CanaryPartial,     // Shadow executes for 10% of decisions
    CanaryFull,        // Shadow promoted to production
}

/// Simplified prediction result
#[derive(Debug, Clone)]
pub struct Prediction {
    pub action: usize,
    pub confidence: u32,
    pub all_outputs: alloc::vec::Vec<f32>,
}

/// Shadow agent manager
pub struct ShadowAgent {
    mode: Mutex<ShadowMode>,
    model: Arc<Mutex<Option<crate::model_lifecycle::lifecycle::Model>>>,
    divergence_count: AtomicU64,
    divergence_threshold: AtomicU32,
    decision_count: AtomicU64,
    dry_run: AtomicBool,
}

impl ShadowAgent {
    /// Create new shadow agent
    pub fn new() -> Self {
        Self {
            mode: Mutex::new(ShadowMode::Disabled),
            model: Arc::new(Mutex::new(None)),
            divergence_count: AtomicU64::new(0),
            divergence_threshold: AtomicU32::new(50),  // Rollback after 50 divergences
            decision_count: AtomicU64::new(0),
            dry_run: AtomicBool::new(false),
        }
    }

    /// Enable shadow agent with model and mode
    pub fn enable(&self, model: Model, mode: ShadowMode) -> Result<()> {
        *self.model.lock() = Some(model);
        *self.mode.lock() = mode;
        self.divergence_count.store(0, Ordering::Relaxed);
        self.decision_count.store(0, Ordering::Relaxed);
        Ok(())
    }

    /// Disable shadow agent
    pub fn disable(&self) {
        *self.mode.lock() = ShadowMode::Disabled;
        *self.model.lock() = None;
    }

    /// Run shadow prediction (parallel to production)
    pub fn predict_shadow(&self, input: &[f32]) -> Option<Prediction> {
        let mode = *self.mode.lock();
        if mode == ShadowMode::Disabled {
            return None;
        }

        let model = self.model.lock();
        let model = model.as_ref()?;

        // Run shadow prediction
        let outputs = model.predict(input);
        let (action, confidence) = self.extract_action_confidence(&outputs);

        self.decision_count.fetch_add(1, Ordering::Relaxed);

        Some(Prediction {
            action,
            confidence,
            all_outputs: outputs,
        })
    }

    /// Compare shadow with production prediction
    pub fn compare(&self, prod: &Prediction, shadow: &Prediction) -> super::compare::ComparisonResult {
        use super::compare::ComparisonResult;

        let confidence_delta = (prod.confidence as i32 - shadow.confidence as i32).abs() as u32;
        let action_matches = prod.action == shadow.action;

        let diverged = !action_matches || confidence_delta > 200;  // 20% confidence delta

        if diverged {
            // Log divergence event (even in dry-run)
            #[cfg(feature = "shadow-mode")]
            {
                crate::shadow::divergence::log_event(confidence_delta, action_matches, self.get_mode());
            }

            if !self.dry_run.load(Ordering::Relaxed) {
                let div_count = self.divergence_count.fetch_add(1, Ordering::Relaxed) + 1;

                // Check if we should rollback
                if div_count >= self.divergence_threshold.load(Ordering::Relaxed) as u64 {
                    return ComparisonResult::Rollback;
                }
            }
        }

        ComparisonResult::Ok {
            diverged,
            confidence_delta,
            action_matches,
        }
    }

    /// Get shadow agent statistics
    pub fn get_stats(&self) -> ShadowStats {
        ShadowStats {
            mode: *self.mode.lock(),
            decision_count: self.decision_count.load(Ordering::Relaxed),
            divergence_count: self.divergence_count.load(Ordering::Relaxed),
            divergence_rate: self.get_divergence_rate(),
        }
    }

    /// Set divergence threshold
    pub fn set_threshold(&self, threshold: u32) {
        self.divergence_threshold.store(threshold, Ordering::Relaxed);
    }

    /// Set shadow mode
    pub fn set_mode(&self, mode: ShadowMode) {
        *self.mode.lock() = mode;
    }

    /// Get current mode
    pub fn get_mode(&self) -> ShadowMode {
        *self.mode.lock()
    }

    /// Enable/disable dry-run (no counters/rollback)
    pub fn set_dry_run(&self, on: bool) { self.dry_run.store(on, Ordering::Relaxed); }
    pub fn is_dry_run(&self) -> bool { self.dry_run.load(Ordering::Relaxed) }

    // Private helpers

    fn extract_action_confidence(&self, outputs: &[f32]) -> (usize, u32) {
        if outputs.is_empty() {
            return (0, 0);
        }

        let (action, &max_val) = outputs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal)
            })
            .unwrap_or((0, &0.0));

        // Convert to 0-1000 scale
        let confidence = ((max_val * 1000.0) as u32).min(1000);

        (action, confidence)
    }

    fn get_divergence_rate(&self) -> f32 {
        let total = self.decision_count.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let divs = self.divergence_count.load(Ordering::Relaxed);
        (divs as f32 / total as f32) * 100.0
    }
}

/// Shadow agent statistics
#[derive(Debug, Clone)]
pub struct ShadowStats {
    pub mode: ShadowMode,
    pub decision_count: u64,
    pub divergence_count: u64,
    pub divergence_rate: f32,
}

// Global instance is provided from shadow::mod via lazy_static

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_agent_new() {
        let agent = ShadowAgent::new();
        assert_eq!(agent.get_mode(), ShadowMode::Disabled);
    }

    #[test]
    fn test_shadow_mode_transitions() {
        let agent = ShadowAgent::new();
        agent.set_mode(ShadowMode::LogOnly);
        assert_eq!(agent.get_mode(), ShadowMode::LogOnly);

        agent.set_mode(ShadowMode::Compare);
        assert_eq!(agent.get_mode(), ShadowMode::Compare);

        agent.disable();
        assert_eq!(agent.get_mode(), ShadowMode::Disabled);
    }
}
