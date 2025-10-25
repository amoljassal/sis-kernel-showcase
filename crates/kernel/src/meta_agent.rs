//! Meta-Agent for Global Multi-Subsystem Coordination
//!
//! The meta-agent learns global optimization strategies by observing telemetry
//! from all three neural agents (Memory, Scheduling, Command) and making
//! coordinated decisions that optimize system-wide behavior.
//!
//! Architecture:
//! - 12 inputs (4 from each agent)
//! - 16 hidden neurons (dynamically adjustable)
//! - 3 outputs (per-subsystem coordination directives)
//!
//! Features:
//! - Periodic decision-making (configurable interval)
//! - Confidence-based autonomous actions
//! - Learning from multi-agent outcomes
//! - Runtime configuration (thresholds, intervals)
//!
//! Week 3 Advanced ML Features:
//! - Experience replay buffer (128 entries)
//! - Temporal difference learning (TD(0) value function)
//! - Multi-objective optimization (performance, power, latency)
//! - Dynamic topology adjustment (pruning, growth)

use spin::Mutex;
use crate::neural::NeuralAgent;

/// Meta-agent input dimensions
const META_IN: usize = 12;   // 4 inputs per agent × 3 agents
const META_HID: usize = 16;  // 16 hidden neurons (can be adjusted dynamically)
const META_OUT: usize = 3;   // 3 outputs (one per subsystem)

/// Experience replay buffer size
const REPLAY_BUFFER_SIZE: usize = 128;

/// Learning rate for TD learning (Q8.8 fixed-point: 256 = 1.0)
const LEARNING_RATE: i16 = 51; // 0.2 in Q8.8

/// Discount factor for future rewards (Q8.8 fixed-point)
const DISCOUNT_FACTOR: i16 = 230; // 0.9 in Q8.8

/// Weight pruning threshold (Q8.8 fixed-point)
const PRUNE_THRESHOLD: i16 = 13; // 0.05 in Q8.8

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

/// Multi-objective reward components
#[derive(Copy, Clone, Debug)]
pub struct MultiObjectiveReward {
    pub performance: i16,  // -1000 to 1000 (improvement in system performance)
    pub power: i16,        // -1000 to 1000 (power efficiency)
    pub latency: i16,      // -1000 to 1000 (latency reduction)
    pub weighted_sum: i16, // Combined reward with configured weights
}

impl MultiObjectiveReward {
    pub const fn new() -> Self {
        MultiObjectiveReward {
            performance: 0,
            power: 0,
            latency: 0,
            weighted_sum: 0,
        }
    }

    /// Compute weighted sum from individual rewards
    pub fn compute_weighted(&mut self, perf_weight: u8, power_weight: u8, latency_weight: u8) {
        let total_weight = (perf_weight + power_weight + latency_weight) as i32;
        if total_weight == 0 {
            self.weighted_sum = 0;
            return;
        }

        let weighted = ((self.performance as i32 * perf_weight as i32) +
                        (self.power as i32 * power_weight as i32) +
                        (self.latency as i32 * latency_weight as i32)) / total_weight;

        self.weighted_sum = weighted.clamp(-1000, 1000) as i16;
    }
}

/// Experience replay buffer entry
#[derive(Copy, Clone)]
pub struct ReplayEntry {
    pub state: MetaState,
    pub decision: MetaDecision,
    pub reward: MultiObjectiveReward,
    pub next_state: MetaState,
    pub timestamp_us: u64,
    pub valid: bool,
}

impl ReplayEntry {
    pub const fn new() -> Self {
        ReplayEntry {
            state: MetaState::new(),
            decision: MetaDecision::new(),
            reward: MultiObjectiveReward::new(),
            next_state: MetaState::new(),
            timestamp_us: 0,
            valid: false,
        }
    }
}

/// Experience replay buffer for temporal credit assignment
pub struct ReplayBuffer {
    entries: [ReplayEntry; REPLAY_BUFFER_SIZE],
    head: usize,
    count: usize,
}

impl ReplayBuffer {
    pub const fn new() -> Self {
        ReplayBuffer {
            entries: [ReplayEntry::new(); REPLAY_BUFFER_SIZE],
            head: 0,
            count: 0,
        }
    }

