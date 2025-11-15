# Phase 2: AI Governance & Multi-Agent Coordination Plan

**Date:** January 2025
**Status:** Planning
**Prerequisites:** Phase 1 AI-Native Implementation (Complete ✅)
**Target Completion:** 1-2 weeks
**Estimated LOC:** ~1,500 lines

---

## Executive Summary

Phase 2 builds upon Phase 1's AI-Native capabilities by adding **production-grade governance** and **multi-agent coordination**. While Phase 1 introduced 5 independent AI components (crash predictor, transformer scheduler, LLM fine-tuning, state inference, AI metrics), Phase 2 ensures they work together harmoniously through orchestration, conflict resolution, drift detection, and version control.

**Key Value Proposition:**
> "Transform independent AI components into a coordinated, self-governing system with enterprise-grade reliability and auditability"

**Primary Goals:**
1. **Multi-Agent Orchestration** - Coordinate Phase 1 components to prevent conflicts
2. **Model Drift Detection** - Monitor AI degradation and trigger retraining
3. **Adapter Version Control** - Git-like versioning for LoRA adapters
4. **Conflict Resolution** - Priority-based decision making when agents disagree
5. **Enhanced Deployment Phases** - Automated phase transitions with safety checks

---

## Motivation

### Current Gaps (Post Phase 1)

**Problem 1: AI Component Conflicts**
```
Scenario: High memory pressure detected

Crash Predictor:    "Trigger compaction NOW!" (85% risk)
Transformer Sched:  "Increase task priority" (conflicts with compaction)
State Inference:    "Continue normal operation" (low confidence)

Result: Undefined behavior - which AI component wins?
```

**Problem 2: Model Degradation Over Time**
```
Week 1:  LoRA adapter trained, 92% accuracy
Week 4:  Accuracy drops to 78% (environment changed)
Week 8:  Accuracy at 65% (model severely degraded)

Issue: No automated detection or retraining trigger
```

**Problem 3: No Adapter History**
```
Scenario: New LoRA adapter deployed, performance tanks

Question: Which previous adapter version worked?
Current: No version tracking, can't rollback
Need: Git-like history of adapter versions
```

**Problem 4: Manual Phase Management**
```
Current: Human manually runs "autoctl phase B"
Issue: No automated transition based on performance
Need: Auto-advance from Learning → Validation → Production
```

---

## Architecture Overview

### Component Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│          AI Governance Layer (Phase 2 - NEW)            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Orchestrator │  │    Conflict  │  │     Drift    │  │
│  │              │  │   Resolver   │  │   Detector   │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│         ▲                 ▲                  ▲          │
│         │                 │                  │          │
└─────────┼─────────────────┼──────────────────┼──────────┘
          │                 │                  │
┌─────────┼─────────────────┼──────────────────┼──────────┐
│         │    Phase 1 AI Components (Existing)│          │
│  ┌──────▼──────┐  ┌──────▼──────┐  ┌─────────▼──────┐  │
│  │   Crash     │  │ Transformer │  │      State     │  │
│  │  Predictor  │  │  Scheduler  │  │   Inference    │  │
│  └─────────────┘  └─────────────┘  └────────────────┘  │
│  ┌─────────────┐  ┌─────────────┐                      │
│  │ LLM Fine-   │  │ AI Metrics  │                      │
│  │   Tuning    │  │  Dashboard  │                      │
│  └─────────────┘  └─────────────┘                      │
└──────────────────────────────────────────────────────────┘
```

### Data Flow

```
1. System Event → Multiple AI components analyze independently
2. Orchestrator collects all recommendations
3. Conflict Resolver applies priority rules
4. Drift Detector monitors decision quality
5. Version Control tracks adapter evolution
6. Final decision executed with full audit trail
```

---

## Phase 2 Components

### 2.1 Multi-Agent Orchestrator

**File:** `crates/kernel/src/ai/orchestrator.rs`
**LOC:** ~400 lines

**Purpose:**
Central coordinator that:
- Queries all Phase 1 AI components
- Aggregates their recommendations
- Resolves conflicts via priority table
- Ensures decisions are coherent

**Core API:**
```rust
pub struct AgentOrchestrator {
    crash_predictor: &'static CrashPredictor,
    transformer_sched: &'static TransformerScheduler,
    state_inference: &'static StateInferenceEngine,
    conflict_resolver: ConflictResolver,
    decision_history: RingBuffer<Decision, 1000>,
}

impl AgentOrchestrator {
    /// Coordinate all AI agents to make a unified decision
    pub fn coordinate(&mut self, system_state: &SystemState)
        -> Result<CoordinatedDecision, OrchestrationError>;

    /// Check if agents are in conflict
    pub fn detect_conflicts(&self, decisions: &[AgentDecision]) -> Vec<Conflict>;

    /// Get coordination statistics
    pub fn get_stats(&self) -> OrchestrationStats;
}
```

**Decision Process:**
```rust
pub enum CoordinatedDecision {
    Unanimous {
        action: Action,
        confidence: f32,
    },
    Majority {
        action: Action,
        agree: Vec<AgentType>,
        disagree: Vec<AgentType>,
    },
    SafetyOverride {
        action: Action,
        overridden_by: AgentType,  // e.g., CrashPredictor
        reason: ExplanationCode,
    },
    NoConsensus {
        defer_to_human: bool,
        conflicting_actions: Vec<(AgentType, Action)>,
    },
}
```

**Example Coordination:**
```rust
let state = SystemState::capture();

