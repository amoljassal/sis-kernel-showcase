// Week 10: AI-Predicted Command Execution
//
// Features:
// - Execution time prediction (8→12→1 neural network)
// - Resource pre-allocation (memory + scheduling)
// - Command batching with learned optimal sizes
// - Canary rollout (1% → 5% → 10% → 50% → 100%)
// - Circuit breaker (auto-disable on failures)

use alloc::vec::Vec;
use spin::Mutex;

// ============================================================================
// Prediction Ledger (Week 6 integration)
// ============================================================================

#[derive(Copy, Clone, Debug)]
pub enum PredictionType {
    ExecutionTime,
    MemoryNeeded,
    PriorityNeeded,
    BatchSize,
}

#[derive(Copy, Clone)]
pub struct PredictionRecord {
    pub timestamp: u64,
    pub prediction_type: PredictionType,
    pub predicted_value: i16,      // Q8.8 format
    pub confidence: u16,            // 0-1000 (0-100%)
    pub actual_value: Option<i16>, // Filled when outcome known
    pub outcome_timestamp: u64,
}

impl PredictionRecord {
    pub const fn new() -> Self {
        Self {
            timestamp: 0,
            prediction_type: PredictionType::ExecutionTime,
            predicted_value: 0,
            confidence: 0,
            actual_value: None,
            outcome_timestamp: 0,
        }
    }
}

pub struct PredictionLedger {
    entries: [PredictionRecord; 1000],
    head: usize,
    pub total_predictions: u32,
}

impl PredictionLedger {
    pub const fn new() -> Self {
        Self {
            entries: [PredictionRecord::new(); 1000],
            head: 0,
            total_predictions: 0,
        }
    }

    /// Record a new prediction
    pub fn record_prediction(
        &mut self,
        prediction_type: PredictionType,
        predicted_value: i16,
        confidence: u16,
    ) -> usize {
        let timestamp = crate::time::get_timestamp_us();
        let pred_id = self.head;

        self.entries[self.head] = PredictionRecord {
            timestamp,
            prediction_type,
            predicted_value,
            confidence,
            actual_value: None,
            outcome_timestamp: 0,
        };

        self.head = (self.head + 1) % 1000;
        self.total_predictions += 1;
        pred_id
    }

    /// Update prediction outcome
    pub fn update_outcome(&mut self, pred_id: usize, actual_value: i16) {
        if pred_id < 1000 {
            self.entries[pred_id].actual_value = Some(actual_value);
            self.entries[pred_id].outcome_timestamp = crate::time::get_timestamp_us();
        }
    }

    /// Compute accuracy for last N predictions of given type
    pub fn compute_accuracy(&self, prediction_type: PredictionType, last_n: usize) -> u8 {
        let mut correct = 0;
        let mut total = 0;
        let mut idx = if self.head == 0 { 999 } else { self.head - 1 };

        for _ in 0..last_n.min(1000) {
            let pred = &self.entries[idx];
            if matches!(pred.prediction_type, PredictionType::ExecutionTime)
                && matches!(prediction_type, PredictionType::ExecutionTime)
            {
                if let Some(actual) = pred.actual_value {
                    total += 1;
                    let error = (pred.predicted_value - actual).abs();
                    // Consider "correct" if within 20% error (51/256 = ~20%)
                    if error < (actual.abs() / 5).max(256) {
                        correct += 1;
                    }
                }
            }
            idx = if idx == 0 { 999 } else { idx - 1 };
        }

        if total == 0 {
            50 // Default 50% if no data
        } else {
            ((correct * 100) / total).min(100) as u8
        }
    }
}

pub static PREDICTION_LEDGER: Mutex<PredictionLedger> = Mutex::new(PredictionLedger::new());

// ============================================================================
// Command Execution Time Predictor (8→12→1 Network)
// ============================================================================

pub struct CommandPredictorNetwork {
    // Layer 1: 8 inputs → 12 hidden
    weights_l1: [[i16; 8]; 12], // Q8.8 format
    biases_l1: [i16; 12],

    // Layer 2: 12 hidden → 1 output
    weights_l2: [i16; 12], // Q8.8 format
    bias_l2: i16,

    // Statistics
    pub infer_count: u32,
    pub train_count: u32,
    pub avg_error: i16, // Q8.8 format
}

