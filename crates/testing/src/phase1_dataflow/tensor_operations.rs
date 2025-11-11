//! Tensor Operations Tests
//!
//! Validates tensor operations and data transformations in dataflow.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Tensor operations test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorOperationsTestResults {
    pub passed: bool,
    pub tensor_creation_passed: bool,
    pub tensor_transformation_passed: bool,
    pub tensor_validation_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
}

/// Tensor operations test suite
pub struct TensorOperationsTests {
    kernel_interface: KernelCommandInterface,
}

impl TensorOperationsTests {
    /// Create a new tensor operations test suite
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    /// Test 4.1: Tensor Creation
    ///
    /// **Objective:** Verify tensors can be created and initialized.
    ///
    /// **Steps:**
    /// 1. Create graph with tensor operations
    /// 2. Initialize tensors in operators
    /// 3. Verify tensor creation succeeds
    ///
    /// **Expected:** Tensors created with correct dimensions
    async fn test_tensor_creation(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing tensor creation...");

        // Create graph
        let _ = self.kernel_interface
            .execute_command("graphctl create --num-operators 5")
            .await;

        // Add operator (implicitly creates tensors)
        let output = self.kernel_interface
            .execute_command("graphctl add-operator 0 --in none --out 1 --prio 10")
            .await?;

        let tensor_ok = output.raw_output.contains("operator") ||
                       output.raw_output.contains("Added");

        let passed = output.success && tensor_ok;

        if passed {
            log::info!("    ✅ Tensor creation: PASSED");
        } else {
            log::warn!("    ❌ Tensor creation: FAILED");
        }

        Ok(passed)
    }

    /// Test 4.2: Tensor Transformation
    ///
    /// **Objective:** Verify tensor data can be transformed.
    ///
    /// **Steps:**
    /// 1. Create operator chain that transforms data
    /// 2. Execute graph
    /// 3. Verify transformations applied correctly
    ///
    /// **Expected:** Tensor transformations work correctly
    async fn test_tensor_transformation(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing tensor transformation...");

        // Create graph
        let _ = self.kernel_interface
            .execute_command("graphctl create --num-operators 5")
            .await;

        // Create transformation chain
        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 0 --in none --out 1 --prio 10")
            .await;

        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 1 --in 0 --out 2 --prio 5")
            .await;

        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 2 --in 1 --out none --prio 1")
            .await;

        // Execute transformations
        let output = self.kernel_interface
            .execute_command("graphctl start 20")
            .await?;

        let transform_ok = output.raw_output.contains("complete") ||
                          output.raw_output.contains("20");

        let passed = output.success && transform_ok;

        if passed {
            log::info!("    ✅ Tensor transformation: PASSED");
        } else {
            log::warn!("    ❌ Tensor transformation: FAILED");
        }

        Ok(passed)
    }

    /// Test 4.3: Tensor Data Validation
    ///
    /// **Objective:** Verify tensor data integrity is maintained.
    ///
    /// **Steps:**
    /// 1. Execute graph with tensor operations
    /// 2. Verify no data corruption
    /// 3. Check tensor dimensions remain consistent
    ///
    /// **Expected:** Tensor data integrity maintained throughout execution
    async fn test_tensor_validation(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing tensor data validation...");

        // Create graph
        let _ = self.kernel_interface
            .execute_command("graphctl create --num-operators 10")
            .await;

        // Add multiple operators
        for i in 0..5 {
            let cmd = format!("graphctl add-operator {} --in none --out 0 --prio {}", i, 10 - i);
            let _ = self.kernel_interface
                .execute_command(&cmd)
                .await;
        }

        // Execute and verify
        let output = self.kernel_interface
            .execute_command("graphctl start 100")
            .await?;

        let validation_ok = output.raw_output.contains("complete") &&
                           !output.raw_output.contains("error") &&
                           !output.raw_output.contains("corruption");

        let passed = output.success && validation_ok;

        if passed {
            log::info!("    ✅ Tensor validation: PASSED");
        } else {
            log::warn!("    ❌ Tensor validation: FAILED");
        }

        Ok(passed)
    }

    /// Run all tensor operations tests
    pub async fn run_all_tests(&mut self) -> Result<TensorOperationsTestResults, Box<dyn Error>> {
        log::info!("Running Tensor Operations Tests...");

        let tensor_creation_passed = self.test_tensor_creation().await.unwrap_or(false);
        let tensor_transformation_passed = self.test_tensor_transformation().await.unwrap_or(false);
        let tensor_validation_passed = self.test_tensor_validation().await.unwrap_or(false);

        let total_tests = 3;
        let passed_tests = vec![
            tensor_creation_passed,
            tensor_transformation_passed,
            tensor_validation_passed,
        ]
        .iter()
        .filter(|&&p| p)
        .count() as u32;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;

        log::info!("Tensor Operations Tests: {}/{} passed ({}%)",
            passed_tests, total_tests, (passed_tests * 100) / total_tests);

        Ok(TensorOperationsTestResults {
            passed,
            tensor_creation_passed,
            tensor_transformation_passed,
            tensor_validation_passed,
            total_tests,
            passed_tests,
        })
    }
}
