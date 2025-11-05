/// Page table management and PTE flag definitions
///
/// AArch64 page table format with support for NX (Execute-Never),
/// copy-on-write, and user/kernel separation.

use crate::lib::error::KernelError;

/// Page size (4KB)
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

/// Kernel/User address space boundary
pub const KERNEL_BASE: u64 = 0xFFFF_0000_0000_0000;

bitflags::bitflags! {
    /// Page table entry flags (AArch64 format)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PteFlags: u64 {
        /// Valid entry
        const VALID = 1 << 0;
        /// Table descriptor (vs block/page)
        const TABLE = 1 << 1;
        /// User accessible
        const USER = 1 << 6;
        /// Read-only (when clear, read-write)
        const READONLY = 1 << 7;
        /// Shareable
        const SHARED = 1 << 8 | 1 << 9;
        /// Access flag (must be set)
        const ACCESS = 1 << 10;
        /// Not global
        const NOT_GLOBAL = 1 << 11;
        /// Execute-never for unprivileged (EL0)
        const UXN = 1 << 54;
        /// Privileged execute-never (EL1)
        const PXN = 1 << 53;
        /// Copy-on-write (software bit)
        const COW = 1 << 55;
    }
}

impl PteFlags {
    /// Create flags for user read-only page
    pub fn user_ro() -> Self {
        Self::VALID | Self::USER | Self::READONLY | Self::ACCESS | Self::NOT_GLOBAL | Self::UXN
    }

    /// Create flags for user read-write page
    pub fn user_rw() -> Self {
        Self::VALID | Self::USER | Self::ACCESS | Self::NOT_GLOBAL | Self::UXN
    }

    /// Create flags for user executable page (read-only, no write)
    pub fn user_rx() -> Self {
        Self::VALID | Self::USER | Self::READONLY | Self::ACCESS | Self::NOT_GLOBAL
    }

    /// Create flags for COW page (user, read-only, with COW bit)
    pub fn user_cow() -> Self {
        Self::VALID | Self::USER | Self::READONLY | Self::ACCESS | Self::NOT_GLOBAL | Self::UXN | Self::COW
    }

    /// Check if page is copy-on-write
    pub fn is_cow(&self) -> bool {
        self.contains(Self::COW)
    }

    /// Check if page is writable
    pub fn is_writable(&self) -> bool {
        !self.contains(Self::READONLY)
    }

    /// Check if page is executable
    pub fn is_executable(&self) -> bool {
        !self.contains(Self::UXN)
    }

    /// Mark page as COW (read-only with COW bit)
    pub fn mark_cow(&mut self) {
        self.insert(Self::READONLY | Self::COW);
    }

    /// Clear COW bit and make writable
    pub fn clear_cow(&mut self) {
        self.remove(Self::COW | Self::READONLY);
    }
}

/// Page table entry
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Pte(u64);

impl Pte {
    /// Create an invalid PTE
    pub const fn invalid() -> Self {
        Self(0)
    }

    /// Create a PTE from physical address and flags
    pub fn new(phys_addr: u64, flags: PteFlags) -> Self {
        let addr_mask = 0x0000_FFFF_FFFF_F000; // Bits [47:12]
        Self((phys_addr & addr_mask) | flags.bits())
    }

    /// Check if PTE is valid
    pub fn is_valid(&self) -> bool {
        (self.0 & PteFlags::VALID.bits()) != 0
    }

    /// Get physical address from PTE
    pub fn phys_addr(&self) -> u64 {
        self.0 & 0x0000_FFFF_FFFF_F000
    }

    /// Get flags from PTE
    pub fn flags(&self) -> PteFlags {
        PteFlags::from_bits_truncate(self.0)
    }

    /// Set flags
    pub fn set_flags(&mut self, flags: PteFlags) {
        let addr = self.phys_addr();
        self.0 = addr | flags.bits();
    }
}

impl core::fmt::Debug for Pte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Pte")
            .field("phys", &format_args!("{:#x}", self.phys_addr()))
            .field("flags", &self.flags())
            .finish()
    }
}

/// Page table (512 entries for 4KB pages)
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [Pte; 512],
}

impl PageTable {
    /// Create a new empty page table
    pub const fn new() -> Self {
        Self {
            entries: [Pte::invalid(); 512],
        }
    }

    /// Get the index for a virtual address at a given level
    pub fn index(virt_addr: u64, level: usize) -> usize {
        ((virt_addr >> (12 + 9 * (3 - level))) & 0x1FF) as usize
    }
}

// Note: alloc_page, alloc_pages, free_page, free_pages are provided by buddy.rs
// and re-exported from mm/mod.rs

/// Map a virtual page to a physical page
pub fn map_page(
    page_table: &mut PageTable,
    virt_addr: u64,
    phys_addr: u64,
    flags: PteFlags,
) -> Result<(), KernelError> {
    // This is a simplified single-level mapping
    // Real implementation would walk/allocate multi-level page tables
    let idx = PageTable::index(virt_addr, 3);
    page_table.entries[idx] = Pte::new(phys_addr, flags);
    Ok(())
}

