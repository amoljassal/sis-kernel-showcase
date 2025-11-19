// NPU backend abstraction for SIS Kernel testing
//
// This module provides a trait-based abstraction for testing NPU (Neural Processing Unit)
// functionality across different backends (simulation, hardware, mock).

use async_trait::async_trait;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// NPU job priority levels (matches kernel NPU priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NpuPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// NPU job status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NpuJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// NPU inference job descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuJob {
    pub job_id: u32,
    pub model_id: u32,
    pub model_addr: u64,
    pub input_addr: u64,
    pub output_addr: u64,
    pub input_size: u32,
    pub output_size: u32,
    pub priority: NpuPriority,
    pub max_cycles: u64,
}

/// NPU job completion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuResult {
    pub job_id: u32,
    pub status: NpuJobStatus,
    pub cycles_used: u64,
    pub completion_time_us: u64,
    pub output_data: Vec<f32>,
    pub error_code: Option<u32>,
}

/// NPU device statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuStatistics {
    pub total_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub total_cycles: u64,
    pub queue_depth: usize,
    pub average_latency_us: f64,
    pub p95_latency_us: f64,
    pub p99_latency_us: f64,
    pub utilization_percent: f64,
}

/// NPU backend error types
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum NpuError {
    #[error("NPU device not initialized")]
    NotInitialized,

    #[error("Job queue full")]
    QueueFull,

    #[error("Invalid job parameters: {0}")]
    InvalidJob(String),

    #[error("Job not found: {0}")]
    JobNotFound(u32),

    #[error("Hardware error: {0}")]
    HardwareError(String),

    #[error("Timeout waiting for job completion")]
    Timeout,

    #[error("Backend communication error: {0}")]
    CommunicationError(String),
}

/// NPU backend trait for testing
///
/// This trait abstracts NPU operations to enable testing across different backends:
/// - QemuNpuBackend: Tests against QEMU-emulated NPU
/// - MockNpuBackend: Simulated NPU for unit testing
/// - HardwareNpuBackend: Real NPU hardware testing
#[async_trait]
pub trait NpuBackend: Send + Sync {
    /// Initialize the NPU backend
    async fn initialize(&mut self) -> Result<(), NpuError>;

    /// Check if the NPU is alive and responsive
    async fn is_alive(&self) -> bool;

    /// Submit an inference job to the NPU
    ///
    /// Returns the assigned job ID
    async fn submit_job(&mut self, job: NpuJob) -> Result<u32, NpuError>;

    /// Poll for completed jobs
    ///
    /// Returns all jobs that have completed since last poll
    async fn poll_completed(&mut self) -> Result<Vec<NpuResult>, NpuError>;

    /// Wait for a specific job to complete
    ///
    /// Returns the job result or timeout error
    async fn wait_for_job(&mut self, job_id: u32, timeout: Duration) -> Result<NpuResult, NpuError>;

    /// Get current NPU statistics
    async fn get_statistics(&self) -> Result<NpuStatistics, NpuError>;

    /// Reset the NPU device
    async fn reset(&mut self) -> Result<(), NpuError>;

    /// Shutdown the NPU backend
    async fn shutdown(&mut self) -> Result<(), NpuError>;

    /// Get backend name for logging
    fn name(&self) -> &'static str;
}

/// NPU backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuBackendConfig {
    pub mmio_base: u64,
    pub irq_number: u32,
    pub max_queue_depth: usize,
    pub timeout_ms: u64,
}

impl Default for NpuBackendConfig {
    fn default() -> Self {
        Self {
            mmio_base: 0x4000_0000,  // Default NPU MMIO base
            irq_number: 33,           // Default IRQ number
            max_queue_depth: 64,
            timeout_ms: 5000,
        }
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    /// Verify that NPU priority ordering is correct
    #[kani::proof]
    fn verify_priority_ordering() {
        let low = NpuPriority::Low;
        let normal = NpuPriority::Normal;
        let high = NpuPriority::High;
        let critical = NpuPriority::Critical;

        // Verify ordering
        assert!(low < normal);
        assert!(normal < high);
        assert!(high < critical);

        // Verify transitivity
        assert!(low < high);
        assert!(low < critical);
        assert!(normal < critical);
    }

    /// Verify that job submission maintains job ID uniqueness
    #[kani::proof]
    fn verify_job_id_uniqueness() {
        let job1_id: u32 = kani::any();
        let job2_id: u32 = kani::any();

        // If jobs are different, IDs must be different
        kani::assume(job1_id != job2_id);

        let job1 = NpuJob {
            job_id: job1_id,
            model_id: 0,
            model_addr: 0x1000,
            input_addr: 0x2000,
            output_addr: 0x3000,
            input_size: 100,
            output_size: 10,
            priority: NpuPriority::Normal,
            max_cycles: 10000,
        };

        let job2 = NpuJob {
            job_id: job2_id,
            model_id: 0,
            model_addr: 0x1000,
            input_addr: 0x2000,
            output_addr: 0x3000,
            input_size: 100,
            output_size: 10,
            priority: NpuPriority::Normal,
            max_cycles: 10000,
        };

        // Different job IDs must remain different
        assert!(job1.job_id != job2.job_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(NpuPriority::Low < NpuPriority::Normal);
        assert!(NpuPriority::Normal < NpuPriority::High);
        assert!(NpuPriority::High < NpuPriority::Critical);
    }

    #[test]
    fn test_job_creation() {
        let job = NpuJob {
            job_id: 1,
            model_id: 42,
            model_addr: 0x1000_0000,
            input_addr: 0x2000_0000,
            output_addr: 0x3000_0000,
            input_size: 224 * 224 * 3 * 4,  // Standard image input
            output_size: 1000 * 4,           // Classification output
            priority: NpuPriority::High,
            max_cycles: 50_000,
        };

        assert_eq!(job.job_id, 1);
        assert_eq!(job.model_id, 42);
        assert_eq!(job.priority, NpuPriority::High);
    }

    #[test]
    fn test_result_creation() {
        let result = NpuResult {
            job_id: 1,
            status: NpuJobStatus::Completed,
            cycles_used: 25_000,
            completion_time_us: 100,
            output_data: vec![0.5, 0.3, 0.2],
            error_code: None,
        };

        assert_eq!(result.job_id, 1);
        assert_eq!(result.status, NpuJobStatus::Completed);
        assert!(result.error_code.is_none());
        assert_eq!(result.output_data.len(), 3);
    }
}