// 1. Query all agents
let crash_risk = crash_predictor::predict(&state);      // 87% risk
let sched_action = transformer_sched::decide(&state);   // Increase priority
let inference = state_inference::infer(&state);         // Compact memory

// 2. Detect conflicts
let decisions = vec![
    AgentDecision { agent: CrashPredictor, action: PreventiveCompaction, priority: 100 },
    AgentDecision { agent: TransformerSched, action: IncreasePriority, priority: 60 },
    AgentDecision { agent: StateInference, action: CompactMemory, priority: 80 },
];

// 3. Resolve via orchestrator
let coordinated = orchestrator.coordinate(&decisions)?;

// Result: SafetyOverride - crash predictor wins due to high priority
```

**Integration Points:**
- `autonomy.rs`: Add orchestration before decision execution
- `compliance.rs`: Log coordination decisions for audit
- `shell/autoctl_helpers.rs`: Add `autoctl coordination status` command

---

### 2.2 Conflict Resolution Engine

**File:** `crates/kernel/src/ai/conflict.rs`
**LOC:** ~300 lines

**Purpose:**
Priority-based conflict resolution when AI agents disagree.

**Priority Table:**
```rust
pub struct ConflictResolver {
    /// Priority table: (agent_type, base_priority, context_modifiers)
    priority_table: &'static [(AgentType, u8)],
}

// Default priority hierarchy
const PRIORITY_TABLE: [(AgentType, u8); 5] = [
    (AgentType::CrashPredictor,    100),  // Safety always wins
    (AgentType::StateInference,     80),  // High-confidence suggestions
    (AgentType::TransformerSched,   60),  // Performance optimization
    (AgentType::FineTuner,          40),  // Learning improvements
    (AgentType::Metrics,            20),  // Monitoring (lowest priority)
];
```

**Conflict Types:**
```rust
pub enum Conflict {
    DirectOpposition {
        agent_a: AgentDecision,
        agent_b: AgentDecision,
        incompatibility: String,  // "Compaction vs Priority Increase"
    },
    ResourceContention {
        agents: Vec<AgentDecision>,
        contested_resource: Resource,  // CPU, Memory, I/O
    },
    ConfidenceDisparity {
        high_conf: AgentDecision,  // 95% confidence
        low_conf: AgentDecision,   // 45% confidence
        delta: f32,                // 0.50
    },
}
```

**Resolution Strategies:**
```rust
impl ConflictResolver {
    /// Resolve conflict using priority table
    pub fn resolve_by_priority(&self, conflict: &Conflict) -> Resolution;

    /// Resolve by combining compatible actions
    pub fn resolve_by_synthesis(&self, conflict: &Conflict) -> Resolution;

    /// Defer to human when conflict is severe
    pub fn escalate_to_human(&self, conflict: &Conflict) -> HumanApprovalRequest;
}
```

**Example Resolution:**
```
Conflict: Crash predictor vs Transformer scheduler

Crash Predictor (priority=100, confidence=0.87):
  Action: Trigger compaction NOW

Transformer Scheduler (priority=60, confidence=0.92):
  Action: Increase task priority (conflicts with compaction)

Resolution: SafetyOverride
  - Crash predictor wins (priority 100 > 60)
  - Log: "Scheduler action deferred due to safety concern"
  - Explanation: "High crash risk (87%) overrides performance optimization"
```

**EU AI Act Compliance:**
- Article 13: Conflict resolution is transparent and explainable
- Article 14: Human can override conflict resolution
- Article 16: All conflicts logged with rationale

---

### 2.3 Model Drift Detector

**File:** `crates/kernel/src/llm/drift_detector.rs`
**LOC:** ~350 lines

**Purpose:**
Monitor AI model performance over time, detect degradation, trigger retraining.

**Core Structure:**
```rust
pub struct DriftDetector {
    /// Baseline accuracy when adapter was trained
    baseline_accuracy: f32,

    /// Recent predictions (ring buffer)
    recent_predictions: RingBuffer<Prediction, 1000>,

    /// Drift thresholds
    warning_threshold: f32,   // -5% accuracy
    critical_threshold: f32,  // -15% accuracy

    /// Statistics
    drift_events: AtomicU32,
    retraining_triggered: AtomicU32,
}
```

**Drift Detection Algorithm:**
```rust
impl DriftDetector {
    /// Check for model drift
    pub fn check_drift(&mut self) -> DriftStatus {
        let current_accuracy = self.compute_rolling_accuracy();
        let degradation = self.baseline_accuracy - current_accuracy;

        if degradation >= self.critical_threshold {
            self.drift_events.fetch_add(1, Ordering::Relaxed);

            DriftStatus::Critical {
                baseline: self.baseline_accuracy,
                current: current_accuracy,
                degradation,
                recommendation: DriftAction::RetrainImmediately,
                confidence: self.compute_drift_confidence(),
            }
        } else if degradation >= self.warning_threshold {
            DriftStatus::Warning {
                baseline: self.baseline_accuracy,
                current: current_accuracy,
                degradation,
                recommendation: DriftAction::ScheduleRetraining,
            }
        } else {
            DriftStatus::Normal { current_accuracy }
        }
    }

