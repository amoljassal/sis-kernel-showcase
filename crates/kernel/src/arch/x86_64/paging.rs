//! # x86_64 Paging & Memory Management
//!
//! This module implements 4-level page table management for x86_64, providing the
//! architecture-specific paging layer that integrates with the platform-independent
//! memory management subsystem.
//!
//! ## x86_64 Paging Overview
//!
//! x86_64 uses a 4-level page table hierarchy:
//!
//! ```text
//! CR3 Register
//!     ↓
//! PML4 (Page Map Level 4) - 512 entries, covers 512 GB each
//!     ↓
//! PDPT (Page Directory Pointer Table) - 512 entries, covers 1 GB each
//!     ↓
//! PD (Page Directory) - 512 entries, covers 2 MB each
//!     ↓
//! PT (Page Table) - 512 entries, covers 4 KB each
//!     ↓
//! Physical Page (4 KB)
//! ```
//!
//! ## Virtual Address Format (48-bit)
//!
//! ```text
//! 63    48 47      39 38      30 29      21 20      12 11       0
//! +--------+---------+---------+---------+---------+------------+
//! | Sign   | PML4    | PDPT    | PD      | PT      | Offset     |
//! | Extend | Index   | Index   | Index   | Index   | (4 KB)     |
//! +--------+---------+---------+---------+---------+------------+
//!  16 bits   9 bits    9 bits    9 bits    9 bits    12 bits
//!
//! - Bits 63-48: Sign extension of bit 47 (canonical address)
//! - Bits 47-39: PML4 index (512 entries)
//! - Bits 38-30: PDPT index (512 entries)
//! - Bits 29-21: PD index (512 entries)
//! - Bits 20-12: PT index (512 entries)
//! - Bits 11-0: Page offset (4096 bytes)
//! ```
//!
//! ## Page Table Entry Format (64-bit)
//!
//! ```text
//! 63  62-52  51-12         11-9   8    7    6    5    4    3    2    1    0
//! +---+------+-------------+-----+---+---+---+---+---+---+---+---+---+---+
//! |NX | Ign  | Phys Addr   | Ign |G |PAT| D | A |PCD|PWT|U/S|R/W| P |
//! +---+------+-------------+-----+---+---+---+---+---+---+---+---+---+---+
//!
//! - Bit 0 (P): Present - page is present in memory
//! - Bit 1 (R/W): Read/Write - 0=read-only, 1=read-write
//! - Bit 2 (U/S): User/Supervisor - 0=kernel, 1=user
//! - Bit 3 (PWT): Page-level Write-Through
//! - Bit 4 (PCD): Page-level Cache Disable
//! - Bit 5 (A): Accessed - set by CPU on read/write
//! - Bit 6 (D): Dirty - set by CPU on write
//! - Bit 7 (PAT): Page Attribute Table
//! - Bit 8 (G): Global - not flushed on CR3 write
//! - Bits 51-12: Physical frame address (4KB aligned)
//! - Bit 63 (NX): No Execute - prevent instruction fetch
//! ```
//!
//! ## Features
//!
//! - **4-level page tables**: Full PML4 → PDPT → PD → PT hierarchy
//! - **NX bit support**: Execute permissions via No-Execute bit
//! - **Global pages**: Kernel pages not flushed on context switch
//! - **Huge pages**: 2MB and 1GB page support (future)
//! - **COW support**: Software bit for copy-on-write
//! - **TLB management**: Selective and global TLB flushing
//!
//! ## Safety Considerations
//!
//! Page table manipulation is inherently unsafe:
//! - Incorrect mappings can cause page faults, data corruption, or security vulnerabilities
//! - TLB must be flushed after changing mappings
//! - Page tables must be properly aligned (4KB)
//! - Physical addresses must be valid
//! - Concurrent access must be synchronized
//!
//! ## Usage
//!
//! ```rust
//! // Create new page table manager
//! let mut ptm = PageTableManager::new()?;
//!
//! // Map a page
//! ptm.map_page(
//!     VirtAddr::new(0x1000),
//!     PhysAddr::new(0x4000),
//!     PageTableFlags::PRESENT | PageTableFlags::WRITABLE
//! )?;
//!
//! // Unmap a page
//! ptm.unmap_page(VirtAddr::new(0x1000))?;
//!
//! // Switch to new address space
//! unsafe {
//!     ptm.switch_to();
//! }
//! ```

