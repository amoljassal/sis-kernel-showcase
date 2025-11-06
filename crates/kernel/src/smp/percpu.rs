/// Per-CPU data structures - Phase E
///
/// Each CPU has its own set of data structures to minimize cache contention and locking.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::collections::VecDeque;
use crate::process::{Pid, Task};

/// Maximum number of CPUs (must match smp/mod.rs)
const MAX_CPUS: usize = 8;

/// Per-CPU data structure
pub struct PerCpuData {
    /// CPU ID
    pub cpu_id: usize,

    /// Current running task PID (0 = idle)
    pub current_pid: AtomicUsize,

    /// Per-CPU runqueue
    pub runqueue: UnsafeCell<VecDeque<Pid>>,

    /// Number of context switches on this CPU
    pub context_switches: AtomicUsize,

    /// Number of timer ticks on this CPU
    pub timer_ticks: AtomicUsize,

    /// CPU load (tasks in runqueue + running)
    pub load: AtomicUsize,

    /// Idle flag (true if CPU is idle)
    pub is_idle: AtomicUsize,
}

impl PerCpuData {
    const fn new(cpu_id: usize) -> Self {
        Self {
            cpu_id,
            current_pid: AtomicUsize::new(0),
            runqueue: UnsafeCell::new(VecDeque::new()),
            context_switches: AtomicUsize::new(0),
            timer_ticks: AtomicUsize::new(0),
            load: AtomicUsize::new(0),
            is_idle: AtomicUsize::new(1),
        }
    }

    /// Get current running PID
    pub fn current_pid(&self) -> Pid {
        self.current_pid.load(Ordering::Acquire) as Pid
    }

    /// Set current running PID
    pub fn set_current_pid(&self, pid: Pid) {
        self.current_pid.store(pid as usize, Ordering::Release);
    }

    /// Increment context switch counter
    pub fn inc_context_switches(&self) {
        self.context_switches.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment timer tick counter
    pub fn inc_timer_ticks(&self) {
        self.timer_ticks.fetch_add(1, Ordering::Relaxed);
    }

    /// Get runqueue length (load)
    pub fn runqueue_len(&self) -> usize {
        // SAFETY: Only accessed from owning CPU or with IRQs disabled
        unsafe { (*self.runqueue.get()).len() }
    }

    /// Update load metric
    pub fn update_load(&self) {
        let load = self.runqueue_len() + if self.current_pid() != 0 { 1 } else { 0 };
        self.load.store(load, Ordering::Release);
    }

    /// Mark CPU as idle
    pub fn set_idle(&self, idle: bool) {
        self.is_idle.store(idle as usize, Ordering::Release);
    }

    /// Check if CPU is idle
    pub fn is_idle(&self) -> bool {
        self.is_idle.load(Ordering::Acquire) != 0
    }
}

// SAFETY: PerCpuData is accessed only by owning CPU or with proper synchronization
unsafe impl Sync for PerCpuData {}

/// Array of per-CPU data structures
static PER_CPU_DATA: [PerCpuData; MAX_CPUS] = [
    PerCpuData::new(0),
    PerCpuData::new(1),
    PerCpuData::new(2),
    PerCpuData::new(3),
    PerCpuData::new(4),
    PerCpuData::new(5),
    PerCpuData::new(6),
    PerCpuData::new(7),
];

/// Initialize per-CPU data for a specific CPU
pub fn init_percpu(cpu_id: usize) {
    if cpu_id >= MAX_CPUS {
        crate::warn!("PerCPU: Invalid CPU ID {}", cpu_id);
        return;
    }

    let percpu = &PER_CPU_DATA[cpu_id];

    // Initialize runqueue
    unsafe {
        *percpu.runqueue.get() = VecDeque::new();
    }

    // Reset counters
    percpu.current_pid.store(0, Ordering::Release);
    percpu.context_switches.store(0, Ordering::Release);
    percpu.timer_ticks.store(0, Ordering::Release);
    percpu.load.store(0, Ordering::Release);
    percpu.is_idle.store(1, Ordering::Release);

    crate::debug!("PerCPU: Initialized per-CPU data for CPU {}", cpu_id);
}

/// Get per-CPU data for current CPU
pub fn current() -> &'static PerCpuData {
    let cpu_id = crate::arch::current_cpu_id();
    get(cpu_id)
}

/// Get per-CPU data for a specific CPU
pub fn get(cpu_id: usize) -> &'static PerCpuData {
    if cpu_id >= MAX_CPUS {
        // Fallback to CPU 0 if invalid
        return &PER_CPU_DATA[0];
    }
    &PER_CPU_DATA[cpu_id]
}

/// Add a task to the current CPU's runqueue
pub fn enqueue_current(pid: Pid) {
    let percpu = current();

    // SAFETY: Only accessed by owning CPU
    unsafe {
        (*percpu.runqueue.get()).push_back(pid);
    }

    percpu.update_load();
}

/// Add a task to a specific CPU's runqueue
pub fn enqueue_on(cpu_id: usize, pid: Pid) {
    if cpu_id >= MAX_CPUS {
        crate::warn!("PerCPU: Invalid CPU ID {} for enqueue", cpu_id);
        return;
    }

    let percpu = get(cpu_id);

    // SAFETY: Only accessed with proper synchronization
    // TODO: Add IRQ disabling or spinlock for cross-CPU access
    unsafe {
        (*percpu.runqueue.get()).push_back(pid);
    }

    percpu.update_load();

    // TODO: Send IPI to wake up target CPU if idle
}

/// Dequeue next task from current CPU's runqueue
pub fn dequeue_current() -> Option<Pid> {
    let percpu = current();

    // SAFETY: Only accessed by owning CPU
    let pid = unsafe {
        (*percpu.runqueue.get()).pop_front()
    };

    if pid.is_some() {
        percpu.update_load();
    }

    pid
}

/// Get statistics for all CPUs
pub fn stats() -> PerCpuStats {
    let mut cpu_stats = [CpuStat::default(); MAX_CPUS];

    for i in 0..MAX_CPUS {
        let percpu = get(i);
        cpu_stats[i] = CpuStat {
            cpu_id: i,
            current_pid: percpu.current_pid(),
            runqueue_len: percpu.runqueue_len(),
            context_switches: percpu.context_switches.load(Ordering::Relaxed),
            timer_ticks: percpu.timer_ticks.load(Ordering::Relaxed),
            load: percpu.load.load(Ordering::Relaxed),
            is_idle: percpu.is_idle(),
        };
    }

    PerCpuStats { cpu_stats }
}

/// Per-CPU statistics
#[derive(Debug, Clone)]
pub struct PerCpuStats {
    pub cpu_stats: [CpuStat; MAX_CPUS],
}

/// Statistics for a single CPU
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuStat {
    pub cpu_id: usize,
    pub current_pid: Pid,
    pub runqueue_len: usize,
    pub context_switches: usize,
    pub timer_ticks: usize,
    pub load: usize,
    pub is_idle: bool,
}
