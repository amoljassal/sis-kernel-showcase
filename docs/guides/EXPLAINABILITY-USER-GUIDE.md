# Explainability Features - User Guide

**Audience:** System operators, DevOps engineers, AI safety teams
**Version:** 1.0
**Date:** January 2025
**Features:** autoctl attention, autoctl whatif

---

## Introduction

This guide explains how to use the SIS Kernel's explainability features to understand autonomous AI decisions and explore "what-if" scenarios. These features support safe deployment and EU AI Act compliance by providing transparency into AI behavior.

**What You'll Learn:**
- How to interpret AI decisions with `autoctl attention`
- How to explore scenarios with `autoctl whatif`
- How to tune confidence thresholds for your use case
- Common workflows for validation and debugging
- Best practices for production deployment

---

## Quick Start

### Installation and Boot

```bash
# Build and boot the kernel with QEMU
SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh build
```

### Enable Autonomy

```bash
# In the SIS shell
sis> autoctl on
[AUTOCTL] Autonomous mode ENABLED
[AUTOCTL] Arming EL1 physical timer with 500ms interval
```

### Your First Explainability Command

```bash
# Wait a few seconds for at least one decision, then:
sis> autoctl attention

=== Decision Attention Analysis ===
Last Decision ID: #3
Timestamp: 5 seconds
...
```

You're now ready to explore the explainability features!

---

## Feature 1: autoctl attention

### Purpose

Shows which inputs influenced the last autonomous decision, providing insight into why the AI made a specific choice.

### When to Use

- **Debugging:** Why did the AI trigger compaction?
- **Validation:** Is the AI focusing on the right metrics?
- **Understanding:** What drives AI behavior in production?
- **Compliance:** Demonstrate transparency to auditors

### Command Syntax

```bash
autoctl attention
```

No parameters needed - analyzes the most recent decision automatically.

---

### Output Explanation

#### Header Section

```
=== Decision Attention Analysis ===
Last Decision ID: #46
Timestamp: 115 seconds
Explanation: Skipped action: confidence below threshold
```

**What It Means:**
- **Decision ID:** Unique identifier for this decision (for audit trail correlation)
- **Timestamp:** When the decision was made (seconds since boot)
- **Explanation:** High-level explanation code (e.g., "High memory pressure detected", "Skipped action: confidence below threshold")

#### Input Feature Influence

```
Input Feature Influence (0-100%):
  Memory Features:      [==================  ] 82% (HIGH)
  Scheduling Features:  [========            ] 41% (MEDIUM)
  Deadline Misses:      [===                 ] 15% (LOW)
  Command Rate:         [=                   ] 5% (MINIMAL)
```

**What It Means:**
- **Percentages:** How much each feature group influenced the decision
- **Progress Bars:** Visual representation (20 chars, filled proportionally)
- **Labels:** Importance level (HIGH/MEDIUM/LOW/MINIMAL)

**Interpretation:**
- **HIGH (60-100%):** This feature group was the primary driver
- **MEDIUM (40-59%):** Significant but not dominant influence
- **LOW (20-39%):** Minor influence on decision
- **MINIMAL (<20%):** Negligible impact

#### System State at Decision Time

```
System State at Decision Time:
  Memory Pressure:      85%
  Memory Fragmentation: 45%
  Deadline Misses:      3%
  Command Rate:         12/100
```

**What It Means:**
- Shows the actual state values when the decision was made
- Helps correlate feature importance with state conditions
- Useful for understanding "why now?"

#### Directives Issued

```
Directives Issued:
  Memory Directive:     -512 (Q8.8)
  Scheduling Directive: 128 (Q8.8)
  Command Directive:    64 (Q8.8)
```

**What It Means:**
- **Q8.8 format:** Fixed-point representation (8 integer bits, 8 fractional bits)
- **Negative values:** Trigger resource reduction (e.g., -512 = trigger compaction)
- **Positive values:** Trigger resource increase (e.g., +256 = increase priority)
- **Near zero:** Maintain current state

