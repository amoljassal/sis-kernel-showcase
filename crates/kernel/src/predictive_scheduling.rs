//! Predictive Scheduling (Week 9: AI-Driven Scheduling)
//!
//! This module implements AI-driven scheduling features:
//! - Neural operator prioritization with dynamic priority adjustment
//! - Workload classification (4 classes: LatencySensitive/Throughput/Interactive/Mixed)
//! - Operator affinity learning for cache optimization
//! - Shadow mode A/B testing framework
//! - Feature flags for per-capability control

use spin::Mutex;
use alloc::vec::Vec;
use crate::time::get_timestamp_us;
use crate::trace::metric_kv;

/// Workload classification types
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WorkloadClass {
    LatencySensitive,  // Many small operators, tight deadlines
    Throughput,        // Large operators, batch processing
    Interactive,       // Command-driven, unpredictable
    Mixed,             // Mixed workload characteristics
}

impl WorkloadClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkloadClass::LatencySensitive => "LatencySensitive",
            WorkloadClass::Throughput => "Throughput",
            WorkloadClass::Interactive => "Interactive",
            WorkloadClass::Mixed => "Mixed",
        }
    }

    /// Get recommended scheduling quantum for this workload class (in microseconds)
    pub fn recommended_quantum_us(&self) -> u64 {
        match self {
            WorkloadClass::LatencySensitive => 50,   // 50us for tight deadlines
            WorkloadClass::Throughput => 500,        // 500us for batch processing
            WorkloadClass::Interactive => 100,       // 100us dynamic
            WorkloadClass::Mixed => 100,             // 100us balanced
        }
    }
}

/// Scheduling policy selected based on workload
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SchedulingPolicy {
    RoundRobin,
    FIFO,
    Adaptive,
    MultiLevel,
}

impl SchedulingPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            SchedulingPolicy::RoundRobin => "RoundRobin",
            SchedulingPolicy::FIFO => "FIFO",
            SchedulingPolicy::Adaptive => "Adaptive",
            SchedulingPolicy::MultiLevel => "MultiLevel",
        }
    }
}

/// Operator information for priority adjustment
#[derive(Debug, Copy, Clone)]
pub struct OperatorInfo {
    pub id: u32,
    pub priority: i16,              // -1000 to 1000
    pub base_priority: i16,         // Original priority
    pub deadline_us: u64,           // Deadline if known
    pub last_latency_us: u64,       // Last execution latency
    pub miss_count: u32,            // Number of deadline misses
    pub total_executions: u32,      // Total times executed
}

impl OperatorInfo {
    pub fn new(id: u32, priority: i16) -> Self {
        Self {
            id,
            priority,
            base_priority: priority,
            deadline_us: 0,
            last_latency_us: 0,
            miss_count: 0,
            total_executions: 0,
        }
    }
}

/// Priority adjustment decision record
#[derive(Debug, Copy, Clone)]
pub struct PriorityAdjustment {
    pub timestamp_us: u64,
    pub operator_id: u32,
    pub old_priority: i16,
    pub new_priority: i16,
    pub reason: AdjustmentReason,
    pub outcome_measured: bool,
    pub prevented_miss: bool,
}

/// Reason for priority adjustment
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AdjustmentReason {
    DeadlineMissPattern,    // Repeated misses
    PredictedCriticalPath,  // On critical path
    MetaAgentDirective,     // Meta-agent decision
    Rebalancing,            // Periodic rebalancing
}

impl AdjustmentReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdjustmentReason::DeadlineMissPattern => "DeadlineMissPattern",
            AdjustmentReason::PredictedCriticalPath => "PredictedCriticalPath",
            AdjustmentReason::MetaAgentDirective => "MetaAgentDirective",
            AdjustmentReason::Rebalancing => "Rebalancing",
        }
    }
}

