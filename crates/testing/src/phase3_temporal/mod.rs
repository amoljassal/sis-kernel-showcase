//! Phase 3: Temporal Isolation Tests
//!
//! Complete validation of temporal isolation, real-time guarantees,
//! deadline validation, and latency characteristics.

pub mod active_isolation;
pub mod deadline_validation;
pub mod latency_tests;

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Phase 3 test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase3Results {
    pub passed: bool,
    pub active_isolation_passed: bool,
    pub deadline_validation_passed: bool,
    pub latency_tests_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub overall_score: f64,
    pub timestamp: String,
}

/// Phase 3 temporal isolation test suite
pub struct Phase3TemporalSuite {
    #[allow(dead_code)]
    kernel_interface: KernelCommandInterface,
    active_isolation: active_isolation::ActiveIsolationTests,
    deadline_validation: deadline_validation::DeadlineValidationTests,
    latency_tests: latency_tests::LatencyTests,
}

impl Phase3TemporalSuite {
    /// Create a new Phase 3 test suite
    pub fn new(serial_log_path: String, qemu_manager: std::sync::Arc<crate::qemu_runtime::QEMURuntimeManager>, node_id: usize, monitor_port: u16) -> Self {
        Self {
            kernel_interface: KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
            active_isolation: active_isolation::ActiveIsolationTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
            deadline_validation: deadline_validation::DeadlineValidationTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
            latency_tests: latency_tests::LatencyTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
        }
    }

    /// Run all Phase 3 tests
    pub async fn validate_phase3(&mut self) -> Result<Phase3Results, Box<dyn Error>> {
        log::info!("==================================================");
        log::info!("Starting Phase 3: Temporal Isolation Validation");
        log::info!("==================================================");

        // Run all test modules
        let isolation_result = self.active_isolation.run_all_tests().await?;
        let deadline_result = self.deadline_validation.run_all_tests().await?;
        let latency_result = self.latency_tests.run_all_tests().await?;

        // Calculate overall results
        let active_isolation_passed = isolation_result.passed;
        let deadline_validation_passed = deadline_result.passed;
        let latency_tests_passed = latency_result.passed;

        let total_tests = isolation_result.total_tests +
                         deadline_result.total_tests +
                         latency_result.total_tests;

        let passed_tests = isolation_result.passed_tests +
                          deadline_result.passed_tests +
                          latency_result.passed_tests;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;
        let overall_score = (passed_tests as f64 / total_tests as f64) * 100.0;

        log::info!("==================================================");
        log::info!("Phase 3 Summary:");
        log::info!("  Active Isolation:     {}", if active_isolation_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Deadline Validation:  {}", if deadline_validation_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Latency Tests:        {}", if latency_tests_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Overall:              {}/{} tests passed ({:.1}%)",
            passed_tests, total_tests, overall_score);
        log::info!("==================================================");

        Ok(Phase3Results {
            passed,
            active_isolation_passed,
            deadline_validation_passed,
            latency_tests_passed,
            total_tests,
            passed_tests,
            overall_score,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

impl Default for Phase3Results {
    fn default() -> Self {
        Self {
            passed: false,
            active_isolation_passed: false,
            deadline_validation_passed: false,
            latency_tests_passed: false,
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
    fn test_phase3_results_default() {
        let results = Phase3Results::default();
        assert_eq!(results.overall_score, 0.0);
        assert!(!results.passed);
    }
}