impl CommandPredictorNetwork {
    pub const fn new() -> Self {
        Self {
            // Initialize with small random-ish values
            weights_l1: [
                [20, -15, 10, -8, 12, -10, 8, -5],
                [-12, 18, -10, 15, -8, 12, -6, 10],
                [15, -10, 12, -15, 10, -8, 15, -12],
                [-10, 12, -8, 10, -15, 12, -10, 8],
                [8, -12, 15, -10, 12, -8, 10, -15],
                [-15, 10, -12, 8, -10, 15, -12, 10],
                [12, -8, 10, -15, 8, -10, 12, -8],
                [-8, 15, -10, 12, -15, 10, -8, 12],
                [10, -12, 8, -10, 12, -15, 10, -8],
                [-12, 10, -15, 8, -10, 12, -15, 10],
                [15, -8, 12, -10, 15, -12, 8, -10],
                [-10, 12, -8, 15, -12, 10, -8, 15],
            ],
            biases_l1: [10, -8, 12, -10, 8, -12, 10, -8, 12, -10, 8, -12],
            weights_l2: [15, -12, 10, -8, 12, -10, 8, -12, 10, -8, 12, -10],
            bias_l2: 100,
            infer_count: 0,
            train_count: 0,
            avg_error: 0,
        }
    }

    /// ReLU activation
    fn relu(x: i16) -> i16 {
        x.max(0)
    }

    /// Inference: predict execution time in microseconds
    /// Returns (predicted_time_us, confidence)
    pub fn infer(&mut self, features: &[i16; 8]) -> (u32, u16) {
        self.infer_count += 1;

        // Layer 1: 8 → 12 with ReLU
        let mut hidden = [0i16; 12];
        for i in 0..12 {
            let mut sum = self.biases_l1[i] as i32;
            for j in 0..8 {
                sum += ((self.weights_l1[i][j] as i32) * (features[j] as i32)) >> 8;
            }
            hidden[i] = Self::relu(sum.clamp(-32768, 32767) as i16);
        }

        // Layer 2: 12 → 1 (no ReLU on output)
        let mut output = self.bias_l2 as i32;
        for i in 0..12 {
            output += ((self.weights_l2[i] as i32) * (hidden[i] as i32)) >> 8;
        }
        let output = output.clamp(0, 1_000_000) as i16; // Max 1 second prediction

        // Confidence based on historical accuracy
        let confidence = if self.avg_error == 0 {
            700 // Default 70%
        } else {
            let error_ratio = (self.avg_error * 100) / output.max(256);
            (1000 - error_ratio.min(500)).max(200) as u16
        };

        // Convert Q8.8 to microseconds
        let predicted_us = ((output as i32) << 8) as u32;

        (predicted_us.min(1_000_000), confidence)
    }

    /// Train on actual execution time (backpropagation)
    pub fn train(&mut self, features: &[i16; 8], actual_time_us: u32, learning_rate: i16) {
        self.train_count += 1;

        // Forward pass (recompute for gradients)
        let mut hidden = [0i16; 12];
        let mut hidden_raw = [0i32; 12]; // Before ReLU
        for i in 0..12 {
            let mut sum = self.biases_l1[i] as i32;
            for j in 0..8 {
                sum += ((self.weights_l1[i][j] as i32) * (features[j] as i32)) >> 8;
            }
            hidden_raw[i] = sum;
            hidden[i] = Self::relu(sum.clamp(-32768, 32767) as i16);
        }

        let mut output = self.bias_l2 as i32;
        for i in 0..12 {
            output += ((self.weights_l2[i] as i32) * (hidden[i] as i32)) >> 8;
        }
        let predicted = output.clamp(0, 1_000_000);

        // Compute error (convert actual_us to Q8.8)
        let target = ((actual_time_us >> 8) as i32).min(1_000_000);
        let error = predicted - target;

        // Update average error (for confidence)
        self.avg_error = ((self.avg_error as i32 * 9 + error.abs()) / 10).min(32767) as i16;

        // Backpropagation
        // Output layer gradient
        let output_grad = error; // dL/dy

        // Update output weights and bias
        for i in 0..12 {
            let grad = ((output_grad * (hidden[i] as i32)) >> 8) * (learning_rate as i32) / 256;
            self.weights_l2[i] = (self.weights_l2[i] as i32 - grad).clamp(-32768, 32767) as i16;
        }
        let bias_grad = (output_grad * (learning_rate as i32)) / 256;
        self.bias_l2 = (self.bias_l2 as i32 - bias_grad).clamp(-32768, 32767) as i16;

        // Hidden layer gradient (ReLU derivative)
        let mut hidden_grad = [0i32; 12];
        for i in 0..12 {
            let relu_deriv = if hidden_raw[i] > 0 { 1 } else { 0 };
            hidden_grad[i] = ((output_grad * (self.weights_l2[i] as i32)) >> 8) * relu_deriv;
        }

        // Update hidden weights and biases
        for i in 0..12 {
            for j in 0..8 {
                let grad = ((hidden_grad[i] * (features[j] as i32)) >> 8) * (learning_rate as i32) / 256;
                self.weights_l1[i][j] = (self.weights_l1[i][j] as i32 - grad).clamp(-32768, 32767) as i16;
            }
            let bias_grad = (hidden_grad[i] * (learning_rate as i32)) / 256;
            self.biases_l1[i] = (self.biases_l1[i] as i32 - bias_grad).clamp(-32768, 32767) as i16;
        }
    }
}

