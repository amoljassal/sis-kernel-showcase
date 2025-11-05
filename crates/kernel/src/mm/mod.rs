/// Memory management subsystem
///
/// Phase A1 implementation including:
/// - Buddy page allocator
/// - Page table management and PTE flags
/// - Virtual memory areas (VMAs)
/// - Page fault handling with COW support
/// - brk/mmap/munmap syscalls

pub mod page;
pub mod buddy;
pub mod paging;
pub mod address_space;
pub mod fault;

// Re-export commonly used items
pub use page::{
    PhysAddr, Pfn, Page, PageFlags,
    pa_to_pfn, pfn_to_pa,
    page_align_down, page_align_up,
};

pub use buddy::{
    init_buddy, alloc_page, alloc_pages,
    free_page, free_pages, get_stats,
    MAX_ORDER, AllocStats,
};

pub use paging::{
    PAGE_SIZE, PAGE_SHIFT, KERNEL_BASE,
    PteFlags, Pte, PageTable,
    map_page, unmap_page,
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
