/// File - represents an open file
///
/// Tracks offset, flags, and provides FileOps for I/O operations.

use super::inode::Inode;
use crate::lib::error::Errno;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

// Forward declaration for pipe type
pub enum PipeEnd {
    Reader(super::pipe::PipeReader),
    Writer(super::pipe::PipeWriter),
}

// PTY ends (master or slave)
pub enum PtyEnd {
    Master(crate::drivers::char::pty::PtyMaster),
    Slave(crate::drivers::char::pty::PtySlave),
}

bitflags::bitflags! {
    /// File open flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OpenFlags: u32 {
        const O_RDONLY = 0o0;
        const O_WRONLY = 0o1;
        const O_RDWR = 0o2;
        const O_CREAT = 0o100;
        const O_TRUNC = 0o1000;
        const O_APPEND = 0o2000;
        const O_DIRECTORY = 0o200000;
        const O_CLOEXEC = 0o2000000;
    }
}

impl OpenFlags {
    /// Check if readable
    pub fn is_readable(&self) -> bool {
        (*self & OpenFlags::O_WRONLY).is_empty()
    }

    /// Check if writable
    pub fn is_writable(&self) -> bool {
        self.intersects(OpenFlags::O_WRONLY | OpenFlags::O_RDWR)
    }
}

/// File operations trait
pub trait FileOps: Send + Sync {
    /// Read from file
    fn read(&self, file: &File, buf: &mut [u8]) -> Result<usize, Errno>;

    /// Write to file
    fn write(&self, file: &File, buf: &[u8]) -> Result<usize, Errno>;

    /// Seek to position
    fn lseek(&self, file: &File, offset: i64, whence: i32) -> Result<u64, Errno> {
        let _ = (file, offset, whence);
        Err(Errno::ESPIPE) // Default: not seekable
    }

    /// I/O control
    fn ioctl(&self, file: &File, cmd: u32, arg: usize) -> Result<isize, Errno> {
        let _ = (file, cmd, arg);
        Err(Errno::ENOTTY) // Default: not a TTY
    }

    /// Poll (stub for A1)
    fn poll(&self, file: &File) -> Result<u32, Errno> {
        let _ = file;
        Ok(0x01 | 0x04) // POLLIN | POLLOUT (always ready)
    }

    /// Memory map (stub for A1)
    fn mmap(&self, file: &File) -> Result<u64, Errno> {
        let _ = file;
        Err(Errno::ENODEV)
    }
}

/// File structure
pub struct File {
    pub inode: Option<Arc<Inode>>,
    pub offset: AtomicU64,
    pub flags: OpenFlags,
    pub fops: &'static dyn FileOps,
    pub pipe: Option<PipeEnd>,
    pub pty: Option<PtyEnd>,
}

impl File {
    /// Create a new file
    pub fn new(inode: Arc<Inode>, flags: OpenFlags) -> Self {
        // Get file operations from filesystem type
        // For now, use default ops that delegate to inode
        Self {
            inode: Some(inode),
            offset: AtomicU64::new(0),
            flags,
            fops: &DefaultFileOps,
            pipe: None,
            pty: None,
        }
    }

    /// Create with specific FileOps
    pub fn new_with_ops(inode: Arc<Inode>, flags: OpenFlags, fops: &'static dyn FileOps) -> Self {
        Self {
            inode: Some(inode),
            offset: AtomicU64::new(0),
            flags,
            fops,
            pipe: None,
            pty: None,
        }
    }

    /// Create a file from a pipe reader
    pub fn from_pipe_reader(reader: super::pipe::PipeReader) -> Self {
        Self {
            inode: None,
            offset: AtomicU64::new(0),
            flags: OpenFlags::O_RDONLY,
            fops: &PipeFileOps,
            pipe: Some(PipeEnd::Reader(reader)),
            pty: None,
        }
    }

