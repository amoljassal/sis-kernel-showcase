// Mock Timer Device
// Phase 6 - Production Readiness Plan

use crate::drivers::traits::TimerDevice;
use crate::lib::error::{Errno, Result};
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// Mock timer device for testing
pub struct MockTimerDevice {
    name: &'static str,
    frequency: u64,
    current_ticks: AtomicU64,
    timeout_ticks: AtomicU64,
    jitter_micros: AtomicU32,  // Simulated jitter
}

impl MockTimerDevice {
    /// Create a new mock timer device
    pub fn new(name: &'static str, frequency: u64) -> Self {
        Self {
            name,
            frequency,
            current_ticks: AtomicU64::new(0),
            timeout_ticks: AtomicU64::new(0),
            jitter_micros: AtomicU32::new(0),
        }
    }

    /// Advance timer by given ticks (for testing)
    pub fn advance(&self, ticks: u64) {
        self.current_ticks.fetch_add(ticks, Ordering::Relaxed);
    }

    /// Advance timer by given milliseconds (for testing)
    pub fn advance_millis(&self, millis: u64) {
        let ticks = (millis * self.frequency) / 1000;
        self.advance(ticks);
    }

    /// Set current time (for testing)
    pub fn set_ticks(&self, ticks: u64) {
        self.current_ticks.store(ticks, Ordering::Relaxed);
    }

    /// Set jitter in microseconds
    pub fn set_jitter(&self, micros: u32) {
        self.jitter_micros.store(micros, Ordering::Relaxed);
    }

    /// Get timeout value (for testing)
    pub fn timeout_ticks(&self) -> u64 {
        self.timeout_ticks.load(Ordering::Relaxed)
    }

    /// Check if timeout has occurred
    pub fn timeout_occurred(&self) -> bool {
        let timeout = self.timeout_ticks.load(Ordering::Relaxed);
        if timeout == 0 {
            return false;
        }
        let current = self.current_ticks.load(Ordering::Relaxed);
        current >= timeout
    }

    /// Simulate jitter
    fn apply_jitter(&self, ticks: u64) -> u64 {
        let jitter = self.jitter_micros.load(Ordering::Relaxed);
        if jitter == 0 {
            return ticks;
        }

        // Simple PRNG for jitter
        static SEED: AtomicU64 = AtomicU64::new(0x123456789abcdef0);
        let mut seed = SEED.load(Ordering::Relaxed);
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        SEED.store(seed, Ordering::Relaxed);

        // Apply jitter (±jitter_micros)
        let jitter_ticks = ((jitter as u64 * self.frequency) / 1_000_000) as i64;
        let jitter_offset = ((seed as i64) % jitter_ticks) - (jitter_ticks / 2);
        ticks.saturating_add_signed(jitter_offset)
    }
}

impl TimerDevice for MockTimerDevice {
    fn read(&self) -> u64 {
        let ticks = self.current_ticks.load(Ordering::Relaxed);
        self.apply_jitter(ticks)
    }

    fn frequency(&self) -> u64 {
        self.frequency
    }

    fn set_timeout(&self, ticks: u64) -> Result<()> {
        let current = self.current_ticks.load(Ordering::Relaxed);
        let timeout = current + ticks;
        self.timeout_ticks.store(timeout, Ordering::Relaxed);
        Ok(())
    }

    fn cancel_timeout(&self) -> Result<()> {
        self.timeout_ticks.store(0, Ordering::Relaxed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_timer_creation() {
        let timer = MockTimerDevice::new("test", 1_000_000); // 1MHz
        assert_eq!(timer.frequency(), 1_000_000);
        assert_eq!(timer.read(), 0);
    }

    #[test]
    fn test_mock_timer_advance() {
        let timer = MockTimerDevice::new("test", 1_000_000);

        timer.advance(1000);
        assert_eq!(timer.read(), 1000);

        timer.advance(500);
        assert_eq!(timer.read(), 1500);
    }

    #[test]
    fn test_mock_timer_millis() {
        let timer = MockTimerDevice::new("test", 1_000_000); // 1MHz

        timer.advance_millis(1); // 1ms = 1000 ticks at 1MHz
        assert_eq!(timer.read(), 1000);

        assert_eq!(timer.millis(), 1);
    }

    #[test]
    fn test_mock_timer_timeout() {
        let timer = MockTimerDevice::new("test", 1_000_000);

        // Set timeout for 1000 ticks
        timer.set_timeout(1000).unwrap();
        assert!(!timer.timeout_occurred());

        // Advance to timeout
        timer.advance(1000);
        assert!(timer.timeout_occurred());

        // Cancel timeout
        timer.cancel_timeout().unwrap();
        assert!(!timer.timeout_occurred());
    }
}
