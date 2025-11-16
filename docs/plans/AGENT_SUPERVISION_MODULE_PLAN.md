# Agent Supervision Module (ASM) Integration Plan

**Status**: Planning
**Priority**: P0 - Critical Infrastructure Enhancement
**Timeline**: 6-8 weeks
**Dependencies**: AgentSys (Phase 9), VFS, Network Stack, Process Manager
**Supersedes**: MCP (Master Control Process) proposal - renamed to avoid Model Context Protocol collision

---

## Executive Summary

This plan describes the integration of the **Agent Supervision Module (ASM)** into the SIS Kernel as a kernel-resident service for comprehensive agent lifecycle management, policy enforcement, cloud API mediation, and telemetry aggregation.

**Critical Design Decision**: ASM is **NOT** a userland super-agent. It is a collection of kernel-resident services that extend the existing AgentSys infrastructure. This approach:

- **Prevents single point of failure**: Distributed responsibilities across kernel subsystems
- **Maintains security**: No privileged userland process to compromise
- **Leverages existing infrastructure**: Extends AgentSys, ProcessManager, NetworkStack
- **Avoids bottlenecks**: Direct syscall access vs. IPC through coordinator process

### Key Components

1. **Agent Supervisor** (kernel-resident): Lifecycle hooks, fault detection, recovery
2. **Policy Controller** (kernel-resident): Dynamic policy updates, hot-patching, compliance
3. **Cloud Gateway** (kernel network service): LLM API routing, rate limiting, fallback
4. **Telemetry Aggregator** (kernel-resident): Metrics collection, /proc export
5. **Admin CLI** (userland, unprivileged): Read-only dashboard and control interface

