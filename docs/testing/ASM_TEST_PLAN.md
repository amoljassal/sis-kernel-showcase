# Agent Supervision Module (ASM) Test Plan

**Date**: 2025-11-16
**Status**: Planning
**Priority**: P1 - Critical Integration Testing
**Timeline**: 4 weeks
**Owner**: SIS Kernel Team

---

## Executive Summary

This document defines the comprehensive test plan for validating the Agent Supervision Module (ASM) through end-to-end integration tests. The plan covers all ASM features including lifecycle management, telemetry, EU AI Act compliance, resource monitoring, dependency management, policy control, cloud gateway, and performance profiling.

**Goal**: Achieve 100% integration test coverage for all ASM features in the external test suite.

---

## Table of Contents

1. [Test Strategy](#test-strategy)
2. [Week 1: Shell Commands](#week-1-shell-commands)
3. [Week 2: Lifecycle & Telemetry Tests](#week-2-lifecycle--telemetry-tests)
4. [Week 3: Resource Monitoring & Dependencies](#week-3-resource-monitoring--dependencies)
5. [Week 4: Cloud Gateway & Stress Tests](#week-4-cloud-gateway--stress-tests)
6. [Test Environment](#test-environment)
7. [Success Criteria](#success-criteria)
8. [Risk Mitigation](#risk-mitigation)

---

## Test Strategy

### Testing Approach

**Three-Layer Testing**:
1. **Unit Tests** (✅ Complete): Individual component validation in kernel modules
2. **Integration Tests** (⚠️ In Progress): End-to-end validation via external test suite
3. **Stress Tests** (🔜 Planned): High-load scenarios with 100+ agents

### Test Execution Method

All integration tests will:
- Run in **QEMU** environment via `crates/testing`
- Use **shell commands** to interact with ASM
- Validate **kernel output** via serial log parsing
- Verify **/proc exports** for data integrity
- Use **KernelCommandInterface** for command execution

### Test Data Strategy

- Test agents with known IDs (1000-1999 range)
- Deterministic test scenarios for reproducibility
- Cleanup after each test to prevent state pollution
- Use test-specific capability sets

---

## Week 1: Shell Commands

**Objective**: Add all necessary shell commands to enable external ASM testing

### 1.1 Lifecycle Commands

#### TC-LC-001: `agentsys spawn`
**Command**: `agentsys spawn <agent_id> <name> <capabilities>`
**Purpose**: Spawn a test agent with specific capabilities
**Expected Output**:
```
[ASM] Spawning agent 1000: test_agent
[ASM] Capabilities: FsBasic, NetClient
[ASM] Agent 1000 spawned successfully (PID: 42)
```

**Implementation Location**: `crates/kernel/src/shell/asm_helpers.rs`

**Validation**:
- Agent appears in telemetry
- Agent has correct capabilities
- PID mapping is correct

---

#### TC-LC-002: `agentsys kill`
**Command**: `agentsys kill <agent_id>`
**Purpose**: Forcefully terminate an agent
**Expected Output**:
```
[ASM] Killing agent 1000
[ASM] Agent 1000 terminated (PID: 42)
```

**Validation**:
- Agent removed from supervisor
- Telemetry updated
- Exit event logged

---

#### TC-LC-003: `agentsys restart`
**Command**: `agentsys restart <agent_id>`
**Purpose**: Manually trigger agent restart
**Expected Output**:
```
[ASM] Restarting agent 1000 (attempt 1/3)
[ASM] Agent 1000 restarted (new PID: 43)
```

**Validation**:
- Old agent terminated
- New agent spawned
- Restart count incremented

---

#### TC-LC-004: `agentsys list`
**Command**: `agentsys list`
**Purpose**: Show all active agents
**Expected Output**:
```
[ASM] Active Agents:
  ID    Name           PID   Capabilities
  ----  -------------  ----  ------------------
  1000  test_agent     42    FsBasic, NetClient
  1001  monitor_agent  43    Admin
  Total: 2 agents
```

**Validation**:
- All active agents shown
- Correct metadata displayed
- Count matches telemetry

---

### 1.2 Telemetry Commands

#### TC-TM-001: `agentsys telemetry`
**Command**: `agentsys telemetry`
**Purpose**: Display system-wide telemetry snapshot
**Expected Output**:
```
Agent Supervision Module - Telemetry Status
===========================================

System Metrics:
  Total Spawns:    10
  Total Exits:     8
  Total Faults:    3
  Total Restarts:  2
  Active Agents:   2
  Total Syscalls:  1,234,567

Per-Agent Metrics:
  ID    Spawns Exits  Faults CPU(us)   Mem(B)
  ----  ------ -----  ------ --------  -------
  1000  3      2      1      125,000   4,096
  1001  1      0      0      50,000    2,048
```

**Validation**:
- Counts are accurate
- Per-agent metrics correct
- Matches /proc export

---

#### TC-TM-002: `agentsys metrics`
**Command**: `agentsys metrics <agent_id>`
**Purpose**: Show detailed metrics for specific agent
**Expected Output**:
```
Agent 1000 Metrics:
  Spawn Count:       3
  Exit Count:        2
  Fault Count:       1
  CPU Time:          125,000 us
  Memory Usage:      4,096 bytes
  Syscall Count:     5,678
  Last Spawn:        12:34:56.789
  Last Exit:         12:35:10.123
  Last Exit Code:    0
  Recent Faults:
    - CpuQuotaExceeded (used: 150000, quota: 100000)
```

**Validation**:
- All metrics present
- Recent faults shown
- Timestamps accurate

---

### 1.3 Compliance Commands

#### TC-CP-001: `agentsys compliance`
**Command**: `agentsys compliance` (✅ Already exists)
**Purpose**: Generate EU AI Act compliance report
**Expected Output**:
```
EU AI Act Compliance Report
===========================
Generated: 2025-11-16 12:34:56 UTC

System Compliance:
  Total Agents:           10
  Policy Violations:      2
  System Score:           0.85

Risk Classification:
  Minimal Risk:           7 agents
  Limited Risk:           2 agents
  High Risk:              1 agent
  Unacceptable Risk:      0 agents

Agent Details:
  ID    Risk Level        Transparency  Operations  Reviews
  ----  ----------------  ------------  ----------  -------
  1000  Minimal           0.95          150         5
  1001  Limited           0.75          500         20
```

**Validation**:
- Risk classification correct
- Scores calculated properly
- Violation tracking accurate

---

#### TC-CP-002: `agentsys risk`
**Command**: `agentsys risk <agent_id>`
**Purpose**: Show risk classification for specific agent
**Expected Output**:
```
Agent 1000 Risk Assessment:
  Risk Level:        Minimal
  Transparency:      0.95
  Compliance Score:  0.92

  Factors:
    - Operations: 150 (low activity)
    - Violations: 0 (clean record)
    - Human Reviews: 5 (adequate oversight)
    - Capabilities: FsBasic, NetClient (low privilege)
```

**Validation**:
- Risk level matches compliance report
- Factors explain classification
- Score calculation correct

---

### 1.4 Resource Monitoring Commands

#### TC-RM-001: `agentsys resources`
**Command**: `agentsys resources <agent_id>`
**Purpose**: Show current resource usage
**Expected Output**:
```
Agent 1000 Resource Usage:
  CPU Time:          125,000 us (quota: 1,000,000 us)
  Memory:            4,096 bytes (limit: 104,857,600 bytes)
  Syscalls:          5,678 (rate: 150/sec, limit: 1000/sec)
  Watchdog:          5,000 us idle (timeout: 30,000,000 us)

  Status: Normal
```

**Validation**:
- Usage values accurate
- Limits displayed correctly
- Status reflects actual state

---

#### TC-RM-002: `agentsys limits`
**Command**: `agentsys limits <agent_id>`
**Purpose**: Show resource limits configuration
**Expected Output**:
```
Agent 1000 Resource Limits:
  CPU Quota:         1,000,000 us per window
  Memory Limit:      104,857,600 bytes (100 MiB)
  Syscall Rate:      1,000 calls/second
  Watchdog Timeout:  30,000,000 us (30 seconds)

  Policy: Default
```

**Validation**:
- All limits shown
- Policy identified
- Values match configuration

---

### 1.5 Dependency Commands

#### TC-DP-001: `agentsys deps`
**Command**: `agentsys deps <agent_id>`
**Purpose**: Show agent dependencies
**Expected Output**:
```
Agent 1000 Dependencies:
  Depends On:
    - Agent 1002 (logger_service)
    - Agent 1003 (config_manager)

  Dependents (agents that depend on this):
    - Agent 1010 (web_frontend)
    - Agent 1011 (api_gateway)
```

**Validation**:
- All dependencies listed
- Bidirectional relationships shown
- IDs and names correct

---

#### TC-DP-002: `agentsys depgraph`
**Command**: `agentsys depgraph`
**Purpose**: Show full dependency graph
**Expected Output**:
```
Agent Dependency Graph:
======================

1000 (test_agent)
  └─> 1002 (logger_service)
  └─> 1003 (config_manager)

1010 (web_frontend)
  └─> 1000 (test_agent)
  └─> 1011 (api_gateway)

1011 (api_gateway)
  └─> 1002 (logger_service)

Circular Dependencies: None
```

**Validation**:
- All agents included
- Tree structure correct
- Circular dependencies detected

---

### 1.6 Policy Commands

#### TC-PL-001: `agentsys policy`
**Command**: `agentsys policy <agent_id>`
**Purpose**: Show active policy configuration
**Expected Output**:
```
Agent 1000 Active Policy:
  Capabilities:
    - FsBasic (scope: /tmp/*)
    - NetClient (scope: 0.0.0.0/0)

  Resource Limits:
    - CPU Quota: 1,000,000 us
    - Memory: 100 MiB
    - Syscall Rate: 1000/sec

  Recovery Policy:
    - CPU Quota Exceeded: Throttle
    - Memory Exceeded: Kill
    - Syscall Flood: Throttle
    - Crash: Restart
    - Policy Violation: Kill

  Last Updated: 2025-11-16 12:00:00 UTC
```

**Validation**:
- All capabilities shown
- Scopes displayed
- Recovery actions listed

---

#### TC-PL-002: `agentsys policy-update`
**Command**: `agentsys policy-update <agent_id> <patch>`
**Purpose**: Hot-patch agent policy
**Expected Output**:
```
[ASM] Updating policy for agent 1000
[ASM] Policy patch validated
[ASM] Policy updated successfully
[AUDIT] PolicyUpdate: agent=1000, timestamp=12:34:56.789
```

**Validation**:
- Policy updated immediately
- Validation prevents escalation
- Audit log entry created

---

### 1.7 Profiling Commands

#### TC-PR-001: `agentsys profile`
**Command**: `agentsys profile <agent_id>`
**Purpose**: Show performance profile
**Expected Output**:
```
Agent 1000 Performance Profile:

  Operation Metrics:
    Operation      Count  Avg Latency  Max Latency  Success Rate
    -------------  -----  -----------  -----------  ------------
    FS_LIST        120    150 us       500 us       100.0%
    FS_READ        80     200 us       1,200 us     98.8%
    NET_CONNECT    10     5,000 us     15,000 us    100.0%

  Total Operations: 210
  Average Latency: 350 us
  Overall Success Rate: 99.5%
```

**Validation**:
- All operations tracked
- Latency values reasonable
- Success rate calculated correctly

---

#### TC-PR-002: `agentsys profile-reset`
**Command**: `agentsys profile-reset [agent_id]`
**Purpose**: Reset profiling data (all or specific agent)
**Expected Output**:
```
[ASM] Resetting profiling data for agent 1000
[ASM] Profiling data reset successfully
```

**Validation**:
- Metrics cleared
- New data collection starts fresh
- System-wide reset if no agent_id

---

### 1.8 Status & Debug Commands

#### TC-ST-001: `agentsys status`
**Command**: `agentsys status`
**Purpose**: Show overall ASM status
**Expected Output**:
```
Agent Supervision Module Status
================================

Subsystems:
  ✓ Agent Supervisor      (initialized)
  ✓ Telemetry Aggregator  (initialized)
  ✓ Fault Detector        (initialized)
  ✓ Policy Controller     (initialized)
  ✓ Compliance Tracker    (initialized)
  ✓ Resource Monitor      (initialized)
  ✓ Dependency Graph      (initialized)
  ✓ System Profiler       (initialized)
  ✓ Cloud Gateway         (initialized)

Active Agents: 2
Total Spawns: 10
Total Exits: 8
System Health: Healthy
```

**Validation**:
- All subsystems initialized
- Counts match telemetry
- Health status accurate

---

#### TC-ST-002: `agentsys dump`
**Command**: `agentsys dump`
**Purpose**: Dump complete ASM state for debugging
**Expected Output**:
```
[ASM] Dumping complete state...
[ASM] State written to /tmp/asm_dump_20251116_123456.json
[ASM] Size: 4,567 bytes
```

**Validation**:
- File created
- JSON valid
- All state included

---

## Week 2: Lifecycle & Telemetry Tests

**Objective**: Create basic integration tests for agent lifecycle and telemetry

### Test Suite Structure

**Location**: `crates/testing/src/phase9_agentic/asm_supervision_tests/`

```
asm_supervision_tests/
├── mod.rs                      # Suite coordinator
├── asm_lifecycle_tests.rs      # Lifecycle tests
├── asm_telemetry_tests.rs      # Telemetry tests
└── common.rs                   # Shared test utilities
```

---

### 2.1 Lifecycle Integration Tests

#### TC-INT-LC-001: Basic Agent Spawn
**Test**: Spawn single agent and verify registration
**Steps**:
1. Execute: `agentsys spawn 1000 test_agent FsBasic,NetClient`
2. Verify output contains: `Agent 1000 spawned successfully`
3. Execute: `agentsys list`
4. Verify agent 1000 is listed
5. Execute: `agentsys telemetry`
6. Verify `Active Agents: 1`

**Pass Criteria**:
- Agent appears in all subsystems
- Telemetry shows 1 active agent
- Metrics initialized for agent 1000

---

#### TC-INT-LC-002: Basic Agent Exit
**Test**: Spawn and exit agent cleanly
**Steps**:
1. Spawn agent 1000
2. Execute: `agentsys kill 1000`
3. Verify: `Agent 1000 terminated`
4. Execute: `agentsys list`
5. Verify agent 1000 NOT listed
6. Execute: `agentsys telemetry`
7. Verify `Active Agents: 0`, `Total Exits: 1`

**Pass Criteria**:
- Agent removed from supervisor
- Exit count incremented
- No memory leaks

---

#### TC-INT-LC-003: Multiple Agent Spawn/Exit
**Test**: Spawn 10 agents concurrently
**Steps**:
1. Loop: Spawn agents 1000-1009
2. Verify all 10 agents active
3. Loop: Kill agents 1000-1009
4. Verify all agents exited
5. Check telemetry consistency

**Pass Criteria**:
- All 10 agents tracked correctly
- No race conditions
- Telemetry accurate

---

#### TC-INT-LC-004: Agent Crash Detection
**Test**: Detect agent crash vs clean exit
**Steps**:
1. Spawn agent 1000
2. Simulate crash (exit code 139 = SIGSEGV)
3. Verify telemetry shows crash
4. Check `Total Crashes` incremented

**Pass Criteria**:
- Crash detected (exit code != 0)
- Crash count in telemetry
- Event logged correctly

---

#### TC-INT-LC-005: Auto-Restart on Crash
**Test**: Verify automatic restart policy
**Steps**:
1. Spawn agent 1000 with `auto_restart=true`, `max_restarts=3`
2. Trigger crash
3. Verify agent restarted automatically
4. Check restart count = 1
5. Trigger 2 more crashes
6. Verify no more restarts (max exceeded)

**Pass Criteria**:
- Agent restarted up to max_restarts
- Restart count tracked
- Exceeded limit prevents further restarts

---

#### TC-INT-LC-006: Manual Restart
**Test**: Manual restart via command
**Steps**:
1. Spawn agent 1000
2. Execute: `agentsys restart 1000`
3. Verify old PID terminated
4. Verify new PID assigned
5. Check restart count incremented

**Pass Criteria**:
- Agent restarted successfully
- New process spawned
- Metadata preserved

---

### 2.2 Telemetry Integration Tests

#### TC-INT-TM-001: Telemetry Snapshot Consistency
**Test**: Verify telemetry data consistency
**Steps**:
1. Spawn 5 agents
2. Execute various operations
3. Capture telemetry snapshot
4. Verify all counters consistent
5. Compare with /proc export

**Pass Criteria**:
- All counters match actual events
- No missing data
- /proc export matches snapshot

---

#### TC-INT-TM-002: Per-Agent Metrics Tracking
**Test**: Track individual agent metrics
**Steps**:
1. Spawn agent 1000
2. Perform 100 syscalls
3. Execute: `agentsys metrics 1000`
4. Verify syscall count ≥ 100
5. Check CPU time > 0
6. Verify memory usage > 0

**Pass Criteria**:
- All metrics tracked accurately
- Syscall count increments
- Resource usage realistic

---

#### TC-INT-TM-003: System-Wide Aggregation
**Test**: Aggregate metrics across all agents
**Steps**:
1. Spawn 10 agents
2. Each performs different operations
3. Execute: `agentsys telemetry`
4. Verify total syscalls = sum of all agents
5. Verify active agents = 10

**Pass Criteria**:
- System metrics = sum of agent metrics
- No double counting
- Aggregation accurate

---

#### TC-INT-TM-004: Ring Buffer Overflow
**Test**: Verify event ring buffer handles overflow
**Steps**:
1. Generate 2000 events (buffer limit: 1024)
2. Read event buffer
3. Verify oldest events evicted
4. Verify newest 1024 events present

**Pass Criteria**:
- Buffer size capped at 1024
- FIFO eviction
- No memory corruption

---

#### TC-INT-TM-005: Telemetry Export Format
**Test**: Verify /proc export format
**Steps**:
1. Spawn agents with known state
2. Read `/proc/agentsys/telemetry` (if implemented)
3. Parse output
4. Verify all fields present
5. Validate data types

**Pass Criteria**:
- Format parseable
- All expected fields present
- Values match internal state

---

## Week 3: Resource Monitoring & Dependencies

**Objective**: Test resource limit enforcement and dependency management

### 3.1 Resource Monitoring Tests

#### TC-INT-RM-001: CPU Quota Enforcement
**Test**: Enforce CPU quota limits
**Steps**:
1. Spawn agent 1000 with CPU quota: 100,000 us
2. Agent consumes CPU (busy loop)
3. Monitor CPU usage via telemetry
4. Wait for quota exceeded fault
5. Verify throttling action applied

**Pass Criteria**:
- Fault detected when quota exceeded
- Throttling action executed
- Agent continues running (not killed)

---

#### TC-INT-RM-002: Memory Limit Enforcement
**Test**: Enforce memory limits
**Steps**:
1. Spawn agent 1000 with memory limit: 1 MiB
2. Agent allocates 2 MiB (exceeds limit)
3. Verify memory fault detected
4. Verify kill action executed
5. Check agent terminated

**Pass Criteria**:
- Fault detected on memory exceeded
- Agent killed per policy
- Telemetry shows memory fault

---

#### TC-INT-RM-003: Syscall Rate Limiting
**Test**: Enforce syscall rate limits
**Steps**:
1. Spawn agent 1000 with rate: 100 calls/sec
2. Agent makes 1000 calls/sec (flood)
3. Verify syscall flood detected
4. Verify throttling applied
5. Check syscall rate reduced

**Pass Criteria**:
- Flood detected correctly
- Rate limiting effective
- Agent throttled, not killed

---

#### TC-INT-RM-004: Watchdog Timeout
**Test**: Detect unresponsive agents
**Steps**:
1. Spawn agent 1000 with watchdog: 5 seconds
2. Agent goes idle (no operations)
3. Wait 10 seconds
4. Verify unresponsive fault
5. Check restart action

**Pass Criteria**:
- Timeout detected after threshold
- Restart action executed
- Agent recovers

---

#### TC-INT-RM-005: Resource Monitoring Display
**Test**: Display resource usage accurately
**Steps**:
1. Spawn agent with known workload
2. Execute: `agentsys resources 1000`
3. Verify CPU, memory, syscall values
4. Compare with telemetry
5. Validate limits shown

**Pass Criteria**:
- Usage values realistic
- Limits displayed correctly
- Percentages accurate

---

### 3.2 Dependency Management Tests

#### TC-INT-DP-001: Simple Dependency
**Test**: Create single dependency relationship
**Steps**:
1. Spawn agent 1001 (service)
2. Spawn agent 1000 depends on 1001
3. Execute: `agentsys deps 1000`
4. Verify 1001 listed as dependency
5. Execute: `agentsys deps 1001`
6. Verify 1000 listed as dependent

**Pass Criteria**:
- Dependency registered
- Bidirectional tracking
- Graph accurate

---

#### TC-INT-DP-002: Cascade Shutdown
**Test**: Verify cascade shutdown on dependency failure
**Steps**:
1. Create chain: 1000 → 1001 → 1002
2. Kill agent 1002 (root)
3. Verify 1001 terminated (depends on 1002)
4. Verify 1000 terminated (depends on 1001)
5. Check cascade order correct

**Pass Criteria**:
- All dependents terminated
- Correct shutdown order
- No orphaned agents

---

#### TC-INT-DP-003: Circular Dependency Detection
**Test**: Detect and reject circular dependencies
**Steps**:
1. Spawn agent 1000 depends on 1001
2. Spawn agent 1001 depends on 1002
3. Attempt: agent 1002 depends on 1000 (creates cycle)
4. Verify dependency rejected
5. Check error message

**Pass Criteria**:
- Circular dependency detected
- Dependency creation rejected
- Error logged

---

#### TC-INT-DP-004: Multi-Level Dependencies
**Test**: Handle complex dependency graphs
**Steps**:
1. Create tree:
   ```
   1000
   ├─> 1001
   │   ├─> 1003
   │   └─> 1004
   └─> 1002
       └─> 1005
   ```
2. Execute: `agentsys depgraph`
3. Verify tree structure correct
4. Kill 1001
5. Verify 1003, 1004 cascade
6. Verify 1000, 1002, 1005 unaffected

**Pass Criteria**:
- Complex graph handled
- Partial cascade works
- Unrelated agents unaffected

---

#### TC-INT-DP-005: Dependency Graph Export
**Test**: Export full dependency graph
**Steps**:
1. Create complex graph (10 agents, 15 deps)
2. Execute: `agentsys depgraph`
3. Parse output
4. Verify all nodes present
5. Verify all edges correct

**Pass Criteria**:
- Graph complete
- Format readable
- Topology accurate

---

## Week 4: Cloud Gateway & Stress Tests

**Objective**: Test cloud LLM integration and high-load scenarios

### 4.1 Cloud Gateway Tests

#### TC-INT-CG-001: Provider Selection
**Test**: Route requests to correct provider
**Steps**:
1. Configure multi-provider: Claude, GPT-4, Gemini
2. Send request with provider=Claude
3. Verify routed to Claude backend
4. Send request with provider=GPT-4
5. Verify routed to GPT-4 backend

**Pass Criteria**:
- Correct provider selected
- Request routed properly
- Response received

---

#### TC-INT-CG-002: Rate Limiting
**Test**: Enforce per-provider rate limits
**Steps**:
1. Configure Claude: 10 req/min
2. Send 15 requests to Claude in 30 seconds
3. Verify first 10 succeed
4. Verify next 5 rate limited
5. Wait 60 seconds
6. Verify requests resume

**Pass Criteria**:
- Rate limit enforced
- Excess requests rejected
- Limit resets correctly

---

#### TC-INT-CG-003: Fallback on Failure
**Test**: Fallback to secondary provider
**Steps**:
1. Configure: Primary=Claude, Fallback=LocalLLM
2. Simulate Claude outage (timeout)
3. Verify fallback to LocalLLM
4. Verify response from LocalLLM
5. Restore Claude
6. Verify switches back to Claude

**Pass Criteria**:
- Fallback triggered on failure
- Local provider works
- Failover transparent

---

#### TC-INT-CG-004: Load Balancing
**Test**: Balance load across providers
**Steps**:
1. Configure: Claude, GPT-4, Gemini (equal weight)
2. Send 300 requests (no provider specified)
3. Verify ~100 requests per provider
4. Check load distribution

**Pass Criteria**:
- Even distribution (±10%)
- All providers utilized
- No overload

---

#### TC-INT-CG-005: Request Timeout
**Test**: Handle slow/hanging requests
**Steps**:
1. Configure timeout: 5 seconds
2. Send request to slow provider
3. Wait 10 seconds
4. Verify timeout error
5. Verify fallback triggered

**Pass Criteria**:
- Timeout detected
- Request cancelled
- Fallback works

---

### 4.2 Policy & Compliance Tests

#### TC-INT-PL-001: Policy Hot-Patch
**Test**: Update policy without restart
**Steps**:
1. Spawn agent 1000 with FsBasic
2. Execute: `agentsys policy-update 1000 +NetClient`
3. Verify agent gains NetClient capability
4. Test network operation succeeds
5. Execute: `agentsys policy 1000`
6. Verify NetClient listed

**Pass Criteria**:
- Policy updated immediately
- New capability active
- No restart required

---

#### TC-INT-PL-002: Policy Validation
**Test**: Prevent privilege escalation
**Steps**:
1. Spawn agent 1000 (non-admin)
2. Attempt: `agentsys policy-update 1000 +Admin`
3. Verify update rejected
4. Check error: "Privilege escalation not allowed"
5. Verify agent still lacks Admin

**Pass Criteria**:
- Escalation blocked
- Error message clear
- Policy unchanged

---

#### TC-INT-CP-001: EU AI Act Classification
**Test**: Classify agents by risk level
**Steps**:
1. Spawn high-risk agent (Admin, many violations)
2. Spawn minimal-risk agent (FsBasic, no violations)
3. Execute: `agentsys compliance`
4. Verify high-risk agent classified correctly
5. Verify minimal-risk agent classified correctly

**Pass Criteria**:
- Risk levels accurate
- Classification criteria applied
- Report complete

---

#### TC-INT-CP-002: Transparency Scoring
**Test**: Calculate transparency scores
**Steps**:
1. Agent performs 100 operations
2. Agent gets 10 human reviews
3. Execute: `agentsys risk 1000`
4. Verify transparency score calculated
5. Verify score factors shown

**Pass Criteria**:
- Score between 0.0-1.0
- Factors influence score
- Calculation documented

---

### 4.3 Profiling Tests

#### TC-INT-PR-001: Operation Latency Tracking
**Test**: Track per-operation latency
**Steps**:
1. Agent performs FS_LIST 100 times
2. Execute: `agentsys profile 1000`
3. Verify FS_LIST metrics shown
4. Check average latency
5. Check max latency

**Pass Criteria**:
- Latency tracked
- Stats calculated
- Values realistic

---

#### TC-INT-PR-002: Success Rate Calculation
**Test**: Track operation success rates
**Steps**:
1. Agent: 95 successful ops, 5 failures
2. Execute: `agentsys profile 1000`
3. Verify success rate = 95%
4. Verify failure count shown

**Pass Criteria**:
- Success rate accurate
- Failures counted
- Percentage correct

---

### 4.4 Stress Tests

#### TC-STRESS-001: 100 Agent Spawn
**Test**: Handle 100 concurrent agents
**Steps**:
1. Spawn 100 agents (IDs 1000-1099)
2. Verify all agents active
3. Check telemetry: Active Agents = 100
4. Execute operations on all agents
5. Kill all agents
6. Verify cleanup complete

**Pass Criteria**:
- All 100 agents tracked
- No performance degradation
- Memory usage reasonable (<10 KiB overhead per agent)

---

#### TC-STRESS-002: Fault Storm
**Test**: Handle many faults simultaneously
**Steps**:
1. Spawn 50 agents
2. Trigger CPU fault on all agents simultaneously
3. Verify all faults detected
4. Check telemetry fault count
5. Verify no crashes

**Pass Criteria**:
- All faults handled
- System stable
- No data corruption

---

#### TC-STRESS-003: High Syscall Rate
**Test**: Handle high syscall volume
**Steps**:
1. Spawn 10 agents
2. Each performs 10,000 syscalls/sec
3. Total: 100,000 syscalls/sec
4. Monitor for 60 seconds
5. Verify telemetry accurate

**Pass Criteria**:
- Syscall tracking scales
- No counter overflow
- Performance acceptable

---

#### TC-STRESS-004: Dependency Cascade
**Test**: Large-scale cascade shutdown
**Steps**:
1. Create dependency tree: 1 root → 10 L1 → 100 L2
2. Kill root agent
3. Verify all 110 agents terminate
4. Check cascade order correct
5. Measure cascade time

**Pass Criteria**:
- All dependents terminated
- Correct order maintained
- Cascade completes quickly (<1 sec)

---

#### TC-STRESS-005: Memory Leak Test
**Test**: No memory leaks over time
**Steps**:
1. Spawn 100 agents
2. Kill all agents
3. Repeat 100 times (10,000 total spawns)
4. Check kernel heap usage
5. Verify no growth

**Pass Criteria**:
- Memory usage stable
- No heap fragmentation
- All resources freed

---

## Test Environment

### Hardware Requirements

- **QEMU**: aarch64 emulation
- **Memory**: 512 MB RAM for kernel
- **CPU**: 4 virtual cores
- **Storage**: 1 GB virtual disk

### Software Requirements

- **Rust**: 1.75+ (nightly)
- **QEMU**: 8.0+
- **Test Suite**: `crates/testing`
- **Features**: `agentsys` enabled

### Test Execution

```bash
# Run all ASM tests
cargo run -p sis-testing --release -- --phase 9

# Run specific test module
cargo run -p sis-testing --release -- --asm-lifecycle
cargo run -p sis-testing --release -- --asm-telemetry
cargo run -p sis-testing --release -- --asm-resources
cargo run -p sis-testing --release -- --asm-dependencies
cargo run -p sis-testing --release -- --asm-cloud-gateway
cargo run -p sis-testing --release -- --asm-stress

# Run specific test case
cargo run -p sis-testing --release -- --test TC-INT-LC-001
```

---

## Success Criteria

### Week 1 Success Criteria

- [ ] All 22 shell commands implemented
- [ ] Commands produce expected output
- [ ] Help text documented
- [ ] Commands integrated into shell

### Week 2 Success Criteria

- [ ] 6 lifecycle tests passing
- [ ] 5 telemetry tests passing
- [ ] Test suite structure created
- [ ] All tests automated in QEMU

### Week 3 Success Criteria

- [ ] 5 resource monitoring tests passing
- [ ] 5 dependency management tests passing
- [ ] Fault detection validated
- [ ] Cascade shutdown verified

### Week 4 Success Criteria

- [ ] 5 cloud gateway tests passing
- [ ] 4 policy/compliance tests passing
- [ ] 2 profiling tests passing
- [ ] 5 stress tests passing
- [ ] Overall ASM test score: 95%+

### Overall Success Criteria

- [ ] **100% test coverage** for all ASM features
- [ ] **All integration tests passing** in QEMU
- [ ] **No regressions** in existing Phase 9 tests
- [ ] **Performance targets** met (memory, latency)
- [ ] **Documentation complete** (test plan, results)

---

## Risk Mitigation

### Risk 1: Shell Commands Too Complex

**Mitigation**: Start with read-only commands, add write operations incrementally

### Risk 2: QEMU Test Flakiness

**Mitigation**: Add retries, longer timeouts, synchronization points

### Risk 3: Integration Test Dependency Order

**Mitigation**: Tests clean up state, use unique agent IDs, run in isolation

### Risk 4: Cloud Gateway Requires Real APIs

**Mitigation**: Mock providers for testing, use LocalLLM fallback

### Risk 5: Stress Tests Cause Instability

**Mitigation**: Gradual load increase, monitor kernel state, abort on errors

---

## Appendix A: Command Quick Reference

```bash
# Lifecycle
agentsys spawn <id> <name> <caps>
agentsys kill <id>
agentsys restart <id>
agentsys list

# Telemetry
agentsys telemetry
agentsys metrics <id>

# Compliance
agentsys compliance
agentsys risk <id>

# Resources
agentsys resources <id>
agentsys limits <id>

# Dependencies
agentsys deps <id>
agentsys depgraph

# Policy
agentsys policy <id>
agentsys policy-update <id> <patch>

# Profiling
agentsys profile <id>
agentsys profile-reset [id]

# Status
agentsys status
agentsys dump
```

---

## Appendix B: Test Data Sets

### Standard Test Agents

- **Agent 1000**: Minimal capabilities (FsBasic), clean record
- **Agent 1001**: Network capabilities (NetClient), moderate activity
- **Agent 1002**: Admin capabilities, high risk
- **Agent 1003-1099**: Stress test agents (variable configurations)

### Test Capability Sets

- **Minimal**: FsBasic
- **Standard**: FsBasic, NetClient
- **Extended**: FsBasic, NetClient, AudioControl, DocBasic
- **Admin**: All capabilities including Admin

---

**End of Test Plan**
