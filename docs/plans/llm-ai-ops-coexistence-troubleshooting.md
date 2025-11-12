# LLM and AI-Ops Feature Coexistence: Troubleshooting & Implementation Plan

**Plan ID**: llm-ai-ops-coexistence-v1
**Created**: 2025-11-12
**Priority**: High
**Estimated Effort**: 2-4 hours
**Target Branch**: `fix/llm-aiops-coexistence`

---

## Executive Summary

Enable the SIS kernel's `llm` and `ai-ops` features to coexist without mutual exclusion, allowing Phase 6 (Web GUI) and Phase 7 (AI Operations) tests to execute properly via interactive command injection through PTY-based serial communication.

---

## Background & Context

### Original Problem
The SIS kernel test suite was failing Phase 6 and Phase 7 tests because:
1. **Mutual Exclusion**: LLM commands were disabled when `ai-ops` feature was enabled via conditional compilation guards `#[cfg(all(feature = "llm", not(feature = "ai-ops")))]`
2. **Missing LLM Functions**: Basic LLM functions (Phase 0/1) were deleted from `llm/basic.rs`
3. **Netcat Socket Communication**: Command injection used obsolete netcat socket approach instead of PTY (pseudo-terminal)
4. **Line-Buffered Logging**: PTY logging used `BufReader::lines()` which blocked on shell prompts (no trailing newline)
5. **Feature Configuration**: Test runner appended features instead of replacing them

### Work Completed So Far
✅ **Removed mutual exclusion guards** in:
- `crates/kernel/src/shell.rs` (13 LLM commands)
- `crates/kernel/src/control.rs` (12 guards)
- `crates/kernel/src/shell/llmctl_helpers.rs` (1 impl block)

✅ **Restored basic LLM functions**: 771-line `llm/basic.rs` restored from git history

✅ **Fixed PTY logging**: Changed from line-buffered to chunk-based with 100ms timeout flush

✅ **Updated command injection architecture**: Modified `KernelCommandInterface` to use PTY via `QEMURuntimeManager`

✅ **Increased boot timeout**: From 90s to 180s for full feature set

### Current State - COMPILATION ERRORS
The code currently fails to compile with **30+ errors** across multiple files:

1. **Arc<QEMURuntimeManager> shutdown issue** (lib.rs:325-326):
   - Cannot call `shutdown_cluster()` on Arc (needs &mut)
   - Variable marked as mutable unnecessarily

2. **Missing constructor arguments** - All Phase test suites need updating:
   - `ai/mod.rs:47`
   - `phase1_dataflow/mod.rs:43,45,48,51,54`
   - `phase2_governance/mod.rs:40,42,45,48`
   - `phase3_temporal/mod.rs:40,42,45,48`
   - `phase5_ux_safety/mod.rs:40,42,45,48`
   - `phase6_web_gui/mod.rs:49,51,55,59,63,67`
   - `phase7_ai_ops/mod.rs:93,95,98,101`

All these files call `KernelCommandInterface::new()` with 2 arguments but now require 4:
```rust
// OLD (2 args):
KernelCommandInterface::new(serial_log_path, monitor_port)

// NEW (4 args required):
KernelCommandInterface::new(serial_log_path, qemu_manager_arc, node_id, monitor_port)
```

---

## Goals & Success Criteria

### Primary Objectives
1. ✅ Fix all compilation errors (30+ errors to resolve)
2. ✅ LLM smoke test passes with PTY command injection
3. ✅ Commands visible in serial log (llmctl, llminfer, llmjson)
4. ✅ Full test suite runs without crashes
5. ✅ Phase 6 and Phase 7 scores improve from baseline

### Success Metrics
- **Compilation**: 0 errors, 0 warnings (allow unused warnings OK)
- **LLM Smoke Test**: Exit code 0, contains "LLM smoke test passed"
- **Serial Log Validation**: Commands echoed back, prompt captured ("sis>")
- **Phase 6 Score**: > 0% (currently 0% due to command injection failure)
- **Phase 7 Score**: > 0% (currently 0% due to command injection failure)

---

## Implementation Steps

### Phase 1: Fix Arc<QEMURuntimeManager> Mutability Issues

**File**: `crates/testing/src/lib.rs`

**Problem Location**: Lines 325-326
```rust
if let Some(mut qemu_manager) = self.qemu_runtime.take() {
    qemu_manager.shutdown_cluster().await?;  // ERROR: cannot borrow Arc as mutable
```

**Solution**: Use `Arc::try_unwrap()` or keep as reference and update `shutdown_cluster()` signature

**Implementation Option A** (Recommended - Change shutdown to use interior mutability):

