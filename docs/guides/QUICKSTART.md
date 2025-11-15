# SIS Kernel Quick Start (5 Minutes)

Get the AI-native kernel running and see intelligent scheduling in action.

## Prerequisites

- **macOS/Linux** with 8GB RAM
- **QEMU** installed:
  ```bash
  # macOS
  brew install qemu

  # Linux
  apt install qemu-system-aarch64
  ```

- **Rust toolchain** 1.75+ (optional for rebuilding)

---

## Step 1: Clone & Build (2 minutes)

```bash
git clone https://github.com/amoljassal/sis-kernel-showcase.git
cd sis-kernel-showcase

# Build kernel (this may take 1-2 minutes)
./scripts/uefi_run.sh build
```

**Expected output:**
```
Compiling sis-kernel v1.0.0
Compiling uefi-boot v1.0.0
    Finished release [optimized] target(s)
✓ Build complete
```

---

## Step 2: Boot Kernel (30 seconds)

```bash
# Boot with AI features enabled
BRINGUP=1 SIS_FEATURES="llm,ai-ops" ./scripts/uefi_run.sh
```

**Expected output:**
```
SIS Kernel v1.0 [AI-Native]
VFS: MOUNT EXT4 AT /models (rw+journal)
AI: NN inference ready (45µs avg)
AI: Transformer scheduler initialized
AI: Crash predictor active
Shell ready. Type 'help' for commands.
>
```

**Tip:** If QEMU window appears, press `Ctrl+Alt+2` to access serial console.

---

## Step 3: See AI Scheduling (1 minute)

```bash
# Enable AI-powered transformer scheduler
> schedctl transformer on
Transformer scheduler enabled

# Show real-time AI metrics
> autoctl ai-metrics
AI Performance:
  NN Inference: 42µs (p50), 89µs (p99)
  Decision Follow Rate: 87.3%
  Crash Prediction Accuracy: 84.0%
  Context Switch Improvement: 18.5%
  Memory Saved: 247 MB

# Enable autonomous operation (AI makes decisions automatically)
> autoctl on
[Autonomy ON - AI making real-time scheduling decisions]
[AI] Task 5 priority boosted (CPU-bound pattern detected)
[AI] Memory compaction recommended (fragmentation: 0.71)
```

**What's happening:**
- Transformer scheduler uses attention mechanism to prioritize tasks
- AI analyzes historical task patterns
- Real-time metrics show performance improvements

---

## Step 4: Test Crash Prediction (1 minute)

```bash
# Trigger memory stress to activate crash predictor
> stresstest mem --pressure high
Allocating memory aggressively...

# Watch AI predict potential crash
> crashctl status
Crash Prediction:
  Confidence: 78% (WARNING)
  Risk Level: ELEVATED
  Factors:
    - Memory fragmentation: 0.73 (increasing trend)
    - Free pages declining: -15% over 5s
    - Allocation failures: 3 recent
  Recommendation: Run 'memctl compact' to prevent OOM

# AI automatically triggers compaction at 90% confidence
[AI] Executing: memctl compact
Memory compacted: 247MB freed
Crash prediction: Confidence dropped to 12% (NORMAL)
```

**What's happening:**
- Crash predictor monitors memory patterns
- Linear regression detects fragmentation trends
- AI automatically prevents predicted crashes

---

## Step 5: Export Decision Traces (30 seconds)

All AI decisions are logged for explainability and EU AI Act compliance.

```bash
# View recent AI decisions
> tracectl list --recent 10
Trace ID | Timestamp | Decision | Confidence | Outcome
---------|-----------|----------|------------|--------
TR-0042  | 10:23:15  | boost_priority(task=5) | 0.87 | success
TR-0043  | 10:23:16  | memory_compact()       | 0.92 | success
TR-0044  | 10:23:18  | schedule_task(task=3)  | 0.74 | success
...

# Export to file (persisted to ext4 filesystem)
> tracectl export --recent 10 --path /incidents/demo.json
Exported 10 traces to /incidents/demo.json

# Verify file persisted
> ls /incidents
demo.json  (2.4 KB)

# View trace details
> cat /incidents/demo.json
{
  "traces": [
    {
      "trace_id": "TR-0042",
      "timestamp_ms": 1699459395000,
      "decision_type": "boost_priority",
      "input_features": {"cpu_ratio": 0.95, "priority": 5},
      "confidence": 0.87,
      "outcome": "success",
      "model_version": "transformer-v1.0"
    },
    ...
  ]
}
```

**What's happening:**
- All AI decisions are traceable
- Audit trail for compliance
- Persisted to ext4 with journaling

---

## Next Steps

### Learn More

