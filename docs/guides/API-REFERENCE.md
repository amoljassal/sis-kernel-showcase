# SIS Kernel API Reference

Comprehensive API documentation for the SIS AI-native kernel interfaces, shell commands, and integration points.

## Table of Contents

1. [Shell Command API](#shell-command-api)
2. [Control Plane API](#control-plane-api)
3. [Metrics API](#metrics-api)
4. [Compliance API](#compliance-api)
5. [Benchmark API](#benchmark-api)
6. [Neural Network API](#neural-network-api)
7. [Autonomous Control API](#autonomous-control-api)

---

## Shell Command API

The kernel provides an interactive shell with 71 top-level commands and 165 total commands including subcommands.

### Core Commands

#### `help`
Display available commands.

**Usage:**
```
help
```

**Output:**
```
Available commands:
  help, version, reboot, ...
```

#### `version`
Display kernel version and build information.

**Usage:**
```
version
```

**Output:**
```
SIS Kernel v0.1.0
Build: aarch64-unknown-none
Features: llm,crypto-real,bringup
```

---

### Benchmark Commands

#### `benchmark memory <duration>`
Memory subsystem benchmark.

**Parameters:**
- `duration`: Test duration in seconds (1-3600)

**Usage:**
```
benchmark memory 10
```

**Output:**
```
[BENCHMARK] Memory Stress Test
  Duration: 10 seconds

Running BASELINE (AI disabled)...
Baseline complete.

Running WITH AI ENABLED...
AI test complete.

========================================
COMPARATIVE BENCHMARK RESULTS
========================================

Memory Subsystem:
  Baseline (no AI):
    - Avg Pressure: 0%
    - Peak Pressure: 0%
    - OOM Events: 0
  With AI Enabled:
    - Avg Pressure: 0%
    - Peak Pressure: 0%
    - OOM Events: 0
  Improvement: 0% OOM reduction
```

**Metrics Emitted:**
- `METRIC nn_infer_count=N` - Neural network inferences
- Memory pressure statistics
- OOM event counts

#### `benchmark commands <duration>`
Command processing benchmark.

**Parameters:**
- `duration`: Test duration in seconds (1-3600)

**Usage:**
```
benchmark commands 5
```

**Output:**
```
[BENCHMARK] Command Flood Test
  Duration: 5 seconds
  Rate: 10 commands/sec

Commands Executed: 50
```

#### `benchmark network <duration>`
Network throughput benchmark.

**Parameters:**
- `duration`: Test duration in seconds (1-3600)

**Usage:**
```
benchmark network 10
```

**Output:**
```
[BENCHMARK] Network Throughput Test
  Duration: 10 seconds

Network Subsystem:
  Baseline (no AI):
    - Packets Sent: 850677
    - Packets Lost: 17013
  With AI Enabled:
    - Packets Sent: 1710353
    - Packets Lost: 34206
```

#### `benchmark full <duration>`
Full system integration benchmark.

**Parameters:**
- `duration`: Test duration in seconds (1-3600)

**Usage:**
```
benchmark full 15
```

**Output:**
```
[BENCHMARK] Full System Integration Test
  Duration: 15 seconds
  Testing: Memory + Scheduling + Commands + Network

Commands Executed: 56891
```

#### `benchmark report`
Display benchmark results summary.

**Usage:**
```
benchmark report
```

**Output:**
```
========================================
COMPARATIVE BENCHMARK RESULTS
========================================

Memory Subsystem: [statistics]
Network Subsystem: [statistics]
Command Subsystem: [statistics]
```

---

### Compliance Commands

#### `compliance eu-ai-act`
Generate EU AI Act compliance report.

**Usage:**
```
compliance eu-ai-act
```

**Output:**
```
========================================
EU AI ACT COMPLIANCE REPORT
========================================

Article 13: Transparency
  Decision rationale: [OK] Available
  Explanations: [OK] Provided
  Human-readable output: [OK] Yes

Article 14: Human Oversight
  Override available: [OK] Yes
  Approval workflows: [ ] Optional
  Stop mechanism: [OK] Functional

Article 15: Accuracy & Robustness
  Accuracy certified: [OK] Yes
  Robustness tested: [OK] Yes
  OOD detection: [OK] Enabled
  Adversarial testing: [OK] Passed
  Cybersecurity: [OK] Validated

Article 16: Recordkeeping
  Automatic logging: [OK] Enabled
  Audit trail: [OK] Complete
  Retention: 90 days

========================================
OVERALL COMPLIANCE SCORE: 92%
Status: [OK] COMPLIANT (High-Risk AI System)
========================================
```

**Compliance Scoring:**
- Article 13: 3/3 required items
- Article 14: 2/3 items (approval workflows optional)
- Article 15: 5/5 required items
- Article 16: 3/3 required items
- Total: 13/14 = 92% (Passing threshold: 85%)

#### `compliance audit`
Generate third-party audit package.

**Usage:**
```
compliance audit
```

**Output:**
```
========================================
THIRD-PARTY AUDIT PACKAGE
========================================

Autonomous Operation:
  Total decisions: 0
  Autonomous: 0
  Manual interventions: 0

Safety Metrics:
  Watchdog triggers: 0
  Rate limit hits: 0
  Hard limit violations: 0 (ZERO TOLERANCE)
  Rollbacks: 0
  Safety score: 100/100

Performance Metrics:
  Avg reward: 0
  Prediction accuracy: 0%
  Learning updates: 0

Incident Summary:
  Critical: 0
  Errors: 0
  Warnings: 0

========================================
Package ready for third-party review
========================================
```

#### `compliance transparency`
Generate transparency report for stakeholders.

**Usage:**
```
compliance transparency
```

**Output:**
```
========================================
TRANSPARENCY REPORT
========================================

Period: Last 24 hours

Usage Statistics:
  Uptime: 6 seconds
  Autonomous operation: 0%
  Total operations: 0

Safety Statistics:
  Safety score: 100/100
  Zero-tolerance violations: 0
  Incidents resolved: 0

Performance Statistics:
  Avg accuracy: 0%
  Performance vs baseline: +0%

Model Updates:
  Versions deployed: 1
  Rollbacks: 0
```

#### `compliance checklist`
Pre-deployment safety checklist.

**Usage:**
```
compliance checklist
```

**Output:**
```
========================================
PRE-DEPLOYMENT SAFETY CHECKLIST
========================================

Core Safety (CRITICAL):
  [[OK]] Hard limits tested
  [[OK]] Watchdog functional
  [[OK]] Rate limiters verified
  [[OK]] Audit log integrity
  [[OK]] Human override tested

Learning Safety:
  [[OK]] OOD detection functional
  [[OK]] Adversarial testing passed
  [[OK]] Reward tampering detection

Operational Safety:
  [[OK]] Incremental autonomy phases
  [[OK]] Circuit breakers tested
  [[OK]] Rollback capability

Monitoring:
  [[OK]] Anomaly detection enabled
  [[OK]] Alerting system configured

Documentation:
  [[OK]] Compliance verified
  [[OK]] Incident runbook reviewed

========================================
Completion: 100%
Production Ready: [OK] YES (all critical items passed)
========================================
```

#### `compliance incidents`
View incident log.

**Usage:**
```
compliance incidents
```

**Output:**
```
========================================
INCIDENT LOG
========================================

Total Incidents: 0
  Critical: 0
  Errors: 0
  Warnings: 0

No incidents recorded.
========================================
```

---

### Autonomous Control Commands

#### `autoctl on`
Enable autonomous mode.

**Usage:**
```
autoctl on
```

**Output:**
```
[AUTOCTL] Autonomous mode ENABLED
[AUTOCTL] Meta-agent will make decisions automatically
[AUTOCTL] Timer-driven at 500ms intervals
```

**Effects:**
- Enables timer interrupts at 500ms intervals
- Activates autonomous decision-making
- Meta-agent begins evaluating system state

#### `autoctl off`
Disable autonomous mode.

**Usage:**
```
autoctl off
```

**Output:**
```
[AUTOCTL] Autonomous mode DISABLED
```

#### `autoctl status`
Display autonomous control status.

**Usage:**
```
autoctl status
```

**Output:**
```
=== Autonomous Control Status ===
  Mode: ENABLED
  Ready Flag: SET (timer will call tick)
  Safe Mode: INACTIVE
  Learning: ACTIVE
  Decision Interval: 500 ms
  Total Decisions: 152
  Audit Log: 0/1000 entries
  Accuracy (last 100): N/A
  Accuracy (last 500): N/A
  Watchdog Triggers: 0 low rewards, 0 high TD errors
```

**Return Values:**
- Mode: ENABLED | DISABLED
- Total Decisions: Count of autonomous decisions made
- Watchdog Triggers: Safety intervention count

---

### Stress Test Commands

#### `stresstest memory --duration <ms> --target-pressure <pct>`
Memory stress test with configurable duration and pressure.

**Parameters:**
- `--duration <ms>`: Test duration in milliseconds (1000-3600000)
- `--target-pressure <pct>`: Target memory pressure percentage (0-100)

**Usage:**
```
stresstest memory --duration 10000 --target-pressure 95
```

**Output:**
```
[STRESSTEST] Memory completed: peak_pressure=95% oom_events=0 compactions=5 duration_ms=10000
```

#### `stresstest commands --duration <ms> --rate <rps>`
Command flood stress test.

**Parameters:**
- `--duration <ms>`: Test duration in milliseconds
- `--rate <rps>`: Commands per second

**Usage:**
```
stresstest commands --duration 5000 --rate 50
```

**Output:**
```
[STRESSTEST] Commands completed: actions=250 duration_ms=5000
```

#### `stresstest multi --duration <ms>`
Multi-subsystem stress test.

**Parameters:**
- `--duration <ms>`: Test duration in milliseconds

**Usage:**
```
stresstest multi --duration 15000
```

**Output:**
```
[STRESSTEST] Multi-subsystem test completed
```

#### `stresstest compare <type> [options]`
Comparative stress test (AI off vs AI on).

**Parameters:**
- `type`: Test type (memory | commands | multi)
- Options: Same as individual stress tests

**Usage:**
```
stresstest compare memory --duration 10000 --target-pressure 85
```

**Output:**
```
[COMPARE] Running with autonomy DISABLED...
[COMPARE] Running with autonomy ENABLED...

=== Comparative Results ===
  Peak pressure: off=85% on=83%
  OOM events: off=0 on=0
  Duration_ms: off=10000 on=10000
```

#### `stresstest report`
Display stress test history.

**Usage:**
```
stresstest report
```

**Output:**
```
=== Stress Test History (last 16) ===
  Type: memory | Duration: 10000 ms | Actions: 0 | OOM: 0
  Type: commands | Duration: 5000 ms | Actions: 250 | OOM: 0
  ...
```

---

### Agent Control Commands

#### `agentctl stats`
Display agent network statistics.

**Usage:**
```
agentctl stats
```

**Output:**
```
=== Agent Network Statistics ===
  Memory Agent: Active
  Network Agent: Active
  Scheduling Agent: Active
  Command Agent: Active
  Meta-Agent: Active
```

#### `agentctl list`
List all available agents.

**Usage:**
```
agentctl list
```

**Output:**
```
Available Agents:
  - memory: Memory management predictions
  - network: Network flow control
  - sched: Scheduling optimization
  - command: Command prediction
  - meta: Meta-agent coordination
```

---

### Demo Commands

#### `fullautodemo`
7-phase autonomous demonstration.

**Usage:**
```
fullautodemo
```

**Phases:**
1. Collecting Baseline Metrics
2. Enabling Autonomous Mode
3. Running Multi-Subsystem Stress Test
4. AI Adaptations During Stress
5. Learning Metrics
6. Comparative Baseline (AI Disabled)
7. Quantified Performance Improvements

**Duration:** ~60 seconds

**Output:**
```
========================================
FULL AUTONOMOUS DEMO - AI-NATIVE KERNEL
========================================

[... detailed phase outputs ...]

KEY ACHIEVEMENTS:
  [OK] Zero-downtime autonomous operation
  [OK] Multi-subsystem AI coordination
  [OK] Real-time learning and adaptation
  [OK] Continuous monitoring and safety

========================================
DEMO COMPLETE
========================================
```

---

## Control Plane API

Binary control plane protocol for host ↔ kernel communication via VirtIO console.

### Frame Format

```
| Version (1 byte) | Op (1 byte) | PayloadLen (2 bytes) | Payload (N bytes) |
```

**Version:** 0x01 (current protocol version)

**Operations:**

| Op Code | Operation | Payload |
|---------|-----------|---------|
| 0x01 | CREATE | None |
| 0x02 | ADD_CHANNEL | 2 bytes: channel_id (u16) |
| 0x03 | ADD_OPERATOR | Operator config (20 bytes) |
| 0x04 | START | 2 bytes: duration_ms (u16) |
| 0x05 | STOP | None |
| 0x06 | STATUS | None |

### Host Tool: `sis_datactl.py`

**Usage:**
```bash
# Create dataflow graph
python3 tools/sis_datactl.py --token 0xHEX create

# Add channel
python3 tools/sis_datactl.py add-channel 64

# Add operator
python3 tools/sis_datactl.py add-operator 1 --in-ch 65535 --out-ch 0 --priority 10 --stage acquire

# Start graph
python3 tools/sis_datactl.py start 100

# Get status
python3 tools/sis_datactl.py status
```

**Token Rotation:**
```bash
# In kernel shell
ctlkey 0x53535F4354524C22

# From host (use new token)
python3 tools/sis_datactl.py --token 0x53535F4354524C22 create
```

---

## Metrics API

### Metric Format

```
METRIC <name>=<value>
```

**Examples:**
```
METRIC cntfrq_hz=62500000
METRIC ctx_switch_ns=992
METRIC memory_alloc_ns=25008
METRIC nn_infer_count=5
METRIC nn_infer_us=447
```

### Core Metrics

| Metric Name | Type | Description | Unit |
|-------------|------|-------------|------|
| `cntfrq_hz` | u64 | Timer frequency | Hz |
| `ctx_switch_ns` | u64 | Context switch latency | nanoseconds |
| `memory_alloc_ns` | u64 | Memory allocation latency | nanoseconds |
| `real_ctx_switch_ns` | u64 | Real context switch (with state save) | nanoseconds |

### Neural Network Metrics

| Metric Name | Type | Description | Unit |
|-------------|------|-------------|------|
| `nn_infer_count` | u64 | Total inferences | count |
| `nn_infer_us` | u64 | Inference latency | microseconds |
| `memory_agent_init` | u64 | Memory agent initialized | boolean (0/1) |
| `meta_agent_init` | u64 | Meta-agent initialized | boolean (0/1) |

### Benchmark Metrics

Extracted from benchmark command outputs:
- Commands executed count
- Network packets sent/lost
- Memory pressure (avg/peak)
- OOM event counts
- Test duration

### Collecting Metrics

**During Runtime:**
```bash
# Enable metrics
metricsctl on

# Disable metrics (for quiet operation)
metricsctl off

# Check status
metricsctl status
```

**From Test Logs:**
```bash
# Extract neural network inference count
grep "METRIC nn_infer_count=" logfile.log | tail -1 | cut -d'=' -f2

# Extract context switch latency (P50)
grep "ctx_switch_ns: P50=" logfile.log

# Extract memory allocation latency (P95)
grep "memory_alloc_ns.*P95=" logfile.log
```

---

## Compliance API

### Compliance Scores

**EU AI Act Overall Score:**
- Formula: (passing_items / total_required_items) × 100
- Threshold: ≥85% for compliance
- Current: 92% (13/14 items)

**Safety Score:**
- Formula: 100 - (violations × penalty)
- Components:
  - Hard limit violations: -100 points each
  - Watchdog triggers: -5 points each (soft penalty)
  - Rate limit hits: -1 point each
  - Rollbacks: -10 points each
- Range: 0-100
- Threshold: ≥90 for production readiness

**Checklist Completion:**
- Formula: (completed_items / total_items) × 100
- Categories: Core Safety, Learning Safety, Operational Safety, Monitoring, Documentation
- Threshold: ≥90% for production deployment

### Article Compliance Details

**Article 13: Transparency (3/3 required)**
- Decision rationale available via audit log
- Human-readable explanations via compliance commands
- Output format: Structured text, JSON available

**Article 14: Human Oversight (2/3 required)**
- Human override: `autoctl off` command
- Stop mechanism: Multiple safety layers (watchdog, rate limiter, hard limits)
- Approval workflows: Optional (not required for current deployment)

**Article 15: Accuracy & Robustness (5/5 required)**
- Accuracy certification: Benchmark validation
- Robustness testing: Stress tests (memory, commands, network)
- OOD detection: Out-of-distribution detection active
- Adversarial testing: Safety checklist validation
- Cybersecurity: Crypto-real signature verification

**Article 16: Recordkeeping (3/3 required)**
- Automatic logging: Audit log (1000 entries)
- Audit trail: Complete decision history
- Retention: 90 days (configurable)

---

## Benchmark API

### Benchmark Command Interface

All benchmark commands follow this pattern:
```
benchmark <subsystem> <duration>
```

Where:
- `subsystem`: memory | commands | network | full | report
- `duration`: Test duration in seconds (1-3600)

### Benchmark Results Format

```
========================================
COMPARATIVE BENCHMARK RESULTS
========================================

<Subsystem Name>:
  Baseline (no AI):
    - <Metric 1>: <Value>
    - <Metric 2>: <Value>
  With AI Enabled:
    - <Metric 1>: <Value>
    - <Metric 2>: <Value>
  Improvement: <Percentage>

========================================
SUMMARY
========================================
AI-native kernel achieved: [summary statistics]
```

### Performance Baselines

**QEMU (Development):**
- Context switch: ~1 µs (P50)
- Memory allocation: ~25 µs (P50)
- NN inference: ~2.3 ms (limited by QEMU simulation)
- Command rate: ~10K/sec
- Network throughput: ~1-2 Mpps

**Hardware (Expected):**
- Context switch: <1 µs
- Memory allocation: <20 µs
- NN inference: <100 µs (with NEON optimization)
- Command rate: >20K/sec
- Network throughput: >5 Mpps

---

## Neural Network API

### Network Inference

Neural network operations are performed via agent networks:
- Memory agent: Predicts memory pressure and OOM risk
- Network agent: Predicts network congestion
- Scheduling agent: Predicts task execution time
- Command agent: Predicts command success/failure
- Meta-agent: Coordinates across agents

### Inference Triggering

Inferences are triggered automatically by:
1. **Benchmark commands:** Each benchmark triggers NN inference
2. **Compliance commands:** Generate predictions for audit
3. **Autonomous mode:** Timer-driven at 500ms intervals
4. **Explicit commands:** Shell commands that invoke AI features

### Neural Network Architecture

**Memory Agent (128→64→1):**
- Inputs: 128 features (memory state, allocation patterns)
- Hidden: 64 neurons
- Output: 1 value (OOM risk prediction)

**Meta-Agent (12→16→3):**
- Inputs: 12 features (cross-subsystem state)
- Hidden: 16 neurons
- Outputs: 3 values (multi-objective decisions)

**Actor Network (12→16→6):**
- Inputs: 12 features (system state)
- Hidden: 16 neurons
- Outputs: 6 values (action probabilities)

### NEON Optimization

All neural network operations use ARM NEON SIMD instructions for performance:
- Matrix multiplication: NEON vectorized
- Activation functions: NEON optimized ReLU/tanh
- Batch operations: SIMD parallelization

---

## Autonomous Control API

### Autonomous Mode Lifecycle

```
DISABLED → (autoctl on) → ENABLED → (autonomous decisions) → DISABLED → (autoctl off)
```

### Decision Interval

Default: 500ms (2 Hz decision rate)

Configurable via:
```rust
// In crates/kernel/src/autonomy.rs
pub const DEFAULT_DECISION_INTERVAL_MS: u64 = 500;
```

### Autonomous Decision Flow

1. **Timer Interrupt** (every 500ms)
2. **State Collection** (gather system metrics)
3. **Meta-Agent Inference** (predict optimal actions)
4. **Actor Network** (select action)
5. **Action Execution** (apply decision)
6. **Watchdog Check** (validate safety)
7. **Audit Log** (record decision)

### Safety Mechanisms

**Hard Limits:**
- CPU usage < 95%
- Memory usage < 90%
- Network bandwidth < 90%
- Violations trigger immediate autonomous mode disable

**Watchdog:**
- Monitors decision quality (reward < threshold)
- Monitors learning stability (TD error > threshold)
- Triggers: Log warning, potential mode disable

**Rate Limiter:**
- Max decisions per second
- Max neural network inferences per second
- Prevents resource exhaustion

**Circuit Breaker:**
- Consecutive failure threshold
- Automatic fallback to safe mode
- Gradual recovery mechanism

---

## Integration Examples

### Python Integration

```python
import subprocess
import re

def run_benchmark(duration=10):
    """Run benchmark and extract metrics."""
    result = subprocess.run(
        ['./scripts/benchmark_suite_expect.sh', str(duration)],
        capture_output=True,
        text=True
    )

    # Extract neural network inference count
    match = re.search(r'Neural Network Inferences:\s+(\d+)', result.stdout)
    nn_inferences = int(match.group(1)) if match else 0

    # Extract commands executed
    match = re.search(r'Commands Executed:\s+(\d+)', result.stdout)
    commands = int(match.group(1)) if match else 0

    return {
        'nn_inferences': nn_inferences,
        'commands': commands,
        'success': result.returncode == 0
    }

# Usage
results = run_benchmark(15)
print(f"Benchmark results: {results}")
```

### CI/CD Integration

```yaml
# GitHub Actions example
name: SIS Kernel Validation

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install expect
        run: brew install expect

      - name: Run quick validation
        run: ./scripts/run_phase4_tests_expect.sh quick

      - name: Upload test results
        uses: actions/upload-artifact@v2
        with:
          name: test-results
          path: |
            ai_verification_results/
            benchmark_results/
            compliance_results/
```

### Monitoring Integration

```bash
#!/bin/bash
# Prometheus metrics exporter example

# Run benchmark and extract metrics
./scripts/benchmark_suite_expect.sh 15 > /tmp/bench.log

# Extract and export metrics
nn_inferences=$(grep "Neural Network Inferences:" /tmp/bench.log | awk '{print $4}')
commands=$(grep "Commands Executed:" /tmp/bench.log | awk '{print $3}')
packets=$(grep "Network throughput good" /tmp/bench.log | grep -o '[0-9]\+' | head -1)

# Write Prometheus format
cat <<EOF > /var/lib/prometheus/textfile/sis_kernel.prom
# HELP sis_nn_inferences Neural network inference count
# TYPE sis_nn_inferences counter
sis_nn_inferences $nn_inferences

# HELP sis_commands_processed Commands processed count
# TYPE sis_commands_processed counter
sis_commands_processed $commands

# HELP sis_network_packets Network packets processed
# TYPE sis_network_packets counter
sis_network_packets $packets
EOF
```

---

## Error Codes and Return Values

### Shell Command Return Codes

All shell commands return status via output messages:
- `[OK]`: Operation successful
- `[WARN]`: Operation completed with warnings
- `[ERROR]`: Operation failed
- `[FAIL]`: Validation failed

### Compliance Status Codes

- `COMPLIANT`: Score ≥85%, all critical items passed
- `NON-COMPLIANT`: Score <85% or critical items failed
- `PENDING`: Compliance check incomplete

### Safety Status Codes

- `SAFE`: Safety score ≥90/100
- `WARNING`: Safety score 70-89/100
- `UNSAFE`: Safety score <70/100

---

## API Versioning

**Current API Version:** 1.0

**Stability Guarantees:**
- Shell command interface: Stable
- Metric format: Stable
- Control plane protocol: Stable (version 0x01)
- Compliance scoring: Stable

**Future API Changes:**
- Will be documented in CHANGELOG
- Breaking changes will increment major version
- New features will increment minor version
- Bug fixes will increment patch version

---

## References

- [Hardware Deployment Guide](HARDWARE-DEPLOYMENT-READINESS.md)
- [Automated Testing Guide](AUTOMATED-TESTING-EXPECT.md)
- [Extended Testing Guide](EXTENDED-TESTING.md)
- [Compliance Framework](../plans/COMPLIANCE-FRAMEWORK.md)

---

**Last Updated:** November 4, 2025
**API Version:** 1.0
**Document Version:** 1.0
**Project Phase:** Phase 4 Week 2 - Documentation
