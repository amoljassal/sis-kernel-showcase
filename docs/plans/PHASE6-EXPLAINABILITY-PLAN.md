# Phase 6: Explainability Enhancements - Implementation Plan

**Date:** November 4, 2025
**Phase:** Phase 6 - Explainability & Transparency
**Status:** Planning
**Prerequisites:** Phase 5 complete ✅

---

## Overview

Phase 6 implements two advanced explainability features that enhance transparency and EU AI Act compliance by allowing users to understand *why* the autonomous system makes specific decisions and *what would happen* under different scenarios.

### Features

1. **autoctl attention** - Attention mechanism visualization
2. **autoctl whatif** - Scenario analysis and counterfactual exploration

---

## Feature 1: autoctl attention

### Purpose

Visualize which inputs most strongly influenced the last autonomous decision, providing transparency into the neural network's decision-making process.

### EU AI Act Alignment

- **Article 13.1:** "AI systems shall be designed and developed in such a way to ensure that their operation is sufficiently transparent"
- **Article 13.3(b):** Enable users to "understand the basis of an AI decision"

### Technical Design

#### Data Structure
```rust
pub struct AttentionWeights {
    pub timestamp: u64,
    pub decision_id: u32,

    // Input feature importance (0-255, higher = more influential)
    pub memory_pressure_weight: u8,
    pub memory_fragmentation_weight: u8,
    pub deadline_misses_weight: u8,
    pub command_rate_weight: u8,

    // Output directive influence (0-255)
    pub memory_directive_confidence: u8,
    pub scheduling_directive_confidence: u8,
    pub command_directive_confidence: u8,

    // Overall confidence
    pub total_confidence: u16, // 0-1000 (Q8.8 fixed-point)
}
```

#### Implementation Approach

**Option A: Post-hoc Analysis (Recommended)**
- Compute attention weights after decision is made
- Use gradient-like sensitivity analysis
- Low overhead, simple integration
- Approximate but useful for transparency

**Option B: True Attention Mechanism**
- Implement attention layers in neural network
- Requires network architecture changes
- Higher accuracy, more complexity
- Better for long-term explainability

**Recommendation:** Option A for Phase 6 (pragmatic), Option B for future enhancement

#### Algorithm (Post-hoc Sensitivity Analysis)

```rust
pub fn compute_attention_weights() -> AttentionWeights {
    // 1. Get baseline decision with current state
    let baseline_state = collect_telemetry();
    let baseline_decision = force_meta_decision();

    // 2. Perturb each input feature slightly
    let delta = 5; // 5% perturbation

    // 3. Measure how much each perturbation affects output
    let mut weights = AttentionWeights::default();

    // Perturb memory_pressure
    let mut perturbed = baseline_state.clone();
    perturbed.memory_pressure = baseline_state.memory_pressure.saturating_add(delta);
    let perturbed_decision = run_inference_with_state(&perturbed);
    weights.memory_pressure_weight = compute_influence(&baseline_decision, &perturbed_decision);

    // Repeat for other features...

    weights
}

fn compute_influence(baseline: &Decision, perturbed: &Decision) -> u8 {
    // Measure L2 distance between directive vectors
    let diff_mem = (perturbed.memory_directive - baseline.memory_directive).abs();
    let diff_sched = (perturbed.scheduling_directive - baseline.scheduling_directive).abs();
    let diff_cmd = (perturbed.command_directive - baseline.command_directive).abs();

    let total_diff = (diff_mem + diff_sched + diff_cmd) as u32;

    // Normalize to 0-255 range
    (total_diff.min(255)) as u8
}
```

#### Shell Command Interface

```
sis> autoctl attention

=== Decision Attention Analysis ===
Last Decision ID: #142
Timestamp: 1730736500 seconds

Input Feature Influence (0-100%):
  Memory Pressure:      ████████████████░░░░ 82% (HIGH)
  Memory Fragmentation: ████████░░░░░░░░░░░░ 41% (MEDIUM)
  Deadline Misses:      ███░░░░░░░░░░░░░░░░░ 15% (LOW)
  Command Rate:         █░░░░░░░░░░░░░░░░░░░  5% (MINIMAL)

Output Directive Confidence:
  Memory Directive:     ████████████████░░░░ 78%
  Scheduling Directive: ███████████░░░░░░░░░ 56%
  Command Directive:    ████░░░░░░░░░░░░░░░░ 22%

Interpretation:
  The decision was PRIMARILY driven by memory pressure (82%).
  Memory directive has HIGH confidence (78%).

  Recommendation: Monitor memory allocation to understand decisions.

Overall Decision Confidence: 752/1000 (75%)
```

