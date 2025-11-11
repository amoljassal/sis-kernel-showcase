//! Phase 1: AI-Native Dataflow Tests
//!
//! Complete validation of AI-native dataflow including graph execution,
//! operator validation, channel throughput, and tensor operations.

pub mod graph_execution;
pub mod operator_validation;
pub mod channel_throughput;
pub mod tensor_operations;

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Phase 1 test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase1Results {
    pub passed: bool,
    pub graph_execution_passed: bool,
    pub operator_validation_passed: bool,
    pub channel_throughput_passed: bool,
    pub tensor_operations_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub overall_score: f64,
    pub timestamp: String,
}

/// Phase 1 dataflow test suite
pub struct Phase1DataflowSuite {
    kernel_interface: KernelCommandInterface,
    graph_execution: graph_execution::GraphExecutionTests,
    operator_validation: operator_validation::OperatorValidationTests,
    channel_throughput: channel_throughput::ChannelThroughputTests,
    tensor_operations: tensor_operations::TensorOperationsTests,
}

impl Phase1DataflowSuite {
    /// Create a new Phase 1 test suite
    pub fn new(serial_log_path: String, monitor_port: u16) -> Self {
        Self {
            kernel_interface: KernelCommandInterface::new(serial_log_path.clone(), monitor_port),
            graph_execution: graph_execution::GraphExecutionTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            operator_validation: operator_validation::OperatorValidationTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            channel_throughput: channel_throughput::ChannelThroughputTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
            tensor_operations: tensor_operations::TensorOperationsTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), monitor_port)
            ),
        }
    }

    /// Run all Phase 1 tests
    pub async fn validate_phase1(&mut self) -> Result<Phase1Results, Box<dyn Error>> {
        log::info!("==================================================");
        log::info!("Starting Phase 1: AI-Native Dataflow Validation");
        log::info!("==================================================");

        // Run all test modules
        let graph_result = self.graph_execution.run_all_tests().await?;
        let operator_result = self.operator_validation.run_all_tests().await?;
        let channel_result = self.channel_throughput.run_all_tests().await?;
        let tensor_result = self.tensor_operations.run_all_tests().await?;

        // Calculate overall results
        let graph_execution_passed = graph_result.passed;
        let operator_validation_passed = operator_result.passed;
        let channel_throughput_passed = channel_result.passed;
        let tensor_operations_passed = tensor_result.passed;

        let total_tests = graph_result.total_tests +
                         operator_result.total_tests +
                         channel_result.total_tests +
                         tensor_result.total_tests;

        let passed_tests = graph_result.passed_tests +
                          operator_result.passed_tests +
                          channel_result.passed_tests +
                          tensor_result.passed_tests;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;
        let overall_score = (passed_tests as f64 / total_tests as f64) * 100.0;

        log::info!("==================================================");
        log::info!("Phase 1 Summary:");
        log::info!("  Graph Execution:      {}", if graph_execution_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Operator Validation:  {}", if operator_validation_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Channel Throughput:   {}", if channel_throughput_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Tensor Operations:    {}", if tensor_operations_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Overall:              {}/{} tests passed ({:.1}%)",
            passed_tests, total_tests, overall_score);
        log::info!("==================================================");

        Ok(Phase1Results {
            passed,
            graph_execution_passed,
            operator_validation_passed,
            channel_throughput_passed,
            tensor_operations_passed,
            total_tests,
            passed_tests,
            overall_score,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

impl Default for Phase1Results {
    fn default() -> Self {
        Self {
            passed: false,
            graph_execution_passed: false,
            operator_validation_passed: false,
            channel_throughput_passed: false,
            tensor_operations_passed: false,
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
    fn test_phase1_results_default() {
        let results = Phase1Results::default();
        assert_eq!(results.overall_score, 0.0);
        assert!(!results.passed);
    }
}
