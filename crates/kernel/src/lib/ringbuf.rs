// Ring buffer utility for kernel logging
// Phase A0 - Simple circular buffer implementation

use core::sync::atomic::{AtomicUsize, Ordering};

pub struct RingBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    pub const fn new() -> Self {
        Self {
            buffer: [None; N],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn push(&mut self, item: T) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        let next_head = (head + 1) % N;

        if next_head == tail {
            // Buffer full, overwrite oldest
            self.tail.store((tail + 1) % N, Ordering::Release);
        }

        self.buffer[head] = Some(item);
        self.head.store(next_head, Ordering::Release);
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Relaxed);

        if head == tail {
            return None; // Empty
        }

        let item = self.buffer[tail];
        self.tail.store((tail + 1) % N, Ordering::Release);
        item
    }

    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);

        if head >= tail {
            head - tail
        } else {
            N - tail + head
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn drain_all(&mut self) -> alloc::vec::Vec<T> {
        let mut result = alloc::vec::Vec::new();
        while let Some(item) = self.pop() {
            result.push(item);
        }
        result
    }
}

// Safe for use in static context
unsafe impl<T: Copy, const N: usize> Sync for RingBuffer<T, N> {}
