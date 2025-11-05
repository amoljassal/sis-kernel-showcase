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
