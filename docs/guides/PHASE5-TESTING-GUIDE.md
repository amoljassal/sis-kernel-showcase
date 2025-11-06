# Phase 5 UX Safety Enhancements - Testing Guide

**Date:** November 4, 2025
**Phase:** Phase 5 - UX Safety Controls
**Status:** Testing Instructions

---

## Overview

This guide provides step-by-step instructions for testing all 4 Phase 5 safety control features:

1. **memctl query-mode on/off** - Dry-run mode for memory operations
2. **memctl approval on/off** - Approval gate for memory operations
3. **autoctl preview [N]** - Preview autonomous decisions
4. **autoctl phase A|B|C|D** - Phase transitions

---

## Prerequisites

1. Build and launch the kernel:
   ```bash
   SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh build
   ```

2. Wait for the shell prompt:
   ```
   === SIS Kernel Shell ===
   Type 'help' for available commands

   sis>
   ```

---

## Test 1: memctl query-mode (Dry-Run Mode)

### Purpose
Verify that memory operations can be predicted without executing.

### Test Steps

1. **Check initial status:**
   ```
   sis> memctl query-mode status
   ```
   **Expected:** `[MEMCTL] Query mode: DISABLED (normal)`

2. **Enable query mode:**
   ```
   sis> memctl query-mode on
   ```
   **Expected:**
   ```
   [MEMCTL] Query mode: ENABLED
     Memory operations will be predicted but NOT executed.
     Use 'memctl query-mode off' to resume normal operation.
   ```

3. **Test prediction in query mode:**
   ```
   sis> memctl predict compaction
   ```
   **Expected:**
   ```
   [PRED_MEM] Compaction Decision Preview:
     Predicted fragmentation (5s ahead): XX%
     Confidence: XXX/1000
     Decision: COMPACT/SKIP (...)
   ```

   If decision is COMPACT, you should also see:
   ```
   [QUERY] Would trigger compaction (dry-run mode)
   ```

4. **Verify no actual compaction occurred:**
   - Check heap stats remain unchanged
   - No compaction side effects

5. **Disable query mode:**
   ```
   sis> memctl query-mode off
   ```
   **Expected:** `[MEMCTL] Query mode: DISABLED`

6. **Verify status:**
   ```
   sis> memctl query-mode status
   ```
   **Expected:** `[MEMCTL] Query mode: DISABLED (normal)`

### Success Criteria
- ✅ Query mode can be enabled/disabled
- ✅ Status command shows correct state
- ✅ Predictions display in query mode
- ✅ "[QUERY] Would trigger..." message appears when applicable
- ✅ No actual operations execute in query mode

---

## Test 2: memctl approval (Approval Gate)

### Purpose
Verify that approval mode infrastructure is in place.

### Test Steps

1. **Check initial status:**
   ```
   sis> memctl approval status
   ```
   **Expected:** `[MEMCTL] Approval mode: DISABLED (automatic)`

2. **Enable approval mode:**
   ```
   sis> memctl approval on
   ```
   **Expected:**
   ```
   [MEMCTL] Approval mode: ENABLED
     Memory operations will require explicit confirmation.
     Use 'memctl approve' to confirm pending operations.
   ```

3. **Verify status:**
   ```
   sis> memctl approval status
   ```
   **Expected:** `[MEMCTL] Approval mode: ENABLED (requires approval)`

4. **Disable approval mode:**
   ```
   sis> memctl approval off
   ```
   **Expected:**
   ```
   [MEMCTL] Approval mode: DISABLED
     Memory operations will execute automatically.
   ```

### Success Criteria
- ✅ Approval mode can be enabled/disabled
- ✅ Status command shows correct state
- ✅ Mode persists until changed
- ✅ Help text guides user to approve/deny commands

### Note
Full approval workflow (approve/deny commands) is infrastructure for future enhancement. Current implementation establishes the flag and UI patterns.

---

## Test 3: autoctl preview (Decision Preview)

### Purpose
Verify that autonomous decisions can be previewed without execution.

### Test Steps

