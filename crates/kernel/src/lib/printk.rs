// Kernel logging (printk) with ring buffer
// Phase A0 - Basic kernel logging facility

use super::ringbuf::RingBuffer;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicU8, Ordering};
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

/// Log output format (human-readable or JSON for automation)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LogFormat {
    Human = 0,  // Current: "GPU: READY"
    Json = 1,   // New: {"ts":67106346,"subsystem":"GPU","status":"READY","level":"INFO"}
}

static LOG_FORMAT: AtomicU8 = AtomicU8::new(LogFormat::Human as u8);

/// Set the global log format
pub fn set_log_format(format: LogFormat) {
    LOG_FORMAT.store(format as u8, Ordering::Relaxed);
}

/// Get the current log format
pub fn get_log_format() -> LogFormat {
    match LOG_FORMAT.load(Ordering::Relaxed) {
        0 => LogFormat::Human,
        1 => LogFormat::Json,
        _ => LogFormat::Human, // Default to human if invalid
    }
}

#[derive(Copy, Clone)]
pub struct LogEntry {
    pub timestamp_us: u64,
    pub level: LogLevel,
    pub message: [u8; 256],
    pub len: usize,
}

pub struct LogBuffer {
    buffer: Mutex<RingBuffer<LogEntry, 4096>>,
}

impl LogBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: Mutex::new(RingBuffer::new()),
        }
    }

    pub fn push(&self, entry: LogEntry) {
        let mut buffer = self.buffer.lock();
        buffer.push(entry);
    }

    pub fn drain_all(&self) -> alloc::vec::Vec<LogEntry> {
        let mut buffer = self.buffer.lock();
        buffer.drain_all()
    }
}

static KERNEL_LOG: LogBuffer = LogBuffer::new();

struct LogWriter;

impl Write for LogWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Output to UART
        unsafe {
            crate::uart::write_bytes(s.as_bytes());
        }
        Ok(())
    }
}

pub fn log(level: LogLevel, args: fmt::Arguments) {
    let mut message = [0u8; 256];
    let mut writer = LogWriter;

    // Format the message
    let msg_str = alloc::format!("{}", args);
    let len = core::cmp::min(msg_str.len(), 256);
    message[..len].copy_from_slice(&msg_str.as_bytes()[..len]);

    // Get timestamp from timer
    let timestamp_us = crate::time::get_timestamp_us();

    let entry = LogEntry {
        timestamp_us,
        level,
        message,
        len,
    };

    KERNEL_LOG.push(entry);

    // Also print to console immediately for ERROR and WARN
    if level <= LogLevel::Warn {
        let _ = write!(writer, "[{:?}] {}\n", level, msg_str);
    }
}

/// Structured logging for automated testing and observability
/// When LOG_FORMAT is Json, emits JSON-structured events
pub fn log_structured(subsystem: &str, status: &str, level: LogLevel) {
    let format = get_log_format();
    let mut writer = LogWriter;

    match format {
        LogFormat::Json => {
            let timestamp_us = crate::time::get_timestamp_us();
            // JSON format: {"ts":67106346,"subsystem":"GPU","status":"READY","level":"INFO"}
            let _ = write!(
                writer,
                "{{\"ts\":{},\"subsystem\":\"{}\",\"status\":\"{}\",\"level\":\"{}\"}}\n",
                timestamp_us,
                subsystem,
                status,
                level.as_str()
            );
        }
        LogFormat::Human => {
            // Human-readable format: "GPU: READY"
            let _ = write!(writer, "{}: {}\n", subsystem, status);
        }
    }

    // Also store in log buffer
    let msg_str = alloc::format!("{}: {}", subsystem, status);
    let mut message = [0u8; 256];
    let len = core::cmp::min(msg_str.len(), 256);
    message[..len].copy_from_slice(&msg_str.as_bytes()[..len]);

    let entry = LogEntry {
        timestamp_us: crate::time::get_timestamp_us(),
        level,
        message,
        len,
    };

    KERNEL_LOG.push(entry);
}

/// Structured logging with arbitrary key-value pairs
/// Useful for more complex logging scenarios
pub fn log_structured_kv(subsystem: &str, level: LogLevel, kvs: &[(&str, &str)]) {
    let format = get_log_format();
    let mut writer = LogWriter;

    match format {
        LogFormat::Json => {
            let timestamp_us = crate::time::get_timestamp_us();
            let _ = write!(
                writer,
                "{{\"ts\":{},\"subsystem\":\"{}\",\"level\":\"{}\"",
                timestamp_us,
                subsystem,
                level.as_str()
            );
            for (key, value) in kvs {
                let _ = write!(writer, ",\"{}\":\"{}\"", key, value);
            }
            let _ = write!(writer, "}}\n");
        }
        LogFormat::Human => {
            let _ = write!(writer, "{}: ", subsystem);
            for (i, (key, value)) in kvs.iter().enumerate() {
                if i > 0 {
                    let _ = write!(writer, ", ");
                }
                let _ = write!(writer, "{}={}", key, value);
            }
            let _ = write!(writer, "\n");
        }
    }
}

#[macro_export]
macro_rules! printk {
    ($level:expr, $($arg:tt)*) => {
        $crate::lib::printk::log($level, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::printk!($crate::lib::printk::LogLevel::Error, $($arg)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::printk!($crate::lib::printk::LogLevel::Warn, $($arg)*)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::printk!($crate::lib::printk::LogLevel::Info, $($arg)*)
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::printk!($crate::lib::printk::LogLevel::Debug, $($arg)*)
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        // Trace is same as debug for now
        $crate::printk!($crate::lib::printk::LogLevel::Debug, $($arg)*)
    };
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        // Allow log! macro with explicit level
        $crate::printk!($level, $($arg)*)
    };
}

/// Structured logging macros for automated testing
#[macro_export]
macro_rules! log_event {
    ($subsystem:expr, $status:expr) => {
        $crate::lib::printk::log_structured(
            $subsystem,
            $status,
            $crate::lib::printk::LogLevel::Info
        )
    };
    ($subsystem:expr, $status:expr, $level:expr) => {
        $crate::lib::printk::log_structured($subsystem, $status, $level)
    };
}

#[macro_export]
macro_rules! log_kv {
    ($subsystem:expr, $level:expr, $($key:expr => $value:expr),+) => {
        {
            let kvs: &[(&str, &str)] = &[$(($key, $value)),+];
            $crate::lib::printk::log_structured_kv($subsystem, $level, kvs)
        }
    };
}

// Syscall to read dmesg
pub fn sys_dmesg(buf: *mut u8, count: usize) -> Result<isize, crate::lib::error::Errno> {
    use crate::lib::error::Errno;

    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    let entries = KERNEL_LOG.drain_all();
    let mut written = 0;

    for entry in entries {
        if written + entry.len > count {
            break;
        }

        // SAFETY: We validated buf is not null
        unsafe {
            core::ptr::copy_nonoverlapping(
                entry.message.as_ptr(),
                buf.add(written),
                entry.len
            );
        }
        written += entry.len;
    }

    Ok(written as isize)
}