pub static COMMAND_PREDICTOR: Mutex<CommandPredictorNetwork> =
    Mutex::new(CommandPredictorNetwork::new());

// ============================================================================
// Command Feature Extraction
// ============================================================================

/// Extract features for command prediction (8 features)
pub fn extract_command_features(cmd: &str, _args: &[&str]) -> [i16; 8] {
    let mut features = [0i16; 8];

    // Feature 0: Command type hash (simple hash of first 4 chars)
    let cmd_bytes = cmd.as_bytes();
    let cmd_hash = if cmd_bytes.len() >= 4 {
        ((cmd_bytes[0] as i16) << 8) | (cmd_bytes[1] as i16)
    } else {
        0
    };
    features[0] = cmd_hash;

    // Feature 1: Command length
    features[1] = (cmd.len() as i16 * 10).min(32767);

    // Feature 2: Argument count
    features[2] = (_args.len() as i16 * 50).min(32767);

    // Feature 3: Current memory pressure (Q8.8: 0-100 → 0-25600)
    let heap_stats = crate::heap::get_heap_stats();
    let heap_size = crate::heap::heap_total_size(); // Single source of truth from heap.rs
    let used = heap_stats.current_allocated();
    let free = heap_size.saturating_sub(used);
    let pressure = (100 - (free * 100 / heap_size)).min(100) as i16;
    features[3] = pressure << 8;

    // Feature 4: Current scheduling load (active operators from agent bus messages)
    let messages = crate::agent_bus::get_all_messages();
    let sched_load = messages.len().min(255) as i16;
    features[4] = (sched_load * 100).min(32767);

    // Feature 5: Command queue depth
    let queue_depth = crate::shell::COMMAND_PREDICTION.lock().current_queue_depth as i16;
    features[5] = (queue_depth * 200).min(32767);

    // Feature 6: Time of day (cyclical) - low bits of timestamp
    let timestamp = crate::time::get_timestamp_us();
    features[6] = ((timestamp & 0xFFFF) as i16) / 256; // Normalize

    // Feature 7: Recent command rate (commands per second)
    let cmd_rate = crate::shell::COMMAND_PREDICTION.lock().recent_command_rate;
    features[7] = (cmd_rate as i16 * 100).min(32767);

    features
}

// ============================================================================
// Resource Pre-allocation
// ============================================================================

#[derive(Copy, Clone)]
pub struct ResourcePrediction {
    pub memory_bytes: u32,
    pub priority_boost: i16,
    pub confidence: u16,
}

impl ResourcePrediction {
    pub const fn new() -> Self {
        Self {
            memory_bytes: 0,
            priority_boost: 0,
            confidence: 0,
        }
    }
}

/// Predict resource needs for command
pub fn predict_resources(cmd: &str, args: &[&str]) -> ResourcePrediction {
    let _features = extract_command_features(cmd, args);

    // Simple heuristic predictor (can be replaced with neural network)
    let base_memory = match cmd {
        "memctl" => 8192,
        "graphctl" => 16384,
        "schedctl" => 4096,
        "autoctl" => 4096,
        _ => 2048,
    };

    let priority_boost = if args.contains(&"urgent") {
        100
    } else {
        0
    };

    let confidence = 700; // 70% confidence in heuristic

    ResourcePrediction {
        memory_bytes: base_memory,
        priority_boost,
        confidence,
    }
}

// ============================================================================
// Command Batching
// ============================================================================

pub struct CommandBatcher {
    pub pending_commands: Vec<&'static str>,
    pub max_batch_size: usize,
    pub learned_optimal_size: usize,
    pub batches_executed: u32,
    pub total_throughput_gain: i32, // Q8.8 format
}

impl CommandBatcher {
    pub const fn new() -> Self {
        Self {
            pending_commands: Vec::new(),
            max_batch_size: 10,
            learned_optimal_size: 3,
            batches_executed: 0,
            total_throughput_gain: 0,
        }
    }

