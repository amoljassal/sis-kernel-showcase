//! Phase 0: Boot - Architecture-specific early initialization
//!
//! This module handles the earliest boot phase:
//! - Stack setup
//! - Exception vector installation
//! - MMU/Paging enablement
//! - PMU (Performance Monitoring Unit) initialization
//!
//! # Safety
//!
//! All functions in this module are unsafe and must be called exactly once
//! during early boot before any other initialization.

use super::{InitError, InitResult};
use core::arch::asm;

/// Bootstrap stack size (64 KiB, 16-byte aligned)
const STACK_SIZE: usize = 64 * 1024;

/// Page table size for L1 translation table
const L1_TABLE_ENTRIES: usize = 512;

/// 64 KiB bootstrap stack (16-byte aligned)
#[repr(C, align(16))]
struct Stack([u8; STACK_SIZE]);
static mut BOOT_STACK: Stack = Stack([0; STACK_SIZE]);

/// Level-1 translation table (4 KiB aligned)
#[repr(C, align(4096))]
struct Table512([u64; L1_TABLE_ENTRIES]);
static mut L1_TABLE: Table512 = Table512([0; L1_TABLE_ENTRIES]);

/// Exception vector table (imported from assembly)
extern "C" {
    static VECTORS: u8;
}

/// Initialize boot stack
///
/// # Safety
/// Must be called once during early boot with no stack set up yet
pub unsafe fn init_stack() -> InitResult<()> {
    let stack_ptr = &raw const BOOT_STACK.0;
    let sp_top = stack_ptr.cast::<u8>().add((*stack_ptr).len()) as u64;

    asm!(
        "mov sp, {sp}",
        sp = in(reg) sp_top,
        options(nostack, preserves_flags)
    );

    Ok(())
}

/// Install exception vectors
///
/// # Safety
/// Must be called once after stack is initialized
pub unsafe fn init_exception_vectors() -> InitResult<()> {
    let base = &VECTORS as *const u8 as u64;

    // Detect current exception level
    let current_el: u64;
    asm!("mrs {el}, CurrentEL", el = out(reg) current_el);

    match (current_el >> 2) & 0x3 {
        1 => asm!(
            "msr VBAR_EL1, {v}",
            v = in(reg) base,
            options(nostack, preserves_flags)
        ),
        2 => asm!(
            "msr VBAR_EL2, {v}",
            v = in(reg) base,
            options(nostack, preserves_flags)
        ),
        _ => return Err(InitError::BootFailed),
    }

    asm!("isb", options(nostack, preserves_flags));

    Ok(())
}

/// Enable MMU (Memory Management Unit) for EL1
///
/// # Safety
/// Must be called once after exception vectors are installed
pub unsafe fn enable_mmu() -> InitResult<()> {
    // Check we're at EL1
    let current_el: u64;
    asm!("mrs {el}, CurrentEL", el = out(reg) current_el);
    let el = (current_el >> 2) & 0x3;

    if el != 1 {
        // Skip MMU setup if not at EL1
        return Ok(());
    }

    // Set up MAIR (Memory Attribute Indirection Register)
    // AttrIdx0 = Device-nGnRE (0x04)
    // AttrIdx1 = Normal WBWA (0xFF)
    let mair = (0x04u64) | (0xFFu64 << 8);
    asm!(
        "msr MAIR_EL1, {x}",
        x = in(reg) mair,
        options(nostack, preserves_flags)
    );

    // Set up TCR (Translation Control Register)
    // - 4KB pages
    // - Inner/Outer Write-Back Write-Allocate
    // - Inner Shareable
    // - 39-bit VA (T0SZ=25)
    // - 48-bit PA (IPS=5)
    let t0sz: u64 = 64 - 39; // 25
    let tcr = (t0sz & 0x3Fu64)
        | (0b01u64 << 8)  // IRGN0 = WBWA
        | (0b01u64 << 10) // ORGN0 = WBWA
        | (0b11u64 << 12) // SH0 = Inner Shareable
        | (0b00u64 << 14) // TG0 = 4KB
        | (0b101u64 << 32); // IPS = 48-bit PA

    asm!(
        "msr TCR_EL1, {x}",
        x = in(reg) tcr,
        options(nostack, preserves_flags)
    );
    asm!("isb", options(nostack, preserves_flags));

    // Build translation tables
    build_page_tables();

    // Set TTBR0 to L1 table
    let l1_pa = &raw const L1_TABLE.0 as *const _ as u64;
    asm!(
        "msr TTBR0_EL1, {x}",
        x = in(reg) l1_pa,
        options(nostack, preserves_flags)
    );
    asm!("dsb ish; isb", options(nostack, preserves_flags));

    // Enable MMU + caches in SCTLR_EL1
    let mut sctlr: u64;
    asm!("mrs {x}, SCTLR_EL1", x = out(reg) sctlr);
    sctlr |= (1 << 0)  // M (MMU enable)
        | (1 << 2)     // C (data cache enable)
        | (1 << 12);   // I (instruction cache enable)
    asm!("msr SCTLR_EL1, {x}", x = in(reg) sctlr);
    asm!("isb", options(nostack, preserves_flags));

    Ok(())
}

