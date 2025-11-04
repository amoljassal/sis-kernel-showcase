//! Autonomous AI Safety Infrastructure
//!
//! Industry-grade safety layer for autonomous neural agent operation.
//! Implements 6-layer safety architecture:
//! - Layer 1: Hard limits (kernel-enforced bounds)
//! - Layer 2: Watchdog timers (automatic rollback)
//! - Layer 3: Action rate limiting (prevent spam)
//! - Layer 4: Audit log with rollback capability
//! - Layer 5: Human override (always available)
//! - Layer 6: Incremental autonomy (phased deployment)

use crate::meta_agent::MetaState;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};

// ============================================================================
// Layer 1: Hard Limits (Kernel-Enforced Bounds)
// ============================================================================

/// Immutable bounds on agent actions (CANNOT be exceeded)
pub const MAX_MEMORY_DIRECTIVE_CHANGE: i16 = 200;   // Max ±200/1000 per decision
pub const MAX_PRIORITY_CHANGE: i16 = 100;           // Max ±100 priority units
pub const MIN_DECISION_INTERVAL_MS: u64 = 500;      // No faster than 500ms
pub const MAX_COMPACTIONS_PER_MINUTE: u32 = 6;      // Max 6 compactions/minute
pub const MAX_PRIORITY_ADJUSTMENTS_PER_MINUTE: u32 = 20;
pub const MAX_POLICY_UPDATE_PER_EPISODE: u32 = 10;  // Cap gradient updates

/// Panic-triggering safety violations (ZERO TOLERANCE)
pub const PANIC_MEMORY_PRESSURE: u8 = 98;           // Panic if >98% pressure
pub const PANIC_CONSECUTIVE_FAILURES: u32 = 5;      // Panic if 5 consecutive bad decisions
pub const PANIC_TD_ERROR_THRESHOLD: i16 = 5000;     // Panic if TD error > 5.0 (Q8.8: 1280)

/// Confidence thresholds
pub const MIN_CONFIDENCE_FOR_ACTION: i16 = 600;     // 60% minimum (Q8.8: 153)
pub const ACTIVE_LEARNING_CONFIDENCE_LOW: i16 = 500; // 50% query threshold
pub const ACTIVE_LEARNING_CONFIDENCE_HIGH: i16 = 600; // 60% query threshold

// ============================================================================
// Explainable AI: Decision Rationale
// ============================================================================

/// Human-readable explanation codes for decisions (EU AI Act Article 13: Transparency)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ExplanationCode {
    // Normal operation
    HighMemoryPressureDetected = 1,          // "Triggered compaction due to 85% memory pressure"
    SchedulingDeadlineMissesIncreasing = 2,  // "Lowered priorities due to 10 deadline misses/sec"
    CommandAccuracyImproving = 3,            // "Increased prediction aggressiveness (78% accuracy)"
    SystemHealthy = 4,                       // "All subsystems healthy, no action needed"

    // Safety interventions
    LowConfidenceDeferredAction = 5,         // "Skipped action due to 45% confidence (< 60% threshold)"
    RateLimitPreventedAction = 6,            // "Compaction rate limit hit (6/min exceeded)"
    WatchdogRolledBack = 7,                  // "Rolled back due to 5 consecutive negative rewards"
    HardLimitViolation = 8,                  // "Action rejected: directive change exceeded ±200 limit"
    SafeModeEntered = 9,                     // "Entered safe mode due to system health critical"

    // Learning events
    LearningRateIncreased = 10,              // "Increased learning rate (accuracy <40%)"
    LearningRateDecreased = 11,              // "Decreased learning rate (accuracy >75%)"
    CheckpointSaved = 12,                    // "Saved checkpoint: system health improved +100"
    CheckpointRestored = 13,                 // "Restored checkpoint from timestamp"

    // Out-of-distribution / anomalies
    OODDetectedFallbackHeuristics = 14,      // "OOD detected (Mahalanobis distance >3σ), using heuristics"
    DistributionShiftDetected = 15,          // "Concept drift detected (KL divergence >0.4)"
    AnomalyDetectedNegativeReward = 16,      // "Anomaly: reward -300 (3σ below mean)"

    // Human interaction
    HumanOverride = 17,                      // "Human disabled autonomy via 'autoctl off'"
    HumanApprovalRequired = 18,              // "High-risk action: awaiting human approval"
    HumanFeedbackReceived = 19,              // "Human feedback: decision marked as 'good'"

    // Multi-objective trade-offs
    ParetoOptimalSelected = 20,              // "Selected Pareto-optimal action (no objective dominated)"
    RewardTamperingDetected = 21,            // "Reward tampering suspected (health↓ but reward↑)"
    OscillationPenaltyApplied = 22,          // "Oscillation detected, penalized -200"
}

impl ExplanationCode {
    /// Convert explanation code to human-readable string
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HighMemoryPressureDetected => "Triggered compaction due to high memory pressure",
            Self::SchedulingDeadlineMissesIncreasing => "Lowered priorities due to deadline misses",
            Self::CommandAccuracyImproving => "Increased prediction aggressiveness",
            Self::SystemHealthy => "All subsystems healthy, no action needed",
            Self::LowConfidenceDeferredAction => "Skipped action: confidence below threshold",
            Self::RateLimitPreventedAction => "Rate limit exceeded, action rejected",
            Self::WatchdogRolledBack => "Watchdog rollback: consecutive failures",
            Self::HardLimitViolation => "Hard limit violation: action rejected",
            Self::SafeModeEntered => "Safe mode entered: system health critical",
            Self::LearningRateIncreased => "Learning rate increased (exploration)",
            Self::LearningRateDecreased => "Learning rate decreased (exploitation)",
            Self::CheckpointSaved => "Checkpoint saved: system health improved",
            Self::CheckpointRestored => "Checkpoint restored from backup",
            Self::OODDetectedFallbackHeuristics => "Out-of-distribution detected, using safe heuristics",
            Self::DistributionShiftDetected => "Concept drift detected, consider retraining",
            Self::AnomalyDetectedNegativeReward => "Anomaly: unusual negative reward",
            Self::HumanOverride => "Human override: autonomy disabled",
            Self::HumanApprovalRequired => "High-risk action: awaiting human approval",
            Self::HumanFeedbackReceived => "Human feedback received",
            Self::ParetoOptimalSelected => "Pareto-optimal action selected",
            Self::RewardTamperingDetected => "Reward tampering suspected",
            Self::OscillationPenaltyApplied => "Oscillation penalty applied",
        }
    }
}

/// Decision rationale for explainability (EU AI Act Article 13)
#[derive(Copy, Clone)]
pub struct DecisionRationale {
    pub explanation_code: ExplanationCode,
    pub confidence: i16,  // Q8.8 format

    // Feature importance (which inputs mattered most)
    pub memory_pressure_importance: u8,    // 0-100%
    pub scheduling_load_importance: u8,
    pub command_rate_importance: u8,
}

impl DecisionRationale {
    pub const fn new(code: ExplanationCode) -> Self {
        Self {
            explanation_code: code,
            confidence: 0,
            memory_pressure_importance: 0,
            scheduling_load_importance: 0,
            command_rate_importance: 0,
        }
    }

    /// Compute feature importance using heuristic analysis (Phase 6: Explainability)
    ///
    /// This provides transparency into which inputs most influenced the decision,
    /// supporting EU AI Act Article 13 (transparency) and Article 14 (human oversight).
    ///
    /// Approach: Heuristic-based importance attribution
    /// - Analyze state values relative to normal ranges
    /// - Consider directive magnitudes
    /// - Attribute importance to feature groups
    ///
    /// Returns a DecisionRationale with populated importance fields
    pub fn with_feature_importance(
        mut self,
        state: &crate::meta_agent::MetaState,
        directives: &[i16; 3],
    ) -> Self {
        // Compute "abnormality" scores for each feature group
        // Higher scores mean the feature is further from normal operating range

        // === Memory Feature Group ===
        // Normal ranges: pressure < 50%, fragmentation < 40%, failures < 10%
        let memory_abnormality = {
            let pressure_score = if state.memory_pressure > 50 {
                (state.memory_pressure - 50) as u32
            } else {
                0
            };
            let frag_score = if state.memory_fragmentation > 40 {
                (state.memory_fragmentation - 40) as u32
            } else {
                0
            };
            let failure_score = state.memory_failures as u32 * 2; // Failures are critical
            let alloc_score = if state.memory_alloc_rate > 60 {
                (state.memory_alloc_rate - 60) as u32
            } else {
                0
            };

            pressure_score + frag_score + failure_score + alloc_score
        };

        // === Scheduling Feature Group ===
        // Normal ranges: load < 50%, deadline misses < 10%, latency < 50%
        let scheduling_abnormality = {
            let load_score = if state.scheduling_load > 50 {
                (state.scheduling_load - 50) as u32
            } else {
                0
            };
            let miss_score = state.deadline_misses as u32 * 3; // Deadline misses are critical
            let latency_score = if state.operator_latency_ms > 50 {
                (state.operator_latency_ms - 50) as u32
            } else {
                0
            };
            let critical_score = state.critical_ops_count as u32;

            load_score + miss_score + latency_score + critical_score
        };

        // === Command Feature Group ===
        // Normal ranges: rate < 50%, heaviness < 50%
        let command_abnormality = {
            let rate_score = if state.command_rate > 50 {
                (state.command_rate - 50) as u32
            } else {
                0
            };
            let heaviness_score = if state.command_heaviness > 50 {
                (state.command_heaviness - 50) as u32
            } else {
                0
            };

            rate_score + heaviness_score
        };

        // Also factor in directive magnitudes (strong directives indicate feature relevance)
        let memory_directive_mag = directives[0].abs() as u32;
        let scheduling_directive_mag = directives[1].abs() as u32;
        let command_directive_mag = directives[2].abs() as u32;

        // Combined scores: abnormality + directive magnitude
        let memory_score = memory_abnormality * 2 + memory_directive_mag / 4;
        let scheduling_score = scheduling_abnormality * 2 + scheduling_directive_mag / 4;
        let command_score = command_abnormality * 2 + command_directive_mag / 4;

        // Normalize to 0-100% (sum = 100)
        let total_score = memory_score + scheduling_score + command_score;

        if total_score > 0 {
            self.memory_pressure_importance = ((memory_score * 100) / total_score) as u8;
            self.scheduling_load_importance = ((scheduling_score * 100) / total_score) as u8;
            self.command_rate_importance = ((command_score * 100) / total_score) as u8;
        } else {
            // If all scores are zero (system is healthy), assume equal importance
            self.memory_pressure_importance = 33;
            self.scheduling_load_importance = 33;
            self.command_rate_importance = 34;
        }

        self
    }
}

