//! PWM (Pulse Width Modulation) Driver for Raspberry Pi 5
//!
//! This module provides PWM control for robotics applications including
//! servo control, motor speed control, LED dimming, and buzzer control.
//!
//! # Overview
//!
//! The RPi5 provides 2 PWM controllers through the RP1 I/O Hub, each with
//! 2 channels, for a total of 4 independent PWM outputs.
//!
//! # Common Use Cases
//!
//! ## Servo Control (50Hz, 1-2ms pulse)
//! ```rust
//! use crate::drivers::pwm;
//!
//! // Initialize PWM for servo control
//! pwm::initialize()?;
//!
//! // Configure for servo (50Hz)
//! pwm::set_frequency(0, 0, 50)?;  // PWM 0, Channel 0, 50Hz
//! pwm::enable(0, 0)?;
//!
//! // Set servo to center position (1.5ms pulse)
//! pwm::set_pulse_width_us(0, 0, 1500)?;
//!
//! // Move to max position (2.0ms pulse)
//! pwm::set_pulse_width_us(0, 0, 2000)?;
//! ```
//!
//! ## Motor Control (1-20kHz, variable duty cycle)
//! ```rust
//! // Configure for motor (10kHz)
//! pwm::set_frequency(0, 0, 10000)?;
//! pwm::enable(0, 0)?;
//!
//! // Set motor speed to 75%
//! pwm::set_duty_cycle(0, 0, 75.0)?;
//! ```
//!
//! ## LED Dimming (100Hz-1kHz)
//! ```rust
//! // Configure for LED (1kHz)
//! pwm::set_frequency(0, 0, 1000)?;
//! pwm::enable(0, 0)?;
//!
//! // Set brightness to 50%
//! pwm::set_duty_cycle(0, 0, 50.0)?;
//! ```

pub mod bcm2712;

use crate::drivers::{DriverError, DriverResult};
use bcm2712::{Bcm2712Pwm, PwmChannel};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Once;

/// Maximum number of PWM controllers (RP1 provides 2)
pub const MAX_PWM_CONTROLLERS: usize = 2;

/// Maximum number of channels per controller
pub const CHANNELS_PER_CONTROLLER: usize = 2;

/// PWM subsystem state
struct PwmState {
    /// PWM controllers (indexed by controller number)
    controllers: [Bcm2712Pwm; MAX_PWM_CONTROLLERS],

    /// Initialization complete
    initialized: AtomicBool,
}

/// Global PWM state
static PWM_STATE: Once<PwmState> = Once::new();

/// Initialize PWM subsystem
///
/// This function must be called after RP1 initialization to set up
/// all PWM controllers.
///
/// # Returns
/// Ok(()) if initialization succeeds, or an error if:
/// - RP1 not initialized
/// - PWM controller initialization fails
pub fn initialize() -> DriverResult<()> {
    // Get RP1 driver
    let rp1 = crate::drivers::pcie::get_rp1()
        .ok_or(DriverError::NotInitialized)?;

    crate::info!("[PWM] Initializing PWM subsystem");

    // Get PWM controller base addresses from RP1
    let pwm0_base = rp1.pwm_base(0).ok_or(DriverError::DeviceNotFound)?;
    let pwm1_base = rp1.pwm_base(1).ok_or(DriverError::DeviceNotFound)?;

    // Create controller instances
    let pwm0 = Bcm2712Pwm::new(pwm0_base, 0);
    let pwm1 = Bcm2712Pwm::new(pwm1_base, 1);

    // Initialize controllers
    pwm0.initialize()?;
    pwm1.initialize()?;

    // Store global state
    PWM_STATE.call_once(|| PwmState {
        controllers: [pwm0, pwm1],
        initialized: AtomicBool::new(true),
    });

    crate::info!("[PWM] PWM subsystem initialized");
    crate::info!("[PWM]   Controllers: {}", MAX_PWM_CONTROLLERS);
    crate::info!("[PWM]   Channels per controller: {}", CHANNELS_PER_CONTROLLER);
    crate::info!("[PWM]   Total channels: {}", MAX_PWM_CONTROLLERS * CHANNELS_PER_CONTROLLER);

    Ok(())
}

/// Check if PWM subsystem is initialized
pub fn is_initialized() -> bool {
    PWM_STATE
        .get()
        .map(|state| state.initialized.load(Ordering::Acquire))
        .unwrap_or(false)
}