    /// Trigger retraining if drift is critical
    pub fn auto_retrain_if_needed(&mut self) -> Result<(), &'static str> {
        match self.check_drift() {
            DriftStatus::Critical { .. } => {
                // Collect new training data from recent errors
                let training_data = self.collect_failure_cases();

                // Trigger LoRA fine-tuning
                finetune::load_training_data(training_data);
                finetune::train()?;

                // Reset baseline
                self.baseline_accuracy = self.compute_rolling_accuracy();
                self.retraining_triggered.fetch_add(1, Ordering::Relaxed);

                Ok(())
            }
            _ => Ok(())
        }
    }
}
```

**Drift Metrics:**
```rust
pub struct DriftMetrics {
    /// Accuracy over time
    pub accuracy_history: Vec<(Timestamp, f32)>,

    /// Drift severity (0.0 = no drift, 1.0 = complete degradation)
    pub drift_severity: f32,

    /// Days since last retraining
    pub days_since_retrain: u32,

    /// Number of automatic retrainings
    pub auto_retrain_count: u32,

    /// Prediction confidence trend
    pub confidence_trend: Trend,  // Improving, Stable, Degrading
}
```

**Integration:**
- Hook into Phase 1.4 state inference after each prediction
- Monitor Phase 1.1 crash predictor accuracy
- Trigger Phase 1.3 fine-tuning when drift detected
- Report metrics via Phase 1.5 AI dashboard

**Example Scenario:**
```
Day 1: Robot deployed in warehouse A
  Baseline accuracy: 92%

Day 30: Robot moved to warehouse B (different lighting)
  Current accuracy: 88% → Warning triggered

Day 45: Accuracy drops to 76%
  Critical drift detected!
  → Auto-collect 100 failure cases from new warehouse
  → Trigger LoRA fine-tuning (28 seconds)
  → New baseline: 91% (adapted to warehouse B)
```

---

### 2.4 Adapter Version Control

**File:** `crates/kernel/src/llm/version.rs`
**LOC:** ~400 lines

**Purpose:**
Git-like versioning for LoRA adapters with lineage tracking and rollback.

**Version Structure:**
```rust
#[derive(Clone)]
pub struct AdapterVersion {
    /// Unique version ID (incremental)
    pub version_id: u32,

    /// Parent version (None for initial version)
    pub parent_version: Option<u32>,

    /// Training metadata
    pub metadata: VersionMetadata,

    /// Content hash (SHA-256 of adapter weights)
    pub hash: [u8; 32],

    /// Storage path
    pub storage_path: String,
}

#[derive(Clone)]
pub struct VersionMetadata {
    pub timestamp: u64,
    pub training_examples: usize,
    pub training_duration_ms: u64,
    pub final_loss: f32,
    pub accuracy_improvement: f32,
    pub environment_tag: String,  // e.g., "warehouse_A", "factory_floor_2"
    pub description: String,
}
```

**Version History (Git-like):**
```
v1 (baseline)
  ├─ v2 (trained on warehouse A failures)
  │   ├─ v3 (fine-tuned for low-light conditions)
  │   └─ v4 (adapted to new product types)
  └─ v5 (branched: trained on factory floor data)
      └─ v6 (merged improvements from v3)
```

**Core API:**
```rust
pub struct AdapterVersionControl {
    /// Current active version
    current_version: AtomicU32,

    /// Version history
    versions: Mutex<BTreeMap<u32, AdapterVersion>>,

    /// Storage backend
    storage: AdapterStorage,
}

impl AdapterVersionControl {
    /// Save current adapter as new version
    pub fn commit(&mut self, adapter: &LoRAAdapter, description: &str)
        -> Result<u32, VersionError>;

    /// Rollback to previous version
    pub fn rollback(&mut self, version_id: u32)
        -> Result<LoRAAdapter, VersionError>;

    /// Get version history
    pub fn history(&self) -> Vec<AdapterVersion>;

    /// Compare two versions
    pub fn diff(&self, v1: u32, v2: u32) -> VersionDiff;

    /// Tag a version (e.g., "production", "stable")
    pub fn tag(&mut self, version_id: u32, tag: &str) -> Result<(), VersionError>;

    /// Garbage collect old versions (keep last N)
    pub fn gc(&mut self, keep_count: usize) -> Result<usize, VersionError>;
}
```

**Version Diff:**
```rust
pub struct VersionDiff {
    pub version_a: u32,
    pub version_b: u32,
    pub accuracy_delta: f32,
    pub param_changes: usize,  // How many weights changed
    pub performance_delta: PerformanceMetrics,
}
```

**Example Usage:**
```rust
// Train new adapter
let stats = finetune::train()?;

// Commit to version control
let version_id = version_control.commit(
    &adapter,
    "Adapted to warehouse B lighting conditions"
)?;

// Tag as stable
version_control.tag(version_id, "stable")?;

