//! Slab Allocator for Small Objects
//!
//! Implementation of a slab allocator optimized for small, frequently-allocated objects.
//! Based on Bonwick's original slab allocator paper (USENIX 1994).
//!
//! # Overview
//!
//! The slab allocator maintains fixed-size object caches for common allocation sizes:
//! 16, 32, 64, 128, and 256 bytes. This provides:
//!
//! - **Fast allocation**: O(1) allocation from free list
//! - **Low fragmentation**: Objects of same size grouped together
//! - **Cache efficiency**: Hot objects remain in cache lines
//! - **No metadata overhead**: Free list stored in freed objects
//!
//! # Design
//!
//! Each size class maintains three lists:
//! - **Partial slabs**: Have both free and allocated objects (best locality)
//! - **Full slabs**: All objects allocated
//! - **Empty slabs**: All objects free (reusable)
//!
//! # Performance Targets
//!
//! - Allocation latency: <5,000 cycles (vs. 28,000 for linked-list allocator)
//! - Deallocation latency: <3,000 cycles
//! - Memory overhead: Zero (free list in freed objects)
//!
//! # Example
//!
//! ```rust,no_run
//! use crate::mm::slab;
//!
//! // Initialize slab allocator
//! slab::init();
//!
//! // Allocate 64-byte object
//! let layout = Layout::from_size_align(64, 8).unwrap();
//! let ptr = slab::allocate(layout);
//!
//! // Use object...
//!
//! // Free object
//! slab::deallocate(ptr, layout);
//! ```

use core::alloc::Layout;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;
use crate::mm;

/// Slab size classes (powers of 2 from 16 to 256 bytes)
const SLAB_SIZES: [usize; 5] = [16, 32, 64, 128, 256];

/// Flag to enable slab allocator (starts disabled until buddy init completes)
static SLAB_ENABLED: AtomicBool = AtomicBool::new(false);

/// Page size for slab allocation (4KB)
const SLAB_PAGE_SIZE: usize = 4096;

/// Maximum number of caches
const NUM_CACHES: usize = 5;

/// Free object header
///
/// Stored in freed objects to form a singly-linked free list.
/// This clever technique means zero metadata overhead.
#[repr(C)]
struct FreeObject {
    next: Option<NonNull<FreeObject>>,
}

/// Individual slab page
///
/// Represents a single 4KB page divided into fixed-size objects.
/// Maintains a free list of available objects within the page.
struct SlabPage {
    /// Base address of this slab page
    base: NonNull<u8>,
    /// Head of free object list
    free_list: Option<NonNull<FreeObject>>,
    /// Number of free objects in this slab
    num_free: usize,
    /// Total number of objects in this slab
    num_total: usize,
}

impl SlabPage {
    /// Create a new slab page
    fn new(base: NonNull<u8>, num_objects: usize) -> Self {
        SlabPage {
            base,
            free_list: None,
            num_free: 0,
            num_total: num_objects,
        }
    }

    /// Initialize free list for this slab
    ///
    /// Builds a singly-linked list through all objects in the slab.
    /// Objects are linked in reverse order for better cache locality.
    fn initialize_free_list(&mut self, obj_size: usize, num_objects: usize) {
        let mut prev: Option<NonNull<FreeObject>> = None;

        for i in (0..num_objects).rev() {
            let offset = i * obj_size;
            let obj_ptr = unsafe {
                NonNull::new_unchecked(
                    self.base.as_ptr().add(offset) as *mut FreeObject
                )
            };

            unsafe {
                obj_ptr.as_ptr().write(FreeObject { next: prev });
            }

            prev = Some(obj_ptr);
        }

        self.free_list = prev;
        self.num_free = num_objects;
    }

    /// Pop object from free list
    ///
    /// Returns the next free object, or None if slab is full.
    fn pop_free(&mut self) -> Option<NonNull<u8>> {
        let free_obj = self.free_list?;

        unsafe {
            let next = (*free_obj.as_ptr()).next;
            self.free_list = next;
            self.num_free -= 1;
            Some(NonNull::new_unchecked(free_obj.as_ptr() as *mut u8))
        }
    }

    /// Push object back to free list
    ///
    /// Adds the object to the head of the free list.
    fn push_free(&mut self, ptr: NonNull<u8>) {
        let free_obj = ptr.cast::<FreeObject>();

        unsafe {
            free_obj.as_ptr().write(FreeObject {
                next: self.free_list,
            });
        }

        self.free_list = Some(free_obj);
        self.num_free += 1;
    }
}

