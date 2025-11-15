/// SMP (Symmetric Multi-Processing) support - Phase E
///
/// Manages multiple CPU cores with per-CPU data, runqueues, and load balancing.

pub mod percpu;
pub mod ipi;

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use spin::Mutex;

/// Maximum number of CPUs supported
pub const MAX_CPUS: usize = 8;

/// Stack size per CPU (64KB)
const CPU_STACK_SIZE: usize = 65536;

/// Secondary CPU stacks (64KB each, 8 CPUs = 512KB total)
#[repr(C, align(16))]
struct CpuStack([u8; CPU_STACK_SIZE]);

static SECONDARY_STACKS: [CpuStack; MAX_CPUS] = [
    CpuStack([0; CPU_STACK_SIZE]),
    CpuStack([0; CPU_STACK_SIZE]),
    CpuStack([0; CPU_STACK_SIZE]),
    CpuStack([0; CPU_STACK_SIZE]),
    CpuStack([0; CPU_STACK_SIZE]),
    CpuStack([0; CPU_STACK_SIZE]),
    CpuStack([0; CPU_STACK_SIZE]),
    CpuStack([0; CPU_STACK_SIZE]),
];

/// Get secondary stack base address
fn get_secondary_stack_base() -> usize {
    SECONDARY_STACKS.as_ptr() as usize
}

/// Number of CPUs detected and online
static CPU_COUNT: AtomicUsize = AtomicUsize::new(1); // Boot CPU = 1

/// CPU online bitmap (bit N = CPU N is online)
static CPU_ONLINE: [AtomicBool; MAX_CPUS] = [
    AtomicBool::new(false), AtomicBool::new(false), AtomicBool::new(false), AtomicBool::new(false),
    AtomicBool::new(false), AtomicBool::new(false), AtomicBool::new(false), AtomicBool::new(false),
];

/// SMP initialization lock (ensures only boot CPU initializes)
static SMP_INIT_LOCK: Mutex<()> = Mutex::new(());

/// Get the number of online CPUs
pub fn num_cpus() -> usize {
    CPU_COUNT.load(Ordering::Acquire)
}

/// Check if a CPU is online
pub fn is_cpu_online(cpu_id: usize) -> bool {
    if cpu_id >= MAX_CPUS {
        return false;
    }
    CPU_ONLINE[cpu_id].load(Ordering::Acquire)
}

/// Mark a CPU as online
pub fn mark_cpu_online(cpu_id: usize) {
    if cpu_id >= MAX_CPUS {
        crate::warn!("SMP: CPU ID {} exceeds MAX_CPUS", cpu_id);
        return;
    }

    if !CPU_ONLINE[cpu_id].swap(true, Ordering::Release) {
        // Was not online before
        CPU_COUNT.fetch_add(1, Ordering::Release);
        crate::info!("SMP: CPU {} is now online", cpu_id);
    }
}

/// Mark a CPU as offline
pub fn mark_cpu_offline(cpu_id: usize) {
    if cpu_id >= MAX_CPUS {
        return;
    }

    if CPU_ONLINE[cpu_id].swap(false, Ordering::Release) {
        // Was online before
        CPU_COUNT.fetch_sub(1, Ordering::Release);
        crate::info!("SMP: CPU {} is now offline", cpu_id);
    }
}

/// Secondary CPU entry point (called by PSCI after CPU_ON)
///
/// This is the Rust entry point for secondary CPUs after they've been brought up.
#[no_mangle]
pub extern "C" fn secondary_cpu_entry(cpu_id: usize) -> ! {
    crate::info!("SMP: CPU {} starting...", cpu_id);

    // Mark this CPU as online
    mark_cpu_online(cpu_id);

    // Initialize per-CPU data
    percpu::init_percpu(cpu_id);

    // TODO: Initialize per-CPU timer
    // TODO: Initialize per-CPU GIC redistributor
    // TODO: Enable interrupts

    crate::info!("SMP: CPU {} initialized and ready", cpu_id);

    // Enter idle loop (will be replaced with scheduler)
    loop {
        unsafe {
            // Wait for interrupt
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}

/// Secondary CPU boot trampoline (assembly)
///
/// This is the physical entry point for secondary CPUs set by PSCI.
/// It sets up minimal state and jumps to Rust code.
#[no_mangle]
pub unsafe extern "C" fn secondary_cpu_boot(cpu_id: usize) -> ! {
    // Calculate stack top for this CPU
    // Stack grows down, so we want: base + (cpu_id + 1) * STACK_SIZE
    let stack_base = get_secondary_stack_base();
    let stack_top = stack_base + ((cpu_id + 1) * CPU_STACK_SIZE);

    // Set stack pointer using inline assembly
    core::arch::asm!(
        "mov sp, {stack_top}",
        "mov x0, {cpu_id}",
        "b {entry}",
        stack_top = in(reg) stack_top,
        cpu_id = in(reg) cpu_id,
        entry = sym secondary_cpu_entry,
        options(noreturn)
    );
}

/// Initialize SMP system (called by boot CPU)
pub fn init() {
    let _lock = SMP_INIT_LOCK.lock();

    crate::info!("SMP: Initializing multi-core support");

    // Mark boot CPU (CPU 0) as online
    mark_cpu_online(0);

    // Initialize per-CPU data for boot CPU
    percpu::init_percpu(0);

    // If PSCI is not available on this platform/firmware, stay single-core
    if !crate::platform::active().psci_available() {
        crate::warn!("SMP: PSCI not available; running in single-core mode");
        return;
    }

    // Detect number of CPUs from device tree or hardcode for QEMU
    // For now, we'll try to bring up CPUs 1-3 (total 4 CPUs)
    let target_cpus = 4;

    crate::info!("SMP: Attempting to bring up {} CPUs", target_cpus);

    // Get PSCI version
    let psci_version = crate::arch::psci_version();
    let major = (psci_version >> 16) & 0xFFFF;
    let minor = psci_version & 0xFFFF;
    crate::info!("SMP: PSCI version {}.{}", major, minor);

    // Bring up secondary CPUs
    for cpu_id in 1..target_cpus {
        if cpu_id >= MAX_CPUS {
            break;
        }

        crate::info!("SMP: Bringing up CPU {}...", cpu_id);

        let entry_point = secondary_cpu_boot as *const () as u64;
        let context_id = cpu_id as u64;

        match crate::arch::cpu_on(cpu_id as u64, entry_point, context_id) {
            Ok(()) => {
                crate::info!("SMP: CPU {} boot initiated", cpu_id);

                // Give CPU time to boot
                for _ in 0..1000000 {
                    core::hint::spin_loop();
                }
            }
            Err(e) => {
                crate::warn!("SMP: Failed to bring up CPU {}: {:?}", cpu_id, e);
            }
        }
    }

    let online_cpus = num_cpus();
    crate::info!("SMP: Initialization complete, {} CPUs online", online_cpus);
}

/// Get statistics about the SMP system
pub fn stats() -> SmpStats {
    let mut online_cpu_ids = [false; MAX_CPUS];
    for i in 0..MAX_CPUS {
        online_cpu_ids[i] = is_cpu_online(i);
    }

    SmpStats {
        num_cpus: num_cpus(),
        online_cpu_ids,
    }
}

/// SMP statistics
pub struct SmpStats {
    pub num_cpus: usize,
    pub online_cpu_ids: [bool; MAX_CPUS],
}