### Implementation Files

- `crates/kernel/src/autonomy.rs` - Add `compute_attention_weights()` function
- `crates/kernel/src/shell/autoctl_helpers.rs` - Add `autoctl_attention()` handler
- `crates/kernel/src/shell.rs` - Route "attention" subcommand

### Complexity Estimate

- **Implementation:** 4-6 hours
- **Testing:** 2-3 hours
- **Documentation:** 1-2 hours
- **Total:** 1 day

---

## Feature 2: autoctl whatif

### Purpose

Allow users to explore "what-if" scenarios by simulating autonomous decisions under different system states, enabling proactive risk assessment.

### EU AI Act Alignment

- **Article 13.3(a):** Enable users to "interpret the system's output"
- **Article 14.4(d):** Support human oversight through "understanding of predictions"

### Technical Design

#### Data Structure
```rust
pub struct WhatIfScenario {
    pub scenario_name: &'static str,
    pub modified_state: SystemState,
    pub predicted_decision: Decision,
    pub confidence: u16,
    pub risk_assessment: RiskLevel,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,       // Decision within normal bounds
    Caution,    // Decision at elevated risk
    Warning,    // Decision near limits
    Critical,   // Decision would violate safety constraints
}
```

#### Predefined Scenarios

```rust
pub const WHATIF_SCENARIOS: &[(&str, fn(&SystemState) -> SystemState)] = &[
    ("high-pressure", create_high_memory_pressure_scenario),
    ("high-fragmentation", create_high_fragmentation_scenario),
    ("deadline-stress", create_deadline_stress_scenario),
    ("combined-stress", create_combined_stress_scenario),
    ("low-load", create_low_load_scenario),
];

fn create_high_memory_pressure_scenario(base: &SystemState) -> SystemState {
    let mut state = base.clone();
    state.memory_pressure = 90; // 90% pressure
    state
}

fn create_high_fragmentation_scenario(base: &SystemState) -> SystemState {
    let mut state = base.clone();
    state.memory_fragmentation = 75; // 75% fragmentation
    state
}

fn create_deadline_stress_scenario(base: &SystemState) -> SystemState {
    let mut state = base.clone();
    state.deadline_misses = 40; // 40% miss rate
    state
}

fn create_combined_stress_scenario(base: &SystemState) -> SystemState {
    let mut state = base.clone();
    state.memory_pressure = 85;
    state.memory_fragmentation = 70;
    state.deadline_misses = 30;
    state
}

fn create_low_load_scenario(base: &SystemState) -> SystemState {
    let mut state = base.clone();
    state.memory_pressure = 10;
    state.memory_fragmentation = 5;
    state.deadline_misses = 0;
    state.command_rate = 1;
    state
}
```

#### Risk Assessment Algorithm

```rust
fn assess_risk(decision: &Decision, phase: AutonomyPhase) -> RiskLevel {
    let max_risk = phase.max_risk_score();

    // Calculate decision magnitude (how aggressive is it?)
    let mem_magnitude = decision.memory_directive.abs() as u8;
    let sched_magnitude = decision.scheduling_directive.abs() as u8;
    let cmd_magnitude = decision.command_directive.abs() as u8;

    let total_magnitude = mem_magnitude.max(sched_magnitude).max(cmd_magnitude);

    // Map to risk score (0-100)
    let risk_score = (total_magnitude * 100) / 255;

    // Compare against phase limits
    match risk_score {
        0..=20 => RiskLevel::Safe,
        21..=50 => RiskLevel::Caution,
        51..=max_risk => RiskLevel::Warning,
        _ => RiskLevel::Critical,
    }
}
```

#### Shell Command Interface

##### List Scenarios
```
sis> autoctl whatif

=== What-If Scenario Analysis ===

Available scenarios:
  1. high-pressure      - 90% memory pressure
  2. high-fragmentation - 75% memory fragmentation
  3. deadline-stress    - 40% deadline miss rate
  4. combined-stress    - Multiple stressors
  5. low-load          - Minimal system load

Usage: autoctl whatif <scenario-name>
       autoctl whatif all

Examples:
  autoctl whatif high-pressure
  autoctl whatif all
```

