// PTY (Pseudo-Terminal) implementation for Phase A2
// Provides master/slave PTY pairs for terminal emulation

use crate::lib::error::{Errno, Result};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

/// PTY buffer size (4KB)
const PTY_BUF_SIZE: usize = 4096;

/// Termios flags for terminal control
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Termios {
    /// Input flags
    pub c_iflag: u32,
    /// Output flags
    pub c_oflag: u32,
    /// Control flags
    pub c_cflag: u32,
    /// Local flags
    pub c_lflag: u32,
    /// Line discipline
    pub c_line: u8,
    /// Control characters
    pub c_cc: [u8; 32],
    /// Input speed
    pub c_ispeed: u32,
    /// Output speed
    pub c_ospeed: u32,
}

impl Default for Termios {
    fn default() -> Self {
        let mut termios = Self {
            c_iflag: ICRNL,
            c_oflag: OPOST | ONLCR,
            c_cflag: CS8 | CREAD,
            c_lflag: ISIG | ICANON | ECHO | ECHOE | ECHOK,
            c_line: 0,
            c_cc: [0; 32],
            c_ispeed: B38400,
            c_ospeed: B38400,
        };

        // Set default control characters
        termios.c_cc[VINTR] = 3;   // Ctrl-C
        termios.c_cc[VQUIT] = 28;  // Ctrl-\
        termios.c_cc[VERASE] = 127; // DEL
        termios.c_cc[VKILL] = 21;  // Ctrl-U
        termios.c_cc[VEOF] = 4;    // Ctrl-D
        termios.c_cc[VTIME] = 0;
        termios.c_cc[VMIN] = 1;
        termios.c_cc[VSTART] = 17; // Ctrl-Q
        termios.c_cc[VSTOP] = 19;  // Ctrl-S

        termios
    }
}

// Termios iflag bits
pub const ICRNL: u32 = 0o000400;  // Map CR to NL on input
pub const IGNBRK: u32 = 0o000001; // Ignore break
pub const BRKINT: u32 = 0o000002; // Signal on break
pub const IGNPAR: u32 = 0o000004; // Ignore parity errors

// Termios oflag bits
pub const OPOST: u32 = 0o000001;  // Post-process output
pub const ONLCR: u32 = 0o000004;  // Map NL to CR-NL on output

// Termios cflag bits
pub const CS8: u32 = 0o000060;    // 8 bits
pub const CREAD: u32 = 0o000200;  // Enable receiver

// Termios lflag bits
pub const ISIG: u32 = 0o000001;   // Enable signals
pub const ICANON: u32 = 0o000002; // Canonical mode
pub const ECHO: u32 = 0o000010;   // Echo input
pub const ECHOE: u32 = 0o000020;  // Visual erase
pub const ECHOK: u32 = 0o000040;  // Echo NL after kill
pub const IEXTEN: u32 = 0o100000; // Extended input processing

// Control character indices
pub const VINTR: usize = 0;
pub const VQUIT: usize = 1;
pub const VERASE: usize = 2;
pub const VKILL: usize = 3;
pub const VEOF: usize = 4;
pub const VTIME: usize = 5;
pub const VMIN: usize = 6;
pub const VSTART: usize = 8;
pub const VSTOP: usize = 9;

// Baud rates
pub const B38400: u32 = 15;

/// PTY pair buffer shared between master and slave
pub struct PtyBuffer {
    /// Master -> Slave buffer (master writes, slave reads)
    m2s_buffer: VecDeque<u8>,
    /// Slave -> Master buffer (slave writes, master reads)
    s2m_buffer: VecDeque<u8>,
    /// Terminal settings
    termios: Termios,
    /// Line buffer for canonical mode
    line_buffer: Vec<u8>,
    /// PTY number
    pty_num: usize,
}

impl PtyBuffer {
    pub fn new(pty_num: usize) -> Self {
        Self {
            m2s_buffer: VecDeque::with_capacity(PTY_BUF_SIZE),
            s2m_buffer: VecDeque::with_capacity(PTY_BUF_SIZE),
            termios: Termios::default(),
            line_buffer: Vec::new(),
            pty_num,
        }
    }

    /// Read from master (gets data from slave)
    pub fn master_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.s2m_buffer.is_empty() {
            return Err(Errno::EAGAIN);
        }

        let to_read = buf.len().min(self.s2m_buffer.len());
        for i in 0..to_read {
            buf[i] = self.s2m_buffer.pop_front().unwrap();
        }

