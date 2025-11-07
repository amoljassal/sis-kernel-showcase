// Kernel logging (printk) with ring buffer
// Phase A0 - Basic kernel logging facility

use super::ringbuf::RingBuffer;
use core::fmt::{self, Write};
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
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

    // Get timestamp (stub for now - will use timer later)
    let timestamp_us = 0; // TODO: Get from timer

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