// Later: performance degrades, rollback
let previous = version_control.rollback(version_id - 1)?;
finetune::import_adapters(previous);

// View history
for version in version_control.history() {
    println!("v{}: {} (+{:.1}% accuracy)",
        version.version_id,
        version.metadata.description,
        version.metadata.accuracy_improvement * 100.0
    );
}
```

**Storage Format:**
```
/var/sis/adapters/
  ├── versions.json          # Metadata index
  ├── v1_baseline.bin        # Adapter weights
  ├── v2_warehouse_a.bin
  ├── v3_low_light.bin
  └── tags/
      ├── production -> v3
      └── stable -> v2
```

**EU AI Act Compliance:**
- Article 16: Complete version history for audit
- Article 15: Can prove model robustness over time
- Article 14: Human can approve/reject version updates

---

### 2.5 Enhanced Deployment Phases

**File:** `crates/kernel/src/ai/deployment.rs`
**LOC:** ~350 lines

**Purpose:**
Automated phase transitions based on performance metrics.

**Phase Definitions (Enhanced):**
```rust
pub struct DeploymentPhase {
    pub phase_id: PhaseId,
    pub constraints: PhaseConstraints,
    pub auto_advance_criteria: AdvanceCriteria,
    pub auto_rollback_criteria: RollbackCriteria,
}

pub enum PhaseId {
    A_Learning,
    B_Validation,
    C_Production,
    D_Emergency,
}

pub struct PhaseConstraints {
    pub max_risk: u8,                     // 0-100
    pub max_autonomous_actions_per_hour: u32,
    pub require_human_approval_above: f32,  // Confidence threshold
    pub max_drift_tolerance: f32,         // Before rollback
    pub min_accuracy: f32,                // Below this = emergency
}

pub struct AdvanceCriteria {
    pub min_decisions: u32,               // Minimum decisions in current phase
    pub min_success_rate: f32,            // Must achieve this success rate
    pub min_uptime_hours: u32,            // Stability requirement
    pub max_incidents: u32,               // Safety requirement
}
```

**Phase Configuration:**
```rust
const PHASES: [DeploymentPhase; 4] = [
    // Phase A: Learning (conservative)
    DeploymentPhase {
        phase_id: PhaseId::A_Learning,
        constraints: PhaseConstraints {
            max_risk: 30,
            max_autonomous_actions_per_hour: 5,
            require_human_approval_above: 0.6,
            max_drift_tolerance: 0.05,
            min_accuracy: 0.85,
        },
        auto_advance_criteria: AdvanceCriteria {
            min_decisions: 100,
            min_success_rate: 0.90,
            min_uptime_hours: 48,
            max_incidents: 2,
        },
        // ...
    },

    // Phase B: Validation (moderate)
    DeploymentPhase {
        phase_id: PhaseId::B_Validation,
        constraints: PhaseConstraints {
            max_risk: 60,
            max_autonomous_actions_per_hour: 20,
            require_human_approval_above: 0.8,
            max_drift_tolerance: 0.10,
            min_accuracy: 0.80,
        },
        auto_advance_criteria: AdvanceCriteria {
            min_decisions: 500,
            min_success_rate: 0.92,
            min_uptime_hours: 168,  // 1 week
            max_incidents: 5,
        },
        // ...
    },

    // Phase C: Production (aggressive)
    DeploymentPhase {
        phase_id: PhaseId::C_Production,
        constraints: PhaseConstraints {
            max_risk: 40,
            max_autonomous_actions_per_hour: 100,
            require_human_approval_above: 0.9,
            max_drift_tolerance: 0.15,
            min_accuracy: 0.75,
        },
        auto_advance_criteria: AdvanceCriteria {
            min_decisions: 0,  // No auto-advance from production
            // ...
        },
        // ...
    },

    // Phase D: Emergency (manual only)
    DeploymentPhase {
        phase_id: PhaseId::D_Emergency,
        constraints: PhaseConstraints {
            max_risk: 10,
            max_autonomous_actions_per_hour: 0,  // All manual
            require_human_approval_above: 1.0,
            max_drift_tolerance: 0.0,
            min_accuracy: 0.0,
        },
        // ...
    },
];
```

**Auto-Transition Logic:**
```rust
impl DeploymentManager {
    /// Check if phase should advance
    pub fn check_auto_advance(&mut self) -> PhaseTransition {
        let current = self.get_current_phase();
        let metrics = self.collect_phase_metrics();

        // Check advance criteria
        if metrics.decisions >= current.auto_advance_criteria.min_decisions &&
           metrics.success_rate >= current.auto_advance_criteria.min_success_rate &&
           metrics.uptime_hours >= current.auto_advance_criteria.min_uptime_hours &&
           metrics.incidents <= current.auto_advance_criteria.max_incidents {

            PhaseTransition::Advance {
                from: current.phase_id,
                to: current.phase_id.next(),
                reason: "All advancement criteria met",
                metrics,
            }
        } else {
            PhaseTransition::Stay { current_metrics: metrics }
        }
    }

