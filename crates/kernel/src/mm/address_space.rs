/// Virtual memory area (VMA) management
///
/// Manages the virtual address space of a process, including
/// memory mappings for text, data, heap, stack, and memory-mapped files.

use crate::process::{Vma, VmaFlags, MemoryManager};
use crate::lib::error::{KernelError, Errno};
use super::paging::{PAGE_SIZE, KERNEL_BASE};
use alloc::vec::Vec;

/// User address space layout constants
pub const USER_STACK_TOP: u64 = 0x0000_7FFF_FFFF_F000;
pub const USER_STACK_SIZE: u64 = 8 * 1024 * 1024; // 8MB default
pub const USER_HEAP_START: u64 = 0x0000_5555_5600_0000;
pub const USER_MMAP_BASE: u64 = 0x0000_7000_0000_0000;

impl MemoryManager {
    /// Create a new empty address space
    pub fn new() -> Self {
        Self {
            page_table: 0,
            brk: USER_HEAP_START,
            brk_start: USER_HEAP_START,
            stack_top: USER_STACK_TOP,
            mmap_base: USER_MMAP_BASE,
            vmas: Vec::new(),
        }
    }

    /// Find VMA containing the given address
    pub fn find_vma(&self, addr: u64) -> Option<&Vma> {
        self.vmas.iter().find(|vma| addr >= vma.start && addr < vma.end)
    }

    /// Find VMA containing the given address (mutable)
    pub fn find_vma_mut(&mut self, addr: u64) -> Option<&mut Vma> {
        self.vmas.iter_mut().find(|vma| addr >= vma.start && addr < vma.end)
    }

    /// Check if a region overlaps with existing VMAs
    pub fn overlaps(&self, start: u64, end: u64) -> bool {
        self.vmas.iter().any(|vma| {
            !(end <= vma.start || start >= vma.end)
        })
    }

    /// Insert a new VMA (sorted by start address)
    pub fn insert_vma(&mut self, vma: Vma) -> Result<(), KernelError> {
        // Check for overlaps
        if self.overlaps(vma.start, vma.end) {
            return Err(KernelError::InvalidArgument);
        }

        // Find insertion point to keep VMAs sorted
        let pos = self.vmas.iter().position(|v| v.start > vma.start)
            .unwrap_or(self.vmas.len());

        self.vmas.insert(pos, vma);
        Ok(())
    }

    /// Remove a VMA
    pub fn remove_vma(&mut self, start: u64) -> Option<Vma> {
        if let Some(pos) = self.vmas.iter().position(|v| v.start == start) {
            Some(self.vmas.remove(pos))
        } else {
            None
        }
    }

    /// Extend the heap (brk syscall)
    pub fn do_brk(&mut self, new_brk: u64) -> Result<u64, Errno> {
        // Validate new brk
        if new_brk < self.brk_start {
            return Err(Errno::EINVAL);
        }

        // Check if new brk overlaps with other VMAs
        if new_brk > self.brk {
            // Expanding heap
            if self.overlaps(self.brk, new_brk) {
                return Err(Errno::ENOMEM);
            }
        }

        // Update brk
        self.brk = new_brk;
        Ok(self.brk)
    }

    /// Map anonymous memory (mmap syscall)
    pub fn do_mmap(
        &mut self,
        addr: u64,
        length: u64,
        prot: i32,
        flags: i32,
    ) -> Result<u64, Errno> {
        // Round up length to page size
        let length = (length + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1);

        // Determine start address
        let start = if addr != 0 {
            // Fixed address requested
            addr
        } else {
            // Find free space
            self.find_free_region(length)?
        };

        // Convert prot flags to VmaFlags
        let mut vma_flags = VmaFlags::ANONYMOUS;
        if (prot & 0x1) != 0 { vma_flags |= VmaFlags::READ; }
        if (prot & 0x2) != 0 { vma_flags |= VmaFlags::WRITE; }
        if (prot & 0x4) != 0 { vma_flags |= VmaFlags::EXEC; }
        if (flags & 0x01) != 0 { vma_flags |= VmaFlags::SHARED; }

        // Create VMA
        let vma = Vma {
            start,
            end: start + length,
            flags: vma_flags,
            offset: 0,
        };

        self.insert_vma(vma).map_err(|_| Errno::ENOMEM)?;
        Ok(start)
    }

    /// Unmap memory region (munmap syscall)
    pub fn do_munmap(&mut self, addr: u64, length: u64) -> Result<(), Errno> {
        // Round up length to page size
        let length = (length + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1);
        let end = addr + length;

        // Find and remove overlapping VMAs
        self.vmas.retain(|vma| {
            // Keep VMA if it doesn't overlap
            !(addr < vma.end && end > vma.start)
        });

        // TODO: Actually unmap pages from page table
        Ok(())
    }

    /// Find a free region of the given size
    fn find_free_region(&self, size: u64) -> Result<u64, Errno> {
        let mut addr = self.mmap_base;

        for vma in &self.vmas {
            if vma.start >= self.mmap_base {
                if vma.start - addr >= size {
                    return Ok(addr);
                }
                addr = vma.end;
            }
        }

        // Check if we have space after the last VMA
        if KERNEL_BASE - addr >= size {
            Ok(addr)
        } else {
            Err(Errno::ENOMEM)
        }
    }

    /// Set up initial user stack
    pub fn setup_stack(&mut self) -> Result<(), KernelError> {
        let stack_start = self.stack_top - USER_STACK_SIZE;
        let vma = Vma {
            start: stack_start,
            end: self.stack_top,
            flags: VmaFlags::READ | VmaFlags::WRITE | VmaFlags::ANONYMOUS,
            offset: 0,
        };
        self.insert_vma(vma)?;
        Ok(())
    }
}
