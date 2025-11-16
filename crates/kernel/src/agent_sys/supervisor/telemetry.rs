//! Telemetry aggregation and metrics collection
//!
//! The TelemetryAggregator collects metrics about agent behavior,
//! resource usage, and system health for observability and debugging.

use super::types::*;
use super::fault::Fault;
use crate::agent_sys::AgentId;
use crate::trace::metric_kv;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub use super::TELEMETRY;

/// Maximum events to keep in the ring buffer
const MAX_EVENTS: usize = 1024;

/// Per-agent metrics
#[derive(Debug, Clone)]
pub struct AgentMetrics {
    /// Agent identifier
    pub agent_id: AgentId,

    /// Number of times this agent was spawned
    pub spawn_count: u64,

    /// Number of times this agent exited
    pub exit_count: u64,

    /// Number of faults detected
    pub fault_count: u64,

    /// Total CPU time in microseconds (TODO: integrate with scheduler)
    pub cpu_time_us: u64,

    /// Current memory usage in bytes (TODO: integrate with memory manager)
    pub memory_bytes: usize,

    /// Total syscalls made by this agent
    pub syscall_count: u64,

    /// Last spawn timestamp
    pub last_spawn: Timestamp,

    /// Last exit timestamp
    pub last_exit: Timestamp,

    /// Last exit code
    pub last_exit_code: i32,

    /// Recent faults (limited to last 10)
    pub recent_faults: Vec<Fault>,
}

impl AgentMetrics {
    fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            spawn_count: 0,
            exit_count: 0,
            fault_count: 0,
            cpu_time_us: 0,
            memory_bytes: 0,
            syscall_count: 0,
            last_spawn: 0,
            last_exit: 0,
            last_exit_code: 0,
            recent_faults: Vec::new(),
        }
    }
}

/// System-wide aggregate metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Total number of agent spawns since boot
    pub total_spawns: u64,

    /// Total number of agent exits since boot
    pub total_exits: u64,

    /// Total number of faults detected since boot
    pub total_faults: u64,

    /// Total number of agent restarts
    pub total_restarts: u64,

    /// Current number of active agents
    pub active_agents: usize,
}

impl SystemMetrics {
    fn new() -> Self {
        Self {
            total_spawns: 0,
            total_exits: 0,
            total_faults: 0,
            total_restarts: 0,
            active_agents: 0,
        }
    }
}

/// Telemetry event for the ring buffer
#[derive(Debug, Clone, Copy)]
pub enum TelemetryEvent {
    /// Agent spawned
    Spawn(AgentId, Timestamp),

    /// Agent exited
    Exit(AgentId, i32, Timestamp),

    /// Fault detected
    Fault(AgentId, Fault, Timestamp),

    /// Agent restarted
    Restart(AgentId, u32, Timestamp),

    /// Policy changed
    PolicyChange(AgentId, Timestamp),
}

/// Ring buffer for recent events
struct RingBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    capacity: usize,
}

