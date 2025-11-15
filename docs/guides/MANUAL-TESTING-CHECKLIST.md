# Phase 4 Manual Testing Checklist

This guide provides step-by-step manual testing procedures for Phase 4 solidification validation.

## Overview

Since the kernel uses an interactive shell, manual testing is currently the most reliable method for validation. Automated scripts are provided for long-duration runs, but interactive commands provide immediate feedback.

## Quick Start

```bash
# Start the kernel with AI features enabled
SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh build
```

Once the kernel boots, you'll see:
```
=== SIS Kernel Shell ===
Type 'help' for available commands

sis>
```

---

## Test Suite 1: AI Neural Network Verification

**Objective:** Verify AI neural network is active and making inferences.

### Commands to Run

```
sis> fullautodemo
```

### Expected Results

- [ ] Demo completes all 7 phases
- [ ] `METRIC nn_infer_count` increases (e.g., 1, 2, 3, 4...)
- [ ] Network packets sent (with/without AI comparison)
- [ ] Commands processed count shown
- [ ] OOM events: 0 (or very low)
- [ ] Demo status: [OK] marks shown

### Success Criteria

- Neural network inferences > 0
- Demo completes without crashes
- Performance comparison shown

### Sample Output Indicators

```
METRIC nn_infer_us=496
METRIC nn_infer_count=1
[AI] Predicting: likely success (confidence: 238/1000)
```

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 2: Memory Benchmark

**Objective:** Validate AI-driven memory management under stress.

### Commands to Run

```
sis> benchmark memory 10
```

**Arguments:**
- `10` = duration in seconds (default: 10)

### Expected Results

- [ ] Baseline (AI disabled) runs first
- [ ] AI-enabled test runs second
- [ ] Comparison report shown
- [ ] OOM reduction % calculated
- [ ] Memory pressure tracked

### Success Criteria

- OOM events: With AI <= Without AI
- OOM reduction: >= 0%
- No kernel panics

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 3: Command Benchmark

**Objective:** Test command execution prediction and optimization.

### Commands to Run

```
sis> benchmark commands 5
```

**Arguments:**
- `5` = duration in seconds (default: 5)

### Expected Results

- [ ] Commands processed count shown
- [ ] AI vs baseline comparison
- [ ] Latency reduction calculated

### Success Criteria

- Commands executed > 1000
- No crashes during execution

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 4: Network Benchmark

**Objective:** Validate AI network flow control and congestion prediction.

### Commands to Run

```
sis> benchmark network 10 1000
```

**Arguments:**
- `10` = duration in seconds
- `1000` = packet rate per second

### Expected Results

- [ ] Packets sent with/without AI
- [ ] Throughput comparison shown
- [ ] Network decisions made
- [ ] Latency improvements calculated

### Success Criteria

- Packets sent > 10,000
- Network subsystem stable
- AI making network decisions

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 5: Full Benchmark

**Objective:** Comprehensive multi-subsystem benchmark.

### Commands to Run

```
sis> benchmark full 15
```

**Arguments:**
- `15` = duration in seconds (default: 15)

### Expected Results

- [ ] All subsystems tested (memory, commands, network)
- [ ] Comprehensive comparison report
- [ ] OOM reduction calculated
- [ ] Latency reduction shown
- [ ] Throughput improvements displayed

### Success Criteria

- All subsystems show activity
- Performance metrics calculated
- System remains stable

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 6: Benchmark Report

**Objective:** Generate summary of all previous benchmarks.

### Commands to Run

```
sis> benchmark report
```

### Expected Results

- [ ] Last benchmark results displayed
- [ ] Comparative metrics shown
- [ ] Improvement percentages calculated

### Success Criteria

- Report displays valid data
- Comparisons are accurate

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 7: EU AI Act Compliance

**Objective:** Verify compliance with EU AI Act Articles 13-16.

### Commands to Run

```
sis> compliance eu-ai-act
```

### Expected Results