/// Workload features for classification (8 features)
#[derive(Debug, Copy, Clone)]
pub struct WorkloadFeatures {
    pub operator_rate: u16,         // Operators/sec (0-1000)
    pub avg_operator_size: u16,     // Normalized size (0-1000)
    pub latency_variance: u16,      // Variance in execution times (0-1000)
    pub deadline_pressure: u16,     // % of operators near deadline (0-1000)
    pub interactive_score: u16,     // Command-driven activity (0-1000)
    pub batch_score: u16,           // Large batch processing activity (0-1000)
    pub queue_depth: u16,           // Current queue depth (0-1000)
    pub miss_rate: u16,             // Recent deadline miss rate (0-1000)
}

impl WorkloadFeatures {
    pub fn new() -> Self {
        Self {
            operator_rate: 0,
            avg_operator_size: 0,
            latency_variance: 0,
            deadline_pressure: 0,
            interactive_score: 0,
            batch_score: 0,
            queue_depth: 0,
            miss_rate: 0,
        }
    }

    /// Convert features to Q8.8 fixed-point array for neural network
    pub fn to_network_inputs(&self) -> [i16; 8] {
        [
            ((self.operator_rate as i32 * 256) / 1000) as i16,
            ((self.avg_operator_size as i32 * 256) / 1000) as i16,
            ((self.latency_variance as i32 * 256) / 1000) as i16,
            ((self.deadline_pressure as i32 * 256) / 1000) as i16,
            ((self.interactive_score as i32 * 256) / 1000) as i16,
            ((self.batch_score as i32 * 256) / 1000) as i16,
            ((self.queue_depth as i32 * 256) / 1000) as i16,
            ((self.miss_rate as i32 * 256) / 1000) as i16,
        ]
    }
}

/// Operator affinity matrix entry
#[derive(Debug, Copy, Clone)]
pub struct AffinityEntry {
    pub operator_a: u32,
    pub operator_b: u32,
    pub co_occurrence_count: u32,      // How many times they ran together
    pub total_opportunities: u32,       // How many times they could have
    pub affinity_score: u16,           // 0-1000 (co_occurrence / opportunities)
}

impl AffinityEntry {
    pub const fn new(operator_a: u32, operator_b: u32) -> Self {
        Self {
            operator_a,
            operator_b,
            co_occurrence_count: 0,
            total_opportunities: 0,
            affinity_score: 0,
        }
    }

    /// Update affinity score based on co-occurrence
    pub fn update(&mut self) {
        self.total_opportunities += 1;
        if self.total_opportunities > 0 {
            self.affinity_score = ((self.co_occurrence_count as u64 * 1000) / self.total_opportunities as u64) as u16;
        }
    }

    /// Record that these operators ran together
    pub fn record_co_occurrence(&mut self) {
        self.co_occurrence_count += 1;
        self.update();
    }
}

/// Shadow mode configuration for A/B testing
#[derive(Debug, Copy, Clone)]
pub struct ShadowModeConfig {
    pub enabled: bool,
    pub shadow_version: u32,
    pub comparison_count: u32,
    pub disagreement_count: u32,
    pub shadow_better_count: u32,
    pub primary_better_count: u32,
}

impl ShadowModeConfig {
    pub const fn new() -> Self {
        Self {
            enabled: false,
            shadow_version: 0,
            comparison_count: 0,
            disagreement_count: 0,
            shadow_better_count: 0,
            primary_better_count: 0,
        }
    }
}

/// Feature flags for granular control
#[derive(Debug, Copy, Clone)]
pub struct FeatureFlags {
    pub autonomous_scheduling: bool,
    pub workload_classification: bool,
    pub affinity_learning: bool,
    pub shadow_mode: bool,
}

impl FeatureFlags {
    pub const fn new() -> Self {
        Self {
            autonomous_scheduling: false,
            workload_classification: false,
            affinity_learning: false,
            shadow_mode: false,
        }
    }
}

/// Predictive scheduling state
pub struct PredictiveScheduling {
    // Operator tracking
    pub operators: Vec<OperatorInfo>,
    pub max_operators: usize,