/// Unmap a virtual page
pub fn unmap_page(page_table: &mut PageTable, virt_addr: u64) {
    let idx = PageTable::index(virt_addr, 3);
    page_table.entries[idx] = Pte::invalid();
}

/// Flush TLB for a specific address
#[inline]
pub fn flush_tlb(virt_addr: u64) {
    unsafe {
        core::arch::asm!(
            "dsb ishst",
            "tlbi vaae1is, {addr}",
            "dsb ish",
            "isb",
            addr = in(reg) virt_addr >> 12,
            options(nostack)
        );
    }
}

/// Flush entire TLB
#[inline]
pub fn flush_tlb_all() {
    unsafe {
        core::arch::asm!(
            "dsb ishst",
            "tlbi vmalle1is",
            "dsb ish",
            "isb",
            options(nostack)
        );
    }
}

/// Allocate a new zero-filled page table
fn alloc_page_table() -> Option<*mut PageTable> {
    let phys_addr = super::alloc_page()?;
    let ptr = phys_addr as *mut PageTable;

    // Zero-fill the page table
    unsafe {
        core::ptr::write_bytes(ptr, 0, 1);
    }

    Some(ptr)
}

/// Walk page table and get PTE for virtual address, allocating tables as needed
/// Returns mutable reference to the L3 (page) PTE
fn walk_page_table(
    root: *mut PageTable,
    virt_addr: u64,
    alloc: bool,
) -> Result<*mut Pte, KernelError> {
    let mut table = root;

    // Walk levels 0-2 (tables)
    for level in 0..3 {
        let idx = PageTable::index(virt_addr, level);

        unsafe {
            let pte = &mut (*table).entries[idx];

            if !pte.is_valid() {
                if !alloc {
                    return Err(KernelError::NotFound);
                }

                // Allocate new page table
                let new_table = alloc_page_table().ok_or(KernelError::OutOfMemory)?;
                let new_phys = new_table as u64;

                // Set table descriptor: VALID | TABLE | USER
                let table_flags = PteFlags::VALID | PteFlags::TABLE | PteFlags::USER;
                *pte = Pte::new(new_phys, table_flags);
            }

            // Follow to next level
            table = pte.phys_addr() as *mut PageTable;
        }
    }

    // Return pointer to L3 (page level) PTE
    let idx = PageTable::index(virt_addr, 3);
    unsafe {
        Ok(&mut (*table).entries[idx] as *mut Pte)
    }
}

/// Map a user virtual page to physical page with given flags
pub fn map_user_page(
    root: *mut PageTable,
    virt_addr: u64,
    phys_addr: u64,
    mut flags: PteFlags,
) -> Result<(), KernelError> {
    // Enforce user bit and no W+X
    flags.insert(PteFlags::USER);
    if flags.contains(PteFlags::VALID) && !flags.contains(PteFlags::READONLY) {
        // Writable, so must not be executable
        flags.insert(PteFlags::UXN);
    }

    // Get or allocate PTE
    let pte_ptr = walk_page_table(root, virt_addr, true)?;

    unsafe {
        *pte_ptr = Pte::new(phys_addr, flags);
    }

    // Flush TLB for this address
    flush_tlb(virt_addr);

    Ok(())
}

/// Unmap a user virtual page
pub fn unmap_user_page(
    root: *mut PageTable,
    virt_addr: u64,
) -> Result<(), KernelError> {
    let pte_ptr = walk_page_table(root, virt_addr, false)?;

    unsafe {
        *pte_ptr = Pte::invalid();
    }

    flush_tlb(virt_addr);

    Ok(())
}

/// Get PTE for a virtual address (without allocating)
pub fn get_pte(
    root: *mut PageTable,
    virt_addr: u64,
) -> Result<Pte, KernelError> {
    let pte_ptr = walk_page_table(root, virt_addr, false)?;
    unsafe {
        Ok(*pte_ptr)
    }
}

/// Get mutable PTE for a virtual address (without allocating tables)
pub fn get_pte_mut(
    root: *mut PageTable,
    virt_addr: u64,
) -> Result<*mut Pte, KernelError> {
    walk_page_table(root, virt_addr, false)
}

/// Switch to user address space (sets TTBR0_EL1)
#[inline]
pub fn switch_user_mm(ttbr0: u64) {
    unsafe {
        core::arch::asm!(
            "msr ttbr0_el1, {ttbr0}",  // Set TTBR0_EL1
            "dsb ish",                   // Ensure write completes
            "tlbi vmalle1is",            // Invalidate all TLB entries for EL1
            "dsb ish",                   // Ensure TLB invalidation completes
            "isb",                       // Instruction barrier
            ttbr0 = in(reg) ttbr0,
            options(nostack)
        );
    }
}

/// Allocate a new page table root for user space
pub fn alloc_user_page_table() -> Result<u64, KernelError> {
    let ptr = alloc_page_table().ok_or(KernelError::OutOfMemory)?;
    Ok(ptr as u64)
}

