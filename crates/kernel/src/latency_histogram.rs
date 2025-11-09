//! Latency Histogram for Percentile Tracking
//!
//! Provides efficient latency tracking and percentile calculation
//! for stress test performance analysis.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Histogram for latency percentile calculations
/// Uses logarithmic buckets for efficient percentile tracking
pub struct LatencyHistogram {
    // Bucket boundaries (in nanoseconds): [0-1us, 1-10us, 10-100us, 100us-1ms, 1-10ms, 10-100ms, 100ms+]
    buckets: [AtomicU32; 10],
    count: AtomicU64,
    min_ns: AtomicU64,
    max_ns: AtomicU64,
    sum_ns: AtomicU64,
}

impl LatencyHistogram {
    pub const fn new() -> Self {
        const BUCKET: AtomicU32 = AtomicU32::new(0);
        Self {
            buckets: [BUCKET; 10],
            count: AtomicU64::new(0),
            min_ns: AtomicU64::new(u64::MAX),
            max_ns: AtomicU64::new(0),
            sum_ns: AtomicU64::new(0),
        }
    }

    /// Record a latency measurement
    pub fn record(&self, latency_ns: u64) {
        // Update count
        self.count.fetch_add(1, Ordering::Relaxed);

        // Update sum
        self.sum_ns.fetch_add(latency_ns, Ordering::Relaxed);

        // Update min
        let mut current_min = self.min_ns.load(Ordering::Relaxed);
        while latency_ns < current_min {
            match self.min_ns.compare_exchange_weak(
                current_min,
                latency_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        // Update max
        let mut current_max = self.max_ns.load(Ordering::Relaxed);
        while latency_ns > current_max {
            match self.max_ns.compare_exchange_weak(
                current_max,
                latency_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        // Determine bucket and increment
        let bucket_idx = Self::get_bucket_index(latency_ns);
        self.buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
    }

    /// Determine which bucket a latency value falls into
    fn get_bucket_index(latency_ns: u64) -> usize {
        match latency_ns {
            0..=999 => 0,                    // < 1us
            1_000..=9_999 => 1,              // 1-10us
            10_000..=99_999 => 2,            // 10-100us
            100_000..=999_999 => 3,          // 100us-1ms
            1_000_000..=9_999_999 => 4,      // 1-10ms
            10_000_000..=99_999_999 => 5,    // 10-100ms
            100_000_000..=999_999_999 => 6,  // 100ms-1s
            1_000_000_000..=9_999_999_999 => 7, // 1-10s
            10_000_000_000..=99_999_999_999 => 8, // 10-100s
            _ => 9,                          // 100s+
        }
    }

    /// Calculate percentile (approximate)
    /// p should be in range 0-100 (e.g., 50 for median, 95 for p95)
    pub fn percentile(&self, p: u8) -> u64 {
        let total_count = self.count.load(Ordering::Relaxed);
        if total_count == 0 {
            return 0;
        }

        let target_count = (total_count * p as u64) / 100;
        let mut cumulative = 0u64;

        for (idx, bucket) in self.buckets.iter().enumerate() {
            let bucket_count = bucket.load(Ordering::Relaxed) as u64;
            cumulative += bucket_count;

            if cumulative >= target_count {
                // Return midpoint of this bucket
                return Self::get_bucket_midpoint(idx);
            }
        }

        // Fallback to max
        self.max_ns.load(Ordering::Relaxed)
    }

    /// Get the midpoint value for a bucket
    fn get_bucket_midpoint(bucket_idx: usize) -> u64 {
        match bucket_idx {
            0 => 500,                     // ~500ns
            1 => 5_000,                   // ~5us
            2 => 50_000,                  // ~50us
            3 => 500_000,                 // ~500us
            4 => 5_000_000,               // ~5ms
            5 => 50_000_000,              // ~50ms
            6 => 500_000_000,             // ~500ms
            7 => 5_000_000_000,           // ~5s
            8 => 50_000_000_000,          // ~50s
            _ => 500_000_000_000,         // ~500s
        }
    }

    /// Get latency report
    pub fn report(&self) -> LatencyReport {
        let count = self.count.load(Ordering::Relaxed);
        let avg_ns = if count > 0 {
            self.sum_ns.load(Ordering::Relaxed) / count
        } else {
            0
        };

        LatencyReport {
            p50: self.percentile(50),
            p95: self.percentile(95),
            p99: self.percentile(99),
            min: self.min_ns.load(Ordering::Relaxed),
            max: self.max_ns.load(Ordering::Relaxed),
            avg: avg_ns,
            count,
        }
    }

    /// Reset histogram
    pub fn reset(&self) {
        for bucket in &self.buckets {
            bucket.store(0, Ordering::Relaxed);
        }
        self.count.store(0, Ordering::Relaxed);
        self.min_ns.store(u64::MAX, Ordering::Relaxed);
        self.max_ns.store(0, Ordering::Relaxed);
        self.sum_ns.store(0, Ordering::Relaxed);
    }
}

/// Latency report structure
#[derive(Copy, Clone, Debug)]
pub struct LatencyReport {
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub min: u64,
    pub max: u64,
    pub avg: u64,
    pub count: u64,
}

impl LatencyReport {
    pub const fn new() -> Self {
        Self {
            p50: 0,
            p95: 0,
            p99: 0,
            min: 0,
            max: 0,
            avg: 0,
            count: 0,
        }
    }
}
