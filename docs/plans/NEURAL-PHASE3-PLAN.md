# Neural Phase 3: Multi-Agent Coordination & Advanced ML
## 3-Week Implementation Plan

**Status:** Ready to start
**Timeline:** 3 weeks (21 days)
**Goal:** Transform SIS kernel from multi-agent architecture to coordinated intelligent system

---

## Executive Summary

This plan implements sophisticated multi-agent coordination and advanced ML techniques using the existing 3 neural agents (Command, Scheduling, Memory) without adding new subsystems. The focus is on the **unique contribution**: cross-agent communication, meta-level coordination, and production-grade ML algorithms.

**Why this approach:**
- ✅ Builds on existing momentum (3 working agents)
- ✅ Novel research contribution (publishable work)
- ✅ Avoids scope creep of new subsystems
- ✅ Demonstrates neural-first kernel innovation
- ✅ Can always add networking/FS later if needed

**What we're NOT doing (to avoid scope creep):**
- ❌ Adding networking subsystem
- ❌ Adding filesystem subsystem
- ❌ Building new VirtIO drivers
- ❌ Implementing TCP/IP stack
- ❌ Any feature unrelated to neural intelligence

---

## Current State

**Existing Neural Agents:**
1. **Command Agent** (3→8→2): Predicts shell command success/failure
2. **Scheduling Agent** (3→8→2): Autonomous priority adjustments for operators
3. **Memory Agent** (4→8→2): OOM prevention and fragmentation detection

**Current Capabilities:**
- ✅ Independent agents make subsystem-specific predictions
- ✅ Autonomous actions (priority boosts, memory warnings)
- ✅ User feedback and online learning
- ✅ Audit logging for observability

**Current Limitations:**
- ❌ No cross-agent communication
- ❌ No global coordination
- ❌ No meta-level optimization
- ❌ Basic gradient descent (no momentum)
- ❌ Uncalibrated confidence scores

---

## Week 1: Cross-Agent Communication Infrastructure

### Objective
Build message-passing layer enabling agents to share information and coordinate decisions.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Agent Message Bus                        │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Ring Buffer: 32 messages, lock-protected          │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │ publish            │ publish            │ publish
         │                    │                    │
    ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
    │ Memory  │         │Schedule │         │ Command │
    │  Agent  │◄───────►│  Agent  │◄───────►│  Agent  │
    └─────────┘  read   └─────────┘  read   └─────────┘
```

### Day 1-2: Message Protocol Design

**File:** `crates/kernel/src/agent_bus.rs` (NEW)

```rust
/// Agent message types for cross-subsystem coordination
#[derive(Copy, Clone, Debug)]
pub enum AgentMessage {
    // Memory agent messages
    MemoryPressure {
        level: u8,           // 0-100 pressure level
        fragmentation: u8,   // 0-100 fragmentation %
        confidence: u16,     // 0-1000 milli-units
        timestamp_us: u64,
    },
    MemoryCompactionNeeded {
        urgency: u8,         // 0=low, 100=critical
        confidence: u16,
        timestamp_us: u64,
    },
    MemoryHealthy {
        headroom_percent: u8,
        timestamp_us: u64,
    },

    // Scheduling agent messages
    SchedulingLoadHigh {
        deadline_misses: u8,   // Count in last window
        avg_latency_us: u32,   // EMA latency
        confidence: u16,
        timestamp_us: u64,
    },
    SchedulingCapacityAvailable {
        free_slots: u8,
        timestamp_us: u64,
    },
    SchedulingCriticalOpsRunning {
        count: u8,
        min_priority: u8,
        timestamp_us: u64,
    },

    // Command agent messages
    CommandHeavyPredicted {
        command_hash: u32,     // Hash of command name
        confidence: u16,
        timestamp_us: u64,
    },
    CommandRapidStream {
        rate_per_sec: u32,     // Commands/second
        timestamp_us: u64,
    },
    CommandIdle {
        seconds: u32,
        timestamp_us: u64,
    },
}

/// Ring buffer for agent messages
pub struct AgentMessageBus {
    messages: [Option<AgentMessage>; 32],
    write_idx: usize,
    read_idx: usize,
    filled: bool,
}

impl AgentMessageBus {
    pub const fn new() -> Self {
        Self {
            messages: [None; 32],
            write_idx: 0,
            read_idx: 0,
            filled: false,
        }
    }

    /// Publish message to bus (called by agents)
    pub fn publish(&mut self, msg: AgentMessage) {
        self.messages[self.write_idx] = Some(msg);
        self.write_idx = (self.write_idx + 1) % 32;
        if self.write_idx == 0 {
            self.filled = true;
        }
    }

    /// Read all messages since last read (called by agents)
    pub fn read_messages(&mut self) -> impl Iterator<Item = &AgentMessage> {
        let start = self.read_idx;
        let end = self.write_idx;
        self.read_idx = self.write_idx;

        // Return slice iterator (handles wraparound)
        self.messages[start..end].iter().filter_map(|m| m.as_ref())
    }

    /// Get last N messages for debugging
    pub fn last_n(&self, n: usize) -> impl Iterator<Item = &AgentMessage> {
        let count = if self.filled { 32 } else { self.write_idx };
        let start = count.saturating_sub(n);
        self.messages[start..count].iter().filter_map(|m| m.as_ref())
    }
}

static AGENT_BUS: Mutex<AgentMessageBus> = Mutex::new(AgentMessageBus::new());
```

**Tasks:**
- [ ] Create `crates/kernel/src/agent_bus.rs`
- [ ] Define `AgentMessage` enum with all message types
- [ ] Implement `AgentMessageBus` ring buffer
- [ ] Add `static AGENT_BUS` global instance
- [ ] Write unit tests for ring buffer wraparound

**Success Criteria:**
- Can publish 100 messages without panics
- Ring buffer correctly wraps at 32 messages
- Read iterator returns messages in order

### Day 3-4: Message Broadcasting

**File:** `crates/kernel/src/neural.rs` (MODIFY)

```rust
// Add to memory agent prediction
pub fn predict_memory_health() -> (u16, bool, bool) {
    // ... existing prediction code ...

    let (conf, oom_risk, compact_needed) = /* ... */;

    // Publish to bus
    let mut bus = crate::agent_bus::AGENT_BUS.lock();
    if oom_risk && conf > 300 {
        bus.publish(AgentMessage::MemoryPressure {
            level: (100 - telem.free_memory_percent) as u8,
            fragmentation: telem.fragmentation_level as u8,
            confidence: conf,
            timestamp_us: crate::graph::cycles_to_ns(crate::graph::now_cycles()) / 1000,
        });
    } else if compact_needed && conf > 300 {
        bus.publish(AgentMessage::MemoryCompactionNeeded {
            urgency: telem.fragmentation_level as u8,
            confidence: conf,
            timestamp_us: crate::graph::cycles_to_ns(crate::graph::now_cycles()) / 1000,
        });
    } else {
        bus.publish(AgentMessage::MemoryHealthy {
            headroom_percent: telem.free_memory_percent as u8,
            timestamp_us: crate::graph::cycles_to_ns(crate::graph::now_cycles()) / 1000,
        });
    }
    drop(bus);

    (conf, oom_risk, compact_needed)
}

// Similar changes for scheduling agent and command agent
pub fn autonomous_scheduling_decision(/* ... */) {
    // ... existing code ...

    // Publish scheduling state
    let mut bus = crate::agent_bus::AGENT_BUS.lock();
    if unhealthy && conf > 700 {
        bus.publish(AgentMessage::SchedulingLoadHigh {
            deadline_misses: deadline_miss_count,
            avg_latency_us: ema_latency,
            confidence: conf,
            timestamp_us: /* ... */,
        });
    }
    // ...
}
```

**Tasks:**
- [ ] Add message publishing to `predict_memory_health()`
- [ ] Add message publishing to autonomous scheduling code
- [ ] Add message publishing to command prediction
- [ ] Add timestamp calculation helper
- [ ] Test: Run `memctl stress` and verify messages published

**Success Criteria:**
- Memory agent publishes messages during stress test
- Scheduling agent publishes during graphdemo
- Command agent publishes on heavy commands
- Timestamps are monotonically increasing

### Day 4-5: Message Consumption

**File:** `crates/kernel/src/neural.rs` (MODIFY)

```rust
/// Handle incoming messages from other agents (called by each agent)
pub fn handle_agent_messages_for_scheduling() {
    let mut bus = crate::agent_bus::AGENT_BUS.lock();

    for msg in bus.read_messages() {
        match msg {
            AgentMessage::MemoryPressure { level, confidence, .. } if level > 70 => {
                unsafe { crate::uart_print(b"[SCHED AGENT] Memory pressure detected, lowering non-critical priorities\n"); }
                // Lower priority of operators with priority < 100
                // (Implementation in graph.rs)
                crate::graph::lower_noncritical_operator_priorities();
                crate::trace::metric_kv("sched_memory_response", *level as usize);
            }

            AgentMessage::MemoryCompactionNeeded { urgency, .. } => {
                unsafe { crate::uart_print(b"[SCHED AGENT] Compaction needed, pausing low-priority ops\n"); }
                // Temporarily pause operators with priority < 50
                crate::graph::pause_low_priority_operators();
                crate::trace::metric_kv("sched_pause_for_compact", *urgency as usize);
            }

            AgentMessage::CommandHeavyPredicted { confidence, .. } if *confidence > 700 => {
                unsafe { crate::uart_print(b"[SCHED AGENT] Heavy command predicted, preemptively boosting capacity\n"); }
                // Increase available operator slots
                crate::trace::metric_kv("sched_preempt_boost", *confidence as usize);
            }

            _ => {}
        }
    }
}

