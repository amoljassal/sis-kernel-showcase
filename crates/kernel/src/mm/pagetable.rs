//! Page Table Duplication for Fork
//!
//! Provides page table duplication functionality for process forking.
//! Works in conjunction with existing COW (Copy-On-Write) infrastructure
//! in mm/fault.rs.
//!
//! # Phase 8 Milestone 4: Fork Scaffolding
//!
//! This module implements the page table duplication needed for fork().
//! The existing COW infrastructure handles the actual copy-on-write faults.
//!
//! ## Architecture
//!
//! ```text
//! Fork Process:
//! 1. duplicate_user_page_table() - Copy parent's page tables
//! 2. setup_cow_for_fork() - Mark pages as COW (already in fault.rs)
//! 3. Page fault handler detects write to COW page
//! 4. handle_cow_fault() - Allocate new page and copy (already in fault.rs)
//! ```

use crate::lib::error::KernelError;
use super::paging::{PageTable, Pte, PteFlags, PAGE_SIZE};
use super::buddy::alloc_page;

/// Duplicate a user page table for fork()
///
/// Creates a complete copy of the parent's page table structure (L0-L3).
/// Page mappings are copied shallowly - both parent and child point to
/// the same physical pages initially. COW is set up separately via
/// `setup_cow_for_fork()`.
///
/// # Arguments
/// * `parent_root` - Physical address of parent's L0 page table
///
/// # Returns
/// * Physical address of child's new L0 page table
///
/// # Page Table Layout (AArch64 4-level)
/// ```text
/// L0 (512 GB)  → L1 (1 GB)   → L2 (2 MB)   → L3 (4 KB pages)
/// 512 entries     512 entries   512 entries   512 entries
/// ```
pub fn duplicate_user_page_table(parent_root: u64) -> Result<u64, KernelError> {
    if parent_root == 0 {
        return Err(KernelError::InvalidArgument);
    }

    // Allocate new L0 page table for child
    let child_root_phys = alloc_page().ok_or(KernelError::OutOfMemory)?;
    let child_root = child_root_phys as *mut PageTable;

    // Zero-fill child page table
    unsafe {
        core::ptr::write_bytes(child_root, 0, 1);
    }

    // Copy parent's L0 entries
    let parent_root = parent_root as *const PageTable;
    unsafe {
        duplicate_page_table_level(parent_root, child_root, 0)?;
    }

    crate::debug!("Duplicated page table: parent={:#x}, child={:#x}",
                  parent_root as u64, child_root_phys);

    Ok(child_root_phys)
}

/// Recursively duplicate page table at a given level
///
/// # Arguments
/// * `parent` - Parent page table at this level
/// * `child` - Child page table at this level (pre-allocated)
/// * `level` - Current level (0=L0, 1=L1, 2=L2, 3=L3)
///
/// # Safety
/// Caller must ensure parent and child are valid page table pointers.
unsafe fn duplicate_page_table_level(
    parent: *const PageTable,
    child: *mut PageTable,
    level: usize,
) -> Result<(), KernelError> {
    // Iterate through all entries in this page table
    for i in 0..512 {
        let parent_pte = (*parent).entries[i];

        if !parent_pte.is_valid() {
            // Skip invalid entries
            continue;
        }

        if level < 3 {
            // This is a table descriptor (L0, L1, or L2) - recurse
            if parent_pte.flags().contains(PteFlags::TABLE) {
                // Allocate new page table for this entry
                let child_table_phys = alloc_page().ok_or(KernelError::OutOfMemory)?;
                let child_table = child_table_phys as *mut PageTable;

                // Zero-fill new table
                core::ptr::write_bytes(child_table, 0, 1);

                // Recursively copy next level
                let parent_table = parent_pte.phys_addr() as *const PageTable;
                duplicate_page_table_level(parent_table, child_table, level + 1)?;

                // Create table descriptor in child's page table
                let table_flags = parent_pte.flags();
                (*child).entries[i] = Pte::new(child_table_phys, table_flags);
            } else {
                // Block descriptor (large page) - copy directly
                // Note: For MVP, we assume only 4KB pages (no huge pages)
                (*child).entries[i] = parent_pte;
            }
        } else {
            // Level 3 (page level) - copy PTE directly
            // Both parent and child will point to same physical page
            // COW will be set up later by setup_cow_for_fork()
            (*child).entries[i] = parent_pte;
        }
    }

    Ok(())
}

/// Clone page table with immediate COW setup (convenience function)
///
/// This combines page table duplication with COW setup into a single call.
/// Useful for fork() implementation.
///
/// # Arguments
/// * `parent_mm` - Parent's memory manager
///
/// # Returns
/// * Physical address of child's page table
pub fn clone_page_table_with_cow(
    parent_mm: &mut crate::process::MemoryManager
) -> Result<u64, KernelError> {
    // Duplicate page tables
    let child_pt = duplicate_user_page_table(parent_mm.page_table)?;

    // Set up COW for parent (this also affects child since they share pages)
    super::fault::setup_cow_for_fork(parent_mm)
        .map_err(|_| KernelError::OutOfMemory)?;

    Ok(child_pt)
}

/// Free a page table and all its sub-tables recursively
///
/// Used when a process exits to clean up its page table hierarchy.
/// Does NOT free the actual data pages - only the page table structure.
///
/// # Arguments
/// * `root` - Physical address of L0 page table
///
/// # Safety
/// This function frees page tables but not the mapped pages themselves.
/// Caller must handle mapped page freeing separately.
pub unsafe fn free_page_table(root: u64, level: usize) {
    if root == 0 {
        return;
    }

    let table = root as *const PageTable;

    if level < 3 {
        // Free sub-tables first (recursively)
        for i in 0..512 {
            let pte = (*table).entries[i];

            if pte.is_valid() && pte.flags().contains(PteFlags::TABLE) {
                free_page_table(pte.phys_addr(), level + 1);
            }
        }
    }

    // Free this table itself
    super::buddy::free_page(root);
}

/// Get statistics about a page table
///
/// Useful for debugging and monitoring memory usage.
///
/// # Returns
/// * (total_tables, total_mapped_pages)
pub fn get_page_table_stats(root: u64) -> (usize, usize) {
    if root == 0 {
        return (0, 0);
    }

    unsafe {
        let mut total_tables = 0;
        let mut total_pages = 0;
        count_page_table_usage(root as *const PageTable, 0, &mut total_tables, &mut total_pages);
        (total_tables, total_pages)
    }
}

/// Recursively count page table usage
unsafe fn count_page_table_usage(
    table: *const PageTable,
    level: usize,
    total_tables: &mut usize,
    total_pages: &mut usize,
) {
    *total_tables += 1;

    for i in 0..512 {
        let pte = (*table).entries[i];

        if !pte.is_valid() {
            continue;
        }

        if level < 3 && pte.flags().contains(PteFlags::TABLE) {
            // Recurse into sub-table
            let sub_table = pte.phys_addr() as *const PageTable;
            count_page_table_usage(sub_table, level + 1, total_tables, total_pages);
        } else if level == 3 {
            // Leaf page
            *total_pages += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_table_stats() {
        // Test statistics counting
        // (Actual test would require setting up page tables)
        let (tables, pages) = get_page_table_stats(0);
        assert_eq!(tables, 0);
        assert_eq!(pages, 0);
    }
}