1. **Update `qemu_runtime.rs`** line 459:
```rust
// BEFORE:
pub async fn shutdown_cluster(&mut self) -> Result<(), TestError> {

// AFTER:
pub async fn shutdown_cluster(&self) -> Result<(), TestError> {
    // Use Arc::get_mut() or Mutex for processes if needed
```

2. **Update `lib.rs`** line 325:
```rust
// BEFORE:
if let Some(mut qemu_manager) = self.qemu_runtime.take() {
    qemu_manager.shutdown_cluster().await?;

// AFTER:
if let Some(qemu_manager) = self.qemu_runtime.take() {
    qemu_manager.shutdown_cluster().await?;
```

**Implementation Option B** (Alternative - Unwrap Arc):

Update `lib.rs` line 325:
```rust
if let Some(qemu_manager_arc) = self.qemu_runtime.take() {
    match Arc::try_unwrap(qemu_manager_arc) {
        Ok(mut qemu_manager) => {
            qemu_manager.shutdown_cluster().await?;
        }
        Err(arc) => {
            log::warn!("Cannot shutdown QEMU: Arc has multiple owners");
            // Store back
            self.qemu_runtime = Some(arc);
        }
    }
}
```

**Verification**:
```bash
cargo build -p sis-testing 2>&1 | grep -E "(error|warning.*shutdown_cluster)"
```

---

### Phase 2: Fix All KernelCommandInterface Constructor Calls

**Files to Update** (14 locations across 7 files):

#### Pattern to Apply
Replace ALL instances of:
```rust
KernelCommandInterface::new(serial_log_path, monitor_port)
```

With:
```rust
KernelCommandInterface::new(serial_log_path, MANAGER_REF.clone(), NODE_ID, monitor_port)
```

Where:
- `MANAGER_REF` = reference to `Arc<QEMURuntimeManager>` (varies by context)
- `NODE_ID` = usually `0` for single-node tests

---

#### 2.1 Fix `crates/testing/src/ai/mod.rs`

**Location**: Line 47

**Current Code**:
```rust
self.kernel_interface = Some(KernelCommandInterface::new(serial_log_path, monitor_port));
```

**Context Needed**: Check what references are available in this scope
```bash
# First, read the function context to find qemu_manager reference
```

**Action**: Read lines 30-60 of `ai/mod.rs` to understand function signature and available variables. Then update line 47 with appropriate manager reference.

**Expected Pattern**:
```rust
// If function receives qemu_manager as parameter:
self.kernel_interface = Some(KernelCommandInterface::new(
    serial_log_path,
    qemu_manager.clone(),
    0,  // node_id
    monitor_port
));
```

---

#### 2.2 Fix `crates/testing/src/phase1_dataflow/mod.rs`

**Locations**: Lines 43, 45, 48, 51, 54 (5 instances)

**Action Steps**:
1. Read lines 30-70 to understand function context
2. Identify where `qemu_manager` reference comes from
3. Apply fix pattern to all 5 lines

**Expected Changes**:
```rust
// Line 43:
kernel_interface: KernelCommandInterface::new(
    serial_log_path.clone(),
    qemu_manager.clone(),
    0,
    monitor_port
),

// Lines 45, 48, 51, 54: Similar pattern
```

---

#### 2.3 Fix `crates/testing/src/phase2_governance/mod.rs`

**Locations**: Lines 40, 42, 45, 48 (4 instances)

**Action**: Same pattern as Phase 1

---

#### 2.4 Fix `crates/testing/src/phase3_temporal/mod.rs`

**Locations**: Lines 40, 42, 45, 48 (4 instances)

**Action**: Same pattern as Phase 1

---

#### 2.5 Fix `crates/testing/src/phase5_ux_safety/mod.rs`

**Locations**: Lines 40, 42, 45, 48 (4 instances)

**Action**: Same pattern as Phase 1

---

#### 2.6 Fix `crates/testing/src/phase6_web_gui/mod.rs`

**Locations**: Lines 49, 51, 55, 59, 63, 67 (6 instances)

**Action**: Same pattern as Phase 1

---

#### 2.7 Fix `crates/testing/src/phase7_ai_ops/mod.rs`

**Locations**: Lines 93, 95, 98, 101 (4 instances)

**Action**: Same pattern as Phase 1

---

#### 2.8 Verification After Each File

After fixing each file, verify compilation progress:
```bash
cargo build -p sis-testing 2>&1 | grep -c "error\[E0061\]"
```

This count should decrease from 29 → 25 → 21 → 17 → 13 → 7 → 3 → 0 as you fix each file.

