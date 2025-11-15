//! # VirtIO Virtqueue Implementation
//!
//! This module implements the VirtIO split virtqueue format as specified in
//! VirtIO 1.0+. Virtqueues are the fundamental data structure for communication
//! between the guest driver and the VirtIO device.
//!
//! ## Virtqueue Structure
//!
//! A virtqueue consists of three parts:
//!
//! ```text
//! ┌─────────────────────────────────┐
//! │     Descriptor Table            │  Array of buffer descriptors
//! │  (16 bytes per descriptor)      │
//! └─────────────────────────────────┘
//! ┌─────────────────────────────────┐
//! │     Available Ring              │  Driver → Device notifications
//! │  flags, idx, ring[], used_event │
//! └─────────────────────────────────┘
//! ┌─────────────────────────────────┐
//! │     Used Ring                   │  Device → Driver completions
//! │  flags, idx, ring[], avail_event│
//! └─────────────────────────────────┘
//! ```
//!
//! ## Memory Layout
//!
//! The three parts must be physically contiguous and properly aligned:
//! - Descriptor table: 16-byte alignment
//! - Available ring: 2-byte alignment
//! - Used ring: 4-byte alignment
//!
//! Total size for queue size N:
//! - Descriptor table: 16 * N bytes
//! - Available ring: 6 + 2 * N bytes
//! - Used ring: 6 + 8 * N bytes
//!
//! ## Descriptor Table
//!
//! Each descriptor describes a buffer:
//!
//! ```text
//! struct VirtqDesc {
//!     addr: u64,    // Physical address
//!     len: u32,     // Buffer length
//!     flags: u16,   // NEXT, WRITE, INDIRECT
//!     next: u16,    // Next descriptor (if NEXT flag set)
//! }
//! ```
//!
//! **Flags:**
//! - `VIRTQ_DESC_F_NEXT` (1): Descriptor is part of a chain
//! - `VIRTQ_DESC_F_WRITE` (2): Buffer is device-writable (not read-only)
//! - `VIRTQ_DESC_F_INDIRECT` (4): Buffer contains descriptor table
//!
//! ## Available Ring
//!
//! Driver uses this to tell device which descriptor chains are available:
//!
//! ```text
//! struct VirtqAvail {
//!     flags: u16,          // VIRTQ_AVAIL_F_NO_INTERRUPT
//!     idx: u16,            // Next slot to use (wraps)
//!     ring: [u16; N],      // Descriptor head indices
//!     used_event: u16,     // Used for event notification (optional)
//! }
//! ```
//!
//! ## Used Ring
//!
//! Device uses this to return completed buffers:
//!
//! ```text
//! struct VirtqUsed {
//!     flags: u16,                // VIRTQ_USED_F_NO_NOTIFY
//!     idx: u16,                  // Next slot device will use
//!     ring: [VirtqUsedElem; N],  // Used elements
//!     avail_event: u16,          // Available event index (optional)
//! }
//!
//! struct VirtqUsedElem {
//!     id: u32,     // Descriptor chain head
//!     len: u32,    // Bytes written
//! }
//! ```
//!
//! ## Operation Flow
//!
//! 1. **Driver allocates buffers** and fills descriptor table
//! 2. **Driver adds descriptor chain head to available ring**
//! 3. **Driver increments avail.idx** (with memory barrier)
//! 4. **Driver notifies device** (doorbell write)
//! 5. **Device processes descriptors**
//! 6. **Device adds used element to used ring**
//! 7. **Device increments used.idx** (with memory barrier)
//! 8. **Device sends interrupt** (if not suppressed)
//! 9. **Driver processes used ring**
//! 10. **Driver reclaims descriptors**

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{fence, Ordering};
use x86_64::PhysAddr;
use alloc::vec::Vec;

/// Maximum queue size (VirtIO spec allows up to 32768, but we use smaller for simplicity)
pub const MAX_QUEUE_SIZE: u16 = 256;

/// Descriptor flags
pub mod desc_flags {
    /// Descriptor continues via next field
    pub const NEXT: u16 = 1;
    /// Buffer is write-only (device writes, driver reads)
    pub const WRITE: u16 = 2;
    /// Buffer contains list of buffer descriptors
    pub const INDIRECT: u16 = 4;
}

