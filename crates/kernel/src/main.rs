#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
// CI lint gate: when built with `--features strict`, fail on any warning
#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "strict", deny(unsafe_op_in_unsafe_fn))]
// During early bringup, suppress warnings to keep logs clean
#![cfg_attr(all(feature = "bringup", not(feature = "strict")), allow(warnings))]

// Required for heap allocation
extern crate alloc;

// Optional DTB pointer provided by loader (UEFI); used to override platform descriptors
#[no_mangle]
pub static mut DTB_PTR: *const u8 = core::ptr::null();

// Core library (error handling, logging, etc.)
#[allow(special_module_name)]
pub mod lib;
// Kernel initialization phases
pub mod init;
// System call interface module
pub mod syscall;
// Process management
pub mod process;
// Memory management
pub mod mm;
// Virtual File System
pub mod vfs;
// Block layer (Phase B)
pub mod block;
// Network layer (Phase C)
pub mod net;
// Security subsystem (Phase D)
pub mod security;
// SMP subsystem (Phase E)
#[cfg(not(target_arch = "x86_64"))]
pub mod smp;
#[cfg(target_arch = "x86_64")]
pub use crate::arch::x86_64::smp as smp;
// Filesystem layer with journaling (Phase F)
pub mod fs;
// Graphics layer (Phase G.0)
pub mod graphics;
// Window manager (Phase G.1)
pub mod window_manager;
// UI Toolkit (Phase G.2)
pub mod ui;
// Desktop Applications (Phase G.3)
pub mod applications;
// AI Integration UI (Phase G.4)
pub mod ai_ui;
// Audio Infrastructure (Phase G.5)
pub mod audio;
// Voice UI (Phase G.5)
pub mod voice;
// Camera Infrastructure (Phase G.5)
pub mod camera;
// Animation System (Phase G.6)
pub mod animation;
// Device drivers (Phase A1)
pub mod drivers;
// Initial RAM filesystem
pub mod initramfs;
// Userspace test module
pub mod userspace_test;
// Interactive shell module
pub mod shell;
// UART driver module
pub mod uart;
// Driver framework module
pub mod driver;
// VirtIO transport layer module
pub mod virtio;
// VirtIO console driver module
pub mod virtio_console;
// Heap allocator module
pub mod heap;
// Phase 1 scaffolding: trace, capabilities, tensors, channels, graph
pub mod trace;
pub mod cap;
pub mod tensor;
pub mod channel;
pub mod graph;
pub mod model;
pub mod ml;
pub mod inference;
pub mod npu;
pub mod npu_driver;
pub mod interrupts;
pub mod control;
pub mod neural;
pub mod internal_agent_bus;
pub mod meta_agent;
pub mod autonomy;
pub mod time;
pub mod log;  // M8: Production logging framework
pub mod validation;  // M7: Comprehensive validation suite
pub mod prediction_tracker;
pub mod stress_test;
pub mod prng;
pub mod autonomy_metrics;
pub mod latency_histogram;
pub mod predictive_memory;
pub mod predictive_scheduling;
pub mod command_predictor;
pub mod network_predictor;
pub mod benchmark;
pub mod compliance;
// Metrics export for observability (Phase 1.3 - Production Readiness)
pub mod metrics_export;
// Chaos engineering for resilience testing (Phase 3.1 - Production Readiness)
#[cfg(feature = "chaos")]
pub mod chaos;
// Build information for forensics (Phase 5 - Production Readiness)
pub mod build_info;
// Test utilities (only compiled for testing)
#[cfg(test)]
pub mod test_utils;
// Phase 8: Performance tests and benchmarks
pub mod tests;
// PMU helpers (feature-gated usage)
pub mod pmu;
// Profiling framework (Phase 8 Milestone 5 - feature-gated)
#[cfg(feature = "profiling")]
pub mod profiling;
// Platform layer abstraction (UART/GIC/Timer/MMU descriptors)
pub mod platform;
// Deterministic scheduler scaffolding (feature-gated)
#[cfg(feature = "deterministic")]
pub mod deterministic;
// AI benchmark module
#[cfg(feature = "arm64-ai")]
pub mod ai_benchmark;
// LLM kernel service (feature-gated)
#[cfg(feature = "llm")]
pub mod llm;

// Phase 2: AI Governance & Multi-Agent Coordination (feature-gated)
#[cfg(feature = "ai-ops")]
pub mod ai;

// Phase 9: AgentSys - Capability-based system for LLM agents (feature-gated)
#[cfg(feature = "agentsys")]
pub mod agent_sys;

// Optional embedded initramfs for models (integration tests)
#[cfg(all(feature = "initramfs-models", have_initramfs_models, any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "riscv64")))]
#[allow(non_upper_case_globals)]
mod embedded_models_initramfs {
    pub static data: &[u8] = include_bytes!(env!("INITRAMFS_MODELS_FILE"));
}

