//! # 16550 UART Serial Driver
//!
//! This module provides a driver for the 16550 UART (Universal Asynchronous Receiver/Transmitter),
//! a standard serial communication controller found on x86 systems.
//!
//! ## Hardware Overview
//!
//! The 16550 UART is accessed via I/O ports. On x86, serial ports are typically located at:
//! - **COM1**: 0x3F8-0x3FF (IRQ 4)
//! - **COM2**: 0x2F8-0x2FF (IRQ 3)
//! - **COM3**: 0x3E8-0x3EF (IRQ 4, shared with COM1)
//! - **COM4**: 0x2E8-0x2EF (IRQ 3, shared with COM2)
//!
//! ## Register Map (offset from base port)
//!
//! ```text
//! Offset  DLAB=0  DLAB=1   Read/Write  Description
//! ------  ------  -------  ----------  -----------
//! 0       RBR     DLL      R / W       Receiver Buffer / Divisor Latch Low
//! 1       IER     DLH      R / W       Interrupt Enable / Divisor Latch High
//! 2       IIR     -        R           Interrupt Identification
//! 2       -       FCR      W           FIFO Control
//! 3       LCR     LCR      R / W       Line Control
//! 4       MCR     MCR      R / W       Modem Control
//! 5       LSR     LSR      R           Line Status
//! 6       MSR     MSR      R           Modem Status
//! 7       SCR     SCR      R / W       Scratch Register
//! ```
//!
//! DLAB (Divisor Latch Access Bit) is bit 7 of LCR.
//!
//! ## Initialization Sequence
//!
//! 1. Disable all interrupts (IER = 0x00)
//! 2. Enable DLAB (LCR bit 7 = 1)
//! 3. Set baud rate divisor (DLL, DLH)
//! 4. Configure line parameters (LCR: 8N1 = 8 data bits, no parity, 1 stop bit)
//! 5. Enable and clear FIFOs (FCR)
//! 6. Enable auxiliary output 2 (MCR bit 3 = 1) for interrupts to work
//! 7. Enable desired interrupts (IER)
//!
//! ## Baud Rate
//!
//! The baud rate is set using a divisor:
//! ```text
//! divisor = 115200 / desired_baud_rate
//! ```
//!
//! Common divisors:
//! - 115200 baud: divisor = 1
//! - 57600 baud: divisor = 2
//! - 38400 baud: divisor = 3
//! - 19200 baud: divisor = 6
//! - 9600 baud: divisor = 12
//!
//! ## Line Status Register (LSR) Bits
//!
//! - **Bit 0 (DR)**: Data Ready - set when data is available to read
//! - **Bit 1 (OE)**: Overrun Error
//! - **Bit 2 (PE)**: Parity Error
//! - **Bit 3 (FE)**: Framing Error
//! - **Bit 4 (BI)**: Break Indicator
//! - **Bit 5 (THRE)**: Transmitter Holding Register Empty - can send data
//! - **Bit 6 (TEMT)**: Transmitter Empty - all data sent
//! - **Bit 7 (FIFO)**: Error in FIFO

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

/// COM1 base I/O port (0x3F8)
pub const COM1_PORT: u16 = 0x3F8;

/// COM2 base I/O port (0x2F8)
pub const COM2_PORT: u16 = 0x2F8;

lazy_static! {
    /// Global COM1 serial port instance
    ///
    /// This is the primary serial port used for kernel logging and debugging.
    /// Uses lazy_static to ensure it's initialized exactly once.
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(COM1_PORT) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Initialize the serial port driver
///
/// This function initializes COM1 (0x3F8) as the primary serial console.
/// Configuration: 115200 baud, 8 data bits, no parity, 1 stop bit (8N1)
///
/// # Safety
///
/// Must be called during early boot, before any serial output is attempted.
pub unsafe fn init_serial() -> Result<(), &'static str> {
    // Force initialization of lazy_static
    let _ = &*SERIAL1;

    Ok(())
}

/// Write a single byte to the serial port
///
/// This function blocks until the UART is ready to accept the byte.
pub fn serial_write_byte(byte: u8) {
    SERIAL1.lock().send(byte);
}

/// Write a string to the serial port
///
/// This is the primary function used for kernel logging and debugging output.
pub fn serial_write(s: &[u8]) {
    for &byte in s {
        serial_write_byte(byte);
    }
}

/// Read a single byte from the serial port
///
/// Returns None if no data is available (non-blocking).
pub fn serial_read() -> Option<u8> {
    SERIAL1.lock().receive()
}

/// Write a formatted string to the serial port
///
/// This function is used by the kernel's print macros.
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Failed to write to serial");
}

/// Print to serial port
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::arch::x86_64::serial::_print(format_args!($($arg)*))
    };
}

/// Print to serial port with newline
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_write() {
        // Basic smoke test - just ensure it doesn't crash
        serial_write(b"Test message\n");
    }

    #[test]
    fn test_serial_macros() {
        serial_print!("Test ");
        serial_println!("message");
        serial_println!("Formatted: {} {}", 42, "test");
    }
}
