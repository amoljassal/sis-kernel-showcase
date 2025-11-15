// Chaos Engineering Framework
// Phase 3.1 - Production Readiness Plan
//
// Provides controlled failure injection for testing system resilience.
// Disabled by default, enabled via "chaos" feature flag.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Chaos injection modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ChaosMode {
    None = 0,
    DiskFull = 1,
    DiskFail = 2,
    NetworkFail = 3,
    MemoryPressure = 4,
    RandomPanic = 5,
    SlowIo = 6,
}

impl ChaosMode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => ChaosMode::DiskFull,
            2 => ChaosMode::DiskFail,
            3 => ChaosMode::NetworkFail,
            4 => ChaosMode::MemoryPressure,
            5 => ChaosMode::RandomPanic,
            6 => ChaosMode::SlowIo,
            _ => ChaosMode::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ChaosMode::None => "none",
            ChaosMode::DiskFull => "disk_full",
            ChaosMode::DiskFail => "disk_fail",
            ChaosMode::NetworkFail => "network_fail",
            ChaosMode::MemoryPressure => "memory_pressure",
            ChaosMode::RandomPanic => "random_panic",
            ChaosMode::SlowIo => "slow_io",
        }
    }
}

/// Global chaos mode (atomic for lock-free access)
static CHAOS_MODE: AtomicU32 = AtomicU32::new(ChaosMode::None as u32);

/// Failure rate (percentage, 0-100)
static FAILURE_RATE: AtomicU32 = AtomicU32::new(10);

/// Chaos invocation counter (for deterministic failures)
static CHAOS_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Set chaos mode
pub fn set_mode(mode: ChaosMode) {
    CHAOS_MODE.store(mode as u32, Ordering::Relaxed);
    #[cfg(feature = "chaos")]
    unsafe {
        crate::uart_print(b"[CHAOS] Mode set to: ");
        crate::uart_print(mode.as_str().as_bytes());
        crate::uart_print(b"\n");
    }
}

/// Get current chaos mode
pub fn get_mode() -> ChaosMode {
    ChaosMode::from_u32(CHAOS_MODE.load(Ordering::Relaxed))
}

/// Set failure rate (percentage, 0-100)
pub fn set_failure_rate(rate: u32) {
    let clamped = rate.min(100);
    FAILURE_RATE.store(clamped, Ordering::Relaxed);
    #[cfg(feature = "chaos")]
    unsafe {
        crate::uart_print(b"[CHAOS] Failure rate set to: ");
        crate::trace::print_usize(clamped as usize);
        crate::uart_print(b"%\n");
    }
}

/// Check if chaos is enabled (compile-time check + runtime mode)
#[inline(always)]
pub fn is_enabled() -> bool {
    cfg!(feature = "chaos") && get_mode() != ChaosMode::None
}

