# Replay Authoring Guide

**Author:** SIS Kernel Team
**Version:** 1.0
**Date:** 2025-11-05

## Overview

Replay mode enables offline testing and development by "replaying" captured UART output logs from previous QEMU runs. This allows you to test the desktop application, daemon parser logic, and API integrations without needing to run the full kernel in QEMU.

This guide explains how to capture, author, and use replay logs for development and testing.

## Table of Contents

1. [What is Replay Mode?](#what-is-replay-mode)
2. [Use Cases](#use-cases)
3. [Capturing Replay Logs](#capturing-replay-logs)
4. [Replay Log Format](#replay-log-format)
5. [Creating Custom Replay Logs](#creating-custom-replay-logs)
6. [Using Replay Mode](#using-replay-mode)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

## What is Replay Mode?

Replay mode is a testing feature that:

1. Reads pre-captured UART output from a file
2. Streams it line-by-line at a configurable rate
3. Parses it through the same pipeline as live QEMU output
4. Emits events via WebSocket to the desktop app

**Benefits:**
- Fast iteration on UI/parser changes (no QEMU boot time)
- Deterministic testing (same output every time)
- Offline development (no QEMU dependencies)
- Capture edge cases and crashes for reproducible debugging

## Use Cases

### 1. Parser Development
Test new parser patterns without waiting for kernel boot:

```bash
# Capture a log with interesting output
./scripts/uefi_run.sh > captured.log 2>&1

# Replay it while iterating on parser code
curl -X POST http://localhost:8871/api/v1/replay \
  -H "Content-Type: application/json" \
  -d '{"mode": "sample"}'
```

### 2. UI Testing
Test UI components with consistent data:

```bash
# Replay a log with metrics, graphs, or crashes
curl -X POST http://localhost:8871/api/v1/replay \
  -H "Content-Type: application/json" \
  -d '{"mode": "sample"}'

# Desktop app connects and sees replayed events
```

### 3. Crash Reproduction
Capture and replay crashes for debugging:

```bash
# Capture a crash
./scripts/uefi_run.sh > crash.log 2>&1

# Replay it to test crash panel UI
curl -X POST http://localhost:8871/api/v1/replay \
  -H "Content-Type: application/json" \
  -d '{"mode": "sample"}'
```

### 4. Demo and Training
Show features without running QEMU:

```bash
# Replay a carefully crafted demo scenario
curl -X POST http://localhost:8871/api/v1/replay \
  -H "Content-Type: application/json" \
  -d '{"mode": "sample", "rate_ms": 50}'
```

## Capturing Replay Logs

### Basic Capture

Redirect QEMU stdout/stderr to a file:

```bash
./scripts/uefi_run.sh > my_replay.log 2>&1
```

### Capture Specific Scenarios

**Boot sequence:**
```bash
BRINGUP=1 ./scripts/uefi_run.sh > boot_replay.log 2>&1
```

**LLM inference:**
```bash
SIS_FEATURES=llm ./scripts/uefi_run.sh > llm_replay.log 2>&1
# In kernel shell:
# sis> llmload /path/to/model.safetensors
# sis> llminfer "test prompt"
# Ctrl-C to stop and save
```

**Graph operations:**
```bash
SIS_FEATURES=graph-demo ./scripts/uefi_run.sh > graph_replay.log 2>&1
# In kernel shell:
# sis> graphctl create
# sis> graphctl add-channel in 512
# sis> graphctl start
# Ctrl-C to stop and save
```

**Crash scenario:**
```bash
./scripts/uefi_run.sh > crash_replay.log 2>&1
# Trigger a panic in the kernel
# Log will contain panic message and stack trace
```

### Capture with Timing Information (Future)

For realistic replay with original timing:

```bash
# Use script command to capture with timestamps
script -t 2> timing.txt my_replay.log
./scripts/uefi_run.sh
# Ctrl-D to stop

# Future: Replay with original timing
# curl -X POST http://localhost:8871/api/v1/replay \
#   -H "Content-Type: application/json" \
#   -d '{"mode": "upload", "path": "my_replay.log", "timing": "timing.txt"}'
```

## Replay Log Format

Replay logs are plain text files containing raw UART output. The daemon's parser extracts structured events from this text.

### Example Log Structure

```
KERNEL(U) entry @ 0x40080000
STACK OK: sp=0x400C0000
[MMU] SCTLR: enable I+C+M
[MMU] ON. Kernel now VA
[UART] READY
[GIC] INIT OK
[VECTORS] OK, vbar=0x40080000
[SHELL] LAUNCHING SHELL
sis>
METRIC irq_latency_ns 1234.5 1699900000000
METRIC mem_free_kb 245760 1699900001000
METRIC cpu_util 0.25 1699900002000
[GRAPH] STATE UPDATE graphId=abc123
[SCHED] WORKLOAD name=task1 pid=42 prio=120
[LOG] INFO kernel Boot complete
```

### Key Line Formats

**Boot markers:**
- `KERNEL(U)` - Kernel entry
- `STACK OK` - Stack initialized
- `MMU ON` - Memory management enabled
- `UART READY` - UART initialized
- `sis>` - Shell prompt

**Metrics:**
```
METRIC <name> <value> <timestamp_ms>
```

**Graph events:**
```
[GRAPH] STATE UPDATE graphId=<id>
[GRAPH] PREDICT result=<value>
```

**Scheduling events:**
```
[SCHED] WORKLOAD name=<name> pid=<pid> prio=<prio>
[SCHED] PRIORITY pid=<pid> old=<old> new=<new>
```

**LLM events:**
```
[LLM] LOAD path=<path> format=<format>
[LLM] INFER request_id=<id> prompt="<text>"
[LLM] TOKEN request_id=<id> chunk="<text>"
```

**Log lines:**
```
[LOG] <level> <source> <message>
```

**Crash events:**
```
[PANIC] <message>
[STACK] #0: <address> <symbol>
[STACK] #1: <address> <symbol>
```

## Creating Custom Replay Logs

You can craft custom replay logs for specific testing scenarios.

### Minimal Boot Log

```
KERNEL(U) entry @ 0x40080000
STACK OK: sp=0x400C0000
[MMU] ON. Kernel now VA
[UART] READY
[GIC] INIT OK
[VECTORS] OK
[SHELL] LAUNCHING SHELL
sis>
```

### Metrics-Heavy Log

```
KERNEL(U) entry @ 0x40080000
sis>
METRIC irq_latency_ns 1234.5 1699900000000
METRIC irq_latency_ns 1245.2 1699900000100
METRIC irq_latency_ns 1223.8 1699900000200
METRIC mem_free_kb 245760 1699900001000
METRIC mem_free_kb 243520 1699900002000
METRIC cpu_util 0.25 1699900002000
METRIC cpu_util 0.32 1699900003000
```

### Graph Workflow Log

```
sis>
[GRAPH] CREATE graphId=test123
[GRAPH] ADD_CHANNEL graphId=test123 id=in capacity=512
[GRAPH] ADD_OPERATOR graphId=test123 id=op1 type=map
[GRAPH] START graphId=test123
[GRAPH] STATE UPDATE graphId=test123
[GRAPH] PREDICT graphId=test123 result=42.5
[GRAPH] FEEDBACK graphId=test123 actual=43.0
```

### Crash Scenario Log

```
KERNEL(U) entry @ 0x40080000
sis>
[PANIC] Null pointer dereference at 0x0000000000000000
[STACK] #0: 0x400850A0 panic_handler
[STACK] #1: 0x40085200 handle_page_fault
[STACK] #2: 0x40084F00 exception_vector_sync
[REGS] x0=0x0 x1=0x400C0000 x2=0x42 sp=0x400BFF00 pc=0x400850A0
```

## Using Replay Mode

### Via Desktop App

1. **Start Daemon** (if not already running)
2. **Open Desktop App**
3. **Navigate to Replay Controls** (left sidebar)
4. **Select Mode:**
   - **Sample** - Uses bundled sample log
   - **Upload** - Browse for custom log file (future)
5. **Adjust Speed** (optional)
   - Default: 10ms per line
   - Fast: 1ms per line
   - Slow: 100ms per line
6. **Click "Start Replay"**

### Via REST API

**Start replay:**
```bash
curl -X POST http://localhost:8871/api/v1/replay \
  -H "Content-Type: application/json" \
  -d '{
    "mode": "sample",
    "rate_ms": 10
  }'
```

**Stop replay:**
```bash
curl -X POST http://localhost:8871/api/v1/replay/stop
```

**Check status:**
```bash
curl http://localhost:8871/api/v1/replay/status
```

Response:
```json
{
  "state": "running",
  "source": "samples/boot.log",
  "mode": "sample",
  "progress": 45
}
```

### Using Custom Logs (Future)

Upload a custom log file:

```bash
curl -X POST http://localhost:8871/api/v1/replay \
  -H "Content-Type: application/json" \
  -d '{
    "mode": "upload",
    "path": "/absolute/path/to/my_replay.log",
    "rate_ms": 10
  }'
```

## Best Practices

### 1. Capture Complete Scenarios

Include the full sequence from boot to the feature you're testing:

```bash
# Good: Full boot + feature
BRINGUP=1 SIS_FEATURES=llm ./scripts/uefi_run.sh > complete.log 2>&1

# Less useful: Just feature commands
echo "llminfer test" | ./scripts/uefi_run.sh > incomplete.log 2>&1
```

### 2. Name Logs Descriptively

```
good/
  boot_basic.log
  boot_llm_graph.log
  crash_null_deref.log
  metrics_heavy_load.log
  graph_prediction_workflow.log

bad/
  log1.log
  test.log
  output.log
```

### 3. Clean Up Logs

Remove unnecessary output before using as replay logs:

```bash
# Remove ANSI escape codes
cat raw.log | sed 's/\x1b\[[0-9;]*m//g' > clean.log

# Remove empty lines
grep -v '^$' raw.log > clean.log

# Remove debug spam (keep important lines)
grep -E 'KERNEL|METRIC|GRAPH|PANIC|sis>' raw.log > clean.log
```

### 4. Version Control Replay Logs

Store important replay logs in version control:

```
apps/daemon/samples/
  boot.log
  llm_inference.log
  graph_demo.log
  crash_scenarios/
    null_deref.log
    stack_overflow.log
    timeout.log
```

### 5. Test Parser Changes

After modifying parser logic, replay logs to verify:

```bash
# Make parser change
vim apps/daemon/src/parser.rs

# Rebuild
cargo build

# Test with replay
cargo run &
curl -X POST http://localhost:8871/api/v1/replay -d '{"mode": "sample"}'

# Verify output in desktop app
```

## Troubleshooting

### Replay Too Fast/Slow

Adjust the `rate_ms` parameter:

```bash
# Slower (100ms per line)
curl -X POST http://localhost:8871/api/v1/replay \
  -d '{"mode": "sample", "rate_ms": 100}'

# Faster (1ms per line)
curl -X POST http://localhost:8871/api/v1/replay \
  -d '{"mode": "sample", "rate_ms": 1}'
```

### Events Not Parsing

Check daemon logs for parser errors:

```bash
RUST_LOG=debug cargo run -p sisctl
```

Look for:
- `[PARSER] Unknown pattern: ...`
- `[PARSER] Failed to parse: ...`

### WebSocket Not Receiving Events

1. Check daemon is streaming events:
   ```bash
   curl http://localhost:8871/api/v1/replay/status
   ```

2. Check WebSocket connection in browser DevTools:
   - Network tab → WS → /events
   - Should show frames with JSON events

3. Restart replay:
   ```bash
   curl -X POST http://localhost:8871/api/v1/replay/stop
   curl -X POST http://localhost:8871/api/v1/replay -d '{"mode": "sample"}'
   ```

### Replay Stops Prematurely

Check for:
- Incomplete log file (add complete boot sequence)
- Parser errors (check daemon logs)
- Rate too fast (daemon can't keep up)

### Desktop App Shows "No QEMU Running"

This is expected in replay mode. The app should still receive events via WebSocket and display them in:
- Terminal
- Boot markers
- Metrics
- Logs
- Graph/Scheduling/LLM panels

If events aren't showing, check WebSocket connection.

## Advanced Topics

### Multi-File Replay (Future)

Combine multiple logs for complex scenarios:

```bash
curl -X POST http://localhost:8871/api/v1/replay \
  -d '{
    "mode": "multi",
    "files": [
      "boot.log",
      "llm_load.log",
      "llm_infer.log"
    ]
  }'
```

### Replay with Timing (Future)

Use captured timing information for realistic replay:

```bash
script -t 2> timing.txt replay.log
./scripts/uefi_run.sh
# Ctrl-D

curl -X POST http://localhost:8871/api/v1/replay \
  -d '{
    "mode": "upload",
    "path": "replay.log",
    "timing": "timing.txt"
  }'
```

### Replay Loop (Future)

Loop replay indefinitely for stress testing:

```bash
curl -X POST http://localhost:8871/api/v1/replay \
  -d '{
    "mode": "sample",
    "loop": true
  }'
```

## Appendix: Sample Replay Logs

### A. Minimal Boot

Located at `apps/daemon/samples/boot.log`:

```
KERNEL(U) entry @ 0x40080000
STACK OK: sp=0x400C0000
[MMU] SCTLR: enable I+C+M
[MMU] ON. Kernel now VA
[UART] READY
[GIC] INIT OK
[VECTORS] OK, vbar=0x40080000
[SHELL] LAUNCHING SHELL
sis>
```

### B. Full Feature Demo

Located at `apps/daemon/samples/full_demo.log` (future):

- Complete boot sequence
- Metrics stream
- Graph creation and operations
- Scheduling workload changes
- LLM inference
- Crash and recovery

## Contributing

To add new replay logs:

1. Capture a log with a specific scenario
2. Clean and verify the log
3. Add to `apps/daemon/samples/`
4. Update this guide with the scenario
5. Add unit tests for new parser patterns

## References

- [Parser Implementation](../../apps/daemon/src/parser.rs)
- [Replay Manager](../../apps/daemon/src/qemu/replay.rs)
- [Desktop Replay Controls](../../apps/desktop/src/components/ReplayControls.tsx)
- [API Documentation](../README.md)

## Changelog

- **2025-11-05** - Initial version (M5 completion)