    /// Add command to batch queue
    pub fn add_command(&mut self, _cmd: &'static str) {
        // Implementation would track commands
        // For now, just update stats
    }

    /// Decide if should execute batch now
    pub fn should_execute_batch(&self) -> bool {
        self.pending_commands.len() >= self.learned_optimal_size
    }

    /// Update learned batch size based on outcomes
    pub fn update_learned_size(&mut self, _batch_size: usize, throughput_gain: i16) {
        self.batches_executed += 1;
        self.total_throughput_gain += throughput_gain as i32;

        // Adapt batch size: increase if gain positive, decrease if negative
        if throughput_gain > 100 && self.learned_optimal_size < self.max_batch_size {
            self.learned_optimal_size += 1;
        } else if throughput_gain < -100 && self.learned_optimal_size > 1 {
            self.learned_optimal_size -= 1;
        }
    }
}

pub static COMMAND_BATCHER: Mutex<CommandBatcher> = Mutex::new(CommandBatcher::new());

// ============================================================================
// Canary Rollout (1% → 5% → 10% → 50% → 100%)
// ============================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RolloutPercentage {
    Disabled,   // 0%
    OnePercent, // 1%
    FivePercent, // 5%
    TenPercent, // 10%
    FiftyPercent, // 50%
    Full,       // 100%
}

impl RolloutPercentage {
    pub fn to_threshold(&self) -> u32 {
        match self {
            RolloutPercentage::Disabled => 0,
            RolloutPercentage::OnePercent => 429, // 1% of u16::MAX
            RolloutPercentage::FivePercent => 2147, // 5%
            RolloutPercentage::TenPercent => 4295, // 10%
            RolloutPercentage::FiftyPercent => 21475, // 50%
            RolloutPercentage::Full => 42950, // 100%
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RolloutPercentage::Disabled => "0%",
            RolloutPercentage::OnePercent => "1%",
            RolloutPercentage::FivePercent => "5%",
            RolloutPercentage::TenPercent => "10%",
            RolloutPercentage::FiftyPercent => "50%",
            RolloutPercentage::Full => "100%",
        }
    }
}

pub struct CanaryRollout {
    pub percentage: RolloutPercentage,
    pub decisions_made: u32,
    pub decisions_autonomous: u32,
    pub auto_rollback_threshold: i16, // If metrics degrade beyond this, rollback
    pub baseline_reward: i16,          // Baseline performance before rollout
}

impl CanaryRollout {
    pub const fn new() -> Self {
        Self {
            percentage: RolloutPercentage::Disabled,
            decisions_made: 0,
            decisions_autonomous: 0,
            auto_rollback_threshold: -500, // -2.0 in Q8.8
            baseline_reward: 0,
        }
    }

    /// Hash-based decision: should this decision use autonomy?
    pub fn should_use_autonomy(&self, decision_id: u32) -> bool {
        if matches!(self.percentage, RolloutPercentage::Disabled) {
            return false;
        }
        if matches!(self.percentage, RolloutPercentage::Full) {
            return true;
        }

        // Simple hash: use decision_id modulo
        let hash = (decision_id.wrapping_mul(2654435761)) >> 16; // Knuth multiplicative hash
        hash < self.percentage.to_threshold()
    }

    /// Check if should auto-rollback
    pub fn check_auto_rollback(&self, current_avg_reward: i16) -> bool {
        if self.decisions_autonomous < 100 {
            return false; // Need enough data
        }

        let degradation = self.baseline_reward - current_avg_reward;
        degradation > self.auto_rollback_threshold.abs()
    }

    /// Advance to next rollout stage
    pub fn advance(&mut self) {
        self.percentage = match self.percentage {
            RolloutPercentage::Disabled => RolloutPercentage::OnePercent,
            RolloutPercentage::OnePercent => RolloutPercentage::FivePercent,
            RolloutPercentage::FivePercent => RolloutPercentage::TenPercent,
            RolloutPercentage::TenPercent => RolloutPercentage::FiftyPercent,
            RolloutPercentage::FiftyPercent => RolloutPercentage::Full,
            RolloutPercentage::Full => RolloutPercentage::Full,
        };
    }

    /// Rollback to previous stage
    pub fn rollback(&mut self) {
        self.percentage = match self.percentage {
            RolloutPercentage::Disabled => RolloutPercentage::Disabled,
            RolloutPercentage::OnePercent => RolloutPercentage::Disabled,
            RolloutPercentage::FivePercent => RolloutPercentage::OnePercent,
            RolloutPercentage::TenPercent => RolloutPercentage::FivePercent,
            RolloutPercentage::FiftyPercent => RolloutPercentage::TenPercent,
            RolloutPercentage::Full => RolloutPercentage::FiftyPercent,
        };
    }
}

