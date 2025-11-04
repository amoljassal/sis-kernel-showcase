//! Test utilities and mocks for kernel unit testing
//!
//! This module provides mock implementations and test fixtures for testing
//! kernel components in isolation without requiring the full kernel environment.
//!
//! # Usage
//!
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     use crate::test_utils::*;
//!
//!     #[test]
//!     fn test_my_function() {
//!         let mut mock_uart = MockUart::new();
//!         mock_uart.expect_write("Hello");
//!         // ... test code
//!     }
//! }
//! ```

use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

/// Mock UART for testing output without real hardware
///
/// Captures writes to verify kernel output in tests
pub struct MockUart {
    pub writes: Vec<Vec<u8>>,
    pub expected_writes: Vec<Vec<u8>>,
}

impl MockUart {
    /// Create a new mock UART
    pub fn new() -> Self {
        Self {
            writes: Vec::new(),
            expected_writes: Vec::new(),
        }
    }

    /// Write bytes to mock UART (captures for verification)
    pub fn write(&mut self, data: &[u8]) {
        self.writes.push(data.to_vec());
    }

    /// Expect a specific write to occur
    pub fn expect_write(&mut self, expected: &str) {
        self.expected_writes.push(expected.as_bytes().to_vec());
    }

    /// Verify all expected writes occurred
    pub fn verify(&self) -> bool {
        if self.writes.len() != self.expected_writes.len() {
            return false;
        }
        for (actual, expected) in self.writes.iter().zip(self.expected_writes.iter()) {
            if actual != expected {
                return false;
            }
        }
        true
    }

    /// Get all captured writes as strings
    pub fn get_writes(&self) -> Vec<String> {
        self.writes
            .iter()
            .map(|w| String::from_utf8_lossy(w).to_string())
            .collect()
    }

    /// Clear all captured writes
    pub fn clear(&mut self) {
        self.writes.clear();
        self.expected_writes.clear();
    }
}

impl Default for MockUart {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock timer for testing time-dependent code
///
/// Provides controllable time for deterministic testing
pub struct MockTimer {
    current_time_us: AtomicU64,
    timer_frequency: u64,
}

impl MockTimer {
    /// Create a new mock timer with specified frequency
    pub fn new(frequency_hz: u64) -> Self {
        Self {
            current_time_us: AtomicU64::new(0),
            timer_frequency: frequency_hz,
        }
    }

    /// Get current timestamp in microseconds
    pub fn get_timestamp_us(&self) -> u64 {
        self.current_time_us.load(Ordering::SeqCst)
    }

    /// Advance time by specified microseconds
    pub fn advance_us(&self, delta_us: u64) {
        self.current_time_us.fetch_add(delta_us, Ordering::SeqCst);
    }

    /// Advance time by specified milliseconds
    pub fn advance_ms(&self, delta_ms: u64) {
        self.advance_us(delta_ms * 1000);
    }

    /// Reset timer to zero
    pub fn reset(&self) {
        self.current_time_us.store(0, Ordering::SeqCst);
    }

    /// Get timer frequency in Hz
    pub fn frequency_hz(&self) -> u64 {
        self.timer_frequency
    }
}

impl Default for MockTimer {
    fn default() -> Self {
        Self::new(62_500_000) // Default QEMU timer frequency
    }
}

/// Test fixture for creating standard test inputs
pub struct TestFixture;

impl TestFixture {
    /// Create a test neural network input (4-element vector)
    pub fn neural_input_4() -> [f32; 4] {
        [0.5, 0.3, 0.8, 0.2]
    }

    /// Create a test neural network input (8-element vector)
    pub fn neural_input_8() -> [f32; 8] {
        [0.5, 0.3, 0.8, 0.2, 0.6, 0.4, 0.7, 0.1]
    }

    /// Create a test target output (single value)
    pub fn neural_target_1() -> [f32; 1] {
        [1.0]
    }

    /// Create a test target output (multi-class, 16 classes)
    pub fn neural_target_16() -> [f32; 16] {
        let mut target = [0.0; 16];
        target[5] = 1.0; // Class 5 is the target
        target
    }

    /// Create test memory pattern (alternating values)
    pub fn memory_pattern(size: usize) -> Vec<u8> {
        (0..size).map(|i| (i % 256) as u8).collect()
    }

