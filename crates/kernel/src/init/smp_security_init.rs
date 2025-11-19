//! Phase 6: SMP and security initialization
//!
//! This phase initializes SMP and security subsystems:
//! - Entropy source/PRNG
//! - SMP (symmetric multiprocessing)
//! - PSCI (power state coordination interface)
//! - GICv3 (generic interrupt controller)
//! - Timer interrupt
//! - PMU (performance monitoring unit)

use super::{InitError, InitResult};

/// Initialize SMP and security subsystems
///
/// # Safety
/// Must be called after network stack (Phase 5)
/// Must be called before graphics (Phase 7)
pub unsafe fn init_smp_and_security() -> InitResult<()> {
    // Initialize entropy source/PRNG
    init_random()?;

    // Initialize SMP (multi-core)
    init_smp_subsystem()?;

    // Initialize PSCI for power management
    init_psci()?;

    // Initialize GICv3 and timer
    init_gic_and_timer()?;

    // Initialize SMP bringup (secondary CPUs)
    init_secondary_cpus()?;

    // Initialize PMU
    init_pmu()?;

    Ok(())
}

/// Initialize entropy source and PRNG
unsafe fn init_random() -> InitResult<()> {
    crate::security::init_random();
    Ok(())
}

/// Initialize SMP subsystem
unsafe fn init_smp_subsystem() -> InitResult<()> {
    crate::smp::init();
    Ok(())
}

/// Initialize PSCI for power management
unsafe fn init_psci() -> InitResult<()> {
    #[cfg(target_arch = "aarch64")]
    crate::arch::psci::init();
    Ok(())
}

/// Initialize GICv3 and timer interrupts
unsafe fn init_gic_and_timer() -> InitResult<()> {
    #[cfg(target_arch = "aarch64")]
    {
        // GIC and timer initialization is handled by the bringup module
        // This will be migrated when we update bringup::run() to use the new framework
        // For now, skip this phase - it's handled by the old bringup code
    }
    Ok(())
}

/// Initialize secondary CPUs (SMP bringup)
unsafe fn init_secondary_cpus() -> InitResult<()> {
    #[cfg(target_arch = "aarch64")]
    crate::arch::smp::init();
    Ok(())
}

/// Initialize PMU (Performance Monitoring Unit)
unsafe fn init_pmu() -> InitResult<()> {
    crate::pmu::init();
    Ok(())
}