- **[AI Scheduling Tutorial](./AI-SCHEDULING-TUTORIAL.md)** - Deep dive into transformer scheduler
- **[Decision Tracing Tutorial](./DECISION-TRACING-TUTORIAL.md)** - Explainability and compliance
- **[Ext4 Journaling Tutorial](./EXT4-JOURNALING-TUTORIAL.md)** - Crash recovery mechanics

### Advanced Features

```bash
# Fine-tune LLM model
> llmctl finetune /models/finetune-data.json

# Run LLM inference on system state
> llmctl infer "What's causing high memory usage?"
LLM Output: "Based on 847MB used with 0.73 fragmentation, run 'memctl compact'"

# View power estimation
> powerctl status
Estimated Power: 5.4W (medium workload)

# Run comprehensive benchmarks
> cargo test --release --features benchmark
```

### Test Full Suite

```bash
# Run all Phase 1 AI features
./scripts/run_ai_demo.sh

# Run crash recovery tests
./scripts/ext4_crash_recovery_harness.sh 5

# Run VFS fuzzing
cargo fuzz run vfs_path_lookup -- -max_total_time=3600
```

---

## Troubleshooting

### Kernel doesn't boot

**Check QEMU version:**
```bash
qemu-system-aarch64 --version
# Need ≥7.0
```

**Try without AI features:**
```bash
BRINGUP=1 ./scripts/uefi_run.sh
```

**Check build output:**
```bash
cargo build --release 2>&1 | grep error
```

---

### AI metrics show 0%

**Wait for warmup (30 seconds):**
AI features collect baseline metrics before making decisions.

**Run workload to generate activity:**
```bash
> stresstest cpu --duration 10
> stresstest mem --duration 10
```

**Check if features are enabled:**
```bash
> autoctl status
Autonomy: OFF  # Should be ON

> schedctl transformer status
Transformer: DISABLED  # Should be ENABLED
```

---

### File operations fail

**Check ext4 filesystem:**
```bash
> ls /models
# Should show model files

> ls /incidents
# Should be writable
```

**Try simple write:**
```bash
> echo test > /incidents/test.txt
> cat /incidents/test.txt
test
```

---

### QEMU crashes

**Reduce memory pressure:**
```bash
# Edit scripts/uefi_run.sh
# Change -m 1024M to -m 512M (line ~20)
```

**Check available disk space:**
```bash
df -h
# Need ~2GB free for ext4 image
```

---

## Key Commands Reference

### AI Features
```bash
schedctl transformer on/off/stats/reset
crashctl status/history/tune <threshold>
llmctl load/infer/finetune/status
autoctl on/off/ai-metrics/export-metrics/reset-baseline
tracectl list/export/filter/stats
powerctl status/history/set-budget/calibrate
```

### System
```bash
help                    # Show all commands
memctl status/compact   # Memory management
schedctl stats          # Scheduler statistics
stresstest cpu/mem      # Load testing
ls/cat/echo/rm         # File operations
```

### Debugging
```bash
tracectl list --verbose    # Detailed decision traces
crashctl history 50        # Last 50 predictions
autoctl ai-metrics --json  # Export metrics
perf stats                 # PMU counters
```

---

## Performance Expectations

Running on QEMU (aarch64):

| Metric | Value | Notes |
|--------|-------|-------|
| Boot Time | ~3s | UEFI → Shell |
| AI Inference (p50) | ~42µs | Transformer scheduler |
| AI Inference (p99) | ~89µs | Target <100µs ✓ |
| Context Switch | ~980ns | 21.6% faster than baseline |
| Crash Prediction | 84% | Accuracy on stress tests |
| Memory Compaction | ~200-300MB | Per compaction cycle |

---

## What Makes This Kernel Unique?

1. **Real-Time AI Inference:**
   - <100µs inference latency in kernel space
   - Transformer-based task scheduling
   - Predictive crash detection

2. **Full Explainability:**
   - Every AI decision logged
   - EU AI Act compliant (92% score)
   - Complete audit trails

3. **Production-Grade Ext4:**
   - Write support with journaling
   - Crash recovery validated
   - Forensic analysis capable

4. **Zero-Allocation AI:**
   - Ring buffers for constant memory
   - No runtime allocation in hot paths
   - Kernel-safe no_std implementation

5. **Autonomous Operation:**
   - Self-tuning scheduler
   - Automatic memory management
   - Proactive crash prevention

---

## Demo Video

Watch the 5-minute demo video: [YouTube Link]

---

## Questions?

- **GitHub Issues:** https://github.com/amoljassal/sis-kernel-showcase/issues
- **Documentation:** All guides in `docs/guides/`
- **Source Code:** Well-commented Rust code in `crates/kernel/src/`

---

**Last Updated:** 2025-11-08
**Kernel Version:** SIS v1.0 AI-Native
**Tested On:** QEMU 8.1.0 (aarch64), macOS 14.2
