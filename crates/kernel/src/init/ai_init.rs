//! Phase 8: AI subsystem initialization
//!
//! This phase initializes AI and autonomous systems:
//! - AI features (if enabled)
//! - Neural memory agent
//! - Meta-agent
//! - Autonomy system
//! - AgentSys framework

use super::{InitError, InitResult};
use core::sync::atomic::Ordering;

/// Initialize AI subsystem and agents
///
/// # Safety
/// Must be called after graphics init (Phase 7)
/// Must be called before userspace init (Phase 9)
pub unsafe fn init_ai_subsystem() -> InitResult<()> {
    // Run AI benchmarks if enabled
    run_ai_benchmarks()?;

    // Emit kernel metrics and run demos
    emit_metrics_and_demos()?;

    // Initialize neural memory agent
    init_neural_agent()?;

    // Initialize meta-agent
    init_meta_agent()?;

    // Mark autonomy as ready
    mark_autonomy_ready()?;

    // Enable autonomous control if in bringup mode
    enable_autonomous_control()?;

    // Print build information
    print_build_info()?;

    // Initialize AgentSys framework
    init_agentsys()?;

    Ok(())
}

/// Run AI benchmarks if enabled
unsafe fn run_ai_benchmarks() -> InitResult<()> {
    #[cfg(feature = "arm64-ai")]
    crate::ai_benchmark::run_ai_benchmarks();
    Ok(())
}

/// Emit kernel metrics and run demos
unsafe fn emit_metrics_and_demos() -> InitResult<()> {
    crate::userspace_test::emit_kernel_metrics();

    #[cfg(target_arch = "aarch64")]
    crate::userspace_test::bench_real_context_switch();

    crate::userspace_test::run_syscall_tests();

    Ok(())
}

/// Initialize memory neural agent
unsafe fn init_neural_agent() -> InitResult<()> {
    crate::neural::init_memory_agent();
    Ok(())
}

/// Initialize meta-agent for global coordination
unsafe fn init_meta_agent() -> InitResult<()> {
    crate::meta_agent::init_meta_agent();
    Ok(())
}

/// Mark autonomy system as ready
unsafe fn mark_autonomy_ready() -> InitResult<()> {
    crate::autonomy::AUTONOMY_READY.store(true, Ordering::Release);
    Ok(())
}

/// Enable autonomous control during bringup (optional)
unsafe fn enable_autonomous_control() -> InitResult<()> {
    #[cfg(all(target_arch = "aarch64", feature = "bringup"))]
    {
        use core::arch::asm;

        // Enable autonomous control
        crate::autonomy::AUTONOMOUS_CONTROL.enable();

        // Disable timer first and clear any pending ISTATUS
        let ctl_off: u64 = 0;
        asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl_off);
        asm!("dsb sy; isb");
        let clear_val: u64 = 1;
        asm!("msr cntp_tval_el0, {x}", x = in(reg) clear_val);
        asm!("isb");

        // Compute absolute compare value
        let mut frq: u64;
        asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        let interval_ms = crate::autonomy::AUTONOMOUS_CONTROL
            .decision_interval_ms
            .load(Ordering::Relaxed)
            .clamp(100, 60_000);
        let cycles = if frq > 0 {
            (frq / 1000).saturating_mul(interval_ms)
        } else {
            (62_500u64).saturating_mul(interval_ms)
        };
        let mut now: u64;
        asm!("mrs {x}, cntpct_el0", x = out(reg) now);
        let next = now.saturating_add(cycles);

        asm!("msr cntp_cval_el0, {x}", x = in(reg) next);
        asm!("isb");
        let ctl_on: u64 = 1; // ENABLE=1, IMASK=0
        asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl_on);
        asm!("isb");
    }

    Ok(())
}

/// Print build information
unsafe fn print_build_info() -> InitResult<()> {
    crate::build_info::print_build_info();
    Ok(())
}

/// Initialize AgentSys framework (optional)
unsafe fn init_agentsys() -> InitResult<()> {
    #[cfg(feature = "agentsys")]
    crate::agent_sys::init();
    Ok(())
}
