//! Metrics store - manages all metric series with cardinality and memory limits

use super::series::{MetricPoint, MetricSeries, SeriesStats};
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Configuration for metrics store
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// High-resolution retention time in milliseconds (default: 5 min)
    pub high_res_retention_ms: u64,
    /// Downsampled retention time in milliseconds (default: 1 hr)
    pub downsample_retention_ms: u64,
    /// Maximum number of unique series (cardinality limit)
    pub cardinality_limit: usize,
    /// Target memory budget in bytes (~64MB)
    pub memory_budget_bytes: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            high_res_retention_ms: 5 * 60 * 1000,      // 5 minutes
            downsample_retention_ms: 60 * 60 * 1000,   // 1 hour
            cardinality_limit: 256,
            memory_budget_bytes: 64 * 1024 * 1024,     // 64 MB
        }
    }
}

/// Metrics store state
pub struct MetricsStore {
    /// All metric series by name
    series: RwLock<HashMap<String, MetricSeries>>,
    /// Configuration
    config: MetricsConfig,
    /// Total series dropped due to cardinality limit
    dropped_series_count: AtomicUsize,
    /// Last time we logged a cardinality warning
    last_cardinality_warn: RwLock<Option<SystemTime>>,
    /// Last time we logged a memory warning
    last_memory_warn: RwLock<Option<SystemTime>>,
}

impl MetricsStore {
    /// Create a new metrics store
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            series: RwLock::new(HashMap::new()),
            config,
            dropped_series_count: AtomicUsize::new(0),
            last_cardinality_warn: RwLock::new(None),
            last_memory_warn: RwLock::new(None),
        }
    }

    /// Record a new metric point
    pub async fn record(&self, name: String, value: i64, timestamp: i64) -> Result<()> {
        let mut series_map = self.series.write().await;

        // Check if series exists
        if let Some(series) = series_map.get_mut(&name) {
            // Validate timestamp (reject future points >5s skew)
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;

            if timestamp > now_ms + 5000 {
                debug!("Rejecting future metric point: {} at {}", name, timestamp);
                return Ok(());
            }

            series.push(MetricPoint { ts: timestamp, value });
            return Ok(());
        }

        // New series - check cardinality limit
        if series_map.len() >= self.config.cardinality_limit {
            self.dropped_series_count.fetch_add(1, Ordering::Relaxed);

            // Log warning (rate-limited to once per minute)
            self.log_cardinality_warning(&series_map).await;

            bail!("Cardinality limit exceeded: {} series", self.config.cardinality_limit);
        }

        // Calculate buffer capacities based on retention times
        // Assume ~10 samples/sec for high-res
        let high_res_capacity = (self.config.high_res_retention_ms / 100) as usize;
        // Assume ~1 sample/10sec for downsampled
        let downsample_capacity = (self.config.downsample_retention_ms / 10000) as usize;

        // Create new series
        let mut new_series = MetricSeries::new(
            name.clone(),
            high_res_capacity,
            downsample_capacity,
        );
        new_series.push(MetricPoint { ts: timestamp, value });

        series_map.insert(name, new_series);
        Ok(())
    }

    /// Get all series names with metadata
    pub async fn list_series(&self) -> Vec<SeriesMetadata> {
        let series_map = self.series.read().await;

        series_map
            .iter()
            .map(|(name, series)| SeriesMetadata {
                name: name.clone(),
                count: series.len(),
                last_ts: series.last_ts,
                stats: series.stats.clone(),
            })
            .collect()
    }

    /// Query a specific series
    pub async fn query(
        &self,
        name: &str,
        from: i64,
        to: i64,
        max_points: usize,
    ) -> Result<QueryResult> {
        let series_map = self.series.read().await;

        let series = series_map
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Series not found: {}", name))?;

        // Get points in range
        let mut points = series.get_points_in_range(from, to);

        // Sort by timestamp (handle out-of-order)
        points.sort_by_key(|p| p.ts);

        // Check if we need downsampling
        let needs_downsample = points.len() > max_points;

        let (final_points, downsampled) = if needs_downsample {
            // Try to use pre-computed downsampled points if available
            let ds_points = series.get_downsampled_in_range(from, to);
            if !ds_points.is_empty() && ds_points.len() <= max_points {
                (ds_points, true)
            } else {
                // Use LTTB for downsampling (fallback to min/max if LTTB fails)
                let lttb_result = crate::metrics::downsample_lttb(&points, max_points);
                if lttb_result.len() <= max_points && !lttb_result.is_empty() {
                    (lttb_result, true)
                } else {
                    // Fallback: simple min/max bucketing
                    (self.downsample_minmax(&points, max_points), true)
                }
            }
        } else {
            (points, false)
        };

        Ok(QueryResult {
            name: name.to_string(),
            points: final_points,
            downsampled,
            from,
            to,
        })
    }

    /// Get the number of series
    pub async fn series_count(&self) -> usize {
        self.series.read().await.len()
    }

    /// Get total dropped series count
    pub fn dropped_series_count(&self) -> usize {
        self.dropped_series_count.load(Ordering::Relaxed)
    }

    /// Simple min/max bucketing downsampling
    fn downsample_minmax(&self, points: &[MetricPoint], target: usize) -> Vec<MetricPoint> {
        if points.len() <= target {
            return points.to_vec();
        }

        let bucket_size = points.len() / target;
        let mut result = Vec::with_capacity(target);

        for chunk in points.chunks(bucket_size) {
            if chunk.is_empty() {
                continue;
            }

            // Find min and max in bucket
            let min_point = chunk.iter().min_by_key(|p| p.value).unwrap();
            let max_point = chunk.iter().max_by_key(|p| p.value).unwrap();

            // Add both (preserving extremes)
            if min_point.ts < max_point.ts {
                result.push(*min_point);
                result.push(*max_point);
            } else {
                result.push(*max_point);
                result.push(*min_point);
            }
        }

        // Trim to target if we exceeded
        result.truncate(target);
        result
    }

    /// Log cardinality warning (rate-limited)
    async fn log_cardinality_warning(&self, series_map: &HashMap<String, MetricSeries>) {
        let mut last_warn = self.last_cardinality_warn.write().await;
        let now = SystemTime::now();

        // Only log once per minute
        if let Some(last) = *last_warn {
            if now.duration_since(last).unwrap().as_secs() < 60 {
                return;
            }
        }

        warn!(
            "Cardinality limit exceeded: {}/{} series, {} dropped",
            series_map.len(),
            self.config.cardinality_limit,
            self.dropped_series_count.load(Ordering::Relaxed),
        );

        *last_warn = Some(now);
    }
}

