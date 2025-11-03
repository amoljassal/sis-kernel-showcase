# Week 8: Predictive Memory Management - Test Guide

## Overview

This guide provides step-by-step testing procedures for Week 8's AI-driven memory management features.

## Quick Start

```bash
# Build and boot
SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh

# In the SIS shell, run:
sis> memctl strategy test
sis> memctl predict compaction
sis> memctl learn stats
```

## Test Suite

### Test 1: Strategy Selection Logic

**Purpose**: Verify strategy selection based on meta-agent memory directive

**Commands**:
```bash
sis> memctl strategy test
```

**Expected Output**:
```
[PRED_MEM] Testing strategy selection:
  Directive -800 -> Conservative
  Directive -400 -> Conservative
  Directive 0 -> Balanced
  Directive 400 -> Balanced
  Directive 800 -> Aggressive
```

**Pass Criteria**:
- ✅ Negative directives < -500 → Conservative
- ✅ Directives -500..500 → Balanced
- ✅ Positive directives > 500 → Aggressive
- ✅ No crashes or errors

---

### Test 2: Compaction Prediction (5-Second Lookahead)

**Purpose**: Verify fragmentation prediction and compaction policy

**Commands**:
```bash
sis> memctl predict compaction
```

**Expected Output**:
```
[PRED_MEM] Compaction Decision Preview:
  Predicted fragmentation (5s ahead): 100%
  Confidence: 800/1000
  Decision: COMPACT (threshold exceeded)
```

**Pass Criteria**:
- ✅ Predicted fragmentation calculated
- ✅ Confidence score (0-1000) displayed
- ✅ Decision (COMPACT/SKIP) based on policy:
  - COMPACT if: confidence ≥ 700 AND predicted_frag ≥ 60
  - SKIP otherwise

---

### Test 3: Learning Statistics (Initial State)

**Purpose**: Verify statistics tracking infrastructure

**Commands**:
```bash
sis> memctl learn stats
```

**Expected Output**:
```
=== Predictive Memory Statistics ===
Current Strategy: Balanced

Compaction:
  Total predictions: 0
  Compactions triggered: 0
  OOMs prevented: 0

Allocation Prediction:
  Command types tracked: 0
  Pre-reservations: 0
  Pre-reserve hits: 0

Strategy Changes:
  Total changes: 0
```

**Pass Criteria**:
- ✅ Statistics structure displays correctly
- ✅ Initial values at 0 (no autonomous decisions yet)
- ✅ Current strategy shows "Balanced"

---

### Test 4: Memory Stress Testing

**Purpose**: Build allocation history and trigger memory agent warnings

**Commands**:
```bash
sis> memctl stress 100
```

**Expected Output**:
```
[MEM] Running allocation stress test: 100 iterations
[MEM] Iteration 0...
[MEM] Iteration 20...

[MEMORY AGENT] AUTONOMOUS WARNING: COMPACTION RECOMMENDED (conf=984/1000)
  Fragmentation: 32%

[MEM] Iteration 40...
[MEMORY AGENT] AUTONOMOUS WARNING: COMPACTION RECOMMENDED (conf=984/1000)
  Fragmentation: 29%

[MEM] Stress test complete
[HEAP] Stats: allocs=239 deallocs=239 current=0 bytes peak=3 KiB failures=0
```

**Pass Criteria**:
- ✅ 100 iterations complete
- ✅ Memory agent warnings fire when fragmentation > threshold
- ✅ High confidence scores (>950/1000)
- ✅ No allocation failures
- ✅ All allocations deallocated (current=0)

---

### Test 5: Autonomous Mode Integration

**Purpose**: Verify predictive memory integration with autonomy decision loop

**Commands**:
```bash
# Enable autonomous mode
sis> autoctl on
sis> autoctl interval 1000

# Wait 30 seconds for autonomous decisions
# (In practice: watch the shell, timer interrupts will trigger decisions)

# After waiting, check statistics
sis> memctl learn stats
```

**Expected Output (after 30 seconds)**:
```
=== Predictive Memory Statistics ===
Current Strategy: Balanced  (or Conservative/Aggressive if pressure changed)

Compaction:
  Total predictions: 30-50
  Compactions triggered: 0-5
  OOMs prevented: 0

Allocation Prediction:
  Command types tracked: 0  (no commands executed during autonomous mode)
  Pre-reservations: 0
  Pre-reserve hits: 0

Strategy Changes:
  Total changes: 0-3  (depends on memory pressure variation)
```

**Pass Criteria**:
- ✅ Compaction predictions > 0 (shows autonomous integration working)
- ✅ No crashes during autonomous operation
- ✅ Statistics accumulate over time
- ✅ Strategy may change based on memory directive

**Troubleshooting**:
- If predictions remain at 0: Check that `autoctl on` succeeded
- If timer not firing: Check GIC/timer initialization (should show in boot log)
- If crashes: Check `autoctl limits` for safety violations

---

### Test 6: Strategy Status with Memory State

**Purpose**: Verify strategy display with current memory telemetry

**Commands**:
```bash
sis> memctl strategy status
```

**Expected Output**:
```
[PRED_MEM] Current Allocation Strategy: Balanced
  Memory pressure: 0%
  Fragmentation: 80%
```