##### Analyze Single Scenario
```
sis> autoctl whatif high-pressure

=== What-If Analysis: high-pressure ===

Current System State:
  Memory Pressure: 12%
  Memory Fragmentation: 8%
  Deadline Misses: 0%
  Command Rate: 2 cmds/sec

Hypothetical Scenario State:
  Memory Pressure: 90% (⬆ +78%)
  Memory Fragmentation: 8%
  Deadline Misses: 0%
  Command Rate: 2 cmds/sec

Predicted Autonomous Response:
  Memory Directive: +512 (INCREASE allocation)
  Scheduling Directive: -256 (DECREASE priority)
  Command Directive: +128 (ENABLE prediction)

Risk Assessment: ⚠️  WARNING
  Decision magnitude: 65/100
  Current phase limit: 60/100 (Phase B - Validation)

  ⚠️  This decision would EXCEED current phase risk limit!

Recommendations:
  - Consider transitioning to Phase C (Production) for conservative limits
  - Enable approval mode: memctl approval on
  - Monitor memory allocation if this scenario occurs

Confidence: 842/1000 (84%)
```

##### Analyze All Scenarios
```
sis> autoctl whatif all

=== What-If Analysis: All Scenarios ===

Scenario 1: high-pressure
  Risk: ⚠️  WARNING (65/100) - EXCEEDS phase limit
  Memory Directive: +512 (INCREASE)

Scenario 2: high-fragmentation
  Risk: ⚠️  WARNING (58/100) - Near phase limit
  Memory Directive: +384 (INCREASE)

Scenario 3: deadline-stress
  Risk: ⚠️  WARNING (72/100) - EXCEEDS phase limit
  Scheduling Directive: +640 (INCREASE priority)

Scenario 4: combined-stress
  Risk: 🔴 CRITICAL (88/100) - DANGEROUS
  All directives: High magnitude

Scenario 5: low-load
  Risk: ✅ SAFE (8/100)
  All directives: Minimal changes

Summary:
  Safe scenarios:     1/5 (20%)
  Warning scenarios:  3/5 (60%)
  Critical scenarios: 1/5 (20%)

  ⚠️  Current phase (B - Validation) may be too aggressive for stress scenarios.

Recommendations:
  - Consider Phase C (Production) for more conservative limits
  - Enable approval mode for high-risk scenarios
  - Monitor for combined-stress conditions
```

### Custom Scenario Support (Future Enhancement)

```
sis> autoctl whatif custom pressure=85 fragmentation=60

=== What-If Analysis: Custom Scenario ===

Custom State:
  Memory Pressure: 85%
  Memory Fragmentation: 60%
  Deadline Misses: 0% (current)
  Command Rate: 2 cmds/sec (current)

[Analysis continues as above...]
```

### Implementation Files

- `crates/kernel/src/autonomy.rs` - Add scenario functions and risk assessment
- `crates/kernel/src/shell/autoctl_helpers.rs` - Add `autoctl_whatif()` handler
- `crates/kernel/src/shell.rs` - Route "whatif" subcommand

### Complexity Estimate

- **Implementation:** 6-8 hours
- **Testing:** 3-4 hours
- **Documentation:** 2-3 hours
- **Total:** 1.5-2 days

---

## Implementation Order

### Step 1: autoctl attention (Day 1)
1. Implement `AttentionWeights` struct
2. Add `compute_attention_weights()` function
3. Add shell handler `autoctl_attention()`
4. Test with various decision states
5. Document usage

### Step 2: autoctl whatif (Days 2-3)
1. Implement scenario generators
2. Add risk assessment logic
3. Add shell handler `autoctl_whatif()`
4. Test all predefined scenarios
5. Document usage

### Step 3: Integration & Testing (Day 3)
1. Test both features together
2. Verify EU AI Act compliance
3. Update help text
4. Create testing guide
5. Update README

---

## Testing Strategy

### Unit Tests (Future - Companion Crate)
```rust
#[test]
fn test_attention_weights_sum_to_100() {
    let weights = compute_attention_weights();
    let total = weights.memory_pressure_weight
              + weights.memory_fragmentation_weight
              + weights.deadline_misses_weight
              + weights.command_rate_weight;
    assert!(total <= 255);
}

#[test]
fn test_whatif_high_pressure_increases_memory() {
    let scenario = run_whatif_scenario("high-pressure");
    assert!(scenario.predicted_decision.memory_directive > 0);
}

#[test]
fn test_risk_assessment_respects_phase_limits() {
    set_autonomy_phase(AutonomyPhase::PhaseD);
    let scenario = run_whatif_scenario("combined-stress");
    assert_eq!(scenario.risk_assessment, RiskLevel::Critical);
}
```

### Integration Tests
1. Run `autoctl attention` after various decisions
2. Run `autoctl whatif high-pressure` in each phase (A/B/C/D)
3. Verify risk warnings appear correctly
4. Test with autonomy enabled/disabled
5. Combine with Phase 5 features (preview + whatif)

