//! Enhanced Resource Monitoring for Agent Supervision
//!
//! This module provides advanced resource tracking with time-windowed aggregation,
//! quota enforcement helpers, and historical resource usage analysis.

use crate::agent_sys::AgentId;
use crate::time::get_timestamp_us;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Time window for resource aggregation (1 second in microseconds)
const RESOURCE_WINDOW_US: u64 = 1_000_000;

/// Number of historical windows to keep
const HISTORY_WINDOWS: usize = 60; // Keep 60 seconds of history

/// Resource usage snapshot for a time window
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceSnapshot {
    /// Window start timestamp
    pub timestamp: u64,

    /// CPU time used in this window (microseconds)
    pub cpu_time_us: u64,

    /// Memory usage at end of window (bytes)
    pub memory_bytes: usize,

    /// Syscall count in this window
    pub syscall_count: u64,

    /// IO operations in this window
    pub io_ops: u64,
}

/// Per-agent resource monitoring with time-windowed history
pub struct AgentResourceMonitor {
    /// Agent ID
    agent_id: AgentId,

    /// Historical resource snapshots (ring buffer)
    history: Vec<ResourceSnapshot>,

    /// Current window index
    current_window: usize,

    /// Current window start time
    window_start: u64,

    /// Accumulated stats for current window
    current_cpu_time: u64,
    current_memory: usize,
    current_syscalls: u64,
    current_io_ops: u64,

    /// Lifetime totals
    total_cpu_time: u64,
    total_syscalls: u64,
    total_io_ops: u64,
}

impl AgentResourceMonitor {
    /// Create a new resource monitor for an agent
    pub fn new(agent_id: AgentId) -> Self {
        let now = get_timestamp_us();
        Self {
            agent_id,
            history: Vec::new(),
            current_window: 0,
            window_start: now,
            current_cpu_time: 0,
            current_memory: 0,
            current_syscalls: 0,
            current_io_ops: 0,
            total_cpu_time: 0,
            total_syscalls: 0,
            total_io_ops: 0,
        }
    }

    /// Check if we need to rotate to a new window
    fn maybe_rotate_window(&mut self) {
        let now = get_timestamp_us();
        if now - self.window_start >= RESOURCE_WINDOW_US {
            // Save current window
            let snapshot = ResourceSnapshot {
                timestamp: self.window_start,
                cpu_time_us: self.current_cpu_time,
                memory_bytes: self.current_memory,
                syscall_count: self.current_syscalls,
                io_ops: self.current_io_ops,
            };

            if self.history.len() < HISTORY_WINDOWS {
                self.history.push(snapshot);
            } else {
                self.history[self.current_window] = snapshot;
            }

            // Advance to next window
            self.current_window = (self.current_window + 1) % HISTORY_WINDOWS;
            self.window_start = now;

            // Reset current window counters (except memory which is cumulative)
            self.current_cpu_time = 0;
            self.current_syscalls = 0;
            self.current_io_ops = 0;
        }
    }

    /// Record CPU time usage
    pub fn record_cpu_time(&mut self, microseconds: u64) {
        self.maybe_rotate_window();
        self.current_cpu_time += microseconds;
        self.total_cpu_time += microseconds;
    }

    /// Update memory usage
    pub fn update_memory(&mut self, bytes: usize) {
        self.maybe_rotate_window();
        self.current_memory = bytes;
    }

    /// Record a syscall
    pub fn record_syscall(&mut self) {
        self.maybe_rotate_window();
        self.current_syscalls += 1;
        self.total_syscalls += 1;
    }

    /// Record IO operation
    pub fn record_io_op(&mut self) {
        self.maybe_rotate_window();
        self.current_io_ops += 1;
        self.total_io_ops += 1;
    }

