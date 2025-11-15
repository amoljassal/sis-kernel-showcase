//! Timeout utilities for driver operations
//!
//! Part of M8 Driver Hardening - provides timeout mechanisms for all
//! hardware wait operations to prevent infinite loops and hangs.

use crate::time;

/// Default timeout for hardware operations (1 second)
pub const DEFAULT_TIMEOUT_US: u64 = 1_000_000;

/// Short timeout for register reads (1ms)
pub const SHORT_TIMEOUT_US: u64 = 1_000;

/// Long timeout for slow operations like firmware calls (5 seconds)
pub const LONG_TIMEOUT_US: u64 = 5_000_000;

/// Timeout error
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimeoutError {
    pub elapsed_us: u64,
    pub timeout_us: u64,
}

impl TimeoutError {
    pub fn new(elapsed_us: u64, timeout_us: u64) -> Self {
        Self {
            elapsed_us,
            timeout_us,
        }
    }
}

/// Timeout context for tracking elapsed time
pub struct Timeout {
    start_us: u64,
    timeout_us: u64,
}

impl Timeout {
    /// Create a new timeout context
    ///
    /// # Arguments
    /// * `timeout_us` - Timeout duration in microseconds
    ///
    /// # Example
    /// ```
    /// let timeout = Timeout::new(1_000_000); // 1 second
    /// while !hardware_ready() {
    ///     if timeout.is_expired() {
    ///         return Err(TimeoutError::new(...));
    ///     }
    ///     core::hint::spin_loop();
    /// }
    /// ```
    pub fn new(timeout_us: u64) -> Self {
        Self {
            start_us: time::get_timestamp_us(),
            timeout_us,
        }
    }

    /// Create a timeout with default duration (1 second)
    pub fn default() -> Self {
        Self::new(DEFAULT_TIMEOUT_US)
    }

    /// Create a short timeout (1ms)
    pub fn short() -> Self {
        Self::new(SHORT_TIMEOUT_US)
    }

    /// Create a long timeout (5 seconds)
    pub fn long() -> Self {
        Self::new(LONG_TIMEOUT_US)
    }

    /// Check if the timeout has expired
    pub fn is_expired(&self) -> bool {
        self.elapsed_us() >= self.timeout_us
    }

    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> u64 {
        time::get_timestamp_us().saturating_sub(self.start_us)
    }

    /// Get remaining time in microseconds
    pub fn remaining_us(&self) -> u64 {
        self.timeout_us.saturating_sub(self.elapsed_us())
    }

    /// Wait until condition is true or timeout expires
    ///
    /// # Arguments
    /// * `condition` - Closure that returns true when condition is met
    ///
    /// # Returns
    /// `Ok(())` if condition met, `Err(TimeoutError)` if timeout
    ///
    /// # Example
    /// ```
    /// Timeout::default().wait(|| hardware_ready())?;
    /// ```
    pub fn wait<F>(&self, mut condition: F) -> Result<(), TimeoutError>
    where
        F: FnMut() -> bool,
    {
        while !condition() {
            if self.is_expired() {
                return Err(TimeoutError::new(self.elapsed_us(), self.timeout_us));
            }
            core::hint::spin_loop();
        }
        Ok(())
    }

    /// Wait with custom delay between checks
    ///
    /// # Arguments
    /// * `condition` - Closure that returns true when condition is met
    /// * `delay_us` - Delay between condition checks in microseconds
    pub fn wait_with_delay<F>(&self, mut condition: F, delay_us: u64) -> Result<(), TimeoutError>
    where
        F: FnMut() -> bool,
    {
        while !condition() {
            if self.is_expired() {
                return Err(TimeoutError::new(self.elapsed_us(), self.timeout_us));
            }
            time::sleep_us(delay_us);
        }
        Ok(())
    }

    /// Reset the timeout to start from now
    pub fn reset(&mut self) {
        self.start_us = time::get_timestamp_us();
    }
}

/// Wait for a condition with timeout
///
/// # Arguments
/// * `timeout_us` - Timeout in microseconds
/// * `condition` - Closure that returns true when condition is met
///
/// # Returns
/// `Ok(())` if condition met, `Err(TimeoutError)` if timeout
///
/// # Example
/// ```
/// wait_timeout(1_000_000, || register.is_ready())?;
/// ```
pub fn wait_timeout<F>(timeout_us: u64, condition: F) -> Result<(), TimeoutError>
where
    F: FnMut() -> bool,
{
    Timeout::new(timeout_us).wait(condition)
}

/// Wait for a condition with default timeout (1 second)
pub fn wait_timeout_default<F>(condition: F) -> Result<(), TimeoutError>
where
    F: FnMut() -> bool,
{
    Timeout::default().wait(condition)
}

/// Retry an operation with timeout
///
/// # Arguments
/// * `timeout_us` - Total timeout in microseconds
/// * `retry_delay_us` - Delay between retries in microseconds
/// * `operation` - Closure that returns Ok(T) on success or Err(E) on failure
///
/// # Returns
/// `Ok(T)` if operation succeeds, `Err(TimeoutError)` if all retries exhausted
pub fn retry_with_timeout<T, E, F>(
    timeout_us: u64,
    retry_delay_us: u64,
    mut operation: F,
) -> Result<T, TimeoutError>
where
    F: FnMut() -> Result<T, E>,
{
    let timeout = Timeout::new(timeout_us);

    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(_) => {
                if timeout.is_expired() {
                    return Err(TimeoutError::new(timeout.elapsed_us(), timeout_us));
                }
                time::sleep_us(retry_delay_us);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_expiry() {
        let timeout = Timeout::new(100); // 100us
        time::sleep_us(150);
        assert!(timeout.is_expired());
    }

    #[test]
    fn test_timeout_wait_success() {
        let mut ready = false;
        let result = Timeout::new(1_000_000).wait(|| {
            ready = true;
            ready
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_timeout_wait_failure() {
        let result = Timeout::new(100).wait(|| false);
        assert!(result.is_err());
    }
}
