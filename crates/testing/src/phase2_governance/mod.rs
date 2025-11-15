//! Phase 2: AI Governance & Safety Policies Tests
//!
//! Complete validation of AI governance including model management,
//! policy enforcement, and audit compliance.

pub mod model_governance;
pub mod policy_enforcement;
pub mod audit_compliance;

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Phase 2 test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase2Results {
    pub passed: bool,
    pub model_governance_passed: bool,
    pub policy_enforcement_passed: bool,
    pub audit_compliance_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub overall_score: f64,
    pub timestamp: String,
}

/// Phase 2 governance test suite
pub struct Phase2GovernanceSuite {
    #[allow(dead_code)]
    kernel_interface: KernelCommandInterface,
    model_governance: model_governance::ModelGovernanceTests,
    policy_enforcement: policy_enforcement::PolicyEnforcementTests,
    audit_compliance: audit_compliance::AuditComplianceTests,
}

impl Phase2GovernanceSuite {
    /// Create a new Phase 2 test suite
    pub fn new(serial_log_path: String, qemu_manager: std::sync::Arc<crate::qemu_runtime::QEMURuntimeManager>, node_id: usize, monitor_port: u16) -> Self {
        Self {
            kernel_interface: KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
            model_governance: model_governance::ModelGovernanceTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
            policy_enforcement: policy_enforcement::PolicyEnforcementTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
            audit_compliance: audit_compliance::AuditComplianceTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port)
            ),
        }
    }

    /// Run all Phase 2 tests
    pub async fn validate_phase2(&mut self) -> Result<Phase2Results, Box<dyn Error>> {
        log::info!("=================================================");
        log::info!("Starting Phase 2: AI Governance & Safety Policies");
        log::info!("=================================================");

        // Run all test modules
        let governance_result = self.model_governance.run_all_tests().await?;
        let policy_result = self.policy_enforcement.run_all_tests().await?;
        let audit_result = self.audit_compliance.run_all_tests().await?;

        // Calculate overall results
        let model_governance_passed = governance_result.passed;
        let policy_enforcement_passed = policy_result.passed;
        let audit_compliance_passed = audit_result.passed;

        let total_tests = governance_result.total_tests +
                         policy_result.total_tests +
                         audit_result.total_tests;

        let passed_tests = governance_result.passed_tests +
                          policy_result.passed_tests +
                          audit_result.passed_tests;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;
        let overall_score = (passed_tests as f64 / total_tests as f64) * 100.0;

        log::info!("=================================================");
        log::info!("Phase 2 Summary:");
        log::info!("  Model Governance:     {}", if model_governance_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Policy Enforcement:   {}", if policy_enforcement_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Audit & Compliance:   {}", if audit_compliance_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Overall:              {}/{} tests passed ({:.1}%)",
            passed_tests, total_tests, overall_score);
        log::info!("=================================================");

        Ok(Phase2Results {
            passed,
            model_governance_passed,
            policy_enforcement_passed,
            audit_compliance_passed,
            total_tests,
            passed_tests,
            overall_score,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

impl Default for Phase2Results {
    fn default() -> Self {
        Self {
            passed: false,
            model_governance_passed: false,
            policy_enforcement_passed: false,
            audit_compliance_passed: false,
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
    fn test_phase2_results_default() {
        let results = Phase2Results::default();
        assert_eq!(results.overall_score, 0.0);
        assert!(!results.passed);
    }
}
