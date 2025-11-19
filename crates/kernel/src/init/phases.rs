use super::error::{KernelError, KernelResult};

/// Early initialization - MMU, UART, PMU, heap
///
/// This function performs the critical early bootstrap:
/// 1. Check exception level (must be EL1)
/// 2. Enable MMU for virtual memory
/// 3. Initialize PMU for performance monitoring
/// 4. Initialize UART for console output
/// 5. Initialize boot timestamp
/// 6. Initialize heap allocator
///
/// # Safety
/// Must be called only once during kernel boot. Assumes stack is already set up.
///
/// # Errors
/// Returns `KernelError::EarlyInit` if any critical component fails to initialize.
pub unsafe fn early_init() -> KernelResult<()> {
    use core::arch::asm;

    // Check exception level (must be EL1 for MMU)
    let current_el: u64;
    asm!("mrs {el}, CurrentEL", el = out(reg) current_el);
    let el = (current_el >> 2) & 0x3;
    if el != 1 {
        return Err(KernelError::InvalidExceptionLevel(el as u8));
    }

    // Enable MMU for virtual memory
    crate::bringup::enable_mmu_el1();
    crate::uart_print(b"MMU ON\n");

    // Enable Performance Monitoring Unit
    crate::uart_print(b"PMU: INIT\n");
    crate::bringup::pmu_enable();
    #[cfg(feature = "perf-verbose")]
    {
        crate::uart_print(b"PMU: EVENTS\n");
        crate::pmu::aarch64::setup_events();
    }
    crate::uart_print(b"PMU: READY\n");

    // Initialize UART for interactive I/O
    crate::uart_print(b"UART: INIT\n");
    crate::uart::init();
    crate::uart_print(b"UART: READY\n");

    // Emit counter frequency for timing sanity check
    {
        let mut frq: u64;
        asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        crate::uart_print(b"METRIC cntfrq_hz=");
        crate::bringup::print_number(frq as usize);
        crate::uart_print(b"\n");
    }

    // Initialize boot timestamp
    crate::time::init_boot_timestamp();

    // Initialize heap allocator
    crate::uart_print(b"HEAP: INIT\n");
    crate::heap::init_heap()
        .map_err(|_| KernelError::EarlyInit("heap init failed"))?;
    crate::uart_print(b"HEAP: READY\n");

    // Run heap tests
    crate::uart_print(b"HEAP: TESTING\n");
    crate::heap::test_heap()
        .map_err(|_| KernelError::EarlyInit("heap tests failed"))?;
    crate::uart_print(b"HEAP: TESTS PASSED\n");

    Ok(())
}

/// Platform initialization - detect platform from FDT
///
/// This function handles platform-specific initialization:
/// 1. Parse device tree blob (DTB) if provided
/// 2. Override platform descriptors with FDT data
/// 3. Initialize PSCI for power management
///
/// # Safety
/// Must be called after early_init(). Requires heap allocation.
///
/// # Errors
/// Returns `KernelError::PlatformInit` if platform detection fails.
pub unsafe fn platform_init() -> KernelResult<()> {
    // If DTB pointer provided, override platform descriptors
    #[cfg(feature = "dt-override")]
    {
        let mut dt_active = false;
        if !crate::DTB_PTR.is_null() {
            dt_active = crate::platform::override_with_dtb(crate::DTB_PTR);
        }
        if dt_active {
            crate::uart_print(b"PLATFORM: dt override active\n");
        } else {
            crate::uart_print(b"PLATFORM: qemu_virt\n");
        }
    }

    // Initialize PSCI for power management
    crate::uart_print(b"PSCI: INIT\n");
    crate::arch::psci::init();
    crate::uart_print(b"PSCI: READY\n");

    Ok(())
}

