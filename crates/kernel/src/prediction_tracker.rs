//! Prediction Tracking and Validation
//!
//! Tracks predictions made by AI agents and measures accuracy over time.
//! Implements Out-of-Distribution (OOD) detection and learning validation.

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Maximum number of predictions to track
const MAX_PREDICTIONS: usize = 1000;

/// Maximum number of features for OOD detection
const MAX_FEATURES: usize = 12;

/// Prediction type classification
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum PredictionType {
    MemoryPressure = 0,
    MemoryCompactionNeeded = 1,
    SchedulingDeadlineMiss = 2,
    CommandHeavy = 3,
    CommandRapidStream = 4,
}

/// Prediction record with outcome tracking
#[derive(Copy, Clone)]
pub struct PredictionRecord {
    pub id: u64,
    pub timestamp: u64,
    pub prediction_type: PredictionType,
    pub predicted_value: i16,  // Q8.8 fixed-point or boolean (0/1)
    pub confidence: i16,        // Q8.8: 0-1000 representing 0.0-1.0
    pub actual_value: Option<i16>,
    pub outcome_timestamp: u64,
    pub valid: bool,
}

impl PredictionRecord {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            timestamp: 0,
            prediction_type: PredictionType::MemoryPressure,
            predicted_value: 0,
            confidence: 0,
            actual_value: None,
            outcome_timestamp: 0,
            valid: false,
        }
    }

    /// Compute prediction error (absolute difference)
    pub fn error(&self) -> Option<i16> {
        self.actual_value.map(|actual| {
            let diff = self.predicted_value - actual;
            if diff < 0 { -diff } else { diff }
        })
    }

    /// Check if prediction was correct (within 10% tolerance)
    pub fn is_correct(&self) -> Option<bool> {
        self.error().map(|err| err < 25)  // Q8.8: 25 = 0.1 (10% tolerance)
    }
}

/// Ring buffer for prediction tracking
pub struct PredictionLedger {
    predictions: [PredictionRecord; MAX_PREDICTIONS],
    head: usize,
    count: usize,
    next_id: u64,
}

impl PredictionLedger {
    const fn new() -> Self {
        Self {
            predictions: [PredictionRecord::empty(); MAX_PREDICTIONS],
            head: 0,
            count: 0,
            next_id: 1,
        }
    }

    /// Record a new prediction
    pub fn record(
        &mut self,
        prediction_type: PredictionType,
        predicted_value: i16,
        confidence: i16,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let timestamp = crate::time::get_timestamp_us();

        let record = PredictionRecord {
            id,
            timestamp,
            prediction_type,
            predicted_value,
            confidence,
            actual_value: None,
            outcome_timestamp: 0,
            valid: true,
        };

        self.predictions[self.head] = record;
        self.head = (self.head + 1) % MAX_PREDICTIONS;
        if self.count < MAX_PREDICTIONS {
            self.count += 1;
        }

        id
    }

    /// Update prediction with actual outcome
    pub fn update_outcome(&mut self, id: u64, actual_value: i16) -> bool {
        for i in 0..self.count {
            let record = &mut self.predictions[i];
            if record.valid && record.id == id {
                record.actual_value = Some(actual_value);
                record.outcome_timestamp = crate::time::get_timestamp_us();
                return true;
            }
        }
        false
    }

    /// Compute accuracy for predictions with outcomes
    pub fn compute_accuracy(&self, last_n: usize) -> (usize, usize) {
        let n = if last_n > self.count { self.count } else { last_n };
        let mut correct = 0;
        let mut total_with_outcomes = 0;

        let start_idx = if self.count < MAX_PREDICTIONS {
            0
        } else {
            self.head
        };

        for i in 0..n {
            let idx = (start_idx + self.count - n + i) % MAX_PREDICTIONS;
            let record = &self.predictions[idx];

            if record.valid && record.actual_value.is_some() {
                total_with_outcomes += 1;
                if let Some(true) = record.is_correct() {
                    correct += 1;
                }
            }
        }

        (correct, total_with_outcomes)
    }

    /// Compute accuracy by prediction type
    pub fn compute_accuracy_by_type(
        &self,
        pred_type: PredictionType,
        last_n: usize,
    ) -> (usize, usize) {
        let n = if last_n > self.count { self.count } else { last_n };
        let mut correct = 0;
        let mut total_with_outcomes = 0;

        let start_idx = if self.count < MAX_PREDICTIONS {
            0
        } else {
            self.head
        };

        for i in 0..n {
            let idx = (start_idx + self.count - n + i) % MAX_PREDICTIONS;
            let record = &self.predictions[idx];

            if record.valid
                && record.prediction_type == pred_type
                && record.actual_value.is_some()
            {
                total_with_outcomes += 1;
                if let Some(true) = record.is_correct() {
                    correct += 1;
                }
            }
        }

        (correct, total_with_outcomes)
    }

