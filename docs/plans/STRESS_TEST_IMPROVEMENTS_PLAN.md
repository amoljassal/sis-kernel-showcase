# Stress Test Improvements Plan

**Date**: 2025-01-10
**Status**: Implementation
**Priority**: High

## Executive Summary

Enhance stress tests to demonstrate realistic failure scenarios, broader autonomy impact, and improved observability. Current tests show 100% success rates and limited AI effectiveness metrics.

## Identified Weaknesses

### 1. No Failure Paths (Critical)
**Problem**: All tests show 0% failure rate, 100% success
- Chaos test: 0 failed recoveries
- Memory test: No allocation failures
- Cannot validate recovery mechanisms

**Impact**: Unable to demonstrate resilience or validate error handling

### 2. Limited Autonomy Impact (High)
**Problem**: AI only affects OOM events, not broader metrics
- Peak pressure unchanged (99% both on/off)
- Avg pressure unchanged (85% both on/off)
- Compaction count unchanged
- Only shows: OOM 6→4 reduction

**Impact**: Autonomy appears ineffective beyond OOM prevention

### 3. Output Truncation (Medium)
**Problem**: Chaos output cuts off mid-list, limited history
- Chaos event list truncated
- Report history limited to 16 entries

**Impact**: Incomplete observability

## Implementation Plan

### Phase 1: Failure Injection (Priority 1)

#### 1.1 Add Failure Rate Parameters
**Files**: `crates/kernel/src/stress_test.rs`, `crates/kernel/src/shell/stresstest_helpers.rs`

Add to `StressTestConfig`:
```rust
pub struct StressTestConfig {
    // ... existing fields
    pub failure_rate: u8,        // 0-100 percentage
    pub oom_probability: u8,     // 0-100 for memory tests
    pub inject_failures: bool,   // Enable failure injection
}
```

#### 1.2 Implement Failure Scenarios
**Location**: `crates/kernel/src/stress_test.rs`

**Chaos Test Failures** (lines 850-950):
- Add failed recovery tracking
- Inject random failures based on `failure_rate`
- Types: allocation failure, timeout, resource unavailable

```rust
if prng::rand_range(0, 100) < config.failure_rate {
    metrics.failed_recoveries += 1;
    // Simulate failure handling
} else {
    metrics.successful_recoveries += 1;
}
```

**Memory Test Failures** (lines 220-350):
- Add `oom_probability` for allocation failures
- Track allocation failure recovery
- Show graceful degradation

#### 1.3 Shell Command Updates
**Location**: `crates/kernel/src/shell/stresstest_helpers.rs`

Add parameters:
```bash
stresstest chaos --duration 10000 --failure-rate 10
stresstest memory --duration 10000 --oom-probability 5
```

Parse `--failure-rate` and `--oom-probability` flags

### Phase 2: Enhanced Autonomy Impact (Priority 1)

#### 2.1 Proactive Compaction Trigger
**Files**: `crates/kernel/src/autonomy.rs`, `crates/kernel/src/heap.rs`

**Current Behavior**:
- Autonomy only reacts to near-OOM (95%+ pressure)

**Enhanced Behavior**:
- Trigger proactive compaction at 75% pressure
- Reduce peak pressure through early intervention
- Track compactions triggered by autonomy

**Implementation**:
```rust
// In autonomy decision loop
if memory_pressure > 75 && memory_pressure < 90 {
    // Proactive intervention
    trigger_early_compaction();
    AUTONOMY_METRICS.record_proactive_compaction();
}
```

#### 2.2 Pressure Reduction Tracking
**File**: `crates/kernel/src/autonomy_metrics.rs`

Add metrics:
```rust
pub struct AutonomyMetrics {
    // ... existing
    pub peak_pressure_reduced: u32,      // Times peak was lowered
    pub avg_pressure_delta: i32,         // Avg pressure change
    pub early_interventions: u32,        // Interventions before crisis
}
```

#### 2.3 Compare Mode Enhancements
**File**: `crates/kernel/src/shell/stresstest_helpers.rs` (lines 78-146)

Show additional comparisons:
```
Impact:
  [+] Peak pressure reduced by 12% (99% -> 87%)
  [+] Avg pressure reduced by 8% (85% -> 77%)
  [+] Proactive compactions: 15
  [+] OOM events reduced by 2 (6 -> 4)
  [+] 244 AI interventions (15 early, 229 reactive)
```

### Phase 3: Output Improvements (Priority 2)

#### 3.1 Chaos Output Pagination
**File**: `crates/kernel/src/stress_test.rs` (lines 850-950)

Add output limiting:
```rust
// Print first 20 and last 20 events
if total_events > 40 {
    print_events(&events[0..20]);
    uart_print(b"  ... (truncated) ...\n");
    print_events(&events[total_events-20..]);
} else {
    print_events(&events);
}
```

