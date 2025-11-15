//! SMP (Symmetric Multi-Processing) support for AArch64
//!
//! This module provides multi-core CPU support for ARM64 platforms, specifically
//! targeting the Raspberry Pi 5 with its 4-core Cortex-A76 CPU configuration.
//!
//! # Architecture
//!
//! ```
//! CPU 0 (Boot)          CPU 1-3 (Secondary)
//!     │                       │
//!     ├─ Boot sequence        ├─ Parked (waiting)
//!     ├─ Init GIC Dist        │
//!     ├─ Init GIC Redist      │
//!     ├─ Init Timer           │
//!     ├─ PSCI init            │
//!     │                       │
//!     ├─ PSCI CPU_ON ────────>├─ secondary_entry()
//!     │                       ├─ Init GIC Redist (per-CPU)
//!     │                       ├─ Init Timer (per-CPU)
//!     │                       ├─ Enable IRQs
//!     │                       ├─ Signal ready
//!     │                       └─ Enter idle loop
//!     │
//!     └─ Wait for all CPUs ready
//! ```
//!
//! # CPU Bring-Up Sequence
//!
//! 1. **Boot CPU (CPU 0)**:
//!    - Initializes platform (UART, GIC Distributor, etc.)
//!    - Calls `smp::init()` to bring up secondary CPUs
//!    - Uses PSCI `CPU_ON` to start each secondary CPU
//!    - Waits for each CPU to signal ready
//!
//! 2. **Secondary CPUs (CPU 1-3)**:
//!    - Start at `secondary_entry()` with CPU ID in x0
//!    - Initialize per-CPU GIC redistributor
//!    - Initialize per-CPU timer
//!    - Enable interrupts
//!    - Signal ready to boot CPU
//!    - Enter scheduler idle loop
//!
//! # References
//!
//! - ARM PSCI Specification (CPU_ON function)
//! - ARM GICv3 Architecture Specification (per-CPU redistributors)
//! - BCM2712 Datasheet (RPi5 CPU configuration)

use crate::arch::psci::{cpu_on, PsciError};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Maximum number of CPUs supported (RPi5 has 4 Cortex-A76 cores)
pub const MAX_CPUS: usize = 4;

/// Number of CPUs currently online
static NUM_CPUS_ONLINE: AtomicU32 = AtomicU32::new(1);

/// Per-CPU boot flags (set to true when CPU is ready)
static CPU_BOOT_FLAGS: [AtomicBool; MAX_CPUS] = [
    AtomicBool::new(true),  // CPU 0 (boot CPU) is always ready
    AtomicBool::new(false), // CPU 1
    AtomicBool::new(false), // CPU 2
    AtomicBool::new(false), // CPU 3
];

/// Per-CPU stacks (16KB each)
///
/// Each secondary CPU needs its own stack before it can call Rust functions.
/// These stacks are used only during early boot; later the scheduler provides
/// per-process stacks.
#[repr(C, align(16))]
struct CpuStack([u8; 16 * 1024]);

static mut CPU_STACKS: [CpuStack; MAX_CPUS] = [
    CpuStack([0; 16 * 1024]),
    CpuStack([0; 16 * 1024]),
    CpuStack([0; 16 * 1024]),
    CpuStack([0; 16 * 1024]),
];

/// Initialize SMP and bring up secondary CPUs
///
/// This function uses PSCI to bring up all secondary CPUs and waits for them
/// to signal ready. It should be called once during kernel initialization,
/// after PSCI, GIC, and timer have been initialized on the boot CPU.
///
/// # Safety
///
/// Must be called exactly once during boot, after:
/// - PSCI initialization
/// - GIC Distributor initialization
/// - Boot CPU timer initialization
pub unsafe fn init() {
    crate::info!("SMP: Initializing multi-core support");

    // Determine how many CPUs are available
    // For RPi5, we know it has 4 cores, but we'll try to bring them all up
    let num_cpus = MAX_CPUS;

    crate::info!("SMP: Attempting to bring up {} CPUs", num_cpus);

    // Bring up each secondary CPU
    for cpu_id in 1..num_cpus {
        bring_up_cpu(cpu_id);
    }

    let cpus_online = NUM_CPUS_ONLINE.load(Ordering::Acquire);
    crate::info!("SMP: {} CPU(s) online", cpus_online);

    if cpus_online == 1 {
        crate::warn!("SMP: Failed to bring up any secondary CPUs");
    } else {
        crate::info!("SMP: Multi-core support active");
    }
}