- [ ] Article 13 (Transparency): >= 80% compliance
- [ ] Article 14 (Human Oversight): >= 80% compliance
- [ ] Article 15 (Accuracy/Robustness): >= 80% compliance
- [ ] Article 16 (Recordkeeping): >= 80% compliance
- [ ] Overall Compliance: >= 85%
- [ ] Safety Score: >= 90/100

### Success Criteria

- Overall compliance >= 85%
- Safety score >= 90
- No critical violations

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 8: Compliance Audit

**Objective:** Generate audit trail and decision log.

### Commands to Run

```
sis> compliance audit
```

### Expected Results

- [ ] Total AI decisions logged
- [ ] Decision categories shown
- [ ] Timestamp tracking functional
- [ ] Audit trail complete

### Success Criteria

- AI decisions > 0
- Audit log non-empty
- Categories properly tracked

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 9: Transparency Report

**Objective:** Verify AI decision explainability.

### Commands to Run

```
sis> compliance transparency
```

### Expected Results

- [ ] Decision rationale available
- [ ] Explanations provided
- [ ] Human-readable output confirmed
- [ ] Metadata tracking functional

### Success Criteria

- All transparency checks pass
- Explanations are clear

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 10: Safety Checklist

**Objective:** Validate pre-deployment safety requirements.

### Commands to Run

```
sis> compliance checklist
```

### Expected Results

- [ ] 15 safety items checked
- [ ] Hard limits tested
- [ ] Watchdog functional
- [ ] Fallback mechanisms verified
- [ ] Validation frameworks active

### Success Criteria

- All 15 items pass
- Score: 15/15

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Test Suite 11: Incident Logging

**Objective:** Test AI incident detection and reporting.

### Commands to Run

```
sis> compliance incidents
```

### Expected Results

- [ ] Incident categories tracked
- [ ] Critical incident count shown
- [ ] Warning/info incidents logged
- [ ] Incident history maintained

### Success Criteria

- Incident tracking functional
- Critical incidents: 0 (or low)

**Status: PASS / FAIL / NEEDS_RETRY**

---

## Additional Diagnostic Commands

### Autonomous Control Status

```
sis> autoctl stats
```

**Shows:**
- Total AI decisions made
- Autonomous/manual decision breakdown
- Success rate
- Autonomy level

### Learning Statistics

```
sis> learnctl stats
```

**Shows:**
- Prediction accuracy
- Training updates
- Learning rates
- Model performance

### Agent Statistics

```
sis> agentctl stats
```

**Shows:**
- Memory agent metrics
- Command agent metrics
- Network agent metrics
- Cross-agent coordination

### System Health

```
sis> health
```

**Shows:**
- Overall system status
- Subsystem health checks
- Resource utilization

---

## Long-Duration Testing

For extended validation (1hr, 4hr, 24hr), use these procedures:

### 1-Hour Autonomous Test

```bash
# Start kernel
SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh build

# In QEMU shell, enable full autonomy
sis> autoctl enable 3
sis> benchmark full 3600
```

**Monitor for:**
- System stability (no crashes)
- AI decisions continuously made
- Memory stability (no leaks)
- Network stability

### 4-Hour Stability Test

Same as 1-hour, but run:
```
sis> benchmark full 14400
```

### Manual 24-Hour Test

Not recommended for interactive testing. Use automated scripts or hardware deployment for 24hr tests.

---

## Troubleshooting

### Issue: No Neural Network Inferences

**Symptoms:**
- `METRIC nn_infer_count` stays at 0
- No AI predictions shown

**Solutions:**
1. Verify `SIS_FEATURES="llm,crypto-real"` is set when starting
2. Check kernel build included LLM feature
3. Verify autonomy is enabled: `autoctl stats`
4. Try: `autoctl enable 2`

### Issue: High OOM Events

**Symptoms:**
- Benchmark report shows many OOM events
- System becomes unstable

