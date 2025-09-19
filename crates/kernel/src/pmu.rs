//! AArch64 PMU minimal helpers (feature-gated).
//! Configures two event counters and reads snapshots.

#[cfg(target_arch = "aarch64")]
pub mod aarch64 {
    #[inline(always)]
    unsafe fn write_pmselelr(idx: u64) {
        core::arch::asm!("msr PMSELR_EL0, {x}", x = in(reg) idx, options(nostack, preserves_flags));
        core::arch::asm!("isb", options(nostack, preserves_flags));
    }

    #[inline(always)]
    unsafe fn write_pmxevtyper(ev: u64) {
        core::arch::asm!("msr PMXEVTYPER_EL0, {x}", x = in(reg) ev, options(nostack, preserves_flags));
        core::arch::asm!("isb", options(nostack, preserves_flags));
    }

    #[inline(always)]
    unsafe fn zero_pmxevcntr() {
        let z: u64 = 0;
        core::arch::asm!("msr PMXEVCNTR_EL0, {x}", x = in(reg) z, options(nostack, preserves_flags));
    }

    #[inline(always)]
    unsafe fn read_pmxevcntr() -> u64 {
        let v: u64;
        core::arch::asm!("mrs {x}, PMXEVCNTR_EL0", x = out(reg) v, options(nostack, preserves_flags));
        v
    }

    #[inline(always)]
    unsafe fn read_pmccntr() -> u64 {
        let v: u64;
        core::arch::asm!("mrs {x}, PMCCNTR_EL0", x = out(reg) v, options(nostack, preserves_flags));
        v
    }

    #[inline(always)]
    unsafe fn enable_counters(mask: u64) {
        core::arch::asm!("msr PMCNTENSET_EL0, {x}", x = in(reg) mask, options(nostack, preserves_flags));
        core::arch::asm!("isb", options(nostack, preserves_flags));
    }

    /// Configure counter 0 = INST_RETIRED (0x08), counter 1 = L1D_CACHE_REFILL (0x03)
    pub unsafe fn setup_events() {
        // Reset event counters and cycle
        let pmcr: u64 = (1 << 0) | (1 << 1) | (1 << 2);
        core::arch::asm!("msr PMCR_EL0, {x}", x = in(reg) pmcr, options(nostack, preserves_flags));

        // Counter 0
        write_pmselelr(0);
        write_pmxevtyper(0x08);
        zero_pmxevcntr();

        // Counter 1
        write_pmselelr(1);
        write_pmxevtyper(0x03);
        zero_pmxevcntr();

        // Enable counters 0,1 and cycle counter (bit31)
        let mask = (1u64 << 0) | (1u64 << 1) | (1u64 << 31);
        enable_counters(mask);
    }

    #[derive(Copy, Clone, Default)]
    pub struct Snapshot {
        pub cycles: u64,
        pub inst: u64,
        pub l1d_refill: u64,
    }

    pub unsafe fn read_snapshot() -> Snapshot {
        // Read PMCCNTR (cycles)
        let cycles = read_pmccntr();

        // Counter 0
        write_pmselelr(0);
        let inst = read_pmxevcntr();

        // Counter 1
        write_pmselelr(1);
        let l1d = read_pmxevcntr();

        Snapshot { cycles, inst, l1d_refill: l1d }
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub mod aarch64 {
    #[derive(Copy, Clone, Default)]
    pub struct Snapshot { pub cycles: u64, pub inst: u64, pub l1d_refill: u64 }
    pub unsafe fn setup_events() {}
    pub unsafe fn read_snapshot() -> Snapshot { Snapshot::default() }
}

