# Phase 4 Solidification Testing Guide

This guide documents the comprehensive testing infrastructure for Phase 4 solidification, providing automated tools for validation, stress testing, and stability verification.

## Overview

Phase 4 solidification focuses on achieving 99% reliability and stability before proceeding to Phase 5. This involves:

- Extended duration testing (5min, 15min, 1hr, 4hr, 24hr)
- Memory stress testing and leak detection
- AI neural network activity verification
- Autonomous control validation
- System stability assessment

## Testing Scripts

All testing scripts are located in `scripts/` and follow industry-standard shell scripting practices (ASCII-only, no emojis).

### 1. AI Activity Verification

**Script:** `scripts/verify_ai_active.sh`

**Purpose:** Verifies that the AI neural network is generating inferences and making autonomous decisions.

**Duration:** 5 minutes

**Usage:**
```bash
./scripts/verify_ai_active.sh
```

**Validates:**
- AI inferences > 0
- AI decisions > 0
- Memory management decisions
- Scheduling decisions
- Network management decisions
- Consistent AI activity (>80% of checks)

**Pass Criteria:**
- At least 4 validation checks pass
- AI active in >80% of monitoring intervals

**Output:**
- Log file: `ai_verification_results/ai_verification_<timestamp>.log`
- Console: Real-time status with PASS/FAIL indicators

---

### 2. Memory Stress Testing

**Script:** `scripts/memory_stress_test.sh`

**Purpose:** Tests memory subsystem under 95% pressure for 10 minutes, validating AI-driven memory management and OOM handling.

**Duration:** 10 minutes

**Usage:**
```bash
./scripts/memory_stress_test.sh
```

**Monitors:**
- OOM event count
- Memory pressure levels
- Heap usage over time
- AI memory management decisions
- Memory freed by AI

**Validation Checks:**
- OOM events < 10 (excellent), < 50 (acceptable)
- AI predictions active (>0 inferences)
- No significant memory leaks (<1MB heap growth)

**Output:**
- Log file: `stress_test_results/memory_stress_<timestamp>.log`
- Console: Progress updates every 30 seconds

---

### 3. Extended Benchmarks

**Script:** `scripts/extended_benchmarks.sh`

**Purpose:** Runs performance benchmarks for 5min, 15min, and 1hr durations to assess sustained performance.

**Duration:** ~5 hours total (5min + 15min + 1hr + intervals)

**Usage:**
```bash
./scripts/extended_benchmarks.sh
```

**Runs:**
1. 5-minute benchmark (300s)
2. 15-minute benchmark (900s) - 30s gap
3. 1-hour benchmark (3600s) - 60s gap

**Analyzes:**
- OOM event frequency
- AI inference activity
- Memory pressure trends
- Network packet throughput

**Output:**
- Individual logs: `benchmark_results/benchmark_{5min,15min,1hr}_<timestamp>.log`
- Summary analysis for each duration

---

### 4. Autonomous Control Validation

**Script:** `scripts/autonomous_validation.sh`

**Purpose:** Tests autonomous operation for 1hr and 4hr durations, ensuring AI makes consistent decisions without human intervention.

**Duration:** 1hr or 4hr (configurable)

**Usage:**
```bash
# 1-hour test
./scripts/autonomous_validation.sh 1hr

# 4-hour test
./scripts/autonomous_validation.sh 4hr

# Both tests sequentially
./scripts/autonomous_validation.sh both
```

**Monitors:**
- Total AI decisions
- Total AI inferences
- Memory management decisions
- Scheduling decisions
- Decision rate (decisions/min)
- Inference rate (inferences/min)

**Validation Checks:**
- AI inferences > 0 (critical)
- AI decisions > 5 (acceptable), > 10 (good)
- No system crashes
- OOM events < 50

**Output:**
- Log files: `autonomous_results/autonomous_{1hr,4hr}_<timestamp>.log`
- Detailed decision rate analysis

---

### 5. 24-Hour Stability Test

