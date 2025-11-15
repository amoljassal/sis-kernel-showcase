//! # Power Management for x86_64
//!
//! This module provides system power management functions including:
//! - System reset (reboot)
//! - System poweroff (shutdown)
//! - Power state transitions
//!
//! ## Reset Methods
//!
//! The system reset sequence tries multiple methods in order:
//! 1. **ACPI Reset** - Uses the ACPI FADT reset register (most reliable)
//! 2. **Keyboard Controller Reset** - 8042 keyboard controller (0x64 port)
//! 3. **Triple Fault** - Load invalid IDT and trigger interrupt (last resort)
//!
//! ## Shutdown Methods
//!
//! The system poweroff sequence:
//! 1. **ACPI S5 State** - Uses PM1a/PM1b control registers to enter S5 (soft off)
//! 2. **Halt** - If ACPI fails, halt all CPUs (requires manual power off)
//!
//! ## ACPI Power States
//!
//! - **S0** - Working state (system is on)
//! - **S1** - Sleep with CPU stopped, RAM powered
//! - **S2** - CPU powered off, RAM powered
//! - **S3** - Suspend to RAM (sleep)
//! - **S4** - Suspend to disk (hibernate)
//! - **S5** - Soft off (shutdown)
//!
//! ## Safety
//!
//! All functions in this module are inherently unsafe as they manipulate
//! hardware directly and never return in normal operation.

use x86_64::instructions::port::Port;
use core::sync::atomic::{AtomicBool, Ordering};

/// Flag indicating if ACPI power management is initialized
static ACPI_PM_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// PM1a Control Block I/O port (from ACPI FADT)
static mut PM1A_CONTROL_PORT: Option<u16> = None;

/// PM1b Control Block I/O port (from ACPI FADT)
static mut PM1B_CONTROL_PORT: Option<u16> = None;

/// SLP_TYPa value for S5 state (shutdown)
static mut SLP_TYP_S5: u16 = 0;

/// SLP_EN bit (Sleep Enable)
const SLP_EN: u16 = 1 << 13;

/// Initialize ACPI power management
///
/// Extracts PM1a/PM1b control block addresses and S5 sleep type from ACPI FADT.
///
/// # Safety
/// Must be called after ACPI tables are parsed
pub unsafe fn init() -> Result<(), &'static str> {
    // For now, we don't have full ACPI FADT parsing
    // This would normally read from the FADT table to get:
    // - PM1a_CNT_BLK address
    // - PM1b_CNT_BLK address
    // - SLP_TYPa/SLP_TYPb values from \_S5 package

    // Default values for QEMU (typical PIIX4 PM)
    PM1A_CONTROL_PORT = Some(0x604);  // QEMU default
    PM1B_CONTROL_PORT = None;         // Not used in QEMU
    SLP_TYP_S5 = 0 << 10;             // Sleep type for S5

    ACPI_PM_INITIALIZED.store(true, Ordering::Release);

    Ok(())
}

/// Reset the system (reboot)
///
/// Tries multiple reset methods in order:
/// 1. ACPI reset register
/// 2. Keyboard controller reset (8042)
/// 3. Triple fault
///
/// # Safety
/// This function never returns.
pub unsafe fn system_reset() -> ! {
    crate::arch::x86_64::serial::serial_write(b"\n[POWER] System reset requested\n");

    // Method 1: ACPI Reset (not implemented yet)
    // Would use ACPI FADT reset register if available

    // Method 2: Keyboard controller reset
    crate::arch::x86_64::serial::serial_write(b"[POWER] Attempting keyboard controller reset...\n");

    // The 8042 keyboard controller has a reset line connected to the CPU
    // Writing 0xFE to port 0x64 triggers a CPU reset
    let mut kb_ctrl = Port::<u8>::new(0x64);
    kb_ctrl.write(0xFE);

    // Wait a bit for reset to happen
    delay_ms(1000);

    // Method 3: Triple fault (if keyboard controller failed)
    crate::arch::x86_64::serial::serial_write(b"[POWER] Keyboard controller reset failed, triggering triple fault...\n");

    // Load an invalid IDT and trigger an interrupt
    // This causes a triple fault which resets the CPU
    core::arch::asm!(
        "lidt [{}]",
        "int3",
        in(reg) 0usize,
        options(noreturn)
    );
}