/// Handle incoming messages for memory agent
pub fn handle_agent_messages_for_memory() {
    let mut bus = crate::agent_bus::AGENT_BUS.lock();

    for msg in bus.read_messages() {
        match msg {
            AgentMessage::SchedulingLoadHigh { .. } => {
                unsafe { crate::uart_print(b"[MEMORY AGENT] High scheduling load, deferring compaction\n"); }
                // Don't compact now, wait for load to decrease
                crate::trace::metric_kv("memory_defer_compact", 1);
            }

            AgentMessage::CommandHeavyPredicted { .. } => {
                unsafe { crate::uart_print(b"[MEMORY AGENT] Heavy command predicted, reserving headroom\n"); }
                // Increase reservation threshold
                crate::trace::metric_kv("memory_reserve_headroom", 1);
            }

            _ => {}
        }
    }
}
```

**File:** `crates/kernel/src/graph.rs` (MODIFY - add helper functions)

```rust
/// Lower priority of non-critical operators (priority < 100)
pub fn lower_noncritical_operator_priorities() {
    // Implementation: iterate operators, lower priority by 10
}

/// Pause operators with priority < 50
pub fn pause_low_priority_operators() {
    // Implementation: mark operators as paused
}
```

**Tasks:**
- [ ] Implement `handle_agent_messages_for_scheduling()`
- [ ] Implement `handle_agent_messages_for_memory()`
- [ ] Implement `handle_agent_messages_for_command()`
- [ ] Add helper functions in `graph.rs`
- [ ] Call message handlers in appropriate places (before predictions)

**Success Criteria:**
- Memory agent responds to scheduling messages
- Scheduling agent responds to memory messages
- Console shows cross-agent coordination logs

### Day 5-7: Integration & Testing

**File:** `crates/kernel/src/shell.rs` (ADD new command)

```rust
fn cmd_agentctl(&self, args: &[&str]) {
    if args.is_empty() {
        unsafe { crate::uart_print(b"Usage: agentctl <bus|stats|clear>\n"); }
        return;
    }

    match args[0] {
        "bus" => {
            // Show last 32 messages from agent bus
            let bus = crate::agent_bus::AGENT_BUS.lock();
            unsafe { crate::uart_print(b"[AGENT BUS] Last 32 messages:\n"); }

            for (i, msg) in bus.last_n(32).enumerate() {
                unsafe { crate::uart_print(b"  "); }
                self.print_number_simple(i as u64);
                unsafe { crate::uart_print(b". "); }

                match msg {
                    AgentMessage::MemoryPressure { level, confidence, .. } => {
                        unsafe { crate::uart_print(b"MEMORY PRESSURE level="); }
                        self.print_number_simple(*level as u64);
                        unsafe { crate::uart_print(b" conf="); }
                        self.print_number_simple(*confidence as u64);
                        unsafe { crate::uart_print(b"\n"); }
                    }
                    AgentMessage::SchedulingLoadHigh { deadline_misses, confidence, .. } => {
                        unsafe { crate::uart_print(b"SCHED LOAD HIGH misses="); }
                        self.print_number_simple(*deadline_misses as u64);
                        unsafe { crate::uart_print(b" conf="); }
                        self.print_number_simple(*confidence as u64);
                        unsafe { crate::uart_print(b"\n"); }
                    }
                    // ... other message types
                    _ => {}
                }
            }
        }

        "stats" => {
            // Show message counts by type
            unsafe { crate::uart_print(b"[AGENT BUS] Message statistics:\n"); }
            // Count each message type in last 32 messages
        }

        "clear" => {
            // Clear message bus (for testing)
            let mut bus = crate::agent_bus::AGENT_BUS.lock();
            // Reset indices
        }

        _ => unsafe { crate::uart_print(b"Unknown subcommand\n"); }
    }
}
```

**Testing Scenarios:**

```bash
# Test 1: Memory pressure triggers scheduling response
memctl stress 200
agentctl bus
# Expected: MemoryPressure messages, then Scheduling responses

# Test 2: Heavy command triggers coordination
graphdemo
agentctl bus
# Expected: CommandHeavyPredicted, then Memory/Scheduling responses

# Test 3: Concurrent stress
graphdemo & memctl stress 100 &
agentctl bus
# Expected: Multiple agents coordinating

# Test 4: Message statistics
agentctl stats
# Expected: Count of each message type
```

**Tasks:**
- [ ] Add `cmd_agentctl()` to shell
- [ ] Implement message display in `agentctl bus`
- [ ] Write test scenarios document
- [ ] Run all test scenarios and verify coordination
- [ ] Measure latency of message handling (<1ms)

**Success Criteria:**
- All 4 test scenarios show correct coordination
- Message handling adds <1ms latency
- No deadlocks or panics
- Console logs show agent responses

**Week 1 Deliverables:**
- ✅ Working agent message bus
- ✅ 3 agents publishing messages
- ✅ 3 agents consuming and responding to messages
- ✅ `agentctl bus` command for debugging
- ✅ Test scenarios demonstrating coordination
- ✅ Documentation of message protocol

---

## Week 2: Meta-Agent Coordination

### Objective
Build supervising meta-agent that coordinates all subsystems and makes global optimization decisions.

### Architecture Overview

```
                    ┌─────────────────────────┐
                    │     META-AGENT          │
                    │   12 inputs → 16 → 3    │
                    │                         │
                    │  Global Coordination    │
                    │  System Health Monitor  │
                    └────────┬────────────────┘
                             │
                 ┌───────────┼───────────┐
                 ▼           ▼           ▼
         ┌───────────┐ ┌──────────┐ ┌──────────┐
         │  Memory   │ │Schedule  │ │ Command  │
         │   Agent   │ │  Agent   │ │  Agent   │
         └───────────┘ └──────────┘ └──────────┘
```

### Day 1-3: Meta-Agent Architecture

**File:** `crates/kernel/src/meta_agent.rs` (NEW)

```rust
use crate::neural::{NeuralAgent, MAX_IN, MAX_OUT};
use spin::Mutex;

/// Meta-agent for global system coordination
pub struct MetaAgent {
    network: NeuralAgent,  // 12→16→3 architecture
    state: MetaState,
    last_decision_ts: u64,
    decision_interval_us: u64,  // Don't decide too frequently
}

/// Global system state aggregated from all subsystems
#[derive(Copy, Clone)]
pub struct MetaState {
    // Memory telemetry (from memory agent)
    memory_pressure: u8,        // 0-100
    memory_fragmentation: u8,   // 0-100
    memory_alloc_rate: u8,      // 0-100 normalized
    memory_failures: u8,        // 0-10

    // Scheduling telemetry (from scheduling agent)
    scheduling_load: u8,        // 0-100
    deadline_misses: u8,        // Count in window
    operator_latency_ms: u8,    // 0-10ms normalized
    critical_ops_count: u8,     // Number of critical ops running

    // Command telemetry (from command agent)
    command_rate: u8,           // Commands per minute (0-100)
    command_heaviness: u8,      // Average predicted complexity (0-100)
    prediction_accuracy: u8,    // Recent prediction accuracy (0-100)
    rapid_stream_detected: u8,  // Boolean (0 or 100)
}

/// Meta-agent decisions
#[derive(Copy, Clone, Debug)]
pub struct MetaDecision {
    memory_directive: MetaDirective,
    scheduling_directive: MetaDirective,
    command_directive: MetaDirective,
    confidence: u16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MetaDirective {
    // Interpreted from Q8.8 output values
    StrongNegative,  // -1000 to -500: strong action needed
    Negative,        // -500 to -100: moderate action
    Neutral,         // -100 to 100: no action
    Positive,        // 100 to 500: moderate positive action
    StrongPositive,  // 500 to 1000: strong positive action
}

impl MetaAgent {
    pub const fn new() -> Self {
        Self {
            network: NeuralAgent::new(),
            state: MetaState::zero(),
            last_decision_ts: 0,
            decision_interval_us: 100_000,  // Decide every 100ms
        }
    }

    /// Initialize meta-agent with 12→16→3 dimensions
    pub fn init(&mut self) {
        self.network.set_dims(12, 16, 3);
        self.network.infer_count = 1;  // Prevent lazy reset
    }

    /// Update state from all subsystem agents
    pub fn update_state(&mut self) {
        // Collect from memory agent
        let mem_telem = crate::neural::get_memory_telemetry();
        self.state.memory_pressure = (100 - mem_telem.free_memory_percent) as u8;
        self.state.memory_fragmentation = mem_telem.fragmentation_level as u8;
        self.state.memory_alloc_rate = (mem_telem.allocation_rate / 10).min(100) as u8;
        self.state.memory_failures = mem_telem.recent_failures.min(10) as u8;

        // Collect from scheduling agent
        let sched_state = crate::neural::get_scheduling_state();
        self.state.scheduling_load = sched_state.load_percent;
        self.state.deadline_misses = sched_state.recent_misses;
        self.state.operator_latency_ms = (sched_state.avg_latency_us / 1000).min(10) as u8;
        self.state.critical_ops_count = sched_state.critical_count;

        // Collect from command agent
        let cmd_state = crate::neural::get_command_state();
        self.state.command_rate = cmd_state.rate_per_min.min(100) as u8;
        self.state.command_heaviness = cmd_state.avg_heaviness;
        self.state.prediction_accuracy = cmd_state.recent_accuracy;
        self.state.rapid_stream_detected = if cmd_state.rapid_stream { 100 } else { 0 };
    }