        Ok(to_read)
    }

    /// Write to master (sends data to slave)
    pub fn master_write(&mut self, buf: &[u8]) -> Result<usize> {
        let available = PTY_BUF_SIZE - self.m2s_buffer.len();
        if available == 0 {
            return Err(Errno::EAGAIN);
        }

        let to_write = buf.len().min(available);
        for i in 0..to_write {
            self.m2s_buffer.push_back(buf[i]);
        }

        Ok(to_write)
    }

    /// Read from slave (gets data from master)
    pub fn slave_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Canonical mode: wait for newline
        if self.termios.c_lflag & ICANON != 0 {
            // Check if we have a complete line in m2s_buffer
            let has_newline = self.m2s_buffer.iter().any(|&c| c == b'\n');
            if !has_newline && !self.m2s_buffer.is_empty() {
                // Accumulate in line buffer
                while let Some(c) = self.m2s_buffer.pop_front() {
                    self.line_buffer.push(c);
                    if c == b'\n' {
                        break;
                    }
                }
            }

            // Return line if we have one
            if !self.line_buffer.is_empty() {
                let to_copy = buf.len().min(self.line_buffer.len());
                buf[..to_copy].copy_from_slice(&self.line_buffer[..to_copy]);
                self.line_buffer.drain(..to_copy);
                return Ok(to_copy);
            }

            // No complete line yet
            return Err(Errno::EAGAIN);
        }

        // Raw mode: read directly
        if self.m2s_buffer.is_empty() {
            return Err(Errno::EAGAIN);
        }

        let to_read = buf.len().min(self.m2s_buffer.len());
        for i in 0..to_read {
            buf[i] = self.m2s_buffer.pop_front().unwrap();
        }

        Ok(to_read)
    }

    /// Write to slave (sends data to master, with line discipline)
    pub fn slave_write(&mut self, buf: &[u8]) -> Result<usize> {
        let available = PTY_BUF_SIZE - self.s2m_buffer.len();
        if available == 0 {
            return Err(Errno::EAGAIN);
        }

        let mut written = 0;
        for &byte in buf.iter().take(available) {
            // Apply output processing if OPOST is set
            if self.termios.c_oflag & OPOST != 0 {
                if byte == b'\n' && (self.termios.c_oflag & ONLCR != 0) {
                    // Map NL to CR-NL
                    if self.s2m_buffer.len() + 2 <= PTY_BUF_SIZE {
                        self.s2m_buffer.push_back(b'\r');
                        self.s2m_buffer.push_back(b'\n');
                        written += 1;
                    } else {
                        break;
                    }
                } else {
                    self.s2m_buffer.push_back(byte);
                    written += 1;
                }
            } else {
                self.s2m_buffer.push_back(byte);
                written += 1;
            }

            // Echo if enabled
            if self.termios.c_lflag & ECHO != 0 {
                // Echo back to master->slave buffer
                if self.m2s_buffer.len() < PTY_BUF_SIZE {
                    self.m2s_buffer.push_back(byte);
                }
            }
        }

        Ok(written)
    }

    /// Get termios settings
    pub fn get_termios(&self) -> Termios {
        self.termios
    }

    /// Set termios settings
    pub fn set_termios(&mut self, termios: Termios) {
        self.termios = termios;
    }

    /// Get PTY number
    pub fn pty_num(&self) -> usize {
        self.pty_num
    }
}

/// PTY master end
#[derive(Clone)]
pub struct PtyMaster {
    buffer: Arc<Mutex<PtyBuffer>>,
}

impl PtyMaster {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        self.buffer.lock().master_read(buf)
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        self.buffer.lock().master_write(buf)
    }

    pub fn get_termios(&self) -> Termios {
        self.buffer.lock().get_termios()
    }

    pub fn set_termios(&self, termios: Termios) {
        self.buffer.lock().set_termios(termios);
    }

    pub fn pty_num(&self) -> usize {
        self.buffer.lock().pty_num()
    }
}

/// PTY slave end
#[derive(Clone)]
pub struct PtySlave {
    buffer: Arc<Mutex<PtyBuffer>>,
}

impl PtySlave {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        self.buffer.lock().slave_read(buf)
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        self.buffer.lock().slave_write(buf)
    }

    pub fn get_termios(&self) -> Termios {
        self.buffer.lock().get_termios()
    }

    pub fn set_termios(&self, termios: Termios) {
        self.buffer.lock().set_termios(termios);
    }

    pub fn pty_num(&self) -> usize {
        self.buffer.lock().pty_num()
    }
}

/// Global PTY allocator
static PTY_ALLOCATOR: Mutex<PtyAllocator> = Mutex::new(PtyAllocator::new());

struct PtyAllocator {
    next_pty: usize,
    max_pty: usize,
}

impl PtyAllocator {
    const fn new() -> Self {
        Self {
            next_pty: 0,
            max_pty: 256, // Maximum 256 PTYs
        }
    }

    fn alloc(&mut self) -> Option<usize> {
        if self.next_pty >= self.max_pty {
            return None;
        }
        let num = self.next_pty;
        self.next_pty += 1;
        Some(num)
    }
}

/// Create a new PTY pair
pub fn create_pty_pair() -> Result<(PtyMaster, PtySlave)> {
    let pty_num = PTY_ALLOCATOR.lock().alloc().ok_or(Errno::ENOMEM)?;

    let buffer = Arc::new(Mutex::new(PtyBuffer::new(pty_num)));

    let master = PtyMaster {
        buffer: buffer.clone(),
    };

    let slave = PtySlave {
        buffer,
    };

    Ok((master, slave))
}

/// IOCTL commands for termios
pub const TCGETS: u32 = 0x5401;
pub const TCSETS: u32 = 0x5402;
pub const TCSETSW: u32 = 0x5403;
pub const TCSETSF: u32 = 0x5404;
pub const TIOCGPTN: u32 = 0x80045430;
pub const TIOCSPTLCK: u32 = 0x40045431;