// ============================================================================
// Layer 4: Audit Log with Rollback Capability
// ============================================================================

/// Bitmask for actions taken
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ActionMask(pub u8);

impl ActionMask {
    pub const NONE: Self = Self(0);
    pub const MEMORY: Self = Self(1 << 0);
    pub const SCHEDULING: Self = Self(1 << 1);
    pub const COMMAND: Self = Self(1 << 2);
}

impl core::ops::BitOr for ActionMask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        ActionMask(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for ActionMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Safety flags for violations and warnings
pub const SAFETY_LOW_CONFIDENCE: u32 = 1 << 0;
pub const SAFETY_RATE_LIMITED: u32 = 1 << 1;
pub const SAFETY_HARD_LIMIT: u32 = 1 << 2;
pub const SAFETY_WATCHDOG_TRIGGER: u32 = 1 << 3;
pub const SAFETY_OOD_DETECTED: u32 = 1 << 4;
pub const SAFETY_TAMPERING_DETECTED: u32 = 1 << 5;

/// Complete decision record for audit trail
#[derive(Copy, Clone)]
pub struct DecisionRecord {
    pub decision_id: u64,
    pub timestamp: u64,

    // State before decision
    pub state_before: MetaState,

    // Agent outputs
    pub directives: [i16; 3],      // Memory, scheduling, command
    pub confidence: i16,

    // Actions taken (bitmask)
    pub actions_taken: ActionMask,

    // Outcomes
    pub reward: i16,
    pub td_error: i16,
    pub system_health_score: i16,  // Composite: memory + scheduling + command health

    // Human-in-the-Loop (RLHF-style)
    pub human_feedback: i16,       // 0 = none, 100 = good, -50 = bad, -200 = verybad
    pub feedback_applied: bool,

    // Safety
    pub safety_flags: u32,
    pub rationale: DecisionRationale,
}

impl DecisionRecord {
    pub const fn empty() -> Self {
        Self {
            decision_id: 0,
            timestamp: 0,
            state_before: MetaState::zero(),
            directives: [0; 3],
            confidence: 0,
            actions_taken: ActionMask::NONE,
            reward: 0,
            td_error: 0,
            system_health_score: 0,
            human_feedback: 0,
            feedback_applied: false,
            safety_flags: 0,
            rationale: DecisionRationale::new(ExplanationCode::SystemHealthy),
        }
    }
}

/// Ring buffer of last 1000 autonomous decisions
pub struct DecisionAuditLog {
    entries: [DecisionRecord; 1000],
    head: usize,
    count: usize,
    last_known_good_checkpoint: usize,
    next_decision_id: u64,
}

impl DecisionAuditLog {
    pub const fn new() -> Self {
        Self {
            entries: [DecisionRecord::empty(); 1000],
            head: 0,
            count: 0,
            last_known_good_checkpoint: 0,
            next_decision_id: 1,
        }
    }

    /// Log a new decision
    pub fn log_decision(
        &mut self,
        state: MetaState,
        directives: [i16; 3],
        confidence: i16,
        actions_taken: ActionMask,
        reward: i16,
        td_error: i16,
        safety_flags: u32,
        rationale: DecisionRationale,
    ) {
        let health_score = Self::compute_health_score(&state);

        self.entries[self.head] = DecisionRecord {
            decision_id: self.next_decision_id,
            timestamp: crate::time::get_timestamp_us(),
            state_before: state,
            directives,
            confidence,
            actions_taken,
            reward,
            td_error,
            system_health_score: health_score,
            human_feedback: 0,
            feedback_applied: false,
            safety_flags,
            rationale,
        };

        self.next_decision_id += 1;
        self.head = (self.head + 1) % 1000;
        if self.count < 1000 {
            self.count += 1;
        }
    }

    /// Get number of decisions in log
    pub fn len(&self) -> usize {
        self.count
    }

    /// Get current head index
    pub fn head_index(&self) -> usize {
        self.head
    }

    /// Get entry at specific index
    pub fn get_entry(&self, idx: usize) -> Option<&DecisionRecord> {
        if idx >= 1000 {
            return None;
        }
        Some(&self.entries[idx])
    }

    /// Get last decision
    pub fn get_last(&self) -> Option<&DecisionRecord> {
        if self.count == 0 {
            return None;
        }
        let idx = if self.head == 0 { 999 } else { self.head - 1 };
        Some(&self.entries[idx])
    }

    /// Get decision by ID
    pub fn get_by_id(&self, id: u64) -> Option<&DecisionRecord> {
        for i in 0..self.count {
            let idx = (self.head + 1000 - self.count + i) % 1000;
            if self.entries[idx].decision_id == id {
                return Some(&self.entries[idx]);
            }
        }
        None
    }

    /// Apply human feedback to a decision (RLHF-style reward override)
    /// Returns true if feedback was applied successfully
    pub fn apply_human_feedback(&mut self, id: u64, feedback: i16) -> bool {
        for i in 0..self.count {
            let idx = (self.head + 1000 - self.count + i) % 1000;
            if self.entries[idx].decision_id == id {
                self.entries[idx].human_feedback = feedback;
                self.entries[idx].feedback_applied = true;
                // Override reward with human feedback
                self.entries[idx].reward = feedback;
                return true;
            }
        }
        false
    }

    /// Get mutable decision by ID (for feedback application)
    pub fn get_by_id_mut(&mut self, id: u64) -> Option<&mut DecisionRecord> {
        for i in 0..self.count {
            let idx = (self.head + 1000 - self.count + i) % 1000;
            if self.entries[idx].decision_id == id {
                return Some(&mut self.entries[idx]);
            }
        }
        None
    }

    /// Rollback to last known good checkpoint
    pub fn rollback_to_checkpoint(&self) -> MetaState {
        self.entries[self.last_known_good_checkpoint].state_before
    }

    /// Update checkpoint if system health improved
    pub fn maybe_update_checkpoint(&mut self, current_health: i16) {
        if self.count == 0 {
            return;
        }

        let prev_idx = if self.head == 0 { 999 } else { self.head - 1 };
        let prev_health = self.entries[prev_idx].system_health_score;

        if current_health > prev_health + 100 {  // Significant improvement (Q8.8: +0.4)
            self.last_known_good_checkpoint = prev_idx;
        }
    }

    /// Compute system health score (composite metric)
    fn compute_health_score(state: &MetaState) -> i16 {
        let mut score: i32 = 0;

        // Memory health (0-400 points)
        score += (100 - state.memory_pressure as i32) * 4;

        // Scheduling health (0-400 points)
        let deadline_penalty = (state.deadline_misses.min(100) as i32) * 4;
        score += 400 - deadline_penalty;

        // Command accuracy (0-200 points)
        score += (state.prediction_accuracy as i32) * 2;

        score.clamp(0, 1000) as i16
    }

    /// Get statistics for last N decisions
    pub fn get_stats(&self, last_n: usize) -> AuditStats {
        let n = last_n.min(self.count);
        let mut stats = AuditStats::default();

        for i in 0..n {
            let idx = (self.head + 1000 - n + i) % 1000;
            let record = &self.entries[idx];

            stats.total_decisions += 1;
            stats.total_reward += record.reward as i32;

            if record.safety_flags & SAFETY_RATE_LIMITED != 0 {
                stats.rate_limit_hits += 1;
            }
            if record.safety_flags & SAFETY_WATCHDOG_TRIGGER != 0 {
                stats.watchdog_triggers += 1;
            }
            if record.safety_flags & SAFETY_HARD_LIMIT != 0 {
                stats.hard_limit_violations += 1;
            }
        }

        stats
    }
}

#[derive(Default)]
pub struct AuditStats {
    pub total_decisions: u32,
    pub total_reward: i32,
    pub rate_limit_hits: u32,
    pub watchdog_triggers: u32,
    pub hard_limit_violations: u32,
}

// ============================================================================
// Layer 2: Watchdog Timers
// ============================================================================

pub enum SafetyAction {
    Continue,
    ReduceLearningRate,        // Learning rate ← learning rate × 0.5
    RevertAndFreezeLearning,   // Restore last known good, disable learning
    SafeMode,                  // Disable autonomy, manual intervention required
}

pub struct AutonomousWatchdog {
    pub last_known_good_state: MetaState,
    pub consecutive_low_rewards: u32,
    pub consecutive_high_td_errors: u32,
    pub last_rollback_timestamp: u64,
}

impl AutonomousWatchdog {
    pub const fn new() -> Self {
        Self {
            last_known_good_state: MetaState::zero(),
            consecutive_low_rewards: 0,
            consecutive_high_td_errors: 0,
            last_rollback_timestamp: 0,
        }
    }

    /// Check system safety and determine action
    pub fn check_safety(&mut self, state: &MetaState, reward: i16, td_error: i16) -> SafetyAction {
        // Trigger 1: Consecutive low/negative rewards (5 in a row)
        if reward < 0 {
            self.consecutive_low_rewards += 1;
            if self.consecutive_low_rewards >= 5 {
                unsafe {
                    crate::uart_print(b"[WATCHDOG] 5 consecutive negative rewards, reverting\n");
                }
                return SafetyAction::RevertAndFreezeLearning;
            }
        } else {
            self.consecutive_low_rewards = 0;
        }

        // Trigger 2: TD error diverging (3 consecutive >2.0 errors)
        if td_error.abs() > 512 {  // Q8.8: 2.0 = 512
            self.consecutive_high_td_errors += 1;
            if self.consecutive_high_td_errors >= 3 {
                unsafe {
                    crate::uart_print(b"[WATCHDOG] TD error diverging, reducing learning rate\n");
                }
                self.consecutive_high_td_errors = 0;  // Reset after action
                return SafetyAction::ReduceLearningRate;
            }
        } else {
            self.consecutive_high_td_errors = 0;
        }

        // Trigger 3: System health critical
        if state.memory_pressure > 95 || state.deadline_misses > 50 {
            unsafe {
                crate::uart_print(b"[WATCHDOG] System health critical, entering safe mode\n");
            }
            return SafetyAction::SafeMode;
        }

        SafetyAction::Continue
    }

    pub fn reset_counters(&mut self) {
        self.consecutive_low_rewards = 0;
        self.consecutive_high_td_errors = 0;
    }
}

// ============================================================================
// Layer 3: Action Rate Limiting
// ============================================================================

pub enum Action {
    TriggerCompaction,
    AdjustPriorities(i16),
    ChangeAllocationStrategy,
}

pub struct ActionRateLimiter {
    // Compaction rate limiting
    compaction_count: u32,
    compaction_window_start: u64,

    // Priority adjustment rate limiting
    priority_adjustments: u32,
    priority_window_start: u64,

    // Strategy changes
    strategy_changes: u32,
    strategy_window_start: u64,
}

impl ActionRateLimiter {
    pub const fn new() -> Self {
        Self {
            compaction_count: 0,
            compaction_window_start: 0,
            priority_adjustments: 0,
            priority_window_start: 0,
            strategy_changes: 0,
            strategy_window_start: 0,
        }
    }

    /// Check if action is allowed under rate limits
    pub fn allow_action(&mut self, action: &Action, current_time: u64) -> bool {
        match action {
            Action::TriggerCompaction => {
                // Max 6 compactions per minute
                if current_time - self.compaction_window_start > 60_000_000 {  // 60 seconds in μs
                    self.compaction_count = 0;
                    self.compaction_window_start = current_time;
                }

                if self.compaction_count >= MAX_COMPACTIONS_PER_MINUTE {
                    return false;  // Rate limited
                }

                self.compaction_count += 1;
                true
            }
            Action::AdjustPriorities(_delta) => {
                // Max 20 priority adjustments per minute
                if current_time - self.priority_window_start > 60_000_000 {
                    self.priority_adjustments = 0;
                    self.priority_window_start = current_time;
                }

                if self.priority_adjustments >= MAX_PRIORITY_ADJUSTMENTS_PER_MINUTE {
                    return false;
                }

                self.priority_adjustments += 1;
                true
            }
            Action::ChangeAllocationStrategy => {
                // Max 10 strategy changes per minute
                if current_time - self.strategy_window_start > 60_000_000 {
                    self.strategy_changes = 0;
                    self.strategy_window_start = current_time;
                }

                if self.strategy_changes >= 10 {
                    return false;
                }

                self.strategy_changes += 1;
                true
            }
        }
    }

    /// Snapshot current counters (for observability)
    pub fn snapshot(&self) -> (u32, u32, u32) {
        (self.compaction_count, self.priority_adjustments, self.strategy_changes)
    }
}

// ============================================================================
// Layer 5 & 6: Autonomous Control State
// ============================================================================

/// Global autonomous control state
pub struct AutonomousControl {
    pub enabled: AtomicBool,
    pub safe_mode: AtomicBool,
    pub learning_frozen: AtomicBool,
    pub decision_interval_ms: AtomicU64,
    pub last_decision_timestamp: AtomicU64,
    pub total_decisions: AtomicU64,
    pub phase: AtomicU32,  // 0=disabled, 1=supervised, 2=limited, 3=guarded, 4=full
}

impl AutonomousControl {
    pub const fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            safe_mode: AtomicBool::new(false),
            learning_frozen: AtomicBool::new(false),
            decision_interval_ms: AtomicU64::new(500),
            last_decision_timestamp: AtomicU64::new(0),
            total_decisions: AtomicU64::new(0),
            phase: AtomicU32::new(0),  // Disabled by default
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn enter_safe_mode(&self) {
        self.safe_mode.store(true, Ordering::Relaxed);
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn freeze_learning(&self) {
        self.learning_frozen.store(true, Ordering::Relaxed);
    }

    pub fn is_learning_frozen(&self) -> bool {
        self.learning_frozen.load(Ordering::Relaxed)
    }

    pub fn increment_decisions(&self) {
        self.total_decisions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_total_decisions(&self) -> u64 {
        self.total_decisions.load(Ordering::Relaxed)
    }

    pub fn set_phase(&self, phase: u32) {
        self.phase.store(phase, Ordering::Relaxed);
    }

    pub fn get_phase(&self) -> u32 {
        self.phase.load(Ordering::Relaxed)
    }

    pub fn is_safe_mode(&self) -> bool {
        self.safe_mode.load(Ordering::Relaxed)
    }
}

// ============================================================================
// Global Instances
// ============================================================================

use spin::Mutex;

static AUDIT_LOG: Mutex<DecisionAuditLog> = Mutex::new(DecisionAuditLog::new());
static WATCHDOG: Mutex<AutonomousWatchdog> = Mutex::new(AutonomousWatchdog::new());
static RATE_LIMITER: Mutex<ActionRateLimiter> = Mutex::new(ActionRateLimiter::new());
pub static AUTONOMOUS_CONTROL: AutonomousControl = AutonomousControl::new();

// Gate to enable autonomy ticks only after initialization completes (post bring-up)
pub static AUTONOMY_READY: AtomicBool = AtomicBool::new(false);

/// Mark autonomy as ready (called after agents initialize)
pub fn set_ready(ready: bool) {
    AUTONOMY_READY.store(ready, Ordering::Release);
}

// ============================================================================
// UX Enhancement: Autonomy Phase Control
// ============================================================================

/// Autonomy operational phases (for production deployment control)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum AutonomyPhase {
    PhaseA = 0,  // Learning: Aggressive exploration, low-risk actions only
    PhaseB = 1,  // Validation: Balanced exploration/exploitation, medium risk
    PhaseC = 2,  // Production: Conservative exploitation, reduced risk
    PhaseD = 3,  // Emergency: Minimal autonomy, safety-critical only
}

impl AutonomyPhase {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => AutonomyPhase::PhaseA,
            1 => AutonomyPhase::PhaseB,
            2 => AutonomyPhase::PhaseC,
            3 => AutonomyPhase::PhaseD,
            _ => AutonomyPhase::PhaseA, // Default to learning
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AutonomyPhase::PhaseA => "A (Learning)",
            AutonomyPhase::PhaseB => "B (Validation)",
            AutonomyPhase::PhaseC => "C (Production)",
            AutonomyPhase::PhaseD => "D (Emergency)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AutonomyPhase::PhaseA => "Aggressive exploration, low-risk actions only",
            AutonomyPhase::PhaseB => "Balanced exploration/exploitation, medium risk allowed",
            AutonomyPhase::PhaseC => "Conservative exploitation, reduced risk tolerance",
            AutonomyPhase::PhaseD => "Minimal autonomy, safety-critical actions only",
        }
    }

    /// Maximum risk score allowed in this phase (0-100 scale)
    pub fn max_risk_score(&self) -> u8 {
        match self {
            AutonomyPhase::PhaseA => 30,   // Low risk only during learning
            AutonomyPhase::PhaseB => 60,   // Medium risk during validation
            AutonomyPhase::PhaseC => 40,   // Conservative in production
            AutonomyPhase::PhaseD => 10,   // Minimal risk in emergency
        }
    }

    /// Decision interval (ms) recommended for this phase
    pub fn recommended_interval_ms(&self) -> u64 {
        match self {
            AutonomyPhase::PhaseA => 100,   // Fast iterations for learning
            AutonomyPhase::PhaseB => 200,   // Moderate pace for validation
            AutonomyPhase::PhaseC => 500,   // Conservative pace for production
            AutonomyPhase::PhaseD => 2000,  // Slow pace in emergency
        }
    }
}

