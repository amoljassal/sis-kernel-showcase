/// AI Decision Logging - Phase G.4
///
/// Tracks and logs AI decisions made by the kernel for visualization

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

/// Type of AI decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionType {
    MemoryPrediction,
    SchedulingDecision,
    LoadBalancing,
    CacheOptimization,
    PrefetchDecision,
}

impl DecisionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionType::MemoryPrediction => "Memory Prediction",
            DecisionType::SchedulingDecision => "Scheduling",
            DecisionType::LoadBalancing => "Load Balancing",
            DecisionType::CacheOptimization => "Cache Optimize",
            DecisionType::PrefetchDecision => "Prefetch",
        }
    }
}

/// AI decision entry
#[derive(Debug, Clone)]
pub struct DecisionEntry {
    pub decision_type: DecisionType,
    pub description: String,
    pub confidence: u8,      // 0-100
    pub outcome: Option<bool>, // None if not yet evaluated, Some(true/false) for success/failure
    pub timestamp: u64,
}

/// AI Decision Log - circular buffer of recent decisions
pub struct DecisionLog {
    entries: Vec<DecisionEntry>,
    max_entries: usize,
    next_index: usize,
    total_decisions: u64,
}

impl DecisionLog {
    /// Create a new decision log
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
            next_index: 0,
            total_decisions: 0,
        }
    }

    /// Log a new decision
    pub fn log_decision(&mut self, decision_type: DecisionType, description: String, confidence: u8) {
        let entry = DecisionEntry {
            decision_type,
            description,
            confidence,
            outcome: None,
            timestamp: self.total_decisions,
        };

        if self.entries.len() < self.max_entries {
            self.entries.push(entry);
        } else {
            self.entries[self.next_index] = entry;
        }

        self.next_index = (self.next_index + 1) % self.max_entries;
        self.total_decisions += 1;
    }

    /// Update outcome of a recent decision
    pub fn update_outcome(&mut self, timestamp: u64, outcome: bool) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.timestamp == timestamp) {
            entry.outcome = Some(outcome);
        }
    }

    /// Get all entries (most recent first)
    pub fn get_entries(&self) -> Vec<DecisionEntry> {
        if self.entries.len() < self.max_entries {
            // Not yet full, return in order
            let mut result = self.entries.clone();
            result.reverse();
            result
        } else {
            // Full, return starting from next_index (oldest) wrapped around
            let mut result = Vec::with_capacity(self.max_entries);
            for i in 0..self.max_entries {
                let idx = (self.next_index + i) % self.max_entries;
                result.push(self.entries[idx].clone());
            }
            result.reverse();
            result
        }
    }

    /// Get decision count by type
    pub fn count_by_type(&self, decision_type: DecisionType) -> usize {
        self.entries.iter().filter(|e| e.decision_type == decision_type).count()
    }

    /// Get average confidence by type
    pub fn avg_confidence(&self, decision_type: DecisionType) -> u8 {
        let decisions: Vec<_> = self.entries.iter()
            .filter(|e| e.decision_type == decision_type)
            .collect();

        if decisions.is_empty() {
            return 0;
        }

        let sum: u32 = decisions.iter().map(|e| e.confidence as u32).sum();
        (sum / decisions.len() as u32) as u8
    }

    /// Get success rate (percentage of successful outcomes)
    pub fn success_rate(&self) -> u8 {
        let with_outcomes: Vec<_> = self.entries.iter()
            .filter(|e| e.outcome.is_some())
            .collect();

        if with_outcomes.is_empty() {
            return 0;
        }

        let successes = with_outcomes.iter()
            .filter(|e| e.outcome == Some(true))
            .count();

        ((successes as f32 / with_outcomes.len() as f32) * 100.0) as u8
    }

    /// Get total decision count
    pub fn total_count(&self) -> u64 {
        self.total_decisions
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_index = 0;
    }
}

/// Global AI decision log
static AI_DECISION_LOG: Mutex<Option<DecisionLog>> = Mutex::new(None);

/// Initialize the global decision log
pub fn init_decision_log(max_entries: usize) {
    *AI_DECISION_LOG.lock() = Some(DecisionLog::new(max_entries));
}

/// Log an AI decision to the global log
pub fn log_ai_decision(decision_type: DecisionType, description: String, confidence: u8) {
    if let Some(ref mut log) = *AI_DECISION_LOG.lock() {
        log.log_decision(decision_type, description, confidence);
    }
}

/// Update outcome of an AI decision
pub fn update_ai_outcome(timestamp: u64, outcome: bool) {
    if let Some(ref mut log) = *AI_DECISION_LOG.lock() {
        log.update_outcome(timestamp, outcome);
    }
}

/// Get a snapshot of recent decisions
pub fn get_recent_decisions(count: usize) -> Vec<DecisionEntry> {
    if let Some(ref log) = *AI_DECISION_LOG.lock() {
        let entries = log.get_entries();
        entries.into_iter().take(count).collect()
    } else {
        Vec::new()
    }
}

/// Get statistics about AI decisions
pub fn get_ai_stats() -> AIStats {
    if let Some(ref log) = *AI_DECISION_LOG.lock() {
        AIStats {
            total_decisions: log.total_count(),
            success_rate: log.success_rate(),
            memory_confidence: log.avg_confidence(DecisionType::MemoryPrediction),
            scheduling_confidence: log.avg_confidence(DecisionType::SchedulingDecision),
            load_balance_confidence: log.avg_confidence(DecisionType::LoadBalancing),
        }
    } else {
        AIStats::default()
    }
}

/// AI statistics
#[derive(Debug, Clone, Copy)]
pub struct AIStats {
    pub total_decisions: u64,
    pub success_rate: u8,
    pub memory_confidence: u8,
    pub scheduling_confidence: u8,
    pub load_balance_confidence: u8,
}

impl Default for AIStats {
    fn default() -> Self {
        Self {
            total_decisions: 0,
            success_rate: 0,
            memory_confidence: 0,
            scheduling_confidence: 0,
            load_balance_confidence: 0,
        }
    }
}
