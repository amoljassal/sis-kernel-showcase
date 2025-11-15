// Mock Block Device
// Phase 6 - Production Readiness Plan

use crate::drivers::traits::BlockDevice;
use crate::lib::error::{Errno, Result};
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// Mock block device for testing
pub struct MockBlockDevice {
    name: String,
    data: Vec<u8>,
    block_size: usize,
    block_count: u64,
    readonly: bool,

    // Chaos/failure injection
    fail_rate: AtomicU32,      // Failure rate 0-100%
    delay_micros: AtomicU32,   // Artificial delay in microseconds

    // Statistics
    read_count: AtomicU64,
    write_count: AtomicU64,
    flush_count: AtomicU64,
    error_count: AtomicU64,
}

impl MockBlockDevice {
    /// Create a new mock block device
    pub fn new(name: &str, capacity_bytes: usize, block_size: usize) -> Self {
        let block_count = (capacity_bytes / block_size) as u64;
        let data = vec![0u8; capacity_bytes];

        Self {
            name: String::from(name),
            data,
            block_size,
            block_count,
            readonly: false,
            fail_rate: AtomicU32::new(0),
            delay_micros: AtomicU32::new(0),
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            flush_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    /// Create a read-only mock device
    pub fn new_readonly(name: &str, data: Vec<u8>, block_size: usize) -> Self {
        let capacity = data.len();
        let block_count = (capacity / block_size) as u64;

        Self {
            name: String::from(name),
            data,
            block_size,
            block_count,
            readonly: true,
            fail_rate: AtomicU32::new(0),
            delay_micros: AtomicU32::new(0),
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            flush_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    /// Set failure rate (0-100%)
    pub fn set_fail_rate(&self, rate: u32) {
        self.fail_rate.store(rate.min(100), Ordering::Relaxed);
    }

    /// Set artificial I/O delay in microseconds
    pub fn set_delay(&self, micros: u32) {
        self.delay_micros.store(micros, Ordering::Relaxed);
    }

    /// Get read count
    pub fn read_count(&self) -> u64 {
        self.read_count.load(Ordering::Relaxed)
    }

    /// Get write count
    pub fn write_count(&self) -> u64 {
        self.write_count.load(Ordering::Relaxed)
    }

    /// Get flush count
    pub fn flush_count(&self) -> u64 {
        self.flush_count.load(Ordering::Relaxed)
    }

    /// Get error count
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.read_count.store(0, Ordering::Relaxed);
        self.write_count.store(0, Ordering::Relaxed);
        self.flush_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
    }

    /// Check if operation should fail (based on fail_rate)
    fn should_fail(&self) -> bool {
        let rate = self.fail_rate.load(Ordering::Relaxed);
        if rate == 0 {
            return false;
        }

        // Simple PRNG for failure injection
        static SEED: AtomicU64 = AtomicU64::new(0x123456789abcdef0);
        let mut seed = SEED.load(Ordering::Relaxed);
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        SEED.store(seed, Ordering::Relaxed);

        (seed % 100) < rate as u64
    }

    /// Simulate I/O delay
    fn simulate_delay(&self) {
        let delay = self.delay_micros.load(Ordering::Relaxed);
        if delay > 0 {
            // In real implementation, this would call a sleep function
            // For now, just busy-wait (simplified for mock)
            for _ in 0..delay {
                core::hint::spin_loop();
            }
        }
    }

    /// Fill device with pattern
    pub fn fill_pattern(&mut self, pattern: u8) {
        for byte in self.data.iter_mut() {
            *byte = pattern;
        }
    }

    /// Get underlying data (for testing)
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl BlockDevice for MockBlockDevice {
    fn read(&self, block: u64, buf: &mut [u8]) -> Result<()> {
        // Simulate delay
        self.simulate_delay();

        // Check for failure injection
        if self.should_fail() {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(Errno::EIO);
        }

        // Validate block number
        if block >= self.block_count {
            return Err(Errno::EINVAL);
        }

        // Validate buffer size
        if buf.len() < self.block_size {
            return Err(Errno::EINVAL);
        }

        // Read block
        let offset = (block as usize) * self.block_size;
        buf[..self.block_size].copy_from_slice(&self.data[offset..offset + self.block_size]);

        self.read_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn write(&self, block: u64, buf: &[u8]) -> Result<()> {
        // Check read-only
        if self.readonly {
            return Err(Errno::EROFS);
        }

        // Simulate delay
        self.simulate_delay();

        // Check for failure injection
        if self.should_fail() {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(Errno::EIO);
        }

        // Validate block number
        if block >= self.block_count {
            return Err(Errno::EINVAL);
        }

        // Validate buffer size
        if buf.len() < self.block_size {
            return Err(Errno::EINVAL);
        }

        // Write block (need unsafe for interior mutability)
        let offset = (block as usize) * self.block_size;
        let data_ptr = self.data.as_ptr() as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                data_ptr.add(offset),
                self.block_size
            );
        }

        self.write_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        // Check read-only
        if self.readonly {
            return Ok(()); // Flush is no-op for readonly
        }

        // Simulate delay
        self.simulate_delay();

        // Check for failure injection
        if self.should_fail() {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(Errno::EIO);
        }

        self.flush_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn block_size(&self) -> usize {
        self.block_size
    }

    fn block_count(&self) -> u64 {
        self.block_count
    }

    fn is_readonly(&self) -> bool {
        self.readonly
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_block_device_creation() {
        let dev = MockBlockDevice::new("test", 4096, 512);
        assert_eq!(dev.block_size(), 512);
        assert_eq!(dev.block_count(), 8);
        assert_eq!(dev.capacity(), 4096);
        assert!(!dev.is_readonly());
    }

    #[test]
    fn test_mock_block_device_read_write() {
        let dev = MockBlockDevice::new("test", 4096, 512);

        // Write data
        let write_buf = vec![0x42u8; 512];
        dev.write(0, &write_buf).unwrap();

        // Read it back
        let mut read_buf = vec![0u8; 512];
        dev.read(0, &mut read_buf).unwrap();

        assert_eq!(read_buf, write_buf);
    }

    #[test]
    fn test_mock_block_device_readonly() {
        let data = vec![0x42u8; 4096];
        let dev = MockBlockDevice::new_readonly("test", data, 512);

        assert!(dev.is_readonly());

        let write_buf = vec![0xAAu8; 512];
        let result = dev.write(0, &write_buf);
        assert_eq!(result, Err(Errno::EROFS));
    }

    #[test]
    fn test_mock_block_device_failure_injection() {
        let dev = MockBlockDevice::new("test", 4096, 512);
        dev.set_fail_rate(100); // 100% failure rate

        let mut buf = vec![0u8; 512];
        let result = dev.read(0, &mut buf);

        // Should fail due to injection
        assert!(result.is_err());
        assert_eq!(dev.error_count(), 1);
    }
}
