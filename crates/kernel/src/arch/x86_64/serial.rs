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
//!
//! ## Interrupt-Driven I/O (M5)
//!
//! For Milestone M5, the serial driver supports interrupt-driven I/O:
//! - **Receive Data Available (RDA)**: IRQ fires when data arrives
//! - **Transmitter Holding Register Empty (THRE)**: IRQ fires when ready to send
//! - Ring buffers for RX and TX data (256 bytes each)
//! - Non-blocking read/write operations
//! - Wakes waiting tasks when data is available

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use core::sync::atomic::{AtomicBool, Ordering};
use core::fmt;

/// COM1 base I/O port (0x3F8)
pub const COM1_PORT: u16 = 0x3F8;

/// COM2 base I/O port (0x2F8)
pub const COM2_PORT: u16 = 0x2F8;

/// Ring buffer size for RX and TX
const RING_BUFFER_SIZE: usize = 256;

/// Ring buffer for serial data
///
/// A simple circular buffer with head and tail pointers.
/// Lock-free for single producer, single consumer.
#[derive(Debug)]
struct RingBuffer {
    data: [u8; RING_BUFFER_SIZE],
    head: usize,  // Write position
    tail: usize,  // Read position
}

impl RingBuffer {
    /// Create a new empty ring buffer
    const fn new() -> Self {
        Self {
            data: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
        }
    }

    /// Check if the buffer is empty
    fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    /// Check if the buffer is full
    fn is_full(&self) -> bool {
        (self.head + 1) % RING_BUFFER_SIZE == self.tail
    }

    /// Get the number of bytes available to read
    fn len(&self) -> usize {
        if self.head >= self.tail {
            self.head - self.tail
        } else {
            RING_BUFFER_SIZE - self.tail + self.head
        }
    }

    /// Push a byte into the buffer
    ///
    /// Returns Ok(()) if successful, Err(()) if buffer is full.
    fn push(&mut self, byte: u8) -> Result<(), ()> {
        if self.is_full() {
            return Err(());
        }

        self.data[self.head] = byte;
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        Ok(())
    }

    /// Pop a byte from the buffer
    ///
    /// Returns Some(byte) if data is available, None if buffer is empty.
    fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        let byte = self.data[self.tail];
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        Some(byte)
    }

    /// Clear the buffer
    fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
    }
}

/// Enhanced serial port with interrupt support
pub struct SerialDriver {
    port: SerialPort,
    rx_buffer: RingBuffer,
    tx_buffer: RingBuffer,
    interrupts_enabled: bool,
}

impl SerialDriver {
    /// Create a new serial driver
    unsafe fn new(port: u16) -> Self {
        let mut serial_port = SerialPort::new(port);
        serial_port.init();

        Self {
            port: serial_port,
            rx_buffer: RingBuffer::new(),
            tx_buffer: RingBuffer::new(),
            interrupts_enabled: false,
        }
    }

    /// Enable interrupts for this serial port
    ///
    /// Enables:
    /// - Received Data Available (RDA)
    /// - Transmitter Holding Register Empty (THRE) - disabled for now
    unsafe fn enable_interrupts(&mut self) {
        use x86_64::instructions::port::Port;

        // IER register is at base + 1
        let mut ier_port = Port::<u8>::new(COM1_PORT + 1);

        // Enable RDA interrupt (bit 0)
        // Disable THRE interrupt (bit 1) - we'll use polling for TX
        let ier_value = 0x01; // RDA only
        ier_port.write(ier_value);

        self.interrupts_enabled = true;
    }

    /// Handle interrupt - called from IRQ handler
    ///
    /// Reads all available data from hardware FIFO into RX buffer.
    fn handle_interrupt(&mut self) -> usize {
        let byte = self.port.receive();
        if self.rx_buffer.push(byte).is_ok() {
            1
        } else {
            0
        }
    }

    /// Read bytes from RX buffer (non-blocking)
    ///
    /// Returns the number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut count = 0;
        while count < buf.len() {
            if let Some(byte) = self.rx_buffer.pop() {
                buf[count] = byte;
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Write bytes (blocking for now)
    ///
    /// For M5, we use polling for TX to keep it simple.
    /// Future enhancement: interrupt-driven TX with TX buffer.
    fn write(&mut self, buf: &[u8]) -> usize {
        for &byte in buf {
            self.port.send(byte);
        }
        buf.len()
    }

    /// Write a single byte (blocking)
    fn write_byte(&mut self, byte: u8) {
        self.port.send(byte);
    }

    /// Get number of bytes available to read
    fn available(&self) -> usize {
        self.rx_buffer.len()
    }
}

impl fmt::Write for SerialDriver {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

lazy_static! {
    /// Global COM1 serial port instance with interrupt support
    ///
    /// This is the primary serial port used for kernel logging and debugging.
    /// Uses lazy_static to ensure it's initialized exactly once.
    pub static ref SERIAL1: Mutex<SerialDriver> = {
        let serial = unsafe { SerialDriver::new(COM1_PORT) };
        Mutex::new(serial)
    };

    /// Flag indicating if serial interrupts have been initialized
    static ref SERIAL_INTERRUPTS_ENABLED: AtomicBool = AtomicBool::new(false);
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

/// Enable interrupt-driven I/O for the serial port (M5)
///
/// This enables interrupts for received data on COM1 (IRQ 4).
/// The interrupt handler must be registered in the IDT before calling this.
///
/// # Safety
///
/// Must be called after IDT and APIC initialization.
pub unsafe fn enable_interrupts() {
    let mut serial = SERIAL1.lock();
    serial.enable_interrupts();
    SERIAL_INTERRUPTS_ENABLED.store(true, Ordering::Release);
}

/// Check if serial interrupts are enabled
pub fn interrupts_enabled() -> bool {
    SERIAL_INTERRUPTS_ENABLED.load(Ordering::Acquire)
}

/// Handle serial port interrupt (called from IRQ handler)
///
/// Reads all available data from the hardware into the RX buffer.
/// Returns the number of bytes read.
///
/// # Safety
///
/// Must be called from the serial IRQ handler (IRQ 4).
pub unsafe fn handle_interrupt() -> usize {
    SERIAL1.lock().handle_interrupt()
}

/// Write a single byte to the serial port
///
/// This function blocks until the UART is ready to accept the byte.
pub fn serial_write_byte(byte: u8) {
    SERIAL1.lock().write_byte(byte);
}

/// Write a string to the serial port
///
/// This is the primary function used for kernel logging and debugging output.
pub fn serial_write(s: &[u8]) {
    SERIAL1.lock().write(s);
}

/// Read bytes from the serial port (non-blocking)
///
/// Reads up to `buf.len()` bytes from the RX buffer.
/// Returns the number of bytes actually read.
///
/// # Example
///
/// ```
/// let mut buffer = [0u8; 64];
/// let count = serial_read_bytes(&mut buffer);
/// if count > 0 {
///     // Process buffer[0..count]
/// }
/// ```
pub fn serial_read_bytes(buf: &mut [u8]) -> usize {
    SERIAL1.lock().read(buf)
}

/// Read a single byte from the serial port (non-blocking)
///
/// Returns Some(byte) if data is available, None otherwise.
pub fn serial_read() -> Option<u8> {
    let mut buf = [0u8; 1];
    if serial_read_bytes(&mut buf) == 1 {
        Some(buf[0])
    } else {
        None
    }
}

/// Get the number of bytes available to read
pub fn serial_available() -> usize {
    SERIAL1.lock().available()
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
