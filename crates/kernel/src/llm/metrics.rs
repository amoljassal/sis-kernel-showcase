//! LLM Performance Metrics and Monitoring
//!
//! # Overview
//!
//! Comprehensive metrics collection for LLM subsystem performance monitoring,
//! debugging, and optimization. Tracks:
//! - Throughput (tokens/second)
//! - Latency (time per token, first token)
//! - Memory usage (arena utilization, peak usage)
//! - Cache performance (hit rate, speedup)
//! - Error rates (timeouts, rejections)
//!
//! # Design Philosophy
//!
//! **Low Overhead**: Metrics collection has minimal performance impact (<1%)
//! **Always On**: Metrics always collected, no runtime overhead to enable/disable
//! **Actionable**: Metrics directly inform optimization decisions
//!
//! # Metrics Categories
//!
//! 1. **Performance Metrics**: Throughput, latency, speedup
//! 2. **Resource Metrics**: Memory, CPU, cache usage
//! 3. **Quality Metrics**: Accuracy, perplexity (future)
//! 4. **Reliability Metrics**: Error rates, timeouts, rejections
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::metrics::{Metrics, record_inference};
//!
//! // Start timing
//! let start = now_cycles();
//!
//! // Run inference
//! let result = infer(prompt, max_tokens);
//!
//! // Record metrics
//! let elapsed = now_cycles() - start;
//! record_inference(elapsed, tokens_generated);
//!
//! // View metrics
//! let metrics = Metrics::global();
//! metrics.print();
//! ```

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Performance metrics
pub struct PerformanceMetrics {
    /// Total inferences completed
    pub total_inferences: AtomicU64,

    /// Total tokens generated
    pub total_tokens: AtomicU64,

    /// Total inference time (microseconds)
    pub total_inference_time_us: AtomicU64,

    /// Total time to first token (microseconds)
    pub total_first_token_time_us: AtomicU64,

    /// Minimum tokens/second observed
    pub min_tokens_per_sec: AtomicU64,

    /// Maximum tokens/second observed
    pub max_tokens_per_sec: AtomicU64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_inferences: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            total_inference_time_us: AtomicU64::new(0),
            total_first_token_time_us: AtomicU64::new(0),
            min_tokens_per_sec: AtomicU64::new(u64::MAX),
            max_tokens_per_sec: AtomicU64::new(0),
        }
    }
}

impl PerformanceMetrics {
    /// Record completed inference
    pub fn record_inference(&self, elapsed_us: u64, tokens: usize) {
        self.total_inferences.fetch_add(1, Ordering::Relaxed);
        self.total_tokens.fetch_add(tokens as u64, Ordering::Relaxed);
        self.total_inference_time_us.fetch_add(elapsed_us, Ordering::Relaxed);

        // Calculate tokens/second
        if elapsed_us > 0 {
            let tokens_per_sec = (tokens as u64 * 1_000_000) / elapsed_us;

            // Update min/max
            let mut current_min = self.min_tokens_per_sec.load(Ordering::Relaxed);
            while tokens_per_sec < current_min {
                match self.min_tokens_per_sec.compare_exchange_weak(
                    current_min,
                    tokens_per_sec,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => current_min = x,
                }
            }

            let mut current_max = self.max_tokens_per_sec.load(Ordering::Relaxed);
            while tokens_per_sec > current_max {
                match self.max_tokens_per_sec.compare_exchange_weak(
                    current_max,
                    tokens_per_sec,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => current_max = x,
                }
            }
        }
    }

    /// Record first token latency
    pub fn record_first_token(&self, elapsed_us: u64) {
        self.total_first_token_time_us.fetch_add(elapsed_us, Ordering::Relaxed);
    }

    /// Get average tokens per second
    pub fn avg_tokens_per_sec(&self) -> f32 {
        let total_time = self.total_inference_time_us.load(Ordering::Relaxed);
        let total_tokens = self.total_tokens.load(Ordering::Relaxed);

        if total_time > 0 {
            (total_tokens as f32 * 1_000_000.0) / total_time as f32
        } else {
            0.0
        }
    }

    /// Get average latency per token (microseconds)
    pub fn avg_latency_per_token(&self) -> f32 {
        let total_time = self.total_inference_time_us.load(Ordering::Relaxed);
        let total_tokens = self.total_tokens.load(Ordering::Relaxed);

        if total_tokens > 0 {
            total_time as f32 / total_tokens as f32
        } else {
            0.0
        }
    }

    /// Get average first token latency (microseconds)
    pub fn avg_first_token_latency(&self) -> f32 {
        let total_time = self.total_first_token_time_us.load(Ordering::Relaxed);
        let total_inferences = self.total_inferences.load(Ordering::Relaxed);

        if total_inferences > 0 {
            total_time as f32 / total_inferences as f32
        } else {
            0.0
        }
    }
}

/// Resource usage metrics
pub struct ResourceMetrics {
    /// Peak arena usage (bytes)
    pub peak_arena_usage: AtomicU64,

    /// Total arena allocations
    pub total_allocations: AtomicU64,

