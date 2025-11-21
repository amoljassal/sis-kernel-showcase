//! Agent lifecycle management
//!
//! The AgentSupervisor tracks all active agents and manages their lifecycle
//! from spawn to exit, including fault handling and automatic recovery.

use super::types::*;
use super::fault::{Fault, FaultAction};
use super::telemetry::TELEMETRY;
use crate::agent_sys::AgentId;
use crate::process::Pid;
use crate::trace::metric_kv;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Agent Supervisor - manages agent lifecycle
///
/// The supervisor maintains a registry of all active agents and provides
/// hooks for the process manager to call during agent lifecycle events.
///
/// # Thread Safety
///
/// AgentSupervisor is designed to be accessed through a global Mutex.
/// All methods assume they are called with the lock held.
pub struct AgentSupervisor {
    /// Registry of all agents with metadata (indexed by AgentId)
    registry: BTreeMap<AgentId, AgentMetadata>,

    /// Mapping from PID to AgentId for reverse lookups
    pid_to_agent: BTreeMap<Pid, AgentId>,

    /// Lifecycle event listeners
    listeners: Vec<Box<dyn LifecycleListener>>,

    /// Next agent ID to allocate (for dynamic agents)
    next_agent_id: AgentId,
}

impl AgentSupervisor {
    /// Create a new agent supervisor
    pub fn new() -> Self {
        Self {
            registry: BTreeMap::new(),
            pid_to_agent: BTreeMap::new(),
            listeners: Vec::new(),
            next_agent_id: 1000, // Start dynamic IDs at 1000
        }
    }

    /// Register a lifecycle event listener
    pub fn add_listener(&mut self, listener: Box<dyn LifecycleListener>) {
        self.listeners.push(listener);
    }

    /// Notify all listeners of an event
    fn notify_listeners(&mut self, event: LifecycleEvent) {
        for listener in &mut self.listeners {
            listener.on_event(event);
        }
    }

    /// Called by process manager when an agent spawns
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID of the new agent
    /// * `spec` - Agent specification with capabilities and policies
    ///
    /// # Returns
    ///
    /// The assigned agent ID
    pub fn on_agent_spawn(&mut self, pid: Pid, spec: AgentSpec) -> AgentId {
        let agent_id = spec.agent_id;

        // Create metadata
        let mut metadata = AgentMetadata::new(agent_id, pid, spec.name.clone());
        metadata.capabilities = spec.capabilities;
        metadata.scope = spec.scope;
        metadata.auto_restart = spec.auto_restart;
        metadata.max_restarts = spec.max_restarts;

        // Insert into registry
        self.registry.insert(agent_id, metadata);
        self.pid_to_agent.insert(pid, agent_id);

        // Notify listeners
        self.notify_listeners(LifecycleEvent::Spawned(agent_id));

        // Record telemetry
        if let Some(telemetry) = TELEMETRY.lock().as_mut() {
            telemetry.record_spawn(agent_id);
        }

        metric_kv("agent_spawn_total", 1);

        agent_id
    }

    /// Called by process manager when an agent exits
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID that exited
    /// * `exit_code` - Exit status code
    ///
    /// # Recovery
    ///
    /// If the agent is configured for auto-restart and hasn't exceeded
    /// the maximum restart count, a restart will be scheduled.
    pub fn on_agent_exit(&mut self, pid: Pid, exit_code: i32) {
        // Find agent by PID
        let agent_id = match self.pid_to_agent.remove(&pid) {
            Some(id) => id,
            None => return, // Not an agent process
        };

        // Get metadata
        let metadata = match self.registry.remove(&agent_id) {
            Some(meta) => meta,
            None => return,
        };

        // Notify listeners
        if exit_code == 0 {
            self.notify_listeners(LifecycleEvent::Exited(agent_id, exit_code));
        } else {
            self.notify_listeners(LifecycleEvent::Crashed(agent_id, exit_code as u32));
        }

        // Record telemetry
        if let Some(telemetry) = TELEMETRY.lock().as_mut() {
            telemetry.record_exit(agent_id, exit_code);
        }

        metric_kv("agent_exit_total", 1);

        // Check if recovery needed
        if metadata.auto_restart && exit_code != 0 {
            if !metadata.has_exceeded_restarts() {
                self.schedule_restart(agent_id, metadata);
            } else {
                crate::uart::print_str("[ASM] Agent ");
                crate::uart::print_u32(agent_id);
                crate::uart::print_str(" exceeded max restarts\n");
                metric_kv("agent_restart_max_exceeded", 1);
            }
        }
    }

