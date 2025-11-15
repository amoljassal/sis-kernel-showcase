// AArch64 Generic Timer support
// Enhanced for Raspberry Pi 5 and QEMU aarch64 virt
//
// The ARM Generic Timer provides system-wide timing and implements:
// - Physical timer (EL1 Physical Timer, PPI 30)
// - Virtual timer (EL1 Virtual Timer, PPI 27)
// - Hypervisor timer (EL2 Physical Timer, PPI 26)
//
// On RPi5: Timer frequency is typically ~54MHz
// On QEMU: Timer frequency is typically 62.5MHz
//
// Frequency is read from CNTFRQ_EL0 register which is set by firmware.

use core::arch::asm;

/// Timer interrupt numbers (PPIs)
pub const TIMER_IRQ_PHYS: u32 = 30;  // EL1 Physical Timer
pub const TIMER_IRQ_VIRT: u32 = 27;  // EL1 Virtual Timer

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

/// Initialize timer with GICv3 integration
///
/// This function:
/// 1. Reads the timer frequency from CNTFRQ_EL0
/// 2. Configures the physical timer for periodic interrupts
/// 3. Enables the timer interrupt in GICv3
///
/// # Arguments
/// * `interval_ms` - Timer interrupt interval in milliseconds
///
/// # Safety
/// Must be called after GICv3 initialization
pub unsafe fn init_with_gic(interval_ms: u64) {
    let freq = read_cntfrq();
    crate::info!("Timer: Frequency {} Hz ({} MHz)", freq, freq / 1_000_000);

    // Calculate ticks for the interval
    let ticks = (freq * interval_ms) / 1000;
    crate::info!("Timer: Interval {} ms ({} ticks)", interval_ms, ticks);

    // Configure timer for periodic interrupts
    init_timer(interval_ms);

    // Enable timer interrupt in GIC (PPI 30)
    // PPIs are per-CPU, so this enables it for the current CPU
    #[cfg(not(test))]
    {
        if let Some(_) = crate::arch::aarch64::gicv3::enable_irq_checked(TIMER_IRQ_PHYS) {
            crate::info!("Timer: Enabled IRQ {} in GIC", TIMER_IRQ_PHYS);
        } else {
            crate::warn!("Timer: Failed to enable IRQ in GIC (GIC may not be initialized)");
        }
    }

    crate::info!("Timer: Initialization complete");
}

/// Handle timer interrupt
///
/// This function should be called from the IRQ handler when a timer interrupt occurs.
/// It reloads the timer for the next interrupt.
///
/// # Safety
/// Must be called from the IRQ exception handler
pub unsafe fn handle_timer_interrupt() {
    // Read current timer control to check if interrupt is pending
    let ctl: u64;
    asm!("mrs {}, CNTP_CTL_EL0", out(reg) ctl);

    // Check if ISTATUS bit is set (interrupt pending)
    if (ctl & 0x04) != 0 {
        // Reload timer for next interrupt
        // We use the previously configured interval
        let freq = read_cntfrq();
        let now = read_cntpct();

        // Set next interrupt at 1 second from now
        // TODO: Make this configurable
        let next = now + freq;
        asm!("msr CNTP_CVAL_EL0, {}", in(reg) next);
        asm!("isb");
    }
}