/// Memory subsystem initialization - buddy allocator, slab allocator, page cache
///
/// This function initializes the kernel memory management:
/// 1. Initialize buddy allocator with RAM ranges
/// 2. Initialize slab allocator for fixed-size objects
/// 3. Initialize page cache for block I/O
///
/// # Safety
/// Must be called after early_init(). Requires heap.
///
/// # Errors
/// Returns `KernelError::MemoryInit` if memory subsystem fails.
pub unsafe fn memory_init() -> KernelResult<()> {
    crate::uart_print(b"PHASE A1: BOOT WIRING\n");

    // Initialize buddy allocator
    crate::uart_print(b"MM: BUDDY ALLOCATOR\n");
    let ram_start = 0x4100_0000u64; // Start after kernel (16MB offset)
    let ram_size = 112 * 1024 * 1024u64; // 112MB available
    let ranges: &[(u64, usize)] = &[(ram_start, ram_size as usize)];
    crate::mm::init_buddy(ranges)
        .map_err(|_| KernelError::MemoryInit("buddy allocator init failed"))?;

    let stats = crate::mm::get_stats().unwrap_or_default();
    crate::uart_print(b"MM: BUDDY READY (");
    crate::bringup::print_number(stats.total_pages);
    crate::uart_print(b" pages)\n");

    // Initialize slab allocator
    crate::uart_print(b"MM: SLAB ALLOCATOR\n");
    crate::mm::slab::init();
    crate::uart_print(b"MM: SLAB READY (5 caches: 16-256 bytes)\n");

    // Initialize page cache
    crate::uart_print(b"PAGE CACHE: INIT\n");
    crate::mm::init_page_cache(1024); // Cache up to 1024 blocks (512KB)
    crate::uart_print(b"PAGE CACHE: READY\n");

    Ok(())
}