#### 3.2 Report History Flag
**File**: `crates/kernel/src/shell/stresstest_helpers.rs` (lines 148-152)

Add `--all` flag:
```bash
stresstest report           # Last 16
stresstest report --all     # All history
```

#### 3.3 Enhanced Formatting
Improve visual hierarchy:
- Use clear section headers
- Add summary tables
- Show deltas with +/- indicators

### Phase 4: Documentation & Examples (Priority 2)

#### 4.1 README Updates
**File**: `README.md` (Week 7.1 section ~line 6510)

Add examples showing:

1. **Failure Injection**:
```bash
# Test with 10% failure rate
stresstest chaos --duration 10000 --failure-rate 10

# Expected output:
# Chaos events: 340
# Successful recoveries: 306
# Failed recoveries: 34
# Success rate: 90%
```

2. **Variability**:
```bash
# Run with 20% noise
stresstest compare memory --duration 10000 --noise 20

# Expected: Different results each run
# Run 1: 340 chaos events, 4 OOMs
# Run 2: 383 chaos events, 6 OOMs
# Run 3: 298 chaos events, 5 OOMs
```

3. **Autonomy Impact**:
```bash
# Compare autonomy effectiveness
stresstest compare memory --duration 10000

# Expected output showing:
# - Peak pressure reduction
# - Proactive compactions
# - OOM prevention
# - Early interventions
```

#### 4.2 Validation Script Updates
**File**: `scripts/validate_stress_results.py`

Add checks for:
- Minimum failure rate when `--failure-rate` set
- Autonomy impact on multiple metrics (not just OOM)
- Variance across runs

## Success Criteria

### Phase 1 Success
- [ ] Chaos test shows configurable failure rate (5-15%)
- [ ] Memory test can inject OOM failures
- [ ] Failed recoveries tracked in metrics
- [ ] Shell commands accept `--failure-rate` and `--oom-probability`

### Phase 2 Success
- [ ] Autonomy triggers compaction at 75% pressure
- [ ] Peak pressure reduced by ≥10% with autonomy ON
- [ ] Avg pressure reduced by ≥5% with autonomy ON
- [ ] Compare mode shows ≥3 impact metrics (peak, avg, compactions, OOM)
- [ ] Early intervention count > 0

### Phase 3 Success
- [ ] Chaos output paginated (no truncation mid-list)
- [ ] Report history supports `--all` flag
- [ ] Output clearly formatted with sections

### Phase 4 Success
- [ ] README includes 3+ enhanced examples
- [ ] Examples show expected variance
- [ ] Validation script checks new metrics

## Testing Plan

### Unit Tests
1. Test failure injection logic
2. Test PRNG distribution for failure rates
3. Test autonomy threshold triggers

### Integration Tests
1. Run chaos test with 10% failure rate → expect ~10% failures
2. Run memory compare → expect autonomy to reduce pressure
3. Run report --all → expect all history entries

### QEMU Validation
1. Verify failure rate within ±3% of target
2. Verify autonomy reduces peak by ≥10%
3. Verify output formatting is complete

## Implementation Timeline

**Phase 1**: 2-3 hours
- Add config parameters (30 min)
- Implement failure injection (1.5 hours)
- Update shell commands (1 hour)

**Phase 2**: 2-3 hours
- Proactive compaction logic (1.5 hours)
- Metrics tracking (1 hour)
- Compare mode enhancements (30 min)

**Phase 3**: 1 hour
- Output pagination (30 min)
- Report flags (30 min)

**Phase 4**: 1 hour
- README examples (45 min)
- Validation script (15 min)

**Total**: 6-8 hours

## Risks & Mitigations

### Risk 1: Failure injection too aggressive
**Mitigation**: Start with low rates (5%), make configurable

### Risk 2: Autonomy changes affect stability
**Mitigation**: Add flag to disable early intervention, test thoroughly

### Risk 3: Output changes break parsing
**Mitigation**: Keep machine-readable format available for CI/CD

## Rollout Plan

1. Implement Phase 1 + 2 on feature branch
2. Test in QEMU with multiple runs
3. Update README with examples
4. Commit and push
5. Validate in CI/CD pipeline
6. Merge to main

## Appendix A: Current Metrics Baseline

**Chaos Test** (no failure injection):
- Events: 340-383
- Success rate: 100%
- Failed recoveries: 0

**Memory Compare** (autonomy impact):
- Peak pressure: 99% (both on/off)
- Avg pressure: 85% (both on/off)
- OOM events: 6→4 (autonomy on)
- Compactions: No difference

**Target Metrics** (post-implementation):
- Success rate: 85-95% (with 10% failure rate)
- Peak pressure: 99%→87% (autonomy on)
- Avg pressure: 85%→77% (autonomy on)
- Proactive compactions: >10 (autonomy on)
