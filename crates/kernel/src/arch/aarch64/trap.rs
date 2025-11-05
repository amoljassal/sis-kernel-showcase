// AArch64 exception handling and trap vectors
// Phase A0 - Basic trap path for syscalls

use crate::lib::error::Errno;
use core::arch::asm;

/// Saved register state on exception
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapFrame {
    // General purpose registers x0-x30
    pub x0: u64,  pub x1: u64,  pub x2: u64,  pub x3: u64,
    pub x4: u64,  pub x5: u64,  pub x6: u64,  pub x7: u64,
    pub x8: u64,  pub x9: u64,  pub x10: u64, pub x11: u64,
    pub x12: u64, pub x13: u64, pub x14: u64, pub x15: u64,
    pub x16: u64, pub x17: u64, pub x18: u64, pub x19: u64,
    pub x20: u64, pub x21: u64, pub x22: u64, pub x23: u64,
    pub x24: u64, pub x25: u64, pub x26: u64, pub x27: u64,
    pub x28: u64, pub x29: u64, pub x30: u64, // x30 = LR

    // Stack pointer and program counter
    pub sp: u64,
    pub pc: u64,      // ELR_EL1
    pub pstate: u64,  // SPSR_EL1
}

impl TrapFrame {
    pub fn new_zeroed() -> Self {
        Self {
            x0: 0, x1: 0, x2: 0, x3: 0, x4: 0, x5: 0, x6: 0, x7: 0,
            x8: 0, x9: 0, x10: 0, x11: 0, x12: 0, x13: 0, x14: 0, x15: 0,
            x16: 0, x17: 0, x18: 0, x19: 0, x20: 0, x21: 0, x22: 0, x23: 0,
            x24: 0, x25: 0, x26: 0, x27: 0, x28: 0, x29: 0, x30: 0,
            sp: 0, pc: 0, pstate: 0,
        }
    }
}

impl Default for TrapFrame {
    fn default() -> Self {
        Self::new_zeroed()
    }
}

/// Exception Syndrome Register (ESR_EL1) bits
const ESR_EC_MASK: u64 = 0xFC000000;
const ESR_EC_SHIFT: u64 = 26;
const ESR_EC_SVC_AARCH64: u64 = 0x15;      // SVC from AArch64 EL0
const ESR_EC_INST_ABORT_LOWER: u64 = 0x20; // Instruction abort from lower EL
const ESR_EC_DATA_ABORT_LOWER: u64 = 0x24; // Data abort from lower EL

/// Data Fault Status Code
const ESR_DFSC_MASK: u64 = 0x3F;

/// Write not Read bit in ESR
const ESR_WNR: u64 = 1 << 6;

/// Read ESR_EL1 (Exception Syndrome Register)
#[inline(always)]
fn read_esr_el1() -> u64 {
    let esr: u64;
    unsafe {
        asm!("mrs {}, ESR_EL1", out(reg) esr);
    }
    esr
}

/// Read FAR_EL1 (Fault Address Register)
#[inline(always)]
fn read_far_el1() -> u64 {
    let far: u64;
    unsafe {
        asm!("mrs {}, FAR_EL1", out(reg) far);
    }
    far
}

/// Handle synchronous exception from lower EL (EL0)
#[no_mangle]
pub extern "C" fn handle_sync_exception(frame: &mut TrapFrame) {
    let esr = read_esr_el1();
    let ec = (esr & ESR_EC_MASK) >> ESR_EC_SHIFT;

    match ec {
        ESR_EC_SVC_AARCH64 => {
            // Syscall from EL0
            handle_syscall(frame);
        }
        ESR_EC_INST_ABORT_LOWER | ESR_EC_DATA_ABORT_LOWER => {
            // Page fault
            let fault_addr = read_far_el1();
            let write_fault = (esr & ESR_WNR) != 0;
            handle_page_fault(frame, fault_addr, write_fault, esr);
        }
        _ => {
            crate::error!(
                "Unhandled exception: EC={:#x}, ESR={:#x}, PC={:#x}",
                ec, esr, frame.pc
            );
            panic!("Unhandled synchronous exception");
        }
    }
}

