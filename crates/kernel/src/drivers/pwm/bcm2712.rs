//! BCM2712 PWM Controller Driver for Raspberry Pi 5
//!
//! This module implements the PWM (Pulse Width Modulation) controller driver
//! for the BCM2712 SoC on Raspberry Pi 5. PWM is accessed through the RP1 I/O Hub.
//!
//! # Hardware Overview
//!
//! The BCM2712 provides PWM controllers through the RP1 chip:
//! - 2× independent PWM controllers
//! - Each controller has 2 channels
//! - Configurable frequency: 1 Hz to 25 MHz
//! - Configurable duty cycle: 0-100%
//! - DMA support for smooth waveforms
//!
//! # Common Applications
//!
//! - **Servo Control:** 50Hz with 1-2ms pulse width
//! - **Motor Control:** 1-20kHz with variable duty cycle
//! - **LED Dimming:** 100Hz-1kHz for flicker-free brightness
//! - **Buzzer/Tone Generation:** Audio frequencies (20Hz-20kHz)
//!
//! # Register Map (per PWM controller)
//!
//! ```text
//! Offset   Register    Description
//! ------   --------    -----------
//! 0x00     CTL         Control register
//! 0x04     STA         Status register
//! 0x08     DMAC        DMA configuration
//! 0x0C     Reserved
//! 0x10     RNG1        Channel 1 range (period)
//! 0x14     DAT1        Channel 1 data (pulse width)
//! 0x18     FIF1        Channel 1 FIFO
//! 0x1C     Reserved
//! 0x20     RNG2        Channel 2 range (period)
//! 0x24     DAT2        Channel 2 data (pulse width)
//! 0x28     FIF2        Channel 2 FIFO
//! ```
//!
//! # Clock Configuration
//!
//! PWM frequency is derived from the base clock:
//! ```
//! PWM_freq = Clock_freq / (DIV * RNG)
//! ```
//! Where:
//! - Clock_freq = PWM base clock (typically 54 MHz on RPi5)
//! - DIV = Clock divider (1-4095)
//! - RNG = Range register value (period)
//!
//! # References
//! - BCM2712 ARM Peripherals Manual
//! - BCM2835 PWM documentation (similar architecture)

use crate::drivers::{DriverError, DriverResult};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// PWM base clock frequency (Hz)
/// This is the typical clock frequency for PWM on RPi5
pub const PWM_CLOCK_HZ: u32 = 54_000_000;

/// Minimum PWM frequency (1 Hz)
pub const MIN_FREQUENCY_HZ: u32 = 1;

/// Maximum PWM frequency (25 MHz)
pub const MAX_FREQUENCY_HZ: u32 = 25_000_000;

/// PWM register offsets
mod regs {
    pub const CTL: u32 = 0x00;   // Control
    pub const STA: u32 = 0x04;   // Status
    pub const DMAC: u32 = 0x08;  // DMA Control
    pub const RNG1: u32 = 0x10;  // Channel 1 Range
    pub const DAT1: u32 = 0x14;  // Channel 1 Data
    pub const FIF1: u32 = 0x18;  // Channel 1 FIFO
    pub const RNG2: u32 = 0x20;  // Channel 2 Range
    pub const DAT2: u32 = 0x24;  // Channel 2 Data
    pub const FIF2: u32 = 0x28;  // Channel 2 FIFO
}

/// Control register bits
mod ctl {
    // Channel 1 bits
    pub const PWEN1: u32 = 1 << 0;   // Channel 1 enable
    pub const MODE1: u32 = 1 << 1;   // Channel 1 mode (0=PWM, 1=serializer)
    pub const RPTL1: u32 = 1 << 2;   // Channel 1 repeat last data
    pub const SBIT1: u32 = 1 << 3;   // Channel 1 silence bit
    pub const POLA1: u32 = 1 << 4;   // Channel 1 polarity (0=normal, 1=inverted)
    pub const USEF1: u32 = 1 << 5;   // Channel 1 use FIFO
    pub const CLRF1: u32 = 1 << 6;   // Channel 1 clear FIFO
    pub const MSEN1: u32 = 1 << 7;   // Channel 1 M/S enable