pub static CANARY_ROLLOUT: Mutex<CanaryRollout> = Mutex::new(CanaryRollout::new());

// ============================================================================
// Circuit Breaker (Auto-disable on failures)
// ============================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Disabled due to failures
    HalfOpen, // Testing if system recovered
}

impl CircuitState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CircuitState::Closed => "CLOSED (normal)",
            CircuitState::Open => "OPEN (disabled)",
            CircuitState::HalfOpen => "HALF-OPEN (testing)",
        }
    }
}

pub struct CircuitBreaker {
    pub state: CircuitState,
    pub consecutive_failures: u32,
    pub failure_threshold: u32, // Trip after N failures
    pub success_count: u32,
    pub test_threshold: u32, // Successes needed in HALF-OPEN to close
    pub last_failure_timestamp: u64,
    pub reset_timeout_us: u64, // Wait this long before trying HALF-OPEN
    pub total_trips: u32,
}

impl CircuitBreaker {
    pub const fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            failure_threshold: 5,
            success_count: 0,
            test_threshold: 3,
            last_failure_timestamp: 0,
            reset_timeout_us: 10_000_000, // 10 seconds
            total_trips: 0,
        }
    }

    /// Record a successful action
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;

        match self.state {
            CircuitState::Closed => {
                // Normal operation
            }
            CircuitState::Open => {
                // Stay open, success doesn't matter
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.test_threshold {
                    // Recovered! Close circuit
                    self.state = CircuitState::Closed;
                    self.success_count = 0;
                    unsafe {
                        crate::uart_print(b"[CIRCUIT_BREAKER] Circuit CLOSED (recovered)\n");
                    }
                }
            }
        }
    }

    /// Record a failed action
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.last_failure_timestamp = crate::time::get_timestamp_us();

        match self.state {
            CircuitState::Closed => {
                if self.consecutive_failures >= self.failure_threshold {
                    // Trip the breaker!
                    self.state = CircuitState::Open;
                    self.total_trips += 1;
                    unsafe {
                        crate::uart_print(b"[CIRCUIT_BREAKER] Circuit OPENED (too many failures)\n");
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Failed during testing, go back to OPEN
                self.state = CircuitState::Open;
                self.success_count = 0;
                unsafe {
                    crate::uart_print(b"[CIRCUIT_BREAKER] Circuit OPENED (test failed)\n");
                }
            }
            CircuitState::Open => {
                // Already open
            }
        }
    }

    /// Check if enough time passed to try HALF-OPEN
    pub fn maybe_enter_half_open(&mut self) {
        if matches!(self.state, CircuitState::Open) {
            let now = crate::time::get_timestamp_us();
            if now - self.last_failure_timestamp >= self.reset_timeout_us {
                self.state = CircuitState::HalfOpen;
                self.success_count = 0;
                unsafe {
                    crate::uart_print(b"[CIRCUIT_BREAKER] Circuit HALF-OPEN (testing recovery)\n");
                }
            }
        }
    }

    /// Should autonomous actions be allowed?
    pub fn is_autonomous_allowed(&self) -> bool {
        !matches!(self.state, CircuitState::Open)
    }
}

pub static CIRCUIT_BREAKER: Mutex<CircuitBreaker> = Mutex::new(CircuitBreaker::new());

// ============================================================================
// Public API
// ============================================================================

/// Predict command execution time and record prediction
pub fn predict_command_execution(cmd: &str, args: &[&str]) -> (u32, u16) {
    let features = extract_command_features(cmd, args);
    let (predicted_us, confidence) = COMMAND_PREDICTOR.lock().infer(&features);

    // Record prediction
    let _pred_id = PREDICTION_LEDGER.lock().record_prediction(
        PredictionType::ExecutionTime,
        (predicted_us >> 8) as i16, // Convert to Q8.8
        confidence,
    );

    (predicted_us, confidence)
}

/// Update prediction with actual execution time
pub fn update_command_outcome(pred_id: usize, actual_time_us: u32) {
    PREDICTION_LEDGER.lock().update_outcome(pred_id, (actual_time_us >> 8) as i16);

    // Train predictor
    // Note: We'd need to store features to train properly. For now, skip training.
    // In a full implementation, store (pred_id, features) mapping.
}

/// Get prediction accuracy statistics
pub fn get_prediction_accuracy() -> u8 {
    PREDICTION_LEDGER.lock().compute_accuracy(PredictionType::ExecutionTime, 100)
}