1. **Preview single decision:**
   ```
   sis> autoctl preview
   ```
   **Expected output structure:**
   ```
   === Autonomy Decision Preview ===
   Timestamp: XXX seconds
   Autonomy Status: DISABLED/ENABLED/SAFE MODE

   Current System State:
     Memory Pressure: XX%
     Memory Fragmentation: XX%
     Deadline Misses: XX%
     Command Rate: XX cmds/sec

   Predicted Directives (Q8.8 fixed-point):
     Memory: XXX (increase allocation/trigger compaction/maintain current)
     Scheduling: XXX (increase priority/decrease priority/maintain current)
     Command Prediction: XXX (enable prediction/disable prediction/maintain current)

   Decision Confidence: XX/100

   Use 'autoctl on' to enable autonomous execution.
   Use 'autoctl tick' to execute one decision manually.
   ```

2. **Preview multiple steps:**
   ```
   sis> autoctl preview 3
   ```
   **Expected:**
   - Note about multi-step preview
   - 3 iterations of the same state (real execution would change state)
   - Each step labeled "--- Step N ---"

3. **Preview with autonomy disabled:**
   ```
   sis> autoctl off
   sis> autoctl preview
   ```
   **Expected:**
   - Status shows "DISABLED (would take no action)"
   - Returns early with state info only

4. **Preview with autonomy enabled:**
   ```
   sis> autoctl on
   sis> autoctl preview
   ```
   **Expected:**
   - Status shows "ENABLED"
   - Full prediction with directives
   - Confidence score displayed

5. **Test warnings:**
   - If memory pressure > 80% or fragmentation > 60%:
     **Expected:** `WARNING: High memory pressure or fragmentation detected!`
   - If deadline misses > 20%:
     **Expected:** `WARNING: High deadline miss rate detected!`

6. **Test edge case:**
   ```
   sis> autoctl preview 10
   ```
   **Expected:** Maximum 5 steps (capped internally)

### Success Criteria
- ✅ Single-step preview displays correctly
- ✅ Multi-step preview shows N iterations
- ✅ Directives are interpreted correctly (positive/negative/neutral)
- ✅ Warnings appear for high pressure/misses
- ✅ Preview works with autonomy on/off
- ✅ No actual decisions executed during preview
- ✅ Help text guides user to enable autonomy

---

## Test 4: autoctl phase (Phase Transitions)

### Purpose
Verify phase transition system for production deployment control.

### Test Steps

1. **Show current phase:**
   ```
   sis> autoctl phase
   ```
   **Expected:**
   ```
   === Autonomy Phase Status ===
   Current Phase: A (Learning)
   Description: Aggressive exploration, low-risk actions only
   Max Risk Score: 30/100
   Recommended Interval: 100 ms

   Available Phases:
     A - Learning (exploration, low risk)
     B - Validation (balanced, medium risk)
     C - Production (conservative, reduced risk)
     D - Emergency (minimal autonomy, safety-critical)

   Use 'autoctl phase <A|B|C|D>' to change phase.
   ```

2. **Check status subcommand:**
   ```
   sis> autoctl phase status
   ```
   **Expected:** Same output as above

3. **Transition to Validation phase:**
   ```
   sis> autoctl phase B
   ```
   **Expected:**
   ```
   [AUTOCTL] Phase transition: A (Learning) -> B (Validation)
     Description: Balanced exploration/exploitation, medium risk allowed
     Max risk score: 60/100
     Recommended interval: 200 ms

   Consider running 'autoctl interval 200' to match phase settings.
   [AUDIT] Phase transition logged
   ```

4. **Verify phase changed:**
   ```
   sis> autoctl phase
   ```
   **Expected:** Current Phase: B (Validation)

5. **Test all phases:**
   ```
   sis> autoctl phase C
   ```
   **Expected:** Transition to Production (max risk 40/100, 500ms interval)

   ```
   sis> autoctl phase D
   ```
   **Expected:** Transition to Emergency (max risk 10/100, 2000ms interval)

   ```
   sis> autoctl phase A
   ```
   **Expected:** Return to Learning

6. **Test case-insensitive input:**
   ```
   sis> autoctl phase b
   sis> autoctl phase c
   sis> autoctl phase d
   ```
   **Expected:** All work (uppercase conversion happens internally)

7. **Test invalid input:**
   ```
   sis> autoctl phase X
   ```
   **Expected:** `Usage: autoctl phase <A|B|C|D|status>`