/// Available ring flags
pub mod avail_flags {
    /// Suppress interrupts
    pub const NO_INTERRUPT: u16 = 1;
}

/// Used ring flags
pub mod used_flags {
    /// Device should not send notifications
    pub const NO_NOTIFY: u16 = 1;
}

/// Virtqueue descriptor (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtqDesc {
    /// Physical address of buffer
    pub addr: u64,
    /// Length of buffer
    pub len: u32,
    /// Descriptor flags (NEXT, WRITE, INDIRECT)
    pub flags: u16,
    /// Next descriptor in chain (if NEXT flag set)
    pub next: u16,
}

/// Virtqueue available ring header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtqAvailHdr {
    /// Flags (NO_INTERRUPT)
    pub flags: u16,
    /// Index where next descriptor head will be placed
    pub idx: u16,
}

/// Virtqueue used element (8 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtqUsedElem {
    /// Descriptor chain head index
    pub id: u32,
    /// Total bytes written to buffer
    pub len: u32,
}

/// Virtqueue used ring header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtqUsedHdr {
    /// Flags (NO_NOTIFY)
    pub flags: u16,
    /// Index where next used element will be placed
    pub idx: u16,
}

/// Virtqueue split-ring implementation
pub struct Virtqueue {
    /// Queue size (number of descriptors)
    queue_size: u16,

    /// Physical base address of entire virtqueue memory
    phys_addr: PhysAddr,

    /// Descriptor table (queue_size entries)
    desc_table: *mut VirtqDesc,

    /// Available ring header
    avail_hdr: *mut VirtqAvailHdr,
    /// Available ring array
    avail_ring: *mut u16,
    /// Available ring used_event field (optional, at end of available ring)
    avail_used_event: *mut u16,

    /// Used ring header
    used_hdr: *mut VirtqUsedHdr,
    /// Used ring array
    used_ring: *mut VirtqUsedElem,
    /// Used ring avail_event field (optional, at end of used ring)
    used_avail_event: *mut u16,

    /// Free descriptor list (indices of free descriptors)
    free_desc: Vec<u16>,

    /// Last seen used index (for tracking completions)
    last_used_idx: u16,

    /// Last available index we've published
    last_avail_idx: u16,
}

impl Virtqueue {
    /// Calculate required memory size for a virtqueue
    ///
    /// Returns (total_size, desc_offset, avail_offset, used_offset)
    pub fn calculate_size(queue_size: u16) -> (usize, usize, usize, usize) {
        let desc_size = 16 * queue_size as usize;
        let avail_size = 6 + 2 * queue_size as usize;
        let used_size = 6 + 8 * queue_size as usize;

        let desc_offset = 0;
        let avail_offset = desc_size;

        // Used ring must be page-aligned (4096 bytes)
        let used_offset = ((avail_offset + avail_size + 4095) / 4096) * 4096;

        let total_size = used_offset + used_size;

        (total_size, desc_offset, avail_offset, used_offset)
    }

    /// Create a new virtqueue
    ///
    /// # Arguments
    /// * `queue_size` - Number of descriptors in the queue (must be power of 2)
    /// * `phys_addr` - Physical address of allocated memory
    /// * `virt_addr` - Virtual address of allocated memory
    ///
    /// # Safety
    /// - Memory at virt_addr must be valid for the entire virtqueue size
    /// - Memory must be physically contiguous
    /// - Memory must be zeroed
    pub unsafe fn new(queue_size: u16, phys_addr: PhysAddr, virt_addr: usize) -> Self {
        assert!(queue_size.is_power_of_two(), "Queue size must be power of 2");
        assert!(queue_size <= MAX_QUEUE_SIZE, "Queue size too large");

        let (_, desc_offset, avail_offset, used_offset) = Self::calculate_size(queue_size);

        // Calculate pointers
        let desc_table = (virt_addr + desc_offset) as *mut VirtqDesc;
        let avail_hdr = (virt_addr + avail_offset) as *mut VirtqAvailHdr;
        let avail_ring = (virt_addr + avail_offset + 4) as *mut u16;
        let avail_used_event = (virt_addr + avail_offset + 4 + 2 * queue_size as usize) as *mut u16;

        let used_hdr = (virt_addr + used_offset) as *mut VirtqUsedHdr;
        let used_ring = (virt_addr + used_offset + 4) as *mut VirtqUsedElem;
        let used_avail_event = (virt_addr + used_offset + 4 + 8 * queue_size as usize) as *mut u16;

        // Initialize free descriptor list
        let free_desc: Vec<u16> = (0..queue_size).collect();

        // Zero out the queue memory (should already be zeroed, but be safe)
        core::ptr::write_bytes(virt_addr as *mut u8, 0, Self::calculate_size(queue_size).0);

        Self {
            queue_size,
            phys_addr,
            desc_table,
            avail_hdr,
            avail_ring,
            avail_used_event,
            used_hdr,
            used_ring,
            used_avail_event,
            free_desc,
            last_used_idx: 0,
            last_avail_idx: 0,
        }
    }

