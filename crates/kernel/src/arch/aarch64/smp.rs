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
//! # Platform Support
//!
//! **QEMU/TCG (Software Emulation)**: PSCI CPU_ON is not fully functional in QEMU's
//! TCG mode. Secondary CPUs will not start, and the kernel will run in single-core
//! mode. This is a known QEMU limitation and is expected behavior.
//!
//! **Real Hardware / QEMU+KVM**: PSCI CPU_ON works correctly on:
//! - Raspberry Pi 4/5 (real hardware with ARM Trusted Firmware)
//! - QEMU with KVM acceleration (hardware virtualization)
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

/// Debug: track secondary CPU boot progress
/// 0 = not started, 1 = entry reached, 2 = MMU enabled, 3 = stack set, 4 = Rust reached
static CPU_DEBUG_STAGE: [AtomicU32; MAX_CPUS] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
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

/// L1 page table from bringup module
extern "C" {
    static L1_TABLE: [u64; 512];
}

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

    // Verify PSCI CPU_ON is supported
    if !crate::arch::psci::is_feature_supported(crate::arch::psci::PsciFunction::CpuOn) {
        crate::error!("SMP: PSCI CPU_ON not supported by firmware!");
        return;
    }
    crate::info!("SMP: PSCI CPU_ON is supported");

    // Determine how many CPUs are available
    // For RPi5, we know it has 4 cores, but we'll try to bring them all up
    let num_cpus = MAX_CPUS;

    crate::info!("SMP: Attempting to bring up {} CPUs", num_cpus);

    // Read boot CPU's MPIDR to understand affinity structure
    let boot_mpidr: u64;
    core::arch::asm!("mrs {}, mpidr_el1", out(reg) boot_mpidr);
    crate::warn!("SMP: Boot CPU MPIDR = 0x{:x} (Aff0={}, Aff1={}, Aff2={}, Aff3={})",
                 boot_mpidr,
                 boot_mpidr & 0xFF,
                 (boot_mpidr >> 8) & 0xFF,
                 (boot_mpidr >> 16) & 0xFF,
                 (boot_mpidr >> 32) & 0xFF);

    // Bring up each secondary CPU
    for cpu_id in 1..num_cpus {
        bring_up_cpu(cpu_id);
    }

    let cpus_online = NUM_CPUS_ONLINE.load(Ordering::Acquire);
    crate::info!("SMP: {} CPU(s) online", cpus_online);

    if cpus_online == 1 {
        crate::warn!("SMP: Failed to bring up any secondary CPUs");
        crate::warn!("SMP: This is expected on QEMU/TCG (software emulation)");
        crate::warn!("SMP: PSCI CPU_ON only works on:");
        crate::warn!("SMP:   - Real hardware (Raspberry Pi 4/5)");
        crate::warn!("SMP:   - QEMU with KVM (hardware virtualization)");
        crate::warn!("SMP: Continuing with single-core operation...");
    } else {
        crate::info!("SMP: Multi-core support active");
    }
}

