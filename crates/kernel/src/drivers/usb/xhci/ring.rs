//! XHCI Ring Management
//!
//! Manages circular buffers of TRBs used for communication with the XHCI controller.
//! Includes Command Rings, Event Rings, and Transfer Rings.

use super::trb::Trb;
use crate::drivers::{DriverError, DriverResult};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Transfer Ring (used for endpoints and command ring)
pub struct Ring {
    /// TRB buffer (DMA-capable memory)
    trbs: Vec<Trb>,

    /// Physical address of TRB buffer
    phys_addr: usize,

    /// Enqueue pointer (index)
    enqueue: AtomicUsize,

    /// Dequeue pointer (index)
    dequeue: AtomicUsize,

    /// Producer Cycle State
    pcs: AtomicBool,

    /// Ring size (number of TRBs)
    size: usize,
}

impl Ring {
    /// Create new ring
    ///
    /// # Arguments
    /// * `size` - Number of TRBs in ring (must be power of 2, max 256)
    pub fn new(size: usize) -> DriverResult<Self> {
        if size == 0 || size > 256 || !size.is_power_of_two() {
            return Err(DriverError::InvalidParameter);
        }

        // Allocate TRB buffer
        let mut trbs = alloc::vec![Trb::new(); size];

        // Get physical address (for now, assume identity mapping)
        // In a real implementation, this would use proper DMA allocation
        let phys_addr = trbs.as_ptr() as usize;

        // Initialize all TRBs with cycle bit = 0
        for trb in &mut trbs {
            trb.set_cycle(false);
        }

        Ok(Self {
            trbs,
            phys_addr,
            enqueue: AtomicUsize::new(0),
            dequeue: AtomicUsize::new(0),
            pcs: AtomicBool::new(true),  // Start with PCS = 1
            size,
        })
    }

    /// Get physical address of ring
    pub fn physical_addr(&self) -> usize {
        self.phys_addr
    }

    /// Get current enqueue pointer
    pub fn enqueue_ptr(&self) -> usize {
        self.enqueue.load(Ordering::Acquire)
    }

    /// Get current dequeue pointer
    pub fn dequeue_ptr(&self) -> usize {
        self.dequeue.load(Ordering::Acquire)
    }

    /// Enqueue a TRB
    pub fn enqueue_trb(&mut self, mut trb: Trb) -> DriverResult<()> {
        let enq = self.enqueue.load(Ordering::Acquire);

        // Check if ring is full
        let next_enq = (enq + 1) % self.size;
        if next_enq == self.dequeue.load(Ordering::Acquire) {
            return Err(DriverError::Busy);
        }

        // Set cycle bit to current PCS
        trb.set_cycle(self.pcs.load(Ordering::Acquire));

        // Write TRB
        self.trbs[enq] = trb;

        // Update enqueue pointer
        self.enqueue.store(next_enq, Ordering::Release);

        // If we wrapped around, toggle PCS
        if next_enq == 0 {
            let pcs = self.pcs.load(Ordering::Acquire);
            self.pcs.store(!pcs, Ordering::Release);
        }

        Ok(())
    }

    /// Dequeue a TRB
    pub fn dequeue_trb(&mut self) -> Option<Trb> {
        let deq = self.dequeue.load(Ordering::Acquire);
        let enq = self.enqueue.load(Ordering::Acquire);

        // Check if ring is empty
        if deq == enq {
            return None;
        }

        // Read TRB
        let trb = self.trbs[deq];

        // Update dequeue pointer
        let next_deq = (deq + 1) % self.size;
        self.dequeue.store(next_deq, Ordering::Release);

        Some(trb)
    }

    /// Check if ring is empty
    pub fn is_empty(&self) -> bool {
        self.enqueue.load(Ordering::Acquire) == self.dequeue.load(Ordering::Acquire)
    }

    /// Check if ring is full
    pub fn is_full(&self) -> bool {
        let enq = self.enqueue.load(Ordering::Acquire);
        let deq = self.dequeue.load(Ordering::Acquire);
        ((enq + 1) % self.size) == deq
    }

    /// Get ring size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Reset ring
    pub fn reset(&mut self) {
        self.enqueue.store(0, Ordering::Release);
        self.dequeue.store(0, Ordering::Release);
        self.pcs.store(true, Ordering::Release);

        // Clear all TRBs
        for trb in &mut self.trbs {
            *trb = Trb::new();
            trb.set_cycle(false);
        }
    }
}

/// Event Ring Segment Table Entry
#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
struct EventRingSegmentTableEntry {
    /// Ring Segment Base Address (64-bit)
    base_addr: u64,

    /// Ring Segment Size (number of TRBs)
    size: u16,

    /// Reserved
    _reserved: [u8; 6],
}

impl EventRingSegmentTableEntry {
    fn new(base_addr: u64, size: u16) -> Self {
        Self {
            base_addr,
            size,
            _reserved: [0; 6],
        }
    }
}

/// Event Ring
pub struct EventRing {
    /// Event Ring Segment Table (ERST)
    segment_table: Vec<EventRingSegmentTableEntry>,

    /// Event TRB buffer
    trbs: Vec<Trb>,

