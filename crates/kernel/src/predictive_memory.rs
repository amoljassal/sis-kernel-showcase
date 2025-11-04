//! Predictive Memory Management (Week 8: AI-Driven Memory Management)
//!
//! This module implements AI-driven memory management features:
//! - Predictive compaction with 5-second lookahead
//! - Neural heap allocation strategies (Conservative/Balanced/Aggressive)
//! - Allocation size prediction per command type
//! - Learning-based outcome tracking and experience replay

use spin::Mutex;
use alloc::vec::Vec;
use crate::time::get_timestamp_us;
use core::sync::atomic::{AtomicBool, Ordering};

// UX Enhancement: Query mode and approval mode flags
pub static MEMORY_QUERY_MODE: AtomicBool = AtomicBool::new(false);
pub static MEMORY_APPROVAL_MODE: AtomicBool = AtomicBool::new(false);

/// Allocation strategy selected by meta-agent
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AllocationStrategy {
    Conservative,  // Small chunks, frequent compaction, low fragmentation tolerance
    Balanced,      // Default balanced approach
    Aggressive,    // Large chunks, defer compaction, maximize throughput
}

impl AllocationStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            AllocationStrategy::Conservative => "Conservative",
            AllocationStrategy::Balanced => "Balanced",
            AllocationStrategy::Aggressive => "Aggressive",
        }
    }
}

/// Compaction decision record
#[derive(Debug, Copy, Clone)]
pub struct CompactionDecision {
    pub timestamp_us: u64,
    pub predicted_frag_future: u8,      // Predicted fragmentation 5s ahead
    pub confidence: u16,                 // 0-1000 milli-units
    pub current_frag: u8,                // Current fragmentation
    pub decision: bool,                  // true = compact, false = skip
    pub outcome_measured: bool,          // Set to true when outcome is known
    pub prevented_oom: bool,             // Did compaction prevent OOM?
}

/// Allocation size record for a specific command
#[derive(Debug, Copy, Clone)]
pub struct AllocationRecord {
    pub timestamp_us: u64,
    pub command_hash: u32,              // Hash of command string
    pub allocation_size: usize,         // Bytes allocated
}

/// Allocation size predictor per command type
#[derive(Debug)]
pub struct AllocationPredictor {
    pub command_hash: u32,
    pub history: [AllocationRecord; 10], // Ring buffer of last 10
    pub head: usize,
    pub count: usize,
}

impl AllocationPredictor {
    pub fn new(command_hash: u32) -> Self {
        Self {
            command_hash,
            history: [AllocationRecord {
                timestamp_us: 0,
                command_hash: 0,
                allocation_size: 0,
            }; 10],
            head: 0,
            count: 0,
        }
    }

    /// Add allocation record
    pub fn record(&mut self, size: usize, timestamp: u64) {
        self.history[self.head] = AllocationRecord {
            timestamp_us: timestamp,
            command_hash: self.command_hash,
            allocation_size: size,
        };
        self.head = (self.head + 1) % 10;
        if self.count < 10 {
            self.count += 1;
        }
    }

    /// Predict next allocation size (simple linear average of last N)
    /// Returns: (predicted_size, confidence)
    pub fn predict(&self) -> (usize, u16) {
        if self.count == 0 {
            return (0, 0); // No data
        }

        // Calculate average of available samples
        let mut sum = 0usize;
        let n = self.count.min(10);
        for i in 0..n {
            let idx = (self.head + 10 - n + i) % 10;
            sum = sum.saturating_add(self.history[idx].allocation_size);
        }

        let avg = sum / n;

        // Confidence increases with more samples
        // 1-2 samples: 20%, 3-5: 50%, 6-9: 70%, 10: 90%
        let confidence = match self.count {
            1..=2 => 200,
            3..=5 => 500,
            6..=9 => 700,
            _ => 900,
        };

        (avg, confidence)
    }
}