    /// Get total number of predictions
    pub fn len(&self) -> usize {
        self.count
    }

    /// Get prediction by ID
    pub fn get_by_id(&self, id: u64) -> Option<&PredictionRecord> {
        for i in 0..self.count {
            let record = &self.predictions[i];
            if record.valid && record.id == id {
                return Some(record);
            }
        }
        None
    }

    /// Get last N predictions
    pub fn get_last_n(&self, n: usize) -> &[PredictionRecord] {
        let count = if n > self.count { self.count } else { n };
        let start_idx = if self.count < MAX_PREDICTIONS {
            if self.count >= count {
                self.count - count
            } else {
                0
            }
        } else {
            (self.head + MAX_PREDICTIONS - count) % MAX_PREDICTIONS
        };

        // Return slice from circular buffer
        // NOTE: This simplified version only works if data is contiguous
        if start_idx + count <= MAX_PREDICTIONS {
            &self.predictions[start_idx..start_idx + count]
        } else {
            // Wrapped around - return just the tail portion for simplicity
            &self.predictions[start_idx..MAX_PREDICTIONS]
        }
    }
}

/// Out-of-Distribution (OOD) detection statistics
#[derive(Copy, Clone)]
pub struct DistributionStats {
    /// Feature means (Q8.8)
    pub means: [i16; MAX_FEATURES],
    /// Feature standard deviations (Q8.8)
    pub stddevs: [i16; MAX_FEATURES],
    /// Feature min values (Q8.8)
    pub mins: [i16; MAX_FEATURES],
    /// Feature max values (Q8.8)
    pub maxs: [i16; MAX_FEATURES],
    /// Number of samples used to compute stats
    pub sample_count: u32,
    pub valid: bool,
}

impl DistributionStats {
    pub const fn new() -> Self {
        Self {
            means: [0; MAX_FEATURES],
            stddevs: [256; MAX_FEATURES], // Q8.8: 256 = 1.0
            mins: [i16::MIN; MAX_FEATURES],
            maxs: [i16::MAX; MAX_FEATURES],
            sample_count: 0,
            valid: false,
        }
    }

    /// Update statistics with new sample
    pub fn update(&mut self, features: &[i16; MAX_FEATURES]) {
        if !self.valid {
            // First sample - initialize
            self.means = *features;
            self.sample_count = 1;
            self.valid = true;
            return;
        }

        // Incremental mean update
        self.sample_count += 1;
        let n = self.sample_count as i32;

        for i in 0..MAX_FEATURES {
            // Update min/max
            if features[i] < self.mins[i] {
                self.mins[i] = features[i];
            }
            if features[i] > self.maxs[i] {
                self.maxs[i] = features[i];
            }

            // Update mean: new_mean = old_mean + (value - old_mean) / n
            let diff = (features[i] as i32) - (self.means[i] as i32);
            self.means[i] = (self.means[i] as i32 + diff / n) as i16;
        }

        // Note: We're not computing true standard deviation here due to complexity
        // Instead, we'll use a simplified range-based estimation
        for i in 0..MAX_FEATURES {
            let range = (self.maxs[i] as i32) - (self.mins[i] as i32);
            self.stddevs[i] = (range / 6) as i16; // Approximate: range ≈ 6σ
        }
    }
}

/// OOD detector using simplified Mahalanobis distance
pub struct OODDetector {
    /// Training distribution statistics
    pub training_stats: DistributionStats,
    /// OOD threshold (Q8.8: 768 = 3.0 standard deviations)
    pub threshold: i16,
}

impl OODDetector {
    pub const fn new() -> Self {
        Self {
            training_stats: DistributionStats::new(),
            threshold: 768, // Q8.8: 3.0 sigma
        }
    }

