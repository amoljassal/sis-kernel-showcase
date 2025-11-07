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

/// Initialize timer with specified interval
///
/// Sets up the EL1 physical timer to generate interrupts at the specified interval.
/// For Phase A0, we configure the timer but IRQ handling is minimal.
pub fn init_timer(interval_ms: u64) {
    unsafe {
        let freq = read_cntfrq();
        crate::info!("Timer frequency: {} Hz ({} MHz)", freq, freq / 1_000_000);

        // Calculate cycles for the interval
        let cycles = (freq * interval_ms) / 1000;
        crate::info!("Timer interval: {} ms ({} cycles)", interval_ms, cycles);

        // Read current counter value
        let now = read_cntpct();

        // Set comparator value (CNTP_CVAL_EL0) = now + cycles
        let cval = now + cycles;
        asm!(
            "msr CNTP_CVAL_EL0, {}",
            in(reg) cval
        );

        // Enable timer (CNTP_CTL_EL0)
        // Bit 0: ENABLE
        // Bit 1: IMASK (0 = not masked)
        // Bit 2: ISTATUS (read-only)
        asm!(
            "msr CNTP_CTL_EL0, {}",
            in(reg) 1u64  // Enable bit set
        );

        // Instruction synchronization barrier
        asm!("isb");

        crate::info!("Timer enabled with {} ms interval", interval_ms);

        // NOTE: GIC setup for PPI 30 needs to be done separately
        // This will be in Phase E when we implement full interrupt handling
        // For Phase A0, timer is configured but interrupts may be masked
    }
}

/// Set timer interrupt for a specific delay
///
/// This will fire after the specified number of microseconds
pub fn set_timer_interrupt(us: u64) {
    unsafe {
        let freq = read_cntfrq();
        let cycles = (freq * us) / 1_000_000;

        let now = read_cntpct();
        let cval = now + cycles;

        asm!(
            "msr CNTP_CVAL_EL0, {}",
            in(reg) cval
        );

        asm!("isb");
    }
}

/// Disable timer interrupts
pub fn disable_timer() {
    unsafe {
        asm!(
            "msr CNTP_CTL_EL0, {}",
            in(reg) 0u64  // Disable
        );
        asm!("isb");
    }
}