/// Strategy change record
#[derive(Debug, Copy, Clone)]
pub struct StrategyChange {
    pub timestamp_us: u64,
    pub old_strategy: AllocationStrategy,
    pub new_strategy: AllocationStrategy,
    pub reason_directive: i16,          // Meta-agent memory directive
    pub outcome_reward: i16,            // +100 if prevented OOM, -50 if thrashing
    pub outcome_measured: bool,
}

/// Predictive memory management state
pub struct PredictiveMemoryState {
    // Current allocation strategy
    pub current_strategy: AllocationStrategy,
    pub strategy_since_us: u64,

    // Compaction decision history (ring buffer of last 100)
    pub compaction_decisions: [CompactionDecision; 100],
    pub compaction_head: usize,
    pub compaction_count: usize,

    // Strategy change history (ring buffer of last 50)
    pub strategy_changes: [StrategyChange; 50],
    pub strategy_head: usize,
    pub strategy_count: usize,

    // Allocation predictors (for up to 16 command types)
    pub predictors: Vec<AllocationPredictor>,

    // Statistics
    pub total_compactions_triggered: u32,
    pub compactions_prevented_oom: u32,
    pub total_pre_reservations: u32,
    pub pre_reservation_hits: u32,      // How many times pre-reserve was used
}

impl PredictiveMemoryState {
    pub fn new() -> Self {
        Self {
            current_strategy: AllocationStrategy::Balanced,
            strategy_since_us: 0,
            compaction_decisions: [CompactionDecision {
                timestamp_us: 0,
                predicted_frag_future: 0,
                confidence: 0,
                current_frag: 0,
                decision: false,
                outcome_measured: false,
                prevented_oom: false,
            }; 100],
            compaction_head: 0,
            compaction_count: 0,
            strategy_changes: [StrategyChange {
                timestamp_us: 0,
                old_strategy: AllocationStrategy::Balanced,
                new_strategy: AllocationStrategy::Balanced,
                reason_directive: 0,
                outcome_reward: 0,
                outcome_measured: false,
            }; 50],
            strategy_head: 0,
            strategy_count: 0,
            predictors: Vec::new(),
            total_compactions_triggered: 0,
            compactions_prevented_oom: 0,
            total_pre_reservations: 0,
            pre_reservation_hits: 0,
        }
    }

    /// Record a compaction decision
    pub fn record_compaction_decision(&mut self, decision: CompactionDecision) {
        self.compaction_decisions[self.compaction_head] = decision;
        self.compaction_head = (self.compaction_head + 1) % 100;
        if self.compaction_count < 100 {
            self.compaction_count += 1;
        }
        if decision.decision {
            self.total_compactions_triggered += 1;
        }
    }

    /// Record a strategy change
    pub fn record_strategy_change(&mut self, change: StrategyChange) {
        self.strategy_changes[self.strategy_head] = change;
        self.strategy_head = (self.strategy_head + 1) % 50;
        if self.strategy_count < 50 {
            self.strategy_count += 1;
        }
    }

    /// Find or create predictor for command
    pub fn get_predictor_mut(&mut self, command_hash: u32) -> Option<&mut AllocationPredictor> {
        // Find existing predictor
        let found_idx = self.predictors.iter().position(|p| p.command_hash == command_hash);

        if let Some(idx) = found_idx {
            return self.predictors.get_mut(idx);
        }

        // Create new predictor if space available
        if self.predictors.len() < 16 {
            self.predictors.push(AllocationPredictor::new(command_hash));
            let idx = self.predictors.len() - 1;
            return self.predictors.get_mut(idx);
        }

        // No space, reuse first (oldest) predictor
        if let Some(predictor) = self.predictors.get_mut(0) {
            *predictor = AllocationPredictor::new(command_hash);
            return Some(predictor);
        }

        None
    }