/// Handle syscall (SVC instruction)
fn handle_syscall(frame: &mut TrapFrame) {
    // Syscall number is in x8
    // Arguments are in x0-x5
    let nr = frame.x8 as usize;
    let args = [frame.x0, frame.x1, frame.x2, frame.x3, frame.x4, frame.x5];

    // Call syscall dispatcher
    let result = crate::syscall::syscall_dispatcher(nr, &args);

    // Return value in x0 (negative for errno)
    frame.x0 = result as u64;

    // Move PC past the SVC instruction (4 bytes)
    frame.pc += 4;
}

/// Handle page fault
fn handle_page_fault(frame: &mut TrapFrame, fault_addr: u64, write: bool, esr: u64) {
    crate::debug!(
        "Page fault at {:#x} ({}), ESR={:#x}, PC={:#x}",
        fault_addr,
        if write { "write" } else { "read" },
        esr,
        frame.pc
    );

    // Call mm page fault handler
    match crate::mm::handle_page_fault(frame, fault_addr, esr) {
        Ok(()) => {
            // Fault handled successfully
            crate::debug!("Page fault handled successfully");
        }
        Err(e) => {
            // Unhandled page fault - terminate process
            crate::error!(
                "Unhandled page fault at {:#x}: {:?}",
                fault_addr, e
            );
            // For now, panic. In full impl, would send SIGSEGV to process
            panic!("Unhandled page fault");
        }
    }
}

/// Handle IRQ
#[no_mangle]
pub extern "C" fn handle_irq(frame: &mut TrapFrame) {
    // Read interrupt number from GIC
    // For now, assume timer interrupt (PPI 30 for AArch64 generic timer)

    // Call scheduler timer tick
    crate::process::scheduler::timer_tick();

    // TODO: ACK/EOI the interrupt via GIC

    // Check if reschedule is needed
    if crate::process::scheduler::need_resched() {
        // Save current task's trap frame
        if let Some(pid) = crate::process::scheduler::current_pid() {
            let mut table = crate::process::get_process_table();
            if let Some(ref mut t) = *table {
                if let Some(task) = t.get_mut(pid) {
                    task.trap_frame = *frame;
                }
            }
        }

        // Schedule next task
        crate::process::scheduler::schedule();

        // Load next task's trap frame (already set by scheduler via set_elr/spsr/sp_el0)
        // When we return via ERET, we'll enter the new task
    }
}

/// Handle FIQ (stub for Phase A0)
#[no_mangle]
pub extern "C" fn handle_fiq(_frame: &mut TrapFrame) {
    crate::warn!("FIQ received but not handled in Phase A0");
}

/// Handle SError (stub for Phase A0)
#[no_mangle]
pub extern "C" fn handle_serror(_frame: &mut TrapFrame) {
    crate::error!("SError received");
    panic!("SError exception");
}

/// Handle synchronous exception from current EL (kernel mode)
#[no_mangle]
pub extern "C" fn handle_sync_curr_el(frame: &mut TrapFrame) {
    let esr = read_esr_el1();
    let ec = (esr & ESR_EC_MASK) >> ESR_EC_SHIFT;

    crate::error!(
        "Kernel exception: EC={:#x}, ESR={:#x}, PC={:#x}",
        ec, esr, frame.pc
    );
    panic!("Kernel synchronous exception");
}

/// Initialize exception vector table
/// This should be called early in boot, before enabling interrupts
pub fn init_exception_vectors() {
    extern "C" {
        static exception_vector_table: u64;
    }

    unsafe {
        let vbar_addr = &exception_vector_table as *const _ as u64;

        // Set VBAR_EL1 to point to our exception vector table
        asm!(
            "msr VBAR_EL1, {}",
            in(reg) vbar_addr
        );

        // Instruction Synchronization Barrier - ensure VBAR is set before continuing
        asm!("isb");

        // Enable alignment fault checking
        let mut sctlr: u64;
        asm!("mrs {}, SCTLR_EL1", out(reg) sctlr);
        sctlr |= (1 << 1);  // A bit: Alignment check enable
        asm!("msr SCTLR_EL1, {}", in(reg) sctlr);
        asm!("isb");

        crate::info!("VBAR_EL1 set to {:#x}", vbar_addr);
    }
}
