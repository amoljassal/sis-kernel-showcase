//! Performance Profiling for Agent Operations
//!
//! This module provides lightweight performance profiling hooks to track
//! agent operation latencies, bottlenecks, and performance characteristics.

use crate::agent_sys::AgentId;
use crate::time::get_timestamp_us;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// Maximum number of profile samples to keep per operation type
const MAX_SAMPLES: usize = 100;

/// A performance measurement sample
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProfileSample {
    /// Timestamp when operation started
    pub start_time: u64,

    /// Duration in microseconds
    pub duration_us: u64,

    /// Was this operation successful
    pub success: bool,
}

/// Statistics for a profiled operation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProfileStats {
    /// Operation name
    pub operation: String,

    /// Number of samples
    pub sample_count: usize,

    /// Minimum duration (microseconds)
    pub min_duration_us: u64,

    /// Maximum duration (microseconds)
    pub max_duration_us: u64,

    /// Average duration (microseconds)
    pub avg_duration_us: u64,

    /// Median duration (microseconds)
    pub median_duration_us: u64,

    /// 95th percentile duration (microseconds)
    pub p95_duration_us: u64,

    /// 99th percentile duration (microseconds)
    pub p99_duration_us: u64,

    /// Success rate (0.0 - 1.0)
    pub success_rate: f32,
}

/// Operation profiler
struct OperationProfiler {
    /// Operation name
    operation: String,

    /// Sample ring buffer
    samples: Vec<ProfileSample>,

    /// Current insertion index
    next_index: usize,

    /// Total samples recorded (including overwritten ones)
    total_samples: u64,
}

impl OperationProfiler {
    fn new(operation: String) -> Self {
        Self {
            operation,
            samples: Vec::new(),
            next_index: 0,
            total_samples: 0,
        }
    }

    fn record_sample(&mut self, sample: ProfileSample) {
        if self.samples.len() < MAX_SAMPLES {
            self.samples.push(sample);
        } else {
            self.samples[self.next_index] = sample;
            self.next_index = (self.next_index + 1) % MAX_SAMPLES;
        }
        self.total_samples += 1;
    }

    fn compute_stats(&self) -> ProfileStats {
        if self.samples.is_empty() {
            return ProfileStats {
                operation: self.operation.clone(),
                sample_count: 0,
                min_duration_us: 0,
                max_duration_us: 0,
                avg_duration_us: 0,
                median_duration_us: 0,
                p95_duration_us: 0,
                p99_duration_us: 0,
                success_rate: 0.0,
            };
        }

        // Sort durations for percentile calculation
        let mut durations: Vec<u64> = self.samples.iter().map(|s| s.duration_us).collect();
        durations.sort_unstable();

        let min = durations[0];
        let max = durations[durations.len() - 1];
        let sum: u64 = durations.iter().sum();
        let avg = sum / durations.len() as u64;
        let median = durations[durations.len() / 2];
        let p95_index = (durations.len() as f32 * 0.95) as usize;
        let p99_index = (durations.len() as f32 * 0.99) as usize;
        let p95 = durations[p95_index.min(durations.len() - 1)];
        let p99 = durations[p99_index.min(durations.len() - 1)];

        let successes = self.samples.iter().filter(|s| s.success).count();
        let success_rate = successes as f32 / self.samples.len() as f32;

        ProfileStats {
            operation: self.operation.clone(),
            sample_count: self.samples.len(),
            min_duration_us: min,
            max_duration_us: max,
            avg_duration_us: avg,
            median_duration_us: median,
            p95_duration_us: p95,
            p99_duration_us: p99,
            success_rate,
        }
    }
}

/// Per-agent profiler
pub struct AgentProfiler {
    /// Agent ID
    agent_id: AgentId,

    /// Profilers by operation name
    operations: BTreeMap<String, OperationProfiler>,
}

impl AgentProfiler {
    /// Create a new agent profiler
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            operations: BTreeMap::new(),
        }
    }

    /// Start profiling an operation (returns start timestamp)
    pub fn start_operation(&self, _operation: &str) -> u64 {
        get_timestamp_us()
    }

    /// End profiling an operation
    pub fn end_operation(&mut self, operation: &str, start_time: u64, success: bool) {
        let now = get_timestamp_us();
        let duration = now.saturating_sub(start_time);

        let sample = ProfileSample {
            start_time,
            duration_us: duration,
            success,
        };

        self.operations
            .entry(operation.to_string())
            .or_insert_with(|| OperationProfiler::new(operation.to_string()))
            .record_sample(sample);
    }

    /// Get statistics for all operations
    pub fn get_all_stats(&self) -> Vec<ProfileStats> {
        self.operations.values().map(|p| p.compute_stats()).collect()
    }

    /// Get statistics for a specific operation
    pub fn get_stats(&self, operation: &str) -> Option<ProfileStats> {
        self.operations.get(operation).map(|p| p.compute_stats())
    }

    /// Get list of profiled operations
    pub fn operations(&self) -> Vec<String> {
        self.operations.keys().cloned().collect()
    }
}