    /// Get accuracy statistics for allocation prediction
    pub fn get_prediction_accuracy(&self) -> (usize, usize) {
        let mut total = 0;
        let mut accurate = 0;

        for predictor in &self.predictors {
            if predictor.count >= 2 {
                total += predictor.count - 1; // Can compare n-1 predictions
                // Count how many predictions were within 20% of actual
                for i in 1..predictor.count {
                    let idx = (predictor.head + 10 - predictor.count + i) % 10;
                    let prev_idx = (predictor.head + 10 - predictor.count + i - 1) % 10;
                    let predicted = predictor.history[prev_idx].allocation_size;
                    let actual = predictor.history[idx].allocation_size;
                    if predicted > 0 && actual > 0 {
                        let error = if predicted > actual {
                            ((predicted - actual) * 100) / predicted
                        } else {
                            ((actual - predicted) * 100) / actual
                        };
                        if error <= 20 {
                            // Within 20%
                            accurate += 1;
                        }
                    }
                }
            }
        }

        (accurate, total)
    }
}

/// Global predictive memory state
static PREDICTIVE_MEMORY: Mutex<PredictiveMemoryState> =
    Mutex::new(PredictiveMemoryState {
        current_strategy: AllocationStrategy::Balanced,
        strategy_since_us: 0,
        compaction_decisions: [CompactionDecision {
            timestamp_us: 0,
            predicted_frag_future: 0,
            confidence: 0,
            current_frag: 0,
            decision: false,
            outcome_measured: false,
            prevented_oom: false,
        }; 100],
        compaction_head: 0,
        compaction_count: 0,
        strategy_changes: [StrategyChange {
            timestamp_us: 0,
            old_strategy: AllocationStrategy::Balanced,
            new_strategy: AllocationStrategy::Balanced,
            reason_directive: 0,
            outcome_reward: 0,
            outcome_measured: false,
        }; 50],
        strategy_head: 0,
        strategy_count: 0,
        predictors: Vec::new(),
        total_compactions_triggered: 0,
        compactions_prevented_oom: 0,
        total_pre_reservations: 0,
        pre_reservation_hits: 0,
    });

/// Predict fragmentation 5 seconds in the future using neural network
/// Returns: (predicted_fragmentation_pct, confidence)
pub fn predict_fragmentation_future() -> (u8, u16) {
    // Get current memory telemetry
    crate::neural::update_memory_telemetry();

    // Access telemetry through neural module's getter
    // For now, we'll use heap stats directly
    let stats = crate::heap::get_heap_stats();
    let heap_size: usize = 100 * 1024;
    let used = stats.current_allocated();
    let free = heap_size.saturating_sub(used);
    let _free_percent = (free * 100) / heap_size;

    // Simple fragmentation estimate based on allocation churn
    let utilization = if stats.peak_allocated() > 0 {
        (stats.current_allocated() * 100) / stats.peak_allocated()
    } else {
        100
    };
    let churn = stats.total_deallocations().saturating_sub(stats.total_allocations() / 2);
    let current_frag = if churn > 10 {
        100u32.saturating_sub(utilization as u32).min(100)
    } else {
        0
    } as u8;

    // Estimate allocation rate (simplified)
    let alloc_rate = stats.total_allocations().min(1000) as u16;

    // High allocation rate increases future fragmentation
    // Formula: future_frag = current + (rate / 100)
    let growth = (alloc_rate / 100).min(50) as u8;
    let predicted_frag = current_frag.saturating_add(growth).min(100);

    // Confidence based on telemetry staleness and allocation rate stability
    // Higher confidence if allocation rate is consistent
    let confidence = if alloc_rate > 0 && alloc_rate < 500 {
        800 // High confidence for moderate allocation rates
    } else if alloc_rate >= 500 {
        600 // Medium confidence for high rates (more uncertainty)
    } else {
        400 // Lower confidence for very low rates
    };

    (predicted_frag, confidence)
}

/// Evaluate compaction policy and decide whether to compact
/// Policy: trigger if prediction > 70% confidence AND predicted frag > 60%
/// Returns: (should_compact, predicted_frag, confidence)
pub fn evaluate_compaction_policy() -> (bool, u8, u16) {
    let (predicted_frag, confidence) = predict_fragmentation_future();

    // Policy thresholds
    const CONFIDENCE_THRESHOLD: u16 = 700; // 70%
    const FRAGMENTATION_THRESHOLD: u8 = 60; // 60%

    let mut should_compact =
        confidence >= CONFIDENCE_THRESHOLD && predicted_frag >= FRAGMENTATION_THRESHOLD;

    // UX Enhancement: Query mode - never execute, only predict
    if MEMORY_QUERY_MODE.load(Ordering::Acquire) {
        if should_compact {
            unsafe { crate::uart_print(b"[QUERY] Would trigger compaction (dry-run mode)\n"); }
        }
        should_compact = false; // Never execute in query mode
    }

    (should_compact, predicted_frag, confidence)
}

