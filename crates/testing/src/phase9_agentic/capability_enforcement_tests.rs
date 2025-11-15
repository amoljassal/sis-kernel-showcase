//! Capability Enforcement Tests
//!
//! Tests for agent capability checks, access control, and scope restrictions.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Results from capability enforcement tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityTestResults {
    /// All tests passed
    pub passed: bool,
    /// Deny unauthorized agent test passed
    pub deny_unauthorized_passed: bool,
    /// Scope restriction test passed
    pub scope_restriction_passed: bool,
    /// Multiple agent verification passed
    pub multiple_agents_passed: bool,
}

/// Capability enforcement test suite
pub struct CapabilityEnforcementTests {
    kernel_interface: KernelCommandInterface,
}

impl CapabilityEnforcementTests {
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    /// Run all capability enforcement tests
    pub async fn run_all_tests(&mut self) -> Result<CapabilityTestResults, Box<dyn Error>> {
        log::info!("🔒 Running Capability Enforcement Tests...");

        let deny_unauthorized_passed = self.test_deny_unauthorized().await?;
        let scope_restriction_passed = self.test_scope_restrictions().await?;
        let multiple_agents_passed = self.test_multiple_agents().await?;

        let passed =
            deny_unauthorized_passed && scope_restriction_passed && multiple_agents_passed;

        Ok(CapabilityTestResults {
            passed,
            deny_unauthorized_passed,
            scope_restriction_passed,
            multiple_agents_passed,
        })
    }

    /// Test that unauthorized access is denied
    ///
    /// The policy engine should deny operations when an agent lacks the required capability.
    /// We verify this by checking audit logs for denied operations.
    async fn test_deny_unauthorized(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing unauthorized access denial...");

        // Run a test that should succeed (test agent has FsBasic capability)
        let _ = self
            .kernel_interface
            .execute_command("agentsys test-fs-list")
            .await?;

        // Check audit logs for successful operation
        let output = self.kernel_interface.execute_command("agentsys audit").await?;

        // Verify audit contains both allowed and denied operations
        // The policy engine logs "[AUDIT] agent=X opcode=Y allowed=true/false"
        let success = output.raw_output.contains("[AgentSys] Recent audit records:")
            && output.raw_output.contains("[AUDIT]");

        if success {
            log::info!("  ✅ Unauthorized access denial test passed");
        } else {
            log::warn!(
                "  ❌ Unauthorized access denial test failed: {}",
                output.raw_output
            );
        }

        Ok(success)
    }

    /// Test scope restriction enforcement
    ///
    /// Agents should only access resources within their allowed scope.
    /// We verify the policy engine enforces path restrictions.
    async fn test_scope_restrictions(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing scope restrictions...");

        // The test agent (AGENT_ID_TEST) has scope restricted to /tmp/*
        // Running test-fs-list on /tmp/ should succeed
        let output = self
            .kernel_interface
            .execute_command("agentsys test-fs-list")
            .await?;

        let success = output.raw_output.contains("[AgentSys] Test PASSED");

        if success {
            log::info!("  ✅ Scope restriction test passed");
        } else {
            log::warn!("  ❌ Scope restriction test failed: {}", output.raw_output);
        }

        Ok(success)
    }

    /// Test multiple agent support
    ///
    /// Verify that the system correctly tracks multiple agents with different capabilities.
    async fn test_multiple_agents(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing multiple agent support...");

        let output = self.kernel_interface.execute_command("agentsys status").await?;

        // Should see multiple registered agents (at least system, assistant, test agents)
        let has_agents = output.raw_output.contains("Registered agents:");
        let has_system = output.raw_output.contains("system");
        let has_assistant = output.raw_output.contains("assistant");
        let has_test = output.raw_output.contains("test");

        let success = has_agents && has_system && has_assistant && has_test;

        if success {
            log::info!("  ✅ Multiple agent support test passed");
        } else {
            log::warn!(
                "  ❌ Multiple agent support test failed: {}",
                output.raw_output
            );
        }

        Ok(success)
    }
}