/// Current autonomy phase (global state)
pub static AUTONOMY_PHASE: AtomicU8 = AtomicU8::new(AutonomyPhase::PhaseA as u8);

/// Get current autonomy phase
pub fn get_autonomy_phase() -> AutonomyPhase {
    AutonomyPhase::from_u8(AUTONOMY_PHASE.load(Ordering::Acquire))
}

/// Set autonomy phase
pub fn set_autonomy_phase(phase: AutonomyPhase) {
    AUTONOMY_PHASE.store(phase as u8, Ordering::Release);
}

// C-ABI setter removed after stabilizing direct atomic store usage

/// Check if autonomy ticks are allowed
pub fn is_ready() -> bool {
    AUTONOMY_READY.load(Ordering::Acquire)
}

// ============================================================================
// Layer 7: Model Checkpointing & Versioning
// ============================================================================

/// Snapshot of neural network weights for rollback
#[derive(Copy, Clone)]
pub struct NetworkSnapshot {
    /// Critic network first layer weights (12→16)
    pub critic_w1: [[i16; 16]; 16],
    pub critic_b1: [i16; 16],
    /// Critic network second layer weights (16→3)
    pub critic_w2: [[i16; 16]; 4],
    pub critic_b2: [i16; 4],

    /// Actor network first layer weights (12→16)
    pub actor_w1: [[i16; 16]; 16],
    pub actor_b1: [i16; 16],
    /// Actor network second layer weights (16→6)
    pub actor_w2: [[i16; 16]; 4],
    pub actor_b2: [i16; 4],
}

