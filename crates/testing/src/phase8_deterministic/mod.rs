//! Phase 8: Performance Optimization Tests
//!
//! Complete validation of CBS+EDF deterministic scheduler, slab allocator,
//! adaptive memory patterns, meta-agent decisions, and autonomy impact.
//!
//! ## Test Coverage
//!
//! - **CBS+EDF Scheduler**: Admission control, deadline guarantees, budget management
//! - **Slab Allocator**: Performance benchmarks, comparison vs linked-list
//! - **Adaptive Memory**: Strategy switching, oscillation detection, rate limiting
//! - **Meta-Agent**: Decision inference, confidence thresholds, reward feedback
//! - **Stress Comparison**: Autonomy ON vs OFF performance delta
//! - **Rate Limiting**: Output rate-limit validation (1 print/sec)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use sis_testing::phase8_deterministic::Phase8DeterministicSuite;
//!
//! let mut suite = Phase8DeterministicSuite::new(
//!     "/tmp/serial.log".to_string(),
//!     5555
//! );
//!
//! let results = suite.validate_phase8().await?;
//! println!("Phase 8 Score: {:.1}%", results.overall_score);
//! ```

pub mod cbs_edf_scheduler;
pub mod slab_allocator;
pub mod adaptive_memory;
pub mod meta_agent;
pub mod stress_comparison;
pub mod rate_limiting;

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Results from Phase 8 Performance Optimization validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase8Results {
    /// CBS+EDF scheduler tests passed
    pub cbs_edf_passed: bool,
    /// Slab allocator tests passed
    pub slab_allocator_passed: bool,
    /// Adaptive memory tests passed
    pub adaptive_memory_passed: bool,
    /// Meta-agent tests passed
    pub meta_agent_passed: bool,
    /// Stress comparison tests passed
    pub stress_comparison_passed: bool,
    /// Rate limiting tests passed
    pub rate_limiting_passed: bool,
    /// Overall score (0-100)
    pub overall_score: f64,
    /// Timestamp of validation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for Phase8Results {
    fn default() -> Self {
        Self {
            cbs_edf_passed: false,
            slab_allocator_passed: false,
            adaptive_memory_passed: false,
            meta_agent_passed: false,
            stress_comparison_passed: false,
            rate_limiting_passed: false,
            overall_score: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Phase 8 Performance Optimization test suite
pub struct Phase8DeterministicSuite {
    #[allow(dead_code)]
    kernel_interface: KernelCommandInterface,
    cbs_edf: cbs_edf_scheduler::CBSEDFSchedulerTests,
    slab_allocator: slab_allocator::SlabAllocatorTests,
    adaptive_memory: adaptive_memory::AdaptiveMemoryTests,
    meta_agent: meta_agent::MetaAgentTests,
    stress_comparison: stress_comparison::StressComparisonTests,
    rate_limiting: rate_limiting::RateLimitingTests,
}

impl Phase8DeterministicSuite {
    /// Create a new Phase 8 test suite
    ///
    /// # Arguments
    ///
    /// * `serial_log_path` - Path to QEMU serial log
    /// * `monitor_port` - QEMU monitor port
    pub fn new(serial_log_path: String, monitor_port: u16) -> Self {
        let serial_log_path_clone = serial_log_path.clone();

        Self {
            kernel_interface: KernelCommandInterface::new(serial_log_path.clone(), monitor_port),
            cbs_edf: cbs_edf_scheduler::CBSEDFSchedulerTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            slab_allocator: slab_allocator::SlabAllocatorTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            adaptive_memory: adaptive_memory::AdaptiveMemoryTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            meta_agent: meta_agent::MetaAgentTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            stress_comparison: stress_comparison::StressComparisonTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            rate_limiting: rate_limiting::RateLimitingTests::new(
                KernelCommandInterface::new(serial_log_path_clone, monitor_port)
            ),
        }
    }

    /// Run complete Phase 8 validation suite
    ///
    /// This executes all Phase 8 test modules in parallel where possible,
    /// then runs stress comparison tests sequentially for clean state.
    ///
    /// # Returns
    ///
    /// `Phase8Results` with pass/fail status for each subsystem and overall score
    pub async fn validate_phase8(&mut self) -> Result<Phase8Results, Box<dyn Error>> {
        log::info!("🚀 Starting Phase 8: Performance Optimization validation");

        // Run tests in parallel where possible
        let (cbs_result, slab_result, adaptive_result, meta_result, rate_result) = tokio::try_join!(
            self.cbs_edf.run_all_tests(),
            self.slab_allocator.run_all_tests(),
            self.adaptive_memory.run_all_tests(),
            self.meta_agent.run_all_tests(),
            self.rate_limiting.run_all_tests(),
        )?;

        // Run stress comparison sequentially (requires clean state)
        let stress_result = self.stress_comparison.run_all_tests().await?;

        // Calculate overall score
        let passed_tests = vec![
            cbs_result.passed,
            slab_result.passed,
            adaptive_result.passed,
            meta_result.passed,
            stress_result.passed,
            rate_result.passed,
        ];

        let passed_count = passed_tests.iter().filter(|&&p| p).count();
        let overall_score = (passed_count as f64 / 6.0) * 100.0;

        log::info!("✅ Phase 8 validation complete: {:.1}% ({}/6 subsystems passed)",
            overall_score, passed_count);

        Ok(Phase8Results {
            cbs_edf_passed: cbs_result.passed,
            slab_allocator_passed: slab_result.passed,
            adaptive_memory_passed: adaptive_result.passed,
            meta_agent_passed: meta_result.passed,
            stress_comparison_passed: stress_result.passed,
            rate_limiting_passed: rate_result.passed,
            overall_score,
            timestamp: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase8_results_default() {
        let results = Phase8Results::default();
        assert_eq!(results.overall_score, 0.0);
        assert!(!results.cbs_edf_passed);
    }
}
