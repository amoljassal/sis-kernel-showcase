//! Time and Timestamp Utilities

use core::sync::atomic::{AtomicU64, Ordering};

static BOOT_TIMESTAMP_US: AtomicU64 = AtomicU64::new(0);

/// Get current timestamp in microseconds
/// Uses ARM generic timer on AArch64
pub fn get_timestamp_us() -> u64 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut cntpct: u64;
        core::arch::asm!("mrs {0}, cntpct_el0", out(reg) cntpct);

        // Read counter frequency
        let mut cntfrq: u64;
        core::arch::asm!("mrs {0}, cntfrq_el0", out(reg) cntfrq);

        // Convert to microseconds
        if cntfrq > 0 {
            (cntpct * 1_000_000) / cntfrq
        } else {
            // Fallback if frequency not set (QEMU sometimes does this)
            cntpct / 62  // Assume 62.5 MHz timer (common in QEMU)
        }
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        // Fallback for other architectures
        0
    }
}

/// Initialize boot timestamp
pub fn init_boot_timestamp() {
    BOOT_TIMESTAMP_US.store(get_timestamp_us(), Ordering::Relaxed);
}

/// Get time since boot in microseconds
pub fn get_time_since_boot_us() -> u64 {
    let current = get_timestamp_us();
    let boot = BOOT_TIMESTAMP_US.load(Ordering::Relaxed);
    current.saturating_sub(boot)
}

/// Get time since boot in milliseconds
pub fn get_time_since_boot_ms() -> u64 {
    get_time_since_boot_us() / 1000
}

/// Get uptime in milliseconds (alias for get_time_since_boot_ms)
pub fn get_uptime_ms() -> u64 {
    get_time_since_boot_ms()
}