/// Power off the system (shutdown)
///
/// Attempts to enter ACPI S5 state (soft off) via PM1a/PM1b control registers.
/// If ACPI is not available or fails, halts all CPUs.
///
/// # Safety
/// This function never returns.
pub unsafe fn system_poweroff() -> ! {
    crate::arch::x86_64::serial::serial_write(b"\n[POWER] System poweroff requested\n");

    // Try ACPI S5 state if initialized
    if ACPI_PM_INITIALIZED.load(Ordering::Acquire) {
        crate::arch::x86_64::serial::serial_write(b"[POWER] Entering ACPI S5 state (poweroff)...\n");

        // Build PM1 control value: SLP_TYPa | SLP_EN
        let pm1_value = SLP_TYP_S5 | SLP_EN;

        // Write to PM1a_CNT
        if let Some(port_addr) = PM1A_CONTROL_PORT {
            let mut pm1a_port = Port::<u16>::new(port_addr);
            pm1a_port.write(pm1_value);
        }

        // Write to PM1b_CNT if present
        if let Some(port_addr) = PM1B_CONTROL_PORT {
            let mut pm1b_port = Port::<u16>::new(port_addr);
            pm1b_port.write(pm1_value);
        }

        // Wait for poweroff to happen
        delay_ms(5000);

        crate::arch::x86_64::serial::serial_write(b"[POWER] ACPI poweroff failed\n");
    }

    // Fall back to halt
    crate::arch::x86_64::serial::serial_write(b"[POWER] Halting CPUs (manual power off required)\n");

    // Disable interrupts and halt all CPUs
    halt_forever();
}

/// Halt the CPU forever
///
/// Disables interrupts and executes HLT instruction in a loop.
/// This is used when shutdown fails or as a fallback.
///
/// # Safety
/// This function never returns.
fn halt_forever() -> ! {
    loop {
        x86_64::instructions::interrupts::disable();
        x86_64::instructions::hlt();
    }
}

/// Delay for approximately the specified number of milliseconds
///
/// Uses a simple busy-wait loop based on TSC or PIT.
/// Not precise, but good enough for power management delays.
unsafe fn delay_ms(ms: u64) {
    // Simple PIT-based delay (very approximate)
    // PIT channel 0 runs at ~1.193 MHz
    // We'll just do a busy loop for now

    const PIT_FREQUENCY: u64 = 1193182;
    let ticks = (ms * PIT_FREQUENCY) / 1000;

    // Read TSC for delay if available
    let start = unsafe { core::arch::x86_64::_rdtsc() };
    let tsc_freq = crate::arch::x86_64::tsc::get_tsc_frequency();

    if tsc_freq > 0 {
        // Use TSC for accurate delay
        let target_cycles = (ms * tsc_freq) / 1000;
        while unsafe { core::arch::x86_64::_rdtsc() } - start < target_cycles {
            core::hint::spin_loop();
        }
    } else {
        // Fall back to busy wait
        for _ in 0..(ticks / 1000) {
            core::hint::spin_loop();
        }
    }
}

/// Enter a specific ACPI sleep state
///
/// # Safety
/// Depending on the sleep state, this may not return.
pub unsafe fn enter_sleep_state(state: SleepState) -> Result<(), &'static str> {
    if !ACPI_PM_INITIALIZED.load(Ordering::Acquire) {
        return Err("ACPI power management not initialized");
    }

    match state {
        SleepState::S5 => {
            // S5 is soft off, never returns
            system_poweroff();
        }
        _ => {
            // Other sleep states not implemented yet
            Err("Sleep state not implemented")
        }
    }
}

/// ACPI Sleep States
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepState {
    /// S0 - Working state
    S0,
    /// S1 - CPU stop, RAM powered
    S1,
    /// S2 - CPU off, RAM powered
    S2,
    /// S3 - Suspend to RAM (sleep)
    S3,
    /// S4 - Suspend to disk (hibernate)
    S4,
    /// S5 - Soft off (shutdown)
    S5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleep_state_enum() {
        assert_eq!(SleepState::S5, SleepState::S5);
        assert_ne!(SleepState::S3, SleepState::S5);
    }
}
