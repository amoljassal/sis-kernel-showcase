//! Shadow vs Production Comparison Logic
//!
//! Compares shadow and production predictions to detect divergence.

use alloc::string::String;

/// Result of comparing shadow vs production predictions
#[derive(Debug)]
pub enum ComparisonResult {
    /// Predictions compared successfully
    Ok {
        diverged: bool,
        confidence_delta: u32,
        action_matches: bool,
    },
    /// Divergence threshold exceeded, rollback recommended
    Rollback,
}

/// Comparison metrics
#[derive(Debug, Clone, Copy)]
pub struct ComparisonMetrics {
    pub total_comparisons: u64,
    pub divergences: u64,
    pub action_mismatches: u64,
    pub high_confidence_delta: u64,
}

impl ComparisonMetrics {
    pub const fn new() -> Self {
        Self {
            total_comparisons: 0,
            divergences: 0,
            action_mismatches: 0,
            high_confidence_delta: 0,
        }
    }

    pub fn divergence_rate(&self) -> f32 {
        if self.total_comparisons == 0 {
            return 0.0;
        }
        (self.divergences as f32 / self.total_comparisons as f32) * 100.0
    }

    pub fn action_mismatch_rate(&self) -> f32 {
        if self.total_comparisons == 0 {
            return 0.0;
        }
        (self.action_mismatches as f32 / self.total_comparisons as f32) * 100.0
    }
}

impl Default for ComparisonMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Comparison configuration
pub struct ComparisonConfig {
    pub confidence_delta_threshold: u32,  // Delta above which is considered divergence
    pub log_all_divergences: bool,
    pub log_action_mismatches: bool,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            confidence_delta_threshold: 200,  // 20%
            log_all_divergences: true,
            log_action_mismatches: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_metrics() {
        let mut metrics = ComparisonMetrics::new();
        metrics.total_comparisons = 100;
        metrics.divergences = 10;

        assert_eq!(metrics.divergence_rate(), 10.0);
    }
}