    /// Run meta-agent inference and return global decision
    pub fn infer(&mut self) -> MetaDecision {
        // Convert state to Q8.8 inputs (0-256 range)
        let inputs_q88 = [
            ((self.state.memory_pressure as u32 * 256 / 100).min(256)) as i16,
            ((self.state.memory_fragmentation as u32 * 256 / 100).min(256)) as i16,
            ((self.state.memory_alloc_rate as u32 * 256 / 100).min(256)) as i16,
            ((self.state.memory_failures as u32 * 256 / 10).min(256)) as i16,

            ((self.state.scheduling_load as u32 * 256 / 100).min(256)) as i16,
            ((self.state.deadline_misses as u32 * 256 / 10).min(256)) as i16,
            ((self.state.operator_latency_ms as u32 * 256 / 10).min(256)) as i16,
            ((self.state.critical_ops_count as u32 * 256 / 10).min(256)) as i16,

            ((self.state.command_rate as u32 * 256 / 100).min(256)) as i16,
            ((self.state.command_heaviness as u32 * 256 / 100).min(256)) as i16,
            ((self.state.prediction_accuracy as u32 * 256 / 100).min(256)) as i16,
            ((self.state.rapid_stream_detected as u32 * 256 / 100).min(256)) as i16,
        ];

        // Run inference
        let out_len = self.network.infer(&inputs_q88);

        if out_len < 3 {
            return MetaDecision::neutral();
        }

        // Extract outputs and convert to milli-units
        let memory_out = ((self.network.last_out[0] as i32) * 1000 / 256).clamp(-1000, 1000);
        let sched_out = ((self.network.last_out[1] as i32) * 1000 / 256).clamp(-1000, 1000);
        let cmd_out = ((self.network.last_out[2] as i32) * 1000 / 256).clamp(-1000, 1000);

        // Compute confidence (average absolute value)
        let confidence = ((memory_out.abs() + sched_out.abs() + cmd_out.abs()) / 3).min(1000) as u16;

        MetaDecision {
            memory_directive: Self::interpret_output(memory_out),
            scheduling_directive: Self::interpret_output(sched_out),
            command_directive: Self::interpret_output(cmd_out),
            confidence,
        }
    }