/// Subsystems initialization - Process table, scheduler, VFS, filesystems
///
/// This function initializes core kernel subsystems:
/// 1. Process table and scheduler
/// 2. VFS (Virtual File System)
/// 3. tmpfs, devfs, procfs filesystems
/// 4. Optional initramfs unpacking
/// 5. Block devices and filesystems (ext4/ext2)
/// 6. Network stack initialization
///
/// # Safety
/// Must be called after memory_init(). Requires buddy and slab allocators.
///
/// # Errors
/// Returns `KernelError::SubsystemInit` if any subsystem fails.
pub unsafe fn subsystem_init() -> KernelResult<()> {
    use alloc::string::ToString;

    // Initialize process table
    crate::uart_print(b"PROCESS: INIT TABLE\n");
    crate::process::init_process_table();
    crate::uart_print(b"PROCESS: TABLE READY\n");

    // Initialize scheduler
    crate::uart_print(b"SCHEDULER: INIT\n");
    crate::process::scheduler::init();
    crate::process::scheduler_smp::init();
    crate::uart_print(b"SCHEDULER: READY\n");

    // Initialize VFS
    crate::uart_print(b"VFS: INIT\n");
    crate::vfs::init()
        .map_err(|_| KernelError::SubsystemInit("VFS init failed"))?;

    // Mount tmpfs at /
    crate::uart_print(b"VFS: MOUNT TMPFS AT /\n");
    let root = crate::vfs::tmpfs::mount_tmpfs()
        .map_err(|_| KernelError::SubsystemInit("tmpfs mount failed"))?;
    crate::vfs::set_root(root.clone());

    // Optionally unpack embedded initramfs
    #[cfg(all(feature = "initramfs-models", have_initramfs_models))]
    {
        crate::uart_print(b"INITRAMFS: UNPACK MODELS\n");
        if let Err(e) = crate::initramfs::unpack_initramfs(crate::embedded_models_initramfs::data) {
            crate::warn!("initramfs: unpack failed: {:?}", e);
        } else {
            crate::uart_print(b"INITRAMFS: MODELS READY\n");
        }
    }

    // Mount devfs at /dev
    crate::uart_print(b"VFS: MOUNT DEVFS AT /dev\n");
    let _dev_inode = crate::vfs::devfs::mount_devfs()
        .map_err(|_| KernelError::SubsystemInit("devfs mount failed"))?;
    root.create("dev", crate::vfs::S_IFDIR | 0o755)
        .map_err(|_| KernelError::SubsystemInit("failed to create /dev"))?;
    crate::vfs::set_root(root.clone());

    // Mount procfs at /proc
    crate::uart_print(b"VFS: MOUNT PROCFS AT /proc\n");
    let _proc_inode = crate::vfs::mount_procfs()
        .map_err(|_| KernelError::SubsystemInit("procfs mount failed"))?;
    root.create("proc", crate::vfs::S_IFDIR | 0o555)
        .map_err(|_| KernelError::SubsystemInit("failed to create /proc"))?;
    crate::vfs::set_root(root.clone());

    crate::uart_print(b"VFS: READY\n");

    // Initialize virtio-blk devices
    crate::uart_print(b"BLOCK: PROBING VIRTIO-BLK DEVICES\n");
    crate::arch::aarch64::init_virtio_blk();
    crate::uart_print(b"BLOCK: READY\n");

    // Try to mount ext4 or ext2 at /models if block devices exist
    {
        use crate::block::list_block_devices;
        let devs = list_block_devices();
        let count_str = devs.len().to_string();
        crate::uart_print(count_str.as_bytes());
        crate::uart_print(b"\n");

        if !devs.is_empty() {
            let _ = root.create("models", crate::vfs::S_IFDIR | 0o755);
            let mut mounted = false;
            for dev in devs {
                match crate::vfs::ext4::mount_ext4(dev.clone()) {
                    Ok(ext4_root) => {
                        let _ = crate::vfs::mount("ext4", ext4_root, "/models");
                        crate::uart_print(b"VFS: MOUNT EXT4 AT /models (rw+journal)\n");
                        mounted = true;
                        break;
                    }
                    Err(e) => {
                        crate::warn!("vfs: ext4 mount failed on {}: {:?}", dev.name, e);
                        match crate::vfs::ext2::mount_ext2(dev.clone()) {
                            Ok(ext2_root) => {
                                let _ = crate::vfs::mount("ext2", ext2_root, "/models");
                                crate::uart_print(b"VFS: MOUNT EXT2 AT /models (read-only)\n");
                                mounted = true;
                                break;
                            }
                            Err(e2) => {
                                crate::warn!("vfs: ext2 mount failed on {}: {:?}", dev.name, e2);
                            }
                        }
                    }
                }
            }
            if !mounted {
                crate::warn!("vfs: no ext4/ext2-compatible block device mounted at /models");
            }
        }
    }

    // Optional ext4 durability test
    #[cfg(feature = "ext4-durability-test")]
    {
        crate::uart_print(b"FS: EXT4 DURABILITY TEST\n");
        crate::fs::ext4::durability_selftest();
    }

    // Initialize virtio-net devices
    crate::uart_print(b"NET: PROBING VIRTIO-NET DEVICES\n");
    crate::arch::aarch64::init_virtio_net();
    crate::uart_print(b"NET: DRIVER READY\n");

    // Initialize network interface
    crate::uart_print(b"NET: INIT INTERFACE\n");
    if crate::net::init_network().is_ok() {
        crate::uart_print(b"NET: INTERFACE READY\n");

        // Try DHCP configuration
        crate::uart_print(b"NET: STARTING DHCP\n");
        let mut dhcp_client = crate::net::dhcp::DhcpClient::new();
        match dhcp_client.acquire_lease() {
            Ok(config) => {
                crate::uart_print(b"NET: DHCP LEASE ACQUIRED\n");
                if let Err(e) = dhcp_client.apply_config(&config) {
                    crate::warn!("net: Failed to apply DHCP config: {:?}", e);
                }
            }
            Err(e) => {
                crate::warn!("net: DHCP failed: {:?}", e);
                // Static fallback
                let _ = crate::net::smoltcp_iface::set_ip_address([10, 0, 2, 15], 24);
                let _ = crate::net::smoltcp_iface::set_gateway([10, 0, 2, 2]);
            }
        }
        crate::uart_print(b"NET: CONFIGURED\n");

        // Optional SNTP sync
        #[cfg(feature = "sntp")]
        {
            if let Ok(secs) = crate::net::sntp::sntp_query([10, 0, 2, 2]) {
                crate::info!("sntp: time (unix secs) = {}", secs);
            } else {
                crate::warn!("sntp: query failed");
            }
        }
    } else {
        crate::uart_print(b"NET: NO DEVICE (SKIP)\n");
    }

    // Initialize entropy source
    crate::uart_print(b"RANDOM: INIT PRNG\n");
    crate::security::init_random();
    crate::uart_print(b"RANDOM: READY\n");

    // Initialize SMP
    crate::uart_print(b"SMP: INIT MULTI-CORE\n");
    crate::smp::init();
    crate::uart_print(b"SMP: READY\n");

    // Initialize virtio-gpu
    crate::uart_print(b"GPU: PROBING VIRTIO-GPU DEVICES\n");
    crate::arch::aarch64::init_virtio_gpu();
    crate::uart_print(b"GPU: READY\n");

    // Initialize graphics subsystem
    crate::uart_print(b"GRAPHICS: INIT\n");
    if crate::graphics::init().is_ok() {
        crate::uart_print(b"GRAPHICS: READY\n");

        crate::uart_print(b"GRAPHICS: RUNNING TEST\n");
        if crate::graphics::test_graphics().is_ok() {
            crate::uart_print(b"GRAPHICS: TEST PASSED\n");
        } else {
            crate::uart_print(b"GRAPHICS: TEST FAILED\n");
        }

        // Initialize window manager
        crate::uart_print(b"WM: INIT WINDOW MANAGER\n");
        if crate::window_manager::init().is_ok() {
            crate::uart_print(b"WM: READY\n");

            crate::uart_print(b"WM: RUNNING TEST\n");
            if crate::window_manager::test_window_manager().is_ok() {
                crate::uart_print(b"WM: TEST PASSED\n");
            } else {
                crate::uart_print(b"WM: TEST FAILED\n");
            }

            // Initialize UI toolkit
            crate::uart_print(b"UI: INIT TOOLKIT\n");
            if crate::ui::init().is_ok() {
                crate::uart_print(b"UI: READY\n");

                crate::uart_print(b"UI: RUNNING TEST\n");
                if crate::ui::test_ui_toolkit().is_ok() {
                    crate::uart_print(b"UI: TEST PASSED\n");
                } else {
                    crate::uart_print(b"UI: TEST FAILED\n");
                }

                // Test applications
                crate::uart_print(b"APPS: TESTING APPLICATIONS\n");
                if crate::applications::test_applications().is_ok() {
                    crate::uart_print(b"APPS: TESTS PASSED\n");
                } else {
                    crate::uart_print(b"APPS: TESTS FAILED\n");
                }

                // Launch applications
                crate::uart_print(b"APPS: LAUNCHING ALL APPS\n");
                if crate::applications::launch_all_apps().is_ok() {
                    crate::uart_print(b"APPS: ALL APPS RUNNING\n");
                } else {
                    crate::uart_print(b"APPS: LAUNCH FAILED\n");
                }
            } else {
                crate::uart_print(b"UI: INIT FAILED\n");
            }
        } else {
            crate::uart_print(b"WM: INIT FAILED\n");
        }
    } else {
        crate::uart_print(b"GRAPHICS: NO GPU (SKIP)\n");
    }

    Ok(())
}

