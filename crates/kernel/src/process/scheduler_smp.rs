/// SMP-aware round-robin preemptive scheduler - Phase E
///
/// Per-CPU scheduler with timeslice-based preemption and load balancing.
/// Each CPU has its own runqueue and schedules independently.

use super::{Pid, ProcessState};
use core::sync::atomic::{AtomicBool, Ordering};

/// Timeslice in timer ticks (assuming 100Hz timer = 10ms per tick)
const TIMESLICE_TICKS: u32 = 1;

/// Per-CPU timeslice remaining (indexed by CPU ID)
static TIMESLICE_REMAINING: [core::sync::atomic::AtomicU32; crate::smp::MAX_CPUS] = [
    core::sync::atomic::AtomicU32::new(TIMESLICE_TICKS),
    core::sync::atomic::AtomicU32::new(TIMESLICE_TICKS),
    core::sync::atomic::AtomicU32::new(TIMESLICE_TICKS),
    core::sync::atomic::AtomicU32::new(TIMESLICE_TICKS),
    core::sync::atomic::AtomicU32::new(TIMESLICE_TICKS),
    core::sync::atomic::AtomicU32::new(TIMESLICE_TICKS),
    core::sync::atomic::AtomicU32::new(TIMESLICE_TICKS),
    core::sync::atomic::AtomicU32::new(TIMESLICE_TICKS),
];

/// Per-CPU reschedule flags (indexed by CPU ID)
static NEED_RESCHED: [AtomicBool; crate::smp::MAX_CPUS] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

/// Initialize scheduler (called once during boot)
pub fn init() {
    // Initialize all CPUs' timeslices
    for i in 0..crate::smp::MAX_CPUS {
        TIMESLICE_REMAINING[i].store(TIMESLICE_TICKS, Ordering::Release);
        NEED_RESCHED[i].store(false, Ordering::Release);
    }

    crate::info!("Scheduler initialized (SMP, timeslice={} ticks)", TIMESLICE_TICKS);
}

/// Get current running task PID (for current CPU)
pub fn current_pid() -> Option<Pid> {
    let cpu_id = crate::arch::current_cpu_id();
    let pid = crate::smp::percpu::get(cpu_id).current_pid();

    if pid == 0 {
        None
    } else {
        Some(pid)
    }
}

/// Set current running task (on current CPU)
pub fn set_current(pid: Pid) {
    let cpu_id = crate::arch::current_cpu_id();

    crate::smp::percpu::get(cpu_id).set_current_pid(pid);
    TIMESLICE_REMAINING[cpu_id].store(TIMESLICE_TICKS, Ordering::Release);

    // Mark CPU as not idle
    crate::smp::percpu::get(cpu_id).set_idle(false);
}

/// Add task to run queue (local CPU)
pub fn enqueue(pid: Pid) {
    crate::smp::percpu::enqueue_current(pid);
    crate::debug!("Scheduler: enqueued task PID {} on CPU {}", pid, crate::arch::current_cpu_id());
}

/// Add task to specific CPU's run queue
pub fn enqueue_on(cpu_id: usize, pid: Pid) {
    crate::smp::percpu::enqueue_on(cpu_id, pid);
    crate::debug!("Scheduler: enqueued task PID {} on CPU {}", pid, cpu_id);
}

/// Remove task from run queue (searches all CPUs)
pub fn dequeue(pid: Pid) {
    // For now, just try to remove from current CPU
    // TODO: Search all CPUs' runqueues
    let cpu_id = crate::arch::current_cpu_id();
    let percpu = crate::smp::percpu::get(cpu_id);

    // SAFETY: Only accessed by owning CPU
    unsafe {
        let runqueue = &mut *percpu.runqueue.get();
        runqueue.retain(|&p| p != pid);
    }

    percpu.update_load();
    crate::debug!("Scheduler: dequeued task PID {} from CPU {}", pid, cpu_id);
}

/// Timer tick handler - called from IRQ handler on each CPU
pub fn timer_tick() {
    let cpu_id = crate::arch::current_cpu_id();

    // Increment per-CPU timer ticks
    crate::smp::percpu::get(cpu_id).inc_timer_ticks();

    // Decrement timeslice
    let remaining = TIMESLICE_REMAINING[cpu_id].load(Ordering::Acquire);
    if remaining > 0 {
        TIMESLICE_REMAINING[cpu_id].store(remaining - 1, Ordering::Release);
    }

    if remaining == 0 {
        // Timeslice expired, request reschedule
        NEED_RESCHED[cpu_id].store(true, Ordering::Release);
        crate::debug!("Scheduler: CPU {} timeslice expired, need resched", cpu_id);
    }

    // Periodic load balancing (every 10 ticks = 100ms)
    if crate::smp::percpu::get(cpu_id).timer_ticks.load(Ordering::Relaxed) % 10 == 0 {
        balance_load();
    }
}

/// Check if reschedule is needed (on current CPU)
pub fn need_resched() -> bool {
    let cpu_id = crate::arch::current_cpu_id();
    NEED_RESCHED[cpu_id].load(Ordering::Acquire)
}

/// Clear reschedule flag (on current CPU)
fn clear_need_resched() {
    let cpu_id = crate::arch::current_cpu_id();
    NEED_RESCHED[cpu_id].store(false, Ordering::Release);
}

/// Pick next task to run (from current CPU's runqueue)
fn pick_next() -> Option<Pid> {
    crate::smp::percpu::dequeue_current()
}