    // Priority adjustment history
    pub adjustments: [PriorityAdjustment; 100],
    pub adjustment_head: usize,
    pub adjustment_count: usize,

    // Workload classification
    pub current_workload: WorkloadClass,
    pub current_policy: SchedulingPolicy,
    pub current_quantum_us: u64,
    pub last_classification_us: u64,
    pub classification_interval_us: u64,

    // Operator affinity matrix (simplified - store top N pairings)
    pub affinity_matrix: [AffinityEntry; 50],
    pub affinity_count: usize,

    // Shadow mode
    pub shadow_config: ShadowModeConfig,

    // Feature flags
    pub features: FeatureFlags,

    // Statistics
    pub total_adjustments: u32,
    pub total_classifications: u32,
    pub misses_prevented: u32,
    pub unnecessary_adjustments: u32,
}

impl PredictiveScheduling {
    pub fn new() -> Self {
        Self {
            operators: Vec::new(),
            max_operators: 100,
            adjustments: [PriorityAdjustment {
                timestamp_us: 0,
                operator_id: 0,
                old_priority: 0,
                new_priority: 0,
                reason: AdjustmentReason::Rebalancing,
                outcome_measured: false,
                prevented_miss: false,
            }; 100],
            adjustment_head: 0,
            adjustment_count: 0,
            current_workload: WorkloadClass::Mixed,
            current_policy: SchedulingPolicy::Adaptive,
            current_quantum_us: 100,
            last_classification_us: 0,
            classification_interval_us: 1_000_000, // 1 second
            affinity_matrix: [AffinityEntry::new(0, 0); 50],
            affinity_count: 0,
            shadow_config: ShadowModeConfig::new(),
            features: FeatureFlags::new(),
            total_adjustments: 0,
            total_classifications: 0,
            misses_prevented: 0,
            unnecessary_adjustments: 0,
        }
    }

    /// Register an operator for tracking
    pub fn register_operator(&mut self, id: u32, priority: i16) {
        if self.operators.len() < self.max_operators {
            self.operators.push(OperatorInfo::new(id, priority));
        }
    }

    /// Find operator by ID
    pub fn find_operator_mut(&mut self, id: u32) -> Option<&mut OperatorInfo> {
        self.operators.iter_mut().find(|op| op.id == id)
    }

    /// Record priority adjustment
    pub fn record_adjustment(&mut self, operator_id: u32, old_priority: i16, new_priority: i16, reason: AdjustmentReason) {
        let timestamp = get_timestamp_us();
        self.adjustments[self.adjustment_head] = PriorityAdjustment {
            timestamp_us: timestamp,
            operator_id,
            old_priority,
            new_priority,
            reason,
            outcome_measured: false,
            prevented_miss: false,
        };
        self.adjustment_head = (self.adjustment_head + 1) % 100;
        if self.adjustment_count < 100 {
            self.adjustment_count += 1;
        }
        self.total_adjustments += 1;

        metric_kv("sched_priority_adjustments", self.total_adjustments as usize);
    }

    /// Adjust operator priority (Day 1-3 feature)
    pub fn adjust_operator_priority(&mut self, operator_id: u32, delta: i16, reason: AdjustmentReason) -> bool {
        if let Some(operator) = self.find_operator_mut(operator_id) {
            let old_priority = operator.priority;
            let new_priority = (operator.priority + delta).clamp(-1000, 1000);

            if old_priority != new_priority {
                operator.priority = new_priority;
                self.record_adjustment(operator_id, old_priority, new_priority, reason);
                return true;
            }
        }
        false
    }

