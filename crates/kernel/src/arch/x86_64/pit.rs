//! # PIT (Programmable Interval Timer) - Intel 8253/8254
//!
//! This module provides support for the Programmable Interval Timer (PIT), also known as
//! the Intel 8253/8254 timer chip. The PIT is a legacy timing device found on all x86 systems.
//!
//! ## Historical Context
//!
//! The Intel 8253 was introduced in 1981 for the original IBM PC. The 8254 (1982) added
//! several features but is largely backward compatible. While modern systems use HPET
//! (High Precision Event Timer) and APIC timers, the PIT is still useful for:
//! - Early boot timing (before HPET is initialized)
//! - TSC calibration
//! - Fallback when better timers aren't available
//!
//! ## Architecture
//!
//! The PIT contains three independent 16-bit counters (channels):
//!
//! ```text
//! Channel 0: System timer (IRQ 0) - Used for periodic interrupts
//! Channel 1: DRAM refresh (legacy, unused on modern systems)
//! Channel 2: PC speaker control
//! ```
//!
//! We primarily use Channel 0 for system timing.
//!
//! ## I/O Ports
//!
//! - **0x40**: Channel 0 data port (read/write counter)
//! - **0x41**: Channel 1 data port (unused)
//! - **0x42**: Channel 2 data port (PC speaker)
//! - **0x43**: Mode/Command register (write-only)
//!
//! ## Operating Modes
//!
//! The PIT supports 6 operating modes:
//!
//! - **Mode 0**: Interrupt on Terminal Count (one-shot)
//! - **Mode 1**: Hardware Retriggerable One-shot
//! - **Mode 2**: Rate Generator (periodic, sawtooth wave)
//! - **Mode 3**: Square Wave Generator (periodic, square wave)
//! - **Mode 4**: Software Triggered Strobe
//! - **Mode 5**: Hardware Triggered Strobe
//!
//! We typically use Mode 2 (Rate Generator) or Mode 3 (Square Wave) for periodic interrupts.
//!
//! ## Frequency Calculation
//!
//! The PIT has a fixed input frequency of 1.193182 MHz (1193182 Hz).
//! To get a desired output frequency:
//!
//! ```text
//! divisor = PIT_FREQUENCY / desired_frequency
//! ```
//!
//! For example, for 1000 Hz (1 ms per tick):
//! ```text
//! divisor = 1193182 / 1000 = 1193
//! ```
//!
//! ## Command Register Format
//!
//! ```text
//! Bits 7-6: Select Channel (00=0, 01=1, 10=2, 11=read-back)
//! Bits 5-4: Access Mode (00=latch, 01=lobyte, 10=hibyte, 11=lobyte/hibyte)
//! Bits 3-1: Operating Mode (000-101 for modes 0-5, 11x for mode 2/3)
//! Bit 0:    BCD/Binary (0=16-bit binary, 1=4-digit BCD)
//! ```
//!
//! ## Common Configuration
//!
//! For periodic timer at 1000 Hz:
//! ```text
//! Command: 0x36
//!   Bits 7-6: 00 (Channel 0)
//!   Bits 5-4: 11 (lobyte/hibyte)
//!   Bits 3-1: 011 (Mode 3 - Square Wave)
//!   Bit 0:    0 (Binary mode)
//!
//! Divisor: 1193 (for 1000 Hz)
//! ```
//!
//! ## TSC Calibration
//!
//! The PIT can be used to calibrate the TSC (Time Stamp Counter):
//! 1. Read TSC
//! 2. Wait for N PIT ticks (known duration)
//! 3. Read TSC again
//! 4. Calculate: TSC_freq = (TSC_end - TSC_start) * PIT_freq / PIT_ticks
//!
//! ## Safety Considerations
//!
//! - PIT programming uses I/O ports, requiring `unsafe` code
//! - Channel 0 must not be reprogrammed while IRQ 0 is enabled
//! - Reading the counter requires latching to avoid race conditions
//! - PIT has limited precision (~838 ns resolution)

use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::port::Port;

/// PIT base frequency in Hz (1.193182 MHz)
///
/// This is the fundamental oscillator frequency of the PIT chip.
/// All divisors are calculated relative to this frequency.
pub const PIT_FREQUENCY: u32 = 1_193_182;

/// Channel 0 data port (system timer)
const CHANNEL0: u16 = 0x40;

/// Channel 1 data port (RAM refresh - unused)
const CHANNEL1: u16 = 0x41;

/// Channel 2 data port (PC speaker)
const CHANNEL2: u16 = 0x42;

/// Mode/Command register
const COMMAND: u16 = 0x43;

/// Default timer frequency (1000 Hz = 1 ms per tick)
pub const DEFAULT_FREQUENCY: u32 = 1000;

/// Global tick counter
///
/// Incremented by the timer interrupt handler. Represents the number of
/// timer ticks since boot.
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