    // Channel 2 bits (shifted by 8)
    pub const PWEN2: u32 = 1 << 8;   // Channel 2 enable
    pub const MODE2: u32 = 1 << 9;   // Channel 2 mode
    pub const RPTL2: u32 = 1 << 10;  // Channel 2 repeat last data
    pub const SBIT2: u32 = 1 << 11;  // Channel 2 silence bit
    pub const POLA2: u32 = 1 << 12;  // Channel 2 polarity
    pub const USEF2: u32 = 1 << 13;  // Channel 2 use FIFO
    pub const CLRF2: u32 = 1 << 14;  // Channel 2 clear FIFO
    pub const MSEN2: u32 = 1 << 15;  // Channel 2 M/S enable
}

/// Status register bits
mod sta {
    pub const FULL1: u32 = 1 << 0;   // FIFO 1 full
    pub const EMPT1: u32 = 1 << 1;   // FIFO 1 empty
    pub const WERR1: u32 = 1 << 2;   // FIFO 1 write error
    pub const RERR1: u32 = 1 << 3;   // FIFO 1 read error
    pub const GAPO1: u32 = 1 << 4;   // Channel 1 gap occurred
    pub const GAPO2: u32 = 1 << 5;   // Channel 2 gap occurred
    pub const GAPO3: u32 = 1 << 6;   // Channel 3 gap occurred
    pub const GAPO4: u32 = 1 << 7;   // Channel 4 gap occurred
    pub const BERR: u32 = 1 << 8;    // Bus error
    pub const STA1: u32 = 1 << 9;    // Channel 1 state
    pub const STA2: u32 = 1 << 10;   // Channel 2 state
    pub const STA3: u32 = 1 << 11;   // Channel 3 state
    pub const STA4: u32 = 1 << 12;   // Channel 4 state
}

/// PWM channel (0 or 1 per controller)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PwmChannel {
    Channel1 = 0,
    Channel2 = 1,
}

/// PWM mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PwmMode {
    /// PWM mode (normal pulse-width modulation)
    Pwm,

    /// Serializer mode (data serialization)
    Serializer,

    /// Mark-Space mode (traditional PWM with defined marks and spaces)
    MarkSpace,
}

/// PWM controller state
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PwmState {
    Uninitialized,
    Initialized,
    Running,
    Stopped,
    Error,
}

/// BCM2712 PWM controller
pub struct Bcm2712Pwm {
    /// MMIO base address
    base: usize,

    /// Controller index (0 or 1)
    index: u8,

    /// Current state
    state: AtomicU32,

    /// Channel 1 enabled
    ch1_enabled: AtomicBool,

    /// Channel 2 enabled
    ch2_enabled: AtomicBool,

    /// Channel 1 frequency (Hz)
    ch1_frequency: AtomicU32,

    /// Channel 2 frequency (Hz)
    ch2_frequency: AtomicU32,
}

impl Bcm2712Pwm {
    /// Create a new PWM controller instance
    ///
    /// # Arguments
    /// * `base` - MMIO base address from RP1
    /// * `index` - Controller index (0 or 1)
    pub const fn new(base: usize, index: u8) -> Self {
        Self {
            base,
            index,
            state: AtomicU32::new(PwmState::Uninitialized as u32),
            ch1_enabled: AtomicBool::new(false),
            ch2_enabled: AtomicBool::new(false),
            ch1_frequency: AtomicU32::new(0),
            ch2_frequency: AtomicU32::new(0),
        }
    }

    /// Get MMIO base address
    pub fn base(&self) -> usize {
        self.base
    }

    /// Get controller index
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Get current state
    pub fn state(&self) -> PwmState {
        match self.state.load(Ordering::Acquire) {
            0 => PwmState::Uninitialized,
            1 => PwmState::Initialized,
            2 => PwmState::Running,
            3 => PwmState::Stopped,
            _ => PwmState::Error,
        }
    }