    /// Classify workload and adapt policy (Day 4-5 feature)
    pub fn classify_and_adapt(&mut self, features: &WorkloadFeatures) -> WorkloadClass {
        let timestamp = get_timestamp_us();

        // Only classify if enough time has passed
        if timestamp - self.last_classification_us < self.classification_interval_us {
            return self.current_workload;
        }

        // Simple classification logic (will be replaced with neural network)
        let workload = if features.deadline_pressure > 700 && features.operator_rate > 500 {
            WorkloadClass::LatencySensitive
        } else if features.batch_score > 700 && features.avg_operator_size > 700 {
            WorkloadClass::Throughput
        } else if features.interactive_score > 700 {
            WorkloadClass::Interactive
        } else {
            WorkloadClass::Mixed
        };

        // Update policy and quantum based on workload
        self.current_workload = workload;
        self.current_quantum_us = workload.recommended_quantum_us();
        self.current_policy = match workload {
            WorkloadClass::LatencySensitive => SchedulingPolicy::RoundRobin,
            WorkloadClass::Throughput => SchedulingPolicy::FIFO,
            WorkloadClass::Interactive => SchedulingPolicy::Adaptive,
            WorkloadClass::Mixed => SchedulingPolicy::MultiLevel,
        };

        self.last_classification_us = timestamp;
        self.total_classifications += 1;

        metric_kv("sched_classifications", self.total_classifications as usize);
        metric_kv("sched_current_quantum_us", self.current_quantum_us as usize);

        workload
    }

    /// Record operator co-occurrence for affinity learning (Day 6-7 feature)
    pub fn record_operator_affinity(&mut self, operator_a: u32, operator_b: u32, ran_together: bool) {
        // Find or create affinity entry
        let mut entry_idx = None;
        for i in 0..self.affinity_count {
            if (self.affinity_matrix[i].operator_a == operator_a && self.affinity_matrix[i].operator_b == operator_b) ||
               (self.affinity_matrix[i].operator_a == operator_b && self.affinity_matrix[i].operator_b == operator_a) {
                entry_idx = Some(i);
                break;
            }
        }

        let idx = if let Some(idx) = entry_idx {
            idx
        } else if self.affinity_count < 50 {
            let idx = self.affinity_count;
            self.affinity_matrix[idx] = AffinityEntry::new(operator_a, operator_b);
            self.affinity_count += 1;
            idx
        } else {
            return; // Matrix full
        };

        // Update affinity
        if ran_together {
            self.affinity_matrix[idx].record_co_occurrence();
        }
        self.affinity_matrix[idx].update();
    }

    /// Get operators with high affinity (>70%)
    pub fn get_affinity_groups(&self) -> Vec<(u32, u32, u16)> {
        let mut groups = Vec::new();
        for i in 0..self.affinity_count {
            if self.affinity_matrix[i].affinity_score > 700 {
                groups.push((
                    self.affinity_matrix[i].operator_a,
                    self.affinity_matrix[i].operator_b,
                    self.affinity_matrix[i].affinity_score,
                ));
            }
        }
        groups
    }

    /// Enable shadow mode for A/B testing
    pub fn enable_shadow_mode(&mut self, version: u32) {
        self.shadow_config.enabled = true;
        self.shadow_config.shadow_version = version;
    }

    /// Disable shadow mode
    pub fn disable_shadow_mode(&mut self) {
        self.shadow_config.enabled = false;
    }

    /// Compare primary and shadow decisions (for shadow mode)
    pub fn compare_decisions(&mut self, primary_priority: i16, shadow_priority: i16, primary_better: bool) {
        if !self.shadow_config.enabled {
            return;
        }

        self.shadow_config.comparison_count += 1;

        if primary_priority != shadow_priority {
            self.shadow_config.disagreement_count += 1;
        }

        if primary_better {
            self.shadow_config.primary_better_count += 1;
        } else {
            self.shadow_config.shadow_better_count += 1;
        }
    }
}

