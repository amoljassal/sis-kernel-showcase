/// Console character device driver
///
/// Provides read/write operations for /dev/console using UART.

use crate::vfs::{File, FileOps};
use crate::lib::error::Errno;

/// Console FileOps implementation
pub struct ConsoleOps;

impl FileOps for ConsoleOps {
    fn read(&self, _file: &File, buf: &mut [u8]) -> Result<usize, Errno> {
        // Simple polled/blocking read from UART
        // Read bytes until buffer full or newline
        let mut count = 0;

        while count < buf.len() {
            // Poll UART for a byte (blocking)
            let byte = read_uart_byte();
            buf[count] = byte;
            count += 1;

            // Stop at newline
            if byte == b'\n' {
                break;
            }
        }

        Ok(count)
    }

    fn write(&self, _file: &File, buf: &[u8]) -> Result<usize, Errno> {
        // Write to UART
        unsafe {
            crate::uart::write_bytes(buf);
        }
        Ok(buf.len())
    }

    fn ioctl(&self, _file: &File, cmd: u32, _arg: usize) -> Result<isize, Errno> {
        // For Phase A1, return ENOTTY for all ioctl commands
        // TODO: Implement TCGETS/TCSETS for termios in Phase A2
        crate::debug!("console ioctl cmd={:#x} - returning ENOTTY", cmd);
        Err(Errno::ENOTTY)
    }
}

/// Global console ops instance
pub static CONSOLE_OPS: ConsoleOps = ConsoleOps;

/// Read a single byte from UART (blocking/polled)
fn read_uart_byte() -> u8 {
    // For Phase A1, use simple polling
    // TODO: Use interrupt-driven I/O in future phases

    // Access UART data register via platform
    let uart_base = crate::platform::active().uart().base as *mut u32;

    unsafe {
        // UART DR (data register) offset 0x00
        let dr = uart_base;

        // UART FR (flag register) offset 0x18
        let fr = uart_base.add(0x18 / 4);

        // Bit 4 of FR is RXFE (receive FIFO empty)
        loop {
            let flags = core::ptr::read_volatile(fr);
            if (flags & (1 << 4)) == 0 {
                // Data available
                break;
            }
            // Spin wait (or could use WFI for power efficiency)
            core::hint::spin_loop();
        }

        // Read the byte
        let data = core::ptr::read_volatile(dr);
        (data & 0xFF) as u8
    }
}

/// Null device operations (write sink, read EOF)
pub struct NullOps;

impl FileOps for NullOps {
    fn read(&self, _file: &File, _buf: &mut [u8]) -> Result<usize, Errno> {
        // Read returns EOF immediately
        Ok(0)
    }

    fn write(&self, _file: &File, buf: &[u8]) -> Result<usize, Errno> {
        // Write succeeds but discards data
        Ok(buf.len())
    }
}

pub static NULL_OPS: NullOps = NullOps;

/// Zero device operations (read zeros, write sink)
pub struct ZeroOps;

impl FileOps for ZeroOps {
    fn read(&self, _file: &File, buf: &mut [u8]) -> Result<usize, Errno> {
        // Fill buffer with zeros
        buf.fill(0);
        Ok(buf.len())
    }

    fn write(&self, _file: &File, buf: &[u8]) -> Result<usize, Errno> {
        // Write succeeds but discards data
        Ok(buf.len())
    }
}

pub static ZERO_OPS: ZeroOps = ZeroOps;

/// Random/urandom device operations (PRNG)
pub struct RandomOps;

impl FileOps for RandomOps {
    fn read(&self, _file: &File, buf: &mut [u8]) -> Result<usize, Errno> {
        // Use Phase D entropy source
        crate::security::fill_random_bytes(buf);
        Ok(buf.len())
    }

    fn write(&self, _file: &File, buf: &[u8]) -> Result<usize, Errno> {
        // Write to random is silently ignored (could be used for entropy)
        Ok(buf.len())
    }
}

pub static RANDOM_OPS: RandomOps = RandomOps;