---

### Phase 3: Compilation Verification

**Action**: Build the entire testing crate
```bash
cargo build -p sis-testing --release 2>&1 | tee /tmp/build_output.log
```

**Expected Output**:
```
Compiling sis-testing v0.1.0 (...)
Finished `release` profile [optimized] target(s) in X.XXs
```

**If Errors Remain**:
1. Check `/tmp/build_output.log` for specific error messages
2. Grep for pattern: `grep "error\[E" /tmp/build_output.log`
3. Address any remaining issues following the same patterns above

---

### Phase 4: LLM Smoke Test Execution

**Action**: Run the LLM smoke test with PTY injection
```bash
cd /Users/amoljassal/sis/sis-kernel
cargo run -p sis-testing --release -- --llm-smoke 2>&1 | tee /tmp/llm_smoke_test.log
```

**Expected Duration**: 60-90 seconds (includes boot time with full feature set)

**Expected Output Patterns**:
1. **Build Success**:
   ```
   Building SIS kernel for QEMU testing
   Building kernel with features: bringup,graphctl-framed,deterministic,ai-ops,crypto-real,llm,neon-optimized
   SIS kernel and UEFI bootloader built successfully
   ```

2. **Boot Success**:
   ```
   Instance 0 booted successfully (detected via serial log)
   QEMU runtime initialized with 1 node(s)
   ```

3. **Command Injection Success** (NEW - should now appear):
   ```
   [DEBUG] Sent command to node 0 via PTY: llmctl load --wcet-cycles 50000
   [DEBUG] Sent command to node 0 via PTY: llminfer hello world from sis shell --max-tokens 8
   [DEBUG] Sent command to node 0 via PTY: llmjson
   ```

4. **Test Pass**:
   ```
   LLM smoke test passed (audit contains op=3)
   ```

**Failure Indicators to Watch For**:
- "Shell prompt not detected" → PTY logging still broken
- "llmjson had no op=3" → Commands not reaching kernel
- "Command timed out" → Commands sent but kernel not responding
- "Unknown command: llmctl" → Feature guards still present

---

### Phase 5: Serial Log Analysis

**Action**: Examine the serial log for command echoes
```bash
cat /Users/amoljassal/sis/sis-kernel/target/testing/serial-node0.log | grep -A5 "llmctl\|llminfer\|llmjson"
```

**Expected Findings**:
1. **Commands echoed back**:
   ```
   [QEMU-OUT] llmctl load --wcet-cycles 50000
   [QEMU-OUT] llminfer hello world from sis shell --max-tokens 8
   [QEMU-OUT] llmjson
   ```

2. **Shell prompts captured**:
   ```
   [QEMU-OUT] sis>
   ```

3. **LLM output present**:
   ```
   [QEMU-OUT] [LLM] load_model: Phase 1 model loaded
   [QEMU-OUT] [LLM] infer: prompt="hello world from sis shell"
   [QEMU-OUT] {"op":3,"timestamp":...}
   ```

**Problem Diagnosis**:
- If commands NOT in log → PTY write failing
- If commands in log but "Unknown command" → Feature guards still active
- If garbled output like "METRIC" → Feedback loop issue

---

### Phase 6: Full Test Suite Execution

**Action**: Run the complete test suite
```bash
cd /Users/amoljassal/sis/sis-kernel
cargo run -p sis-testing --release 2>&1 | tee /tmp/full_test_results.log
```

**Duration**: 15-30 minutes

**Key Metrics to Extract**:
```bash
# Phase 6 score
grep "Phase 6.*Score" /tmp/full_test_results.log

# Phase 7 score
grep "Phase 7.*Score" /tmp/full_test_results.log

# Overall summary
grep -A20 "Test Suite Summary" /tmp/full_test_results.log
```

**Expected Improvements**:
- **Phase 6 (Web GUI)**: Should increase from 0% to > 50%
- **Phase 7 (AI Ops)**: Should increase from 0% to > 30%
- **No new regressions**: Other phases should maintain scores

---

### Phase 7: Git Branch Creation & Commit

**Branch Naming**: `fix/llm-aiops-coexistence`