// Phase 7: AI Operations Platform
#[cfg(feature = "model-lifecycle")]
pub mod model_lifecycle;
#[cfg(feature = "decision-traces")]
pub mod trace_decision;
pub mod ai_insights;
pub mod sched;
#[cfg(feature = "shadow-mode")]
pub mod shadow;
#[cfg(feature = "otel")]
pub mod otel;

// Architecture-specific modules
#[cfg(target_arch = "aarch64")]
pub mod arch {
    pub mod aarch64;
    pub use aarch64::*;
}

#[cfg(target_arch = "aarch64")]
pub mod aarch64_context;

#[cfg(target_arch = "x86_64")]
pub mod arch {
    pub mod x86_64;
    pub use x86_64::*;
}

#[cfg(target_arch = "riscv64")]
pub mod arch {
    pub mod riscv64;
    pub use riscv64::*;
}

#[cfg(target_arch = "aarch64")]
#[link_section = ".text._start"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        uart_print(b"KERNEL(U)\n");
    }

    #[cfg(all(target_arch = "aarch64", feature = "bringup"))]
    unsafe {
        bringup::run();
    }

    loop {}
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub extern "C" fn _start(boot_info: *const crate::arch::x86_64::boot::BootInfo) -> ! {
    // Early architecture initialization
    unsafe {
        crate::arch::x86_64::boot::init_boot_info(boot_info);
        if let Err(e) = arch::boot::early_init() {
            // Critical error during boot
            arch::serial::serial_write(b"\n[FATAL] Boot error: ");
            arch::serial::serial_write(e.as_bytes());
            arch::serial::serial_write(b"\n");
            arch::boot::halt_forever();
        }
    }

    // Print boot information
    arch::boot::print_boot_info();

    // TODO: Continue with platform-independent initialization
    arch::serial::serial_write(b"[BOOT] Kernel initialization complete\n");
    arch::serial::serial_write(b"[BOOT] Entering idle loop with timer demonstration...\n");
    arch::serial::serial_write(b"[BOOT] Timer is configured for 1000 Hz (1 ms per tick)\n");
    arch::serial::serial_write(b"[BOOT] PS/2 keyboard is active - press keys to test!\n");
    arch::serial::serial_write(b"\n");

    // Idle loop with periodic timer tick display (will be replaced with scheduler in M8)
    let mut last_displayed_second = 0u64;
    loop {
        x86_64::instructions::hlt();

        // Check for keyboard input
        #[cfg(target_arch = "x86_64")]
        if let Some(ch) = arch::ps2_keyboard::read_char() {
            arch::serial::serial_write(b"[KEYBOARD] Key pressed: '");
            arch::serial::serial_write_byte(ch as u8);
            arch::serial::serial_write(b"' (ASCII: ");
            print_u64(ch as u64);
            arch::serial::serial_write(b")\n");
        }

        // Display timer ticks every second (1000 ticks)
        let ticks = arch::pit::ticks();
        let current_second = ticks / 1000;

        if current_second > last_displayed_second && current_second % 1 == 0 {
            last_displayed_second = current_second;

            arch::serial::serial_write(b"[TIMER] Uptime: ");
            print_u64(current_second);
            arch::serial::serial_write(b" seconds (");
            print_u64(ticks);
            arch::serial::serial_write(b" ticks)\n");
        }
    }
}