**Interpretation:**
- **< -256:** Strong decrease action
- **-256 to -50:** Moderate decrease
- **-50 to +50:** Maintain current
- **+50 to +256:** Moderate increase
- **> +256:** Strong increase action

#### Overall Decision Confidence

```
Overall Decision Confidence: 752/1000
Confidence Reason: Normal confidence level
```

**What It Means:**
- **Confidence (0-1000):** How confident the AI is in this decision
- **Confidence Reason:** Why confidence is at this level
  - `Normal` - Confidence at expected levels
  - `InsufficientHistory` - Too few decisions (<50)
  - `AllDirectivesNeutral` - Network indecisive (outputs near zero)
  - `ModelInitializing` - Very early training (<10 decisions)
  - `HighStateUncertainty` - Unusual operating conditions

#### Interpretation Guidance

```
Interpretation:
  The decision was PRIMARILY driven by memory pressure (82%).
  Memory directive has HIGH confidence (78%).

  Recommendation: Monitor memory allocation to understand decisions.
```

**What It Means:**
- **Automatic interpretation:** System analyzes feature importance and provides guidance
- **Primary driver:** Identifies which feature group had the most influence
- **Recommendations:** Actionable advice for understanding AI behavior

---

### Example Scenarios

#### Scenario 1: High Memory Pressure Triggers Compaction

```bash
sis> autoctl attention

=== Decision Attention Analysis ===
Last Decision ID: #142
Timestamp: 285 seconds
Explanation: Triggered compaction due to high memory pressure

Input Feature Influence (0-100%):
  Memory Features:      [==================  ] 92% (HIGH)
  Scheduling Features:  [==                  ] 8% (MINIMAL)
  Command Features:     [                    ] 2% (MINIMAL)

System State at Decision Time:
  Memory Pressure:      88%
  Memory Fragmentation: 72%
  Deadline Misses:      0%
  Command Rate:         3/100

Directives Issued:
  Memory Directive:     -850 (Q8.8) [trigger compaction]
  Scheduling Directive: 25 (Q8.8) [maintain current]
  Command Directive:    12 (Q8.8) [maintain current]

Overall Decision Confidence: 895/1000
Confidence Reason: Normal confidence level

Interpretation:
  The decision was PRIMARILY driven by memory conditions.
  Monitor memory allocation patterns to understand decisions.
```

**Analysis:**
- Memory features dominate (92% influence)
- High memory pressure (88%) and fragmentation (72%)
- Strong compaction directive (-850)
- High confidence (895/1000)
- **Action:** Compaction triggered appropriately

#### Scenario 2: Low Confidence Defers Action

```bash
sis> autoctl attention

=== Decision Attention Analysis ===
Last Decision ID: #8
Timestamp: 12 seconds
Explanation: Skipped action: confidence below threshold

Input Feature Influence (0-100%):
  Memory Features:      [======              ] 33% (LOW)
  Scheduling Features:  [======              ] 33% (LOW)
  Command Features:     [======              ] 34% (LOW)

System State at Decision Time:
  Memory Pressure:      5%
  Memory Fragmentation: 12%
  Deadline Misses:      0%
  Command Rate:         0/100

Directives Issued:
  Memory Directive:     0 (Q8.8)
  Scheduling Directive: 0 (Q8.8)
  Command Directive:    0 (Q8.8)

Overall Decision Confidence: 0/1000
Confidence Reason: All neural outputs near zero (model indecisive)

Interpretation:
  The decision was influenced EQUALLY by multiple factors.
  System is operating in balanced conditions.
```

**Analysis:**
- Equal feature influence (33-34%)
- All directives near zero
- Zero confidence (model indecisive)
- **Reason:** Model initializing, low system load
- **Action:** Appropriately deferred due to low confidence

---

### Common Patterns

#### Pattern: Memory-Driven Decisions

