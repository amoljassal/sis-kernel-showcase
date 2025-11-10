//! Heap Allocator for SIS Kernel
//!
//! This module implements a heap allocator using linked_list_allocator for dynamic memory management.
//! Features:
//! - Thread-safe allocation with spin locks
//! - Memory region management with bounds checking
//! - Performance monitoring with allocation tracking
//! - Debug support with allocation statistics
//! - Integration with SIS kernel memory safety framework

use core::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::mm;

/// Cache-aligned array wrapper for heap memory
#[repr(align(64))] // Align to cache line size for RISC-V
struct CacheAlignedArray([u8; HEAP_SIZE]);
// Note: Previously exposed an as_mut_ptr(); now removed in favor of
// direct raw pointers via addr_of_mut! to avoid dead_code and keep APIs minimal.

/// Global heap allocator instance (wrapped by guarded allocator below)
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Deterministic no-alloc guard (when true, allocations are rejected)
static DET_NO_ALLOC: AtomicBool = AtomicBool::new(false);

/// Heap statistics for monitoring and debugging
pub struct HeapStats {
    total_allocations: usize,
    total_deallocations: usize,
    current_allocated: usize,
    peak_allocated: usize,
    allocation_failures: usize,
}

impl HeapStats {
    pub fn total_allocations(&self) -> usize { self.total_allocations }
    pub fn total_deallocations(&self) -> usize { self.total_deallocations }
    pub fn current_allocated(&self) -> usize { self.current_allocated }
    pub fn peak_allocated(&self) -> usize { self.peak_allocated }
    pub fn allocation_failures(&self) -> usize { self.allocation_failures }
}

static HEAP_STATS: Mutex<HeapStats> = Mutex::new(HeapStats {
    total_allocations: 0,
    total_deallocations: 0,
    current_allocated: 0,
    peak_allocated: 0,
    allocation_failures: 0,
});

/// Heap configuration
const HEAP_START: usize = 0x444_44440_0000;
const HEAP_SIZE: usize = 8 * 1024 * 1024; // 8 MiB heap for bringup (avoid WM test OOM)

/// Return total heap size for telemetry calculations
/// This provides a single source of truth for heap size across all subsystems
pub const fn heap_total_size() -> usize {
    HEAP_SIZE
}

/// Heap initialization status (lock-free, avoids potential early boot stalls)
static HEAP_INIT_DONE: AtomicBool = AtomicBool::new(false);

/// Initialize the kernel heap
pub fn init_heap() -> Result<(), &'static str> {
    // Initialize once using a lock-free guard
    if !HEAP_INIT_DONE.load(Ordering::SeqCst) {
        unsafe { crate::uart_print(b"[HEAP] ENTER INIT\n"); }
        if HEAP_INIT_DONE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            unsafe {
                crate::uart_print(b"[HEAP] GUARD SET\n");
                // Use a static array as heap memory (cache-aligned)
                static mut HEAP_MEMORY: CacheAlignedArray = CacheAlignedArray([0; HEAP_SIZE]);
                let heap_arr_ptr = core::ptr::addr_of_mut!(HEAP_MEMORY) as *mut CacheAlignedArray;
                let heap_start = core::ptr::addr_of_mut!((*heap_arr_ptr).0) as *mut u8;

                crate::uart_print(b"[HEAP] BEFORE INIT ALLOCATOR\n");
                // Initialize the allocator with our memory region
                ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
                crate::uart_print(b"[HEAP] AFTER INIT ALLOCATOR\n");

                // Print initialization message (simple, non-allocating)
                crate::uart_print(b"[HEAP] Initialized ");
                print_size(HEAP_SIZE);
                crate::uart_print(b" heap at 0x");
                print_hex(heap_start as usize);
                crate::uart_print(b"\n");
            }
        }
    }

    Ok(())
}

/// Custom wrapper around GlobalAlloc to track statistics
pub struct StatsTrackingAllocator;