use core::ptr::{read_volatile, write_volatile};
use x86_64::{
    structures::paging::{
        Page, PageTable, PageTableFlags, PageTableIndex,
        PhysFrame, Size4KiB,
    },
    VirtAddr, PhysAddr,
    registers::control::Cr3,
};

/// Physical memory offset for direct mapping
///
/// All physical memory is mapped at this offset in the kernel's virtual address space.
/// This allows easy conversion: phys_addr + PHYS_MEM_OFFSET = virt_addr
///
/// On x86_64, we use the -2GB region (0xFFFF_FFFF_8000_0000) for the direct map.
pub const PHYS_MEM_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;

/// Maximum physical memory supported (512 GB)
///
/// This is the size of the direct mapping region.
pub const MAX_PHYS_MEM: u64 = 512 * 1024 * 1024 * 1024; // 512 GB

/// Page size (4 KB)
pub const PAGE_SIZE: usize = 4096;

/// Page table level indices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageTableLevel {
    /// Page Map Level 4 (top level)
    PML4 = 4,
    /// Page Directory Pointer Table
    PDPT = 3,
    /// Page Directory
    PD = 2,
    /// Page Table (bottom level)
    PT = 1,
}

/// Page Table Manager
///
/// Manages the 4-level page table hierarchy and provides high-level operations
/// for mapping/unmapping pages.
pub struct PageTableManager {
    /// Physical address of the PML4 table
    pml4_phys: PhysAddr,
    /// Virtual address of the PML4 table
    pml4_virt: VirtAddr,
}

impl PageTableManager {
    /// Create a new page table manager using the current CR3
    ///
    /// This reads the currently active PML4 from CR3 and creates a manager
    /// that can manipulate the current address space.
    ///
    /// # Safety
    ///
    /// - Must be called after paging is enabled
    /// - Assumes direct mapping of physical memory is set up
    ///
    /// # Returns
    ///
    /// - `Ok(PageTableManager)` on success
    /// - `Err(&str)` if CR3 is invalid or direct mapping not available
    pub unsafe fn new() -> Result<Self, &'static str> {
        // Read current PML4 from CR3
        let (pml4_frame, _flags) = Cr3::read();
        let pml4_phys = pml4_frame.start_address();

        // Convert to virtual address via direct mapping
        let pml4_virt = phys_to_virt(pml4_phys);

