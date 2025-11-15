//! Firmware interface drivers
//!
//! This module provides drivers for communicating with platform firmware.
//! Currently supports Raspberry Pi VideoCore mailbox interface.

pub mod mailbox;

// Re-export common types
pub use mailbox::{MailboxError, get_temperature, get_board_serial, get_firmware_revision};