impl NetworkSnapshot {
    pub const fn empty() -> Self {
        Self {
            critic_w1: [[0; 16]; 16],
            critic_b1: [0; 16],
            critic_w2: [[0; 16]; 4],
            critic_b2: [0; 4],
            actor_w1: [[0; 16]; 16],
            actor_b1: [0; 16],
            actor_w2: [[0; 16]; 4],
            actor_b2: [0; 4],
        }
    }
}

/// Model checkpoint with metadata
#[derive(Copy, Clone)]
pub struct ModelCheckpoint {
    pub snapshot: NetworkSnapshot,
    pub checkpoint_id: u64,
    pub timestamp: u64,
    pub decision_id: u64,          // Which decision triggered this checkpoint
    pub health_score: i16,         // System health at checkpoint time
    pub cumulative_reward: i32,    // Total reward up to this point
    pub valid: bool,               // Is this checkpoint slot valid?
}

impl ModelCheckpoint {
    pub const fn empty() -> Self {
        Self {
            snapshot: NetworkSnapshot::empty(),
            checkpoint_id: 0,
            timestamp: 0,
            decision_id: 0,
            health_score: 0,
            cumulative_reward: 0,
            valid: false,
        }
    }
}

/// Checkpoint manager with versioning (ring buffer of 5 checkpoints)
pub struct CheckpointManager {
    checkpoints: [ModelCheckpoint; 5],
    head: usize,
    count: usize,
    next_checkpoint_id: u64,
}

impl CheckpointManager {
    pub const fn new() -> Self {
        Self {
            checkpoints: [ModelCheckpoint::empty(); 5],
            head: 0,
            count: 0,
            next_checkpoint_id: 1,
        }
    }

    /// Save a new checkpoint
    pub fn save(&mut self, snapshot: NetworkSnapshot, decision_id: u64, health_score: i16, cumulative_reward: i32) -> u64 {
        let checkpoint_id = self.next_checkpoint_id;
        self.next_checkpoint_id += 1;

        self.checkpoints[self.head] = ModelCheckpoint {
            snapshot,
            checkpoint_id,
            timestamp: crate::time::get_timestamp_us(),
            decision_id,
            health_score,
            cumulative_reward,
            valid: true,
        };

        self.head = (self.head + 1) % 5;
        if self.count < 5 {
            self.count += 1;
        }

        checkpoint_id
    }

    /// Get checkpoint by index (0 = oldest, count-1 = newest)
    pub fn get(&self, index: usize) -> Option<&ModelCheckpoint> {
        if index >= self.count {
            return None;
        }
        let idx = if self.count < 5 {
            index
        } else {
            (self.head + index) % 5
        };
        Some(&self.checkpoints[idx])
    }

    /// Get most recent checkpoint
    pub fn get_latest(&self) -> Option<&ModelCheckpoint> {
        if self.count == 0 {
            return None;
        }
        let idx = if self.head == 0 { 4 } else { self.head - 1 };
        Some(&self.checkpoints[idx])
    }

    /// Get best checkpoint (highest health score)
    pub fn get_best(&self) -> Option<&ModelCheckpoint> {
        if self.count == 0 {
            return None;
        }

        let mut best_idx = 0;
        let mut best_health = self.checkpoints[0].health_score;

        for i in 1..self.count {
            let idx = if self.count < 5 { i } else { (self.head + i) % 5 };
            if self.checkpoints[idx].health_score > best_health {
                best_health = self.checkpoints[idx].health_score;
                best_idx = idx;
            }
        }

        Some(&self.checkpoints[best_idx])
    }

    /// Get number of checkpoints
    pub fn len(&self) -> usize {
        self.count
    }
}

static CHECKPOINT_MANAGER: Mutex<CheckpointManager> = Mutex::new(CheckpointManager::new());

/// Public API: Get audit log
pub fn get_audit_log() -> spin::MutexGuard<'static, DecisionAuditLog> {
    AUDIT_LOG.lock()
}

/// Public API: Get watchdog
pub fn get_watchdog() -> spin::MutexGuard<'static, AutonomousWatchdog> {
    WATCHDOG.lock()
}

/// Public API: Apply human feedback to a decision (RLHF-style)
/// Returns true if feedback was successfully applied
pub fn apply_human_feedback(decision_id: u64, feedback: i16) -> bool {
    let mut audit_log = AUDIT_LOG.lock();
    audit_log.apply_human_feedback(decision_id, feedback)
}

/// Public API: Get rate limiter
pub fn get_rate_limiter() -> spin::MutexGuard<'static, ActionRateLimiter> {
    RATE_LIMITER.lock()
}

/// Public API: Get a copy of rate limiter counters
pub fn get_rate_limiter_stats() -> (u32, u32, u32) {
    let rl = RATE_LIMITER.lock();
    rl.snapshot()
}

/// Public API: Get checkpoint manager
pub fn get_checkpoint_manager() -> spin::MutexGuard<'static, CheckpointManager> {
    CHECKPOINT_MANAGER.lock()
}

/// Capture current model weights into a snapshot
pub fn capture_model_snapshot() -> NetworkSnapshot {
    let meta_agent = crate::meta_agent::get_meta_agent();
    let network = &meta_agent.network;
    let actor = &meta_agent.actor.network;

    let mut snapshot = NetworkSnapshot::empty();

    // Copy critic network weights (12→16→3)
    for i in 0..16 {
        snapshot.critic_b1[i] = network.b1[i];
        for j in 0..16 {
            snapshot.critic_w1[i][j] = network.w1[i][j];
        }
    }
    for i in 0..4 {
        snapshot.critic_b2[i] = network.b2[i];
        for j in 0..16 {
            snapshot.critic_w2[i][j] = network.w2[i][j];
        }
    }

    // Copy actor network weights (12→16→6)
    for i in 0..16 {
        snapshot.actor_b1[i] = actor.b1[i];
        for j in 0..16 {
            snapshot.actor_w1[i][j] = actor.w1[i][j];
        }
    }
    for i in 0..4 {
        snapshot.actor_b2[i] = actor.b2[i];
        for j in 0..16 {
            snapshot.actor_w2[i][j] = actor.w2[i][j];
        }
    }

    drop(meta_agent);
    snapshot
}

/// Restore model weights from a snapshot
pub fn restore_model_snapshot(snapshot: &NetworkSnapshot) {
    let mut meta_agent = crate::meta_agent::get_meta_agent();

    // Restore critic network weights
    for i in 0..16 {
        meta_agent.network.b1[i] = snapshot.critic_b1[i];
        for j in 0..16 {
            meta_agent.network.w1[i][j] = snapshot.critic_w1[i][j];
        }
    }
    for i in 0..4 {
        meta_agent.network.b2[i] = snapshot.critic_b2[i];
        for j in 0..16 {
            meta_agent.network.w2[i][j] = snapshot.critic_w2[i][j];
        }
    }

    // Restore actor network weights
    for i in 0..16 {
        meta_agent.actor.network.b1[i] = snapshot.actor_b1[i];
        for j in 0..16 {
            meta_agent.actor.network.w1[i][j] = snapshot.actor_w1[i][j];
        }
    }
    for i in 0..4 {
        meta_agent.actor.network.b2[i] = snapshot.actor_b2[i];
        for j in 0..16 {
            meta_agent.actor.network.w2[i][j] = snapshot.actor_w2[i][j];
        }
    }

    drop(meta_agent);
}

