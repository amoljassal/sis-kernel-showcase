//! # HPET (High Precision Event Timer)
//!
//! This module provides support for the HPET, a high-resolution hardware timer
//! introduced to replace the legacy PIT (Programmable Interval Timer).
//!
//! ## HPET vs PIT
//!
//! The HPET offers significant improvements over the PIT:
//! - **Higher Resolution**: Femtosecond vs microsecond precision
//! - **64-bit Counter**: No overflow issues (PIT is 16-bit)
//! - **Multiple Timers**: At least 3 comparators vs single PIT channel
//! - **Better Accuracy**: Crystal oscillator vs bus clock
//! - **No Calibration**: Frequency specified in ACPI
//!
//! ## Hardware Architecture
//!
//! The HPET consists of:
//! - **Main Counter**: 64-bit up-counter (read-only)
//! - **Comparators**: 3-32 independent timer comparators
//! - **Configuration**: Global and per-comparator settings
//! - **Status**: Interrupt status flags
//!
//! ## Memory Map
//!
//! HPET is memory-mapped, with base address specified in ACPI HPET table:
//!
//! ```text
//! Offset  Size  Name                    Description
//! ------  ----  ----------------------  ---------------------------
//! 0x000   8     Capabilities & ID       HPET capabilities
//! 0x010   8     Configuration           Global configuration
//! 0x020   8     Interrupt Status        Interrupt status flags
//! 0x0F0   8     Main Counter Value      64-bit counter (R/W)
//! 0x100   8     Timer 0 Config/Cap      Timer 0 configuration
//! 0x108   8     Timer 0 Comparator      Timer 0 compare value
//! 0x120   8     Timer 1 Config/Cap      Timer 1 configuration
//! 0x128   8     Timer 1 Comparator      Timer 1 compare value
//! ...
//! ```
//!
//! ## Capabilities Register Format
//!
//! ```text
//! Bits 63-32: Counter period (femtoseconds)
//! Bits 31-16: Vendor ID
//! Bit  15:    Legacy replacement capable
//! Bit  14:    Reserved
//! Bit  13:    Counter size (0=32-bit, 1=64-bit)
//! Bits 12-8:  Number of comparators - 1
//! Bits 7-0:   Hardware revision
//! ```
//!
//! ## Configuration Register Format
//!
//! ```text
//! Bit 1: Legacy replacement mode enable
//! Bit 0: Overall HPET enable
//! ```
//!
//! ## Usage Scenarios
//!
//! ### TSC Calibration
//!
//! Use HPET's known frequency to calibrate TSC:
//! ```rust
//! let start_hpet = hpet.read_counter();
//! let start_tsc = read_tsc();
//! // Wait for N HPET ticks
//! let end_hpet = hpet.read_counter();
//! let end_tsc = read_tsc();
//! let tsc_freq = (end_tsc - start_tsc) * hpet_freq / (end_hpet - start_hpet);
//! ```
//!
//! ### High-Resolution Delays
//!
//! ```rust
//! let target = hpet.read_counter() + ns_to_ticks(delay_ns);
//! while hpet.read_counter() < target {
//!     core::hint::spin_loop();
//! }
//! ```
//!
//! ### Periodic Timer
//!
//! Configure comparator for periodic interrupts at precise intervals.
//!
//! ## ACPI Integration
//!
//! The HPET base address and capabilities are described in the ACPI HPET table.
//! We'll parse this in M9 (ACPI), but for M2 we'll use a common address (0xFED00000).
//!
//! ## Safety Considerations
//!
//! - HPET registers must be accessed with correct size (32-bit or 64-bit)
//! - Counter can wrap around (64-bit counter takes ~58,000 years at 14 MHz)
//! - Comparator values must account for counter increment rate
//! - Legacy mode conflicts with PIT/RTC if enabled

use core::ptr::{read_volatile, write_volatile};
use x86_64::VirtAddr;

/// Common HPET base address (specified in ACPI, but this is the usual location)
const HPET_DEFAULT_BASE: u64 = 0xFED00000;

/// HPET register offsets
const HPET_REG_CAPS: usize = 0x000;          // Capabilities & ID
const HPET_REG_CONFIG: usize = 0x010;        // Global configuration
const HPET_REG_INT_STATUS: usize = 0x020;    // Interrupt status
const HPET_REG_COUNTER: usize = 0x0F0;       // Main counter value