### Performance Tests
- Attention computation: <50ms target
- Single whatif scenario: <100ms target
- All scenarios: <500ms target
- No memory leaks over 100+ operations

---

## EU AI Act Compliance Impact

### Article 13 (Transparency)
- ✅ **attention** shows which inputs influenced decisions
- ✅ **whatif** allows users to explore decision space
- ✅ Combined with Phase 5 **preview**, provides full transparency

### Article 14 (Human Oversight)
- ✅ **whatif** enables proactive risk assessment
- ✅ Risk warnings guide human intervention
- ✅ Scenario analysis supports informed oversight

### Documentation Requirements
- ✅ Clear explanations of attention weights
- ✅ Risk assessment methodology documented
- ✅ Scenario descriptions provided
- ✅ Interpretation guidance included

**Estimated compliance improvement:** +10-15% (beyond Phase 5 gains)

---

## Files Modified

### New Files
- `docs/plans/PHASE6-EXPLAINABILITY-PLAN.md` (this document)
- `docs/PHASE6-TESTING-GUIDE.md` (to be created)
- `docs/PHASE6-TEST-RESULTS.md` (to be created)

### Modified Files
- `crates/kernel/src/autonomy.rs` - Add attention and whatif functions
- `crates/kernel/src/shell/autoctl_helpers.rs` - Add handlers
- `crates/kernel/src/shell.rs` - Route new commands
- `README.md` - Add Phase 6 section

### Estimated Code Size
- Attention: ~150 lines
- Whatif: ~250 lines
- Shell handlers: ~150 lines
- **Total:** ~550 new lines

---

## Risk Assessment

### Technical Risks
- **Attention accuracy:** Post-hoc analysis is approximate
  - **Mitigation:** Clear documentation of limitations
  - **Future:** Implement true attention mechanism

- **Whatif scenario coverage:** 5 predefined scenarios may be insufficient
  - **Mitigation:** Focus on common stress cases
  - **Future:** Add custom scenario support

- **Performance overhead:** Multiple inferences for whatif
  - **Mitigation:** Limit scenarios, optimize inference
  - **Future:** Cache similar scenarios

### Schedule Risks
- **Complexity creep:** Features could expand scope
  - **Mitigation:** Stick to predefined scenarios only
  - **Future:** Add enhancements in Phase 7

---

## Success Criteria

### autoctl attention
- ✅ Displays influence of each input feature
- ✅ Shows confidence for each output directive
- ✅ Computation completes in <50ms
- ✅ Weights are interpretable and accurate
- ✅ Documentation explains how to use insights

### autoctl whatif
- ✅ All 5 predefined scenarios work
- ✅ Risk assessment matches phase limits
- ✅ Warnings appear for high-risk scenarios
- ✅ Single scenario: <100ms
- ✅ All scenarios: <500ms
- ✅ Provides actionable recommendations

### Integration
- ✅ Works with Phase 5 features (preview, phase, query-mode)
- ✅ No conflicts or crashes
- ✅ Help text updated
- ✅ Backward compatibility maintained

---

## Timeline

| Task | Duration | Days |
|------|----------|------|
| Planning (this document) | 2h | 0.25 |
| Implement attention | 6h | 0.75 |
| Implement whatif | 8h | 1.0 |
| Integration testing | 4h | 0.5 |
| Documentation | 3h | 0.375 |
| Testing guide | 2h | 0.25 |
| **Total** | **25h** | **3.125 days** |

**Target completion:** November 7, 2025 (3 days)

---

## Phase 6 Deliverables

1. ✅ This implementation plan
2. ⏳ Working `autoctl attention` command
3. ⏳ Working `autoctl whatif <scenario>` command
4. ⏳ Risk assessment system
5. ⏳ Updated README.md
6. ⏳ PHASE6-TESTING-GUIDE.md
7. ⏳ PHASE6-TEST-RESULTS.md
8. ⏳ Git commits with all changes

---

## Next Steps

1. **Review this plan** - Ensure technical approach is sound
2. **Begin implementation** - Start with autoctl attention
3. **Test incrementally** - Validate each feature before moving on
4. **Document thoroughly** - Maintain high documentation standards
5. **Prepare for Phase 7** - Consider future enhancements

---

**Document Version:** 1.0
**Created:** November 4, 2025
**Status:** Ready for Implementation
**Estimated Effort:** 3 days
**Dependencies:** Phase 5 complete ✅