**Characteristics:**
- Memory features: >60% influence
- High memory pressure or fragmentation
- Negative memory directive (< -256)

**What It Means:** AI is responding to memory issues

**Action:** Review memory allocation patterns, consider manual compaction if needed

#### Pattern: Balanced/No-Op Decisions

**Characteristics:**
- Equal feature influence (~33% each)
- All directives near zero
- Low or zero confidence

**What It Means:** System in normal state, no action needed

**Action:** No operator intervention required, system healthy

#### Pattern: Scheduling-Driven Decisions

**Characteristics:**
- Scheduling features: >60% influence
- High deadline misses
- Significant scheduling directive (|value| > 256)

**What It Means:** AI is responding to deadline pressure

**Action:** Review task priorities and deadlines

---

## Feature 2: autoctl whatif

### Purpose

Simulates AI decisions under hypothetical conditions without actually executing them or modifying system state. Enables "what-if" scenario exploration for validation and risk assessment.

### When to Use

- **Pre-deployment:** What will the AI do under high load?
- **Threshold tuning:** Should I raise the confidence threshold?
- **Risk assessment:** Is this scenario safe?
- **Validation:** Does the AI behave correctly across operating ranges?
- **Debugging:** Why didn't the AI act in this scenario?

### Command Syntax

```bash
# Analyze current state
autoctl whatif

# Single parameter
autoctl whatif mem=80

# Multiple parameters
autoctl whatif mem=80 frag=70 misses=30

# All parameters
autoctl whatif mem=90 frag=75 misses=40 rate=50
```

**Parameters (all optional, 0-100%):**
- `mem=N` - Memory pressure (0-100%)
- `frag=N` - Memory fragmentation (0-100%)
- `misses=N` - Deadline misses (0-100%)
- `rate=N` - Command rate (0-100%)

**Note:** Parameters not specified use current system state values.

---

### Output Explanation

#### Scenario Configuration

```
=== What-If Scenario Analysis ===

Scenario: HYPOTHETICAL STATE with overrides:
  mem=80%
  frag=70%
```

**What It Means:**
- Shows which parameters you specified
- Unspecified parameters use current state

#### System State Comparison

```
--- System State Comparison ---
                      Current   ->  Hypothetical
Memory Pressure:        12%     ->  80%
Memory Fragmentation:   18%     ->  70%
Deadline Misses:        0%     ->  0%
Command Rate:           5%     ->  5%
```

**What It Means:**
- **Current:** Actual system state right now
- **Hypothetical:** State after your overrides applied
- **Arrows (->):** Visual indication of change

**Interpretation:**
- Unchanged values (0% -> 0%): Using current state
- Changed values (12% -> 80%): Your override applied

#### Predicted AI Directives

```
--- Predicted AI Directives (Q8.8 fixed-point) ---
Memory Directive:       796 (increase allocation)
Scheduling Directive:   699 (increase priority)
Command Directive:      386 (enable prediction)
```

**What It Means:**
- **Q8.8 values:** Raw directive values from neural network
- **Interpretation:** Human-readable action descriptions
  - "increase allocation" - AI wants more memory
  - "trigger compaction" - AI wants to free memory
  - "increase priority" - AI wants higher task priority
  - "decrease priority" - AI wants lower task priority
  - "enable prediction" - AI wants to use command prediction
  - "disable prediction" - AI wants to stop predicting
  - "maintain current" - AI wants no change

#### Decision Confidence

```
Decision Confidence:    62/100 (627/1000)
Would Execute?:         YES (confidence >= threshold 600/1000)
```

**What It Means:**
- **Confidence:** How confident the AI is in this scenario
- **Would Execute?:** Whether the action would actually run
  - YES: Confidence meets threshold, action would execute
  - NO: Confidence below threshold, action would be deferred

**Critical for Safety:** This tells you if enabling autonomy RIGHT NOW would trigger the predicted actions!