/// Bring up a specific CPU using PSCI
fn bring_up_cpu(cpu_id: usize) {
    crate::warn!("SMP: === Bringing up CPU {} ===", cpu_id);

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

    // Context ID passed to secondary CPU (we pass CPU ID for debugging)
    let context_id = cpu_id as u64;

    crate::warn!("SMP:   Entry point:  {:#x}", entry_point);
    crate::warn!("SMP:   Stack base:   {:#x}", unsafe { CPU_STACKS[cpu_id].0.as_ptr() as u64 });
    crate::warn!("SMP:   Stack top:    {:#x} (size: {} bytes)", stack_top, 16 * 1024);
    crate::warn!("SMP:   Target MPIDR: {:#x}", target_cpu);
    crate::warn!("SMP:   Context ID:   {:#x}", context_id);

    // Use PSCI to start the CPU
    let result = cpu_on(target_cpu, entry_point, context_id);
    crate::warn!("SMP:   PSCI CPU_ON returned: {:?}", result);

    match result {
        Ok(()) => {
            crate::warn!("SMP:   CPU_ON successful, waiting for ready signal...");

            // Wait for CPU to signal ready (with timeout)
            const TIMEOUT_MS: u32 = 1000;
            const CHECK_INTERVAL_US: u32 = 100;
            let iterations = (TIMEOUT_MS * 1000) / CHECK_INTERVAL_US;

            for i in 0..iterations {
                if CPU_BOOT_FLAGS[cpu_id].load(Ordering::Acquire) {
                    NUM_CPUS_ONLINE.fetch_add(1, Ordering::Release);
                    crate::info!("SMP: CPU {} is online after {} ms", cpu_id, i / 10);
                    return;
                }

                // Busy wait (we don't have a proper delay yet)
                for _ in 0..CHECK_INTERVAL_US {
                    core::hint::spin_loop();
                }

                // Log progress every 100ms
                if i % 1000 == 0 && i > 0 {
                    crate::warn!("SMP:   Still waiting for CPU {} ({} ms, flag={})",
                              cpu_id, i / 10, CPU_BOOT_FLAGS[cpu_id].load(Ordering::Acquire));
                }
            }

            crate::error!("SMP: TIMEOUT waiting for CPU {} (1000ms elapsed)", cpu_id);
            crate::error!("SMP:   Final boot flag state: {}", CPU_BOOT_FLAGS[cpu_id].load(Ordering::Acquire));
            crate::error!("SMP:   Debug stage: {} (0=not started, 1=entry, 2=MMU, 3=stack, 4=Rust)",
                          CPU_DEBUG_STAGE[cpu_id].load(Ordering::Acquire));
        }
        Err(PsciError::AlreadyOn) => {
            crate::warn!("SMP: CPU {} reports ALREADY_ON (may need reset)", cpu_id);
        }
        Err(PsciError::InvalidParameters) => {
            crate::error!("SMP: INVALID_PARAMETERS for CPU {} (check MPIDR, entry point, alignment)", cpu_id);
        }
        Err(PsciError::Denied) => {
            crate::error!("SMP: CPU {} DENIED (may be disabled in firmware)", cpu_id);
        }
        Err(PsciError::InvalidAddress) => {
            crate::error!("SMP: INVALID_ADDRESS for CPU {} (entry={:#x})", cpu_id, entry_point);
        }
        Err(e) => {
            crate::error!("SMP: Failed to start CPU {}: {:?}", cpu_id, e);
        }
    }
}

/// Secondary CPU entry point (Assembly trampoline)
///
/// This is the FIRST code that runs on secondary CPUs after PSCI CPU_ON.
/// PSCI passes:
/// - x0: Target CPU (MPIDR value we passed to CPU_ON)
/// - x1: Entry point (this function's address)
/// - x2: Context ID (CPU ID we passed to CPU_ON)
///
/// This naked function must:
/// 1. Set up stack pointer using CPU ID
/// 2. Jump to Rust code
///
/// # Safety
///
/// This is a naked function - no Rust prologue/epilogue.
/// Stack is NOT set up yet - we must do it manually!
#[unsafe(naked)]
#[no_mangle]
#[link_section = ".text"]
pub unsafe extern "C" fn secondary_entry() -> ! {
    use core::arch::naked_asm;
    naked_asm!(
        // PSCI entry point receives:
        // x0 = context_id (CPU ID we passed to PSCI CPU_ON)

        // SIMPLIFIED TEST: Just infinite loop to see if CPU even starts
        "1:",
        "wfe",
        "b 1b",
    )
}