### Integration Points

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
│  │  │ Telemetry      │  │ Cloud Gateway          │    │  │
│  │  │ Aggregator     │  │ - LLM routing          │    │  │
│  │  │ - Metrics      │  │ - Rate limiting        │    │  │
│  │  │ - /proc export │  │ - Fallback logic       │    │  │
│  │  │ - Audit logs   │  │ - Multi-backend        │    │  │
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

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Design Principles](#design-principles)
3. [Milestone 0: Agent Supervisor Foundation](#milestone-0-agent-supervisor-foundation)
4. [Milestone 1: Policy Controller Extensions](#milestone-1-policy-controller-extensions)
5. [Milestone 2: Cloud Gateway Implementation](#milestone-2-cloud-gateway-implementation)
6. [Milestone 3: Telemetry Aggregation](#milestone-3-telemetry-aggregation)
7. [Milestone 4: Fault Detection & Recovery](#milestone-4-fault-detection--recovery)
8. [Milestone 5: Admin CLI & Dashboard](#milestone-5-admin-cli--dashboard)
9. [Milestone 6: Testing & Validation](#milestone-6-testing--validation)
10. [Timeline](#timeline)
11. [Migration from Original MCP Design](#migration-from-original-mcp-design)
12. [Security Considerations](#security-considerations)
13. [Performance Considerations](#performance-considerations)
14. [References](#references)

---

## Architecture Overview

### Why NOT a Userland Super-Agent?

The original MCP (Master Control Process) design proposed a privileged userland agent with "root capabilities" over all agents. This is **rejected** for the following reasons:

| Issue | Userland Super-Agent | Kernel-Resident Services |
|-------|---------------------|--------------------------|
| **Single Point of Failure** | MCP crash → all agent management fails | Distributed: each subsystem independent |
| **Attack Surface** | Compromise MCP → attacker owns userland | Kernel privilege separation, no single target |
| **Performance** | IPC overhead for every agent operation | Direct syscall, no context switching |
| **Recovery** | If MCP dies, who supervises it? | Kernel survives, userland tools restart |
| **Privilege Model** | Violates least privilege principle | Each service has minimal needed capabilities |

### Component Breakdown

#### 1. **Agent Supervisor** (crates/kernel/src/agentsys/supervisor.rs)

```rust
pub struct AgentSupervisor {
    /// Registry of all agents with metadata
    registry: HashMap<AgentId, AgentMetadata>,

    /// Fault detector for health monitoring
    fault_detector: FaultDetector,

    /// Recovery strategies
    recovery_policy: RecoveryPolicy,

    /// Lifecycle event subscribers
    listeners: Vec<Box<dyn LifecycleListener>>,
}

impl AgentSupervisor {
    /// Called by process manager when agent spawns
    pub fn on_agent_spawn(&mut self, id: AgentId, spec: AgentSpec) {
        self.registry.insert(id, AgentMetadata::new(spec));
        self.notify_listeners(LifecycleEvent::Spawned(id));
        TELEMETRY.record_spawn(id);
    }

    /// Called by process manager when agent exits
    pub fn on_agent_exit(&mut self, id: AgentId, exit_code: i32) {
        if let Some(meta) = self.registry.remove(&id) {
            self.notify_listeners(LifecycleEvent::Exited(id, exit_code));
            TELEMETRY.record_exit(id, exit_code);

            // Check if recovery needed
            if meta.auto_restart && exit_code != 0 {
                self.schedule_restart(id, meta);
            }
        }
    }

    /// Called by fault detector on resource violation
    pub fn on_fault(&mut self, id: AgentId, fault: Fault) -> FaultAction {
        AUDIT_LOG.log(AuditEvent::Fault { id, fault });

        match self.recovery_policy.action_for(&fault) {
            FaultAction::Kill => {
                PROCESS_MGR.kill(id);
                FaultAction::Kill
            }
            FaultAction::Throttle => {
                PROCESS_MGR.throttle(id, fault.severity());
                FaultAction::Throttle
            }
            FaultAction::Alert => {
                TELEMETRY.alert(id, fault);
                FaultAction::Alert
            }
        }
    }
}
```

**Integration Point**: Hooks into existing `crates/kernel/src/process/mod.rs`

#### 2. **Policy Controller** (crates/kernel/src/agentsys/policy_controller.rs)

```rust
pub struct PolicyController {
    /// Current policy rules by agent ID
    policies: HashMap<AgentId, PolicySet>,

    /// Global default policies
    defaults: PolicySet,

    /// Hot-patch queue for dynamic updates
    patch_queue: VecDeque<PolicyPatch>,

    /// Compliance export buffer
    compliance_log: ComplianceLog,
}

impl PolicyController {
    /// Syscall handler for policy updates
    pub fn update_policy(&mut self, id: AgentId, patch: PolicyPatch) -> Result<(), PolicyError> {
        // Validate patch (no privilege escalation)
        if !patch.is_safe() {
            return Err(PolicyError::PrivilegeEscalation);
        }

        // Audit the change
        AUDIT_LOG.log(AuditEvent::PolicyUpdate { id, patch: patch.clone() });

        // Apply hot-patch
        if let Some(policy) = self.policies.get_mut(&id) {
            policy.apply(patch)?;
        }

        // Notify supervisor
        AGENT_SUPERVISOR.on_policy_change(id);

        Ok(())
    }

    /// Dynamic capability grant/revoke
    pub fn update_capabilities(&mut self, id: AgentId, caps: CapabilitySet) -> Result<(), PolicyError> {
        // Check if requester has permission to modify capabilities
        let requester = current_agent_id()?;
        if !self.can_modify_caps(requester, id) {
            return Err(PolicyError::InsufficientPermission);
        }

        // Apply change
        if let Some(policy) = self.policies.get_mut(&id) {
            policy.capabilities = caps;
            AUDIT_LOG.log(AuditEvent::CapabilityChange { id, caps });
        }

        Ok(())
    }

    /// Export compliance report (EU AI Act, etc.)
    pub fn export_compliance(&self) -> ComplianceReport {
        ComplianceReport {
            timestamp: current_timestamp(),
            agents: self.policies.iter().map(|(id, policy)| {
                ComplianceEntry {
                    id: *id,
                    capabilities: policy.capabilities.clone(),
                    violations: policy.violations.clone(),
                    audit_trail: policy.audit_trail.clone(),
                }
            }).collect(),
        }
    }
}
```

**Integration Point**: Extends `crates/kernel/src/agentsys/policy.rs`

#### 3. **Cloud Gateway** (crates/kernel/src/net/cloud_gateway.rs)

```rust
pub struct CloudGateway {
    /// Backend routing table
    backends: HashMap<Provider, Box<dyn CloudBackend>>,

    /// Rate limiter per agent
    rate_limiters: HashMap<AgentId, RateLimiter>,

    /// Fallback policy
    fallback_policy: FallbackPolicy,

    /// Request metrics
    metrics: GatewayMetrics,
}

impl CloudGateway {
    /// Route LLM request to appropriate backend
    pub fn route_request(&mut self, req: LLMRequest) -> Result<LLMResponse, GatewayError> {
        // Check rate limit
        if !self.rate_limiters.get_mut(&req.agent_id)
            .ok_or(GatewayError::UnknownAgent)?
            .check_and_consume()
        {
            return Err(GatewayError::RateLimitExceeded);
        }

        // Select backend based on policy
        let provider = self.select_provider(&req)?;

        // Attempt request with fallback
        match self.backends.get_mut(&provider).unwrap().call(req.clone()) {
            Ok(resp) => {
                self.metrics.record_success(provider, req.agent_id);
                Ok(resp)
            }
            Err(e) => {
                self.metrics.record_failure(provider, req.agent_id);

                // Fallback logic
                match self.fallback_policy.next_provider(provider) {
                    Some(fallback) => {
                        AUDIT_LOG.log(AuditEvent::CloudFallback {
                            from: provider,
                            to: fallback,
                            reason: e
                        });
                        self.backends.get_mut(&fallback).unwrap().call(req)
                    }
                    None => Err(e),
                }
            }
        }
    }

    /// Select provider based on policy, load, cost
    fn select_provider(&self, req: &LLMRequest) -> Result<Provider, GatewayError> {
        // User preference override
        if let Some(pref) = req.preferred_provider {
            return Ok(pref);
        }

        // Load balancing: cheapest available provider
        self.backends.iter()
            .filter(|(_, backend)| backend.is_healthy())
            .min_by_key(|(_, backend)| backend.cost_per_token())
            .map(|(provider, _)| *provider)
            .ok_or(GatewayError::NoHealthyBackends)
    }
}

/// Cloud backend trait
pub trait CloudBackend: Send + Sync {
    fn call(&mut self, req: LLMRequest) -> Result<LLMResponse, GatewayError>;
    fn is_healthy(&self) -> bool;
    fn cost_per_token(&self) -> u64;
}

/// Provider implementations
pub struct ClaudeBackend { /* ... */ }
pub struct GPT4Backend { /* ... */ }
pub struct GeminiBackend { /* ... */ }
pub struct LocalFallbackBackend { /* ... */ }
```

**Integration Point**: New module in `crates/kernel/src/net/`

#### 4. **Telemetry Aggregator** (crates/kernel/src/agentsys/telemetry.rs)

```rust
pub struct TelemetryAggregator {
    /// Per-agent metrics
    agent_metrics: HashMap<AgentId, AgentMetrics>,

    /// System-wide aggregates
    system_metrics: SystemMetrics,

    /// Ring buffer for recent events
    event_buffer: RingBuffer<TelemetryEvent>,

    /// /proc export formatter
    proc_formatter: ProcFormatter,
}

impl TelemetryAggregator {
    /// Record agent spawn
    pub fn record_spawn(&mut self, id: AgentId) {
        let metrics = self.agent_metrics.entry(id).or_default();
        metrics.spawn_count += 1;
        metrics.last_spawn = current_timestamp();

        self.system_metrics.total_spawns += 1;
        self.event_buffer.push(TelemetryEvent::Spawn(id));
    }

    /// Record agent exit
    pub fn record_exit(&mut self, id: AgentId, exit_code: i32) {
        if let Some(metrics) = self.agent_metrics.get_mut(&id) {
            metrics.exit_count += 1;
            metrics.last_exit = current_timestamp();
            metrics.last_exit_code = exit_code;
        }

        self.event_buffer.push(TelemetryEvent::Exit(id, exit_code));
    }

    /// Record fault
    pub fn record_fault(&mut self, id: AgentId, fault: Fault) {
        if let Some(metrics) = self.agent_metrics.get_mut(&id) {
            metrics.fault_count += 1;
            metrics.faults.push(fault);
        }

        self.system_metrics.total_faults += 1;
        self.event_buffer.push(TelemetryEvent::Fault(id, fault));
    }

    /// Export to /proc/agentsys/status
    pub fn export_proc(&self, buf: &mut [u8]) -> usize {
        self.proc_formatter.format(&self.agent_metrics, &self.system_metrics, buf)
    }

    /// Get snapshot for userland tools
    pub fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            timestamp: current_timestamp(),
            agents: self.agent_metrics.clone(),
            system: self.system_metrics.clone(),
            recent_events: self.event_buffer.to_vec(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AgentMetrics {
    pub spawn_count: u64,
    pub exit_count: u64,
    pub fault_count: u64,
    pub cpu_time_us: u64,
    pub memory_bytes: usize,
    pub syscall_count: u64,
    pub last_spawn: Timestamp,
    pub last_exit: Timestamp,
    pub last_exit_code: i32,
    pub faults: Vec<Fault>,
}
```

**Integration Point**: New module, exports via /proc filesystem

---

## Design Principles

1. **No Userland Super-Agent**: All supervision logic in kernel
2. **Distributed Responsibilities**: Each subsystem handles its domain
3. **Least Privilege**: Services have minimal needed capabilities
4. **Fail-Safe**: Kernel services survive agent failures
5. **Observable**: All actions logged and visible via /proc
6. **Policy-Driven**: Runtime behavior controlled by policy engine
7. **Performance**: Direct syscalls, no IPC bottlenecks

---

## Milestone 0: Agent Supervisor Foundation

**Duration**: 1 week
**Goal**: Kernel-resident supervisor with lifecycle hooks

### Tasks

1. Create `crates/kernel/src/agentsys/supervisor.rs`
2. Define `AgentMetadata` struct with spawn time, capabilities, policy
3. Implement lifecycle hooks:
   - `on_agent_spawn()`
   - `on_agent_exit()`
   - `on_fault()`
4. Integrate with `crates/kernel/src/process/mod.rs`:
   ```rust
   // In process manager
   pub fn spawn_process(&mut self, spec: ProcessSpec) -> Result<Pid, SpawnError> {
       let pid = self.allocate_pid()?;
       // ...

       // Hook: notify supervisor
       if spec.is_agent {
           AGENT_SUPERVISOR.on_agent_spawn(pid, spec.agent_spec);
       }

       Ok(pid)
   }
   ```

5. Add global supervisor instance:
   ```rust
   // crates/kernel/src/agentsys/mod.rs
   use lazy_static::lazy_static;
   use spin::Mutex;

   lazy_static! {
       pub static ref AGENT_SUPERVISOR: Mutex<AgentSupervisor> =
           Mutex::new(AgentSupervisor::new());
   }
   ```

### Testing

- Unit tests for `AgentSupervisor` methods
- Integration test: spawn/exit 100 agents, verify registry correctness
- Stress test: concurrent spawn/exit, check for race conditions

### Deliverable

- `AgentSupervisor` operational
- Lifecycle hooks firing correctly
- Registry tracking all agents

---

## Milestone 1: Policy Controller Extensions

**Duration**: 1 week
**Goal**: Dynamic policy updates and hot-patching

### Tasks

1. Extend `crates/kernel/src/agentsys/policy.rs` with `PolicyController`
2. Implement syscall `sys_agentsys_update_policy()`:
   ```rust
   // crates/kernel/src/syscall/agentsys.rs
   pub fn sys_agentsys_update_policy(
       agent_id: AgentId,
       patch_ptr: *const PolicyPatch
   ) -> SyscallResult {
       // Validate caller has permission
       let caller = current_process().agent_id()?;

       // Safety: validate user pointer
       let patch = unsafe {
           validate_user_ptr(patch_ptr, size_of::<PolicyPatch>())?
       };

       // Apply via policy controller
       POLICY_CONTROLLER.lock().update_policy(agent_id, patch)?;

       Ok(0)
   }
   ```

3. Implement hot-patch validation:
   - No privilege escalation
   - No capability additions without admin permission
   - Audit all changes

4. Add compliance export:
   ```rust
   pub fn export_eu_ai_act_report(&self) -> ComplianceReport {
       // Export per Articles 13-16
       // ...
   }
   ```

### Testing

- Test privilege escalation prevention
- Test hot-patch application
- Verify audit trail completeness
- Compliance report generation

### Deliverable

- Dynamic policy updates working
- Hot-patching functional
- Compliance export available

---

## Milestone 2: Cloud Gateway Implementation

**Duration**: 2 weeks
**Goal**: Multi-provider LLM routing with fallback

### Tasks

1. Create `crates/kernel/src/net/cloud_gateway.rs`
2. Define `CloudBackend` trait
3. Implement backends:
   - `ClaudeBackend` (Anthropic API)
   - `GPT4Backend` (OpenAI API)
   - `GeminiBackend` (Google API)
   - `LocalFallbackBackend` (kernel LLM stub)

4. Implement rate limiting:
   ```rust
   pub struct RateLimiter {
       tokens: AtomicU32,
       capacity: u32,
       refill_rate: u32, // tokens per second
       last_refill: Timestamp,
   }

   impl RateLimiter {
       pub fn check_and_consume(&mut self) -> bool {
           self.refill();

           if self.tokens.load(Ordering::Acquire) > 0 {
               self.tokens.fetch_sub(1, Ordering::Release);
               true
           } else {
               false
           }
       }
   }
   ```

5. Implement fallback policy:
   ```rust
   pub enum FallbackPolicy {
       /// Try next cheapest provider
       CostOptimized,
       /// Try specific fallback chain
       Explicit(Vec<Provider>),
       /// Use local LLM
       LocalOnly,
   }
   ```

6. Add syscall `sys_llm_request()`:
   ```rust
   pub fn sys_llm_request(req_ptr: *const LLMRequest) -> SyscallResult {
       let req = unsafe { validate_user_ptr(req_ptr, size_of::<LLMRequest>())? };

       // Check capability
       if !current_process().has_capability(Capability::LLM_ACCESS) {
           return Err(SyscallError::PermissionDenied);
       }

       // Route via gateway
       let resp = CLOUD_GATEWAY.lock().route_request(req)?;

       // Write response to userland buffer
       Ok(resp.serialize())
   }
   ```

### Testing

- Test each backend independently
- Test fallback transitions
- Test rate limiting
- Stress test: concurrent requests from 100 agents
- Failure injection: network errors, API timeouts

### Deliverable

- Cloud gateway operational
- All backends working
- Fallback logic functional
- Rate limiting enforced

---

## Milestone 3: Telemetry Aggregation

**Duration**: 1 week
**Goal**: Centralized metrics collection and /proc export

### Tasks

1. Create `crates/kernel/src/agentsys/telemetry.rs`
2. Implement `TelemetryAggregator` with ring buffer
3. Add recording hooks in supervisor:
   ```rust
   impl AgentSupervisor {
       pub fn on_agent_spawn(&mut self, id: AgentId, spec: AgentSpec) {
           // ... existing code ...
           TELEMETRY.lock().record_spawn(id);
       }
   }
   ```

4. Export to /proc filesystem:
   ```rust
   // crates/kernel/src/fs/procfs.rs
   impl ProcFS {
       fn register_agentsys_entries(&mut self) {
           self.add_file("/proc/agentsys/status", |buf| {
               TELEMETRY.lock().export_proc(buf)
           });

           self.add_file("/proc/agentsys/telemetry", |buf| {
               let snapshot = TELEMETRY.lock().snapshot();
               serde_json::to_vec(&snapshot).unwrap()
           });
       }
   }
   ```

5. Add syscall `sys_agentsys_telemetry()`:
   ```rust
   pub fn sys_agentsys_telemetry(buf: *mut u8, len: usize) -> SyscallResult {
       let snapshot = TELEMETRY.lock().snapshot();
       let serialized = serde_json::to_vec(&snapshot)?;

       unsafe {
           copy_to_user(buf, &serialized, min(len, serialized.len()))?;
       }

       Ok(serialized.len())
   }
   ```

### Testing

- Verify /proc/agentsys/status readable
- Verify metrics accuracy
- Stress test: high-frequency events, check for overruns
- Test syscall telemetry retrieval

### Deliverable

- Telemetry aggregation working
- /proc export functional
- Syscall telemetry access

---

## Milestone 4: Fault Detection & Recovery

**Duration**: 1 week
**Goal**: Automatic fault detection and recovery strategies

### Tasks

1. Create `crates/kernel/src/agentsys/fault_detector.rs`
2. Define fault types:
   ```rust
   #[derive(Debug, Clone, Copy)]
   pub enum Fault {
       /// CPU usage exceeded quota
       CpuQuotaExceeded { used: u64, quota: u64 },

       /// Memory usage exceeded limit
       MemoryExceeded { used: usize, limit: usize },

       /// Syscall rate limit exceeded
       SyscallFlood { rate: u64, threshold: u64 },

       /// Crash/segfault
       Crashed { signal: Signal },

       /// Capability violation attempt
       CapabilityViolation { attempted: Capability },
   }
   ```

3. Implement recovery policy:
   ```rust
   pub enum RecoveryPolicy {
       /// Kill the agent
       Kill,

       /// Throttle resources
       Throttle { cpu_factor: f32 },

       /// Restart with same config
       Restart { max_attempts: u32 },

       /// Alert admin, continue running
       AlertOnly,
   }

   impl RecoveryPolicy {
       pub fn action_for(&self, fault: &Fault) -> FaultAction {
           match (self, fault) {
               (_, Fault::Crashed { .. }) => FaultAction::Restart,
               (_, Fault::CapabilityViolation { .. }) => FaultAction::Kill,
               (_, Fault::CpuQuotaExceeded { .. }) => FaultAction::Throttle,
               // ...
           }
       }
   }
   ```

4. Integrate with process manager:
   ```rust
   // In scheduler tick
   pub fn check_agent_health(&mut self) {
       for agent_id in self.active_agents() {
           if let Some(fault) = self.detect_fault(agent_id) {
               AGENT_SUPERVISOR.lock().on_fault(agent_id, fault);
           }
       }
   }
   ```

5. Implement auto-restart:
   ```rust
   impl AgentSupervisor {
       pub fn schedule_restart(&mut self, id: AgentId, meta: AgentMetadata) {
           if meta.restart_count < meta.max_restarts {
               let new_id = PROCESS_MGR.spawn(meta.spec);
               self.registry.insert(new_id, meta.with_restart_count(meta.restart_count + 1));
           } else {
               AUDIT_LOG.log(AuditEvent::MaxRestartsExceeded { id });
           }
       }
   }
   ```

### Testing

- Test each fault type triggers correct action
- Test auto-restart logic
- Test max restart limit enforcement
- Inject faults, verify recovery

### Deliverable

- Fault detection operational
- Recovery strategies working
- Auto-restart functional

---

## Milestone 5: Admin CLI & Dashboard

**Duration**: 1 week
**Goal**: Userland tools for observability and control

### Tasks

1. Create `/bin/agentctl` CLI tool:
   ```rust
   // userland/agentctl/src/main.rs
   fn main() {
       let args = Args::parse();

       match args.command {
           Command::Status => {
               // Read /proc/agentsys/status
               let status = read_to_string("/proc/agentsys/status")?;
               println!("{}", status);
           }

           Command::List => {
               // Get telemetry via syscall
               let snapshot = syscall::agentsys_telemetry()?;
               for (id, metrics) in snapshot.agents {
                   println!("{}: spawns={} exits={} faults={}",
                       id, metrics.spawn_count, metrics.exit_count, metrics.fault_count);
               }
           }

           Command::PolicyUpdate { agent_id, policy } => {
               // Update policy via syscall
               syscall::agentsys_update_policy(agent_id, &policy)?;
               println!("Policy updated for agent {}", agent_id);
           }

           Command::CloudStatus => {
               // Query cloud gateway metrics
               let metrics = syscall::cloud_gateway_metrics()?;
               println!("Active backends: {:?}", metrics.healthy_backends);
               println!("Request rate: {} req/s", metrics.request_rate);
           }
       }
   }
   ```

2. Add shell commands:
   ```rust
   // crates/kernel/src/shell/agent_helpers.rs
   impl Shell {
       pub fn agent_status_cmd(&self) {
           let snapshot = TELEMETRY.lock().snapshot();
           uart_println!("Active agents: {}", snapshot.agents.len());

           for (id, metrics) in snapshot.agents {
               uart_println!("  {}: cpu={}us mem={} faults={}",
                   id, metrics.cpu_time_us, metrics.memory_bytes, metrics.fault_count);
           }
       }

       pub fn agent_policy_cmd(&self, agent_id: AgentId) {
           if let Some(policy) = POLICY_CONTROLLER.lock().get_policy(agent_id) {
               uart_println!("Policy for agent {}:", agent_id);
               uart_println!("  Capabilities: {:?}", policy.capabilities);
               uart_println!("  Auto-restart: {}", policy.auto_restart);
           }
       }
   }
   ```

3. Extend web dashboard:
   ```typescript
   // web/dashboard/src/components/AgentPanel.tsx
   function AgentPanel() {
       const [agents, setAgents] = useState<AgentMetrics[]>([]);

       useEffect(() => {
           const fetchAgents = async () => {
               const resp = await fetch('/api/agentsys/telemetry');
               const data = await resp.json();
               setAgents(data.agents);
           };

           const interval = setInterval(fetchAgents, 1000);
           return () => clearInterval(interval);
       }, []);

       return (
           <div>
               <h2>Agent Status</h2>
               <table>
                   <thead>
                       <tr>
                           <th>Agent ID</th>
                           <th>Spawns</th>
                           <th>Exits</th>
                           <th>Faults</th>
                           <th>CPU (us)</th>
                           <th>Memory (bytes)</th>
                       </tr>
                   </thead>
                   <tbody>
                       {agents.map(agent => (
                           <tr key={agent.id}>
                               <td>{agent.id}</td>
                               <td>{agent.spawn_count}</td>
                               <td>{agent.exit_count}</td>
                               <td>{agent.fault_count}</td>
                               <td>{agent.cpu_time_us}</td>
                               <td>{agent.memory_bytes}</td>
                           </tr>
                       ))}
                   </tbody>
               </table>
           </div>
       );
   }
   ```

### Testing

- Test agentctl commands
- Test shell commands
- Verify dashboard updates in real-time
- Stress test: 100 agents, verify UI responsiveness

### Deliverable

- agentctl CLI functional
- Shell commands working
- Dashboard displaying agent metrics

---

## Milestone 6: Testing & Validation

**Duration**: 1 week
**Goal**: Comprehensive testing and validation

### Test Plan

#### Unit Tests

- [ ] AgentSupervisor lifecycle hooks
- [ ] PolicyController validation logic
- [ ] CloudGateway routing logic
- [ ] TelemetryAggregator metrics
- [ ] FaultDetector fault classification

#### Integration Tests

- [ ] Spawn 1000 agents, verify registry correctness
- [ ] Dynamic policy update during execution
- [ ] Cloud gateway fallback transitions
- [ ] Fault injection and recovery
- [ ] Telemetry export accuracy

#### Stress Tests

- [ ] 10,000 agents spawning concurrently
- [ ] 1000 req/s through cloud gateway
- [ ] Continuous fault injection
- [ ] Memory pressure during telemetry collection

#### Security Tests

- [ ] Privilege escalation attempts
- [ ] Capability violation detection
- [ ] Policy update authorization checks
- [ ] /proc information disclosure

### Validation Criteria

- All unit tests pass
- All integration tests pass
- Stress tests show <5% overhead
- No security vulnerabilities found
- Documentation complete

---

## Timeline

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1 | M0: Agent Supervisor | Lifecycle hooks operational |
| 2 | M1: Policy Controller | Dynamic policy updates |
| 3-4 | M2: Cloud Gateway | Multi-provider routing |
| 5 | M3: Telemetry | /proc export and metrics |
| 6 | M4: Fault Detection | Auto-recovery working |
| 7 | M5: Admin CLI | Tools and dashboard |
| 8 | M6: Testing | Full validation |

**Total: 8 weeks**

---

## Migration from Original MCP Design

### What Changes

| Original MCP | ASM (This Plan) | Rationale |
|--------------|-----------------|-----------|
| Userland super-agent | Kernel services | No single point of failure |
| Root capabilities | Distributed privileges | Least privilege principle |
| IPC-based coordination | Direct syscalls | Performance |
| Single supervisor process | Multiple kernel modules | Fault isolation |
| Manual recovery | Automatic recovery | Resilience |

### What Stays the Same

- Agent lifecycle management (spawn, exit, fault)
- Policy enforcement and hot-patching
- Cloud API routing and fallback
- Telemetry aggregation and export
- Admin CLI for observability

### Migration Path

1. **Week 1-2**: Implement kernel services (no userland changes)
2. **Week 3-4**: Add cloud gateway (deprecate direct API calls)
3. **Week 5-6**: Migrate agents to use new telemetry syscalls
4. **Week 7**: Add agentctl CLI
5. **Week 8**: Final testing and documentation

---

## Security Considerations

### Threat Model

| Threat | Mitigation |
|--------|------------|
| Compromised agent tries privilege escalation | Policy validation rejects escalation attempts |
| Malicious agent floods cloud APIs | Rate limiting enforced per-agent |
| Agent tries to kill other agents | Capability checks prevent cross-agent interference |
| Userland process reads sensitive agent data | /proc exports only aggregate, non-sensitive data |
| Agent bypasses supervision | All lifecycle events go through kernel hooks |

### Security Properties

1. **Isolation**: Agents cannot interfere with each other
2. **Least Privilege**: Each agent has minimal needed capabilities
3. **Audit Trail**: All actions logged immutably
4. **Policy Enforcement**: Kernel enforces all policies
5. **Fail-Safe**: Compromised agent cannot crash supervisor

---

## Performance Considerations

### Overhead Analysis

| Operation | Overhead | Mitigation |
|-----------|----------|------------|
| Agent spawn | +50μs (supervisor hook) | Acceptable for infrequent operation |
| Agent exit | +30μs (telemetry update) | Minimal |
| Policy check | +5μs (hash table lookup) | Cached lookups |
| Telemetry recording | +2μs (atomic increment) | Lock-free counters |
| Cloud request | +100μs (gateway routing) | Necessary for fallback logic |

### Scalability

- **1000 agents**: All operations remain <100μs
- **10,000 agents**: Hash map resizing may cause 1-2ms spikes
- **100,000 agents**: Consider sharded registries

### Memory Footprint

- AgentSupervisor: ~100 bytes per agent
- PolicyController: ~200 bytes per agent
- TelemetryAggregator: ~500 bytes per agent
- **Total**: ~800 bytes per agent = 800KB for 1000 agents

---

## References

1. [AgentSys Implementation](../../crates/kernel/src/agentsys/)
2. [Process Manager](../../crates/kernel/src/process/mod.rs)
3. [VFS](../../crates/kernel/src/fs/vfs.rs)
4. [Network Stack](../../crates/kernel/src/net/)
5. [Anthropic Model Context Protocol](https://modelcontextprotocol.io/) (naming collision avoided)
6. [EU AI Act](https://artificialintelligenceact.eu/)

---

## Appendix A: File Structure

```
crates/kernel/src/agentsys/
├── mod.rs                      # Module exports
├── supervisor.rs               # AgentSupervisor (M0)
├── policy_controller.rs        # PolicyController (M1)
├── telemetry.rs               # TelemetryAggregator (M3)
├── fault_detector.rs          # FaultDetector (M4)
└── types.rs                   # Shared types

crates/kernel/src/net/
└── cloud_gateway.rs           # CloudGateway (M2)

crates/kernel/src/syscall/
└── agentsys.rs                # ASM syscalls

crates/kernel/src/shell/
└── agent_helpers.rs           # Shell commands

userland/agentctl/
├── src/main.rs                # CLI tool
└── Cargo.toml

web/dashboard/src/components/
└── AgentPanel.tsx             # Dashboard UI
```

## Appendix B: Syscall Interface

```rust
// sys_agentsys_update_policy(agent_id, patch)
syscall!(AGENTSYS_UPDATE_POLICY = 300);

// sys_agentsys_telemetry(buf, len)
syscall!(AGENTSYS_TELEMETRY = 301);

// sys_cloud_gateway_request(req)
syscall!(CLOUD_GATEWAY_REQUEST = 302);

// sys_cloud_gateway_metrics(buf, len)
syscall!(CLOUD_GATEWAY_METRICS = 303);
```

## Appendix C: /proc Filesystem Layout

```
/proc/agentsys/
├── status                     # Human-readable status
├── telemetry                  # JSON telemetry snapshot
├── policies                   # Current policy rules
└── compliance                 # EU AI Act compliance export
```

---

**End of Plan**
