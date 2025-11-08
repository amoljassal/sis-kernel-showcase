//! Model Drift Detection
//!
//! Monitors model performance over time to detect
//! degradation (drift) from baseline metrics.

use alloc::string::String;
use spin::Mutex;

#[cfg(feature = "decision-traces")]
use crate::trace_decision::DecisionTrace;

/// Drift detection status
#[derive(Debug, Clone)]
pub enum DriftStatus {
    /// No drift detected
    Ok,
    /// Warning level drift
    Warning {
        metric: String,
        delta: f32,
    },
    /// Alert level drift (action required)
    Alert {
        metric: String,
        baseline: f32,
        current: f32,
        delta: f32,
    },
}

/// Ring buffer for recent values
struct RingBuffer<T, const N: usize> {
    data: [Option<T>; N],
    index: usize,
    count: usize,
}

impl<T: Copy + Default, const N: usize> RingBuffer<T, N> {
    const fn new() -> Self {
        Self {
            data: [None; N],
            index: 0,
            count: 0,
        }
    }

    fn push(&mut self, value: T) {
        self.data[self.index] = Some(value);
        self.index = (self.index + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }

    fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.data[..self.count].iter().filter_map(|&x| x)
    }

    fn len(&self) -> usize {
        self.count
    }
}

/// Drift monitor
pub struct DriftMonitor {
    confidence_baseline: f32,
    confidence_window: RingBuffer<u32, 100>,
    warning_threshold: f32,   // 10% drift
    alert_threshold: f32,     // 20% drift
}

impl DriftMonitor {
    /// Create new drift monitor
    pub const fn new() -> Self {
        Self {
            confidence_baseline: 800.0,  // 80% baseline
            confidence_window: RingBuffer::new(),
            warning_threshold: 100.0,    // 10%
            alert_threshold: 200.0,      // 20%
        }
    }

    /// Check for drift in decision trace
    #[cfg(feature = "decision-traces")]
    pub fn check_drift(&mut self, trace: &DecisionTrace) -> DriftStatus {
        // Add to window
        self.confidence_window.push(trace.confidence);

        // Need enough samples
        if self.confidence_window.len() < 10 {
            return DriftStatus::Ok;
        }

        // Calculate recent average
        let recent_avg = self.calculate_average();

        // Check for drift
        let drift_delta = (recent_avg - self.confidence_baseline).abs();

        if drift_delta > self.alert_threshold {
            DriftStatus::Alert {
                metric: String::from("confidence"),
                baseline: self.confidence_baseline,
                current: recent_avg,
                delta: drift_delta,
            }
        } else if drift_delta > self.warning_threshold {
            DriftStatus::Warning {
                metric: String::from("confidence"),
                delta: drift_delta,
            }
        } else {
            DriftStatus::Ok
        }
    }

    /// Set baseline from current average
    pub fn set_baseline(&mut self) {
        if self.confidence_window.len() > 0 {
            self.confidence_baseline = self.calculate_average();
        }
    }

    /// Set custom baseline
    pub fn set_baseline_value(&mut self, baseline: f32) {
        self.confidence_baseline = baseline;
    }

    /// Get current baseline
    pub fn get_baseline(&self) -> f32 {
        self.confidence_baseline
    }

    fn calculate_average(&self) -> f32 {
        let values: alloc::vec::Vec<u32> = self.confidence_window.iter().collect();
        if values.is_empty() {
            return 0.0;
        }
        let sum: u32 = values.iter().sum();
        sum as f32 / values.len() as f32
    }
}

/// Handle drift status with automatic actions
pub fn handle_drift(status: DriftStatus) {
    match status {
        DriftStatus::Ok => {}
        DriftStatus::Warning { ref metric, delta } => {
            crate::println!("[DRIFT WARNING] {}: delta={:.2}", metric, delta);
        }
        DriftStatus::Alert { ref metric, baseline, current, delta } => {
            crate::println!("[DRIFT ALERT] {}: baseline={:.2}, current={:.2}, delta={:.2}",
                metric, baseline, current, delta);

            // TODO: Automatic action - switch to safe mode
            crate::println!("[DRIFT] Switching to safe mode (not implemented)");
        }
    }
}

/// Global drift monitor instance
pub static DRIFT_MONITOR: Mutex<DriftMonitor> = Mutex::new(DriftMonitor::new());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_monitor() {
        let monitor = DriftMonitor::new();
        assert_eq!(monitor.get_baseline(), 800.0);
    }

    #[test]
    fn test_ring_buffer() {
        let mut buf = RingBuffer::<u32, 5>::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.len(), 3);

        let values: alloc::vec::Vec<u32> = buf.iter().collect();
        assert_eq!(values, alloc::vec![1, 2, 3]);
    }
}