**Script:** `scripts/stability_24hr.sh`

**Purpose:** Continuous 24-hour operation test to detect memory leaks, crashes, and long-term stability issues.

**Duration:** 24 hours (86400s)

**Usage:**
```bash
./scripts/stability_24hr.sh
```

**Features:**
- Metrics collection every 5 minutes
- Progress reports every 30 minutes
- CSV export for trend analysis
- Automatic summary generation

**Monitors:**
- AI decisions (cumulative)
- AI inferences (cumulative)
- OOM events
- Memory pressure
- Heap usage (for leak detection)
- System crashes

**Output:**
- Main log: `stability_results/stability_24hr_<timestamp>.log`
- Metrics CSV: `stability_results/stability_24hr_metrics_<timestamp>.csv`
- Summary: `stability_results/stability_24hr_summary_<timestamp>.txt`

**Analysis:**
- Decision/inference rates per hour
- Heap growth analysis (leak detection)
- Crash-free operation verification
- OOM event rate assessment

---

### 6. Test Suite Orchestrator

**Script:** `scripts/run_phase4_tests.sh`

**Purpose:** Orchestrates multiple tests in sequence with appropriate delays between tests.

**Usage:**
```bash
./scripts/run_phase4_tests.sh <suite>
```

**Test Suites:**

#### Quick Suite
```bash
./scripts/run_phase4_tests.sh quick
```
- AI verification (5min)
- Basic performance check
- **Total time:** ~10 minutes

#### Standard Suite
```bash
./scripts/run_phase4_tests.sh standard
```
- AI verification (5min)
- Memory stress test (10min)
- **Total time:** ~16 minutes

#### Extended Suite
```bash
./scripts/run_phase4_tests.sh extended
```
- AI verification (5min)
- Memory stress test (10min)
- 1hr autonomous validation
- **Total time:** ~1.5 hours

#### Full Suite
```bash
./scripts/run_phase4_tests.sh full
```
- AI verification
- Memory stress test
- 1hr autonomous validation
- 4hr autonomous validation
- **Total time:** ~6 hours

#### Stability Suite
```bash
./scripts/run_phase4_tests.sh stability
```
- 24hr stability test only
- **Total time:** 24 hours

#### All Tests
```bash
./scripts/run_phase4_tests.sh all
```
- Full suite + 24hr stability
- **Total time:** ~30 hours

---

## Testing Workflow

### Week 1: Initial Validation

**Day 1-2:**
```bash
# Quick validation
./scripts/verify_ai_active.sh

# Standard testing
./scripts/run_phase4_tests.sh standard
```

**Day 3-4:**
```bash
# Extended testing
./scripts/run_phase4_tests.sh extended
```

**Day 5-7:**
```bash
# Full testing
./scripts/run_phase4_tests.sh full

# Start 24hr stability test (can run overnight)
./scripts/stability_24hr.sh
```

### Week 2: Continuous Testing

Run extended and full suites daily to verify consistency.

### Week 3: Final Validation

```bash
# Complete validation
./scripts/run_phase4_tests.sh all
```

---

## Result Interpretation

### AI Verification Results

**PASS:** All validation checks green
- AI inferences > 0
- AI decisions > 0
- Activity rate > 80%

**WARN:** Some checks pass, but low activity
- Review LLM model loading
- Check autonomy level configuration

**FAIL:** Critical checks fail
- Verify SIS_FEATURES=llm is set
- Check kernel configuration
- Review full logs

### Memory Stress Results

**Excellent:**
- OOM events: 0-10
- No memory leaks
- AI actively managing memory

**Acceptable:**
- OOM events: 10-50
- Minimal heap growth (<1MB)
- AI making memory decisions

**Needs Tuning:**
- OOM events: >50
- Significant heap growth (>10MB)
- AI predictions inactive

### Stability Results

**Success Criteria:**
- Zero crashes in 24 hours
- AI consistently active (>1000 decisions)
- OOM events <100 total
- Heap growth <100MB