    /// KV cache hits
    pub kv_cache_hits: AtomicU64,

    /// KV cache misses
    pub kv_cache_misses: AtomicU64,
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            peak_arena_usage: AtomicU64::new(0),
            total_allocations: AtomicU64::new(0),
            kv_cache_hits: AtomicU64::new(0),
            kv_cache_misses: AtomicU64::new(0),
        }
    }
}

impl ResourceMetrics {
    /// Record arena allocation
    pub fn record_allocation(&self, size: usize) {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);

        // Update peak if needed
        let mut current_peak = self.peak_arena_usage.load(Ordering::Relaxed);
        while (size as u64) > current_peak {
            match self.peak_arena_usage.compare_exchange_weak(
                current_peak,
                size as u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_peak = x,
            }
        }
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.kv_cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.kv_cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f32 {
        let hits = self.kv_cache_hits.load(Ordering::Relaxed);
        let misses = self.kv_cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
            hits as f32 / total as f32
        } else {
            0.0
        }
    }
}

/// Error and reliability metrics
pub struct ReliabilityMetrics {
    /// Total errors
    pub total_errors: AtomicU64,

    /// Timeouts
    pub total_timeouts: AtomicU64,

    /// Rejections (resource limits)
    pub total_rejections: AtomicU64,

    /// Retries
    pub total_retries: AtomicU64,
}

impl Default for ReliabilityMetrics {
    fn default() -> Self {
        Self {
            total_errors: AtomicU64::new(0),
            total_timeouts: AtomicU64::new(0),
            total_rejections: AtomicU64::new(0),
            total_retries: AtomicU64::new(0),
        }
    }
}

impl ReliabilityMetrics {
    /// Record error
    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record timeout
    pub fn record_timeout(&self) {
        self.total_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record rejection
    pub fn record_rejection(&self) {
        self.total_rejections.fetch_add(1, Ordering::Relaxed);
    }

    /// Record retry
    pub fn record_retry(&self) {
        self.total_retries.fetch_add(1, Ordering::Relaxed);
    }

    /// Get error rate
    pub fn error_rate(&self, total_inferences: u64) -> f32 {
        let errors = self.total_errors.load(Ordering::Relaxed);
        if total_inferences > 0 {
            errors as f32 / total_inferences as f32
        } else {
            0.0
        }
    }
}

/// Comprehensive metrics collection
pub struct Metrics {
    /// Performance metrics
    pub performance: PerformanceMetrics,

    /// Resource metrics
    pub resources: ResourceMetrics,

    /// Reliability metrics
    pub reliability: ReliabilityMetrics,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            performance: PerformanceMetrics::default(),
            resources: ResourceMetrics::default(),
            reliability: ReliabilityMetrics::default(),
        }
    }
}

impl Metrics {
    /// Create new metrics collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.performance = PerformanceMetrics::default();
        self.resources = ResourceMetrics::default();
        self.reliability = ReliabilityMetrics::default();
    }

    /// Print comprehensive metrics report
    pub fn print(&self) {
        let total_inferences = self.performance.total_inferences.load(Ordering::Relaxed);
        let total_tokens = self.performance.total_tokens.load(Ordering::Relaxed);

        crate::info!("=== LLM Metrics Report ===");

        crate::info!("\nPerformance:");
        crate::info!("  Total Inferences: {}", total_inferences);
        crate::info!("  Total Tokens: {}", total_tokens);
        crate::info!("  Avg Throughput: {:.2} tokens/sec", self.performance.avg_tokens_per_sec());
        crate::info!("  Avg Latency/Token: {:.2} us", self.performance.avg_latency_per_token());
        crate::info!("  Avg First Token: {:.2} ms", self.performance.avg_first_token_latency() / 1000.0);
        crate::info!("  Min Throughput: {} tokens/sec", self.performance.min_tokens_per_sec.load(Ordering::Relaxed));
        crate::info!("  Max Throughput: {} tokens/sec", self.performance.max_tokens_per_sec.load(Ordering::Relaxed));

        crate::info!("\nResources:");
        crate::info!("  Peak Arena Usage: {} KB", self.resources.peak_arena_usage.load(Ordering::Relaxed) / 1024);
        crate::info!("  Total Allocations: {}", self.resources.total_allocations.load(Ordering::Relaxed));
        crate::info!("  Cache Hit Rate: {:.2}%", self.resources.cache_hit_rate() * 100.0);
        crate::info!("  Cache Hits: {}", self.resources.kv_cache_hits.load(Ordering::Relaxed));
        crate::info!("  Cache Misses: {}", self.resources.kv_cache_misses.load(Ordering::Relaxed));

        crate::info!("\nReliability:");
        crate::info!("  Errors: {}", self.reliability.total_errors.load(Ordering::Relaxed));
        crate::info!("  Timeouts: {}", self.reliability.total_timeouts.load(Ordering::Relaxed));
        crate::info!("  Rejections: {}", self.reliability.total_rejections.load(Ordering::Relaxed));
        crate::info!("  Retries: {}", self.reliability.total_retries.load(Ordering::Relaxed));
        crate::info!("  Error Rate: {:.2}%", self.reliability.error_rate(total_inferences) * 100.0);

        crate::info!("=== End Metrics ===");
    }

    /// Get metrics snapshot (for JSON export)
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_inferences: self.performance.total_inferences.load(Ordering::Relaxed),
            total_tokens: self.performance.total_tokens.load(Ordering::Relaxed),
            avg_tokens_per_sec: self.performance.avg_tokens_per_sec(),
            avg_latency_per_token_us: self.performance.avg_latency_per_token(),
            avg_first_token_latency_us: self.performance.avg_first_token_latency(),
            peak_arena_usage: self.resources.peak_arena_usage.load(Ordering::Relaxed),
            cache_hit_rate: self.resources.cache_hit_rate(),
            total_errors: self.reliability.total_errors.load(Ordering::Relaxed),
            error_rate: self.reliability.error_rate(
                self.performance.total_inferences.load(Ordering::Relaxed)
            ),
        }
    }
}