/// Schedule next task (on current CPU)
///
/// This is the core scheduler function that performs context switching.
/// Should be called with interrupts disabled.
pub fn schedule() {
    let cpu_id = crate::arch::current_cpu_id();
    clear_need_resched();

    // Get current and next tasks
    let current = current_pid();
    let next_pid = match pick_next() {
        Some(pid) => pid,
        None => {
            // No runnable tasks, mark CPU as idle
            crate::smp::percpu::get(cpu_id).set_idle(true);
            crate::debug!("Scheduler: CPU {} idle, no runnable tasks", cpu_id);
            return;
        }
    };

    // Re-enqueue next task for round-robin
    enqueue(next_pid);

    // If same task, just reset timeslice
    if current == Some(next_pid) {
        TIMESLICE_REMAINING[cpu_id].store(TIMESLICE_TICKS, Ordering::Release);
        return;
    }

    crate::debug!("Scheduler: CPU {} switching from {:?} to {}", cpu_id, current, next_pid);

    // Increment context switch counter
    crate::smp::percpu::get(cpu_id).inc_context_switches();

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
    if next.mm.page_table != 0 {
        crate::mm::switch_user_mm(next.mm.page_table);
    }

    // Set EL0 context from trap frame
    crate::arch::set_elr_el1(next.trap_frame.pc);
    crate::arch::set_spsr_el1(next.trap_frame.pstate);
    crate::arch::set_sp_el0(next.trap_frame.sp);

    // Update current PID
    set_current(next_pid);

    crate::debug!("Scheduler: CPU {} switched to task {}", cpu_id, next_pid);
}

/// Yield CPU voluntarily (on current CPU)
pub fn yield_now() {
    let cpu_id = crate::arch::current_cpu_id();
    NEED_RESCHED[cpu_id].store(true, Ordering::Release);
    schedule();
}

/// Block current process (on current CPU)
pub fn block_current() {
    if let Some(pid) = current_pid() {
        dequeue(pid);

        let mut table = super::get_process_table();
        if let Some(ref mut t) = *table {
            if let Some(task) = t.get_mut(pid) {
                task.state = ProcessState::Sleeping;
            }
        }

        let cpu_id = crate::arch::current_cpu_id();
        NEED_RESCHED[cpu_id].store(true, Ordering::Release);
        schedule();
    }
}

/// Wake a process by PID (enqueues on least loaded CPU)
pub fn wake_process(pid: Pid) {
    let mut table = super::get_process_table();
    if let Some(ref mut t) = *table {
        if let Some(task) = t.get_mut(pid) {
            if task.state == ProcessState::Sleeping {
                task.state = ProcessState::Running;

                // Find least loaded CPU
                let target_cpu = find_least_loaded_cpu();
                enqueue_on(target_cpu, pid);
            }
        }
    }
}

/// Find least loaded CPU
fn find_least_loaded_cpu() -> usize {
    let mut min_load = usize::MAX;
    let mut min_cpu = 0;

    for cpu_id in 0..crate::smp::MAX_CPUS {
        if !crate::smp::is_cpu_online(cpu_id) {
            continue;
        }

        let load = crate::smp::percpu::get(cpu_id).load.load(Ordering::Relaxed);
        if load < min_load {
            min_load = load;
            min_cpu = cpu_id;
        }
    }

    min_cpu
}

/// Load balancing across CPUs (Phase E)
///
/// Called periodically from timer tick to redistribute tasks across CPUs.
/// Simple algorithm: if a CPU has much more load than average, migrate tasks.
fn balance_load() {
    let cpu_id = crate::arch::current_cpu_id();

    // Calculate average load across all online CPUs
    let mut total_load = 0;
    let mut num_online = 0;

    for i in 0..crate::smp::MAX_CPUS {
        if crate::smp::is_cpu_online(i) {
            total_load += crate::smp::percpu::get(i).load.load(Ordering::Relaxed);
            num_online += 1;
        }
    }

    if num_online <= 1 {
        return; // Single CPU, no balancing needed
    }

    let avg_load = total_load / num_online;
    let my_load = crate::smp::percpu::get(cpu_id).load.load(Ordering::Relaxed);

    // If my load is significantly higher than average, try to migrate a task
    if my_load > avg_load + 2 {
        // Try to migrate one task to least loaded CPU
        if let Some(pid) = pick_next() {
            let target_cpu = find_least_loaded_cpu();

            if target_cpu != cpu_id {
                crate::debug!("Load balance: migrating PID {} from CPU {} to CPU {}",
                             pid, cpu_id, target_cpu);

                // Don't re-enqueue locally, send to target CPU
                enqueue_on(target_cpu, pid);
            } else {
                // Re-enqueue locally if no better target
                enqueue(pid);
            }
        }
    }
}

/// Get scheduler statistics
pub fn stats() -> SchedulerStats {
    let cpu_id = crate::arch::current_cpu_id();
    let percpu = crate::smp::percpu::get(cpu_id);

    SchedulerStats {
        cpu_id,
        current_pid: percpu.current_pid(),
        runqueue_len: percpu.runqueue_len(),
        context_switches: percpu.context_switches.load(Ordering::Relaxed),
        timer_ticks: percpu.timer_ticks.load(Ordering::Relaxed),
        timeslice_remaining: TIMESLICE_REMAINING[cpu_id].load(Ordering::Relaxed),
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Copy)]
pub struct SchedulerStats {
    pub cpu_id: usize,
    pub current_pid: Pid,
    pub runqueue_len: usize,
    pub context_switches: usize,
    pub timer_ticks: usize,
    pub timeslice_remaining: u32,
}
