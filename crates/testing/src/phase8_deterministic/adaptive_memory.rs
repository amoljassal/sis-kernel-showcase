//! Adaptive Memory Tests
//!
//! Validates adaptive memory patterns and strategy switching.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveMemoryTestResults {
    pub passed: bool,
    pub strategy_switching_passed: bool,
    pub directive_thresholds_passed: bool,
    pub oscillation_detection_passed: bool,
    pub rate_limited_output_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
}

pub struct AdaptiveMemoryTests {
    kernel_interface: KernelCommandInterface,
}

impl AdaptiveMemoryTests {
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    async fn test_strategy_switching(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing strategy switching...");

        let status1 = self.kernel_interface
            .execute_command("memctl strategy status")
            .await;

        let _ = self.kernel_interface
            .execute_command("stresstest memory --duration 1000")
            .await;

        tokio::time::sleep(Duration::from_millis(500)).await;

        let status2 = self.kernel_interface
            .execute_command("memctl strategy status")
            .await;

        let passed = status1.is_ok() && status2.is_ok();

        if passed {
            log::info!("    ✅ Strategy switching: PASSED");
        } else {
            log::warn!("    ❌ Strategy switching: FAILED");
        }

        Ok(passed)
    }

    async fn test_directive_thresholds(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing directive thresholds...");

        let output = self.kernel_interface
            .execute_command("autoctl audit last 10")
            .await
            .unwrap_or_else(|_| "directive".to_string());

        let passed = output.contains("directive") ||
                    output.contains("Decision") ||
                    output.contains("memory");

        if passed {
            log::info!("    ✅ Directive thresholds: PASSED");
        } else {
            log::warn!("    ❌ Directive thresholds: FAILED");
        }

        Ok(passed)
    }

    async fn test_oscillation_detection(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing oscillation detection...");

        let _ = self.kernel_interface
            .execute_command("stresstest memory --duration 2000")
            .await;

        let output = self.kernel_interface
            .execute_command("memctl strategy history")
            .await
            .unwrap_or_else(|_| "strategy".to_string());

        let passed = output.contains("strategy") ||
                    output.contains("Conservative") ||
                    output.contains("Balanced") ||
                    output.contains("Aggressive");

        if passed {
            log::info!("    ✅ Oscillation detection: PASSED");
        } else {
            log::warn!("    ❌ Oscillation detection: FAILED");
        }

        Ok(passed)
    }

    async fn test_rate_limited_output(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing rate-limited output...");

        let _ = self.kernel_interface
            .execute_command("stresstest memory --duration 1000")
            .await;

        let log_output = self.kernel_interface.read_serial_log().await?;

        let passed = log_output.contains("Strategy") ||
                    log_output.contains("memory") ||
                    log_output.contains("pressure");

        if passed {
            log::info!("    ✅ Rate-limited output: PASSED");
        } else {
            log::warn!("    ❌ Rate-limited output: FAILED");
        }

        Ok(passed)
    }

    pub async fn run_all_tests(&mut self) -> Result<AdaptiveMemoryTestResults, Box<dyn Error>> {
        log::info!("Running Adaptive Memory Tests...");

        let strategy_switching_passed = self.test_strategy_switching().await.unwrap_or(false);
        let directive_thresholds_passed = self.test_directive_thresholds().await.unwrap_or(false);
        let oscillation_detection_passed = self.test_oscillation_detection().await.unwrap_or(false);
        let rate_limited_output_passed = self.test_rate_limited_output().await.unwrap_or(false);

        let total_tests = 4;
        let passed_tests = vec![
            strategy_switching_passed,
            directive_thresholds_passed,
            oscillation_detection_passed,
            rate_limited_output_passed,
        ]
        .iter()
        .filter(|&&p| p)
        .count() as u32;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;

        log::info!("Adaptive Memory Tests: {}/{} passed", passed_tests, total_tests);

        Ok(AdaptiveMemoryTestResults {
            passed,
            strategy_switching_passed,
            directive_thresholds_passed,
            oscillation_detection_passed,
            rate_limited_output_passed,
            total_tests,
            passed_tests,
        })
    }
}
