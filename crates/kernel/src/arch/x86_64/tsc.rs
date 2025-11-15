//! # Time Stamp Counter (TSC)
//!
//! The TSC is a high-resolution CPU cycle counter that increments on every clock cycle.
//! It's accessed via the RDTSC instruction and provides the highest resolution timing
//! available on x86_64.
//!
//! ## TSC Characteristics
//!
//! Modern CPUs provide different TSC variants:
//!
//! ### Legacy TSC
//! - Increments at CPU frequency
//! - Affected by frequency scaling (SpeedStep, Turbo Boost)
//! - Can be different across CPU cores
//! - **Not reliable** for timekeeping!
//!
//! ### Constant TSC
//! - Increments at a constant rate
//! - Unaffected by frequency scaling
//! - Synchronized across cores
//! - **Suitable for timekeeping**
//!
//! ### Invariant TSC
//! - Constant TSC + guaranteed not to stop in deep C-states
//! - Best for timekeeping
//! - Available on most modern CPUs
//!
//! ## TSC Detection (CPUID)
//!
//! - **CPUID.01H:EDX[4]**: TSC supported
//! - **CPUID.80000007H:EDX[8]**: Invariant TSC supported
//!
//! ## TSC Calibration
//!
//! Since the TSC increments at CPU frequency, we need to calibrate it to convert
//! TSC ticks to actual time. Calibration methods (in order of accuracy):
//!
//! 1. **CPUID.15H**: TSC frequency (best, if available)
//! 2. **MSR 0xCE (IA32_PLATFORM_INFO)**: Base frequency on Intel
//! 3. **HPET**: Calibrate TSC against HPET (accurate)
//! 4. **PIT**: Calibrate TSC against PIT (less accurate)
//! 5. **Assume 1 GHz**: Last resort (very inaccurate)
//!
//! ## Usage
//!
//! ```rust
//! // Read current TSC value
//! let start = read_tsc();
//!
//! // ... do work ...
//!
//! let end = read_tsc();
//! let elapsed_cycles = end - start;
//!
//! // Convert to nanoseconds (requires calibration)
//! let elapsed_ns = tsc_to_ns(elapsed_cycles);
//! ```
//!
//! ## Safety Considerations
//!
//! - RDTSC can be serializing or non-serializing depending on CPU
//! - For accurate measurements, use RDTSCP or fence instructions
//! - TSC may not be synchronized across NUMA nodes
//! - Virtualization can affect TSC behavior

use core::sync::atomic::{AtomicU64, Ordering};

/// TSC frequency in Hz (calibrated at boot)
///
/// This is set during TSC calibration and used to convert TSC ticks to time.
static TSC_FREQUENCY_HZ: AtomicU64 = AtomicU64::new(0);

/// Read the Time Stamp Counter
///
/// Returns the current TSC value (CPU cycles since boot/reset).
///
/// # Note
///
/// This uses RDTSC which is not serializing. For precise measurements,
/// consider using `read_tsc_serialized()`.
#[inline]
pub fn read_tsc() -> u64 {
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
}

/// Read the Time Stamp Counter with serialization
///
/// Uses CPUID as a serializing instruction before RDTSC to ensure
/// all previous instructions have completed. This provides more
/// accurate timing measurements but is slower than plain RDTSC.
#[inline]
pub fn read_tsc_serialized() -> u64 {
    unsafe {
        // CPUID is a serializing instruction
        core::arch::asm!(
            "cpuid",
            "rdtsc",
            out("rax") _,
            out("rbx") _,
            out("rcx") _,
            out("rdx") _,
            options(nostack)
        );
        core::arch::x86_64::_rdtsc()
    }
}

/// Read the Time Stamp Counter and processor ID
///
/// Uses RDTSCP instruction which provides both TSC and CPU ID.
/// RDTSCP is serializing with respect to subsequent instructions.
///
/// Returns (tsc, processor_id).
#[inline]
pub fn read_tscp() -> (u64, u32) {
    let tsc: u64;
    let cpu_id: u32;

    unsafe {
        core::arch::asm!(
            "rdtscp",
            out("rax") tsc,
            out("rcx") cpu_id,
            out("rdx") _,
            options(nostack)
        );
    }

    (tsc, cpu_id)
}

/// Get the calibrated TSC frequency in Hz
///
/// Returns the TSC frequency determined during boot calibration.
/// Returns 0 if TSC has not been calibrated yet.
pub fn get_tsc_frequency() -> u64 {
    TSC_FREQUENCY_HZ.load(Ordering::Relaxed)
}

/// Set the TSC frequency (called during calibration)
///
/// # Safety
///
/// Should only be called once during boot, after successful calibration.
pub unsafe fn set_tsc_frequency(freq_hz: u64) {
    TSC_FREQUENCY_HZ.store(freq_hz, Ordering::Relaxed);
}

/// Convert TSC ticks to nanoseconds
///
/// Requires TSC to be calibrated (frequency known).
/// Returns 0 if TSC is not calibrated.
pub fn tsc_to_ns(tsc_ticks: u64) -> u64 {
    let freq = get_tsc_frequency();
    if freq == 0 {
        return 0;
    }

    // Calculate: (tsc_ticks * 1_000_000_000) / freq_hz
    // Use 128-bit arithmetic to avoid overflow
    let ticks_u128 = tsc_ticks as u128;
    let ns_u128 = (ticks_u128 * 1_000_000_000) / (freq as u128);
    ns_u128 as u64
}

