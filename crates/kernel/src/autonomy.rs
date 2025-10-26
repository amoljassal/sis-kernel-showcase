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
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

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
            safety_flags,
            rationale,
        };

        self.next_decision_id += 1;
        self.head = (self.head + 1) % 1000;
        if self.count < 1000 {
            self.count += 1;
        }
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
}

// ============================================================================
// Global Instances
// ============================================================================

use spin::Mutex;

static AUDIT_LOG: Mutex<DecisionAuditLog> = Mutex::new(DecisionAuditLog::new());
static WATCHDOG: Mutex<AutonomousWatchdog> = Mutex::new(AutonomousWatchdog::new());
static RATE_LIMITER: Mutex<ActionRateLimiter> = Mutex::new(ActionRateLimiter::new());
pub static AUTONOMOUS_CONTROL: AutonomousControl = AutonomousControl::new();

/// Public API: Get audit log
pub fn get_audit_log() -> spin::MutexGuard<'static, DecisionAuditLog> {
    AUDIT_LOG.lock()
}

/// Public API: Get watchdog
pub fn get_watchdog() -> spin::MutexGuard<'static, AutonomousWatchdog> {
    WATCHDOG.lock()
}

/// Public API: Get rate limiter
pub fn get_rate_limiter() -> spin::MutexGuard<'static, ActionRateLimiter> {
    RATE_LIMITER.lock()
}
