//! Pseudo-Random Number Generator for Kernel Use
//!
//! Provides deterministic but varied random number generation for stress tests
//! and chaos engineering. Uses a combination of Linear Congruential Generator (LCG)
//! and Xorshift for better randomness.

use core::sync::atomic::{AtomicU64, Ordering};

/// Global PRNG state (seeded from timestamp on first use)
static PRNG_STATE: AtomicU64 = AtomicU64::new(0);

/// Initialize the PRNG with a seed (typically timestamp)
pub fn init_prng(seed: u64) {
    PRNG_STATE.store(seed, Ordering::Relaxed);
}

/// Generate next random u64 using Xorshift64
fn next_u64() -> u64 {
    let mut x = PRNG_STATE.load(Ordering::Relaxed);

    // If uninitialized, seed with a basic value
    if x == 0 {
        x = 0x123456789ABCDEF0u64;
    }

    // Xorshift64 algorithm
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;

    PRNG_STATE.store(x, Ordering::Relaxed);
    x
}

/// Generate random u32
pub fn rand_u32() -> u32 {
    (next_u64() & 0xFFFFFFFF) as u32
}

/// Generate random u64
pub fn rand_u64() -> u64 {
    next_u64()
}

/// Generate random number in range [min, max)
pub fn rand_range(min: u32, max: u32) -> u32 {
    if min >= max {
        return min;
    }
    let range = max - min;
    min + (rand_u32() % range)
}

/// Generate random number in range [min, max) for u64
pub fn rand_range_u64(min: u64, max: u64) -> u64 {
    if min >= max {
        return min;
    }
    let range = max - min;
    min + (rand_u64() % range)
}

/// Generate random float in range [0.0, 1.0)
pub fn rand_float() -> f32 {
    (rand_u32() as f32) / (u32::MAX as f32)
}

/// Generate random float in range [min, max)
pub fn rand_float_range(min: f32, max: f32) -> f32 {
    min + rand_float() * (max - min)
}

/// Generate random boolean with given probability (0.0 to 1.0)
pub fn rand_bool(probability: f32) -> bool {
    rand_float() < probability
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_range() {
        init_prng(12345);

        for _ in 0..100 {
            let val = rand_range(10, 20);
            assert!(val >= 10 && val < 20);
        }
    }

    #[test]
    fn test_rand_float() {
        init_prng(67890);

        for _ in 0..100 {
            let val = rand_float();
            assert!(val >= 0.0 && val < 1.0);
        }
    }
}