impl<T: Clone> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::new(),
            head: 0,
            capacity,
        }
    }

    fn push(&mut self, item: T) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(item);
        } else {
            self.buffer[self.head] = item;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    fn to_vec(&self) -> Vec<T> {
        if self.buffer.len() < self.capacity {
            self.buffer.clone()
        } else {
            let mut result = Vec::with_capacity(self.buffer.len());
            for i in 0..self.buffer.len() {
                let idx = (self.head + i) % self.buffer.len();
                result.push(self.buffer[idx].clone());
            }
            result
        }
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

/// Telemetry aggregator
///
/// Collects and aggregates metrics from all agents in the system.
/// Thread-safe when accessed through the global TELEMETRY mutex.
pub struct TelemetryAggregator {
    /// Per-agent metrics
    agent_metrics: BTreeMap<AgentId, AgentMetrics>,

    /// System-wide aggregates
    system_metrics: SystemMetrics,

    /// Ring buffer for recent events
    event_buffer: RingBuffer<TelemetryEvent>,

    /// Atomic counter for fast syscall tracking
    syscall_counter: AtomicU64,
}

impl TelemetryAggregator {
    /// Create a new telemetry aggregator
    pub fn new() -> Self {
        Self {
            agent_metrics: BTreeMap::new(),
            system_metrics: SystemMetrics::new(),
            event_buffer: RingBuffer::new(MAX_EVENTS),
            syscall_counter: AtomicU64::new(0),
        }
    }

    /// Record an agent spawn
    pub fn record_spawn(&mut self, agent_id: AgentId) {
        let metrics = self.agent_metrics.entry(agent_id).or_insert_with(|| AgentMetrics::new(agent_id));
        metrics.spawn_count += 1;
        metrics.last_spawn = current_timestamp();

        self.system_metrics.total_spawns += 1;
        self.system_metrics.active_agents = self.agent_metrics.len();

        let now = current_timestamp();
        self.event_buffer.push(TelemetryEvent::Spawn(agent_id, now));

        metric_kv("telemetry_spawn_recorded", 1);
    }

    /// Record an agent exit
    pub fn record_exit(&mut self, agent_id: AgentId, exit_code: i32) {
        if let Some(metrics) = self.agent_metrics.get_mut(&agent_id) {
            metrics.exit_count += 1;
            metrics.last_exit = current_timestamp();
            metrics.last_exit_code = exit_code;
        }

        self.system_metrics.total_exits += 1;

        let now = current_timestamp();
        self.event_buffer.push(TelemetryEvent::Exit(agent_id, exit_code, now));

        metric_kv("telemetry_exit_recorded", 1);
    }

    /// Record a fault
    pub fn record_fault(&mut self, agent_id: AgentId, fault: Fault) {
        let metrics = self.agent_metrics.entry(agent_id).or_insert_with(|| AgentMetrics::new(agent_id));
        metrics.fault_count += 1;

        // Keep only last 10 faults
        if metrics.recent_faults.len() >= 10 {
            metrics.recent_faults.remove(0);
        }
        metrics.recent_faults.push(fault);

        self.system_metrics.total_faults += 1;

        let now = current_timestamp();
        self.event_buffer.push(TelemetryEvent::Fault(agent_id, fault, now));

        metric_kv("telemetry_fault_recorded", 1);
    }

    /// Record an agent restart
    pub fn record_restart(&mut self, agent_id: AgentId, attempt: u32) {
        self.system_metrics.total_restarts += 1;

        let now = current_timestamp();
        self.event_buffer.push(TelemetryEvent::Restart(agent_id, attempt, now));

        metric_kv("telemetry_restart_recorded", 1);
    }

    /// Record a policy change
    pub fn record_policy_change(&mut self, agent_id: AgentId) {
        let now = current_timestamp();
        self.event_buffer.push(TelemetryEvent::PolicyChange(agent_id, now));

        metric_kv("telemetry_policy_change_recorded", 1);
    }

    /// Record a syscall
    pub fn record_syscall(&self, agent_id: AgentId) {
        self.syscall_counter.fetch_add(1, Ordering::Relaxed);
        // Note: Per-agent syscall tracking would require atomic access,
        // so we only track global count for now
    }

    /// Get metrics for a specific agent
    pub fn get_agent_metrics(&self, agent_id: AgentId) -> Option<&AgentMetrics> {
        self.agent_metrics.get(&agent_id)
    }

    /// Get system-wide metrics
    pub fn get_system_metrics(&self) -> &SystemMetrics {
        &self.system_metrics
    }

    /// Get a snapshot of all telemetry data
    pub fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            timestamp: current_timestamp(),
            agent_metrics: self.agent_metrics.clone(),
            system_metrics: self.system_metrics.clone(),
            recent_events: self.event_buffer.to_vec(),
            total_syscalls: self.syscall_counter.load(Ordering::Relaxed),
        }
    }

    /// Export to /proc format (human-readable)
    pub fn export_proc(&self, buf: &mut [u8]) -> usize {
        use core::fmt::Write;

        struct BufWriter<'a> {
            buf: &'a mut [u8],
            pos: usize,
        }

        impl<'a> Write for BufWriter<'a> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let remaining = self.buf.len() - self.pos;
                let to_write = bytes.len().min(remaining);

                self.buf[self.pos..self.pos + to_write].copy_from_slice(&bytes[..to_write]);
                self.pos += to_write;

                Ok(())
            }
        }

        let mut writer = BufWriter { buf, pos: 0 };

        let _ = writeln!(writer, "Agent Supervision Module - Telemetry Status");
        let _ = writeln!(writer, "===========================================");
        let _ = writeln!(writer, "");
        let _ = writeln!(writer, "System Metrics:");
        let _ = writeln!(writer, "  Total Spawns:    {}", self.system_metrics.total_spawns);
        let _ = writeln!(writer, "  Total Exits:     {}", self.system_metrics.total_exits);
        let _ = writeln!(writer, "  Total Faults:    {}", self.system_metrics.total_faults);
        let _ = writeln!(writer, "  Total Restarts:  {}", self.system_metrics.total_restarts);
        let _ = writeln!(writer, "  Active Agents:   {}", self.system_metrics.active_agents);
        let _ = writeln!(writer, "  Total Syscalls:  {}", self.syscall_counter.load(Ordering::Relaxed));
        let _ = writeln!(writer, "");
        let _ = writeln!(writer, "Per-Agent Metrics:");
        let _ = writeln!(writer, "  ID    Spawns Exits  Faults CPU(us)   Mem(B)");
        let _ = writeln!(writer, "  ----  ------ -----  ------ --------  -------");

        for (_, metrics) in &self.agent_metrics {
            let _ = writeln!(
                writer,
                "  {:4}  {:6} {:5}  {:6} {:8}  {:7}",
                metrics.agent_id,
                metrics.spawn_count,
                metrics.exit_count,
                metrics.fault_count,
                metrics.cpu_time_us,
                metrics.memory_bytes
            );
        }

        let _ = writeln!(writer, "");
        let _ = writeln!(writer, "Recent Events: ({} events)", self.event_buffer.len());

        writer.pos
    }
}

impl Default for TelemetryAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of telemetry data
#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    /// Snapshot timestamp
    pub timestamp: Timestamp,

    /// Per-agent metrics
    pub agent_metrics: BTreeMap<AgentId, AgentMetrics>,

    /// System metrics
    pub system_metrics: SystemMetrics,

    /// Recent events
    pub recent_events: Vec<TelemetryEvent>,

    /// Total syscalls
    pub total_syscalls: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_spawn() {
        let mut telemetry = TelemetryAggregator::new();

        telemetry.record_spawn(100);
        telemetry.record_spawn(100);

        let metrics = telemetry.get_agent_metrics(100).unwrap();
        assert_eq!(metrics.spawn_count, 2);
        assert_eq!(telemetry.get_system_metrics().total_spawns, 2);
    }

    #[test]
    fn test_record_exit() {
        let mut telemetry = TelemetryAggregator::new();

        telemetry.record_spawn(100);
        telemetry.record_exit(100, 0);

        let metrics = telemetry.get_agent_metrics(100).unwrap();
        assert_eq!(metrics.exit_count, 1);
        assert_eq!(metrics.last_exit_code, 0);
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4); // Should overwrite 1

        let vec = buffer.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec, vec![2, 3, 4]);
    }
}
