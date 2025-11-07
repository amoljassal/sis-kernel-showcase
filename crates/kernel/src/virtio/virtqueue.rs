/// VirtIO virtqueue implementation
///
/// Implements split virtqueues (descriptor table, available ring, used ring)
/// for VirtIO 1.0+ specification.

use crate::lib::error::{Result, Errno};
use crate::mm;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

/// Virtqueue descriptor flags
pub const VIRTQ_DESC_F_NEXT: u16 = 1;     // Descriptor continues via next field
pub const VIRTQ_DESC_F_WRITE: u16 = 2;    // Device writes (vs reads)
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4; // Descriptor contains list of descriptors

/// Virtqueue descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtqDesc {
    /// Buffer address (guest physical)
    pub addr: u64,
    /// Buffer length
    pub len: u32,
    /// Descriptor flags
    pub flags: u16,
    /// Next descriptor index (if flags & VIRTQ_DESC_F_NEXT)
    pub next: u16,
}

/// VirtQueue implementation
pub struct VirtQueue {
    /// Queue index
    pub index: u16,
    /// Queue size (must be power of 2)
    pub size: u16,
    /// Descriptor table (size * sizeof(VirtqDesc))
    desc_table: *mut VirtqDesc,
    /// Available ring base address
    avail_base: u64,
    /// Used ring base address
    used_base: u64,
    /// Legacy contiguous region base (if allocated that way)
    region_base: u64,
    /// Next available descriptor index
    next_desc: u16,
    /// Last seen used index
    last_used_idx: u16,
    /// Shadow available index (avoid MMIO reads)
    avail_idx_shadow: u16,
    /// Free descriptor list
    free_list: alloc::vec::Vec<u16>,
}

impl VirtQueue {
    /// Create a new virtqueue
    ///
    /// # Arguments
    /// * `index` - Queue index
    /// * `size` - Queue size (must be power of 2, typically 128 or 256)
    pub fn new(index: u16, size: u16) -> Result<Self> {
        if !size.is_power_of_two() || size > 32768 {
            return Err(Errno::EINVAL);
        }

        // Calculate memory requirements
        // Descriptor table: size * 16 bytes
        // Available ring: 6 + 2*size bytes  (flags, idx, ring[size], used_event)
        // Used ring: 6 + 8*size bytes       (flags, idx, used[size], avail_event)

        let desc_size = size as usize * core::mem::size_of::<VirtqDesc>();
        let avail_size = 6 + 2 * size as usize;
        let used_size = 6 + 8 * size as usize;

        // Allocate aligned memory (4096-byte alignment for VirtIO)
        // Descriptor table
        let desc_pages = (desc_size + 4095) / 4096;
        let desc_phys = mm::alloc_pages(desc_pages.trailing_zeros().try_into().unwrap())
            .ok_or(Errno::ENOMEM)?;
        let desc_table = desc_phys as *mut VirtqDesc;

        // Available ring (must be 2-byte aligned)
        let avail_pages = (avail_size + 4095) / 4096;
        let avail_phys = mm::alloc_pages(avail_pages.trailing_zeros().try_into().unwrap())
            .ok_or(Errno::ENOMEM)?;

        // Used ring (must be 4-byte aligned)
        let used_pages = (used_size + 4095) / 4096;
        let used_phys = mm::alloc_pages(used_pages.trailing_zeros().try_into().unwrap())
            .ok_or(Errno::ENOMEM)?;

        // Zero out all allocated memory
        unsafe {
            ptr::write_bytes(desc_table as *mut u8, 0, desc_size);
            ptr::write_bytes(avail_phys as *mut u8, 0, avail_size);
            ptr::write_bytes(used_phys as *mut u8, 0, used_size);
        }

        // Initialize free descriptor list
        let free_list: alloc::vec::Vec<u16> = (0..size).collect();

        Ok(Self {
            index,
            size,
            desc_table,
            avail_base: avail_phys,
            used_base: used_phys,
            region_base: 0,
            next_desc: 0,
            last_used_idx: 0,
            avail_idx_shadow: 0,
            free_list,
        })
    }