// SAFETY: SlabPage contains raw pointers but is only accessed while holding the
// slab allocator's mutex lock. The pointers are valid kernel memory addresses
// that remain valid for the lifetime of the slab page.
unsafe impl Send for SlabPage {}
unsafe impl Sync for SlabPage {}

/// Slab cache for a specific object size
///
/// Manages all slabs for a particular size class (e.g., all 64-byte objects).
struct SlabCache {
    /// Object size for this cache
    size: usize,
    /// Number of objects per slab
    objects_per_slab: usize,
    /// Partial slabs (have free objects - check first!)
    partial_slabs: alloc::vec::Vec<SlabPage>,
    /// Full slabs (no free objects)
    full_slabs: alloc::vec::Vec<SlabPage>,
    /// Empty slabs (all objects free - reusable)
    empty_slabs: alloc::vec::Vec<SlabPage>,
    /// Number of currently allocated objects
    allocated_objects: usize,
    /// Total number of slabs
    total_slabs: usize,
}

impl SlabCache {
    /// Create new slab cache for given object size
    const fn new(size: usize) -> Self {
        let objects_per_slab = SLAB_PAGE_SIZE / size;
        SlabCache {
            size,
            objects_per_slab,
            partial_slabs: alloc::vec::Vec::new(),
            full_slabs: alloc::vec::Vec::new(),
            empty_slabs: alloc::vec::Vec::new(),
            allocated_objects: 0,
            total_slabs: 0,
        }
    }

    /// Allocate object from cache
    ///
    /// Algorithm:
    /// 1. Try partial slabs first (best locality)
    /// 2. Try empty slabs (reuse existing pages)
    /// 3. Allocate new slab from buddy allocator
    fn allocate(&mut self) -> Option<NonNull<u8>> {
        // CRITICAL: Temporarily disable slab to prevent deadlock.
        // When we call buddy allocator below, Vec operations might trigger
        // heap allocations which would recursively try to lock this same cache.
        let was_enabled = SLAB_ENABLED.swap(false, core::sync::atomic::Ordering::SeqCst);

        // Try partial slabs first (locality - hot objects)
        if let Some(slab) = self.partial_slabs.last_mut() {
            if let Some(obj) = slab.pop_free() {
                self.allocated_objects += 1;

                // Move to full list if completely allocated
                if slab.num_free == 0 {
                    let full = self.partial_slabs.pop().unwrap();
                    self.full_slabs.push(full);
                }

                // Restore slab state before returning
                SLAB_ENABLED.store(was_enabled, core::sync::atomic::Ordering::SeqCst);
                return Some(obj);
            }
        }

        // Try empty slabs (reuse existing pages)
        if let Some(mut slab) = self.empty_slabs.pop() {
            slab.initialize_free_list(self.size, self.objects_per_slab);
            let obj = slab.pop_free().unwrap();
            self.partial_slabs.push(slab);
            self.allocated_objects += 1;

            // Restore slab state before returning
            SLAB_ENABLED.store(was_enabled, core::sync::atomic::Ordering::SeqCst);
            return Some(obj);
        }

        // Allocate new slab from buddy allocator
        let phys = mm::alloc_page()?;

        // CRITICAL: Use identity mapping during early boot.
        // The kernel runs at low physical addresses (0x4xxxxxxx) with identity mapping.
        // mm::phys_to_virt() adds KERNEL_BASE (0xFFFF_0000_0000_0000), creating an
        // unmapped high address that causes translation faults.
        // TODO Phase 9: Detect if we've switched to high kernel and use phys_to_virt().
        let virt = phys as usize; // Identity mapped: VA == PA
        let base = unsafe { NonNull::new_unchecked(virt as *mut u8) };

        let mut slab = SlabPage::new(base, self.objects_per_slab);
        slab.initialize_free_list(self.size, self.objects_per_slab);

        let obj = slab.pop_free().unwrap();
        self.partial_slabs.push(slab);
        self.total_slabs += 1;
        self.allocated_objects += 1;

        // Restore slab state before returning
        SLAB_ENABLED.store(was_enabled, core::sync::atomic::Ordering::SeqCst);
        Some(obj)
    }

    /// Free object back to cache
    ///
    /// Returns true if the pointer was found and freed, false otherwise.
    ///
    /// Algorithm:
    /// 1. Find which slab owns this pointer (slab-aligned address)
    /// 2. Return object to slab's free list
    /// 3. Move slab between lists if state changed
    fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
        // CRITICAL: Temporarily disable slab to prevent deadlock.
        // Vec operations below (swap_remove, push) might trigger heap allocations
        // which would recursively try to lock this same cache.
        let was_enabled = SLAB_ENABLED.swap(false, core::sync::atomic::Ordering::SeqCst);

