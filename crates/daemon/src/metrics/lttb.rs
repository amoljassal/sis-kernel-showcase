//! LTTB (Largest Triangle Three Buckets) downsampling algorithm
//!
//! Preserves visual characteristics of time-series data by selecting
//! points that form the largest triangles with their neighbors.
//!
//! Reference: Sveinn Steinarsson, 2013
//! https://skemman.is/bitstream/1946/15343/3/SS_MSthesis.pdf

use super::series::MetricPoint;

/// Downsample data using LTTB algorithm
///
/// # Arguments
/// * `data` - Input data points (must be sorted by timestamp)
/// * `threshold` - Target number of output points
///
/// # Returns
/// Downsampled points with length ≤ threshold
pub fn downsample_lttb(data: &[MetricPoint], threshold: usize) -> Vec<MetricPoint> {
    if data.len() <= threshold || threshold < 3 {
        return data.to_vec();
    }

    let mut sampled = Vec::with_capacity(threshold);

    // Always include first point
    sampled.push(data[0]);

    // Bucket size (float for precision)
    let bucket_size = (data.len() - 2) as f64 / (threshold - 2) as f64;

    // Index of point selected in previous bucket
    let mut prev_selected_idx = 0_usize;

    for bucket_idx in 0..(threshold - 2) {
        // Calculate range of current bucket
        let start_idx = ((bucket_idx as f64 * bucket_size) as usize + 1).min(data.len() - 2);
        let end_idx = ((bucket_idx as f64 + 1.0) * bucket_size + 1.0) as usize + 1;
        let end_idx = end_idx.min(data.len());

        if start_idx >= end_idx {
            continue;
        }

        // Calculate average point of next bucket (for triangle calculation)
        let next_bucket_start = end_idx;
        let next_bucket_end = (((bucket_idx + 1) as f64 + 1.0) * bucket_size + 1.0) as usize + 1;
        let next_bucket_end = next_bucket_end.min(data.len());

        let (avg_ts, avg_value) = if next_bucket_end > next_bucket_start {
            let sum_ts: i64 = data[next_bucket_start..next_bucket_end]
                .iter()
                .map(|p| p.ts)
                .sum();
            let sum_value: i64 = data[next_bucket_start..next_bucket_end]
                .iter()
                .map(|p| p.value)
                .sum();
            let count = (next_bucket_end - next_bucket_start) as i64;
            (sum_ts / count, sum_value / count)
        } else {
            // Last bucket - use last point
            let last = data[data.len() - 1];
            (last.ts, last.value)
        };

        // Find point in current bucket that forms largest triangle
        let prev_point = data[prev_selected_idx];
        let mut max_area = -1.0_f64;
        let mut max_idx = start_idx;

        for idx in start_idx..end_idx {
            let point = data[idx];

            // Calculate triangle area using cross product
            // Area = 0.5 * |x1(y2-y3) + x2(y3-y1) + x3(y1-y2)|
            let area = ((prev_point.ts - avg_ts) * (point.value - prev_point.value)
                - (prev_point.ts - point.ts) * (avg_value - prev_point.value))
                .abs() as f64;

            if area > max_area {
                max_area = area;
                max_idx = idx;
            }
        }

        sampled.push(data[max_idx]);
        prev_selected_idx = max_idx;
    }

    // Always include last point
    sampled.push(data[data.len() - 1]);

    sampled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lttb_basic() {
        // Create test data: sine wave with 100 points
        let data: Vec<MetricPoint> = (0..100)
            .map(|i| {
                let x = i as f64 / 10.0;
                let y = (x.sin() * 100.0) as i64;
                MetricPoint {
                    ts: i as i64 * 1000,
                    value: y,
                }
            })
            .collect();

        // Downsample to 20 points
        let downsampled = downsample_lttb(&data, 20);

        assert_eq!(downsampled.len(), 20);
        // First and last points should be preserved
        assert_eq!(downsampled[0].ts, data[0].ts);
        assert_eq!(downsampled[19].ts, data[99].ts);
    }

    #[test]
    fn test_lttb_too_few_points() {
        let data: Vec<MetricPoint> = vec![
            MetricPoint { ts: 1000, value: 10 },
            MetricPoint { ts: 2000, value: 20 },
        ];

        // Requesting more points than available returns all
        let result = downsample_lttb(&data, 10);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_lttb_threshold_less_than_3() {
        let data: Vec<MetricPoint> = (0..100)
            .map(|i| MetricPoint {
                ts: i as i64 * 1000,
                value: i as i64,
            })
            .collect();

        // Threshold < 3 returns original data
        let result = downsample_lttb(&data, 2);
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_lttb_preserves_extremes() {
        // Create data with a spike in the middle
        let mut data = Vec::new();
        for i in 0..50 {
            data.push(MetricPoint {
                ts: i as i64 * 1000,
                value: 10,
            });
        }
        // Spike
        data.push(MetricPoint {
            ts: 50 * 1000,
            value: 1000,
        });
        for i in 51..100 {
            data.push(MetricPoint {
                ts: i as i64 * 1000,
                value: 10,
            });
        }

        let downsampled = downsample_lttb(&data, 10);

        // Spike should be included (large triangle)
        assert!(downsampled.iter().any(|p| p.value == 1000));
    }
}
