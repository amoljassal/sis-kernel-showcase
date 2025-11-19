//! Phase 2: Memory Management initialization
//!
//! Initializes all memory allocators in the correct order:
//! 1. Heap allocator (for kernel dynamic allocation)
//! 2. Buddy allocator (for page-level physical memory)
//! 3. Slab allocator (for fixed-size kernel objects)

use super::{InitError, InitResult};

/// Helper for UART output during init
#[inline(always)]
unsafe fn uart_print(msg: &[u8]) {
    let base = crate::platform::active().uart().base as *mut u32;
    for &b in msg {
        core::ptr::write_volatile(base, b as u32);
    }
}

/// Initialize memory management subsystem
///
/// # Safety
/// Must be called after platform initialization
pub unsafe fn init_memory() -> InitResult<()> {
    uart_print(b"MM: INIT\n");

    // Initialize heap allocator
    if let Err(e) = crate::heap::init_heap() {
        uart_print(b"HEAP: INIT FAILED - ");
        uart_print(e.as_bytes());
        uart_print(b"\n");
        return Err(InitError::HeapFailed);
    }
    uart_print(b"HEAP: READY\n");

    // Run heap tests
    uart_print(b"HEAP: TESTING\n");
    if let Err(e) = crate::heap::test_heap() {
        uart_print(b"HEAP: TEST FAILED - ");
        uart_print(e.as_bytes());
        uart_print(b"\n");
        return Err(InitError::HeapFailed);
    }
    uart_print(b"HEAP: TESTS PASSED\n");

    // Initialize buddy allocator
    uart_print(b"MM: BUDDY ALLOCATOR\n");
    let ram_start = 0x4100_0000u64; // Start after kernel
    let ram_size = 112 * 1024 * 1024u64; // 112MB available
    let ranges: &[(u64, usize)] = &[(ram_start, ram_size as usize)];

    crate::mm::init_buddy(ranges).map_err(|_| InitError::BuddyFailed)?;

    let stats = crate::mm::get_stats().unwrap_or_default();
    uart_print(b"MM: BUDDY READY (");
    // Print stats.total_pages
    uart_print(b" pages)\n");

    // Initialize slab allocator
    uart_print(b"MM: SLAB ALLOCATOR\n");
    crate::mm::slab::init();
    uart_print(b"MM: SLAB READY\n");

    Ok(())
}