/// Bring up a specific CPU using PSCI
fn bring_up_cpu(cpu_id: usize) {
    crate::info!("SMP: Bringing up CPU {}", cpu_id);

    // Get entry point address
    let entry_point = secondary_entry as *const () as u64;

    // Get stack pointer for this CPU (top of stack)
    let stack_top = unsafe {
        let stack = &CPU_STACKS[cpu_id].0;
        stack.as_ptr().add(stack.len()) as u64
    };

    // MPIDR value for this CPU
    // For RPi5, CPU IDs map directly to MPIDR affinity level 0
    let target_cpu = cpu_id as u64;

    // Context ID passed to secondary CPU (we pass stack pointer)
    let context_id = stack_top;

    crate::info!("SMP:   Entry point: {:#x}", entry_point);
    crate::info!("SMP:   Stack top:   {:#x}", stack_top);
    crate::info!("SMP:   Target CPU:  {}", target_cpu);

    // Use PSCI to start the CPU
    match cpu_on(target_cpu, entry_point, context_id) {
        Ok(()) => {
            crate::info!("SMP:   CPU_ON successful, waiting for ready signal...");

            // Wait for CPU to signal ready (with timeout)
            const TIMEOUT_MS: u32 = 1000;
            const CHECK_INTERVAL_US: u32 = 100;
            let iterations = (TIMEOUT_MS * 1000) / CHECK_INTERVAL_US;

            for i in 0..iterations {
                if CPU_BOOT_FLAGS[cpu_id].load(Ordering::Acquire) {
                    NUM_CPUS_ONLINE.fetch_add(1, Ordering::Release);
                    crate::info!("SMP: CPU {} is online", cpu_id);
                    return;
                }

                // Busy wait (we don't have a proper delay yet)
                for _ in 0..CHECK_INTERVAL_US {
                    core::hint::spin_loop();
                }

                // Log progress every 100ms
                if i % 1000 == 0 && i > 0 {
                    crate::info!("SMP:   Still waiting for CPU {} ({} ms)...", cpu_id, i / 10);
                }
            }

            crate::warn!("SMP: Timeout waiting for CPU {} to come online", cpu_id);
        }
        Err(PsciError::AlreadyOn) => {
            crate::warn!("SMP: CPU {} is already on (unexpected)", cpu_id);
        }
        Err(PsciError::InvalidParameters) => {
            crate::error!("SMP: Invalid parameters for CPU {}", cpu_id);
        }
        Err(e) => {
            crate::error!("SMP: Failed to start CPU {}: {:?}", cpu_id, e);
        }
    }
}

/// Secondary CPU entry point
///
/// This function is called by firmware (via PSCI) when a secondary CPU is started.
/// It receives:
/// - x0: CPU ID (MPIDR value)
/// - x1: Context ID (stack pointer in our case)
///
/// # Safety
///
/// This is the first Rust code that runs on secondary CPUs. It must:
/// 1. Set up the stack pointer (passed in x1)
/// 2. Initialize per-CPU hardware (GIC redistributor, timer)
/// 3. Enable interrupts
/// 4. Signal ready to boot CPU
/// 5. Never return
#[no_mangle]
#[link_section = ".text"]
pub unsafe extern "C" fn secondary_entry(cpu_id: u64, stack_ptr: u64) -> ! {
    // Extract CPU ID from MPIDR
    let cpu = (cpu_id & 0xFF) as usize;

    // Set up stack pointer
    // Note: In a real implementation, this would be done in assembly before
    // calling this function. For now, we assume firmware has set SP to stack_ptr.

    // At this point, we can safely use the stack and call Rust functions
    secondary_rust_entry(cpu);
}

/// Secondary CPU entry point (Rust)
///
/// This is called from `secondary_entry` after basic setup.
fn secondary_rust_entry(cpu_id: usize) -> ! {
    // We're on a secondary CPU now!
    // Note: We can't use info! yet because UART might not be safe for concurrent access
    // In production, we'd use per-CPU buffers or atomic logging

    // 1. Initialize GIC redistributor for this CPU
    #[cfg(not(test))]
    unsafe {
        crate::arch::aarch64::gicv3::init_cpu(cpu_id);
    }

    // 2. Initialize timer for this CPU
    // (The timer itself is per-CPU, but the configuration is shared)
    #[cfg(not(test))]
    unsafe {
        // Enable timer IRQ for this CPU
        if let Some(_) = crate::arch::aarch64::gicv3::enable_irq_checked(
            crate::arch::aarch64::timer::TIMER_IRQ_PHYS
        ) {
            // Timer IRQ enabled for this CPU
        }
    }

    // 3. Enable interrupts
    unsafe {
        core::arch::asm!(
            "msr DAIFClr, #2",  // Clear IRQ mask (bit 1)
            options(nomem, nostack)
        );
    }

    // 4. Signal that this CPU is ready
    CPU_BOOT_FLAGS[cpu_id].store(true, Ordering::Release);

    // Now it's safe to log
    crate::info!("SMP: CPU {} initialized and ready", cpu_id);

    // 5. Enter idle loop
    // In production, this would enter the scheduler's idle loop
    // For now, just spin with WFI
    cpu_idle_loop(cpu_id);
}

