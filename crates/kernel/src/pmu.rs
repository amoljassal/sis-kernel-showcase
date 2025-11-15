//! ARM Performance Monitoring Unit (PMU) support
//!
//! Provides access to the ARM PMU for performance monitoring and profiling.
//! Supports cycle counters, event counters, and performance snapshots.
//!
//! # Features
//!
//! - Cycle counter (PMCCNTR_EL0)
//! - 6 programmable event counters
//! - Predefined events (instructions, cache misses, branches, etc.)
//! - Performance snapshots
//! - Shell command integration
//!
//! # Usage
//!
//! ```rust
//! // Initialize PMU (call once during boot)
//! unsafe { crate::pmu::init(); }
//!
//! // Read performance snapshot
//! let snap = crate::pmu::read_snapshot()?;
//! println!("Cycles: {}, Instructions: {}", snap.cycles, snap.inst);
//! ```
//!
//! # References
//!
//! - ARM Architecture Reference Manual (PMU chapter)
//! - ARM Cortex-A76 Core Technical Reference Manual
//!
//! # M8 Hardening
//!
//! All public functions return DriverResult for proper error handling.
//! Counter indices are validated (0-5 for event counters).

use core::sync::atomic::{AtomicBool, Ordering};
use crate::drivers::{DriverError, DriverResult, Validator};

/// Maximum event counter index (counters 0-5)
const MAX_EVENT_COUNTER: u64 = 5;

/// PMU initialization state
static PMU_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[cfg(target_arch = "aarch64")]
pub mod aarch64 {
    use super::PMU_INITIALIZED;
    use super::{DriverError, DriverResult, Validator};
    use core::sync::atomic::Ordering;
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

    /// ARM PMU Event types (common events for Cortex-A76)
    #[repr(u64)]
    #[derive(Debug, Copy, Clone)]
    pub enum PmuEvent {
        /// Software increment
        SwIncr = 0x00,
        /// L1 instruction cache refill
        L1ICache = 0x01,
        /// L1 instruction TLB refill
        L1ITLB = 0x02,
        /// L1 data cache refill
        L1DCache = 0x03,
        /// L1 data cache access
        L1DCacheAccess = 0x04,
        /// L1 data TLB refill
        L1DTLB = 0x05,
        /// Instruction architecturally executed
        InstRetired = 0x08,
        /// Exception taken
        ExcTaken = 0x09,
        /// Exception return
        ExcReturn = 0x0A,
        /// Software change of PC
        PcWrite = 0x0C,
        /// Immediate branch
        BranchImm = 0x0D,
        /// Procedure return
        ProcReturn = 0x0E,
        /// Unaligned load/store
        UnalignedAccess = 0x0F,
        /// Branch mispredicted
        BranchMispred = 0x10,
        /// CPU cycles
        CpuCycles = 0x11,
        /// Predictable branch
        BranchPred = 0x12,
        /// Data memory access
        MemAccess = 0x13,
        /// L2 data cache access
        L2DCache = 0x16,
        /// L2 data cache refill
        L2DCacheRefill = 0x17,
    }

    /// Initialize PMU
    ///
    /// Enables user-space access (optional), resets all counters, and configures
    /// default event counters.
    ///
    /// # Safety
    ///
    /// Must be called once during kernel initialization.
    pub unsafe fn init() {
        if PMU_INITIALIZED.swap(true, Ordering::AcqRel) {
            // Already initialized
            return;
        }

        crate::info!("PMU: Initializing ARM Performance Monitoring Unit");

        // Enable user-mode access to PMU (optional - for debugging)
        // Set PMUSERENR_EL0 to 0x0F to allow user access
        // For production, keep at 0 to restrict to kernel only
        let user_access: u64 = 0x00; // 0x0F to enable user access
        core::arch::asm!("msr PMUSERENR_EL0, {x}", x = in(reg) user_access, options(nostack, preserves_flags));

        // Reset all counters: E=1 (enable), P=1 (reset event counters), C=1 (reset cycle counter)
        let pmcr: u64 = (1 << 0) | (1 << 1) | (1 << 2);
        core::arch::asm!("msr PMCR_EL0, {x}", x = in(reg) pmcr, options(nostack, preserves_flags));
        core::arch::asm!("isb", options(nostack, preserves_flags));

        // Configure event counters
        setup_default_events();

        crate::info!("PMU: Performance monitoring enabled");
        crate::info!("PMU: Cycle counter and 6 event counters active");
    }