    /// Create a file from a pipe writer
    pub fn from_pipe_writer(writer: super::pipe::PipeWriter) -> Self {
        Self {
            inode: None,
            offset: AtomicU64::new(0),
            flags: OpenFlags::O_WRONLY,
            fops: &PipeFileOps,
            pipe: Some(PipeEnd::Writer(writer)),
            pty: None,
        }
    }

    /// Create a file from a PTY master
    pub fn from_pty_master(master: crate::drivers::char::pty::PtyMaster) -> Self {
        Self {
            inode: None,
            offset: AtomicU64::new(0),
            flags: OpenFlags::O_RDWR,
            fops: &PtyFileOps,
            pipe: None,
            pty: Some(PtyEnd::Master(master)),
        }
    }

    /// Create a file from a PTY slave
    pub fn from_pty_slave(slave: crate::drivers::char::pty::PtySlave) -> Self {
        Self {
            inode: None,
            offset: AtomicU64::new(0),
            flags: OpenFlags::O_RDWR,
            fops: &PtyFileOps,
            pipe: None,
            pty: Some(PtyEnd::Slave(slave)),
        }
    }

    /// Read from file
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, Errno> {
        if !self.flags.is_readable() {
            return Err(Errno::EBADF);
        }
        self.fops.read(self, buf)
    }

    /// Write to file
    pub fn write(&self, buf: &[u8]) -> Result<usize, Errno> {
        if !self.flags.is_writable() {
            return Err(Errno::EBADF);
        }
        self.fops.write(self, buf)
    }

    /// Seek
    pub fn lseek(&self, offset: i64, whence: i32) -> Result<u64, Errno> {
        self.fops.lseek(self, offset, whence)
    }

    /// I/O control
    pub fn ioctl(&self, cmd: u32, arg: usize) -> Result<isize, Errno> {
        self.fops.ioctl(self, cmd, arg)
    }

    /// Get current offset
    pub fn offset(&self) -> u64 {
        self.offset.load(Ordering::Acquire)
    }

    /// Set offset
    pub fn set_offset(&self, offset: u64) {
        self.offset.store(offset, Ordering::Release);
    }

    /// Advance offset
    pub fn advance_offset(&self, n: usize) {
        self.offset.fetch_add(n as u64, Ordering::AcqRel);
    }
}

impl core::fmt::Debug for File {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("File");
        if let Some(ref inode) = self.inode {
            d.field("inode", &inode.ino());
        } else if self.pipe.is_some() {
            d.field("type", &"pipe");
        } else if self.pty.is_some() {
            d.field("type", &"pty");
        }
        d.field("offset", &self.offset())
            .field("flags", &self.flags)
            .finish()
    }
}

/// Default file operations (delegate to inode)
struct DefaultFileOps;

impl FileOps for DefaultFileOps {
    fn read(&self, file: &File, buf: &mut [u8]) -> Result<usize, Errno> {
        let inode = file.inode.as_ref().ok_or(Errno::EBADF)?;
        let offset = file.offset();
        let n = inode.read(offset, buf)?;
        file.advance_offset(n);
        Ok(n)
    }

    fn write(&self, file: &File, buf: &[u8]) -> Result<usize, Errno> {
        let inode = file.inode.as_ref().ok_or(Errno::EBADF)?;
        let offset = file.offset();
        let n = inode.write(offset, buf)?;
        file.advance_offset(n);
        Ok(n)
    }

    fn lseek(&self, file: &File, offset: i64, whence: i32) -> Result<u64, Errno> {
        const SEEK_SET: i32 = 0;
        const SEEK_CUR: i32 = 1;
        const SEEK_END: i32 = 2;

        let current = file.offset() as i64;
        let size = file.inode.size() as i64;

        let new_offset = match whence {
            SEEK_SET => offset,
            SEEK_CUR => current + offset,
            SEEK_END => size + offset,
            _ => return Err(Errno::EINVAL),
        };

        if new_offset < 0 {
            return Err(Errno::EINVAL);
        }

        file.set_offset(new_offset as u64);
        Ok(new_offset as u64)
    }
}

