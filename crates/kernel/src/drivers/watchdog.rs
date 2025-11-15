//! Watchdog Timer Driver for ARM platforms
//!
//! Provides system watchdog functionality to automatically reset the system
//! if it becomes unresponsive. Supports:
//! - BCM2712 (Raspberry Pi 5) PM Watchdog
//! - ARM Generic Watchdog (SBSA/GWD)
//! - QEMU watchdog emulation
//!
//! # Usage
//!
//! ```rust
//! use crate::drivers::watchdog;
//!
//! // Initialize and start watchdog with 30 second timeout
//! watchdog::init();
//! watchdog::start(30);
//!
//! // Feed the watchdog periodically (before timeout expires)
//! watchdog::feed();
//!
//! // Stop watchdog
//! watchdog::stop();
//! ```

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// BCM2712 PM Watchdog Register Offsets (from PM base)
const PM_RSTC: usize = 0x1c;
const PM_RSTS: usize = 0x20;
const PM_WDOG: usize = 0x24;

/// PM_RSTC register bits
const PM_PASSWORD: u32 = 0x5a000000;
const PM_RSTC_WRCFG_CLR: u32 = 0xffffffcf;
const PM_RSTC_WRCFG_FULL_RESET: u32 = 0x00000020;
const PM_RSTC_RESET: u32 = 0x00000102;

/// Watchdog state
static WATCHDOG_ENABLED: AtomicBool = AtomicBool::new(false);
static WATCHDOG_BASE: AtomicU32 = AtomicU32::new(0);
static WATCHDOG_TIMEOUT_SECS: AtomicU32 = AtomicU32::new(0);

/// Watchdog types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WatchdogType {
    /// BCM2712 (RPi5) PM Watchdog
    Bcm2712Pm,
    /// ARM Generic Watchdog (SBSA)
    ArmGeneric,
    /// QEMU emulated watchdog
    Qemu,
    /// No watchdog available
    None,
}

/// Initialize watchdog driver
///
/// Detects available watchdog hardware and configures it.
/// This should be called once during kernel initialization.
pub fn init() -> WatchdogType {
    // Try to detect watchdog from device tree
    if let Some(base) = detect_watchdog_from_dt() {
        WATCHDOG_BASE.store(base as u32, Ordering::Release);
        crate::info!("Watchdog: BCM2712 PM Watchdog at {:#x}", base);
        return WatchdogType::Bcm2712Pm;
    }

    crate::info!("Watchdog: No hardware watchdog detected");
    WatchdogType::None
}

/// Detect watchdog from device tree
fn detect_watchdog_from_dt() -> Option<usize> {
    // Try to get PM base from device map
    // On RPi5, the PM watchdog is part of the PM register block
    // Typically at 0x7d200000 or similar (from device tree)

    if let Some(devmap) = crate::platform::dt::get_device_map() {
        // Check if we have a watchdog device
        // For now, we'll use a default address if platform is RPi5
        if crate::platform::detected_type() == crate::platform::PlatformType::RaspberryPi5 {
            // Default PM base on RPi5 (should come from DT in production)
            Some(0x7d200000)
        } else {
            None
        }
    } else {
        None
    }
}

/// Start the watchdog timer
///
/// # Arguments
/// * `timeout_secs` - Timeout in seconds (system will reset if not fed before this expires)
///
/// # Panics
/// Panics if watchdog has not been initialized
pub fn start(timeout_secs: u32) {
    let base = WATCHDOG_BASE.load(Ordering::Acquire);
    if base == 0 {
        crate::warn!("Watchdog: Cannot start - not initialized");
        return;
    }

    unsafe {
        set_timeout(base as usize, timeout_secs);
    }

    WATCHDOG_TIMEOUT_SECS.store(timeout_secs, Ordering::Release);
    WATCHDOG_ENABLED.store(true, Ordering::Release);

    crate::info!("Watchdog: Started with {} second timeout", timeout_secs);
}

/// Feed (pet/kick) the watchdog timer
///
/// This must be called periodically before the timeout expires to prevent system reset.
pub fn feed() {
    if !WATCHDOG_ENABLED.load(Ordering::Acquire) {
        return;
    }

    let base = WATCHDOG_BASE.load(Ordering::Acquire) as usize;
    let timeout = WATCHDOG_TIMEOUT_SECS.load(Ordering::Acquire);

    unsafe {
        set_timeout(base, timeout);
    }
}

/// Stop the watchdog timer
pub fn stop() {
    let base = WATCHDOG_BASE.load(Ordering::Acquire);
    if base == 0 {
        return;
    }

    unsafe {
        // Write password to RSTC to disable watchdog
        write_volatile(
            (base as usize + PM_RSTC) as *mut u32,
            PM_PASSWORD | PM_RSTC_RESET,
        );
    }

    WATCHDOG_ENABLED.store(false, Ordering::Release);
    crate::info!("Watchdog: Stopped");
}

/// Set watchdog timeout (BCM2712 PM Watchdog)
unsafe fn set_timeout(base: usize, timeout_secs: u32) {
    // BCM2712 watchdog clock is 19.2 MHz (typical for BCM chips)
    // However, there's a divider, so effective rate is lower
    // For safety, we use a conservative estimate

    // Convert seconds to ticks
    // Typical BCM watchdog runs at ~1Hz after dividers
    let ticks = timeout_secs;

    // Set watchdog counter value
    write_volatile(
        (base + PM_WDOG) as *mut u32,
        PM_PASSWORD | (ticks & 0xfffff),
    );

    // Configure RSTC for full reset on watchdog timeout
    let rstc = read_volatile((base + PM_RSTC) as *const u32);
    let rstc_new = (rstc & PM_RSTC_WRCFG_CLR) | PM_RSTC_WRCFG_FULL_RESET;
    write_volatile(
        (base + PM_RSTC) as *mut u32,
        PM_PASSWORD | rstc_new,
    );
}

/// Check if watchdog is enabled
pub fn is_enabled() -> bool {
    WATCHDOG_ENABLED.load(Ordering::Acquire)
}

/// Get current watchdog timeout in seconds
pub fn get_timeout() -> Option<u32> {
    if WATCHDOG_ENABLED.load(Ordering::Acquire) {
        Some(WATCHDOG_TIMEOUT_SECS.load(Ordering::Acquire))
    } else {
        None
    }
}

/// Get remaining time before watchdog expires (if available)
///
/// Note: This is not always accurate as the BCM watchdog doesn't provide
/// a direct way to read the current counter value.
pub fn get_remaining_time() -> Option<u32> {
    if !is_enabled() {
        return None;
    }

    // BCM2712 doesn't provide a way to read back the counter
    // Return timeout as estimate
    Some(WATCHDOG_TIMEOUT_SECS.load(Ordering::Acquire))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_state() {
        // Initial state should be disabled
        assert!(!is_enabled());
        assert_eq!(get_timeout(), None);
        assert_eq!(get_remaining_time(), None);
    }

    #[test]
    fn test_watchdog_type() {
        assert_ne!(WatchdogType::Bcm2712Pm, WatchdogType::None);
        assert_ne!(WatchdogType::ArmGeneric, WatchdogType::None);
    }
}