/// System-wide profiler
pub struct SystemProfiler {
    /// Per-agent profilers
    agents: BTreeMap<AgentId, AgentProfiler>,
}

impl SystemProfiler {
    /// Create a new system profiler
    pub fn new() -> Self {
        Self {
            agents: BTreeMap::new(),
        }
    }

    /// Add an agent to profiling
    pub fn add_agent(&mut self, agent_id: AgentId) {
        self.agents.insert(agent_id, AgentProfiler::new(agent_id));
    }

    /// Remove an agent from profiling
    pub fn remove_agent(&mut self, agent_id: AgentId) {
        self.agents.remove(&agent_id);
    }

    /// Get profiler for an agent
    pub fn get_agent(&mut self, agent_id: AgentId) -> Option<&mut AgentProfiler> {
        self.agents.get_mut(&agent_id)
    }

    /// Get all agent profilers
    pub fn all_agents(&self) -> impl Iterator<Item = (&AgentId, &AgentProfiler)> {
        self.agents.iter()
    }

    /// Get aggregated stats across all agents for an operation
    pub fn aggregate_stats(&self, operation: &str) -> Option<ProfileStats> {
        let mut all_samples = Vec::new();

        for profiler in self.agents.values() {
            if let Some(stats) = profiler.get_stats(operation) {
                // Reconstruct samples from stats (approximate)
                for _ in 0..stats.sample_count {
                    all_samples.push(stats.avg_duration_us);
                }
            }
        }

        if all_samples.is_empty() {
            return None;
        }

        all_samples.sort_unstable();
        let sum: u64 = all_samples.iter().sum();

        Some(ProfileStats {
            operation: operation.to_string(),
            sample_count: all_samples.len(),
            min_duration_us: all_samples[0],
            max_duration_us: all_samples[all_samples.len() - 1],
            avg_duration_us: sum / all_samples.len() as u64,
            median_duration_us: all_samples[all_samples.len() / 2],
            p95_duration_us: all_samples[(all_samples.len() as f32 * 0.95) as usize],
            p99_duration_us: all_samples[(all_samples.len() as f32 * 0.99) as usize],
            success_rate: 1.0, // Cannot reconstruct from aggregated stats
        })
    }
}

/// RAII profiler guard for automatic timing
pub struct ProfileGuard<'a> {
    profiler: &'a mut AgentProfiler,
    operation: String,
    start_time: u64,
    success: bool,
}

impl<'a> ProfileGuard<'a> {
    /// Create a new profile guard
    pub fn new(profiler: &'a mut AgentProfiler, operation: String) -> Self {
        let start_time = get_timestamp_us();
        Self {
            profiler,
            operation,
            start_time,
            success: true,
        }
    }

    /// Mark operation as failed
    pub fn set_failed(&mut self) {
        self.success = false;
    }

    /// Mark operation as successful (default)
    pub fn set_success(&mut self) {
        self.success = true;
    }
}

impl<'a> Drop for ProfileGuard<'a> {
    fn drop(&mut self) {
        self.profiler.end_operation(&self.operation, self.start_time, self.success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = AgentProfiler::new(100);
        assert_eq!(profiler.agent_id, 100);
        assert_eq!(profiler.operations().len(), 0);
    }

    #[test]
    fn test_operation_profiling() {
        let mut profiler = AgentProfiler::new(100);

        for _ in 0..10 {
            let start = profiler.start_operation("test_op");
            // Simulate some work
            profiler.end_operation("test_op", start, true);
        }

        let stats = profiler.get_stats("test_op").unwrap();
        assert_eq!(stats.sample_count, 10);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[test]
    fn test_success_rate() {
        let mut profiler = AgentProfiler::new(100);

        // 7 successes, 3 failures
        for i in 0..10 {
            let start = profiler.start_operation("mixed_op");
            profiler.end_operation("mixed_op", start, i < 7);
        }

        let stats = profiler.get_stats("mixed_op").unwrap();
        assert_eq!(stats.success_rate, 0.7);
    }

    #[test]
    fn test_system_profiler() {
        let mut system = SystemProfiler::new();

        system.add_agent(100);
        system.add_agent(101);

        if let Some(profiler) = system.get_agent(100) {
            let start = profiler.start_operation("op1");
            profiler.end_operation("op1", start, true);
        }

        assert!(system.aggregate_stats("op1").is_some());
    }
}
