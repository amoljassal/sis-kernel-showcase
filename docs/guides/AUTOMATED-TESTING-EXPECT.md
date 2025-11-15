# Automated Testing with Expect

This guide documents the expect-based automated testing infrastructure for Phase 4 solidification.

## Overview

The expect-based test scripts provide full automation of the QEMU interactive shell, enabling:
- Programmatic command execution
- Automated metric extraction
- Pass/fail validation
- Comprehensive result analysis

## Prerequisites

### Install Expect (macOS)

```bash
brew install expect
```

### Verify Installation

```bash
expect -v
# Should output: expect version 5.x
```

## Test Scripts

### 1. AI Verification Script

**Script:** `scripts/verify_ai_active_expect.sh`

**Purpose:** Automated AI neural network activity verification

**Usage:**
```bash
./scripts/verify_ai_active_expect.sh
```

**What it does:**
1. Starts QEMU with AI features enabled
2. Waits for shell prompt
3. Runs `fullautodemo` command
4. Collects `autoctl stats` and `agentctl stats`
5. Extracts neural network inference count
6. Validates AI activity
7. Exits QEMU cleanly

**Success Criteria:**
- Demo completes successfully
- Neural network inferences > 0
- Shell responds to commands

**Duration:** ~2-3 minutes

**Output:** `ai_verification_results/ai_verification_<timestamp>.log`

---

### 2. Benchmark Suite Script

**Script:** `scripts/benchmark_suite_expect.sh`

**Purpose:** Comprehensive automated benchmarking

**Usage:**
```bash
./scripts/benchmark_suite_expect.sh [duration]

# Examples:
./scripts/benchmark_suite_expect.sh          # 15s default
./scripts/benchmark_suite_expect.sh 30       # 30s benchmarks
./scripts/benchmark_suite_expect.sh 300      # 5min benchmarks
```

**What it does:**
1. Starts QEMU
2. Runs `benchmark memory 10`
3. Runs `benchmark commands 5`
4. Runs `benchmark network 10`
5. Runs `benchmark full <duration>`
6. Runs `benchmark report`
7. Extracts metrics and validates results

**Success Criteria:**
- All 5 benchmark commands complete
- Neural network inferences > 0
- Commands processed > 100
- Network packets > 10,000
- Zero system crashes

**Duration:** ~3-5 minutes (default), scales with benchmark duration

**Output:** `benchmark_results/benchmark_suite_<timestamp>.log`

---

### 3. Compliance Suite Script

**Script:** `scripts/compliance_suite_expect.sh`

**Purpose:** EU AI Act compliance validation

**Usage:**
```bash
./scripts/compliance_suite_expect.sh
```

**What it does:**
1. Starts QEMU
2. Runs `compliance eu-ai-act`
3. Runs `compliance audit`
4. Runs `compliance transparency`
5. Runs `compliance checklist`
6. Runs `compliance incidents`
7. Extracts compliance scores and validates

**Success Criteria:**
- All 5 compliance commands complete
- EU AI Act compliance >= 85%
- Safety score >= 90/100
- Checklist completion >= 90%
- Critical incidents = 0
- Production ready = YES

**Duration:** ~2 minutes

**Output:** `compliance_results/compliance_suite_<timestamp>.log`

---

### 4. Test Suite Orchestrator

**Script:** `scripts/run_phase4_tests_expect.sh`

**Purpose:** Orchestrates complete test suites with proper sequencing

**Usage:**
```bash
./scripts/run_phase4_tests_expect.sh [options] <suite>

# Quick validation
./scripts/run_phase4_tests_expect.sh quick

# Standard testing
./scripts/run_phase4_tests_expect.sh standard

# Compliance only
./scripts/run_phase4_tests_expect.sh compliance

# Full validation
./scripts/run_phase4_tests_expect.sh full

# Custom duration
./scripts/run_phase4_tests_expect.sh -d 60 full
```

**Test Suites:**

| Suite | Duration | Tests Included |
|-------|----------|----------------|
| quick | ~3 min | AI verification only |
| standard | ~8 min | AI + benchmarks |
| compliance | ~2 min | Compliance tests only |
| full | ~12 min | All tests |

**Full Suite Sequence:**
1. AI verification
2. Wait 30s
3. Benchmark suite
4. Wait 30s
5. Compliance suite

---

## Expected Output Format