**Pass Criteria**:
- ✅ Shows current strategy
- ✅ Displays memory pressure (0-100%)
- ✅ Displays fragmentation (0-100%)
- ✅ Values match `memctl status` output

---

### Test 7: Full Integration Test

**Purpose**: End-to-end validation of all Week 8 features

**Script**: Run `scripts/test_week8_autonomous.sh` (documentation/reference)

**Manual Steps**:
1. Boot kernel
2. Run basic tests (strategy test, predict compaction, learn stats)
3. Run stress test to build history
4. Enable autonomous mode
5. Wait 30+ seconds
6. Check learning statistics
7. Check autonomous audit log
8. Disable autonomous mode

**Pass Criteria**:
- ✅ All individual tests pass
- ✅ No crashes or panics
- ✅ Autonomous decisions accumulate predictive memory stats
- ✅ System remains stable under autonomous operation

---

## Advanced Testing

### Test 8: Repeated Stress Cycles

**Purpose**: Build longer allocation history for prediction accuracy

**Commands**:
```bash
sis> memctl stress 50
sis> memctl learn stats
sis> memctl stress 50
sis> memctl learn stats
sis> memctl stress 50
sis> memctl learn stats
```

**Expected**: Each iteration may show slight changes in stats if command hashing is working

---

### Test 9: Compaction Policy Thresholds

**Purpose**: Verify policy triggers at correct thresholds

**Observations**:
- Confidence threshold: 700/1000 (70%)
- Fragmentation threshold: 60%
- Decision should be SKIP if either threshold not met

**Test Cases**:
1. High confidence, low frag → SKIP
2. Low confidence, high frag → SKIP
3. High confidence, high frag → COMPACT

---

### Test 10: Autonomous Audit Log

**Purpose**: Verify predictive memory calls are logged in autonomous decisions

**Commands**:
```bash
sis> autoctl on
sis> autoctl interval 1000
# Wait 10 seconds
sis> autoctl audit last 10
sis> autoctl off
```

**Expected**: Each decision record should show meta-agent directives being used

---

## Performance Validation

### Metrics to Check

1. **Neural Inference Latency**:
   - Memory predictions: <500μs (from METRIC nn_infer_us)
   - Meta-agent inference: <1000μs

2. **Prediction Overhead**:
   - Fragmentation prediction: <50μs
   - Strategy selection: <10μs

3. **Memory Footprint**:
   - PredictiveMemoryState: ~12KB (ring buffers + predictors)
   - Per-decision overhead: ~100 bytes (compaction decision record)

### Validation Commands

```bash
sis> metrics
# Check neural inference metrics
# Look for nn_infer_us and nn_infer_count

sis> memctl status
# Shows memory agent neural predictions
# Compare confidence scores across multiple runs
```

---

## Known Behaviors

### Statistics Start at Zero

**Why**: Predictive memory functions are called from the autonomous decision loop, which only runs when:
1. `autoctl on` is executed
2. Timer interrupts fire (every 1000ms by default)
3. Confidence threshold is met (60%)

**To populate stats**: Enable autonomy and wait for timer ticks

### Strategy Rarely Changes

**Why**: Strategy changes only when meta-agent memory directive crosses thresholds (-500, +500)

**To trigger changes**:
- Run heavy memory stress
- Check with `autoctl state` to see memory_directive value

### Command Type Tracking

**Why zero**: Allocation tracking requires `record_allocation()` to be called during command execution

**Future**: Week 9 will integrate command-level allocation tracking

---

## Troubleshooting

### Issue: `memctl strategy test` shows large numbers for negatives

**Fixed**: Added `print_number_signed()` function in Week 8 final commit

---

### Issue: Learning stats remain at 0 after autonomy enabled

**Check**:
1. `autoctl on` succeeded? (should show "Autonomous mode ENABLED")
2. Timer interrupts firing? (boot log should show GIC/timer init)
3. Waited long enough? (30 seconds = ~30 decisions at 1000ms interval)

**Debug**:
```bash
sis> autoctl status
# Shows: enabled, interval, last decision timestamp

sis> autoctl audit last 5
# Shows recent autonomous decisions
```

---

### Issue: Compaction always shows COMPACT decision

**Expected**: Initial boot state may show high predicted fragmentation

**After stress testing**: Fragmentation values should vary based on allocation patterns

---

## Success Criteria Summary

Week 8 is **PASSING** if:

- ✅ Strategy selection logic works (Conservative/Balanced/Aggressive)
- ✅ Compaction prediction shows 5-second lookahead
- ✅ Learning stats display correctly (even if at 0 initially)
- ✅ Stress testing completes without failures
- ✅ Autonomous integration doesn't crash
- ✅ All shell commands respond correctly
- ✅ No compilation warnings or errors

**All core infrastructure is in place and validated!**

---

## Next Steps

**Week 9**: AI-driven scheduling with learned operator prioritization
- Will integrate with existing scheduling subsystem
- Will track per-operator performance history
- Will use similar ring buffer + prediction architecture

**Week 10**: Command execution prediction and resource pre-allocation
- Will add `record_allocation()` calls to shell command execution
- Will populate command type tracking
- Will demonstrate pre-reservation feature
