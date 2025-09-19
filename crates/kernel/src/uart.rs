//! ARM PL011 UART driver for SIS kernel
//!
//! Provides input and output functionality for the PL011 UART controller
//! commonly found in ARM platforms including QEMU's virt machine.

use core::ptr;

/// PL011 UART register offsets
const UART_BASE: usize = 0x0900_0000;
const UART_DR: usize = UART_BASE + 0x000; // Data Register
const UART_RSR_ECR: usize = UART_BASE + 0x004; // Receive Status/Error Clear
const UART_FR: usize = UART_BASE + 0x018; // Flag Register
#[allow(dead_code)]
const UART_ILPR: usize = UART_BASE + 0x020; // IrDA Low-Power Counter
const UART_IBRD: usize = UART_BASE + 0x024; // Integer Baud Rate Divisor
const UART_FBRD: usize = UART_BASE + 0x028; // Fractional Baud Rate Divisor
const UART_LCRH: usize = UART_BASE + 0x02C; // Line Control Register
const UART_CR: usize = UART_BASE + 0x030; // Control Register
#[allow(dead_code)]
const UART_IFLS: usize = UART_BASE + 0x034; // Interrupt FIFO Level Select
#[allow(dead_code)]
const UART_IMSC: usize = UART_BASE + 0x038; // Interrupt Mask Set/Clear
#[allow(dead_code)]
const UART_RIS: usize = UART_BASE + 0x03C; // Raw Interrupt Status
#[allow(dead_code)]
const UART_MIS: usize = UART_BASE + 0x040; // Masked Interrupt Status
#[allow(dead_code)]
const UART_ICR: usize = UART_BASE + 0x044; // Interrupt Clear Register

/// Flag Register bits
#[allow(dead_code)]
const UART_FR_TXFE: u32 = 1 << 7; // Transmit FIFO Empty
#[allow(dead_code)]
const UART_FR_RXFF: u32 = 1 << 6; // Receive FIFO Full
const UART_FR_TXFF: u32 = 1 << 5; // Transmit FIFO Full
const UART_FR_RXFE: u32 = 1 << 4; // Receive FIFO Empty
const UART_FR_BUSY: u32 = 1 << 3; // UART Busy

/// Data Register bits
const UART_DR_OE: u32 = 1 << 11; // Overrun Error
const UART_DR_BE: u32 = 1 << 10; // Break Error
const UART_DR_PE: u32 = 1 << 9; // Parity Error
const UART_DR_FE: u32 = 1 << 8; // Framing Error
const UART_DR_DATA: u32 = 0xFF; // Data bits mask

/// Control Register bits
#[allow(dead_code)]
const UART_CR_CTSEN: u32 = 1 << 15; // CTS hardware flow control enable
#[allow(dead_code)]
const UART_CR_RTSEN: u32 = 1 << 14; // RTS hardware flow control enable
#[allow(dead_code)]
const UART_CR_RTS: u32 = 1 << 11; // Request to Send
const UART_CR_RXE: u32 = 1 << 9; // Receive Enable
const UART_CR_TXE: u32 = 1 << 8; // Transmit Enable
#[allow(dead_code)]
const UART_CR_LBE: u32 = 1 << 7; // Loopback Enable
const UART_CR_UARTEN: u32 = 1 << 0; // UART Enable

/// Line Control Register bits
#[allow(dead_code)]
const UART_LCRH_SPS: u32 = 1 << 7; // Stick Parity Select
const UART_LCRH_WLEN_8: u32 = 3 << 5; // Word Length 8 bits
const UART_LCRH_FEN: u32 = 1 << 4; // Enable FIFOs
#[allow(dead_code)]
const UART_LCRH_STP2: u32 = 1 << 3; // Two Stop Bits
#[allow(dead_code)]
const UART_LCRH_EPS: u32 = 1 << 2; // Even Parity Select
#[allow(dead_code)]
const UART_LCRH_PEN: u32 = 1 << 1; // Parity Enable
#[allow(dead_code)]
const UART_LCRH_BRK: u32 = 1 << 0; // Send Break

/// UART driver structure
pub struct Uart {
    initialized: bool,
}

impl Uart {
    /// Create a new UART instance
    pub const fn new() -> Self {
        Uart { initialized: false }
    }

    /// Initialize the UART controller
    pub unsafe fn init(&mut self) {
        // Disable UART during initialization
        ptr::write_volatile(UART_CR as *mut u32, 0);

        // Set baud rate to 115200 (assuming 24MHz UARTCLK)
        // Baud rate divisor = UARTCLK / (16 * baud_rate)
        // For 24MHz clock and 115200 baud: divisor = 24000000 / (16 * 115200) = 13.0208...
        // Integer part: 13, Fractional part: 0.0208 * 64 = 1.33 ≈ 1
        ptr::write_volatile(UART_IBRD as *mut u32, 13);
        ptr::write_volatile(UART_FBRD as *mut u32, 1);

        // Set 8 bits, no parity, 1 stop bit, enable FIFOs
        ptr::write_volatile(UART_LCRH as *mut u32, UART_LCRH_WLEN_8 | UART_LCRH_FEN);

        // Enable UART, transmit, and receive
        ptr::write_volatile(
            UART_CR as *mut u32,
            UART_CR_UARTEN | UART_CR_TXE | UART_CR_RXE,
        );

        self.initialized = true;
    }