### Successful AI Verification

```
[INFO] ==========================================
[INFO]   AI Neural Network Activity Verification
[INFO] ==========================================
[INFO] Output: ai_verification_results/ai_verification_20251103_235959.log

[INFO] Starting QEMU with expect automation...

[EXPECT] Shell prompt detected
[EXPECT] Running fullautodemo...
[EXPECT] Demo starting, sending keypress...
[EXPECT] Demo completed successfully
[EXPECT] Autonomy re-enabled
[EXPECT] Getting autonomy statistics...
[EXPECT] Getting agent statistics...
[EXPECT] Exiting QEMU...

[INFO] ==========================================
[INFO]   AI Verification Results
[INFO] ==========================================
Neural Network Inferences: 14
AI Decisions Made:         152
Autonomy Level:            2
Demo Completed:            1 times

[INFO] Validation Checks:

[PASS] Full autonomous demo completed successfully
[PASS] Neural network generating inferences (14 total)
[PASS] Shell interaction successful (8 prompts)

==========================================
Overall Result: 3 passed, 0 failed
[PASS] AI VERIFICATION SUCCESSFUL
==========================================
```

### Successful Benchmark Suite

```
[INFO] ==========================================
[INFO]   Comprehensive Benchmark Suite
[INFO] ==========================================

[EXPECT] ========== TEST 1: Memory Benchmark ==========
[EXPECT] Memory benchmark completed
[EXPECT] ========== TEST 2: Commands Benchmark ==========
[EXPECT] Commands benchmark completed
[EXPECT] ========== TEST 3: Network Benchmark ==========
[EXPECT] Network benchmark completed
[EXPECT] ========== TEST 4: Full Benchmark ==========
[EXPECT] Full benchmark completed
[EXPECT] ========== TEST 5: Benchmark Report ==========
[EXPECT] Report generated

[INFO] ==========================================
[INFO]   Benchmark Results Analysis
[INFO] ==========================================
Neural Network Inferences:    18
Commands Executed:            106501

Network Performance:
  Baseline packets:           1991743
  With AI packets:            2169245
  Improvement:                +8%

Memory Management:
  Baseline OOM events:        0
  With AI OOM events:         0

[INFO] Validation Checks:

[PASS] All benchmark tests completed (5/5)
[PASS] Neural network active (18 inferences)
[PASS] Commands processed (106501 commands)
[PASS] Network throughput good (2169245 packets)
[PASS] System stable (no crashes)

==========================================
Overall Result: 5 passed, 0 failed
[PASS] BENCHMARK SUITE SUCCESSFUL
==========================================

Key Achievements:
  - Neural network: 18 inferences
  - Commands: 106501 processed
  - Network: 2169245 packets (AI enabled)
  - Stability: Zero crashes
```

### Successful Compliance Suite

```
[INFO] ==========================================
[INFO]   Comprehensive Compliance Suite
[INFO] ==========================================

[EXPECT] ========== TEST 1: EU AI Act Compliance ==========
[EXPECT] EU AI Act report completed
[EXPECT] ========== TEST 2: Audit Package ==========
[EXPECT] Audit package generated
[EXPECT] ========== TEST 3: Transparency Report ==========
[EXPECT] Transparency report generated
[EXPECT] ========== TEST 4: Safety Checklist ==========
[EXPECT] Safety checklist completed
[EXPECT] ========== TEST 5: Incident Log ==========
[EXPECT] Incident log retrieved

[INFO] ==========================================
[INFO]   Compliance Results Analysis
[INFO] ==========================================
EU AI Act Compliance:         92%
Safety Score:                 100/100
Checklist Completion:         100%
Critical Incidents:           0
Production Ready:             YES

Article Compliance:
  Article 13 (Transparency):   PASS
  Article 14 (Human Oversight):PASS
  Article 15 (Accuracy/Robust):PASS
  Article 16 (Recordkeeping):  PASS

[INFO] Validation Checks:

[PASS] All compliance tests completed (5/5)
[PASS] EU AI Act compliance: 92% (>= 85% required)
[PASS] Safety score: 100/100 (>= 90 required)
[PASS] Safety checklist: 100% complete
[PASS] No critical incidents
[PASS] System marked production ready

==========================================
Overall Result: 6 passed, 0 failed
[PASS] COMPLIANCE SUITE SUCCESSFUL
==========================================

Key Achievements:
  - EU AI Act: 92% compliant
  - Safety: 100/100
  - Checklist: 100% complete
  - Critical incidents: 0
  - Production ready: YES
```