unsafe impl GlobalAlloc for StatsTrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Reject allocations under deterministic guard
        if DET_NO_ALLOC.load(Ordering::Relaxed) {
            crate::trace::metric_kv("det_alloc_violation", 1);
            return core::ptr::null_mut();
        }
        // Runtime verification hook for memory allocation
        #[cfg(target_arch = "riscv64")]
        {
            use crate::arch::riscv64::verification::CriticalOperation;
            crate::verify_lightweight!(CriticalOperation::MemoryAllocation, "heap_alloc");
        }

        // Large-allocation fast path: back big blocks with contiguous pages from buddy
        const LARGE_ALLOC_THRESHOLD: usize = 1 * 1024 * 1024; // 1 MiB
        let ptr = if layout.size() >= LARGE_ALLOC_THRESHOLD {
            large_alloc(layout)
        } else {
            ALLOCATOR.alloc(layout)
        };

        if !ptr.is_null() {
            let mut stats = HEAP_STATS.lock();
            stats.total_allocations += 1;
            stats.current_allocated += layout.size();
            if stats.current_allocated > stats.peak_allocated {
                stats.peak_allocated = stats.current_allocated;
            }

            // Verify allocation result
            #[cfg(target_arch = "riscv64")]
            {
                // Check alignment
                if (ptr as usize) % layout.align() != 0 {
                    crate::uart_print(b"[HEAP] Alignment violation in allocation\n");
                }
                // Check bounds (simplified)
                if (ptr as usize) < 0x4444_4440_0000 {
                    crate::uart_print(b"[HEAP] Allocation outside heap bounds\n");
                }
            }
        } else {
            let mut stats = HEAP_STATS.lock();
            stats.allocation_failures += 1;

            // Log allocation failure for verification
            #[cfg(target_arch = "riscv64")]
            {
                crate::uart_print(b"[VERIFY] Memory allocation failed\n");
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Runtime verification hook for memory deallocation
        #[cfg(target_arch = "riscv64")]
        {
            use crate::arch::riscv64::verification::CriticalOperation;
            crate::verify_lightweight!(CriticalOperation::MemoryDeallocation, "heap_dealloc");

            // Verify pointer validity before deallocation
            if ptr.is_null() {
                crate::uart_print(b"[VERIFY] Attempt to deallocate null pointer\n");
                return;
            }

            // Basic bounds check
            if (ptr as usize) < 0x4444_4440_0000 {
                crate::uart_print(b"[VERIFY] Deallocation outside heap bounds\n");
                return;
            }
        }

        // Update stats BEFORE checking deallocation path (fixes stats leak bug)
        let mut stats = HEAP_STATS.lock();
        stats.total_deallocations += 1;
        stats.current_allocated = stats.current_allocated.saturating_sub(layout.size());
        drop(stats); // Release lock before potentially expensive operations

        // Check if this was a large allocation backed by buddy pages
        if large_dealloc(ptr) {
            // deallocated via buddy path
            return;
        }
        ALLOCATOR.dealloc(ptr, layout);
    }
}

/// Install guarded global allocator
#[global_allocator]
static GLOBAL_ALLOC: StatsTrackingAllocator = StatsTrackingAllocator;

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    unsafe {
        crate::uart_print(b"[HEAP] ALLOCATION ERROR: size=");
        print_size(layout.size());
        crate::uart_print(b" align=");
        print_size(layout.align());
        crate::uart_print(b"\n");

        // Print heap statistics for debugging
        print_heap_stats();

        // In no_std bare-metal, abort by halting
        loop {}
    }
}

// Note: Newer toolchains may refer to internal OOM symbols only in certain
// optimization modes. We avoid release linking issues by building the kernel
// in debug for test runs. Our alloc_error_handler plus the fallback below
// ensure graceful handling without pulling in extra runtime.

// Some toolchains expect this low-level alloc error hook symbol as well.
// Provide a minimal definition to satisfy the linker in no_std.
// NOTE: Disabled when using -Zbuild-std as core/alloc library provides its own handler
// #[no_mangle]
// pub extern "C" fn __rust_alloc_error_handler(_size: usize, _align: usize) -> ! {
//     loop {}
// }

// Newer nightly toolchains reference an internal OOM hook symbol `__rg_oom`.
// Export a minimal handler to satisfy the linker in no_std release builds.
// NOTE: Disabled when using -Zbuild-std as core library provides its own handler
// #[allow(improper_ctypes_definitions)]
// #[no_mangle]
// pub extern "C" fn __rg_oom(_layout: core::alloc::Layout) -> ! {
//     loop {}
// }

// --------- Large allocation fallback using buddy pages ---------

const LARGE_MAGIC: u64 = 0x4C4152475F414C4Cu64; // "LARG_ALL"

#[repr(C)]
struct LargeAllocHeader {
    magic: u64,
    phys: u64,
    order: u8,
    _pad: [u8; 7],
}

#[inline(always)]
fn align_up(value: usize, align: usize) -> usize {
    let a = align.max(1);
    (value + a - 1) & !(a - 1)
}

