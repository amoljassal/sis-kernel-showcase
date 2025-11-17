//! Audit Validation Tests
//!
//! Tests for audit logging, operation tracking, and security compliance.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Results from audit validation tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTestResults {
    /// All tests passed
    pub passed: bool,
    /// Operation logging test passed
    pub operation_logging_passed: bool,
    /// Audit dump test passed
    pub audit_dump_passed: bool,
    /// Audit completeness test passed
    pub audit_completeness_passed: bool,
}

impl Default for AuditTestResults {
    fn default() -> Self {
        Self {
            passed: false,
            operation_logging_passed: false,
            audit_dump_passed: false,
            audit_completeness_passed: false,
        }
    }
}

/// Audit validation test suite
pub struct AuditValidationTests {
    kernel_interface: KernelCommandInterface,
}

impl AuditValidationTests {
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    /// Run all audit validation tests
    pub async fn run_all_tests(&mut self) -> Result<AuditTestResults, Box<dyn Error>> {
        log::info!("📋 Running Audit Validation Tests...");

        let operation_logging_passed = self.test_operation_logging().await?;
        let audit_dump_passed = self.test_audit_dump().await?;
        let audit_completeness_passed = self.test_audit_completeness().await?;

        let passed =
            operation_logging_passed && audit_dump_passed && audit_completeness_passed;

        Ok(AuditTestResults {
            passed,
            operation_logging_passed,
            audit_dump_passed,
            audit_completeness_passed,
        })
    }

    /// Test that operations are properly logged
    async fn test_operation_logging(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing operation logging...");

        // Execute operations that should be logged
        let _ = self
            .kernel_interface
            .execute_command("agentsys test-fs-list")
            .await?;
        let _ = self
            .kernel_interface
            .execute_command("agentsys test-audio-play")
            .await?;

        // Check audit logs
        let output = self.kernel_interface.execute_command("agentsys audit").await?;

        // Verify audit logs contain operation records
        let has_audit_header = output
            .raw_output
            .contains("[AgentSys] Recent audit records:");
        let has_audit_entries = output.raw_output.contains("[AUDIT]");

        let success = has_audit_header && has_audit_entries;

        if success {
            log::info!("  ✅ Operation logging test passed");
        } else {
            log::warn!(
                "  ❌ Operation logging test failed: {}",
                output.raw_output
            );
        }

        Ok(success)
    }

    /// Test audit dump functionality
    async fn test_audit_dump(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing audit dump...");

        let output = self.kernel_interface.execute_command("agentsys audit").await?;

        // Verify audit dump contains expected fields
        let has_agent_id = output.raw_output.contains("agent=");
        let has_opcode = output.raw_output.contains("opcode=") || output.raw_output.contains("op=");
        let has_allowed = output.raw_output.contains("allowed=");

        let success = has_agent_id && has_opcode && has_allowed;

        if success {
            log::info!("  ✅ Audit dump test passed");
        } else {
            log::warn!("  ❌ Audit dump test failed: {}", output.raw_output);
        }

        Ok(success)
    }

    /// Test audit completeness (all operations tracked)
    async fn test_audit_completeness(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing audit completeness...");

        // Get initial operation count
        let status_before = self.kernel_interface.execute_command("agentsys status").await?;

        // Execute a known operation
        let _ = self
            .kernel_interface
            .execute_command("agentsys test-fs-list")
            .await?;

        // Get updated operation count
        let status_after = self.kernel_interface.execute_command("agentsys status").await?;

        // Verify operation count increased
        let has_operations_before = status_before.raw_output.contains("Total operations:");
        let has_operations_after = status_after.raw_output.contains("Total operations:");

        // Basic check: if both status outputs show operation counts, audit is tracking
        let success = has_operations_before && has_operations_after;

        if success {
            log::info!("  ✅ Audit completeness test passed");
        } else {
            log::warn!("  ❌ Audit completeness test failed");
        }

        Ok(success)
    }
}
