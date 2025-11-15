# AI Governance & Multi-Agent Coordination Guide

**Phase 2 Implementation - User Guide**
**Date:** November 2025
**Version:** 1.0

---

## Table of Contents

1. [Overview](#overview)
2. [Multi-Agent Orchestration](#multi-agent-orchestration)
3. [Conflict Resolution](#conflict-resolution)
4. [Model Drift Detection](#model-drift-detection)
5. [Adapter Version Control](#adapter-version-control)
6. [Enhanced Deployment Phases](#enhanced-deployment-phases)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

---

## Overview

Phase 2 transforms the SIS kernel from having 5 independent AI components (Phase 1) into a coordinated, self-governing AI system with enterprise-grade reliability and auditability.

### Key Benefits

- **Coordination**: Multiple AI agents work together without conflicts
- **Safety**: Automatic conflict resolution with safety-first priorities
- **Reliability**: Model drift detection prevents degraded performance
- **Auditability**: Complete version history of all AI models
- **Automation**: Automated phase transitions based on performance

### Architecture

```
┌─────────────────────────────────────────────────────┐
│          AI Governance Layer (Phase 2)              │
│  ┌──────────────┐  ┌──────────────┐                │
│  │ Orchestrator │  │   Conflict   │                │
│  │              │  │   Resolver   │                │
│  └──────────────┘  └──────────────┘                │
└─────────────────────────────────────────────────────┘
         │                 │
┌────────┼─────────────────┼──────────────────────────┐
│        │  Phase 1 AI Components                     │
│  ┌─────▼──────┐  ┌──────▼──────┐                   │
│  │   Crash    │  │ Transformer │                   │
│  │ Predictor  │  │  Scheduler  │                   │
│  └────────────┘  └─────────────┘                   │
└──────────────────────────────────────────────────────┘
```

---

## Multi-Agent Orchestration

### What is Orchestration?

The orchestrator coordinates multiple AI agents (crash predictor, transformer scheduler, state inference, etc.) to make unified decisions instead of conflicting recommendations.

### How It Works

1. **System Event** occurs (e.g., high memory pressure)
2. **Orchestrator** queries all relevant AI agents
3. Each agent provides its **recommendation** with confidence level
4. Orchestrator **aggregates** recommendations
5. **Conflict resolver** handles any disagreements
6. Final **coordinated decision** is executed

### Decision Types

#### Unanimous Decision
All agents agree on the same action.

```
Crash Predictor:    Compact memory (confidence: 90%)
State Inference:    Compact memory (confidence: 85%)
Transformer Sched:  Compact memory (confidence: 88%)

→ Decision: Unanimous - Compact memory (avg confidence: 87.7%)
```

#### Majority Decision
Most agents agree, some dissent.

```
Crash Predictor:    Compact memory (90%)
State Inference:    Compact memory (85%)
Transformer Sched:  Increase priority (70%)

→ Decision: Majority - Compact memory (2 agree, 1 disagrees)
```

#### Safety Override
High-priority safety agent overrides others.

```
Crash Predictor:    STOP - Critical crash risk (95%)
State Inference:    Continue normal operation (60%)
Transformer Sched:  Increase priority (80%)

→ Decision: Safety Override by Crash Predictor
   Reason: Critical crash risk detected (95% confidence)
```

#### No Consensus
Agents deeply divided, defer to human.

```
Agent A: Action X (50% confidence)
Agent B: Action Y (48% confidence)
Agent C: Action Z (52% confidence)

→ Decision: No Consensus - Defer to human operator
```

### Code Example

```rust
use sis_kernel::ai::{AgentOrchestrator, AgentDecision, AgentType, Action};

// Create orchestrator
let orchestrator = AgentOrchestrator::new();

// Collect agent decisions
let decisions = vec![
    AgentDecision::new(
        AgentType::CrashPredictor,
        Action::PreventiveCompaction,
        0.9
    ),
    AgentDecision::new(
        AgentType::StateInference,
        Action::CompactMemory,
        0.85
    ),
];

// Coordinate
let decision = orchestrator.coordinate(&decisions)?;

match decision {
    CoordinatedDecision::Unanimous { action, confidence, .. } => {
        println!("All agents agree: {} ({:.1}%)",
            action.description(), confidence * 100.0);
    }
    CoordinatedDecision::SafetyOverride { action, overridden_by, reason, .. } => {
        println!("Safety override by {}: {}",
            overridden_by.name(), reason);
    }
    _ => {}
}
```

---

## Conflict Resolution

### Priority Table

When agents disagree, the conflict resolver uses a priority table:

| Agent Type | Base Priority | Purpose |
|------------|---------------|---------|
| Crash Predictor | 100 | Safety always wins |
| State Inference | 80 | High-confidence suggestions |
| Transformer Scheduler | 60 | Performance optimization |
| Fine-Tuner | 40 | Learning improvements |
| Metrics | 20 | Monitoring only |

### Effective Priority

Effective priority = Base priority × Confidence

Example:
- Crash Predictor (priority 100) at 50% confidence = 50 effective priority
- Transformer Scheduler (priority 60) at 90% confidence = 54 effective priority
- **Winner**: Transformer Scheduler (54 > 50)

### Conflict Types

#### Direct Opposition

Two agents want incompatible actions.

```
Crash Predictor:    Trigger compaction NOW
Transformer Sched:  Increase task priority (conflicts with compaction)

Resolution: Crash predictor wins (priority 100 > 60)
```

#### Resource Contention

Multiple agents want the same resource.

```
Agent A: Need CPU for inference
Agent B: Need CPU for scheduling
Agent C: Need CPU for compaction

Resolution: Highest priority agent gets the resource
```

#### Confidence Disparity

One agent very confident, another uncertain.

```
Agent A: Action X (95% confident)
Agent B: Action Y (45% confident)

Resolution: Agent A wins due to high confidence delta
```

### Code Example

```rust
use sis_kernel::ai::conflict::{ConflictResolver, Conflict};

let resolver = ConflictResolver::new();

// Detect conflicts
let conflicts = resolver.detect_conflicts(&decisions);

for conflict in conflicts {
    match conflict {
        Conflict::DirectOpposition { agent_a, agent_b, .. } => {
            let resolution = resolver.resolve_by_priority(&conflict);
            println!("Conflict resolved: {:?}", resolution);
        }
        _ => {}
    }
}
```

---

## Model Drift Detection

### What is Model Drift?

Model drift occurs when an AI model's performance degrades over time due to:
- Environmental changes (e.g., warehouse A → warehouse B)
- New data patterns (e.g., new product types)
- Sensor degradation
- Distribution shift

### How It Works

1. **Baseline** accuracy established when model trained (e.g., 92%)
2. **Monitor** predictions continuously (rolling window of 1000 predictions)
3. **Compare** current accuracy to baseline
4. **Alert** on degradation:
   - Warning: -5% accuracy (e.g., 92% → 87%)
   - Critical: -15% accuracy (e.g., 92% → 77%)
5. **Auto-retrain** when critical drift detected

### Drift States

#### Normal
```
Baseline: 92%
Current:  91%
Status:   Normal (within tolerance)
```

#### Warning
```
Baseline: 92%
Current:  87%
Status:   Warning (-5% degradation)
Action:   Schedule retraining
```

#### Critical
```
Baseline: 92%
Current:  76%
Status:   Critical (-16% degradation)
Action:   Retrain immediately!
```

### Auto-Retraining

When critical drift detected:

1. **Collect** recent failure cases (incorrect predictions)
2. **Generate** training data from failures
3. **Trigger** LoRA fine-tuning (Phase 1.3)
4. **Update** baseline accuracy
5. **Commit** new adapter version (Version Control)

### Code Example

```rust
use sis_kernel::llm::{DriftDetector, Prediction};

// Create detector with baseline
let detector = DriftDetector::new(0.92);  // 92% baseline

// Record predictions
for prediction in predictions {
    detector.record_prediction(prediction);
}

// Check for drift
match detector.check_drift() {
    DriftStatus::Normal { current_accuracy } => {
        println!("Normal: {:.1}%", current_accuracy * 100.0);
    }
    DriftStatus::Critical { baseline, current, degradation, .. } => {
        println!("CRITICAL DRIFT: {:.1}% → {:.1}% (Δ{:.1}%)",
            baseline * 100.0, current * 100.0, degradation * 100.0);

        // Auto-retrain
        detector.auto_retrain_if_needed()?;
    }
    _ => {}
}
```

---

## Adapter Version Control

### Git-like Versioning

LoRA adapters are versioned like Git commits:

```
v1 (baseline)
├─ v2 (trained on warehouse A failures)
│   ├─ v3 (fine-tuned for low-light conditions)
│   └─ v4 (adapted to new product types)
└─ v5 (branched: trained on factory floor data)
    └─ v6 (merged improvements from v3)
```

### Operations

#### Commit
Save current adapter as new version.

```rust
use sis_kernel::llm::AdapterVersionControl;

let vc = AdapterVersionControl::new();

// After training
let version_id = vc.commit("Adapted to warehouse B lighting")?;
println!("Committed as version {}", version_id);

// Tag as stable
vc.tag(version_id, "stable")?;
```

#### Rollback
Restore previous version.

```rust
// Performance degraded, rollback to v2
vc.rollback(2)?;
println!("Rolled back to version 2");
```

#### History
View version lineage.

```rust
for version in vc.history() {
    println!("v{}: {} ({} examples, {:.1}% improvement)",
        version.version_id,
        version.metadata.description,
        version.metadata.training_examples,
        version.metadata.accuracy_improvement * 100.0
    );
}
```

Output:
```
v1: Baseline adapter (0 examples, 0.0% improvement)
v2: Warehouse A adaptation (150 examples, 3.2% improvement)
v3: Low-light optimization (75 examples, 1.8% improvement)
v4: Product type expansion (200 examples, 2.5% improvement)
```

#### Diff
Compare two versions.

```rust
let diff = vc.diff(2, 4)?;
println!("v2 → v4: {:.1}% accuracy improvement, {} params changed",
    diff.accuracy_delta * 100.0,
    diff.param_changes
);
```

#### Tags
Mark important versions.

```
/var/sis/adapters/
  ├── v1_baseline.bin
  ├── v2_warehouse_a.bin
  ├── v3_low_light.bin
  └── tags/
      ├── production → v3
      └── stable → v2
```

#### Garbage Collection
Clean up old versions.

```rust
// Keep last 10 versions, delete older ones
let removed = vc.gc(10)?;
println!("Removed {} old versions", removed);
```

---

## Enhanced Deployment Phases

### Phase Progression

```
Phase A (Learning)    →    Phase B (Validation)    →    Phase C (Production)
Conservative              Moderate                      Aggressive
5 actions/hour            20 actions/hour               100 actions/hour
48h minimum               168h minimum                  Manual advance only
90% success required      92% success required          ----
```

### Phase Definitions

#### Phase A: Learning
**Purpose**: Initial deployment, conservative operation

- Max risk: 30/100
- Max autonomous actions: 5 per hour
- Human approval required above 60% confidence
- Advancement criteria:
  - 100+ decisions made
  - 90%+ success rate
  - 48+ hours uptime
  - ≤2 incidents

#### Phase B: Validation
**Purpose**: Extended testing, moderate autonomy

- Max risk: 60/100
- Max autonomous actions: 20 per hour
- Human approval required above 80% confidence
- Advancement criteria:
  - 500+ decisions made
  - 92%+ success rate
  - 168+ hours uptime (1 week)
  - ≤5 incidents

#### Phase C: Production
**Purpose**: Full deployment, aggressive optimization

- Max risk: 40/100 (safety critical)
- Max autonomous actions: 100 per hour
- Human approval required above 90% confidence
- No auto-advance (manual only)
- Auto-rollback on drift/accuracy drop

#### Phase D: Emergency
**Purpose**: Manual control only

- Max risk: 10/100
- Max autonomous actions: 0 (all manual)
- Triggered by:
  - Critical accuracy drop
  - Severe model drift
  - Too many incidents

### Auto-Advancement

System automatically advances when criteria met:

```rust
use sis_kernel::ai::deployment::{DeploymentManager, PhaseTransition};

let manager = DeploymentManager::new();

// Record decisions
for _ in 0..100 {
    manager.record_decision(true);  // success
}

// Check if should advance
match manager.check_auto_advance() {
    PhaseTransition::Advance { from, to, reason, .. } => {
        println!("Auto-advancing {} → {}: {}",
            from.name(), to.name(), reason);
        manager.apply_transition(&transition);
    }
    PhaseTransition::Stay { .. } => {
        println!("Staying in current phase");
    }
    _ => {}
}
```

### Auto-Rollback

System automatically rolls back on degradation:

```rust
// Check if should rollback
match manager.check_auto_rollback() {
    PhaseTransition::Rollback { from, to, reason, .. } => {
        println!("Auto-rolling back {} → {}: {}",
            from.name(), to.name(), reason);
        manager.apply_transition(&transition);
    }
    _ => {}
}
```

---

## Best Practices

### 1. Orchestration

✅ **DO**:
- Let orchestrator coordinate all AI decisions
- Trust safety overrides (crash predictor)
- Monitor orchestration statistics
- Review no-consensus cases

❌ **DON'T**:
- Bypass orchestrator for ad-hoc decisions
- Override safety decisions without review
- Ignore repeated conflicts (tune priority table)

### 2. Conflict Resolution

✅ **DO**:
- Adjust priority table based on your use case
- Log and review all escalated conflicts
- Use synthesis when actions are compatible
- Document conflict patterns

❌ **DON'T**:
- Set all agents to same priority
- Ignore resource contention
- Disable conflict resolution

### 3. Drift Detection

✅ **DO**:
- Set appropriate baseline accuracy
- Monitor drift metrics regularly
- Collect diverse training data
- Test retrained models before production

❌ **DON'T**:
- Set thresholds too aggressive (false positives)
- Ignore warning-level drift
- Skip baseline updates after retraining
- Disable auto-retrain in production (unless supervised)

### 4. Version Control

✅ **DO**:
- Commit after every training session
- Use descriptive commit messages
- Tag production versions
- Run garbage collection regularly
- Keep rollback versions accessible

❌ **DON'T**:
- Skip version commits
- Delete tagged versions
- Rollback without testing
- Let storage fill up

### 5. Deployment Phases

✅ **DO**:
- Start in Phase A (Learning)
- Let system auto-advance when ready
- Monitor phase metrics
- Use auto-rollback for safety

❌ **DON'T**:
- Jump directly to Phase C
- Disable auto-rollback
- Ignore incident counts
- Force advancement without meeting criteria

---

## Troubleshooting

### Orchestration Issues

**Problem**: Too many "No Consensus" decisions

**Solutions**:
- Review agent confidence levels (too similar?)
- Adjust priority table
- Check for truly incompatible actions
- Consider adding synthesis strategies

---

**Problem**: Safety overrides too frequent

**Solutions**:
- Review crash predictor sensitivity
- Check if system is actually at risk
- Ensure other agents respect safety constraints
- Lower non-safety agent confidence thresholds

---

### Conflict Resolution Issues

**Problem**: Wrong agent wins conflicts

**Solutions**:
- Review priority table
- Check effective priorities (base × confidence)
- Ensure confidence levels are calibrated
- Document expected behavior for your use case

---

### Drift Detection Issues

**Problem**: False positive drift alerts

**Solutions**:
- Increase warning threshold (e.g., -5% → -8%)
- Use longer rolling window (>1000 predictions)
- Check for data quality issues
- Ensure baseline accuracy is correct

---

**Problem**: Drift not detected when it should be

**Solutions**:
- Lower critical threshold (e.g., -15% → -10%)
- Check prediction recording
- Verify accuracy calculation
- Ensure sufficient prediction samples

---

### Version Control Issues

**Problem**: Storage filling up

**Solutions**:
- Run garbage collection more frequently
- Reduce retention count (e.g., keep last 20 instead of 100)
- Compress old versions
- Delete untagged versions

---

**Problem**: Rollback doesn't improve performance

**Solutions**:
- Verify version ID is correct
- Check if environment changed (version may not match)
- Review version metadata (accuracy improvement)
- Consider creating new version instead

---

### Deployment Phase Issues

**Problem**: Not advancing to next phase

**Solutions**:
- Check advancement criteria progress
- Verify success rate calculation
- Ensure uptime counter is working
- Review incident count (may be too high)

---

**Problem**: Unexpected rollback to previous phase

**Solutions**:
- Review auto-rollback criteria
- Check accuracy metrics
- Verify drift detection settings
- Examine incident logs

---

## Conclusion

Phase 2 AI Governance transforms independent AI components into a coordinated, self-governing system. By following this guide and the best practices outlined, you can deploy robust, production-grade AI systems with confidence.

### Key Takeaways

1. **Orchestration** ensures AI agents work together harmoniously
2. **Conflict Resolution** provides transparent, priority-based arbitration
3. **Drift Detection** maintains model quality over time
4. **Version Control** enables safe experimentation and rollback
5. **Deployment Phases** provide graduated autonomy with safety checks

### Next Steps

- Review the [Phase 2 Plan](../plans/PHASE2-AI-GOVERNANCE-PLAN.md)
- Check the [Phase 2 Completion Report](../results/PHASE2-COMPLETION-REPORT.md)
- Explore the [EU AI Act Compliance Update](../guides/EU-AI-ACT-COMPLIANCE-UPDATE.md)

### Support

For issues or questions:
- Check [Troubleshooting](#troubleshooting) section above
- Review code examples in this guide
- Consult inline documentation in source code
- Create an issue in the repository

---

**Document Version**: 1.0
**Last Updated**: November 2025
**Maintained by**: SIS Kernel Team