/// Get PWM controller by index
fn get_controller(controller: u8) -> DriverResult<&'static Bcm2712Pwm> {
    let state = PWM_STATE.get().ok_or(DriverError::NotInitialized)?;

    if (controller as usize) >= MAX_PWM_CONTROLLERS {
        return Err(DriverError::InvalidParameter);
    }

    Ok(&state.controllers[controller as usize])
}

/// Convert channel number to PwmChannel enum
fn get_channel(channel: u8) -> DriverResult<PwmChannel> {
    match channel {
        0 => Ok(PwmChannel::Channel1),
        1 => Ok(PwmChannel::Channel2),
        _ => Err(DriverError::InvalidParameter),
    }
}

/// Enable a PWM channel
///
/// # Arguments
/// * `controller` - Controller index (0-1)
/// * `channel` - Channel index (0-1)
///
/// # Example
/// ```rust
/// pwm::enable(0, 0)?;  // Enable PWM0, Channel 0
/// ```
pub fn enable(controller: u8, channel: u8) -> DriverResult<()> {
    let pwm = get_controller(controller)?;
    let ch = get_channel(channel)?;
    pwm.enable(ch)
}

/// Disable a PWM channel
///
/// # Arguments
/// * `controller` - Controller index (0-1)
/// * `channel` - Channel index (0-1)
pub fn disable(controller: u8, channel: u8) -> DriverResult<()> {
    let pwm = get_controller(controller)?;
    let ch = get_channel(channel)?;
    pwm.disable(ch)
}

/// Set PWM frequency
///
/// # Arguments
/// * `controller` - Controller index (0-1)
/// * `channel` - Channel index (0-1)
/// * `frequency_hz` - Desired frequency in Hz (1 Hz to 25 MHz)
///
/// # Returns
/// Actual frequency achieved (may differ slightly from requested)
///
/// # Example
/// ```rust
/// // Configure for servo (50Hz)
/// let actual_freq = pwm::set_frequency(0, 0, 50)?;
/// assert_eq!(actual_freq, 50);
/// ```
pub fn set_frequency(controller: u8, channel: u8, frequency_hz: u32) -> DriverResult<u32> {
    let pwm = get_controller(controller)?;
    let ch = get_channel(channel)?;
    pwm.set_frequency(ch, frequency_hz)
}

/// Set PWM duty cycle as percentage
///
/// # Arguments
/// * `controller` - Controller index (0-1)
/// * `channel` - Channel index (0-1)
/// * `duty_percent` - Duty cycle (0.0 to 100.0)
///
/// # Example
/// ```rust
/// // Set motor to 75% speed
/// pwm::set_duty_cycle(0, 0, 75.0)?;
/// ```
pub fn set_duty_cycle(controller: u8, channel: u8, duty_percent: f32) -> DriverResult<()> {
    let pwm = get_controller(controller)?;
    let ch = get_channel(channel)?;
    pwm.set_duty_cycle(ch, duty_percent)
}

/// Set PWM pulse width in microseconds
///
/// This is particularly useful for servo control where exact pulse widths
/// are required (typically 1000-2000 microseconds).
///
/// # Arguments
/// * `controller` - Controller index (0-1)
/// * `channel` - Channel index (0-1)
/// * `pulse_us` - Pulse width in microseconds
///
/// # Example
/// ```rust
/// // Configure servo
/// pwm::set_frequency(0, 0, 50)?;  // 50Hz = 20ms period
/// pwm::enable(0, 0)?;
///
/// // Set servo positions
/// pwm::set_pulse_width_us(0, 0, 1000)?;  // Min position (1ms)
/// pwm::set_pulse_width_us(0, 0, 1500)?;  // Center (1.5ms)
/// pwm::set_pulse_width_us(0, 0, 2000)?;  // Max position (2ms)
/// ```
pub fn set_pulse_width_us(controller: u8, channel: u8, pulse_us: u32) -> DriverResult<()> {
    let pwm = get_controller(controller)?;
    let ch = get_channel(channel)?;
    pwm.set_pulse_width_us(ch, pulse_us)
}

