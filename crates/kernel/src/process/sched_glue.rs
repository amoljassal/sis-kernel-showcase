//! Process Scheduler Glue Layer
//!
//! Bridges the process subsystem with the CBS+EDF deterministic scheduler.
//! This module converts process tasks to scheduler jobs and manages their lifecycle
//! in the deterministic scheduler.
//!
//! # Architecture
//!
//! The glue layer provides:
//! - Process admission control using CBS utilization bounds
//! - Process-to-scheduler job conversion
//! - Lifecycle management (admission, scheduling, completion)
//! - Metrics aggregation for monitoring
//!
//! # Design Decisions
//!
//! 1. **Default Parameters**: Processes without specified timing parameters get
//!    conservative defaults (10ms WCET, 100ms period)
//! 2. **Admission Control**: Uses 85% utilization bound from CBS scheduler
//! 3. **Priority Mapping**: User priorities (nice values) map to CBS budget allocations
//! 4. **Singleton Pattern**: Single global scheduler instance for all processes
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use crate::process::sched_glue;
//!
//! // Initialize during kernel boot
//! sched_glue::init();
//!
//! // Admit a new process
//! let task = create_task(...);
//! sched_glue::admit_process(pid, &task)?;
//!
//! // Schedule next process (called on timer tick or yield)
//! if let Some(next_pid) = sched_glue::schedule() {
//!     switch_to_process(next_pid);
//! }
//!
//! // Process completed - remove from scheduler
//! sched_glue::complete_process(pid);
//! ```

use super::{Pid, Task};
use crate::deterministic::{TaskSpec, AdmissionController, DeterministicScheduler, ProcessSpec};
use spin::Mutex;

/// Maximum number of processes in scheduler
const MAX_PROCESSES: usize = 64;

/// Scheduler instance (singleton)
///
/// Protected by Mutex for thread-safe access. Initialized during kernel boot
/// via `init()` function.
static UNIFIED_SCHEDULER: Mutex<Option<DeterministicScheduler<MAX_PROCESSES>>> = Mutex::new(None);

/// Admission controller instance
///
/// Separate from scheduler for cleaner separation of concerns. Uses 85%
/// utilization bound as per CBS theory.
static ADMISSION: Mutex<AdmissionController> = Mutex::new(AdmissionController::new(850_000));

/// Scheduler metrics
#[derive(Debug, Clone, Default)]
pub struct SchedulerMetrics {
    /// Total processes admitted
    pub processes_admitted: u64,
    /// Total processes rejected
    pub processes_rejected: u64,
    /// Current active processes
    pub active_processes: u64,
    /// Total context switches
    pub context_switches: u64,
    /// Current utilization (parts per million)
    pub utilization_ppm: u32,
}

static METRICS: Mutex<SchedulerMetrics> = Mutex::new(SchedulerMetrics {
    processes_admitted: 0,
    processes_rejected: 0,
    active_processes: 0,
    context_switches: 0,
    utilization_ppm: 0,
});

/// Initialize unified scheduler
///
/// Must be called during kernel boot before any processes are created.
/// Creates a new DeterministicScheduler instance and initializes admission control.
pub fn init() {
    let mut sched = UNIFIED_SCHEDULER.lock();
    *sched = Some(DeterministicScheduler::<MAX_PROCESSES>::new(850_000));
    crate::info!("✓ Unified scheduler initialized with CBS+EDF");
    crate::info!("  - Admission bound: 85% utilization");
    crate::info!("  - Default WCET: 10ms");
    crate::info!("  - Default period: 100ms");
}

/// Admit a new process to the scheduler
///
/// Performs CBS admission control to ensure the process can be scheduled
/// without violating timing guarantees for existing processes.
///
/// # Arguments
///
/// * `pid` - Process ID to admit
/// * `task` - Task structure containing process parameters
///
/// # Returns
///
/// * `Ok(())` - Process admitted successfully
/// * `Err(&'static str)` - Admission failed (utilization bound exceeded or scheduler not initialized)
///
/// # Examples
///
/// ```rust,no_run
/// let task = create_task(pid);
/// match sched_glue::admit_process(pid, &task) {
///     Ok(()) => println!("Process {} admitted", pid),
///     Err(e) => println!("Admission failed: {}", e),
/// }
/// ```
pub fn admit_process(pid: Pid, task: &Task) -> Result<(), &'static str> {
    let mut sched = UNIFIED_SCHEDULER.lock();
    let sched = sched.as_mut().ok_or("Scheduler not initialized")?;

    // Convert task to ProcessSpec
    let spec = ProcessSpec::from_task(task)?;

    // Attempt CBS admission control (85% utilization bound)
    let mut admission = ADMISSION.lock();
    let task_spec = spec.to_task_spec();

    if !admission.try_admit(&task_spec) {
        let mut metrics = METRICS.lock();
        metrics.processes_rejected += 1;
        crate::warn!("Process {} admission rejected - utilization bound exceeded", pid);
        return Err("Process admission rejected - utilization bound exceeded");
    }

    // Admit to scheduler
    if !sched.admit_process(pid as u32, spec.clone()) {
        // Rollback admission control
        // TODO: Implement rollback in AdmissionController
        let mut metrics = METRICS.lock();
        metrics.processes_rejected += 1;
        return Err("Scheduler admission failed");
    }

    // Update metrics
    let mut metrics = METRICS.lock();
    metrics.processes_admitted += 1;
    metrics.active_processes += 1;
    let (util_ppm, _, _) = admission.stats();
    metrics.utilization_ppm = util_ppm;

    crate::debug!("Process {} admitted to scheduler (util: {}.{}%)",
                  pid, util_ppm / 10_000, (util_ppm % 10_000) / 100);

    Ok(())
}