#### Risk Warnings

```
[WARNING] High memory pressure or fragmentation in scenario!
```

**What It Means:**
- Automatic warning when scenario conditions are risky
- Triggers on:
  - Memory pressure > 80%
  - Memory fragmentation > 60%
  - Deadline misses > 20%

---

### Example Workflows

#### Workflow 1: Pre-Deployment Validation

**Goal:** Ensure AI behaves correctly before enabling autonomy

```bash
# Test normal load scenario
sis> autoctl whatif mem=30 frag=20
Decision Confidence: 45/100
Would Execute?: NO (confidence < threshold 600/1000)
# Good: Low load doesn't trigger unnecessary actions

# Test moderate load scenario
sis> autoctl whatif mem=60 frag=40
Decision Confidence: 68/100
Would Execute?: YES (confidence >= threshold 600/1000)
Memory Directive: 512 (increase allocation)
# Good: Moderate load triggers appropriate response

# Test high load scenario
sis> autoctl whatif mem=90 frag=80
Decision Confidence: 92/100
Would Execute?: YES (confidence >= threshold 600/1000)
Memory Directive: -920 (trigger compaction)
[WARNING] High memory pressure or fragmentation in scenario!
# Good: High load triggers compaction with high confidence

# Validation complete - AI behavior looks correct across ranges
sis> autoctl on
```

#### Workflow 2: Confidence Threshold Tuning

**Goal:** Find the right confidence threshold for your risk tolerance

```bash
# Current threshold
sis> autoctl conf-threshold
[AUTOCTL] Current confidence threshold: 600/1000 (60%)

# Test borderline scenario
sis> autoctl whatif mem=75 frag=65
Decision Confidence: 65/100 (650/1000)
Would Execute?: YES (confidence >= threshold 600/1000)

# Too aggressive? Raise threshold
sis> autoctl conf-threshold 700
[AUTOCTL] Confidence threshold set to: 700/1000 (70%)

# Re-check scenario
sis> autoctl whatif mem=75 frag=65
Decision Confidence: 65/100 (650/1000)
Would Execute?: NO (confidence < threshold 700/1000)

# Now AI is more conservative - perfect!
```

#### Workflow 3: Debugging "Why Didn't It Act?"

**Goal:** Understand why autonomy didn't trigger expected action

```bash
# Operator notices high memory but no compaction triggered
# Memory is at 70%, fragmentation at 50%

# Simulate current state
sis> autoctl whatif mem=70 frag=50
Decision Confidence: 55/100 (550/1000)
Would Execute?: NO (confidence < threshold 600/1000)
Confidence Reason: Insufficient history (need more decisions)

# Ah! Confidence is below threshold
# Option 1: Wait for more decisions (build history)
# Option 2: Lower threshold temporarily
sis> autoctl conf-threshold 500
# Option 3: Trigger manual compaction
sis> memctl predict compaction
```

#### Workflow 4: Stress Testing

**Goal:** Verify AI handles extreme scenarios safely

```bash
# Extreme memory pressure
sis> autoctl whatif mem=95 frag=90
[WARNING] High memory pressure or fragmentation in scenario!
Decision Confidence: 98/100
Memory Directive: -1020 (trigger compaction)
# Good: AI responds strongly to crisis

# Combined stress
sis> autoctl whatif mem=85 frag=75 misses=35
[WARNING] High memory pressure or fragmentation in scenario!
Decision Confidence: 94/100
Memory Directive: -890 (trigger compaction)
Scheduling Directive: 780 (increase priority)
# Good: AI addresses multiple issues

# Verify actions won't exceed safety limits
# (Check that directives are within acceptable ranges)
```

---

### Integration with Other Features

#### With Approval Workflow