/// PIT operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PitMode {
    /// Mode 0: Interrupt on Terminal Count (one-shot)
    InterruptOnTerminalCount = 0,
    /// Mode 1: Hardware Retriggerable One-shot
    HardwareOneShot = 1,
    /// Mode 2: Rate Generator (periodic, sawtooth)
    RateGenerator = 2,
    /// Mode 3: Square Wave Generator (periodic, square wave)
    SquareWaveGenerator = 3,
    /// Mode 4: Software Triggered Strobe
    SoftwareStrobe = 4,
    /// Mode 5: Hardware Triggered Strobe
    HardwareStrobe = 5,
}

/// PIT channel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PitChannel {
    Channel0 = 0,  // System timer
    Channel1 = 1,  // RAM refresh (unused)
    Channel2 = 2,  // PC speaker
}

/// Initialize the PIT for periodic interrupts
///
/// Configures Channel 0 to generate periodic interrupts at the specified frequency.
///
/// # Arguments
///
/// * `frequency` - Desired interrupt frequency in Hz (typically 1000 for 1 ms ticks)
///
/// # Safety
///
/// Must be called with interrupts disabled. After initialization, enable IRQ 0
/// to start receiving timer interrupts.
pub unsafe fn init(frequency: u32) {
    let divisor = PIT_FREQUENCY / frequency;

    // Sanity check divisor
    if divisor > 65535 || divisor == 0 {
        panic!("Invalid PIT divisor: {}", divisor);
    }

    // Configure Channel 0, Mode 3 (Square Wave), lobyte/hibyte
    // Command: 0x36
    //   Bits 7-6: 00 (Channel 0)
    //   Bits 5-4: 11 (lobyte/hibyte)
    //   Bits 3-1: 011 (Mode 3)
    //   Bit 0:    0 (Binary)
    let command: u8 = 0x36;
    Port::<u8>::new(COMMAND).write(command);

    // Write divisor (low byte, then high byte)
    Port::<u8>::new(CHANNEL0).write((divisor & 0xFF) as u8);
    Port::<u8>::new(CHANNEL0).write(((divisor >> 8) & 0xFF) as u8);

    // Calculate actual frequency (may differ slightly due to integer division)
    let actual_freq = PIT_FREQUENCY / divisor;

    crate::arch::x86_64::serial::serial_write(b"[PIT] Programmable Interval Timer initialized\n");
    crate::arch::x86_64::serial::serial_write(b"[PIT] Target frequency: ");
    print_u32(frequency);
    crate::arch::x86_64::serial::serial_write(b" Hz\n");
    crate::arch::x86_64::serial::serial_write(b"[PIT] Actual frequency: ");
    print_u32(actual_freq);
    crate::arch::x86_64::serial::serial_write(b" Hz\n");
    crate::arch::x86_64::serial::serial_write(b"[PIT] Divisor: ");
    print_u32(divisor);
    crate::arch::x86_64::serial::serial_write(b"\n");
}

/// Set PIT frequency
///
/// Reconfigures the PIT to the specified frequency.
///
/// # Safety
///
/// Should be called with IRQ 0 disabled to avoid race conditions.
pub unsafe fn set_frequency(frequency: u32) {
    let divisor = PIT_FREQUENCY / frequency;

    if divisor > 65535 || divisor == 0 {
        return; // Invalid frequency
    }

    // Configure Channel 0, Mode 3, lobyte/hibyte
    Port::<u8>::new(COMMAND).write(0x36);
    Port::<u8>::new(CHANNEL0).write((divisor & 0xFF) as u8);
    Port::<u8>::new(CHANNEL0).write(((divisor >> 8) & 0xFF) as u8);
}

/// Read the current counter value (Channel 0)
///
/// Returns the current countdown value. This can be used for precise timing.
///
/// # Safety
///
/// Reading the counter requires latching to avoid reading inconsistent values
/// (race condition between low and high byte reads).
pub unsafe fn read_counter() -> u16 {
    // Latch counter value (prevents it from changing during read)
    // Command: 0x00 = latch counter for Channel 0
    Port::<u8>::new(COMMAND).write(0x00);

    // Read latched value (low byte, then high byte)
    let low = Port::<u8>::new(CHANNEL0).read();
    let high = Port::<u8>::new(CHANNEL0).read();

    ((high as u16) << 8) | (low as u16)
}

/// Increment tick counter (called by interrupt handler)
///
/// This function should be called by the timer interrupt handler (IRQ 0).
pub fn tick() {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Get the current tick count
///
/// Returns the number of timer ticks since boot.
pub fn ticks() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}

/// Get the elapsed time in milliseconds (assuming 1000 Hz timer)
///
/// This is only accurate if the PIT is configured for 1000 Hz.
pub fn uptime_ms() -> u64 {
    // Assume 1 ms per tick (1000 Hz)
    ticks()
}

/// Busy-wait delay using the PIT
///
/// Delays for approximately the specified number of microseconds using
/// the PIT counter. This is a busy-wait (CPU spinning) delay.
///
/// # Arguments
///
/// * `us` - Delay in microseconds
///
/// # Safety
///
/// This function busy-waits and will block the CPU. Should only be used
/// during early boot or when interrupts are disabled.
pub unsafe fn delay_us(us: u32) {
    // Calculate number of PIT ticks for the delay
    // PIT frequency is ~1.193 MHz, so 1 tick ≈ 0.838 µs
    let ticks = (us as u64 * PIT_FREQUENCY as u64) / 1_000_000;

    if ticks == 0 {
        return;
    }

    // If delay is very long, break it into smaller chunks
    if ticks > 65535 {
        let iterations = ticks / 65535;
        let remainder = ticks % 65535;

        for _ in 0..iterations {
            delay_ticks(65535);
        }
        if remainder > 0 {
            delay_ticks(remainder as u16);
        }
    } else {
        delay_ticks(ticks as u16);
    }
}

