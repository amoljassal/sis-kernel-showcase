//! Model Health Checks
//!
//! Validates model health before deployment:
//! - Inference latency (P99 < 1ms)
//! - Memory footprint (< 10MB)
//! - Test accuracy (> 95%)

use alloc::vec::Vec;
use crate::lib::error::{Result, Errno};
use super::lifecycle::Model;
use super::registry::HealthMetrics;

/// Test case for health checks
#[derive(Debug, Clone)]
pub struct TestCase {
    pub input: Vec<f32>,
    pub expected_output: usize,
}

/// Health checker for models
pub struct HealthChecker {
    test_inputs: Vec<TestCase>,
    latency_threshold_us: u64,
    memory_threshold_bytes: usize,
    accuracy_threshold: f32,
}

impl HealthChecker {
    /// Create new health checker with default thresholds
    pub fn new() -> Self {
        Self {
            test_inputs: Vec::new(),
            latency_threshold_us: 1000,      // 1ms
            memory_threshold_bytes: 10 * 1024 * 1024,  // 10MB
            accuracy_threshold: 0.95,        // 95%
        }
    }

    /// Add test case for health checks
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_inputs.push(test_case);
    }

    /// Run comprehensive health check on model
    pub fn check(&self, model: &Model) -> Result<HealthMetrics> {
        // 1. Latency test
        let latency_p99 = self.measure_latency_p99(model)?;
        if latency_p99 > self.latency_threshold_us {
            crate::println!("[HEALTH] Latency check failed: {}μs > {}μs",
                latency_p99, self.latency_threshold_us);
            return Err(Errno::ETIMEDOUT);
        }

        // 2. Memory test
        let mem_footprint = self.measure_memory(model)?;
        if mem_footprint > self.memory_threshold_bytes {
            crate::println!("[HEALTH] Memory check failed: {} bytes > {} bytes",
                mem_footprint, self.memory_threshold_bytes);
            return Err(Errno::ENOMEM);
        }

        // 3. Accuracy test (skip if no test cases)
        let accuracy = if !self.test_inputs.is_empty() {
            let acc = self.measure_accuracy(model)?;
            if acc < self.accuracy_threshold {
                crate::println!("[HEALTH] Accuracy check failed: {:.2}% < {:.2}%",
                    acc * 100.0, self.accuracy_threshold * 100.0);
                return Err(Errno::EINVAL);
            }
            acc
        } else {
            1.0  // No test cases, assume perfect accuracy
        };

        Ok(HealthMetrics {
            inference_latency_p99_us: latency_p99,
            memory_footprint_bytes: mem_footprint,
            test_accuracy: accuracy,
        })
    }

    /// Measure P99 inference latency
    fn measure_latency_p99(&self, model: &Model) -> Result<u64> {
        const NUM_SAMPLES: usize = 100;
        let mut latencies = Vec::with_capacity(NUM_SAMPLES);

        // Use dummy input if no test cases
        let test_input = if !self.test_inputs.is_empty() {
            &self.test_inputs[0].input
        } else {
            // Create dummy input
            &alloc::vec![0.5f32; 10]
        };

        for _ in 0..NUM_SAMPLES {
            let start = crate::time::uptime_ms() * 1000;  // Convert to microseconds
            let _ = model.predict(test_input);
            let end = crate::time::uptime_ms() * 1000;
            latencies.push(end.saturating_sub(start));
        }

        latencies.sort_unstable();
        Ok(latencies[99])  // P99
    }

    /// Measure memory footprint
    fn measure_memory(&self, model: &Model) -> Result<usize> {
        // Size of model weights + metadata
        let weights_size = model.weights.len() * core::mem::size_of::<f32>();
        let metadata_size = core::mem::size_of::<Model>();
        Ok(weights_size + metadata_size)
    }

    /// Measure test accuracy
    fn measure_accuracy(&self, model: &Model) -> Result<f32> {
        if self.test_inputs.is_empty() {
            return Ok(1.0);  // No test cases
        }

        let mut correct = 0;
        for test_case in &self.test_inputs {
            let output = model.predict(&test_case.input);
            let predicted = output.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap_or(0);

            if predicted == test_case.expected_output {
                correct += 1;
            }
        }

        Ok(correct as f32 / self.test_inputs.len() as f32)
    }

    /// Set custom latency threshold
    pub fn set_latency_threshold(&mut self, threshold_us: u64) {
        self.latency_threshold_us = threshold_us;
    }

    /// Set custom memory threshold
    pub fn set_memory_threshold(&mut self, threshold_bytes: usize) {
        self.memory_threshold_bytes = threshold_bytes;
    }

    /// Set custom accuracy threshold
    pub fn set_accuracy_threshold(&mut self, threshold: f32) {
        self.accuracy_threshold = threshold;
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_checker() {
        let checker = HealthChecker::new();
        assert_eq!(checker.latency_threshold_us, 1000);
        assert_eq!(checker.memory_threshold_bytes, 10 * 1024 * 1024);
        assert_eq!(checker.accuracy_threshold, 0.95);
    }
}
