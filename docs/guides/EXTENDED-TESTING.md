# Extended Duration Testing Guide

This guide documents the extended duration testing infrastructure for Phase 4 Week 1 performance and stability validation.

## Overview

Extended duration tests validate system stability, autonomous control, and performance characteristics over longer time periods (5 minutes to 24 hours).

## Prerequisites

```bash
brew install expect
```

## Quick Start

Master runner script with pre-configured test profiles:

```bash
./scripts/run_extended_tests.sh <test-type>
```

## Test Types

### Benchmark Tests

Extended duration benchmarks for performance characterization:

```bash
# 5-minute benchmarks (~8 min total)
./scripts/run_extended_tests.sh benchmark-5min

# 15-minute benchmarks (~18 min total)
./scripts/run_extended_tests.sh benchmark-15min

# 1-hour benchmarks (~65 min total)
./scripts/run_extended_tests.sh benchmark-1hr
```

**What Gets Tested:**
- Memory subsystem performance
- Command processing throughput
- Network packet processing
- Multi-subsystem integration
- Neural network activity

**Expected Results:**
- Commands processed: 50K+ (5min), 150K+ (15min), 600K+ (1hr)
- Network packets: 1M+ (5min), 3M+ (15min), 12M+ (1hr)
- Neural inferences: 5+ throughout
- System stability: 0 crashes

### Memory Stress Tests

High-pressure memory testing to validate stability under extreme load:

```bash
# 10-minute test at 95% pressure
./scripts/run_extended_tests.sh memory-stress

# 30-minute test at 95% pressure
./scripts/run_extended_tests.sh memory-stress-30min

# Direct script invocation (custom duration and pressure)
./scripts/memory_stress_expect.sh <duration_ms> <pressure_pct>

# Examples:
./scripts/memory_stress_expect.sh 600000 95    # 10min at 95%
./scripts/memory_stress_expect.sh 1800000 98   # 30min at 98%
```

**What Gets Tested:**
- Memory allocator under sustained pressure
- Out-of-memory (OOM) handling
- Memory compaction triggers
- System stability at high memory utilization

**Expected Results:**
- Peak pressure: 90-100% (target-dependent)
- OOM events: <10 (acceptable at 95%+ pressure)
- Compaction triggers: Varies by duration
- System stability: 0 crashes

**Validation Criteria:**
- Test completes successfully
- Achieves target pressure (within 10%)
- OOM events within acceptable range
- Test duration within 20% of target
- No system crashes

### Autonomous Control Validation

Long-duration autonomous AI operation testing:

```bash
# 1-hour autonomous validation (~65 min)
./scripts/run_extended_tests.sh autonomous-1hr

# 4-hour autonomous validation (~4.1 hours)
./scripts/run_extended_tests.sh autonomous-4hr

# 24-hour autonomous validation (~24.5 hours)
./scripts/run_extended_tests.sh autonomous-24hr

# Direct script invocation (custom duration)
./scripts/autonomous_validation_expect.sh <duration_seconds>

# Examples:
./scripts/autonomous_validation_expect.sh 3600    # 1 hour
./scripts/autonomous_validation_expect.sh 14400   # 4 hours
./scripts/autonomous_validation_expect.sh 86400   # 24 hours
```

**What Gets Tested:**
- Autonomous mode stability over extended periods
- AI decision-making frequency and consistency
- Neural network activity throughout test
- Watchdog trigger frequency
- Multi-subsystem stress under autonomous control

**Expected Results:**
- Total decisions: 100+ (1hr), 400+ (4hr), 2400+ (24hr)
- AI decisions: Proportional to total
- Neural inferences: >0 throughout
- Watchdog triggers: <1% of decisions
- System stability: 0 crashes

**Validation Criteria:**
- Test completes successfully
- Autonomous mode remains enabled
- Sufficient decision-making activity (1 per 36s minimum)
- Neural network remains active
- Watchdog triggers within acceptable range
- No system crashes

### 24-Hour Stability Test

Comprehensive stability validation combining multiple test types:

```bash
./scripts/run_extended_tests.sh stability-24hr
```

**Test Sequence:**
1. 24-hour autonomous validation
2. 1-hour wait
3. 1-hour memory stress test (95% pressure)
4. 1-hour wait
5. 1-hour benchmark suite

**Total Duration:** ~27 hours

**What Gets Validated:**
- Long-term autonomous operation stability
- Memory subsystem resilience
- Performance consistency
- Recovery from stress conditions
- Overall system stability

## Result Logs

All test results are stored with timestamps:

```
stress_test_results/
  memory_stress_<timestamp>.log

autonomous_validation_results/
  autonomous_validation_<timestamp>.log

benchmark_results/
  benchmark_suite_<timestamp>.log
```

## Test Duration Planning

| Test Type | Duration | Total Time | Use Case |
|-----------|----------|------------|----------|
| benchmark-5min | 5 min | ~8 min | Quick performance baseline |
| benchmark-15min | 15 min | ~18 min | Standard performance validation |
| benchmark-1hr | 1 hour | ~65 min | Extended performance characterization |
| memory-stress | 10 min | ~15 min | Memory subsystem validation |
| memory-stress-30min | 30 min | ~35 min | Extended memory validation |
| autonomous-1hr | 1 hour | ~65 min | Autonomous control validation |
| autonomous-4hr | 4 hours | ~4.1 hours | Long-term autonomous validation |
| autonomous-24hr | 24 hours | ~24.5 hours | Extended autonomous validation |
| stability-24hr | 27 hours | ~27 hours | Comprehensive stability test |

## Expected Metrics

### Benchmark Tests

```
Neural Network Inferences:    5-20 (varies by duration)
Commands Executed:            50K+ (5min), 600K+ (1hr)
Network Packets:              1M+ (5min), 12M+ (1hr)
System Crashes:               0
```

### Memory Stress Tests

```
Peak Memory Pressure:         90-100% (target-dependent)
OOM Events:                   0-10 (acceptable at high pressure)
Compaction Triggers:          Varies by duration
System Crashes:               0
```

### Autonomous Validation

```
Autonomous Mode:              ENABLED throughout
Total Decisions:              100+ (1hr), 2400+ (24hr)
Neural Inferences:            >0 throughout
Watchdog Triggers:            <1% of decisions
System Crashes:               0
```

## Troubleshooting

### Issue: Test times out

**Solutions:**
1. Check QEMU starts correctly
2. Verify kernel doesn't panic on boot
3. Check system resources (disk space, memory)
4. Review test log for actual errors

### Issue: Low decision count in autonomous tests

**Solutions:**
1. Verify autonomous mode is enabled: `autoctl status`
2. Check timer configuration
3. Review test log for autonomous control messages
4. Ensure neural network is active

### Issue: High OOM events in memory stress

**Solutions:**
1. This is expected at 95%+ pressure
2. Check OOM count is <10 for 10-minute test
3. If >10 OOMs, consider reducing pressure to 90%
4. Review heap allocation patterns

### Issue: System crashes detected

**Solutions:**
1. Review full test log for crash messages
2. Check kernel panic messages
3. Verify build configuration
4. Report issue with full log

## CI/CD Integration

Extended tests can be integrated into CI/CD for nightly builds:

```bash
#!/bin/bash
# Nightly extended validation

# Quick validation first
./scripts/run_phase4_tests_expect.sh quick || exit 1

# Extended benchmarks
./scripts/run_extended_tests.sh benchmark-15min || exit 1

# Memory stress
./scripts/run_extended_tests.sh memory-stress || exit 1

# Autonomous validation (1hr on weekdays, 4hr on weekends)
if [ "$(date +%u)" -lt 6 ]; then
    ./scripts/run_extended_tests.sh autonomous-1hr || exit 1
else
    ./scripts/run_extended_tests.sh autonomous-4hr || exit 1
fi

echo "Nightly validation complete"
```

## Performance Tracking

Archive results for trend analysis:

```bash
# Archive all extended test results
tar -czf extended_results_$(date +%Y%m%d).tar.gz \
    stress_test_results/ \
    autonomous_validation_results/ \
    benchmark_results/

# Upload to artifact storage
# (example: S3, GCS, artifact server, etc.)
```

## Next Steps

After successful extended testing:

1. **Analysis**: Review metrics for performance trends
2. **Optimization**: Identify and optimize bottlenecks
3. **Documentation**: Document performance characteristics
4. **Hardware**: Validate on real hardware
5. **Production**: Deploy with confidence

## References

- [Automated Testing Guide](AUTOMATED-TESTING-EXPECT.md)
- [Manual Testing Checklist](MANUAL-TESTING-CHECKLIST.md)
- [Phase 4 Testing Guide](PHASE4-TESTING-GUIDE.md)
- [Phase 4 Solidification Plan](../plans/PHASE4-SOLIDIFICATION-PLAN.md)

---

**Last Updated:** November 4, 2025
**Testing Infrastructure Version:** 2.1 (Extended Duration Support)
**Project Phase:** Phase 4 Week 1 - Performance & Stability