/// Convert nanoseconds to TSC ticks
///
/// Requires TSC to be calibrated (frequency known).
/// Returns 0 if TSC is not calibrated.
pub fn ns_to_tsc(ns: u64) -> u64 {
    let freq = get_tsc_frequency();
    if freq == 0 {
        return 0;
    }

    // Calculate: (ns * freq_hz) / 1_000_000_000
    // Use 128-bit arithmetic to avoid overflow
    let ns_u128 = ns as u128;
    let ticks_u128 = (ns_u128 * (freq as u128)) / 1_000_000_000;
    ticks_u128 as u64
}

/// Calibrate TSC using CPUID (if available)
///
/// Intel CPUs with CPUID.15H provide TSC frequency information.
/// This is the most accurate calibration method.
///
/// Returns TSC frequency in Hz, or None if not available.
pub fn calibrate_tsc_cpuid() -> Option<u64> {
    use raw_cpuid::CpuId;

    let cpuid = CpuId::new();

    // Try CPUID.15H (TSC/Core Crystal Clock Information)
    if let Some(tsc_info) = cpuid.get_tsc_info() {
        let crystal_hz = tsc_info.tsc_frequency();
        if let Some(freq) = crystal_hz {
            return Some(freq);
        }
    }

    // Try CPUID.16H (Processor Frequency Information) on Intel
    if let Some(freq_info) = cpuid.get_processor_frequency_info() {
        // Base frequency in MHz
        let base_mhz = freq_info.processor_base_frequency();
        if base_mhz > 0 {
            return Some(base_mhz as u64 * 1_000_000);
        }
    }

    None
}

/// Calibrate TSC using MSR (Intel-specific)
///
/// Reads IA32_PLATFORM_INFO MSR to get base frequency.
/// Only works on Intel CPUs.
///
/// Returns TSC frequency in Hz, or None if not available.
pub unsafe fn calibrate_tsc_msr() -> Option<u64> {
    // IA32_PLATFORM_INFO (0xCE) on Intel
    const IA32_PLATFORM_INFO: u32 = 0xCE;

    // Try to read MSR (may fault on non-Intel or virtualized systems)
    let platform_info = super::rdmsr(IA32_PLATFORM_INFO);

    // Bits 15:8 contain the maximum non-turbo ratio
    let ratio = ((platform_info >> 8) & 0xFF) as u64;

    if ratio == 0 {
        return None;
    }

    // Base clock is typically 100 MHz on modern Intel CPUs
    const BASE_CLOCK_HZ: u64 = 100_000_000;
    Some(ratio * BASE_CLOCK_HZ)
}

/// Initialize TSC
///
/// Attempts to calibrate the TSC frequency using available methods.
/// Should be called during early boot.
///
/// # Safety
///
/// Must be called once during boot, after CPU initialization.
pub unsafe fn init_tsc() {
    // Try CPUID-based calibration first (most accurate)
    if let Some(freq) = calibrate_tsc_cpuid() {
        set_tsc_frequency(freq);
        crate::arch::x86_64::serial::serial_write(b"[TSC] Calibrated via CPUID: ");
        print_u64(freq);
        crate::arch::x86_64::serial::serial_write(b" Hz\n");
        return;
    }

    // Try MSR-based calibration (Intel only)
    if let Some(freq) = calibrate_tsc_msr() {
        set_tsc_frequency(freq);
        crate::arch::x86_64::serial::serial_write(b"[TSC] Calibrated via MSR: ");
        print_u64(freq);
        crate::arch::x86_64::serial::serial_write(b" Hz\n");
        return;
    }

    // Try HPET-based calibration (M2+)
    if let Some(freq) = crate::arch::x86_64::hpet::calibrate_tsc(100) {
        set_tsc_frequency(freq);
        crate::arch::x86_64::serial::serial_write(b"[TSC] Calibrated via HPET: ");
        print_u64(freq);
        crate::arch::x86_64::serial::serial_write(b" Hz\n");
        return;
    }

    // Try PIT-based calibration (M1+)
    let freq = crate::arch::x86_64::pit::calibrate_tsc(100);
    if freq > 0 {
        set_tsc_frequency(freq);
        return; // PIT calibration already prints message
    }

    // Fallback: assume 1 GHz (very inaccurate, but better than nothing)
    const FALLBACK_FREQ: u64 = 1_000_000_000; // 1 GHz
    set_tsc_frequency(FALLBACK_FREQ);
    crate::arch::x86_64::serial::serial_write(b"[TSC] Using fallback frequency: ");
    print_u64(FALLBACK_FREQ);
    crate::arch::x86_64::serial::serial_write(b" Hz (inaccurate!)\n");
}

/// Helper function to print u64 to serial (temporary, until we have proper formatting)
fn print_u64(mut n: u64) {
    if n == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 20]; // Max 20 digits for u64
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    // Print in reverse order
    while i > 0 {
        i -= 1;
        crate::arch::x86_64::serial::serial_write_byte(buf[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_tsc() {
        let tsc1 = read_tsc();
        let tsc2 = read_tsc();

        // TSC should be monotonically increasing
        assert!(tsc2 >= tsc1, "TSC went backwards!");
    }

    #[test]
    fn test_tsc_conversion() {
        unsafe {
            // Set a known frequency for testing
            set_tsc_frequency(1_000_000_000); // 1 GHz

            // 1 second = 1 billion cycles at 1 GHz
            assert_eq!(tsc_to_ns(1_000_000_000), 1_000_000_000);

            // 1 millisecond = 1 million cycles
            assert_eq!(tsc_to_ns(1_000_000), 1_000_000);

            // Test reverse conversion
            assert_eq!(ns_to_tsc(1_000_000_000), 1_000_000_000);
            assert_eq!(ns_to_tsc(1_000_000), 1_000_000);
        }
    }
}
