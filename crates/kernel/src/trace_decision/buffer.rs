//! Decision Trace Ring Buffer
//!
//! Maintains last N decision traces in memory for quick access.
//! Uses lock-protected ring buffer for thread-safe operations.

use alloc::vec::Vec;
use spin::Mutex;
use super::decision::DecisionTrace;

/// Ring buffer capacity (last 1024 decisions)
const TRACE_BUFFER_SIZE: usize = 1024;

/// Thread-safe ring buffer for decision traces
pub struct TraceBuffer {
    buffer: Mutex<Vec<DecisionTrace>>,
    capacity: usize,
    next_index: Mutex<usize>,
}

impl TraceBuffer {
    /// Create new trace buffer with default capacity
    pub const fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
            capacity: TRACE_BUFFER_SIZE,
            next_index: Mutex::new(0),
        }
    }

    /// Create trace buffer with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(Vec::with_capacity(capacity)),
            capacity,
            next_index: Mutex::new(0),
        }
    }

    /// Record a decision trace
    pub fn record(&self, trace: DecisionTrace) {
        let mut buffer = self.buffer.lock();
        let mut index = self.next_index.lock();

        if buffer.len() < self.capacity {
            // Buffer not full yet, just push
            buffer.push(trace);
        } else {
            // Buffer full, overwrite oldest
            buffer[*index % self.capacity] = trace;
        }

        *index = (*index + 1) % self.capacity;
    }

    /// Get last N traces (most recent first)
    pub fn get_last_n(&self, n: usize) -> Vec<DecisionTrace> {
        let buffer = self.buffer.lock();
        let count = core::cmp::min(n, buffer.len());

        buffer.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    /// Find trace by trace ID
    pub fn find_by_trace_id(&self, trace_id: u64) -> Option<DecisionTrace> {
        let buffer = self.buffer.lock();
        buffer.iter()
            .find(|t| t.trace_id == trace_id)
            .cloned()
    }

    /// Find all traces matching a predicate
    pub fn find_all<F>(&self, predicate: F) -> Vec<DecisionTrace>
    where
        F: Fn(&DecisionTrace) -> bool,
    {
        let buffer = self.buffer.lock();
        buffer.iter()
            .filter(|t| predicate(t))
            .cloned()
            .collect()
    }

    /// Get total number of traces stored
    pub fn len(&self) -> usize {
        self.buffer.lock().len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().is_empty()
    }

    /// Clear all traces
    pub fn clear(&self) {
        let mut buffer = self.buffer.lock();
        buffer.clear();
        *self.next_index.lock() = 0;
    }

    /// Drain all traces (returns and clears)
    pub fn drain_all(&self) -> Vec<DecisionTrace> {
        let mut buffer = self.buffer.lock();
        let traces = buffer.clone();
        buffer.clear();
        *self.next_index.lock() = 0;
        traces
    }

    /// Get buffer statistics
    pub fn stats(&self) -> BufferStats {
        let buffer = self.buffer.lock();

        let total = buffer.len();
        let executed = buffer.iter().filter(|t| t.was_executed).count();
        let overridden = buffer.iter().filter(|t| t.was_overridden).count();
        let high_confidence = buffer.iter().filter(|t| t.is_high_confidence()).count();

        BufferStats {
            total,
            executed,
            overridden,
            high_confidence,
        }
    }
}

/// Buffer statistics
#[derive(Debug, Clone, Copy)]
pub struct BufferStats {
    pub total: usize,
    pub executed: usize,
    pub overridden: usize,
    pub high_confidence: usize,
}

/// Global trace buffer instance
pub static TRACE_BUFFER: TraceBuffer = TraceBuffer::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_buffer_record() {
        let buffer = TraceBuffer::with_capacity(10);

        let trace = DecisionTrace::new(1);
        buffer.record(trace);

        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_trace_buffer_overflow() {
        let buffer = TraceBuffer::with_capacity(3);

        for i in 0..5 {
            buffer.record(DecisionTrace::new(i));
        }

        // Should only have 3 traces (capacity)
        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_find_by_trace_id() {
        let buffer = TraceBuffer::with_capacity(10);

        buffer.record(DecisionTrace::new(123));
        buffer.record(DecisionTrace::new(456));

        let found = buffer.find_by_trace_id(123);
        assert!(found.is_some());
        assert_eq!(found.unwrap().trace_id, 123);

        let not_found = buffer.find_by_trace_id(999);
        assert!(not_found.is_none());
    }
}