/// Build page tables for identity mapping
///
/// Maps RAM and MMIO regions using 1GiB block descriptors
unsafe fn build_page_tables() {
    // Clear all table entries
    let table_ptr = &raw mut L1_TABLE.0 as *mut [u64; L1_TABLE_ENTRIES];
    for e in (*table_ptr).iter_mut() {
        *e = 0;
    }

    // Descriptor bit definitions
    const DESC_BLOCK: u64 = 1;           // bits[1:0]=01 for block
    const SH_INNER: u64 = 0b11 << 8;    // Inner Shareable
    const AF: u64 = 1 << 10;             // Access Flag
    const ATTRIDX_NORMAL: u64 = 1 << 2; // AttrIndx=1 (Normal WBWA)
    const ATTRIDX_DEVICE: u64 = 0 << 2; // AttrIndx=0 (Device-nGnRE)

    // Map RAM ranges as Normal WBWA, Inner Shareable
    let plat = crate::platform::active();
    for r in plat.ram_ranges() {
        let mut base = (r.start as u64) & !((1u64 << 30) - 1); // Align to 1GiB
        let end = (r.start as u64).saturating_add(r.size as u64);

        while base < end {
            let idx = (base >> 30) as usize;
            if idx < L1_TABLE_ENTRIES {
                L1_TABLE.0[idx] = base | DESC_BLOCK | AF | SH_INNER | ATTRIDX_NORMAL;
            }
            base = base.saturating_add(1u64 << 30);
        }
    }

    // Map MMIO ranges as Device-nGnRE
    for m in plat.mmio_ranges() {
        let mut base = (m.start as u64) & !((1u64 << 30) - 1); // Align to 1GiB
        let end = (m.start as u64).saturating_add(m.size as u64);

        while base < end {
            let idx = (base >> 30) as usize;
            if idx < L1_TABLE_ENTRIES && L1_TABLE.0[idx] == 0 {
                // Only map if not already mapped as RAM
                L1_TABLE.0[idx] = base | DESC_BLOCK | AF | ATTRIDX_DEVICE;
            }
            base = base.saturating_add(1u64 << 30);
        }
    }
}

/// Initialize Performance Monitoring Unit
///
/// # Safety
/// Must be called after MMU is enabled
pub unsafe fn init_pmu() -> InitResult<()> {
    // PMCR_EL0: E=1 (enable), P=1 (reset event counters), C=1 (reset cycle counter)
    let pmcr: u64 = (1 << 0) | (1 << 1) | (1 << 2);
    asm!(
        "msr PMCR_EL0, {x}",
        x = in(reg) pmcr,
        options(nostack, preserves_flags)
    );

    // Enable cycle counter in PMCNTENSET_EL0 (bit 31)
    let pmcntenset: u64 = 1u64 << 31;
    asm!(
        "msr PMCNTENSET_EL0, {x}",
        x = in(reg) pmcntenset,
        options(nostack, preserves_flags)
    );

    // Allow EL0 reads (for future userspace access)
    let pmuserenr: u64 = 1; // EN=1
    asm!(
        "msr PMUSERENR_EL0, {x}",
        x = in(reg) pmuserenr,
        options(nostack, preserves_flags)
    );

    asm!("isb", options(nostack, preserves_flags));

    Ok(())
}

/// Run all boot phase initialization
///
/// # Safety
/// Must be called once during early boot
pub unsafe fn init_boot_phase() -> InitResult<()> {
    init_stack()?;
    init_exception_vectors()?;
    enable_mmu()?;
    init_pmu()?;
    Ok(())
}
