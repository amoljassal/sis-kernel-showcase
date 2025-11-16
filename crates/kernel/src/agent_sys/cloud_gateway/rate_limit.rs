//! Rate limiting for Cloud Gateway
//!
//! Implements token bucket rate limiting per agent to prevent abuse
//! and ensure fair resource allocation.

use crate::time::get_timestamp_us;

/// Token bucket rate limiter
///
/// # Algorithm
///
/// The token bucket algorithm allows bursty traffic while enforcing
/// long-term rate limits:
///
/// 1. Bucket starts with `capacity` tokens
/// 2. Each request consumes 1 token
/// 3. Tokens refill at `refill_rate` tokens/second
/// 4. Bucket cannot exceed `capacity`
///
/// # Example
///
/// ```
/// let mut limiter = RateLimiter::new(10, 5); // 10 capacity, 5/sec refill
///
/// // First 10 requests succeed (burst)
/// for _ in 0..10 {
///     assert!(limiter.check_and_consume());
/// }
///
/// // 11th request fails (bucket empty)
/// assert!(!limiter.check_and_consume());
///
/// // Wait 1 second...
/// // 5 more requests succeed (refilled)
/// ```
#[derive(Debug)]
pub struct RateLimiter {
    /// Current tokens available
    tokens: u32,

    /// Maximum tokens (bucket capacity)
    capacity: u32,

    /// Tokens added per second
    refill_rate: u32,

    /// Last refill timestamp (microseconds)
    last_refill_us: u64,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum burst size (tokens)
    /// * `refill_rate` - Tokens per second
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill_us: get_timestamp_us(),
        }
    }

    /// Create a permissive rate limiter (high limits)
    pub fn permissive() -> Self {
        Self::new(100, 50) // 100 burst, 50/sec sustained
    }

    /// Create a restrictive rate limiter (low limits)
    pub fn restrictive() -> Self {
        Self::new(10, 2) // 10 burst, 2/sec sustained
    }

    /// Create a standard rate limiter (balanced)
    pub fn standard() -> Self {
        Self::new(30, 10) // 30 burst, 10/sec sustained
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now_us = get_timestamp_us();
        let elapsed_us = now_us.saturating_sub(self.last_refill_us);

        if elapsed_us == 0 {
            return;
        }

        // Calculate tokens to add
        let elapsed_seconds = elapsed_us as f64 / 1_000_000.0;
        let tokens_to_add = (elapsed_seconds * self.refill_rate as f64) as u32;

        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            self.last_refill_us = now_us;
        }
    }

    /// Check if request can proceed and consume token if available
    ///
    /// Returns `true` if token was consumed (request allowed),
    /// `false` if rate limit exceeded.
    pub fn check_and_consume(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Get current token count (after refill)
    pub fn available_tokens(&mut self) -> u32 {
        self.refill();
        self.tokens
    }

    /// Get capacity
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Get refill rate
    pub fn refill_rate(&self) -> u32 {
        self.refill_rate
    }

    /// Reset to full capacity
    pub fn reset(&mut self) {
        self.tokens = self.capacity;
        self.last_refill_us = get_timestamp_us();
    }

    /// Check if any tokens available (without consuming)
    pub fn has_tokens(&mut self) -> bool {
        self.refill();
        self.tokens > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_burst() {
        let mut limiter = RateLimiter::new(10, 5);

        // Should allow 10 requests (full capacity)
        for i in 0..10 {
            assert!(limiter.check_and_consume(), "Request {} should succeed", i);
        }

        // 11th should fail
        assert!(!limiter.check_and_consume(), "Request 11 should fail");
    }

    #[test]
    fn test_rate_limiter_available_tokens() {
        let mut limiter = RateLimiter::new(10, 5);

        assert_eq!(limiter.available_tokens(), 10);

        limiter.check_and_consume();
        assert_eq!(limiter.available_tokens(), 9);

        limiter.check_and_consume();
        limiter.check_and_consume();
        assert_eq!(limiter.available_tokens(), 7);
    }

    #[test]
    fn test_rate_limiter_reset() {
        let mut limiter = RateLimiter::new(10, 5);

        // Consume all tokens
        for _ in 0..10 {
            limiter.check_and_consume();
        }

        assert_eq!(limiter.available_tokens(), 0);

        // Reset
        limiter.reset();

        assert_eq!(limiter.available_tokens(), 10);
    }

    #[test]
    fn test_rate_limiter_has_tokens() {
        let mut limiter = RateLimiter::new(1, 1);

        assert!(limiter.has_tokens());

        limiter.check_and_consume();

        assert!(!limiter.has_tokens());
    }

    #[test]
    fn test_permissive_limiter() {
        let limiter = RateLimiter::permissive();
        assert_eq!(limiter.capacity(), 100);
        assert_eq!(limiter.refill_rate(), 50);
    }

    #[test]
    fn test_restrictive_limiter() {
        let limiter = RateLimiter::restrictive();
        assert_eq!(limiter.capacity(), 10);
        assert_eq!(limiter.refill_rate(), 2);
    }

    #[test]
    fn test_standard_limiter() {
        let limiter = RateLimiter::standard();
        assert_eq!(limiter.capacity(), 30);
        assert_eq!(limiter.refill_rate(), 10);
    }
}