---

## Automated Metric Extraction

The expect scripts automatically extract and validate:

### AI Metrics
- `METRIC nn_infer_count=N` - Neural network inferences
- AI decisions count
- Autonomy level

### Benchmark Metrics
- Commands executed
- Network packets (baseline vs AI)
- OOM events (baseline vs AI)
- Performance improvements (%)

### Compliance Metrics
- Overall compliance score (%)
- Safety score (/100)
- Checklist completion (%)
- Critical incidents count
- Production readiness (YES/NO)

---

## Integration with CI/CD

These scripts can be integrated into CI/CD pipelines:

```bash
#!/bin/bash
# CI/CD integration example

set -e

# Run quick validation
./scripts/run_phase4_tests_expect.sh quick || {
    echo "Quick validation failed"
    exit 1
}

# Run full suite for main branch
if [ "$BRANCH" = "main" ]; then
    ./scripts/run_phase4_tests_expect.sh full || {
        echo "Full validation failed"
        exit 1
    }
fi

echo "All tests passed"
```

---

## Troubleshooting

### Issue: expect not found

**Error:**
```
expect: command not found
```

**Solution:**
```bash
brew install expect
```

### Issue: Timeout waiting for shell

**Error:**
```
[EXPECT] Timeout waiting for shell
```

**Solutions:**
1. Check QEMU starts correctly: `SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh build`
2. Increase timeout in expect script (default: 120s)
3. Verify kernel doesn't panic on boot

### Issue: Commands fail silently

**Error:**
No errors, but metrics show 0

**Solutions:**
1. Check that commands exist: run manual test in QEMU
2. Verify correct command syntax (e.g., `benchmark full` not `benchmark-full`)
3. Review full log file for actual errors

### Issue: QEMU doesn't exit cleanly

**Error:**
Scripts hang at end

**Solutions:**
1. Manually kill QEMU processes: `killall qemu-system-aarch64`
2. Check exit sequence: `Ctrl+a`, then `x`
3. Review expect script for proper exit handling

---

## Performance Comparison

| Method | Pros | Cons | Use Case |
|--------|------|------|----------|
| Manual Testing | Visual feedback, Interactive debugging | Time-consuming, Not reproducible | Development, Initial validation |
| Expect Scripts | Fully automated, Reproducible, CI/CD ready | Requires expect, Less visual | Regression testing, CI/CD |

---

## Extended Testing

For longer duration tests, modify the duration parameter:

```bash
# 5-minute benchmarks
./scripts/benchmark_suite_expect.sh 300

# 15-minute benchmarks
./scripts/benchmark_suite_expect.sh 900

# 1-hour benchmarks
./scripts/benchmark_suite_expect.sh 3600
```

**Note:** Longer durations require proportionally longer timeouts in expect scripts.

---

## Result Storage

All test results are stored with timestamps:

```
ai_verification_results/
  ai_verification_20251103_235959.log

benchmark_results/
  benchmark_suite_20251103_235959.log

compliance_results/
  compliance_suite_20251103_235959.log
```

Results can be archived for trend analysis:

```bash
# Archive all results
tar -czf phase4_results_$(date +%Y%m%d).tar.gz \
    ai_verification_results/ \
    benchmark_results/ \
    compliance_results/
```

---

## Next Steps

After automated testing is successful:

1. **Continuous Integration:** Add expect scripts to CI/CD pipeline
2. **Regression Testing:** Run automated tests on every commit
3. **Performance Tracking:** Monitor metrics over time
4. **Hardware Validation:** Port expect scripts for real hardware testing
5. **Extended Testing:** Run 1hr, 4hr, 24hr tests with modified durations

---

## References

- [Manual Testing Checklist](MANUAL-TESTING-CHECKLIST.md)
- [Phase 4 Testing Guide](PHASE4-TESTING-GUIDE.md)
- [Phase 4 Solidification Plan](../plans/PHASE4-SOLIDIFICATION-PLAN.md)
- [Expect Documentation](https://linux.die.net/man/1/expect)

---

**Last Updated:** November 3, 2025
**Testing Infrastructure Version:** 2.0 (Expect-based)
**Project Phase:** Phase 4 Solidification
