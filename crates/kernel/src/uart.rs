//! ARM PL011 UART driver for SIS kernel
//!
//! Provides input and output functionality for the PL011 UART controller
//! commonly found in ARM platforms including QEMU's virt machine.

use core::ptr;

// Runtime-configurable base and clock populated during init()
// Default to 0 to avoid hardcoded MMIO; early-boot prints use platform UART directly.
static mut UART_BASE_ADDR: usize = 0;
static mut UART_CLOCK_HZ: u32 = 24_000_000;     // default when platform doesn't specify

/// PL011 UART register offsets helpers (computed from runtime base)
#[inline(always)]
fn uart_base() -> usize { unsafe { UART_BASE_ADDR } }
#[inline(always)]
fn reg_dr() -> usize { uart_base() + 0x000 }
#[inline(always)]
fn reg_rsr_ecr() -> usize { uart_base() + 0x004 }
#[inline(always)]
fn reg_fr() -> usize { uart_base() + 0x018 }
#[allow(dead_code)]
#[inline(always)]
fn reg_ilpr() -> usize { uart_base() + 0x020 }
#[inline(always)]
fn reg_ibrd() -> usize { uart_base() + 0x024 }
#[inline(always)]
fn reg_fbrd() -> usize { uart_base() + 0x028 }
#[inline(always)]
fn reg_lcrh() -> usize { uart_base() + 0x02C }
#[inline(always)]
fn reg_cr() -> usize { uart_base() + 0x030 }
#[allow(dead_code)]
#[inline(always)]
fn reg_ifls() -> usize { uart_base() + 0x034 }
#[allow(dead_code)]
#[inline(always)]
fn reg_imsc() -> usize { uart_base() + 0x038 }
#[allow(dead_code)]
#[inline(always)]
fn reg_ris() -> usize { uart_base() + 0x03C }
#[allow(dead_code)]
#[inline(always)]
fn reg_mis() -> usize { uart_base() + 0x040 }
#[allow(dead_code)]
#[inline(always)]
fn reg_icr() -> usize { uart_base() + 0x044 }

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
        // Capture platform-provided UART base/clock
        let desc = crate::platform::active().uart();
        UART_BASE_ADDR = desc.base;
        UART_CLOCK_HZ = if desc.clock_hz != 0 { desc.clock_hz } else { UART_CLOCK_HZ };

        // Disable UART during initialization
        ptr::write_volatile(reg_cr() as *mut u32, 0);

        // Set baud rate to 115200 using platform clock
        // Baud rate divisor = UARTCLK / (16 * baud_rate)
        let baud: u32 = 115_200;
        let clk: u32 = UART_CLOCK_HZ;
        let div_times_64: u32 = (clk / (16 * baud)) * 64 + (((clk % (16 * baud)) * 64) / (16 * baud));
        let ibrd: u32 = div_times_64 / 64;
        let fbrd: u32 = div_times_64 % 64;
        ptr::write_volatile(reg_ibrd() as *mut u32, ibrd.max(1));
        ptr::write_volatile(reg_fbrd() as *mut u32, fbrd);

        // Set 8 bits, no parity, 1 stop bit, enable FIFOs
        ptr::write_volatile(reg_lcrh() as *mut u32, UART_LCRH_WLEN_8 | UART_LCRH_FEN);

        // Enable UART, transmit, and receive
        ptr::write_volatile(
            reg_cr() as *mut u32,
            UART_CR_UARTEN | UART_CR_TXE | UART_CR_RXE,
        );

        self.initialized = true;
    }

    /// Write a single byte to UART
    pub unsafe fn write_byte(&self, byte: u8) {
        // Wait for transmit FIFO to have space
        while ptr::read_volatile(reg_fr() as *const u32) & UART_FR_TXFF != 0 {
            core::hint::spin_loop();
        }

        ptr::write_volatile(reg_dr() as *mut u32, byte as u32);
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
        if ptr::read_volatile(reg_fr() as *const u32) & UART_FR_RXFE != 0 {
            return None;
        }

        let data = ptr::read_volatile(reg_dr() as *const u32);

        // Check for errors
        if data & (UART_DR_OE | UART_DR_BE | UART_DR_PE | UART_DR_FE) != 0 {
            // Clear error flags
            ptr::write_volatile(reg_rsr_ecr() as *mut u32, 0);
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
        ptr::read_volatile(reg_fr() as *const u32) & UART_FR_TXFF == 0
    }

    /// Check if UART has received data
    pub unsafe fn has_rx_data(&self) -> bool {
        ptr::read_volatile(reg_fr() as *const u32) & UART_FR_RXFE == 0
    }

    /// Flush transmit buffer
    pub unsafe fn flush_tx(&self) {
        while ptr::read_volatile(reg_fr() as *const u32) & UART_FR_BUSY != 0 {
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
