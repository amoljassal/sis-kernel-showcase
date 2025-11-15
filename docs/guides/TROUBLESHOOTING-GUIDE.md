# SIS Kernel Troubleshooting Guide

This guide provides solutions to common issues when building, deploying, testing, and operating the SIS AI-native kernel.

---

## Table of Contents

1. [Build Issues](#build-issues)
2. [Boot Issues](#boot-issues)
3. [Runtime Issues](#runtime-issues)
4. [Testing Issues](#testing-issues)
5. [Performance Issues](#performance-issues)
6. [Integration Issues](#integration-issues)
7. [Hardware-Specific Issues](#hardware-specific-issues)
8. [Debugging Tools and Techniques](#debugging-tools-and-techniques)

---

## Build Issues

### Issue: Cargo build fails with "linker not found"

**Symptoms:**
```
error: linker `rust-lld` not found
```

**Cause:** Missing Rust target or linker for aarch64-unknown-none

**Solution:**
```bash
# Add the target
rustup target add aarch64-unknown-none

# If still failing, reinstall Rust toolchain
rustup update stable
rustup component add rust-src
```

---

### Issue: Build fails with "no default toolchain configured"

**Symptoms:**
```
error: no default toolchain configured
```

**Cause:** Rust toolchain not installed or not set as default

**Solution:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Set default toolchain
rustup default stable

# Add aarch64 target
rustup target add aarch64-unknown-none
```

---

### Issue: Compilation error with feature flags

**Symptoms:**
```
error: feature `llm` not found in package
```

**Cause:** Incorrect feature flag syntax

**Solution:**
```bash
# Correct syntax (comma-separated, no spaces)
SIS_FEATURES="llm,crypto-real" cargo build --release --target aarch64-unknown-none

# NOT: SIS_FEATURES="llm crypto-real"  (wrong)
# NOT: SIS_FEATURES="llm, crypto-real" (wrong - space after comma)
```

---

### Issue: Out of memory during build

**Symptoms:**
```
error: could not compile `sis-kernel` due to previous error
SIGKILL received
```

**Cause:** Insufficient RAM for compilation (especially with LTO)

**Solution:**
```bash
# Disable LTO for development builds
cargo build --release --target aarch64-unknown-none --config profile.release.lto=false

# Or reduce codegen units
cargo build --release --target aarch64-unknown-none --config profile.release.codegen-units=4

# For production, use a machine with more RAM or enable swap
```

---

## Boot Issues

### Issue: QEMU hangs at UEFI firmware screen

**Symptoms:**
- QEMU window shows UEFI logo
- No boot progress
- No kernel output

**Cause:** Kernel binary not found or incorrect path

**Solution:**
```bash
# Verify kernel binary exists
ls -lh scripts/esp/EFI/SIS/KERNEL.ELF

# If missing, rebuild
./scripts/uefi_run.sh build

# Check UEFI boot path
ls -R scripts/esp/EFI/
```

**Expected Structure:**
```
scripts/esp/EFI/
├── BOOT/
│   └── BOOTAA64.EFI
└── SIS/
    └── KERNEL.ELF
```

---

### Issue: Kernel panics immediately after "!KERNEL(U)"

**Symptoms:**
```
!KERNEL(U)
PANIC: ...
```

**Cause:** Stack initialization failure or MMU setup error

**Solution:**
```bash
# Check kernel was built for correct target
file target/aarch64-unknown-none/release/sis-kernel
# Should show: ELF 64-bit LSB executable, ARM aarch64

# Rebuild with debug symbols for more info
cargo build --release --target aarch64-unknown-none

# Check QEMU version (need 5.0+)
qemu-system-aarch64 --version
```

---

### Issue: Kernel boots but hangs at "MMU ON"

**Symptoms:**
```
!KERNEL(U)
STACK OK
VECTORS OK
MMU: MAIR/TCR
MMU: TABLES
MMU: TTBR0
MMU: SCTLR
MMU ON
[hangs]
```

**Cause:** Post-MMU initialization failure (UART, GIC, timer)

**Solution:**
```bash
# Increase QEMU timeout
timeout 120 ./scripts/uefi_run.sh build

# Check QEMU arguments in script
grep "qemu-system-aarch64" scripts/uefi_run.sh

# Verify machine type is "virt"
# Should have: -machine virt,gic-version=3
```

---

### Issue: No shell prompt appears

**Symptoms:**
- Boot completes
- No "sis>" prompt
- Cursor blinking but no input

**Cause:** Shell not initialized or terminal issue

**Solution:**
```bash
# Press Enter a few times to trigger prompt

# Check serial console settings
# Should be: 115200 baud, 8N1, no flow control

# If using screen:
screen /dev/tty... 115200

# If using minicom:
minicom -D /dev/tty... -b 115200
```

---

### Issue: Boot succeeds but features missing

**Symptoms:**
- Shell works
- Commands like "imagedemo" or "llmtest" not found

**Cause:** Kernel built without required features

**Solution:**
```bash
# Rebuild with required features
SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh build

# Verify features enabled
grep "default =" crates/kernel/Cargo.toml

# Check which features are active via banner
# Should see: "Features: llm, crypto-real" in boot log
```

---

## Runtime Issues

### Issue: Out of memory (OOM) errors

**Symptoms:**
```
[ERROR] Heap allocation failed
[WARN] OOM event detected
```

**Cause:** Heap too small for workload

**Solution:**
```rust
// Edit crates/kernel/src/main.rs
pub const HEAP_SIZE: usize = 1_048_576;  // Increase to 1 MB

// Rebuild
cargo build --release --target aarch64-unknown-none
```

**Temporary Workaround:**
```bash
# Reduce workload
# Instead of: stresstest memory --duration 600000 --target-pressure 95
# Use: stresstest memory --duration 60000 --target-pressure 75
```

---

### Issue: Kernel panic with "attempted to divide by zero"

**Symptoms:**
```
PANIC at 'attempted to divide by zero'
```

**Cause:** Timer frequency (CNTFRQ_EL0) not set or zero

**Solution:**
- **In QEMU:** Should auto-detect as 62.5 MHz
- **On Hardware:** Check device tree provides timer frequency

```bash
# Verify CNTFRQ_EL0 in boot log
grep "METRIC cntfrq_hz" kernel.log

# If zero, check QEMU/hardware timer configuration
```

---

### Issue: Commands execute slowly

**Symptoms:**
- Shell commands take seconds to complete
- "help" command takes >5 seconds

**Cause:** Neural network predictions blocking command execution

**Solution:**
```bash
# Disable autonomous mode
autoctl off

# Check if performance improves

# If yes, adjust autonomous decision interval
# (requires kernel rebuild with different DECISION_INTERVAL_MS)
```

---

### Issue: Autonomous mode triggers watchdog frequently

**Symptoms:**
```
[WARN] Watchdog triggered: confidence below threshold
Autonomous decisions: 100
Watchdog triggers: 25 (25%)
```

**Cause:** Neural network predictions below confidence threshold

**Solution:**
```bash
# This is expected during cold start (first 100-200 decisions)
# Should improve over time

# If persistent:
# 1. Check neural network is being trained
autoctl status
# Look for: Neural inferences: >0

# 2. Verify heap not exhausted
# Look for: OOM events: 0

# 3. Consider pre-trained weights (future work)
```

---

### Issue: High packet loss in networking tests

**Symptoms:**
```
Packet Loss: 15.2%
Expected: <3%
```

**Cause:** Buffer overflow or congestion

**Solution:**
```bash
# If networking features enabled (Week 11):

# 1. Check buffer pool size
# Edit crates/kernel/src/network_predictor.rs
const NETWORK_BUFFER_POOL_SIZE: usize = 512_000;  // Increase to 512 KB

# 2. Reduce send rate
# In network stress test

# 3. Check for QEMU network device configuration
# scripts/uefi_run.sh should have VirtIO network device
```

---

## Testing Issues

### Issue: Expect script times out

**Symptoms:**
```
[EXPECT] Timeout waiting for shell
Test FAILED
```

**Cause:** QEMU boot time exceeds expect timeout

**Solution:**
```bash
# Increase timeout in expect script
# Edit scripts/verify_ai_active_expect.sh
set timeout 240  # Increase from 120 to 240 seconds

# Or check if QEMU process crashed
ps aux | grep qemu

# Check for kernel panic in QEMU output
```

---

### Issue: Metrics extraction returns 0

**Symptoms:**
```
Neural network inferences: 0
Expected: >0
```

**Cause:** Metric not found in output or parsing error

**Solution:**
```bash
# Debug: capture raw output
./scripts/verify_ai_active_expect.sh > debug.log 2>&1

# Check if METRIC lines present
grep "METRIC nn_infer_count" debug.log

# If missing, check if LLM feature enabled
grep "default =" crates/kernel/Cargo.toml
# Should include: "llm"

# Rebuild if needed
SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh build
```

---

### Issue: Compliance score shows 0% but should be 92%

**Symptoms:**
```
EU AI Act Compliance Score: 0%
Expected: 92%
```

**Cause:** Parsing error due to multi-line output with [EXPECT] markers

**Solution:**
```bash
# This was fixed in scripts/compliance_suite_expect.sh

# Verify you have the latest version:
grep -A 2 "OVERALL COMPLIANCE SCORE" scripts/compliance_suite_expect.sh
# Should contain: grep -v "\[EXPECT\]"

# Update script if needed:
git pull origin main
```

---

### Issue: Benchmark timeout

**Symptoms:**
```
[EXPECT] Timeout waiting for benchmark completion
Duration: 300 seconds
Expected: <360 seconds
```

**Cause:** Benchmark running longer than expected (e.g., in QEMU)

**Solution:**
```bash
# Increase expect timeout
# Edit scripts/benchmark_suite_expect.sh
calc_timeout() {
    local duration=$1
    echo $((duration + 90))  # Increase buffer from 60 to 90
}

# Or reduce benchmark duration
./scripts/benchmark_suite_expect.sh 60  # Instead of 300
```

---

### Issue: Test fails with "integer expression expected"

**Symptoms:**
```
./scripts/verify_ai_active_expect.sh: line 95: [: : integer expression expected
```

**Cause:** Variable contains whitespace or non-numeric value

**Solution:**
```bash
# This was fixed with robust parsing:
nn_infer_count=$(grep "METRIC nn_infer_count=" "$OUTPUT_FILE" | tail -1 | cut -d'=' -f2 | tr -d '[:space:]' || echo "0")
nn_infer_count=${nn_infer_count:-0}

# Verify you have the latest version:
git pull origin main

# Check if the fix is present:
grep "tr -d '\[:space:\]'" scripts/verify_ai_active_expect.sh
```

---

## Performance Issues

### Issue: Context switch latency >10 µs

**Symptoms:**
```
METRIC ctx_switch_ns=12000
Expected (QEMU): <2000 ns
```

**Cause:** Heavy system load or QEMU overhead

**Solution:**
```bash
# 1. Check system load
top

# 2. Reduce QEMU CPU load
# In scripts/uefi_run.sh, add:
-smp 1  # Single CPU (reduce contention)

# 3. Check for other QEMU instances
ps aux | grep qemu | grep -v grep | wc -l

# 4. Close unnecessary QEMU instances
```

---

### Issue: Neural network inference >5 ms

**Symptoms:**
```
METRIC nn_infer_us=5200
Expected (QEMU): ~2300 µs
```

**Cause:** NEON optimizations not enabled or heap fragmentation

**Solution:**
```bash
# 1. Verify NEON enabled
grep "target-cpu" .cargo/config.toml
# Should have: target-cpu=native

# 2. Check heap not fragmented
# Look for: compaction triggers >100 in 10 minutes

# 3. Rebuild with optimizations
cargo build --release --target aarch64-unknown-none --config profile.release.opt-level=3
```

---

### Issue: Benchmark throughput low

**Symptoms:**
```
Commands Executed: 28,000
Expected: >50,000 (for 5-minute test)
```

**Cause:** System overhead or autonomous mode interference

**Solution:**
```bash
# 1. Disable autonomous mode during benchmarks
autoctl off
benchmark commands 300
autoctl on

# 2. Check QEMU resources
# Allocate more RAM to QEMU
# In scripts/uefi_run.sh:
-m 2048  # Increase from 1024 to 2048 MB

# 3. Check for competing processes
top
```

---

## Integration Issues

### Issue: Serial console not responding

**Symptoms:**
- Characters typed but no echo
- Commands not executed

**Cause:** Terminal settings or device path incorrect

**Solution:**
```bash
# Check device exists
ls -l /dev/ttyUSB0  # or /dev/ttyACM0 or /dev/tty.usbserial-*

# Check permissions
sudo chmod 666 /dev/ttyUSB0

# Correct screen invocation
screen /dev/ttyUSB0 115200

# Correct minicom invocation
minicom -D /dev/ttyUSB0 -b 115200 -8

# Check for other processes using device
lsof /dev/ttyUSB0
```

---

### Issue: Control plane connection refused

**Symptoms:**
```
socket.error: [Errno 111] Connection refused
```

**Cause:** Control plane not enabled or port incorrect

**Solution:**
```bash
# 1. Check if VirtIO console feature enabled
grep "virtio-console" crates/kernel/Cargo.toml

# 2. Verify QEMU forwards port
grep "hostfwd" scripts/uefi_run.sh

# 3. Check kernel listening on port
# (In kernel, control plane should be bound)

# 4. Try connecting to correct port
telnet localhost 9000
```

---

### Issue: Metrics not appearing in Prometheus

**Symptoms:**
- Prometheus shows no data for SIS kernel metrics
- Exporter running but no metrics scraped

**Cause:** Exporter configuration or scrape interval

**Solution:**
```bash
# 1. Verify exporter is running
curl http://localhost:8000/metrics
# Should show SIS kernel metrics

# 2. Check Prometheus scrape config
# prometheus.yml should have:
scrape_configs:
  - job_name: 'sis-kernel'
    static_configs:
      - targets: ['localhost:8000']

# 3. Reload Prometheus configuration
curl -X POST http://localhost:9090/-/reload

# 4. Check Prometheus targets
# Navigate to http://localhost:9090/targets
# SIS kernel target should be "UP"
```

---

## Hardware-Specific Issues

### Issue: Raspberry Pi 4 boot failure

**Symptoms:**
- UEFI firmware loads
- Kernel starts but crashes early

**Cause:** GIC version mismatch (Pi 4 has GICv2, kernel expects GICv3)

**Solution:**
```rust
// Modify crates/kernel/src/main.rs
// Add GICv2 fallback detection

// Or use Raspberry Pi 5 (has GICv3)
```

**Workaround:**
Use Raspberry Pi 5 or boards with GICv3 (96Boards HiKey, NVIDIA Jetson)

---

### Issue: NVIDIA Jetson UART not detected

**Symptoms:**
- Boot progresses
- No serial output
- Shell not accessible

**Cause:** UART address differs from QEMU virt platform

**Solution:**
```bash
# 1. Check device tree for UART address
# Kernel should auto-discover via device tree parsing

# 2. Verify serial console connection
# Jetson may use /dev/ttyTHS* instead of /dev/ttyUSB*
ls -l /dev/ttyTHS*

# 3. Connect to correct device
screen /dev/ttyTHS2 115200  # Adjust TTY device
```

---

### Issue: Hardware timer frequency incorrect

**Symptoms:**
```
METRIC cntfrq_hz=0
or
METRIC cntfrq_hz=<unexpected value>
```

**Cause:** Hardware timer not configured in UEFI or device tree

**Solution:**
```bash
# 1. Check CNTFRQ_EL0 register manually
# (requires kernel modification to print raw register value)

# 2. Verify UEFI firmware sets timer frequency

# 3. Check device tree timer node
# Should have:
# timer {
#     compatible = "arm,armv8-timer";
#     clock-frequency = <62500000>;  // Or actual frequency
# };
```

---

## Debugging Tools and Techniques

### Enable Verbose Logging

**Build with perf-verbose feature:**
```bash
cargo build --release --target aarch64-unknown-none --features perf-verbose
```

**Effect:** Enables detailed [PERF] logs for debugging

---

### Capture Full Boot Log

**QEMU:**
```bash
./scripts/uefi_run.sh build > boot.log 2>&1
```

**Hardware:**
```bash
# Via screen
screen -L -Logfile boot.log /dev/ttyUSB0 115200

# Via minicom
minicom -D /dev/ttyUSB0 -b 115200 -C boot.log
```

---

### GDB Debugging

**Start QEMU with GDB server:**
```bash
# Modify scripts/uefi_run.sh, add:
-s -S  # Start GDB server, wait for connection

# In another terminal:
gdb-multiarch target/aarch64-unknown-none/release/sis-kernel
(gdb) target remote localhost:1234
(gdb) continue
```

---

### Check Heap Usage

**Command:**
```bash
# In shell, run:
benchmark report

# Look for:
# Memory: allocated=X KB, peak=Y KB, OOM events=Z
```

---

### Analyze Metrics

**Extract all metrics:**
```bash
grep "METRIC" boot.log | sort | uniq > metrics.txt
```

**Analyze trends:**
```bash
# Extract metric over time
grep "METRIC nn_infer_count" boot.log | awk -F'=' '{print NR, $2}'
```

---

### Common Log Patterns

**Successful boot:**
```
!KERNEL(U)
STACK OK
VECTORS OK
MMU: MAIR/TCR
MMU: TABLES
MMU: TTBR0
MMU: SCTLR
MMU ON
PMU: READY
UART: READY
METRIC cntfrq_hz=62500000
HEAP: READY
GIC: READY
...
=== SIS Kernel Shell ===
sis>
```

**OOM condition:**
```
[WARN] OOM event detected
[WARN] Compaction triggered
METRIC oom_events=5
```

**Autonomous mode working:**
```
[AUTONOMY] Decision tick
METRIC autonomous_decisions=42
METRIC nn_infer_count=84
METRIC watchdog_triggers=1
```

---

## Getting Help

### Collect Diagnostic Information

**Before reporting issues, collect:**

1. **Build environment:**
   ```bash
   rustc --version
   cargo --version
   uname -a
   ```

2. **Full boot log:**
   ```bash
   ./scripts/uefi_run.sh build > boot.log 2>&1
   ```

3. **Metrics dump:**
   ```bash
   grep "METRIC" boot.log > metrics.txt
   ```

4. **Feature flags:**
   ```bash
   grep "default =" crates/kernel/Cargo.toml
   ```

5. **Hardware/QEMU version:**
   ```bash
   qemu-system-aarch64 --version  # For QEMU
   # Or hardware specifications
   ```

### Report Issues

**GitHub Issues:** https://github.com/amoljassal/sis-kernel-showcase/issues

**Include:**
- Diagnostic information (above)
- Steps to reproduce
- Expected vs actual behavior
- Error messages (full, not truncated)

---

## Quick Reference

### Build Commands
```bash
# Standard build
cargo build --release --target aarch64-unknown-none

# With features
SIS_FEATURES="llm,crypto-real" cargo build --release --target aarch64-unknown-none

# QEMU run
./scripts/uefi_run.sh build
```

### Test Commands
```bash
# Quick validation
./scripts/run_phase4_tests_expect.sh quick

# Full validation
./scripts/run_phase4_tests_expect.sh full

# Extended tests
./scripts/run_extended_tests.sh benchmark-5min
./scripts/run_extended_tests.sh memory-stress
./scripts/run_extended_tests.sh autonomous-1hr
```

### Shell Commands
```bash
help                          # List all commands
version                       # Kernel version
autoctl status                # Autonomous mode status
benchmark report              # Benchmark summary
compliance eu-ai-act          # Compliance report
```

---

## References

- [Integration Guide](INTEGRATION-GUIDE.md)
- [API Reference](API-REFERENCE.md)
- [Hardware Deployment Readiness](HARDWARE-DEPLOYMENT-READINESS.md)
- [Automated Testing Guide](AUTOMATED-TESTING-EXPECT.md)
- [Extended Testing Guide](EXTENDED-TESTING.md)

---

**Last Updated:** November 4, 2025
**Document Version:** 1.0
**Project Phase:** Phase 4 Week 2 - Documentation
