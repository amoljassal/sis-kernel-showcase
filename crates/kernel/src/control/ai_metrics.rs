//! AI Impact Dashboard - Metrics Collection and Visualization
//!
//! This module tracks the performance impact of AI components across the system:
//! - Neural network inference latencies
//! - Decision follow rates
//! - Crash prediction accuracy
//! - Context switch improvements
//! - Memory savings from AI-driven optimizations
//!
//! # Performance Targets
//!
//! - Dashboard updates in real-time (<1s refresh)
//! - All key metrics tracked with minimal overhead
//! - Metrics exportable to JSON for analysis

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Ring buffer size for latency tracking
const LATENCY_BUFFER_SIZE: usize = 1000;

/// AI metrics collector
pub struct AiMetricsCollector {
    // Inference latencies (microseconds)
    nn_infer_latencies: RingBuffer<u64>,
    transformer_latencies: RingBuffer<u64>,
    llm_latencies: RingBuffer<u64>,

    // Decision metrics
    decisions_made: AtomicU64,
    decisions_followed: AtomicU64,

    // Crash prediction metrics
    crash_predictions: AtomicU64,
    crash_predictions_correct: AtomicU64,

    // Performance improvements
    baseline_ctx_switch_avg_us: AtomicU64,
    ai_ctx_switch_avg_us: AtomicU64,
    memory_saved_bytes: AtomicU64,

    // Scheduler metrics
    scheduler_invocations: AtomicU64,
    scheduler_improvements: AtomicU64,
}

impl AiMetricsCollector {
    pub fn new() -> Self {
        Self {
            nn_infer_latencies: RingBuffer::new(LATENCY_BUFFER_SIZE),
            transformer_latencies: RingBuffer::new(LATENCY_BUFFER_SIZE),
            llm_latencies: RingBuffer::new(LATENCY_BUFFER_SIZE),
            decisions_made: AtomicU64::new(0),
            decisions_followed: AtomicU64::new(0),
            crash_predictions: AtomicU64::new(0),
            crash_predictions_correct: AtomicU64::new(0),
            baseline_ctx_switch_avg_us: AtomicU64::new(100), // Default baseline
            ai_ctx_switch_avg_us: AtomicU64::new(100),
            memory_saved_bytes: AtomicU64::new(0),
            scheduler_invocations: AtomicU64::new(0),
            scheduler_improvements: AtomicU64::new(0),
        }
    }

    /// Record NN inference latency
    pub fn record_nn_latency(&mut self, latency_us: u64) {
        self.nn_infer_latencies.push(latency_us);
    }

    /// Record transformer scheduler latency
    pub fn record_transformer_latency(&mut self, latency_us: u64) {
        self.transformer_latencies.push(latency_us);
    }

    /// Record LLM inference latency
    pub fn record_llm_latency(&mut self, latency_us: u64) {
        self.llm_latencies.push(latency_us);
    }

