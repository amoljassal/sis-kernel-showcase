//! Core types for the Agent Supervision Module
//!
//! This module defines the fundamental data structures used throughout ASM.

use crate::agent_sys::AgentId;
use crate::process::Pid;
use crate::security::agent_policy::{Capability, Scope};
use alloc::vec::Vec;
use alloc::string::String;

/// Timestamp in microseconds since kernel boot
pub type Timestamp = u64;

/// Get current timestamp in microseconds
#[inline]
pub fn current_timestamp() -> Timestamp {
    crate::time::get_timestamp_us()
}

/// Agent metadata tracked by the supervisor
#[derive(Debug, Clone)]
pub struct AgentMetadata {
    /// Agent identifier
    pub agent_id: AgentId,

    /// Process ID associated with this agent
    pub pid: Pid,

    /// Agent name
    pub name: String,

    /// Current capabilities
    pub capabilities: Vec<Capability>,

    /// Scope restrictions
    pub scope: Scope,

    /// Whether this agent should auto-restart on failure
    pub auto_restart: bool,

    /// Maximum number of restart attempts
    pub max_restarts: u32,

    /// Current restart count
    pub restart_count: u32,

    /// Timestamp when agent was spawned
    pub spawn_time: Timestamp,

    /// Timestamp of last activity
    pub last_activity: Timestamp,

    /// Whether the agent is currently active
    pub active: bool,
}

impl AgentMetadata {
    /// Create new agent metadata
    pub fn new(agent_id: AgentId, pid: Pid, name: String) -> Self {
        let now = current_timestamp();
        Self {
            agent_id,
            pid,
            name,
            capabilities: Vec::new(),
            scope: Scope::UNRESTRICTED,
            auto_restart: false,
            max_restarts: 3,
            restart_count: 0,
            spawn_time: now,
            last_activity: now,
            active: true,
        }
    }

    /// Create metadata with restart count incremented
    pub fn with_restart_count(&self, count: u32) -> Self {
        let mut meta = self.clone();
        meta.restart_count = count;
        meta.spawn_time = current_timestamp();
        meta
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = current_timestamp();
    }

    /// Check if agent has exceeded max restarts
    pub fn has_exceeded_restarts(&self) -> bool {
        self.restart_count >= self.max_restarts
    }

    /// Get agent uptime in microseconds
    pub fn uptime(&self) -> u64 {
        current_timestamp().saturating_sub(self.spawn_time)
    }
}

/// Agent lifecycle event
#[derive(Debug, Clone, Copy)]
pub enum LifecycleEvent {
    /// Agent spawned
    Spawned(AgentId),

    /// Agent exited normally
    Exited(AgentId, i32),

    /// Agent crashed (signal)
    Crashed(AgentId, u32),

    /// Agent policy changed
    PolicyChanged(AgentId),

    /// Agent restarted
    Restarted(AgentId, u32),
}

/// Lifecycle event listener trait
pub trait LifecycleListener: Send + Sync {
    /// Called when a lifecycle event occurs
    fn on_event(&mut self, event: LifecycleEvent);
}

/// Agent spawn specification
#[derive(Debug, Clone)]
pub struct AgentSpec {
    /// Agent identifier
    pub agent_id: AgentId,

    /// Agent name
    pub name: String,

    /// Initial capabilities
    pub capabilities: Vec<Capability>,

    /// Scope restrictions
    pub scope: Scope,

    /// Auto-restart configuration
    pub auto_restart: bool,

    /// Max restart attempts
    pub max_restarts: u32,
}

impl AgentSpec {
    /// Create a new agent specification
    pub fn new(agent_id: AgentId, name: String) -> Self {
        Self {
            agent_id,
            name,
            capabilities: Vec::new(),
            scope: Scope::UNRESTRICTED,
            auto_restart: false,
            max_restarts: 3,
        }
    }

    /// Add a capability to the spec
    pub fn with_capability(mut self, cap: Capability) -> Self {
        self.capabilities.push(cap);
        self
    }

    /// Set scope restrictions
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// Enable auto-restart
    pub fn with_auto_restart(mut self, max_restarts: u32) -> Self {
        self.auto_restart = true;
        self.max_restarts = max_restarts;
        self
    }
}