/// Pipe file operations
struct PipeFileOps;

impl FileOps for PipeFileOps {
    fn read(&self, file: &File, buf: &mut [u8]) -> Result<usize, Errno> {
        match &file.pipe {
            Some(PipeEnd::Reader(reader)) => reader.read(buf),
            Some(PipeEnd::Writer(_)) => Err(Errno::EBADF), // Can't read from write end
            None => Err(Errno::EBADF),
        }
    }

    fn write(&self, file: &File, buf: &[u8]) -> Result<usize, Errno> {
        match &file.pipe {
            Some(PipeEnd::Writer(writer)) => writer.write(buf),
            Some(PipeEnd::Reader(_)) => Err(Errno::EBADF), // Can't write to read end
            None => Err(Errno::EBADF),
        }
    }

    fn lseek(&self, _file: &File, _offset: i64, _whence: i32) -> Result<u64, Errno> {
        Err(Errno::ESPIPE) // Pipes are not seekable
    }
}

/// PTY file operations
struct PtyFileOps;

impl FileOps for PtyFileOps {
    fn read(&self, file: &File, buf: &mut [u8]) -> Result<usize, Errno> {
        match &file.pty {
            Some(PtyEnd::Master(master)) => master.read(buf),
            Some(PtyEnd::Slave(slave)) => slave.read(buf),
            None => Err(Errno::EBADF),
        }
    }

    fn write(&self, file: &File, buf: &[u8]) -> Result<usize, Errno> {
        match &file.pty {
            Some(PtyEnd::Master(master)) => master.write(buf),
            Some(PtyEnd::Slave(slave)) => slave.write(buf),
            None => Err(Errno::EBADF),
        }
    }

    fn lseek(&self, _file: &File, _offset: i64, _whence: i32) -> Result<u64, Errno> {
        Err(Errno::ESPIPE) // PTYs are not seekable
    }

    fn ioctl(&self, file: &File, cmd: u32, arg: usize) -> Result<isize, Errno> {
        use crate::drivers::char::pty::{TCGETS, TCSETS, TCSETSW, TCSETSF, TIOCGPTN, TIOCSPTLCK, Termios};

        match &file.pty {
            Some(PtyEnd::Master(master)) => {
                match cmd {
                    TCGETS => {
                        // Get termios
                        let termios = master.get_termios();
                        let user_ptr = arg as *mut Termios;
                        unsafe {
                            core::ptr::write(user_ptr, termios);
                        }
                        Ok(0)
                    }
                    TCSETS | TCSETSW | TCSETSF => {
                        // Set termios (all three treated the same in Phase A2)
                        let user_ptr = arg as *const Termios;
                        let termios = unsafe { core::ptr::read(user_ptr) };
                        master.set_termios(termios);
                        Ok(0)
                    }
                    TIOCGPTN => {
                        // Get PTY number
                        let user_ptr = arg as *mut u32;
                        unsafe {
                            core::ptr::write(user_ptr, master.pty_num() as u32);
                        }
                        Ok(0)
                    }
                    TIOCSPTLCK => {
                        // Lock/unlock PTY (Phase A2: no-op, always unlocked)
                        Ok(0)
                    }
                    _ => Err(Errno::ENOTTY),
                }
            }
            Some(PtyEnd::Slave(slave)) => {
                match cmd {
                    TCGETS => {
                        let termios = slave.get_termios();
                        let user_ptr = arg as *mut Termios;
                        unsafe {
                            core::ptr::write(user_ptr, termios);
                        }
                        Ok(0)
                    }
                    TCSETS | TCSETSW | TCSETSF => {
                        let user_ptr = arg as *const Termios;
                        let termios = unsafe { core::ptr::read(user_ptr) };
                        slave.set_termios(termios);
                        Ok(0)
                    }
                    _ => Err(Errno::ENOTTY),
                }
            }
            None => Err(Errno::EBADF),
        }
    }
}