    /// Called by fault detector when an agent violates policy
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent that faulted
    /// * `fault` - Type of fault detected
    ///
    /// # Returns
    ///
    /// The action taken in response to the fault
    pub fn on_fault(&mut self, agent_id: AgentId, fault: Fault) -> FaultAction {
        // Get recovery policy from fault detector
        let action = super::fault::FAULT_DETECTOR
            .lock()
            .as_ref()
            .map(|fd| fd.get_recovery_policy().action_for(&fault))
            .unwrap_or(FaultAction::Alert);

        // Record telemetry
        if let Some(telemetry) = TELEMETRY.lock().as_mut() {
            telemetry.record_fault(agent_id, fault.clone());
        }

        // Log the fault
        crate::uart::print_str("[ASM] Agent ");
        crate::uart::print_u32(agent_id);
        crate::uart::print_str(" fault detected: ");
        crate::uart::print_str(fault.description());
        crate::uart::print_str("\n");

        metric_kv("agent_fault_total", 1);

        // Execute action
        match action {
            FaultAction::Kill => {
                if let Some(metadata) = self.registry.get(&agent_id) {
                    // Kill via process manager
                    let _ = crate::process::signal::send_signal(metadata.pid, crate::process::signal::Signal::SIGKILL);
                }
            }
            FaultAction::Throttle => {
                // TODO: Implement CPU throttling via scheduler
                crate::uart::print_str("[ASM] Throttling not yet implemented\n");
            }
            FaultAction::Restart => {
                if let Some(metadata) = self.registry.get(&agent_id).cloned() {
                    if let Some(metadata) = self.registry.remove(&agent_id) {
                        self.schedule_restart(agent_id, metadata);
                    }
                }
            }
            FaultAction::Alert => {
                // Just log, already done above
            }
        }

        action
    }

    /// Schedule an agent restart
    ///
    /// This creates a new process with the same configuration but
    /// incremented restart count.
    fn schedule_restart(&mut self, old_id: AgentId, mut metadata: AgentMetadata) {
        metadata.restart_count += 1;

        crate::uart::print_str("[ASM] Restarting agent ");
        crate::uart::print_u32(old_id);
        crate::uart::print_str(" (attempt ");
        crate::uart::print_u32(metadata.restart_count);
        crate::uart::print_str("/");
        crate::uart::print_u32(metadata.max_restarts);
        crate::uart::print_str(")\n");

        // TODO: Integrate with process spawning
        // For now, just log the restart intent
        // In a full implementation, this would call process::spawn()

        self.notify_listeners(LifecycleEvent::Restarted(old_id, metadata.restart_count));
        metric_kv("agent_restart_total", 1);
    }

    /// Get agent metadata by ID
    pub fn get_agent(&self, agent_id: AgentId) -> Option<&AgentMetadata> {
        self.registry.get(&agent_id)
    }

    /// Get agent metadata by PID
    pub fn get_agent_by_pid(&self, pid: Pid) -> Option<&AgentMetadata> {
        self.pid_to_agent
            .get(&pid)
            .and_then(|agent_id| self.registry.get(agent_id))
    }

    /// Get mutable agent metadata by ID
    pub fn get_agent_mut(&mut self, agent_id: AgentId) -> Option<&mut AgentMetadata> {
        self.registry.get_mut(&agent_id)
    }

    /// List all active agents
    pub fn list_agents(&self) -> Vec<&AgentMetadata> {
        self.registry.values().collect()
    }

    /// Get count of active agents
    pub fn agent_count(&self) -> usize {
        self.registry.len()
    }

    /// Allocate a new dynamic agent ID
    pub fn allocate_agent_id(&mut self) -> AgentId {
        let id = self.next_agent_id;
        self.next_agent_id += 1;
        id
    }

    /// Check if an agent exists
    pub fn has_agent(&self, agent_id: AgentId) -> bool {
        self.registry.contains_key(&agent_id)
    }

    /// Update agent last activity timestamp
    pub fn touch_agent(&mut self, agent_id: AgentId) {
        if let Some(metadata) = self.registry.get_mut(&agent_id) {
            metadata.touch();
        }
    }
}

impl Default for AgentSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_agent_spawn() {
        let mut supervisor = AgentSupervisor::new();

        let spec = AgentSpec::new(100, "test_agent".to_string());
        let agent_id = supervisor.on_agent_spawn(1, spec);

        assert_eq!(agent_id, 100);
        assert_eq!(supervisor.agent_count(), 1);
        assert!(supervisor.has_agent(100));
    }

    #[test]
    fn test_agent_exit() {
        let mut supervisor = AgentSupervisor::new();

        let spec = AgentSpec::new(100, "test_agent".to_string());
        supervisor.on_agent_spawn(1, spec);

        supervisor.on_agent_exit(1, 0);

        assert_eq!(supervisor.agent_count(), 0);
        assert!(!supervisor.has_agent(100));
    }

    #[test]
    fn test_pid_to_agent_lookup() {
        let mut supervisor = AgentSupervisor::new();

        let spec = AgentSpec::new(100, "test_agent".to_string());
        supervisor.on_agent_spawn(42, spec);

        let metadata = supervisor.get_agent_by_pid(42);
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().agent_id, 100);
        assert_eq!(metadata.unwrap().pid, 42);
    }
}