/// Schedule next process (called on timer tick or yield)
///
/// Uses CBS+EDF to pick the process with the earliest deadline that has
/// remaining budget. This is called from the timer interrupt handler or
/// when a process explicitly yields.
///
/// # Returns
///
/// * `Some(pid)` - PID of next process to run
/// * `None` - No runnable processes
///
/// # Implementation Notes
///
/// This function updates context switch metrics and delegates to the
/// deterministic scheduler's EDF selection algorithm.
pub fn schedule() -> Option<Pid> {
    let mut sched = UNIFIED_SCHEDULER.lock();
    let sched = sched.as_mut()?;

    // Use CBS+EDF to pick next process
    let next_pid = sched.schedule_next_process()?;

    // Update metrics
    let mut metrics = METRICS.lock();
    metrics.context_switches += 1;

    Some(next_pid as Pid)
}

/// Process completed - remove from scheduler
///
/// Called when a process exits or is terminated. Removes the process from
/// the scheduler and updates utilization accounting.
///
/// # Arguments
///
/// * `pid` - Process ID to remove
pub fn complete_process(pid: Pid) {
    let mut sched = UNIFIED_SCHEDULER.lock();
    if let Some(ref mut s) = sched.as_mut() {
        s.remove_process(pid as u32);

        // Update metrics
        let mut metrics = METRICS.lock();
        metrics.active_processes = metrics.active_processes.saturating_sub(1);

        crate::debug!("Process {} removed from scheduler", pid);
    }
}

/// Process yields CPU (voluntary context switch)
///
/// Called when a process voluntarily gives up the CPU before its timeslice
/// expires. This is more efficient than waiting for preemption.
///
/// # Arguments
///
/// * `pid` - Process ID that is yielding
pub fn yield_process(pid: Pid) {
    crate::debug!("Process {} yielding CPU", pid);
    // Mark need for reschedule
    // The next timer tick will call schedule() to pick next process
}

/// Update process timing parameters
///
/// Allows dynamic adjustment of process timing parameters. Useful for
/// adaptive QoS or priority changes.
///
/// # Arguments
///
/// * `pid` - Process ID to update
/// * `wcet_ns` - New worst-case execution time
/// * `period_ns` - New period
///
/// # Returns
///
/// * `Ok(())` - Parameters updated
/// * `Err(&'static str)` - Update failed (would violate admission control)
pub fn update_process_timing(pid: Pid, wcet_ns: u64, period_ns: u64) -> Result<(), &'static str> {
    // TODO: Implement dynamic parameter updates
    // This requires:
    // 1. Remove process from admission control
    // 2. Try to re-admit with new parameters
    // 3. Update CBS server parameters if successful
    // 4. Rollback if admission fails
    Err("Dynamic timing updates not yet implemented")
}

/// Get scheduler metrics
///
/// Returns current scheduler statistics for monitoring and debugging.
///
/// # Returns
///
/// Copy of current scheduler metrics
pub fn get_metrics() -> SchedulerMetrics {
    let metrics = METRICS.lock();
    metrics.clone()
}

/// Print scheduler status (for debugging)
///
/// Outputs detailed scheduler state to kernel log.
pub fn print_status() {
    let metrics = get_metrics();

    crate::info!("=== Unified Scheduler Status ===");
    crate::info!("Admitted: {}", metrics.processes_admitted);
    crate::info!("Rejected: {}", metrics.processes_rejected);
    crate::info!("Active: {}", metrics.active_processes);
    crate::info!("Context switches: {}", metrics.context_switches);
    crate::info!("Utilization: {}.{}%",
                 metrics.utilization_ppm / 10_000,
                 (metrics.utilization_ppm % 10_000) / 100);

    let admission = ADMISSION.lock();
    let (util_ppm, accepted, rejected) = admission.stats();
    crate::info!("Admission controller: util={}.{}%, accepted={}, rejected={}",
                 util_ppm / 10_000,
                 (util_ppm % 10_000) / 100,
                 accepted,
                 rejected);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_init() {
        init();
        let sched = UNIFIED_SCHEDULER.lock();
        assert!(sched.is_some());
    }

    #[test]
    fn test_metrics_initialization() {
        let metrics = get_metrics();
        assert_eq!(metrics.processes_admitted, 0);
        assert_eq!(metrics.active_processes, 0);
    }
}