    fn interpret_output(value: i32) -> MetaDirective {
        match value {
            v if v < -500 => MetaDirective::StrongNegative,
            v if v < -100 => MetaDirective::Negative,
            v if v > 500 => MetaDirective::StrongPositive,
            v if v > 100 => MetaDirective::Positive,
            _ => MetaDirective::Neutral,
        }
    }
}

impl MetaState {
    const fn zero() -> Self {
        Self {
            memory_pressure: 0,
            memory_fragmentation: 0,
            memory_alloc_rate: 0,
            memory_failures: 0,
            scheduling_load: 0,
            deadline_misses: 0,
            operator_latency_ms: 0,
            critical_ops_count: 0,
            command_rate: 0,
            command_heaviness: 0,
            prediction_accuracy: 0,
            rapid_stream_detected: 0,
        }
    }
}

impl MetaDecision {
    fn neutral() -> Self {
        Self {
            memory_directive: MetaDirective::Neutral,
            scheduling_directive: MetaDirective::Neutral,
            command_directive: MetaDirective::Neutral,
            confidence: 0,
        }
    }
}

static META_AGENT: Mutex<MetaAgent> = Mutex::new(MetaAgent::new());

/// Initialize meta-agent at boot
pub fn init_meta_agent() {
    let mut agent = META_AGENT.lock();
    agent.init();
    crate::trace::metric_kv("meta_agent_init", 1);
}

/// Get current meta-agent state (for debugging)
pub fn get_meta_state() -> MetaState {
    META_AGENT.lock().state
}
```

**File:** `crates/kernel/src/neural.rs` (ADD helper functions)

```rust
// Add public helper functions to expose agent state

pub fn get_memory_telemetry() -> MemoryTelemetry {
    let telem = MEMORY_TELEMETRY.lock();
    MemoryTelemetry {
        free_memory_percent: telem.free_memory_percent,
        allocation_rate: telem.allocation_rate,
        fragmentation_level: telem.fragmentation_level,
        recent_failures: telem.recent_failures,
        last_update_ns: telem.last_update_ns,
        prev_alloc_count: telem.prev_alloc_count,
    }
}

pub struct SchedulingState {
    pub load_percent: u8,
    pub recent_misses: u8,
    pub avg_latency_us: u32,
    pub critical_count: u8,
}

pub fn get_scheduling_state() -> SchedulingState {
    // Collect from SCHED_AUDIT or similar
    // TODO: Implement based on existing scheduling telemetry
    SchedulingState {
        load_percent: 0,
        recent_misses: 0,
        avg_latency_us: 0,
        critical_count: 0,
    }
}

pub struct CommandState {
    pub rate_per_min: u8,
    pub avg_heaviness: u8,
    pub recent_accuracy: u8,
    pub rapid_stream: bool,
}

pub fn get_command_state() -> CommandState {
    // Collect from NN_AUDIT or similar
    // TODO: Implement based on existing command audit
    CommandState {
        rate_per_min: 0,
        avg_heaviness: 50,
        recent_accuracy: 75,
        rapid_stream: false,
    }
}
```

**Tasks:**
- [ ] Create `crates/kernel/src/meta_agent.rs`
- [ ] Implement `MetaAgent` struct with 12→16→3 architecture
- [ ] Implement `MetaState` aggregation
- [ ] Implement `MetaDecision` interpretation
- [ ] Add telemetry helper functions in `neural.rs`
- [ ] Add `mod meta_agent;` to `lib.rs`

**Success Criteria:**
- Meta-agent initializes without panics
- State collection gathers data from all 3 agents
- Inference runs and produces 3 outputs
- Directives correctly interpreted from outputs

### Day 4-5: Meta-Agent Decision Execution

**File:** `crates/kernel/src/meta_agent.rs` (ADD)

```rust
/// Execute meta-agent decision across all subsystems
pub fn execute_meta_decision(decision: &MetaDecision) {
    unsafe { crate::uart_print(b"[META-AGENT] Executing decision (conf="); }
    crate::shell::print_number_simple(decision.confidence as u64);
    unsafe { crate::uart_print(b"/1000)\n"); }

    // Execute memory directive
    match decision.memory_directive {
        MetaDirective::StrongNegative => {
            unsafe { crate::uart_print(b"[META→MEMORY] URGENT: Force compaction now\n"); }
            // Trigger immediate compaction
            crate::trace::metric_kv("meta_memory_force_compact", 1);
        }
        MetaDirective::Negative => {
            unsafe { crate::uart_print(b"[META→MEMORY] Schedule compaction soon\n"); }
            crate::trace::metric_kv("meta_memory_schedule_compact", 1);
        }
        MetaDirective::Positive => {
            unsafe { crate::uart_print(b"[META→MEMORY] Reserve headroom\n"); }
            crate::trace::metric_kv("meta_memory_reserve", 1);
        }
        MetaDirective::StrongPositive => {
            unsafe { crate::uart_print(b"[META→MEMORY] Aggressive allocation allowed\n"); }
            crate::trace::metric_kv("meta_memory_aggressive", 1);
        }
        MetaDirective::Neutral => {}
    }

    // Execute scheduling directive
    match decision.scheduling_directive {
        MetaDirective::StrongNegative => {
            unsafe { crate::uart_print(b"[META→SCHED] URGENT: Pause non-critical ops\n"); }
            crate::graph::pause_low_priority_operators();
            crate::trace::metric_kv("meta_sched_pause", 1);
        }
        MetaDirective::Negative => {
            unsafe { crate::uart_print(b"[META→SCHED] Lower non-critical priorities\n"); }
            crate::graph::lower_noncritical_operator_priorities();
            crate::trace::metric_kv("meta_sched_lower", 1);
        }
        MetaDirective::Positive => {
            unsafe { crate::uart_print(b"[META→SCHED] Boost critical ops\n"); }
            crate::trace::metric_kv("meta_sched_boost", 1);
        }
        MetaDirective::StrongPositive => {
            unsafe { crate::uart_print(b"[META→SCHED] Aggressive parallelism\n"); }
            crate::trace::metric_kv("meta_sched_aggressive", 1);
        }
        MetaDirective::Neutral => {}
    }

    // Execute command directive
    match decision.command_directive {
        MetaDirective::StrongNegative => {
            unsafe { crate::uart_print(b"[META→COMMAND] URGENT: Throttle commands\n"); }
            crate::trace::metric_kv("meta_cmd_throttle", 1);
        }
        MetaDirective::Negative => {
            unsafe { crate::uart_print(b"[META→COMMAND] Rate limit commands\n"); }
            crate::trace::metric_kv("meta_cmd_ratelimit", 1);
        }
        MetaDirective::Positive => {
            unsafe { crate::uart_print(b"[META→COMMAND] Allow normal throughput\n"); }
            crate::trace::metric_kv("meta_cmd_allow", 1);
        }
        MetaDirective::StrongPositive => {
            unsafe { crate::uart_print(b"[META→COMMAND] Encourage heavy commands\n"); }
            crate::trace::metric_kv("meta_cmd_encourage", 1);
        }
        MetaDirective::Neutral => {}
    }
}

/// Meta-agent coordination tick (call periodically)
pub fn meta_agent_tick() {
    let mut agent = META_AGENT.lock();

    // Check if enough time has passed since last decision
    let now_us = crate::graph::cycles_to_ns(crate::graph::now_cycles()) / 1000;
    if now_us - agent.last_decision_ts < agent.decision_interval_us {
        return;  // Too soon to decide again
    }

    // Update state from all agents
    agent.update_state();

    // Run inference
    let decision = agent.infer();
    agent.last_decision_ts = now_us;
    drop(agent);

    // Only execute if confidence is sufficient
    const MIN_CONFIDENCE: u16 = 500;  // 50%
    if decision.confidence >= MIN_CONFIDENCE {
        execute_meta_decision(&decision);

        // Log decision
        crate::trace::metric_kv("meta_decision_conf", decision.confidence as usize);
        crate::trace::metric_kv("meta_decisions_total", 1);
    }
}
```

**File:** `crates/kernel/src/main.rs` (MODIFY - add meta-agent init)

```rust
// After memory agent init
super::uart_print(b"META-AGENT: INIT\n");
crate::meta_agent::init_meta_agent();
super::uart_print(b"META-AGENT: READY\n");
```

**Tasks:**
- [ ] Implement `execute_meta_decision()`
- [ ] Implement `meta_agent_tick()`
- [ ] Add meta-agent initialization in `main.rs`
- [ ] Add periodic calling of `meta_agent_tick()` (every 100ms)
- [ ] Test decision execution with debug output

**Success Criteria:**
- Meta-agent makes decisions every 100ms
- Decisions execute correctly (console shows actions)
- Confidence threshold prevents low-confidence actions
- Metrics track decision frequency

### Day 6-7: Shell Integration & Testing

**File:** `crates/kernel/src/shell.rs` (ADD new command)

```rust
fn cmd_metaclassctl(&self, args: &[&str]) {
    if args.is_empty() {
        unsafe { crate::uart_print(b"Usage: metaclassctl <status|force|interval|threshold>\n"); }
        return;
    }

    match args[0] {
        "status" => {
            // Show meta-agent state and last decision
            let state = crate::meta_agent::get_meta_state();

            unsafe { crate::uart_print(b"[META-AGENT] Current State:\n"); }
            unsafe { crate::uart_print(b"  Memory: pressure="); }
            self.print_number_simple(state.memory_pressure as u64);
            unsafe { crate::uart_print(b"% fragmentation="); }
            self.print_number_simple(state.memory_fragmentation as u64);
            unsafe { crate::uart_print(b"%\n"); }

            unsafe { crate::uart_print(b"  Scheduling: load="); }
            self.print_number_simple(state.scheduling_load as u64);
            unsafe { crate::uart_print(b"% misses="); }
            self.print_number_simple(state.deadline_misses as u64);
            unsafe { crate::uart_print(b"\n"); }

            unsafe { crate::uart_print(b"  Command: rate="); }
            self.print_number_simple(state.command_rate as u64);
            unsafe { crate::uart_print(b"/min heaviness="); }
            self.print_number_simple(state.command_heaviness as u64);
            unsafe { crate::uart_print(b"%\n"); }
        }

        "force" => {
            // Force meta-agent decision now (ignore interval)
            unsafe { crate::uart_print(b"[META-AGENT] Forcing decision...\n"); }
            crate::meta_agent::force_decision();
        }

        "interval" => {
            // Set decision interval
            if args.len() < 2 {
                unsafe { crate::uart_print(b"Usage: metaclassctl interval <ms>\n"); }
                return;
            }
            let ms = args[1].parse::<u64>().unwrap_or(100);
            crate::meta_agent::set_decision_interval(ms * 1000);
            unsafe { crate::uart_print(b"[META-AGENT] Interval set to "); }
            self.print_number_simple(ms);
            unsafe { crate::uart_print(b"ms\n"); }
        }

        "threshold" => {
            // Set confidence threshold
            if args.len() < 2 {
                unsafe { crate::uart_print(b"Usage: metaclassctl threshold <0-1000>\n"); }
                return;
            }
            let threshold = args[1].parse::<u16>().unwrap_or(500);
            crate::meta_agent::set_confidence_threshold(threshold);
            unsafe { crate::uart_print(b"[META-AGENT] Confidence threshold: "); }
            self.print_number_simple(threshold as u64);
            unsafe { crate::uart_print(b"/1000\n"); }
        }

        "export" => {
            // Export complete telemetry as JSON for external visualization
            unsafe { crate::uart_print(b"[META-AGENT] JSON Export:\n"); }
            crate::meta_agent::export_telemetry_json();
        }

        _ => unsafe { crate::uart_print(b"Unknown subcommand\n"); }
    }
}
```

**File:** `crates/kernel/src/meta_agent.rs` (ADD JSON export)

```rust
/// Export complete system telemetry as JSON for external visualization/dashboards
pub fn export_telemetry_json() {
    let state = get_meta_state();
    let agent = META_AGENT.lock();

    unsafe {
        crate::uart_print(b"{\n");
        crate::uart_print(b"  \"timestamp_us\": ");
    }
    let now_us = crate::graph::cycles_to_ns(crate::graph::now_cycles()) / 1000;
    crate::shell::print_number_simple(now_us);

    unsafe {
        crate::uart_print(b",\n  \"memory\": {\n");
        crate::uart_print(b"    \"pressure\": ");
    }
    crate::shell::print_number_simple(state.memory_pressure as u64);
    unsafe {
        crate::uart_print(b",\n    \"fragmentation\": ");
    }
    crate::shell::print_number_simple(state.memory_fragmentation as u64);
    unsafe {
        crate::uart_print(b",\n    \"alloc_rate\": ");
    }
    crate::shell::print_number_simple(state.memory_alloc_rate as u64);
    unsafe {
        crate::uart_print(b",\n    \"failures\": ");
    }
    crate::shell::print_number_simple(state.memory_failures as u64);
    unsafe {
        crate::uart_print(b"\n  },\n  \"scheduling\": {\n");
        crate::uart_print(b"    \"load\": ");
    }
    crate::shell::print_number_simple(state.scheduling_load as u64);
    unsafe {
        crate::uart_print(b",\n    \"deadline_misses\": ");
    }
    crate::shell::print_number_simple(state.deadline_misses as u64);
    unsafe {
        crate::uart_print(b",\n    \"latency_ms\": ");
    }
    crate::shell::print_number_simple(state.operator_latency_ms as u64);
    unsafe {
        crate::uart_print(b",\n    \"critical_ops\": ");
    }
    crate::shell::print_number_simple(state.critical_ops_count as u64);
    unsafe {
        crate::uart_print(b"\n  },\n  \"command\": {\n");
        crate::uart_print(b"    \"rate\": ");
    }
    crate::shell::print_number_simple(state.command_rate as u64);
    unsafe {
        crate::uart_print(b",\n    \"heaviness\": ");
    }
    crate::shell::print_number_simple(state.command_heaviness as u64);
    unsafe {
        crate::uart_print(b",\n    \"accuracy\": ");
    }
    crate::shell::print_number_simple(state.prediction_accuracy as u64);
    unsafe {
        crate::uart_print(b",\n    \"rapid_stream\": ");
        crate::uart_print(if state.rapid_stream_detected > 0 { b"true" } else { b"false" });
        crate::uart_print(b"\n  },\n  \"meta_decision\": {\n");
        crate::uart_print(b"    \"last_ts\": ");
    }
    crate::shell::print_number_simple(agent.last_decision_ts);
    unsafe {
        crate::uart_print(b",\n    \"interval_us\": ");
    }
    crate::shell::print_number_simple(agent.decision_interval_us);
    unsafe {
        crate::uart_print(b"\n  }\n}\n");
    }
}
```

**Testing Scenarios:**

```bash
# Test 1: Meta-agent status
metaclassctl status
# Expected: Shows aggregated state from all agents

# Test 1.5: JSON export for dashboard
metaclassctl export > /tmp/telemetry.json
# Expected: Valid JSON snapshot of all subsystem telemetry
# Can be parsed by external tools for visualization

# Test 2: Trigger coordinated stress
graphdemo & memctl stress 200 &
# Expected: Meta-agent coordinates response across subsystems

# Test 3: Force decision
metaclassctl force
# Expected: Immediate decision with actions logged

# Test 4: Adjust sensitivity
metaclassctl threshold 300
memctl stress 100
# Expected: More frequent decisions (lower threshold)

# Test 5: System overload scenario
# Run heavy workload and watch meta-agent coordinate
for i in 1 2 3; do graphdemo & done
memctl stress 500 &
metaclassctl status
# Expected: Meta-agent detects overload, activates defensive measures
```

**Tasks:**
- [ ] Add `cmd_metaclassctl()` to shell
- [ ] Implement meta-agent status display
- [ ] Add `force_decision()` helper
- [ ] Add configuration functions (interval, threshold)
- [ ] **Add telemetry export: `metaclassctl export` (JSON snapshot)**
- [ ] **Add dynamic interval tuning: `metaclassctl interval <ms>`**
- [ ] Run all test scenarios
- [ ] Verify coordination across scenarios

**Success Criteria:**
- All test scenarios show correct meta-agent behavior
- Meta-agent coordinates 3 subsystems simultaneously
- Console logs show decision rationale
- Metrics track meta-agent activity
- **JSON export works for external visualization**
- **Interval can be adjusted at runtime (10ms-1000ms range)**

**Week 2 Deliverables:**
- ✅ Working meta-agent with 12→16→3 architecture
- ✅ Global state aggregation from all subsystems
- ✅ Autonomous decision execution
- ✅ `metaclassctl` command for control and debugging
- ✅ Test scenarios demonstrating global coordination
- ✅ Documentation of meta-agent decisions

---

## Week 3: Advanced ML Techniques

### Objective
Enhance neural network intelligence with production-grade ML algorithms.

### Day 1-2: Momentum-Based Gradient Descent

**File:** `crates/kernel/src/optimizer.rs` (NEW)

```rust
use crate::neural::{NeuralAgent, MAX_IN, MAX_H, MAX_OUT};

/// Gradient descent optimizer with momentum
pub struct Optimizer {
    learning_rate: i16,   // Q8.8 (e.g., 0.01 = 2, 0.1 = 25)
    momentum: i16,        // Q8.8 (e.g., 0.9 = 230)

