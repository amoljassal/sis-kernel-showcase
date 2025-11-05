// AArch64 Generic Timer support
// Phase A0 - Basic timer stub, full implementation in Phase A1/E

use core::arch::asm;

/// Read the system counter (CNTPCT_EL0)
#[inline(always)]
pub fn read_cntpct() -> u64 {
    let count: u64;
    unsafe {
        asm!("mrs {}, CNTPCT_EL0", out(reg) count);
    }
    count
}

/// Read the counter frequency (CNTFRQ_EL0)
#[inline(always)]
pub fn read_cntfrq() -> u64 {
    let freq: u64;
    unsafe {
        asm!("mrs {}, CNTFRQ_EL0", out(reg) freq);
    }
    freq
}

/// Get current time in microseconds
pub fn get_time_us() -> u64 {
    let count = read_cntpct();
    let freq = read_cntfrq();

    // Convert to microseconds
    (count * 1_000_000) / freq
}

/// Get current time in milliseconds
pub fn get_time_ms() -> u64 {
    get_time_us() / 1000
}

/// Initialize timer (stub for Phase A0)
pub fn init_timer() {
    let freq = read_cntfrq();
    crate::info!("Timer frequency: {} Hz ({} MHz)", freq, freq / 1_000_000);

    // Phase A0: No timer interrupts yet
    // Full implementation in Phase A1 for preemptive scheduling
}

/// Set timer interrupt (stub for Phase A0)
pub fn set_timer_interrupt(_us: u64) {
    // Phase A0: Timer interrupts not implemented yet
}
