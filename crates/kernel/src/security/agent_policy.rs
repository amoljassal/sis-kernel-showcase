//! AgentSys Policy Engine: Capability-based access control
//!
//! This module implements a least-privilege security model where each agent
//! is granted explicit capabilities with optional scope restrictions.
//!
//! Example:
//! ```
//! Agent "FileManager" {
//!   capabilities: [FsBasic]
//!   scope: { path_prefix: "/tmp/files/" }
//! }
//! ```

use crate::trace::metric_kv;

/// Agent identifier (unique per agent instance)
pub type AgentId = u32;

/// Reserved agent IDs
pub const AGENT_ID_SYSTEM: AgentId = 0;
pub const AGENT_ID_AGENTD: AgentId = 1;
pub const AGENT_ID_TEST: AgentId = 0xFFFF;

/// Capability enum (what an agent can do)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Capability {
    /// File operations: list, read, write, stat, create, delete
    FsBasic,

    /// Audio control: play, stop, volume
    AudioControl,

    /// Document operations: new, edit, save
    DocBasic,

    /// Screen capture
    Capture,

    /// Screenshot generation
    Screenshot,

    /// Agent administration (register new agents, modify policies)
    Admin,
}

/// Scope restrictions for a capability
#[derive(Copy, Clone, Debug)]
pub struct Scope {
    /// Path prefix pattern (e.g., "/tmp/docs/")
    /// None = unrestricted
    pub path_prefix: Option<&'static str>,

    /// Maximum file size (bytes), None = unlimited
    pub max_file_size: Option<usize>,

    /// Maximum operations per second, None = unlimited
    pub max_ops_per_sec: Option<u16>,
}

impl Scope {
    pub const UNRESTRICTED: Self = Scope {
        path_prefix: None,
        max_file_size: None,
        max_ops_per_sec: None,
    };

    pub const fn with_path(prefix: &'static str) -> Self {
        Scope {
            path_prefix: Some(prefix),
            max_file_size: None,
            max_ops_per_sec: None,
        }
    }
}

/// Agent registration entry
#[derive(Copy, Clone, Debug)]
pub struct AgentToken {
    pub agent_id: AgentId,
    pub name: &'static str,
    pub capabilities: &'static [Capability],
    pub scope: Scope,
    pub enabled: bool,
}

/// Policy decision result
#[derive(Debug, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny { reason: &'static str },
    RateLimit { retry_after_ms: u32 },
}

/// Policy engine (static configuration for Phase 1)
pub struct PolicyEngine {
    agents: &'static [AgentToken],
}

impl PolicyEngine {
    /// Create default policy engine with built-in agents
    pub fn new_default() -> Self {
        PolicyEngine {
            agents: DEFAULT_AGENTS,
        }
    }

    /// Check if operation is allowed
    pub fn check(
        &self,
        agent_id: AgentId,
        capability: Capability,
        resource: &Resource,
    ) -> PolicyDecision {
        // Find agent
        let agent = match self.agents.iter().find(|a| a.agent_id == agent_id) {
            Some(a) if a.enabled => a,
            Some(_) => return PolicyDecision::Deny { reason: "Agent disabled" },
            None => return PolicyDecision::Deny { reason: "Agent not registered" },
        };

        // Check capability granted
        if !agent.capabilities.contains(&capability) {
            metric_kv("agentsys_policy_denies", 1);
            return PolicyDecision::Deny { reason: "Capability not granted" };
        }

        // Check scope restrictions
        match resource {
            Resource::FilePath(path) => {
                if let Some(prefix) = agent.scope.path_prefix {
                    if !path.starts_with(prefix) {
                        metric_kv("agentsys_scope_violations", 1);
                        return PolicyDecision::Deny { reason: "Path outside allowed scope" };
                    }
                }
            }
            Resource::FileSize(size) => {
                if let Some(max_size) = agent.scope.max_file_size {
                    if *size > max_size {
                        return PolicyDecision::Deny { reason: "File size exceeds limit" };
                    }
                }
            }
            _ => {}
        }

        metric_kv("agentsys_policy_allows", 1);
        PolicyDecision::Allow
    }

    /// Get agent info by ID
    pub fn get_agent(&self, agent_id: AgentId) -> Option<&AgentToken> {
        self.agents.iter().find(|a| a.agent_id == agent_id)
    }

    /// List all registered agents
    pub fn list_agents(&self) -> &[AgentToken] {
        self.agents
    }
}

/// Resource being accessed (for scope validation)
#[derive(Debug)]
pub enum Resource<'a> {
    FilePath(&'a str),
    FileSize(usize),
    AudioTrack(u32),
    DocRef(u32),
    NoResource,
}

/// Default agent registry (Phase 1: static compilation)
static DEFAULT_AGENTS: &[AgentToken] = &[
    AgentToken {
        agent_id: AGENT_ID_SYSTEM,
        name: "system",
        capabilities: &[
            Capability::FsBasic,
            Capability::AudioControl,
            Capability::DocBasic,
            Capability::Capture,
            Capability::Screenshot,
            Capability::Admin,
        ],
        scope: Scope::UNRESTRICTED,
        enabled: true,
    },
    AgentToken {
        agent_id: AGENT_ID_AGENTD,
        name: "agentd",
        capabilities: &[
            Capability::FsBasic,
            Capability::AudioControl,
            Capability::DocBasic,
        ],
        scope: Scope::UNRESTRICTED,
        enabled: true,
    },
    AgentToken {
        agent_id: 2,
        name: "files_agent",
        capabilities: &[Capability::FsBasic],
        scope: Scope::with_path("/tmp/files/"),
        enabled: true,
    },
    AgentToken {
        agent_id: 3,
        name: "docs_agent",
        capabilities: &[Capability::FsBasic, Capability::DocBasic],
        scope: Scope::with_path("/tmp/docs/"),
        enabled: true,
    },
    AgentToken {
        agent_id: 4,
        name: "music_agent",
        capabilities: &[Capability::AudioControl],
        scope: Scope::UNRESTRICTED,
        enabled: true,
    },
    AgentToken {
        agent_id: AGENT_ID_TEST,
        name: "test_agent",
        capabilities: &[
            Capability::FsBasic,
            Capability::AudioControl,
            Capability::DocBasic,
            Capability::Capture,
            Capability::Screenshot,
        ],
        scope: Scope::UNRESTRICTED,
        enabled: true,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_allow() {
        let engine = PolicyEngine::new_default();
        let decision = engine.check(
            AGENT_ID_AGENTD,
            Capability::FsBasic,
            &Resource::FilePath("/tmp/test.txt"),
        );
        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn test_policy_deny_capability() {
        let engine = PolicyEngine::new_default();
        let decision = engine.check(
            4, // music_agent (no FsBasic)
            Capability::FsBasic,
            &Resource::NoResource,
        );
        assert!(matches!(decision, PolicyDecision::Deny { .. }));
    }

    #[test]
    fn test_scope_restriction() {
        let engine = PolicyEngine::new_default();
        let decision = engine.check(
            2, // files_agent (restricted to /tmp/files/)
            Capability::FsBasic,
            &Resource::FilePath("/etc/passwd"),
        );
        assert!(matches!(decision, PolicyDecision::Deny { .. }));
    }
}
