/// Round-robin preemptive scheduler
///
/// Simple timeslice-based scheduler that switches between runnable tasks.
/// Each task gets a 10ms timeslice before being preempted.

use super::{Pid, ProcessState};
use alloc::collections::VecDeque;
use spin::Mutex;

/// Timeslice in timer ticks (assuming 100Hz timer = 10ms per tick)
const TIMESLICE_TICKS: u32 = 1;

/// Global run queue (simple FIFO for round-robin)
static RUN_QUEUE: Mutex<VecDeque<Pid>> = Mutex::new(VecDeque::new());

/// Current running task PID
static CURRENT_PID: Mutex<Option<Pid>> = Mutex::new(None);

/// Ticks remaining in current timeslice
static TIMESLICE_REMAINING: Mutex<u32> = Mutex::new(TIMESLICE_TICKS);

/// Flag indicating reschedule needed
static NEED_RESCHED: Mutex<bool> = Mutex::new(false);

/// Initialize scheduler
pub fn init() {
    *CURRENT_PID.lock() = None;
    *TIMESLICE_REMAINING.lock() = TIMESLICE_TICKS;
    *NEED_RESCHED.lock() = false;
    crate::info!("Scheduler initialized (timeslice={} ticks)", TIMESLICE_TICKS);
}

/// Get current running task PID
pub fn current_pid() -> Option<Pid> {
    *CURRENT_PID.lock()
}

/// Set current running task
pub fn set_current(pid: Pid) {
    *CURRENT_PID.lock() = Some(pid);
    *TIMESLICE_REMAINING.lock() = TIMESLICE_TICKS;
}

/// Add task to run queue
pub fn enqueue(pid: Pid) {
    let mut queue = RUN_QUEUE.lock();
    if !queue.contains(&pid) {
        queue.push_back(pid);
        crate::debug!("Scheduler: enqueued task PID {}", pid);
    }
}

/// Remove task from run queue
pub fn dequeue(pid: Pid) {
    let mut queue = RUN_QUEUE.lock();
    queue.retain(|&p| p != pid);
    crate::debug!("Scheduler: dequeued task PID {}", pid);
}

/// Timer tick handler - called from IRQ handler
pub fn timer_tick() {
    let mut remaining = TIMESLICE_REMAINING.lock();

    if *remaining > 0 {
        *remaining -= 1;
    }

    if *remaining == 0 {
        // Timeslice expired, request reschedule
        *NEED_RESCHED.lock() = true;
        crate::debug!("Scheduler: timeslice expired, need resched");
    }
}

/// Check if reschedule is needed
pub fn need_resched() -> bool {
    *NEED_RESCHED.lock()
}

/// Clear reschedule flag
fn clear_need_resched() {
    *NEED_RESCHED.lock() = false;
}

/// Pick next task to run (round-robin)
fn pick_next() -> Option<Pid> {
    let mut queue = RUN_QUEUE.lock();

    // Try to get next task from queue
    if let Some(pid) = queue.pop_front() {
        // Re-enqueue for next round
        queue.push_back(pid);
        Some(pid)
    } else {
        None
    }
}

/// Schedule next task
///
/// This is the core scheduler function that performs context switching.
/// Should be called with interrupts disabled.
pub fn schedule() {
    clear_need_resched();

    // Get current and next tasks
    let current = current_pid();
    let next_pid = match pick_next() {
        Some(pid) => pid,
        None => {
            crate::debug!("Scheduler: no runnable tasks");
            return;
        }
    };

    // If same task, just reset timeslice
    if current == Some(next_pid) {
        *TIMESLICE_REMAINING.lock() = TIMESLICE_TICKS;
        return;
    }

    crate::debug!("Scheduler: switching from {:?} to {}", current, next_pid);

    // Get process table
    let mut table = super::get_process_table();
    let table = match table.as_mut() {
        Some(t) => t,
        None => {
            crate::error!("Scheduler: process table not initialized");
            return;
        }
    };

    // Get next task
    let next = match table.get_mut(next_pid) {
        Some(task) => task,
        None => {
            crate::error!("Scheduler: next task {} not found", next_pid);
            return;
        }
    };

    // Switch to next task's address space
    crate::mm::switch_user_mm(next.mm.page_table);

    // Set EL0 context from trap frame
    crate::arch::set_elr_el1(next.trap_frame.pc);
    crate::arch::set_spsr_el1(next.trap_frame.pstate);
    crate::arch::set_sp_el0(next.trap_frame.sp);

    // Update current PID
    set_current(next_pid);

    // TODO: For full context switching with kernel stacks:
    // if let Some(prev_pid) = current {
    //     let prev = table.get_mut(prev_pid).unwrap();
    //     unsafe {
    //         crate::arch::switch_to(&mut prev.cpu_context, &next.cpu_context);
    //     }
    // }

    crate::debug!("Scheduler: switched to task {}", next_pid);
}

/// Yield CPU voluntarily
pub fn yield_now() {
    *NEED_RESCHED.lock() = true;
    schedule();
}

/// Block current process
pub fn block_current() {
    if let Some(pid) = current_pid() {
        dequeue(pid);

        let mut table = super::get_process_table();
        if let Some(ref mut t) = *table {
            if let Some(task) = t.get_mut(pid) {
                task.state = ProcessState::Sleeping;
            }
        }

        *NEED_RESCHED.lock() = true;
        schedule();
    }
}

/// Wake a process by PID
pub fn wake_process(pid: Pid) {
    let mut table = super::get_process_table();
    if let Some(ref mut t) = *table {
        if let Some(task) = t.get_mut(pid) {
            if task.state == ProcessState::Sleeping {
                task.state = ProcessState::Running;
                enqueue(pid);
            }
        }
    }
}