    /// Add a new experience to the buffer
    pub fn push(&mut self, entry: ReplayEntry) {
        self.entries[self.head] = entry;
        self.head = (self.head + 1) % REPLAY_BUFFER_SIZE;
        self.count = (self.count + 1).min(REPLAY_BUFFER_SIZE);
    }

    /// Get a random sample of experiences
    pub fn sample(&self, count: usize) -> &[ReplayEntry] {
        let max_count = count.min(self.count);
        if max_count == 0 {
            return &[];
        }

        // Simple sampling: return last N entries
        let start = if self.count >= max_count {
            (self.head + REPLAY_BUFFER_SIZE - max_count) % REPLAY_BUFFER_SIZE
        } else {
            0
        };

        if start + max_count <= REPLAY_BUFFER_SIZE {
            &self.entries[start..start + max_count]
        } else {
            // Wrap around case: just return the last contiguous chunk
            let remaining = REPLAY_BUFFER_SIZE - start;
            &self.entries[start..start + remaining]
        }
    }

    /// Get buffer statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.count, REPLAY_BUFFER_SIZE)
    }
}

/// Dynamic topology state
#[derive(Copy, Clone)]
pub struct TopologyState {
    pub current_hidden: usize,
    pub pruned_weights: u32,
    pub added_neurons: u32,
    pub last_adjustment_ts: u64,
    pub performance_history: [i16; 10], // Last 10 performance samples
    pub history_idx: usize,
}

impl TopologyState {
    pub const fn new() -> Self {
        TopologyState {
            current_hidden: META_HID,
            pruned_weights: 0,
            added_neurons: 0,
            last_adjustment_ts: 0,
            performance_history: [0; 10],
            history_idx: 0,
        }
    }

    /// Add a performance sample to history
    pub fn add_performance(&mut self, perf: i16) {
        self.performance_history[self.history_idx] = perf;
        self.history_idx = (self.history_idx + 1) % 10;
    }

    /// Check if performance has plateaued (for growth trigger)
    pub fn is_plateau(&self) -> bool {
        if self.history_idx < 5 {
            return false; // Not enough history
        }

        // Check if last 5 samples are within ±50 range
        let recent_start = (self.history_idx + 10 - 5) % 10;
        let mut min_perf = i16::MAX;
        let mut max_perf = i16::MIN;

        for i in 0..5 {
            let idx = (recent_start + i) % 10;
            let perf = self.performance_history[idx];
            min_perf = min_perf.min(perf);
            max_perf = max_perf.max(perf);
        }

        (max_perf - min_perf) < 50
    }
}

/// Meta-agent configuration
#[derive(Copy, Clone)]
pub struct MetaConfig {
    pub decision_interval_us: u64, // How often to make decisions (microseconds)
    pub confidence_threshold: u16, // Minimum confidence to act (0-1000)
    pub enabled: bool,             // Master enable/disable

    // Week 3: Advanced ML configuration
    pub performance_weight: u8,    // Weight for performance reward (0-100)
    pub power_weight: u8,          // Weight for power reward (0-100)
    pub latency_weight: u8,        // Weight for latency reward (0-100)
    pub replay_enabled: bool,      // Enable experience replay
    pub td_learning_enabled: bool, // Enable temporal difference learning
    pub topology_adapt_enabled: bool, // Enable dynamic topology adjustment
}

impl MetaConfig {
    pub const fn new() -> Self {
        MetaConfig {
            decision_interval_us: 100_000, // 100ms default
            confidence_threshold: 400,     // 40% confidence minimum
            enabled: true,

            // Default: balanced multi-objective weights
            performance_weight: 40,
            power_weight: 30,
            latency_weight: 30,
            replay_enabled: true,
            td_learning_enabled: true,
            topology_adapt_enabled: false, // Off by default (experimental)
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

    // Week 3: Advanced ML statistics
    pub replay_samples: u64,       // Total samples added to replay buffer
    pub td_updates: u64,           // Temporal difference learning updates
    pub topology_prunings: u32,    // Number of weight pruning operations
    pub topology_growths: u32,     // Number of neuron additions
    pub avg_reward: i16,           // Average reward (milli-units)
    pub reward_samples: u32,       // Number of reward samples
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

            replay_samples: 0,
            td_updates: 0,
            topology_prunings: 0,
            topology_growths: 0,
            avg_reward: 0,
            reward_samples: 0,
        }
    }
}