        // Find which slab owns this pointer (page-aligned)
        let slab_base = (ptr.as_ptr() as usize) & !(SLAB_PAGE_SIZE - 1);

        // Search full slabs first (most likely - recently allocated)
        if let Some(idx) = self.full_slabs.iter().position(|s| {
            s.base.as_ptr() as usize == slab_base
        }) {
            let mut slab = self.full_slabs.swap_remove(idx);
            slab.push_free(ptr);
            self.partial_slabs.push(slab);
            self.allocated_objects -= 1;

            // Restore slab state before returning
            SLAB_ENABLED.store(was_enabled, core::sync::atomic::Ordering::SeqCst);
            return true;
        }

        // Search partial slabs
        if let Some(slab) = self.partial_slabs.iter_mut().find(|s| {
            s.base.as_ptr() as usize == slab_base
        }) {
            slab.push_free(ptr);
            self.allocated_objects -= 1;

            // Move to empty if fully freed
            if slab.num_free == slab.num_total {
                let idx = self.partial_slabs.iter().position(|s| {
                    s.base.as_ptr() as usize == slab_base
                }).unwrap();
                let empty = self.partial_slabs.swap_remove(idx);
                self.empty_slabs.push(empty);
            }

            // Restore slab state before returning
            SLAB_ENABLED.store(was_enabled, core::sync::atomic::Ordering::SeqCst);
            return true;
        }

        // Pointer not owned by this cache
        // Restore slab state before returning false
        SLAB_ENABLED.store(was_enabled, core::sync::atomic::Ordering::SeqCst);
        false
    }

    /// Get cache statistics
    fn stats(&self) -> SlabStats {
        SlabStats {
            size: self.size,
            objects_allocated: self.allocated_objects,
            total_slabs: self.total_slabs,
            partial_slabs: self.partial_slabs.len(),
            full_slabs: self.full_slabs.len(),
            empty_slabs: self.empty_slabs.len(),
        }
    }
}

/// Slab statistics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct SlabStats {
    /// Object size for this cache
    pub size: usize,
    /// Number of currently allocated objects
    pub objects_allocated: usize,
    /// Total number of slabs
    pub total_slabs: usize,
    /// Number of partial slabs
    pub partial_slabs: usize,
    /// Number of full slabs
    pub full_slabs: usize,
    /// Number of empty slabs
    pub empty_slabs: usize,
}

impl SlabStats {
    /// Calculate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.total_slabs * SLAB_PAGE_SIZE
    }

    /// Calculate utilization percentage (0-100)
    pub fn utilization_percent(&self) -> usize {
        if self.total_slabs == 0 {
            return 0;
        }

        let total_objects = self.total_slabs * (SLAB_PAGE_SIZE / self.size);
        if total_objects == 0 {
            return 0;
        }

        (self.objects_allocated * 100) / total_objects
    }
}

/// Global slab allocator
///
/// Manages all size class caches (16, 32, 64, 128, 256 bytes).
pub struct SlabAllocator {
    caches: [Mutex<SlabCache>; NUM_CACHES],
}

impl SlabAllocator {
    /// Create new slab allocator
    pub const fn new() -> Self {
        SlabAllocator {
            caches: [
                Mutex::new(SlabCache::new(16)),
                Mutex::new(SlabCache::new(32)),
                Mutex::new(SlabCache::new(64)),
                Mutex::new(SlabCache::new(128)),
                Mutex::new(SlabCache::new(256)),
            ],
        }
    }

    /// Allocate from appropriate slab cache
    ///
    /// Returns pointer to allocated object, or None if allocation fails.
    pub fn allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
        let size = layout.size();

        // Only handle sizes <= 256 bytes
        if size > 256 {
            return None;
        }

        // Map size to cache index (round up to next power of 2)
        let cache_idx = match size {
            1..=16 => 0,
            17..=32 => 1,
            33..=64 => 2,
            65..=128 => 3,
            129..=256 => 4,
            _ => unreachable!(),
        };