    // Velocity buffers (same shape as weights)
    velocity_w1: [[i16; MAX_IN]; MAX_H],
    velocity_b1: [i16; MAX_H],
    velocity_w2: [[i16; MAX_H]; MAX_OUT],
    velocity_b2: [i16; MAX_OUT],
}

impl Optimizer {
    pub const fn new() -> Self {
        Self {
            learning_rate: 25,   // 0.1 in Q8.8
            momentum: 230,       // 0.9 in Q8.8
            velocity_w1: [[0; MAX_IN]; MAX_H],
            velocity_b1: [0; MAX_H],
            velocity_w2: [[0; MAX_H]; MAX_OUT],
            velocity_b2: [0; MAX_OUT],
        }
    }

    /// Update network weights with momentum
    /// velocity = momentum * velocity - learning_rate * gradient
    /// weight = weight + velocity
    pub fn update_with_momentum(
        &mut self,
        agent: &mut NeuralAgent,
        gradients: &Gradients
    ) {
        // Update W1 with momentum
        for r in 0..agent.hid_sz {
            for c in 0..agent.in_sz {
                // velocity = momentum * velocity - lr * gradient
                let momentum_term = q88_mul(self.momentum, self.velocity_w1[r][c]);
                let gradient_term = q88_mul(self.learning_rate, gradients.w1[r][c]);
                self.velocity_w1[r][c] = q88_sub(momentum_term, gradient_term);

                // weight = weight + velocity
                agent.w1[r][c] = q88_add(agent.w1[r][c], self.velocity_w1[r][c]);
            }
        }

        // Update B1 with momentum
        for r in 0..agent.hid_sz {
            let momentum_term = q88_mul(self.momentum, self.velocity_b1[r]);
            let gradient_term = q88_mul(self.learning_rate, gradients.b1[r]);
            self.velocity_b1[r] = q88_sub(momentum_term, gradient_term);
            agent.b1[r] = q88_add(agent.b1[r], self.velocity_b1[r]);
        }

        // Update W2 with momentum
        for r in 0..agent.out_sz {
            for c in 0..agent.hid_sz {
                let momentum_term = q88_mul(self.momentum, self.velocity_w2[r][c]);
                let gradient_term = q88_mul(self.learning_rate, gradients.w2[r][c]);
                self.velocity_w2[r][c] = q88_sub(momentum_term, gradient_term);
                agent.w2[r][c] = q88_add(agent.w2[r][c], self.velocity_w2[r][c]);
            }
        }

        // Update B2 with momentum
        for r in 0..agent.out_sz {
            let momentum_term = q88_mul(self.momentum, self.velocity_b2[r]);
            let gradient_term = q88_mul(self.learning_rate, gradients.b2[r]);
            self.velocity_b2[r] = q88_sub(momentum_term, gradient_term);
            agent.b2[r] = q88_add(agent.b2[r], self.velocity_b2[r]);
        }
    }

    /// Set learning rate (Q8.8 format)
    pub fn set_learning_rate(&mut self, lr: i16) {
        self.learning_rate = lr;
    }

    /// Set momentum (Q8.8 format)
    pub fn set_momentum(&mut self, m: i16) {
        self.momentum = m;
    }
}

/// Computed gradients (same shape as network)
pub struct Gradients {
    pub w1: [[i16; MAX_IN]; MAX_H],
    pub b1: [i16; MAX_H],
    pub w2: [[i16; MAX_H]; MAX_OUT],
    pub b2: [i16; MAX_OUT],
}

// Q8.8 arithmetic helpers
fn q88_add(a: i16, b: i16) -> i16 {
    a.saturating_add(b)
}

fn q88_sub(a: i16, b: i16) -> i16 {
    a.saturating_sub(b)
}

fn q88_mul(a: i16, b: i16) -> i16 {
    let result = (a as i32 * b as i32) >> 8;
    result.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}
```

**File:** `crates/kernel/src/neural.rs` (MODIFY - integrate optimizer)

```rust
use crate::optimizer::Optimizer;

static COMMAND_OPTIMIZER: Mutex<Optimizer> = Mutex::new(Optimizer::new());
static SCHEDULING_OPTIMIZER: Mutex<Optimizer> = Mutex::new(Optimizer::new());
static MEMORY_OPTIMIZER: Mutex<Optimizer> = Mutex::new(Optimizer::new());

// Modify existing train functions to use optimizer
pub fn retrain_from_feedback(count: usize) -> usize {
    // ... existing code ...

    // Compute gradients
    let gradients = compute_gradients(&agent, &inputs, &targets);

    // Apply with momentum
    let mut optimizer = COMMAND_OPTIMIZER.lock();
    optimizer.update_with_momentum(&mut agent, &gradients);
    drop(optimizer);

    // ... rest of existing code ...
}
```

**Tasks:**
- [ ] Create `crates/kernel/src/optimizer.rs`
- [ ] Implement `Optimizer` with momentum
- [ ] Implement gradient computation helpers
- [ ] Integrate optimizer into existing train functions
- [ ] Test training with momentum vs without
- [ ] Measure convergence speed improvement

**Success Criteria:**
- Training converges faster with momentum
- No overflow/underflow in Q8.8 arithmetic
- Weights update smoothly (less oscillation)

### Day 2-3: Confidence Calibration

**File:** `crates/kernel/src/calibration.rs` (NEW)

```rust
use spin::Mutex;

/// Confidence calibration via historical accuracy tracking
pub struct ConfidenceCalibrator {
    bins: [CalibBin; 10],  // 10 bins: [0-100), [100-200), ..., [900-1000]
}

#[derive(Copy, Clone)]
struct CalibBin {
    predictions: u32,      // Total predictions in this confidence range
    correct: u32,          // Correct predictions
    accuracy: u16,         // Calibrated accuracy (0-1000)
}

impl ConfidenceCalibrator {
    pub const fn new() -> Self {
        Self {
            bins: [CalibBin::zero(); 10],
        }
    }

    /// Record prediction outcome
    pub fn record(&mut self, raw_confidence: u16, was_correct: bool) {
        let bin_idx = (raw_confidence / 100).min(9) as usize;
        self.bins[bin_idx].predictions += 1;
        if was_correct {
            self.bins[bin_idx].correct += 1;
        }

        // Update accuracy
        if self.bins[bin_idx].predictions > 0 {
            self.bins[bin_idx].accuracy = (self.bins[bin_idx].correct * 1000
                / self.bins[bin_idx].predictions) as u16;
        }
    }

    /// Calibrate confidence based on historical accuracy
    pub fn calibrate(&self, raw_confidence: u16) -> u16 {
        let bin_idx = (raw_confidence / 100).min(9) as usize;

        // If we have enough data in this bin, use calibrated accuracy
        if self.bins[bin_idx].predictions >= 10 {
            self.bins[bin_idx].accuracy
        } else {
            // Not enough data, return raw confidence
            raw_confidence
        }
    }