/// Simple PRNG for chaos decisions (using counter as seed)
#[inline(always)]
fn pseudo_random() -> u32 {
    let counter = CHAOS_COUNTER.fetch_add(1, Ordering::Relaxed);
    // Simple xorshift-like PRNG
    let mut x = (counter as u32).wrapping_add(0xdeadbeef);
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

/// Check if we should inject failure based on rate
#[inline(always)]
fn should_fail_random() -> bool {
    let rate = FAILURE_RATE.load(Ordering::Relaxed);
    if rate == 0 {
        return false;
    }
    (pseudo_random() % 100) < rate
}

// ============================================================================
// Chaos Injection Points
// ============================================================================

/// Inject disk full error (ENOSPC)
#[inline(always)]
pub fn should_fail_disk_full() -> bool {
    #[cfg(feature = "chaos")]
    {
        if get_mode() == ChaosMode::DiskFull && should_fail_random() {
            unsafe { crate::uart_print(b"[CHAOS] Injecting ENOSPC (disk full)\n"); }
            return true;
        }
    }
    false
}

/// Inject disk I/O error (EIO)
#[inline(always)]
pub fn should_fail_disk_io() -> bool {
    #[cfg(feature = "chaos")]
    {
        if get_mode() == ChaosMode::DiskFail && should_fail_random() {
            unsafe { crate::uart_print(b"[CHAOS] Injecting EIO (disk failure)\n"); }
            return true;
        }
    }
    false
}

/// Inject network failure (ENETDOWN)
#[inline(always)]
pub fn should_fail_network() -> bool {
    #[cfg(feature = "chaos")]
    {
        if get_mode() == ChaosMode::NetworkFail && should_fail_random() {
            unsafe { crate::uart_print(b"[CHAOS] Injecting ENETDOWN (network failure)\n"); }
            return true;
        }
    }
    false
}

/// Inject memory pressure (simulate OOM)
#[inline(always)]
pub fn should_fail_allocation() -> bool {
    #[cfg(feature = "chaos")]
    {
        if get_mode() == ChaosMode::MemoryPressure && should_fail_random() {
            unsafe { crate::uart_print(b"[CHAOS] Injecting ENOMEM (allocation failure)\n"); }
            return true;
        }
    }
    false
}

/// Inject random panic (for panic handler testing)
#[inline(always)]
pub fn maybe_panic() {
    #[cfg(feature = "chaos")]
    {
        if get_mode() == ChaosMode::RandomPanic && should_fail_random() {
            unsafe { crate::uart_print(b"[CHAOS] Injecting panic!\n"); }
            panic!("Chaos-injected panic for testing");
        }
    }
}

/// Inject I/O delay (simulate slow disk/network)
#[inline(always)]
pub fn maybe_delay_io() {
    #[cfg(feature = "chaos")]
    {
        if get_mode() == ChaosMode::SlowIo && should_fail_random() {
            unsafe { crate::uart_print(b"[CHAOS] Injecting I/O delay\n"); }
            // Busy-wait for a bit (not ideal, but simple)
            for _ in 0..10000 {
                core::hint::spin_loop();
            }
        }
    }
}

// ============================================================================
// Statistics
// ============================================================================

static DISK_FULL_COUNT: AtomicU64 = AtomicU64::new(0);
static DISK_FAIL_COUNT: AtomicU64 = AtomicU64::new(0);
static NETWORK_FAIL_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOC_FAIL_COUNT: AtomicU64 = AtomicU64::new(0);

/// Record chaos event statistics
pub fn record_disk_full() {
    DISK_FULL_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn record_disk_fail() {
    DISK_FAIL_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn record_network_fail() {
    NETWORK_FAIL_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn record_alloc_fail() {
    ALLOC_FAIL_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Get chaos statistics
pub fn get_stats() -> ChaosStats {
    ChaosStats {
        mode: get_mode(),
        failure_rate: FAILURE_RATE.load(Ordering::Relaxed),
        disk_full_count: DISK_FULL_COUNT.load(Ordering::Relaxed),
        disk_fail_count: DISK_FAIL_COUNT.load(Ordering::Relaxed),
        network_fail_count: NETWORK_FAIL_COUNT.load(Ordering::Relaxed),
        alloc_fail_count: ALLOC_FAIL_COUNT.load(Ordering::Relaxed),
    }
}

pub struct ChaosStats {
    pub mode: ChaosMode,
    pub failure_rate: u32,
    pub disk_full_count: u64,
    pub disk_fail_count: u64,
    pub network_fail_count: u64,
    pub alloc_fail_count: u64,
}

/// Reset chaos statistics
pub fn reset_stats() {
    DISK_FULL_COUNT.store(0, Ordering::Relaxed);
    DISK_FAIL_COUNT.store(0, Ordering::Relaxed);
    NETWORK_FAIL_COUNT.store(0, Ordering::Relaxed);
    ALLOC_FAIL_COUNT.store(0, Ordering::Relaxed);
}