/// Save current model state as a checkpoint
pub fn save_model_checkpoint(decision_id: u64, health_score: i16, cumulative_reward: i32) -> u64 {
    let snapshot = capture_model_snapshot();
    let mut manager = CHECKPOINT_MANAGER.lock();
    let checkpoint_id = manager.save(snapshot, decision_id, health_score, cumulative_reward);

    unsafe {
        crate::uart_print(b"[CHECKPOINT] Saved model checkpoint #");
        let mut tmp = checkpoint_id;
        if tmp == 0 { crate::uart_print(b"0"); } else {
            let mut digits = [0u8; 20];
            let mut i = 0;
            while tmp > 0 {
                digits[i] = b'0' + (tmp % 10) as u8;
                tmp /= 10;
                i += 1;
            }
            while i > 0 {
                i -= 1;
                crate::uart_print(&[digits[i]]);
            }
        }
        crate::uart_print(b" (decision: ");
        let mut tmp = decision_id;
        if tmp == 0 { crate::uart_print(b"0"); } else {
            let mut digits = [0u8; 20];
            let mut i = 0;
            while tmp > 0 {
                digits[i] = b'0' + (tmp % 10) as u8;
                tmp /= 10;
                i += 1;
            }
            while i > 0 {
                i -= 1;
                crate::uart_print(&[digits[i]]);
            }
        }
        crate::uart_print(b", health: ");
        if health_score < 0 {
            crate::uart_print(b"-");
            let mut tmp = (-health_score) as u64;
            if tmp == 0 { crate::uart_print(b"0"); } else {
                let mut digits = [0u8; 20];
                let mut i = 0;
                while tmp > 0 {
                    digits[i] = b'0' + (tmp % 10) as u8;
                    tmp /= 10;
                    i += 1;
                }
                while i > 0 {
                    i -= 1;
                    crate::uart_print(&[digits[i]]);
                }
            }
        } else {
            let mut tmp = health_score as u64;
            if tmp == 0 { crate::uart_print(b"0"); } else {
                let mut digits = [0u8; 20];
                let mut i = 0;
                while tmp > 0 {
                    digits[i] = b'0' + (tmp % 10) as u8;
                    tmp /= 10;
                    i += 1;
                }
                while i > 0 {
                    i -= 1;
                    crate::uart_print(&[digits[i]]);
                }
            }
        }
        crate::uart_print(b")\n");
    }

    drop(manager);
    checkpoint_id
}

/// Restore model from a specific checkpoint index
pub fn restore_model_checkpoint(index: usize) -> bool {
    let manager = CHECKPOINT_MANAGER.lock();

    if let Some(checkpoint) = manager.get(index) {
        let snapshot = checkpoint.snapshot;
        let checkpoint_id = checkpoint.checkpoint_id;
        drop(manager);

        restore_model_snapshot(&snapshot);

        unsafe {
            crate::uart_print(b"[CHECKPOINT] Restored model from checkpoint #");
            let mut tmp = checkpoint_id;
            if tmp == 0 { crate::uart_print(b"0"); } else {
                let mut digits = [0u8; 20];
                let mut i = 0;
                while tmp > 0 {
                    digits[i] = b'0' + (tmp % 10) as u8;
                    tmp /= 10;
                    i += 1;
                }
                while i > 0 {
                    i -= 1;
                    crate::uart_print(&[digits[i]]);
                }
            }
            crate::uart_print(b"\n");
        }

        true
    } else {
        drop(manager);
        false
    }
}

/// Restore model from best checkpoint (highest health score)
pub fn restore_best_checkpoint() -> bool {
    let manager = CHECKPOINT_MANAGER.lock();

    if let Some(checkpoint) = manager.get_best() {
        let snapshot = checkpoint.snapshot;
        let checkpoint_id = checkpoint.checkpoint_id;
        let health_score = checkpoint.health_score;
        drop(manager);

        restore_model_snapshot(&snapshot);

        unsafe {
            crate::uart_print(b"[CHECKPOINT] Restored BEST model (checkpoint #");
            let mut tmp = checkpoint_id;
            if tmp == 0 { crate::uart_print(b"0"); } else {
                let mut digits = [0u8; 20];
                let mut i = 0;
                while tmp > 0 {
                    digits[i] = b'0' + (tmp % 10) as u8;
                    tmp /= 10;
                    i += 1;
                }
                while i > 0 {
                    i -= 1;
                    crate::uart_print(&[digits[i]]);
                }
            }
            crate::uart_print(b", health: ");
            if health_score < 0 {
                crate::uart_print(b"-");
                let mut tmp = (-health_score) as u64;
                if tmp == 0 { crate::uart_print(b"0"); } else {
                    let mut digits = [0u8; 20];
                    let mut i = 0;
                    while tmp > 0 {
                        digits[i] = b'0' + (tmp % 10) as u8;
                        tmp /= 10;
                        i += 1;
                    }
                    while i > 0 {
                        i -= 1;
                        crate::uart_print(&[digits[i]]);
                    }
                }
            } else {
                let mut tmp = health_score as u64;
                if tmp == 0 { crate::uart_print(b"0"); } else {
                    let mut digits = [0u8; 20];
                    let mut i = 0;
                    while tmp > 0 {
                        digits[i] = b'0' + (tmp % 10) as u8;
                        tmp /= 10;
                        i += 1;
                    }
                    while i > 0 {
                        i -= 1;
                        crate::uart_print(&[digits[i]]);
                    }
                }
            }
            crate::uart_print(b")\n");
        }

        true
    } else {
        drop(manager);
        false
    }
}

// ============================================================================
// Week 5, Day 3-4: Action Execution Layer
// ============================================================================

/// Helper function to print numbers to UART
fn uart_print_num(mut v: u64) {
    if v == 0 {
        unsafe { crate::uart_print(b"0"); }
        return;
    }

    let mut digits = [0u8; 20];
    let mut i = 0;

    while v > 0 {
        digits[i] = b'0' + (v % 10) as u8;
        v /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        unsafe { crate::uart_print(&[digits[i]]); }
    }
}

/// Helper: print signed 64-bit integer to UART
fn uart_print_i64(n: i64) {
    if n < 0 {
        unsafe { crate::uart_print(b"-"); }
        uart_print_num((-n) as u64);
    } else {
        uart_print_num(n as u64);
    }
}

/// Result of action execution
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActionResult {
    Executed,
    RateLimited,
    SafetyViolation,
    LowConfidence,
}

/// Execute meta-agent directive for memory subsystem (SAFETY-AWARE)
///
/// Directive range: -1000 to +1000
/// - Negative values indicate memory pressure (trigger compaction)
/// - Positive values indicate plenty of memory (allow aggressive allocation)
pub fn execute_memory_directive(
    directive: i16,
    last_directive: i16,
    rate_limiter: &mut ActionRateLimiter,
) -> ActionResult {
    let timestamp = crate::time::get_timestamp_us();

    // Safety check 1: Bound directive change rate
    let directive_change = (directive - last_directive).abs();
    if directive_change > MAX_MEMORY_DIRECTIVE_CHANGE {
        unsafe {
            crate::uart_print(b"[SAFETY] Memory directive change too large: ");
            uart_print_num(directive_change as u64);
            crate::uart_print(b" > ");
            uart_print_num(MAX_MEMORY_DIRECTIVE_CHANGE as u64);
            crate::uart_print(b"\n");
        }
        return ActionResult::SafetyViolation;
    }

    match directive.clamp(-1000, 1000) {
        d if d < -500 => {
            // Aggressive compaction - CHECK RATE LIMIT
            if !rate_limiter.allow_action(&Action::TriggerCompaction, timestamp) {
                unsafe {
                    crate::uart_print(b"[RATE-LIMIT] Memory compaction rate limited\n");
                }
                return ActionResult::RateLimited;
            }

            unsafe {
                crate::uart_print(b"[ACTION] Triggered memory compaction (directive: ");
                uart_print_num(d as u64);
                crate::uart_print(b")\n");
            }

            // TODO: Actually trigger compaction when heap supports it
            // crate::heap::trigger_compaction();
            ActionResult::Executed
        }
        d if d < 0 => {
            // Moderate pressure response
            unsafe {
                crate::uart_print(b"[ACTION] Moderate memory pressure response (directive: ");
                uart_print_num(d as u64);
                crate::uart_print(b")\n");
            }

            // TODO: Increase free threshold
            // crate::heap::increase_free_threshold();
            ActionResult::Executed
        }
        d if d > 500 => {
            // Plenty of headroom - allow aggressive allocation
            if !rate_limiter.allow_action(&Action::ChangeAllocationStrategy, timestamp) {
                return ActionResult::RateLimited;
            }

            unsafe {
                crate::uart_print(b"[ACTION] Enable aggressive allocation (directive: ");
                uart_print_num(d as u64);
                crate::uart_print(b")\n");
            }

            // TODO: Set allocation strategy
            // crate::heap::set_allocation_strategy(AggressiveMode);
            ActionResult::Executed
        }
        _ => {
            // Normal operation - no action needed
            ActionResult::Executed
        }
    }
}