    /// Configure default event counters
    ///
    /// - Counter 0: Instructions retired (INST_RETIRED)
    /// - Counter 1: L1 data cache refill (L1D_CACHE_REFILL)
    /// - Counter 2: Branch mispredictions (BRANCH_MISPRED)
    /// - Counter 3: L2 data cache access (L2D_CACHE)
    /// - Counter 4: L1 instruction cache refill (L1I_CACHE)
    /// - Counter 5: Exception taken (EXC_TAKEN)
    unsafe fn setup_default_events() {
        // Configure counters
        configure_event_counter(0, PmuEvent::InstRetired);
        configure_event_counter(1, PmuEvent::L1DCache);
        configure_event_counter(2, PmuEvent::BranchMispred);
        configure_event_counter(3, PmuEvent::L2DCache);
        configure_event_counter(4, PmuEvent::L1ICache);
        configure_event_counter(5, PmuEvent::ExcTaken);

        // Enable all counters (0-5) and cycle counter (bit 31)
        let mask = (1u64 << 0) | (1u64 << 1) | (1u64 << 2) |
                   (1u64 << 3) | (1u64 << 4) | (1u64 << 5) |
                   (1u64 << 31);
        enable_counters(mask);
    }

    /// Configure a specific event counter
    ///
    /// # Arguments
    ///
    /// * `counter_idx` - Counter index (0-5)
    /// * `event` - Event type to count
    unsafe fn configure_event_counter(counter_idx: u64, event: PmuEvent) {
        write_pmselelr(counter_idx);
        write_pmxevtyper(event as u64);
        zero_pmxevcntr();
    }

    /// Performance snapshot with cycle and event counters
    #[derive(Copy, Clone, Default, Debug)]
    pub struct Snapshot {
        /// CPU cycles (PMCCNTR_EL0)
        pub cycles: u64,
        /// Instructions retired (counter 0)
        pub inst: u64,
        /// L1 data cache refill (counter 1)
        pub l1d_refill: u64,
        /// Branch mispredictions (counter 2)
        pub branch_mispred: u64,
        /// L2 data cache access (counter 3)
        pub l2d_cache: u64,
        /// L1 instruction cache refill (counter 4)
        pub l1i_refill: u64,
        /// Exceptions taken (counter 5)
        pub exc_taken: u64,
    }

    impl Snapshot {
        /// Calculate instructions per cycle (IPC)
        pub fn ipc(&self) -> f64 {
            if self.cycles > 0 {
                self.inst as f64 / self.cycles as f64
            } else {
                0.0
            }
        }

        /// Calculate L1D cache miss rate
        pub fn l1d_miss_rate(&self) -> f64 {
            if self.inst > 0 {
                (self.l1d_refill as f64 / self.inst as f64) * 100.0
            } else {
                0.0
            }
        }

        /// Calculate branch misprediction rate
        pub fn branch_miss_rate(&self) -> f64 {
            if self.inst > 0 {
                (self.branch_mispred as f64 / self.inst as f64) * 100.0
            } else {
                0.0
            }
        }
    }

    /// Read performance snapshot from all PMU counters
    ///
    /// # Safety
    ///
    /// PMU must be initialized before calling this function.
    ///
    /// # M8 Hardening
    ///
    /// Returns DriverError::NotInitialized if PMU is not initialized.
    pub unsafe fn read_snapshot() -> DriverResult<Snapshot> {
        if !PMU_INITIALIZED.load(Ordering::Acquire) {
            return Err(DriverError::NotInitialized);
        }

        // Read cycle counter (PMCCNTR_EL0)
        let cycles = read_pmccntr();

        // Read event counters 0-5
        write_pmselelr(0);
        let inst = read_pmxevcntr();

        write_pmselelr(1);
        let l1d_refill = read_pmxevcntr();

        write_pmselelr(2);
        let branch_mispred = read_pmxevcntr();

        write_pmselelr(3);
        let l2d_cache = read_pmxevcntr();

        write_pmselelr(4);
        let l1i_refill = read_pmxevcntr();

        write_pmselelr(5);
        let exc_taken = read_pmxevcntr();

        Ok(Snapshot {
            cycles,
            inst,
            l1d_refill,
            branch_mispred,
            l2d_cache,
            l1i_refill,
            exc_taken,
        })
    }