/// Busy-wait for a specific number of PIT ticks
///
/// # Safety
///
/// Busy-waits (CPU spinning). Should only be used during early boot.
unsafe fn delay_ticks(ticks: u16) {
    // Configure PIT for one-shot mode (Mode 0)
    // This will count down to 0 and then stop
    let command: u8 = 0x30; // Channel 0, lobyte/hibyte, Mode 0, binary
    Port::<u8>::new(COMMAND).write(command);

    // Write initial count
    Port::<u8>::new(CHANNEL0).write((ticks & 0xFF) as u8);
    Port::<u8>::new(CHANNEL0).write(((ticks >> 8) & 0xFF) as u8);

    // Wait for counter to reach 0
    // In Mode 0, bit 7 of the status becomes 1 when count reaches 0
    loop {
        // Read-back command: read status of Channel 0
        Port::<u8>::new(COMMAND).write(0xE2); // 1110_0010
        let status = Port::<u8>::new(CHANNEL0).read();

        // Bit 7 = OUT pin state (1 when count expires in Mode 0)
        if (status & 0x80) != 0 {
            break;
        }

        core::hint::spin_loop();
    }

    // Restore normal mode (Mode 3, periodic)
    Port::<u8>::new(COMMAND).write(0x36);
    let divisor = PIT_FREQUENCY / DEFAULT_FREQUENCY;
    Port::<u8>::new(CHANNEL0).write((divisor & 0xFF) as u8);
    Port::<u8>::new(CHANNEL0).write(((divisor >> 8) & 0xFF) as u8);
}

/// Calibrate TSC using the PIT
///
/// Uses the PIT to measure TSC frequency. Returns TSC frequency in Hz.
///
/// # Arguments
///
/// * `duration_ms` - Calibration duration in milliseconds (longer = more accurate)
///
/// # Safety
///
/// Should be called with interrupts disabled. Busy-waits for the specified duration.
pub unsafe fn calibrate_tsc(duration_ms: u32) -> u64 {
    use crate::arch::x86_64::tsc::read_tsc;

    // Calculate number of PIT ticks for calibration
    // At 1.193182 MHz, 1 ms ≈ 1193 ticks
    let pit_ticks_target = (duration_ms as u64 * PIT_FREQUENCY as u64) / 1000;

    // Read initial TSC
    let tsc_start = read_tsc();

    // Wait for specified duration using PIT
    let mut elapsed_ticks = 0u64;
    while elapsed_ticks < pit_ticks_target {
        let chunk = core::cmp::min(pit_ticks_target - elapsed_ticks, 65535);
        delay_ticks(chunk as u16);
        elapsed_ticks += chunk;
    }

    // Read final TSC
    let tsc_end = read_tsc();

    // Calculate TSC frequency
    // tsc_freq = (tsc_delta / time_seconds)
    // tsc_freq = (tsc_delta * pit_freq) / pit_ticks
    let tsc_delta = tsc_end - tsc_start;
    let tsc_freq = (tsc_delta * PIT_FREQUENCY as u64) / pit_ticks_target;

    crate::arch::x86_64::serial::serial_write(b"[PIT] TSC calibrated using PIT\n");
    crate::arch::x86_64::serial::serial_write(b"[PIT] Calibration duration: ");
    print_u32(duration_ms);
    crate::arch::x86_64::serial::serial_write(b" ms\n");
    crate::arch::x86_64::serial::serial_write(b"[PIT] TSC frequency: ");
    print_u64(tsc_freq);
    crate::arch::x86_64::serial::serial_write(b" Hz (");
    print_u64(tsc_freq / 1_000_000);
    crate::arch::x86_64::serial::serial_write(b" MHz)\n");

    tsc_freq
}

/// Helper function to print u32 to serial
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

/// Helper function to print u64 to serial
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divisor_calculation() {
        // 1000 Hz should give divisor of 1193
        let divisor = PIT_FREQUENCY / 1000;
        assert_eq!(divisor, 1193);

        // 100 Hz should give divisor of 11931
        let divisor = PIT_FREQUENCY / 100;
        assert_eq!(divisor, 11931);

        // 18.2 Hz (original IBM PC frequency) should give ~65543 (clamped to 65535)
        let divisor = PIT_FREQUENCY / 18;
        assert!(divisor <= 65535);
    }

    #[test]
    fn test_frequency_accuracy() {
        // Check that we can achieve reasonable frequencies
        let target = 1000u32;
        let divisor = PIT_FREQUENCY / target;
        let actual = PIT_FREQUENCY / divisor;

        // Should be within 1 Hz
        assert!((actual as i32 - target as i32).abs() <= 1);
    }
}
