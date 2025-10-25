//! Meta-Agent for Global Multi-Subsystem Coordination
//!
//! The meta-agent learns global optimization strategies by observing telemetry
//! from all three neural agents (Memory, Scheduling, Command) and making
//! coordinated decisions that optimize system-wide behavior.
//!
//! Architecture:
//! - 12 inputs (4 from each agent)
//! - 16 hidden neurons
//! - 3 outputs (per-subsystem coordination directives)
//!
//! Features:
//! - Periodic decision-making (configurable interval)
//! - Confidence-based autonomous actions
//! - Learning from multi-agent outcomes
//! - Runtime configuration (thresholds, intervals)

use spin::Mutex;
use crate::neural::NeuralAgent;

/// Meta-agent input dimensions
const META_IN: usize = 12;   // 4 inputs per agent × 3 agents
const META_HID: usize = 16;  // 16 hidden neurons
const META_OUT: usize = 3;   // 3 outputs (one per subsystem)

/// Meta-agent state: aggregated telemetry from all agents
#[derive(Copy, Clone)]
pub struct MetaState {
    // Memory telemetry (4 inputs)
    pub memory_pressure: u8,       // 0-100 %
    pub memory_fragmentation: u8,  // 0-100 %
    pub memory_alloc_rate: u8,     // 0-100 (normalized from 0-1000/sec)
    pub memory_failures: u8,       // 0-100 (capped)

    // Scheduling telemetry (4 inputs)
    pub scheduling_load: u8,       // 0-100 (based on deadline misses)
    pub deadline_misses: u8,       // 0-100 (recent count)
    pub operator_latency_ms: u8,   // 0-100 (normalized from 0-10ms)
    pub critical_ops_count: u8,    // 0-100 (count of critical ops)

    // Command telemetry (4 inputs)
    pub command_rate: u8,          // 0-100 (normalized from 0-50/sec)
    pub command_heaviness: u8,     // 0-100 (avg command complexity)
    pub prediction_accuracy: u8,   // 0-100 %
    pub rapid_stream_detected: u8, // 0 or 100 (binary flag)
}

impl MetaState {
    pub const fn new() -> Self {
        MetaState {
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
            prediction_accuracy: 50, // Default to neutral
            rapid_stream_detected: 0,
        }
    }

    /// Convert state to Q8.8 fixed-point inputs for neural network
    pub fn to_q88_inputs(&self) -> [i16; META_IN] {
        [
            // Memory (4 inputs)
            ((self.memory_pressure as i32 * 256) / 100) as i16,
            ((self.memory_fragmentation as i32 * 256) / 100) as i16,
            ((self.memory_alloc_rate as i32 * 256) / 100) as i16,
            ((self.memory_failures as i32 * 256) / 100) as i16,
            // Scheduling (4 inputs)
            ((self.scheduling_load as i32 * 256) / 100) as i16,
            ((self.deadline_misses as i32 * 256) / 100) as i16,
            ((self.operator_latency_ms as i32 * 256) / 100) as i16,
            ((self.critical_ops_count as i32 * 256) / 100) as i16,
            // Command (4 inputs)
            ((self.command_rate as i32 * 256) / 100) as i16,
            ((self.command_heaviness as i32 * 256) / 100) as i16,
            ((self.prediction_accuracy as i32 * 256) / 100) as i16,
            ((self.rapid_stream_detected as i32 * 256) / 100) as i16,
        ]
    }
}

/// Meta-agent decision outputs
#[derive(Copy, Clone, Debug)]
pub struct MetaDecision {
    pub memory_directive: i16,     // -1000 to 1000 milli-units
    pub scheduling_directive: i16, // -1000 to 1000 milli-units
    pub command_directive: i16,    // -1000 to 1000 milli-units
    pub confidence: u16,           // 0-1000 milli-units
    pub timestamp_us: u64,
}

impl MetaDecision {
    pub const fn new() -> Self {
        MetaDecision {
            memory_directive: 0,
            scheduling_directive: 0,
            command_directive: 0,
            confidence: 0,
            timestamp_us: 0,
        }
    }
}

/// Meta-agent configuration
#[derive(Copy, Clone)]
pub struct MetaConfig {
    pub decision_interval_us: u64, // How often to make decisions (microseconds)
    pub confidence_threshold: u16, // Minimum confidence to act (0-1000)
    pub enabled: bool,             // Master enable/disable
}

impl MetaConfig {
    pub const fn new() -> Self {
        MetaConfig {
            decision_interval_us: 100_000, // 100ms default
            confidence_threshold: 400,     // 40% confidence minimum
            enabled: true,
        }
    }
}

/// Statistics for meta-agent monitoring
#[derive(Copy, Clone)]
pub struct MetaStats {
    pub total_decisions: u64,
    pub autonomous_actions: u64,
    pub memory_adjustments: u32,
    pub scheduling_adjustments: u32,
    pub command_adjustments: u32,
    pub last_decision_ts: u64,
}