    /// Create test command history (for command prediction)
    pub fn command_history_5() -> [u8; 5] {
        [1, 2, 3, 2, 1] // Simple repeating pattern
    }
}

/// Assertion helpers for floating point comparison
pub mod assert_helpers {
    /// Assert two floats are approximately equal (within epsilon)
    pub fn assert_approx_eq(a: f32, b: f32, epsilon: f32) {
        assert!(
            (a - b).abs() < epsilon,
            "assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`,\n delta: `{:?}`,\n epsilon: `{:?}`",
            a, b, (a - b).abs(), epsilon
        );
    }

    /// Assert float is within range [min, max]
    pub fn assert_in_range(value: f32, min: f32, max: f32) {
        assert!(
            value >= min && value <= max,
            "assertion failed: `(min <= value <= max)`\n  value: `{:?}`,\n  min: `{:?}`,\n  max: `{:?}`",
            value, min, max
        );
    }

    /// Assert float array elements are all within range [0.0, 1.0]
    pub fn assert_normalized(arr: &[f32]) {
        for (i, &val) in arr.iter().enumerate() {
            assert_in_range(val, 0.0, 1.0);
        }
    }

    /// Assert monotonic decrease (e.g., loss function)
    pub fn assert_decreasing(values: &[f32]) {
        for i in 1..values.len() {
            assert!(
                values[i] <= values[i - 1],
                "assertion failed: values not decreasing at index {}: {} > {}",
                i, values[i], values[i - 1]
            );
        }
    }
}

/// Statistical helpers for test validation
pub mod stats {
    /// Calculate mean of a slice
    pub fn mean(values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f32>() / values.len() as f32
    }

    /// Calculate standard deviation
    pub fn std_dev(values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        let m = mean(values);
        let variance = values.iter()
            .map(|v| (v - m).powi(2))
            .sum::<f32>() / values.len() as f32;
        variance.sqrt()
    }

    /// Calculate min value
    pub fn min(values: &[f32]) -> f32 {
        values.iter().copied().fold(f32::INFINITY, f32::min)
    }

    /// Calculate max value
    pub fn max(values: &[f32]) -> f32 {
        values.iter().copied().fold(f32::NEG_INFINITY, f32::max)
    }

    /// Calculate percentile (0.0-1.0)
    pub fn percentile(values: &[f32], p: f32) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let index = (p * (sorted.len() - 1) as f32).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// Property-based testing helpers
pub mod property {
    use super::*;

    /// Generate random f32 in range [0.0, 1.0] (deterministic seed)
    pub fn rand_f32(seed: &mut u64) -> f32 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*seed >> 32) as f32 / u32::MAX as f32
    }

    /// Generate random f32 array
    pub fn rand_f32_array<const N: usize>(seed: &mut u64) -> [f32; N] {
        let mut arr = [0.0; N];
        for i in 0..N {
            arr[i] = rand_f32(seed);
        }
        arr
    }

    /// Generate random input in range [min, max]
    pub fn rand_range(seed: &mut u64, min: f32, max: f32) -> f32 {
        min + rand_f32(seed) * (max - min)
    }

    /// Check property holds for N random inputs
    pub fn check_property<F>(n: usize, mut property: F) -> bool
    where
        F: FnMut(f32) -> bool,
    {
        let mut seed = 42u64;
        for _ in 0..n {
            let input = rand_f32(&mut seed);
            if !property(input) {
                return false;
            }
        }
        true
    }

    /// Check property holds for N random arrays
    pub fn check_property_array<F, const N: usize>(count: usize, mut property: F) -> bool
    where
        F: FnMut([f32; N]) -> bool,
    {
        let mut seed = 42u64;
        for _ in 0..count {
            let input = rand_f32_array(&mut seed);
            if !property(input) {
                return false;
            }
        }
        true
    }
}

/// Benchmark helpers for performance testing
pub mod benchmark {
    use super::MockTimer;

    /// Measure execution time of a function
    pub fn measure<F, R>(timer: &MockTimer, f: F) -> (R, u64)
    where
        F: FnOnce() -> R,
    {
        let start = timer.get_timestamp_us();
        let result = f();
        let end = timer.get_timestamp_us();
        (result, end - start)
    }

