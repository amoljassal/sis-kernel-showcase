//! ASM (Agent Supervision Module) Integration Tests
//!
//! Week 2: Lifecycle and Telemetry Integration Testing
//! Week 3: Resource Monitoring and Dependency Integration Testing
//! Week 4: Cloud Gateway and Advanced Feature Testing
//!
//! These tests validate ASM supervision features end-to-end through the shell
//! command interface, ensuring real kernel integration works as expected.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Results from ASM supervision tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASMSupervisionResults {
    /// Overall pass/fail status
    pub passed: bool,
    /// Lifecycle tests passed
    pub lifecycle_tests_passed: bool,
    /// Telemetry tests passed
    pub telemetry_tests_passed: bool,
    /// Number of tests passed
    pub tests_passed: usize,
    /// Total number of tests
    pub total_tests: usize,
    /// Individual test results
    pub test_details: ASMTestDetails,
}

/// Detailed ASM test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASMTestDetails {
    // Week 2: Lifecycle tests (TC-INT-LC-*)
    pub status_command: bool,
    pub list_command: bool,
    pub metrics_command: bool,
    pub resources_command: bool,
    pub telemetry_command: bool,
    pub compliance_command: bool,

    // Week 3: Resource & Dependency tests
    pub limits_command: bool,
    pub deps_command: bool,
    pub depgraph_command: bool,
    pub profile_command: bool,
    pub dump_command: bool,

    // Week 4: Cloud Gateway tests (TC-INT-CG-*)
    pub gwstatus_command: bool,
}

impl Default for ASMTestDetails {
    fn default() -> Self {
        Self {
            status_command: false,
            list_command: false,
            metrics_command: false,
            resources_command: false,
            telemetry_command: false,
            compliance_command: false,
            limits_command: false,
            deps_command: false,
            depgraph_command: false,
            profile_command: false,
            dump_command: false,
            gwstatus_command: false,
        }
    }
}

/// ASM Supervision test suite
pub struct ASMSupervisionTests {
    kernel_interface: KernelCommandInterface,
}

impl ASMSupervisionTests {
    /// Create a new ASM supervision test suite
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    /// Run all ASM supervision tests
    pub async fn run_all_tests(&mut self) -> Result<ASMSupervisionResults, Box<dyn Error>> {
        log::info!("🧪 Starting ASM Supervision integration tests");

        let mut test_details = ASMTestDetails::default();

        // Test 1: agentsys status command
        test_details.status_command = self.test_status_command().await?;

        // Test 2: agentsys list command
        test_details.list_command = self.test_list_command().await?;

        // Test 3: agentsys metrics command
        test_details.metrics_command = self.test_metrics_command().await?;

        // Test 4: agentsys resources command
        test_details.resources_command = self.test_resources_command().await?;

        // Test 5: agentsys telemetry command
        test_details.telemetry_command = self.test_telemetry_command().await?;

        // Test 6: agentsys compliance command
        test_details.compliance_command = self.test_compliance_command().await?;

        // Test 7: agentsys limits command
        test_details.limits_command = self.test_limits_command().await?;

        // Test 8: agentsys deps command
        test_details.deps_command = self.test_deps_command().await?;

        // Test 9: agentsys depgraph command
        test_details.depgraph_command = self.test_depgraph_command().await?;

        // Test 10: agentsys profile command
        test_details.profile_command = self.test_profile_command().await?;

        // Test 11: agentsys dump command
        test_details.dump_command = self.test_dump_command().await?;

        // Test 12: gwstatus command (Week 4 - Cloud Gateway)
        test_details.gwstatus_command = self.test_gwstatus_command().await?;

        // Calculate results
        let tests = vec![
            test_details.status_command,
            test_details.list_command,
            test_details.metrics_command,
            test_details.resources_command,
            test_details.telemetry_command,
            test_details.compliance_command,
            test_details.limits_command,
            test_details.deps_command,
            test_details.depgraph_command,
            test_details.profile_command,
            test_details.dump_command,
            test_details.gwstatus_command,
        ];

        let tests_passed = tests.iter().filter(|&&p| p).count();
        let total_tests = tests.len();
        let lifecycle_tests_passed = tests[0..6].iter().filter(|&&p| p).count() == 6;
        let telemetry_tests_passed = tests[3..5].iter().filter(|&&p| p).count() == 2;

        log::info!(
            "✅ ASM Supervision tests complete: {}/{} passed",
            tests_passed,
            total_tests
        );

        Ok(ASMSupervisionResults {
            passed: tests_passed == total_tests,
            lifecycle_tests_passed,
            telemetry_tests_passed,
            tests_passed,
            total_tests,
            test_details,
        })
    }