/// Global predictive scheduling state
pub static PREDICTIVE_SCHEDULING: Mutex<PredictiveScheduling> = Mutex::new(PredictiveScheduling {
    operators: Vec::new(),
    max_operators: 100,
    adjustments: [PriorityAdjustment {
        timestamp_us: 0,
        operator_id: 0,
        old_priority: 0,
        new_priority: 0,
        reason: AdjustmentReason::Rebalancing,
        outcome_measured: false,
        prevented_miss: false,
    }; 100],
    adjustment_head: 0,
    adjustment_count: 0,
    current_workload: WorkloadClass::Mixed,
    current_policy: SchedulingPolicy::Adaptive,
    current_quantum_us: 100,
    last_classification_us: 0,
    classification_interval_us: 1_000_000,
    affinity_matrix: [AffinityEntry::new(0, 0); 50],
    affinity_count: 0,
    shadow_config: ShadowModeConfig::new(),
    features: FeatureFlags::new(),
    total_adjustments: 0,
    total_classifications: 0,
    misses_prevented: 0,
    unnecessary_adjustments: 0,
});

/// Execute scheduling directive from meta-agent (integrated with autonomy)
pub fn execute_scheduling_directive(directive: i16, verbose: bool) -> bool {
    let mut state = PREDICTIVE_SCHEDULING.lock();

    if !state.features.autonomous_scheduling {
        return false; // Feature disabled
    }

    let adjusted = match directive.clamp(-1000, 1000) {
        d if d < -500 => {
            // Critical load - boost priorities of latency-sensitive operators
            if verbose {
                unsafe {
                    crate::uart_print(b"[PRED_SCHED] Critical load - boosting priorities\n");
                }
            }
            // Adjust priorities for operators with recent misses
            let mut adjusted_any = false;
            for i in 0..state.operators.len() {
                if state.operators[i].miss_count > 0 {
                    let id = state.operators[i].id;
                    adjusted_any |= state.adjust_operator_priority(id, 200, AdjustmentReason::MetaAgentDirective);
                }
            }
            adjusted_any
        }
        d if d < 0 => {
            // Moderate load - minor adjustments
            if verbose {
                unsafe {
                    crate::uart_print(b"[PRED_SCHED] Moderate load - minor adjustments\n");
                }
            }
            false
        }
        d if d > 500 => {
            // Low load - restore base priorities
            if verbose {
                unsafe {
                    crate::uart_print(b"[PRED_SCHED] Low load - restoring base priorities\n");
                }
            }
            let mut adjusted_any = false;
            for i in 0..state.operators.len() {
                let op = &state.operators[i];
                if op.priority != op.base_priority {
                    let id = op.id;
                    let delta = op.base_priority - op.priority;
                    adjusted_any |= state.adjust_operator_priority(id, delta, AdjustmentReason::Rebalancing);
                }
            }
            adjusted_any
        }
        _ => {
            // Normal operation
            false
        }
    };

    adjusted
}

/// Extract workload features from current system state
pub fn extract_workload_features() -> WorkloadFeatures {
    // Placeholder implementation - will be integrated with actual telemetry
    WorkloadFeatures::new()
}

/// Classify workload and adapt scheduling policy
pub fn classify_and_adapt_scheduling() -> WorkloadClass {
    let features = extract_workload_features();
    let mut state = PREDICTIVE_SCHEDULING.lock();

    if !state.features.workload_classification {
        return state.current_workload; // Feature disabled
    }

    state.classify_and_adapt(&features)
}

//
// Workload Classifier Neural Network (8→8→4)
//

/// Workload classifier network: 8 inputs → 8 hidden → 4 outputs
pub struct WorkloadClassifierNetwork {
    // Layer 1: 8 inputs → 8 hidden neurons (Q8.8 fixed-point)
    weights_l1: [[i16; 8]; 8],
    biases_l1: [i16; 8],

    // Layer 2: 8 hidden → 4 output neurons (Q8.8)
    weights_l2: [[i16; 8]; 4],
    biases_l2: [i16; 4],

    // Inference count
    infer_count: u32,
}