/// CPU idle loop
///
/// This is where a CPU goes when it has no work to do. It uses the ARM WFI
/// (Wait For Interrupt) instruction to save power until an interrupt arrives.
fn cpu_idle_loop(cpu_id: usize) -> ! {
    crate::info!("SMP: CPU {} entering idle loop", cpu_id);

    loop {
        // Wait for interrupt (low power)
        unsafe {
            core::arch::asm!(
                "wfi",  // Wait For Interrupt
                options(nomem, nostack)
            );
        }

        // When we wake up from WFI, an interrupt has occurred
        // The interrupt handler will run, then we return here

        // In a real scheduler, we'd check for work here:
        // - Check run queue
        // - Handle IPIs
        // - Run processes
        //
        // For now, just loop back to WFI
    }
}

/// Get the number of CPUs currently online
pub fn num_cpus() -> usize {
    NUM_CPUS_ONLINE.load(Ordering::Acquire) as usize
}

/// Get the current CPU ID
///
/// Reads the MPIDR_EL1 register to determine which CPU we're running on.
#[inline]
pub fn current_cpu_id() -> usize {
    crate::arch::psci::current_cpu_id()
}

/// Check if a specific CPU is online
pub fn is_cpu_online(cpu_id: usize) -> bool {
    if cpu_id >= MAX_CPUS {
        return false;
    }
    CPU_BOOT_FLAGS[cpu_id].load(Ordering::Acquire)
}

/// Send an Inter-Processor Interrupt (IPI) to a specific CPU
///
/// Uses GICv3 SGI (Software Generated Interrupt) to signal another CPU.
///
/// # Arguments
/// * `target_cpu` - CPU ID to interrupt (0-3)
/// * `sgi_num` - SGI number (0-15)
pub fn send_ipi(target_cpu: usize, sgi_num: u8) {
    if target_cpu >= MAX_CPUS {
        crate::warn!("SMP: Invalid target CPU {} for IPI", target_cpu);
        return;
    }

    if !is_cpu_online(target_cpu) {
        crate::warn!("SMP: Cannot send IPI to offline CPU {}", target_cpu);
        return;
    }

    if sgi_num >= 16 {
        crate::warn!("SMP: Invalid SGI number {} (must be 0-15)", sgi_num);
        return;
    }

    // Send SGI via ICC_SGI1R_EL1 system register
    // Format: [55:48] Aff3, [39:32] Aff2, [23:16] Aff1, [15:0] target list + INTID
    let sgi_value = (sgi_num as u64) << 24 | (1u64 << target_cpu);

    unsafe {
        core::arch::asm!(
            "msr ICC_SGI1R_EL1, {}",
            in(reg) sgi_value,
            options(nomem, nostack)
        );
    }
}

/// Broadcast an IPI to all CPUs except the current one
pub fn send_ipi_broadcast(sgi_num: u8) {
    let current = current_cpu_id();

    for cpu in 0..num_cpus() {
        if cpu != current {
            send_ipi(cpu, sgi_num);
        }
    }
}

/// SGI numbers for different IPI purposes
pub mod ipi {
    /// IPI for scheduler (wake up idle CPU)
    pub const RESCHEDULE: u8 = 0;

    /// IPI for TLB flush
    pub const TLB_FLUSH: u8 = 1;

    /// IPI for generic function call
    pub const CALL_FUNCTION: u8 = 2;

    /// IPI for stopping a CPU
    pub const STOP: u8 = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_limits() {
        assert_eq!(MAX_CPUS, 4);
    }

    #[test]
    fn test_initial_state() {
        // CPU 0 should be online
        assert!(is_cpu_online(0));

        // Other CPUs should be offline initially
        assert!(!is_cpu_online(1));
        assert!(!is_cpu_online(2));
        assert!(!is_cpu_online(3));
    }

    #[test]
    fn test_ipi_numbers() {
        // SGI numbers should be in valid range
        assert!(ipi::RESCHEDULE < 16);
        assert!(ipi::TLB_FLUSH < 16);
        assert!(ipi::CALL_FUNCTION < 16);
        assert!(ipi::STOP < 16);
    }
}