**Solutions:**
1. This is expected under extreme stress
2. Check AI is enabled during test
3. Compare baseline vs AI OOM counts
4. AI should reduce OOMs, not eliminate them

### Issue: Commands Don't Work

**Symptoms:**
- "Unknown command" errors

**Solutions:**
1. Use `help` to list all commands
2. Check command syntax (e.g., `benchmark full` not `benchmark-full`)
3. Subcommands need parent: `compliance eu-ai-act` not `eu-ai-act`

### Issue: QEMU Hangs or Crashes

**Symptoms:**
- Shell stops responding
- Kernel panic messages

**Solutions:**
1. Exit QEMU: `Ctrl+a`, then `x`
2. Restart with `uefi_run.sh build`
3. Check for kernel bugs in recent changes
4. Review crash logs

---

## Weekly Testing Schedule

### Week 1: Initial Validation

**Day 1:**
- [ ] AI verification (fullautodemo)
- [ ] Memory benchmark (10s)
- [ ] Commands benchmark (5s)
- [ ] Network benchmark (10s)

**Day 2:**
- [ ] Full benchmark (15s)
- [ ] All 5 compliance commands
- [ ] Autonomous control validation

**Day 3:**
- [ ] Extended benchmarks (60s, 120s, 300s)
- [ ] Review metrics for consistency

**Day 4-5:**
- [ ] 1-hour autonomous test
- [ ] Memory leak monitoring

**Day 6-7:**
- [ ] 4-hour stability test
- [ ] Document any issues found

### Week 2: Consistency Validation

Repeat Week 1 tests daily to ensure consistent results.

### Week 3: Final Validation

- [ ] Complete all test suites
- [ ] Document results
- [ ] Verify all success criteria met
- [ ] Prepare for Phase 5

---

## Success Criteria Summary

For Phase 4 solidification to be complete:

### Critical Requirements

- [x] AI neural network active (nn_infer_count > 0)
- [ ] Zero kernel crashes in 1hr test
- [ ] Zero kernel crashes in 4hr test
- [ ] All 11 Week 12 commands functional
- [ ] EU AI Act compliance >= 85%
- [ ] Safety score >= 90/100

### Performance Requirements

- [ ] OOM reduction >= 0% (AI should not increase OOMs)
- [ ] Commands processed > 1000 in 5s test
- [ ] Network packets > 10,000 in 10s test
- [ ] No memory leaks (heap growth < 1MB/hour)

### Quality Requirements

- [ ] All tests pass consistently (3+ consecutive runs)
- [ ] Documentation complete
- [ ] Known issues documented
- [ ] Troubleshooting guide updated

---

## Recording Test Results

Use this template to record results:

```
Date: __________
Tester: __________
Kernel Version: __________
Environment: QEMU / Hardware

Test Results:
- fullautodemo: PASS / FAIL - Notes: __________
- benchmark memory: PASS / FAIL - OOM reduction: ____%
- benchmark commands: PASS / FAIL - Commands: _____
- benchmark network: PASS / FAIL - Packets: _____
- benchmark full: PASS / FAIL - Notes: __________
- benchmark report: PASS / FAIL - Notes: __________
- compliance eu-ai-act: PASS / FAIL - Score: ___%
- compliance audit: PASS / FAIL - Decisions: _____
- compliance transparency: PASS / FAIL - Notes: __________
- compliance checklist: PASS / FAIL - Score: __/15
- compliance incidents: PASS / FAIL - Critical: _____

Overall: PASS / FAIL
Issues Found: __________
```

---

## Next Steps

After completing all manual tests successfully:

1. **Document Results:** Create Week 1-7 results documents
2. **Hardware Testing:** Test on real ARM hardware (Raspberry Pi)
3. **Code Review:** Refactor based on findings
4. **Automated Tests:** Consider adding auto-run mode for CI/CD
5. **Phase 5 Planning:** Proceed to AI-Native Intelligence features

---

**Last Updated:** November 3, 2025
**Testing Version:** 1.0
**Project Phase:** Phase 4 Solidification