/// Execute predictive compaction check and record decision
/// Returns: true if compaction was triggered
pub fn execute_predictive_compaction() -> bool {
    execute_predictive_compaction_verbose(true)
}

/// Execute predictive compaction with optional verbosity control
pub fn execute_predictive_compaction_verbose(verbose: bool) -> bool {
    let timestamp = get_timestamp_us();
    let (should_compact, predicted_frag, confidence) = evaluate_compaction_policy();

    // Get current fragmentation from heap stats
    let stats = crate::heap::get_heap_stats();
    let utilization = if stats.peak_allocated() > 0 {
        (stats.current_allocated() * 100) / stats.peak_allocated()
    } else {
        100
    };
    let churn = stats.total_deallocations().saturating_sub(stats.total_allocations() / 2);
    let current_frag = if churn > 10 {
        100u32.saturating_sub(utilization as u32).min(100)
    } else {
        0
    } as u8;

    // Record decision
    let decision = CompactionDecision {
        timestamp_us: timestamp,
        predicted_frag_future: predicted_frag,
        confidence,
        current_frag,
        decision: should_compact,
        outcome_measured: false,
        prevented_oom: false,
    };

    let mut state = PREDICTIVE_MEMORY.lock();
    state.record_compaction_decision(decision);

    if should_compact && verbose {
        unsafe {
            crate::uart_print(b"[PRED_MEM] Compaction recommended (decision pending autonomy)\n");
            crate::uart_print(b"  Predicted frag in 5s: ");
            crate::shell::print_number_simple(predicted_frag as u64);
            crate::uart_print(b"%\n  Confidence: ");
            crate::shell::print_number_simple(confidence as u64);
            crate::uart_print(b"/1000\n");
        }
    }

    // TODO: Actual compaction would be triggered here
    // For now, this is a demonstration of the decision logic
    // In a real implementation, this would call a heap compaction function

    should_compact
}

/// Select allocation strategy based on meta-agent directive
/// Directive ranges: <-500=Conservative, -500..500=Balanced, >500=Aggressive
pub fn select_allocation_strategy(memory_directive: i16) -> AllocationStrategy {
    if memory_directive < -500 {
        AllocationStrategy::Conservative
    } else if memory_directive > 500 {
        AllocationStrategy::Aggressive
    } else {
        AllocationStrategy::Balanced
    }
}

/// Update allocation strategy based on meta-agent directive
/// Returns: true if strategy changed
pub fn update_allocation_strategy(memory_directive: i16) -> bool {
    let timestamp = get_timestamp_us();
    let new_strategy = select_allocation_strategy(memory_directive);

    let mut state = PREDICTIVE_MEMORY.lock();
    let old_strategy = state.current_strategy;

    if new_strategy != old_strategy {
        // Record strategy change
        let change = StrategyChange {
            timestamp_us: timestamp,
            old_strategy,
            new_strategy,
            reason_directive: memory_directive,
            outcome_reward: 0,
            outcome_measured: false,
        };
        state.record_strategy_change(change);
        state.current_strategy = new_strategy;
        state.strategy_since_us = timestamp;

        unsafe {
            crate::uart_print(b"[PRED_MEM] Strategy change: ");
            crate::uart_print(old_strategy.as_str().as_bytes());
            crate::uart_print(b" -> ");
            crate::uart_print(new_strategy.as_str().as_bytes());
            crate::uart_print(b"\n  Directive: ");
            crate::shell::print_number_simple(memory_directive as i64 as u64);
            crate::uart_print(b"\n");
        }

        true
    } else {
        false
    }
}

