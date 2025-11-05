/// Buddy allocator for physical page management
///
/// Implements a buddy system allocator with free lists per order.
/// Supports allocation of power-of-2 sized page blocks.

use super::page::{Page, Pfn, PhysAddr, pa_to_pfn, pfn_to_pa, page_align_down, page_align_up, PAGE_SIZE, PageFlags};
use crate::lib::error::KernelError;
use alloc::vec::Vec;
use spin::Mutex;

/// Maximum order (2^10 = 4 MiB max allocation)
pub const MAX_ORDER: u8 = 10;

/// Buddy allocator state
pub struct BuddyAllocator {
    /// Free lists per order (order 0 = 4KB, order 1 = 8KB, etc.)
    free_lists: [Vec<Pfn>; MAX_ORDER as usize + 1],
    /// Page metadata array (indexed by PFN)
    pages: Vec<Page>,
    /// Base PFN
    base_pfn: Pfn,
    /// Number of pages
    num_pages: usize,
    /// Statistics
    stats: AllocStats,
}

#[derive(Debug, Clone, Copy)]
pub struct AllocStats {
    pub total_pages: usize,
    pub free_pages: usize,
    pub allocated_pages: usize,
}

impl BuddyAllocator {
    /// Create a new empty buddy allocator
    fn new() -> Self {
        Self {
            free_lists: Default::default(),
            pages: Vec::new(),
            base_pfn: 0,
            num_pages: 0,
            stats: AllocStats {
                total_pages: 0,
                free_pages: 0,
                allocated_pages: 0,
            },
        }
    }

    /// Initialize from RAM ranges
    ///
    /// ram_ranges: list of (physical_address, size) tuples
    pub fn init(&mut self, ram_ranges: &[(PhysAddr, usize)]) -> Result<(), KernelError> {
        if ram_ranges.is_empty() {
            return Err(KernelError::InvalidArgument);
        }

        // Calculate total memory and determine base PFN
        let mut min_addr = u64::MAX;
        let mut max_addr = 0u64;

        for &(addr, size) in ram_ranges {
            let start = page_align_down(addr);
            let end = page_align_down(addr + size as u64);
            min_addr = min_addr.min(start);
            max_addr = max_addr.max(end);
        }

        self.base_pfn = pa_to_pfn(min_addr);
        let max_pfn = pa_to_pfn(max_addr);
        self.num_pages = max_pfn - self.base_pfn;

        crate::info!(
            "Buddy: initializing {} pages ({} MB) from PFN {}-{}",
            self.num_pages,
            self.num_pages * PAGE_SIZE / (1024 * 1024),
            self.base_pfn,
            max_pfn
        );

        // Allocate page metadata array
        self.pages = Vec::with_capacity(self.num_pages);
        for _ in 0..self.num_pages {
            self.pages.push(Page::new());
        }

        // Initialize free lists
        for list in &mut self.free_lists {
            *list = Vec::new();
        }

        // Add all RAM ranges to free lists
        for &(addr, size) in ram_ranges {
            let start_pfn = pa_to_pfn(page_align_up(addr));
            let end_pfn = pa_to_pfn(page_align_down(addr + size as u64));
            let num_pages = end_pfn.saturating_sub(start_pfn);

            crate::debug!(
                "Buddy: adding range PFN {}-{} ({} pages)",
                start_pfn, end_pfn, num_pages
            );

            // Add pages to free list
            self.add_free_range(start_pfn, num_pages);
        }

        self.stats.total_pages = self.num_pages;
        self.update_stats();

        crate::info!(
            "Buddy: initialized with {} free pages",
            self.stats.free_pages
        );

        Ok(())
    }

    /// Add a range of pages to the free lists
    fn add_free_range(&mut self, start_pfn: Pfn, num_pages: usize) {
        let mut pfn = start_pfn;
        let mut remaining = num_pages;

        while remaining > 0 {
            // Find largest power-of-2 block that fits
            let mut order = 0u8;
            while order < MAX_ORDER {
                let block_pages = 1 << (order + 1);
                if block_pages > remaining || (pfn & (block_pages - 1)) != 0 {
                    break;
                }
                order += 1;
            }

            // Add block to appropriate free list
            let block_pages = 1 << order;
            self.free_block(pfn, order);

            pfn += block_pages;
            remaining -= block_pages;
        }
    }

    /// Allocate a single page (order 0)
    pub fn alloc_page(&mut self) -> Option<PhysAddr> {
        self.alloc_pages(0)
    }

    /// Allocate 2^order pages
    pub fn alloc_pages(&mut self, order: u8) -> Option<PhysAddr> {
        if order > MAX_ORDER {
            return None;
        }

        // Find a free block of the requested order or larger
        let mut current_order = order;
        while current_order <= MAX_ORDER {
            if !self.free_lists[current_order as usize].is_empty() {
                // Found a free block
                let pfn = self.free_lists[current_order as usize].pop().unwrap();

                // Split larger blocks if necessary
                while current_order > order {
                    current_order -= 1;
                    let buddy_pfn = pfn + (1 << current_order);
                    self.free_block(buddy_pfn, current_order);
                }

                // Mark page as allocated
                if let Some(page) = self.get_page_mut(pfn) {
                    page.set_refcount(1);
                    page.order = order;
                    page.flags.remove(PageFlags::BUDDY);
                }

                self.stats.allocated_pages += 1 << order;
                self.stats.free_pages -= 1 << order;

                let pa = pfn_to_pa(pfn);
                crate::debug!("Buddy: allocated {} pages at PFN {} (PA {:#x})", 1 << order, pfn, pa);

                // Zero the allocated pages
                self.zero_pages(pa, order);

                return Some(pa);
            }
            current_order += 1;
        }

        // Out of memory
        crate::warn!("Buddy: allocation failed for order {}", order);
        None
    }

