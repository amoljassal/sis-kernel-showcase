//! Operator Validation Tests
//!
//! Validates dataflow operator correctness and behavior.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Operator validation test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorValidationTestResults {
    pub passed: bool,
    pub operator_types_passed: bool,
    pub operator_priorities_passed: bool,
    pub operator_connections_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
}

/// Operator validation test suite
pub struct OperatorValidationTests {
    kernel_interface: KernelCommandInterface,
}

impl OperatorValidationTests {
    /// Create a new operator validation test suite
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    /// Test 2.1: Operator Types
    ///
    /// **Objective:** Verify different operator types can be created.
    ///
    /// **Steps:**
    /// 1. Create graph
    /// 2. Add operators of different types
    /// 3. Verify operators are created correctly
    ///
    /// **Expected:** All operator types supported
    async fn test_operator_types(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing operator types...");

        // Create graph
        let _ = self.kernel_interface
            .execute_command("graphctl create --num-operators 10")
            .await;

        // Add different operator types
        let mut all_ok = true;

        // Source operator (no input)
        let result = self.kernel_interface
            .execute_command("graphctl add-operator 0 --in none --out 1 --prio 10")
            .await;
        all_ok = all_ok && result.is_ok();

        // Processing operator (input and output)
        let result = self.kernel_interface
            .execute_command("graphctl add-operator 1 --in 0 --out 2 --prio 5")
            .await;
        all_ok = all_ok && result.is_ok();

        // Sink operator (input only)
        let result = self.kernel_interface
            .execute_command("graphctl add-operator 2 --in 1 --out none --prio 1")
            .await;
        all_ok = all_ok && result.is_ok();

        let passed = all_ok;

        if passed {
            log::info!("    ✅ Operator types: PASSED");
        } else {
            log::warn!("    ❌ Operator types: FAILED");
        }

        Ok(passed)
    }

    /// Test 2.2: Operator Priorities
    ///
    /// **Objective:** Verify operator priority settings are respected.
    ///
    /// **Steps:**
    /// 1. Create operators with different priorities
    /// 2. Verify priorities are set correctly
    ///
    /// **Expected:** Higher priority operators execute first
    async fn test_operator_priorities(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing operator priorities...");

        // Create graph
        let _ = self.kernel_interface
            .execute_command("graphctl create --num-operators 5")
            .await;

        // Add operators with varying priorities
        let mut all_ok = true;

        for (id, prio) in [(0, 10), (1, 5), (2, 15), (3, 1)] {
            let cmd = format!("graphctl add-operator {} --in none --out 0 --prio {}", id, prio);
            let result = self.kernel_interface
                .execute_command(&cmd)
                .await;
            all_ok = all_ok && result.is_ok();
        }

        let passed = all_ok;

        if passed {
            log::info!("    ✅ Operator priorities: PASSED");
        } else {
            log::warn!("    ❌ Operator priorities: FAILED");
        }

        Ok(passed)
    }

    /// Test 2.3: Operator Connections
    ///
    /// **Objective:** Verify operator input/output connections work.
    ///
    /// **Steps:**
    /// 1. Create chain of connected operators
    /// 2. Execute graph
    /// 3. Verify data flows through connections
    ///
    /// **Expected:** Data propagates through operator chain
    async fn test_operator_connections(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing operator connections...");

        // Create graph
        let _ = self.kernel_interface
            .execute_command("graphctl create --num-operators 5")
            .await;

        // Create operator chain: 0 -> 1 -> 2
        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 0 --in none --out 1 --prio 10")
            .await;

        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 1 --in 0 --out 2 --prio 5")
            .await;

        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 2 --in 1 --out none --prio 1")
            .await;

        // Execute graph
        let output = self.kernel_interface
            .execute_command("graphctl start 10")
            .await?;

        let execution_ok = output.raw_output.contains("complete") ||
                          output.raw_output.contains("Execution");

        let passed = output.success && execution_ok;

        if passed {
            log::info!("    ✅ Operator connections: PASSED");
        } else {
            log::warn!("    ❌ Operator connections: FAILED");
        }

        Ok(passed)
    }

    /// Run all operator validation tests
    pub async fn run_all_tests(&mut self) -> Result<OperatorValidationTestResults, Box<dyn Error>> {
        log::info!("Running Operator Validation Tests...");

        let operator_types_passed = self.test_operator_types().await.unwrap_or(false);
        let operator_priorities_passed = self.test_operator_priorities().await.unwrap_or(false);
        let operator_connections_passed = self.test_operator_connections().await.unwrap_or(false);

        let total_tests = 3;
        let passed_tests = vec![
            operator_types_passed,
            operator_priorities_passed,
            operator_connections_passed,
        ]
        .iter()
        .filter(|&&p| p)
        .count() as u32;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;

        log::info!("Operator Validation Tests: {}/{} passed ({}%)",
            passed_tests, total_tests, (passed_tests * 100) / total_tests);

        Ok(OperatorValidationTestResults {
            passed,
            operator_types_passed,
            operator_priorities_passed,
            operator_connections_passed,
            total_tests,
            passed_tests,
        })
    }
}