    /// Get CPU usage rate over last N seconds
    pub fn cpu_usage_rate(&self, seconds: u64) -> f32 {
        let windows = (seconds as usize).min(self.history.len());
        if windows == 0 {
            return 0.0;
        }

        let total_cpu: u64 = self.history.iter().rev().take(windows).map(|s| s.cpu_time_us).sum();
        let total_time = windows as u64 * RESOURCE_WINDOW_US;

        (total_cpu as f32 / total_time as f32) * 100.0
    }

    /// Get average syscall rate over last N seconds
    pub fn syscall_rate(&self, seconds: u64) -> f32 {
        let windows = (seconds as usize).min(self.history.len());
        if windows == 0 {
            return 0.0;
        }

        let total_syscalls: u64 = self.history.iter().rev().take(windows).map(|s| s.syscall_count).sum();
        total_syscalls as f32 / seconds as f32
    }

    /// Get current memory usage
    pub fn current_memory(&self) -> usize {
        self.current_memory
    }

    /// Get peak memory usage from history
    pub fn peak_memory(&self) -> usize {
        self.history.iter().map(|s| s.memory_bytes).max().unwrap_or(0).max(self.current_memory)
    }

    /// Get historical snapshots
    pub fn history(&self) -> &[ResourceSnapshot] {
        &self.history
    }

    /// Get lifetime totals
    pub fn lifetime_stats(&self) -> (u64, u64, u64) {
        (self.total_cpu_time, self.total_syscalls, self.total_io_ops)
    }
}

/// System-wide resource monitor
pub struct SystemResourceMonitor {
    /// Per-agent monitors
    agents: BTreeMap<AgentId, AgentResourceMonitor>,
}

impl SystemResourceMonitor {
    /// Create a new system resource monitor
    pub fn new() -> Self {
        Self {
            agents: BTreeMap::new(),
        }
    }

    /// Add an agent to monitoring
    pub fn add_agent(&mut self, agent_id: AgentId) {
        self.agents.insert(agent_id, AgentResourceMonitor::new(agent_id));
    }

    /// Remove an agent from monitoring
    pub fn remove_agent(&mut self, agent_id: AgentId) {
        self.agents.remove(&agent_id);
    }

    /// Get monitor for an agent
    pub fn get_agent(&mut self, agent_id: AgentId) -> Option<&mut AgentResourceMonitor> {
        self.agents.get_mut(&agent_id)
    }

    /// Get all agent monitors
    pub fn all_agents(&self) -> impl Iterator<Item = (&AgentId, &AgentResourceMonitor)> {
        self.agents.iter()
    }

    /// Get system-wide CPU usage percentage
    pub fn system_cpu_usage(&self, seconds: u64) -> f32 {
        let total: f32 = self.agents.values().map(|m| m.cpu_usage_rate(seconds)).sum();
        total
    }

    /// Get system-wide memory usage
    pub fn system_memory_usage(&self) -> usize {
        self.agents.values().map(|m| m.current_memory()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_monitor_creation() {
        let monitor = AgentResourceMonitor::new(100);
        assert_eq!(monitor.agent_id, 100);
        assert_eq!(monitor.current_memory(), 0);
    }

    #[test]
    fn test_syscall_recording() {
        let mut monitor = AgentResourceMonitor::new(100);

        for _ in 0..100 {
            monitor.record_syscall();
        }

        let (_, total_syscalls, _) = monitor.lifetime_stats();
        assert_eq!(total_syscalls, 100);
    }

    #[test]
    fn test_memory_tracking() {
        let mut monitor = AgentResourceMonitor::new(100);

        monitor.update_memory(1024);
        assert_eq!(monitor.current_memory(), 1024);

        monitor.update_memory(2048);
        assert_eq!(monitor.current_memory(), 2048);
    }

    #[test]
    fn test_system_monitor() {
        let mut system = SystemResourceMonitor::new();

        system.add_agent(100);
        system.add_agent(101);

        if let Some(monitor) = system.get_agent(100) {
            monitor.update_memory(4096);
        }

        assert_eq!(system.system_memory_usage(), 4096);
    }
}
