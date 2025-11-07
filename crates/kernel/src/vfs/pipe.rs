// Pipe implementation for Phase A1
// Provides anonymous pipes for IPC between processes

use crate::lib::error::{Errno, Result};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;

/// Pipe buffer size (4KB)
const PIPE_BUF_SIZE: usize = 4096;

/// Pipe buffer shared between reader and writer
pub struct PipeBuffer {
    /// Ring buffer for data
    buffer: VecDeque<u8>,
    /// Number of readers still alive
    reader_count: usize,
    /// Number of writers still alive
    writer_count: usize,
    /// Flag indicating if pipe is closed
    closed: bool,
}

impl PipeBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(PIPE_BUF_SIZE),
            reader_count: 1,
            writer_count: 1,
            closed: false,
        }
    }

    /// Read from pipe buffer
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // If buffer is empty and no writers, return EOF
        if self.buffer.is_empty() && self.writer_count == 0 {
            return Ok(0);
        }

        // If buffer is empty and writers exist, would block
        // Phase A1: Simple blocking read
        if self.buffer.is_empty() {
            // TODO: Implement proper blocking/waiting
            // For now, return EAGAIN (would block)
            return Err(Errno::EAGAIN);
        }

        // Read available data
        let to_read = buf.len().min(self.buffer.len());
        for i in 0..to_read {
            buf[i] = self.buffer.pop_front().unwrap();
        }

        Ok(to_read)
    }

    /// Write to pipe buffer
    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // If no readers, send SIGPIPE (Phase A1: just return EPIPE)
        if self.reader_count == 0 {
            // TODO: Send SIGPIPE to writer
            return Err(Errno::EPIPE);
        }

        // If buffer is full, would block
        // Phase A1: Write what we can
        let available = PIPE_BUF_SIZE - self.buffer.len();
        if available == 0 {
            // TODO: Implement proper blocking/waiting
            return Err(Errno::EAGAIN);
        }

        // Write data
        let to_write = buf.len().min(available);
        for i in 0..to_write {
            self.buffer.push_back(buf[i]);
        }

        Ok(to_write)
    }

    /// Increment reader count
    pub fn add_reader(&mut self) {
        self.reader_count += 1;
    }

    /// Decrement reader count
    pub fn remove_reader(&mut self) {
        if self.reader_count > 0 {
            self.reader_count -= 1;
        }
    }

    /// Increment writer count
    pub fn add_writer(&mut self) {
        self.writer_count += 1;
    }

    /// Decrement writer count
    pub fn remove_writer(&mut self) {
        if self.writer_count > 0 {
            self.writer_count -= 1;
        }
    }

    /// Check if pipe is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Check if pipe is full
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= PIPE_BUF_SIZE
    }

    /// Get available data size
    pub fn available(&self) -> usize {
        self.buffer.len()
    }

    /// Get free space size
    pub fn free_space(&self) -> usize {
        PIPE_BUF_SIZE - self.buffer.len()
    }
}

/// Pipe read end
#[derive(Clone)]
pub struct PipeReader {
    buffer: Arc<Mutex<PipeBuffer>>,
}

impl PipeReader {
    pub fn new(buffer: Arc<Mutex<PipeBuffer>>) -> Self {
        buffer.lock().add_reader();
        Self { buffer }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        self.buffer.lock().read(buf)
    }
}

impl Drop for PipeReader {
    fn drop(&mut self) {
        self.buffer.lock().remove_reader();
    }
}

/// Pipe write end
#[derive(Clone)]
pub struct PipeWriter {
    buffer: Arc<Mutex<PipeBuffer>>,
}

impl PipeWriter {
    pub fn new(buffer: Arc<Mutex<PipeBuffer>>) -> Self {
        buffer.lock().add_writer();
        Self { buffer }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        self.buffer.lock().write(buf)
    }
}

impl Drop for PipeWriter {
    fn drop(&mut self) {
        self.buffer.lock().remove_writer();
    }
}

/// Create a new pipe (returns reader and writer)
pub fn create_pipe() -> (PipeReader, PipeWriter) {
    let buffer = Arc::new(Mutex::new(PipeBuffer::new()));
    let reader = PipeReader::new(buffer.clone());
    let writer = PipeWriter::new(buffer);
    (reader, writer)
}

// Pipe ends are now wrapped in File objects in vfs/file.rs using File::from_pipe_reader/writer