    /// Check if phase should rollback
    pub fn check_auto_rollback(&mut self) -> PhaseTransition {
        let current = self.get_current_phase();
        let metrics = self.collect_phase_metrics();

        // Critical: accuracy below minimum
        if metrics.accuracy < current.constraints.min_accuracy {
            return PhaseTransition::Rollback {
                from: current.phase_id,
                to: PhaseId::D_Emergency,
                reason: "Accuracy critically low",
                metrics,
            };
        }

        // Warning: high drift
        if metrics.drift > current.constraints.max_drift_tolerance {
            return PhaseTransition::Rollback {
                from: current.phase_id,
                to: current.phase_id.previous(),
                reason: "Model drift exceeds tolerance",
                metrics,
            };
        }

        PhaseTransition::Stay { current_metrics: metrics }
    }
}
```

**Example Progression:**
```
Week 1 (Phase A - Learning):
  - 120 decisions made
  - 95% success rate
  - 72 hours uptime
  - 1 incident
  → AUTO-ADVANCE to Phase B ✓

Week 3 (Phase B - Validation):
  - 550 decisions made
  - 93% success rate
  - 180 hours uptime
  - 3 incidents
  → AUTO-ADVANCE to Phase C ✓

Week 7 (Phase C - Production):
  - Accuracy drops from 92% to 73% (drift detected)
  → AUTO-ROLLBACK to Phase B ✓
  → Trigger retraining

Week 8 (Phase B - after retraining):
  - Accuracy recovered to 91%
  - 600 decisions, 94% success
  → AUTO-ADVANCE to Phase C ✓
```

**Shell Integration:**
```bash
# Check current phase
sis> autoctl phase status
Current Phase: B (Validation)
Progress to Phase C:
  ✓ Decisions: 550/500 (110%)
  ✓ Success Rate: 93%/92% (101%)
  ✓ Uptime: 180h/168h (107%)
  ✓ Incidents: 3/5 (60%)
→ Ready to advance! Run 'autoctl phase advance' to proceed.

# Auto-advance mode
sis> autoctl phase auto on
Auto-advance enabled. System will transition phases automatically.

# View phase history
sis> autoctl phase history
[2025-01-15 10:00] A → B (Auto-advance: criteria met)
[2025-02-01 14:30] B → C (Auto-advance: criteria met)
[2025-02-14 09:15] C → B (Auto-rollback: drift detected)
[2025-02-18 11:00] B → C (Auto-advance: criteria met)
```

---

## Implementation Plan

### Week 1: Core Infrastructure

**Days 1-2: Multi-Agent Orchestrator**
- [ ] Create `crates/kernel/src/ai/mod.rs` module structure
- [ ] Implement `orchestrator.rs` with decision aggregation
- [ ] Add `CoordinatedDecision` types
- [ ] Integrate with existing Phase 1 components
- [ ] Unit tests for orchestration logic
- [ ] Shell command: `autoctl coordination status`

**Days 3-4: Conflict Resolution**
- [ ] Implement `conflict.rs` with priority table
- [ ] Add conflict detection algorithms
- [ ] Implement resolution strategies (priority, synthesis, escalation)
- [ ] Integration tests with multiple conflicting scenarios
- [ ] Shell command: `autoctl conflicts show`

**Days 5-6: Model Drift Detection**
- [ ] Implement `llm/drift_detector.rs`
- [ ] Add rolling accuracy calculation
- [ ] Implement drift threshold checks
- [ ] Auto-retraining trigger logic
- [ ] Hook into state inference predictions
- [ ] Shell command: `llm drift status`

**Day 7: Testing & Documentation**
- [ ] Integration tests for Week 1 components
- [ ] Update README.md with Phase 2 documentation
- [ ] Create user guide for orchestration features

---

### Week 2: Advanced Features

**Days 8-9: Adapter Version Control**
- [ ] Implement `llm/version.rs` version control system
- [ ] Add storage backend (filesystem with JSON metadata)
- [ ] Implement commit/rollback/history APIs
- [ ] Add tagging system (production, stable, etc.)
- [ ] Garbage collection for old versions
- [ ] Shell commands: `llm version commit`, `llm version rollback`, `llm version history`

**Days 10-11: Enhanced Deployment Phases**
- [ ] Implement `ai/deployment.rs` phase manager
- [ ] Add auto-advance logic based on metrics
- [ ] Add auto-rollback on drift/accuracy drop
- [ ] Phase transition history tracking
- [ ] Integration with existing `autoctl phase` commands
- [ ] Shell commands: `autoctl phase auto on/off`, `autoctl phase history`

**Days 12-13: Integration & Polish**
- [ ] Full integration testing of all Phase 2 components
- [ ] Performance profiling (ensure <5% overhead)
- [ ] EU AI Act compliance verification
- [ ] Update compliance dashboard
- [ ] Shell integration for all new features

**Day 14: Documentation & Release**
- [ ] Complete Phase 2 implementation notes
- [ ] Update EU-AI-ACT-COMPLIANCE-UPDATE.md
- [ ] Create Phase 2 completion report
- [ ] Demo script for showcasing features
- [ ] Merge to main branch

---

## File Structure

```
crates/kernel/src/
├── ai/                              # NEW: Phase 2 AI Governance
│   ├── mod.rs                       # Module definitions
│   ├── orchestrator.rs              # Multi-agent coordinator
│   ├── conflict.rs                  # Conflict resolution engine
│   └── deployment.rs                # Enhanced phase manager
├── llm/
│   ├── mod.rs                       # Update with Phase 2 exports
│   ├── finetune.rs                  # Existing (Phase 1)
│   ├── state_inference.rs           # Existing (Phase 1)
│   ├── drift_detector.rs            # NEW: Model drift detection
│   └── version.rs                   # NEW: Adapter version control
├── control/
│   └── ai_metrics.rs                # Update with Phase 2 metrics
└── shell/
    ├── autoctl_helpers.rs           # Update with Phase 2 commands
    └── llm_helpers.rs               # NEW: LLM version control commands