    /// Read cycle counter only (faster than full snapshot)
    ///
    /// # M8 Hardening
    ///
    /// Returns DriverError::NotInitialized if PMU is not initialized.
    pub unsafe fn read_cycles() -> DriverResult<u64> {
        if !PMU_INITIALIZED.load(Ordering::Acquire) {
            return Err(DriverError::NotInitialized);
        }
        Ok(read_pmccntr())
    }

    /// Read a specific event counter
    ///
    /// # Arguments
    ///
    /// * `counter_idx` - Counter index (0-5)
    ///
    /// # M8 Hardening
    ///
    /// Returns DriverError::NotInitialized if PMU is not initialized.
    /// Returns DriverError::InvalidParameter if counter_idx > 5.
    pub unsafe fn read_event_counter(counter_idx: u64) -> DriverResult<u64> {
        if !PMU_INITIALIZED.load(Ordering::Acquire) {
            return Err(DriverError::NotInitialized);
        }

        // M8: Validate counter index (0-5)
        Validator::check_bounds(counter_idx as usize, 0, super::MAX_EVENT_COUNTER as usize)?;

        write_pmselelr(counter_idx);
        Ok(read_pmxevcntr())
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub mod aarch64 {
    use super::PMU_INITIALIZED;
    use super::{DriverError, DriverResult, Validator};
    use core::sync::atomic::Ordering;

    #[derive(Copy, Clone, Default, Debug)]
    pub struct Snapshot {
        pub cycles: u64,
        pub inst: u64,
        pub l1d_refill: u64,
        pub branch_mispred: u64,
        pub l2d_cache: u64,
        pub l1i_refill: u64,
        pub exc_taken: u64,
    }

    #[derive(Copy, Clone, Debug)]
    pub enum PmuEvent {
        Unsupported,
    }

    impl Snapshot {
        pub fn ipc(&self) -> f64 { 0.0 }
        pub fn l1d_miss_rate(&self) -> f64 { 0.0 }
        pub fn branch_miss_rate(&self) -> f64 { 0.0 }
    }

    pub unsafe fn init() {
        PMU_INITIALIZED.store(true, Ordering::Release);
    }

    pub unsafe fn read_snapshot() -> DriverResult<Snapshot> {
        if !PMU_INITIALIZED.load(Ordering::Acquire) {
            return Err(DriverError::NotInitialized);
        }
        Ok(Snapshot::default())
    }

    pub unsafe fn read_cycles() -> DriverResult<u64> {
        if !PMU_INITIALIZED.load(Ordering::Acquire) {
            return Err(DriverError::NotInitialized);
        }
        Ok(0)
    }

    pub unsafe fn read_event_counter(counter_idx: u64) -> DriverResult<u64> {
        if !PMU_INITIALIZED.load(Ordering::Acquire) {
            return Err(DriverError::NotInitialized);
        }
        Validator::check_bounds(counter_idx as usize, 0, super::MAX_EVENT_COUNTER as usize)?;
        Ok(0)
    }
}

// Public API (re-export from aarch64 module)
pub use aarch64::{Snapshot, PmuEvent};

/// Initialize the PMU
///
/// Should be called once during kernel initialization, after platform setup.
///
/// # Safety
///
/// Must only be called once during boot on the boot CPU.
pub unsafe fn init() {
    aarch64::init();
}

/// Read a complete performance snapshot
///
/// Returns counters for cycles, instructions, cache misses, etc.
///
/// # M8 Hardening
///
/// Returns DriverError::NotInitialized if PMU is not initialized.
pub fn read_snapshot() -> DriverResult<Snapshot> {
    unsafe { aarch64::read_snapshot() }
}

/// Read cycle counter only
///
/// Faster than reading a full snapshot when only cycle count is needed.
///
/// # M8 Hardening
///
/// Returns DriverError::NotInitialized if PMU is not initialized.
pub fn read_cycles() -> DriverResult<u64> {
    unsafe { aarch64::read_cycles() }
}

/// Read a specific event counter
///
/// # Arguments
///
/// * `counter_idx` - Counter index (0-5)
///
/// # M8 Hardening
///
/// Returns DriverError::NotInitialized if PMU is not initialized.
/// Returns DriverError::InvalidParameter if counter_idx > 5.
pub fn read_event_counter(counter_idx: u64) -> DriverResult<u64> {
    unsafe { aarch64::read_event_counter(counter_idx) }
}

/// Check if PMU is initialized
pub fn is_initialized() -> bool {
    PMU_INITIALIZED.load(Ordering::Acquire)
}