/// Metrics snapshot (for serialization)
#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub total_inferences: u64,
    pub total_tokens: u64,
    pub avg_tokens_per_sec: f32,
    pub avg_latency_per_token_us: f32,
    pub avg_first_token_latency_us: f32,
    pub peak_arena_usage: u64,
    pub cache_hit_rate: f32,
    pub total_errors: u64,
    pub error_rate: f32,
}

/// Global metrics instance
static GLOBAL_METRICS: Mutex<Metrics> = Mutex::new(Metrics {
    performance: PerformanceMetrics {
        total_inferences: AtomicU64::new(0),
        total_tokens: AtomicU64::new(0),
        total_inference_time_us: AtomicU64::new(0),
        total_first_token_time_us: AtomicU64::new(0),
        min_tokens_per_sec: AtomicU64::new(u64::MAX),
        max_tokens_per_sec: AtomicU64::new(0),
    },
    resources: ResourceMetrics {
        peak_arena_usage: AtomicU64::new(0),
        total_allocations: AtomicU64::new(0),
        kv_cache_hits: AtomicU64::new(0),
        kv_cache_misses: AtomicU64::new(0),
    },
    reliability: ReliabilityMetrics {
        total_errors: AtomicU64::new(0),
        total_timeouts: AtomicU64::new(0),
        total_rejections: AtomicU64::new(0),
        total_retries: AtomicU64::new(0),
    },
});

/// Record completed inference (global)
pub fn record_inference(elapsed_us: u64, tokens: usize) {
    let metrics = GLOBAL_METRICS.lock();
    metrics.performance.record_inference(elapsed_us, tokens);
}

/// Record first token latency (global)
pub fn record_first_token(elapsed_us: u64) {
    let metrics = GLOBAL_METRICS.lock();
    metrics.performance.record_first_token(elapsed_us);
}

/// Record arena allocation (global)
pub fn record_allocation(size: usize) {
    let metrics = GLOBAL_METRICS.lock();
    metrics.resources.record_allocation(size);
}

/// Record cache hit (global)
pub fn record_cache_hit() {
    let metrics = GLOBAL_METRICS.lock();
    metrics.resources.record_cache_hit();
}

/// Record cache miss (global)
pub fn record_cache_miss() {
    let metrics = GLOBAL_METRICS.lock();
    metrics.resources.record_cache_miss();
}

/// Record error (global)
pub fn record_error() {
    let metrics = GLOBAL_METRICS.lock();
    metrics.reliability.record_error();
}

/// Print metrics report (global)
pub fn print_metrics() {
    let metrics = GLOBAL_METRICS.lock();
    metrics.print();
}

/// Get metrics snapshot (global)
pub fn get_snapshot() -> MetricsSnapshot {
    let metrics = GLOBAL_METRICS.lock();
    metrics.snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::default();

        metrics.record_inference(1_000_000, 10); // 1 second, 10 tokens
        assert_eq!(metrics.total_inferences.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_tokens.load(Ordering::Relaxed), 10);

        let tps = metrics.avg_tokens_per_sec();
        assert!((tps - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_resource_metrics() {
        let metrics = ResourceMetrics::default();

        metrics.record_allocation(1024);
        assert_eq!(metrics.total_allocations.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.peak_arena_usage.load(Ordering::Relaxed), 1024);

        metrics.record_allocation(512);
        assert_eq!(metrics.peak_arena_usage.load(Ordering::Relaxed), 1024); // Still 1024
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = ResourceMetrics::default();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let hit_rate = metrics.cache_hit_rate();
        assert!((hit_rate - 0.666).abs() < 0.01); // 2/3 = 66.6%
    }

    #[test]
    fn test_reliability_metrics() {
        let metrics = ReliabilityMetrics::default();

        metrics.record_error();
        metrics.record_timeout();
        metrics.record_rejection();

        assert_eq!(metrics.total_errors.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_timeouts.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_rejections.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_metrics_snapshot() {
        let metrics = Metrics::new();
        metrics.performance.record_inference(100_000, 10);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_inferences, 1);
        assert_eq!(snapshot.total_tokens, 10);
    }
}
