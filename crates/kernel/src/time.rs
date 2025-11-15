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

// ========== Hardware Timer Base ==========
// Additional timer functions for scheduler and syscalls

/// Cache for timer frequency (initialized at boot)
static mut TIMER_FREQUENCY: u64 = 0;

/// Read hardware cycle counter
#[cfg(target_arch = "aarch64")]
pub fn read_cycle_counter() -> u64 {
    let counter: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) counter);
    }
    counter
}

#[cfg(not(target_arch = "aarch64"))]
pub fn read_cycle_counter() -> u64 {
    0
}

/// Initialize timer and cache frequency
pub fn init_timer() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        TIMER_FREQUENCY = freq;
    }
}

/// Convert cycles to nanoseconds
pub fn cycles_to_ns(cycles: u64) -> u64 {
    let freq = unsafe { TIMER_FREQUENCY };
    if freq == 0 {
        // Fallback: assume 62.5 MHz (common in QEMU)
        (cycles * 1_000_000_000) / 62_500_000
    } else {
        // Avoid overflow: (cycles * 1_000_000_000) / freq
        let seconds = cycles / freq;
        let remainder = cycles % freq;
        seconds * 1_000_000_000 + (remainder * 1_000_000_000) / freq
    }
}

/// Get current time in nanoseconds
pub fn current_time_ns() -> u64 {
    cycles_to_ns(read_cycle_counter())
}

/// Get current time in microseconds
pub fn current_time_us() -> u64 {
    current_time_ns() / 1000
}

/// Sleep for the specified number of milliseconds
///
/// This is a busy-wait implementation and will block the current CPU.
/// Use sparingly and prefer proper async/await or timer-based solutions
/// for production code.
pub fn sleep_ms(ms: u64) {
    let start = get_timestamp_us();
    let target = start + (ms * 1000);
    while get_timestamp_us() < target {
        core::hint::spin_loop();
    }
}

/// Sleep for the specified number of microseconds
pub fn sleep_us(us: u64) {
    let start = get_timestamp_us();
    let target = start + us;
    while get_timestamp_us() < target {
        core::hint::spin_loop();
    }
}
