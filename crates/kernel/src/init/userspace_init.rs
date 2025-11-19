//! Phase 9: Userspace initialization
//!
//! This phase initializes userspace:
//! - Create PID 1 (init process)
//! - Enqueue init process to scheduler
//! - Launch interactive shell on alternate stack

use super::{InitError, InitResult};

/// Initialize userspace (create init process)
///
/// # Safety
/// Must be called after AI subsystem (Phase 8)
/// This is the final initialization phase before entering the shell
pub unsafe fn init_userspace() -> InitResult<()> {
    // Create PID 1 (init process)
    create_init_process()?;

    // Enqueue PID 1 to scheduler
    enqueue_init_process()?;

    Ok(())
}

/// Create PID 1 (init process)
unsafe fn create_init_process() -> InitResult<()> {
    let init_task = crate::process::Task::new_init();
    crate::process::insert_task(init_task)
        .map_err(|_| InitError::ShellFailed)?;
    Ok(())
}

/// Enqueue PID 1 to scheduler
unsafe fn enqueue_init_process() -> InitResult<()> {
    crate::process::scheduler::enqueue(1);
    crate::process::scheduler::set_current(1);
    Ok(())
}

/// Launch interactive shell on alternate stack
///
/// This function never returns - it either enters the shell or falls back to minishell
pub unsafe fn launch_shell() -> ! {
    // Probe shell module
    crate::shell::shell_probe_trampoline();

    // Run full shell on a dedicated stack to avoid any latent stack issues
    launch_full_shell_on_alt_stack();

    // If shell exits, fall back to minishell
    crate::run_minishell_loop();
}

/// 64 KiB stack dedicated to the full shell runtime (16-byte aligned)
#[repr(C, align(16))]
struct ShellStack([u8; 64 * 1024]);
static mut SHELL_STACK: ShellStack = ShellStack([0; 64 * 1024]);

/// Switch to an alternate stack, run the full shell, then restore the original stack
/// Keep IRQs enabled during the shell to preserve timers; mask only during SP switch.
unsafe fn launch_full_shell_on_alt_stack() {
    let base = &raw const SHELL_STACK.0;
    let sp_top = base.cast::<u8>().add((*base).len()) as u64;

    // Mask IRQs for the short window of SP switch
    core::arch::asm!("msr daifset, #2", options(nostack, preserves_flags));
    core::arch::asm!(
        r#"
        mov x9, sp          // save old SP
        mov sp, {new_sp}    // switch to shell stack
        msr daifclr, #2     // unmask IRQs
        bl {entry}          // run full shell (may return on 'exit')
        msr daifset, #2     // mask before restoring SP
        mov sp, x9          // restore old SP
        msr daifclr, #2     // unmask
        "#,
        new_sp = in(reg) sp_top,
        entry = sym crate::shell::run_shell_c,
        lateout("x9") _,
        clobber_abi("C")
    );
}
