//! Phase 7: AI Operations Platform Tests
//!
//! Complete validation of AI Ops infrastructure including model lifecycle,
//! shadow deployment, OpenTelemetry integration, and decision traces.
//!
//! ## Test Coverage
//!
//! - **Model Lifecycle**: Registration, hot-swap, rollback operations
//! - **Shadow Mode**: Shadow deployment, canary routing, A/B comparison
//! - **OpenTelemetry**: Trace export, span creation, context propagation
//! - **Decision Traces**: Decision collection, buffer management, export/replay
//! - **Integration**: End-to-end AI Ops workflows
//!
//! ## Usage
//!
//! ```rust,no_run
//! use sis_testing::phase7_ai_ops::Phase7AIOpsSuite;
//!
//! let mut suite = Phase7AIOpsSuite::new(
//!     "/tmp/serial.log".to_string(),
//!     5555
//! );
//!
//! let results = suite.validate_phase7().await?;
//! println!("Phase 7 Score: {:.1}%", results.overall_score);
//! ```

pub mod model_lifecycle;
pub mod shadow_mode;
pub mod otel_exporter;
pub mod decision_traces;
pub mod integration_tests;

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Results from Phase 7 AI Operations Platform validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase7Results {
    /// Model lifecycle tests passed
    pub model_lifecycle_passed: bool,
    /// Shadow mode tests passed
    pub shadow_mode_passed: bool,
    /// OpenTelemetry exporter tests passed
    pub otel_exporter_passed: bool,
    /// Decision traces tests passed
    pub decision_traces_passed: bool,
    /// Integration tests passed
    pub integration_passed: bool,
    /// Overall score (0-100)
    pub overall_score: f64,
    /// Timestamp of validation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for Phase7Results {
    fn default() -> Self {
        Self {
            model_lifecycle_passed: false,
            shadow_mode_passed: false,
            otel_exporter_passed: false,
            decision_traces_passed: false,
            integration_passed: false,
            overall_score: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Phase 7 AI Operations Platform test suite
pub struct Phase7AIOpsSuite {
    #[allow(dead_code)]
    kernel_interface: KernelCommandInterface,
    model_lifecycle: model_lifecycle::ModelLifecycleTests,
    shadow_mode: shadow_mode::ShadowModeTests,
    otel_exporter: otel_exporter::OTelExporterTests,
    decision_traces: decision_traces::DecisionTracesTests,
    integration: integration_tests::Phase7IntegrationTests,
}

impl Phase7AIOpsSuite {
    /// Create a new Phase 7 test suite
    ///
    /// # Arguments
    ///
    /// * `serial_log_path` - Path to QEMU serial log
    /// * `qemu_manager` - Arc-wrapped QEMU runtime manager
    /// * `node_id` - Node ID for PTY communication
    /// * `monitor_port` - QEMU monitor port
    pub fn new(serial_log_path: String, qemu_manager: std::sync::Arc<crate::qemu_runtime::QEMURuntimeManager>, node_id: usize, monitor_port: u16) -> Self {
        let serial_log_path_clone = serial_log_path.clone();

        Self {
            kernel_interface: KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
            model_lifecycle: model_lifecycle::ModelLifecycleTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
            shadow_mode: shadow_mode::ShadowModeTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
            otel_exporter: otel_exporter::OTelExporterTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
            decision_traces: decision_traces::DecisionTracesTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
            integration: integration_tests::Phase7IntegrationTests::new(
                KernelCommandInterface::new(serial_log_path_clone, qemu_manager.clone(), node_id, monitor_port)
            ),
        }
    }

    /// Run complete Phase 7 validation suite
    ///
    /// This executes all Phase 7 test modules in parallel where possible,
    /// then runs integration tests sequentially.
    ///
    /// # Returns
    ///
    /// `Phase7Results` with pass/fail status for each subsystem and overall score
    pub async fn validate_phase7(&mut self) -> Result<Phase7Results, Box<dyn Error>> {
        log::info!("🚀 Starting Phase 7: AI Operations Platform validation");

        // Run all test suites in parallel
        let (lifecycle_result, shadow_result, otel_result, traces_result) = tokio::try_join!(
            self.model_lifecycle.run_all_tests(),
            self.shadow_mode.run_all_tests(),
            self.otel_exporter.run_all_tests(),
            self.decision_traces.run_all_tests(),
        )?;

        // Run integration tests sequentially (depends on above)
        let integration_result = self.integration.run_all_tests().await?;

        // Calculate overall score
        let passed_tests = vec![
            lifecycle_result.passed,
            shadow_result.passed,
            otel_result.passed,
            traces_result.passed,
            integration_result.passed,
        ];

        let passed_count = passed_tests.iter().filter(|&&p| p).count();
        let overall_score = (passed_count as f64 / 5.0) * 100.0;

        log::info!("✅ Phase 7 validation complete: {:.1}% ({}/5 subsystems passed)",
            overall_score, passed_count);

        Ok(Phase7Results {
            model_lifecycle_passed: lifecycle_result.passed,
            shadow_mode_passed: shadow_result.passed,
            otel_exporter_passed: otel_result.passed,
            decision_traces_passed: traces_result.passed,
            integration_passed: integration_result.passed,
            overall_score,
            timestamp: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase7_results_default() {
        let results = Phase7Results::default();
        assert_eq!(results.overall_score, 0.0);
        assert!(!results.model_lifecycle_passed);
    }
}