8. **Verify interval recommendation:**
   - After each phase change, note the recommended interval
   - Optionally set it: `autoctl interval 200` (for phase B, etc.)
   - Verify with: `autoctl status`

### Success Criteria
- ✅ Phase status displays all information
- ✅ Transitions work between all phases (A↔B↔C↔D)
- ✅ Each phase shows correct risk score and interval
- ✅ Recommendations appear for interval matching
- ✅ Audit log message appears
- ✅ Case-insensitive input works
- ✅ Invalid input shows usage help

---

## Integration Test: Combined Features

### Purpose
Test multiple features working together.

### Test Scenario

1. **Setup query mode and phase:**
   ```
   sis> memctl query-mode on
   sis> autoctl phase B
   ```

2. **Preview decision in validation phase:**
   ```
   sis> autoctl preview
   ```
   **Expected:** Decision preview with Phase B characteristics

3. **Test memory prediction with query mode:**
   ```
   sis> memctl predict compaction
   ```
   **Expected:** Dry-run prediction (no execution)

4. **Enable autonomy and preview:**
   ```
   sis> autoctl on
   sis> autoctl preview 2
   ```
   **Expected:** 2-step preview with autonomy enabled

5. **Switch to production phase:**
   ```
   sis> autoctl phase C
   sis> autoctl preview
   ```
   **Expected:** Preview reflects more conservative Phase C settings

6. **Emergency phase:**
   ```
   sis> autoctl phase D
   sis> autoctl preview
   ```
   **Expected:** Minimal autonomy, very conservative directives

7. **Cleanup:**
   ```
   sis> autoctl off
   sis> memctl query-mode off
   sis> autoctl phase A
   ```

### Success Criteria
- ✅ All features work independently
- ✅ Features can be combined without conflicts
- ✅ Phase changes affect preview output
- ✅ Query mode remains active across phase changes
- ✅ State can be reset to defaults

---

## Regression Test: Existing Functionality

### Purpose
Ensure Phase 5 additions don't break existing features.

### Test Steps

1. **Basic autonomy still works:**
   ```
   sis> autoctl status
   sis> autoctl on
   sis> autoctl tick
   sis> autoctl off
   ```

2. **Memory control still works:**
   ```
   sis> memctl status
   sis> memctl predict
   sis> memctl stress 50
   ```

3. **Help command updated:**
   ```
   sis> help
   ```
   **Expected:** New commands appear in help output:
   - `autoctl ... | preview [N] | phase [A|B|C|D] | ...`
   - `memctl ... | query-mode on/off | approval on/off`

4. **Existing autoctl commands:**
   ```
   sis> autoctl interval 500
   sis> autoctl limits
   sis> autoctl dashboard
   ```
   **Expected:** All work as before

### Success Criteria
- ✅ All existing commands work
- ✅ Help text shows new features
- ✅ No crashes or errors
- ✅ Backward compatibility maintained

---

## Performance Test

### Purpose
Verify Phase 5 features have minimal performance impact.

### Test Steps

1. **Baseline performance:**
   ```
   sis> autoctl preview
   ```
   Note response time (should be near-instant)

2. **Multi-step performance:**
   ```
   sis> autoctl preview 5
   ```
   **Expected:** Still fast (<1 second)

3. **Phase transition performance:**
   ```
   sis> autoctl phase B
   sis> autoctl phase C
   sis> autoctl phase D
   sis> autoctl phase A
   ```
   **Expected:** Instant transitions

4. **Query mode overhead:**
   ```
   sis> memctl query-mode on
   sis> memctl predict compaction
   sis> memctl predict compaction
   sis> memctl predict compaction
   ```
   **Expected:** Minimal overhead vs normal mode

### Success Criteria
- ✅ Preview completes in <100ms
- ✅ Phase transitions are instant
- ✅ Query mode adds <10% overhead
- ✅ No memory leaks over 100+ operations

---

## Error Handling Test

### Purpose
Verify proper error handling for invalid inputs.

### Test Steps

1. **Invalid query-mode argument:**
   ```
   sis> memctl query-mode invalid
   ```
   **Expected:** `Usage: memctl query-mode <on|off|status>`

2. **Invalid approval argument:**
   ```
   sis> memctl approval maybe
   ```
   **Expected:** `Usage: memctl approval <on|off|status>`