**Actions**:
```bash
cd /Users/amoljassal/sis/sis-kernel

# Create and checkout new branch
git checkout -b fix/llm-aiops-coexistence

# Stage all changes
git add crates/testing/src/lib.rs
git add crates/testing/src/kernel_interface.rs
git add crates/testing/src/qemu_runtime.rs
git add crates/testing/src/ai/mod.rs
git add crates/testing/src/phase1_dataflow/mod.rs
git add crates/testing/src/phase2_governance/mod.rs
git add crates/testing/src/phase3_temporal/mod.rs
git add crates/testing/src/phase5_ux_safety/mod.rs
git add crates/testing/src/phase6_web_gui/mod.rs
git add crates/testing/src/phase7_ai_ops/mod.rs

# Create commit with detailed message
git commit -m "$(cat <<'EOF'
fix(testing): Enable LLM and AI-Ops feature coexistence with PTY command injection

## Problem
Phase 6 and Phase 7 tests were failing because:
1. LLM commands disabled when ai-ops feature enabled (mutual exclusion guards)
2. Command injection used obsolete netcat socket approach instead of PTY
3. KernelCommandInterface architecture didn't support Arc<QEMURuntimeManager>

## Changes Made

### Architecture Updates
- Updated KernelCommandInterface to accept Arc<QEMURuntimeManager> for PTY access
- Changed TestSuite.qemu_runtime from Option<QEMURuntimeManager> to Option<Arc<QEMURuntimeManager>>
- Updated QEMURuntimeManager.write_command() to flush PTY after writes

### Constructor Updates (30+ call sites)
- Updated all KernelCommandInterface::new() calls to pass 4 arguments:
  - serial_log_path: String
  - qemu_manager: Arc<QEMURuntimeManager>
  - node_id: usize
  - monitor_port: u16

Files updated:
- crates/testing/src/ai/mod.rs
- crates/testing/src/phase1_dataflow/mod.rs
- crates/testing/src/phase2_governance/mod.rs
- crates/testing/src/phase3_temporal/mod.rs
- crates/testing/src/phase5_ux_safety/mod.rs
- crates/testing/src/phase6_web_gui/mod.rs
- crates/testing/src/phase7_ai_ops/mod.rs
- crates/testing/src/lib.rs (4 call sites in llm smoke tests)

### Shutdown Fix
- Fixed Arc<QEMURuntimeManager> mutability issue in shutdown path
- Updated shutdown_cluster() signature or used Arc::try_unwrap()

## Testing
- LLM smoke test passes with PTY command injection
- Commands visible in serial log (llmctl, llminfer, llmjson)
- Shell prompts captured properly
- Phase 6 and Phase 7 tests now execute

## Impact
- Phase 6 (Web GUI) score: 0% → X%
- Phase 7 (AI Operations) score: 0% → Y%
- No regressions in other phases

Generated with Claude Code - https://claude.com/claude-code

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"

# Push to remote
git push -u origin fix/llm-aiops-coexistence
```

---

## Detailed File Reference

### Files Modified (Summary)

| File | Lines Changed | Purpose |
|------|---------------|---------|
| `crates/testing/src/lib.rs` | 224, 311, 325, 385, 1042, 1098, 1129 | Arc wrapper, shutdown fix, 4 constructor calls |
| `crates/testing/src/kernel_interface.rs` | 1-12, 65-75, 78-88, 185-192, 402-426 | Add qemu_manager param, update constructor, PTY communication |
| `crates/testing/src/qemu_runtime.rs` | 459, 495-512 | Shutdown signature, flush PTY writes |
| `crates/testing/src/ai/mod.rs` | 47 | Constructor call |
| `crates/testing/src/phase1_dataflow/mod.rs` | 43, 45, 48, 51, 54 | Constructor calls |
| `crates/testing/src/phase2_governance/mod.rs` | 40, 42, 45, 48 | Constructor calls |
| `crates/testing/src/phase3_temporal/mod.rs` | 40, 42, 45, 48 | Constructor calls |
| `crates/testing/src/phase5_ux_safety/mod.rs` | 40, 42, 45, 48 | Constructor calls |
| `crates/testing/src/phase6_web_gui/mod.rs` | 49, 51, 55, 59, 63, 67 | Constructor calls |
| `crates/testing/src/phase7_ai_ops/mod.rs` | 93, 95, 98, 101 | Constructor calls |

**Total**: 10 files, ~50 lines changed

---

## Troubleshooting Guide

### Issue: Compilation still fails after Phase 1

**Symptom**: Arc borrowing errors persist

**Diagnosis**:
```bash
cargo build -p sis-testing 2>&1 | grep "error\[E0596\]"
```

**Solutions**:
1. Check if `shutdown_cluster()` signature was updated correctly
2. Verify `self.qemu_runtime.take()` removes `mut` from binding
3. Consider using `Arc::try_unwrap()` approach instead

---

### Issue: Many E0061 errors remain after Phase 2

**Symptom**: Still seeing "this function takes 4 arguments but 2 arguments were supplied"

