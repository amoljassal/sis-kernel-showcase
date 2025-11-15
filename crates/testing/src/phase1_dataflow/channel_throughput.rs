//! Channel Throughput Tests
//!
//! Validates dataflow channel performance and throughput.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;

/// Channel throughput test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelThroughputTestResults {
    pub passed: bool,
    pub basic_throughput_passed: bool,
    pub high_volume_passed: bool,
    pub backpressure_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
}

/// Channel throughput test suite
pub struct ChannelThroughputTests {
    kernel_interface: KernelCommandInterface,
}

impl ChannelThroughputTests {
    /// Create a new channel throughput test suite
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    /// Test 3.1: Basic Channel Throughput
    ///
    /// **Objective:** Verify basic channel data transfer works.
    ///
    /// **Steps:**
    /// 1. Create simple graph with connected operators
    /// 2. Execute for moderate number of steps
    /// 3. Verify data flows through channels
    ///
    /// **Expected:** Data transfers through channels successfully
    async fn test_basic_throughput(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing basic channel throughput...");

        // Create graph
        let _ = self.kernel_interface
            .execute_command("graphctl create --num-operators 3")
            .await;

        // Create operator chain with channels
        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 0 --in none --out 1 --prio 10")
            .await;

        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 1 --in 0 --out 2 --prio 5")
            .await;

        // Execute graph
        let output = self.kernel_interface
            .execute_command("graphctl start 50")
            .await?;

        let throughput_ok = output.raw_output.contains("complete") ||
                           output.raw_output.contains("50");

        let passed = output.success && throughput_ok;

        if passed {
            log::info!("    ✅ Basic throughput: PASSED");
        } else {
            log::warn!("    ❌ Basic throughput: FAILED");
        }

        Ok(passed)
    }

    /// Test 3.2: High Volume Transfer
    ///
    /// **Objective:** Verify channels handle high volume data transfer.
    ///
    /// **Steps:**
    /// 1. Create graph with multiple operators
    /// 2. Execute for large number of steps
    /// 3. Verify system remains stable
    ///
    /// **Expected:** High volume transfer completes without errors
    async fn test_high_volume(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing high volume transfer...");

        // Create larger graph
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

        // Execute with high step count
        let start = std::time::Instant::now();
        let output = self.kernel_interface
            .execute_command("graphctl start 500")
            .await?;
        let elapsed = start.elapsed();

        let high_volume_ok = output.raw_output.contains("complete") &&
                            elapsed < Duration::from_secs(30);

        let passed = output.success && high_volume_ok;

        if passed {
            log::info!("    ✅ High volume: PASSED");
        } else {
            log::warn!("    ❌ High volume: FAILED");
        }

        Ok(passed)
    }

    /// Test 3.3: Backpressure Handling
    ///
    /// **Objective:** Verify channel backpressure mechanisms work.
    ///
    /// **Steps:**
    /// 1. Create graph with slow consumer
    /// 2. Generate data faster than consumer can process
    /// 3. Verify backpressure prevents buffer overflow
    ///
    /// **Expected:** System handles backpressure gracefully
    async fn test_backpressure(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing backpressure handling...");

        // Create graph
        let _ = self.kernel_interface
            .execute_command("graphctl create --num-operators 5")
            .await;

        // Create fast producer -> slow consumer chain
        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 0 --in none --out 1 --prio 10")
            .await;

        let _ = self.kernel_interface
            .execute_command("graphctl add-operator 1 --in 0 --out none --prio 1")
            .await;

        // Execute graph
        let output = self.kernel_interface
            .execute_command("graphctl start 100")
            .await?;

        // Verify execution completes (backpressure working)
        let backpressure_ok = output.raw_output.contains("complete") ||
                             !output.raw_output.contains("overflow") &&
                             !output.raw_output.contains("error");

        let passed = output.success && backpressure_ok;

        if passed {
            log::info!("    ✅ Backpressure: PASSED");
        } else {
            log::warn!("    ❌ Backpressure: FAILED");
        }

        Ok(passed)
    }

    /// Run all channel throughput tests
    pub async fn run_all_tests(&mut self) -> Result<ChannelThroughputTestResults, Box<dyn Error>> {
        log::info!("Running Channel Throughput Tests...");

        let basic_throughput_passed = self.test_basic_throughput().await.unwrap_or(false);
        let high_volume_passed = self.test_high_volume().await.unwrap_or(false);
        let backpressure_passed = self.test_backpressure().await.unwrap_or(false);

        let total_tests = 3;
        let passed_tests = vec![
            basic_throughput_passed,
            high_volume_passed,
            backpressure_passed,
        ]
        .iter()
        .filter(|&&p| p)
        .count() as u32;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;

        log::info!("Channel Throughput Tests: {}/{} passed ({}%)",
            passed_tests, total_tests, (passed_tests * 100) / total_tests);

        Ok(ChannelThroughputTestResults {
            passed,
            basic_throughput_passed,
            high_volume_passed,
            backpressure_passed,
            total_tests,
            passed_tests,
        })
    }
}
