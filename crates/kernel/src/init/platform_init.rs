//! Phase 1: Platform initialization
//!
//! This module handles platform detection and early peripheral initialization:
//! - FDT/DTB parsing and platform detection
//! - UART early console
//! - Time/Clock initialization
//! - Boot timestamp recording
//!
//! # Thread Safety
//!
//! This phase must complete before any multi-threading or SMP initialization.
//! All functions use thread-safe platform detection via Once and Atomic types.

use super::{InitError, InitResult};

/// Helper to print messages during early init
#[inline(always)]
unsafe fn uart_print(msg: &[u8]) {
    let base = crate::platform::active().uart().base as *mut u32;
    for &b in msg {
        core::ptr::write_volatile(base, b as u32);
    }
}

/// Initialize platform detection from FDT/DTB
///
/// Attempts to parse the device tree and detect the platform type.
/// Falls back to default (QEMU virt) if parsing fails.
///
/// # Safety
/// Must be called once during early boot
pub unsafe fn detect_platform() -> InitResult<()> {
    uart_print(b"PLATFORM: DETECTING\n");

    #[cfg(feature = "dt-override")]
    {
        let dtb_ptr = crate::DTB_PTR;
        if !dtb_ptr.is_null() {
            let detected = crate::platform::override_with_dtb(dtb_ptr);
            if detected {
                uart_print(b"PLATFORM: FDT PARSED\n");
                let platform_type = crate::platform::detected_type();
                match platform_type {
                    crate::platform::PlatformType::RaspberryPi5 => {
                        uart_print(b"PLATFORM: RASPBERRY PI 5\n");
                    }
                    crate::platform::PlatformType::QemuVirt => {
                        uart_print(b"PLATFORM: QEMU VIRT\n");
                    }
                    crate::platform::PlatformType::Unknown => {
                        uart_print(b"PLATFORM: UNKNOWN (USING FDT)\n");
                    }
                }
                return Ok(());
            } else {
                uart_print(b"PLATFORM: FDT PARSE FAILED\n");
            }
        }
    }

    // Default platform
    uart_print(b"PLATFORM: QEMU VIRT (DEFAULT)\n");
    Ok(())
}

/// Initialize UART for console output
///
/// # Safety
/// Must be called after platform detection
pub unsafe fn init_uart() -> InitResult<()> {
    uart_print(b"UART: INIT\n");

    crate::uart::init();

    uart_print(b"UART: READY\n");
    Ok(())
}

/// Initialize time/clock subsystem
///
/// Records boot timestamp and emits counter frequency for diagnostics.
///
/// # Safety
/// Must be called after UART initialization
pub unsafe fn init_time() -> InitResult<()> {
    uart_print(b"TIME: INIT\n");

    // Emit counter frequency for timing sanity check
    let mut frq: u64;
    core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);

    uart_print(b"TIME: CNTFRQ=");
    print_number(frq as usize);
    uart_print(b" HZ\n");

    // Initialize boot timestamp
    crate::time::init_boot_timestamp();

    uart_print(b"TIME: READY\n");
    Ok(())
}

/// Print a decimal number to UART
unsafe fn print_number(mut n: usize) {
    if n == 0 {
        uart_print(b"0");
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
        let base = crate::platform::active().uart().base as *mut u32;
        core::ptr::write_volatile(base, buf[i] as u32);
    }
}

/// Initialize platform subsystem (Phase 1)
///
/// Runs all platform initialization steps in order:
/// 1. Platform detection (FDT parsing)
/// 2. UART console
/// 3. Time/Clock setup
///
/// # Safety
/// Must be called once after boot phase completes
pub unsafe fn init_platform() -> InitResult<()> {
    detect_platform()?;
    init_uart()?;
    init_time()?;
    Ok(())
}
