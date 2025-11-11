//! Phase 5: User Experience Safety Tests
//!
//! Complete validation of UX safety including safety controls,
//! explainability features, and user feedback mechanisms.

pub mod safety_controls;
pub mod explainability;
pub mod user_feedback;

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Phase 5 test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase5Results {
    pub passed: bool,
    pub safety_controls_passed: bool,
    pub explainability_passed: bool,
    pub user_feedback_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub overall_score: f64,
    pub timestamp: String,
}

/// Phase 5 UX safety test suite
pub struct Phase5UXSafetySuite {
    #[allow(dead_code)]
    kernel_interface: KernelCommandInterface,
    safety_controls: safety_controls::SafetyControlsTests,
    explainability: explainability::ExplainabilityTests,
    user_feedback: user_feedback::UserFeedbackTests,
}

impl Phase5UXSafetySuite {
    /// Create a new Phase 5 test suite
    pub fn new(serial_log_path: String, monitor_port: u16) -> Self {
        Self {
            kernel_interface: KernelCommandInterface::new(serial_log_path.clone(), monitor_port),
            safety_controls: safety_controls::SafetyControlsTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            explainability: explainability::ExplainabilityTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            user_feedback: user_feedback::UserFeedbackTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
        }
    }

    /// Run all Phase 5 tests
    pub async fn validate_phase5(&mut self) -> Result<Phase5Results, Box<dyn Error>> {
        log::info!("=================================================");
        log::info!("Starting Phase 5: User Experience Safety");
        log::info!("=================================================");

        // Run all test modules
        let safety_result = self.safety_controls.run_all_tests().await?;
        let explainability_result = self.explainability.run_all_tests().await?;
        let feedback_result = self.user_feedback.run_all_tests().await?;

        // Calculate overall results
        let safety_controls_passed = safety_result.passed;
        let explainability_passed = explainability_result.passed;
        let user_feedback_passed = feedback_result.passed;

        let total_tests = safety_result.total_tests +
                         explainability_result.total_tests +
                         feedback_result.total_tests;

        let passed_tests = safety_result.passed_tests +
                          explainability_result.passed_tests +
                          feedback_result.passed_tests;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;
        let overall_score = (passed_tests as f64 / total_tests as f64) * 100.0;

        log::info!("=================================================");
        log::info!("Phase 5 Summary:");
        log::info!("  Safety Controls:      {}", if safety_controls_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Explainability:       {}", if explainability_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  User Feedback:        {}", if user_feedback_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Overall:              {}/{} tests passed ({:.1}%)",
            passed_tests, total_tests, overall_score);
        log::info!("=================================================");

        Ok(Phase5Results {
            passed,
            safety_controls_passed,
            explainability_passed,
            user_feedback_passed,
            total_tests,
            passed_tests,
            overall_score,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

impl Default for Phase5Results {
    fn default() -> Self {
        Self {
            passed: false,
            safety_controls_passed: false,
            explainability_passed: false,
            user_feedback_passed: false,
            total_tests: 0,
            passed_tests: 0,
            overall_score: 0.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase5_results_default() {
        let results = Phase5Results::default();
        assert_eq!(results.overall_score, 0.0);
        assert!(!results.passed);
    }
}