    /// Compute simplified Mahalanobis distance
    /// Returns distance in Q8.8 format
    pub fn compute_distance(&self, features: &[i16; MAX_FEATURES]) -> i16 {
        if !self.training_stats.valid {
            return 0; // No training data yet
        }

        let mut _sum_squared_z_scores: i32 = 0;

        for i in 0..MAX_FEATURES {
            let mean = self.training_stats.means[i] as i32;
            let stddev = self.training_stats.stddevs[i] as i32;

            if stddev == 0 {
                continue; // Avoid division by zero
            }

            // Compute z-score: (value - mean) / stddev
            let diff = (features[i] as i32) - mean;
            let z_score = (diff * 256) / stddev; // Q8.8 math

            // Square and accumulate (unused for now, using max z-score instead)
            _sum_squared_z_scores += (z_score * z_score) / 256; // Normalize Q8.8
        }

        // Approximate Mahalanobis distance: sqrt(sum of squared z-scores)
        // Simplified: return max z-score instead of sqrt(sum) for efficiency
        let mut max_z_score: i16 = 0;
        for i in 0..MAX_FEATURES {
            let mean = self.training_stats.means[i] as i32;
            let stddev = self.training_stats.stddevs[i] as i32;

            if stddev == 0 {
                continue;
            }

            let diff = (features[i] as i32) - mean;
            let z_score = ((diff * 256) / stddev) as i16;
            let abs_z_score = if z_score < 0 { -z_score } else { z_score };

            if abs_z_score > max_z_score {
                max_z_score = abs_z_score;
            }
        }

        max_z_score
    }

    /// Check if features are out-of-distribution
    pub fn is_ood(&self, features: &[i16; MAX_FEATURES]) -> bool {
        let distance = self.compute_distance(features);
        distance > self.threshold
    }

    /// Train the detector with new sample
    pub fn train(&mut self, features: &[i16; MAX_FEATURES]) {
        self.training_stats.update(features);
    }
}

/// Global prediction ledger
static PREDICTION_LEDGER: Mutex<PredictionLedger> = Mutex::new(PredictionLedger::new());

/// Global OOD detector
static OOD_DETECTOR: Mutex<OODDetector> = Mutex::new(OODDetector::new());

/// Total OOD detections
static OOD_DETECTION_COUNT: AtomicU64 = AtomicU64::new(0);

// ============================================================================
// Public API
// ============================================================================

/// Record a new prediction
pub fn record_prediction(
    prediction_type: PredictionType,
    predicted_value: i16,
    confidence: i16,
) -> u64 {
    let mut ledger = PREDICTION_LEDGER.lock();
    ledger.record(prediction_type, predicted_value, confidence)
}

/// Update prediction with actual outcome
pub fn update_outcome(prediction_id: u64, actual_value: i16) -> bool {
    let mut ledger = PREDICTION_LEDGER.lock();
    ledger.update_outcome(prediction_id, actual_value)
}

/// Compute overall accuracy for last N predictions
pub fn compute_accuracy(last_n: usize) -> (usize, usize) {
    let ledger = PREDICTION_LEDGER.lock();
    ledger.compute_accuracy(last_n)
}

/// Compute accuracy by prediction type
pub fn compute_accuracy_by_type(
    pred_type: PredictionType,
    last_n: usize,
) -> (usize, usize) {
    let ledger = PREDICTION_LEDGER.lock();
    ledger.compute_accuracy_by_type(pred_type, last_n)
}

/// Get prediction ledger (for detailed queries)
pub fn get_ledger() -> spin::MutexGuard<'static, PredictionLedger> {
    PREDICTION_LEDGER.lock()
}

/// Check if features are out-of-distribution
pub fn check_ood(features: &[i16; MAX_FEATURES]) -> (bool, i16) {
    let detector = OOD_DETECTOR.lock();
    let distance = detector.compute_distance(features);
    let is_ood = detector.is_ood(features);

    if is_ood {
        OOD_DETECTION_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    drop(detector);
    (is_ood, distance)
}

/// Train OOD detector with new sample
pub fn train_ood_detector(features: &[i16; MAX_FEATURES]) {
    let mut detector = OOD_DETECTOR.lock();
    detector.train(features);
}

/// Get OOD detection statistics
pub fn get_ood_stats() -> (u64, DistributionStats) {
    let detector = OOD_DETECTOR.lock();
    let count = OOD_DETECTION_COUNT.load(Ordering::Relaxed);
    let stats = detector.training_stats;
    drop(detector);
    (count, stats)
}

/// Get OOD threshold
pub fn get_ood_threshold() -> i16 {
    let detector = OOD_DETECTOR.lock();
    detector.threshold
}

/// Set OOD threshold
pub fn set_ood_threshold(threshold: i16) {
    let mut detector = OOD_DETECTOR.lock();
    detector.threshold = threshold;
}
