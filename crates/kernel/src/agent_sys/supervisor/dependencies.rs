//! Agent Dependency Tracking
//!
//! This module tracks dependencies and relationships between agents,
//! enabling coordinated lifecycle management and cascade handling.

use crate::agent_sys::AgentId;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

/// Type of dependency relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DependencyType {
    /// Strong dependency - dependent must exit if dependency exits
    Required,

    /// Weak dependency - dependent should be notified but can continue
    Optional,

    /// Coordination dependency - agents coordinate but neither requires the other
    Peer,
}

/// A dependency edge in the agent graph
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dependency {
    /// Agent that depends on another
    pub dependent: AgentId,

    /// Agent being depended upon
    pub dependency: AgentId,

    /// Type of dependency
    pub dep_type: DependencyType,
}

/// Agent dependency graph
pub struct DependencyGraph {
    /// Forward edges: agent -> agents it depends on
    dependencies: BTreeMap<AgentId, Vec<Dependency>>,

    /// Reverse edges: agent -> agents that depend on it
    dependents: BTreeMap<AgentId, Vec<AgentId>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            dependencies: BTreeMap::new(),
            dependents: BTreeMap::new(),
        }
    }

    /// Add an agent to the graph
    pub fn add_agent(&mut self, agent_id: AgentId) {
        self.dependencies.entry(agent_id).or_insert_with(Vec::new);
        self.dependents.entry(agent_id).or_insert_with(Vec::new);
    }

    /// Remove an agent from the graph
    pub fn remove_agent(&mut self, agent_id: AgentId) {
        // Remove all dependencies from this agent
        if let Some(deps) = self.dependencies.remove(&agent_id) {
            // Remove reverse edges
            for dep in deps {
                if let Some(reverse) = self.dependents.get_mut(&dep.dependency) {
                    reverse.retain(|id| *id != agent_id);
                }
            }
        }

        // Remove all dependencies to this agent
        if let Some(dependents) = self.dependents.remove(&agent_id) {
            for dependent_id in dependents {
                if let Some(forward) = self.dependencies.get_mut(&dependent_id) {
                    forward.retain(|dep| dep.dependency != agent_id);
                }
            }
        }
    }

    /// Add a dependency edge
    pub fn add_dependency(&mut self, dependent: AgentId, dependency: AgentId, dep_type: DependencyType) {
        // Ensure both agents exist in graph
        self.add_agent(dependent);
        self.add_agent(dependency);

        // Add forward edge
        let deps = self.dependencies.get_mut(&dependent).unwrap();
        let dep_edge = Dependency {
            dependent,
            dependency,
            dep_type,
        };

        // Only add if not already present
        if !deps.iter().any(|d| d.dependency == dependency) {
            deps.push(dep_edge);
        }

        // Add reverse edge
        let rev = self.dependents.get_mut(&dependency).unwrap();
        if !rev.contains(&dependent) {
            rev.push(dependent);
        }
    }

    /// Get all dependencies for an agent
    pub fn get_dependencies(&self, agent_id: AgentId) -> Option<&[Dependency]> {
        self.dependencies.get(&agent_id).map(|v| v.as_slice())
    }

    /// Get all agents that depend on this agent
    pub fn get_dependents(&self, agent_id: AgentId) -> Option<&[AgentId]> {
        self.dependents.get(&agent_id).map(|v| v.as_slice())
    }

    /// Get agents that should exit when the given agent exits
    pub fn get_cascade_exits(&self, agent_id: AgentId) -> Vec<AgentId> {
        let mut result = Vec::new();

        if let Some(dependents) = self.dependents.get(&agent_id) {
            for &dependent_id in dependents {
                // Check if this dependent has a Required dependency on agent_id
                if let Some(deps) = self.dependencies.get(&dependent_id) {
                    for dep in deps {
                        if dep.dependency == agent_id && dep.dep_type == DependencyType::Required {
                            result.push(dependent_id);
                            // Recursively find cascade exits
                            result.extend(self.get_cascade_exits(dependent_id));
                            break;
                        }
                    }
                }
            }
        }

        result
    }

    /// Detect circular dependencies
    pub fn has_circular_dependency(&self, agent_id: AgentId) -> bool {
        let mut visited = BTreeSet::new();
        let mut stack = Vec::new();

        self.detect_cycle(agent_id, &mut visited, &mut stack)
    }

    fn detect_cycle(&self, agent_id: AgentId, visited: &mut BTreeSet<AgentId>, stack: &mut Vec<AgentId>) -> bool {
        if stack.contains(&agent_id) {
            return true; // Cycle detected
        }

        if visited.contains(&agent_id) {
            return false; // Already visited, no cycle from this path
        }

        visited.insert(agent_id);
        stack.push(agent_id);

        if let Some(deps) = self.dependencies.get(&agent_id) {
            for dep in deps {
                if self.detect_cycle(dep.dependency, visited, stack) {
                    return true;
                }
            }
        }

        stack.pop();
        false
    }

    /// Get dependency chain (transitive closure) for an agent
    pub fn get_dependency_chain(&self, agent_id: AgentId) -> Vec<AgentId> {
        let mut result = Vec::new();
        let mut visited = BTreeSet::new();
        self.collect_dependencies(agent_id, &mut visited, &mut result);
        result
    }

    fn collect_dependencies(&self, agent_id: AgentId, visited: &mut BTreeSet<AgentId>, result: &mut Vec<AgentId>) {
        if visited.contains(&agent_id) {
            return;
        }

        visited.insert(agent_id);

        if let Some(deps) = self.dependencies.get(&agent_id) {
            for dep in deps {
                if !visited.contains(&dep.dependency) {
                    result.push(dep.dependency);
                    self.collect_dependencies(dep.dependency, visited, result);
                }
            }
        }
    }

    /// Get number of agents in graph
    pub fn agent_count(&self) -> usize {
        self.dependencies.len()
    }

    /// Get total number of dependency edges
    pub fn edge_count(&self) -> usize {
        self.dependencies.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph_creation() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.agent_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();

        graph.add_dependency(100, 101, DependencyType::Required);

        assert_eq!(graph.agent_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        let deps = graph.get_dependencies(100).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].dependency, 101);
    }

    #[test]
    fn test_cascade_exits() {
        let mut graph = DependencyGraph::new();

        // 100 -> 101 (required)
        // 101 -> 102 (required)
        graph.add_dependency(100, 101, DependencyType::Required);
        graph.add_dependency(101, 102, DependencyType::Required);

        // If 102 exits, both 101 and 100 should cascade
        let cascade = graph.get_cascade_exits(102);
        assert!(cascade.contains(&101));
        assert!(cascade.contains(&100));
    }

    #[test]
    fn test_circular_detection() {
        let mut graph = DependencyGraph::new();

        // Create a cycle: 100 -> 101 -> 102 -> 100
        graph.add_dependency(100, 101, DependencyType::Required);
        graph.add_dependency(101, 102, DependencyType::Required);
        graph.add_dependency(102, 100, DependencyType::Required);

        assert!(graph.has_circular_dependency(100));
    }

    #[test]
    fn test_remove_agent() {
        let mut graph = DependencyGraph::new();

        graph.add_dependency(100, 101, DependencyType::Required);
        graph.add_dependency(100, 102, DependencyType::Required);

        graph.remove_agent(100);

        assert_eq!(graph.agent_count(), 2); // 101 and 102 remain
        assert_eq!(graph.edge_count(), 0); // All edges involving 100 removed
    }

    #[test]
    fn test_dependency_chain() {
        let mut graph = DependencyGraph::new();

        // 100 depends on 101, which depends on 102
        graph.add_dependency(100, 101, DependencyType::Required);
        graph.add_dependency(101, 102, DependencyType::Required);

        let chain = graph.get_dependency_chain(100);
        assert_eq!(chain.len(), 2);
        assert!(chain.contains(&101));
        assert!(chain.contains(&102));
    }
}