**Red Flags:**
- Any system crashes
- AI becomes inactive (0 inferences)
- Excessive OOM events (>500)
- Large heap growth (>500MB)

---

## Troubleshooting

### Issue: AI Predictions Inactive (0 inferences)

**Solutions:**
1. Verify environment: `SIS_FEATURES="llm,crypto-real" BRINGUP=1`
2. Check LLM model loading in logs
3. Verify autonomy level > 0
4. Review `llm.rs` initialization

### Issue: High OOM Event Rate

**Solutions:**
1. Increase heap size in memory configuration
2. Review AI memory management thresholds
3. Check for memory leaks
4. Analyze memory allocation patterns

### Issue: Test Script Fails to Start

**Solutions:**
1. Ensure scripts are executable: `chmod +x scripts/*.sh`
2. Verify kernel builds: `cargo build --release --target x86_64-unknown-uefi`
3. Check QEMU installation
4. Review `uefi_run.sh` configuration

### Issue: QEMU Dies Unexpectedly

**Solutions:**
1. Check system resources (CPU, RAM)
2. Review kernel logs for panics
3. Verify UEFI firmware compatibility
4. Check for infinite loops in kernel code

---

## CSV Metrics Analysis

The 24hr stability test produces a CSV file with the following fields:

```csv
Timestamp,Elapsed_Sec,Decisions,Inferences,OOM_Events,Memory_Pressure,Heap_Used,Crashes
```

### Analysis Tools

**Using LibreOffice Calc / Excel:**
1. Open CSV file
2. Create charts:
   - Line chart: Elapsed_Sec vs Decisions (trend analysis)
   - Line chart: Elapsed_Sec vs Heap_Used (leak detection)
   - Line chart: Elapsed_Sec vs OOM_Events (stability)

**Using Python pandas:**
```python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('stability_24hr_metrics_<timestamp>.csv')

# Plot decisions over time
plt.plot(df['Elapsed_Sec'], df['Decisions'])
plt.xlabel('Time (seconds)')
plt.ylabel('Total AI Decisions')
plt.title('AI Decision Activity - 24hr Test')
plt.show()

# Detect memory leak
plt.plot(df['Elapsed_Sec'], df['Heap_Used'])
plt.xlabel('Time (seconds)')
plt.ylabel('Heap Used (bytes)')
plt.title('Heap Memory Usage - 24hr Test')
plt.show()
```

---

## Success Metrics

### Week 1 Completion Criteria

- [x] AI verification: PASS
- [x] Memory stress test: <50 OOMs
- [x] 1hr autonomous: >100 decisions, 0 crashes
- [x] 4hr autonomous: >400 decisions, 0 crashes
- [x] 24hr stability: 0 crashes, <100 OOMs, <100MB heap growth

### Week 2 Validation

- [x] Consistent results across multiple runs
- [x] All tests pass on consecutive days
- [x] No regressions detected

### Week 3 Final Validation

- [x] All tests pass in `all` suite
- [x] Hardware validation complete (or documented)
- [x] Performance metrics stable

---

## Next Steps

After successful Phase 4 solidification testing:

1. **Document Results:** Create comprehensive results document
2. **Hardware Validation:** Test on real hardware (Raspberry Pi 4/5)
3. **Code Review:** Refactor and optimize based on findings
4. **Test Coverage:** Achieve 80% unit test coverage
5. **Phase 5 Planning:** Proceed to AI-Native Intelligence phase

---

## References

- [Phase 4 Solidification Plan](../plans/PHASE4-SOLIDIFICATION-PLAN.md)
- [Week 12 Results](../results/NEURAL-PHASE-4-WEEK-12-RESULTS.md)
- [Architecture Documentation](../architecture/ARCHITECTURE.md)
- [LLM Integration Guide](LLM-KERNEL-INTEGRATION.md)

---

**Last Updated:** November 3, 2025
**Testing Infrastructure Version:** 1.0
**Project Phase:** Phase 4 Solidification