/// Record allocation for command type (for learning)
pub fn record_allocation(command: &str, size: usize) {
    let timestamp = get_timestamp_us();
    let command_hash = simple_hash(command);

    let mut state = PREDICTIVE_MEMORY.lock();
    if let Some(predictor) = state.get_predictor_mut(command_hash) {
        predictor.record(size, timestamp);
    }
}

/// Predict allocation size for command
/// Returns: (predicted_size, confidence, should_pre_reserve)
pub fn predict_allocation_size(command: &str) -> (usize, u16, bool) {
    let command_hash = simple_hash(command);

    let mut state = PREDICTIVE_MEMORY.lock();
    if let Some(predictor) = state.get_predictor_mut(command_hash) {
        let (predicted_size, confidence) = predictor.predict();

        // Pre-reserve if confidence > 70%
        const PRE_RESERVE_CONFIDENCE: u16 = 700;
        let should_pre_reserve = confidence >= PRE_RESERVE_CONFIDENCE && predicted_size > 0;

        if should_pre_reserve {
            state.total_pre_reservations += 1;
        }

        (predicted_size, confidence, should_pre_reserve)
    } else {
        (0, 0, false)
    }
}

/// Simple hash function for command strings
fn simple_hash(s: &str) -> u32 {
    let mut hash = 5381u32;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
}

/// Get current allocation strategy
pub fn get_current_strategy() -> AllocationStrategy {
    PREDICTIVE_MEMORY.lock().current_strategy
}

/// Get predictive memory statistics
pub fn get_statistics() -> (u32, u32, u32, u32, usize, usize, usize) {
    let state = PREDICTIVE_MEMORY.lock();
    let (accurate, total) = state.get_prediction_accuracy();
    (
        state.total_compactions_triggered,
        state.compactions_prevented_oom,
        state.total_pre_reservations,
        state.pre_reservation_hits,
        state.compaction_count,
        accurate,
        total,
    )
}

/// Print predictive memory statistics
pub fn print_statistics() {
    let state = PREDICTIVE_MEMORY.lock();

    unsafe {
        crate::uart_print(b"\n=== Predictive Memory Statistics ===\n");
        crate::uart_print(b"Current Strategy: ");
        crate::uart_print(state.current_strategy.as_str().as_bytes());
        crate::uart_print(b"\n");

        crate::uart_print(b"\nCompaction:\n");
        crate::uart_print(b"  Total predictions: ");
        crate::shell::print_number_simple(state.compaction_count as u64);
        crate::uart_print(b"\n  Compactions triggered: ");
        crate::shell::print_number_simple(state.total_compactions_triggered as u64);
        crate::uart_print(b"\n  OOMs prevented: ");
        crate::shell::print_number_simple(state.compactions_prevented_oom as u64);
        crate::uart_print(b"\n");

        crate::uart_print(b"\nAllocation Prediction:\n");
        crate::uart_print(b"  Command types tracked: ");
        crate::shell::print_number_simple(state.predictors.len() as u64);
        crate::uart_print(b"\n  Pre-reservations: ");
        crate::shell::print_number_simple(state.total_pre_reservations as u64);
        crate::uart_print(b"\n  Pre-reserve hits: ");
        crate::shell::print_number_simple(state.pre_reservation_hits as u64);
        crate::uart_print(b"\n");

        let (accurate, total) = state.get_prediction_accuracy();
        if total > 0 {
            crate::uart_print(b"  Prediction accuracy: ");
            crate::shell::print_number_simple(accurate as u64);
            crate::uart_print(b"/");
            crate::shell::print_number_simple(total as u64);
            let accuracy_pct = (accurate * 100) / total;
            crate::uart_print(b" (");
            crate::shell::print_number_simple(accuracy_pct as u64);
            crate::uart_print(b"%)\n");
        }

        crate::uart_print(b"\nStrategy Changes:\n");
        crate::uart_print(b"  Total changes: ");
        crate::shell::print_number_simple(state.strategy_count as u64);
        crate::uart_print(b"\n");
    }
}
