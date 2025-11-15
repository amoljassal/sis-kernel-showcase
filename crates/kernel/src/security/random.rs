/// Entropy source and PRNG for /dev/urandom (Phase D)
///
/// Provides a simple PRNG seeded from timer jitter and system counters.
/// For MVP, uses a basic LCG (Linear Congruential Generator) with periodic reseeding.

use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// Simple PRNG state using LCG algorithm
struct PrngState {
    state: u64,
    counter: u64,
}

impl PrngState {
    const fn new() -> Self {
        Self {
            state: 0x123456789abcdef0, // Initial seed
            counter: 0,
        }
    }

    /// LCG parameters from Numerical Recipes
    const MULTIPLIER: u64 = 6364136223846793005;
    const INCREMENT: u64 = 1442695040888963407;

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::MULTIPLIER).wrapping_add(Self::INCREMENT);
        self.counter += 1;

        // Reseed periodically from jitter
        if self.counter % 1024 == 0 {
            self.reseed();
        }

        self.state
    }

    fn reseed(&mut self) {
        // Mix in timer/counter jitter for entropy
        let jitter = get_jitter();
        self.state ^= jitter;
        self.state = self.state.wrapping_mul(Self::MULTIPLIER);
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        let mut remaining = buf;

        while !remaining.is_empty() {
            let rand = self.next();
            let bytes = rand.to_le_bytes();

            let to_copy = core::cmp::min(remaining.len(), 8);
            remaining[..to_copy].copy_from_slice(&bytes[..to_copy]);
            remaining = &mut remaining[to_copy..];
        }
    }
}

static PRNG: Mutex<PrngState> = Mutex::new(PrngState::new());

/// Get entropy from timer jitter and system counters
fn get_jitter() -> u64 {
    static JITTER_COUNTER: AtomicU64 = AtomicU64::new(0);

    // Read ARM64 system counter
    #[cfg(target_arch = "aarch64")]
    let timer: u64 = unsafe {
        let value;
        core::arch::asm!(
            "mrs {0}, cntvct_el0",
            out(reg) value,
            options(nomem, nostack)
        );
        value
    };
    #[cfg(target_arch = "x86_64")]
    let timer: u64 = crate::arch::x86_64::tsc::read_tsc();

    // Mix in monotonic counter
    let counter = JITTER_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Simple hash mixing
    let mut hash = timer;
    hash ^= counter;
    hash = hash.wrapping_mul(0x517cc1b727220a95);
    hash ^= hash >> 32;
    hash = hash.wrapping_mul(0x517cc1b727220a95);
    hash ^= hash >> 32;

    hash
}

/// Initialize the PRNG with entropy from jitter
pub fn init() {
    let mut prng = PRNG.lock();

    // Collect initial entropy from multiple jitter samples
    let mut seed = 0u64;
    for _ in 0..16 {
        seed ^= get_jitter();
        // Small delay to increase jitter variation
        for _ in 0..100 {
            core::hint::spin_loop();
        }
    }

    prng.state ^= seed;
    crate::info!("Random: Initialized PRNG with jitter entropy");
}

/// Fill buffer with random bytes (used by /dev/urandom)
pub fn fill_random_bytes(buf: &mut [u8]) {
    let mut prng = PRNG.lock();
    prng.fill_bytes(buf);
}

/// Get a random u64 value
pub fn random_u64() -> u64 {
    let mut prng = PRNG.lock();
    prng.next()
}

/// Get a random u32 value
pub fn random_u32() -> u32 {
    (random_u64() >> 32) as u32
}

/// Get a random value in range [0, max)
pub fn random_range(max: u64) -> u64 {
    if max == 0 {
        return 0;
    }

    // Use rejection sampling to avoid modulo bias
    let threshold = u64::MAX - (u64::MAX % max);
    loop {
        let val = random_u64();
        if val < threshold {
            return val % max;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prng_deterministic() {
        let mut prng = PrngState::new();
        let first = prng.next();
        let second = prng.next();

        assert_ne!(first, second, "PRNG should produce different values");
    }

    #[test]
    fn test_fill_bytes() {
        let mut buf = [0u8; 32];
        fill_random_bytes(&mut buf);

        // Check that not all bytes are zero
        assert!(buf.iter().any(|&b| b != 0), "Random bytes should not all be zero");
    }

    #[test]
    fn test_random_range() {
        for _ in 0..100 {
            let val = random_range(10);
            assert!(val < 10, "Random range value should be less than max");
        }
    }
}