```bash
# Enable approval mode
sis> memctl approval on
sis> autoctl on

# Predict what will queue
sis> autoctl whatif
Memory Directive: -650 (trigger compaction)
Would Execute?: YES
# Expect compaction to queue

# Wait for operation
sis> memctl approvals
ID   | Type       | Confidence | Risk | Reason
-----|------------|------------|------|------------------
1    | Compaction | 750/1000   | 60   | High fragmentation

# Matches prediction! Approve
sis> memctl approve 1
```

#### With Preview

```bash
# Preview shows what WILL happen
sis> autoctl preview
Predicted Directives:
  Memory: -512 (trigger compaction)
Confidence: 72/100

# Whatif shows what WOULD happen under different conditions
sis> autoctl whatif mem=50
Predicted Directives:
  Memory: -200 (maintain current)
Confidence: 45/100
Would Execute?: NO

# Use together for comprehensive understanding
```

---

## Advanced Topics

### Understanding Confidence Scores

**Confidence Formula:**
```
confidence = (|memory_directive| + |scheduling_directive| + |command_directive|) / 3
```

**Normalized to 0-1000 scale:**
- 0-300: Very low (model uncertain or in normal state)
- 300-600: Low to moderate (model has some indication)
- 600-800: Moderate to high (model confident in action)
- 800-1000: Very high (model very confident, strong signal)

**Confidence Reasons:**
- **Normal:** Expected confidence for current decision count and state
- **InsufficientHistory:** <50 decisions recorded, model still learning
- **AllDirectivesNeutral:** All outputs < 50 (model sees no strong action needed)
- **ModelInitializing:** <10 decisions, very early training
- **HighStateUncertainty:** State values outside normal ranges (unusual conditions)

### Zero Side Effects Guarantee

**What "Zero Side Effects" Means:**
- `autoctl whatif` does NOT:
  - Modify the neural network state
  - Update decision statistics
  - Change the agent's last decision
  - Trigger any actual actions
  - Affect audit logs

- `autoctl whatif` DOES:
  - Temporarily inject hypothetical state
  - Run neural network inference
  - Restore original state
  - Return simulation results

**Why It Matters:**
- Safe to run as many whatif simulations as you want
- No risk of affecting production behavior
- Can explore scenarios without consequences

### Parameter Selection Guide

**mem (Memory Pressure):**
- 0-30%: Low pressure, plenty of free memory
- 30-60%: Moderate pressure, normal operation
- 60-80%: High pressure, consider action
- 80-100%: Critical pressure, immediate action needed

**frag (Memory Fragmentation):**
- 0-30%: Low fragmentation, memory compact
- 30-50%: Moderate fragmentation, manageable
- 50-70%: High fragmentation, consider compaction
- 70-100%: Severe fragmentation, compaction urgent

**misses (Deadline Misses):**
- 0-10%: Excellent deadline compliance
- 10-20%: Good compliance, minor misses
- 20-40%: Moderate misses, investigate cause
- 40-100%: Poor compliance, immediate attention

**rate (Command Rate):**
- 0-20%: Low command load
- 20-50%: Moderate load, normal operation
- 50-80%: High load, busy system
- 80-100%: Very high load, peak traffic

---

## Best Practices

### For Pre-Deployment

1. **Test Edge Cases**
   ```bash
   autoctl whatif mem=90 frag=80    # High stress
   autoctl whatif mem=5 frag=5      # Low load
   autoctl whatif mem=50 frag=50    # Normal
   ```

2. **Validate Confidence Thresholds**
   ```bash
   autoctl whatif mem=70 frag=60
   # Note confidence
   # Adjust threshold if needed
   autoctl conf-threshold <new-value>
   # Re-test
   ```

3. **Document Expected Behavior**
   - Record whatif results for baseline scenarios
   - Compare production behavior to predictions
   - Update expectations as model learns

### For Production Monitoring

1. **Regular Attention Checks**
   ```bash
   # After any significant event
   autoctl attention
   # Verify AI reasoning matches expectations
   ```

2. **Periodic Whatif Testing**
   ```bash
   # Weekly stress test
   autoctl whatif mem=85 frag=70
   # Ensure behavior remains consistent
   ```

