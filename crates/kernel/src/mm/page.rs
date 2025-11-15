/// Physical page management
///
/// Tracks metadata for each physical page frame including
/// reference counts, order, and flags.

use core::sync::atomic::{AtomicU32, Ordering};

/// Physical address type
pub type PhysAddr = u64;

/// Page frame number
pub type Pfn = usize;

/// Page size (4 KiB)
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

/// Convert physical address to page frame number
#[inline]
pub const fn pa_to_pfn(pa: PhysAddr) -> Pfn {
    (pa as usize) >> PAGE_SHIFT
}

/// Convert page frame number to physical address
#[inline]
pub const fn pfn_to_pa(pfn: Pfn) -> PhysAddr {
    (pfn << PAGE_SHIFT) as PhysAddr
}

/// Round physical address down to page boundary
#[inline]
pub const fn page_align_down(pa: PhysAddr) -> PhysAddr {
    pa & !(PAGE_SIZE as u64 - 1)
}

/// Round physical address up to page boundary
#[inline]
pub const fn page_align_up(pa: PhysAddr) -> PhysAddr {
    (pa + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1)
}

bitflags::bitflags! {
    /// Page flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PageFlags: u32 {
        /// Page is on a buddy free list
        const BUDDY = 1 << 0;
        /// Page is reserved (not available for allocation)
        const RESERVED = 1 << 1;
        /// Page is used by kernel
        const KERNEL = 1 << 2;
        /// Page is dirty
        const DIRTY = 1 << 3;
    }
}

/// Page metadata structure
///
/// One instance per physical page frame, indexed by PFN.
/// Size: 16 bytes per page (for 128MB RAM = 32K pages = 512KB metadata)
#[repr(C)]
pub struct Page {
    /// Reference count (0 = free)
    pub refcount: AtomicU32,
    /// Buddy order (0 for 4KB, 1 for 8KB, etc.)
    pub order: u8,
    /// Flags
    pub flags: PageFlags,
    /// Reserved for future use
    _reserved: u16,
}

impl Page {
    /// Create a new page with zero refcount
    pub const fn new() -> Self {
        Self {
            refcount: AtomicU32::new(0),
            order: 0,
            flags: PageFlags::empty(),
            _reserved: 0,
        }
    }

    /// Check if page is free (refcount == 0)
    pub fn is_free(&self) -> bool {
        self.refcount.load(Ordering::Acquire) == 0
    }

    /// Increment reference count
    pub fn get(&self) {
        self.refcount.fetch_add(1, Ordering::AcqRel);
    }

    /// Decrement reference count and return new value
    pub fn put(&self) -> u32 {
        self.refcount.fetch_sub(1, Ordering::AcqRel) - 1
    }

    /// Set reference count
    pub fn set_refcount(&self, count: u32) {
        self.refcount.store(count, Ordering::Release);
    }

    /// Get reference count
    pub fn get_refcount(&self) -> u32 {
        self.refcount.load(Ordering::Acquire)
    }
}

impl core::fmt::Debug for Page {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Page")
            .field("refcount", &self.get_refcount())
            .field("order", &self.order)
            .field("flags", &self.flags)
            .finish()
    }
}