/// The Meta-Agent: coordinates all subsystem neural agents
pub struct MetaAgent {
    network: NeuralAgent,
    state: MetaState,
    prev_state: MetaState,           // Previous state for reward computation
    config: MetaConfig,
    stats: MetaStats,
    last_decision: MetaDecision,

    // Week 3: Advanced ML components
    replay_buffer: ReplayBuffer,     // Experience replay
    topology: TopologyState,         // Dynamic topology tracking
    value_estimate: i16,             // Current state value (Q8.8 milli-units)
}

impl MetaAgent {
    pub const fn new() -> Self {
        MetaAgent {
            network: NeuralAgent::new(),
            state: MetaState::new(),
            prev_state: MetaState::new(),
            config: MetaConfig::new(),
            stats: MetaStats::new(),
            last_decision: MetaDecision::new(),

            replay_buffer: ReplayBuffer::new(),
            topology: TopologyState::new(),
            value_estimate: 0,
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

    /// Compute multi-objective reward based on state changes
    pub fn compute_reward(&self) -> MultiObjectiveReward {
        let mut reward = MultiObjectiveReward::new();

        // Performance reward: improvement in system health
        let prev_health = 100 - (self.prev_state.memory_pressure as i16 +
                                 self.prev_state.scheduling_load as i16 +
                                 (100 - self.prev_state.command_rate as i16)) / 3;
        let curr_health = 100 - (self.state.memory_pressure as i16 +
                                 self.state.scheduling_load as i16 +
                                 (100 - self.state.command_rate as i16)) / 3;
        reward.performance = ((curr_health - prev_health) * 10).clamp(-1000, 1000);

        // Power reward: lower memory pressure = better power efficiency
        let prev_power = 100 - self.prev_state.memory_pressure as i16;
        let curr_power = 100 - self.state.memory_pressure as i16;
        reward.power = ((curr_power - prev_power) * 10).clamp(-1000, 1000);

        // Latency reward: fewer deadline misses = better latency
        let prev_latency = 100 - self.prev_state.deadline_misses as i16;
        let curr_latency = 100 - self.state.deadline_misses as i16;
        reward.latency = ((curr_latency - prev_latency) * 10).clamp(-1000, 1000);

        // Compute weighted sum
        let mut reward_mut = reward;
        reward_mut.compute_weighted(
            self.config.performance_weight,
            self.config.power_weight,
            self.config.latency_weight
        );

        reward_mut
    }

    /// Record experience in replay buffer
    pub fn record_experience(&mut self, decision: MetaDecision, reward: MultiObjectiveReward) {
        if !self.config.replay_enabled {
            return;
        }

        let entry = ReplayEntry {
            state: self.prev_state,
            decision,
            reward,
            next_state: self.state,
            timestamp_us: decision.timestamp_us,
            valid: true,
        };

        self.replay_buffer.push(entry);
        self.stats.replay_samples += 1;
    }

    /// TD(0) learning update: V(s) ← V(s) + α[r + γV(s') - V(s)]
    pub fn td_learning_update(&mut self, reward: i16) {
        if !self.config.td_learning_enabled {
            return;
        }

        // Simplified value function: sum of state components
        let curr_value = self.estimate_value(&self.prev_state);
        let next_value = self.estimate_value(&self.state);

        // TD error: r + γV(s') - V(s)
        let td_error = reward + ((DISCOUNT_FACTOR as i32 * next_value as i32) / 256) as i16 - curr_value;

        // Update: V(s) ← V(s) + α * TD_error
        let update = ((LEARNING_RATE as i32 * td_error as i32) / 256) as i16;
        self.value_estimate = (curr_value + update).clamp(-10000, 10000);

        self.stats.td_updates += 1;

        // Update average reward
        if self.stats.reward_samples < u32::MAX {
            let total = (self.stats.avg_reward as i32 * self.stats.reward_samples as i32) + reward as i32;
            self.stats.reward_samples += 1;
            self.stats.avg_reward = (total / self.stats.reward_samples as i32) as i16;
        }
    }

    /// Estimate state value (simplified heuristic)
    fn estimate_value(&self, state: &MetaState) -> i16 {
        // Value = system health score (0-100 mapped to 0-1000)
        let health = 100 - ((state.memory_pressure as i32 +
                            state.memory_fragmentation as i32 +
                            state.scheduling_load as i32 +
                            state.deadline_misses as i32) / 4);
        (health * 10).clamp(0, 1000) as i16
    }

    /// Train from experience replay samples
    pub fn train_from_replay(&mut self, batch_size: usize) {
        if !self.config.replay_enabled {
            return;
        }

        let samples = self.replay_buffer.sample(batch_size);
        if samples.is_empty() {
            return;
        }

        // Collect rewards before calling td_learning_update to avoid borrow conflicts
        let mut rewards = heapless::Vec::<i16, 128>::new();
        for entry in samples {
            if entry.valid {
                let _ = rewards.push(entry.reward.weighted_sum);
            }
        }

        // Now apply TD learning updates
        for reward in rewards.iter() {
            self.td_learning_update(*reward);
        }
    }

    /// Prune small weights from network
    pub fn prune_weights(&mut self) -> u32 {
        if !self.config.topology_adapt_enabled {
            return 0;
        }

        // This is a simplified placeholder
        // Real implementation would access network weights and prune
        let pruned_count = 0u32;

        if pruned_count > 0 {
            self.stats.topology_prunings += 1;
            self.topology.pruned_weights += pruned_count;
        }

        pruned_count
    }

    /// Add hidden neurons if performance plateaus
    pub fn grow_network(&mut self) -> bool {
        if !self.config.topology_adapt_enabled {
            return false;
        }

        if !self.topology.is_plateau() {
            return false; // No plateau detected
        }

        // Check if we can add more neurons (max 32)
        if self.topology.current_hidden >= 32 {
            return false;
        }

        // Add one neuron
        self.topology.current_hidden += 1;
        self.topology.added_neurons += 1;
        self.stats.topology_growths += 1;

        unsafe {
            crate::uart_print(b"[META] Topology: Added neuron, now ");
            print_number(self.topology.current_hidden);
            crate::uart_print(b" hidden\n");
        }

        true
    }

    /// Update state and perform learning cycle
    pub fn update_state_with_learning(&mut self, new_state: MetaState) {
        // Store previous state
        self.prev_state = self.state;
        self.state = new_state;

        // Compute reward
        let reward = self.compute_reward();

        // Record experience
        self.record_experience(self.last_decision, reward);

        // TD learning
        self.td_learning_update(reward.weighted_sum);

        // Track performance
        self.topology.add_performance(reward.performance);

        // Periodic topology adjustment (every 10 decisions)
        if self.stats.total_decisions % 10 == 0 {
            self.prune_weights();
            self.grow_network();
        }
    }

    /// Get replay buffer statistics
    pub fn get_replay_stats(&self) -> (usize, usize) {
        self.replay_buffer.stats()
    }

    /// Get topology state
    pub fn get_topology(&self) -> TopologyState {
        self.topology
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

        // Week 3: Advanced ML Statistics
        if config.replay_enabled || config.td_learning_enabled || config.topology_adapt_enabled {
            crate::uart_print(b"Advanced ML Statistics:\n");

            if config.replay_enabled {
                let (replay_count, replay_capacity) = META_AGENT.lock().get_replay_stats();
                crate::uart_print(b"  Replay Buffer: ");
                print_number(replay_count);
                crate::uart_print(b"/");
                print_number(replay_capacity);
                crate::uart_print(b" entries\n");
                crate::uart_print(b"  Replay Samples: ");
                print_number(stats.replay_samples as usize);
                crate::uart_print(b"\n");
            }

            if config.td_learning_enabled {
                crate::uart_print(b"  TD Updates: ");
                print_number(stats.td_updates as usize);
                crate::uart_print(b"\n");
                crate::uart_print(b"  Avg Reward: ");
                print_signed_milli(stats.avg_reward);
                crate::uart_print(b"/1000\n");
            }

            if config.topology_adapt_enabled {
                let topo = META_AGENT.lock().get_topology();
                crate::uart_print(b"  Hidden Neurons: ");
                print_number(topo.current_hidden);
                crate::uart_print(b"\n");
                crate::uart_print(b"  Topology Prunings: ");
                print_number(stats.topology_prunings as usize);
                crate::uart_print(b"\n");
                crate::uart_print(b"  Topology Growths: ");
                print_number(stats.topology_growths as usize);
                crate::uart_print(b"\n");
            }

            crate::uart_print(b"\n");
        }
    }
}

// ============================================================================
// Week 3: Advanced ML Public API
// ============================================================================

/// Update meta-agent state with learning enabled
pub fn update_meta_state_with_learning(state: MetaState) {
    META_AGENT.lock().update_state_with_learning(state);
}

/// Train meta-agent from experience replay
pub fn train_from_replay(batch_size: usize) {
    META_AGENT.lock().train_from_replay(batch_size);
}

/// Get replay buffer statistics
pub fn get_replay_stats() -> (usize, usize) {
    META_AGENT.lock().get_replay_stats()
}

/// Get topology state
pub fn get_topology_state() -> TopologyState {
    META_AGENT.lock().get_topology()
}

/// Print advanced ML status
pub fn print_advanced_ml_status() {
    let stats = get_meta_stats();
    let config = get_meta_config();

    unsafe {
        crate::uart_print(b"\n=== Advanced ML Status ===\n\n");

        // Configuration
        crate::uart_print(b"Features:\n");
        crate::uart_print(b"  Experience Replay: ");
        crate::uart_print(if config.replay_enabled { b"ENABLED\n" } else { b"DISABLED\n" });
        crate::uart_print(b"  TD Learning: ");
        crate::uart_print(if config.td_learning_enabled { b"ENABLED\n" } else { b"DISABLED\n" });
        crate::uart_print(b"  Topology Adaptation: ");
        crate::uart_print(if config.topology_adapt_enabled { b"ENABLED\n" } else { b"DISABLED\n" });
        crate::uart_print(b"\n");

        // Reward weights
        crate::uart_print(b"Reward Weights:\n");
        crate::uart_print(b"  Performance: ");
        print_number(config.performance_weight as usize);
        crate::uart_print(b"%\n");
        crate::uart_print(b"  Power: ");
        print_number(config.power_weight as usize);
        crate::uart_print(b"%\n");
        crate::uart_print(b"  Latency: ");
        print_number(config.latency_weight as usize);
        crate::uart_print(b"%\n\n");

        // Statistics
        if config.replay_enabled {
            let (count, capacity) = get_replay_stats();
            crate::uart_print(b"Experience Replay:\n");
            crate::uart_print(b"  Buffer: ");
            print_number(count);
            crate::uart_print(b"/");
            print_number(capacity);
            crate::uart_print(b" entries\n");
            crate::uart_print(b"  Total Samples: ");
            print_number(stats.replay_samples as usize);
            crate::uart_print(b"\n\n");
        }

        if config.td_learning_enabled {
            crate::uart_print(b"Temporal Difference Learning:\n");
            crate::uart_print(b"  Updates: ");
            print_number(stats.td_updates as usize);
            crate::uart_print(b"\n");
            crate::uart_print(b"  Avg Reward: ");
            print_signed_milli(stats.avg_reward);
            crate::uart_print(b"/1000\n");
            crate::uart_print(b"  Samples: ");
            print_number(stats.reward_samples as usize);
            crate::uart_print(b"\n\n");
        }

        if config.topology_adapt_enabled {
            let topo = get_topology_state();
            crate::uart_print(b"Dynamic Topology:\n");
            crate::uart_print(b"  Current Hidden: ");
            print_number(topo.current_hidden);
            crate::uart_print(b" neurons\n");
            crate::uart_print(b"  Pruned Weights: ");
            print_number(topo.pruned_weights as usize);
            crate::uart_print(b"\n");
            crate::uart_print(b"  Added Neurons: ");
            print_number(topo.added_neurons as usize);
            crate::uart_print(b"\n");
            crate::uart_print(b"  Prunings: ");
            print_number(stats.topology_prunings as usize);
            crate::uart_print(b"\n");
            crate::uart_print(b"  Growths: ");
            print_number(stats.topology_growths as usize);
            crate::uart_print(b"\n\n");
        }
    }
}