    /// Get calibration statistics for debugging
    pub fn get_stats(&self) -> [(u32, u32, u16); 10] {
        let mut stats = [(0u32, 0u32, 0u16); 10];
        for i in 0..10 {
            stats[i] = (
                self.bins[i].predictions,
                self.bins[i].correct,
                self.bins[i].accuracy,
            );
        }
        stats
    }
}

impl CalibBin {
    const fn zero() -> Self {
        Self {
            predictions: 0,
            correct: 0,
            accuracy: 0,
        }
    }
}

static COMMAND_CALIBRATOR: Mutex<ConfidenceCalibrator> = Mutex::new(ConfidenceCalibrator::new());
static SCHEDULING_CALIBRATOR: Mutex<ConfidenceCalibrator> = Mutex::new(ConfidenceCalibrator::new());
static MEMORY_CALIBRATOR: Mutex<ConfidenceCalibrator> = Mutex::new(ConfidenceCalibrator::new());

/// Record command prediction outcome for calibration
pub fn record_command_prediction(confidence: u16, actual_success: bool, predicted_success: bool) {
    let was_correct = actual_success == predicted_success;
    COMMAND_CALIBRATOR.lock().record(confidence, was_correct);
}

/// Get calibrated confidence for command prediction
pub fn calibrate_command_confidence(raw: u16) -> u16 {
    COMMAND_CALIBRATOR.lock().calibrate(raw)
}
```

**File:** `crates/kernel/src/shell.rs` (MODIFY - record outcomes)

```rust
// In command execution loop, after running command
let actual_success = executed_successfully;
let predicted_success = last_prediction.success;
let raw_confidence = last_prediction.confidence;

crate::calibration::record_command_prediction(
    raw_confidence,
    actual_success,
    predicted_success
);
```

**Tasks:**
- [ ] Create `crates/kernel/src/calibration.rs`
- [ ] Implement `ConfidenceCalibrator` with binning
- [ ] Add outcome recording in command execution
- [ ] Add outcome recording in scheduling decisions
- [ ] Add outcome recording in memory predictions
- [ ] Add calibration stats command: `neuralctl calib`

**Success Criteria:**
- Confidence scores become more accurate over time
- Bins with 10+ predictions show calibrated accuracy
- Console command shows calibration statistics

### Day 3-5: Pre-Training on Synthetic Data

**File:** `crates/kernel/src/pretrain.rs` (NEW)

```rust
use crate::neural::NeuralAgent;
use crate::meta_agent::MetaAgent;

/// Synthetic training scenarios for meta-agent pre-training
pub struct TrainingScenario {
    pub name: &'static str,
    pub inputs: [u8; 12],      // 12 meta-agent inputs
    pub targets: [i32; 3],     // 3 expected outputs (milli-units)
}

/// Pre-defined training scenarios covering common system states
pub const TRAINING_SCENARIOS: &[TrainingScenario] = &[
    // Scenario 1: Memory pressure spike
    TrainingScenario {
        name: "memory_pressure_high",
        inputs: [
            80,  // memory_pressure
            50,  // memory_fragmentation
            30,  // memory_alloc_rate
            5,   // memory_failures
            20,  // scheduling_load
            0,   // deadline_misses
            100, // operator_latency_ms
            1,   // critical_ops_count
            5,   // command_rate
            20,  // command_heaviness
            80,  // prediction_accuracy
            0,   // rapid_stream_detected
        ],
        targets: [
            -800,  // Memory: strong negative (compact urgently)
            -300,  // Scheduling: moderate negative (lower priorities)
            0,     // Command: neutral
        ],
    },

    // Scenario 2: Scheduling overload
    TrainingScenario {
        name: "scheduling_overload",
        inputs: [
            20,  // memory_pressure (low)
            10,  // memory_fragmentation
            5,   // memory_alloc_rate
            0,   // memory_failures
            90,  // scheduling_load (HIGH)
            10,  // deadline_misses (HIGH)
            500, // operator_latency_ms (HIGH)
            8,   // critical_ops_count
            15,  // command_rate
            50,  // command_heaviness
            75,  // prediction_accuracy
            0,   // rapid_stream_detected
        ],
        targets: [
            0,     // Memory: neutral
            -700,  // Scheduling: strong negative (pause non-critical)
            -500,  // Command: negative (throttle commands)
        ],
    },

    // Scenario 3: System idle (all healthy)
    TrainingScenario {
        name: "system_idle_healthy",
        inputs: [
            10,  // memory_pressure (very low)
            5,   // memory_fragmentation
            2,   // memory_alloc_rate
            0,   // memory_failures
            15,  // scheduling_load (low)
            0,   // deadline_misses
            50,  // operator_latency_ms (low)
            2,   // critical_ops_count
            3,   // command_rate (low)
            30,  // command_heaviness
            85,  // prediction_accuracy
            0,   // rapid_stream_detected
        ],
        targets: [
            300,   // Memory: positive (allow aggressive allocation)
            400,   // Scheduling: positive (boost critical ops)
            200,   // Command: positive (allow normal throughput)
        ],
    },

    // Scenario 4: Memory + Scheduling overload (system stress)
    TrainingScenario {
        name: "system_overload_critical",
        inputs: [
            95,  // memory_pressure (CRITICAL)
            75,  // memory_fragmentation (HIGH)
            80,  // memory_alloc_rate (HIGH)
            8,   // memory_failures (HIGH)
            95,  // scheduling_load (CRITICAL)
            15,  // deadline_misses (CRITICAL)
            800, // operator_latency_ms (VERY HIGH)
            10,  // critical_ops_count
            50,  // command_rate (HIGH)
            80,  // command_heaviness (HIGH)
            60,  // prediction_accuracy (degraded)
            100, // rapid_stream_detected (YES)
        ],
        targets: [
            -900,  // Memory: URGENT compact
            -900,  // Scheduling: URGENT pause
            -900,  // Command: URGENT throttle
        ],
    },

    // Scenario 5: Recovery from overload
    TrainingScenario {
        name: "recovery_mode",
        inputs: [
            45,  // memory_pressure (recovering)
            30,  // memory_fragmentation (improving)
            15,  // memory_alloc_rate (decreasing)
            2,   // memory_failures (low)
            50,  // scheduling_load (moderate)
            3,   // deadline_misses (low)
            200, // operator_latency_ms (improving)
            5,   // critical_ops_count
            10,  // command_rate (moderate)
            40,  // command_heaviness
            75,  // prediction_accuracy (recovering)
            0,   // rapid_stream_detected
        ],
        targets: [
            -200,  // Memory: slight negative (gentle compact)
            100,   // Scheduling: slight positive (resume normal)
            0,     // Command: neutral
        ],
    },

    // Scenario 6: Security event (simulated intrusion attempt)
    TrainingScenario {
        name: "security_intrusion_detected",
        inputs: [
            60,  // memory_pressure
            40,  // memory_fragmentation
            50,  // memory_alloc_rate (suspicious pattern)
            0,   // memory_failures
            40,  // scheduling_load
            2,   // deadline_misses
            150, // operator_latency_ms
            3,   // critical_ops_count
            80,  // command_rate (VERY HIGH - rapid commands)
            90,  // command_heaviness (HIGH - complex commands)
            40,  // prediction_accuracy (LOW - unexpected behavior)
            100, // rapid_stream_detected (YES)
        ],
        targets: [
            -400,  // Memory: moderate defensive (prevent exploitation)
            -600,  // Scheduling: strong defensive (isolate suspicious ops)
            -800,  // Command: strong defensive (throttle aggressively)
        ],
    },

    // Scenario 7: Race condition / timing anomaly
    TrainingScenario {
        name: "race_condition_timing_anomaly",
        inputs: [
            35,  // memory_pressure
            25,  // memory_fragmentation
            10,  // memory_alloc_rate (low)
            0,   // memory_failures
            70,  // scheduling_load (moderate-high)
            12,  // deadline_misses (VERY HIGH - anomaly)
            1000,// operator_latency_ms (EXTREME spike)
            6,   // critical_ops_count
            5,   // command_rate (low)
            30,  // command_heaviness
            70,  // prediction_accuracy
            0,   // rapid_stream_detected
        ],
        targets: [
            0,     // Memory: neutral
            -700,  // Scheduling: strong negative (pause and investigate)
            -300,  // Command: moderate negative (slow down to stabilize)
        ],
    },

    // Scenario 8: Memory exhaustion imminent
    TrainingScenario {
        name: "memory_exhaustion_critical",
        inputs: [
            98,  // memory_pressure (CRITICAL)
            90,  // memory_fragmentation (CRITICAL)
            95,  // memory_alloc_rate (CRITICAL)
            10,  // memory_failures (MAX)
            30,  // scheduling_load (low - not the problem)
            1,   // deadline_misses
            100, // operator_latency_ms
            2,   // critical_ops_count
            8,   // command_rate
            40,  // command_heaviness
            80,  // prediction_accuracy
            0,   // rapid_stream_detected
        ],
        targets: [
            -1000, // Memory: MAXIMUM urgency (emergency compaction)
            -400,  // Scheduling: pause all non-critical
            -700,  // Command: block new allocations
        ],
    },

    // Scenario 9: Pathological allocation pattern
    TrainingScenario {
        name: "pathological_alloc_pattern",
        inputs: [
            70,  // memory_pressure
            85,  // memory_fragmentation (VERY HIGH)
            100, // memory_alloc_rate (MAX - thrashing)
            7,   // memory_failures (high)
            50,  // scheduling_load
            5,   // deadline_misses
            200, // operator_latency_ms
            4,   // critical_ops_count
            20,  // command_rate
            60,  // command_heaviness
            65,  // prediction_accuracy (degrading)
            0,   // rapid_stream_detected
        ],
        targets: [
            -900,  // Memory: URGENT defragmentation
            -200,  // Scheduling: slight pause
            -500,  // Command: prevent allocation-heavy commands
        ],
    },

    // Scenario 10: Slow recovery from crash
    TrainingScenario {
        name: "post_recovery_stabilization",
        inputs: [
            55,  // memory_pressure (moderate)
            60,  // memory_fragmentation (moderate-high)
            20,  // memory_alloc_rate (recovering)
            3,   // memory_failures (some)
            45,  // scheduling_load
            8,   // deadline_misses (still high)
            300, // operator_latency_ms (still elevated)
            3,   // critical_ops_count
            5,   // command_rate (low)
            35,  // command_heaviness
            50,  // prediction_accuracy (degraded)
            0,   // rapid_stream_detected
        ],
        targets: [
            -300,  // Memory: gradual compaction
            -100,  // Scheduling: gentle priority adjustment
            -100,  // Command: cautious throttle
        ],
    },

    // Add 15-45 more scenarios covering:
    // - Power state transitions (if power agent exists)
    // - Network congestion patterns (if network agent exists)
    // - I/O storms (if I/O agent exists)
    // - Mixed failure modes (memory + scheduling + command)
    // - Edge cases (single subsystem critical, others healthy)
    // ...
];

/// Pre-train meta-agent on synthetic scenarios
pub fn pretrain_meta_agent(epochs: usize) -> usize {
    let mut total_loss = 0i64;
    let mut samples = 0;

    for epoch in 0..epochs {
        for scenario in TRAINING_SCENARIOS {
            // Convert inputs to Q8.8
            let inputs_q88: [i16; 12] = scenario.inputs.map(|v| ((v as u32 * 256 / 100).min(256)) as i16);

            // Convert targets to Q8.8
            let targets_q88: [i16; 3] = scenario.targets.map(|v| ((v * 256 / 1000).clamp(-256, 256)) as i16);

            // Train meta-agent
            let loss = crate::meta_agent::train_on_sample(&inputs_q88, &targets_q88);
            total_loss += loss as i64;
            samples += 1;
        }

        // Print progress every 10 epochs
        if epoch % 10 == 0 {
            let avg_loss = (total_loss / samples.max(1)) as usize;
            unsafe { crate::uart_print(b"[PRETRAIN] Epoch "); }
            crate::shell::print_number_simple(epoch as u64);
            unsafe { crate::uart_print(b" avg_loss="); }
            crate::shell::print_number_simple(avg_loss as u64);
            unsafe { crate::uart_print(b"\n"); }
            total_loss = 0;
            samples = 0;
        }
    }

    samples
}
```

**File:** `crates/kernel/src/meta_agent.rs` (ADD training function)

```rust
/// Train meta-agent on a single sample
pub fn train_on_sample(inputs: &[i16; 12], targets: &[i16; 3]) -> i32 {
    let mut agent = META_AGENT.lock();

    // Forward pass
    agent.network.infer(inputs);

    // Compute loss (MSE)
    let mut loss: i32 = 0;
    for i in 0..3 {
        let error = agent.network.last_out[i] - targets[i];
        loss += (error as i32 * error as i32) >> 8;  // Scaled squared error
    }

    // Backward pass (compute gradients)
    let gradients = compute_gradients_meta(&agent.network, targets);

    // Update with optimizer
    let mut optimizer = crate::optimizer::META_OPTIMIZER.lock();
    optimizer.update_with_momentum(&mut agent.network, &gradients);

    loss
}
```

**Tasks:**
- [ ] Create `crates/kernel/src/pretrain.rs`
- [ ] Define 20+ training scenarios
- [ ] Implement `pretrain_meta_agent()`
- [ ] Add `train_on_sample()` to meta-agent
- [ ] Add shell command: `metaclassctl pretrain --epochs N`
- [ ] Test pre-training convergence

**Success Criteria:**
- Pre-training reduces average loss over epochs
- Meta-agent makes better decisions after pre-training
- Shell command runs without errors

### Day 5-7: Advanced Techniques & Polish

**Techniques to implement:**

1. **Learning Rate Scheduling**
```rust
pub fn update_learning_rate(epoch: usize, initial_lr: i16) -> i16 {
    // Decay: lr = initial_lr / (1 + decay_rate * epoch)
    let decay_rate = 10;  // Q8.8: 0.01 = 2
    let divisor = 256 + (decay_rate * epoch as i16);
    ((initial_lr as i32 * 256) / divisor as i32) as i16
}
```

2. **Input Normalization**
```rust
pub fn normalize_inputs(inputs: &mut [i16; 12]) {
    // Z-score normalization: (x - mean) / stddev
    let mean = inputs.iter().map(|x| *x as i32).sum::<i32>() / 12;
    let variance = inputs.iter()
        .map(|x| (*x as i32 - mean).pow(2))
        .sum::<i32>() / 12;
    let stddev = integer_sqrt(variance as u32);

    for x in inputs.iter_mut() {
        *x = ((*x as i32 - mean) * 256 / stddev as i32) as i16;
    }
}
```

3. **Early Stopping**
```rust
pub struct EarlyStopping {
    patience: usize,
    best_loss: i32,
    epochs_without_improvement: usize,
}

impl EarlyStopping {
    pub fn should_stop(&mut self, current_loss: i32) -> bool {
        if current_loss < self.best_loss {
            self.best_loss = current_loss;
            self.epochs_without_improvement = 0;
            false
        } else {
            self.epochs_without_improvement += 1;
            self.epochs_without_improvement >= self.patience
        }
    }
}
```

**Tasks:**
- [ ] Implement learning rate decay
- [ ] Add input normalization
- [ ] Implement early stopping
- [ ] Test each technique independently
- [ ] Combine all techniques in final training loop
- [ ] Measure accuracy improvements
- [ ] Document best hyperparameters

**Success Criteria:**
- Training converges faster
- Better generalization to unseen scenarios
- Automatic stopping when training plateaus

### Day 7: Documentation & Testing

**Documentation Tasks:**
- [ ] Update README.md with Week 3 features
- [ ] Create NEURAL-ML-TECHNIQUES.md documenting:
  - Momentum-based gradient descent
  - Confidence calibration
  - Pre-training methodology
  - Learning rate scheduling
- [ ] Add code comments to all new functions
- [ ] Create usage examples for each technique

**Comprehensive Testing:**

```bash
# Test 1: Momentum training
neuralctl retrain 50 --momentum 0.9
# Expected: Faster convergence than without momentum

# Test 2: Confidence calibration
neuralctl calib
# Expected: Shows calibration bins with accuracy

# Test 3: Meta-agent pre-training
metaclassctl pretrain --epochs 100
metaclassctl status
# Expected: Meta-agent makes better decisions

# Test 4: Combined system stress test
graphdemo & memctl stress 500 &
agentctl bus
metaclassctl status
# Expected: Coordinated response across all agents with calibrated confidence

# Test 5: Learning rate decay
neuralctl config --lr-decay 0.01
neuralctl retrain 100
# Expected: Smooth convergence with decaying learning rate
```

**Metrics to collect:**
- Training loss curves (before/after improvements)
- Prediction accuracy (before/after calibration)
- Decision quality (before/after pre-training)
- Convergence speed (with/without momentum)

**Week 3 Deliverables:**
- ✅ Momentum-based gradient descent
- ✅ Confidence calibration system
- ✅ Pre-training on synthetic scenarios
- ✅ Learning rate scheduling
- ✅ Input normalization
- ✅ Early stopping
- ✅ Comprehensive documentation
- ✅ Full test suite

---

## Success Metrics

### Week 1 Metrics
- [ ] Agent message bus operational
- [ ] 3 agents communicating (measured by message count)
- [ ] Cross-agent coordination demonstrated (console logs)
- [ ] <1ms latency for message handling
- [ ] 0 deadlocks or panics

### Week 2 Metrics
- [ ] Meta-agent making decisions every 100ms
- [ ] Global coordination demonstrated across all subsystems
- [ ] Confidence-gated decisions (50%+ threshold)
- [ ] System overload detection working
- [ ] Metrics show decision frequency and outcomes

### Week 3 Metrics
- [ ] Training convergence 30%+ faster with momentum
- [ ] Confidence calibration accuracy within 10% of actual
- [ ] Pre-training reduces initial decision errors by 50%+
- [ ] All ML techniques integrated and tested
- [ ] Complete documentation

### Overall Success Criteria
- [ ] All 3 weeks completed on schedule
- [ ] No scope creep (stayed within neural work)
- [ ] Publishable results (novel multi-agent architecture)
- [ ] Production-ready code quality
- [ ] Comprehensive test coverage
- [ ] Full documentation

---

## Risk Mitigation

### Risk: Falling behind schedule
**Mitigation:**
- Timebox each day strictly (8 hours max)
- If a day's work isn't done by EOD, move to next day's tasks
- Mark incomplete items as "deferred" rather than blocking

### Risk: Debugging spirals
**Mitigation:**
- Add comprehensive debug logging from day 1
- Use `agentctl bus` to inspect messages
- Add metrics for every critical operation
- If stuck >2 hours, simplify and move on

### Risk: Losing motivation
**Mitigation:**
- Celebrate daily wins (first message sent, first meta-decision)
- Keep a "wins" log visible
- Take breaks between weeks
- Remember: this is the unique contribution

### Risk: Feature creep
**Mitigation:**
- Re-read scope contract daily
- No new subsystems (networking/FS deferred)
- No additional agents (3 is enough)
- If tempted: add to "future work" doc instead

---

## Daily Checklist Template

Use this for each day:

```
## Day X - [Date]

Goal: [One sentence]

Tasks:
[ ] Task 1
[ ] Task 2
[ ] Task 3

Completed:
- [x] Completed item 1
- [x] Completed item 2

Blocked/Deferred:
- [ ] Deferred item (reason)

Tomorrow:
- [ ] Next task

Notes:
- Insight/challenge encountered
- Code location changed
- Test results
```

---

## Post-Plan Options

After completing this 3-week plan, you'll have:
- ✅ Novel multi-agent kernel architecture
- ✅ Cross-agent coordination
- ✅ Meta-agent supervision
- ✅ Production-grade ML techniques
- ✅ Publishable research contribution

**Then you can choose:**

1. **Option A: Write Paper**
   - "Neural-First Operating Systems: Multi-Agent Architecture for Autonomous Kernel Management"
   - Submit to SOSP, OSDI, or similar conference

2. **Option B: Add Subsystems**
   - Now add networking/filesystem with neural agents
   - Demonstrate scalability beyond 3 agents

3. **Option C: Deeper ML**
   - Reinforcement learning (Q-learning for meta-agent)
   - Multi-agent communication protocols
   - Transfer learning across agents

4. **Option D: Production Hardening**
   - Formal verification of coordination logic
   - Byzantine fault tolerance
   - Performance optimization

---

## Appendix: Code Organization

```
crates/kernel/src/
├── neural.rs           (existing - command, scheduling, memory agents)
├── agent_bus.rs        (NEW - Week 1)
├── meta_agent.rs       (NEW - Week 2)
├── optimizer.rs        (NEW - Week 3)
├── calibration.rs      (NEW - Week 3)
├── pretrain.rs         (NEW - Week 3)
├── lib.rs              (modified - add new modules)
├── shell.rs            (modified - add agentctl, metaclassctl)
├── main.rs             (modified - init meta-agent)
└── graph.rs            (modified - add helper functions)

docs/
├── NEURAL-PHASE3-PLAN.md           (this document)
├── AGENT-MESSAGE-PROTOCOL.md       (Week 1 output)
├── META-AGENT-DECISIONS.md         (Week 2 output)
└── NEURAL-ML-TECHNIQUES.md         (Week 3 output)
```

---

## Enhancements & Future Extensibility

### Layered Agent Design with Stubs

**Objective:** Design agent architecture to support future subsystem agents without implementing full functionality.

**Why:** Demonstrates scalability of multi-agent architecture while avoiding scope creep.

**File:** `crates/kernel/src/agent_stubs.rs` (NEW - Optional, Week 3 Day 7)

```rust
/// Stub agents for future subsystems (demonstrates extensibility)
/// These don't implement full functionality but show the pattern

use crate::agent_bus::{AgentMessage, AgentMessageBus};

/// Network agent stub (for future networking subsystem)
pub struct NetworkAgentStub {
    enabled: bool,
}

impl NetworkAgentStub {
    pub const fn new() -> Self {
        Self { enabled: false }
    }