docs/
├── plans/
│   └── PHASE2-AI-GOVERNANCE-PLAN.md # This document
├── guides/
│   ├── AI-GOVERNANCE-GUIDE.md       # NEW: User guide
│   └── EU-AI-ACT-COMPLIANCE-UPDATE.md # Update for Phase 2
└── results/
    └── PHASE2-COMPLETION-REPORT.md  # Created at completion
```

---

## Success Criteria

### Functional Requirements

✅ **Multi-Agent Orchestration:**
- [ ] All 5 Phase 1 components can be queried simultaneously
- [ ] Conflict detection works for >3 conflicting recommendations
- [ ] Priority-based resolution matches expected outcomes 95%+ of time
- [ ] Human escalation triggers for severe conflicts

✅ **Conflict Resolution:**
- [ ] Priority table correctly ranks all agent types
- [ ] Resolution explanations are human-readable (EU AI Act Article 13)
- [ ] All conflicts logged for audit trail (Article 16)
- [ ] Can handle 10+ decisions/second without performance degradation

✅ **Model Drift Detection:**
- [ ] Detects 5% accuracy drop within 24 hours
- [ ] Auto-retraining triggers when drift exceeds threshold
- [ ] Baseline resets after successful retraining
- [ ] Drift metrics exported via AI dashboard

✅ **Adapter Version Control:**
- [ ] Can store 100+ adapter versions without filesystem issues
- [ ] Rollback completes in <1 second
- [ ] Version history shows parent-child relationships
- [ ] Tags work correctly (production, stable, etc.)
- [ ] Garbage collection preserves tagged versions

✅ **Enhanced Deployment Phases:**
- [ ] Auto-advance works when criteria met
- [ ] Auto-rollback triggers on drift/accuracy drop
- [ ] Phase transition history is accurate
- [ ] Manual override still works (human control)

### Performance Requirements

✅ **Orchestrator:**
- Coordination latency: <10ms per decision
- Memory overhead: <512KB
- Support: 100+ decisions/minute

✅ **Drift Detector:**
- Check interval: Every 100 predictions or 1 hour
- Detection latency: <5ms
- Memory overhead: <256KB (ring buffer)

✅ **Version Control:**
- Commit time: <100ms
- Rollback time: <1 second
- Storage: <10MB per 100 versions
- GC time: <5 seconds for 1000 versions

✅ **Deployment Manager:**
- Phase check interval: Every 1 hour
- Transition time: <100ms
- Memory overhead: <128KB

### EU AI Act Compliance

✅ **Article 13 (Transparency):**
- [ ] Orchestration decisions fully explainable
- [ ] Conflict resolutions include rationale
- [ ] Drift detection alerts human-readable

✅ **Article 14 (Human Oversight):**
- [ ] Human can override orchestrator decisions
- [ ] Manual phase transitions still supported
- [ ] Version rollback requires human approval (optional flag)

✅ **Article 15 (Accuracy/Robustness):**
- [ ] Drift detection ensures accuracy maintained
- [ ] Auto-rollback prevents degraded models in production
- [ ] Version control enables robustness audits

✅ **Article 16 (Recordkeeping):**
- [ ] All orchestration decisions logged
- [ ] Conflict resolutions recorded
- [ ] Drift events tracked
- [ ] Version history complete
- [ ] Phase transitions auditable

---

## Testing Strategy

### Unit Tests

**Orchestrator Tests:**
```rust
#[test]
fn test_unanimous_decision() {
    let decisions = vec![
        AgentDecision { agent: CrashPredictor, action: Compact, confidence: 0.9 },
        AgentDecision { agent: StateInference, action: Compact, confidence: 0.85 },
    ];
    let result = orchestrator.coordinate(&decisions);
    assert_matches!(result, CoordinatedDecision::Unanimous { action: Compact, .. });
}

