# Agent Supervision Module (ASM) API Reference

**Version**: 1.0.0
**Status**: Milestone 0 Complete
**Last Updated**: 2025-11-16

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Core Components](#core-components)
4. [API Reference](#api-reference)
5. [Usage Examples](#usage-examples)
6. [Integration Guide](#integration-guide)
7. [Performance Characteristics](#performance-characteristics)
8. [Security Model](#security-model)

---

## Overview

The Agent Supervision Module (ASM) provides comprehensive lifecycle management, fault detection, and recovery for all agents in the SIS kernel. ASM is implemented as a collection of kernel-resident services rather than a userland super-agent, ensuring robustness, security, and performance.

### Key Features

- **Lifecycle Management**: Automatic tracking of agent spawn, exit, and crash events
- **Fault Detection**: Real-time monitoring of resource usage and policy violations
- **Auto-Recovery**: Configurable restart policies with exponential backoff
- **Telemetry**: Comprehensive metrics collection and /proc export
- **Policy Control**: Dynamic policy updates with hot-patching support
- **Compliance**: EU AI Act compliance reporting

### Design Principles

1. **No Userland Super-Agent**: All supervision logic resides in kernel space
2. **Distributed Responsibilities**: Each subsystem handles its specific domain
3. **Least Privilege**: Services operate with minimal required capabilities
4. **Fail-Safe**: Kernel services survive agent failures
5. **Observable**: All actions are logged and visible via /proc
6. **Policy-Driven**: Runtime behavior controlled by policy engine

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Userland (Unprivileged)                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Agent 1  │  │ Agent 2  │  │ Agent N  │  │  agentctl│  │
│  │ (PID 10) │  │ (PID 11) │  │ (PID 50) │  │  (CLI)   │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
└───────┼─────────────┼──────────────┼─────────────┼─────────┘
        │ syscall     │ syscall      │ syscall     │ read /proc
        ▼             ▼              ▼             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Kernel Services                           │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Agent Supervision Module               │  │
│  ├─────────────────────────────────────────────────────┤  │
│  │  ┌────────────────┐  ┌────────────────────────┐    │  │
│  │  │ Agent          │  │ Policy Controller      │    │  │
│  │  │ Supervisor     │  │ - Dynamic policies     │    │  │
│  │  │ - Lifecycle    │  │ - Hot-patch rules      │    │  │
│  │  │ - Fault detect │  │ - Compliance export    │    │  │
│  │  │ - Recovery     │  │ - Capability mgmt      │    │  │
│  │  └────────────────┘  └────────────────────────┘    │  │
│  │                                                     │  │
│  │  ┌────────────────┐  ┌────────────────────────┐    │  │
│  │  │ Telemetry      │  │ Fault Detector         │    │  │
│  │  │ Aggregator     │  │ - Resource limits      │    │  │
│  │  │ - Metrics      │  │ - Watchdog             │    │  │
│  │  │ - /proc export │  │ - Recovery policy      │    │  │
│  │  │ - Audit logs   │  │ - Health checks        │    │  │
│  │  └────────────────┘  └────────────────────────┘    │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │         Existing Kernel Infrastructure              │  │
│  ├──────────┬──────────┬──────────┬──────────┬─────────┤  │
│  │ AgentSys │ Process  │ Network  │   VFS    │  Audit  │  │
│  │ Registry │ Manager  │  Stack   │          │  Logger │  │
│  └──────────┴──────────┴──────────┴──────────┴─────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. AgentSupervisor

**Location**: `crates/kernel/src/agent_sys/supervisor/lifecycle.rs`

The AgentSupervisor maintains a registry of all active agents and manages their lifecycle from spawn to exit.

#### Key Responsibilities

- Track agent metadata (capabilities, policies, uptime)
- Receive lifecycle event notifications from process manager
- Coordinate fault recovery actions
- Manage auto-restart policies
- Maintain PID-to-AgentID mapping

#### Data Structures

```rust
pub struct AgentSupervisor {
    registry: BTreeMap<AgentId, AgentMetadata>,
    pid_to_agent: BTreeMap<Pid, AgentId>,
    listeners: Vec<Box<dyn LifecycleListener>>,
    next_agent_id: AgentId,
}

pub struct AgentMetadata {
    pub agent_id: AgentId,
    pub pid: Pid,
    pub name: String,
    pub capabilities: Vec<Capability>,
    pub scope: Scope,
    pub auto_restart: bool,
    pub max_restarts: u32,
    pub restart_count: u32,
    pub spawn_time: Timestamp,
    pub last_activity: Timestamp,
    pub active: bool,
}
```

### 2. TelemetryAggregator

**Location**: `crates/kernel/src/agent_sys/supervisor/telemetry.rs`

Collects metrics about agent behavior, resource usage, and system health.

#### Metrics Collected

- **Per-Agent Metrics**:
  - Spawn/exit counts
  - Fault counts
  - CPU time (μs)
  - Memory usage (bytes)
  - Syscall counts
  - Recent faults (last 10)

- **System Metrics**:
  - Total spawns/exits since boot
  - Total faults detected
  - Total restarts
  - Active agent count

#### Ring Buffer

The telemetry system maintains a ring buffer of the last 1024 events for debugging and analysis.

### 3. FaultDetector

**Location**: `crates/kernel/src/agent_sys/supervisor/fault.rs`

Monitors agent behavior and detects violations of resource limits and policy.

#### Fault Types

| Fault Type | Description | Default Action |
|------------|-------------|----------------|
| `CpuQuotaExceeded` | CPU time exceeds quota | Throttle |
| `MemoryExceeded` | Memory usage exceeds limit | Kill |
| `SyscallFlood` | Syscall rate too high | Throttle |
| `Crashed` | Agent received fatal signal | Restart |
| `CapabilityViolation` | Attempted unauthorized operation | Kill |
| `Unresponsive` | Watchdog timeout | Restart |
| `PolicyViolation` | General policy violation | Kill |

#### Recovery Actions

- **Kill**: Immediately terminate the agent
- **Throttle**: Reduce agent's CPU quota
- **Restart**: Kill and respawn with same configuration
- **Alert**: Log event but allow agent to continue

### 4. PolicyController

**Location**: `crates/kernel/src/agent_sys/supervisor/policy_controller.rs`

Manages dynamic policy updates with hot-patching support.

#### Policy Patches

```rust
pub enum PolicyPatch {
    AddCapability(Capability),
    RemoveCapability(Capability),
    UpdateScope(Scope),
    EnableAutoRestart { max_restarts: u32 },
    DisableAutoRestart,
}
```

#### Safety Checks

All policy patches are validated before application:
- No privilege escalation (cannot add Admin capability)
- All changes are audited
- Compliance trail maintained

---

## API Reference

### Global Instances

All ASM components are accessed through global static mutexes:

```rust
pub static AGENT_SUPERVISOR: Mutex<Option<AgentSupervisor>>;
pub static TELEMETRY: Mutex<Option<TelemetryAggregator>>;
pub static FAULT_DETECTOR: Mutex<Option<FaultDetector>>;
pub static POLICY_CONTROLLER: Mutex<Option<PolicyController>>;
```

### Initialization

```rust
/// Initialize the Agent Supervision Module
pub fn agent_sys::supervisor::init()
```

Must be called during kernel boot, after AgentSys initialization.

### AgentSupervisor API

#### on_agent_spawn

```rust
pub fn on_agent_spawn(
    &mut self,
    pid: Pid,
    spec: AgentSpec
) -> AgentId
```

Called by process manager when an agent spawns.

**Parameters**:
- `pid`: Process ID of the new agent
- `spec`: Agent specification with capabilities and policies

**Returns**: The assigned agent ID

**Side Effects**:
- Inserts entry into registry
- Notifies lifecycle listeners
- Records telemetry event

#### on_agent_exit

```rust
pub fn on_agent_exit(
    &mut self,
    pid: Pid,
    exit_code: i32
)
```

Called by process manager when an agent exits.

**Parameters**:
- `pid`: Process ID that exited
- `exit_code`: Exit status code

**Side Effects**:
- Removes entry from registry
- Checks auto-restart policy
- Records telemetry event
- Schedules restart if applicable

#### on_fault

```rust
pub fn on_fault(
    &mut self,
    agent_id: AgentId,
    fault: Fault
) -> FaultAction
```

Called when a fault is detected.

**Parameters**:
- `agent_id`: Agent that faulted
- `fault`: Type of fault detected

**Returns**: Action taken in response

**Side Effects**:
- Records telemetry
- Executes recovery action (kill/throttle/restart/alert)
- Logs event to audit trail

#### Query Methods

```rust
pub fn get_agent(&self, agent_id: AgentId) -> Option<&AgentMetadata>
pub fn get_agent_by_pid(&self, pid: Pid) -> Option<&AgentMetadata>
pub fn list_agents(&self) -> Vec<&AgentMetadata>
pub fn agent_count(&self) -> usize
pub fn has_agent(&self, agent_id: AgentId) -> bool
pub fn touch_agent(&mut self, agent_id: AgentId)
```

### TelemetryAggregator API

#### Recording Events

```rust
pub fn record_spawn(&mut self, agent_id: AgentId)
pub fn record_exit(&mut self, agent_id: AgentId, exit_code: i32)
pub fn record_fault(&mut self, agent_id: AgentId, fault: Fault)
pub fn record_restart(&mut self, agent_id: AgentId, attempt: u32)
pub fn record_policy_change(&mut self, agent_id: AgentId)
pub fn record_syscall(&self, agent_id: AgentId)
```

#### Querying Metrics

```rust
pub fn get_agent_metrics(&self, agent_id: AgentId) -> Option<&AgentMetrics>
pub fn get_system_metrics(&self) -> &SystemMetrics
pub fn snapshot(&self) -> TelemetrySnapshot
```

#### Export

```rust
pub fn export_proc(&self, buf: &mut [u8]) -> usize
```

Exports telemetry in human-readable format for `/proc/agentsys/status`.

### FaultDetector API

#### Health Checks

```rust
pub fn check_cpu_quota(&self, agent_id: AgentId, cpu_time_us: u64) -> Option<Fault>
pub fn check_memory_limit(&self, agent_id: AgentId, memory_bytes: usize) -> Option<Fault>
pub fn check_syscall_rate(&self, agent_id: AgentId, rate: u64) -> Option<Fault>
pub fn check_watchdog(&self, agent_id: AgentId, idle_time: u64) -> Option<Fault>
```

#### Fault Reporting

```rust
pub fn report_crash(&self, agent_id: AgentId, signal: Signal) -> Fault
pub fn report_capability_violation(&self, agent_id: AgentId, cap: Capability) -> Fault
pub fn report_policy_violation(&self, agent_id: AgentId, reason: &'static str) -> Fault
```

#### Policy Management

```rust
pub fn get_recovery_policy(&self) -> &RecoveryPolicy
pub fn set_recovery_policy(&mut self, policy: RecoveryPolicy)
pub fn get_default_limits(&self) -> &ResourceLimits
pub fn set_default_limits(&mut self, limits: ResourceLimits)
```

### PolicyController API

#### Policy Updates

```rust
pub fn update_policy(
    &mut self,
    agent_id: AgentId,
    patch: PolicyPatch
) -> Result<(), PolicyError>
```

Applies a policy patch to an agent with safety validation.

#### Query

```rust
pub fn get_policy(&self, agent_id: AgentId) -> Option<&PolicySet>
pub fn get_policy_mut(&mut self, agent_id: AgentId) -> Option<&mut PolicySet>
```

#### Violations

```rust
pub fn record_violation(
    &mut self,
    agent_id: AgentId,
    description: String,
    decision: PolicyDecision,
)
```

#### Compliance

```rust
pub fn export_compliance(&self) -> ComplianceReport
pub fn export_eu_ai_act_report(&self) -> ComplianceReport
```

---

## Usage Examples

### Example 1: Agent Spawn Hook

```rust
// In process manager spawn function
pub fn spawn_agent_process(spec: AgentSpec) -> Result<Pid, SpawnError> {
    let pid = allocate_pid()?;

    // ... create process ...

    // Notify supervisor
    let agent_id = agent_sys::supervisor::AGENT_SUPERVISOR
        .lock()
        .as_mut()
        .unwrap()
        .on_agent_spawn(pid, spec);

    Ok(pid)
}
```

### Example 2: Fault Detection

```rust
// In scheduler tick
pub fn check_agent_health() {
    for (agent_id, metadata) in list_active_agents() {
        let cpu_time = get_cpu_time(agent_id);

        if let Some(fault) = FAULT_DETECTOR
            .lock()
            .as_ref()
            .unwrap()
            .check_cpu_quota(agent_id, cpu_time)
        {
            AGENT_SUPERVISOR
                .lock()
                .as_mut()
                .unwrap()
                .on_fault(agent_id, fault);
        }
    }
}
```

### Example 3: Dynamic Policy Update

```rust
// Update agent capabilities at runtime
use agent_sys::supervisor::{POLICY_CONTROLLER, PolicyPatch};

fn grant_filesystem_access(agent_id: AgentId) -> Result<(), PolicyError> {
    let patch = PolicyPatch::AddCapability(Capability::FsBasic);

    POLICY_CONTROLLER
        .lock()
        .as_mut()
        .unwrap()
        .update_policy(agent_id, patch)?;

    Ok(())
}
```

### Example 4: Reading Telemetry

```rust
// Get system-wide statistics
fn print_agent_stats() {
    let snapshot = TELEMETRY
        .lock()
        .as_ref()
        .unwrap()
        .snapshot();

    println!("Total Spawns: {}", snapshot.system_metrics.total_spawns);
    println!("Active Agents: {}", snapshot.system_metrics.active_agents);

    for (agent_id, metrics) in snapshot.agent_metrics {
        println!("Agent {}: {} spawns, {} faults",
            agent_id, metrics.spawn_count, metrics.fault_count);
    }
}
```

---

## Integration Guide

### Step 1: Initialize ASM

Add to kernel initialization:

```rust
// In kernel main
pub fn kernel_main() {
    // ... other initialization ...

    agent_sys::init();  // This now initializes ASM too

    // ... continue boot ...
}
```

### Step 2: Add Process Manager Hooks

```rust
// In process/mod.rs or process/scheduler.rs

// When spawning agent process
if is_agent_process {
    let spec = create_agent_spec(agent_id, capabilities);
    agent_sys::supervisor::AGENT_SUPERVISOR
        .lock()
        .as_mut()
        .unwrap()
        .on_agent_spawn(pid, spec);
}

// When process exits
if is_agent_process {
    agent_sys::supervisor::AGENT_SUPERVISOR
        .lock()
        .as_mut()
        .unwrap()
        .on_agent_exit(pid, exit_code);
}
```

### Step 3: Add Fault Detection

```rust
// In scheduler or periodic health check
pub fn periodic_health_check() {
    for agent in active_agents() {
        // Check CPU usage
        if let Some(fault) = check_cpu_usage(agent.id) {
            report_fault(agent.id, fault);
        }

        // Check memory usage
        if let Some(fault) = check_memory_usage(agent.id) {
            report_fault(agent.id, fault);
        }

        // Check watchdog
        if let Some(fault) = check_watchdog(agent.id) {
            report_fault(agent.id, fault);
        }
    }
}
```

### Step 4: Export Telemetry via /proc

```rust
// In fs/procfs.rs
impl ProcFS {
    fn register_agentsys_entries(&mut self) {
        self.add_file("/proc/agentsys/status", |buf| {
            agent_sys::supervisor::TELEMETRY
                .lock()
                .as_ref()
                .unwrap()
                .export_proc(buf)
        });
    }
}
```

---

## Performance Characteristics

### Memory Overhead

| Component | Per-Agent Overhead | Fixed Overhead |
|-----------|-------------------|----------------|
| AgentSupervisor | ~200 bytes | ~1 KB |
| TelemetryAggregator | ~500 bytes | ~4 KB |
| PolicyController | ~200 bytes | ~1 KB |
| FaultDetector | ~50 bytes | ~512 bytes |
| **Total** | **~950 bytes** | **~6.5 KB** |

For 1000 agents: ~950 KB total memory usage

### Time Overhead

| Operation | Overhead | Notes |
|-----------|----------|-------|
| Agent spawn | +50 μs | Registry insertion |
| Agent exit | +30 μs | Telemetry update |
| Policy check | +5 μs | Hash table lookup (cached) |
| Telemetry recording | +2 μs | Atomic increment |
| Fault detection | +10 μs | Per health check |

### Scalability

- **1,000 agents**: All operations < 100 μs
- **10,000 agents**: Hash map resizing may cause 1-2 ms spikes
- **100,000 agents**: Consider sharded registries

---

## Security Model

### Threat Mitigation

| Threat | Mitigation |
|--------|------------|
| Compromised agent privilege escalation | Policy validation rejects escalation attempts |
| Malicious agent floods APIs | Rate limiting enforced per-agent |
| Agent tries to kill other agents | Capability checks prevent cross-agent interference |
| Userland process reads sensitive data | /proc exports only aggregate, non-sensitive data |
| Agent bypasses supervision | All lifecycle events go through kernel hooks |

### Security Properties

1. **Isolation**: Agents cannot interfere with each other
2. **Least Privilege**: Each agent has minimal needed capabilities
3. **Audit Trail**: All actions logged immutably
4. **Policy Enforcement**: Kernel enforces all policies
5. **Fail-Safe**: Compromised agent cannot crash supervisor

### Privilege Separation

ASM components run in kernel space with ring-0 privileges, but:
- Each component has minimal scope
- No single component has system-wide control
- Policy changes are validated and audited
- Admin capability required for sensitive operations

---

## Future Work

### Milestone 1: Policy Controller Extensions (Week 2)
- Hot-patch validation improvements
- Fine-grained capability management
- Enhanced compliance reporting

### Milestone 2: Cloud Gateway (Weeks 3-4)
- Multi-provider LLM routing
- Rate limiting integration
- Fallback policy implementation

### Milestone 3: Advanced Telemetry (Week 5)
- Real-time dashboard integration
- Predictive fault detection
- ML-based anomaly detection

### Milestone 4: Enhanced Fault Detection (Week 6)
- Adaptive resource limits
- Pattern-based fault prediction
- Advanced recovery strategies

---

## References

- [SIS Kernel AgentSys](../../crates/kernel/src/agent_sys/)
- [Process Manager](../../crates/kernel/src/process/)
- [Agent Supervision Plan](../plans/AGENT_SUPERVISION_MODULE_PLAN.md)
- [EU AI Act Compliance](https://artificialintelligenceact.eu/)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-16
**Status**: Milestone 0 Complete