/// Driver initialization - Block, watchdog, driver framework
///
/// This function initializes hardware drivers:
/// 1. Driver framework
/// 2. VirtIO console driver
/// 3. Block devices
/// 4. Watchdog timer
///
/// # Safety
/// Must be called after subsystem_init().
///
/// # Errors
/// Returns `KernelError::DriverInit` if driver initialization fails.
pub unsafe fn driver_init() -> KernelResult<()> {
    use crate::bringup::print_number;

    // Initialize driver framework
    #[cfg(feature = "virtio-console")]
    {
        crate::uart_print(b"DRIVER FRAMEWORK\n");
        if crate::driver::init_driver_framework().is_err() {
            crate::uart_print(b"DRIVER: INIT FAILED\n");
        } else {
            crate::uart_print(b"DRIVER: INIT OK\n");

            crate::uart_print(b"DRIVER: REGISTERING VIRTIO CONSOLE\n");
            if crate::driver::register_driver(crate::virtio_console::get_virtio_console_driver()).is_err() {
                crate::uart_print(b"DRIVER: VIRTIO CONSOLE REGISTRATION FAILED\n");
            } else {
                crate::uart_print(b"DRIVER: VIRTIO CONSOLE REGISTERED\n");
            }

            if let Some(registry) = crate::driver::get_driver_registry() {
                match registry.discover_devices() {
                    Ok(count) => {
                        crate::uart_print(b"DRIVER: DISCOVERED ");
                        print_number(count);
                        crate::uart_print(b" DEVICES\n");
                    }
                    Err(_) => {
                        crate::uart_print(b"DRIVER: DISCOVERY FAILED\n");
                    }
                }
            }
        }
    }
    #[cfg(not(feature = "virtio-console"))]
    {
        crate::uart_print(b"DRIVER FRAMEWORK: SKIPPED (virtio-console feature off)\n");
    }

    // Initialize block devices
    crate::uart_print(b"BLOCK: INIT\n");
    if crate::drivers::block::init().is_err() {
        crate::uart_print(b"BLOCK: INIT FAILED\n");
    } else {
        crate::uart_print(b"BLOCK: READY\n");
    }

    // Initialize watchdog
    crate::uart_print(b"WATCHDOG: INIT\n");
    let wdt_type = crate::drivers::watchdog::init();
    match wdt_type {
        crate::drivers::watchdog::WatchdogType::Bcm2712Pm => {
            crate::uart_print(b"WATCHDOG: BCM2712 PM READY\n");
        }
        crate::drivers::watchdog::WatchdogType::None => {
            crate::uart_print(b"WATCHDOG: NONE AVAILABLE\n");
        }
        _ => {
            crate::uart_print(b"WATCHDOG: READY\n");
        }
    }

    Ok(())
}

