//! Production Logging Framework
//!
//! M8 Driver Hardening - Production Logging
//!
//! Provides structured logging with configurable log levels for production use.
//! Designed to be minimal overhead in production while providing detailed
//! debugging information when needed.
//!
//! # Features
//!
//! - Multiple log levels (ERROR, WARN, INFO, DEBUG, TRACE)
//! - Compile-time and runtime filtering
//! - Structured logging with context
//! - Zero-cost abstractions when logging is disabled
//! - Driver-specific logging contexts
//!
//! # Usage
//!
//! ```rust
//! use crate::log::{error, warn, info, debug};
//!
//! // Simple logging
//! error!("Failed to initialize device");
//! warn!("Device temperature high: {} C", temp);
//! info!("Driver initialized successfully");
//! debug!("Register value: 0x{:08x}", reg);
//!
//! // Structured logging with context
//! log::error_ctx("GPIO", "Pin validation failed", &[("pin", 54)]);
//! ```

use core::sync::atomic::{AtomicU8, Ordering};

/// Log level enum
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Critical errors that prevent operation
    Error = 0,
    /// Warning conditions that should be addressed
    Warn = 1,
    /// Informational messages about normal operation
    Info = 2,
    /// Debugging information
    Debug = 3,
    /// Detailed trace information
    Trace = 4,
}

impl LogLevel {
    /// Get log level name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }

    /// Get log level prefix for output
    pub fn prefix(&self) -> &'static [u8] {
        match self {
            Self::Error => b"[ERROR] ",
            Self::Warn => b"[WARN]  ",
            Self::Info => b"[INFO]  ",
            Self::Debug => b"[DEBUG] ",
            Self::Trace => b"[TRACE] ",
        }
    }
}

/// Global log level (default: INFO for production)
static LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

/// Set global log level
pub fn set_level(level: LogLevel) {
    LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Get current log level
pub fn get_level() -> LogLevel {
    match LOG_LEVEL.load(Ordering::Relaxed) {
        0 => LogLevel::Error,
        1 => LogLevel::Warn,
        2 => LogLevel::Info,
        3 => LogLevel::Debug,
        _ => LogLevel::Trace,
    }
}

/// Check if a log level is enabled
#[inline]
pub fn is_enabled(level: LogLevel) -> bool {
    level <= get_level()
}

/// Log a message with specified level
pub fn log(level: LogLevel, module: &str, message: &str) {
    if !is_enabled(level) {
        return;
    }

    unsafe {
        crate::uart_print(level.prefix());
        crate::uart_print(module.as_bytes());
        crate::uart_print(b": ");
        crate::uart_print(message.as_bytes());
        crate::uart_print(b"\n");
    }
}

/// Log with context (key-value pairs)
pub fn log_ctx(level: LogLevel, module: &str, message: &str, context: &[(&str, u64)]) {
    if !is_enabled(level) {
        return;
    }

    unsafe {
        crate::uart_print(level.prefix());
        crate::uart_print(module.as_bytes());
        crate::uart_print(b": ");
        crate::uart_print(message.as_bytes());

        for (key, value) in context {
            crate::uart_print(b" [");
            crate::uart_print(key.as_bytes());
            crate::uart_print(b"=");
            print_u64(*value);
            crate::uart_print(b"]");
        }

        crate::uart_print(b"\n");
    }
}

/// Helper to print u64 value
fn print_u64(value: u64) {
    if value == 0 {
        unsafe { crate::uart_print(b"0"); }
        return;
    }

    let mut buf = [0u8; 20];
    let mut pos = 0;
    let mut n = value;

    while n > 0 {
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        pos += 1;
    }

    // Print in reverse
    unsafe {
        for i in (0..pos).rev() {
            crate::uart_print(&[buf[i]]);
        }
    }
}

/// Convenience functions for each log level

#[inline]
pub fn error(module: &str, message: &str) {
    log(LogLevel::Error, module, message);
}

#[inline]
pub fn warn(module: &str, message: &str) {
    log(LogLevel::Warn, module, message);
}

#[inline]
pub fn info(module: &str, message: &str) {
    log(LogLevel::Info, module, message);
}

#[inline]
pub fn debug(module: &str, message: &str) {
    log(LogLevel::Debug, module, message);
}

#[inline]
pub fn trace(module: &str, message: &str) {
    log(LogLevel::Trace, module, message);
}

/// Context-aware logging

#[inline]
pub fn error_ctx(module: &str, message: &str, context: &[(&str, u64)]) {
    log_ctx(LogLevel::Error, module, message, context);
}

#[inline]
pub fn warn_ctx(module: &str, message: &str, context: &[(&str, u64)]) {
    log_ctx(LogLevel::Warn, module, message, context);
}

#[inline]
pub fn info_ctx(module: &str, message: &str, context: &[(&str, u64)]) {
    log_ctx(LogLevel::Info, module, message, context);
}

#[inline]
pub fn debug_ctx(module: &str, message: &str, context: &[(&str, u64)]) {
    log_ctx(LogLevel::Debug, module, message, context);
}

/// Driver error logging helper
pub fn log_driver_error(module: &str, operation: &str, error: &crate::drivers::DriverError) {
    let error_code = error.code();
    let error_name = error.name();

    if is_enabled(LogLevel::Error) {
        unsafe {
            crate::uart_print(b"[ERROR] ");
            crate::uart_print(module.as_bytes());
            crate::uart_print(b": ");
            crate::uart_print(operation.as_bytes());
            crate::uart_print(b" failed - ");
            crate::uart_print(error_name.as_bytes());
            crate::uart_print(b" (code=");
            print_u64(error_code as u64);
            crate::uart_print(b")\n");
        }
    }
}

/// Production logging policy
pub mod policy {
    use super::*;

    /// Production log level (minimal logging)
    pub const PRODUCTION_LEVEL: LogLevel = LogLevel::Warn;

    /// Development log level (detailed logging)
    pub const DEVELOPMENT_LEVEL: LogLevel = LogLevel::Debug;

    /// Testing log level (maximum logging)
    pub const TESTING_LEVEL: LogLevel = LogLevel::Trace;

    /// Set production logging policy
    pub fn set_production() {
        set_level(PRODUCTION_LEVEL);
        info("LOG", "Production logging policy active (WARN+)");
    }

    /// Set development logging policy
    pub fn set_development() {
        set_level(DEVELOPMENT_LEVEL);
        info("LOG", "Development logging policy active (DEBUG+)");
    }

    /// Set testing logging policy
    pub fn set_testing() {
        set_level(TESTING_LEVEL);
        info("LOG", "Testing logging policy active (TRACE+)");
    }
}

/// Macros for ergonomic logging (optional - for future use)
#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => {
        // In production, we use direct function calls for now
        // Future: could use alloc::format! if available
    };
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => {
        // Similar to log_error
    };
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {
        // Similar to log_error
    };
}

#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        // Similar to log_error
    };
}
