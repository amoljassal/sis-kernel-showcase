/// Predictive Crash Detection via Memory Patterns
///
/// Uses decision traces and memory metrics to predict kernel panics before they occur.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use spin::Mutex;

/// Ring buffer size for allocation history
const HISTORY_SIZE: usize = 100;

/// Crash predictor state
pub struct CrashPredictor {
    /// Rolling window of memory allocation patterns
    alloc_history: RingBuffer<AllocMetrics>,
    /// Failed allocation counter
    oom_signals: AtomicU32,
    /// Fragmentation trend detector
    frag_trend: LinearRegression,
    /// Prediction threshold (tunable)
    danger_threshold: f32,
    /// Peak free pages seen
    peak_free_pages: AtomicU64,
    /// Prediction history for accuracy tracking
    predictions: Mutex<Vec<PredictionRecord>>,
}

#[derive(Debug, Clone, Copy)]
pub struct AllocMetrics {
    pub timestamp_ms: u64,
    pub free_pages: usize,
    pub largest_free_block: usize,
    pub fragmentation_ratio: f32,
    pub allocation_failures: u32,
}

impl Default for AllocMetrics {
    fn default() -> Self {
        Self {
            timestamp_ms: 0,
            free_pages: 0,
            largest_free_block: 0,
            fragmentation_ratio: 0.0,
            allocation_failures: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PredictionRecord {
    pub timestamp_ms: u64,
    pub confidence: f32,
    pub free_pages: usize,
    pub fragmentation: f32,
    pub outcome: PredictionOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PredictionOutcome {
    Pending,
    TruePositive,  // Predicted crash, crash occurred
    TrueNegative,  // Predicted no crash, no crash occurred
    FalsePositive, // Predicted crash, no crash occurred
    FalseNegative, // Predicted no crash, crash occurred
}

/// Simple ring buffer implementation
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    head: usize,
    size: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
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

    pub fn push(&mut self, item: T) {
        self.buffer[self.head] = item;
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let start = if self.size < self.capacity {
            0
        } else {
            self.head
        };

        (0..self.size).map(move |i| {
            &self.buffer[(start + i) % self.capacity]
        })
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn latest(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let idx = if self.head == 0 {
                self.capacity - 1
            } else {
                self.head - 1
            };
            Some(&self.buffer[idx])
        }
    }
}

/// Simple linear regression for trend detection
pub struct LinearRegression {
    /// Stored x values (timestamps)
    x_values: Vec<f32>,
    /// Stored y values (fragmentation ratios)
    y_values: Vec<f32>,
    /// Maximum number of points to keep
    max_points: usize,
}

impl LinearRegression {
    pub fn new(max_points: usize) -> Self {
        Self {
            x_values: Vec::with_capacity(max_points),
            y_values: Vec::with_capacity(max_points),
            max_points,
        }
    }

    pub fn add_point(&mut self, x: f32, y: f32) {
        if self.x_values.len() >= self.max_points {
            self.x_values.remove(0);
            self.y_values.remove(0);
        }
        self.x_values.push(x);
        self.y_values.push(y);
    }

    /// Calculate slope (trend)
    pub fn slope(&self) -> f32 {
        if self.x_values.len() < 2 {
            return 0.0;
        }

        let n = self.x_values.len() as f32;
        let sum_x: f32 = self.x_values.iter().sum();
        let sum_y: f32 = self.y_values.iter().sum();
        let sum_xy: f32 = self.x_values.iter()
            .zip(self.y_values.iter())
            .map(|(x, y)| x * y)
            .sum();
        let sum_x2: f32 = self.x_values.iter()
            .map(|x| x * x)
            .sum();

        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < 0.0001 {
            return 0.0;
        }

        (n * sum_xy - sum_x * sum_y) / denominator
    }

    pub fn clear(&mut self) {
        self.x_values.clear();
        self.y_values.clear();
    }
}

impl CrashPredictor {
    /// Create a new crash predictor
    pub fn new() -> Self {
        Self {
            alloc_history: RingBuffer::new(HISTORY_SIZE),
            oom_signals: AtomicU32::new(0),
            frag_trend: LinearRegression::new(50),
            danger_threshold: 0.8, // 80% confidence triggers warning
            peak_free_pages: AtomicU64::new(0),
            predictions: Mutex::new(Vec::new()),
        }
    }

    /// Update with current memory metrics
    pub fn update(&mut self, metrics: AllocMetrics) {
        // Track peak free pages
        let current_peak = self.peak_free_pages.load(Ordering::Relaxed);
        if metrics.free_pages as u64 > current_peak {
            self.peak_free_pages.store(metrics.free_pages as u64, Ordering::Relaxed);
        }

        // Add to history
        self.alloc_history.push(metrics);

        // Update fragmentation trend
        let time = metrics.timestamp_ms as f32 / 1000.0; // Convert to seconds
        self.frag_trend.add_point(time, metrics.fragmentation_ratio);

        // Track OOM signals
        if metrics.allocation_failures > 0 {
            self.oom_signals.fetch_add(metrics.allocation_failures, Ordering::Relaxed);
        }
    }

    /// Calculate crash prediction confidence (0.0 - 1.0)
    pub fn predict(&mut self) -> f32 {
        if self.alloc_history.len() < 10 {
            return 0.0; // Not enough data
        }

        let mut confidence = 0.0;
        let latest = match self.alloc_history.latest() {
            Some(m) => m,
            None => return 0.0,
        };

        // Factor 1: Rapid decline in free pages (>20% in 1 second)
        let decline_confidence = self.calculate_decline_confidence();
        confidence += decline_confidence * 0.35;

        // Factor 2: High fragmentation ratio (>0.7)
        if latest.fragmentation_ratio > 0.7 {
            confidence += 0.25;
        } else if latest.fragmentation_ratio > 0.5 {
            confidence += 0.15 * (latest.fragmentation_ratio - 0.5) / 0.2;
        }

        // Factor 3: Repeated allocation failures (>3 in window)
        let failure_count = self.count_recent_failures(1000); // Last 1 second
        if failure_count > 3 {
            confidence += 0.25;
        } else if failure_count > 0 {
            confidence += 0.08 * failure_count as f32;
        }

        // Factor 4: Fragmentation trend (increasing slope)
        let slope = self.frag_trend.slope();
        if slope > 0.1 {
            confidence += 0.15;
        } else if slope > 0.05 {
            confidence += 0.10;
        }

        confidence.min(1.0)
    }

    /// Calculate confidence based on free page decline rate
    fn calculate_decline_confidence(&self) -> f32 {
        if self.alloc_history.len() < 10 {
            return 0.0;
        }

        let peak = self.peak_free_pages.load(Ordering::Relaxed) as f32;
        if peak < 1.0 {
            return 0.0;
        }

        // Check decline over last second (assume ~10 samples)
        let latest = match self.alloc_history.latest() {
            Some(m) => m,
            None => return 0.0,
        };

        let current_free = latest.free_pages as f32;
        let decline_ratio = (peak - current_free) / peak;

        // >20% decline in 1 second is concerning
        if decline_ratio > 0.2 {
            1.0
        } else if decline_ratio > 0.1 {
            (decline_ratio - 0.1) / 0.1 // Linear scaling
        } else {
            0.0
        }
    }

    /// Count allocation failures in recent history
    fn count_recent_failures(&self, window_ms: u64) -> u32 {
        let latest = match self.alloc_history.latest() {
            Some(m) => m,
            None => return 0,
        };

        let cutoff_time = latest.timestamp_ms.saturating_sub(window_ms);

        self.alloc_history
            .iter()
            .filter(|m| m.timestamp_ms >= cutoff_time)
            .map(|m| m.allocation_failures)
            .sum()
    }

    /// Get current prediction status
    pub fn status(&mut self) -> PredictionStatus {
        let confidence = self.predict();
        let latest = self.alloc_history.latest().copied();

        PredictionStatus {
            confidence,
            free_pages: latest.map(|m| m.free_pages).unwrap_or(0),
            fragmentation: latest.map(|m| m.fragmentation_ratio).unwrap_or(0.0),
            recent_failures: self.count_recent_failures(1000),
            recommendation: self.get_recommendation(confidence),
        }
    }

    /// Get recommendation based on confidence
    fn get_recommendation(&self, confidence: f32) -> &'static str {
        if confidence >= 0.9 {
            "CRITICAL: Run 'memctl compact' immediately!"
        } else if confidence >= 0.8 {
            "WARNING: Run 'memctl compact' to prevent crash"
        } else if confidence >= 0.6 {
            "Monitor closely, consider running 'memctl compact'"
        } else {
            "Normal operation"
        }
    }

    /// Record prediction for accuracy tracking
    pub fn record_prediction(&mut self, confidence: f32) {
        let latest = match self.alloc_history.latest() {
            Some(m) => m,
            None => return,
        };

        let record = PredictionRecord {
            timestamp_ms: latest.timestamp_ms,
            confidence,
            free_pages: latest.free_pages,
            fragmentation: latest.fragmentation_ratio,
            outcome: PredictionOutcome::Pending,
        };

        let mut predictions = self.predictions.lock();
        predictions.push(record);

        // Keep only last 50 predictions
        if predictions.len() > 50 {
            predictions.remove(0);
        }
    }

    /// Get prediction history
    pub fn get_history(&self) -> Vec<PredictionRecord> {
        self.predictions.lock().clone()
    }

    /// Tune danger threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.danger_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get current threshold
    pub fn threshold(&self) -> f32 {
        self.danger_threshold
    }

    /// Check if automatic compaction should be triggered
    pub fn should_auto_compact(&mut self) -> bool {
        let confidence = self.predict();
        confidence >= 0.9 // 90% confidence triggers auto-compaction
    }

    /// Reset peak tracking (e.g., after compaction)
    pub fn reset_peak(&mut self) {
        if let Some(latest) = self.alloc_history.latest() {
            self.peak_free_pages.store(latest.free_pages as u64, Ordering::Relaxed);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PredictionStatus {
    pub confidence: f32,
    pub free_pages: usize,
    pub fragmentation: f32,
    pub recent_failures: u32,
    pub recommendation: &'static str,
}

impl Default for CrashPredictor {
    fn default() -> Self {
        Self::new()
    }
}

/// Global crash predictor instance
static CRASH_PREDICTOR: Mutex<Option<CrashPredictor>> = Mutex::new(None);

/// Initialize the crash predictor
pub fn init() {
    let mut predictor = CRASH_PREDICTOR.lock();
    *predictor = Some(CrashPredictor::new());
    crate::info!("crash_predictor: initialized");
}

/// Update predictor with memory metrics
pub fn update_metrics(metrics: AllocMetrics) {
    if let Some(predictor) = CRASH_PREDICTOR.lock().as_mut() {
        predictor.update(metrics);
    }
}

/// Get current prediction status
pub fn get_status() -> Option<PredictionStatus> {
    CRASH_PREDICTOR.lock().as_mut().map(|p| p.status())
}

/// Check if auto-compaction should be triggered
pub fn should_auto_compact() -> bool {
    CRASH_PREDICTOR.lock()
        .as_mut()
        .map(|p| p.should_auto_compact())
        .unwrap_or(false)
}

/// Get prediction history
pub fn get_history() -> Vec<PredictionRecord> {
    CRASH_PREDICTOR.lock()
        .as_ref()
        .map(|p| p.get_history())
        .unwrap_or_else(Vec::new)
}

/// Tune prediction threshold
pub fn set_threshold(threshold: f32) {
    if let Some(predictor) = CRASH_PREDICTOR.lock().as_mut() {
        predictor.set_threshold(threshold);
    }
}

/// Reset peak tracking
pub fn reset_peak() {
    if let Some(predictor) = CRASH_PREDICTOR.lock().as_mut() {
        predictor.reset_peak();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);
        assert_eq!(buffer.len(), 0);

        buffer.push(AllocMetrics {
            timestamp_ms: 100,
            free_pages: 1000,
            ..Default::default()
        });
        assert_eq!(buffer.len(), 1);

        buffer.push(AllocMetrics {
            timestamp_ms: 200,
            free_pages: 900,
            ..Default::default()
        });
        buffer.push(AllocMetrics {
            timestamp_ms: 300,
            free_pages: 800,
            ..Default::default()
        });
        assert_eq!(buffer.len(), 3);

        // Overflow - should replace oldest
        buffer.push(AllocMetrics {
            timestamp_ms: 400,
            free_pages: 700,
            ..Default::default()
        });
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.latest().unwrap().free_pages, 700);
    }

    #[test]
    fn test_linear_regression() {
        let mut lr = LinearRegression::new(10);

        // Perfect upward trend
        for i in 0..5 {
            lr.add_point(i as f32, i as f32);
        }

        let slope = lr.slope();
        assert!((slope - 1.0).abs() < 0.01, "Expected slope ~1.0, got {}", slope);
    }

    #[test]
    fn test_crash_prediction_low_risk() {
        let mut predictor = CrashPredictor::new();

        // Simulate stable memory conditions
        for i in 0..20 {
            predictor.update(AllocMetrics {
                timestamp_ms: i * 100,
                free_pages: 1000,
                largest_free_block: 512,
                fragmentation_ratio: 0.3,
                allocation_failures: 0,
            });
        }

        let confidence = predictor.predict();
        assert!(confidence < 0.3, "Low risk should have low confidence, got {}", confidence);
    }

    #[test]
    fn test_crash_prediction_high_risk() {
        let mut predictor = CrashPredictor::new();

        // Simulate deteriorating memory conditions
        for i in 0..20 {
            let free = 1000 - i * 40; // Rapid decline
            predictor.update(AllocMetrics {
                timestamp_ms: i * 100,
                free_pages: free,
                largest_free_block: free / 4,
                fragmentation_ratio: 0.3 + (i as f32 * 0.03),
                allocation_failures: if i > 15 { 1 } else { 0 },
            });
        }

        let confidence = predictor.predict();
        assert!(confidence > 0.5, "High risk should have high confidence, got {}", confidence);
    }
}