/// Check if a PWM channel is enabled
///
/// # Arguments
/// * `controller` - Controller index (0-1)
/// * `channel` - Channel index (0-1)
pub fn is_enabled(controller: u8, channel: u8) -> bool {
    if let Ok(pwm) = get_controller(controller) {
        if let Ok(ch) = get_channel(channel) {
            return pwm.is_enabled(ch);
        }
    }
    false
}

/// Get current frequency of a PWM channel
///
/// # Arguments
/// * `controller` - Controller index (0-1)
/// * `channel` - Channel index (0-1)
pub fn get_frequency(controller: u8, channel: u8) -> DriverResult<u32> {
    let pwm = get_controller(controller)?;
    let ch = get_channel(channel)?;
    Ok(pwm.get_frequency(ch))
}

/// Get status of a PWM controller
///
/// # Arguments
/// * `controller` - Controller index (0-1)
pub fn get_status(controller: u8) -> DriverResult<u32> {
    let pwm = get_controller(controller)?;
    Ok(pwm.get_status())
}

/// Clear error flags on a PWM controller
///
/// # Arguments
/// * `controller` - Controller index (0-1)
pub fn clear_errors(controller: u8) -> DriverResult<()> {
    let pwm = get_controller(controller)?;
    pwm.clear_errors()
}

/// Servo control helper functions
pub mod servo {
    use super::*;

    /// Standard servo frequency (50Hz = 20ms period)
    pub const SERVO_FREQUENCY_HZ: u32 = 50;

    /// Minimum pulse width for standard servos (1ms)
    pub const SERVO_MIN_PULSE_US: u32 = 1000;

    /// Center pulse width for standard servos (1.5ms)
    pub const SERVO_CENTER_PULSE_US: u32 = 1500;

    /// Maximum pulse width for standard servos (2ms)
    pub const SERVO_MAX_PULSE_US: u32 = 2000;

    /// Initialize a channel for servo control
    ///
    /// Sets the frequency to 50Hz (standard for servos)
    pub fn init(controller: u8, channel: u8) -> DriverResult<()> {
        set_frequency(controller, channel, SERVO_FREQUENCY_HZ)?;
        enable(controller, channel)?;
        Ok(())
    }

    /// Set servo position by angle
    ///
    /// # Arguments
    /// * `controller` - Controller index
    /// * `channel` - Channel index
    /// * `angle` - Angle in degrees (-90 to +90, where 0 is center)
    pub fn set_angle(controller: u8, channel: u8, angle: i32) -> DriverResult<()> {
        if angle < -90 || angle > 90 {
            return Err(DriverError::InvalidParameter);
        }

        // Map angle to pulse width
        // -90° = 1000us, 0° = 1500us, +90° = 2000us
        let pulse_us = SERVO_CENTER_PULSE_US as i32 + (angle * 500 / 90);

        set_pulse_width_us(controller, channel, pulse_us as u32)
    }

    /// Set servo to center position
    pub fn center(controller: u8, channel: u8) -> DriverResult<()> {
        set_pulse_width_us(controller, channel, SERVO_CENTER_PULSE_US)
    }

    /// Set servo to minimum position
    pub fn min(controller: u8, channel: u8) -> DriverResult<()> {
        set_pulse_width_us(controller, channel, SERVO_MIN_PULSE_US)
    }

    /// Set servo to maximum position
    pub fn max(controller: u8, channel: u8) -> DriverResult<()> {
        set_pulse_width_us(controller, channel, SERVO_MAX_PULSE_US)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_conversion() {
        assert!(matches!(get_channel(0), Ok(PwmChannel::Channel1)));
        assert!(matches!(get_channel(1), Ok(PwmChannel::Channel2)));
        assert!(get_channel(2).is_err());
    }

    #[test]
    fn test_servo_angle_mapping() {
        // Test angle to pulse width mapping logic
        let center = servo::SERVO_CENTER_PULSE_US as i32;

        // -90° should give min pulse
        let pulse_min = center + (-90 * 500 / 90);
        assert_eq!(pulse_min, servo::SERVO_MIN_PULSE_US as i32);

        // 0° should give center pulse
        let pulse_center = center + (0 * 500 / 90);
        assert_eq!(pulse_center, servo::SERVO_CENTER_PULSE_US as i32);

        // +90° should give max pulse
        let pulse_max = center + (90 * 500 / 90);
        assert_eq!(pulse_max, servo::SERVO_MAX_PULSE_US as i32);
    }
}
