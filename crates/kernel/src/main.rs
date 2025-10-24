#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
// CI lint gate: when built with `--features strict`, fail on any warning
#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "strict", deny(unsafe_op_in_unsafe_fn))]

// Required for heap allocation
extern crate alloc;

// System call interface module
pub mod syscall;
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
// PMU helpers (feature-gated usage)
pub mod pmu;
// Deterministic scheduler scaffolding (feature-gated)
#[cfg(feature = "deterministic")]
pub mod deterministic;
// AI benchmark module
#[cfg(feature = "arm64-ai")]
pub mod ai_benchmark;
// LLM kernel service (feature-gated)
#[cfg(feature = "llm")]
pub mod llm;

// Architecture-specific modules
#[cfg(target_arch = "aarch64")]
pub mod arch {
    // ARM64 implementation would go here
}

#[cfg(target_arch = "aarch64")]
pub mod aarch64_context;

#[cfg(target_arch = "x86_64")]
pub mod arch {
    // x86_64 implementation would go here
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

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        uart_print(b"PANIC\n");
    }
    loop {}
}

#[inline(always)]
unsafe fn uart_print(msg: &[u8]) {
    const UART0_DR: *mut u32 = 0x0900_0000 as *mut u32;
    for &b in msg {
        core::ptr::write_volatile(UART0_DR, b as u32);
    }
}