    /// TC-INT-LC-001: Test agentsys status command
    async fn test_status_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-LC-001: Testing agentsys status");

        let output = self
            .kernel_interface
            .execute_command("agentsys status")
            .await?;

        // Verify output contains expected sections
        let checks = vec![
            output.raw_output.contains("Agent Supervision Module Status"),
            output.raw_output.contains("Subsystems:"),
            output.raw_output.contains("Agent Supervisor"),
            output.raw_output.contains("Telemetry Aggregator"),
            output.raw_output.contains("Fault Detector"),
            output.raw_output.contains("Policy Controller"),
            output.raw_output.contains("Compliance Tracker"),
            output.raw_output.contains("Resource Monitor"),
            output.raw_output.contains("Dependency Graph"),
            output.raw_output.contains("System Profiler"),
            output.raw_output.contains("Quick Stats:"),
            output.raw_output.contains("System Health:"),
        ];

        let passed = checks.iter().all(|&c| c);

        if passed {
            log::info!("    ✓ Status command working correctly");
        } else {
            log::error!("    ✗ Status command output incomplete");
        }

        Ok(passed)
    }

    /// TC-INT-LC-002: Test agentsys list command
    async fn test_list_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-LC-002: Testing agentsys list");

        let output = self.kernel_interface.execute_command("agentsys list").await?;

        // Should show active agents header
        let passed = output.raw_output.contains("Active Agents") || output.raw_output.contains("No active agents");

        if passed {
            log::info!("    ✓ List command working correctly");
        } else {
            log::error!("    ✗ List command output invalid");
        }

        Ok(passed)
    }

    /// TC-INT-TM-001: Test agentsys metrics command
    async fn test_metrics_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-TM-001: Testing agentsys metrics");

        // Test with agent ID 1 (common system agent)
        let output = self
            .kernel_interface
            .execute_command("agentsys metrics 1")
            .await?;

        // Should either show metrics or "no metrics found"
        let passed = output.raw_output.contains("Agent 1")
            || output.raw_output.contains("No metrics found")
            || output.raw_output.contains("Error:");

        if passed {
            log::info!("    ✓ Metrics command working correctly");
        } else {
            log::error!("    ✗ Metrics command output invalid");
        }

        Ok(passed)
    }

    /// TC-INT-TM-002: Test agentsys resources command
    async fn test_resources_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-TM-002: Testing agentsys resources");

        let output = self
            .kernel_interface
            .execute_command("agentsys resources 1")
            .await?;

        // Should either show resources or error
        let passed = output.raw_output.contains("Agent 1")
            || output.raw_output.contains("No resource data")
            || output.raw_output.contains("Error:");

        if passed {
            log::info!("    ✓ Resources command working correctly");
        } else {
            log::error!("    ✗ Resources command output invalid");
        }

        Ok(passed)
    }

    /// TC-INT-TM-003: Test agentsys telemetry command
    async fn test_telemetry_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-TM-003: Testing agentsys telemetry");

        let output = self
            .kernel_interface
            .execute_command("agentsys telemetry")
            .await?;

        // Verify telemetry structure
        let checks = vec![
            output.raw_output.contains("Agent Supervision Module Status"),
            output.raw_output.contains("System Metrics:"),
            output.raw_output.contains("Total Spawns:"),
            output.raw_output.contains("Total Exits:"),
            output.raw_output.contains("Total Faults:"),
            output.raw_output.contains("Active Agents:"),
        ];

        let passed = checks.iter().all(|&c| c);

        if passed {
            log::info!("    ✓ Telemetry command working correctly");
        } else {
            log::error!("    ✗ Telemetry command output incomplete");
        }

        Ok(passed)
    }

    /// TC-INT-CP-001: Test agentsys compliance command
    async fn test_compliance_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-CP-001: Testing agentsys compliance");

        let output = self
            .kernel_interface
            .execute_command("agentsys compliance")
            .await?;

        // Verify compliance report structure
        let checks = vec![
            output.raw_output.contains("EU AI Act Compliance Report"),
            output.raw_output.contains("Timestamp:"),
            output.raw_output.contains("Total Agents:"),
            output.raw_output.contains("Risk Level Distribution:"),
            output.raw_output.contains("Compliance Requirements"),
        ];

        let passed = checks.iter().all(|&c| c);

        if passed {
            log::info!("    ✓ Compliance command working correctly");
        } else {
            log::error!("    ✗ Compliance command output incomplete");
        }

        Ok(passed)
    }

    /// TC-INT-RM-002: Test agentsys limits command
    async fn test_limits_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-RM-002: Testing agentsys limits");

        let output = self
            .kernel_interface
            .execute_command("agentsys limits 1")
            .await?;

        // Verify limits output structure
        let checks = vec![
            output.raw_output.contains("Agent 1 - Resource Limits"),
            output.raw_output.contains("CPU Quota:"),
            output.raw_output.contains("Memory Limit:"),
            output.raw_output.contains("Syscall Rate:"),
            output.raw_output.contains("Watchdog Timeout:"),
        ];

        let passed = checks.iter().all(|&c| c);

        if passed {
            log::info!("    ✓ Limits command working correctly");
        } else {
            log::error!("    ✗ Limits command output incomplete");
        }

        Ok(passed)
    }

    /// TC-INT-DP-001: Test agentsys deps command
    async fn test_deps_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-DP-001: Testing agentsys deps");

        let output = self
            .kernel_interface
            .execute_command("agentsys deps 1")
            .await?;

        // Verify deps output structure
        let checks = vec![
            output.raw_output.contains("Agent 1 - Dependencies"),
            output.raw_output.contains("Depends On:"),
            output.raw_output.contains("Depended On By:"),
        ];

        let passed = checks.iter().all(|&c| c);

        if passed {
            log::info!("    ✓ Deps command working correctly");
        } else {
            log::error!("    ✗ Deps command output incomplete");
        }

        Ok(passed)
    }

    /// TC-INT-DP-002: Test agentsys depgraph command
    async fn test_depgraph_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-DP-002: Testing agentsys depgraph");

        let output = self
            .kernel_interface
            .execute_command("agentsys depgraph")
            .await?;

        // Should show dependency graph header
        let passed = output.raw_output.contains("Agent Dependency Graph")
            || output.raw_output.contains("No active agents");

        if passed {
            log::info!("    ✓ Depgraph command working correctly");
        } else {
            log::error!("    ✗ Depgraph command output invalid");
        }

        Ok(passed)
    }

    /// TC-INT-PR-001: Test agentsys profile command
    async fn test_profile_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-PR-001: Testing agentsys profile");

        let output = self
            .kernel_interface
            .execute_command("agentsys profile 1")
            .await?;

        // Should either show profile or "no data"
        let passed = output.raw_output.contains("Agent 1")
            || output.raw_output.contains("No profile data")
            || output.raw_output.contains("Error:");

        if passed {
            log::info!("    ✓ Profile command working correctly");
        } else {
            log::error!("    ✗ Profile command output invalid");
        }

        Ok(passed)
    }

    /// TC-INT-ST-002: Test agentsys dump command
    async fn test_dump_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-ST-002: Testing agentsys dump");

        let output = self.kernel_interface.execute_command("agentsys dump").await?;

        // Verify dump combines multiple outputs
        let checks = vec![
            output.raw_output.contains("ASM Debug Dump"),
            output.raw_output.contains("Agent Supervision Module Status"),
            output.raw_output.contains("Active Agents"),
            output.raw_output.contains("Compliance Report"),
            output.raw_output.contains("Cloud Gateway"),
        ];

        let passed = checks.iter().all(|&c| c);

        if passed {
            log::info!("    ✓ Dump command working correctly");
        } else {
            log::error!("    ✗ Dump command output incomplete");
        }

        Ok(passed)
    }

    /// TC-INT-CG-001: Test gwstatus command (Week 4 - Cloud Gateway)
    async fn test_gwstatus_command(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  → TC-INT-CG-001: Testing gwstatus");

        let output = self
            .kernel_interface
            .execute_command("gwstatus")
            .await?;

        // Verify Cloud Gateway status structure
        let checks = vec![
            output.raw_output.contains("Cloud Gateway Status"),
            output.raw_output.contains("Request Statistics:"),
            output.raw_output.contains("Total Requests:"),
            output.raw_output.contains("Provider Statistics:"),
            output.raw_output.contains("Claude"),
            output.raw_output.contains("GPT-4"),
            output.raw_output.contains("Gemini"),
            output.raw_output.contains("Local"),
            output.raw_output.contains("Performance:"),
        ];

        let passed = checks.iter().all(|&c| c);

        if passed {
            log::info!("    ✓ Cloud Gateway status command working correctly");
        } else {
            log::error!("    ✗ Cloud Gateway status output incomplete");
        }

        Ok(passed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asm_test_details_default() {
        let details = ASMTestDetails::default();
        assert!(!details.status_command);
        assert!(!details.list_command);
        assert!(!details.metrics_command);
    }
}