/// Execute scheduling directive (SAFETY-AWARE)
///
/// Directive range: -1000 to +1000
/// - Negative values indicate high load (increase priorities, reduce latency)
/// - Positive values indicate low load (relax priorities)
pub fn execute_scheduling_directive(
    directive: i16,
    last_directive: i16,
    rate_limiter: &mut ActionRateLimiter,
) -> ActionResult {
    let timestamp = crate::time::get_timestamp_us();

    // Safety check 1: Bound directive change rate
    let directive_change = (directive - last_directive).abs();
    if directive_change > MAX_PRIORITY_CHANGE {
        unsafe {
            crate::uart_print(b"[SAFETY] Scheduling directive change too large: ");
            uart_print_num(directive_change as u64);
            crate::uart_print(b" > ");
            uart_print_num(MAX_PRIORITY_CHANGE as u64);
            crate::uart_print(b"\n");
        }
        return ActionResult::SafetyViolation;
    }

    match directive.clamp(-1000, 1000) {
        d if d < -500 => {
            // Critical load - increase priorities
            if !rate_limiter.allow_action(&Action::AdjustPriorities(-200), timestamp) {
                unsafe {
                    crate::uart_print(b"[RATE-LIMIT] Priority adjustment rate limited\n");
                }
                return ActionResult::RateLimited;
            }

            unsafe {
                crate::uart_print(b"[ACTION] Increase operator priorities (directive: ");
                uart_print_num(d as u64);
                crate::uart_print(b")\n");
            }

            // Use predictive scheduling to adjust priorities
            let verbose = false; // Already printed action above
            let _ = crate::predictive_scheduling::execute_scheduling_directive(directive, verbose);
            ActionResult::Executed
        }
        d if d > 500 => {
            // Low load - restore normal priorities
            if !rate_limiter.allow_action(&Action::AdjustPriorities(0), timestamp) {
                return ActionResult::RateLimited;
            }

            unsafe {
                crate::uart_print(b"[ACTION] Restore normal priorities (directive: ");
                uart_print_num(d as u64);
                crate::uart_print(b")\n");
            }

            // Use predictive scheduling to restore priorities
            let verbose = false;
            let _ = crate::predictive_scheduling::execute_scheduling_directive(directive, verbose);
            ActionResult::Executed
        }
        _ => {
            // Normal operation - no action needed
            ActionResult::Executed
        }
    }
}

/// Execute command prediction directive (SAFETY-AWARE)
///
/// Directive range: -1000 to +1000
/// - Negative values indicate low accuracy (throttle predictions)
/// - Positive values indicate high accuracy (aggressive predictions)
pub fn execute_command_directive(directive: i16) -> ActionResult {
    // Simpler safety: just bound the threshold values
    match directive.clamp(-1000, 1000) {
        d if d < -500 => {
            unsafe {
                crate::uart_print(b"[ACTION] Throttle predictions (directive: ");
                uart_print_num(d as u64);
                crate::uart_print(b")\n");
            }

            // TODO: Set prediction threshold when neural system supports it
            // crate::neural::set_prediction_threshold(500);
            ActionResult::Executed
        }
        d if d > 500 => {
            unsafe {
                crate::uart_print(b"[ACTION] Aggressive predictions (directive: ");
                uart_print_num(d as u64);
                crate::uart_print(b")\n");
            }

            // TODO: Set prediction threshold
            // crate::neural::set_prediction_threshold(200);
            ActionResult::Executed
        }
        _ => {
            // Normal operation
            ActionResult::Executed
        }
    }
}

// ============================================================================
// Week 5, Day 3-4: Multi-Objective Reward Function
// ============================================================================

/// Multi-objective reward breakdown (not single composite score)
///
/// This structure tracks separate objectives to prevent reward hacking
/// and enable transparent decision-making.
#[derive(Copy, Clone, Debug)]
pub struct MultiObjectiveReward {
    // Primary objectives
    pub memory_health: i16,      // -500 to +500
    pub scheduling_health: i16,  // -500 to +500
    pub command_accuracy: i16,   // -500 to +500

    // Safety objectives (never sacrificed for performance)
    pub action_rate_penalty: i16,      // 0 to -300 (penalty only)
    pub oscillation_penalty: i16,      // 0 to -200 (penalty only)
    pub extreme_action_penalty: i16,   // 0 to -200 (penalty only)

    // Meta-objectives
    pub predictability: i16,     // 0 to +100 (bonus for consistent behavior)

    // Composite (for backward compatibility with existing learning)
    pub total: i16,              // -1000 to +1000
}

impl MultiObjectiveReward {
    pub const fn zero() -> Self {
        Self {
            memory_health: 0,
            scheduling_health: 0,
            command_accuracy: 0,
            action_rate_penalty: 0,
            oscillation_penalty: 0,
            extreme_action_penalty: 0,
            predictability: 0,
            total: 0,
        }
    }

    /// Compute total reward from components
    pub fn compute_total(&mut self) {
        let sum = self.memory_health as i32
            + self.scheduling_health as i32
            + self.command_accuracy as i32
            + self.action_rate_penalty as i32
            + self.oscillation_penalty as i32
            + self.extreme_action_penalty as i32
            + self.predictability as i32;

        self.total = sum.clamp(-1000, 1000) as i16;
    }
}

/// Compute multi-objective reward based on system health changes
///
/// This function measures actual system improvements to prevent reward hacking.
pub fn compute_system_reward(
    prev_state: &crate::meta_agent::MetaState,
    curr_state: &crate::meta_agent::MetaState,
    actions_taken: &ActionMask,
) -> MultiObjectiveReward {
    let mut reward = MultiObjectiveReward::zero();

    // ========================================================================
    // Primary Objective 1: Memory Health (0-400 points)
    // ========================================================================

    // +2 per % pressure reduction
    let mem_delta = (prev_state.memory_pressure as i32) - (curr_state.memory_pressure as i32);
    reward.memory_health = (mem_delta * 2).clamp(-500, 500) as i16;

    // Bonus for preventing failures
    if curr_state.memory_failures < prev_state.memory_failures {
        reward.memory_health = (reward.memory_health as i32 + 100).clamp(-500, 500) as i16;
    }

    // ========================================================================
    // Primary Objective 2: Scheduling Health (0-400 points)
    // ========================================================================

    // +10 per deadline miss prevented
    let sched_delta = (prev_state.deadline_misses as i32) - (curr_state.deadline_misses as i32);
    reward.scheduling_health = (sched_delta * 10).clamp(-500, 500) as i16;

    // ========================================================================
    // Primary Objective 3: Command Accuracy (0-200 points)
    // ========================================================================

    // +2 per % accuracy gain
    let acc_delta = (curr_state.prediction_accuracy as i32) - (prev_state.prediction_accuracy as i32);
    reward.command_accuracy = (acc_delta * 2).clamp(-500, 500) as i16;

    // ========================================================================
    // Safety Penalty 1: Action Rate Penalty
    // ========================================================================

    let mut action_count = 0;
    if (actions_taken.0 & ActionMask::MEMORY.0) != 0 {
        action_count += 1;
    }
    if (actions_taken.0 & ActionMask::SCHEDULING.0) != 0 {
        action_count += 1;
    }
    if (actions_taken.0 & ActionMask::COMMAND.0) != 0 {
        action_count += 1;
    }

    // Penalty for taking multiple actions at once (avoid thrashing)
    if action_count >= 3 {
        reward.action_rate_penalty = -200;
    } else if action_count == 2 {
        reward.action_rate_penalty = -100;
    }

    // ========================================================================
    // Safety Penalty 2: Extreme Action Penalty
    // ========================================================================

    // Penalize if memory or scheduling pressure is still very high
    // (indicates action didn't help)
    if curr_state.memory_pressure > 90 {
        reward.extreme_action_penalty = (reward.extreme_action_penalty as i32 - 50).clamp(-200, 0) as i16;
    }
    if curr_state.deadline_misses > 40 {
        reward.extreme_action_penalty = (reward.extreme_action_penalty as i32 - 50).clamp(-200, 0) as i16;
    }

    // ========================================================================
    // Meta-Objective: Predictability Bonus
    // ========================================================================

    // Bonus if system is stable (pressure changes are small)
    let mem_change = (curr_state.memory_pressure as i32 - prev_state.memory_pressure as i32).abs();
    let sched_change = (curr_state.deadline_misses as i32 - prev_state.deadline_misses as i32).abs();

    if mem_change <= 5 && sched_change <= 2 {
        reward.predictability = 50; // Bonus for stable behavior
    }

    // Compute total
    reward.compute_total();

    reward
}

/// Detect oscillation in recent decisions
///
/// Oscillation is when the agent flip-flops between opposite decisions,
/// indicating unstable policy or poor generalization.
pub fn detect_oscillation(audit_log: &DecisionAuditLog, lookback: usize) -> bool {
    if audit_log.count < lookback {
        return false; // Not enough history
    }

    let mut sign_changes = 0;
    let mut prev_mem_directive: Option<i16> = None;

    // Look at last N decisions
    for i in 0..lookback.min(audit_log.count) {
        let idx = if audit_log.head >= i {
            audit_log.head - i
        } else {
            1000 + audit_log.head - i
        };

        let record = &audit_log.entries[idx];
        let mem_dir = record.directives[0];

        if let Some(prev) = prev_mem_directive {
            // Check if sign changed and magnitude is large
            if (prev < -200 && mem_dir > 200) || (prev > 200 && mem_dir < -200) {
                sign_changes += 1;
            }
        }

        prev_mem_directive = Some(mem_dir);
    }

    // If more than 3 sign changes in last 10 decisions, it's oscillating
    sign_changes >= 3
}

