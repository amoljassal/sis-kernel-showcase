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

    // TODO: Walk page table to find PTE
    // TODO: Check if page has COW bit set
    // TODO: If refcount == 1, just make writable
    // TODO: If refcount > 1, allocate new page and copy

    // For now, stub implementation
    // In a real implementation:
    // 1. Allocate a new physical page
    // 2. Copy contents from old page to new page
    // 3. Update PTE to point to new page with RW permissions
    // 4. Decrement refcount on old page
    // 5. Flush TLB for this address

    let new_page = alloc_page().ok_or(Errno::ENOMEM)?;

    // TODO: Copy old page to new page
    // TODO: Update PTE
    // flush_tlb(page_addr);

    crate::info!("COW: allocated new page {:#x} for fault at {:#x}", new_page, fault_addr);
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

    // Allocate a physical page
    let phys_page = alloc_page().ok_or(Errno::ENOMEM)?;

    // Convert VMA flags to PTE flags
    let pte_flags = if vma_flags.contains(crate::process::VmaFlags::WRITE) {
        PteFlags::user_rw()
    } else if vma_flags.contains(crate::process::VmaFlags::EXEC) {
        PteFlags::user_rx()
    } else {
        PteFlags::user_ro()
    };

    // TODO: Map the page in the page table
    // map_page(&mut task.mm.page_table, page_addr, phys_page, pte_flags);
    // flush_tlb(page_addr);

    crate::info!("Lazy: allocated page {:#x} for fault at {:#x}", phys_page, fault_addr);
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