    /// Record a decision
    pub fn record_decision(&self, followed: bool) {
        self.decisions_made.fetch_add(1, Ordering::Relaxed);
        if followed {
            self.decisions_followed.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record crash prediction
    pub fn record_crash_prediction(&self, correct: bool) {
        self.crash_predictions.fetch_add(1, Ordering::Relaxed);
        if correct {
            self.crash_predictions_correct.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record scheduler improvement
    pub fn record_scheduler_improvement(&self) {
        self.scheduler_invocations.fetch_add(1, Ordering::Relaxed);
        self.scheduler_improvements.fetch_add(1, Ordering::Relaxed);
    }

    /// Set baseline context switch average
    pub fn set_baseline_ctx_switch(&self, avg_us: u64) {
        self.baseline_ctx_switch_avg_us.store(avg_us, Ordering::Relaxed);
    }

    /// Update AI context switch average
    pub fn update_ai_ctx_switch(&self, avg_us: u64) {
        self.ai_ctx_switch_avg_us.store(avg_us, Ordering::Relaxed);
    }

    /// Add memory savings
    pub fn add_memory_saved(&self, bytes: u64) {
        self.memory_saved_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Get snapshot of current metrics
    pub fn snapshot(&self) -> AiMetricsSnapshot {
        let (nn_p50, nn_p99) = self.nn_infer_latencies.percentiles();
        let (transformer_p50, transformer_p99) = self.transformer_latencies.percentiles();
        let (llm_p50, llm_p99) = self.llm_latencies.percentiles();

        let decisions_made = self.decisions_made.load(Ordering::Relaxed);
        let decisions_followed = self.decisions_followed.load(Ordering::Relaxed);
        let decision_follow_rate = if decisions_made > 0 {
            (decisions_followed as f32 / decisions_made as f32) * 100.0
        } else {
            0.0
        };

        let crash_predictions = self.crash_predictions.load(Ordering::Relaxed);
        let crash_predictions_correct = self.crash_predictions_correct.load(Ordering::Relaxed);
        let crash_prediction_accuracy = if crash_predictions > 0 {
            (crash_predictions_correct as f32 / crash_predictions as f32) * 100.0
        } else {
            0.0
        };

        let baseline_ctx = self.baseline_ctx_switch_avg_us.load(Ordering::Relaxed);
        let ai_ctx = self.ai_ctx_switch_avg_us.load(Ordering::Relaxed);
        let ctx_switch_improvement = if baseline_ctx > 0 && ai_ctx > 0 {
            ((baseline_ctx as f32 - ai_ctx as f32) / baseline_ctx as f32) * 100.0
        } else {
            0.0
        };

        let memory_saved_bytes = self.memory_saved_bytes.load(Ordering::Relaxed);
        let memory_saved_mb = memory_saved_bytes / (1024 * 1024);

        let scheduler_invocations = self.scheduler_invocations.load(Ordering::Relaxed);
        let scheduler_improvements = self.scheduler_improvements.load(Ordering::Relaxed);
        let scheduler_improvement_rate = if scheduler_invocations > 0 {
            (scheduler_improvements as f32 / scheduler_invocations as f32) * 100.0
        } else {
            0.0
        };

        AiMetricsSnapshot {
            nn_infer_p50_us: nn_p50,
            nn_infer_p99_us: nn_p99,
            transformer_p50_us: transformer_p50,
            transformer_p99_us: transformer_p99,
            llm_p50_us: llm_p50,
            llm_p99_us: llm_p99,
            decision_follow_rate,
            crash_prediction_accuracy,
            ctx_switch_improvement,
            memory_saved_mb,
            scheduler_improvement_rate,
        }
    }

    /// Export metrics to JSON-like string
    pub fn export_json(&self) -> String {
        let snapshot = self.snapshot();
        format!(
            "{{\n\
             \"nn_inference\": {{\n\
             \"p50_us\": {},\n\
             \"p99_us\": {}\n\
             }},\n\
             \"transformer_scheduler\": {{\n\
             \"p50_us\": {},\n\
             \"p99_us\": {},\n\
             \"improvement_rate\": {:.1}\n\
             }},\n\
             \"llm_inference\": {{\n\
             \"p50_us\": {},\n\
             \"p99_us\": {}\n\
             }},\n\
             \"decisions\": {{\n\
             \"follow_rate\": {:.1}\n\
             }},\n\
             \"crash_prediction\": {{\n\
             \"accuracy\": {:.1}\n\
             }},\n\
             \"performance\": {{\n\
             \"ctx_switch_improvement\": {:.1},\n\
             \"memory_saved_mb\": {}\n\
             }}\n\
             }}",
            snapshot.nn_infer_p50_us,
            snapshot.nn_infer_p99_us,
            snapshot.transformer_p50_us,
            snapshot.transformer_p99_us,
            snapshot.scheduler_improvement_rate,
            snapshot.llm_p50_us,
            snapshot.llm_p99_us,
            snapshot.decision_follow_rate,
            snapshot.crash_prediction_accuracy,
            snapshot.ctx_switch_improvement,
            snapshot.memory_saved_mb
        )
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.nn_infer_latencies = RingBuffer::new(LATENCY_BUFFER_SIZE);
        self.transformer_latencies = RingBuffer::new(LATENCY_BUFFER_SIZE);
        self.llm_latencies = RingBuffer::new(LATENCY_BUFFER_SIZE);
        self.decisions_made.store(0, Ordering::Relaxed);
        self.decisions_followed.store(0, Ordering::Relaxed);
        self.crash_predictions.store(0, Ordering::Relaxed);
        self.crash_predictions_correct.store(0, Ordering::Relaxed);
        self.memory_saved_bytes.store(0, Ordering::Relaxed);
        self.scheduler_invocations.store(0, Ordering::Relaxed);
        self.scheduler_improvements.store(0, Ordering::Relaxed);
    }
}

impl Default for AiMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Ring buffer for latency tracking
struct RingBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    head: usize,
    size: usize,
}

impl<T: Copy + Default + Ord> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(T::default());
        }
        Self {
            buffer,
            capacity,
            head: 0,
            size: 0,
        }
    }

    fn push(&mut self, value: T) {
        self.buffer[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        }
    }

    fn percentiles(&self) -> (T, T) {
        if self.size == 0 {
            return (T::default(), T::default());
        }

        let mut sorted: Vec<T> = self.buffer[0..self.size].to_vec();
        sorted.sort();

        let p50_idx = (self.size as f32 * 0.5) as usize;
        let p99_idx = (self.size as f32 * 0.99) as usize;

        (sorted[p50_idx], sorted[p99_idx.min(self.size - 1)])
    }
}

/// AI metrics snapshot
#[derive(Debug, Clone, Copy)]
pub struct AiMetricsSnapshot {
    pub nn_infer_p50_us: u64,
    pub nn_infer_p99_us: u64,
    pub transformer_p50_us: u64,
    pub transformer_p99_us: u64,
    pub llm_p50_us: u64,
    pub llm_p99_us: u64,
    pub decision_follow_rate: f32,
    pub crash_prediction_accuracy: f32,
    pub ctx_switch_improvement: f32,
    pub memory_saved_mb: u64,
    pub scheduler_improvement_rate: f32,
}

/// Global AI metrics collector
static AI_METRICS: Mutex<Option<AiMetricsCollector>> = Mutex::new(None);

/// Initialize AI metrics collector
pub fn init() {
    let mut metrics = AI_METRICS.lock();
    *metrics = Some(AiMetricsCollector::new());
    crate::info!("ai_metrics: initialized");
}

/// Record NN inference latency
pub fn record_nn_latency(latency_us: u64) {
    if let Some(metrics) = AI_METRICS.lock().as_mut() {
        metrics.record_nn_latency(latency_us);
    }
}

/// Record transformer scheduler latency
pub fn record_transformer_latency(latency_us: u64) {
    if let Some(metrics) = AI_METRICS.lock().as_mut() {
        metrics.record_transformer_latency(latency_us);
    }
}

/// Record LLM inference latency
pub fn record_llm_latency(latency_us: u64) {
    if let Some(metrics) = AI_METRICS.lock().as_mut() {
        metrics.record_llm_latency(latency_us);
    }
}

/// Record a decision
pub fn record_decision(followed: bool) {
    if let Some(metrics) = AI_METRICS.lock().as_ref() {
        metrics.record_decision(followed);
    }
}

/// Record crash prediction
pub fn record_crash_prediction(correct: bool) {
    if let Some(metrics) = AI_METRICS.lock().as_ref() {
        metrics.record_crash_prediction(correct);
    }
}

/// Record scheduler improvement
pub fn record_scheduler_improvement() {
    if let Some(metrics) = AI_METRICS.lock().as_ref() {
        metrics.record_scheduler_improvement();
    }
}

/// Set baseline context switch average
pub fn set_baseline_ctx_switch(avg_us: u64) {
    if let Some(metrics) = AI_METRICS.lock().as_ref() {
        metrics.set_baseline_ctx_switch(avg_us);
    }
}

/// Update AI context switch average
pub fn update_ai_ctx_switch(avg_us: u64) {
    if let Some(metrics) = AI_METRICS.lock().as_ref() {
        metrics.update_ai_ctx_switch(avg_us);
    }
}

/// Add memory savings
pub fn add_memory_saved(bytes: u64) {
    if let Some(metrics) = AI_METRICS.lock().as_ref() {
        metrics.add_memory_saved(bytes);
    }
}

/// Get metrics snapshot
pub fn get_snapshot() -> Option<AiMetricsSnapshot> {
    AI_METRICS.lock()
        .as_ref()
        .map(|m| m.snapshot())
}

/// Export metrics as JSON
pub fn export_json() -> Option<String> {
    AI_METRICS.lock()
        .as_ref()
        .map(|m| m.export_json())
}

/// Reset metrics
pub fn reset_metrics() {
    if let Some(metrics) = AI_METRICS.lock().as_mut() {
        metrics.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer() {
        let mut buffer: RingBuffer<u64> = RingBuffer::new(100);
        for i in 0..50 {
            buffer.push(i);
        }
        let (p50, p99) = buffer.percentiles();
        assert!(p50 > 0);
        assert!(p99 >= p50);
    }

    #[test]
    fn test_metrics_snapshot() {
        let collector = AiMetricsCollector::new();
        collector.record_decision(true);
        collector.record_decision(false);
        let snapshot = collector.snapshot();
        assert_eq!(snapshot.decision_follow_rate, 50.0);
    }
}