unsafe fn large_alloc(layout: core::alloc::Layout) -> *mut u8 {
    let header_size = core::mem::size_of::<LargeAllocHeader>();
    let align_req = layout.align().min(mm::PAGE_SIZE);
    let need = layout.size().saturating_add(header_size).saturating_add(align_req);
    let pages = (need + mm::PAGE_SIZE - 1) / mm::PAGE_SIZE;
    let mut order: u8 = 0; while (1usize << order) < pages { order += 1; }
    let phys = match mm::alloc_pages(order) { Some(p) => p, None => return core::ptr::null_mut() };
    let base = phys as usize; // identity map: VA == PA for RAM
    let ret_ptr = align_up(base + header_size, align_req);
    let header_ptr = (ret_ptr - header_size) as *mut LargeAllocHeader;
    // Write header immediately before the returned pointer
    core::ptr::write(header_ptr, LargeAllocHeader { magic: LARGE_MAGIC, phys, order, _pad: [0;7] });
    ret_ptr as *mut u8
}

unsafe fn large_dealloc(ptr: *mut u8) -> bool {
    if ptr.is_null() { return false; }
    let header_size = core::mem::size_of::<LargeAllocHeader>();
    let header_ptr = (ptr as usize - header_size) as *const LargeAllocHeader;
    // Best-effort check; invalid read is unlikely since we always place header
    let hdr = &*header_ptr;
    if hdr.magic != LARGE_MAGIC { return false; }
    mm::free_pages(hdr.phys, hdr.order);
    true
}


/// Print current heap statistics
pub fn print_heap_stats() {
    let stats = HEAP_STATS.lock();

    unsafe {
        crate::uart_print(b"[HEAP] Stats: allocs=");
        print_number(stats.total_allocations);
        crate::uart_print(b" deallocs=");
        print_number(stats.total_deallocations);
        crate::uart_print(b" current=");
        print_size(stats.current_allocated);
        crate::uart_print(b" peak=");
        print_size(stats.peak_allocated);
        crate::uart_print(b" failures=");
        print_number(stats.allocation_failures);
        crate::uart_print(b"\n");
    }
}

/// Get current heap usage statistics
pub fn get_heap_stats() -> HeapStats {
    let stats = HEAP_STATS.lock();
    HeapStats {
        total_allocations: stats.total_allocations,
        total_deallocations: stats.total_deallocations,
        current_allocated: stats.current_allocated,
        peak_allocated: stats.peak_allocated,
        allocation_failures: stats.allocation_failures,
    }
}

/// Reset current_allocated counter for testing
/// WARNING: Only use this for stress tests where you want to start from a clean state
pub fn reset_current_allocated_for_test() {
    let mut stats = HEAP_STATS.lock();
    stats.current_allocated = 0;
    stats.peak_allocated = 0;
}

/// Test heap functionality with various allocation patterns
pub fn test_heap() -> Result<(), &'static str> {
    unsafe {
        crate::uart_print(b"[HEAP] Starting heap tests...\n");
    }

    // Test 1: Basic allocation and deallocation
    unsafe {
        let layout = Layout::from_size_align(1024, 8).unwrap();
        let ptr = ALLOCATOR.alloc(layout);
        if ptr.is_null() {
            return Err("Failed to allocate 1KB");
        }

        // Write test pattern
        for i in 0..1024 {
            *ptr.add(i) = (i % 256) as u8;
        }

        // Verify test pattern
        for i in 0..1024 {
            if *ptr.add(i) != (i % 256) as u8 {
                ALLOCATOR.dealloc(ptr, layout);
                return Err("Memory corruption detected");
            }
        }

        ALLOCATOR.dealloc(ptr, layout);
        crate::uart_print(b"[HEAP] Test 1 passed: basic allocation/deallocation\n");
    }

    // Test 2: Multiple allocations
    let mut ptrs = heapless::Vec::<(*mut u8, Layout), 10>::new();

    unsafe {
        for i in 0..5 {
            let size = 64 * (i + 1);
            let layout = Layout::from_size_align(size, 8).unwrap();
            let ptr = ALLOCATOR.alloc(layout);

            if ptr.is_null() {
                // Clean up any successful allocations
                for (ptr, layout) in ptrs.iter() {
                    ALLOCATOR.dealloc(*ptr, *layout);
                }
                return Err("Failed multiple allocation test");
            }

            ptrs.push((ptr, layout)).map_err(|_| "Vec full")?;
        }

        // Clean up
        for (ptr, layout) in ptrs.iter() {
            ALLOCATOR.dealloc(*ptr, *layout);
        }

        crate::uart_print(b"[HEAP] Test 2 passed: multiple allocations\n");
    }

    // Test 3: Alignment requirements
    unsafe {
        for align in [8, 16, 32, 64].iter() {
            let layout = Layout::from_size_align(128, *align).unwrap();
            let ptr = ALLOCATOR.alloc(layout);

            if ptr.is_null() {
                return Err("Failed alignment test");
            }

            // Check alignment
            if (ptr as usize) % align != 0 {
                ALLOCATOR.dealloc(ptr, layout);
                return Err("Alignment requirement not met");
            }

            ALLOCATOR.dealloc(ptr, layout);
        }

        crate::uart_print(b"[HEAP] Test 3 passed: alignment requirements\n");
    }

    unsafe {
        crate::uart_print(b"[HEAP] All tests passed!\n");
    }
    print_heap_stats();
    Ok(())
}

