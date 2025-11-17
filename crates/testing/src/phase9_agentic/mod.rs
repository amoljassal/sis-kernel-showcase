//! Phase 9: Agentic Platform Tests
//!
//! Complete validation of the AgentSys capability-based system call layer
//! for LLM-driven agents.
//!
//! ## Test Coverage
//!
//! - **Protocol Tests**: TLV encoding/decoding, frame parsing, error handling
//! - **Capability Enforcement**: Access control, scope restrictions, capability checks
//! - **Audit Validation**: Audit logging, operation tracking, security compliance
//!
//! ## Usage
//!
//! ```rust,no_run
//! use sis_testing::phase9_agentic::Phase9AgenticSuite;
//!
//! let mut suite = Phase9AgenticSuite::new(
//!     "/tmp/serial.log".to_string(),
//!     qemu_manager,
//!     0,
//!     5555
//! );
//!
//! let results = suite.validate_phase9().await?;
//! println!("Phase 9 Score: {:.1}%", results.overall_score);
//! ```

pub mod agentsys_protocol_tests;
pub mod capability_enforcement_tests;
pub mod audit_validation_tests;
pub mod asm_supervision_tests;

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Results from Phase 9 Agentic Platform validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase9Results {
    /// AgentSys protocol tests passed
    pub protocol_tests_passed: bool,
    /// Capability enforcement tests passed
    pub capability_tests_passed: bool,
    /// Audit validation tests passed
    pub audit_tests_passed: bool,
    /// ASM supervision tests passed
    pub asm_supervision_tests_passed: bool,
    /// Overall score (0-100)
    pub overall_score: f64,
    /// Individual test details
    pub test_details: Phase9TestDetails,
    /// ASM supervision test details
    pub asm_test_details: Option<asm_supervision_tests::ASMSupervisionResults>,
    /// Timestamp of validation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detailed test results for Phase 9
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase9TestDetails {
    /// Protocol tests: FS_LIST success
    pub fs_list_test: bool,
    /// Protocol tests: AUDIO_PLAY success
    pub audio_play_test: bool,
    /// Protocol tests: Invalid opcode handling
    pub invalid_opcode_test: bool,
    /// Capability tests: Access denied for unauthorized agent
    pub capability_deny_test: bool,
    /// Capability tests: Scope restriction enforcement
    pub scope_restriction_test: bool,
    /// Audit tests: Operation logging verification
    pub audit_logging_test: bool,
    /// Audit tests: Audit dump validation
    pub audit_dump_test: bool,
    /// AgentSys status command validation
    pub status_command_test: bool,
    /// Memory overhead check (< 100 KiB)
    pub memory_overhead_check: bool,
}

impl Default for Phase9TestDetails {
    fn default() -> Self {
        Self {
            fs_list_test: false,
            audio_play_test: false,
            invalid_opcode_test: false,
            capability_deny_test: false,
            scope_restriction_test: false,
            audit_logging_test: false,
            audit_dump_test: false,
            status_command_test: false,
            memory_overhead_check: false,
        }
    }
}

impl Default for Phase9Results {
    fn default() -> Self {
        Self {
            protocol_tests_passed: false,
            capability_tests_passed: false,
            audit_tests_passed: false,
            asm_supervision_tests_passed: false,
            overall_score: 0.0,
            test_details: Phase9TestDetails::default(),
            asm_test_details: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Phase 9 Agentic Platform test suite
pub struct Phase9AgenticSuite {
    #[allow(dead_code)]
    kernel_interface: KernelCommandInterface,
    protocol_tests: agentsys_protocol_tests::AgentSysProtocolTests,
    capability_tests: capability_enforcement_tests::CapabilityEnforcementTests,
    audit_tests: audit_validation_tests::AuditValidationTests,
    asm_supervision_tests: asm_supervision_tests::ASMSupervisionTests,
}

impl Phase9AgenticSuite {
    /// Create a new Phase 9 test suite
    ///
    /// # Arguments
    ///
    /// * `serial_log_path` - Path to QEMU serial log
    /// * `qemu_manager` - Arc-wrapped QEMU runtime manager
    /// * `node_id` - Node ID for PTY communication
    /// * `monitor_port` - QEMU monitor port
    pub fn new(
        serial_log_path: String,
        qemu_manager: std::sync::Arc<crate::qemu_runtime::QEMURuntimeManager>,
        node_id: usize,
        monitor_port: u16,
    ) -> Self {
        Self {
            kernel_interface: KernelCommandInterface::new(
                serial_log_path.clone(),
                qemu_manager.clone(),
                node_id,
                monitor_port,
            ),
            protocol_tests: agentsys_protocol_tests::AgentSysProtocolTests::new(
                KernelCommandInterface::new(
                    serial_log_path.clone(),
                    qemu_manager.clone(),
                    node_id,
                    monitor_port,
                ),
            ),
            capability_tests: capability_enforcement_tests::CapabilityEnforcementTests::new(
                KernelCommandInterface::new(
                    serial_log_path.clone(),
                    qemu_manager.clone(),
                    node_id,
                    monitor_port,
                ),
            ),
            audit_tests: audit_validation_tests::AuditValidationTests::new(
                KernelCommandInterface::new(
                    serial_log_path.clone(),
                    qemu_manager.clone(),
                    node_id,
                    monitor_port,
                ),
            ),
            asm_supervision_tests: asm_supervision_tests::ASMSupervisionTests::new(
                KernelCommandInterface::new(serial_log_path, qemu_manager, node_id, monitor_port),
            ),
        }
    }

    /// Run complete Phase 9 validation suite
    ///
    /// This executes all Phase 9 test modules sequentially, as they
    /// share the same kernel state and need to run in order.
    ///
    /// # Returns
    ///
    /// `Phase9Results` with pass/fail status for each subsystem and overall score
    pub async fn validate_phase9(&mut self) -> Result<Phase9Results, Box<dyn Error>> {
        log::info!("🚀 Starting Phase 9: Agentic Platform validation");

        // Run protocol tests (don't fail if they timeout)
        let protocol_result = match self.protocol_tests.run_all_tests().await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Protocol tests failed: {} - continuing with other tests", e);
                agentsys_protocol_tests::ProtocolTestResults::default()
            }
        };

        // Run capability tests (don't fail if they timeout)
        let capability_result = match self.capability_tests.run_all_tests().await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Capability tests failed: {} - continuing with other tests", e);
                capability_enforcement_tests::CapabilityTestResults::default()
            }
        };

        // Run audit tests (don't fail if they timeout)
        let audit_result = match self.audit_tests.run_all_tests().await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Audit tests failed: {} - continuing with other tests", e);
                audit_validation_tests::AuditTestResults::default()
            }
        };

        // Run ASM supervision tests (don't fail if they timeout)
        let asm_result = match self.asm_supervision_tests.run_all_tests().await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("ASM supervision tests failed: {} - continuing", e);
                asm_supervision_tests::ASMSupervisionResults {
                    passed: false,
                    lifecycle_tests_passed: false,
                    telemetry_tests_passed: false,
                    tests_passed: 0,
                    total_tests: 11,
                    test_details: asm_supervision_tests::ASMTestDetails::default(),
                }
            }
        };

        // Collect detailed results
        let test_details = Phase9TestDetails {
            fs_list_test: protocol_result.fs_list_passed,
            audio_play_test: protocol_result.audio_play_passed,
            invalid_opcode_test: protocol_result.invalid_opcode_passed,
            capability_deny_test: capability_result.deny_unauthorized_passed,
            scope_restriction_test: capability_result.scope_restriction_passed,
            audit_logging_test: audit_result.operation_logging_passed,
            audit_dump_test: audit_result.audit_dump_passed,
            status_command_test: protocol_result.status_command_passed,
            memory_overhead_check: protocol_result.memory_overhead_check_passed,
        };

        // Calculate overall score
        let all_tests = vec![
            test_details.fs_list_test,
            test_details.audio_play_test,
            test_details.invalid_opcode_test,
            test_details.capability_deny_test,
            test_details.scope_restriction_test,
            test_details.audit_logging_test,
            test_details.audit_dump_test,
            test_details.status_command_test,
            test_details.memory_overhead_check,
        ];

        let passed_count = all_tests.iter().filter(|&&p| p).count();
        let overall_score = (passed_count as f64 / all_tests.len() as f64) * 100.0;

        log::info!(
            "✅ Phase 9 validation complete: {:.1}% ({}/{} tests passed)",
            overall_score,
            passed_count,
            all_tests.len()
        );

        log::info!(
            "   ASM Supervision: {}/{} tests passed",
            asm_result.tests_passed,
            asm_result.total_tests
        );

        Ok(Phase9Results {
            protocol_tests_passed: protocol_result.passed,
            capability_tests_passed: capability_result.passed,
            audit_tests_passed: audit_result.passed,
            asm_supervision_tests_passed: asm_result.passed,
            overall_score,
            test_details,
            asm_test_details: Some(asm_result),
            timestamp: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase9_results_default() {
        let results = Phase9Results::default();
        assert_eq!(results.overall_score, 0.0);
        assert!(!results.protocol_tests_passed);
        assert!(!results.capability_tests_passed);
        assert!(!results.audit_tests_passed);
    }

    #[test]
    fn test_phase9_test_details_default() {
        let details = Phase9TestDetails::default();
        assert!(!details.fs_list_test);
        assert!(!details.audio_play_test);
        assert!(!details.invalid_opcode_test);
    }
}