    /// Run benchmark N times and return mean execution time
    pub fn benchmark<F>(timer: &MockTimer, n: usize, mut f: F) -> u64
    where
        F: FnMut(),
    {
        let start = timer.get_timestamp_us();
        for _ in 0..n {
            f();
        }
        let end = timer.get_timestamp_us();
        (end - start) / n as u64
    }

    /// Run benchmark and return statistics (mean, min, max, stddev)
    pub fn benchmark_stats<F>(timer: &MockTimer, n: usize, mut f: F) -> BenchmarkStats
    where
        F: FnMut(),
    {
        let mut times = Vec::with_capacity(n);
        for _ in 0..n {
            let start = timer.get_timestamp_us();
            f();
            let end = timer.get_timestamp_us();
            times.push((end - start) as f32);
        }

        use super::stats;
        BenchmarkStats {
            mean: stats::mean(&times) as u64,
            min: stats::min(&times) as u64,
            max: stats::max(&times) as u64,
            std_dev: stats::std_dev(&times) as u64,
            samples: n,
        }
    }

    pub struct BenchmarkStats {
        pub mean: u64,
        pub min: u64,
        pub max: u64,
        pub std_dev: u64,
        pub samples: usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_uart_write_and_verify() {
        let mut uart = MockUart::new();
        uart.expect_write("Hello");
        uart.write(b"Hello");
        assert!(uart.verify());
    }

    #[test]
    fn test_mock_uart_multiple_writes() {
        let mut uart = MockUart::new();
        uart.write(b"Line 1");
        uart.write(b"Line 2");
        assert_eq!(uart.writes.len(), 2);
        assert_eq!(uart.get_writes()[0], "Line 1");
        assert_eq!(uart.get_writes()[1], "Line 2");
    }

    #[test]
    fn test_mock_timer_advance() {
        let timer = MockTimer::default();
        assert_eq!(timer.get_timestamp_us(), 0);
        timer.advance_us(1000);
        assert_eq!(timer.get_timestamp_us(), 1000);
        timer.advance_ms(5);
        assert_eq!(timer.get_timestamp_us(), 6000);
    }

    #[test]
    fn test_mock_timer_reset() {
        let timer = MockTimer::default();
        timer.advance_us(5000);
        assert_eq!(timer.get_timestamp_us(), 5000);
        timer.reset();
        assert_eq!(timer.get_timestamp_us(), 0);
    }

    #[test]
    fn test_assert_approx_eq() {
        assert_helpers::assert_approx_eq(1.0, 1.0001, 0.001);
        assert_helpers::assert_approx_eq(0.5, 0.5, 0.0001);
    }

    #[test]
    fn test_assert_in_range() {
        assert_helpers::assert_in_range(0.5, 0.0, 1.0);
        assert_helpers::assert_in_range(0.0, 0.0, 1.0);
        assert_helpers::assert_in_range(1.0, 0.0, 1.0);
    }

    #[test]
    fn test_assert_normalized() {
        let arr = [0.0, 0.5, 1.0, 0.3, 0.8];
        assert_helpers::assert_normalized(&arr);
    }

    #[test]
    fn test_stats_mean() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_helpers::assert_approx_eq(stats::mean(&values), 3.0, 0.001);
    }

    #[test]
    fn test_stats_min_max() {
        let values = [1.0, 5.0, 3.0, 2.0, 4.0];
        assert_helpers::assert_approx_eq(stats::min(&values), 1.0, 0.001);
        assert_helpers::assert_approx_eq(stats::max(&values), 5.0, 0.001);
    }

    #[test]
    fn test_stats_percentile() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_helpers::assert_approx_eq(stats::percentile(&values, 0.5), 3.0, 0.5);
    }

    #[test]
    fn test_property_rand_f32_in_range() {
        let mut seed = 42u64;
        for _ in 0..100 {
            let val = property::rand_f32(&mut seed);
            assert_helpers::assert_in_range(val, 0.0, 1.0);
        }
    }

    #[test]
    fn test_property_check() {
        // Property: all random values in [0, 1] are >= 0
        assert!(property::check_property(100, |x| x >= 0.0));
    }

    #[test]
    fn test_benchmark_measure() {
        let timer = MockTimer::default();
        let (result, time) = benchmark::measure(&timer, || {
            timer.advance_us(100);
            42
        });
        assert_eq!(result, 42);
        assert_eq!(time, 100);
    }
}