    /// Write a single byte to UART
    pub unsafe fn write_byte(&self, byte: u8) {
        // Wait for transmit FIFO to have space
        while ptr::read_volatile(UART_FR as *const u32) & UART_FR_TXFF != 0 {
            core::hint::spin_loop();
        }

        ptr::write_volatile(UART_DR as *mut u32, byte as u32);
    }

    /// Write bytes to UART
    pub unsafe fn write_bytes(&self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }

    /// Read a single byte from UART (non-blocking)
    /// Returns None if no data is available
    pub unsafe fn read_byte(&self) -> Option<u8> {
        // Check if receive FIFO is empty
        if ptr::read_volatile(UART_FR as *const u32) & UART_FR_RXFE != 0 {
            return None;
        }

        let data = ptr::read_volatile(UART_DR as *const u32);

        // Check for errors
        if data & (UART_DR_OE | UART_DR_BE | UART_DR_PE | UART_DR_FE) != 0 {
            // Clear error flags
            ptr::write_volatile(UART_RSR_ECR as *mut u32, 0);
            return None;
        }

        Some((data & UART_DR_DATA) as u8)
    }

    /// Read a byte with blocking wait
    pub unsafe fn read_byte_blocking(&self) -> u8 {
        loop {
            if let Some(byte) = self.read_byte() {
                return byte;
            }
            core::hint::spin_loop();
        }
    }

    /// Read a line from UART with basic line editing
    /// Returns the number of bytes read (excluding newline)
    pub unsafe fn read_line(&self, buffer: &mut [u8]) -> usize {
        let mut pos = 0;

        loop {
            let byte = self.read_byte_blocking();

            match byte {
                b'\r' | b'\n' => {
                    // Echo newline
                    self.write_bytes(b"\r\n");
                    return pos;
                }
                b'\x08' | b'\x7f' => {
                    // Backspace or DEL
                    if pos > 0 {
                        pos -= 1;
                        // Echo backspace sequence: backspace, space, backspace
                        self.write_bytes(b"\x08 \x08");
                    }
                }
                b'\x03' => {
                    // Ctrl+C - cancel line
                    self.write_bytes(b"^C\r\n");
                    return 0;
                }
                byte if byte >= 0x20 && byte < 0x7f => {
                    // Printable ASCII character
                    if pos < buffer.len() - 1 {
                        buffer[pos] = byte;
                        pos += 1;
                        self.write_byte(byte); // Echo character
                    }
                }
                _ => {
                    // Ignore other control characters
                }
            }
        }
    }

    /// Check if UART is ready for transmission
    pub unsafe fn is_tx_ready(&self) -> bool {
        ptr::read_volatile(UART_FR as *const u32) & UART_FR_TXFF == 0
    }

    /// Check if UART has received data
    pub unsafe fn has_rx_data(&self) -> bool {
        ptr::read_volatile(UART_FR as *const u32) & UART_FR_RXFE == 0
    }

    /// Flush transmit buffer
    pub unsafe fn flush_tx(&self) {
        while ptr::read_volatile(UART_FR as *const u32) & UART_FR_BUSY != 0 {
            core::hint::spin_loop();
        }
    }
}

/// Global UART instance
static mut GLOBAL_UART: Uart = Uart::new();

/// Initialize the global UART instance
pub unsafe fn init() {
    let uart_ptr = &raw mut GLOBAL_UART;
    (*uart_ptr).init();
}

/// Write bytes to the global UART
pub unsafe fn write_bytes(bytes: &[u8]) {
    let uart_ptr = &raw const GLOBAL_UART;
    (*uart_ptr).write_bytes(bytes);
}

/// Write a single byte to the global UART
pub unsafe fn write_byte(byte: u8) {
    let uart_ptr = &raw const GLOBAL_UART;
    (*uart_ptr).write_byte(byte);
}

/// Read a byte from the global UART (non-blocking)
pub unsafe fn read_byte() -> Option<u8> {
    let uart_ptr = &raw const GLOBAL_UART;
    (*uart_ptr).read_byte()
}

/// Read a byte from the global UART (blocking)
pub unsafe fn read_byte_blocking() -> u8 {
    let uart_ptr = &raw const GLOBAL_UART;
    (*uart_ptr).read_byte_blocking()
}

/// Read a line from the global UART with line editing
pub unsafe fn read_line(buffer: &mut [u8]) -> usize {
    let uart_ptr = &raw const GLOBAL_UART;
    (*uart_ptr).read_line(buffer)
}

/// Check if UART has received data
pub unsafe fn has_rx_data() -> bool {
    let uart_ptr = &raw const GLOBAL_UART;
    (*uart_ptr).has_rx_data()
}