    /// Get physical addresses for queue regions (for device configuration)
    pub fn get_physical_addresses(&self) -> (PhysAddr, PhysAddr, PhysAddr) {
        let (_, desc_offset, avail_offset, used_offset) = Self::calculate_size(self.queue_size);

        let desc_phys = PhysAddr::new(self.phys_addr.as_u64() + desc_offset as u64);
        let avail_phys = PhysAddr::new(self.phys_addr.as_u64() + avail_offset as u64);
        let used_phys = PhysAddr::new(self.phys_addr.as_u64() + used_offset as u64);

        (desc_phys, avail_phys, used_phys)
    }

    /// Allocate a descriptor from the free list
    fn alloc_desc(&mut self) -> Option<u16> {
        self.free_desc.pop()
    }

    /// Free a descriptor back to the free list
    fn free_desc(&mut self, index: u16) {
        self.free_desc.push(index);
    }

    /// Add a buffer chain to the virtqueue
    ///
    /// # Arguments
    /// * `buffers` - List of (physical_address, length, writable) tuples
    ///
    /// # Returns
    /// Descriptor head index, or None if queue is full
    pub fn add_buffer_chain(&mut self, buffers: &[(PhysAddr, u32, bool)]) -> Option<u16> {
        if buffers.is_empty() || buffers.len() > self.free_desc.len() {
            return None;
        }

        // Allocate descriptors for the chain
        let mut desc_indices = Vec::with_capacity(buffers.len());
        for _ in 0..buffers.len() {
            if let Some(idx) = self.alloc_desc() {
                desc_indices.push(idx);
            } else {
                // Free already allocated descriptors
                for &idx in &desc_indices {
                    self.free_desc(idx);
                }
                return None;
            }
        }

        // Fill in descriptors
        for (i, &(phys_addr, len, writable)) in buffers.iter().enumerate() {
            let desc_idx = desc_indices[i];

            unsafe {
                let desc = &mut *self.desc_table.add(desc_idx as usize);
                desc.addr = phys_addr.as_u64();
                desc.len = len;
                desc.flags = if writable { desc_flags::WRITE } else { 0 };

                // Link to next descriptor if not last
                if i < buffers.len() - 1 {
                    desc.flags |= desc_flags::NEXT;
                    desc.next = desc_indices[i + 1];
                } else {
                    desc.next = 0;
                }
            }
        }

        // Add chain head to available ring
        let head = desc_indices[0];
        self.add_to_avail_ring(head);

        Some(head)
    }

    /// Add a single buffer to the virtqueue
    ///
    /// Convenience wrapper for single-buffer operations.
    pub fn add_buffer(&mut self, phys_addr: PhysAddr, len: u32, writable: bool) -> Option<u16> {
        self.add_buffer_chain(&[(phys_addr, len, writable)])
    }

    /// Add descriptor head to available ring
    fn add_to_avail_ring(&mut self, desc_head: u16) {
        unsafe {
            // Get current available index
            let avail_idx = read_volatile(&(*self.avail_hdr).idx);

            // Add descriptor to ring
            let ring_idx = (avail_idx % self.queue_size) as usize;
            write_volatile(self.avail_ring.add(ring_idx), desc_head);

            // Memory barrier before updating index
            fence(Ordering::Release);

            // Increment available index
            write_volatile(&mut (*self.avail_hdr).idx, avail_idx.wrapping_add(1));

            self.last_avail_idx = avail_idx.wrapping_add(1);
        }
    }

