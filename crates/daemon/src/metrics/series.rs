//! Metric series storage with ring buffers and statistics

use serde::Serialize;
use std::collections::VecDeque;

/// A single metric data point
#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema)]
pub struct MetricPoint {
    /// Unix timestamp in milliseconds
    pub ts: i64,
    /// Metric value
    pub value: i64,
}

/// Statistics for a metric series
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SeriesStats {
    /// Total number of points recorded (including those evicted)
    pub count: usize,
    /// Minimum value seen
    pub min: i64,
    /// Maximum value seen
    pub max: i64,
    /// Mean (average) value
    pub mean: f64,
    /// Most recent value
    pub last_value: i64,
}

impl Default for SeriesStats {
    fn default() -> Self {
        Self {
            count: 0,
            min: i64::MAX,
            max: i64::MIN,
            mean: 0.0,
            last_value: 0,
        }
    }
}

/// Ring buffer for storing metric points with fixed capacity
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a new item, evicting oldest if at capacity
    pub fn push(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    /// Get the number of items in the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get all items as a slice-like iterator
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get a reference to the buffer's underlying VecDeque
    pub fn as_vecdeque(&self) -> &VecDeque<T> {
        &self.buffer
    }
}

/// A time-series of metric data points with statistics
#[derive(Debug, Clone)]
pub struct MetricSeries {
    /// Series name (normalized)
    pub name: String,
    /// High-resolution points (5 min retention)
    pub points: RingBuffer<MetricPoint>,
    /// Downsampled points (1 hr retention, using LTTB)
    pub downsampled: RingBuffer<MetricPoint>,
    /// Timestamp of the most recent point
    pub last_ts: i64,
    /// Running statistics
    pub stats: SeriesStats,
}

impl MetricSeries {
    /// Create a new metric series
    ///
    /// # Arguments
    /// * `name` - Series name
    /// * `high_res_capacity` - Number of high-res points to retain
    /// * `downsampled_capacity` - Number of downsampled points to retain
    pub fn new(name: String, high_res_capacity: usize, downsampled_capacity: usize) -> Self {
        Self {
            name,
            points: RingBuffer::new(high_res_capacity),
            downsampled: RingBuffer::new(downsampled_capacity),
            last_ts: 0,
            stats: SeriesStats::default(),
        }
    }

    /// Add a new data point to the series
    pub fn push(&mut self, point: MetricPoint) {
        // Update timestamp
        self.last_ts = point.ts;

        // Add to high-res buffer
        self.points.push(point);

        // Update statistics
        self.update_stats(point.value);
    }

    /// Update running statistics with a new value
    fn update_stats(&mut self, value: i64) {
        self.stats.count += 1;
        self.stats.min = self.stats.min.min(value);
        self.stats.max = self.stats.max.max(value);
        self.stats.last_value = value;

        // Update mean incrementally using Welford's online algorithm
        let delta = value as f64 - self.stats.mean;
        self.stats.mean += delta / self.stats.count as f64;
    }

    /// Get the number of high-res points
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if series is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Get all high-res points
    pub fn get_points(&self) -> Vec<MetricPoint> {
        self.points.iter().copied().collect()
    }

    /// Get points within a time range
    pub fn get_points_in_range(&self, from: i64, to: i64) -> Vec<MetricPoint> {
        self.points
            .iter()
            .filter(|p| p.ts >= from && p.ts <= to)
            .copied()
            .collect()
    }

    /// Get downsampled points within a time range
    pub fn get_downsampled_in_range(&self, from: i64, to: i64) -> Vec<MetricPoint> {
        self.downsampled
            .iter()
            .filter(|p| p.ts >= from && p.ts <= to)
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_capacity() {
        let mut buffer = RingBuffer::new(3);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        assert_eq!(buffer.len(), 3);

        // Push one more - should evict oldest
        buffer.push(4);
        assert_eq!(buffer.len(), 3);

        let items: Vec<&i32> = buffer.iter().collect();
        assert_eq!(items, vec![&2, &3, &4]);
    }

    #[test]
    fn test_metric_series_push() {
        let mut series = MetricSeries::new("test".to_string(), 10, 100);

        series.push(MetricPoint { ts: 1000, value: 100 });
        series.push(MetricPoint { ts: 2000, value: 200 });
        series.push(MetricPoint { ts: 3000, value: 150 });

        assert_eq!(series.len(), 3);
        assert_eq!(series.last_ts, 3000);
        assert_eq!(series.stats.count, 3);
        assert_eq!(series.stats.min, 100);
        assert_eq!(series.stats.max, 200);
        assert_eq!(series.stats.last_value, 150);
        assert!((series.stats.mean - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_series_time_range() {
        let mut series = MetricSeries::new("test".to_string(), 10, 100);

        series.push(MetricPoint { ts: 1000, value: 1 });
        series.push(MetricPoint { ts: 2000, value: 2 });
        series.push(MetricPoint { ts: 3000, value: 3 });
        series.push(MetricPoint { ts: 4000, value: 4 });

        let range = series.get_points_in_range(2000, 3000);
        assert_eq!(range.len(), 2);
        assert_eq!(range[0].value, 2);
        assert_eq!(range[1].value, 3);
    }

    #[test]
    fn test_stats_negative_values() {
        let mut series = MetricSeries::new("temp".to_string(), 10, 100);

        series.push(MetricPoint { ts: 1000, value: -10 });
        series.push(MetricPoint { ts: 2000, value: 0 });
        series.push(MetricPoint { ts: 3000, value: 10 });

        assert_eq!(series.stats.min, -10);
        assert_eq!(series.stats.max, 10);
        assert!((series.stats.mean - 0.0).abs() < 0.1);
    }
}
