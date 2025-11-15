# Phase 2 GUI Implementation Plan
## AI Governance Visualization & Control Plane

**Document Version:** 1.0
**Date:** November 9, 2025
**Target Completion:** Q1 2026
**Status:** 🔴 PLANNING

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Analysis](#current-state-analysis)
3. [Architecture Overview](#architecture-overview)
4. [Backend Requirements (sisctl daemon)](#backend-requirements-sisctl-daemon)
5. [Frontend Components Specification](#frontend-components-specification)
6. [WebSocket Events Specification](#websocket-events-specification)
7. [Implementation Phases](#implementation-phases)
8. [Testing Strategy](#testing-strategy)
9. [Success Criteria](#success-criteria)
10. [Technical Specifications](#technical-specifications)
11. [File Structure](#file-structure)
12. [Code Examples](#code-examples)

---

## Executive Summary

### Objective

Implement comprehensive GUI visualizations for Phase 2 AI Governance features in the SIS Kernel, providing real-time visibility and control over multi-agent orchestration, conflict resolution, deployment management, model drift detection, and adapter version control.

### Scope

**Backend (Rust):**
- 15 new API endpoints in sisctl daemon
- 5 new WebSocket event types
- Integration with Phase 2 kernel modules

**Frontend (React + TypeScript):**
- 5 new panels (Orchestration, Conflicts, Deployment, Drift, Versions)
- 20+ new React components
- Real-time data visualization with charts and state machines
- ~3,000 lines of TypeScript code

### Impact

- **User Experience**: Full visibility into AI governance decisions
- **Debugging**: Real-time conflict detection and resolution visualization
- **Operations**: Manual override controls for deployment phases
- **Monitoring**: Drift detection alerts and model performance tracking
- **Version Control**: Git-like interface for adapter management

### Timeline

- **Phase 1 (Backend)**: 2 weeks - API endpoints + WebSocket events
- **Phase 2 (Frontend)**: 3 weeks - Core panels and components
- **Phase 3 (Testing)**: 1 week - E2E tests and integration
- **Phase 4 (Polish)**: 1 week - UX refinement and documentation

**Total:** 7 weeks

---

## Current State Analysis

### What Exists (Phase 1 GUI)

✅ **Fully Implemented:**
- LLM Inference panel with token streaming
- Autonomy decision control with audit log
- Decision explanation with attention weights
- What-If scenario simulator
- Metrics visualization with time-range queries
- Terminal with xterm.js
- Crash monitoring with incident workflow
- Graph (M4) creation and management
- Scheduling workload tuning
- Memory approval workflow

### What's Missing (Phase 2 GUI)

❌ **Not Implemented:**
- Multi-agent orchestration dashboard
- Conflict resolution visualization
- Deployment phase management UI
- Model drift detection panel
- Adapter version control interface

### Technical Debt

- No WebSocket events for Phase 2 governance events
- No API endpoints to query orchestration stats
- No conflict history tracking in daemon
- No deployment phase state management
- No drift metrics aggregation
- No version control API

---

## Architecture Overview

### High-Level Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         SIS Kernel                              │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Phase 2 AI Governance Modules                            │  │
│  │  - AgentOrchestrator                                     │  │
│  │  - ConflictResolver                                      │  │
│  │  - DeploymentManager                                     │  │
│  │  - DriftDetector                                         │  │
│  │  - AdapterVersionControl                                 │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            ↓ Shell Commands                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Shell Interface                                          │  │
│  │  agentctl, coordctl, driftctl, versionctl               │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ VirtIO Console
┌─────────────────────────────────────────────────────────────────┐
│                      sisctl Daemon (Rust)                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ New API Endpoints (15 endpoints)                         │  │
│  │  /api/v1/orchestrator/*                                  │  │
│  │  /api/v1/conflicts/*                                     │  │
│  │  /api/v1/deployment/*                                    │  │
│  │  /api/v1/drift/*                                         │  │
│  │  /api/v1/versions/*                                      │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ WebSocket Event Bus                                      │  │
│  │  - orchestration_decision                                │  │
│  │  - conflict_resolved                                     │  │
│  │  - phase_transition                                      │  │
│  │  - drift_alert                                           │  │
│  │  - version_commit                                        │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↓ HTTP + WebSocket
┌─────────────────────────────────────────────────────────────────┐
│                  GUI Frontend (React + TypeScript)              │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ New Panels (5 panels)                                    │  │
│  │  - OrchestrationPanel.tsx                                │  │
│  │  - ConflictPanel.tsx                                     │  │
│  │  - DeploymentPanel.tsx                                   │  │
│  │  - DriftPanel.tsx                                        │  │
│  │  - VersionsPanel.tsx                                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Supporting Components (20+ components)                   │  │
│  │  - AgentCard, ConflictVisualization, PhaseTimeline,     │  │
│  │    DriftChart, VersionTree, etc.                        │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Real-time Updates                                        │  │
│  │  - useWebSocket hook (existing)                          │  │
│  │  - TanStack React Query polling                         │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Technology Stack

**Backend:**
- Language: Rust
- HTTP server: Axum (existing)
- WebSocket: tokio-tungstenite (existing)
- Serialization: serde_json
- Shell command parsing: Regex-based parsing of kernel output

**Frontend:**
- Framework: React 18 + TypeScript
- State management: TanStack React Query v5 + Zustand
- Charts: Recharts (existing) + react-flow (new - for state machines)
- WebSocket: Custom useWebSocket hook (existing)
- UI components: Radix UI + Tailwind CSS (existing)
- Icons: Lucide React (existing)
- Testing: Playwright (existing)

---

## Backend Requirements (sisctl daemon)

### 1. Orchestrator API Endpoints

#### `GET /api/v1/orchestrator/stats`

**Purpose:** Get orchestration statistics

**Shell Command:** `agentctl stats` (or parse from `coordctl status`)

**Response:**
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

**Implementation Notes:**
- Execute `coordctl status` via shell
- Parse output using regex to extract stats
- Cache for 1 second to reduce shell overhead

#### `GET /api/v1/orchestrator/decisions`

**Purpose:** Get recent coordinated decisions (last 100)

**Query Parameters:**
- `limit`: Max decisions to return (default: 100, max: 1000)
- `type`: Filter by decision type (unanimous, majority, safety_override, no_consensus)

**Shell Command:** `coordctl history --limit 100` (may need to add this command)

**Response:**
```json
{
  "decisions": [
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
}
```

**Implementation Notes:**
- Execute `coordctl history` via shell (need to add this command to kernel)
- Parse JSON-formatted output from kernel
- Maintain ring buffer in daemon (last 1000 decisions)
- Return filtered subset based on query params

#### `GET /api/v1/orchestrator/agents`

**Purpose:** Get status of all agents

**Shell Command:** `agentctl list` or `agentctl status`

**Response:**
```json
{
  "agents": [
    {
      "name": "CrashPredictor",
      "type": "crash_predictor",
      "status": "active",
      "priority": 100,
      "last_decision": {
        "timestamp": "2025-11-09T10:23:45.123456789Z",
        "action": "Stop",
        "confidence": 0.95
      },
      "stats": {
        "total_decisions": 234,
        "avg_confidence": 0.87
      }
    },
    {
      "name": "StateInference",
      "type": "state_inference",
      "status": "active",
      "priority": 80,
      "last_decision": {
        "timestamp": "2025-11-09T10:23:45.123456789Z",
        "action": "CompactMemory",
        "confidence": 0.92
      },
      "stats": {
        "total_decisions": 1543,
        "avg_confidence": 0.78
      }
    }
  ]
}
```

### 2. Conflict Resolution API Endpoints

#### `GET /api/v1/conflicts/stats`

**Purpose:** Get conflict resolution statistics

**Shell Command:** `coordctl conflict-stats`

**Response:**
```json
{
  "total_conflicts": 89,
  "resolved_by_priority": 67,
  "resolved_by_voting": 18,
  "unresolved": 4,
  "avg_resolution_time_us": 145
}
```

#### `GET /api/v1/conflicts/history`

**Purpose:** Get recent conflicts (last 100)

**Query Parameters:**
- `limit`: Max conflicts to return (default: 100)
- `resolved`: Filter by resolution status (true/false)

**Shell Command:** `coordctl conflict-history --limit 100`

**Response:**
```json
{
  "conflicts": [
    {
      "id": "conflict_1234567890",
      "timestamp": "2025-11-09T10:23:45.123456789Z",
      "agents": [
        {
          "agent": "CrashPredictor",
          "action": "Stop",
          "confidence": 0.95,
          "priority": 100
        },
        {
          "agent": "TransformerScheduler",
          "action": "ContinueNormal",
          "confidence": 0.80,
          "priority": 60
        }
      ],
      "resolution": {
        "strategy": "priority",
        "winner": "CrashPredictor",
        "action": "Stop",
        "reason": "Safety agent override (priority 100 > 60)"
      },
      "resolution_time_us": 156
    }
  ]
}
```

#### `GET /api/v1/conflicts/priority-table`

**Purpose:** Get agent priority table

**Shell Command:** `coordctl priorities`

**Response:**
```json
{
  "priorities": [
    {"agent": "CrashPredictor", "priority": 100},
    {"agent": "StateInference", "priority": 80},
    {"agent": "TransformerScheduler", "priority": 60},
    {"agent": "FineTuner", "priority": 40},
    {"agent": "Metrics", "priority": 20}
  ]
}
```

### 3. Deployment Management API Endpoints

#### `GET /api/v1/deployment/status`

**Purpose:** Get current deployment phase status

**Shell Command:** `deployctl status` (new command needed in kernel)

**Response:**
```json
{
  "current_phase": {
    "id": "B",
    "name": "Canary",
    "description": "Deploy to 10% of traffic",
    "entered_at": "2025-11-09T10:00:00.000000000Z",
    "min_duration_ms": 43200000,
    "elapsed_ms": 3600000,
    "can_advance": false,
    "traffic_percentage": 10,
    "error_rate": 0.02,
    "success_rate": 0.98
  },
  "auto_advance_enabled": true,
  "auto_rollback_enabled": true,
  "rollback_count": 1,
  "max_rollbacks": 3
}
```

#### `GET /api/v1/deployment/history`

**Purpose:** Get deployment phase transition history

**Shell Command:** `deployctl history --limit 50`

**Response:**
```json
{
  "transitions": [
    {
      "timestamp": "2025-11-09T10:00:00.000000000Z",
      "from_phase": "A",
      "to_phase": "B",
      "trigger": "auto_advance",
      "reason": "Metrics met: error_rate < 1%, duration > 24h",
      "metrics_snapshot": {
        "error_rate": 0.005,
        "success_rate": 0.995,
        "uptime_hours": 26.5
      }
    },
    {
      "timestamp": "2025-11-08T08:00:00.000000000Z",
      "from_phase": "B",
      "to_phase": "A",
      "trigger": "auto_rollback",
      "reason": "Error rate exceeded threshold: 5.2% > 5%",
      "metrics_snapshot": {
        "error_rate": 0.052,
        "success_rate": 0.948,
        "uptime_hours": 2.1
      }
    }
  ]
}
```

#### `POST /api/v1/deployment/advance`

**Purpose:** Manually advance to next phase

**Request Body:**
```json
{
  "force": false
}
```

**Shell Command:** `deployctl advance` or `deployctl advance --force`

**Response:**
```json
{
  "success": true,
  "old_phase": "B",
  "new_phase": "C",
  "timestamp": "2025-11-09T10:30:00.000000000Z"
}
```

#### `POST /api/v1/deployment/rollback`

**Purpose:** Manually rollback to previous phase

**Request Body:**
```json
{
  "reason": "User-initiated rollback due to unexpected behavior"
}
```

**Shell Command:** `deployctl rollback`

**Response:**
```json
{
  "success": true,
  "old_phase": "C",
  "new_phase": "B",
  "timestamp": "2025-11-09T10:30:00.000000000Z",
  "rollback_count": 2
}
```

#### `POST /api/v1/deployment/config`

**Purpose:** Update deployment configuration

**Request Body:**
```json
{
  "auto_advance_enabled": true,
  "auto_rollback_enabled": true,
  "error_rate_threshold": 0.05
}
```

**Shell Command:** `deployctl config --auto-advance=on --auto-rollback=on --error-threshold=0.05`

**Response:**
```json
{
  "success": true,
  "config": {
    "auto_advance_enabled": true,
    "auto_rollback_enabled": true,
    "error_rate_threshold": 0.05
  }
}
```

### 4. Drift Detection API Endpoints

#### `GET /api/v1/drift/status`

**Purpose:** Get current drift detection status

**Shell Command:** `driftctl status` (new command needed in kernel)

**Response:**
```json
{
  "baseline_accuracy": 0.92,
  "current_accuracy": 0.87,
  "accuracy_delta": -0.05,
  "drift_level": "warning",
  "sample_window_size": 100,
  "samples_analyzed": 1543,
  "last_retrain": "2025-11-08T10:00:00.000000000Z",
  "auto_retrain_enabled": true,
  "auto_retrain_threshold": 0.15
}
```

**Drift Levels:**
- `normal`: accuracy_delta < 0.05
- `warning`: 0.05 <= accuracy_delta < 0.15
- `critical`: accuracy_delta >= 0.15

#### `GET /api/v1/drift/history`

**Purpose:** Get drift detection history

**Query Parameters:**
- `limit`: Max entries to return (default: 100)
- `time_range`: Time range in seconds (default: 86400 = 24 hours)

**Shell Command:** `driftctl history --limit 100`

**Response:**
```json
{
  "samples": [
    {
      "timestamp": "2025-11-09T10:23:45.123456789Z",
      "accuracy": 0.87,
      "drift_level": "warning",
      "accuracy_delta": -0.05,
      "sample_count": 1543
    },
    {
      "timestamp": "2025-11-09T10:23:40.234567890Z",
      "accuracy": 0.89,
      "drift_level": "normal",
      "accuracy_delta": -0.03,
      "sample_count": 1542
    }
  ]
}
```

#### `POST /api/v1/drift/retrain`

**Purpose:** Manually trigger model retraining

**Request Body:**
```json
{
  "training_examples": 1000,
  "epochs": 10
}
```

**Shell Command:** `driftctl retrain --examples=1000 --epochs=10`

**Response:**
```json
{
  "success": true,
  "training_started": true,
  "timestamp": "2025-11-09T10:30:00.000000000Z",
  "estimated_duration_ms": 5000
}
```

#### `POST /api/v1/drift/reset-baseline`

**Purpose:** Reset baseline accuracy to current accuracy

**Shell Command:** `driftctl reset-baseline`

**Response:**
```json
{
  "success": true,
  "old_baseline": 0.92,
  "new_baseline": 0.87,
  "timestamp": "2025-11-09T10:30:00.000000000Z"
}
```

### 5. Version Control API Endpoints

#### `GET /api/v1/versions/list`

**Purpose:** Get adapter version history

**Query Parameters:**
- `limit`: Max versions to return (default: 10)

**Shell Command:** `versionctl list` or `versionctl log` (new commands needed in kernel)

**Response:**
```json
{
  "current_version": 5,
  "versions": [
    {
      "version_id": 5,
      "parent_version": 4,
      "timestamp": "2025-11-09T10:00:00.000000000Z",
      "description": "Trained on warehouse A failures",
      "metadata": {
        "training_examples": 1000,
        "training_duration_ms": 5000,
        "final_loss": 0.12,
        "accuracy_improvement": 0.03,
        "environment_tag": "warehouse_A"
      },
      "hash": "a3f5e9d2...",
      "storage_path": "v5_adapter.bin",
      "tags": ["stable"]
    },
    {
      "version_id": 4,
      "parent_version": 3,
      "timestamp": "2025-11-08T10:00:00.000000000Z",
      "description": "Baseline version",
      "metadata": {
        "training_examples": 500,
        "training_duration_ms": 2500,
        "final_loss": 0.15,
        "accuracy_improvement": 0.0,
        "environment_tag": "baseline"
      },
      "hash": "b2e4c8f1...",
      "storage_path": "v4_adapter.bin",
      "tags": []
    }
  ]
}
```

#### `POST /api/v1/versions/commit`

**Purpose:** Commit current adapter as new version

**Request Body:**
```json
{
  "description": "Fine-tuned for low-light conditions",
  "environment_tag": "low_light",
  "metadata": {
    "training_examples": 1200,
    "training_duration_ms": 6000,
    "final_loss": 0.10,
    "accuracy_improvement": 0.05
  }
}
```

**Shell Command:** `versionctl commit -m "Fine-tuned for low-light conditions" --env=low_light`

**Response:**
```json
{
  "success": true,
  "version_id": 6,
  "parent_version": 5,
  "timestamp": "2025-11-09T10:30:00.000000000Z"
}
```

#### `POST /api/v1/versions/rollback`

**Purpose:** Rollback to previous version

**Request Body:**
```json
{
  "version_id": 4
}
```

**Shell Command:** `versionctl rollback 4`

**Response:**
```json
{
  "success": true,
  "old_version": 5,
  "new_version": 4,
  "timestamp": "2025-11-09T10:30:00.000000000Z"
}
```

#### `GET /api/v1/versions/diff`

**Purpose:** Compare two adapter versions

**Query Parameters:**
- `v1`: First version ID
- `v2`: Second version ID

**Shell Command:** `versionctl diff 4 5`

**Response:**
```json
{
  "version_a": 4,
  "version_b": 5,
  "accuracy_delta": 0.03,
  "param_changes": 150,
  "time_delta_hours": 24
}
```

#### `POST /api/v1/versions/tag`

**Purpose:** Tag a version

**Request Body:**
```json
{
  "version_id": 5,
  "tag": "production"
}
```

**Shell Command:** `versionctl tag 5 production`

**Response:**
```json
{
  "success": true,
  "version_id": 5,
  "tag": "production",
  "timestamp": "2025-11-09T10:30:00.000000000Z"
}
```

#### `POST /api/v1/versions/gc`

**Purpose:** Garbage collect old versions

**Request Body:**
```json
{
  "keep_count": 10
}
```

**Shell Command:** `versionctl gc --keep=10`

**Response:**
```json
{
  "success": true,
  "removed_count": 5,
  "kept_count": 10,
  "timestamp": "2025-11-09T10:30:00.000000000Z"
}
```

---

## WebSocket Events Specification

### Event Format

All WebSocket events follow this structure:

```json
{
  "type": "event_type",
  "timestamp": "2025-11-09T10:23:45.123456789Z",
  "data": { /* event-specific data */ }
}
```

### 1. Orchestration Decision Event

**Event Type:** `orchestration_decision`

**Trigger:** When orchestrator makes a coordinated decision

**Data:**
```json
{
  "type": "orchestration_decision",
  "timestamp": "2025-11-09T10:23:45.123456789Z",
  "data": {
    "decision_type": "unanimous",
    "action": "CompactMemory",
    "confidence": 0.92,
    "agents": ["CrashPredictor", "StateInference", "TransformerScheduler"],
    "latency_us": 189,
    "description": "Unanimous: CompactMemory (confidence: 92.0%, 3 agents)"
  }
}
```

**Decision Types:**
- `unanimous`: All agents agree
- `majority`: >50% vote
- `safety_override`: Safety agent overrode others
- `no_consensus`: No agreement reached

### 2. Conflict Resolved Event

**Event Type:** `conflict_resolved`

**Trigger:** When a conflict is detected and resolved

**Data:**
```json
{
  "type": "conflict_resolved",
  "timestamp": "2025-11-09T10:23:45.123456789Z",
  "data": {
    "conflict_id": "conflict_1234567890",
    "agents": [
      {
        "agent": "CrashPredictor",
        "action": "Stop",
        "confidence": 0.95,
        "priority": 100
      },
      {
        "agent": "TransformerScheduler",
        "action": "ContinueNormal",
        "confidence": 0.80,
        "priority": 60
      }
    ],
    "resolution": {
      "strategy": "priority",
      "winner": "CrashPredictor",
      "action": "Stop",
      "reason": "Safety agent override (priority 100 > 60)"
    },
    "resolution_time_us": 156
  }
}
```

### 3. Phase Transition Event

**Event Type:** `phase_transition`

**Trigger:** When deployment phase changes

**Data:**
```json
{
  "type": "phase_transition",
  "timestamp": "2025-11-09T10:00:00.000000000Z",
  "data": {
    "from_phase": "A",
    "to_phase": "B",
    "trigger": "auto_advance",
    "reason": "Metrics met: error_rate < 1%, duration > 24h",
    "metrics_snapshot": {
      "error_rate": 0.005,
      "success_rate": 0.995,
      "uptime_hours": 26.5
    }
  }
}
```

**Triggers:**
- `auto_advance`: Automatic progression based on metrics
- `auto_rollback`: Automatic rollback due to errors
- `manual_advance`: User-initiated advance
- `manual_rollback`: User-initiated rollback

### 4. Drift Alert Event

**Event Type:** `drift_alert`

**Trigger:** When model accuracy degrades beyond threshold

**Data:**
```json
{
  "type": "drift_alert",
  "timestamp": "2025-11-09T10:23:45.123456789Z",
  "data": {
    "drift_level": "warning",
    "baseline_accuracy": 0.92,
    "current_accuracy": 0.87,
    "accuracy_delta": -0.05,
    "sample_count": 1543,
    "auto_retrain_triggered": false,
    "message": "Model accuracy dropped 5% (92% → 87%). Warning threshold reached."
  }
}
```

**Drift Levels:**
- `normal`: accuracy_delta < 0.05
- `warning`: 0.05 <= accuracy_delta < 0.15
- `critical`: accuracy_delta >= 0.15 (triggers auto-retrain)

### 5. Version Commit Event

**Event Type:** `version_commit`

**Trigger:** When a new adapter version is committed

**Data:**
```json
{
  "type": "version_commit",
  "timestamp": "2025-11-09T10:30:00.000000000Z",
  "data": {
    "version_id": 6,
    "parent_version": 5,
    "description": "Fine-tuned for low-light conditions",
    "metadata": {
      "training_examples": 1200,
      "training_duration_ms": 6000,
      "final_loss": 0.10,
      "accuracy_improvement": 0.05,
      "environment_tag": "low_light"
    }
  }
}
```

---

## Frontend Components Specification

### Panel 1: OrchestrationPanel.tsx

**File Path:** `/gui/desktop/src/components/OrchestrationPanel.tsx`

**Purpose:** Visualize multi-agent orchestration and decision flow

**Layout:**
```
┌─────────────────────────────────────────────────────────────────┐
│ Multi-Agent Orchestration                                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Orchestration Statistics                                │   │
│ │  • Total Decisions: 1543       • Avg Latency: 234 μs    │   │
│ │  • Unanimous: 892 (58%)        • Safety Overrides: 178  │   │
│ │  • Majority: 451 (29%)         • No Consensus: 22       │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Active Agents                                           │   │
│ │                                                         │   │
│ │ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │   │
│ │ │CrashPredictor│  │StateInference│  │Transformer   │  │   │
│ │ │Priority: 100 │  │Priority: 80  │  │Scheduler     │  │   │
│ │ │Status: ACTIVE│  │Status: ACTIVE│  │Priority: 60  │  │   │
│ │ │Last: Stop    │  │Last: Compact │  │Last: Continue│  │   │
│ │ │Conf: 95%     │  │Conf: 92%     │  │Conf: 80%     │  │   │
│ │ └──────────────┘  └──────────────┘  └──────────────┘  │   │
│ │                                                         │   │
│ │ ┌──────────────┐  ┌──────────────┐                    │   │
│ │ │FineTuner     │  │Metrics       │                    │   │
│ │ │Priority: 40  │  │Priority: 20  │                    │   │
│ │ │Status: ACTIVE│  │Status: ACTIVE│                    │   │
│ │ │Last: NoAction│  │Last: Monitor │                    │   │
│ │ │Conf: 65%     │  │Conf: 50%     │                    │   │
│ │ └──────────────┘  └──────────────┘                    │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Recent Decisions (Last 100)                             │   │
│ │                                                         │   │
│ │ Time       Type            Action        Agents  Latency│   │
│ │ 10:23:45   Unanimous       Compact       3       189 μs │   │
│ │ 10:23:40   Safety Override Stop          1       312 μs │   │
│ │ 10:23:35   Majority        Continue      2       156 μs │   │
│ │ 10:23:30   Unanimous       NoAction      5       98 μs  │   │
│ │ ...                                                      │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**React Component Structure:**

```typescript
interface OrchestrationStats {
  total_decisions: number;
  unanimous: number;
  majority: number;
  safety_overrides: number;
  no_consensus: number;
  avg_latency_us: number;
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

interface CoordinatedDecision {
  timestamp: string;
  type: 'unanimous' | 'majority' | 'safety_override' | 'no_consensus';
  action: string;
  confidence?: number;
  agents: string[];
  latency_us: number;
  description: string;
}

export function OrchestrationPanel() {
  // Fetch orchestration stats (polling every 5s)
  const { data: stats } = useQuery<OrchestrationStats>({
    queryKey: ['orchestrator', 'stats'],
    queryFn: () => api.get('/api/v1/orchestrator/stats'),
    refetchInterval: 5000,
  });

  // Fetch active agents (polling every 5s)
  const { data: agents } = useQuery<{ agents: Agent[] }>({
    queryKey: ['orchestrator', 'agents'],
    queryFn: () => api.get('/api/v1/orchestrator/agents'),
    refetchInterval: 5000,
  });

  // Fetch recent decisions (polling every 2s)
  const { data: decisions } = useQuery<{ decisions: CoordinatedDecision[] }>({
    queryKey: ['orchestrator', 'decisions'],
    queryFn: () => api.get('/api/v1/orchestrator/decisions?limit=100'),
    refetchInterval: 2000,
  });

  // Listen for real-time orchestration decisions via WebSocket
  const { lastMessage } = useWebSocket();

  useEffect(() => {
    if (lastMessage?.type === 'orchestration_decision') {
      // Update decisions list in real-time
      queryClient.invalidateQueries(['orchestrator', 'decisions']);
    }
  }, [lastMessage]);

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
      <DecisionHistoryTable decisions={decisions?.decisions || []} />
    </div>
  );
}
```

**Supporting Components:**

1. **OrchestrationStatsCard** - Display stats with percentage breakdowns
2. **AgentCard** - Card showing agent status, priority, last decision
3. **DecisionHistoryTable** - Virtualized table with filtering by type

### Panel 2: ConflictPanel.tsx

**File Path:** `/gui/desktop/src/components/ConflictPanel.tsx`

**Purpose:** Visualize agent conflicts and resolution strategies

**Layout:**
```
┌─────────────────────────────────────────────────────────────────┐
│ Conflict Resolution                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Conflict Statistics                                     │   │
│ │  • Total Conflicts: 89         • Avg Resolution: 145 μs │   │
│ │  • By Priority: 67 (75%)       • Unresolved: 4          │   │
│ │  • By Voting: 18 (20%)                                  │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Agent Priority Table                                    │   │
│ │                                                         │   │
│ │  Agent                    Priority  Decision Weight     │   │
│ │  CrashPredictor              100    ████████████ (Veto)│   │
│ │  StateInference               80    ██████████          │   │
│ │  TransformerScheduler         60    ████████            │   │
│ │  FineTuner                    40    █████               │   │
│ │  Metrics                      20    ███                 │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Recent Conflicts (Last 100)                             │   │
│ │                                                         │   │
│ │ Time       Agents                 Resolution  Winner    │   │
│ │ 10:23:45   CrashPredictor vs     Priority    Crash     │   │
│ │            TransformerScheduler                Predictor│   │
│ │            (Stop vs Continue)                           │   │
│ │                                                         │   │
│ │ 10:22:10   StateInference vs     Voting      State     │   │
│ │            FineTuner                          Inference │   │
│ │            (Compact vs NoAction)                        │   │
│ │ ...                                                      │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Conflict Detail (Selected)                              │   │
│ │                                                         │   │
│ │ Conflict ID: conflict_1234567890                        │   │
│ │ Timestamp: 2025-11-09 10:23:45                          │   │
│ │                                                         │   │
│ │ Conflicting Agents:                                     │   │
│ │  • CrashPredictor: Stop (95% confidence, priority 100) │   │
│ │  • TransformerScheduler: Continue (80% conf, pri 60)   │   │
│ │                                                         │   │
│ │ Resolution:                                             │   │
│ │  Strategy: Priority-based                               │   │
│ │  Winner: CrashPredictor                                 │   │
│ │  Action: Stop                                           │   │
│ │  Reason: Safety agent override (priority 100 > 60)      │   │
│ │  Resolution Time: 156 μs                                │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**React Component Structure:**

```typescript
interface ConflictStats {
  total_conflicts: number;
  resolved_by_priority: number;
  resolved_by_voting: number;
  unresolved: number;
  avg_resolution_time_us: number;
}

interface PriorityEntry {
  agent: string;
  priority: number;
}

interface Conflict {
  id: string;
  timestamp: string;
  agents: Array<{
    agent: string;
    action: string;
    confidence: number;
    priority: number;
  }>;
  resolution: {
    strategy: 'priority' | 'voting' | 'unresolved';
    winner: string;
    action: string;
    reason: string;
  };
  resolution_time_us: number;
}

export function ConflictPanel() {
  const [selectedConflict, setSelectedConflict] = useState<Conflict | null>(null);

  // Fetch conflict stats (polling every 5s)
  const { data: stats } = useQuery<ConflictStats>({
    queryKey: ['conflicts', 'stats'],
    queryFn: () => api.get('/api/v1/conflicts/stats'),
    refetchInterval: 5000,
  });

  // Fetch priority table (static, fetch once)
  const { data: priorities } = useQuery<{ priorities: PriorityEntry[] }>({
    queryKey: ['conflicts', 'priority-table'],
    queryFn: () => api.get('/api/v1/conflicts/priority-table'),
    staleTime: Infinity,
  });

  // Fetch conflict history (polling every 2s)
  const { data: conflicts } = useQuery<{ conflicts: Conflict[] }>({
    queryKey: ['conflicts', 'history'],
    queryFn: () => api.get('/api/v1/conflicts/history?limit=100'),
    refetchInterval: 2000,
  });

  // Listen for real-time conflict events via WebSocket
  const { lastMessage } = useWebSocket();

  useEffect(() => {
    if (lastMessage?.type === 'conflict_resolved') {
      queryClient.invalidateQueries(['conflicts', 'history']);
      queryClient.invalidateQueries(['conflicts', 'stats']);
    }
  }, [lastMessage]);

  return (
    <div className="space-y-4 p-4">
      {/* Statistics Card */}
      <ConflictStatsCard stats={stats} />

      {/* Priority Table */}
      <PriorityTable priorities={priorities?.priorities || []} />

      {/* Conflict History */}
      <ConflictHistoryTable
        conflicts={conflicts?.conflicts || []}
        onSelectConflict={setSelectedConflict}
      />

      {/* Conflict Detail */}
      {selectedConflict && (
        <ConflictDetailCard conflict={selectedConflict} />
      )}
    </div>
  );
}
```

**Supporting Components:**

1. **ConflictStatsCard** - Display conflict resolution statistics
2. **PriorityTable** - Visual priority bars for each agent
3. **ConflictHistoryTable** - Virtualized table with conflict history
4. **ConflictDetailCard** - Detailed view of selected conflict

### Panel 3: DeploymentPanel.tsx

**File Path:** `/gui/desktop/src/components/DeploymentPanel.tsx`

**Purpose:** Visualize deployment phases and manage rollouts

**Layout:**
```
┌─────────────────────────────────────────────────────────────────┐
│ Deployment Management                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Current Phase                                           │   │
│ │                                                         │   │
│ │  Phase B: Canary (10% traffic)                          │   │
│ │                                                         │   │
│ │  ┌─────┬─────┬─────┬─────┐                             │   │
│ │  │  A  │  B  │  C  │  D  │                             │   │
│ │  │ ✓   │ ●   │     │     │                             │   │
│ │  └─────┴─────┴─────┴─────┘                             │   │
│ │                                                         │   │
│ │  Elapsed: 1h / Min: 12h (8%)                            │   │
│ │  Error Rate: 2.0% (Threshold: 5.0%)                     │   │
│ │  Success Rate: 98.0%                                    │   │
│ │                                                         │   │
│ │  [Auto-Advance: ON]  [Auto-Rollback: ON]               │   │
│ │  [Advance Now] [Rollback] [Configure]                   │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Phase Definitions                                       │   │
│ │                                                         │   │
│ │  Phase  Name      Traffic  Min Duration  Description    │   │
│ │  A      Shadow    0%       24h           Monitor only   │   │
│ │  B      Canary    10%      12h           Limited test   │   │
│ │  C      Gradual   50%      24h           Wider rollout  │   │
│ │  D      Full      100%     N/A           Complete       │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Phase Transition History                                │   │
│ │                                                         │   │
│ │ Time       From → To  Trigger       Reason              │   │
│ │ 10:00:00   A → B      Auto-Advance  Metrics met         │   │
│ │ 08:00:00   B → A      Auto-Rollback Error rate > 5%     │   │
│ │ 06:00:00   A → B      Manual        User initiated      │   │
│ │ ...                                                      │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Metrics Over Time (Last 24h)                            │   │
│ │                                                         │   │
│ │  Error Rate %                                           │   │
│ │  5% ┤                                                   │   │
│ │  4% ┤                                                   │   │
│ │  3% ┤                                                   │   │
│ │  2% ┤        ●●●●●●●●                                   │   │
│ │  1% ┤  ●●●●●                                            │   │
│ │  0% ┼────────────────────────────────────────          │   │
│ │      0h    6h    12h   18h   24h                        │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**React Component Structure:**

```typescript
interface DeploymentStatus {
  current_phase: {
    id: 'A' | 'B' | 'C' | 'D';
    name: string;
    description: string;
    entered_at: string;
    min_duration_ms: number;
    elapsed_ms: number;
    can_advance: boolean;
    traffic_percentage: number;
    error_rate: number;
    success_rate: number;
  };
  auto_advance_enabled: boolean;
  auto_rollback_enabled: boolean;
  rollback_count: number;
  max_rollbacks: number;
}

interface PhaseTransition {
  timestamp: string;
  from_phase: string;
  to_phase: string;
  trigger: 'auto_advance' | 'auto_rollback' | 'manual_advance' | 'manual_rollback';
  reason: string;
  metrics_snapshot: {
    error_rate: number;
    success_rate: number;
    uptime_hours: number;
  };
}

export function DeploymentPanel() {
  // Fetch deployment status (polling every 5s)
  const { data: status } = useQuery<DeploymentStatus>({
    queryKey: ['deployment', 'status'],
    queryFn: () => api.get('/api/v1/deployment/status'),
    refetchInterval: 5000,
  });

  // Fetch transition history (polling every 10s)
  const { data: history } = useQuery<{ transitions: PhaseTransition[] }>({
    queryKey: ['deployment', 'history'],
    queryFn: () => api.get('/api/v1/deployment/history'),
    refetchInterval: 10000,
  });

  // Mutations
  const advanceMutation = useMutation({
    mutationFn: (force: boolean) => api.post('/api/v1/deployment/advance', { force }),
    onSuccess: () => queryClient.invalidateQueries(['deployment']),
  });

  const rollbackMutation = useMutation({
    mutationFn: (reason: string) => api.post('/api/v1/deployment/rollback', { reason }),
    onSuccess: () => queryClient.invalidateQueries(['deployment']),
  });

  // Listen for real-time phase transitions via WebSocket
  const { lastMessage } = useWebSocket();

  useEffect(() => {
    if (lastMessage?.type === 'phase_transition') {
      queryClient.invalidateQueries(['deployment']);
      // Show toast notification
      toast.info(`Phase transition: ${lastMessage.data.from_phase} → ${lastMessage.data.to_phase}`);
    }
  }, [lastMessage]);

  return (
    <div className="space-y-4 p-4">
      {/* Current Phase Card */}
      <CurrentPhaseCard
        status={status}
        onAdvance={() => advanceMutation.mutate(false)}
        onRollback={() => rollbackMutation.mutate('User-initiated rollback')}
      />

      {/* Phase Definitions Table */}
      <PhaseDefinitionsTable />

      {/* Transition History */}
      <TransitionHistoryTable transitions={history?.transitions || []} />

      {/* Metrics Chart */}
      <DeploymentMetricsChart />
    </div>
  );
}
```

**Supporting Components:**

1. **CurrentPhaseCard** - Display current phase with progress bar
2. **PhaseTimeline** - Visual timeline showing A → B → C → D
3. **PhaseDefinitionsTable** - Static table of phase definitions
4. **TransitionHistoryTable** - Virtualized table with transition history
5. **DeploymentMetricsChart** - Line chart of error rate over time

### Panel 4: DriftPanel.tsx

**File Path:** `/gui/desktop/src/components/DriftPanel.tsx`

**Purpose:** Monitor model performance drift and trigger retraining

**Layout:**
```
┌─────────────────────────────────────────────────────────────────┐
│ Model Drift Detection                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Current Drift Status                                    │   │
│ │                                                         │   │
│ │  ⚠️  WARNING                                            │   │
│ │                                                         │   │
│ │  Baseline Accuracy: 92.0%                               │   │
│ │  Current Accuracy:  87.0%                               │   │
│ │  Accuracy Delta:    -5.0% ▼                             │   │
│ │                                                         │   │
│ │  ┌─────────────────────────────────────────────┐        │   │
│ │  │ Normal    │ Warning   │ Critical            │        │   │
│ │  │  (<5%)    │  (5-15%)  │  (>15%)             │        │   │
│ │  │           ● You are here                    │        │   │
│ │  └─────────────────────────────────────────────┘        │   │
│ │                                                         │   │
│ │  Samples Analyzed: 1543 / Window: 100                   │   │
│ │  Last Retrain: 2025-11-08 10:00:00 (26h ago)            │   │
│ │                                                         │   │
│ │  [Trigger Retrain] [Reset Baseline] [Configure]        │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Accuracy Over Time (Last 24h)                           │   │
│ │                                                         │   │
│ │  100% ┤                                                 │   │
│ │   95% ┤ ●●●●●●                                          │   │
│ │   90% ┤       ●●●●●●                                    │   │
│ │   85% ┤             ●●●●●●●●●●                          │   │
│ │   80% ┤                                                 │   │
│ │       ┼────────────────────────────────────────        │   │
│ │        0h    6h    12h   18h   24h                      │   │
│ │                                                         │   │
│ │  Legend: ● Current  ─ Baseline (92%)                    │   │
│ │          ▬ Warning Threshold  ▬ Critical Threshold      │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Drift History                                           │   │
│ │                                                         │   │
│ │ Time       Accuracy  Drift Level  Delta    Samples      │   │
│ │ 10:23:45   87.0%     Warning      -5.0%    1543         │   │
│ │ 10:23:40   89.0%     Normal       -3.0%    1542         │   │
│ │ 10:23:35   90.5%     Normal       -1.5%    1541         │   │
│ │ ...                                                      │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Auto-Retrain Configuration                              │   │
│ │                                                         │   │
│ │  Auto-Retrain: [ON]                                     │   │
│ │  Critical Threshold: 15%                                │   │
│ │  Sample Window: 100 samples                             │   │
│ │  Training Examples: 1000                                │   │
│ │  Epochs: 10                                             │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**React Component Structure:**

```typescript
interface DriftStatus {
  baseline_accuracy: number;
  current_accuracy: number;
  accuracy_delta: number;
  drift_level: 'normal' | 'warning' | 'critical';
  sample_window_size: number;
  samples_analyzed: number;
  last_retrain: string;
  auto_retrain_enabled: boolean;
  auto_retrain_threshold: number;
}

interface DriftSample {
  timestamp: string;
  accuracy: number;
  drift_level: 'normal' | 'warning' | 'critical';
  accuracy_delta: number;
  sample_count: number;
}

export function DriftPanel() {
  // Fetch drift status (polling every 5s)
  const { data: status } = useQuery<DriftStatus>({
    queryKey: ['drift', 'status'],
    queryFn: () => api.get('/api/v1/drift/status'),
    refetchInterval: 5000,
  });

  // Fetch drift history (polling every 10s)
  const { data: history } = useQuery<{ samples: DriftSample[] }>({
    queryKey: ['drift', 'history'],
    queryFn: () => api.get('/api/v1/drift/history?time_range=86400'),
    refetchInterval: 10000,
  });

  // Mutations
  const retrainMutation = useMutation({
    mutationFn: () => api.post('/api/v1/drift/retrain', {
      training_examples: 1000,
      epochs: 10,
    }),
    onSuccess: () => {
      toast.success('Model retraining started');
      queryClient.invalidateQueries(['drift']);
    },
  });

  const resetBaselineMutation = useMutation({
    mutationFn: () => api.post('/api/v1/drift/reset-baseline'),
    onSuccess: () => {
      toast.success('Baseline reset to current accuracy');
      queryClient.invalidateQueries(['drift']);
    },
  });

  // Listen for real-time drift alerts via WebSocket
  const { lastMessage } = useWebSocket();

  useEffect(() => {
    if (lastMessage?.type === 'drift_alert') {
      queryClient.invalidateQueries(['drift']);
      // Show alert toast based on level
      const level = lastMessage.data.drift_level;
      if (level === 'critical') {
        toast.error(lastMessage.data.message);
      } else if (level === 'warning') {
        toast.warning(lastMessage.data.message);
      }
    }
  }, [lastMessage]);

  return (
    <div className="space-y-4 p-4">
      {/* Current Status Card */}
      <DriftStatusCard
        status={status}
        onRetrain={() => retrainMutation.mutate()}
        onResetBaseline={() => resetBaselineMutation.mutate()}
      />

      {/* Accuracy Chart */}
      <DriftChart samples={history?.samples || []} baseline={status?.baseline_accuracy} />

      {/* Drift History Table */}
      <DriftHistoryTable samples={history?.samples || []} />

      {/* Configuration Card */}
      <DriftConfigCard status={status} />
    </div>
  );
}
```

**Supporting Components:**

1. **DriftStatusCard** - Display current drift status with level indicator
2. **DriftLevelIndicator** - Visual indicator (Normal/Warning/Critical)
3. **DriftChart** - Line chart showing accuracy over time with thresholds
4. **DriftHistoryTable** - Virtualized table with drift history
5. **DriftConfigCard** - Configuration form for auto-retrain settings

### Panel 5: VersionsPanel.tsx

**File Path:** `/gui/desktop/src/components/VersionsPanel.tsx`

**Purpose:** Manage adapter versions with Git-like interface

**Layout:**
```
┌─────────────────────────────────────────────────────────────────┐
│ Adapter Version Control                                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Current Version: v5                                     │   │
│ │                                                         │   │
│ │  [Commit New Version] [Rollback] [Tag] [Garbage Collect]│   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Version History (Git-like)                              │   │
│ │                                                         │   │
│ │  v5 (HEAD) [stable]                                     │   │
│ │  │  Trained on warehouse A failures                     │   │
│ │  │  Training: 1000 examples, 5s, loss: 0.12            │   │
│ │  │  Improvement: +3.0% accuracy                         │   │
│ │  │  Environment: warehouse_A                            │   │
│ │  │  2025-11-09 10:00:00                                 │   │
│ │  │                                                      │   │
│ │  v4                                                      │   │
│ │  │  Baseline version                                    │   │
│ │  │  Training: 500 examples, 2.5s, loss: 0.15           │   │
│ │  │  Improvement: 0.0% accuracy                          │   │
│ │  │  Environment: baseline                               │   │
│ │  │  2025-11-08 10:00:00                                 │   │
│ │  │                                                      │   │
│ │  v3 [production]                                         │   │
│ │  │  Fine-tuned for low-light conditions                │   │
│ │  │  Training: 800 examples, 4s, loss: 0.13             │   │
│ │  │  Improvement: +2.0% accuracy                         │   │
│ │  │  Environment: low_light                              │   │
│ │  │  2025-11-07 10:00:00                                 │   │
│ │  │                                                      │   │
│ │  ...                                                     │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Version Comparison (v4 vs v5)                           │   │
│ │                                                         │   │
│ │  Accuracy Delta: +3.0%                                  │   │
│ │  Parameter Changes: 150 weights modified                │   │
│ │  Time Delta: 24 hours                                   │   │
│ │                                                         │   │
│ │  [View Full Diff]                                       │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│ ┌─────────────────────────────────────────────────────────┐   │
│ │ Commit New Version                                      │   │
│ │                                                         │   │
│ │  Description: ___________________________________        │   │
│ │  Environment Tag: ___________________________________    │   │
│ │  Training Examples: [1000]                              │   │
│ │  Epochs: [10]                                           │   │
│ │                                                         │   │
│ │  [Cancel] [Commit]                                      │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**React Component Structure:**

```typescript
interface AdapterVersion {
  version_id: number;
  parent_version: number | null;
  timestamp: string;
  description: string;
  metadata: {
    training_examples: number;
    training_duration_ms: number;
    final_loss: number;
    accuracy_improvement: number;
    environment_tag: string;
  };
  hash: string;
  storage_path: string;
  tags: string[];
}

interface VersionDiff {
  version_a: number;
  version_b: number;
  accuracy_delta: number;
  param_changes: number;
  time_delta_hours: number;
}

export function VersionsPanel() {
  const [selectedVersions, setSelectedVersions] = useState<[number, number] | null>(null);
  const [showCommitDialog, setShowCommitDialog] = useState(false);

  // Fetch version list (polling every 10s)
  const { data: versions } = useQuery<{ current_version: number; versions: AdapterVersion[] }>({
    queryKey: ['versions', 'list'],
    queryFn: () => api.get('/api/v1/versions/list'),
    refetchInterval: 10000,
  });

  // Fetch version diff (when two versions selected)
  const { data: diff } = useQuery<VersionDiff>({
    queryKey: ['versions', 'diff', selectedVersions],
    queryFn: () => {
      if (!selectedVersions) return null;
      return api.get(`/api/v1/versions/diff?v1=${selectedVersions[0]}&v2=${selectedVersions[1]}`);
    },
    enabled: !!selectedVersions,
  });

  // Mutations
  const commitMutation = useMutation({
    mutationFn: (data: {
      description: string;
      environment_tag: string;
      training_examples: number;
      epochs: number;
    }) => api.post('/api/v1/versions/commit', data),
    onSuccess: () => {
      toast.success('Version committed successfully');
      queryClient.invalidateQueries(['versions']);
      setShowCommitDialog(false);
    },
  });

  const rollbackMutation = useMutation({
    mutationFn: (version_id: number) => api.post('/api/v1/versions/rollback', { version_id }),
    onSuccess: () => {
      toast.success('Rolled back to previous version');
      queryClient.invalidateQueries(['versions']);
    },
  });

  const tagMutation = useMutation({
    mutationFn: (data: { version_id: number; tag: string }) =>
      api.post('/api/v1/versions/tag', data),
    onSuccess: () => {
      toast.success('Version tagged');
      queryClient.invalidateQueries(['versions']);
    },
  });

  const gcMutation = useMutation({
    mutationFn: (keep_count: number) => api.post('/api/v1/versions/gc', { keep_count }),
    onSuccess: (data) => {
      toast.success(`Garbage collection complete: ${data.removed_count} versions removed`);
      queryClient.invalidateQueries(['versions']);
    },
  });

  // Listen for real-time version commits via WebSocket
  const { lastMessage } = useWebSocket();

  useEffect(() => {
    if (lastMessage?.type === 'version_commit') {
      queryClient.invalidateQueries(['versions']);
      toast.info(`New version committed: v${lastMessage.data.version_id}`);
    }
  }, [lastMessage]);

  return (
    <div className="space-y-4 p-4">
      {/* Header with actions */}
      <VersionHeader
        currentVersion={versions?.current_version}
        onCommit={() => setShowCommitDialog(true)}
        onGC={() => gcMutation.mutate(10)}
      />

      {/* Version History Tree */}
      <VersionTree
        versions={versions?.versions || []}
        currentVersion={versions?.current_version}
        onSelectVersions={setSelectedVersions}
        onRollback={(versionId) => rollbackMutation.mutate(versionId)}
        onTag={(versionId, tag) => tagMutation.mutate({ version_id: versionId, tag })}
      />

      {/* Version Diff (when two selected) */}
      {diff && selectedVersions && (
        <VersionDiffCard diff={diff} versions={selectedVersions} />
      )}

      {/* Commit Dialog */}
      {showCommitDialog && (
        <CommitDialog
          onClose={() => setShowCommitDialog(false)}
          onCommit={(data) => commitMutation.mutate(data)}
        />
      )}
    </div>
  );
}
```

**Supporting Components:**

1. **VersionHeader** - Header with current version and action buttons
2. **VersionTree** - Git-like tree view of version history
3. **VersionCard** - Card showing version details with metadata
4. **VersionDiffCard** - Side-by-side version comparison
5. **CommitDialog** - Modal dialog for committing new version
6. **TagBadge** - Badge showing version tags (stable, production, etc.)

---

## Implementation Phases

### Phase 1: Backend Foundation (2 weeks)

**Week 1: API Endpoints**

**Tasks:**
1. Implement 15 new API endpoints in sisctl daemon
2. Add shell command execution for Phase 2 commands
3. Add output parsing for stats/history/status commands
4. Add request validation and error handling
5. Add API documentation

**Deliverables:**
- All 15 API endpoints functional
- Unit tests for each endpoint
- API documentation (OpenAPI spec)

**Acceptance Criteria:**
- All endpoints return correct JSON format
- Error handling for invalid requests
- Response times < 500ms
- Unit test coverage > 80%

**Week 2: WebSocket Events**

**Tasks:**
1. Implement 5 new WebSocket event types
2. Add event serialization/deserialization
3. Add event broadcasting to connected clients
4. Add output parsing for real-time events from kernel
5. Add event filtering support

**Deliverables:**
- 5 WebSocket event types implemented
- Event broadcasting working
- Real-time updates streaming to clients

**Acceptance Criteria:**
- Events broadcast within 100ms of kernel output
- Multiple clients can subscribe to same events
- Event format matches specification
- No memory leaks in long-running connections

### Phase 2: Frontend Components (3 weeks)

**Week 3: Core Panels (Orchestration + Conflicts)**

**Tasks:**
1. Create OrchestrationPanel.tsx with stats, agents, decisions
2. Create ConflictPanel.tsx with priority table, conflict history
3. Add supporting components (AgentCard, ConflictVisualization, etc.)
4. Integrate with API endpoints and WebSocket events
5. Add responsive layout

**Deliverables:**
- OrchestrationPanel fully functional
- ConflictPanel fully functional
- Real-time updates working

**Acceptance Criteria:**
- Data updates in real-time via WebSocket
- Polling fallback works if WebSocket disconnects
- Components render with no layout issues
- Responsive design works on different screen sizes

**Week 4: Management Panels (Deployment + Drift)**

**Tasks:**
1. Create DeploymentPanel.tsx with phase timeline, history
2. Create DriftPanel.tsx with status, chart, history
3. Add supporting components (PhaseTimeline, DriftChart, etc.)
4. Add manual control actions (advance, rollback, retrain)
5. Add configuration forms

**Deliverables:**
- DeploymentPanel fully functional
- DriftPanel fully functional
- Manual controls working

**Acceptance Criteria:**
- Phase transitions update in real-time
- Drift alerts show immediately
- Manual actions execute successfully
- Configuration changes persist

**Week 5: Version Control Panel**

**Tasks:**
1. Create VersionsPanel.tsx with version tree
2. Add version comparison functionality
3. Add commit dialog
4. Add rollback/tag/GC actions
5. Add Git-like visualization

**Deliverables:**
- VersionsPanel fully functional
- Version tree visualization working
- All version control operations functional

**Acceptance Criteria:**
- Version history displays correctly
- Commit/rollback/tag operations work
- Version diff shows accurate comparison
- GC removes old versions correctly

### Phase 3: Testing & Integration (1 week)

**Week 6: E2E Tests**

**Tasks:**
1. Write Playwright E2E tests for all 5 panels
2. Test WebSocket event handling
3. Test API error scenarios
4. Test real-time updates
5. Test manual control actions

**Deliverables:**
- E2E test suite with 50+ test cases
- All tests passing
- Test coverage report

**Acceptance Criteria:**
- E2E tests cover all major user flows
- Tests run in CI/CD pipeline
- Test coverage > 70%
- No flaky tests

### Phase 4: Polish & Documentation (1 week)

**Week 7: UX Polish + Docs**

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

**Acceptance Criteria:**
- No UI jank or layout shift
- Error messages are user-friendly
- Documentation is comprehensive
- Demo video is clear and concise

---

## Testing Strategy

### Unit Tests (Backend)

**Location:** `crates/sisctl/src/api/tests/`

**Coverage:**
- API endpoint handlers
- Shell command execution
- Output parsing
- WebSocket event serialization
- Request validation

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_stats_endpoint() {
        let response = get_orchestrator_stats().await.unwrap();
        assert_eq!(response.total_decisions, 0);
    }
}
```

### E2E Tests (Frontend)

**Location:** `gui/desktop/e2e/orchestration.spec.ts`, etc.

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
});

test('conflict panel shows real-time conflicts', async ({ page }) => {
  await page.goto('/');
  await page.click('[data-testid="conflicts-tab"]');

  // Wait for WebSocket connection
  await page.waitForTimeout(1000);

  // Trigger conflict in kernel (via API)
  // ...

  // Check conflict appears in UI
  await expect(page.locator('[data-testid="conflict-item"]').first()).toBeVisible();
});
```

### Integration Tests

**Approach:** Run full stack (kernel + daemon + GUI) and test end-to-end flows

**Test Scenarios:**
1. Start kernel → Check orchestration stats update
2. Trigger conflict → Check conflict appears in UI
3. Advance deployment phase → Check phase transition in UI
4. Model drift → Check drift alert appears
5. Commit adapter version → Check version appears in tree

---

## Success Criteria

### Functional Requirements

✅ **Orchestration Panel:**
- [ ] Display orchestration statistics (total, unanimous, majority, overrides)
- [ ] Show all active agents with status
- [ ] Display recent coordinated decisions (last 100)
- [ ] Update in real-time via WebSocket

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
- [ ] API response time < 500ms
- [ ] WebSocket event latency < 100ms
- [ ] UI render time < 16ms (60fps)
- [ ] No memory leaks in long-running sessions

✅ **Reliability:**
- [ ] WebSocket auto-reconnect on disconnect
- [ ] Polling fallback if WebSocket fails
- [ ] Error boundaries prevent full page crashes
- [ ] Graceful degradation on API errors

✅ **Usability:**
- [ ] Loading states for all async operations
- [ ] Error messages are user-friendly
- [ ] Keyboard navigation works
- [ ] Tooltips explain all features
- [ ] Responsive design on different screen sizes

✅ **Testing:**
- [ ] Unit test coverage > 80% (backend)
- [ ] E2E test coverage > 70% (frontend)
- [ ] All tests pass in CI/CD pipeline
- [ ] No flaky tests

---

## Technical Specifications

### API Endpoint Summary

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/orchestrator/stats` | GET | Get orchestration statistics |
| `/api/v1/orchestrator/decisions` | GET | Get recent decisions |
| `/api/v1/orchestrator/agents` | GET | Get agent status |
| `/api/v1/conflicts/stats` | GET | Get conflict statistics |
| `/api/v1/conflicts/history` | GET | Get conflict history |
| `/api/v1/conflicts/priority-table` | GET | Get agent priorities |
| `/api/v1/deployment/status` | GET | Get deployment status |
| `/api/v1/deployment/history` | GET | Get phase transitions |
| `/api/v1/deployment/advance` | POST | Advance deployment phase |
| `/api/v1/deployment/rollback` | POST | Rollback deployment phase |
| `/api/v1/deployment/config` | POST | Update deployment config |
| `/api/v1/drift/status` | GET | Get drift status |
| `/api/v1/drift/history` | GET | Get drift history |
| `/api/v1/drift/retrain` | POST | Trigger model retrain |
| `/api/v1/drift/reset-baseline` | POST | Reset baseline accuracy |
| `/api/v1/versions/list` | GET | Get version history |
| `/api/v1/versions/commit` | POST | Commit new version |
| `/api/v1/versions/rollback` | POST | Rollback to version |
| `/api/v1/versions/diff` | GET | Compare versions |
| `/api/v1/versions/tag` | POST | Tag version |
| `/api/v1/versions/gc` | POST | Garbage collect versions |

**Total:** 21 API endpoints

### WebSocket Event Summary

| Event Type | Trigger | Data |
|------------|---------|------|
| `orchestration_decision` | Coordinated decision made | Decision type, action, agents, latency |
| `conflict_resolved` | Conflict detected and resolved | Conflict ID, agents, resolution strategy |
| `phase_transition` | Deployment phase changed | From/to phase, trigger, reason, metrics |
| `drift_alert` | Model accuracy degraded | Drift level, accuracy delta, message |
| `version_commit` | Adapter version committed | Version ID, parent, description, metadata |

**Total:** 5 WebSocket event types

### Frontend Component Summary

| Component | Purpose | Lines of Code (Est.) |
|-----------|---------|---------------------|
| OrchestrationPanel.tsx | Main orchestration panel | 300 |
| ConflictPanel.tsx | Main conflict panel | 350 |
| DeploymentPanel.tsx | Main deployment panel | 400 |
| DriftPanel.tsx | Main drift panel | 350 |
| VersionsPanel.tsx | Main versions panel | 450 |
| AgentCard.tsx | Agent status card | 80 |
| ConflictVisualization.tsx | Conflict visualization | 120 |
| PhaseTimeline.tsx | Deployment phase timeline | 100 |
| DriftChart.tsx | Drift accuracy chart | 150 |
| VersionTree.tsx | Version history tree | 200 |
| DecisionHistoryTable.tsx | Decision history table | 150 |
| ConflictHistoryTable.tsx | Conflict history table | 150 |
| TransitionHistoryTable.tsx | Phase transition table | 150 |
| DriftHistoryTable.tsx | Drift history table | 150 |
| CommitDialog.tsx | Version commit dialog | 120 |
| (15+ more supporting components) | Various | 800 |

**Total:** ~3,520 lines of TypeScript

---

## File Structure

### Backend (sisctl daemon)

```
crates/sisctl/src/
├── api/
│   ├── mod.rs                      # API router
│   ├── orchestrator.rs             # NEW: Orchestrator endpoints
│   ├── conflicts.rs                # NEW: Conflict endpoints
│   ├── deployment.rs               # NEW: Deployment endpoints
│   ├── drift.rs                    # NEW: Drift endpoints
│   ├── versions.rs                 # NEW: Version control endpoints
│   └── tests/
│       ├── orchestrator_tests.rs   # NEW: Orchestrator tests
│       ├── conflicts_tests.rs      # NEW: Conflict tests
│       ├── deployment_tests.rs     # NEW: Deployment tests
│       ├── drift_tests.rs          # NEW: Drift tests
│       └── versions_tests.rs       # NEW: Version tests
├── websocket/
│   ├── mod.rs                      # WebSocket handler
│   ├── events.rs                   # UPDATED: Add 5 new event types
│   └── parser.rs                   # UPDATED: Parse new event formats
└── shell/
    ├── mod.rs                      # Shell executor
    └── parsers.rs                  # UPDATED: Add parsers for new commands
```

### Frontend (GUI)

```
gui/desktop/src/
├── components/
│   ├── OrchestrationPanel.tsx      # NEW: Orchestration panel
│   ├── ConflictPanel.tsx           # NEW: Conflict panel
│   ├── DeploymentPanel.tsx         # NEW: Deployment panel
│   ├── DriftPanel.tsx              # NEW: Drift panel
│   ├── VersionsPanel.tsx           # NEW: Versions panel
│   │
│   ├── orchestration/              # NEW: Orchestration components
│   │   ├── OrchestrationStatsCard.tsx
│   │   ├── AgentCard.tsx
│   │   ├── DecisionHistoryTable.tsx
│   │   └── AgentStatusBadge.tsx
│   │
│   ├── conflicts/                  # NEW: Conflict components
│   │   ├── ConflictStatsCard.tsx
│   │   ├── PriorityTable.tsx
│   │   ├── ConflictHistoryTable.tsx
│   │   ├── ConflictDetailCard.tsx
│   │   └── ConflictVisualization.tsx
│   │
│   ├── deployment/                 # NEW: Deployment components
│   │   ├── CurrentPhaseCard.tsx
│   │   ├── PhaseTimeline.tsx
│   │   ├── PhaseDefinitionsTable.tsx
│   │   ├── TransitionHistoryTable.tsx
│   │   └── DeploymentMetricsChart.tsx
│   │
│   ├── drift/                      # NEW: Drift components
│   │   ├── DriftStatusCard.tsx
│   │   ├── DriftLevelIndicator.tsx
│   │   ├── DriftChart.tsx
│   │   ├── DriftHistoryTable.tsx
│   │   └── DriftConfigCard.tsx
│   │
│   └── versions/                   # NEW: Version components
│       ├── VersionHeader.tsx
│       ├── VersionTree.tsx
│       ├── VersionCard.tsx
│       ├── VersionDiffCard.tsx
│       ├── CommitDialog.tsx
│       └── TagBadge.tsx
│
├── lib/
│   ├── api.ts                      # UPDATED: Add new API methods
│   ├── useWebSocket.ts             # UPDATED: Handle new event types
│   └── types/                      # NEW: Type definitions
│       ├── orchestration.ts
│       ├── conflicts.ts
│       ├── deployment.ts
│       ├── drift.ts
│       └── versions.ts
│
├── e2e/
│   ├── orchestration.spec.ts       # NEW: Orchestration E2E tests
│   ├── conflicts.spec.ts           # NEW: Conflict E2E tests
│   ├── deployment.spec.ts          # NEW: Deployment E2E tests
│   ├── drift.spec.ts               # NEW: Drift E2E tests
│   └── versions.spec.ts            # NEW: Versions E2E tests
│
└── App.tsx                         # UPDATED: Add 5 new tabs
```

---

## Code Examples

### Backend API Endpoint Example

```rust
// crates/sisctl/src/api/orchestrator.rs

use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct OrchestrationStats {
    pub total_decisions: u64,
    pub unanimous: u64,
    pub majority: u64,
    pub safety_overrides: u64,
    pub no_consensus: u64,
    pub avg_latency_us: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoordinatedDecision {
    pub timestamp: String,
    pub r#type: String,
    pub action: String,
    pub confidence: Option<f32>,
    pub agents: Vec<String>,
    pub latency_us: u64,
    pub description: String,
}

/// GET /api/v1/orchestrator/stats
pub async fn get_orchestrator_stats(
    shell: Arc<Mutex<ShellExecutor>>,
) -> Result<Json<OrchestrationStats>, ApiError> {
    let mut shell = shell.lock().await;

    // Execute shell command to get stats
    let output = shell.exec("coordctl stats").await?;

    // Parse output (example format: "total_decisions=1543 unanimous=892 ...")
    let stats = parse_orchestrator_stats(&output)?;

    Ok(Json(stats))
}

/// GET /api/v1/orchestrator/decisions
pub async fn get_orchestrator_decisions(
    Query(params): Query<HashMap<String, String>>,
    shell: Arc<Mutex<ShellExecutor>>,
) -> Result<Json<Vec<CoordinatedDecision>>, ApiError> {
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(100);
    let filter_type = params.get("type");

    let mut shell = shell.lock().await;

    // Execute shell command
    let output = shell.exec(&format!("coordctl history --limit {}", limit)).await?;

    // Parse JSON output from kernel
    let mut decisions: Vec<CoordinatedDecision> = serde_json::from_str(&output)?;

    // Apply filter if specified
    if let Some(filter_type) = filter_type {
        decisions.retain(|d| d.r#type == filter_type);
    }

    Ok(Json(decisions))
}

fn parse_orchestrator_stats(output: &str) -> Result<OrchestrationStats, ApiError> {
    // Parse key=value format
    let mut stats = OrchestrationStats {
        total_decisions: 0,
        unanimous: 0,
        majority: 0,
        safety_overrides: 0,
        no_consensus: 0,
        avg_latency_us: 0,
    };

    for line in output.lines() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].trim();
        let value = parts[1].trim().parse::<u64>().unwrap_or(0);

        match key {
            "total_decisions" => stats.total_decisions = value,
            "unanimous" => stats.unanimous = value,
            "majority" => stats.majority = value,
            "safety_overrides" => stats.safety_overrides = value,
            "no_consensus" => stats.no_consensus = value,
            "avg_latency_us" => stats.avg_latency_us = value,
            _ => {}
        }
    }

    Ok(stats)
}
```

### WebSocket Event Example

```rust
// crates/sisctl/src/websocket/events.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketEvent {
    // Existing events...
    RawLine { timestamp: DateTime<Utc>, data: String },
    Parsed { timestamp: DateTime<Utc>, data: ParsedEvent },
    MetricBatch { timestamp: DateTime<Utc>, data: MetricBatch },

    // New Phase 2 events
    OrchestrationDecision {
        timestamp: DateTime<Utc>,
        data: OrchestrationDecisionData,
    },
    ConflictResolved {
        timestamp: DateTime<Utc>,
        data: ConflictResolvedData,
    },
    PhaseTransition {
        timestamp: DateTime<Utc>,
        data: PhaseTransitionData,
    },
    DriftAlert {
        timestamp: DateTime<Utc>,
        data: DriftAlertData,
    },
    VersionCommit {
        timestamp: DateTime<Utc>,
        data: VersionCommitData,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationDecisionData {
    pub decision_type: String,
    pub action: String,
    pub confidence: Option<f32>,
    pub agents: Vec<String>,
    pub latency_us: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolvedData {
    pub conflict_id: String,
    pub agents: Vec<ConflictAgent>,
    pub resolution: Resolution,
    pub resolution_time_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictAgent {
    pub agent: String,
    pub action: String,
    pub confidence: f32,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub strategy: String,
    pub winner: String,
    pub action: String,
    pub reason: String,
}

// Parser for orchestration decision event
pub fn parse_orchestration_decision(line: &str) -> Option<OrchestrationDecisionData> {
    // Example line format:
    // [ORCHESTRATION] decision_type=unanimous action=CompactMemory confidence=0.92 agents=CrashPredictor,StateInference,TransformerScheduler latency_us=189

    if !line.contains("[ORCHESTRATION]") {
        return None;
    }

    let parts: HashMap<&str, &str> = line
        .split_whitespace()
        .skip(1) // Skip [ORCHESTRATION]
        .filter_map(|s| {
            let kv: Vec<&str> = s.split('=').collect();
            if kv.len() == 2 {
                Some((kv[0], kv[1]))
            } else {
                None
            }
        })
        .collect();

    Some(OrchestrationDecisionData {
        decision_type: parts.get("decision_type")?.to_string(),
        action: parts.get("action")?.to_string(),
        confidence: parts.get("confidence").and_then(|s| s.parse().ok()),
        agents: parts.get("agents")?.split(',').map(|s| s.to_string()).collect(),
        latency_us: parts.get("latency_us")?.parse().ok()?,
        description: format!("..."), // Generate description
    })
}
```

### Frontend API Client Example

```typescript
// gui/desktop/src/lib/api.ts

import axios from 'axios';

const API_BASE = 'http://127.0.0.1:8871';

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

  commit: async (data: {
    description: string;
    environment_tag: string;
    metadata: {
      training_examples: number;
      training_duration_ms: number;
      final_loss: number;
      accuracy_improvement: number;
    };
  }): Promise<{ success: boolean; version_id: number }> => {
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

### Frontend WebSocket Hook Example

```typescript
// gui/desktop/src/lib/useWebSocket.ts (updated)

import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';

interface WebSocketMessage {
  type: string;
  timestamp: string;
  data: any;
}

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
          // Show toast notification
          toast.info(`Phase transition: ${message.data.from_phase} → ${message.data.to_phase}`);
          break;

        case 'drift_alert':
          queryClient.invalidateQueries(['drift', 'status']);
          queryClient.invalidateQueries(['drift', 'history']);
          // Show alert based on level
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

## Appendix: Shell Commands Needed in Kernel

To support the API endpoints, the following shell commands need to be implemented in the kernel:

### Orchestration Commands

```
coordctl stats              # Output orchestration statistics
coordctl history [--limit]  # Output JSON array of recent decisions
agentctl list              # Output JSON array of all agents with status
```

### Conflict Commands

```
coordctl conflict-stats         # Output conflict statistics
coordctl conflict-history       # Output JSON array of recent conflicts
coordctl priorities             # Output agent priority table
```

### Deployment Commands

```
deployctl status                # Output current deployment phase status
deployctl history [--limit]     # Output JSON array of phase transitions
deployctl advance [--force]     # Advance to next phase
deployctl rollback              # Rollback to previous phase
deployctl config [options]      # Update deployment configuration
```

### Drift Commands

```
driftctl status                 # Output current drift status
driftctl history [--limit]      # Output JSON array of drift samples
driftctl retrain [options]      # Trigger model retraining
driftctl reset-baseline         # Reset baseline accuracy
```

### Version Control Commands

```
versionctl list [--limit]       # Output JSON array of version history
versionctl log                  # Alias for list
versionctl commit [options]     # Commit new adapter version
versionctl rollback <version>   # Rollback to version
versionctl diff <v1> <v2>       # Compare two versions
versionctl tag <version> <tag>  # Tag a version
versionctl gc [--keep=N]        # Garbage collect old versions
```

---

## Appendix: Kernel Output Formats

For the daemon to parse kernel output, all Phase 2 shell commands should output structured data (JSON preferred, or key=value pairs).

### Example: coordctl stats output

```bash
SIS> coordctl stats
total_decisions=1543
unanimous=892
majority=451
safety_overrides=178
no_consensus=22
avg_latency_us=234
```

### Example: coordctl history output

```bash
SIS> coordctl history --limit 2
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

### Example: deployctl status output

```bash
SIS> deployctl status
{
  "current_phase": {
    "id": "B",
    "name": "Canary",
    "description": "Deploy to 10% of traffic",
    "entered_at": "2025-11-09T10:00:00.000000000Z",
    "min_duration_ms": 43200000,
    "elapsed_ms": 3600000,
    "can_advance": false,
    "traffic_percentage": 10,
    "error_rate": 0.02,
    "success_rate": 0.98
  },
  "auto_advance_enabled": true,
  "auto_rollback_enabled": true,
  "rollback_count": 1,
  "max_rollbacks": 3
}
```

---

## Document Control

**Version History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-11-09 | Claude Code | Initial comprehensive implementation plan |

**Approval:**

- [ ] Technical Lead: _________________
- [ ] Product Manager: _________________
- [ ] QA Lead: _________________

**Related Documents:**

- Phase 2 AI Governance Implementation Plan (docs/AI-ML-KERNEL-IMPLEMENTATION-PLAN.md)
- Phase 2 Test Report (docs/results/PHASE2-TEST-REPORT.md)
- README Build Configurations (README.md#build-configurations)

---

**END OF DOCUMENT**