    /// Check if there are completed buffers in the used ring
    pub fn has_used(&self) -> bool {
        unsafe {
            let used_idx = read_volatile(&(*self.used_hdr).idx);
            used_idx != self.last_used_idx
        }
    }

    /// Get next completed buffer from used ring
    ///
    /// Returns (descriptor_head, bytes_written) or None if no completions
    pub fn get_used(&mut self) -> Option<(u16, u32)> {
        unsafe {
            // Memory barrier before reading used ring
            fence(Ordering::Acquire);

            let used_idx = read_volatile(&(*self.used_hdr).idx);

            if used_idx == self.last_used_idx {
                return None;
            }

            // Get used element
            let ring_idx = (self.last_used_idx % self.queue_size) as usize;
            let used_elem = read_volatile(self.used_ring.add(ring_idx));

            self.last_used_idx = self.last_used_idx.wrapping_add(1);

            Some((used_elem.id as u16, used_elem.len))
        }
    }

    /// Reclaim a descriptor chain
    ///
    /// Frees all descriptors in the chain back to the free list.
    pub fn reclaim_chain(&mut self, head: u16) {
        unsafe {
            let mut current = head;
            loop {
                let desc = &*self.desc_table.add(current as usize);
                let has_next = (desc.flags & desc_flags::NEXT) != 0;
                let next = desc.next;

                self.free_desc(current);

                if !has_next {
                    break;
                }
                current = next;
            }
        }
    }

    /// Disable interrupts for this queue
    pub fn disable_interrupts(&mut self) {
        unsafe {
            let mut flags = read_volatile(&(*self.avail_hdr).flags);
            flags |= avail_flags::NO_INTERRUPT;
            write_volatile(&mut (*self.avail_hdr).flags, flags);
        }
    }

    /// Enable interrupts for this queue
    pub fn enable_interrupts(&mut self) {
        unsafe {
            let mut flags = read_volatile(&(*self.avail_hdr).flags);
            flags &= !avail_flags::NO_INTERRUPT;
            write_volatile(&mut (*self.avail_hdr).flags, flags);
        }
    }

    /// Get queue size
    pub fn size(&self) -> u16 {
        self.queue_size
    }

    /// Get number of free descriptors
    pub fn free_count(&self) -> usize {
        self.free_desc.len()
    }
}

unsafe impl Send for Virtqueue {}
unsafe impl Sync for Virtqueue {}

/// Allocate physically contiguous memory for a virtqueue
///
/// # Arguments
/// * `queue_size` - Number of descriptors
///
/// # Returns
/// (physical_address, virtual_address, size_bytes)
///
/// # Safety
/// Caller must ensure the returned memory is properly freed
pub unsafe fn alloc_virtqueue_memory(queue_size: u16) -> Result<(PhysAddr, usize, usize), &'static str> {
    let (total_size, _, _, _) = Virtqueue::calculate_size(queue_size);

    // Round up to page size
    let pages_needed = (total_size + 4095) / 4096;
    let alloc_size = pages_needed * 4096;

    // Allocate physical pages
    // For now, we use a simple allocation from the buddy allocator
    let phys_addr = crate::mm::alloc_pages(pages_needed)
        .ok_or("Failed to allocate virtqueue memory")?;

    // Map to virtual address using direct physical mapping
    const PHYS_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;
    let virt_addr = (phys_addr + PHYS_OFFSET) as usize;

    // Zero the memory
    core::ptr::write_bytes(virt_addr as *mut u8, 0, alloc_size);

    Ok((PhysAddr::new(phys_addr), virt_addr, alloc_size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtqueue_size_calculation() {
        let (total, desc, avail, used) = Virtqueue::calculate_size(256);
        assert_eq!(desc, 0);
        assert_eq!(avail, 16 * 256);
        assert!(used >= avail + 6 + 2 * 256);
        assert!(total >= used + 6 + 8 * 256);
    }

    #[test]
    fn test_virtqueue_power_of_two() {
        for &size in &[1, 2, 4, 8, 16, 32, 64, 128, 256] {
            let (total, _, _, _) = Virtqueue::calculate_size(size);
            assert!(total > 0);
        }
    }
}