        self.caches[cache_idx].lock().allocate()
    }

    /// Deallocate to appropriate slab cache
    ///
    /// Returns true if successfully deallocated, false if pointer not owned by slab.
    ///
    /// # Safety
    ///
    /// Caller must ensure:
    /// - `layout` matches the layout used for allocation
    /// - `ptr` is not used after deallocation
    pub unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        let size = layout.size();

        if size > 256 {
            return false;
        }

        let cache_idx = match size {
            1..=16 => 0,
            17..=32 => 1,
            33..=64 => 2,
            65..=128 => 3,
            129..=256 => 4,
            _ => unreachable!(),
        };

        self.caches[cache_idx].lock().deallocate(ptr)
    }

    /// Get statistics for all caches
    pub fn stats(&self) -> [SlabStats; NUM_CACHES] {
        [
            self.caches[0].lock().stats(),
            self.caches[1].lock().stats(),
            self.caches[2].lock().stats(),
            self.caches[3].lock().stats(),
            self.caches[4].lock().stats(),
        ]
    }

    /// Get total memory usage across all caches
    pub fn total_memory_usage(&self) -> usize {
        self.stats().iter().map(|s| s.memory_usage()).sum()
    }

    /// Get average utilization across all caches
    pub fn average_utilization(&self) -> usize {
        let stats = self.stats();
        let total: usize = stats.iter().map(|s| s.utilization_percent()).sum();
        total / NUM_CACHES
    }
}

// Global slab allocator instance
static SLAB_ALLOCATOR: SlabAllocator = SlabAllocator::new();

/// Initialize slab allocator
///
/// Must be called after buddy allocator initialization.
/// This enables the slab allocator for use.
pub fn init() {
    // Enable slab allocator (safe to use now that buddy is initialized)
    SLAB_ENABLED.store(true, Ordering::Release);

    crate::info!("Slab allocator initialized");
    crate::info!("  - Size classes: {:?}", SLAB_SIZES);
    crate::info!("  - Objects per page: 16B={}, 32B={}, 64B={}, 128B={}, 256B={}",
                 4096/16, 4096/32, 4096/64, 4096/128, 4096/256);
}

/// Get slab allocator instance
pub fn get() -> &'static SlabAllocator {
    &SLAB_ALLOCATOR
}

/// Check if slab allocator is enabled
///
/// Returns false during early boot (before buddy allocator is initialized).
/// This prevents circular dependency between heap, slab, and buddy allocators.
pub fn is_enabled() -> bool {
    SLAB_ENABLED.load(Ordering::Acquire)
}

/// Allocate from slab
///
/// Public API for slab allocation. Falls back to None if size > 256 bytes
/// or if slab is not yet enabled (during early boot).
pub fn allocate(layout: Layout) -> Option<NonNull<u8>> {
    if !is_enabled() {
        return None;
    }
    SLAB_ALLOCATOR.allocate(layout)
}

/// Deallocate from slab
///
/// Returns true if the pointer was owned by slab and deallocated,
/// false if the pointer is not from slab (caller should use alternate path).
///
/// # Safety
///
/// See `SlabAllocator::deallocate`
pub unsafe fn deallocate(ptr: NonNull<u8>, layout: Layout) -> bool {
    if !is_enabled() {
        return false;
    }
    SLAB_ALLOCATOR.deallocate(ptr, layout)
}

/// Get slab statistics
pub fn stats() -> [SlabStats; NUM_CACHES] {
    SLAB_ALLOCATOR.stats()
}

/// Print slab statistics
pub fn print_stats() {
    let stats = SLAB_ALLOCATOR.stats();

    crate::info!("=== Slab Allocator Statistics ===");
    for stat in &stats {
        crate::info!("{}B cache: {} objects, {} slabs ({} partial, {} full, {} empty), {}% util",
                     stat.size,
                     stat.objects_allocated,
                     stat.total_slabs,
                     stat.partial_slabs,
                     stat.full_slabs,
                     stat.empty_slabs,
                     stat.utilization_percent());
    }

    let total_mem = SLAB_ALLOCATOR.total_memory_usage();
    let avg_util = SLAB_ALLOCATOR.average_utilization();

    crate::info!("Total memory: {} KB", total_mem / 1024);
    crate::info!("Average utilization: {}%", avg_util);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slab_cache_creation() {
        let cache = SlabCache::new(64);
        assert_eq!(cache.size, 64);
        assert_eq!(cache.objects_per_slab, 4096 / 64);
        assert_eq!(cache.allocated_objects, 0);
    }

    #[test]
    fn test_slab_stats_utilization() {
        let stats = SlabStats {
            size: 64,
            objects_allocated: 32,
            total_slabs: 1,
            partial_slabs: 1,
            full_slabs: 0,
            empty_slabs: 0,
        };

        // 1 slab of 64-byte objects = 4096/64 = 64 objects
        // 32 allocated out of 64 = 50%
        assert_eq!(stats.utilization_percent(), 50);
    }
}