    /// Simulated network telemetry (would come from real network stack)
    pub fn get_telemetry(&self) -> NetworkTelemetry {
        NetworkTelemetry {
            packet_rate: 0,
            queue_depth: 0,
            retransmits: 0,
            congestion_level: 0,
        }
    }

    /// Publish network status (stub - would analyze real traffic)
    pub fn publish_status(&self, bus: &mut AgentMessageBus) {
        if !self.enabled {
            return;
        }
        // In future: publish NetworkCongestion, NetworkHealthy, etc.
    }

    /// Handle messages from other agents (stub)
    pub fn handle_messages(&self, bus: &AgentMessageBus) {
        if !self.enabled {
            return;
        }
        // In future: respond to MemoryPressure (reduce buffer size), etc.
    }
}

#[derive(Copy, Clone)]
pub struct NetworkTelemetry {
    pub packet_rate: u32,
    pub queue_depth: u8,
    pub retransmits: u8,
    pub congestion_level: u8,
}

/// Power management agent stub (for future power subsystem)
pub struct PowerAgentStub {
    enabled: bool,
}

impl PowerAgentStub {
    pub const fn new() -> Self {
        Self { enabled: false }
    }

    pub fn get_telemetry(&self) -> PowerTelemetry {
        PowerTelemetry {
            power_state: 0,  // 0=active, 1=idle, 2=sleep
            thermal_level: 0,
            throttling: false,
        }
    }

