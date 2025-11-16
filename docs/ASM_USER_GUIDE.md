# Agent Supervision Module - User Guide

**Version**: 1.0.0
**Target Audience**: Kernel developers, system administrators, agent developers
**Last Updated**: 2025-11-16

---

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Understanding ASM Components](#understanding-asm-components)
4. [Working with Agents](#working-with-agents)
5. [Monitoring and Telemetry](#monitoring-and-telemetry)
6. [Using Shell Commands](#using-shell-commands)
7. [Cloud Gateway for LLM Requests](#cloud-gateway-for-llm-requests)
8. [Using Syscall Interface](#using-syscall-interface)
9. [Policy Management](#policy-management)
10. [Fault Handling](#fault-handling)
11. [Best Practices](#best-practices)
12. [Troubleshooting](#troubleshooting)
13. [FAQ](#faq)

---

## Introduction

### What is ASM?

The Agent Supervision Module (ASM) is a kernel-resident service that provides comprehensive lifecycle management for all agents running in the SIS kernel. Think of it as a "guardian angel" for your agents - it watches over them, helps them recover from failures, and ensures they play nicely with system resources.

### Why ASM?

Traditional operating systems treat all processes equally. ASM recognizes that agent processes have special characteristics and needs:

- **Autonomous Operation**: Agents run without constant human supervision
- **Resource Intensive**: LLM inference and decision-making consume significant resources
- **Fault Tolerance**: Agents should gracefully recover from transient failures
- **Policy Compliance**: Agents must adhere to security and resource policies
- **Observability**: Agent behavior must be visible for debugging and auditing

ASM provides specialized infrastructure to handle these unique requirements.

### Key Benefits

1. **Automatic Recovery**: Agents can automatically restart after crashes
2. **Resource Protection**: System resources are protected from runaway agents
3. **Comprehensive Monitoring**: Detailed telemetry for all agent activities
4. **Dynamic Policies**: Update agent permissions without restarting
5. **Compliance Ready**: Built-in audit trails for regulatory compliance

---

## Getting Started

### Prerequisites

- SIS Kernel with AgentSys enabled
- Basic understanding of agent capabilities and policies
- Familiarity with kernel modules and /proc filesystem

### Enabling ASM

ASM is automatically initialized when AgentSys starts. No additional configuration is needed for basic operation.

To verify ASM is running:

```bash
# Check kernel log
dmesg | grep ASM

# Expected output:
[ASM] Agent Supervision Module initialized
```

### First Steps

1. **Verify Installation**
   ```bash
   cat /proc/agentsys/status
   ```

2. **Spawn a Test Agent**
   ```rust
   let spec = AgentSpec::new(100, "test_agent".to_string())
       .with_capability(Capability::FsBasic);

   let pid = spawn_agent(spec)?;
   ```

3. **Monitor Activity**
   ```bash
   watch -n 1 'cat /proc/agentsys/status'
   ```

---

## Understanding ASM Components

### The Four Pillars of ASM

ASM consists of four main components, each with a specific responsibility:

#### 1. AgentSupervisor 👁️

**Role**: Lifecycle coordinator

The supervisor is like a register at a hotel - it keeps track of all agents, knows when they arrive (spawn), when they leave (exit), and maintains their records (metadata).

**Key Functions**:
- Tracks all active agents
- Maintains agent metadata
- Coordinates recovery actions
- Notifies other components of lifecycle events

#### 2. TelemetryAggregator 📊

**Role**: Metrics collector

The telemetry aggregator is your dashboard - it collects statistics about what agents are doing and how they're performing.

**Key Metrics**:
- How many times an agent has spawned/exited
- Resource usage (CPU, memory)
- Fault history
- Recent events

#### 3. FaultDetector 🔍

**Role**: Health monitor

The fault detector is like a smoke alarm - it continuously monitors for problems and alerts when something goes wrong.

**What It Watches**:
- CPU usage exceeding quotas
- Memory consumption
- Syscall flood attacks
- Watchdog timeouts (unresponsive agents)
- Policy violations

#### 4. PolicyController 📋

**Role**: Permission manager

The policy controller manages what each agent is allowed to do, and can update permissions on-the-fly.

**Capabilities**:
- Dynamic capability grants/revokes
- Scope restrictions (e.g., file path limits)
- Auto-restart configuration
- Compliance reporting

---

## Working with Agents

### Agent Lifecycle

```
┌─────────┐
│ SPAWN   │  ← Agent process created
└────┬────┘
     │
     ▼
┌─────────┐
│ ACTIVE  │  ← Agent running normally
└────┬────┘
     │
     ├──► FAULT ──► RECOVERY ──┐
     │                         │
     │                         ▼
     │                    ┌─────────┐
     │                    │RESTARTED│
     │                    └────┬────┘
     │                         │
     ▼                         ▼
┌─────────┐             ┌──────────┐
│  EXIT   │  ← Normal   │  FAILED  │
└─────────┘    exit     └──────────┘
```

### Creating an Agent

```rust
use agent_sys::supervisor::{AgentSpec, AgentSupervisor};
use security::agent_policy::{Capability, Scope};

// Create agent specification
let spec = AgentSpec::new(100, "my_agent".to_string())
    .with_capability(Capability::FsBasic)
    .with_capability(Capability::AudioControl)
    .with_scope(Scope::with_path("/tmp/agent_workspace/"))
    .with_auto_restart(3);  // Restart up to 3 times

// Spawn the agent (this would be done by process manager)
let pid = spawn_agent_process(spec)?;

// ASM automatically tracks it!
```

### Configuring Auto-Restart

Auto-restart allows agents to automatically recover from crashes:

```rust
// Enable auto-restart with 5 attempts
let spec = AgentSpec::new(101, "resilient_agent".to_string())
    .with_auto_restart(5);
```

**How It Works**:
1. Agent crashes (non-zero exit code)
2. ASM checks if auto-restart is enabled
3. If restart_count < max_restarts, agent is respawned
4. Otherwise, agent is marked as failed

**Best Practice**: Set max_restarts based on criticality:
- Critical agents: 5-10 restarts
- Normal agents: 2-3 restarts
- Experimental agents: 1 restart or disabled

---

## Monitoring and Telemetry

### Reading Telemetry

#### Via /proc Filesystem

The easiest way to monitor ASM is through `/proc/agentsys/status`:

```bash
$ cat /proc/agentsys/status

Agent Supervision Module - Telemetry Status
===========================================

System Metrics:
  Total Spawns:    42
  Total Exits:     38
  Total Faults:    5
  Total Restarts:  3
  Active Agents:   4
  Total Syscalls:  1234567

Per-Agent Metrics:
  ID    Spawns Exits  Faults CPU(us)   Mem(B)
  ----  ------ -----  ------ --------  -------
  100   1      0      0      125000    4096000
  101   3      2      2      89000     2048000
  102   1      0      1      45000     1024000
  103   1      0      0      67000     3072000
```

#### Via Kernel API

For programmatic access:

```rust
use agent_sys::supervisor::TELEMETRY;

// Get snapshot of all telemetry
let snapshot = TELEMETRY.lock().as_ref().unwrap().snapshot();

println!("Active Agents: {}", snapshot.system_metrics.active_agents);

// Get specific agent metrics
if let Some(metrics) = snapshot.agent_metrics.get(&100) {
    println!("Agent 100:");
    println!("  Spawn count: {}", metrics.spawn_count);
    println!("  Fault count: {}", metrics.fault_count);
}
```

### Understanding Metrics

#### Spawn Count vs. Exit Count

- **Spawn Count**: How many times the agent has been started
- **Exit Count**: How many times the agent has terminated

If `spawn_count > exit_count + 1`, the agent has been restarted.

#### Fault Count

Number of times the agent triggered fault detection:
- Resource limit exceeded
- Policy violation
- Crash/signal

**Normal**: 0 faults
**Warning**: 1-5 faults (investigate)
**Critical**: >5 faults (agent misbehaving)

#### CPU Time

Cumulative CPU time in microseconds. Use this to:
- Identify CPU-intensive agents
- Detect performance regressions
- Plan resource allocation

---

## Using Shell Commands

ASM provides built-in shell commands for interactive monitoring and management.

### Available Commands

#### `asmstatus` - View System Status

Display overall ASM telemetry:

```bash
$ asmstatus

Agent Supervision Module - System Status
========================================

System Metrics:
  Total Spawns:     15
  Total Exits:      12
  Total Crashes:    2
  Total Faults:     3
  Total Restarts:   2
  Active Agents:    3
  Total Syscalls:   45678

Recent Events (last 10):
  [12345678] Agent 103 exited (code 0)
  [12345670] Agent 102 spawned
  [12345665] Agent 101 fault: CPU quota exceeded
```

#### `asmlist` - List Active Agents

Show all currently running agents:

```bash
$ asmlist

Active Agents:
ID    PID   Name              Capabilities        Restarts
----  ----  ----------------  ------------------  --------
100   42    fs_monitor        FsBasic             0
101   43    net_agent         NetClient           2
102   44    audio_ctrl        AudioControl        0
```

#### `asminfo <agent_id>` - Agent Details

Get detailed information about a specific agent:

```bash
$ asminfo 100

Agent Information:
==================
Agent ID:        100
PID:             42
Name:            fs_monitor
State:           Active
Capabilities:    FsBasic, FsExtended
Auto-Restart:    Yes (max 3)
Restart Count:   0

Telemetry:
  Spawn Count:   1
  Exit Count:    0
  Fault Count:   0
  CPU Time:      125000 μs
  Memory:        4096000 bytes

Last Activity:   2 seconds ago
```

#### `asmpolicy <agent_id>` - View Agent Policy

Display the current policy for an agent:

```bash
$ asmpolicy 100

Policy for Agent 100:
====================
Capabilities:
  - FsBasic
  - FsExtended

Scope:
  Path: /tmp/agent_workspace/

Auto-Restart: Yes
Max Restarts: 3
Current Restarts: 0

Resource Limits:
  CPU Quota:     1000000 μs
  Memory Limit:  104857600 bytes
  Syscall Rate:  1000/sec
```

### Shell Command Best Practices

1. **Monitoring Loop**: Use `watch` for continuous monitoring:
   ```bash
   watch -n 2 asmstatus
   ```

2. **Quick Health Check**: Check for unhealthy agents:
   ```bash
   asmlist | awk '{if ($6 > 2) print "Agent " $1 " restarted " $6 " times"}'
   ```

3. **Audit Trail**: Log all agent info periodically:
   ```bash
   for id in $(asmlist | tail -n +3 | awk '{print $1}'); do
       asminfo $id >> /var/log/asm_audit.log
   done
   ```

---

## Cloud Gateway for LLM Requests

The Cloud Gateway provides intelligent multi-provider routing for LLM API requests with automatic fallback, rate limiting, and comprehensive monitoring.

### Architecture

```
Agent Process → syscall(503) → Cloud Gateway
                                     ↓
                         [Rate Limit Check]
                                     ↓
                         [Provider Selection]
                                     ↓
           ┌──────────────────┬──────────────┬──────────────┐
           ▼                  ▼              ▼              ▼
       Claude API         GPT-4 API     Gemini API    Local Fallback
           │                  │              │              │
           └──────────────────┴──────────────┴──────────────┘
                                     ↓
                         [Response / Fallback]
                                     ↓
                             Agent Process
```

### Key Features

1. **Multi-Provider Support**: Claude, GPT-4, Gemini, Local Fallback
2. **Intelligent Fallback**: Automatic failover on errors/timeouts
3. **Per-Agent Rate Limiting**: Token bucket rate limiting
4. **Cost Optimization**: Configurable provider selection policies
5. **Comprehensive Metrics**: Track usage, costs, and performance

### Making LLM Requests

#### C Example

```c
#include <syscall.h>
#include <string.h>
#include <stdio.h>

int main() {
    // Build request JSON
    const char *request =
        "{\"agent_id\":100,"
        "\"prompt\":\"Explain kernel memory management\","
        "\"max_tokens\":500,"
        "\"temperature\":0.7}";

    char response[8192];

    // Make syscall 503
    ssize_t ret = syscall(503,
        request, strlen(request),
        response, sizeof(response)
    );

    if (ret > 0) {
        printf("LLM Response:\n%.*s\n", (int)ret, response);
    } else {
        printf("Error: %ld\n", ret);
    }

    return 0;
}
```

#### Python Example

```python
import ctypes
import json

libc = ctypes.CDLL(None)

def llm_request(agent_id, prompt, max_tokens=1000, temperature=0.7):
    """Make an LLM request via Cloud Gateway"""

    req = {
        "agent_id": agent_id,
        "prompt": prompt,
        "max_tokens": max_tokens,
        "temperature": temperature
    }

    req_json = json.dumps(req).encode()
    req_buf = ctypes.create_string_buffer(req_json)

    resp_buf = ctypes.create_string_buffer(8192)

    ret = libc.syscall(503,
        req_buf, len(req_json),
        resp_buf, len(resp_buf)
    )

    if ret > 0:
        return json.loads(resp_buf.value.decode())
    else:
        raise OSError(-ret, "LLM request failed")

# Usage
response = llm_request(100, "What is an operating system kernel?")
print(f"Provider: {response['provider']}")
print(f"Response: {response['text']}")
print(f"Tokens used: {response['tokens_used']}")
```

### Fallback Policies

The Cloud Gateway supports multiple fallback policies:

#### 1. Cost-Optimized (Default)

Tries providers in order of cost (cheapest first):
```
Local → GPT-4 → Claude → Gemini
```

#### 2. Reliability-Optimized

Tries providers in order of reliability:
```
Claude → GPT-4 → Gemini → Local
```

#### 3. Local-Only

Only use local fallback (no cloud API calls):
```
Local only
```

#### 4. Explicit Chain

Specify exact provider order:
```
Gemini → Claude → Local
```

### Rate Limiting

Each agent has an independent rate limiter using the token bucket algorithm.

**Default Limits**:
- Burst capacity: 30 requests
- Refill rate: 10 requests/second

**Check Your Limits** (via Cloud Gateway metrics):
```bash
# View rate limit status
cat /proc/agentsys/gateway_status
```

**Rate Limit Errors**:
When rate limited, syscall 503 returns `-EAGAIN` (errno 11). Wait and retry.

### Request Format

**JSON Schema**:
```json
{
  "agent_id": 100,           // Required: Your agent ID
  "prompt": "Your prompt",   // Required: The prompt text
  "max_tokens": 1000,        // Optional: Max response tokens (default: 1000)
  "temperature": 0.7,        // Optional: Temperature 0.0-1.0 (default: 0.7)
  "preferred_provider": "claude",  // Optional: "claude", "gpt4", "gemini", "local"
  "system_message": "...",   // Optional: System message
  "timeout_ms": 30000        // Optional: Request timeout (default: 30000)
}
```

### Response Format

**JSON Schema**:
```json
{
  "provider": "claude",      // Provider that fulfilled request
  "text": "Response...",     // The response text
  "tokens_used": 450,        // Tokens consumed
  "duration_us": 123456,     // Request duration in microseconds
  "was_fallback": false      // True if fallback was used
}
```

### Error Handling

| Error Code | Errno | Meaning | Solution |
|------------|-------|---------|----------|
| `-EAGAIN` | 11 | Rate limit exceeded | Wait and retry |
| `-EPERM` | 1 | Permission denied | Check capabilities |
| `-ETIMEDOUT` | 110 | Request timeout | Retry or increase timeout |
| `-EIO` | 5 | All providers failed | Check network/API keys |
| `-EINVAL` | 22 | Invalid JSON | Fix request format |
| `-EFAULT` | 14 | Bad pointer | Check buffer addresses |

### Monitoring Gateway Performance

View Cloud Gateway metrics:

```bash
$ cat /proc/agentsys/cloud_gateway

Cloud Gateway Metrics:
====================
Total Requests:       1234
Successful:           1198
Failed:               36
Rate Limited:         15
Fallback Used:        42

Provider Success Rates:
  Claude:   890/900  (98.9%)
  GPT-4:    250/270  (92.6%)
  Gemini:   58/64    (90.6%)
  Local:    42/42    (100%)

Average Response Time: 245ms
```

### Best Practices

1. **Always Check Return Value**: Handle errors appropriately
   ```c
   if (ret < 0) {
       if (ret == -EAGAIN) {
           // Rate limited, wait and retry
           sleep(1);
           ret = syscall(503, ...);
       }
   }
   ```

2. **Use Appropriate Timeouts**: Set `timeout_ms` based on prompt complexity
   ```json
   {
       "prompt": "Complex multi-step task...",
       "timeout_ms": 60000  // 60 seconds
   }
   ```

3. **Specify Preferred Provider**: For consistency
   ```json
   {
       "preferred_provider": "claude",
       "prompt": "..."
   }
   ```

4. **Monitor Token Usage**: Track costs via metrics
   ```python
   response = llm_request(100, prompt)
   total_tokens += response['tokens_used']
   ```

5. **Handle Fallbacks Gracefully**: Check `was_fallback` flag
   ```python
   if response['was_fallback']:
       print("Warning: Cloud providers unavailable, using fallback")
   ```

### Security Considerations

1. **API Keys**: Stored securely in kernel memory (not in userspace)
2. **Rate Limiting**: Prevents abuse and DoS attacks
3. **Capability Checks**: Only agents with `LLM_ACCESS` capability can use
4. **Audit Trail**: All LLM requests logged for compliance

---

## Using Syscall Interface

For programmatic access from userspace applications, ASM provides syscalls.

### Available Syscalls

#### Syscall 500: Get Telemetry

Retrieve telemetry data as JSON:

```c
#include <syscall.h>
#include <string.h>

char buffer[8192];
ssize_t len = syscall(500, buffer, sizeof(buffer));

if (len > 0) {
    // Parse JSON telemetry
    printf("Telemetry: %.*s\n", (int)len, buffer);
}
```

**Returns**: JSON-formatted telemetry snapshot, or negative errno on error.

**Buffer Format**:
```json
{
  "system": {
    "total_agents_spawned": 15,
    "total_agents_exited": 12,
    "active_agents": 3,
    "total_faults": 5
  },
  "agents": {
    "100": {
      "spawn_count": 1,
      "exit_count": 0,
      "fault_count": 0,
      "cpu_time_us": 125000,
      "memory_bytes": 4096000
    }
  },
  "recent_events": [...]
}
```

#### Syscall 501: Update Agent Policy

Dynamically update an agent's policy:

```c
// Add a capability
syscall(501, agent_id, 1 /* ADD_CAPABILITY */, capability_value);

// Remove a capability
syscall(501, agent_id, 2 /* REMOVE_CAPABILITY */, capability_value);

// Update scope
syscall(501, agent_id, 3 /* UPDATE_SCOPE */, scope_value);

// Set auto-restart
syscall(501, agent_id, 4 /* SET_AUTO_RESTART */, max_restarts);
```

**Patch Types**:
- `1`: Add capability
- `2`: Remove capability
- `3`: Update scope
- `4`: Set auto-restart limit

**Returns**: 0 on success, negative errno on error.

**Error Codes**:
- `-EPERM` (1): Privilege escalation denied
- `-EINVAL` (22): Invalid patch type or argument
- `-ESRCH` (3): Agent not found

#### Syscall 502: Get Agent Info

Retrieve detailed information about a specific agent:

```c
char info_buffer[4096];
ssize_t len = syscall(502, agent_id, info_buffer, sizeof(info_buffer));

if (len > 0) {
    // Parse JSON agent info
    printf("Agent Info: %.*s\n", (int)len, info_buffer);
}
```

**Returns**: JSON-formatted agent metadata, or negative errno on error.

**Buffer Format**:
```json
{
  "agent_id": 100,
  "pid": 42,
  "name": "my_agent",
  "capabilities": ["FsBasic", "NetClient"],
  "auto_restart": true,
  "max_restarts": 3,
  "restart_count": 0,
  "spawn_count": 1,
  "fault_count": 0
}
```

### Syscall Usage Examples

#### Python Wrapper

```python
import ctypes
import json

libc = ctypes.CDLL(None)

def get_asm_telemetry():
    """Get ASM telemetry as dict"""
    buf = ctypes.create_string_buffer(8192)
    ret = libc.syscall(500, buf, len(buf))
    if ret < 0:
        raise OSError(-ret, "syscall failed")
    return json.loads(buf.value.decode())

def update_agent_policy(agent_id, patch_type, arg):
    """Update agent policy"""
    ret = libc.syscall(501, agent_id, patch_type, arg)
    if ret < 0:
        raise OSError(-ret, "syscall failed")
    return ret

# Usage
telemetry = get_asm_telemetry()
print(f"Active agents: {telemetry['system']['active_agents']}")

# Add capability (example)
update_agent_policy(100, 1, 0x02)  # Add FsBasic (0x02)
```

#### C Monitoring Tool

```c
#include <stdio.h>
#include <syscall.h>
#include <unistd.h>

int main() {
    char buf[8192];

    while (1) {
        ssize_t len = syscall(500, buf, sizeof(buf));

        if (len > 0) {
            // Simple parsing - count active agents
            // (In production, use proper JSON parser)
            printf("\033[2J\033[H");  // Clear screen
            printf("ASM Telemetry:\n%.*s\n", (int)len, buf);
        }

        sleep(2);
    }

    return 0;
}
```

### Security Considerations

1. **Buffer Size**: Always provide adequate buffer size (8KB+ recommended)
2. **Validation**: Validate all inputs before syscall invocation
3. **Permissions**: Policy updates may require elevated privileges
4. **Rate Limiting**: Don't spam syscalls - cache telemetry when possible

---

## Policy Management

### Understanding Policies

Each agent has a policy that defines:
1. **Capabilities**: What operations the agent can perform
2. **Scope**: Restrictions on those operations
3. **Limits**: Resource quotas
4. **Behavior**: Auto-restart, recovery actions

### Updating Policies Dynamically

One of ASM's most powerful features is hot-patching policies:

```rust
use agent_sys::supervisor::{POLICY_CONTROLLER, PolicyPatch};

// Grant file access to agent 100
let patch = PolicyPatch::AddCapability(Capability::FsBasic);
POLICY_CONTROLLER
    .lock()
    .as_mut()
    .unwrap()
    .update_policy(100, patch)?;

// Restrict to specific directory
let patch = PolicyPatch::UpdateScope(
    Scope::with_path("/tmp/safe_zone/")
);
POLICY_CONTROLLER
    .lock()
    .as_mut()
    .unwrap()
    .update_policy(100, patch)?;
```

### Policy Validation

ASM validates all policy changes to prevent privilege escalation:

```rust
// ❌ This will FAIL - cannot grant Admin capability
let patch = PolicyPatch::AddCapability(Capability::Admin);
let result = update_policy(100, patch);
assert_eq!(result, Err(PolicyError::PrivilegeEscalation));

// ✅ This succeeds - normal capability
let patch = PolicyPatch::AddCapability(Capability::FsBasic);
let result = update_policy(100, patch);
assert!(result.is_ok());
```

### Compliance Reporting

Generate compliance reports for auditing:

```rust
let report = POLICY_CONTROLLER
    .lock()
    .as_ref()
    .unwrap()
    .export_compliance();

for entry in report.agents {
    println!("Agent {}: {} capabilities, {} violations",
        entry.agent_id,
        entry.capabilities.len(),
        entry.violations.len()
    );
}
```

---

## Fault Handling

### Fault Types

| Fault | Trigger | Default Action |
|-------|---------|----------------|
| CPU Quota | CPU time > quota | Throttle |
| Memory Limit | Memory > limit | Kill |
| Syscall Flood | Syscall rate > limit | Throttle |
| Crash | Fatal signal received | Restart |
| Capability Violation | Unauthorized operation | Kill |
| Unresponsive | Watchdog timeout | Restart |

### Recovery Policies

Configure how ASM responds to faults:

```rust
use agent_sys::supervisor::fault::{RecoveryPolicy, FaultAction};

// Permissive policy - just log faults
let policy = RecoveryPolicy::permissive();

// Strict policy - kill on any fault
let policy = RecoveryPolicy::strict();

// Custom policy
let policy = RecoveryPolicy {
    cpu_quota_action: FaultAction::Throttle,
    memory_action: FaultAction::Kill,
    syscall_flood_action: FaultAction::Throttle,
    crash_action: FaultAction::Restart,
    capability_violation_action: FaultAction::Kill,
    unresponsive_action: FaultAction::Restart,
    policy_violation_action: FaultAction::Kill,
};

FAULT_DETECTOR.lock().as_mut().unwrap()
    .set_recovery_policy(policy);
```

### Resource Limits

Set conservative limits to protect the system:

```rust
use agent_sys::supervisor::fault::ResourceLimits;

let limits = ResourceLimits {
    cpu_quota_us: Some(1_000_000),  // 1 second per window
    memory_limit_bytes: Some(100 * 1024 * 1024),  // 100 MB
    syscall_rate_limit: Some(1000),  // 1000 syscalls/sec
    watchdog_timeout_us: Some(30_000_000),  // 30 seconds
};

FAULT_DETECTOR.lock().as_mut().unwrap()
    .set_default_limits(limits);
```

### Handling Crashes

When an agent crashes, ASM automatically:

1. Records the crash in telemetry
2. Notifies lifecycle listeners
3. Checks auto-restart policy
4. Either restarts or marks as failed

Monitor crash patterns:

```bash
$ cat /proc/agentsys/status | grep "Agent 100"
Agent 100: spawns=5 exits=4 faults=4

# This agent has crashed 4 times and been restarted
```

---

## Best Practices

### 1. Configure Auto-Restart Appropriately

**Do**:
- Enable for production agents
- Set max_restarts = 3-5 for normal agents
- Use higher limits for critical services

**Don't**:
- Enable for debug/test agents
- Set unlimited restarts (prevents failure detection)
- Ignore restart patterns (indicates deeper issues)

### 2. Monitor Telemetry Regularly

Set up automated monitoring:

```bash
#!/bin/bash
# Alert if any agent has >5 faults

FAULTS=$(cat /proc/agentsys/status | awk '/Faults/ {if ($6 > 5) print $1}')
if [ -n "$FAULTS" ]; then
    echo "WARNING: Agents with high fault counts: $FAULTS"
fi
```

### 3. Use Scoped Capabilities

Always restrict agent access:

```rust
// ❌ Bad - unrestricted access
let spec = AgentSpec::new(100, "agent".to_string())
    .with_capability(Capability::FsBasic);

// ✅ Good - scoped to specific directory
let spec = AgentSpec::new(100, "agent".to_string())
    .with_capability(Capability::FsBasic)
    .with_scope(Scope::with_path("/tmp/agent_data/"));
```

### 4. Review Compliance Reports

Periodically audit agent behavior:

```rust
// Monthly compliance audit
let report = POLICY_CONTROLLER.lock().as_ref().unwrap()
    .export_eu_ai_act_report();

// Check for violations
for entry in report.agents {
    if !entry.violations.is_empty() {
        println!("Agent {} has {} violations - review needed",
            entry.agent_id, entry.violations.len());
    }
}
```

### 5. Test Fault Recovery

Validate that your agents recover properly:

```rust
#[test]
fn test_agent_recovery() {
    let spec = AgentSpec::new(999, "test".to_string())
        .with_auto_restart(2);

    spawn_agent(spec)?;

    // Simulate crash
    kill(pid, SIGKILL);

    // Wait for restart
    sleep(Duration::from_millis(100));

    // Verify agent restarted
    let metadata = AGENT_SUPERVISOR.lock().as_ref().unwrap()
        .get_agent(999).unwrap();
    assert_eq!(metadata.restart_count, 1);
}
```

---

## Troubleshooting

### Agent Won't Start

**Symptom**: Agent spawns but immediately exits

**Diagnosis**:
```bash
cat /proc/agentsys/status | grep "Agent <ID>"
```

**Common Causes**:
1. Missing required capabilities
2. Invalid scope configuration
3. Resource limits too restrictive

**Solution**: Review agent specification and ensure all required capabilities are granted.

### Agent Keeps Restarting

**Symptom**: Spawn count >> exit count

**Diagnosis**:
```rust
let metrics = TELEMETRY.lock().as_ref().unwrap()
    .get_agent_metrics(agent_id).unwrap();

println!("Recent faults: {:?}", metrics.recent_faults);
```

**Common Causes**:
1. Bug in agent code (crash loop)
2. Resource limits too restrictive
3. Missing dependencies

**Solution**:
- Review recent fault history
- Check agent logs
- Consider disabling auto-restart during debugging

### High Fault Count

**Symptom**: Agent has many faults but is still running

**Diagnosis**: Check fault types in telemetry

**Common Causes**:
1. Resource-intensive workload
2. Inefficient agent implementation
3. Too-aggressive limits

**Solution**:
- Profile agent performance
- Adjust resource limits
- Optimize agent code

### Memory Leaks

**Symptom**: Agent memory usage grows over time

**Diagnosis**:
```bash
watch -n 5 'cat /proc/agentsys/status | grep "Agent <ID>"'
```

**Solution**:
- Review agent's memory management
- Set memory limits to prevent system exhaustion
- Consider periodic restarts for leaky agents

---

## FAQ

### Q: Can I disable ASM?

**A**: ASM is integral to agent management and cannot be disabled. However, you can use permissive policies for testing.

### Q: How much overhead does ASM add?

**A**: Minimal - approximately 50μs per spawn and 2μs per telemetry event. See [Performance Characteristics](ASM_API_REFERENCE.md#performance-characteristics).

### Q: Can agents bypass ASM supervision?

**A**: No. All lifecycle events are handled through kernel hooks that agents cannot bypass.

### Q: What happens if an agent exceeds max restarts?

**A**: The agent is marked as failed and won't be restarted again. You must manually spawn a new instance.

### Q: Can I update policies for running agents?

**A**: Yes! This is one of ASM's key features. Use `PolicyController::update_policy()`.

### Q: Are policy changes audited?

**A**: Yes. All policy changes are recorded in the agent's audit trail and included in compliance reports.

### Q: How do I debug agent failures?

**A**:
1. Check `/proc/agentsys/status` for fault history
2. Review telemetry for resource usage patterns
3. Check kernel logs for ASM messages
4. Use the fault detector to identify specific issues

### Q: Can I have different recovery policies for different agents?

**A**: Currently, recovery policy is global. Per-agent policies are planned for Milestone 4.

### Q: How long are telemetry events retained?

**A**: The ring buffer holds the last 1024 events. System metrics are cumulative since boot.

### Q: What compliance standards does ASM support?

**A**: ASM provides audit trails suitable for EU AI Act compliance. Additional standards can be supported via custom reporting.

---

## Next Steps

- **Advanced Usage**: See [ASM API Reference](ASM_API_REFERENCE.md)
- **Implementation Details**: Review [ASM Implementation Plan](plans/AGENT_SUPERVISION_MODULE_PLAN.md)
- **Source Code**: Browse `crates/kernel/src/agent_sys/supervisor/`

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-16
**Maintained By**: SIS Kernel Team