impl WorkloadClassifierNetwork {
    /// Create new workload classifier with random initialization
    pub fn new() -> Self {
        Self {
            weights_l1: [[0; 8]; 8],
            biases_l1: [0; 8],
            weights_l2: [[0; 8]; 4],
            biases_l2: [0; 4],
            infer_count: 0,
        }
    }

    /// Initialize with small random weights (using timestamp for seed)
    pub fn init_random(&mut self) {
        let seed = get_timestamp_us() as u32;
        let mut rng_state = seed;

        // Simple LCG for pseudo-random initialization
        let mut next_random = || {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            ((rng_state / 65536) % 512) as i16 - 256 // Range: -256 to 256
        };

        // Initialize layer 1 weights and biases
        for i in 0..8 {
            for j in 0..8 {
                self.weights_l1[i][j] = next_random() / 4; // Small weights: -64 to 64
            }
            self.biases_l1[i] = next_random() / 8; // Small biases: -32 to 32
        }

        // Initialize layer 2 weights and biases
        for i in 0..4 {
            for j in 0..8 {
                self.weights_l2[i][j] = next_random() / 4;
            }
            self.biases_l2[i] = next_random() / 8;
        }
    }

    /// ReLU activation (Q8.8): max(0, x)
    #[inline(always)]
    fn relu(x: i16) -> i16 {
        if x > 0 { x } else { 0 }
    }

    /// Softmax approximation for classification (Q8.8)
    /// Returns index of maximum value (argmax)
    fn argmax(outputs: &[i16; 4]) -> usize {
        let mut max_idx = 0;
        let mut max_val = outputs[0];
        for i in 1..4 {
            if outputs[i] > max_val {
                max_val = outputs[i];
                max_idx = i;
            }
        }
        max_idx
    }

    /// Inference: 8 inputs → 8 hidden → 4 outputs
    /// Returns: (class_index, confidence)
    pub fn infer(&mut self, inputs: &[i16; 8]) -> (usize, u16) {
        self.infer_count += 1;

        // Layer 1: 8 inputs → 8 hidden (with ReLU)
        let mut hidden = [0i16; 8];
        for i in 0..8 {
            let mut sum = self.biases_l1[i] as i32;
            for j in 0..8 {
                // Q8.8 * Q8.8 = Q16.16, shift right 8 bits to get Q8.8
                sum += ((self.weights_l1[i][j] as i32) * (inputs[j] as i32)) >> 8;
            }
            hidden[i] = Self::relu(sum.clamp(-32768, 32767) as i16);
        }

        // Layer 2: 8 hidden → 4 outputs
        let mut outputs = [0i16; 4];
        for i in 0..4 {
            let mut sum = self.biases_l2[i] as i32;
            for j in 0..8 {
                sum += ((self.weights_l2[i][j] as i32) * (hidden[j] as i32)) >> 8;
            }
            outputs[i] = sum.clamp(-32768, 32767) as i16;
        }

        // Get class with maximum output (argmax)
        let class_idx = Self::argmax(&outputs);

        // Compute confidence: (max - second_max) / max
        let max_val = outputs[class_idx];
        let mut second_max = i16::MIN;
        for i in 0..4 {
            if i != class_idx && outputs[i] > second_max {
                second_max = outputs[i];
            }
        }

        let confidence = if max_val > 0 {
            let diff = (max_val - second_max).max(0) as u32;
            let conf = ((diff * 1000) / (max_val as u32)).min(1000);
            conf as u16
        } else {
            500 // Low confidence if all outputs are negative
        };

        (class_idx, confidence)
    }