3. **Incident Investigation**
   ```bash
   # After unexpected behavior
   autoctl attention              # Why did AI do that?
   autoctl audit last 5           # Recent decision history
   autoctl whatif mem=X frag=Y    # Can I reproduce it?
   ```

### For Debugging

1. **Compare Attention and Whatif**
   ```bash
   # What did AI actually do?
   autoctl attention

   # What would AI do now?
   autoctl whatif

   # Differences reveal state changes
   ```

2. **Narrow Down Feature Influence**
   ```bash
   # Isolate each feature
   autoctl whatif mem=90                # Only memory
   autoctl whatif frag=90               # Only fragmentation
   autoctl whatif misses=40             # Only scheduling
   autoctl whatif mem=90 frag=90        # Combined
   ```

3. **Test Threshold Sensitivity**
   ```bash
   # Find confidence breakpoint
   autoctl conf-threshold 500
   autoctl whatif mem=70
   # Increment threshold
   autoctl conf-threshold 600
   autoctl whatif mem=70
   # Continue until behavior changes
   ```

---

## Troubleshooting

### Issue: "No decisions have been made yet"

**Symptom:**
```
sis> autoctl attention
No decisions have been made yet.
```

**Cause:** Autonomy hasn't run yet

**Solution:**
```bash
autoctl on          # Enable autonomy
# Wait a few seconds
autoctl attention   # Try again
```

### Issue: All confidence scores are 0

**Symptom:**
```
Decision Confidence: 0/1000
Confidence Reason: All neural outputs near zero
```

**Cause:** Model initializing or system in normal state

**Solution:**
- Wait for more decisions (build history)
- Create load to give model signal
- Lower confidence threshold temporarily

### Issue: Whatif shows different results than actual behavior

**Symptom:** Whatif predicts compaction, but autonomy doesn't trigger it

**Possible Causes:**
1. **State changed between whatif and actual decision**
   - Solution: Run whatif immediately before checking behavior
2. **Confidence threshold changed**
   - Solution: Check `autoctl conf-threshold`
3. **Approval mode enabled**
   - Solution: Check `memctl approval status`, review `memctl approvals`

### Issue: "Would Execute" doesn't match expectations

**Symptom:**
```
Would Execute?: NO (confidence < threshold 700/1000)
```

**But you expect YES**

**Cause:** Confidence threshold too high

**Solution:**
```bash
autoctl conf-threshold          # Check current threshold
autoctl conf-threshold 600      # Lower if appropriate
autoctl whatif mem=X frag=Y     # Re-test
```

---

## Quick Reference

### Command Summary

| Command | Purpose | Parameters |
|---------|---------|------------|
| `autoctl attention` | Show feature importance for last decision | None |
| `autoctl whatif` | Simulate current state | None |
| `autoctl whatif mem=N` | Simulate with memory pressure | mem (0-100%) |
| `autoctl whatif frag=N` | Simulate with fragmentation | frag (0-100%) |
| `autoctl whatif mem=N frag=N` | Multiple conditions | mem, frag (0-100%) |
| `autoctl conf-threshold` | Show current threshold | None |
| `autoctl conf-threshold N` | Set threshold | N (0-1000) |

### Confidence Threshold Guidelines

| Use Case | Recommended Threshold | Rationale |
|----------|----------------------|-----------|
| Development/Testing | 400-500 (40-50%) | Permissive, observe all behaviors |
| Staging | 600 (60%) | Default, balanced |
| Production (Low Risk) | 600-700 (60-70%) | Moderate confidence required |
| Production (High Risk) | 700-800 (70-80%) | High confidence required |
| Safety-Critical | 800-900 (80-90%) | Very high confidence required |

### Scenario Testing Checklist

