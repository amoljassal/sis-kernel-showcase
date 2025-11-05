/// Page fault handler with copy-on-write support
///
/// Handles data and instruction aborts from EL0 and EL1.
/// Implements COW (copy-on-write) for fork.

use crate::arch::TrapFrame;
use crate::process::{current_pid, get_process_table};
use crate::lib::error::Errno;
use super::paging::{PteFlags, alloc_page, flush_tlb, PAGE_SIZE};

/// Fault Status Code (FSC) from ESR_EL1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultType {
    /// Permission fault (write to read-only page)
    Permission,
    /// Translation fault (page not mapped)
    Translation,
    /// Access flag fault
    AccessFlag,
    /// Other/Unknown
    Other,
}

/// Parse fault type from ESR_EL1
pub fn parse_fault_type(esr: u64) -> FaultType {
    let dfsc = esr & 0x3F; // Data Fault Status Code
    match dfsc {
        // Translation faults (level 0-3)
        0b000100 | 0b000101 | 0b000110 | 0b000111 => FaultType::Translation,
        // Access flag faults (level 0-3)
        0b001000 | 0b001001 | 0b001010 | 0b001011 => FaultType::AccessFlag,
        // Permission faults (level 0-3)
        0b001100 | 0b001101 | 0b001110 | 0b001111 => FaultType::Permission,
        _ => FaultType::Other,
    }
}

/// Check if fault is a write fault
pub fn is_write_fault(esr: u64) -> bool {
    // WnR bit (bit 6) indicates write not read
    (esr & (1 << 6)) != 0
}

/// Handle page fault
///
/// Called from trap handler when a data or instruction abort occurs.
/// Returns Ok(()) if fault was handled, Err otherwise.
pub fn handle_page_fault(frame: &mut TrapFrame, far: u64, esr: u64) -> Result<(), Errno> {
    let fault_type = parse_fault_type(esr);
    let is_write = is_write_fault(esr);

    crate::debug!("Page fault at {:#x}: type={:?}, write={}", far, fault_type, is_write);

    // Get current process
    let pid = current_pid();
    let mut table = get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    // Check if address is in a valid VMA
    let vma = task.mm.find_vma(far).ok_or_else(|| {
        crate::error!("Page fault: no VMA for address {:#x}", far);
        Errno::EFAULT
    })?;

    // Handle based on fault type
    match (fault_type, is_write) {
        (FaultType::Permission, true) => {
            // Write to read-only page - check for COW
            handle_cow_fault(task, far)
        }
        (FaultType::Translation, _) => {
            // Page not mapped - lazy allocation
            handle_lazy_fault(task, far, vma.flags)
        }
        (FaultType::AccessFlag, _) => {
            // Access flag fault - set access flag
            // (For now, treat as translation fault)
            handle_lazy_fault(task, far, vma.flags)
        }
        _ => {
            crate::error!("Unhandled page fault: type={:?}, write={}", fault_type, is_write);
            Err(Errno::EFAULT)
        }
    }
}

/// Handle copy-on-write fault
fn handle_cow_fault(task: &mut crate::process::Task, fault_addr: u64) -> Result<(), Errno> {
    // Round down to page boundary
    let page_addr = fault_addr & !(PAGE_SIZE as u64 - 1);

    crate::debug!("COW fault at {:#x}", page_addr);

    let page_table = task.mm.page_table as *mut super::paging::PageTable;

    // Get the current PTE
    let pte_ptr = super::paging::get_pte_mut(page_table, page_addr)
        .map_err(|_| Errno::EFAULT)?;

    let old_pte = unsafe { *pte_ptr };

    // Verify this is actually a COW fault
    if !old_pte.flags().is_cow() {
        crate::error!("Permission fault but not COW at {:#x}", page_addr);
        return Err(Errno::EFAULT);
    }

    let old_phys = old_pte.phys_addr();

    // Allocate new page
    let new_phys = alloc_page().ok_or(Errno::ENOMEM)?;

    // Copy old page to new page (both are identity mapped in kernel space)
    unsafe {
        core::ptr::copy_nonoverlapping(
            old_phys as *const u8,
            new_phys as *mut u8,
            PAGE_SIZE
        );
    }

    // Update PTE: clear COW and READONLY, keep other flags
    let mut new_flags = old_pte.flags();
    new_flags.clear_cow(); // Removes COW and READONLY

    unsafe {
        *pte_ptr = super::paging::Pte::new(new_phys, new_flags);
    }

    // Flush TLB for this address
    super::paging::flush_tlb(page_addr);

    // TODO: Decrement refcount on old page (when refcounting is implemented)

    crate::debug!("COW: copied page {:#x} -> {:#x} for fault at {:#x}",
                  old_phys, new_phys, fault_addr);
    Ok(())
}

/// Handle lazy allocation fault (translation fault)
fn handle_lazy_fault(
    task: &mut crate::process::Task,
    fault_addr: u64,
    vma_flags: crate::process::VmaFlags,
) -> Result<(), Errno> {
    // Round down to page boundary
    let page_addr = fault_addr & !(PAGE_SIZE as u64 - 1);

    crate::debug!("Lazy fault at {:#x}, flags={:?}", page_addr, vma_flags);

    // Allocate a physical page (buddy allocator already zero-fills)
    let phys_page = alloc_page().ok_or(Errno::ENOMEM)?;

    // Convert VMA flags to PTE flags
    let pte_flags = if vma_flags.contains(crate::process::VmaFlags::EXEC) {
        // Executable: R|X (never writable)
        PteFlags::user_rx()
    } else if vma_flags.contains(crate::process::VmaFlags::WRITE) {
        // Writable: R|W (never executable, enforced in map_user_page)
        PteFlags::user_rw()
    } else {
        // Read-only
        PteFlags::user_ro()
    };

    let page_table = task.mm.page_table as *mut super::paging::PageTable;

    // Map the page in the page table
    super::paging::map_user_page(page_table, page_addr, phys_page, pte_flags)
        .map_err(|_| Errno::ENOMEM)?;

    crate::debug!("Lazy: allocated and mapped page {:#x} for fault at {:#x}",
                  phys_page, fault_addr);
    Ok(())
}

/// Set up COW for fork
///
/// Mark all writable pages in the parent and child as read-only with COW bit.
/// This is called during fork to enable copy-on-write.
pub fn setup_cow_for_fork(parent_mm: &mut crate::process::MemoryManager) -> Result<(), Errno> {
    // TODO: Walk page tables
    // TODO: For each writable user page:
    //   1. Mark as read-only
    //   2. Set COW bit
    //   3. Increment refcount
    // TODO: Flush TLB

    crate::info!("COW setup for fork (stub)");
    Ok(())
}