    /// Free a single page
    pub fn free_page(&mut self, pa: PhysAddr) {
        self.free_pages(pa, 0);
    }

    /// Free 2^order pages
    pub fn free_pages(&mut self, pa: PhysAddr, order: u8) {
        let pfn = pa_to_pfn(pa);

        crate::debug!("Buddy: freeing {} pages at PFN {} (PA {:#x})", 1 << order, pfn, pa);

        // Decrement refcount
        if let Some(page) = self.get_page_mut(pfn) {
            let refcount = page.put();
            if refcount > 0 {
                // Still has references, don't free yet
                return;
            }
        }

        // Free and try to coalesce
        self.free_and_coalesce(pfn, order);

        self.stats.allocated_pages -= 1 << order;
        self.stats.free_pages += 1 << order;
    }

    /// Free a block and add to free list (no coalescing)
    fn free_block(&mut self, pfn: Pfn, order: u8) {
        if let Some(page) = self.get_page_mut(pfn) {
            page.set_refcount(0);
            page.order = order;
            page.flags.insert(PageFlags::BUDDY);
        }
        self.free_lists[order as usize].push(pfn);
    }

    /// Free a block and try to coalesce with buddy
    fn free_and_coalesce(&mut self, mut pfn: Pfn, mut order: u8) {
        while order < MAX_ORDER {
            // Calculate buddy PFN
            let block_size = 1 << order;
            let buddy_pfn = pfn ^ block_size;

            // Check if buddy is free and same order
            let buddy_free = if let Some(buddy_page) = self.get_page(buddy_pfn) {
                buddy_page.is_free()
                    && buddy_page.order == order
                    && buddy_page.flags.contains(PageFlags::BUDDY)
            } else {
                false
            };

            if !buddy_free {
                // Can't coalesce, add to free list
                self.free_block(pfn, order);
                return;
            }

            // Remove buddy from free list
            if let Some(pos) = self.free_lists[order as usize].iter().position(|&p| p == buddy_pfn) {
                self.free_lists[order as usize].swap_remove(pos);
            }

            // Coalesce: use lower PFN as the merged block
            pfn = pfn.min(buddy_pfn);
            order += 1;
        }

        // Reached max order, add to free list
        self.free_block(pfn, order);
    }

    /// Zero allocated pages for security
    fn zero_pages(&self, pa: PhysAddr, order: u8) {
        let num_pages = 1 << order;
        let size = num_pages * PAGE_SIZE;
        unsafe {
            let ptr = pa as *mut u8;
            core::ptr::write_bytes(ptr, 0, size);
        }
    }

    /// Get page metadata (relative to base_pfn)
    fn get_page(&self, pfn: Pfn) -> Option<&Page> {
        if pfn < self.base_pfn {
            return None;
        }
        let idx = pfn - self.base_pfn;
        self.pages.get(idx)
    }

    /// Get mutable page metadata
    fn get_page_mut(&mut self, pfn: Pfn) -> Option<&mut Page> {
        if pfn < self.base_pfn {
            return None;
        }
        let idx = pfn - self.base_pfn;
        self.pages.get_mut(idx)
    }

    /// Update statistics
    fn update_stats(&mut self) {
        let mut free = 0;
        for (order, list) in self.free_lists.iter().enumerate() {
            free += list.len() * (1 << order);
        }
        self.stats.free_pages = free;
        self.stats.allocated_pages = self.stats.total_pages - free;
    }

    /// Get allocation statistics
    pub fn stats(&mut self) -> AllocStats {
        self.update_stats();
        self.stats
    }
}

/// Global buddy allocator
static BUDDY: Mutex<Option<BuddyAllocator>> = Mutex::new(None);

/// Initialize the buddy allocator with RAM ranges
pub fn init_buddy(ram_ranges: &[(PhysAddr, usize)]) -> Result<(), KernelError> {
    let mut buddy = BUDDY.lock();
    let mut allocator = BuddyAllocator::new();
    allocator.init(ram_ranges)?;
    *buddy = Some(allocator);
    Ok(())
}

/// Allocate a single page
pub fn alloc_page() -> Option<PhysAddr> {
    let mut buddy = BUDDY.lock();
    if let Some(ref mut alloc) = *buddy {
        alloc.alloc_page()
    } else {
        None
    }
}

/// Allocate 2^order pages
pub fn alloc_pages(order: u8) -> Option<PhysAddr> {
    let mut buddy = BUDDY.lock();
    if let Some(ref mut alloc) = *buddy {
        alloc.alloc_pages(order)
    } else {
        None
    }
}

/// Free a single page
pub fn free_page(pa: PhysAddr) {
    let mut buddy = BUDDY.lock();
    if let Some(ref mut alloc) = *buddy {
        alloc.free_page(pa);
    }
}

/// Free 2^order pages
pub fn free_pages(pa: PhysAddr, order: u8) {
    let mut buddy = BUDDY.lock();
    if let Some(ref mut alloc) = *buddy {
        alloc.free_pages(pa, order);
    }
}

/// Get allocation statistics
pub fn get_stats() -> Option<AllocStats> {
    let mut buddy = BUDDY.lock();
    if let Some(ref mut alloc) = *buddy {
        Some(alloc.stats())
    } else {
        None
    }
}