- [ ] Low load (mem=10, frag=10)
- [ ] Normal load (mem=40, frag=30)
- [ ] Moderate load (mem=60, frag=50)
- [ ] High load (mem=80, frag=70)
- [ ] Critical load (mem=95, frag=90)
- [ ] Deadline stress (misses=30)
- [ ] Combined stress (mem=80, frag=70, misses=30)

---

## Examples and Patterns

### Example 1: Complete Pre-Deployment Workflow

```bash
# 1. Review current threshold
sis> autoctl conf-threshold
[AUTOCTL] Current confidence threshold: 600/1000 (60%)

# 2. Test low load - should not trigger actions
sis> autoctl whatif mem=15 frag=10
Would Execute?: NO
✓ GOOD

# 3. Test moderate load - might trigger or might not
sis> autoctl whatif mem=50 frag=40
Would Execute?: NO (confidence 520/1000)
✓ ACCEPTABLE

# 4. Test high load - should trigger
sis> autoctl whatif mem=85 frag=70
Would Execute?: YES (confidence 780/1000)
Memory Directive: -850 (trigger compaction)
✓ GOOD

# 5. Enable autonomy
sis> autoctl on
[AUTOCTL] Autonomous mode ENABLED

# 6. Wait for first decision
# ... wait 5-10 seconds ...

# 7. Verify behavior with attention
sis> autoctl attention
Last Decision ID: #1
Confidence: 250/1000
Confidence Reason: Insufficient history
✓ Expected for first decision

# 8. Monitor over time
# Run attention periodically to verify correct behavior
```

### Example 2: Threshold Tuning Workflow

```bash
# Goal: Find threshold where mem=75 frag=60 doesn't execute

# Test with current threshold (600)
sis> autoctl whatif mem=75 frag=60
Decision Confidence: 68/100 (680/1000)
Would Execute?: YES

# Too low, raise to 700
sis> autoctl conf-threshold 700
sis> autoctl whatif mem=75 frag=60
Would Execute?: NO (confidence 680/1000 < threshold 700/1000)
✓ Threshold tuned!

# Verify high-priority scenarios still execute
sis> autoctl whatif mem=90 frag=80
Decision Confidence: 92/100 (920/1000)
Would Execute?: YES
✓ Good, critical scenarios still trigger
```

---

## Glossary

**Attention:** Feature importance analysis showing which inputs influenced a decision

**Confidence:** How certain the AI is about a decision (0-1000 scale)

**Confidence Reason:** Explanation for why confidence is at a specific level

**Confidence Threshold:** Minimum confidence required for action execution

**Directive:** AI's desired action (memory, scheduling, command)

**Feature Importance:** Percentage indicating how much each feature influenced the decision

**Hypothetical State:** Simulated system state with user-specified overrides

**Q8.8 Fixed-Point:** Number format with 8 integer bits and 8 fractional bits

**Scenario:** Hypothetical system state for whatif analysis

**Zero Side Effects:** Guarantee that simulation doesn't modify system state

---

## Additional Resources

**Documentation:**
- README.md - Main project documentation
- PHASE5-6-COMPLETION-REPORT.md - Feature implementation details
- EU-AI-ACT-COMPLIANCE-UPDATE.md - Regulatory compliance details

**Related Commands:**
- `autoctl status` - View autonomy status
- `autoctl preview` - Preview next decision
- `autoctl audit last N` - View decision history
- `memctl approvals` - View pending operations
- `compliance eu-ai-act` - View compliance status

**Getting Help:**
- GitHub Issues: https://github.com/amoljassal/sis-kernel-showcase/issues
- Command help: `help` in SIS shell

---

**Document Version:** 1.0
**Last Updated:** January 2025
**Status:** ✅ COMPLETE

---

## Feedback

This is a living document. If you have suggestions for improvements or encounter issues not covered here, please:

1. Open a GitHub issue with the label "documentation"
2. Provide specific examples of confusion or missing information
3. Suggest improvements or additional examples

Your feedback helps improve the explainability features for all users!