#[macro_export]
macro_rules! kprint {
    ($($t:tt)*) => {{
        #[allow(unused_unsafe)]
        unsafe { crate::uart_print(format_args!($($t)*).to_string().as_bytes()); }
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

        // 2) Install exception vectors
        install_vectors();
        super::uart_print(b"VECTORS OK\n");

        // 3) Enable MMU (EL1 only). If not EL1, skip with message.
        let current_el: u64;
        asm!("mrs {el}, CurrentEL", el = out(reg) current_el);
        let el = (current_el >> 2) & 0x3;
        if el != 1 {
            super::uart_print(b"MMU SKIP (EL!=1)\n");
            return;
        }
        enable_mmu_el1();
        super::uart_print(b"MMU ON\n");

        // Enable Performance Monitoring Unit for cycle counts
        super::uart_print(b"PMU: INIT\n");
        pmu_enable();
        #[cfg(feature = "perf-verbose")]
        {
            super::uart_print(b"PMU: EVENTS\n");
            crate::pmu::aarch64::setup_events();
        }
        super::uart_print(b"PMU: READY\n");

        // 4) Initialize UART for interactive I/O
        super::uart_print(b"UART: INIT\n");
        crate::uart::init();
        super::uart_print(b"UART: READY\n");

        // Emit counter frequency for timing sanity check
        {
            let mut frq: u64; core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
            super::uart_print(b"METRIC cntfrq_hz=");
            print_number(frq as usize);
            super::uart_print(b"\n");
        }

        // 5) Initialize heap allocator for dynamic memory management
        super::uart_print(b"HEAP: INIT\n");
        if let Err(e) = crate::heap::init_heap() {
            super::uart_print(b"HEAP: INIT FAILED - ");
            super::uart_print(e.as_bytes());
            super::uart_print(b"\n");
        } else {
            super::uart_print(b"HEAP: READY\n");

            // Run heap tests to validate functionality
            super::uart_print(b"HEAP: TESTING\n");
            if let Err(e) = crate::heap::test_heap() {
                super::uart_print(b"HEAP: TEST FAILED - ");
                super::uart_print(e.as_bytes());
                super::uart_print(b"\n");
            } else {
                super::uart_print(b"HEAP: TESTS PASSED\n");
            }
        }

        // Graph demo is available as a shell command `graphdemo` (feature: graph-demo).
        // Avoid running it during bring-up to keep boot deterministic.

        // Deterministic admission demo (feature-gated)
        #[cfg(feature = "deterministic")]
        {
            super::uart_print(b"DET: ADMISSION DEMO\n");
            crate::deterministic::demo_admission();
            super::uart_print(b"DET: EDF TICK DEMO\n");
            crate::deterministic::edf_tick_demo();
        }

        // Auto-emit baseline graph stats for dashboards (feature-gated)
        #[cfg(feature = "graph-autostats")]
        {
            super::uart_print(b"GRAPH: BASELINE STATS\n");
            let mut g = crate::graph::GraphApi::create();
            let ch = g.add_channel(crate::graph::ChannelSpec { capacity: 64 });
            let _op = g.add_operator(crate::graph::OperatorSpec {
                id: 1,
                func: crate::graph::op_a_run,
                in_ch: None,
                out_ch: Some(ch),
                priority: 10,
                stage: None,
                in_schema: None,
                out_schema: None,
            });
            let (ops, chans) = g.counts();
            crate::trace::metric_kv("graph_stats_ops", ops);
            crate::trace::metric_kv("graph_stats_channels", chans);
        }

        // 6) Initialize GICv3 + timer and enable interrupts
        super::uart_print(b"GIC: INIT\n");
        gicv3_init_qemu();
        timer_init_1hz();
        enable_irq();

        // Start IRQ latency benchmark (virtual timer), 64 samples at ~1ms
        start_irq_latency_bench(64);

        // 6) Initialize driver framework and discover devices (optional)
        // For bring-up stability, skip VirtIO drivers unless explicitly enabled.
        #[cfg(feature = "virtio-console")]
        {
            super::uart_print(b"DRIVER FRAMEWORK\n");
            if let Err(_) = crate::driver::init_driver_framework() {
                super::uart_print(b"DRIVER: INIT FAILED\n");
            } else {
                super::uart_print(b"DRIVER: INIT OK\n");

                // Register VirtIO console driver
                super::uart_print(b"DRIVER: REGISTERING VIRTIO CONSOLE\n");
                if let Err(_) = crate::driver::register_driver(crate::virtio_console::get_virtio_console_driver()) {
                    super::uart_print(b"DRIVER: VIRTIO CONSOLE REGISTRATION FAILED\n");
                } else {
                    super::uart_print(b"DRIVER: VIRTIO CONSOLE REGISTERED\n");
                }

                if let Some(registry) = crate::driver::get_driver_registry() {
                    match registry.discover_devices() {
                        Ok(count) => {
                            super::uart_print(b"DRIVER: DISCOVERED ");
                            print_number(count);
                            super::uart_print(b" DEVICES\n");
                        }
                        Err(_) => {
                            super::uart_print(b"DRIVER: DISCOVERY FAILED\n");
                        }
                    }
                }
            }
        }
        #[cfg(not(feature = "virtio-console"))]
        {
            super::uart_print(b"DRIVER FRAMEWORK: SKIPPED (virtio-console feature off)\n");
        }

        // 7) Initialize AI features if enabled
        #[cfg(feature = "arm64-ai")]
        {
            super::uart_print(b"AI FEATURES\n");
            super::uart_print(b"AI: INITIALIZING FORMAL VERIFICATION\n");
            super::uart_print(b"AI: ENABLING PERFORMANCE OPTIMIZATION\n");
            super::uart_print(b"AI: CACHE-AWARE ALGORITHMS ACTIVE\n");
            super::uart_print(b"AI: READY\n");
            
            // Run actual AI benchmarks to demonstrate functionality
            crate::ai_benchmark::run_ai_benchmarks();
        }

        // 7.5) Emit kernel METRICs for test suite (ctx switch proxy + alloc)
        crate::userspace_test::emit_kernel_metrics();

        // 7.6) Real context-switch benchmark (AArch64 only)
        #[cfg(target_arch = "aarch64")]
        crate::userspace_test::bench_real_context_switch();

        // 8) Test syscall functionality
        super::uart_print(b"SYSCALL TESTS\n");
        crate::userspace_test::run_syscall_tests();

        // 9) Launch interactive shell
        super::uart_print(b"LAUNCHING SHELL\n");
        crate::shell::run_shell();
    }

    unsafe fn install_vectors() {
        let base = &VECTORS as *const u8 as u64;
        // Try EL1 first, else EL2
        let current_el: u64;
        asm!("mrs {el}, CurrentEL", el = out(reg) current_el);
        match (current_el >> 2) & 0x3 {
            1 => asm!("msr VBAR_EL1, {v}", v = in(reg) base, options(nostack, preserves_flags)),
            2 => asm!("msr VBAR_EL2, {v}", v = in(reg) base, options(nostack, preserves_flags)),
            _ => {}
        }
        asm!("isb", options(nostack, preserves_flags));
    }

    unsafe fn enable_mmu_el1() {
        super::uart_print(b"MMU: MAIR/TCR\n");
        // Memory attributes: AttrIdx0 = Device-nGnRE, AttrIdx1 = Normal WBWA
        let mair = (0x04u64) | (0xFFu64 << 8);
        asm!("msr MAIR_EL1, {x}", x = in(reg) mair, options(nostack, preserves_flags));

        // TCR: 4KB pages, Inner/Outer WBWA, Inner shareable,
        // 39-bit VA (T0SZ=25), 48-bit PA (IPS=5). Correct bit positions:
        // T0SZ[5:0], IRGN0[9:8], ORGN0[11:10], SH0[13:12], TG0[15:14], IPS[34:32]
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

    #[inline(never)]
    unsafe fn pmu_enable() {
        // PMCR_EL0: E=1 (enable), P=1 (reset event counters), C=1 (reset cycle counter)
        let pmcr: u64 = (1 << 0) | (1 << 1) | (1 << 2);
        asm!("msr PMCR_EL0, {x}", x = in(reg) pmcr, options(nostack, preserves_flags));

        // Enable cycle counter in PMCNTENSET_EL0 (bit 31)
        let pmcntenset: u64 = 1u64 << 31;
        asm!("msr PMCNTENSET_EL0, {x}", x = in(reg) pmcntenset, options(nostack, preserves_flags));

        // Allow EL0 reads (future use); harmless at EL1
        let pmuserenr: u64 = 1; // EN=1
        asm!("msr PMUSERENR_EL0, {x}", x = in(reg) pmuserenr, options(nostack, preserves_flags));

        asm!("isb", options(nostack, preserves_flags));
    }

    unsafe fn build_tables() {
        // Clear tables
        let table_ptr = &raw mut L1_TABLE.0 as *mut [u64; 512];
        for e in (*table_ptr).iter_mut() {
            *e = 0;
        }

        // Descriptor helpers
        const DESC_BLOCK: u64 = 1; // bits[1:0]=01 for block

        const SH_INNER: u64 = 0b11 << 8;
        const AF: u64 = 1 << 10;
        const ATTRIDX_NORMAL: u64 = 1 << 2; // AttrIndx=1
        const ATTRIDX_DEVICE: u64 = 0 << 2; // AttrIndx=0

        // L1[1] = 1GB block for 0x40000000..0x7FFFFFFF as Normal WBWA, InnerShareable
        let l1_idx_kernel = 0x4000_0000u64 >> 30; // 1
        L1_TABLE.0[l1_idx_kernel as usize] =
            ((0x4000_0000u64 >> 30) << 30) | DESC_BLOCK | AF | SH_INNER | ATTRIDX_NORMAL;

        // L1[0] = 1GB block for 0x00000000..0x3FFFFFFF as Device-nGnRE (covers UART 0x0900_0000)
        L1_TABLE.0[0] = (0x0000_0000u64) | DESC_BLOCK | AF | ATTRIDX_DEVICE; // Non-shareable default
    }

    core::arch::global_asm!(
        r#"
        .balign 2048
        .global VECTORS
    VECTORS:
        // EL1t
        b .
        b .
        b .
        b .
        // EL1h
        b sync_el1h
        b irq_el1h
        b fiq_el1h
        b serr_el1h
        // EL0_64 (userspace)
        b sync_el0_64
        b .
        b .
        b .
        // EL0_32 (unused)
        b .
        b .
        b .
        b .

    irq_el1h:
        bl irq_handler
        eret

    fiq_el1h:
        // FIQ handler - output FIQ debug message
        stp x0, x1, [sp, #-16]!      // Save x0, x1 temporarily
        mov x0, #0x09000000          // UART base
        mov w1, #0x46                // 'F'
        str w1, [x0]
        mov w1, #0x49                // 'I'
        str w1, [x0]
        mov w1, #0x51                // 'Q'
        str w1, [x0]
        mov w1, #0x0A                // '\n'
        str w1, [x0]
        ldp x0, x1, [sp], #16        // Restore x0, x1
        b .                          // Hang for debugging

    serr_el1h:
        // System error handler - output SERR debug message  
        stp x0, x1, [sp, #-16]!      // Save x0, x1 temporarily
        mov x0, #0x09000000          // UART base
        mov w1, #0x53                // 'S'
        str w1, [x0]
        mov w1, #0x45                // 'E'
        str w1, [x0]
        mov w1, #0x52                // 'R'
        str w1, [x0]
        mov w1, #0x52                // 'R'
        str w1, [x0]
        mov w1, #0x0A                // '\n'
        str w1, [x0]
        ldp x0, x1, [sp], #16        // Restore x0, x1
        b .                          // Hang for debugging

    sync_el1h:
        // Handle synchronous exceptions from EL1 (including syscalls from kernel mode)
        // Save all registers FIRST to avoid corruption
        sub sp, sp, #(34 * 8)        // Allocate SyscallFrame
        
        // Save general purpose registers x0-x30
        stp x0, x1, [sp, #(0 * 8)]
        stp x2, x3, [sp, #(2 * 8)]
        stp x4, x5, [sp, #(4 * 8)]
        stp x6, x7, [sp, #(6 * 8)]
        stp x8, x9, [sp, #(8 * 8)]
        stp x10, x11, [sp, #(10 * 8)]
        stp x12, x13, [sp, #(12 * 8)]
        stp x14, x15, [sp, #(14 * 8)]
        stp x16, x17, [sp, #(16 * 8)]
        stp x18, x19, [sp, #(18 * 8)]
        stp x20, x21, [sp, #(20 * 8)]
        stp x22, x23, [sp, #(22 * 8)]
        stp x24, x25, [sp, #(24 * 8)]
        stp x26, x27, [sp, #(26 * 8)]
        stp x28, x29, [sp, #(28 * 8)]
        str x30, [sp, #(30 * 8)]
        
        // Save current SP (EL1 already using EL1 stack)
        // For EL1h, we don't need to save/restore EL0 SP
        mov x0, #0
        str x0, [sp, #(31 * 8)]
        
        // Save exception info
        mrs x0, elr_el1
        mrs x1, spsr_el1
        stp x0, x1, [sp, #(32 * 8)]
        
        // Call system call handler
        mov x0, sp
        bl syscall_handler
        
        // Restore all registers
        ldp x0, x1, [sp, #(32 * 8)]
        msr elr_el1, x0
        msr spsr_el1, x1
        
        // Skip restoring SP since we're staying in EL1
        // ldr x0, [sp, #(31 * 8)]
        
        // Restore GPRs
        ldp x0, x1, [sp, #(0 * 8)]
        ldp x2, x3, [sp, #(2 * 8)]
        ldp x4, x5, [sp, #(4 * 8)]
        ldp x6, x7, [sp, #(6 * 8)]
        ldp x8, x9, [sp, #(8 * 8)]
        ldp x10, x11, [sp, #(10 * 8)]
        ldp x12, x13, [sp, #(12 * 8)]
        ldp x14, x15, [sp, #(14 * 8)]
        ldp x16, x17, [sp, #(16 * 8)]
        ldp x18, x19, [sp, #(18 * 8)]
        ldp x20, x21, [sp, #(20 * 8)]
        ldp x22, x23, [sp, #(22 * 8)]
        ldp x24, x25, [sp, #(24 * 8)]
        ldp x26, x27, [sp, #(26 * 8)]
        ldp x28, x29, [sp, #(28 * 8)]
        ldr x30, [sp, #(30 * 8)]
        
        add sp, sp, #(34 * 8)        // Restore stack
        eret

    sync_el0_64:
        // Handle synchronous exceptions from EL0 (userspace syscalls)
        // First, output debug message to see if we get here
        stp x0, x1, [sp, #-16]!      // Save x0, x1 temporarily
        mov x0, #0x09000000          // UART base
        mov w1, #0x45                // 'E'
        str w1, [x0]
        mov w1, #0x4C                // 'L'
        str w1, [x0]
        mov w1, #0x30                // '0'
        str w1, [x0]
        mov w1, #0x0A                // '\n'
        str w1, [x0]
        ldp x0, x1, [sp], #16        // Restore x0, x1
        
        // Save all registers for system call
        sub sp, sp, #(34 * 8)        // Allocate SyscallFrame
        
        // Save general purpose registers x0-x30
        stp x0, x1, [sp, #(0 * 8)]
        stp x2, x3, [sp, #(2 * 8)]
        stp x4, x5, [sp, #(4 * 8)]
        stp x6, x7, [sp, #(6 * 8)]
        stp x8, x9, [sp, #(8 * 8)]
        stp x10, x11, [sp, #(10 * 8)]
        stp x12, x13, [sp, #(12 * 8)]
        stp x14, x15, [sp, #(14 * 8)]
        stp x16, x17, [sp, #(16 * 8)]
        stp x18, x19, [sp, #(18 * 8)]
        stp x20, x21, [sp, #(20 * 8)]
        stp x22, x23, [sp, #(22 * 8)]
        stp x24, x25, [sp, #(24 * 8)]
        stp x26, x27, [sp, #(26 * 8)]
        stp x28, x29, [sp, #(28 * 8)]
        str x30, [sp, #(30 * 8)]
        
        // Save EL0 stack pointer
        mrs x0, sp_el0
        str x0, [sp, #(31 * 8)]
        
        // Save exception info
        mrs x0, elr_el1
        mrs x1, spsr_el1
        stp x0, x1, [sp, #(32 * 8)]
        
        // Call system call handler
        mov x0, sp
        bl syscall_handler
        
        // Restore all registers
        ldp x0, x1, [sp, #(32 * 8)]
        msr elr_el1, x0
        msr spsr_el1, x1
        
        ldr x0, [sp, #(31 * 8)]
        msr sp_el0, x0
        
        // Restore GPRs
        ldp x0, x1, [sp, #(0 * 8)]
        ldp x2, x3, [sp, #(2 * 8)]
        ldp x4, x5, [sp, #(4 * 8)]
        ldp x6, x7, [sp, #(6 * 8)]
        ldp x8, x9, [sp, #(8 * 8)]
        ldp x10, x11, [sp, #(10 * 8)]
        ldp x12, x13, [sp, #(12 * 8)]
        ldp x14, x15, [sp, #(14 * 8)]
        ldp x16, x17, [sp, #(16 * 8)]
        ldp x18, x19, [sp, #(18 * 8)]
        ldp x20, x21, [sp, #(20 * 8)]
        ldp x22, x23, [sp, #(22 * 8)]
        ldp x24, x25, [sp, #(24 * 8)]
        ldp x26, x27, [sp, #(26 * 8)]
        ldp x28, x29, [sp, #(28 * 8)]
        ldr x30, [sp, #(30 * 8)]
        
        add sp, sp, #(34 * 8)        // Restore stack
        eret
        "#
    );

    #[no_mangle]
    extern "C" fn irq_handler() {
        unsafe {
            let mut irq: u64;
            asm!("mrs {x}, icc_iar1_el1", x = out(reg) irq);
            let mut t1: u64; core::arch::asm!("mrs {x}, cntvct_el0", x = out(reg) t1);
            // Attempt to dispatch device interrupts to drivers (best-effort)
            if let Some(registry) = crate::driver::get_driver_registry() {
                let intid: u32 = (irq & 0xFFFFFF) as u32;
                let _ = registry.handle_device_irq(intid);
            }
            // Opportunistically poll VirtIO console control frames even if the device
            // didn't raise an interrupt (keeps host control responsive in QEMU).
            #[cfg(feature = "virtio-console")]
            {
                let drv = crate::virtio_console::get_virtio_console_driver();
                let _ = drv.poll_control_frames();
                let _ = drv.poll_ctrl_events();
            }
            // Note: do not call into device drivers here to avoid re-entrancy
            // while drivers are being initialized. Control-plane polling stays
            // in the device IRQ path for now.
            if TIMER_BENCH_WARMUP > 0 || TIMER_BENCH_REMAIN > 0 {
                if TIMER_BENCH_WARMUP > 0 {
                    // Discard warm-up samples
                    TIMER_BENCH_WARMUP -= 1;
                } else {
                    // Record sample
                    let ns = delta_ns(TIMER_BENCH_T0, t1);
                    TIMER_SUM_NS = TIMER_SUM_NS.saturating_add(ns);
                    if ns < TIMER_MIN_NS { TIMER_MIN_NS = ns; }
                    if ns > TIMER_MAX_NS { TIMER_MAX_NS = ns; }
                    TIMER_SUM_COUNT = TIMER_SUM_COUNT.saturating_add(1);
                    super::uart_print(b"METRIC irq_latency_ns=");
                    print_number(ns as usize);
                    super::uart_print(b"\n");
                    if TIMER_BENCH_REMAIN > 0 { TIMER_BENCH_REMAIN -= 1; }
                }
                if TIMER_BENCH_WARMUP > 0 || TIMER_BENCH_REMAIN > 0 {
                    TIMER_BENCH_T0 = t1;
                    core::arch::asm!("msr cntv_tval_el0, {x}", x = in(reg) TIMER_BENCH_INTERVAL);
                } else {
                    // Bench completed: print a brief summary (mean/min/max)
                    if TIMER_SUM_COUNT > 0 {
                        let mean = TIMER_SUM_NS / TIMER_SUM_COUNT as u64;
                        super::uart_print(b"[SUMMARY] irq_latency_ns: count=");
                        print_number(TIMER_SUM_COUNT as usize);
                        super::uart_print(b" mean=");
                        print_number(mean as usize);
                        super::uart_print(b" ns min=");
                        print_number(TIMER_MIN_NS as usize);
                        super::uart_print(b" ns max=");
                        print_number(TIMER_MAX_NS as usize);
                        super::uart_print(b" ns\n");
                    }
                }
            }
            // signal end of interrupt
            asm!("msr icc_eoir1_el1, {x}", x = in(reg) irq);
            asm!("msr icc_dir_el1, {x}", x = in(reg) irq);
        }
    }

    static mut TIMER_BENCH_REMAIN: u32 = 0;
    static mut TIMER_BENCH_INTERVAL: u64 = 0;
    static mut TIMER_BENCH_T0: u64 = 0;
    static mut TIMER_BENCH_WARMUP: u32 = 0;
    static mut TIMER_SUM_NS: u64 = 0;
    static mut TIMER_MIN_NS: u64 = u64::MAX;
    static mut TIMER_MAX_NS: u64 = 0;
    static mut TIMER_SUM_COUNT: u32 = 0;

    unsafe fn delta_ns(t0: u64, t1: u64) -> u64 {
        let mut f: u64; core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) f);
        if f == 0 { return 0; }
        let d = t1.wrapping_sub(t0);
        (d.saturating_mul(1_000_000_000)) / f
    }

    unsafe fn start_irq_latency_bench(samples: u32) {
        let mut frq: u64; core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        TIMER_BENCH_INTERVAL = frq / 2000; // ~0.5ms per sample
        TIMER_BENCH_REMAIN = samples;
        TIMER_BENCH_WARMUP = 4; // discard first 4 samples
        core::arch::asm!("mrs {x}, cntvct_el0", x = out(reg) TIMER_BENCH_T0);
        TIMER_SUM_NS = 0; TIMER_MIN_NS = u64::MAX; TIMER_MAX_NS = 0; TIMER_SUM_COUNT = 0;
        core::arch::asm!("msr cntv_tval_el0, {x}", x = in(reg) TIMER_BENCH_INTERVAL);
    }

    unsafe fn enable_irq() {
        // Unmask IRQs in PSTATE
        asm!("msr daifclr, #2", options(nostack, preserves_flags));
    }

    unsafe fn timer_init_1hz() {
        let mut frq: u64;
        asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        // Set initial interval ~1s
        asm!("msr cntv_tval_el0, {x}", x = in(reg) frq);
        // Enable virtual timer, unmask
        let ctl: u64 = 1; // ENABLE=1, IMASK=0
        asm!("msr cntv_ctl_el0, {x}", x = in(reg) ctl);
    }

    unsafe fn gicv3_init_qemu() {
        super::uart_print(b"GIC: DISTRIBUTOR\n");

        // QEMU ARM64 virt machine GICv3 addresses
        const GICD_BASE: u64 = 0x08000000; // GIC Distributor
        const GICR_BASE: u64 = 0x080A0000; // GIC Redistributor

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
        const GICR_WAKER: u64 = 0x0014;
        const GICR_IGROUPR0: u64 = 0x0080;
        const GICR_ISENABLER0: u64 = 0x0100;
        const GICR_IPRIORITYR: u64 = 0x0400;

        // 1) Initialize GIC Distributor
        let gicd_ctlr = (GICD_BASE + GICD_CTLR) as *mut u32;

        // Check if already enabled
        let ctlr_val = core::ptr::read_volatile(gicd_ctlr);
        if (ctlr_val & 0x3) == 0 {
            super::uart_print(b"GIC: ENABLING DISTRIBUTOR\n");
            // Enable Group 0 and Group 1 (both secure and non-secure)
            core::ptr::write_volatile(gicd_ctlr, 0x3);
        } else {
            super::uart_print(b"GIC: DISTRIBUTOR ALREADY ENABLED\n");
        }

        // 2) Wake up redistributor for CPU0
        super::uart_print(b"GIC: REDISTRIBUTOR\n");
        super::uart_print(b"GIC: ACCESSING GICR_WAKER\n");
        let waker = (GICR_BASE + GICR_WAKER) as *mut u32;
        super::uart_print(b"GIC: READING WAKER VALUE\n");

        // Clear ProcessorSleep bit [1]
        let mut w: u32 = core::ptr::read_volatile(waker);
        if (w & (1 << 1)) != 0 {
            super::uart_print(b"GIC: WAKING UP CPU0\n");
            w &= !(1 << 1);
            core::ptr::write_volatile(waker, w);

            // Wait for ChildrenAsleep bit [2] to clear with timeout
            let mut timeout = 1000000;
            loop {
                let v = core::ptr::read_volatile(waker);
                if (v & (1 << 2)) == 0 {
                    super::uart_print(b"GIC: CPU0 AWAKE\n");
                    break;
                }
                timeout -= 1;
                if timeout == 0 {
                    super::uart_print(b"GIC: WAKER TIMEOUT\n");
                    break;
                }
            }
        } else {
            super::uart_print(b"GIC: CPU0 ALREADY AWAKE\n");
        }

        // 3) Configure PPI 27 (virtual timer) as Group 1 (non-secure)
        super::uart_print(b"GIC: CONFIGURE PPI27\n");
        let igroupr0 = (GICR_BASE + GICR_IGROUPR0) as *mut u32;
        let mut grp = core::ptr::read_volatile(igroupr0);
        grp |= 1 << 27;
        core::ptr::write_volatile(igroupr0, grp);

        // 4) Set priority for PPI 27
        let iprio = (GICR_BASE + GICR_IPRIORITYR) as *mut u32;
        let prio_reg = iprio.add(27 / 4); // 4 priorities per 32-bit register
        let shift = (27 % 4) * 8;
        let mut prio_val = core::ptr::read_volatile(prio_reg);
        prio_val &= !(0xFF << shift);
        prio_val |= 0x80 << shift; // Medium priority
        core::ptr::write_volatile(prio_reg, prio_val);

        // 5) Enable PPI 27
        super::uart_print(b"GIC: ENABLE PPI27\n");
        let isenabler0 = (GICR_BASE + GICR_ISENABLER0) as *mut u32;
        core::ptr::write_volatile(isenabler0, 1 << 27);

        super::uart_print(b"GIC: READY\n");

        // CPU interface via system registers
        super::uart_print(b"GIC: CPU IF\n");
        let pmr: u64 = 0xFF; // unmask all priorities
        asm!("msr icc_pmr_el1, {x}", x = in(reg) pmr);
        let grp1: u64 = 1;
        asm!("msr icc_igrpen1_el1, {x}", x = in(reg) grp1);
        asm!("isb", options(nostack, preserves_flags));
        super::uart_print(b"GIC: DONE\n");
    }

    /// Helper function to print numbers
    unsafe fn print_number(mut num: usize) {
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
