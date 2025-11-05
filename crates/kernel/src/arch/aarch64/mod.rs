// AArch64 architecture-specific code
// Phase A0 - Exception handling and syscall infrastructure

pub mod trap;
pub mod timer;

// TODO: Add in later phases:
// pub mod psci;    // SMP CPU bring-up (Phase E)
// pub mod gicv3;   // Interrupt controller (Phase E)
// pub mod mmu;     // Page tables, TLB (Phase A1)

pub use trap::*;

/// Flush instruction cache for entire system
/// Required after writing executable code to memory
#[inline]
pub fn flush_icache_all() {
    unsafe {
        core::arch::asm!(
            "dsb ish",      // Data Synchronization Barrier
            "ic iallu",     // Invalidate all instruction caches to PoU
            "dsb ish",      // Ensure completion
            "isb",          // Instruction Synchronization Barrier
            options(nostack, preserves_flags)
        );
    }
}

/// Flush instruction cache for a specific address range
/// addr: starting virtual address
/// len: length in bytes
#[inline]
pub fn flush_icache_range(addr: u64, len: usize) {
    // IC IVAU operates on cache lines (typically 64 bytes)
    const CACHE_LINE_SIZE: usize = 64;

    let start = addr & !(CACHE_LINE_SIZE as u64 - 1); // Align down
    let end = (addr + len as u64 + CACHE_LINE_SIZE as u64 - 1) & !(CACHE_LINE_SIZE as u64 - 1);

    unsafe {
        core::arch::asm!("dsb ish", options(nostack, preserves_flags));

        let mut current = start;
        while current < end {
            core::arch::asm!(
                "ic ivau, {addr}",  // Invalidate instruction cache by VA to PoU
                addr = in(reg) current,
                options(nostack, preserves_flags)
            );
            current += CACHE_LINE_SIZE as u64;
        }

        core::arch::asm!(
            "dsb ish",
            "isb",
            options(nostack, preserves_flags)
        );
    }
}
