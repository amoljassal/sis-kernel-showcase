//! Autonomy Metrics Collection
//!
//! Tracks autonomous AI interventions for comparison and observability.
//! Enables measurement of AI impact on system performance.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Metrics tracking autonomous AI interventions
#[derive(Copy, Clone, Debug)]
pub struct AutonomyMetrics {
    // Memory management interventions
    pub proactive_compactions: u32,
    pub preemptive_oom_preventions: u32,
    pub memory_pressure_predictions: u32,

    // Scheduling interventions
    pub deadline_adjustments: u32,
    pub priority_boosts: u32,
    pub workload_rebalancing: u32,

    // Learning interventions
    pub policy_updates: u32,
    pub exploration_actions: u32,
    pub exploitation_actions: u32,

    // Overall metrics
    pub total_interventions: u32,
    pub intervention_success_count: u32,
}

impl AutonomyMetrics {
    pub const fn new() -> Self {
        Self {
            proactive_compactions: 0,
            preemptive_oom_preventions: 0,
            memory_pressure_predictions: 0,
            deadline_adjustments: 0,
            priority_boosts: 0,
            workload_rebalancing: 0,
            policy_updates: 0,
            exploration_actions: 0,
            exploitation_actions: 0,
            total_interventions: 0,
            intervention_success_count: 0,
        }
    }

    /// Calculate intervention success rate as percentage (0-100)
    pub fn success_rate_pct(&self) -> u8 {
        if self.total_interventions == 0 {
            return 0;
        }
        ((self.intervention_success_count as u64 * 100) / self.total_interventions as u64).min(100) as u8
    }
}

/// Global autonomy metrics state (atomic for thread-safety)
pub struct AutonomyMetricsState {
    // Memory management
    pub proactive_compactions: AtomicU32,
    pub preemptive_oom_preventions: AtomicU32,
    pub memory_pressure_predictions: AtomicU32,

    // Scheduling
    pub deadline_adjustments: AtomicU32,
    pub priority_boosts: AtomicU32,
    pub workload_rebalancing: AtomicU32,

    // Learning
    pub policy_updates: AtomicU32,
    pub exploration_actions: AtomicU32,
    pub exploitation_actions: AtomicU32,

    // Overall
    pub total_interventions: AtomicU32,
    pub intervention_success_count: AtomicU32,

    // Latency tracking
    pub intervention_latency_sum_ns: AtomicU64,
    pub intervention_latency_count: AtomicU32,
}

impl AutonomyMetricsState {
    pub const fn new() -> Self {
        Self {
            proactive_compactions: AtomicU32::new(0),
            preemptive_oom_preventions: AtomicU32::new(0),
            memory_pressure_predictions: AtomicU32::new(0),
            deadline_adjustments: AtomicU32::new(0),
            priority_boosts: AtomicU32::new(0),
            workload_rebalancing: AtomicU32::new(0),
            policy_updates: AtomicU32::new(0),
            exploration_actions: AtomicU32::new(0),
            exploitation_actions: AtomicU32::new(0),
            total_interventions: AtomicU32::new(0),
            intervention_success_count: AtomicU32::new(0),
            intervention_latency_sum_ns: AtomicU64::new(0),
            intervention_latency_count: AtomicU32::new(0),
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.proactive_compactions.store(0, Ordering::Relaxed);
        self.preemptive_oom_preventions.store(0, Ordering::Relaxed);
        self.memory_pressure_predictions.store(0, Ordering::Relaxed);
        self.deadline_adjustments.store(0, Ordering::Relaxed);
        self.priority_boosts.store(0, Ordering::Relaxed);
        self.workload_rebalancing.store(0, Ordering::Relaxed);
        self.policy_updates.store(0, Ordering::Relaxed);
        self.exploration_actions.store(0, Ordering::Relaxed);
        self.exploitation_actions.store(0, Ordering::Relaxed);
        self.total_interventions.store(0, Ordering::Relaxed);
        self.intervention_success_count.store(0, Ordering::Relaxed);
        self.intervention_latency_sum_ns.store(0, Ordering::Relaxed);
        self.intervention_latency_count.store(0, Ordering::Relaxed);
    }

    /// Take a snapshot of current metrics
    pub fn snapshot(&self) -> AutonomyMetrics {
        AutonomyMetrics {
            proactive_compactions: self.proactive_compactions.load(Ordering::Relaxed),
            preemptive_oom_preventions: self.preemptive_oom_preventions.load(Ordering::Relaxed),
            memory_pressure_predictions: self.memory_pressure_predictions.load(Ordering::Relaxed),
            deadline_adjustments: self.deadline_adjustments.load(Ordering::Relaxed),
            priority_boosts: self.priority_boosts.load(Ordering::Relaxed),
            workload_rebalancing: self.workload_rebalancing.load(Ordering::Relaxed),
            policy_updates: self.policy_updates.load(Ordering::Relaxed),
            exploration_actions: self.exploration_actions.load(Ordering::Relaxed),
            exploitation_actions: self.exploitation_actions.load(Ordering::Relaxed),
            total_interventions: self.total_interventions.load(Ordering::Relaxed),
            intervention_success_count: self.intervention_success_count.load(Ordering::Relaxed),
        }
    }

    /// Record a proactive compaction
    pub fn record_proactive_compaction(&self) {
        self.proactive_compactions.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a preemptive OOM prevention
    pub fn record_oom_prevention(&self) {
        self.preemptive_oom_preventions.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a memory pressure prediction
    pub fn record_memory_prediction(&self) {
        self.memory_pressure_predictions.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a deadline adjustment
    pub fn record_deadline_adjustment(&self) {
        self.deadline_adjustments.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a priority boost
    pub fn record_priority_boost(&self) {
        self.priority_boosts.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record workload rebalancing
    pub fn record_workload_rebalancing(&self) {
        self.workload_rebalancing.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a policy update
    pub fn record_policy_update(&self) {
        self.policy_updates.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an exploration action
    pub fn record_exploration(&self) {
        self.exploration_actions.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an exploitation action
    pub fn record_exploitation(&self) {
        self.exploitation_actions.fetch_add(1, Ordering::Relaxed);
        self.total_interventions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record intervention success
    pub fn record_success(&self) {
        self.intervention_success_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record intervention latency
    pub fn record_latency(&self, latency_ns: u64) {
        self.intervention_latency_sum_ns.fetch_add(latency_ns, Ordering::Relaxed);
        self.intervention_latency_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get average intervention latency in nanoseconds
    pub fn avg_latency_ns(&self) -> u64 {
        let count = self.intervention_latency_count.load(Ordering::Relaxed);
        if count == 0 {
            return 0;
        }
        self.intervention_latency_sum_ns.load(Ordering::Relaxed) / count as u64
    }
}

/// Global autonomy metrics instance
pub static AUTONOMY_METRICS: AutonomyMetricsState = AutonomyMetricsState::new();