    pub fn publish_status(&self, bus: &mut AgentMessageBus) {
        if !self.enabled {
            return;
        }
        // In future: publish ThermalWarning, PowerStateChange, etc.
    }

    pub fn handle_messages(&self, bus: &AgentMessageBus) {
        if !self.enabled {
            return;
        }
        // In future: respond to SystemOverload (throttle CPU), etc.
    }
}

#[derive(Copy, Clone)]
pub struct PowerTelemetry {
    pub power_state: u8,
    pub thermal_level: u8,
    pub throttling: bool,
}

/// I/O agent stub (for future filesystem/storage subsystem)
pub struct IoAgentStub {
    enabled: bool,
}

impl IoAgentStub {
    pub const fn new() -> Self {
        Self { enabled: false }
    }

    pub fn get_telemetry(&self) -> IoTelemetry {
        IoTelemetry {
            read_latency_us: 0,
            write_latency_us: 0,
            queue_depth: 0,
            cache_hit_rate: 0,
        }
    }

    pub fn publish_status(&self, bus: &mut AgentMessageBus) {
        if !self.enabled {
            return;
        }
        // In future: publish IoSlowdown, CacheThrashing, etc.
    }

    pub fn handle_messages(&self, bus: &AgentMessageBus) {
        if !self.enabled {
            return;
        }
        // In future: respond to MemoryPressure (evict cache), etc.
    }
}

#[derive(Copy, Clone)]
pub struct IoTelemetry {
    pub read_latency_us: u32,
    pub write_latency_us: u32,
    pub queue_depth: u8,
    pub cache_hit_rate: u8,
}

// Global stubs (disabled by default)
static NETWORK_STUB: spin::Mutex<NetworkAgentStub> = spin::Mutex::new(NetworkAgentStub::new());
static POWER_STUB: spin::Mutex<PowerAgentStub> = spin::Mutex::new(PowerAgentStub::new());
static IO_STUB: spin::Mutex<IoAgentStub> = spin::Mutex::new(IoAgentStub::new());
```

**Benefits of stub design:**
1. ✅ Shows architecture scales beyond 3 agents
2. ✅ Documents future agent interface pattern
3. ✅ Zero implementation cost (just stub methods)
4. ✅ Can enable stubs for testing coordination logic
5. ✅ Makes it easy to add real agents later (just fill in stubs)

**Meta-agent extension (add stub telemetry to inputs):**

```rust
// Extend MetaState to include stub agents (when enabled)
pub struct MetaState {
    // Existing fields...
    memory_pressure: u8,
    // ...

    // Optional stub agent inputs (set to 0 if disabled)
    network_congestion: u8,   // From network stub
    power_thermal: u8,        // From power stub
    io_latency_level: u8,     // From I/O stub
}

// Meta-agent can now handle 15 inputs (12 existing + 3 stub)
// When stubs are disabled, their inputs are 0 (no effect on decisions)
// When stubs are enabled, they participate in coordination
```

**Shell command to enable stubs (testing only):**

```bash
# Enable network stub for testing coordination logic
agentctl stub network on
metaclassctl status
# Now shows network telemetry (simulated)

# Disable again
agentctl stub network off
```

**Value:**
- Demonstrates architectural foresight
- Makes future work easier
- Proves coordination scales to 6+ agents
- **Zero scope creep** (no real subsystems added)

### Periodic Telemetry Snapshots

**Enhancement:** Automatically export JSON snapshots every N seconds for continuous monitoring.

**File:** `crates/kernel/src/meta_agent.rs` (ADD)

```rust
/// Configuration for automatic telemetry export
static TELEMETRY_EXPORT_CONFIG: Mutex<TelemetryExportConfig> = Mutex::new(TelemetryExportConfig::new());

struct TelemetryExportConfig {
    enabled: bool,
    interval_ms: u64,
    last_export_ts: u64,
}

impl TelemetryExportConfig {
    const fn new() -> Self {
        Self {
            enabled: false,
            interval_ms: 1000,  // 1 second default
            last_export_ts: 0,
        }
    }
}

/// Periodic telemetry export (call from main loop or timer)
pub fn periodic_telemetry_export() {
    let mut config = TELEMETRY_EXPORT_CONFIG.lock();
    if !config.enabled {
        return;
    }

    let now_us = crate::graph::cycles_to_ns(crate::graph::now_cycles()) / 1000;
    let elapsed_ms = (now_us - config.last_export_ts) / 1000;

    if elapsed_ms >= config.interval_ms {
        config.last_export_ts = now_us;
        drop(config);

        // Export JSON snapshot
        unsafe { crate::uart_print(b"\n--- TELEMETRY SNAPSHOT ---\n"); }
        export_telemetry_json();
        unsafe { crate::uart_print(b"--- END SNAPSHOT ---\n\n"); }

        crate::trace::metric_kv("telemetry_snapshot", 1);
    }
}

/// Enable/disable periodic export
pub fn set_telemetry_export(enabled: bool, interval_ms: u64) {
    let mut config = TELEMETRY_EXPORT_CONFIG.lock();
    config.enabled = enabled;
    config.interval_ms = interval_ms;
}
```

**Shell command:**

```bash
# Enable snapshots every 2 seconds
metaclassctl autoexport on --interval 2000

# Disable
metaclassctl autoexport off

# Snapshots appear automatically in console, can redirect to file
# Run in background: ./uefi_run.sh > telemetry_log.json 2>&1
```

**Use case:**
- Long-running stress tests with continuous monitoring
- Time-series data collection for ML training
- External dashboard integration (parse JSON stream)
- Post-mortem analysis of system behavior

---

## Commitment Contract

**I commit to:**
- [ ] Following this 3-week plan strictly
- [ ] Not adding new subsystems (no networking/FS)
- [ ] Timeboxing each day's work
- [ ] Documenting as I go
- [ ] Celebrating milestones
- [ ] Completing all deliverables

**Signed:** [Date]

**Start Date:** [Fill in]
**Expected Completion:** [Start + 21 days]

---

**Ready to begin Week 1?** The journey to a truly neural-first kernel starts with the first message passed between agents.

Let's build something novel. 🚀