impl MetaStats {
    const fn new() -> Self {
        MetaStats {
            total_decisions: 0,
            autonomous_actions: 0,
            memory_adjustments: 0,
            scheduling_adjustments: 0,
            command_adjustments: 0,
            last_decision_ts: 0,
        }
    }
}

/// The Meta-Agent: coordinates all subsystem neural agents
pub struct MetaAgent {
    network: NeuralAgent,
    state: MetaState,
    config: MetaConfig,
    stats: MetaStats,
    last_decision: MetaDecision,
}

impl MetaAgent {
    pub const fn new() -> Self {
        MetaAgent {
            network: NeuralAgent::new(),
            state: MetaState::new(),
            config: MetaConfig::new(),
            stats: MetaStats::new(),
            last_decision: MetaDecision::new(),
        }
    }

    /// Initialize meta-agent with proper dimensions (12→16→3)
    pub fn init(&mut self) {
        self.network.set_dims(META_IN, META_HID, META_OUT);
        self.network.infer_count = 1; // Prevent lazy init from resetting dims
        unsafe {
            crate::uart_print(b"[META] Initialized meta-agent: 12 inputs, 16 hidden, 3 outputs\n");
        }
    }

    /// Update state from subsystem telemetry
    pub fn update_state(&mut self, new_state: MetaState) {
        self.state = new_state;
    }

    /// Get current state
    pub fn get_state(&self) -> MetaState {
        self.state
    }

    /// Get current configuration
    pub fn get_config(&self) -> MetaConfig {
        self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: MetaConfig) {
        self.config = config;
    }

    /// Get statistics
    pub fn get_stats(&self) -> MetaStats {
        self.stats
    }

    /// Get last decision
    pub fn get_last_decision(&self) -> MetaDecision {
        self.last_decision
    }

    /// Check if it's time to make a decision
    pub fn should_decide(&self, current_time_us: u64) -> bool {
        if !self.config.enabled {
            return false;
        }
        let elapsed = current_time_us.saturating_sub(self.stats.last_decision_ts);
        elapsed >= self.config.decision_interval_us
    }

    /// Make a coordination decision based on current state
    pub fn decide(&mut self, current_time_us: u64) -> MetaDecision {
        // Convert state to Q8.8 inputs
        let inputs = self.state.to_q88_inputs();

        // Run inference
        let out_len = self.network.infer(&inputs);

        if out_len < META_OUT {
            // Not enough outputs, return neutral decision
            return MetaDecision::new();
        }

        // Extract outputs (Q8.8 format)
        let out0_q88 = self.network.last_out[0]; // Memory directive
        let out1_q88 = self.network.last_out[1]; // Scheduling directive
        let out2_q88 = self.network.last_out[2]; // Command directive

        // Convert Q8.8 to milli-units (-1000 to 1000)
        let memory_milli = ((out0_q88 as i32) * 1000 / 256).clamp(-1000, 1000) as i16;
        let scheduling_milli = ((out1_q88 as i32) * 1000 / 256).clamp(-1000, 1000) as i16;
        let command_milli = ((out2_q88 as i32) * 1000 / 256).clamp(-1000, 1000) as i16;

        // Compute confidence (average absolute value of outputs)
        let confidence = ((memory_milli.abs() + scheduling_milli.abs() + command_milli.abs()) / 3)
            .min(1000) as u16;

        let decision = MetaDecision {
            memory_directive: memory_milli,
            scheduling_directive: scheduling_milli,
            command_directive: command_milli,
            confidence,
            timestamp_us: current_time_us,
        };

        // Update stats
        self.stats.total_decisions += 1;
        self.stats.last_decision_ts = current_time_us;
        self.last_decision = decision;

        decision
    }

    /// Execute a decision if confidence exceeds threshold
    pub fn execute_decision(&mut self, decision: &MetaDecision) -> bool {
        if decision.confidence < self.config.confidence_threshold {
            return false; // Confidence too low
        }

        self.stats.autonomous_actions += 1;

        // Determine which subsystems need adjustment
        let memory_action = decision.memory_directive.abs() > 300;
        let scheduling_action = decision.scheduling_directive.abs() > 300;
        let command_action = decision.command_directive.abs() > 300;

        if memory_action {
            self.stats.memory_adjustments += 1;
            unsafe {
                crate::uart_print(b"[META] Memory directive: ");
                print_signed_milli(decision.memory_directive);
                crate::uart_print(b" (conf=");
                print_number(decision.confidence as usize);
                crate::uart_print(b")\n");
            }
        }

        if scheduling_action {
            self.stats.scheduling_adjustments += 1;
            unsafe {
                crate::uart_print(b"[META] Scheduling directive: ");
                print_signed_milli(decision.scheduling_directive);
                crate::uart_print(b" (conf=");
                print_number(decision.confidence as usize);
                crate::uart_print(b")\n");
            }
        }

        if command_action {
            self.stats.command_adjustments += 1;
            unsafe {
                crate::uart_print(b"[META] Command directive: ");
                print_signed_milli(decision.command_directive);
                crate::uart_print(b" (conf=");
                print_number(decision.confidence as usize);
                crate::uart_print(b")\n");
            }
        }

        memory_action || scheduling_action || command_action
    }
}