    /// Physical address of segment table
    erst_phys_addr: usize,

    /// Physical address of TRB buffer
    trbs_phys_addr: usize,

    /// Dequeue pointer (index)
    dequeue: AtomicUsize,

    /// Consumer Cycle State
    ccs: AtomicBool,

    /// Ring size (number of TRBs)
    size: usize,
}

impl EventRing {
    /// Create new event ring
    ///
    /// # Arguments
    /// * `size` - Number of TRBs in ring
    pub fn new(size: usize) -> DriverResult<Self> {
        if size == 0 || size > 4096 {
            return Err(DriverError::InvalidParameter);
        }

        // Allocate TRB buffer
        let trbs = alloc::vec![Trb::new(); size];
        let trbs_phys_addr = trbs.as_ptr() as usize;

        // Create segment table with one entry
        let segment_table = alloc::vec![
            EventRingSegmentTableEntry::new(trbs_phys_addr as u64, size as u16)
        ];
        let erst_phys_addr = segment_table.as_ptr() as usize;

        Ok(Self {
            segment_table,
            trbs,
            erst_phys_addr,
            trbs_phys_addr,
            dequeue: AtomicUsize::new(0),
            ccs: AtomicBool::new(true),  // Start with CCS = 1
            size,
        })
    }

    /// Get physical address of event ring (ERST base)
    pub fn physical_addr(&self) -> usize {
        self.erst_phys_addr
    }

    /// Get physical address of TRB buffer
    pub fn trb_buffer_addr(&self) -> usize {
        self.trbs_phys_addr
    }

    /// Get current dequeue pointer
    pub fn dequeue_ptr(&self) -> usize {
        self.dequeue.load(Ordering::Acquire)
    }

    /// Dequeue an event TRB
    pub fn dequeue_event(&mut self) -> Option<Trb> {
        let deq = self.dequeue.load(Ordering::Acquire);
        let trb = self.trbs[deq];

        // Check if TRB is ready (cycle bit matches CCS)
        if trb.cycle() != self.ccs.load(Ordering::Acquire) {
            return None;  // No event available
        }

        // Update dequeue pointer
        let next_deq = (deq + 1) % self.size;
        self.dequeue.store(next_deq, Ordering::Release);

        // If we wrapped around, toggle CCS
        if next_deq == 0 {
            let ccs = self.ccs.load(Ordering::Acquire);
            self.ccs.store(!ccs, Ordering::Release);
        }

        Some(trb)
    }

    /// Peek at next event without dequeueing
    pub fn peek_event(&self) -> Option<Trb> {
        let deq = self.dequeue.load(Ordering::Acquire);
        let trb = self.trbs[deq];

        // Check if TRB is ready
        if trb.cycle() != self.ccs.load(Ordering::Acquire) {
            return None;
        }

        Some(trb)
    }

    /// Check if event ring has events
    pub fn has_events(&self) -> bool {
        self.peek_event().is_some()
    }

    /// Get ring size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get segment table size
    pub fn segment_table_size(&self) -> usize {
        self.segment_table.len()
    }

    /// Reset event ring
    pub fn reset(&mut self) {
        self.dequeue.store(0, Ordering::Release);
        self.ccs.store(true, Ordering::Release);
    }
}

/// Transfer Ring for an endpoint
pub struct TransferRing {
    /// Underlying ring
    ring: Ring,

    /// Endpoint DCI (Device Context Index)
    dci: u8,
}

impl TransferRing {
    /// Create new transfer ring for endpoint
    pub fn new(dci: u8, size: usize) -> DriverResult<Self> {
        let ring = Ring::new(size)?;
        Ok(Self { ring, dci })
    }

    /// Get endpoint DCI
    pub fn dci(&self) -> u8 {
        self.dci
    }

    /// Enqueue transfer TRB
    pub fn enqueue(&mut self, trb: Trb) -> DriverResult<()> {
        self.ring.enqueue_trb(trb)
    }

    /// Get physical address
    pub fn physical_addr(&self) -> usize {
        self.ring.physical_addr()
    }

    /// Get enqueue pointer
    pub fn enqueue_ptr(&self) -> usize {
        self.ring.enqueue_ptr()
    }

    /// Reset ring
    pub fn reset(&mut self) {
        self.ring.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_creation() {
        let ring = Ring::new(16).unwrap();
        assert_eq!(ring.size(), 16);
        assert!(ring.is_empty());
        assert!(!ring.is_full());
    }

    #[test]
    fn test_ring_enqueue_dequeue() {
        let mut ring = Ring::new(16).unwrap();
        let trb = Trb::noop();

        assert!(ring.enqueue_trb(trb).is_ok());
        assert!(!ring.is_empty());

        let dequeued = ring.dequeue_trb().unwrap();
        assert_eq!(dequeued.trb_type(), trb.trb_type());
        assert!(ring.is_empty());
    }

    #[test]
    fn test_event_ring_creation() {
        let event_ring = EventRing::new(256).unwrap();
        assert_eq!(event_ring.size(), 256);
        assert_eq!(event_ring.segment_table_size(), 1);
        assert!(!event_ring.has_events());
    }
}