    /// Update weights based on outcome (simple gradient descent)
    /// target_class: 0=LatencySensitive, 1=Throughput, 2=Interactive, 3=Mixed
    pub fn learn(&mut self, inputs: &[i16; 8], target_class: usize, learning_rate: i16) {
        // Simplified learning: adjust weights toward correct classification
        // This is a placeholder - full backpropagation would be more complex

        // Re-run inference to get hidden layer activations
        let mut hidden = [0i16; 8];
        for i in 0..8 {
            let mut sum = self.biases_l1[i] as i32;
            for j in 0..8 {
                sum += ((self.weights_l1[i][j] as i32) * (inputs[j] as i32)) >> 8;
            }
            hidden[i] = Self::relu(sum.clamp(-32768, 32767) as i16);
        }

        // Compute output errors (target - output)
        let mut outputs = [0i16; 4];
        for i in 0..4 {
            let mut sum = self.biases_l2[i] as i32;
            for j in 0..8 {
                sum += ((self.weights_l2[i][j] as i32) * (hidden[j] as i32)) >> 8;
            }
            outputs[i] = sum.clamp(-32768, 32767) as i16;
        }

        // Target: 1.0 (256 in Q8.8) for correct class, 0.0 for others
        let mut errors = [0i16; 4];
        for i in 0..4 {
            let target = if i == target_class { 256 } else { 0 };
            errors[i] = target - outputs[i];
        }

        // Update layer 2 weights: w += learning_rate * error * hidden
        for i in 0..4 {
            let error = errors[i] as i32;
            for j in 0..8 {
                let gradient = ((error * hidden[j] as i32) >> 8) as i16;
                let update = ((learning_rate as i32 * gradient as i32) >> 8) as i16;
                self.weights_l2[i][j] = (self.weights_l2[i][j] as i32 + update as i32)
                    .clamp(-32768, 32767) as i16;
            }
            // Update bias
            let bias_update = ((learning_rate as i32 * error) >> 8) as i16;
            self.biases_l2[i] = (self.biases_l2[i] as i32 + bias_update as i32)
                .clamp(-32768, 32767) as i16;
        }

        // Backpropagate to layer 1 (simplified)
        let mut hidden_errors = [0i32; 8];
        for j in 0..8 {
            let mut err_sum = 0i32;
            for i in 0..4 {
                err_sum += (errors[i] as i32 * self.weights_l2[i][j] as i32) >> 8;
            }
            hidden_errors[j] = err_sum;
        }

        // Update layer 1 weights: w += learning_rate * error * input
        for i in 0..8 {
            let error = hidden_errors[i];
            for j in 0..8 {
                let gradient = ((error * inputs[j] as i32) >> 8) as i16;
                let update = ((learning_rate as i32 * gradient as i32) >> 8) as i16;
                self.weights_l1[i][j] = (self.weights_l1[i][j] as i32 + update as i32)
                    .clamp(-32768, 32767) as i16;
            }
            // Update bias
            let bias_update = ((learning_rate as i32 * error) >> 8) as i16;
            self.biases_l1[i] = (self.biases_l1[i] as i32 + bias_update as i32)
                .clamp(-32768, 32767) as i16;
        }
    }
}

/// Global workload classifier network
pub static WORKLOAD_CLASSIFIER: Mutex<WorkloadClassifierNetwork> = Mutex::new(WorkloadClassifierNetwork {
    weights_l1: [[0; 8]; 8],
    biases_l1: [0; 8],
    weights_l2: [[0; 8]; 4],
    biases_l2: [0; 4],
    infer_count: 0,
});

/// Neural network-based workload classification
pub fn neural_classify_workload(features: &WorkloadFeatures) -> (WorkloadClass, u16) {
    let inputs = features.to_network_inputs();
    let mut classifier = WORKLOAD_CLASSIFIER.lock();

    // Initialize network if this is the first inference
    if classifier.infer_count == 0 {
        classifier.init_random();
    }

    let (class_idx, confidence) = classifier.infer(&inputs);

    let workload = match class_idx {
        0 => WorkloadClass::LatencySensitive,
        1 => WorkloadClass::Throughput,
        2 => WorkloadClass::Interactive,
        3 => WorkloadClass::Mixed,
        _ => WorkloadClass::Mixed,
    };

    metric_kv("sched_classifier_inference_count", classifier.infer_count as usize);
    metric_kv("sched_classifier_confidence", confidence as usize);

    (workload, confidence)
}