/// Metadata for a metric series
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SeriesMetadata {
    pub name: String,
    pub count: usize,
    pub last_ts: i64,
    pub stats: SeriesStats,
}

/// Result of a metric query
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct QueryResult {
    pub name: String,
    pub points: Vec<MetricPoint>,
    pub downsampled: bool,
    pub from: i64,
    pub to: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_and_query() {
        let store = MetricsStore::new(MetricsConfig::default());

        // Record some points
        store.record("test_metric".to_string(), 100, 1000).await.unwrap();
        store.record("test_metric".to_string(), 200, 2000).await.unwrap();
        store.record("test_metric".to_string(), 150, 3000).await.unwrap();

        // Query
        let result = store.query("test_metric", 0, 5000, 1000).await.unwrap();
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.downsampled, false);
    }

    #[tokio::test]
    async fn test_cardinality_limit() {
        let mut config = MetricsConfig::default();
        config.cardinality_limit = 2;

        let store = MetricsStore::new(config);

        // First two series should succeed
        store.record("series1".to_string(), 100, 1000).await.unwrap();
        store.record("series2".to_string(), 200, 2000).await.unwrap();

        // Third should fail
        let result = store.record("series3".to_string(), 300, 3000).await;
        assert!(result.is_err());
        assert_eq!(store.dropped_series_count(), 1);
    }

    #[tokio::test]
    async fn test_list_series() {
        let store = MetricsStore::new(MetricsConfig::default());

        store.record("metric1".to_string(), 100, 1000).await.unwrap();
        store.record("metric2".to_string(), 200, 2000).await.unwrap();

        let list = store.list_series().await;
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|m| m.name == "metric1"));
        assert!(list.iter().any(|m| m.name == "metric2"));
    }

    #[tokio::test]
    async fn test_out_of_order_timestamps() {
        let store = MetricsStore::new(MetricsConfig::default());

        // Record out of order
        store.record("test".to_string(), 3, 3000).await.unwrap();
        store.record("test".to_string(), 1, 1000).await.unwrap();
        store.record("test".to_string(), 2, 2000).await.unwrap();

        // Query should return sorted
        let result = store.query("test", 0, 5000, 1000).await.unwrap();
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.points[0].ts, 1000);
        assert_eq!(result.points[1].ts, 2000);
        assert_eq!(result.points[2].ts, 3000);
    }
}