3. **Invalid phase:**
   ```
   sis> autoctl phase Z
   ```
   **Expected:** `Usage: autoctl phase <A|B|C|D|status>`

4. **Preview with invalid count:**
   ```
   sis> autoctl preview abc
   ```
   **Expected:** Defaults to 1 (parse error handled gracefully)

5. **Missing arguments:**
   ```
   sis> memctl query-mode
   ```
   **Expected:** Usage help

### Success Criteria
- ✅ All invalid inputs show usage help
- ✅ No crashes or panics
- ✅ Error messages are clear
- ✅ System remains stable after errors

---

## Automated Test Script

For convenience, here's a quick automated test sequence:

```bash
# Copy this to scripts/test_phase5.sh and run in QEMU shell

# Test 1: Query Mode
echo "=== Testing memctl query-mode ==="
memctl query-mode status
memctl query-mode on
memctl query-mode status
memctl predict compaction
memctl query-mode off
memctl query-mode status

# Test 2: Approval Mode
echo "=== Testing memctl approval ==="
memctl approval status
memctl approval on
memctl approval status
memctl approval off

# Test 3: Preview
echo "=== Testing autoctl preview ==="
autoctl preview
autoctl preview 3

# Test 4: Phase Transitions
echo "=== Testing autoctl phase ==="
autoctl phase
autoctl phase B
autoctl phase C
autoctl phase D
autoctl phase A

# Integration Test
echo "=== Integration Test ==="
memctl query-mode on
autoctl phase B
autoctl preview
memctl predict compaction
autoctl off
memctl query-mode off
autoctl phase A

echo "=== All Tests Complete ==="
```

---

## Expected Results Summary

| Feature | Command | Expected Behavior |
|---------|---------|-------------------|
| Query Mode Status | `memctl query-mode status` | Shows ENABLED or DISABLED |
| Query Mode On | `memctl query-mode on` | Enables dry-run mode |
| Query Mode Off | `memctl query-mode off` | Resumes normal operation |
| Approval Status | `memctl approval status` | Shows approval mode state |
| Approval On | `memctl approval on` | Requires confirmation |
| Approval Off | `memctl approval off` | Automatic execution |
| Preview Single | `autoctl preview` | Shows 1 decision preview |
| Preview Multi | `autoctl preview N` | Shows N previews (max 5) |
| Phase Status | `autoctl phase` | Shows current phase info |
| Phase Transition | `autoctl phase <A\|B\|C\|D>` | Changes phase, shows new settings |

---

## Troubleshooting

### Issue: Query mode not preventing execution
- **Check:** Verify `MEMORY_QUERY_MODE` flag is set
- **Debug:** Add print statement in `evaluate_compaction_policy()`

### Issue: Preview shows wrong directives
- **Check:** Ensure meta-agent is initialized
- **Debug:** Run `autoctl status` to verify autonomy system

### Issue: Phase changes not affecting behavior
- **Check:** Phase is stored in `AUTONOMY_PHASE` atomic
- **Note:** Full phase-based risk filtering is future work
- **Current:** Phase sets recommended parameters

### Issue: Approval mode doesn't block operations
- **Note:** Full approval workflow (approve/deny) is infrastructure
- **Current:** Mode flag is set, full workflow is future enhancement

---

## Test Report Template

```
PHASE 5 TESTING REPORT
Date: ___________
Tester: ___________

Feature 1: memctl query-mode
[ ] Status command works
[ ] Enable/disable works
[ ] Dry-run prevents execution
[ ] Query message appears

Feature 2: memctl approval
[ ] Status command works
[ ] Enable/disable works
[ ] Mode persists

Feature 3: autoctl preview
[ ] Single preview works
[ ] Multi-step preview works
[ ] Directives display correctly
[ ] Warnings appear appropriately

Feature 4: autoctl phase
[ ] Phase status displays
[ ] All transitions work (A↔B↔C↔D)
[ ] Risk scores correct
[ ] Recommendations appear

Integration:
[ ] Features work together
[ ] No conflicts
[ ] State can be reset

Regression:
[ ] Existing features work
[ ] Help text updated
[ ] No crashes

Overall Status: PASS / FAIL
Notes: ___________
```

---

**Document Version:** 1.0
**Last Updated:** November 4, 2025
**Status:** Ready for Testing