/// Timer comparator base offset (each timer is 32 bytes apart)
const HPET_TIMER_BASE: usize = 0x100;
const HPET_TIMER_STRIDE: usize = 0x020;

/// HPET Configuration register bits
const HPET_CFG_ENABLE: u64 = 1 << 0;         // Global enable
const HPET_CFG_LEGACY: u64 = 1 << 1;         // Legacy replacement mode

/// HPET Timer Configuration register bits
const HPET_TIMER_INT_ENABLE: u64 = 1 << 2;   // Interrupt enable
const HPET_TIMER_PERIODIC: u64 = 1 << 3;     // Periodic mode
const HPET_TIMER_PERIODIC_CAP: u64 = 1 << 4; // Periodic mode capable
const HPET_TIMER_SIZE_64: u64 = 1 << 5;      // 64-bit comparator
const HPET_TIMER_VAL_SET: u64 = 1 << 6;      // Value set for periodic

/// High Precision Event Timer
pub struct Hpet {
    /// Virtual base address
    base: VirtAddr,
    /// HPET frequency in Hz
    frequency: u64,
    /// Counter period in femtoseconds
    period_fs: u64,
    /// Number of comparators
    num_comparators: u8,
    /// Counter is 64-bit (vs 32-bit)
    is_64bit: bool,
}

impl Hpet {
    /// Create a new HPET instance
    ///
    /// # Safety
    ///
    /// - `base_addr` must point to valid HPET MMIO region
    /// - Caller must ensure HPET is present at that address
    pub unsafe fn new(base_addr: u64) -> Result<Self, &'static str> {
        // TODO: Proper memory mapping via MM subsystem (M3)
        // For now, use identity mapping
        let base = VirtAddr::new(base_addr);

        // Read capabilities register
        let caps = read_volatile((base.as_u64() + HPET_REG_CAPS as u64) as *const u64);

        // Extract information from capabilities
        let period_fs = caps >> 32; // Bits 63-32: counter period (femtoseconds)
        let rev_id = (caps & 0xFF) as u8;
        let num_comparators = (((caps >> 8) & 0x1F) as u8) + 1; // Bits 12-8: num - 1
        let is_64bit = ((caps >> 13) & 1) != 0;

        if period_fs == 0 || period_fs > 100_000_000 {
            return Err("Invalid HPET period");
        }

        // Calculate frequency: freq = 1e15 / period_fs
        let frequency = 1_000_000_000_000_000 / period_fs;

        let mut hpet = Self {
            base,
            frequency,
            period_fs,
            num_comparators,
            is_64bit,
        };

        hpet.init();

        Ok(hpet)
    }

    /// Initialize the HPET
    unsafe fn init(&mut self) {
        // Disable HPET
        let config_addr = (self.base.as_u64() + HPET_REG_CONFIG as u64) as *mut u64;
        let mut config = read_volatile(config_addr);
        config &= !HPET_CFG_ENABLE;
        write_volatile(config_addr, config);

        // Reset main counter to 0
        let counter_addr = (self.base.as_u64() + HPET_REG_COUNTER as u64) as *mut u64;
        write_volatile(counter_addr, 0);

        // Disable legacy mode (we'll use APIC)
        config &= !HPET_CFG_LEGACY;
        write_volatile(config_addr, config);

        // Enable HPET
        config |= HPET_CFG_ENABLE;
        write_volatile(config_addr, config);
    }

    /// Read the current counter value
    pub fn read_counter(&self) -> u64 {
        unsafe {
            let counter_addr = (self.base.as_u64() + HPET_REG_COUNTER as u64) as *const u64;
            read_volatile(counter_addr)
        }
    }

    /// Get the HPET frequency in Hz
    pub fn frequency(&self) -> u64 {
        self.frequency
    }

    /// Get the counter period in femtoseconds
    pub fn period_fs(&self) -> u64 {
        self.period_fs
    }

    /// Get the number of comparators
    pub fn num_comparators(&self) -> u8 {
        self.num_comparators
    }

    /// Convert nanoseconds to HPET ticks
    pub fn ns_to_ticks(&self, ns: u64) -> u64 {
        // ticks = (ns * frequency) / 1_000_000_000
        (ns * self.frequency) / 1_000_000_000
    }

    /// Convert HPET ticks to nanoseconds
    pub fn ticks_to_ns(&self, ticks: u64) -> u64 {
        // ns = (ticks * 1_000_000_000) / frequency
        (ticks * 1_000_000_000) / self.frequency
    }

    /// Calibrate TSC using HPET
    ///
    /// Measures TSC frequency by comparing TSC increments to HPET ticks
    /// over a known duration.
    ///
    /// # Arguments
    ///
    /// * `duration_ms` - Calibration duration in milliseconds (longer = more accurate)
    ///
    /// Returns TSC frequency in Hz
    pub fn calibrate_tsc(&self, duration_ms: u32) -> u64 {
        use crate::arch::x86_64::tsc::read_tsc;

        // Calculate target HPET ticks for the calibration duration
        let target_ticks = (duration_ms as u64 * self.frequency) / 1000;

        // Read initial values
        let start_hpet = self.read_counter();
        let start_tsc = read_tsc();

        // Wait for the specified duration
        let end_target = start_hpet + target_ticks;
        while self.read_counter() < end_target {
            core::hint::spin_loop();
        }

        // Read final values
        let end_hpet = self.read_counter();
        let end_tsc = read_tsc();

        // Calculate TSC frequency
        // tsc_freq = (tsc_delta * hpet_freq) / hpet_delta
        let tsc_delta = end_tsc - start_tsc;
        let hpet_delta = end_hpet - start_hpet;

        (tsc_delta * self.frequency) / hpet_delta
    }

    /// Busy-wait delay using HPET
    ///
    /// Delays for approximately the specified number of nanoseconds.
    ///
    /// # Safety
    ///
    /// This is a busy-wait (CPU spinning) delay. Should only be used when
    /// absolutely necessary.
    pub unsafe fn delay_ns(&self, ns: u64) {
        let ticks = self.ns_to_ticks(ns);
        let start = self.read_counter();
        let target = start + ticks;

        while self.read_counter() < target {
            core::hint::spin_loop();
        }
    }
}

