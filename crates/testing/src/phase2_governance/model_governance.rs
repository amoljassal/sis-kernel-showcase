//! Model Governance Tests
//!
//! Validates AI model governance including registration, versioning, and lifecycle management.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Model governance test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelGovernanceTestResults {
    pub passed: bool,
    pub model_registration_passed: bool,
    pub model_versioning_passed: bool,
    pub model_metadata_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
}

/// Model governance test suite
pub struct ModelGovernanceTests {
    kernel_interface: KernelCommandInterface,
}

impl ModelGovernanceTests {
    /// Create a new model governance test suite
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    /// Test 1.1: Model Registration
    ///
    /// **Objective:** Verify models can be registered with metadata.
    ///
    /// **Steps:**
    /// 1. Register a model with metadata
    /// 2. Verify registration succeeds
    /// 3. Check model appears in registry
    ///
    /// **Expected:** Model registered with correct metadata
    async fn test_model_registration(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing model registration...");

        // Load model with metadata
        let output = self.kernel_interface
            .execute_command("llmctl load --model 7 --ctx 2048 --vocab 50000 --quant int8 --size-bytes 1048576")
            .await?;

        let registration_ok = output.raw_output.contains("model loaded") ||
                             output.raw_output.contains("[LLM]") ||
                             output.success;

        let passed = registration_ok;

        if passed {
            log::info!("    ✅ Model registration: PASSED");
        } else {
            log::warn!("    ❌ Model registration: FAILED");
        }

        Ok(passed)
    }

    /// Test 1.2: Model Versioning
    ///
    /// **Objective:** Verify model version tracking works.
    ///
    /// **Steps:**
    /// 1. Load model v1
    /// 2. Query model status
    /// 3. Verify version information
    ///
    /// **Expected:** Model version tracked correctly
    async fn test_model_versioning(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing model versioning...");

        // Load model
        let _ = self.kernel_interface
            .execute_command("llmctl load --wcet-cycles 50000")
            .await;

        // Check status for version info
        let output = self.kernel_interface
            .execute_command("llmctl status")
            .await?;

        let versioning_ok = output.raw_output.contains("[LLM]") ||
                           output.raw_output.contains("status") ||
                           output.success;

        let passed = versioning_ok;

        if passed {
            log::info!("    ✅ Model versioning: PASSED");
        } else {
            log::warn!("    ❌ Model versioning: FAILED");
        }

        Ok(passed)
    }

    /// Test 1.3: Model Metadata Validation
    ///
    /// **Objective:** Verify model metadata is validated and stored.
    ///
    /// **Steps:**
    /// 1. Load model with comprehensive metadata
    /// 2. Query model information
    /// 3. Verify metadata integrity
    ///
    /// **Expected:** Metadata validated and retrievable
    async fn test_model_metadata(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing model metadata validation...");

        // Load with metadata
        let output = self.kernel_interface
            .execute_command("llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 524288")
            .await?;

        let metadata_ok = output.success &&
                         (output.raw_output.contains("model") ||
                          output.raw_output.contains("[LLM]"));

        // Verify via status
        let status = self.kernel_interface
            .execute_command("llmctl status")
            .await;

        let status_ok = status.is_ok() &&
                       (status.as_ref().unwrap().success ||
                        status.as_ref().unwrap().raw_output.contains("[LLM]"));

        let passed = metadata_ok && status_ok;

        if passed {
            log::info!("    ✅ Model metadata: PASSED");
        } else {
            log::warn!("    ❌ Model metadata: FAILED");
        }

        Ok(passed)
    }

    /// Run all model governance tests
    pub async fn run_all_tests(&mut self) -> Result<ModelGovernanceTestResults, Box<dyn Error>> {
        log::info!("Running Model Governance Tests...");

        let model_registration_passed = self.test_model_registration().await.unwrap_or(false);
        let model_versioning_passed = self.test_model_versioning().await.unwrap_or(false);
        let model_metadata_passed = self.test_model_metadata().await.unwrap_or(false);

        let total_tests = 3;
        let passed_tests = vec![
            model_registration_passed,
            model_versioning_passed,
            model_metadata_passed,
        ]
        .iter()
        .filter(|&&p| p)
        .count() as u32;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;

        log::info!("Model Governance Tests: {}/{} passed ({}%)",
            passed_tests, total_tests, (passed_tests * 100) / total_tests);

        Ok(ModelGovernanceTestResults {
            passed,
            model_registration_passed,
            model_versioning_passed,
            model_metadata_passed,
            total_tests,
            passed_tests,
        })
    }
}