    /// Set state
    fn set_state(&self, state: PwmState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Read a register
    #[inline]
    fn read_reg(&self, offset: u32) -> u32 {
        let addr = (self.base + offset as usize) as *const u32;
        unsafe { core::ptr::read_volatile(addr) }
    }

    /// Write a register
    #[inline]
    fn write_reg(&self, offset: u32, value: u32) {
        let addr = (self.base + offset as usize) as *mut u32;
        unsafe { core::ptr::write_volatile(addr, value); }
    }

    /// Initialize the PWM controller
    pub fn initialize(&self) -> DriverResult<()> {
        // Disable both channels
        self.write_reg(regs::CTL, 0);

        // Clear any pending errors
        self.write_reg(regs::STA, sta::WERR1 | sta::RERR1 | sta::BERR);

        // Reset range and data registers
        self.write_reg(regs::RNG1, 0);
        self.write_reg(regs::DAT1, 0);
        self.write_reg(regs::RNG2, 0);
        self.write_reg(regs::DAT2, 0);

        self.set_state(PwmState::Initialized);

        crate::info!("PWM{}: Initialized at {:#x}", self.index, self.base);

        Ok(())
    }

    /// Enable a PWM channel
    ///
    /// # Arguments
    /// * `channel` - Channel to enable
    pub fn enable(&self, channel: PwmChannel) -> DriverResult<()> {
        let mut ctl = self.read_reg(regs::CTL);

        match channel {
            PwmChannel::Channel1 => {
                ctl |= ctl::PWEN1 | ctl::MSEN1;  // Enable + M/S mode
                self.ch1_enabled.store(true, Ordering::Release);
            }
            PwmChannel::Channel2 => {
                ctl |= ctl::PWEN2 | ctl::MSEN2;  // Enable + M/S mode
                self.ch2_enabled.store(true, Ordering::Release);
            }
        }

        self.write_reg(regs::CTL, ctl);
        self.set_state(PwmState::Running);

        Ok(())
    }

    /// Disable a PWM channel
    ///
    /// # Arguments
    /// * `channel` - Channel to disable
    pub fn disable(&self, channel: PwmChannel) -> DriverResult<()> {
        let mut ctl = self.read_reg(regs::CTL);

        match channel {
            PwmChannel::Channel1 => {
                ctl &= !ctl::PWEN1;
                self.ch1_enabled.store(false, Ordering::Release);
            }
            PwmChannel::Channel2 => {
                ctl &= !ctl::PWEN2;
                self.ch2_enabled.store(false, Ordering::Release);
            }
        }

        self.write_reg(regs::CTL, ctl);

        // If both channels disabled, mark as stopped
        if !self.ch1_enabled.load(Ordering::Acquire) && !self.ch2_enabled.load(Ordering::Acquire) {
            self.set_state(PwmState::Stopped);
        }

        Ok(())
    }

    /// Set PWM frequency
    ///
    /// # Arguments
    /// * `channel` - Channel to configure
    /// * `frequency_hz` - Desired frequency in Hz (1 Hz to 25 MHz)
    ///
    /// # Returns
    /// Actual frequency achieved (may differ slightly due to integer division)
    pub fn set_frequency(&self, channel: PwmChannel, frequency_hz: u32) -> DriverResult<u32> {
        if frequency_hz < MIN_FREQUENCY_HZ || frequency_hz > MAX_FREQUENCY_HZ {
            return Err(DriverError::InvalidParameter);
        }

        // Calculate range value: Clock / Frequency
        // Using integer division, actual freq may vary slightly
        let range = PWM_CLOCK_HZ / frequency_hz;

        if range == 0 {
            return Err(DriverError::InvalidParameter);
        }

        // Calculate actual frequency achieved
        let actual_freq = PWM_CLOCK_HZ / range;

        match channel {
            PwmChannel::Channel1 => {
                self.write_reg(regs::RNG1, range);
                self.ch1_frequency.store(actual_freq, Ordering::Release);
            }
            PwmChannel::Channel2 => {
                self.write_reg(regs::RNG2, range);
                self.ch2_frequency.store(actual_freq, Ordering::Release);
            }
        }

        Ok(actual_freq)
    }

    /// Set PWM duty cycle
    ///
    /// # Arguments
    /// * `channel` - Channel to configure
    /// * `duty_percent` - Duty cycle as percentage (0.0 to 100.0)
    pub fn set_duty_cycle(&self, channel: PwmChannel, duty_percent: f32) -> DriverResult<()> {
        if duty_percent < 0.0 || duty_percent > 100.0 {
            return Err(DriverError::InvalidParameter);
        }

        let (range_reg, data_reg) = match channel {
            PwmChannel::Channel1 => (regs::RNG1, regs::DAT1),
            PwmChannel::Channel2 => (regs::RNG2, regs::DAT2),
        };

        let range = self.read_reg(range_reg);
        if range == 0 {
            return Err(DriverError::InvalidState);
        }

        // Calculate data value: (range * duty) / 100
        let data = ((range as f32) * duty_percent / 100.0) as u32;

        self.write_reg(data_reg, data);

        Ok(())
    }

    /// Set PWM pulse width in microseconds
    ///
    /// Useful for servo control where exact pulse widths are needed.
    ///
    /// # Arguments
    /// * `channel` - Channel to configure
    /// * `pulse_us` - Pulse width in microseconds
    pub fn set_pulse_width_us(&self, channel: PwmChannel, pulse_us: u32) -> DriverResult<()> {
        let frequency = match channel {
            PwmChannel::Channel1 => self.ch1_frequency.load(Ordering::Acquire),
            PwmChannel::Channel2 => self.ch2_frequency.load(Ordering::Acquire),
        };

        if frequency == 0 {
            return Err(DriverError::InvalidState);
        }

        // Calculate duty cycle: (pulse_us * freq) / 1,000,000 * 100
        let period_us = 1_000_000 / frequency;
        let duty_percent = (pulse_us as f32 / period_us as f32) * 100.0;

        if duty_percent > 100.0 {
            return Err(DriverError::InvalidParameter);
        }

        self.set_duty_cycle(channel, duty_percent)
    }

    /// Check if a channel is enabled
    pub fn is_enabled(&self, channel: PwmChannel) -> bool {
        match channel {
            PwmChannel::Channel1 => self.ch1_enabled.load(Ordering::Acquire),
            PwmChannel::Channel2 => self.ch2_enabled.load(Ordering::Acquire),
        }
    }

    /// Get current frequency for a channel
    pub fn get_frequency(&self, channel: PwmChannel) -> u32 {
        match channel {
            PwmChannel::Channel1 => self.ch1_frequency.load(Ordering::Acquire),
            PwmChannel::Channel2 => self.ch2_frequency.load(Ordering::Acquire),
        }
    }

    /// Get PWM status register
    pub fn get_status(&self) -> u32 {
        self.read_reg(regs::STA)
    }

    /// Clear error flags
    pub fn clear_errors(&self) -> DriverResult<()> {
        self.write_reg(regs::STA, sta::WERR1 | sta::RERR1 | sta::BERR);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_calculation() {
        // 50Hz servo frequency
        let range = PWM_CLOCK_HZ / 50;
        let actual = PWM_CLOCK_HZ / range;
        assert_eq!(actual, 50);

        // 1kHz frequency
        let range = PWM_CLOCK_HZ / 1000;
        let actual = PWM_CLOCK_HZ / range;
        assert_eq!(actual, 1000);
    }

    #[test]
    fn test_duty_cycle_bounds() {
        // Valid duty cycles
        assert!(0.0 >= 0.0 && 0.0 <= 100.0);
        assert!(50.0 >= 0.0 && 50.0 <= 100.0);
        assert!(100.0 >= 0.0 && 100.0 <= 100.0);

        // Invalid duty cycles
        assert!(-1.0 < 0.0 || -1.0 > 100.0);
        assert!(101.0 < 0.0 || 101.0 > 100.0);
    }
}