**Diagnosis**:
```bash
# Count remaining errors
cargo build -p sis-testing 2>&1 | grep -c "error\[E0061\]"

# List specific locations
cargo build -p sis-testing 2>&1 | grep "error\[E0061\]" -A5
```

**Solutions**:
1. Ensure you updated ALL instances in each file (use Grep to find all)
2. Check that you're passing correct variable names for qemu_manager
3. Verify node_id is available (usually 0 or passed as parameter)

---

### Issue: LLM smoke test fails with "Shell prompt not detected"

**Symptom**: Test times out waiting for "sis>" prompt

**Diagnosis**:
```bash
tail -50 /Users/amoljassal/sis/sis-kernel/target/testing/serial-node0.log
```

**Solutions**:
1. Verify PTY logging task is using chunk-based reads (not lines())
2. Check that 100ms timeout flush is implemented
3. Increase boot timeout if needed (currently 180s)

---

### Issue: Commands not appearing in serial log

**Symptom**: No trace of llmctl/llminfer/llmjson in serial log

**Diagnosis**:
```bash
grep -i "llmctl\|llminfer\|llmjson" /Users/amoljassal/sis/sis-kernel/target/testing/serial-node0.log
```

**Solutions**:
1. Verify `write_command()` is calling `flush()` on PTY writer
2. Check that serial_writers HashMap is populated during launch
3. Add debug logging to `write_command()` to confirm it's being called
4. Verify commands aren't being sent before boot completes

---

### Issue: "Unknown command" errors for LLM commands

**Symptom**: Shell responds with "Unknown command: llmctl"

**Diagnosis**:
```bash
# Check kernel was built with correct features
grep "Building kernel with features" /tmp/llm_smoke_test.log
```

**Expected**: Should include both `ai-ops` and `llm`

**Solutions**:
1. Verify SIS_TEST_FEATURES env var is set correctly in main.rs:132
2. Confirm feature guards were removed from shell.rs, control.rs, llmctl_helpers.rs
3. Rebuild kernel with clean: `cargo clean -p sis_kernel && cargo build`

---

## Critical Paths & Dependencies

### Compilation Dependencies (Must be done in order)
1. ✅ Phase 1 (Arc/shutdown fix) → Enables Phase 2
2. ✅ Phase 2 (Constructor fixes) → Enables Phase 3
3. ✅ Phase 3 (Compilation) → Enables Phase 4

### Testing Dependencies (Can parallelize after compilation)
- Phase 4 (LLM smoke) || Phase 6 (Full suite) → Can run concurrently
- Phase 5 (Log analysis) → Requires Phase 4 output

### Git Dependencies
- Phase 7 (Git commit) → Requires Phases 3, 4, 6 to verify success

---

## Iteration Strategy

If the AI agent encounters blockers, it should:

1. **First Attempt**: Follow plan strictly
2. **If stuck**: Read surrounding code context (±20 lines) for each error location
3. **If still stuck**: Search for similar patterns in successfully fixed files
4. **Last resort**: Grep entire codebase for the error pattern and analyze all instances

**Do NOT**:
- Skip compilation verification between phases
- Commit code that doesn't compile
- Ignore test failures
- Modify files outside the testing crate (kernel code already fixed)

---

## Validation Checklist

Before marking complete, verify:

- [ ] All 30+ compilation errors resolved
- [ ] `cargo build -p sis-testing --release` succeeds
- [ ] LLM smoke test exits with code 0
- [ ] Serial log contains: "llmctl", "llminfer", "llmjson", "sis>" prompt
- [ ] Serial log contains: "[LLM]" output lines
- [ ] No "Unknown command" errors for LLM commands
- [ ] Phase 6 score > 0%
- [ ] Phase 7 score > 0%
- [ ] Git commit created with detailed message
- [ ] Git push to `fix/llm-aiops-coexistence` succeeds

---

## Contact & Questions

If the AI agent has questions or encounters unplanned scenarios:
1. Document the issue in commit message or PR description
2. Leave a TODO comment in code if incomplete
3. Ensure code at least compiles before committing

---

## References

### Previous Work
- Mutual exclusion removal: Completed in previous session
- Basic LLM functions restore: Completed (771 lines from git history)
- PTY logging fix: Completed (chunk-based with timeout flush)

### Key Git Commits
- Basic LLM restore: `a2fc346b^` (llm/basic.rs restoration point)

### Test Output Locations
- LLM smoke log: `/tmp/llm_smoke_test.log`
- Full test log: `/tmp/full_test_results.log`
- Serial capture: `/Users/amoljassal/sis/sis-kernel/target/testing/serial-node0.log`

---

**END OF PLAN**
