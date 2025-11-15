//! Firmware interface drivers
//!
//! This module provides drivers for communicating with platform firmware.
//! Currently supports Raspberry Pi VideoCore mailbox interface.
//!
//! M8 Hardening: All functions return DriverResult for consistent error handling

pub mod mailbox;

// Re-export common functions (M8: now return DriverResult)
pub use mailbox::{get_temperature, get_board_serial, get_firmware_revision};