#[test]
fn test_safety_override() {
    let decisions = vec![
        AgentDecision { agent: CrashPredictor, action: Stop, priority: 100 },
        AgentDecision { agent: TransformerSched, action: Continue, priority: 60 },
    ];
    let result = orchestrator.coordinate(&decisions);
    assert_matches!(result, CoordinatedDecision::SafetyOverride {
        overridden_by: CrashPredictor, ..
    });
}
```

**Drift Detector Tests:**
```rust
#[test]
fn test_drift_detection_critical() {
    let mut detector = DriftDetector::new(0.92);  // 92% baseline

    // Simulate accuracy drop
    for _ in 0..100 {
        detector.record_prediction(Prediction { correct: false });
    }

    let status = detector.check_drift();
    assert_matches!(status, DriftStatus::Critical { degradation, .. } if degradation > 0.15);
}
```

**Version Control Tests:**
```rust
#[test]
fn test_version_lineage() {
    let v1 = vc.commit(&adapter1, "Initial");
    let v2 = vc.commit(&adapter2, "Improved");
    let v3 = vc.commit(&adapter3, "Further improved");

    let history = vc.history();
    assert_eq!(history.len(), 3);
    assert_eq!(history[1].parent_version, Some(v1));
    assert_eq!(history[2].parent_version, Some(v2));
}
```

### Integration Tests

**Multi-Component Scenarios:**
```rust
#[test]
fn test_full_orchestration_pipeline() {
    // 1. Trigger high memory pressure
    let state = SystemState { memory_pressure: 0.90, .. };

    // 2. Phase 1 components analyze
    let crash_risk = crash_predictor::predict(&state);
    let inference = state_inference::infer(&state);

    // 3. Orchestrator coordinates
    let decision = orchestrator.coordinate(&state)?;

    // 4. Verify safety override
    assert_matches!(decision, CoordinatedDecision::SafetyOverride { .. });

    // 5. Check compliance logging
    let logs = compliance::get_recent_logs();
    assert!(logs.iter().any(|l| l.contains("orchestration")));
}
```

**Drift → Retraining → Version Control:**
```rust
#[test]
fn test_drift_retraining_versioning() {
    // 1. Detect drift
    drift_detector.simulate_accuracy_drop(0.92, 0.75);
    let status = drift_detector.check_drift();
    assert_matches!(status, DriftStatus::Critical { .. });

    // 2. Trigger retraining
    drift_detector.auto_retrain_if_needed()?;

    // 3. Verify new version created
    let versions = version_control.history();
    assert_eq!(versions.len(), 2);
    assert!(versions[1].metadata.description.contains("auto-retrain"));
}
```

### System Tests

**Deployment Phase Progression:**
```bash
#!/bin/bash
# Test auto-advance from Phase A → B → C

echo "=== Starting in Phase A ==="
./sis_kernel

# Simulate 100 successful decisions
for i in {1..100}; do
    autoctl tick  # Execute one decision
done

# Check auto-advance
autoctl phase status | grep "Ready to advance"

# Advance to Phase B
autoctl phase advance

# Verify constraints changed
autoctl phase status | grep "max_autonomous_actions_per_hour: 20"
```

**Conflict Resolution Stress Test:**
```bash
# Generate 1000 conflicting decisions
for i in {1..1000}; do
    # Trigger simultaneous recommendations from all agents
    stress_test_conflict
done

# Verify no deadlocks
ps aux | grep sis_kernel  # Should still be running

# Check resolution accuracy
autoctl coordination stats
# Expected: >95% resolved without human intervention
```

---

## Risks & Mitigations

### Risk 1: Performance Overhead

**Concern:** Orchestration adds latency to every AI decision
**Mitigation:**
- Benchmark early: Target <10ms overhead
- Parallel agent queries (don't wait sequentially)
- Cache recent decisions to avoid redundant queries
- Profile with `perf` and optimize hot paths

**Acceptance Criteria:** <5% total overhead on decision latency

---

### Risk 2: Conflict Resolution Complexity

**Concern:** Priority table may not handle all edge cases
**Mitigation:**
- Start with simple priority table, iterate based on real conflicts
- Log all escalations to human for review
- Add resolution strategy plugins (priority, synthesis, voting)
- Document conflict patterns and resolutions

**Acceptance Criteria:** <10% of conflicts escalated to human

---

### Risk 3: Drift Detection False Positives

**Concern:** Normal variance triggers unnecessary retraining
**Mitigation:**
- Use rolling average (last 1000 predictions) to smooth variance
- Require sustained degradation (>1 hour) before triggering
- Add confidence intervals to accuracy measurements
- Allow manual override of auto-retrain

**Acceptance Criteria:** <2% false positive rate

---

### Risk 4: Version Control Storage Growth

**Concern:** Storing 100s of versions fills disk
**Mitigation:**
- Implement garbage collection (keep last N versions)
- Preserve tagged versions (production, stable)
- Compress old versions (gzip)
- Add storage quota warnings

**Acceptance Criteria:** <100MB storage for 100 versions

---

### Risk 5: Phase Auto-Advance Too Aggressive

**Concern:** System advances to production prematurely
**Mitigation:**
- Conservative advancement criteria (92%+ success rate)
- Require sustained performance (168+ hours)
- Allow manual override to stay in current phase
- Add "dry-run" mode to preview advancement

**Acceptance Criteria:** Zero premature production deployments

---

## Metrics & Monitoring

### Phase 2 Metrics Dashboard

**New Metrics (Added to `ai_metrics.rs`):**

```rust
pub struct Phase2Metrics {
    // Orchestration
    pub orchestration_decisions_total: u64,
    pub orchestration_unanimous: u64,
    pub orchestration_conflicts: u64,
    pub orchestration_safety_overrides: u64,
    pub orchestration_latency_ms: f32,