/// Helper to print u64
fn print_u64(mut n: u64) {
    if n == 0 {
        arch::serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        arch::serial::serial_write_byte(buf[i]);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Use enhanced panic handler with diagnostics
    crate::lib::panic::panic_handler(info)
}

#[inline(always)]
unsafe fn uart_print(msg: &[u8]) {
    // Early boot UART writes using platform-provided base; avoids hardcoded MMIO
    let base = crate::platform::active().uart().base as *mut u32;
    for &b in msg {
        core::ptr::write_volatile(base, b as u32);
    }
}

#[macro_export]
macro_rules! kprint {
    ($($t:tt)*) => {{
        let s = alloc::format!($($t)*);
        #[allow(unused_unsafe)]
        unsafe { crate::uart_print(s.as_bytes()); }
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($t:tt)*) => { $crate::kprint!("{}\n", format_args!($($t)*)) };
}

#[cfg(all(target_arch = "aarch64", feature = "bringup"))]
mod bringup {
    use core::arch::asm;
    use core::sync::atomic::{AtomicBool, Ordering};

    // 64 KiB bootstrap stack (16-byte aligned)
    #[repr(C, align(16))]
    struct Stack([u8; 64 * 1024]);
    static mut BOOT_STACK: Stack = Stack([0; 64 * 1024]);

    // Level-1 translation table (4 KiB aligned)
    #[repr(C, align(4096))]
    struct Table512([u64; 512]);
    static mut L1_TABLE: Table512 = Table512([0; 512]);

    // Simple EL-aware VBAR install and UART helper from outer module
    extern "C" {
        static VECTORS: u8;
    }

    pub unsafe fn run() {
        // 1) Install stack
        let stack_ptr = &raw const BOOT_STACK.0;
        let sp_top = stack_ptr.cast::<u8>().add((*stack_ptr).len()) as u64;
        asm!("mov sp, {sp}", sp = in(reg) sp_top, options(nostack, preserves_flags));
        super::uart_print(b"STACK OK\n");

        // 2) Install exception vectors before any interrupts
        crate::arch::trap::init_exception_vectors();
        super::uart_print(b"VECTORS OK\n");

        // 3) Run modular initialization phases with error propagation
        use crate::init::phases;
        use alloc::string::ToString;

        // Early init: MMU, UART, PMU, heap
        if let Err(e) = phases::early_init() {
            super::uart_print(b"EARLY INIT FAILED: ");
            super::uart_print(e.to_string().as_bytes());
            super::uart_print(b"\n");
            loop {}
        }

        // Platform init: FDT parsing, PSCI
        if let Err(e) = phases::platform_init() {
            super::uart_print(b"PLATFORM INIT FAILED: ");
            super::uart_print(e.to_string().as_bytes());
            super::uart_print(b"\n");
            loop {}
        }

        // Memory subsystem init: buddy, slab, page cache
        if let Err(e) = phases::memory_init() {
            super::uart_print(b"MEMORY INIT FAILED: ");
            super::uart_print(e.to_string().as_bytes());
            super::uart_print(b"\n");
            loop {}
        }

        // Subsystem init: VFS, network, graphics, process table, scheduler
        if let Err(e) = phases::subsystem_init() {
            super::uart_print(b"SUBSYSTEM INIT FAILED: ");
            super::uart_print(e.to_string().as_bytes());
            super::uart_print(b"\n");
            loop {}
        }

        // Driver init: block devices, watchdog, driver framework
        if let Err(e) = phases::driver_init() {
            super::uart_print(b"DRIVER INIT FAILED: ");
            super::uart_print(e.to_string().as_bytes());
            super::uart_print(b"\n");
            loop {}
        }

        // Late init: GIC, interrupts, SMP, AI, shell
        if let Err(e) = phases::late_init() {
            super::uart_print(b"LATE INIT FAILED: ");
            super::uart_print(e.to_string().as_bytes());
            super::uart_print(b"\n");
            loop {}
        }

        // All initialization phases complete! Launch interactive shell
        super::uart_print(b"LAUNCHING SHELL\n");
        super::uart_print(b"[MAIN] STARTING FULL SHELL\n");

        // Attempt a quick probe into the shell module
        super::uart_print(b"[SHELL] PROBE PRE\n");
        crate::shell::shell_probe_trampoline();
        super::uart_print(b"[SHELL] PROBE POST\n");

        // Run full shell on a dedicated stack to avoid any latent stack issues
        launch_full_shell_on_alt_stack();

        super::uart_print(b"[MAIN] FULL SHELL EXITED -> FALLBACK TO MINI\n");
        crate::run_minishell_loop();
    }

    // 64 KiB stack dedicated to the full shell runtime (16-byte aligned)
    #[repr(C, align(16))]
    struct ShellStack([u8; 64 * 1024]);
    static mut SHELL_STACK: ShellStack = ShellStack([0; 64 * 1024]);

    /// Switch to an alternate stack, run the full shell, then restore the original stack
    unsafe fn launch_full_shell_on_alt_stack() {
        use core::arch::asm;
        let base = &raw const SHELL_STACK.0;
        let sp_top = base.cast::<u8>().add((*base).len()) as u64;

        // Mask IRQs for the short window of SP switch
        asm!("msr daifset, #2", options(nostack, preserves_flags));
        asm!(
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

    // Timer tick counter (exposed for shell commands)
    #[no_mangle]
    pub static mut TIMER_TICK_COUNT: u32 = 0;

    // BRINGUP HELPER FUNCTIONS (used by init phases)

    pub unsafe fn gicv3_init_qemu() {
        super::uart_print(b"GIC:A\n");
        // Obtain GIC base addresses from platform descriptor
        let g = crate::platform::active().gic();
        let gicd_base: u64 = g.gicd as u64; // GIC Distributor
        let gicr_base: u64 = g.gicr as u64; // GIC Redistributor

        // GIC Distributor registers
        const GICD_CTLR: u64 = 0x0000;
        #[allow(dead_code)] // Complete register set for future use
        const GICD_TYPER: u64 = 0x0004;
        #[allow(dead_code)] // Complete register set for future use
        const GICD_IGROUPR: u64 = 0x0080;
        #[allow(dead_code)] // Complete register set for future use
        const GICD_ISENABLER: u64 = 0x0100;
        #[allow(dead_code)] // Complete register set for future use
        const GICD_IPRIORITYR: u64 = 0x0400;

        // GIC Redistributor registers
        // IMPORTANT: SGI/PPI configuration is in the second 64KB page!
        const GICR_WAKER: u64 = 0x0014;  // In RD_base (first page)
        const SGI_BASE: u64 = 0x10000;   // SGI/PPI config base (second page)
        const GICR_IGROUPR0: u64 = SGI_BASE + 0x0080;
        const GICR_ISENABLER0: u64 = SGI_BASE + 0x0100;
        const GICR_IPRIORITYR: u64 = SGI_BASE + 0x0400;

        // 1) Initialize GIC Distributor
        super::uart_print(b"GIC:B\n");
        let gicd_ctlr = (gicd_base + GICD_CTLR) as *mut u32;

        // Check if already enabled
        super::uart_print(b"GIC:C\n");
        let ctlr_val = core::ptr::read_volatile(gicd_ctlr);
        super::uart_print(b"GIC:D\n");
        if (ctlr_val & 0x3) == 0 {
            super::uart_print(b"GIC:E\n");
            // Enable Group 0 and Group 1 (both secure and non-secure)
            core::ptr::write_volatile(gicd_ctlr, 0x3);
            core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));
        } else {
            super::uart_print(b"GIC:F\n");
        }

        // 2) Verify redistributor and wake it up for CPU0
        super::uart_print(b"GIC:G\n");

        // Check GICR_TYPER to verify this is the right redistributor
        const GICR_TYPER: u64 = 0x0008;
        let typer_lo = (gicr_base + GICR_TYPER) as *const u32;
        let typer_hi = (gicr_base + GICR_TYPER + 4) as *const u32;
        let typer_val_lo = core::ptr::read_volatile(typer_lo);
        let typer_val_hi = core::ptr::read_volatile(typer_hi);
        super::uart_print(b"  GICR_TYPER: 0x");
        print_number(typer_val_hi as usize);
        print_number(typer_val_lo as usize);
        super::uart_print(b"\n");

        // Check if this is the last redistributor (bit 4)
        if (typer_val_lo & (1 << 4)) != 0 {
            super::uart_print(b"  This is the LAST redistributor\n");
        }

        // Get CPU number from TYPER
        let cpu_num = (typer_val_hi >> 8) & 0xFFFF;
        super::uart_print(b"  CPU number: ");
        print_number(cpu_num as usize);
        super::uart_print(b"\n");

        let waker = (gicr_base + GICR_WAKER) as *mut u32;
        super::uart_print(b"GIC:H\n");

        // Clear ProcessorSleep bit [1]
        let mut w: u32 = core::ptr::read_volatile(waker);
        super::uart_print(b"GIC:I\n");
        if (w & (1 << 1)) != 0 {
            super::uart_print(b"GIC:J\n");
            w &= !(1 << 1);
            core::ptr::write_volatile(waker, w);
            core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));

            // Wait for ChildrenAsleep bit [2] to clear with timeout
            let mut timeout = 1000000;
            loop {
                let v = core::ptr::read_volatile(waker);
                if (v & (1 << 2)) == 0 {
                    super::uart_print(b"GIC:K\n");
                    break;
                }
                timeout -= 1;
                if timeout == 0 {
                    super::uart_print(b"GIC:L\n");
                    break;
                }
            }
        } else {
            super::uart_print(b"GIC:M\n");
        }

        // 3) Configure PPI 30 (EL1 physical timer) as Group 1 (non-secure)
        super::uart_print(b"GIC:N\n");
        super::uart_print(b"[GIC] Using SGI/PPI base at offset 0x10000 from redistributor\n");

        // First, disable all PPIs in ICENABLER0
        const GICR_ICENABLER0: u64 = SGI_BASE + 0x0180;
        let icenabler0 = (gicr_base + GICR_ICENABLER0) as *mut u32;
        core::ptr::write_volatile(icenabler0, 0xFFFFFFFF); // Clear all enables
        core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));

        // Now set Group 1 for PPI 30
        let igroupr0 = (gicr_base + GICR_IGROUPR0) as *mut u32;
        let mut grp = core::ptr::read_volatile(igroupr0);
        super::uart_print(b"  IGROUPR0 before: 0x");
        print_number(grp as usize);
        super::uart_print(b"\n");

        grp |= 1 << 30;
        core::ptr::write_volatile(igroupr0, grp);
        core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));

        // Verify group was set
        let grp_check = core::ptr::read_volatile(igroupr0);
        super::uart_print(b"  IGROUPR0 after: 0x");
        print_number(grp_check as usize);
        if (grp_check & (1 << 30)) != 0 {
            super::uart_print(b" (PPI 30 is Group 1)\n");
        } else {
            super::uart_print(b" (WARNING: PPI 30 NOT Group 1!)\n");
        }

        // 4) Set priority for PPI 30
        // Must be < ICC_PMR_EL1 to fire. Since ICC_PMR_EL1 is stuck at 248 (0xF8),
        // we need priority < 248. Set to 0x60 (96) for safety.
        let iprio = (gicr_base + GICR_IPRIORITYR) as *mut u32;
        let prio_reg = iprio.add(30 / 4); // 4 priorities per 32-bit register
        let shift = (30 % 4) * 8;
        super::uart_print(b"GIC:O\n");

        // Try direct write of full register
        // PPI 30 is in bits [15:8] of this register (30 % 4 = 2, so shift by 16)
        let prio_val: u32 = 0x60606060; // Set all 4 priorities to 0x60
        super::uart_print(b"  Writing priority register 0x");
        print_number(prio_val as usize);
        super::uart_print(b" to offset ");
        print_number((30 / 4) as usize);
        super::uart_print(b"\n");

        core::ptr::write_volatile(prio_reg, prio_val);
        core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));

        // Verify priority was set
        let prio_check = core::ptr::read_volatile(prio_reg);
        super::uart_print(b"  Priority register readback: 0x");
        print_number(prio_check as usize);
        super::uart_print(b"\n");

        let actual_prio = (prio_check >> shift) & 0xFF;
        super::uart_print(b"  PPI 30 priority: ");
        print_number(actual_prio as usize);
        if actual_prio == 0x60 {
            super::uart_print(b" (OK - set to 96)\n");
        } else if actual_prio == 0 {
            super::uart_print(b" (ERROR - still 0!)\n");
        } else {
            super::uart_print(b" (unexpected value)\n");
        }

        // 5) Enable PPI 30 with retries
        super::uart_print(b"GIC: ENABLE PPI30\n");
        let isenabler0 = (gicr_base + GICR_ISENABLER0) as *mut u32;

        // First, verify redistributor is awake
        let waker_check = core::ptr::read_volatile(waker);
        if (waker_check & 0x6) != 0 {  // Check both ChildrenAsleep and ProcessorSleep
            super::uart_print(b"[WARNING] Redistributor may not be fully awake: ");
            print_number(waker_check as usize);
            super::uart_print(b"\n");
        }

        // Try multiple times to enable PPI 30
        for attempt in 0..3 {
            // Write directly (not read-modify-write) to set bit 30
            core::ptr::write_volatile(isenabler0, 1u32 << 30);
            core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));

            // Small delay to let write settle
            for _ in 0..100 {
                core::arch::asm!("nop");
            }

            // Read back to verify
            let enabled = core::ptr::read_volatile(isenabler0);
            if (enabled & (1 << 30)) != 0 {
                super::uart_print(b"GIC: ISENABLER0 success on attempt ");
                print_number(attempt as usize + 1);
                super::uart_print(b", readback: 0x");
                print_number(enabled as usize);
                super::uart_print(b"\n");
                break;
            } else if attempt == 2 {
                super::uart_print(b"GIC: ISENABLER0 FAILED after 3 attempts, readback: 0x");
                print_number(enabled as usize);
                super::uart_print(b" (bit 30 NOT set!)\n");

                // Try to understand why it's failing
                super::uart_print(b"  Redistributor GICR_WAKER: ");
                let waker_final = core::ptr::read_volatile(waker);
                print_number(waker_final as usize);
                super::uart_print(b"\n");

                // Check if we can write ANY bits to ISENABLER0
                core::ptr::write_volatile(isenabler0, 0xFFFFFFFF);
                core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));
                let test_write = core::ptr::read_volatile(isenabler0);
                super::uart_print(b"  Test write 0xFFFFFFFF readback: 0x");
                print_number(test_write as usize);
                super::uart_print(b"\n");

                // Restore to just PPI 30
                core::ptr::write_volatile(isenabler0, 1u32 << 30);
                core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));
            }
        }

        // Final verification
        let enabled = core::ptr::read_volatile(isenabler0);
        super::uart_print(b"GIC: ISENABLER0 readback: 0x");
        print_number(enabled as usize);
        super::uart_print(b" (bit 30 should be set)\n");

        super::uart_print(b"GIC:P\n");

        // CPU interface via system registers
        super::uart_print(b"GIC:Q\n");

        // Set priority mask to unmask all interrupts - try multiple approaches
        super::uart_print(b"[GIC] Setting ICC_PMR_EL1...\n");

        // First attempt: Direct write of 0xFF
        let pmr: u64 = 0xFF;
        asm!("msr icc_pmr_el1, {x}", x = in(reg) pmr);
        asm!("isb", options(nostack, preserves_flags));

        let pmr_readback1: u64;
        asm!("mrs {x}, icc_pmr_el1", x = out(reg) pmr_readback1);
        super::uart_print(b"  Attempt 1 (0xFF): readback=");
        print_number(pmr_readback1 as usize);
        super::uart_print(b"\n");

        if pmr_readback1 != 0xFF {
            // Try writing 0xF0 instead (might be more permissive than 0xF8)
            let pmr2: u64 = 0xF0;
            asm!("msr icc_pmr_el1, {x}", x = in(reg) pmr2);
            asm!("isb", options(nostack, preserves_flags));

            let pmr_readback2: u64;
            asm!("mrs {x}, icc_pmr_el1", x = out(reg) pmr_readback2);
            super::uart_print(b"  Attempt 2 (0xF0): readback=");
            print_number(pmr_readback2 as usize);
            super::uart_print(b"\n");

            // Try one more time with original 0xFF
            let pmr3: u64 = 0xFF;
            asm!("msr icc_pmr_el1, {x}", x = in(reg) pmr3);
            asm!("dsb sy", options(nostack, preserves_flags));
            asm!("isb", options(nostack, preserves_flags));

            let pmr_readback3: u64;
            asm!("mrs {x}, icc_pmr_el1", x = out(reg) pmr_readback3);
            super::uart_print(b"  Attempt 3 (0xFF with dsb): readback=");
            print_number(pmr_readback3 as usize);
            super::uart_print(b"\n");
        }

        // Final readback for status
        let pmr_readback: u64;
        asm!("mrs {x}, icc_pmr_el1", x = out(reg) pmr_readback);
        super::uart_print(b"[GIC] ICC_PMR_EL1 final: ");
        print_number(pmr_readback as usize);
        if pmr_readback == 0xFF {
            super::uart_print(b" (OK)\n");
        } else if pmr_readback == 0xF8 {
            super::uart_print(b" (WARNING: Stuck at 0xF8! Will allow priorities 0-0xF7)\n");
        } else if pmr_readback == 0xF0 {
            super::uart_print(b" (WARNING: Set to 0xF0! Will allow priorities 0-0xEF)\n");
        } else {
            super::uart_print(b" (WARNING: Unexpected value!)\n");
        }

        // Enable Group 1 interrupts
        let grp1: u64 = 1;
        asm!("msr icc_igrpen1_el1, {x}", x = in(reg) grp1);
        asm!("isb", options(nostack, preserves_flags));

        // Verify Group 1 enable
        let grp1_readback: u64;
        asm!("mrs {x}, icc_igrpen1_el1", x = out(reg) grp1_readback);
        super::uart_print(b"[GIC] ICC_IGRPEN1_EL1 set to 1, readback: ");
        print_number(grp1_readback as usize);
        if grp1_readback != 1 {
            super::uart_print(b" (WARNING: Expected 1!)\n");
        } else {
            super::uart_print(b" (OK)\n");
        }

        super::uart_print(b"GIC:R\n");
    }

    pub unsafe fn enable_mmu_el1() {
        use core::arch::asm;
        super::uart_print(b"MMU: MAIR/TCR\n");
        // Memory attributes: AttrIdx0 = Device-nGnRE, AttrIdx1 = Normal WBWA
        let mair = (0x04u64) | (0xFFu64 << 8);
        asm!("msr MAIR_EL1, {x}", x = in(reg) mair, options(nostack, preserves_flags));

        // TCR: 4KB pages, Inner/Outer WBWA, Inner shareable,
        // 39-bit VA (T0SZ=25), 48-bit PA (IPS=5)
        let t0sz: u64 = 64 - 39; // 25
        let tcr = (t0sz & 0x3Fu64) |
            (0b01u64 << 8)  | // IRGN0 = WBWA
            (0b01u64 << 10) | // ORGN0 = WBWA
            (0b11u64 << 12) | // SH0 = Inner Shareable
            (0b00u64 << 14) | // TG0 = 4KB
            (0b101u64 << 32); // IPS = 48-bit PA
        asm!("msr TCR_EL1, {x}", x = in(reg) tcr, options(nostack, preserves_flags));
        asm!("isb", options(nostack, preserves_flags));

        // Build translation tables
        super::uart_print(b"MMU: TABLES\n");
        build_tables();

        // Set TTBR0 to L1 table
        let l1_pa = &raw const L1_TABLE.0 as *const _ as u64;
        super::uart_print(b"MMU: TTBR0\n");
        asm!("msr TTBR0_EL1, {x}", x = in(reg) l1_pa, options(nostack, preserves_flags));
        asm!("dsb ish; isb", options(nostack, preserves_flags));

        // Enable MMU + caches
        super::uart_print(b"MMU: SCTLR\n");
        let mut sctlr: u64;
        asm!("mrs {x}, SCTLR_EL1", x = out(reg) sctlr);
        sctlr |= (1 << 0) | (1 << 2) | (1 << 12); // M, C, I
        asm!("msr SCTLR_EL1, {x}", x = in(reg) sctlr);
        asm!("isb", options(nostack, preserves_flags));
    }

    unsafe fn build_tables() {
        // Clear tables
        let table_ptr = &raw mut L1_TABLE.0 as *mut [u64; 512];
        for e in (*table_ptr).iter_mut() {
            *e = 0;
        }

        // Descriptor helpers
        const DESC_BLOCK: u64 = 1;
        const SH_INNER: u64 = 0b11 << 8;
        const AF: u64 = 1 << 10;
        const ATTRIDX_NORMAL: u64 = 1 << 2;
        const ATTRIDX_DEVICE: u64 = 0 << 2;

        // Map RAM ranges as Normal WBWA
        let plat = crate::platform::active();
        for r in plat.ram_ranges() {
            let mut base = (r.start as u64) & !((1u64 << 30) - 1);
            let end = (r.start as u64).saturating_add(r.size as u64);
            while base < end {
                let idx = (base >> 30) as usize;
                L1_TABLE.0[idx] = (base) | DESC_BLOCK | AF | SH_INNER | ATTRIDX_NORMAL;
                base = base.saturating_add(1u64 << 30);
            }
        }

        // Map MMIO ranges as Device-nGnRE
        for m in plat.mmio_ranges() {
            let mut base = (m.start as u64) & !((1u64 << 30) - 1);
            let end = (m.start as u64).saturating_add(m.size as u64);
            while base < end {
                let idx = (base >> 30) as usize;
                if L1_TABLE.0[idx] == 0 {
                    L1_TABLE.0[idx] = (base) | DESC_BLOCK | AF | ATTRIDX_DEVICE;
                }
                base = base.saturating_add(1u64 << 30);
            }
        }
    }

    pub unsafe fn pmu_enable() {
        use core::arch::asm;
        // PMCR_EL0: E=1 (enable), P=1 (reset event counters), C=1 (reset cycle counter)
        let pmcr: u64 = (1 << 0) | (1 << 1) | (1 << 2);
        asm!("msr PMCR_EL0, {x}", x = in(reg) pmcr, options(nostack, preserves_flags));

        // Enable cycle counter in PMCNTENSET_EL0 (bit 31)
        let pmcntenset: u64 = 1u64 << 31;
        asm!("msr PMCNTENSET_EL0, {x}", x = in(reg) pmcntenset, options(nostack, preserves_flags));

        // Allow EL0 reads (future use)
        let pmuserenr: u64 = 1;
        asm!("msr PMUSERENR_EL0, {x}", x = in(reg) pmuserenr, options(nostack, preserves_flags));
        asm!("isb", options(nostack, preserves_flags));
    }

    pub unsafe fn enable_irq() {
        use core::arch::asm;
        super::uart_print(b"[IRQ_ENABLE] enable_irq() called\n");
        asm!("msr daifclr, #2", options(nostack, preserves_flags));
        asm!("isb", options(nostack, preserves_flags));
    }

    pub unsafe fn timer_init_1hz() {
        use core::arch::asm;
        super::uart_print(b"[TIMER_INIT] Starting timer initialization...\n");

        let mut frq: u64;
        asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        if frq == 0 {
            frq = crate::platform::active().timer().freq_hz;
        }

        // Set initial interval ~1s but don't enable
        asm!("msr cntp_tval_el0, {x}", x = in(reg) frq);
        let ctl: u64 = 0; // Timer configured but disabled
        asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl);
        super::uart_print(b"[TIMER_INIT] Timer initialization complete.\n");
    }

    /// Helper function to print numbers
    pub unsafe fn print_number(mut num: usize) {
        if num == 0 {
            super::uart_print(b"0");
            return;
        }

        let mut digits = [0u8; 20];
        let mut i = 0;

        while num > 0 {
            digits[i] = b'0' + (num % 10) as u8;
            num /= 10;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            super::uart_print(&[digits[i]]);
        }
    }

    // Exception vector table (assembly)
    core::arch::global_asm!(
        r#"
        .balign 2048
        .global VECTORS
        .global exception_vector_table
    VECTORS:
    exception_vector_table = VECTORS
        // Each entry MUST be 0x80 (128) bytes apart!
        "#
    );
}