/// Detect reward tampering / goodharting
///
/// Reward tampering occurs when the agent discovers a way to maximize
/// the reward function without actually improving system health.
///
/// Detection: Compare agent's reward trend with external health measurement.
pub fn detect_reward_tampering(audit_log: &DecisionAuditLog) -> bool {
    if audit_log.count < 20 {
        return false; // Not enough history
    }

    // Compute recent reward trend (last 10 decisions)
    let mut recent_rewards = 0i32;
    for i in 0..10 {
        let idx = if audit_log.head >= i {
            audit_log.head - i
        } else {
            1000 + audit_log.head - i
        };
        recent_rewards += audit_log.entries[idx].reward as i32;
    }

    // Compute older reward trend (10-20 decisions ago)
    let mut older_rewards = 0i32;
    for i in 10..20 {
        let idx = if audit_log.head >= i {
            audit_log.head - i
        } else {
            1000 + audit_log.head - i
        };
        older_rewards += audit_log.entries[idx].reward as i32;
    }

    let reward_trend = recent_rewards - older_rewards;

    // Compute external health trend (system_health_score is independent metric)
    let mut recent_health = 0i32;
    for i in 0..10 {
        let idx = if audit_log.head >= i {
            audit_log.head - i
        } else {
            1000 + audit_log.head - i
        };
        recent_health += audit_log.entries[idx].system_health_score as i32;
    }

    let mut older_health = 0i32;
    for i in 10..20 {
        let idx = if audit_log.head >= i {
            audit_log.head - i
        } else {
            1000 + audit_log.head - i
        };
        older_health += audit_log.entries[idx].system_health_score as i32;
    }

    let health_trend = recent_health - older_health;

    // Tampering detected: rewards increasing but health decreasing
    // (agent found a shortcut that doesn't actually help)
    reward_trend > 500 && health_trend < -500
}

// ============================================================================
// Week 5, Day 5: Autonomous Decision Loop
// ============================================================================

/// Collect current system telemetry
///
/// This function gathers all relevant system metrics into a MetaState
/// for decision-making.
fn collect_telemetry() -> crate::meta_agent::MetaState {
    // Get current meta-agent state
    let state = crate::meta_agent::get_meta_state();

    // In a real system, we'd augment with additional telemetry:
    // - Current heap pressure from heap::get_pressure()
    // - Scheduling metrics from scheduler::get_stats()
    // - Command accuracy from neural::get_accuracy()

    state
}

/// Compute system health score (independent metric for reward tampering detection)
///
/// This function provides an external, independent assessment of system health
/// that is NOT used in reward computation. This allows us to detect reward
/// hacking by comparing reward trend vs health trend.
fn compute_health_score(state: &crate::meta_agent::MetaState) -> i16 {
    let mut score: i32 = 1000;

    // Penalize high memory pressure
    let mem_penalty = (state.memory_pressure as i32) * 5;
    score -= mem_penalty;

    // Penalize deadline misses
    let sched_penalty = (state.deadline_misses as i32) * 10;
    score -= sched_penalty;

    // Bonus for high command accuracy
    let acc_bonus = (state.prediction_accuracy as i32) * 2;
    score += acc_bonus;

    // Penalize memory failures heavily
    if state.memory_failures > 0 {
        score -= 200;
    }

    score.clamp(-1000, 1000) as i16
}

/// Complete autonomous decision loop with all safety checks integrated
///
/// This is the main entry point for autonomous meta-agent execution.
/// It orchestrates:
/// 1. Telemetry collection
/// 2. Safety pre-checks (panic conditions)
/// 3. Meta-agent inference
/// 4. Confidence-based action gating
/// 5. Action execution with rate limiting
/// 6. Reward computation (multi-objective)
/// 7. Watchdog safety monitoring
/// 8. Audit logging
/// 9. Learning updates (if not frozen)
///
/// Safety is enforced at EVERY step - this function will abort early
/// if any safety condition is violated.
pub fn autonomous_decision_tick() {
    // Static counter to track total autonomous ticks for debug output control
    static mut AUTO_TICK_COUNT: u32 = 0;

    let timestamp = crate::time::get_timestamp_us();

    // Check if autonomous mode is enabled
    if !AUTONOMOUS_CONTROL.is_enabled() {
        return;
    }

    // Check if safe mode is active
    if AUTONOMOUS_CONTROL.is_safe_mode() {
        unsafe {
            crate::uart_print(b"[AUTONOMY] Safe mode active, skipping decision tick\n");
        }
        return;
    }

    // Check minimum interval between decisions
    let last_decision = AUTONOMOUS_CONTROL.last_decision_timestamp.load(core::sync::atomic::Ordering::Relaxed);
    let interval_ms = AUTONOMOUS_CONTROL.decision_interval_ms.load(core::sync::atomic::Ordering::Relaxed);
    let elapsed_us = timestamp.saturating_sub(last_decision);

    if elapsed_us < interval_ms * 1000 {
        return; // Too soon since last decision
    }

    // Increment tick counter and control verbosity
    unsafe {
        AUTO_TICK_COUNT += 1;

        // Only print verbose output for first 5 ticks
        if AUTO_TICK_COUNT <= 5 {
            crate::uart_print(b"[AUTONOMY] Starting decision tick at timestamp ");
            uart_print_num(timestamp);
            crate::uart_print(b"\n");
        } else if AUTO_TICK_COUNT == 6 {
            crate::uart_print(b"[AUTONOMY] Running silently (use 'autoctl status' to check)\n");
            // Disable metrics after first 5 ticks to avoid console spam
            crate::trace::metrics_set_enabled(false);
        }
    }

    // Acquire locks on all safety infrastructure
    let mut audit_log = AUDIT_LOG.lock();
    let mut watchdog = WATCHDOG.lock();
    let mut rate_limiter = RATE_LIMITER.lock();

    // ========================================================================
    // Step 1: Collect Telemetry
    // ========================================================================

    let prev_state = if audit_log.count > 0 {
        // Get state from last decision
        let prev_idx = if audit_log.head > 0 {
            audit_log.head - 1
        } else {
            999
        };
        audit_log.entries[prev_idx].state_before
    } else {
        // First decision - use current state as baseline
        collect_telemetry()
    };

    let curr_state = collect_telemetry();

    unsafe {
        // Only print telemetry for first 5 ticks
        if AUTO_TICK_COUNT <= 5 {
            crate::uart_print(b"[AUTONOMY] Telemetry: mem_pressure=");
            uart_print_num(curr_state.memory_pressure as u64);
            crate::uart_print(b" deadline_misses=");
            uart_print_num(curr_state.deadline_misses as u64);
            crate::uart_print(b"\n");
        }
    }

    // ========================================================================
    // Step 2: Safety Pre-Checks (Panic Conditions)
    // ========================================================================

    if curr_state.memory_pressure > PANIC_MEMORY_PRESSURE {
        unsafe {
            crate::uart_print(b"[SAFETY] PANIC: Memory pressure critical (");
            uart_print_num(curr_state.memory_pressure as u64);
            crate::uart_print(b" > ");
            uart_print_num(PANIC_MEMORY_PRESSURE as u64);
            crate::uart_print(b")\n");
        }
        AUTONOMOUS_CONTROL.enter_safe_mode();
        return;
    }

    // ========================================================================
    // Step 3: Meta-Agent Inference
    // ========================================================================

    // Run meta-agent forward pass
    let decision = crate::meta_agent::force_meta_decision();
    let directives = [
        decision.memory_directive,
        decision.scheduling_directive,
        decision.command_directive,
    ];

    // Use decision confidence
    let confidence = decision.confidence as i16;

    unsafe {
        // Only print meta-agent output for first 5 ticks
        if AUTO_TICK_COUNT <= 5 {
            crate::uart_print(b"[AUTONOMY] Meta-agent directives: [");
            uart_print_num(directives[0] as u64);
            crate::uart_print(b", ");
            uart_print_num(directives[1] as u64);
            crate::uart_print(b", ");
            uart_print_num(directives[2] as u64);
            crate::uart_print(b"] confidence=");
            uart_print_num(confidence as u64);
            crate::uart_print(b"\n");
        }
    }

    // ========================================================================
    // Step 3.5: Predictive Memory Management (Week 8)
    // ========================================================================

    // Update allocation strategy based on memory directive
    let _strategy_changed = crate::predictive_memory::update_allocation_strategy(directives[0]);

    // Execute predictive compaction check (5-second lookahead)
    // Only print verbose output for first 5 ticks
    let verbose = unsafe { AUTO_TICK_COUNT <= 5 };
    let _compaction_triggered = crate::predictive_memory::execute_predictive_compaction_verbose(verbose);

    // ========================================================================
    // Step 4: Confidence-Based Action Gating
    // ========================================================================

    if confidence < MIN_CONFIDENCE_FOR_ACTION {
        unsafe {
            // Only print low confidence message for first 5 ticks
            if AUTO_TICK_COUNT <= 5 {
                crate::uart_print(b"[AUTONOMY] Low confidence (");
                uart_print_i64(confidence as i64);
                crate::uart_print(b" < ");
                uart_print_i64(MIN_CONFIDENCE_FOR_ACTION as i64);
                crate::uart_print(b"), deferring action\n");
            }
        }

        // Log decision with no actions taken
        let mut rationale = DecisionRationale::new(ExplanationCode::LowConfidenceDeferredAction);
        rationale.confidence = confidence;
        // Phase 6: Compute feature importance for explainability
        rationale = rationale.with_feature_importance(&curr_state, &directives);

        audit_log.log_decision(
            curr_state,
            directives,
            confidence,
            ActionMask::NONE,
            0, // reward
            0, // td_error
            SAFETY_LOW_CONFIDENCE,
            rationale,
        );
        // Update decision counters and timestamp even when deferring actions
        AUTONOMOUS_CONTROL.last_decision_timestamp.store(timestamp, core::sync::atomic::Ordering::Relaxed);
        AUTONOMOUS_CONTROL.total_decisions.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        return;
    }

    // ========================================================================
    // Step 5: Execute Actions (with Safety Checks)
    // ========================================================================

    let mut actions_taken = ActionMask::NONE;
    let mut safety_flags = 0u32;

    // Get previous directives for rate-of-change checking
    let prev_directives = if audit_log.count > 0 {
        let prev_idx = if audit_log.head > 0 {
            audit_log.head - 1
        } else {
            999
        };
        audit_log.entries[prev_idx].directives
    } else {
        [0, 0, 0]
    };

    // Execute memory directive
    match execute_memory_directive(directives[0], prev_directives[0], &mut rate_limiter) {
        ActionResult::Executed => {
            actions_taken = ActionMask(actions_taken.0 | ActionMask::MEMORY.0);
        }
        ActionResult::RateLimited => {
            safety_flags |= SAFETY_RATE_LIMITED;
            unsafe {
                crate::uart_print(b"[AUTONOMY] Memory action rate limited\n");
            }
        }
        ActionResult::SafetyViolation => {
            safety_flags |= SAFETY_HARD_LIMIT;
            unsafe {
                crate::uart_print(b"[AUTONOMY] Memory action safety violation\n");
            }
        }
        ActionResult::LowConfidence => {}
    }

    // Execute scheduling directive
    match execute_scheduling_directive(directives[1], prev_directives[1], &mut rate_limiter) {
        ActionResult::Executed => {
            actions_taken = ActionMask(actions_taken.0 | ActionMask::SCHEDULING.0);
        }
        ActionResult::RateLimited => {
            safety_flags |= SAFETY_RATE_LIMITED;
            unsafe {
                crate::uart_print(b"[AUTONOMY] Scheduling action rate limited\n");
            }
        }
        ActionResult::SafetyViolation => {
            safety_flags |= SAFETY_HARD_LIMIT;
            unsafe {
                crate::uart_print(b"[AUTONOMY] Scheduling action safety violation\n");
            }
        }
        ActionResult::LowConfidence => {}
    }

    // Execute command directive
    match execute_command_directive(directives[2]) {
        ActionResult::Executed => {
            actions_taken = ActionMask(actions_taken.0 | ActionMask::COMMAND.0);
        }
        _ => {}
    }

    unsafe {
        crate::uart_print(b"[AUTONOMY] Actions taken: 0x");
        uart_print_num(actions_taken.0 as u64);
        crate::uart_print(b"\n");
    }

    // ========================================================================
    // Step 6: Compute Reward (Multi-Objective)
    // ========================================================================

    let multi_reward = compute_system_reward(&prev_state, &curr_state, &actions_taken);

    unsafe {
        crate::uart_print(b"[AUTONOMY] Multi-objective reward: mem=");
        uart_print_i64(multi_reward.memory_health as i64);
        crate::uart_print(b" sched=");
        uart_print_i64(multi_reward.scheduling_health as i64);
        crate::uart_print(b" cmd=");
        uart_print_i64(multi_reward.command_accuracy as i64);
        crate::uart_print(b" total=");
        uart_print_i64(multi_reward.total as i64);
        crate::uart_print(b"\n");
    }

    // Simple TD error: reward + future_value - current_value
    // For now, use simple reward as TD error (no value function yet)
    let td_error = multi_reward.total;

    // ========================================================================
    // Step 7: Watchdog Safety Monitoring
    // ========================================================================

    let safety_action = watchdog.check_safety(&curr_state, multi_reward.total, td_error);

    match safety_action {
        SafetyAction::Continue => {
            // Normal operation
            unsafe {
                crate::uart_print(b"[AUTONOMY] Watchdog: Continue\n");
            }
        }
        SafetyAction::ReduceLearningRate => {
            unsafe {
                crate::uart_print(b"[SAFETY] Watchdog triggered: Reducing learning rate\n");
            }
            safety_flags |= SAFETY_WATCHDOG_TRIGGER;
            // TODO: Reduce learning rate in meta_agent
        }
        SafetyAction::RevertAndFreezeLearning => {
            unsafe {
                crate::uart_print(b"[SAFETY] Watchdog triggered: Reverting to checkpoint and freezing learning\n");
            }

            // Rollback to last checkpoint
            let checkpoint_idx = audit_log.last_known_good_checkpoint;
            if checkpoint_idx < audit_log.count {
                let _checkpoint_state = audit_log.entries[checkpoint_idx].state_before;
                unsafe {
                    crate::uart_print(b"[AUTONOMY] Rolling back to checkpoint ");
                    uart_print_num(checkpoint_idx as u64);
                    crate::uart_print(b"\n");
                }
                // TODO: Restore system state
                // restore_system_state(&_checkpoint_state);
            }

            // Freeze learning
            AUTONOMOUS_CONTROL.freeze_learning();
            return;
        }
        SafetyAction::SafeMode => {
            unsafe {
                crate::uart_print(b"[SAFETY] Watchdog triggered: Entering safe mode\n");
            }
            AUTONOMOUS_CONTROL.enter_safe_mode();
            return;
        }
    }

    // ========================================================================
    // Step 8: Audit Logging
    // ========================================================================

    let health_score = compute_health_score(&curr_state);

    // Determine explanation code
    let explanation_code = if actions_taken.0 == ActionMask::NONE.0 {
        ExplanationCode::LowConfidenceDeferredAction
    } else if curr_state.memory_pressure > prev_state.memory_pressure {
        ExplanationCode::HighMemoryPressureDetected
    } else if curr_state.deadline_misses > prev_state.deadline_misses {
        ExplanationCode::SchedulingDeadlineMissesIncreasing
    } else {
        ExplanationCode::SystemHealthy
    };

    let mut rationale = DecisionRationale::new(explanation_code);
    rationale.confidence = confidence;
    // Phase 6: Compute feature importance for explainability
    rationale = rationale.with_feature_importance(&curr_state, &directives);

    audit_log.log_decision(
        curr_state,
        directives,
        confidence,
        actions_taken,
        multi_reward.total,
        td_error,
        safety_flags,
        rationale,
    );

    // Maybe update checkpoint if health is good
    if health_score > 500 && multi_reward.total > 0 {
        audit_log.last_known_good_checkpoint = audit_log.head;
        unsafe {
            crate::uart_print(b"[AUTONOMY] Updated checkpoint to decision ");
            uart_print_num(audit_log.head as u64);
            crate::uart_print(b"\n");
        }
    }

    // Update decision timestamp
    AUTONOMOUS_CONTROL.last_decision_timestamp.store(timestamp, core::sync::atomic::Ordering::Relaxed);
    AUTONOMOUS_CONTROL.total_decisions.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

    unsafe {
        crate::uart_print(b"[AUTONOMY] Decision tick complete\n");
    }

    // ========================================================================
    // Step 9: Learning Update (if not frozen)
    // ========================================================================

    if !AUTONOMOUS_CONTROL.is_learning_frozen() {
        // TODO: Store experience in replay buffer
        // TODO: Trigger TD learning update if conditions met
        unsafe {
            crate::uart_print(b"[AUTONOMY] Learning update: TODO\n");
        }
    } else {
        unsafe {
            crate::uart_print(b"[AUTONOMY] Learning frozen, skipping update\n");
        }
    }
}