/// Global HPET instance (if available)
static mut HPET: Option<Hpet> = None;

/// Initialize HPET
///
/// # Safety
///
/// Must be called once during boot, after memory management is set up.
pub unsafe fn init() -> Result<(), &'static str> {
    // Try to initialize HPET at default address
    // TODO: Read actual address from ACPI HPET table (M9)
    let hpet = Hpet::new(HPET_DEFAULT_BASE)?;

    crate::arch::x86_64::serial::serial_write(b"[HPET] High Precision Event Timer initialized\n");
    crate::arch::x86_64::serial::serial_write(b"[HPET] Frequency: ");
    print_u64(hpet.frequency() / 1_000_000);
    crate::arch::x86_64::serial::serial_write(b" MHz\n");
    crate::arch::x86_64::serial::serial_write(b"[HPET] Period: ");
    print_u64(hpet.period_fs());
    crate::arch::x86_64::serial::serial_write(b" fs\n");
    crate::arch::x86_64::serial::serial_write(b"[HPET] Comparators: ");
    print_u32(hpet.num_comparators() as u32);
    crate::arch::x86_64::serial::serial_write(b"\n");

    HPET = Some(hpet);

    Ok(())
}

/// Get reference to global HPET instance
pub fn get() -> Option<&'static Hpet> {
    unsafe { HPET.as_ref() }
}

/// Calibrate TSC using HPET (if available)
///
/// Returns TSC frequency in Hz, or None if HPET is not available.
pub fn calibrate_tsc(duration_ms: u32) -> Option<u64> {
    unsafe {
        HPET.as_ref().map(|hpet| hpet.calibrate_tsc(duration_ms))
    }
}

/// Helper to print u64
fn print_u64(mut n: u64) {
    if n == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::arch::x86_64::serial::serial_write_byte(buf[i]);
    }
}

/// Helper to print u32
fn print_u32(mut n: u32) {
    if n == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 10];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::arch::x86_64::serial::serial_write_byte(buf[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ns_conversion() {
        // Create a mock HPET with 14.318 MHz frequency (common)
        let hpet = unsafe {
            Hpet {
                base: VirtAddr::new(0),
                frequency: 14_318_180,
                period_fs: 69841279,
                num_comparators: 3,
                is_64bit: true,
            }
        };

        // 1 second = frequency ticks
        let ticks = hpet.ns_to_ticks(1_000_000_000);
        assert!((ticks as i64 - hpet.frequency as i64).abs() < 1000);

        // Convert back
        let ns = hpet.ticks_to_ns(hpet.frequency);
        assert!((ns as i64 - 1_000_000_000).abs() < 1000);
    }
}