        Ok(Self {
            pml4_phys,
            pml4_virt,
        })
    }

    /// Create a new empty address space
    ///
    /// Allocates a new PML4 table and returns a manager for it.
    /// The new address space is empty (no mappings).
    ///
    /// # Safety
    ///
    /// - Caller must have a frame allocator available
    /// - New page tables must be properly initialized before use
    ///
    /// # Returns
    ///
    /// - `Ok(PageTableManager)` with new PML4
    /// - `Err(&str)` if frame allocation fails
    pub unsafe fn new_address_space() -> Result<Self, &'static str> {
        // Allocate a new frame for PML4
        let pml4_frame = allocate_frame()?;
        let pml4_phys = pml4_frame.start_address();
        let pml4_virt = phys_to_virt(pml4_phys);

        // Zero out the new PML4
        let pml4 = &mut *(pml4_virt.as_u64() as *mut PageTable);
        pml4.zero();

        Ok(Self {
            pml4_phys,
            pml4_virt,
        })
    }

    /// Get reference to PML4 table
    unsafe fn pml4(&self) -> &PageTable {
        &*(self.pml4_virt.as_u64() as *const PageTable)
    }

    /// Get mutable reference to PML4 table
    unsafe fn pml4_mut(&mut self) -> &mut PageTable {
        &mut *(self.pml4_virt.as_u64() as *mut PageTable)
    }

    /// Map a single 4KB page
    ///
    /// Creates a mapping from virtual address to physical address with the given flags.
    /// This walks the page table hierarchy, creating intermediate tables as needed.
    ///
    /// # Arguments
    ///
    /// * `virt` - Virtual address to map (will be page-aligned down)
    /// * `phys` - Physical address to map to (will be page-aligned down)
    /// * `flags` - Page table flags (PRESENT, WRITABLE, USER, etc.)
    ///
    /// # Safety
    ///
    /// - Physical address must be valid
    /// - Caller must ensure no conflicting mappings
    /// - TLB must be flushed after mapping
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(&str)` if mapping fails (out of memory, invalid address, etc.)
    pub unsafe fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: PageTableFlags,
    ) -> Result<(), &'static str> {
        // Page-align addresses
        let virt = VirtAddr::new(virt.as_u64() & !0xFFF);
        let phys = PhysAddr::new(phys.as_u64() & !0xFFF);

        // Extract indices from virtual address
        let p4_index = PageTableIndex::new((virt.as_u64() >> 39) as u16 & 0x1FF);
        let p3_index = PageTableIndex::new((virt.as_u64() >> 30) as u16 & 0x1FF);
        let p2_index = PageTableIndex::new((virt.as_u64() >> 21) as u16 & 0x1FF);
        let p1_index = PageTableIndex::new((virt.as_u64() >> 12) as u16 & 0x1FF);

        // Walk/create page table hierarchy
        let p4 = self.pml4_mut();
        let p3 = Self::get_or_create_next_table(&mut p4[p4_index])?;
        let p2 = Self::get_or_create_next_table(&mut p3[p3_index])?;
        let p1 = Self::get_or_create_next_table(&mut p2[p2_index])?;

        // Check if already mapped
        if p1[p1_index].flags().contains(PageTableFlags::PRESENT) {
            return Err("Page already mapped");
        }

        // Set the PTE
        p1[p1_index].set_addr(phys, flags | PageTableFlags::PRESENT);

        // Flush TLB for this page
        flush_tlb_page(virt);

        Ok(())
    }

    /// Unmap a single 4KB page
    ///
    /// Removes the mapping for a virtual address. Does not free the physical frame.
    ///
    /// # Arguments
    ///
    /// * `virt` - Virtual address to unmap (will be page-aligned down)
    ///
    /// # Safety
    ///
    /// - Must not unmap kernel code/data currently in use
    /// - TLB must be flushed after unmapping
    ///
    /// # Returns
    ///
    /// - `Ok(PhysAddr)` - physical address that was unmapped
    /// - `Err(&str)` if page was not mapped
    pub unsafe fn unmap_page(&mut self, virt: VirtAddr) -> Result<PhysAddr, &'static str> {
        // Page-align address
        let virt = VirtAddr::new(virt.as_u64() & !0xFFF);

        // Extract indices
        let p4_index = PageTableIndex::new((virt.as_u64() >> 39) as u16 & 0x1FF);
        let p3_index = PageTableIndex::new((virt.as_u64() >> 30) as u16 & 0x1FF);
        let p2_index = PageTableIndex::new((virt.as_u64() >> 21) as u16 & 0x1FF);
        let p1_index = PageTableIndex::new((virt.as_u64() >> 12) as u16 & 0x1FF);

        // Walk page table hierarchy
        let p4 = self.pml4();
        let p3 = Self::get_next_table(&p4[p4_index])?;
        let p2 = Self::get_next_table(&p3[p3_index])?;
        let p1 = Self::get_next_table_mut(&mut self.pml4_mut()[p4_index])?;
        let p2_mut = Self::get_next_table_mut(&mut p1[p3_index])?;
        let p1_mut = Self::get_next_table_mut(&mut p2_mut[p2_index])?;

        // Check if mapped
        let entry = &mut p1_mut[p1_index];
        if !entry.flags().contains(PageTableFlags::PRESENT) {
            return Err("Page not mapped");
        }

        // Get physical address before clearing
        let phys = entry.addr();

        // Clear the PTE
        entry.set_unused();

        // Flush TLB for this page
        flush_tlb_page(virt);

        Ok(phys)
    }

    /// Translate virtual address to physical address
    ///
    /// Walks the page table hierarchy to find the physical address mapped to
    /// a virtual address.
    ///
    /// # Arguments
    ///
    /// * `virt` - Virtual address to translate
    ///
    /// # Returns
    ///
    /// - `Some(PhysAddr)` - physical address if mapped
    /// - `None` if not mapped or not present
    pub unsafe fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        // Extract indices
        let p4_index = PageTableIndex::new((virt.as_u64() >> 39) as u16 & 0x1FF);
        let p3_index = PageTableIndex::new((virt.as_u64() >> 30) as u16 & 0x1FF);
        let p2_index = PageTableIndex::new((virt.as_u64() >> 21) as u16 & 0x1FF);
        let p1_index = PageTableIndex::new((virt.as_u64() >> 12) as u16 & 0x1FF);
        let offset = virt.as_u64() & 0xFFF;

        // Walk page table hierarchy
        let p4 = self.pml4();
        let p3 = Self::get_next_table(&p4[p4_index]).ok()?;
        let p2 = Self::get_next_table(&p3[p3_index]).ok()?;
        let p1 = Self::get_next_table(&p2[p2_index]).ok()?;

        // Get physical address from PTE
        let entry = &p1[p1_index];
        if !entry.flags().contains(PageTableFlags::PRESENT) {
            return None;
        }

        Some(entry.addr() + offset)
    }

    /// Switch to this address space
    ///
    /// Loads this page table's PML4 into CR3, making it the active address space.
    ///
    /// # Safety
    ///
    /// - Page table must be properly initialized
    /// - Kernel mappings must be present
    /// - Stack must be mapped in new address space
    pub unsafe fn switch_to(&self) {
        use x86_64::registers::control::Cr3Flags;

        let frame = PhysFrame::containing_address(self.pml4_phys);
        Cr3::write(frame, Cr3Flags::empty());
    }

    /// Get or create next level page table
    ///
    /// If the entry is present, returns the existing table.
    /// If not present, allocates a new table and updates the entry.
    unsafe fn get_or_create_next_table(
        entry: &mut x86_64::structures::paging::PageTableEntry
    ) -> Result<&'static mut PageTable, &'static str> {
        if entry.flags().contains(PageTableFlags::PRESENT) {
            // Entry exists, return it
            let phys = entry.addr();
            let virt = phys_to_virt(phys);
            Ok(&mut *(virt.as_u64() as *mut PageTable))
        } else {
            // Allocate new table
            let frame = allocate_frame()?;
            let phys = frame.start_address();
            let virt = phys_to_virt(phys);

            // Zero the new table
            let table = &mut *(virt.as_u64() as *mut PageTable);
            table.zero();

            // Update entry
            entry.set_addr(
                phys,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER
            );

            Ok(table)
        }
    }

    /// Get next level page table (immutable)
    unsafe fn get_next_table(
        entry: &x86_64::structures::paging::PageTableEntry
    ) -> Result<&'static PageTable, &'static str> {
        if entry.flags().contains(PageTableFlags::PRESENT) {
            let phys = entry.addr();
            let virt = phys_to_virt(phys);
            Ok(&*(virt.as_u64() as *const PageTable))
        } else {
            Err("Page table entry not present")
        }
    }

    /// Get next level page table (mutable)
    unsafe fn get_next_table_mut(
        entry: &mut x86_64::structures::paging::PageTableEntry
    ) -> Result<&'static mut PageTable, &'static str> {
        if entry.flags().contains(PageTableFlags::PRESENT) {
            let phys = entry.addr();
            let virt = phys_to_virt(phys);
            Ok(&mut *(virt.as_u64() as *mut PageTable))
        } else {
            Err("Page table entry not present")
        }
    }
}

/// Convert physical address to virtual address
///
/// Uses the direct mapping at PHYS_MEM_OFFSET.
///
/// # Arguments
///
/// * `phys` - Physical address
///
/// # Returns
///
/// Virtual address in kernel space
pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    VirtAddr::new(phys.as_u64() + PHYS_MEM_OFFSET)
}

/// Convert virtual address to physical address
///
/// Only works for addresses in the direct mapping region.
///
/// # Arguments
///
/// * `virt` - Virtual address in direct mapping region
///
/// # Returns
///
/// - `Some(PhysAddr)` if in direct mapping region
/// - `None` if not in direct mapping region
pub fn virt_to_phys(virt: VirtAddr) -> Option<PhysAddr> {
    let addr = virt.as_u64();
    if addr >= PHYS_MEM_OFFSET && addr < PHYS_MEM_OFFSET + MAX_PHYS_MEM {
        Some(PhysAddr::new(addr - PHYS_MEM_OFFSET))
    } else {
        None
    }
}

/// Flush TLB entry for a single page
///
/// Invalidates the TLB entry for the specified virtual address.
///
/// # Arguments
///
/// * `virt` - Virtual address to flush
pub fn flush_tlb_page(virt: VirtAddr) {
    use x86_64::instructions::tlb;
    tlb::flush(Page::<Size4KiB>::containing_address(virt));
}

/// Flush entire TLB
///
/// Invalidates all TLB entries by reloading CR3.
/// This is slower than flushing individual pages but simpler.
pub fn flush_tlb_all() {
    use x86_64::instructions::tlb;
    tlb::flush_all();
}

/// Allocate a physical frame
///
/// This is a placeholder that will be replaced with proper frame allocator integration.
///
/// # Returns
///
/// - `Ok(PhysFrame)` on success
/// - `Err(&str)` if out of memory
unsafe fn allocate_frame() -> Result<PhysFrame<Size4KiB>, &'static str> {
    // Use the buddy allocator from mm subsystem
    let phys_addr = crate::mm::alloc_page()
        .ok_or("Out of physical memory")?;

    Ok(PhysFrame::containing_address(PhysAddr::new(phys_addr)))
}

/// Free a physical frame
///
/// Returns a frame to the buddy allocator.
///
/// # Arguments
///
/// * `frame` - Physical frame to free
unsafe fn free_frame(frame: PhysFrame<Size4KiB>) {
    let phys_addr = frame.start_address().as_u64();
    crate::mm::free_page(phys_addr);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phys_virt_conversion() {
        let phys = PhysAddr::new(0x1000);
        let virt = phys_to_virt(phys);
        assert_eq!(virt.as_u64(), 0x1000 + PHYS_MEM_OFFSET);

        let phys_back = virt_to_phys(virt);
        assert_eq!(phys_back, Some(phys));
    }

    #[test]
    fn test_page_table_indices() {
        let virt = VirtAddr::new(0xFFFF_8000_0000_1234);

        let p4_index = (virt.as_u64() >> 39) as u16 & 0x1FF;
        let p3_index = (virt.as_u64() >> 30) as u16 & 0x1FF;
        let p2_index = (virt.as_u64() >> 21) as u16 & 0x1FF;
        let p1_index = (virt.as_u64() >> 12) as u16 & 0x1FF;
        let offset = virt.as_u64() & 0xFFF;

        // Verify indices are extracted correctly
        assert_eq!(p4_index, 256); // Bit 47 is set
        assert_eq!(offset, 0x234);
    }
}