/// Global meta-agent instance
static META_AGENT: Mutex<MetaAgent> = Mutex::new(MetaAgent::new());

/// Initialize the meta-agent
pub fn init_meta_agent() {
    let mut agent = META_AGENT.lock();
    agent.init();
    crate::trace::metric_kv("meta_agent_init", 1);
}

/// Update meta-agent state from subsystem telemetry
pub fn update_meta_state(state: MetaState) {
    META_AGENT.lock().update_state(state);
}

/// Get current meta-agent state
pub fn get_meta_state() -> MetaState {
    META_AGENT.lock().get_state()
}

/// Get meta-agent configuration
pub fn get_meta_config() -> MetaConfig {
    META_AGENT.lock().get_config()
}

/// Set meta-agent configuration
pub fn set_meta_config(config: MetaConfig) {
    META_AGENT.lock().set_config(config);
}

/// Get meta-agent statistics
pub fn get_meta_stats() -> MetaStats {
    META_AGENT.lock().get_stats()
}

/// Get last meta-agent decision
pub fn get_last_decision() -> MetaDecision {
    META_AGENT.lock().get_last_decision()
}

/// Periodic meta-agent tick: check if decision is needed and execute
pub fn meta_agent_tick() {
    let current_time = crate::agent_bus::get_timestamp_us();

    let should_decide = META_AGENT.lock().should_decide(current_time);
    if !should_decide {
        return;
    }

    // Make decision
    let decision = META_AGENT.lock().decide(current_time);

    // Execute if confidence is sufficient
    let executed = META_AGENT.lock().execute_decision(&decision);

    if executed {
        crate::trace::metric_kv("meta_decision_executed", 1);
        crate::trace::metric_kv("meta_confidence", decision.confidence as usize);
    }
}

/// Force a meta-agent decision immediately (for testing/debugging)
pub fn force_meta_decision() -> MetaDecision {
    let current_time = crate::agent_bus::get_timestamp_us();
    let decision = META_AGENT.lock().decide(current_time);
    let _ = META_AGENT.lock().execute_decision(&decision);
    decision
}

/// Helper function to print signed milli-units
unsafe fn print_signed_milli(value: i16) {
    if value < 0 {
        crate::uart_print(b"-");
        print_number((-value) as usize);
    } else {
        crate::uart_print(b"+");
        print_number(value as usize);
    }
}