    /// Create a legacy (0.9) virtqueue: a single contiguous region holding
    /// descriptor table, available ring and used ring. Used with MMIO version=1.
    pub fn new_contiguous(index: u16, size: u16) -> Result<Self> {
        if !size.is_power_of_two() || size > 32768 {
            return Err(Errno::EINVAL);
        }

        let desc_size = size as usize * core::mem::size_of::<VirtqDesc>();
        let avail_size = 6 + 2 * size as usize; // includes used_event
        let used_size = 6 + 8 * size as usize;  // excludes avail_event (not negotiated)

        // Place used ring at 4096-aligned offset after avail to satisfy legacy QueueAlign
        let mut offset: usize = 0;
        let desc_off = offset; offset += desc_size;
        // Align avail to 2 bytes (already true), keep simple
        let avail_off = offset; offset += avail_size;
        // Align to 4096 for used ring
        let used_off = (offset + 4095) & !4095usize; offset = used_off + used_size;
        let total = offset;

        // Allocate region as 2^order pages
        let pages = (total + 4095) / 4096;
        let mut order: u8 = 0; while (1usize << order) < pages { order += 1; }
        let region_pa = mm::alloc_pages(order).ok_or(Errno::ENOMEM)?;
        let region_va = region_pa as usize as *mut u8;

        unsafe {
            core::ptr::write_bytes(region_va, 0, (1usize << order) * 4096);
        }

        let desc_table = unsafe { region_va.add(desc_off) } as *mut VirtqDesc;
        let avail_base = region_pa + (avail_off as u64);
        let used_base = region_pa + (used_off as u64);

        // Initialize free descriptor list
        let free_list: alloc::vec::Vec<u16> = (0..size).collect();

        Ok(Self {
            index,
            size,
            desc_table,
            avail_base,
            used_base,
            region_base: region_pa,
            next_desc: 0,
            last_used_idx: 0,
            avail_idx_shadow: 0,
            free_list,
        })
    }

    /// Get descriptor table physical address
    pub fn desc_table_addr(&self) -> u64 {
        self.desc_table as u64
    }

    /// Get available ring physical address
    pub fn avail_ring_addr(&self) -> u64 {
        self.avail_base
    }

    /// Get used ring physical address
    pub fn used_ring_addr(&self) -> u64 {
        self.used_base
    }

    /// Get all queue addresses (descriptor table, available ring, used ring)
    pub fn get_addresses(&self) -> (u64, u64, u64) {
        (self.desc_table_addr(), self.avail_ring_addr(), self.used_ring_addr())
    }

    /// Get contiguous region base (legacy); 0 if not contiguous
    pub fn region_base(&self) -> u64 { self.region_base }

    /// Allocate a descriptor from the free list
    pub fn alloc_desc(&mut self) -> Option<u16> {
        self.free_list.pop()
    }

    /// Free a descriptor back to the free list
    pub fn free_desc(&mut self, index: u16) {
        if index < self.size {
            self.free_list.push(index);
        }
    }

    /// Add a buffer to the virtqueue
    ///
    /// # Arguments
    /// * `buffers` - List of (addr, len, writeable) tuples
    ///
    /// # Returns
    /// Index of the first descriptor in the chain
    pub fn add_buf(&mut self, buffers: &[(u64, u32, bool)]) -> Result<u16> {
        if buffers.is_empty() {
            return Err(Errno::EINVAL);
        }

        // Allocate descriptors
        let mut desc_indices = alloc::vec::Vec::with_capacity(buffers.len());
        for _ in 0..buffers.len() {
            let idx = self.alloc_desc().ok_or(Errno::ENOSPC)?;
            desc_indices.push(idx);
        }

        // Fill in descriptors
        let head = desc_indices[0];
        for (i, &idx) in desc_indices.iter().enumerate() {
            let (addr, len, writeable) = buffers[i];

            unsafe {
                let desc = &mut *self.desc_table.add(idx as usize);
                desc.addr = addr;
                desc.len = len;
                desc.flags = if writeable { VIRTQ_DESC_F_WRITE } else { 0 };

                // Link to next descriptor if not last
                if i + 1 < desc_indices.len() {
                    desc.flags |= VIRTQ_DESC_F_NEXT;
                    desc.next = desc_indices[i + 1];
                } else {
                    desc.next = 0;
                }
            }
        }

        // Add to available ring
        unsafe {
            let avail_idx = self.avail_idx_shadow;
            let ring_idx = avail_idx % self.size;

            // Write to available ring
            // Layout: u16 flags, u16 idx, u16 ring[size]
            let ring_ptr = (self.avail_base + 4 + 2 * ring_idx as u64) as *mut u16;
            ptr::write_volatile(ring_ptr, head);

            // Update available index
            self.avail_idx_shadow = avail_idx.wrapping_add(1);
            let idx_ptr = (self.avail_base + 2) as *mut u16;
            ptr::write_volatile(idx_ptr, self.avail_idx_shadow);
        }

        // Ensure ring updates are visible to the device before notify
        compiler_fence(Ordering::Release);

        Ok(head)
    }

