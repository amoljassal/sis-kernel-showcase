//! GPIO (General Purpose Input/Output) drivers
//!
//! This module provides GPIO control for different hardware platforms.
//! Currently supports BCM2xxx GPIO controllers (Raspberry Pi 5).

pub mod bcm2xxx;

// Re-export common types and functions
pub use bcm2xxx::{
    BcmGpio,
    GpioFunction,
    GpioPull,
    is_initialized,
    set_function,
    set_pin,
    clear_pin,
    toggle_pin,
    read_pin,
    set_pull,
};