/// Common main function for all architectures
/// This is called from architecture-specific entry points
pub fn main() -> ! {
    // Runtime verification hook for kernel boot process
    #[cfg(target_arch = "riscv64")]
    {
        use arch::riscv64::verification::CriticalOperation;
        verify_comprehensive!(CriticalOperation::KernelBoot, "kernel_main_entry");
    }

    unsafe {
        uart_print(b"\n=== SIS Kernel Starting ===\n");
        
        #[cfg(target_arch = "aarch64")]
        uart_print(b"Architecture: ARM64 (AArch64)\n");
        
        #[cfg(target_arch = "x86_64")]
        uart_print(b"Architecture: x86_64\n");
        
        #[cfg(target_arch = "riscv64")]
        uart_print(b"Architecture: RISC-V RV64GC\n");
        
        // Initialize heap allocator with verification hooks
        uart_print(b"HEAP: INIT\n");
        
        #[cfg(target_arch = "riscv64")]
        {
            use arch::riscv64::verification::{CriticalOperation, verify_critical_section};
            
            let heap_result = verify_critical_section(
                CriticalOperation::HeapInitialization,
                "heap_initialization",
                || heap::init_heap()
            );
            
            match heap_result {
                Ok(Ok(())) => {
                    uart_print(b"HEAP: READY (verified)\n");
                },
                Ok(Err(e)) => {
                    uart_print(b"HEAP: INIT FAILED - ");
                    uart_print(e.as_bytes());
                    uart_print(b"\n");
                },
                Err(_) => {
                    uart_print(b"HEAP: VERIFICATION FAILED\n");
                }
            }
        }
        
        #[cfg(not(target_arch = "riscv64"))]
        {
            if let Err(e) = heap::init_heap() {
                uart_print(b"HEAP: INIT FAILED - ");
                uart_print(e.as_bytes());
                uart_print(b"\n");
            } else {
                uart_print(b"HEAP: READY\n");
            }
        }
        
        // Start interactive shell with verification hook
        uart_print(b"SHELL: STARTING\n");
        
        #[cfg(target_arch = "riscv64")]
        {
            use arch::riscv64::verification::CriticalOperation;
            let _ = verify_critical_section(
                CriticalOperation::ShellCommand,
                "shell_startup",
                || shell::run_shell()
            );
        }
        
        #[cfg(not(target_arch = "riscv64"))]
        shell::run_shell();
    }
    
    // Should never reach here
    loop {
        core::hint::spin_loop();
    }
}
    // Minimal built-in shell: header + simple command loop using UART
    #[inline(never)]
    pub fn run_minishell_loop() -> ! {
        unsafe {
            crate::uart_print(b"\n=== SIS Kernel Shell (mini) ===\n");
            crate::uart_print(b"Type 'help' for commands\n\n");
        }

        static mut CMD_BUF: [u8; 256] = [0; 256];
        loop {
            unsafe { crate::uart_print(b"sis> "); }
            // Avoid creating a direct mutable reference to a `static mut`.
            let len = unsafe {
                let ptr = core::ptr::addr_of_mut!(CMD_BUF).cast::<u8>();
                let slice = core::slice::from_raw_parts_mut(ptr, 256);
                crate::uart::read_line(slice)
            };
            if len == 0 { continue; }
            let cmd = unsafe { core::str::from_utf8_unchecked(&CMD_BUF[..len]) };
            let mut parts = cmd.split_whitespace();
            match parts.next().unwrap_or("") {
                "help" => unsafe {
                    crate::uart_print(b"Commands: help, autoctl on|off|status, stresstest memory [duration_ms] [target_pressure], exit\n");
                },
                "autoctl" => {
                    match parts.next().unwrap_or("") {
                        "on" => { crate::autonomy::AUTONOMOUS_CONTROL.enable(); unsafe { crate::uart_print(b"Autonomy: ENABLED\n"); } }
                        "off" => { crate::autonomy::AUTONOMOUS_CONTROL.disable(); unsafe { crate::uart_print(b"Autonomy: DISABLED\n"); } }
                        "status" => {
                            let en = crate::autonomy::AUTONOMOUS_CONTROL.is_enabled();
                            let safe = crate::autonomy::AUTONOMOUS_CONTROL.is_safe_mode();
                            unsafe {
                                crate::uart_print(b"Autonomy status: ");
                                if en { crate::uart_print(b"ENABLED"); } else { crate::uart_print(b"DISABLED"); }
                                crate::uart_print(b", safe_mode=");
                                if safe { crate::uart_print(b"ON"); } else { crate::uart_print(b"OFF"); }
                                crate::uart_print(b", decisions=(n)\n");
                            }
                        }
                        _ => unsafe { crate::uart_print(b"Usage: autoctl on|off|status\n"); }
                    }
                }
                "stresstest" => {
                    match parts.next().unwrap_or("") {
                        "memory" => {
                            let duration_ms: u64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(2000);
                            let target: u8 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(80);
                            let mut cfg = crate::stress_test::StressTestConfig::new(crate::stress_test::StressTestType::Memory);
                            cfg.duration_ms = duration_ms;
                            cfg.target_pressure = target;
                            let _m = crate::stress_test::run_memory_stress(cfg);
                        }
                        _ => unsafe { crate::uart_print(b"Usage: stresstest memory [duration_ms] [target_pressure]\n"); }
                    }
                }
                "exit" => unsafe {
                    crate::uart_print(b"Bye.\n");
                    loop { core::hint::spin_loop(); }
                },
                _ => unsafe {
                    crate::uart_print(b"Unknown command. Type 'help'.\n");
                }
            }
        }
    }