    /// Check if there are used buffers ready
    pub fn has_used_buf(&self) -> bool {
        unsafe {
            let used_idx_ptr = (self.used_base + 2) as *const u16;
            let used_idx = ptr::read_volatile(used_idx_ptr);
            used_idx != self.last_used_idx
        }
    }

    /// Get the next used buffer
    ///
    /// # Returns
    /// (descriptor index, length written by device)
    pub fn get_used_buf(&mut self) -> Option<(u16, u32)> {
        if !self.has_used_buf() {
            return None;
        }

        unsafe {
            let ring_idx = self.last_used_idx % self.size;

            // Used ring layout: u16 flags, u16 idx, { u32 id, u32 len }[size]
            let elem_ptr = (self.used_base + 4 + 8 * ring_idx as u64) as *const u32;
            let id = ptr::read_volatile(elem_ptr) as u16;
            let len = ptr::read_volatile(elem_ptr.add(1));

            self.last_used_idx = self.last_used_idx.wrapping_add(1);

            // Free the descriptor chain
            let mut desc_idx = id;
            loop {
                let desc = &*self.desc_table.add(desc_idx as usize);
                let has_next = (desc.flags & VIRTQ_DESC_F_NEXT) != 0;
                let next = desc.next;

                self.free_desc(desc_idx);

                if !has_next {
                    break;
                }
                desc_idx = next;
            }

            Some((id, len))
        }
    }

    /// Notify the device (kick)
    /// This should be done after adding buffers to the available ring
    pub fn notify_needed(&self) -> bool {
        // For now, always notify (optimization: check used_event)
        true
    }

    /// Add a descriptor chain consisting of read-only and write-only slices
    /// Convenience wrapper over `add_buf` for common patterns
    pub fn add_chain(&mut self, reads: &[&[u8]], writes: &[&mut [u8]]) -> Result<u16> {
        let mut buffers: alloc::vec::Vec<(u64, u32, bool)> = alloc::vec::Vec::new();
        for s in reads {
            buffers.push((s.as_ptr() as u64, s.len() as u32, false));
        }
        // `writes` is a slice of mutable slices; iterating yields `&(&mut [u8])`.
        // Safely recover the inner `&mut [u8]` for pointer extraction.
        for s in writes {
            // SAFETY: `s` refers to a unique &mut [u8] element inside `writes`.
            // We transiently obtain the inner &mut [u8] to read its pointer/len.
            let w: &mut [u8] = unsafe { &mut *(*s as *const [u8] as *mut [u8]) };
            buffers.push((w.as_mut_ptr() as u64, w.len() as u32, true));
        }
        self.add_buf(&buffers)
    }

    /// Busy-wait for at least one used buffer to appear
    pub fn wait_for_used(&mut self) -> Result<()> {
        // Simple bounded spin-wait; replace with IRQ/notify integration later
        // Some devices (e.g., virtio-gpu) may take longer on first command.
        let mut spins = 0usize;
        while self.get_used_buf().is_none() {
            core::hint::spin_loop();
            spins = spins.wrapping_add(1);
            if spins > 50_000_000 {
                return Err(Errno::ETIMEDOUT);
            }
        }
        Ok(())
    }
}

impl Drop for VirtQueue {
    fn drop(&mut self) {
        // Free allocated pages
        // Note: In a real implementation, we'd track the original allocation sizes
        // For now, we just leak the memory as this is Phase B
    }
}

unsafe impl Send for VirtQueue {}
unsafe impl Sync for VirtQueue {}