/// Helper function to print numbers
unsafe fn print_number(mut num: usize) {
    if num == 0 {
        crate::uart_print(b"0");
        return;
    }

    let mut digits = [0u8; 20];
    let mut i = 0;

    while num > 0 {
        digits[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::uart_print(&[digits[i]]);
    }
}

/// Collect telemetry from all subsystems and update meta-agent state
pub fn collect_telemetry() -> MetaState {
    let mut state = MetaState::new();

    // Collect memory telemetry
    let heap_stats = crate::heap::get_heap_stats();
    let heap_size: usize = 100 * 1024; // 100 KiB
    let used = heap_stats.current_allocated();
    let free = heap_size.saturating_sub(used);
    state.memory_pressure = (100 - (free * 100 / heap_size)).min(100) as u8;

    // Estimate fragmentation from peak vs current
    let peak = heap_stats.peak_allocated();
    if peak > 0 {
        let utilization_ratio = (used * 100 / peak).min(100);
        // Low utilization with high peak = fragmentation
        if utilization_ratio < 80 {
            state.memory_fragmentation = (80 - utilization_ratio) as u8;
        }
    }

    // Allocation rate (simplified)
    state.memory_alloc_rate = (heap_stats.total_allocations() % 100) as u8;

    // Recent failures
    state.memory_failures = heap_stats.allocation_failures().min(100) as u8;

    // Collect scheduling telemetry from agent bus
    let messages = crate::agent_bus::get_all_messages();
    let mut deadline_misses = 0u8;
    let mut load_high = false;

    for msg in messages.iter() {
        match msg {
            crate::agent_bus::AgentMessage::SchedulingLoadHigh { deadline_misses: misses, .. } => {
                deadline_misses = deadline_misses.saturating_add(*misses);
                load_high = true;
            }
            crate::agent_bus::AgentMessage::SchedulingCriticalOperatorLatency { .. } => {
                state.critical_ops_count = state.critical_ops_count.saturating_add(1);
            }
            _ => {}
        }
    }

    state.deadline_misses = deadline_misses.min(100);
    state.scheduling_load = if load_high { 70 } else { 20 };
    state.operator_latency_ms = (deadline_misses * 2).min(100); // Approximate

    // Collect command telemetry from agent bus
    let mut command_count = 0u16;
    let mut rapid_detected = false;

    for msg in messages.iter() {
        match msg {
            crate::agent_bus::AgentMessage::CommandHeavyPredicted { .. } => {
                state.command_heaviness = state.command_heaviness.saturating_add(10).min(100);
            }
            crate::agent_bus::AgentMessage::CommandRapidStream { commands_per_sec, .. } => {
                command_count = command_count.saturating_add(*commands_per_sec);
                rapid_detected = true;
            }
            crate::agent_bus::AgentMessage::CommandLowAccuracy { recent_accuracy, .. } => {
                state.prediction_accuracy = *recent_accuracy;
            }
            _ => {}
        }
    }

    state.command_rate = (command_count / 2).min(100) as u8; // Normalize to 0-100
    state.rapid_stream_detected = if rapid_detected { 100 } else { 0 };

    // Update the meta-agent state
    update_meta_state(state);

    state
}

/// Print meta-agent status
pub fn print_meta_status() {
    let state = get_meta_state();
    let config = get_meta_config();
    let stats = get_meta_stats();
    let decision = get_last_decision();

    unsafe {
        crate::uart_print(b"\n=== Meta-Agent Status ===\n\n");

        // Configuration
        crate::uart_print(b"Configuration:\n");
        crate::uart_print(b"  Enabled: ");
        crate::uart_print(if config.enabled { b"YES\n" } else { b"NO\n" });
        crate::uart_print(b"  Decision Interval: ");
        print_number((config.decision_interval_us / 1000) as usize);
        crate::uart_print(b" ms\n");
        crate::uart_print(b"  Confidence Threshold: ");
        print_number(config.confidence_threshold as usize);
        crate::uart_print(b"/1000\n\n");

        // Current State
        crate::uart_print(b"Current State:\n");
        crate::uart_print(b"  Memory: pressure=");
        print_number(state.memory_pressure as usize);
        crate::uart_print(b"% frag=");
        print_number(state.memory_fragmentation as usize);
        crate::uart_print(b"% rate=");
        print_number(state.memory_alloc_rate as usize);
        crate::uart_print(b" failures=");
        print_number(state.memory_failures as usize);
        crate::uart_print(b"\n  Scheduling: load=");
        print_number(state.scheduling_load as usize);
        crate::uart_print(b"% misses=");
        print_number(state.deadline_misses as usize);
        crate::uart_print(b" latency=");
        print_number(state.operator_latency_ms as usize);
        crate::uart_print(b"ms critical=");
        print_number(state.critical_ops_count as usize);
        crate::uart_print(b"\n  Command: rate=");
        print_number(state.command_rate as usize);
        crate::uart_print(b"/sec heavy=");
        print_number(state.command_heaviness as usize);
        crate::uart_print(b" accuracy=");
        print_number(state.prediction_accuracy as usize);
        crate::uart_print(b"% rapid=");
        crate::uart_print(if state.rapid_stream_detected > 0 { b"YES" } else { b"NO" });
        crate::uart_print(b"\n\n");

        // Last Decision
        crate::uart_print(b"Last Decision:\n");
        crate::uart_print(b"  Memory: ");
        print_signed_milli(decision.memory_directive);
        crate::uart_print(b"/1000\n");
        crate::uart_print(b"  Scheduling: ");
        print_signed_milli(decision.scheduling_directive);
        crate::uart_print(b"/1000\n");
        crate::uart_print(b"  Command: ");
        print_signed_milli(decision.command_directive);
        crate::uart_print(b"/1000\n");
        crate::uart_print(b"  Confidence: ");
        print_number(decision.confidence as usize);
        crate::uart_print(b"/1000\n\n");

        // Statistics
        crate::uart_print(b"Statistics:\n");
        crate::uart_print(b"  Total Decisions: ");
        print_number(stats.total_decisions as usize);
        crate::uart_print(b"\n  Autonomous Actions: ");
        print_number(stats.autonomous_actions as usize);
        crate::uart_print(b"\n  Memory Adjustments: ");
        print_number(stats.memory_adjustments as usize);
        crate::uart_print(b"\n  Scheduling Adjustments: ");
        print_number(stats.scheduling_adjustments as usize);
        crate::uart_print(b"\n  Command Adjustments: ");
        print_number(stats.command_adjustments as usize);
        crate::uart_print(b"\n\n");
    }
}
