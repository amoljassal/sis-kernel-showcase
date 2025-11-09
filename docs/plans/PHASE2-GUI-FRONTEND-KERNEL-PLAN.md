# Phase 2 GUI Frontend & Kernel Commands Implementation Plan
## Complete Integration of AI Governance Visualization

**Document Version:** 1.0
**Date:** November 9, 2025
**Prerequisites:** Phase 2 GUI Backend (Branch: `claude/review-phase2-gui-plan-011CUxKrRGdXy5k14Q2Mn1PH`)
**Target Completion:** Q1 2026
**Status:** 🔴 PLANNING

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State](#current-state)
3. [Architecture Overview](#architecture-overview)
4. [Part 1: Kernel Shell Commands](#part-1-kernel-shell-commands)
5. [Part 2: Frontend Components](#part-2-frontend-components)
6. [Implementation Phases](#implementation-phases)
7. [Testing Strategy](#testing-strategy)
8. [Success Criteria](#success-criteria)
9. [Technical Specifications](#technical-specifications)
10. [File Structure](#file-structure)
11. [Code Examples](#code-examples)

---

## Executive Summary

### Objective

Complete the Phase 2 GUI implementation by:
1. Implementing 25 shell commands in the kernel for AI governance operations
2. Building 5 React panels with 20+ supporting components (~3,520 lines TypeScript)
3. Integrating real-time WebSocket event handling
4. Creating comprehensive E2E test coverage

### Scope

**Part 1 - Kernel Commands (Rust):**
- 25 shell commands across 5 domains (coordctl, deployctl, driftctl, versionctl, agentctl)
- JSON output formatting for all commands
- Integration with Phase 2 kernel modules
- ~1,500 lines of Rust code

**Part 2 - Frontend Components (TypeScript/React):**
- 5 main panels (Orchestration, Conflicts, Deployment, Drift, Versions)
- 20+ supporting components
- API client methods for 22 endpoints
- WebSocket event handlers for 5 event types
- ~3,520 lines of TypeScript code

**Part 3 - Testing:**
- 5 E2E test suites (Playwright)
- WebSocket integration tests
- API endpoint tests
- ~800 lines of test code

### Impact

- **Developers**: Full visibility into AI governance decisions via GUI
- **Operations**: Manual override controls for deployment and drift management
- **Debugging**: Real-time conflict visualization and resolution tracking
- **Version Control**: Git-like interface for adapter management

### Timeline

- **Phase 1 (Kernel Commands)**: 2 weeks - 25 shell commands with JSON output
- **Phase 2 (Frontend Core)**: 3 weeks - 5 panels and supporting components
- **Phase 3 (Integration)**: 1 week - WebSocket handlers and API client
- **Phase 4 (Testing)**: 1 week - E2E tests and integration testing
- **Phase 5 (Polish)**: 1 week - UX refinement and documentation

**Total:** 8 weeks

---

## Current State

### ✅ What Exists (Completed)

**Backend API (sisctl daemon):**
- 22 REST API endpoints implemented
- 5 WebSocket event types defined
- OpenAPI documentation complete
- Error handling (RFC 7807 problem+json)
- Compilation successful (0 errors)

**Phase 2 Kernel Modules:**
- `AgentOrchestrator` - Multi-agent coordination
- `ConflictResolver` - Conflict resolution engine
- `DeploymentManager` - Phase-based rollout
- `DriftDetector` - Model performance monitoring
- `AdapterVersionControl` - Git-like versioning

### ❌ What's Missing (This Plan)

**Kernel Shell Commands:**
- No `coordctl` commands implemented
- No `deployctl` commands implemented
- No `driftctl` commands implemented
- No `versionctl` commands implemented
- No JSON output formatting

**Frontend Components:**
- No React panels for Phase 2 features
- No API client methods
- No WebSocket event handlers
- No E2E tests

### 🔗 Dependencies

**This implementation requires:**
1. Backend API endpoints (✅ Done - from previous implementation)
2. Phase 2 kernel modules (✅ Done - already merged to main)
3. Kernel shell infrastructure (✅ Done - existing shell.rs)

---

## Architecture Overview

### High-Level Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                       GUI Frontend                          │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ React Components (5 panels + 20 components)           │ │
│  │  - OrchestrationPanel.tsx                             │ │
│  │  - ConflictPanel.tsx                                  │ │
│  │  - DeploymentPanel.tsx                                │ │
│  │  - DriftPanel.tsx                                     │ │
│  │  - VersionsPanel.tsx                                  │ │
│  └───────────────────────────────────────────────────────┘ │
│            ↑ HTTP API            ↑ WebSocket               │
└────────────┼────────────────────┼─────────────────────────┘
             │                    │
┌────────────┼────────────────────┼─────────────────────────┐
│            ↓                    ↓                          │
│                   sisctl Daemon (Rust)                     │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ API Handlers (22 endpoints) ✅ DONE                   │ │
│  │  /api/v1/orchestrator/*                               │ │
│  │  /api/v1/conflicts/*                                  │ │
│  │  /api/v1/deployment/*                                 │ │
│  │  /api/v1/drift/*                                      │ │
│  │  /api/v1/versions/*                                   │ │
│  └───────────────────────────────────────────────────────┘ │
│            ↓ Shell Commands                                │
└────────────┼────────────────────────────────────────────────┘
             │
┌────────────┼────────────────────────────────────────────────┐
│            ↓                                                │
│                    SIS Kernel (Rust)                        │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ Shell Interface (shell.rs)                            │ │
│  │  ❌ NEW: coordctl, deployctl, driftctl, versionctl   │ │
│  └───────────────────────────────────────────────────────┘ │
│            ↓                                                │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ Phase 2 Modules ✅ DONE                               │ │
│  │  - AgentOrchestrator (src/ai/orchestrator.rs)        │ │
│  │  - ConflictResolver (src/ai/conflict.rs)             │ │
│  │  - DeploymentManager (src/ai/deployment.rs)          │ │
│  │  - DriftDetector (src/llm/drift_detector.rs)         │ │
│  │  - AdapterVersionControl (src/llm/version.rs)        │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Technology Stack

**Kernel (Rust):**
- Shell command parsing in `src/shell.rs`
- JSON serialization with `serde_json`
- Integration with existing Phase 2 modules

**Frontend (TypeScript/React):**
- React 18 + TypeScript
- TanStack React Query v5 (HTTP polling)
- Custom WebSocket hook (existing)
- Recharts (existing charts)
- react-flow (NEW - for state machines/trees)
- Radix UI + Tailwind CSS (existing)

---

## Part 1: Kernel Shell Commands

### Overview

Implement 25 shell commands across 5 domains to expose Phase 2 AI governance functionality to the sisctl daemon.

**Total Commands:** 25
**Estimated Lines:** ~1,500 lines Rust
**Location:** `crates/kernel/src/shell.rs`

---

### 1. Orchestration Commands (3 commands)

#### Command: `coordctl status [--json]`

**Purpose:** Get orchestration statistics

**Rust Implementation:**
```rust
// In shell.rs
fn handle_coordctl(args: &[&str]) {
    if args.len() == 0 || args[0] == "status" {
        let json_mode = args.contains(&"--json");

        // Get stats from orchestrator
        let stats = ORCHESTRATOR.get_stats();

        if json_mode {
            // JSON output
            let json = serde_json::json!({
                "total_decisions": stats.total_decisions,
                "unanimous": stats.unanimous,
                "majority": stats.majority,
                "safety_overrides": stats.safety_overrides,
                "no_consensus": stats.no_consensus,
                "avg_latency_us": stats.avg_latency_us
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        } else {
            // Human-readable output
            println!("Orchestration Statistics:");
            println!("  Total Decisions: {}", stats.total_decisions);
            println!("  Unanimous: {} ({:.1}%)", stats.unanimous,
                (stats.unanimous as f64 / stats.total_decisions as f64) * 100.0);
            println!("  Majority: {} ({:.1}%)", stats.majority,
                (stats.majority as f64 / stats.total_decisions as f64) * 100.0);
            println!("  Safety Overrides: {} ({:.1}%)", stats.safety_overrides,
                (stats.safety_overrides as f64 / stats.total_decisions as f64) * 100.0);
            println!("  No Consensus: {} ({:.1}%)", stats.no_consensus,
                (stats.no_consensus as f64 / stats.total_decisions as f64) * 100.0);
            println!("  Avg Latency: {} μs", stats.avg_latency_us);
        }
    }
}
```

**Output (JSON):**
```json
{
  "total_decisions": 1543,
  "unanimous": 892,
  "majority": 451,
  "safety_overrides": 178,
  "no_consensus": 22,
  "avg_latency_us": 234
}
```

**Output (Human):**
```
Orchestration Statistics:
  Total Decisions: 1543
  Unanimous: 892 (57.8%)
  Majority: 451 (29.2%)
  Safety Overrides: 178 (11.5%)
  No Consensus: 22 (1.4%)
  Avg Latency: 234 μs
```

#### Command: `coordctl history [--limit N] [--json]`

**Purpose:** Get recent coordinated decisions

**Implementation:**
```rust
fn handle_coordctl_history(args: &[&str]) {
    let limit = parse_arg_u32(args, "--limit").unwrap_or(100);
    let json_mode = args.contains(&"--json");

    // Get decision history (need to add history tracking to orchestrator)
    let decisions = ORCHESTRATOR.get_decision_history(limit);

    if json_mode {
        let json = serde_json::json!(decisions);
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Recent Decisions (last {}):", decisions.len());
        for decision in decisions {
            println!("  [{}] {} - {} ({:.0}% confidence)",
                decision.timestamp,
                decision.decision_type,
                decision.action,
                decision.confidence * 100.0
            );
        }
    }
}
```

**Output (JSON):**
```json
[
  {
    "timestamp": "2025-11-09T10:23:45.123456789Z",
    "type": "unanimous",
    "action": "CompactMemory",
    "confidence": 0.92,
    "agents": ["CrashPredictor", "StateInference", "TransformerScheduler"],
    "latency_us": 189
  },
  {
    "timestamp": "2025-11-09T10:23:40.234567890Z",
    "type": "safety_override",
    "action": "Stop",
    "overridden_by": "CrashPredictor",
    "reason": "High crash risk detected (95% confidence)",
    "overridden_agents": ["TransformerScheduler", "FineTuner"],
    "latency_us": 312
  }
]
```

#### Command: `agentctl list [--json]`

**Purpose:** Get status of all agents

**Implementation:**
```rust
fn handle_agentctl_list(args: &[&str]) {
    let json_mode = args.contains(&"--json");

    // Get agent status (need to add agent tracking to orchestrator)
    let agents = ORCHESTRATOR.get_agents();

    if json_mode {
        let json = serde_json::json!({ "agents": agents });
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Active Agents:");
        for agent in agents {
            println!("  {} (priority: {})", agent.name, agent.priority);
            println!("    Status: {}", agent.status);
            println!("    Last Decision: {} @ {:.0}% confidence",
                agent.last_decision.action,
                agent.last_decision.confidence * 100.0
            );
        }
    }
}
```

---

### 2. Conflict Resolution Commands (3 commands)

#### Command: `coordctl conflict-stats [--json]`

**Purpose:** Get conflict resolution statistics

**Implementation:**
```rust
fn handle_coordctl_conflict_stats(args: &[&str]) {
    let json_mode = args.contains(&"--json");

    let stats = CONFLICT_RESOLVER.get_stats();

    if json_mode {
        let json = serde_json::json!({
            "total_conflicts": stats.total_conflicts,
            "resolved_by_priority": stats.resolved_by_priority,
            "resolved_by_voting": stats.resolved_by_voting,
            "unresolved": stats.unresolved,
            "avg_resolution_time_us": stats.avg_resolution_time_us
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Conflict Resolution Statistics:");
        println!("  Total Conflicts: {}", stats.total_conflicts);
        println!("  Resolved by Priority: {} ({:.1}%)",
            stats.resolved_by_priority,
            (stats.resolved_by_priority as f64 / stats.total_conflicts as f64) * 100.0
        );
        println!("  Resolved by Voting: {} ({:.1}%)",
            stats.resolved_by_voting,
            (stats.resolved_by_voting as f64 / stats.total_conflicts as f64) * 100.0
        );
        println!("  Unresolved: {}", stats.unresolved);
        println!("  Avg Resolution Time: {} μs", stats.avg_resolution_time_us);
    }
}
```

#### Command: `coordctl conflict-history [--limit N] [--json]`

**Purpose:** Get conflict history

**Implementation:**
```rust
fn handle_coordctl_conflict_history(args: &[&str]) {
    let limit = parse_arg_u32(args, "--limit").unwrap_or(100);
    let json_mode = args.contains(&"--json");

    let conflicts = CONFLICT_RESOLVER.get_history(limit);

    if json_mode {
        let json = serde_json::json!({ "conflicts": conflicts });
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Recent Conflicts (last {}):", conflicts.len());
        for conflict in conflicts {
            println!("  [{}] {} vs {}",
                conflict.timestamp,
                conflict.agents[0].agent,
                conflict.agents[1].agent
            );
            println!("    Resolution: {} won ({})",
                conflict.resolution.winner,
                conflict.resolution.strategy
            );
        }
    }
}
```

#### Command: `coordctl priorities [--json]`

**Purpose:** Get agent priority table

**Implementation:**
```rust
fn handle_coordctl_priorities(args: &[&str]) {
    let json_mode = args.contains(&"--json");

    let priorities = CONFLICT_RESOLVER.get_priorities();

    if json_mode {
        let json = serde_json::json!({ "priorities": priorities });
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Agent Priority Table:");
        for entry in priorities {
            println!("  {:25} Priority: {:3}", entry.agent, entry.priority);
        }
    }
}
```

---

### 3. Deployment Management Commands (5 commands)

#### Command: `deployctl status [--json]`

**Purpose:** Get current deployment phase status

**Implementation:**
```rust
fn handle_deployctl_status(args: &[&str]) {
    let json_mode = args.contains(&"--json");

    let status = DEPLOYMENT_MANAGER.get_status();

    if json_mode {
        let json = serde_json::json!({
            "current_phase": {
                "id": status.current_phase.id,
                "name": status.current_phase.name,
                "description": status.current_phase.description,
                "entered_at": status.current_phase.entered_at,
                "min_duration_ms": status.current_phase.min_duration_ms,
                "elapsed_ms": status.current_phase.elapsed_ms,
                "can_advance": status.current_phase.can_advance,
                "traffic_percentage": status.current_phase.traffic_percentage,
                "error_rate": status.current_phase.error_rate,
                "success_rate": status.current_phase.success_rate
            },
            "auto_advance_enabled": status.auto_advance_enabled,
            "auto_rollback_enabled": status.auto_rollback_enabled,
            "rollback_count": status.rollback_count,
            "max_rollbacks": status.max_rollbacks
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Deployment Status:");
        println!("  Current Phase: {} ({})",
            status.current_phase.id,
            status.current_phase.name
        );
        println!("  Traffic: {}%", status.current_phase.traffic_percentage);
        println!("  Error Rate: {:.2}%", status.current_phase.error_rate * 100.0);
        println!("  Elapsed: {}ms / Min: {}ms",
            status.current_phase.elapsed_ms,
            status.current_phase.min_duration_ms
        );
        println!("  Auto-Advance: {}", if status.auto_advance_enabled { "ON" } else { "OFF" });
        println!("  Auto-Rollback: {}", if status.auto_rollback_enabled { "ON" } else { "OFF" });
        println!("  Rollbacks: {}/{}", status.rollback_count, status.max_rollbacks);
    }
}
```

#### Command: `deployctl history [--limit N] [--json]`

**Purpose:** Get phase transition history

#### Command: `deployctl advance [--force] [--json]`

**Purpose:** Manually advance to next phase

#### Command: `deployctl rollback [--json]`

**Purpose:** Manually rollback to previous phase

#### Command: `deployctl config [options] [--json]`

**Purpose:** Update deployment configuration

**Options:**
- `--auto-advance=on|off`
- `--auto-rollback=on|off`
- `--error-threshold=N`

---

### 4. Drift Detection Commands (4 commands)

#### Command: `driftctl status [--json]`

**Purpose:** Get drift detection status

**Implementation:**
```rust
fn handle_driftctl_status(args: &[&str]) {
    let json_mode = args.contains(&"--json");

    let status = DRIFT_DETECTOR.get_status();

    if json_mode {
        let json = serde_json::json!({
            "baseline_accuracy": status.baseline_accuracy,
            "current_accuracy": status.current_accuracy,
            "accuracy_delta": status.accuracy_delta,
            "drift_level": status.drift_level,
            "sample_window_size": status.sample_window_size,
            "samples_analyzed": status.samples_analyzed,
            "last_retrain": status.last_retrain,
            "auto_retrain_enabled": status.auto_retrain_enabled,
            "auto_retrain_threshold": status.auto_retrain_threshold
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Drift Detection Status:");
        println!("  Baseline Accuracy: {:.2}%", status.baseline_accuracy * 100.0);
        println!("  Current Accuracy: {:.2}%", status.current_accuracy * 100.0);
        println!("  Drift: {:.2}% ({})",
            status.accuracy_delta * 100.0,
            status.drift_level
        );
        println!("  Samples: {} (window: {})",
            status.samples_analyzed,
            status.sample_window_size
        );
        println!("  Auto-Retrain: {} (threshold: {:.0}%)",
            if status.auto_retrain_enabled { "ON" } else { "OFF" },
            status.auto_retrain_threshold * 100.0
        );
    }
}
```

#### Command: `driftctl history [--limit N] [--json]`

**Purpose:** Get drift history

#### Command: `driftctl retrain [--examples N] [--epochs N] [--json]`

**Purpose:** Manually trigger model retraining

#### Command: `driftctl reset-baseline [--json]`

**Purpose:** Reset baseline accuracy to current accuracy

---

### 5. Version Control Commands (6 commands)

#### Command: `versionctl list [--limit N] [--json]`

**Purpose:** Get adapter version history

**Implementation:**
```rust
fn handle_versionctl_list(args: &[&str]) {
    let limit = parse_arg_u32(args, "--limit").unwrap_or(10);
    let json_mode = args.contains(&"--json");

    let versions = VERSION_CONTROL.history();
    let versions = versions.into_iter().take(limit as usize).collect::<Vec<_>>();

    if json_mode {
        let json = serde_json::json!({
            "current_version": VERSION_CONTROL.current_version(),
            "versions": versions
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("Adapter Version History:");
        for version in versions {
            let marker = if version.version_id == VERSION_CONTROL.current_version() {
                " (HEAD)"
            } else {
                ""
            };
            println!("  v{}{}", version.version_id, marker);
            println!("    Description: {}", version.metadata.description);
            println!("    Trained: {} examples, {:.1}s",
                version.metadata.training_examples,
                version.metadata.training_duration_ms as f64 / 1000.0
            );
            println!("    Accuracy: {:+.1}%",
                version.metadata.accuracy_improvement * 100.0
            );
            if !version.tags.is_empty() {
                println!("    Tags: {}", version.tags.join(", "));
            }
        }
    }
}
```

#### Command: `versionctl commit -m "description" [--env TAG] [--json]`

**Purpose:** Commit current adapter as new version

#### Command: `versionctl rollback VERSION_ID [--json]`

**Purpose:** Rollback to previous version

#### Command: `versionctl diff VERSION1 VERSION2 [--json]`

**Purpose:** Compare two adapter versions

#### Command: `versionctl tag VERSION_ID TAG_NAME [--json]`

**Purpose:** Tag a version

#### Command: `versionctl gc [--keep N] [--json]`

**Purpose:** Garbage collect old versions

---

### Kernel Module Extensions

To support these commands, the following additions are needed to Phase 2 modules:

**1. AgentOrchestrator** (`src/ai/orchestrator.rs`):
- Add `decision_history` ring buffer (capacity: 1000)
- Add `get_decision_history(limit: usize)` method
- Add `get_agents()` method to query agent status

**2. ConflictResolver** (`src/ai/conflict.rs`):
- Add `conflict_history` ring buffer (capacity: 1000)
- Add `get_history(limit: usize)` method
- Add `get_priorities()` method

**3. DeploymentManager** (`src/ai/deployment.rs`):
- Add `transition_history` ring buffer (capacity: 500)
- Add `advance_phase(force: bool)` method
- Add `rollback_phase()` method
- Add `update_config(config: DeploymentConfig)` method

**4. DriftDetector** (`src/llm/drift_detector.rs`):
- Add `drift_history` ring buffer (capacity: 1000)
- Add `get_history(limit: usize)` method
- Add `trigger_retrain(examples: usize, epochs: usize)` method
- Add `reset_baseline()` method

**5. AdapterVersionControl** (`src/llm/version.rs`):
- Already has necessary methods ✅

---

### Command Registration in Shell

**Location:** `crates/kernel/src/shell.rs`

**Add to command dispatcher:**
```rust
fn handle_command(cmd: &str, args: &[&str]) {
    match cmd {
        // Existing commands...
        "neuralctl" => handle_neuralctl(args),
        "memctl" => handle_memctl(args),

        // NEW: Phase 2 AI Governance commands
        "coordctl" => handle_coordctl(args),
        "agentctl" => handle_agentctl(args),
        "deployctl" => handle_deployctl(args),
        "driftctl" => handle_driftctl(args),
        "versionctl" => handle_versionctl(args),

        _ => println!("Unknown command: {}", cmd),
    }
}
```

---

## Part 2: Frontend Components

### Overview

Build 5 React panels with 20+ supporting components to visualize Phase 2 AI governance.

**Total Components:** 25+ components
**Estimated Lines:** ~3,520 lines TypeScript
**Location:** `gui/desktop/src/components/`

---

### Component Hierarchy

```
App.tsx (UPDATED - add 5 new tabs)
├── OrchestrationPanel.tsx (NEW - 300 lines)
│   ├── OrchestrationStatsCard.tsx (NEW - 80 lines)
│   ├── AgentCard.tsx (NEW - 80 lines)
│   ├── DecisionHistoryTable.tsx (NEW - 150 lines)
│   └── AgentStatusBadge.tsx (NEW - 40 lines)
│
├── ConflictPanel.tsx (NEW - 350 lines)
│   ├── ConflictStatsCard.tsx (NEW - 80 lines)
│   ├── PriorityTable.tsx (NEW - 120 lines)
│   ├── ConflictHistoryTable.tsx (NEW - 150 lines)
│   ├── ConflictDetailCard.tsx (NEW - 150 lines)
│   └── ConflictVisualization.tsx (NEW - 120 lines)
│
├── DeploymentPanel.tsx (NEW - 400 lines)
│   ├── CurrentPhaseCard.tsx (NEW - 150 lines)
│   ├── PhaseTimeline.tsx (NEW - 100 lines)
│   ├── PhaseDefinitionsTable.tsx (NEW - 80 lines)
│   ├── TransitionHistoryTable.tsx (NEW - 150 lines)
│   └── DeploymentMetricsChart.tsx (NEW - 150 lines)
│
├── DriftPanel.tsx (NEW - 350 lines)
│   ├── DriftStatusCard.tsx (NEW - 150 lines)
│   ├── DriftLevelIndicator.tsx (NEW - 60 lines)
│   ├── DriftChart.tsx (NEW - 150 lines)
│   ├── DriftHistoryTable.tsx (NEW - 150 lines)
│   └── DriftConfigCard.tsx (NEW - 120 lines)
│
└── VersionsPanel.tsx (NEW - 450 lines)
    ├── VersionHeader.tsx (NEW - 80 lines)
    ├── VersionTree.tsx (NEW - 200 lines)
    ├── VersionCard.tsx (NEW - 100 lines)
    ├── VersionDiffCard.tsx (NEW - 120 lines)
    ├── CommitDialog.tsx (NEW - 120 lines)
    └── TagBadge.tsx (NEW - 40 lines)
```

---

### 1. OrchestrationPanel.tsx

**Purpose:** Visualize multi-agent orchestration and decision flow

**Features:**
- Orchestration statistics cards
- Active agents grid with status
- Recent decisions table (virtualized)
- Real-time updates via WebSocket

**Key Code:**
```typescript
import { useQuery } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import { orchestratorApi } from '@/lib/api';
import { useWebSocket } from '@/lib/useWebSocket';

interface OrchestrationStats {
  total_decisions: number;
  unanimous: number;
  majority: number;
  safety_overrides: number;
  no_consensus: number;
  avg_latency_us: number;
}

interface CoordinatedDecision {
  timestamp: string;
  type: 'unanimous' | 'majority' | 'safety_override' | 'no_consensus';
  action: string;
  confidence?: number;
  agents: string[];
  latency_us: number;
}

export function OrchestrationPanel() {
  const parentRef = React.useRef<HTMLDivElement>(null);

  // Fetch stats (polling every 5s)
  const { data: stats } = useQuery<OrchestrationStats>({
    queryKey: ['orchestrator', 'stats'],
    queryFn: orchestratorApi.getStats,
    refetchInterval: 5000,
  });

  // Fetch decisions (polling every 2s)
  const { data: decisions } = useQuery<{ decisions: CoordinatedDecision[] }>({
    queryKey: ['orchestrator', 'decisions'],
    queryFn: () => orchestratorApi.getDecisions(100),
    refetchInterval: 2000,
  });

  // Listen for real-time orchestration decisions
  const { lastMessage } = useWebSocket();
  React.useEffect(() => {
    if (lastMessage?.type === 'orchestration_decision') {
      queryClient.invalidateQueries(['orchestrator']);
    }
  }, [lastMessage]);

  // Virtual scrolling for decision table
  const rowVirtualizer = useVirtualizer({
    count: decisions?.decisions.length || 0,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 50,
    overscan: 5,
  });

  return (
    <div className="space-y-4 p-4">
      {/* Statistics Cards */}
      <OrchestrationStatsCard stats={stats} />

      {/* Active Agents Grid */}
      <div className="grid grid-cols-3 gap-4">
        {agents?.agents.map(agent => (
          <AgentCard key={agent.name} agent={agent} />
        ))}
      </div>

      {/* Recent Decisions Table */}
      <div ref={parentRef} className="h-96 overflow-auto">
        <div style={{ height: `${rowVirtualizer.getTotalSize()}px` }}>
          {rowVirtualizer.getVirtualItems().map(virtualRow => {
            const decision = decisions.decisions[virtualRow.index];
            return (
              <DecisionRow
                key={virtualRow.index}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                decision={decision}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
}
```

---

### 2. ConflictPanel.tsx

**Purpose:** Visualize agent conflicts and resolution strategies

**Features:**
- Conflict statistics card
- Priority table with visual bars
- Conflict history table
- Conflict detail view with agent comparison

**Key Components:**
- **PriorityTable** - Visual representation of agent priorities
- **ConflictVisualization** - Shows conflicting agents and resolution

---

### 3. DeploymentPanel.tsx

**Purpose:** Manage deployment phases and rollouts

**Features:**
- Current phase display with progress bar
- Phase timeline (A → B → C → D)
- Phase transition history
- Manual advance/rollback controls
- Configuration form

**Key Components:**
- **PhaseTimeline** - Visual state machine showing phases
- **CurrentPhaseCard** - Detailed phase metrics and controls

**Example Phase Timeline:**
```typescript
import ReactFlow, { Node, Edge } from 'react-flow-renderer';

const nodes: Node[] = [
  {
    id: 'A',
    data: { label: 'A: Shadow\n0% traffic\n24h min' },
    position: { x: 0, y: 0 },
    style: { background: phase === 'A' ? '#10b981' : '#6b7280' },
  },
  {
    id: 'B',
    data: { label: 'B: Canary\n10% traffic\n12h min' },
    position: { x: 200, y: 0 },
    style: { background: phase === 'B' ? '#10b981' : '#6b7280' },
  },
  {
    id: 'C',
    data: { label: 'C: Gradual\n50% traffic\n24h min' },
    position: { x: 400, y: 0 },
    style: { background: phase === 'C' ? '#10b981' : '#6b7280' },
  },
  {
    id: 'D',
    data: { label: 'D: Full\n100% traffic' },
    position: { x: 600, y: 0 },
    style: { background: phase === 'D' ? '#10b981' : '#6b7280' },
  },
];

const edges: Edge[] = [
  { id: 'A-B', source: 'A', target: 'B', animated: true },
  { id: 'B-C', source: 'B', target: 'C', animated: true },
  { id: 'C-D', source: 'C', target: 'D', animated: true },
];

export function PhaseTimeline({ currentPhase }: { currentPhase: string }) {
  return (
    <div style={{ height: 300 }}>
      <ReactFlow nodes={nodes} edges={edges} fitView />
    </div>
  );
}
```

---

### 4. DriftPanel.tsx

**Purpose:** Monitor model performance drift

**Features:**
- Drift status card with level indicator
- Accuracy chart over time
- Drift history table
- Manual retrain button
- Configuration form

**Key Components:**
- **DriftChart** - Line chart showing accuracy degradation
- **DriftLevelIndicator** - Traffic light style indicator (Normal/Warning/Critical)

**Example Drift Chart:**
```typescript
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ReferenceLine } from 'recharts';

export function DriftChart({ samples, baseline }: { samples: DriftSample[], baseline: number }) {
  return (
    <LineChart width={800} height={400} data={samples}>
      <CartesianGrid strokeDasharray="3 3" />
      <XAxis dataKey="timestamp" />
      <YAxis domain={[0, 1]} tickFormatter={(v) => `${(v * 100).toFixed(0)}%`} />
      <Tooltip />
      <Legend />

      {/* Baseline */}
      <ReferenceLine y={baseline} stroke="#3b82f6" strokeDasharray="3 3" label="Baseline" />

      {/* Warning threshold (baseline - 5%) */}
      <ReferenceLine y={baseline - 0.05} stroke="#f59e0b" strokeDasharray="3 3" label="Warning" />

      {/* Critical threshold (baseline - 15%) */}
      <ReferenceLine y={baseline - 0.15} stroke="#ef4444" strokeDasharray="3 3" label="Critical" />

      {/* Actual accuracy */}
      <Line
        type="monotone"
        dataKey="accuracy"
        stroke="#10b981"
        strokeWidth={2}
        dot={{ fill: '#10b981', r: 3 }}
      />
    </LineChart>
  );
}
```

---

### 5. VersionsPanel.tsx

**Purpose:** Manage adapter versions with Git-like interface

**Features:**
- Version history tree (like git log --graph)
- Version metadata display
- Version comparison (diff)
- Commit new version dialog
- Rollback/tag operations
- Garbage collection

**Key Components:**
- **VersionTree** - Tree visualization showing parent-child relationships
- **CommitDialog** - Modal form for committing new versions

**Example Version Tree:**
```typescript
import ReactFlow, { Node, Edge } from 'react-flow-renderer';

export function VersionTree({ versions, currentVersion }: VersionTreeProps) {
  // Build tree structure
  const nodes: Node[] = versions.map((v, idx) => ({
    id: `v${v.version_id}`,
    data: {
      label: (
        <VersionCard
          version={v}
          isCurrent={v.version_id === currentVersion}
        />
      ),
    },
    position: { x: 0, y: idx * 200 },
  }));

  const edges: Edge[] = versions
    .filter(v => v.parent_version)
    .map(v => ({
      id: `v${v.parent_version}-v${v.version_id}`,
      source: `v${v.parent_version}`,
      target: `v${v.version_id}`,
      animated: false,
    }));

  return (
    <div style={{ height: 600 }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        fitView
        nodesDraggable={false}
      />
    </div>
  );
}
```

---

### API Client Integration

**Location:** `gui/desktop/src/lib/api.ts`

**Add Phase 2 API methods:**

```typescript
// Orchestrator API
export const orchestratorApi = {
  getStats: async (): Promise<OrchestrationStats> => {
    const response = await axios.get(`${API_BASE}/api/v1/orchestrator/stats`);
    return response.data;
  },

  getDecisions: async (limit = 100, type?: string): Promise<{ decisions: CoordinatedDecision[] }> => {
    const params = new URLSearchParams();
    params.set('limit', limit.toString());
    if (type) params.set('type', type);
    const response = await axios.get(`${API_BASE}/api/v1/orchestrator/decisions?${params}`);
    return response.data;
  },

  getAgents: async (): Promise<{ agents: Agent[] }> => {
    const response = await axios.get(`${API_BASE}/api/v1/orchestrator/agents`);
    return response.data;
  },
};

// Conflicts API
export const conflictsApi = {
  getStats: async (): Promise<ConflictStats> => {
    const response = await axios.get(`${API_BASE}/api/v1/conflicts/stats`);
    return response.data;
  },

  getHistory: async (limit = 100, resolved?: boolean): Promise<{ conflicts: Conflict[] }> => {
    const params = new URLSearchParams();
    params.set('limit', limit.toString());
    if (resolved !== undefined) params.set('resolved', resolved.toString());
    const response = await axios.get(`${API_BASE}/api/v1/conflicts/history?${params}`);
    return response.data;
  },

  getPriorityTable: async (): Promise<{ priorities: PriorityEntry[] }> => {
    const response = await axios.get(`${API_BASE}/api/v1/conflicts/priority-table`);
    return response.data;
  },
};

// Deployment API
export const deploymentApi = {
  getStatus: async (): Promise<DeploymentStatus> => {
    const response = await axios.get(`${API_BASE}/api/v1/deployment/status`);
    return response.data;
  },

  getHistory: async (): Promise<{ transitions: PhaseTransition[] }> => {
    const response = await axios.get(`${API_BASE}/api/v1/deployment/history`);
    return response.data;
  },

  advance: async (force = false): Promise<{ success: boolean; old_phase: string; new_phase: string }> => {
    const response = await axios.post(`${API_BASE}/api/v1/deployment/advance`, { force });
    return response.data;
  },

  rollback: async (reason: string): Promise<{ success: boolean; old_phase: string; new_phase: string }> => {
    const response = await axios.post(`${API_BASE}/api/v1/deployment/rollback`, { reason });
    return response.data;
  },

  updateConfig: async (config: DeploymentConfig): Promise<{ success: boolean; config: DeploymentConfig }> => {
    const response = await axios.post(`${API_BASE}/api/v1/deployment/config`, config);
    return response.data;
  },
};

// Drift API
export const driftApi = {
  getStatus: async (): Promise<DriftStatus> => {
    const response = await axios.get(`${API_BASE}/api/v1/drift/status`);
    return response.data;
  },

  getHistory: async (timeRange = 86400): Promise<{ samples: DriftSample[] }> => {
    const response = await axios.get(`${API_BASE}/api/v1/drift/history?time_range=${timeRange}`);
    return response.data;
  },

  triggerRetrain: async (trainingExamples = 1000, epochs = 10): Promise<{ success: boolean }> => {
    const response = await axios.post(`${API_BASE}/api/v1/drift/retrain`, {
      training_examples: trainingExamples,
      epochs,
    });
    return response.data;
  },

  resetBaseline: async (): Promise<{ success: boolean; old_baseline: number; new_baseline: number }> => {
    const response = await axios.post(`${API_BASE}/api/v1/drift/reset-baseline`);
    return response.data;
  },
};

// Versions API
export const versionsApi = {
  getList: async (limit = 10): Promise<{ current_version: number; versions: AdapterVersion[] }> => {
    const response = await axios.get(`${API_BASE}/api/v1/versions/list?limit=${limit}`);
    return response.data;
  },

  commit: async (data: CommitVersionRequest): Promise<{ success: boolean; version_id: number }> => {
    const response = await axios.post(`${API_BASE}/api/v1/versions/commit`, data);
    return response.data;
  },

  rollback: async (versionId: number): Promise<{ success: boolean; old_version: number; new_version: number }> => {
    const response = await axios.post(`${API_BASE}/api/v1/versions/rollback`, { version_id: versionId });
    return response.data;
  },

  getDiff: async (v1: number, v2: number): Promise<VersionDiff> => {
    const response = await axios.get(`${API_BASE}/api/v1/versions/diff?v1=${v1}&v2=${v2}`);
    return response.data;
  },

  tag: async (versionId: number, tag: string): Promise<{ success: boolean }> => {
    const response = await axios.post(`${API_BASE}/api/v1/versions/tag`, { version_id: versionId, tag });
    return response.data;
  },

  gc: async (keepCount = 10): Promise<{ success: boolean; removed_count: number }> => {
    const response = await axios.post(`${API_BASE}/api/v1/versions/gc`, { keep_count: keepCount });
    return response.data;
  },
};
```

---

### WebSocket Event Handlers

**Location:** `gui/desktop/src/lib/useWebSocket.ts`

**Update to handle Phase 2 events:**

```typescript
import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';

export function useWebSocket() {
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const queryClient = useQueryClient();

  useEffect(() => {
    const ws = new WebSocket('ws://127.0.0.1:8871/events');

    ws.onopen = () => {
      console.log('WebSocket connected');
      setIsConnected(true);
    };

    ws.onmessage = (event) => {
      const message: WebSocketMessage = JSON.parse(event.data);
      setLastMessage(message);

      // Handle Phase 2 events
      switch (message.type) {
        case 'orchestration_decision':
          queryClient.invalidateQueries(['orchestrator', 'decisions']);
          queryClient.invalidateQueries(['orchestrator', 'stats']);
          break;

        case 'conflict_resolved':
          queryClient.invalidateQueries(['conflicts', 'history']);
          queryClient.invalidateQueries(['conflicts', 'stats']);
          break;

        case 'phase_transition':
          queryClient.invalidateQueries(['deployment', 'status']);
          queryClient.invalidateQueries(['deployment', 'history']);
          toast.info(`Phase transition: ${message.data.from_phase} → ${message.data.to_phase}`);
          break;

        case 'drift_alert':
          queryClient.invalidateQueries(['drift', 'status']);
          queryClient.invalidateQueries(['drift', 'history']);
          if (message.data.drift_level === 'critical') {
            toast.error(message.data.message);
          } else if (message.data.drift_level === 'warning') {
            toast.warning(message.data.message);
          }
          break;

        case 'version_commit':
          queryClient.invalidateQueries(['versions', 'list']);
          toast.info(`New version committed: v${message.data.version_id}`);
          break;
      }
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      setIsConnected(false);
      // Reconnect after 3 seconds
      setTimeout(() => {
        // Trigger re-render to create new connection
      }, 3000);
    };

    return () => {
      ws.close();
    };
  }, [queryClient]);

  return { lastMessage, isConnected };
}
```

---

### App.tsx Integration

**Location:** `gui/desktop/src/App.tsx`

**Add 5 new tabs:**

```typescript
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { OrchestrationPanel } from '@/components/OrchestrationPanel';
import { ConflictPanel } from '@/components/ConflictPanel';
import { DeploymentPanel } from '@/components/DeploymentPanel';
import { DriftPanel } from '@/components/DriftPanel';
import { VersionsPanel } from '@/components/VersionsPanel';

export default function App() {
  return (
    <Tabs defaultValue="dashboard" className="h-screen">
      <TabsList>
        {/* Existing tabs */}
        <TabsTrigger value="dashboard">Dashboard</TabsTrigger>
        <TabsTrigger value="terminal">Terminal</TabsTrigger>
        <TabsTrigger value="metrics">Metrics</TabsTrigger>
        <TabsTrigger value="logs">Logs</TabsTrigger>
        <TabsTrigger value="llm">LLM</TabsTrigger>
        <TabsTrigger value="autonomy">Autonomy</TabsTrigger>

        {/* NEW: Phase 2 tabs */}
        <TabsTrigger value="orchestration">Orchestration</TabsTrigger>
        <TabsTrigger value="conflicts">Conflicts</TabsTrigger>
        <TabsTrigger value="deployment">Deployment</TabsTrigger>
        <TabsTrigger value="drift">Drift</TabsTrigger>
        <TabsTrigger value="versions">Versions</TabsTrigger>
      </TabsList>

      {/* Existing panels */}
      <TabsContent value="dashboard"><DashboardPanel /></TabsContent>
      <TabsContent value="terminal"><TerminalPanel /></TabsContent>
      {/* ... */}

      {/* NEW: Phase 2 panels */}
      <TabsContent value="orchestration"><OrchestrationPanel /></TabsContent>
      <TabsContent value="conflicts"><ConflictPanel /></TabsContent>
      <TabsContent value="deployment"><DeploymentPanel /></TabsContent>
      <TabsContent value="drift"><DriftPanel /></TabsContent>
      <TabsContent value="versions"><VersionsPanel /></TabsContent>
    </Tabs>
  );
}
```

---

## Implementation Phases

### Phase 1: Kernel Shell Commands (2 weeks)

**Week 1: Core Commands**

**Tasks:**
1. Implement `coordctl` commands (status, history)
2. Implement `agentctl` commands (list)
3. Implement `deployctl` commands (status, history, advance, rollback, config)
4. Add history tracking to orchestrator and deployment manager
5. Unit tests for command parsing

**Deliverables:**
- 10 commands functional
- JSON output working
- Integration with Phase 2 modules

**Week 2: Drift & Version Commands**

**Tasks:**
1. Implement `driftctl` commands (status, history, retrain, reset-baseline)
2. Implement `versionctl` commands (list, commit, rollback, diff, tag, gc)
3. Implement conflict resolution commands (conflict-stats, conflict-history, priorities)
4. Add history tracking to drift detector and conflict resolver
5. Integration tests

**Deliverables:**
- All 25 commands functional
- Comprehensive testing
- Documentation

---

### Phase 2: Frontend Core Components (3 weeks)

**Week 3: Orchestration & Conflicts Panels**

**Tasks:**
1. Create OrchestrationPanel.tsx with stats, agents, decisions
2. Create ConflictPanel.tsx with priority table, conflict history
3. Add supporting components (AgentCard, ConflictVisualization, etc.)
4. Integrate with API endpoints
5. Add WebSocket event handlers for orchestration and conflicts

**Deliverables:**
- OrchestrationPanel fully functional
- ConflictPanel fully functional
- Real-time updates working

**Week 4: Deployment & Drift Panels**

**Tasks:**
1. Create DeploymentPanel.tsx with phase timeline, history
2. Create DriftPanel.tsx with status, chart, history
3. Add supporting components (PhaseTimeline, DriftChart, etc.)
4. Add manual control actions (advance, rollback, retrain)
5. Add WebSocket event handlers for deployment and drift

**Deliverables:**
- DeploymentPanel fully functional
- DriftPanel fully functional
- Manual controls working

**Week 5: Versions Panel**

**Tasks:**
1. Create VersionsPanel.tsx with version tree
2. Add version comparison functionality
3. Add commit dialog
4. Add rollback/tag/GC actions
5. Add WebSocket event handler for version commits

**Deliverables:**
- VersionsPanel fully functional
- Version tree visualization working
- All version control operations functional

---

### Phase 3: Integration (1 week)

**Week 6: API Client & WebSocket**

**Tasks:**
1. Complete API client methods (22 endpoints)
2. Update WebSocket hook to handle 5 new event types
3. Add TypeScript types for all Phase 2 data structures
4. Test real-time updates end-to-end
5. Error handling and loading states

**Deliverables:**
- Complete API integration
- WebSocket events flowing correctly
- Type-safe TypeScript throughout

---

### Phase 4: Testing (1 week)

**Week 7: E2E Tests**

**Tasks:**
1. Write Playwright E2E tests for 5 panels
2. Test WebSocket event handling
3. Test API error scenarios
4. Test manual control actions
5. Performance testing (large datasets)

**Deliverables:**
- E2E test suite with 50+ test cases
- All tests passing
- Test coverage report

---

### Phase 5: Polish & Documentation (1 week)

**Week 8: UX Polish + Docs**

**Tasks:**
1. Add loading states, error boundaries
2. Add keyboard shortcuts
3. Add tooltips and help text
4. Write user documentation
5. Create demo video

**Deliverables:**
- Polished UI with smooth interactions
- User documentation (README updates)
- Demo video showing all features

---

## Testing Strategy

### Kernel Unit Tests

**Location:** `crates/kernel/src/shell.rs` (inline tests)

**Coverage:**
- Command parsing
- JSON output formatting
- Integration with Phase 2 modules
- Error handling

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordctl_status_json() {
        let output = capture_output(|| {
            handle_coordctl(&["status", "--json"]);
        });

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(json["total_decisions"].is_number());
        assert!(json["unanimous"].is_number());
    }

    #[test]
    fn test_versionctl_commit() {
        let output = capture_output(|| {
            handle_versionctl(&["commit", "-m", "Test version", "--json"]);
        });

        let json: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json["success"], true);
        assert!(json["version_id"].is_number());
    }
}
```

### Frontend E2E Tests

**Location:** `gui/desktop/e2e/`

**Coverage:**
- Panel rendering
- Real-time updates via WebSocket
- API interactions
- Manual control actions
- Error handling

**Example:**
```typescript
import { test, expect } from '@playwright/test';

test('orchestration panel shows agent decisions', async ({ page }) => {
  await page.goto('/');
  await page.click('[data-testid="orchestration-tab"]');

  // Wait for stats to load
  await expect(page.locator('[data-testid="total-decisions"]')).toBeVisible();

  // Check agents are displayed
  await expect(page.locator('[data-testid="agent-card"]')).toHaveCount(5);

  // Check decision table renders
  await expect(page.locator('[data-testid="decision-row"]').first()).toBeVisible();
});

test('deployment panel allows manual advance', async ({ page }) => {
  await page.goto('/');
  await page.click('[data-testid="deployment-tab"]');

  // Wait for status to load
  await expect(page.locator('[data-testid="current-phase"]')).toBeVisible();

  // Click advance button
  await page.click('[data-testid="advance-button"]');

  // Wait for confirmation
  await expect(page.locator('[data-testid="phase-transition-toast"]')).toBeVisible();

  // Check phase changed
  await expect(page.locator('[data-testid="current-phase"]')).toContainText('Phase C');
});

test('drift panel shows real-time alerts', async ({ page }) => {
  await page.goto('/');
  await page.click('[data-testid="drift-tab"]');

  // Wait for WebSocket connection
  await page.waitForTimeout(1000);

  // Trigger drift in kernel (via API)
  // ...

  // Check drift alert appears
  await expect(page.locator('[data-testid="drift-alert"]')).toBeVisible();
  await expect(page.locator('[data-testid="drift-level"]')).toContainText('Warning');
});

test('versions panel shows git-like history', async ({ page }) => {
  await page.goto('/');
  await page.click('[data-testid="versions-tab"]');

  // Wait for version list to load
  await expect(page.locator('[data-testid="version-card"]').first()).toBeVisible();

  // Check current version is marked
  await expect(page.locator('[data-testid="version-card"][data-current="true"]')).toHaveCount(1);

  // Check version tree renders
  await expect(page.locator('[data-testid="version-tree"]')).toBeVisible();
});
```

---

## Success Criteria

### Functional Requirements

✅ **Kernel Commands:**
- [ ] All 25 commands implemented
- [ ] JSON output for all commands
- [ ] Integration with Phase 2 modules
- [ ] Error handling for invalid inputs

✅ **Orchestration Panel:**
- [ ] Display orchestration statistics
- [ ] Show all active agents with status
- [ ] Display recent coordinated decisions (last 100)
- [ ] Update in real-time via WebSocket
- [ ] Virtual scrolling for large decision lists

✅ **Conflict Panel:**
- [ ] Display conflict statistics
- [ ] Show agent priority table
- [ ] Display recent conflicts (last 100)
- [ ] Show conflict detail view
- [ ] Update in real-time via WebSocket

✅ **Deployment Panel:**
- [ ] Display current deployment phase
- [ ] Show phase timeline (A → B → C → D)
- [ ] Display phase transition history
- [ ] Manual advance/rollback controls
- [ ] Configuration form for auto-advance/rollback
- [ ] Update in real-time via WebSocket

✅ **Drift Panel:**
- [ ] Display current drift status
- [ ] Show accuracy chart over time
- [ ] Display drift history
- [ ] Manual retrain button
- [ ] Reset baseline button
- [ ] Configuration form for auto-retrain
- [ ] Update in real-time via WebSocket

✅ **Versions Panel:**
- [ ] Display version history tree (Git-like)
- [ ] Show version metadata
- [ ] Version comparison (diff)
- [ ] Commit new version dialog
- [ ] Rollback to version button
- [ ] Tag version button
- [ ] Garbage collection button
- [ ] Update in real-time via WebSocket

### Non-Functional Requirements

✅ **Performance:**
- [ ] Kernel command execution < 100ms
- [ ] API response time < 500ms
- [ ] WebSocket event latency < 100ms
- [ ] UI render time < 16ms (60fps)
- [ ] No memory leaks in long-running sessions

✅ **Reliability:**
- [ ] WebSocket auto-reconnect on disconnect
- [ ] Polling fallback if WebSocket fails
- [ ] Error boundaries prevent full page crashes
- [ ] Graceful degradation on API errors
- [ ] Command validation in kernel

✅ **Usability:**
- [ ] Loading states for all async operations
- [ ] Error messages are user-friendly
- [ ] Keyboard navigation works
- [ ] Tooltips explain all features
- [ ] Responsive design on different screen sizes

✅ **Testing:**
- [ ] Kernel unit test coverage > 80%
- [ ] Frontend E2E test coverage > 70%
- [ ] All tests pass in CI/CD pipeline
- [ ] No flaky tests

---

## Technical Specifications

### Kernel Command Summary

| Domain | Commands | Lines (Est.) |
|--------|----------|--------------|
| Orchestration | `coordctl status`, `coordctl history`, `agentctl list` | 300 |
| Conflicts | `coordctl conflict-stats`, `coordctl conflict-history`, `coordctl priorities` | 250 |
| Deployment | `deployctl status`, `deployctl history`, `deployctl advance`, `deployctl rollback`, `deployctl config` | 400 |
| Drift | `driftctl status`, `driftctl history`, `driftctl retrain`, `driftctl reset-baseline` | 300 |
| Versions | `versionctl list`, `versionctl commit`, `versionctl rollback`, `versionctl diff`, `versionctl tag`, `versionctl gc` | 350 |

**Total:** 25 commands, ~1,600 lines Rust

### Frontend Component Summary

| Component | Purpose | Lines (Est.) |
|-----------|---------|--------------|
| OrchestrationPanel.tsx | Main orchestration panel | 300 |
| ConflictPanel.tsx | Main conflict panel | 350 |
| DeploymentPanel.tsx | Main deployment panel | 400 |
| DriftPanel.tsx | Main drift panel | 350 |
| VersionsPanel.tsx | Main versions panel | 450 |
| 20+ Supporting Components | Cards, tables, charts, dialogs | 1,670 |

**Total:** 25+ components, ~3,520 lines TypeScript

### API Client Summary

| Domain | Methods | Lines (Est.) |
|--------|---------|--------------|
| orchestratorApi | getStats, getDecisions, getAgents | 50 |
| conflictsApi | getStats, getHistory, getPriorityTable | 50 |
| deploymentApi | getStatus, getHistory, advance, rollback, updateConfig | 80 |
| driftApi | getStatus, getHistory, triggerRetrain, resetBaseline | 60 |
| versionsApi | getList, commit, rollback, getDiff, tag, gc | 90 |

**Total:** ~330 lines TypeScript

---

## File Structure

### Kernel Files

```
crates/kernel/src/
├── shell.rs                    # UPDATED: Add 25 new commands
│   ├── handle_coordctl()       # NEW: Orchestration commands
│   ├── handle_agentctl()       # NEW: Agent commands
│   ├── handle_deployctl()      # NEW: Deployment commands
│   ├── handle_driftctl()       # NEW: Drift commands
│   └── handle_versionctl()     # NEW: Version control commands
│
├── ai/
│   ├── orchestrator.rs         # UPDATED: Add decision history + agent tracking
│   ├── conflict.rs             # UPDATED: Add conflict history
│   └── deployment.rs           # UPDATED: Add transition history + manual controls
│
└── llm/
    ├── drift_detector.rs       # UPDATED: Add drift history + manual controls
    └── version.rs              # Already has necessary methods ✅
```

### Frontend Files

```
gui/desktop/src/
├── App.tsx                     # UPDATED: Add 5 new tabs
│
├── components/
│   ├── OrchestrationPanel.tsx  # NEW: Orchestration panel
│   ├── ConflictPanel.tsx       # NEW: Conflict panel
│   ├── DeploymentPanel.tsx     # NEW: Deployment panel
│   ├── DriftPanel.tsx          # NEW: Drift panel
│   ├── VersionsPanel.tsx       # NEW: Versions panel
│   │
│   ├── orchestration/          # NEW: Orchestration components
│   │   ├── OrchestrationStatsCard.tsx
│   │   ├── AgentCard.tsx
│   │   ├── DecisionHistoryTable.tsx
│   │   └── AgentStatusBadge.tsx
│   │
│   ├── conflicts/              # NEW: Conflict components
│   │   ├── ConflictStatsCard.tsx
│   │   ├── PriorityTable.tsx
│   │   ├── ConflictHistoryTable.tsx
│   │   ├── ConflictDetailCard.tsx
│   │   └── ConflictVisualization.tsx
│   │
│   ├── deployment/             # NEW: Deployment components
│   │   ├── CurrentPhaseCard.tsx
│   │   ├── PhaseTimeline.tsx
│   │   ├── PhaseDefinitionsTable.tsx
│   │   ├── TransitionHistoryTable.tsx
│   │   └── DeploymentMetricsChart.tsx
│   │
│   ├── drift/                  # NEW: Drift components
│   │   ├── DriftStatusCard.tsx
│   │   ├── DriftLevelIndicator.tsx
│   │   ├── DriftChart.tsx
│   │   ├── DriftHistoryTable.tsx
│   │   └── DriftConfigCard.tsx
│   │
│   └── versions/               # NEW: Version components
│       ├── VersionHeader.tsx
│       ├── VersionTree.tsx
│       ├── VersionCard.tsx
│       ├── VersionDiffCard.tsx
│       ├── CommitDialog.tsx
│       └── TagBadge.tsx
│
├── lib/
│   ├── api.ts                  # UPDATED: Add Phase 2 API methods
│   ├── useWebSocket.ts         # UPDATED: Handle 5 new event types
│   └── types/                  # NEW: Type definitions
│       ├── orchestration.ts
│       ├── conflicts.ts
│       ├── deployment.ts
│       ├── drift.ts
│       └── versions.ts
│
└── e2e/
    ├── orchestration.spec.ts   # NEW: Orchestration E2E tests
    ├── conflicts.spec.ts       # NEW: Conflict E2E tests
    ├── deployment.spec.ts      # NEW: Deployment E2E tests
    ├── drift.spec.ts           # NEW: Drift E2E tests
    └── versions.spec.ts        # NEW: Versions E2E tests
```

---

## Code Examples

### Kernel Command Example (Full Implementation)

```rust
// crates/kernel/src/shell.rs

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use crate::ai::orchestrator::ORCHESTRATOR;
use crate::ai::conflict::CONFLICT_RESOLVER;
use crate::ai::deployment::DEPLOYMENT_MANAGER;
use crate::llm::drift_detector::DRIFT_DETECTOR;
use crate::llm::version::VERSION_CONTROL;

/// Handle coordctl command
pub fn handle_coordctl(args: &[&str]) {
    if args.is_empty() {
        print_coordctl_help();
        return;
    }

    match args[0] {
        "status" => handle_coordctl_status(&args[1..]),
        "history" => handle_coordctl_history(&args[1..]),
        "conflict-stats" => handle_coordctl_conflict_stats(&args[1..]),
        "conflict-history" => handle_coordctl_conflict_history(&args[1..]),
        "priorities" => handle_coordctl_priorities(&args[1..]),
        _ => println!("Unknown coordctl subcommand: {}", args[0]),
    }
}

fn handle_coordctl_status(args: &[&str]) {
    let json_mode = args.contains(&"--json");
    let stats = ORCHESTRATOR.get_stats();

    if json_mode {
        // JSON output
        println!("{{");
        println!("  \"total_decisions\": {},", stats.total_decisions);
        println!("  \"unanimous\": {},", stats.unanimous);
        println!("  \"majority\": {},", stats.majority);
        println!("  \"safety_overrides\": {},", stats.safety_overrides);
        println!("  \"no_consensus\": {},", stats.no_consensus);
        println!("  \"avg_latency_us\": {}", stats.avg_latency_us);
        println!("}}");
    } else {
        // Human-readable output
        println!("Orchestration Statistics:");
        println!("  Total Decisions: {}", stats.total_decisions);

        if stats.total_decisions > 0 {
            let total = stats.total_decisions as f64;
            println!("  Unanimous: {} ({:.1}%)",
                stats.unanimous,
                (stats.unanimous as f64 / total) * 100.0
            );
            println!("  Majority: {} ({:.1}%)",
                stats.majority,
                (stats.majority as f64 / total) * 100.0
            );
            println!("  Safety Overrides: {} ({:.1}%)",
                stats.safety_overrides,
                (stats.safety_overrides as f64 / total) * 100.0
            );
            println!("  No Consensus: {} ({:.1}%)",
                stats.no_consensus,
                (stats.no_consensus as f64 / total) * 100.0
            );
        }

        println!("  Avg Latency: {} μs", stats.avg_latency_us);
    }
}

fn handle_coordctl_history(args: &[&str]) {
    let limit = parse_arg_u32(args, "--limit").unwrap_or(100);
    let json_mode = args.contains(&"--json");

    let decisions = ORCHESTRATOR.get_decision_history(limit as usize);

    if json_mode {
        // JSON array output
        println!("[");
        for (i, decision) in decisions.iter().enumerate() {
            println!("  {{");
            println!("    \"timestamp\": \"{}\",", decision.timestamp);
            println!("    \"type\": \"{}\",", decision.decision_type);
            println!("    \"action\": \"{}\",", decision.action);
            if let Some(conf) = decision.confidence {
                println!("    \"confidence\": {},", conf);
            }
            println!("    \"agents\": {:?},", decision.agents);
            println!("    \"latency_us\": {}", decision.latency_us);
            if i < decisions.len() - 1 {
                println!("  }},");
            } else {
                println!("  }}");
            }
        }
        println!("]");
    } else {
        // Human-readable output
        println!("Recent Decisions (last {}):", decisions.len());
        for decision in decisions {
            println!("  [{}] {} - {} ({} agents, {} μs)",
                decision.timestamp,
                decision.decision_type,
                decision.action,
                decision.agents.len(),
                decision.latency_us
            );
        }
    }
}

/// Parse --arg=value or --arg value
fn parse_arg_u32(args: &[&str], key: &str) -> Option<u32> {
    for (i, arg) in args.iter().enumerate() {
        if arg.starts_with(key) {
            if let Some(val) = arg.strip_prefix(&format!("{}=", key)) {
                return val.parse().ok();
            } else if i + 1 < args.len() {
                return args[i + 1].parse().ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordctl_status_json() {
        // Test would capture stdout and verify JSON structure
    }
}
```

---

### Frontend Component Example (Full Implementation)

```typescript
// gui/desktop/src/components/OrchestrationPanel.tsx

import React from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import { orchestratorApi } from '@/lib/api';
import { useWebSocket } from '@/lib/useWebSocket';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface OrchestrationStats {
  total_decisions: number;
  unanimous: number;
  majority: number;
  safety_overrides: number;
  no_consensus: number;
  avg_latency_us: number;
}

interface CoordinatedDecision {
  timestamp: string;
  type: 'unanimous' | 'majority' | 'safety_override' | 'no_consensus';
  action: string;
  confidence?: number;
  agents: string[];
  latency_us: number;
}

interface Agent {
  name: string;
  type: string;
  status: 'active' | 'inactive' | 'error';
  priority: number;
  last_decision: {
    timestamp: string;
    action: string;
    confidence: number;
  };
  stats: {
    total_decisions: number;
    avg_confidence: number;
  };
}

export function OrchestrationPanel() {
  const queryClient = useQueryClient();
  const parentRef = React.useRef<HTMLDivElement>(null);

  // Fetch orchestration stats (polling every 5s)
  const { data: stats, isLoading: statsLoading } = useQuery<OrchestrationStats>({
    queryKey: ['orchestrator', 'stats'],
    queryFn: orchestratorApi.getStats,
    refetchInterval: 5000,
  });

  // Fetch active agents (polling every 5s)
  const { data: agentsData, isLoading: agentsLoading } = useQuery<{ agents: Agent[] }>({
    queryKey: ['orchestrator', 'agents'],
    queryFn: orchestratorApi.getAgents,
    refetchInterval: 5000,
  });

  // Fetch recent decisions (polling every 2s)
  const { data: decisionsData, isLoading: decisionsLoading } = useQuery<{ decisions: CoordinatedDecision[] }>({
    queryKey: ['orchestrator', 'decisions'],
    queryFn: () => orchestratorApi.getDecisions(100),
    refetchInterval: 2000,
  });

  // Listen for real-time orchestration decisions
  const { lastMessage } = useWebSocket();

  React.useEffect(() => {
    if (lastMessage?.type === 'orchestration_decision') {
      queryClient.invalidateQueries(['orchestrator', 'decisions']);
      queryClient.invalidateQueries(['orchestrator', 'stats']);
    }
  }, [lastMessage, queryClient]);

  // Virtual scrolling for decision table
  const rowVirtualizer = useVirtualizer({
    count: decisionsData?.decisions.length || 0,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60,
    overscan: 10,
  });

  const getDecisionTypeBadge = (type: CoordinatedDecision['type']) => {
    const variants = {
      unanimous: 'default',
      majority: 'secondary',
      safety_override: 'destructive',
      no_consensus: 'outline',
    } as const;
    return variants[type];
  };

  if (statsLoading) {
    return <div className="flex items-center justify-center h-full">Loading...</div>;
  }

  return (
    <div className="space-y-6 p-6">
      {/* Orchestration Statistics */}
      <Card>
        <CardHeader>
          <CardTitle>Orchestration Statistics</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-6 gap-4">
            <div>
              <div className="text-2xl font-bold">{stats?.total_decisions || 0}</div>
              <div className="text-sm text-muted-foreground">Total Decisions</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-green-600">
                {stats?.unanimous || 0}
              </div>
              <div className="text-sm text-muted-foreground">
                Unanimous ({stats?.total_decisions ? ((stats.unanimous / stats.total_decisions) * 100).toFixed(1) : 0}%)
              </div>
            </div>
            <div>
              <div className="text-2xl font-bold text-blue-600">
                {stats?.majority || 0}
              </div>
              <div className="text-sm text-muted-foreground">
                Majority ({stats?.total_decisions ? ((stats.majority / stats.total_decisions) * 100).toFixed(1) : 0}%)
              </div>
            </div>
            <div>
              <div className="text-2xl font-bold text-red-600">
                {stats?.safety_overrides || 0}
              </div>
              <div className="text-sm text-muted-foreground">
                Safety Overrides ({stats?.total_decisions ? ((stats.safety_overrides / stats.total_decisions) * 100).toFixed(1) : 0}%)
              </div>
            </div>
            <div>
              <div className="text-2xl font-bold text-yellow-600">
                {stats?.no_consensus || 0}
              </div>
              <div className="text-sm text-muted-foreground">
                No Consensus ({stats?.total_decisions ? ((stats.no_consensus / stats.total_decisions) * 100).toFixed(1) : 0}%)
              </div>
            </div>
            <div>
              <div className="text-2xl font-bold">{stats?.avg_latency_us || 0} μs</div>
              <div className="text-sm text-muted-foreground">Avg Latency</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Active Agents */}
      <Card>
        <CardHeader>
          <CardTitle>Active Agents</CardTitle>
        </CardHeader>
        <CardContent>
          {agentsLoading ? (
            <div>Loading agents...</div>
          ) : (
            <div className="grid grid-cols-3 gap-4">
              {agentsData?.agents.map((agent) => (
                <Card key={agent.name}>
                  <CardHeader>
                    <CardTitle className="text-base">{agent.name}</CardTitle>
                    <Badge variant={agent.status === 'active' ? 'default' : 'secondary'}>
                      {agent.status}
                    </Badge>
                  </CardHeader>
                  <CardContent className="space-y-2">
                    <div className="text-sm">
                      <span className="font-medium">Priority:</span> {agent.priority}
                    </div>
                    <div className="text-sm">
                      <span className="font-medium">Last Decision:</span> {agent.last_decision.action}
                    </div>
                    <div className="text-sm">
                      <span className="font-medium">Confidence:</span>{' '}
                      {(agent.last_decision.confidence * 100).toFixed(0)}%
                    </div>
                    <div className="text-sm">
                      <span className="font-medium">Total Decisions:</span> {agent.stats.total_decisions}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Recent Decisions Table */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Decisions</CardTitle>
        </CardHeader>
        <CardContent>
          {decisionsLoading ? (
            <div>Loading decisions...</div>
          ) : (
            <div ref={parentRef} className="h-96 overflow-auto">
              <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, position: 'relative' }}>
                {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                  const decision = decisionsData?.decisions[virtualRow.index];
                  if (!decision) return null;

                  return (
                    <div
                      key={virtualRow.index}
                      style={{
                        position: 'absolute',
                        top: 0,
                        left: 0,
                        width: '100%',
                        height: `${virtualRow.size}px`,
                        transform: `translateY(${virtualRow.start}px)`,
                      }}
                      className="flex items-center gap-4 border-b px-4 py-2"
                    >
                      <div className="text-sm text-muted-foreground w-32">
                        {new Date(decision.timestamp).toLocaleTimeString()}
                      </div>
                      <Badge variant={getDecisionTypeBadge(decision.type)}>
                        {decision.type.replace('_', ' ')}
                      </Badge>
                      <div className="font-medium">{decision.action}</div>
                      {decision.confidence && (
                        <div className="text-sm text-muted-foreground">
                          {(decision.confidence * 100).toFixed(0)}% confidence
                        </div>
                      )}
                      <div className="text-sm text-muted-foreground">
                        {decision.agents.length} agents
                      </div>
                      <div className="text-sm text-muted-foreground ml-auto">
                        {decision.latency_us} μs
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
```

---

## Appendix: Dependencies

### New Frontend Dependencies

Add to `gui/desktop/package.json`:

```json
{
  "dependencies": {
    "react-flow-renderer": "^10.3.17",
    "sonner": "^1.2.0"
  }
}
```

**react-flow-renderer:** For phase timeline and version tree visualization
**sonner:** For toast notifications (drift alerts, phase transitions)

---

## Document Control

**Version History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-11-09 | Claude Code | Initial comprehensive plan |

**Approval:**

- [ ] Technical Lead: _________________
- [ ] Product Manager: _________________
- [ ] QA Lead: _________________

**Related Documents:**

- Phase 2 AI Governance Implementation Plan (docs/AI-ML-KERNEL-IMPLEMENTATION-PLAN.md)
- Phase 2 GUI Backend Implementation Plan (docs/plans/PHASE2-GUI-IMPLEMENTATION-PLAN.md)
- Phase 2 Test Report (docs/results/PHASE2-TEST-REPORT.md)
- Phase 2 GUI Backend Implementation Report (docs/results/PHASE2-GUI-BACKEND-IMPLEMENTATION.md)

---

**END OF DOCUMENT**