/// Enter deterministic no-alloc region
pub fn det_no_alloc_enter() {
    DET_NO_ALLOC.store(true, Ordering::Relaxed);
}

/// Exit deterministic no-alloc region
pub fn det_no_alloc_exit() {
    DET_NO_ALLOC.store(false, Ordering::Relaxed);
}

/// Helper function to print hex numbers
unsafe fn print_hex(mut num: usize) {
    crate::uart_print(b"0x");
    if num == 0 {
        crate::uart_print(b"0");
        return;
    }

    let mut digits = [0u8; 16];
    let mut i = 0;

    while num > 0 {
        let digit = num % 16;
        digits[i] = if digit < 10 {
            b'0' + digit as u8
        } else {
            b'A' + (digit - 10) as u8
        };
        num /= 16;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::uart_print(&[digits[i]]);
    }
}

/// Helper function to print numbers
unsafe fn print_number(mut num: usize) {
    if num == 0 {
        crate::uart_print(b"0");
        return;
    }

    let mut digits = [0u8; 20];
    let mut i = 0;

    while num > 0 {
        digits[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::uart_print(&[digits[i]]);
    }
}

/// Helper function to print sizes with units
unsafe fn print_size(size: usize) {
    if size >= 1024 * 1024 {
        print_number(size / (1024 * 1024));
        crate::uart_print(b" MiB");
    } else if size >= 1024 {
        print_number(size / 1024);
        crate::uart_print(b" KiB");
    } else {
        print_number(size);
        crate::uart_print(b" bytes");
    }
}

/// Memory safety: Bounds checking for heap allocations
pub fn is_valid_heap_ptr(ptr: *const u8, size: usize) -> bool {
    let addr = ptr as usize;

    // Check if pointer is within heap bounds
    // Note: In a real implementation, this would check against actual mapped memory
    addr >= HEAP_START && addr.saturating_add(size) <= HEAP_START + HEAP_SIZE
}

/// Advanced heap features for AI workloads
pub mod ai_heap {
    use super::*;
    use core::alloc::Layout;

    /// Allocate aligned memory for tensor operations
    pub fn alloc_tensor_aligned(size: usize, alignment: usize) -> Result<*mut u8, &'static str> {
        let layout =
            Layout::from_size_align(size, alignment).map_err(|_| "Invalid tensor layout")?;

        unsafe {
            let ptr = ALLOCATOR.alloc(layout);
            if ptr.is_null() {
                Err("Tensor allocation failed")
            } else {
                // Zero the memory for deterministic AI operations
                for i in 0..size {
                    *ptr.add(i) = 0;
                }
                Ok(ptr)
            }
        }
    }

    /// Free tensor-aligned memory
    pub unsafe fn free_tensor_aligned(ptr: *mut u8, size: usize, alignment: usize) {
        let layout = Layout::from_size_align(size, alignment).unwrap();
        ALLOCATOR.dealloc(ptr, layout);
    }

    /// Allocate contiguous memory for AI model weights
    pub fn alloc_model_weights(num_weights: usize) -> Result<*mut f32, &'static str> {
        let size = num_weights * core::mem::size_of::<f32>();
        let alignment = core::mem::align_of::<f32>();

        let ptr = alloc_tensor_aligned(size, alignment)? as *mut f32;
        Ok(ptr)
    }
}

/// Performance monitoring for heap operations
pub struct HeapProfiler {
    #[allow(dead_code)]
    start_time: u64,
    operation: &'static str,
}

impl HeapProfiler {
    pub fn new(operation: &'static str) -> Self {
        // In a real implementation, this would use a proper timer
        HeapProfiler {
            start_time: 0,
            operation,
        }
    }
}

impl Drop for HeapProfiler {
    fn drop(&mut self) {
        unsafe {
            crate::uart_print(b"[HEAP] ");
            crate::uart_print(self.operation.as_bytes());
            crate::uart_print(b" completed\n");
        }
    }
}
