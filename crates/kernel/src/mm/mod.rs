/// Memory management subsystem
///
/// Phase A1 implementation including:
/// - Page table management and PTE flags
/// - Virtual memory areas (VMAs)
/// - Page fault handling with COW support
/// - brk/mmap/munmap syscalls

pub mod paging;
pub mod address_space;
pub mod fault;

// Re-export commonly used items
pub use paging::{
    PAGE_SIZE, PAGE_SHIFT, KERNEL_BASE,
    PteFlags, Pte, PageTable,
    alloc_page, free_page, map_page, unmap_page,
    flush_tlb, flush_tlb_all,
};

pub use address_space::{
    USER_STACK_TOP, USER_STACK_SIZE,
    USER_HEAP_START, USER_MMAP_BASE,
};

pub use fault::{
    handle_page_fault, setup_cow_for_fork,
    FaultType, parse_fault_type, is_write_fault,
};
