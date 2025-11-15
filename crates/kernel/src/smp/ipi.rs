/// Inter-Processor Interrupts (IPIs) - Phase E
///
/// IPIs allow one CPU to send interrupts to other CPUs for:
/// - Rescheduling (wake up idle CPUs)
/// - TLB shootdown (invalidate TLB entries on all CPUs)
/// - Function calls (execute function on remote CPU)
/// - System-wide operations

use core::sync::atomic::{AtomicU64, Ordering};

/// IPI types (SGI - Software Generated Interrupt IDs 0-15)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpiType {
    /// Reschedule request (SGI 0)
    Reschedule = 0,
    /// TLB shootdown (SGI 1)
    TlbShootdown = 1,
    /// Function call (SGI 2)
    FunctionCall = 2,
    /// Generic IPI (SGI 3)
    Generic = 3,
}

/// IPI statistics per CPU
static IPI_STATS: [IpiStats; super::MAX_CPUS] = [
    IpiStats::new(),
    IpiStats::new(),
    IpiStats::new(),
    IpiStats::new(),
    IpiStats::new(),
    IpiStats::new(),
    IpiStats::new(),
    IpiStats::new(),
];

/// IPI statistics for a single CPU
pub struct IpiStats {
    /// Number of reschedule IPIs received
    pub reschedule: AtomicU64,
    /// Number of TLB shootdown IPIs received
    pub tlb_shootdown: AtomicU64,
    /// Number of function call IPIs received
    pub function_call: AtomicU64,
    /// Number of generic IPIs received
    pub generic: AtomicU64,
}

impl IpiStats {
    const fn new() -> Self {
        Self {
            reschedule: AtomicU64::new(0),
            tlb_shootdown: AtomicU64::new(0),
            function_call: AtomicU64::new(0),
            generic: AtomicU64::new(0),
        }
    }

    fn inc(&self, ipi_type: IpiType) {
        match ipi_type {
            IpiType::Reschedule => self.reschedule.fetch_add(1, Ordering::Relaxed),
            IpiType::TlbShootdown => self.tlb_shootdown.fetch_add(1, Ordering::Relaxed),
            IpiType::FunctionCall => self.function_call.fetch_add(1, Ordering::Relaxed),
            IpiType::Generic => self.generic.fetch_add(1, Ordering::Relaxed),
        };
    }
}

/// Send IPI to a specific CPU
///
/// Uses ARM64 GICv3 System Register interface to send SGI (Software Generated Interrupt).
pub fn send_ipi(target_cpu: usize, ipi_type: IpiType) {
    if target_cpu >= super::MAX_CPUS {
        crate::warn!("IPI: Invalid target CPU {}", target_cpu);
        return;
    }

    if !super::is_cpu_online(target_cpu) {
        crate::debug!("IPI: Target CPU {} is offline", target_cpu);
        return;
    }

    // ARM64 GICv3 SGI using ICC_SGI1R_EL1
    // Format: [55:48] Aff3, [39:32] IRM|Aff2, [23:16] Aff1, [15:0] TargetList|IntID
    // For QEMU virt: Aff0 = CPU ID, others = 0
    // IntID = SGI number (0-15)
    // TargetList = bitmap of CPUs in Aff0 (bit N = CPU N)

    let target_list = 1u64 << target_cpu; // Bitmap: bit N = CPU N
    let intid = ipi_type as u64;

    // ICC_SGI1R_EL1 format:
    // [55:48] = Aff3 (0 for QEMU virt)
    // [47:40] = RS (RangeSelector, 0 for single CPU)
    // [39:32] = Aff2 (0 for QEMU virt)
    // [31:24] = IRM (Interrupt Routing Mode, 0 = use TargetList)
    // [23:16] = Aff1 (0 for QEMU virt)
    // [15:0] = TargetList (bits 15-0) | IntID (bits 3-0)

    let sgi_value = (intid & 0xF) | ((target_list & 0xFFFF) << 16);

    unsafe {
        // Write to ICC_SGI1R_EL1 to send SGI
        core::arch::asm!(
            "msr ICC_SGI1R_EL1, {}",
            "isb",
            in(reg) sgi_value,
            options(nomem, nostack)
        );
    }

    crate::debug!("IPI: Sent {:?} to CPU {}", ipi_type, target_cpu);
}

/// Send IPI to all other CPUs (broadcast)
pub fn send_ipi_all_but_self(ipi_type: IpiType) {
    let current_cpu = crate::arch::current_cpu_id();

    for cpu_id in 0..super::MAX_CPUS {
        if cpu_id == current_cpu {
            continue;
        }

        if super::is_cpu_online(cpu_id) {
            send_ipi(cpu_id, ipi_type);
        }
    }
}

/// Handle IPI interrupt
///
/// Called from interrupt handler when SGI is received.
/// Returns true if IPI was handled, false otherwise.
pub fn handle_ipi(intid: u32) -> bool {
    let cpu_id = crate::arch::current_cpu_id();

    // Determine IPI type from interrupt ID
    let ipi_type = match intid {
        0 => IpiType::Reschedule,
        1 => IpiType::TlbShootdown,
        2 => IpiType::FunctionCall,
        3 => IpiType::Generic,
        _ => return false, // Not an IPI we handle
    };

    // Update statistics
    if cpu_id < super::MAX_CPUS {
        IPI_STATS[cpu_id].inc(ipi_type);
    }

    crate::debug!("IPI: CPU {} received {:?}", cpu_id, ipi_type);

    // Handle IPI based on type
    match ipi_type {
        IpiType::Reschedule => {
            // Wake up scheduler on this CPU
            // TODO: Trigger scheduler reschedule
            crate::debug!("IPI: Reschedule on CPU {}", cpu_id);
        }

        IpiType::TlbShootdown => {
            // Invalidate TLB on this CPU
            crate::mm::flush_tlb_all();
            crate::debug!("IPI: TLB flush on CPU {}", cpu_id);
        }

        IpiType::FunctionCall => {
            // Execute queued function
            // TODO: Implement function call queue
            crate::debug!("IPI: Function call on CPU {}", cpu_id);
        }

        IpiType::Generic => {
            // Generic wakeup
            crate::debug!("IPI: Generic on CPU {}", cpu_id);
        }
    }

    true
}

/// Send reschedule IPI to a CPU
pub fn send_reschedule_ipi(target_cpu: usize) {
    send_ipi(target_cpu, IpiType::Reschedule);
}

/// Send TLB shootdown to all CPUs
pub fn tlb_shootdown_all() {
    // Flush local TLB first
    crate::mm::flush_tlb_all();

    // Send IPI to all other CPUs
    send_ipi_all_but_self(IpiType::TlbShootdown);

    // TODO: Wait for all CPUs to acknowledge (use atomic counter)
}

/// Get IPI statistics for a CPU
pub fn get_stats(cpu_id: usize) -> Option<(u64, u64, u64, u64)> {
    if cpu_id >= super::MAX_CPUS {
        return None;
    }

    let stats = &IPI_STATS[cpu_id];
    Some((
        stats.reschedule.load(Ordering::Relaxed),
        stats.tlb_shootdown.load(Ordering::Relaxed),
        stats.function_call.load(Ordering::Relaxed),
        stats.generic.load(Ordering::Relaxed),
    ))
}