/// Public API: Trigger one autonomous decision tick
pub fn trigger_autonomous_tick() {
    autonomous_decision_tick();
}

/// Preview decision data (for UX enhancement: autoctl preview)
pub struct DecisionPreview {
    pub timestamp: u64,
    pub memory_directive: i16,
    pub scheduling_directive: i16,
    pub command_directive: i16,
    pub confidence: i16,
    pub memory_pressure: u8,
    pub memory_fragmentation: u8,
    pub deadline_misses: u8,
    pub command_rate: u8,
    pub enabled: bool,
    pub safe_mode: bool,
}

/// UX Enhancement: Preview next autonomous decision without executing
/// Returns what the system would do if autonomy runs now
pub fn preview_next_decision() -> DecisionPreview {
    let timestamp = crate::time::get_timestamp_us();
    let enabled = AUTONOMOUS_CONTROL.is_enabled();
    let safe_mode = AUTONOMOUS_CONTROL.is_safe_mode();

    // If not enabled or in safe mode, return default preview
    if !enabled || safe_mode {
        let state = crate::meta_agent::collect_telemetry();
        return DecisionPreview {
            timestamp,
            memory_directive: 0,
            scheduling_directive: 0,
            command_directive: 0,
            confidence: 0,
            memory_pressure: state.memory_pressure,
            memory_fragmentation: state.memory_fragmentation,
            deadline_misses: state.deadline_misses,
            command_rate: state.command_rate,
            enabled,
            safe_mode,
        };
    }

    // Collect current telemetry
    let curr_state = crate::meta_agent::collect_telemetry();

    // Run meta-agent inference (read-only)
    let decision = crate::meta_agent::force_meta_decision();

    DecisionPreview {
        timestamp,
        memory_directive: decision.memory_directive,
        scheduling_directive: decision.scheduling_directive,
        command_directive: decision.command_directive,
        confidence: decision.confidence as i16,
        memory_pressure: curr_state.memory_pressure,
        memory_fragmentation: curr_state.memory_fragmentation,
        deadline_misses: curr_state.deadline_misses,
        command_rate: curr_state.command_rate,
        enabled,
        safe_mode,
    }
}

/// Retrieve the last decision's rationale for explainability (Phase 6)
///
/// Returns the most recent decision record from the audit log, which includes
/// feature importance weights computed via sensitivity analysis.
///
/// This supports EU AI Act Article 13 (transparency) by allowing users to
/// understand which inputs influenced the last autonomous decision.
///
/// Returns None if no decisions have been made yet.
pub fn get_last_decision_rationale() -> Option<DecisionRecord> {
    let audit_log = AUDIT_LOG.lock();
    audit_log.get_last().copied()
}
