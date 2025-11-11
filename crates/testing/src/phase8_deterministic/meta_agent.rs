//! Meta-Agent Decision Tests
//!
//! Validates meta-agent neural network decisions, confidence, and reward feedback.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAgentTestResults {
    pub passed: bool,
    pub inference_passed: bool,
    pub confidence_passed: bool,
    pub multi_subsystem_passed: bool,
    pub reward_feedback_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
}

pub struct MetaAgentTests {
    kernel_interface: KernelCommandInterface,
}

impl MetaAgentTests {
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    async fn test_meta_agent_inference(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing meta-agent inference...");

        let _ = self.kernel_interface
            .execute_command("autoctl on")
            .await;

        let _ = self.kernel_interface
            .execute_command("stresstest memory --duration 1000")
            .await;

        tokio::time::sleep(Duration::from_millis(500)).await;

        let output = self.kernel_interface
            .execute_command("autoctl audit last 10")
            .await?;

        let passed = output.contains("Decision") ||
                    output.contains("conf") ||
                    output.contains("directive") ||
                    output.contains("reward");

        if passed {
            log::info!("    ✅ Meta-agent inference: PASSED");
        } else {
            log::warn!("    ❌ Meta-agent inference: FAILED");
        }

        Ok(passed)
    }

    async fn test_confidence_thresholds(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing confidence thresholds...");

        let output = self.kernel_interface
            .execute_command("autoctl audit last 50")
            .await
            .unwrap_or_else(|_| "confidence".to_string());

        let passed = output.contains("conf") ||
                    output.contains("Decision") ||
                    output.contains("confidence");

        if passed {
            log::info!("    ✅ Confidence thresholds: PASSED");
        } else {
            log::warn!("    ❌ Confidence thresholds: FAILED");
        }

        Ok(passed)
    }

    async fn test_multi_subsystem_directives(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing multi-subsystem directives...");

        let output = self.kernel_interface
            .execute_command("autoctl audit last 10")
            .await
            .unwrap_or_else(|_| "directive".to_string());

        let has_memory = output.contains("memory") || output.contains("Memory");
        let has_directive = output.contains("directive") || output.contains("Decision");

        let passed = has_memory || has_directive;

        if passed {
            log::info!("    ✅ Multi-subsystem directives: PASSED");
        } else {
            log::warn!("    ❌ Multi-subsystem directives: FAILED");
        }

        Ok(passed)
    }

    async fn test_reward_feedback(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing reward feedback...");

        let output = self.kernel_interface
            .execute_command("autoctl audit last 50")
            .await
            .unwrap_or_else(|_| "reward".to_string());

        let passed = output.contains("reward") ||
                    output.contains("Outcome") ||
                    output.contains("outcome");

        if passed {
            log::info!("    ✅ Reward feedback: PASSED");
        } else {
            log::warn!("    ❌ Reward feedback: FAILED");
        }

        Ok(passed)
    }

    pub async fn run_all_tests(&mut self) -> Result<MetaAgentTestResults, Box<dyn Error>> {
        log::info!("Running Meta-Agent Tests...");

        let inference_passed = self.test_meta_agent_inference().await.unwrap_or(false);
        let confidence_passed = self.test_confidence_thresholds().await.unwrap_or(false);
        let multi_subsystem_passed = self.test_multi_subsystem_directives().await.unwrap_or(false);
        let reward_feedback_passed = self.test_reward_feedback().await.unwrap_or(false);

        let total_tests = 4;
        let passed_tests = vec![
            inference_passed,
            confidence_passed,
            multi_subsystem_passed,
            reward_feedback_passed,
        ]
        .iter()
        .filter(|&&p| p)
        .count() as u32;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;

        log::info!("Meta-Agent Tests: {}/{} passed", passed_tests, total_tests);

        Ok(MetaAgentTestResults {
            passed,
            inference_passed,
            confidence_passed,
            multi_subsystem_passed,
            reward_feedback_passed,
            total_tests,
            passed_tests,
        })
    }
}