    // Conflict Resolution
    pub conflicts_by_priority: u64,
    pub conflicts_by_synthesis: u64,
    pub conflicts_escalated_to_human: u64,

    // Drift Detection
    pub drift_checks_total: u64,
    pub drift_warnings: u64,
    pub drift_critical_events: u64,
    pub auto_retrainings_triggered: u64,

    // Version Control
    pub adapter_versions_total: u32,
    pub adapter_commits: u64,
    pub adapter_rollbacks: u64,
    pub adapter_gc_runs: u64,

    // Deployment
    pub current_phase: PhaseId,
    pub phase_transitions_total: u64,
    pub phase_auto_advances: u64,
    pub phase_auto_rollbacks: u64,
}
```

**Export Format (Prometheus):**
```
# Orchestration
ai_orchestration_decisions_total 1247
ai_orchestration_conflicts_total 38
ai_orchestration_latency_ms 7.2

# Drift
ai_drift_checks_total 523
ai_drift_warnings_total 12
ai_drift_critical_total 2
ai_auto_retrainings_total 2

# Versions
ai_adapter_versions_total 15
ai_adapter_rollbacks_total 3

# Deployment
ai_current_phase 2  # (B = Validation)
ai_phase_transitions_total 4
```

---

## Documentation Updates

### README.md Updates

Add Phase 2 section after Phase 1:

```markdown
## Phase 2: AI Governance & Multi-Agent Coordination (COMPLETE ✅)

**Status:** PRODUCTION READY - Enterprise AI governance

Phase 2 adds production-grade governance to Phase 1's AI components through
orchestration, conflict resolution, drift detection, and version control.

**Key Achievements:**
- ✅ **Multi-Agent Orchestration** for coordinated AI decisions
- ✅ **Conflict Resolution** with priority-based arbitration
- ✅ **Model Drift Detection** with auto-retraining
- ✅ **Adapter Version Control** with Git-like history
- ✅ **Enhanced Deployment Phases** with auto-advancement
- ✅ **~1,500 lines of code** across 5 new modules

### 2.1 Multi-Agent Orchestration
[documentation...]

### 2.2 Conflict Resolution
[documentation...]

### 2.3 Model Drift Detection
[documentation...]

### 2.4 Adapter Version Control
[documentation...]

### 2.5 Enhanced Deployment Phases
[documentation...]
```

### User Guide

**New Document:** `docs/guides/AI-GOVERNANCE-GUIDE.md`

Contents:
- Introduction to multi-agent systems
- How to monitor orchestration decisions
- Resolving conflicts manually
- Interpreting drift alerts
- Managing adapter versions
- Configuring deployment phases

---

## Rollout Plan

### Phase 2.0: Alpha (Internal Testing)
- Week 1 components only
- Enable on development systems
- Collect feedback from orchestration decisions
- Tune priority table based on conflicts
- **Gate:** `#[cfg(feature = "ai-governance-alpha")]`

### Phase 2.1: Beta (Limited Production)
- All Week 2 components
- Enable drift detection (monitoring only, no auto-retrain)
- Version control without auto-commits
- Manual phase transitions only
- **Gate:** `#[cfg(feature = "ai-governance-beta")]`

### Phase 2.2: Production (Full Release)
- Enable auto-retraining on drift
- Auto-commit adapter versions
- Auto-advance deployment phases (opt-in)
- Full integration with compliance dashboard
- **Gate:** `#[cfg(feature = "ai-governance")]` (default in `ai-ops`)

---

## Future Enhancements (Phase 3+)

**Potential Phase 3 Features:**

1. **Federated Learning**
   - Share LoRA adapters across robot fleet
   - Privacy-preserving aggregation
   - Differential privacy guarantees

2. **Multi-Objective Optimization**
   - Pareto-optimal decision making
   - Trade-off analysis (safety vs performance)
   - Customizable objective weights

3. **Explainable AI Enhancements**
   - SHAP values for feature importance
   - Counterfactual explanations
   - Decision replay for debugging

4. **Advanced Drift Detection**
   - Concept drift vs data drift separation
   - Kolmogorov-Smirnov tests
   - LIME for prediction explanations

5. **Hierarchical Agents**
   - Meta-agent that manages other agents
   - Agent specialization by domain
   - Dynamic agent creation/retirement

---

## Conclusion

Phase 2 transforms the SIS kernel from "5 independent AI components" to "1 coordinated AI system with enterprise-grade governance". By adding orchestration, conflict resolution, drift detection, version control, and enhanced deployment management, we ensure that AI operations are:

✅ **Coordinated** - Agents work together, not against each other
✅ **Reliable** - Drift detection prevents model degradation
✅ **Auditable** - Version control provides complete lineage
✅ **Safe** - Automated rollback on quality degradation
✅ **Compliant** - EU AI Act requirements maintained

**Timeline:** 2 weeks
**Complexity:** Medium
**Risk:** Low (builds on proven Phase 1 foundation)
**Value:** High (essential for production robotics deployment)

---

**Prepared by:** SIS Kernel Team
**Date:** January 2025
**Next Review:** Upon Phase 2 completion
