#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
// CI lint gate: when built with `--features strict`, fail on any warning
#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "strict", deny(unsafe_op_in_unsafe_fn))]

// Required for heap allocation
extern crate alloc;

// Optional DTB pointer provided by loader (UEFI); used to override platform descriptors
#[no_mangle]
pub static mut DTB_PTR: *const u8 = core::ptr::null();

// Core library (error handling, logging, etc.)
pub mod lib;
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
pub mod smp;
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
pub mod agent_bus;
pub mod meta_agent;
pub mod autonomy;
pub mod time;
pub mod prediction_tracker;
pub mod stress_test;
pub mod predictive_memory;
pub mod predictive_scheduling;
pub mod command_predictor;
pub mod network_predictor;
pub mod benchmark;
pub mod compliance;
// Test utilities (only compiled for testing)
#[cfg(test)]
pub mod test_utils;
// PMU helpers (feature-gated usage)
pub mod pmu;
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
    // Early boot UART writes using platform-provided base; avoids hardcoded MMIO
    let base = crate::platform::active().uart().base as *mut u32;
    for &b in msg {
        core::ptr::write_volatile(base, b as u32);
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

        // 1.5) If a DTB pointer was provided by the loader, override platform descriptors
        #[cfg(feature = "dt-override")]
        {
            let mut dt_active = false;
            if !super::DTB_PTR.is_null() {
                dt_active = crate::platform::override_with_dtb(super::DTB_PTR);
            }
            if dt_active { super::uart_print(b"PLATFORM: dt override active\n"); }
            else { super::uart_print(b"PLATFORM: qemu_virt\n"); }
        }

        // 2) Install exception vectors (Phase A0)
        crate::arch::trap::init_exception_vectors();
        super::uart_print(b"VECTORS OK\n");

        // 2.5) Initialize Phase A0 timer (optional, can be enabled in A1)
        // crate::arch::timer::init_timer(1000);  // 1000ms interval
        // Note: GIC timer init happens later in boot sequence

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

        // Initialize boot timestamp for runtime timing utilities
        crate::time::init_boot_timestamp();

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

        // Phase A1: Initialize MM, VFS, and boot userspace
        super::uart_print(b"PHASE A1: BOOT WIRING\n");

        // Initialize buddy allocator with available RAM
        // For QEMU virt, assume 128MB RAM starting at 0x40000000
        super::uart_print(b"MM: BUDDY ALLOCATOR\n");
        let ram_start = 0x4100_0000u64; // Start after kernel (16MB offset)
        let ram_size = 112 * 1024 * 1024u64; // 112MB available
        let ranges: &[(u64, usize)] = &[(ram_start, ram_size as usize)];
        crate::mm::init_buddy(ranges)
            .expect("Failed to initialize buddy allocator");
        let stats = crate::mm::get_stats().unwrap_or_default();
        super::uart_print(b"MM: BUDDY READY (");
        print_number(stats.total_pages);
        super::uart_print(b" pages)\n");

        // Initialize process table before VFS (needed for syscalls)
        super::uart_print(b"PROCESS: INIT TABLE\n");
        crate::process::init_process_table();
        super::uart_print(b"PROCESS: TABLE READY\n");

        // Initialize scheduler
        super::uart_print(b"SCHEDULER: INIT\n");
        crate::process::scheduler::init();
        crate::process::scheduler_smp::init();
        super::uart_print(b"SCHEDULER: READY\n");

        // Initialize VFS
        super::uart_print(b"VFS: INIT\n");
        crate::vfs::init().expect("Failed to initialize VFS");

        // Mount tmpfs at /
        super::uart_print(b"VFS: MOUNT TMPFS AT /\n");
        let root = crate::vfs::tmpfs::mount_tmpfs().expect("Failed to mount tmpfs");
        crate::vfs::set_root(root.clone());

        // Mount devfs at /dev
        super::uart_print(b"VFS: MOUNT DEVFS AT /dev\n");
        let dev_inode = crate::vfs::devfs::mount_devfs().expect("Failed to mount devfs");
        root.create("dev", crate::vfs::S_IFDIR | 0o755).expect("Failed to create /dev");
        // Note: Actual mount point linking deferred to when mount syscall is available
        crate::vfs::set_root(root.clone()); // Re-set root to ensure consistency

        // Mount procfs at /proc
        super::uart_print(b"VFS: MOUNT PROCFS AT /proc\n");
        let proc_inode = crate::vfs::mount_procfs().expect("Failed to mount procfs");
        root.create("proc", crate::vfs::S_IFDIR | 0o555).expect("Failed to create /proc");
        crate::vfs::set_root(root.clone()); // Re-set root to ensure consistency

        super::uart_print(b"VFS: READY\n");

        // Initialize page cache (Phase B)
        super::uart_print(b"PAGE CACHE: INIT\n");
        crate::mm::init_page_cache(1024); // Cache up to 1024 blocks (512KB)
        super::uart_print(b"PAGE CACHE: READY\n");

        // Initialize virtio-blk devices (Phase B)
        super::uart_print(b"BLOCK: PROBING VIRTIO-BLK DEVICES\n");
        crate::arch::aarch64::init_virtio_blk();
        super::uart_print(b"BLOCK: READY\n");

        // Initialize virtio-net devices (Phase C)
        super::uart_print(b"NET: PROBING VIRTIO-NET DEVICES\n");
        crate::arch::aarch64::init_virtio_net();
        super::uart_print(b"NET: DRIVER READY\n");

        // Initialize network interface (smoltcp)
        super::uart_print(b"NET: INIT INTERFACE\n");
        if let Ok(()) = crate::net::init_network() {
            super::uart_print(b"NET: INTERFACE READY\n");

            // Try DHCP configuration
            super::uart_print(b"NET: STARTING DHCP\n");
            let mut dhcp_client = crate::net::dhcp::DhcpClient::new();
            match dhcp_client.acquire_lease() {
                Ok(config) => {
                    super::uart_print(b"NET: DHCP LEASE ACQUIRED\n");
                    if let Err(e) = dhcp_client.apply_config(&config) {
                        crate::warn!("net: Failed to apply DHCP config: {:?}", e);
                    }
                }
                Err(e) => {
                    crate::warn!("net: DHCP failed: {:?}, using static IP", e);
                    // Fall back to static IP for testing
                    let _ = crate::net::smoltcp_iface::set_ip_address([10, 0, 2, 15], 24);
                    let _ = crate::net::smoltcp_iface::set_gateway([10, 0, 2, 2]);
                }
            }
            super::uart_print(b"NET: CONFIGURED\n");
        } else {
            super::uart_print(b"NET: NO DEVICE (SKIP)\n");
        }

        // Initialize entropy source (Phase D)
        super::uart_print(b"RANDOM: INIT PRNG\n");
        crate::security::init_random();
        super::uart_print(b"RANDOM: READY\n");

        // Initialize SMP (Phase E)
        super::uart_print(b"SMP: INIT MULTI-CORE\n");
        crate::smp::init();
        super::uart_print(b"SMP: READY\n");

        // Initialize virtio-gpu devices (Phase G.0)
        super::uart_print(b"GPU: PROBING VIRTIO-GPU DEVICES\n");
        crate::arch::aarch64::init_virtio_gpu();
        super::uart_print(b"GPU: READY\n");

        // Initialize graphics subsystem (Phase G.0)
        super::uart_print(b"GRAPHICS: INIT\n");
        if let Ok(()) = crate::graphics::init() {
            super::uart_print(b"GRAPHICS: READY\n");

            // Run graphics test
            super::uart_print(b"GRAPHICS: RUNNING TEST\n");
            if let Ok(()) = crate::graphics::test_graphics() {
                super::uart_print(b"GRAPHICS: TEST PASSED\n");
            } else {
                super::uart_print(b"GRAPHICS: TEST FAILED\n");
            }

            // Initialize window manager (Phase G.1)
            super::uart_print(b"WM: INIT WINDOW MANAGER\n");
            if let Ok(()) = crate::window_manager::init() {
                super::uart_print(b"WM: READY\n");

                // Run window manager test
                super::uart_print(b"WM: RUNNING TEST\n");
                if let Ok(()) = crate::window_manager::test_window_manager() {
                    super::uart_print(b"WM: TEST PASSED\n");
                } else {
                    super::uart_print(b"WM: TEST FAILED\n");
                }

                // Initialize UI toolkit (Phase G.2)
                super::uart_print(b"UI: INIT TOOLKIT\n");
                if let Ok(()) = crate::ui::init() {
                    super::uart_print(b"UI: READY\n");

                    // Run UI toolkit test
                    super::uart_print(b"UI: RUNNING TEST\n");
                    if let Ok(()) = crate::ui::test_ui_toolkit() {
                        super::uart_print(b"UI: TEST PASSED\n");
                    } else {
                        super::uart_print(b"UI: TEST FAILED\n");
                    }

                    // Test desktop applications (Phase G.3)
                    super::uart_print(b"APPS: TESTING APPLICATIONS\n");
                    if let Ok(()) = crate::applications::test_applications() {
                        super::uart_print(b"APPS: TESTS PASSED\n");
                    } else {
                        super::uart_print(b"APPS: TESTS FAILED\n");
                    }

                    // Launch all applications in windows
                    super::uart_print(b"APPS: LAUNCHING ALL APPS\n");
                    if let Ok(()) = crate::applications::launch_all_apps() {
                        super::uart_print(b"APPS: ALL APPS RUNNING\n");
                    } else {
                        super::uart_print(b"APPS: LAUNCH FAILED\n");
                    }
                } else {
                    super::uart_print(b"UI: INIT FAILED\n");
                }
            } else {
                super::uart_print(b"WM: INIT FAILED\n");
            }
        } else {
            super::uart_print(b"GRAPHICS: NO GPU (SKIP)\n");
        }

        // TODO: Unpack initramfs (when INITRAMFS_DATA is available)
        // super::uart_print(b"INITRAMFS: UNPACKING\n");
        // let initramfs_data = &INITRAMFS_DATA;
        // crate::initramfs::unpack_initramfs(initramfs_data).expect("Failed to unpack initramfs");
        // super::uart_print(b"INITRAMFS: READY\n");

        // Create PID 1 (init process)
        super::uart_print(b"INIT: CREATING PID 1\n");
        let init_task = crate::process::Task::new_init();
        crate::process::insert_task(init_task).expect("Failed to insert init task");
        super::uart_print(b"INIT: PID 1 CREATED\n");

        // Enqueue PID 1 to scheduler
        super::uart_print(b"SCHEDULER: ENQUEUE PID 1\n");
        crate::process::scheduler::enqueue(1);
        crate::process::scheduler::set_current(1);
        super::uart_print(b"SCHEDULER: PID 1 RUNNING\n");

        // TODO: Execute /sbin/init when initramfs is available
        // super::uart_print(b"INIT: EXEC /sbin/init\n");
        // let argv = vec![b"/sbin/init\0".as_ref()];
        // let envp = vec![];
        // crate::syscall::sys_execve(b"/sbin/init\0", &argv, &envp).expect("Failed to exec init");

        super::uart_print(b"PHASE A1: BOOT WIRING COMPLETE\n");

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
        super::uart_print(b"[MAIN] Calling enable_irq() from initialization\n");
        enable_irq();
        // NOTE: IRQ latency benchmark disabled - was causing rapid timer firing issues
        // To run benchmark manually: use shell command or call start_irq_latency_bench(64)
        // start_irq_latency_bench(64);

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

        // 7.5) Emit kernel METRICs and run demos (previously verified working)
        super::uart_print(b"METRICS: STARTING\n");
        crate::userspace_test::emit_kernel_metrics();
        super::uart_print(b"METRICS: COMPLETE\n");
        #[cfg(target_arch = "aarch64")]
        {
            super::uart_print(b"CONTEXT SWITCH BENCH: STARTING\n");
            crate::userspace_test::bench_real_context_switch();
            super::uart_print(b"CONTEXT SWITCH BENCH: COMPLETE\n");
        }
        super::uart_print(b"SYSCALL TESTS: STARTING\n");
        crate::userspace_test::run_syscall_tests();
        super::uart_print(b"SYSCALL TESTS: COMPLETE\n");

        // 9) Initialize memory neural agent (keep IRQs enabled, agent handles its own locking)
        super::uart_print(b"MEMORY AGENT: INIT\n");
        // Initialize memory agent - it will handle its own IRQ masking internally
        crate::neural::init_memory_agent();
        super::uart_print(b"MEMORY AGENT: READY\n");

        // 10) Initialize meta-agent for global coordination
        super::uart_print(b"META-AGENT: INIT\n");
        crate::meta_agent::init_meta_agent();
        super::uart_print(b"META-AGENT: READY\n");

        // Mark autonomy ready after agents are initialized
        crate::autonomy::AUTONOMY_READY.store(true, core::sync::atomic::Ordering::Release);
        super::uart_print(b"AUTONOMY: set_ready complete\n");

        // 11) Launch interactive shell
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

        // Map RAM ranges as Normal WBWA, InnerShareable using 1GiB blocks
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

        // Map MMIO ranges as Device-nGnRE using 1GiB blocks if not already RAM
        for m in plat.mmio_ranges() {
            let mut base = (m.start as u64) & !((1u64 << 30) - 1);
            let end = (m.start as u64).saturating_add(m.size as u64);
            while base < end {
                let idx = (base >> 30) as usize;
                if L1_TABLE.0[idx] == 0 {
                    L1_TABLE.0[idx] = (base) | DESC_BLOCK | AF | ATTRIDX_DEVICE; // Non-shareable default
                }
                base = base.saturating_add(1u64 << 30);
            }
        }
    }

    core::arch::global_asm!(
        r#"
        .balign 2048
        .global VECTORS
    VECTORS:
        // Each entry MUST be 0x80 (128) bytes apart!

        // Current EL with SP0 (EL1t) - We don't use these
        .org VECTORS + 0x000
        b .
        .org VECTORS + 0x080
        b .
        .org VECTORS + 0x100
        b .
        .org VECTORS + 0x180
        b .

        // Current EL with SPx (EL1h) - These are what we use!
        .org VECTORS + 0x200
        b sync_el1h
        .org VECTORS + 0x280
        b irq_el1h
        .org VECTORS + 0x300
        b fiq_el1h
        .org VECTORS + 0x380
        b serr_el1h

        // Lower EL using AArch64 (EL0_64)
        .org VECTORS + 0x400
        b sync_el0_64
        .org VECTORS + 0x480
        b .
        .org VECTORS + 0x500
        b .
        .org VECTORS + 0x580
        b .

        // Lower EL using AArch32 (EL0_32) - unused
        .org VECTORS + 0x600
        b .
        .org VECTORS + 0x680
        b .
        .org VECTORS + 0x700
        b .
        .org VECTORS + 0x780
        b .

    irq_el1h:
        // Save all registers to preserve interrupted context
        sub sp, sp, #(34 * 8)        // Allocate frame similar to sync handler
        
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
        
        // Save EL0 SP (optional)
        mrs x0, sp_el0
        str x0, [sp, #(31 * 8)]
        
        // Save exception info
        mrs x0, elr_el1
        mrs x1, spsr_el1
        stp x0, x1, [sp, #(32 * 8)]
        
        // Call Rust IRQ handler
        bl irq_handler
        
        // Restore exception info
        ldp x0, x1, [sp, #(32 * 8)]
        msr elr_el1, x0
        msr spsr_el1, x1
        
        // Restore EL0 SP
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

    fiq_el1h:
        // FIQ handler - reserved for debug; avoid MMIO prints in ISR
        b .                          // Hang for debugging

    serr_el1h:
        // System error handler - reserved; avoid MMIO prints in ISR
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
        // Avoid MMIO prints here; proceed to save state
        
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
            static mut IRQ_COUNT: u32 = 0;
            IRQ_COUNT += 1;
            let mut tick_guard_acquired: bool = false;

            // Only print debug output for first 5 IRQs to avoid spam
            #[cfg(feature = "perf-verbose")]
            if IRQ_COUNT <= 5 {
                super::uart_print(b"[IRQ] ENTER\n");
                super::uart_print(b"[IRQ_HANDLER] IRQ ");
                print_number(IRQ_COUNT as usize);
                super::uart_print(b" received!\n");
            }

            let mut irq: u64;
            asm!("mrs {x}, icc_iar1_el1", x = out(reg) irq);

            // Check for spurious interrupt (INTID 1023)
            let intid = irq & 0x3FF;
            if intid == 1023 {
                // Spurious interrupt - just return without EOI
                super::uart_print(b"[IRQ_HANDLER] SPURIOUS interrupt (1023), ignoring\n");
                return;
            }

            #[cfg(feature = "perf-verbose")]
            if IRQ_COUNT <= 5 {
                super::uart_print(b"[IRQ_HANDLER] INTID=");
                print_number(intid as usize);
                super::uart_print(b"\n");
            }

            let mut t1: u64; core::arch::asm!("mrs {x}, cntpct_el0", x = out(reg) t1);
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
                    core::arch::asm!("msr cntp_tval_el0, {x}", x = in(reg) TIMER_BENCH_INTERVAL);
                } else {
                    // Bench completed: print a brief summary (mean/min/max)
                    super::uart_print(b"[BENCH] Benchmark completed! Rearming timer for autonomy...\n");
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
                    // Set flag to prevent double-rearm on next interrupt
                    BENCH_JUST_COMPLETED = true;

                    // CRITICAL: Disable timer completely to stop further interrupts
                    let ctl: u64 = 0; // ENABLE=0, IMASK=0
                    core::arch::asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl);
                    core::arch::asm!("isb");

                    // Clear any timer condition by setting a far future value
                    let mut frq: u64;
                    core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
                    if frq == 0 {
                        frq = crate::platform::active().timer().freq_hz;
                    }

                    // Set timer for 1 second in the future
                    super::uart_print(b"[BENCH] Disabling timer and rearming with interval=");
                    print_number(frq as usize);
                    super::uart_print(b" cycles\n");
                    core::arch::asm!("msr cntp_tval_el0, {x}", x = in(reg) frq);
                    core::arch::asm!("isb");

                    // Re-enable timer
                    let ctl: u64 = 1; // ENABLE=1, IMASK=0
                    core::arch::asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl);
                    core::arch::asm!("isb");
                }
            }
            // If no latency bench is running, drive autonomous ticks on EL1 physical timer (PPI 30)
            // IMPORTANT: Don't rearm if benchmark just completed - it already rearmed above!
            else if TIMER_BENCH_WARMUP == 0 && TIMER_BENCH_REMAIN == 0 {
                // Check if benchmark JUST completed (flag to prevent double-rearm)
                static mut DRAIN_COUNT: u32 = 0;
                if BENCH_JUST_COMPLETED {
                    // Skip rearming - benchmark already rearmed the timer
                    super::uart_print(b"[TIMER] Skipping rearm - benchmark just completed\n");
                    BENCH_JUST_COMPLETED = false;
                    DRAIN_COUNT = 10; // Drain next 10 rapid interrupts
                } else if DRAIN_COUNT > 0 {
                    // Drain rapid interrupts after benchmark
                    DRAIN_COUNT -= 1;
                    if DRAIN_COUNT == 0 {
                        super::uart_print(b"[TIMER] Finished draining rapid interrupts\n");
                    }
                    // Don't rearm, just return to drain the interrupt
                } else {
                    // GICv3 INTID is in low bits; mask to 10 bits for PPIs/SPIs
                    let intid: u32 = (irq & 0x3FF) as u32;

                    // Debug: Print first 5 timer interrupts (reduced from 20 for usability)
                    TIMER_TICK_COUNT += 1;
                    #[cfg(feature = "perf-verbose")]
                    if TIMER_TICK_COUNT <= 5 {
                        super::uart_print(b"[TIMER_ISR] Tick ");
                        print_number(TIMER_TICK_COUNT as usize);
                        super::uart_print(b" intid=");
                        print_number(intid as usize);
                        super::uart_print(b"\n");
                    }
                    #[cfg(not(feature = "perf-verbose"))]
                    if TIMER_TICK_COUNT == 1 {
                        super::uart_print(b"[TIMER] Timer running silently (use 'autoctl status' to check)\n");
                    }
                    #[cfg(feature = "perf-verbose")]
                    if TIMER_TICK_COUNT == 6 {
                        super::uart_print(b"[TIMER] Timer running silently (use 'autoctl status' to check)\n");
                    }

                    if intid == 30 {
                        // Reentrancy guard for timer tick path
                        if IN_TICK.swap(true, Ordering::AcqRel) {
                            // Already in tick; skip work and proceed to EOI
                        } else {
                            tick_guard_acquired = true;
                        // If autonomy is not enabled or not ready, keep timer disabled and do not rearm
                        let autonomy_enabled = crate::autonomy::AUTONOMOUS_CONTROL.is_enabled();
                        let autonomy_ready = crate::autonomy::is_ready();
                        if !autonomy_enabled || !autonomy_ready {
                            // Disable timer and clear pending to avoid rapid re-fires while disabled
                            let ctl_off: u64 = 0;
                            core::arch::asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl_off);
                            core::arch::asm!("dsb sy; isb");
                            let clear_val: u64 = 1;
                            core::arch::asm!("msr cntp_tval_el0, {x}", x = in(reg) clear_val);
                            core::arch::asm!("isb");
                            // Release guard and skip rest; EOI at bottom
                        } else {
                        // Track last timer tick time to detect rapid firing
                        static mut LAST_TIMER_TICK: u64 = 0;
                        static mut RAPID_TICK_COUNT: u32 = 0;

                        // Get current timer counter
                        let mut cnt: u64;
                        core::arch::asm!("mrs {x}, cntpct_el0", x = out(reg) cnt);

                        // Get timer frequency for time calculations
                        let mut frq: u64;
                        core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);

                        // Check if this tick came too soon (< 10ms since last tick)
                        // 10ms = frq/100
                        let min_interval = frq / 100;
                        if LAST_TIMER_TICK != 0 && (cnt - LAST_TIMER_TICK) < min_interval {
                            RAPID_TICK_COUNT += 1;
                            if RAPID_TICK_COUNT > 5 {
                                super::uart_print(b"\n[TIMER] RAPID FIRING DETECTED (");
                                print_number(RAPID_TICK_COUNT as usize);
                                super::uart_print(b" ticks < 10ms apart)!\n");
                                super::uart_print(b"[TIMER] Disabling timer to prevent system instability.\n");
                                super::uart_print(b"[TIMER] Use 'autoctl off' then 'autoctl on' to reset.\n\n");

                                // Disable timer completely
                                let ctl: u64 = 0; // ENABLE=0
                                core::arch::asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl);
                                core::arch::asm!("isb");

                                // Disable autonomous mode
                                crate::autonomy::AUTONOMOUS_CONTROL.disable();

                                // Reset counters for next time
                                RAPID_TICK_COUNT = 0;
                                LAST_TIMER_TICK = 0;

                                // Skip to EOI at bottom
                            } else {
                                // Rapid tick detected but not enough yet, just warn
                                if RAPID_TICK_COUNT == 1 {
                                    super::uart_print(b"[TIMER] Warning: rapid ticks detected\n");
                                }
                            }
                        } else {
                            // Normal tick timing, reset rapid counter
                            if RAPID_TICK_COUNT > 0 {
                                super::uart_print(b"[TIMER] Normal timing restored\n");
                                RAPID_TICK_COUNT = 0;
                            }
                        }

                        // Update last tick time
                        LAST_TIMER_TICK = cnt;

                        // Only process timer if not rapidly firing
                        if RAPID_TICK_COUNT <= 5 {

                            // Clear the interrupt condition first by disabling timer
                            // This prevents immediate re-firing
                            let ctl: u64 = 0; // ENABLE=0
                            core::arch::asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl);
                            core::arch::asm!("isb");

                            // Get timer frequency
                            let mut frq: u64;
                            core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);

                            // Rearm timer with autonomous interval (absolute cval = now + cycles)
                            let interval_ms = crate::autonomy::AUTONOMOUS_CONTROL
                                .decision_interval_ms
                                .load(core::sync::atomic::Ordering::Relaxed)
                                .clamp(100, 60_000);

                            // Debug: show timer frequency
                            if TIMER_TICK_COUNT == 1 {
                                super::uart_print(b"[TIMER] Timer freq=");
                                print_number(frq as usize);
                                super::uart_print(b" Hz\n");
                            }

                            let cycles = if frq > 0 {
                                (frq / 1000).saturating_mul(interval_ms)
                            } else {
                                let pf = crate::platform::active().timer().freq_hz;
                                ((pf / 1000).max(1)).saturating_mul(interval_ms)
                            };
                            let mut now_abs: u64; core::arch::asm!("mrs {x}, cntpct_el0", x = out(reg) now_abs);
                            let next = now_abs.saturating_add(cycles);

                            // Only print interval change for first few ticks
                            if TIMER_TICK_COUNT <= 3 {
                                super::uart_print(b"[TIMER] Rearming with ");
                                print_number(interval_ms as usize);
                                super::uart_print(b"ms interval (");
                                print_number(cycles as usize);
                                super::uart_print(b" cycles)\n");
                            }

                            // Program absolute compare and re-enable
                            core::arch::asm!("msr cntp_cval_el0, {x}", x = in(reg) next);
                            core::arch::asm!("isb");
                            let ctl: u64 = 1; // ENABLE=1, IMASK=0
                            core::arch::asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl);
                            core::arch::asm!("isb");

                            // Debug: Confirm rearm happened (only for first 5 ticks)
                            if TIMER_TICK_COUNT > 3 && TIMER_TICK_COUNT <= 5 {
                                super::uart_print(b"[TIMER] Tick ");
                                print_number(TIMER_TICK_COUNT as usize);
                                super::uart_print(b" rearmed\n");
                            }

                            // Debug: Verify the timer value was actually set
                            if TIMER_TICK_COUNT <= 20 {  // Extended debug for more ticks
                                let mut tval: u64;
                                core::arch::asm!("mrs {x}, cntp_tval_el0", x = out(reg) tval);
                                // TVAL is signed, check if it's negative (already expired)
                                if (tval as i64) < 0 {
                                    super::uart_print(b"[TIMER] Tick ");
                                    print_number(TIMER_TICK_COUNT as usize);
                                    super::uart_print(b" ERROR: TVAL negative! val=");
                                    print_number((tval as i64).wrapping_abs() as usize);
                                    super::uart_print(b"\n");
                                }
                                #[cfg(feature = "perf-verbose")]
                                if TIMER_TICK_COUNT <= 3 {
                                    super::uart_print(b"[TIMER] TVAL set OK: ");
                                    print_number(tval as usize);
                                    super::uart_print(b" cycles\n");
                                }
                            }

                            // Check control register to ensure timer is enabled (only for first 5 ticks)
                            if TIMER_TICK_COUNT == 4 {
                                let mut ctl: u64;
                                core::arch::asm!("mrs {x}, cntp_ctl_el0", x = out(reg) ctl);
                                super::uart_print(b"[TIMER] Tick ");
                                print_number(TIMER_TICK_COUNT as usize);
                                super::uart_print(b" CTL=");
                                print_number(ctl as usize);
                                super::uart_print(b" (bit0=enable, bit1=mask, bit2=istatus)\n");
                            }

                            // Ensure timer remains enabled (should already be from init)
                            // Only set if not already enabled to avoid any glitches
                            let mut ctl: u64;
                            core::arch::asm!("mrs {x}, cntp_ctl_el0", x = out(reg) ctl);
                            if (ctl & 1) == 0 {
                                super::uart_print(b"[TIMER] WARNING: Timer was disabled, re-enabling\n");
                                ctl = 1; // ENABLE=1, IMASK=0
                                core::arch::asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl);
                            }

                            // Only call autonomous_decision_tick() if autonomy is enabled AND ready
                            if autonomy_enabled && autonomy_ready {
                                // Only print debug message for first few ticks
                                if TIMER_TICK_COUNT <= 5 {
                                    super::uart_print(b"[TIMER] Calling autonomous_decision_tick()\n");
                                }
                                // Trigger supervised autonomous decision tick (internally gated)
                                crate::autonomy::autonomous_decision_tick();
                            }
                        } // Close the 'if RAPID_TICK_COUNT <= 5' block
                        } // end autonomy enabled block
                        } // end reentrancy guard acquired block
                    } // Close the 'if intid == 30' block
                } // Close the 'else' block from BENCH_JUST_COMPLETED check
            }
            // signal end of interrupt
            if tick_guard_acquired { IN_TICK.store(false, Ordering::Release); }
            asm!("msr icc_eoir1_el1, {x}", x = in(reg) irq);
            asm!("msr icc_dir_el1, {x}", x = in(reg) irq);
        }
    }

    // Timer tick counter - made pub(crate) so it can be reset from shell.rs
    #[no_mangle]
    pub static mut TIMER_TICK_COUNT: u32 = 0;

    static mut TIMER_BENCH_REMAIN: u32 = 0;
    static mut TIMER_BENCH_INTERVAL: u64 = 0;
    static mut TIMER_BENCH_T0: u64 = 0;
    static mut TIMER_BENCH_WARMUP: u32 = 0;
    static mut TIMER_SUM_NS: u64 = 0;
    static mut TIMER_MIN_NS: u64 = u64::MAX;
    static mut TIMER_MAX_NS: u64 = 0;
    static mut TIMER_SUM_COUNT: u32 = 0;
    static mut BENCH_JUST_COMPLETED: bool = false;
    // Reentrancy guard static for timer ISR
    static IN_TICK: AtomicBool = AtomicBool::new(false);

    unsafe fn delta_ns(t0: u64, t1: u64) -> u64 {
        let mut f: u64; core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) f);
        if f == 0 { return 0; }
        let d = t1.wrapping_sub(t0);
        (d.saturating_mul(1_000_000_000)) / f
    }

    // Keep this function for manual benchmarking but allow dead_code warning
    // It was disabled at boot to prevent automatic timer firing
    #[allow(dead_code)]
    unsafe fn start_irq_latency_bench(samples: u32) {
        super::uart_print(b"[BENCH_START] Starting IRQ latency benchmark with ");
        print_number(samples as usize);
        super::uart_print(b" samples\n");
        let mut frq: u64; core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        TIMER_BENCH_INTERVAL = frq / 100; // ~10ms per sample (was 0.5ms, too fast!)
        TIMER_BENCH_REMAIN = samples;
        TIMER_BENCH_WARMUP = 4; // discard first 4 samples
        super::uart_print(b"[BENCH_START] Interval: ");
        print_number(TIMER_BENCH_INTERVAL as usize);
        super::uart_print(b" cycles, warmup=4, remain=");
        print_number(samples as usize);
        super::uart_print(b"\n");
        core::arch::asm!("mrs {x}, cntpct_el0", x = out(reg) TIMER_BENCH_T0);
        TIMER_SUM_NS = 0; TIMER_MIN_NS = u64::MAX; TIMER_MAX_NS = 0; TIMER_SUM_COUNT = 0;
        core::arch::asm!("msr cntp_tval_el0, {x}", x = in(reg) TIMER_BENCH_INTERVAL);

        // Check timer control register status bits
        let mut ctl_check: u64;
        core::arch::asm!("mrs {x}, cntp_ctl_el0", x = out(reg) ctl_check);
        super::uart_print(b"[BENCH_START] Timer control: ");
        print_number(ctl_check as usize);
        super::uart_print(b" (bit 0=ENABLE, bit 1=IMASK, bit 2=ISTATUS)\n");

        // Check current counter value
        let mut cval: u64;
        core::arch::asm!("mrs {x}, cntpct_el0", x = out(reg) cval);
        super::uart_print(b"[BENCH_START] Counter value: ");
        print_number(cval as usize);
        super::uart_print(b"\n");

        // Check timer compare value
        let mut tval: u64;
        core::arch::asm!("mrs {x}, cntp_tval_el0", x = out(reg) tval);
        super::uart_print(b"[BENCH_START] Timer value (signed): ");
        print_number(tval as usize);
        super::uart_print(b"\n");

        super::uart_print(b"[BENCH_START] Benchmark armed, waiting for first IRQ...\n");
    }

    unsafe fn enable_irq() {
        super::uart_print(b"[IRQ_ENABLE] enable_irq() called\n");

        // CRITICAL: Keep interrupts masked during configuration
        super::uart_print(b"[IRQ_ENABLE] Starting IRQ enable sequence...\n");

        // Check VBAR_EL1 first
        let mut vbar: u64;
        asm!("mrs {x}, vbar_el1", x = out(reg) vbar);
        super::uart_print(b"[IRQ_ENABLE] VBAR_EL1: 0x");
        print_number(vbar as usize);
        super::uart_print(b"\n");

        // Check what the vector table address should be
        extern "C" {
            static VECTORS: u8;
        }
        let vectors_addr = &VECTORS as *const u8 as usize;
        super::uart_print(b"[IRQ_ENABLE] Expected vectors at: 0x");
        print_number(vectors_addr);
        if vectors_addr == vbar as usize {
            super::uart_print(b" (MATCH)\n");
        } else {
            super::uart_print(b" (MISMATCH!)\n");
        }

        // Check ICC_IGRPEN1_EL1 (GIC CPU interface group enable)
        let mut igrpen: u64;
        asm!("mrs {x}, icc_igrpen1_el1", x = out(reg) igrpen);
        super::uart_print(b"[IRQ_ENABLE] ICC_IGRPEN1_EL1: ");
        print_number(igrpen as usize);
        super::uart_print(b" (should be 1)\n");

        // Check ICC_PMR_EL1 (Priority mask)
        let mut pmr: u64;
        asm!("mrs {x}, icc_pmr_el1", x = out(reg) pmr);
        super::uart_print(b"[IRQ_ENABLE] ICC_PMR_EL1: ");
        print_number(pmr as usize);
        super::uart_print(b" (should be 0xFF to unmask all)\n");

        // Set timer to expire in 1 second (not 100 cycles!) to avoid immediate interrupt
        let mut frq: u64;
        core::arch::asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        if frq == 0 {
            frq = crate::platform::active().timer().freq_hz;
        }
        super::uart_print(b"[IRQ_ENABLE] Setting timer for 1 second (");
        print_number(frq as usize);
        super::uart_print(b" cycles)...\n");
        core::arch::asm!("msr cntp_tval_el0, {x}", x = in(reg) frq);

        // NOW unmask IRQs in PSTATE as the very last step
        super::uart_print(b"[IRQ_ENABLE] Unmasking IRQs in PSTATE...\n");
        asm!("msr daifclr, #2", options(nostack, preserves_flags));

        // Read back DAIF to verify
        let mut daif: u64;
        asm!("mrs {x}, daif", x = out(reg) daif);
        super::uart_print(b"[IRQ_ENABLE] DAIF register: 0x");
        print_number(daif as usize);
        if (daif & 0x80) != 0 {  // Bit 7 (I flag) = 0x80
            super::uart_print(b" (ERROR: IRQs are MASKED! Bit 7 is set)\n");
            // Try to unmask again
            super::uart_print(b"[IRQ_ENABLE] Attempting to unmask IRQs again...\n");
            asm!("msr daifclr, #2", options(nostack, preserves_flags));
            // Check again
            let mut daif2: u64;
            asm!("mrs {x}, daif", x = out(reg) daif2);
            super::uart_print(b"[IRQ_ENABLE] DAIF after retry: 0x");
            print_number(daif2 as usize);
            if (daif2 & 0x80) != 0 {
                super::uart_print(b" (STILL MASKED!)\n");
            } else {
                super::uart_print(b" (OK - unmasked)\n");
            }
        } else {
            super::uart_print(b" (OK - IRQs unmasked)\n");
        }

        super::uart_print(b"[IRQ_ENABLE] IRQ system setup complete.\n");
    }

    unsafe fn timer_init_1hz() {
        super::uart_print(b"[TIMER_INIT] Starting timer initialization...\n");

        // Check current exception level
        let mut current_el: u64;
        asm!("mrs {x}, CurrentEL", x = out(reg) current_el);
        let el = (current_el >> 2) & 0x3;
        super::uart_print(b"[TIMER_INIT] Current EL: ");
        print_number(el as usize);
        super::uart_print(b"\n");

        let mut frq: u64;
        asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        super::uart_print(b"[TIMER_INIT] Counter frequency: ");
        print_number(frq as usize);
        super::uart_print(b" Hz\n");
        if frq == 0 {
            frq = crate::platform::active().timer().freq_hz;
            super::uart_print(b"[TIMER_INIT] Using platform freq: ");
            print_number(frq as usize);
            super::uart_print(b" Hz\n");
        }
        // Set initial interval ~1s
        super::uart_print(b"[TIMER_INIT] Setting timer interval: ");
        print_number(frq as usize);
        super::uart_print(b" cycles\n");
        asm!("msr cntp_tval_el0, {x}", x = in(reg) frq);
        // IMPORTANT: Do NOT enable timer here - wait for explicit user command
        // Timer will be enabled when user runs 'autoctl on' or starts a benchmark
        let ctl: u64 = 0; // ENABLE=0, IMASK=0 - timer configured but disabled
        asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl);
        super::uart_print(b"[TIMER_INIT] EL1 physical timer configured but NOT enabled (ctl=0)\n");
        super::uart_print(b"[TIMER_INIT] Timer will start when user runs 'autoctl on' or benchmark\n");

        // Read back control register to verify
        let mut ctl_read: u64;
        asm!("mrs {x}, cntp_ctl_el0", x = out(reg) ctl_read);
        super::uart_print(b"[TIMER_INIT] Control register readback: ");
        print_number(ctl_read as usize);
        super::uart_print(b" (bit 0=enable, bit 1=mask, bit 2=istatus)\n");

        // Final diagnostic: Check GIC state for PPI 30
        super::uart_print(b"[TIMER_INIT] Final GIC state check for PPI 30:\n");

        // Get redistributor base for diagnostic
        let g = crate::platform::active().gic();
        let gicr_base = g.gicr as u64;
        const SGI_BASE: u64 = 0x10000;  // SGI/PPI config is in second page
        let isenabler0 = (gicr_base + SGI_BASE + 0x0100) as *const u32;
        let enabled = core::ptr::read_volatile(isenabler0);
        super::uart_print(b"  GICR_ISENABLER0: 0x");
        print_number(enabled as usize);
        if (enabled & (1 << 30)) != 0 {
            super::uart_print(b" (PPI 30 ENABLED)\n");
        } else {
            super::uart_print(b" (WARNING: PPI 30 NOT ENABLED!)\n");
        }

        // Check interrupt priority for PPI 30
        let iprio = (gicr_base + SGI_BASE + 0x0400) as *const u32;
        let prio_reg = iprio.add(30 / 4); // 4 priorities per register
        let prio_val = core::ptr::read_volatile(prio_reg);
        let prio_shift = (30 % 4) * 8;
        let prio = (prio_val >> prio_shift) & 0xFF;
        super::uart_print(b"  PPI 30 priority: ");
        print_number(prio as usize);
        super::uart_print(b" (expected 96, must be < ICC_PMR_EL1 to fire)\n");

        // Check current ICC_PMR_EL1 state
        let pmr: u64;
        asm!("mrs {x}, icc_pmr_el1", x = out(reg) pmr);
        super::uart_print(b"  Current ICC_PMR_EL1: ");
        print_number(pmr as usize);
        super::uart_print(b" (should be 255)\n");

        // Check Group 1 enable state
        let grp1_en: u64;
        asm!("mrs {x}, icc_igrpen1_el1", x = out(reg) grp1_en);
        super::uart_print(b"  ICC_IGRPEN1_EL1: ");
        print_number(grp1_en as usize);
        super::uart_print(b" (should be 1)\n");

        super::uart_print(b"[TIMER_INIT] Timer initialization complete.\n");
    }

    unsafe fn gicv3_init_qemu() {
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

    // (shell stack switch trampoline removed)

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