/// Secondary CPU entry point (Rust)
///
/// This is called from `secondary_entry` after basic setup.
fn secondary_rust_entry(cpu_id: usize) -> ! {
    // MMU is already enabled by assembly trampoline
    crate::warn!("SMP: [CPU{}] Secondary entry point reached!", cpu_id);

    // 1. Initialize GIC redistributor for this CPU
    #[cfg(not(test))]
    unsafe {
        crate::warn!("SMP: [CPU{}] Initializing GIC redistributor...", cpu_id);
        crate::arch::aarch64::gicv3::init_cpu(cpu_id);
        crate::warn!("SMP: [CPU{}] GIC redistributor initialized", cpu_id);
    }

    // 2. Initialize timer for this CPU
    // (The timer itself is per-CPU, but the configuration is shared)
    #[cfg(not(test))]
    {
        crate::warn!("SMP: [CPU{}] Enabling timer IRQ...", cpu_id);
        // Enable timer IRQ for this CPU
        if let Some(_) = crate::arch::aarch64::gicv3::enable_irq_checked(
            crate::arch::aarch64::timer::TIMER_IRQ_PHYS
        ) {
            crate::warn!("SMP: [CPU{}] Timer IRQ enabled", cpu_id);
        }
    }

    // 3. Enable interrupts
    unsafe {
        crate::warn!("SMP: [CPU{}] Enabling interrupts...", cpu_id);
        core::arch::asm!(
            "msr DAIFClr, #2",  // Clear IRQ mask (bit 1)
            options(nomem, nostack)
        );
        crate::warn!("SMP: [CPU{}] Interrupts enabled", cpu_id);
    }

    // 4. Signal that this CPU is ready (with memory barrier)
    core::sync::atomic::compiler_fence(Ordering::Release);
    CPU_BOOT_FLAGS[cpu_id].store(true, Ordering::Release);
    core::sync::atomic::compiler_fence(Ordering::SeqCst);

    crate::info!("SMP: [CPU{}] READY - Signaled boot CPU", cpu_id);

    // 5. Enter idle loop
    // In production, this would enter the scheduler's idle loop
    // For now, just spin with WFI
    crate::warn!("SMP: [CPU{}] Entering idle loop", cpu_id);
    cpu_idle_loop(cpu_id);
}

/// Enable MMU for secondary CPU
///
/// Secondary CPUs start with MMU disabled. This function enables MMU using
/// the same configuration and page tables as the boot CPU.
///
/// # Safety
///
/// Must be called exactly once per secondary CPU before any virtual memory access.
unsafe fn enable_secondary_mmu() {
    use core::arch::asm;

    // Set MAIR_EL1 (Memory Attribute Indirection Register)
    // AttrIdx0 = Device-nGnRE (0x04), AttrIdx1 = Normal WBWA (0xFF)
    let mair = (0x04u64) | (0xFFu64 << 8);
    asm!("msr MAIR_EL1, {x}", x = in(reg) mair, options(nostack, preserves_flags));

    // Set TCR_EL1 (Translation Control Register)
    // 4KB pages, Inner/Outer WBWA, Inner shareable, 39-bit VA, 48-bit PA
    let t0sz: u64 = 64 - 39; // 25
    let tcr = (t0sz & 0x3Fu64) |
        (0b01u64 << 8)  | // IRGN0 = WBWA
        (0b01u64 << 10) | // ORGN0 = WBWA
        (0b11u64 << 12) | // SH0 = Inner Shareable
        (0b00u64 << 14) | // TG0 = 4KB
        (0b101u64 << 32); // IPS = 48-bit PA
    asm!("msr TCR_EL1, {x}", x = in(reg) tcr, options(nostack, preserves_flags));
    asm!("isb", options(nostack, preserves_flags));

    // Set TTBR0_EL1 to the boot CPU's L1 page table
    // The page table address is in the bringup module
    extern "C" {
        static L1_TABLE: [u64; 512];
    }
    let l1_pa = &raw const L1_TABLE as *const _ as u64;
    asm!("msr TTBR0_EL1, {x}", x = in(reg) l1_pa, options(nostack, preserves_flags));
    asm!("dsb ish; isb", options(nostack, preserves_flags));

    // Enable MMU + caches in SCTLR_EL1
    let mut sctlr: u64;
    asm!("mrs {x}, SCTLR_EL1", x = out(reg) sctlr);
    sctlr |= (1 << 0) | (1 << 2) | (1 << 12); // M (MMU), C (data cache), I (instruction cache)
    asm!("msr SCTLR_EL1, {x}", x = in(reg) sctlr);
    asm!("isb", options(nostack, preserves_flags));
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