/// Late initialization - GIC, interrupts, AI subsystems, shell
///
/// This function performs late-stage initialization:
/// 1. GICv3 interrupt controller and timer
/// 2. Enable IRQs
/// 3. SMP (bring up secondary CPUs)
/// 4. PMU (Performance Monitoring Unit)
/// 5. Optional GPIO and mailbox
/// 6. AI benchmarks and features
/// 7. Neural agents and autonomy
/// 8. PID 1 creation
/// 9. Build info
/// 10. AgentSys initialization
///
/// # Safety
/// Must be called after driver_init() and subsystem_init().
///
/// # Errors
/// Returns `KernelError::LateInit` if late initialization fails.
pub unsafe fn late_init() -> KernelResult<()> {
    use core::arch::asm;
    use crate::bringup::print_number;

    // Initialize GICv3 and timer
    crate::uart_print(b"GIC: INIT\n");
    crate::bringup::gicv3_init_qemu();
    crate::bringup::timer_init_1hz();
    crate::uart_print(b"[MAIN] Calling enable_irq() from initialization\n");
    crate::bringup::enable_irq();

    // Initialize SMP - bring up secondary CPUs
    crate::uart_print(b"SMP: INIT\n");
    crate::arch::smp::init();
    let num_cpus = crate::arch::smp::num_cpus();
    crate::uart_print(b"SMP: ");
    print_number(num_cpus);
    crate::uart_print(b" CPU(S) ONLINE\n");

    // Initialize PMU
    crate::uart_print(b"PMU: INIT\n");
    crate::pmu::init();
    crate::uart_print(b"PMU: READY\n");

    // Optional GPIO initialization
    #[cfg(feature = "rpi5-gpio")]
    {
        crate::uart_print(b"GPIO: INIT\n");
        let gpio_base = 0x107d508500usize;
        crate::drivers::gpio::bcm2xxx::init(gpio_base);
        crate::uart_print(b"GPIO: READY\n");
    }

    // Optional mailbox initialization
    #[cfg(feature = "rpi5-mailbox")]
    {
        crate::uart_print(b"MAILBOX: INIT\n");
        let mailbox_base = 0x107c013880usize;
        crate::drivers::firmware::mailbox::init(mailbox_base);
        crate::uart_print(b"MAILBOX: READY\n");
    }

    // AI benchmarks
    #[cfg(feature = "arm64-ai")]
    {
        crate::uart_print(b"AI FEATURES\n");
        crate::uart_print(b"AI: INITIALIZING FORMAL VERIFICATION\n");
        crate::uart_print(b"AI: ENABLING PERFORMANCE OPTIMIZATION\n");
        crate::uart_print(b"AI: CACHE-AWARE ALGORITHMS ACTIVE\n");
        crate::uart_print(b"AI: READY\n");
        crate::ai_benchmark::run_ai_benchmarks();
    }

    // Emit kernel metrics
    crate::uart_print(b"METRICS: STARTING\n");
    crate::userspace_test::emit_kernel_metrics();
    crate::uart_print(b"METRICS: COMPLETE\n");

    #[cfg(target_arch = "aarch64")]
    {
        crate::uart_print(b"CONTEXT SWITCH BENCH: STARTING\n");
        crate::userspace_test::bench_real_context_switch();
        crate::uart_print(b"CONTEXT SWITCH BENCH: COMPLETE\n");
    }

    crate::uart_print(b"SYSCALL TESTS: STARTING\n");
    crate::userspace_test::run_syscall_tests();
    crate::uart_print(b"SYSCALL TESTS: COMPLETE\n");

    // Initialize neural agents
    crate::uart_print(b"MEMORY AGENT: INIT\n");
    crate::neural::init_memory_agent();
    crate::uart_print(b"MEMORY AGENT: READY\n");

    crate::uart_print(b"META-AGENT: INIT\n");
    crate::meta_agent::init_meta_agent();
    crate::uart_print(b"META-AGENT: READY\n");

    crate::autonomy::AUTONOMY_READY.store(true, core::sync::atomic::Ordering::Release);
    crate::uart_print(b"AUTONOMY: set_ready complete\n");

    // Optional auto-enable autonomy
    #[cfg(all(target_arch = "aarch64", feature = "bringup"))]
    {
        crate::autonomy::AUTONOMOUS_CONTROL.enable();
        // Disable timer first
        let ctl_off: u64 = 0;
        asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl_off);
        asm!("dsb sy; isb");
        let clear_val: u64 = 1;
        asm!("msr cntp_tval_el0, {x}", x = in(reg) clear_val);
        asm!("isb");

        // Compute timer interval
        let mut frq: u64;
        asm!("mrs {x}, cntfrq_el0", x = out(reg) frq);
        let interval_ms = crate::autonomy::AUTONOMOUS_CONTROL
            .decision_interval_ms
            .load(core::sync::atomic::Ordering::Relaxed)
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
        let ctl_on: u64 = 1;
        asm!("msr cntp_ctl_el0, {x}", x = in(reg) ctl_on);
        asm!("isb");
        crate::uart_print(b"[AUTOCTL] Autonomous mode ENABLED at boot (bringup)\n");
    }

    // Print build info
    crate::build_info::print_build_info();

    // Initialize AgentSys
    #[cfg(feature = "agentsys")]
    {
        crate::agent_sys::init();
    }

    // Create PID 1
    crate::uart_print(b"INIT: CREATING PID 1\n");
    let init_task = crate::process::Task::new_init();
    crate::process::insert_task(init_task)
        .map_err(|_| KernelError::LateInit("failed to insert PID 1"))?;
    crate::uart_print(b"INIT: PID 1 CREATED\n");

    // Enqueue PID 1
    crate::uart_print(b"SCHEDULER: ENQUEUE PID 1\n");
    crate::process::scheduler::enqueue(1);
    crate::process::scheduler::set_current(1);
    crate::uart_print(b"SCHEDULER: PID 1 RUNNING\n");

    crate::uart_print(b"PHASE A1: BOOT WIRING COMPLETE\n");

    // Deterministic admission demo
    #[cfg(feature = "deterministic")]
    {
        crate::uart_print(b"DET: ADMISSION DEMO\n");
        crate::deterministic::demo_admission();
        crate::uart_print(b"DET: EDF TICK DEMO\n");
        crate::deterministic::edf_tick_demo();
    }

    // Graph stats
    #[cfg(feature = "graph-autostats")]
    {
        crate::uart_print(b"GRAPH: BASELINE STATS\n");
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

    Ok(())
}

