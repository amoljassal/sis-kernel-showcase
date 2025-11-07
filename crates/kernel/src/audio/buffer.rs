/// Ring Buffer - Phase G.5
///
/// Circular buffer for audio samples

use core::marker::PhantomData;

/// Ring buffer with fixed capacity
pub struct RingBuffer<T: Copy + Default, const N: usize> {
    buffer: [T; N],
    read_pos: usize,
    write_pos: usize,
    count: usize,
    _marker: PhantomData<T>,
}

impl<T: Copy + Default, const N: usize> RingBuffer<T, N> {
    /// Create a new ring buffer
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); N],
            read_pos: 0,
            write_pos: 0,
            count: 0,
            _marker: PhantomData,
        }
    }

    /// Write a single sample
    pub fn write(&mut self, sample: T) -> bool {
        if self.count >= N {
            return false; // Buffer full
        }

        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % N;
        self.count += 1;
        true
    }

    /// Write multiple samples
    pub fn write_slice(&mut self, samples: &[T]) -> usize {
        let mut written = 0;
        for &sample in samples {
            if !self.write(sample) {
                break;
            }
            written += 1;
        }
        written
    }

    /// Read a single sample
    pub fn read(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }

        let sample = self.buffer[self.read_pos];
        self.read_pos = (self.read_pos + 1) % N;
        self.count -= 1;
        Some(sample)
    }

    /// Read multiple samples
    pub fn read_slice(&mut self, output: &mut [T]) -> usize {
        let mut read_count = 0;
        for i in 0..output.len() {
            if let Some(sample) = self.read() {
                output[i] = sample;
                read_count += 1;
            } else {
                break;
            }
        }
        read_count
    }

    /// Peek at a sample without removing it
    pub fn peek(&self, offset: usize) -> Option<T> {
        if offset >= self.count {
            return None;
        }
        let pos = (self.read_pos + offset) % N;
        Some(self.buffer[pos])
    }

    /// Get number of samples available
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.count >= N
    }

    /// Get remaining capacity
    pub fn remaining(&self) -> usize {
        N - self.count
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
        self.count = 0;
    }

    /// Get buffer capacity
    pub const fn capacity(&self) -> usize {
        N
    }
}

impl<T: Copy + Default, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer() {
        let mut buf: RingBuffer<i16, 8> = RingBuffer::new();

        // Test write
        assert!(buf.write(1));
        assert!(buf.write(2));
        assert_eq!(buf.len(), 2);

        // Test read
        assert_eq!(buf.read(), Some(1));
        assert_eq!(buf.read(), Some(2));
        assert_eq!(buf.len(), 0);

        // Test wrap around
        for i in 0..10 {
            buf.write(i);
        }
        assert_eq!(buf.len(), 8); // Buffer capacity

        // Test peek
        assert_eq!(buf.peek(0), Some(2));
        assert_eq!(buf.peek(1), Some(3));
        assert_eq!(buf.len(), 8); // Peek doesn't consume
    }
}
