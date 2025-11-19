// Mock NPU backend for unit testing without real hardware

use crate::npu_backend::*;
use async_trait::async_trait;
use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Mock NPU backend for testing
pub struct MockNpuBackend {
    initialized: bool,
    job_queue: Arc<Mutex<VecDeque<NpuJob>>>,
    completed_jobs: Arc<Mutex<HashMap<u32, NpuResult>>>,
    next_job_id: Arc<Mutex<u32>>,
    statistics: Arc<Mutex<NpuStatistics>>,
    config: NpuBackendConfig,
    start_time: Instant,
    /// Simulated failure mode for testing
    simulate_failure: bool,
    /// Simulated queue full condition
    simulate_queue_full: bool,
}

impl MockNpuBackend {
    /// Create a new mock NPU backend
    pub fn new(config: NpuBackendConfig) -> Self {
        Self {
            initialized: false,
            job_queue: Arc::new(Mutex::new(VecDeque::new())),
            completed_jobs: Arc::new(Mutex::new(HashMap::new())),
            next_job_id: Arc::new(Mutex::new(1)),
            statistics: Arc::new(Mutex::new(NpuStatistics {
                total_jobs: 0,
                completed_jobs: 0,
                failed_jobs: 0,
                total_cycles: 0,
                queue_depth: 0,
                average_latency_us: 0.0,
                p95_latency_us: 0.0,
                p99_latency_us: 0.0,
                utilization_percent: 0.0,
            })),
            config,
            start_time: Instant::now(),
            simulate_failure: false,
            simulate_queue_full: false,
        }
    }

    /// Enable simulated failure mode for testing error handling
    pub fn set_simulate_failure(&mut self, enable: bool) {
        self.simulate_failure = enable;
    }

    /// Enable simulated queue full condition
    pub fn set_simulate_queue_full(&mut self, enable: bool) {
        self.simulate_queue_full = enable;
    }

    /// Simulate job processing (internal helper)
    fn process_job(&self, mut job: NpuJob) -> NpuResult {
        if self.simulate_failure {
            return NpuResult {
                job_id: job.job_id,
                status: NpuJobStatus::Failed,
                cycles_used: 0,
                completion_time_us: 0,
                output_data: vec![],
                error_code: Some(0xE001),
            };
        }

        // Simulate realistic NPU processing
        let cycles = self.estimate_cycles(&job);
        let latency_us = (cycles as f64 / 1000.0) as u64;  // Assume 1GHz clock

        // Generate mock output data
        let output_data = vec![0.5; job.output_size as usize / 4];  // Assume f32 outputs

        // Update job ID
        let job_id = {
            let mut next_id = self.next_job_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };
        job.job_id = job_id;

        NpuResult {
            job_id,
            status: NpuJobStatus::Completed,
            cycles_used: cycles,
            completion_time_us: latency_us,
            output_data,
            error_code: None,
        }
    }

    /// Estimate processing cycles for a job (simulation)
    fn estimate_cycles(&self, job: &NpuJob) -> u64 {
        let base_cycles = 1000u64;
        let input_cycles = (job.input_size as u64) / 4;  // 1 cycle per 4 bytes
        let output_cycles = (job.output_size as u64) / 8;

        // Priority affects scheduling but not execution time
        let total = base_cycles + input_cycles + output_cycles;

        // Add model complexity factor
        let complexity_factor = match job.model_id % 4 {
            0 => 1.0,   // Simple model
            1 => 1.5,   // Medium model
            2 => 2.0,   // Complex model
            _ => 3.0,   // Very complex model
        };

        (total as f64 * complexity_factor) as u64
    }

    /// Update statistics (internal helper)
    fn update_statistics(&self, result: &NpuResult) {
        let mut stats = self.statistics.lock().unwrap();
        stats.total_jobs += 1;

        if result.status == NpuJobStatus::Completed {
            stats.completed_jobs += 1;
            stats.total_cycles += result.cycles_used;

            // Update latency statistics (simplified)
            let n = stats.completed_jobs as f64;
            stats.average_latency_us =
                (stats.average_latency_us * (n - 1.0) + result.completion_time_us as f64) / n;

            stats.p95_latency_us = result.completion_time_us as f64 * 1.2;  // Simplified
            stats.p99_latency_us = result.completion_time_us as f64 * 1.5;  // Simplified
        } else {
            stats.failed_jobs += 1;
        }

        // Update utilization
        let queue = self.job_queue.lock().unwrap();
        stats.queue_depth = queue.len();
        stats.utilization_percent = (queue.len() as f64 / self.config.max_queue_depth as f64) * 100.0;
    }
}

#[async_trait]
impl NpuBackend for MockNpuBackend {
    async fn initialize(&mut self) -> Result<(), NpuError> {
        if self.simulate_failure {
            return Err(NpuError::HardwareError("Simulated initialization failure".to_string()));
        }

        self.initialized = true;
        self.start_time = Instant::now();
        Ok(())
    }

    async fn is_alive(&self) -> bool {
        self.initialized && !self.simulate_failure
    }

    async fn submit_job(&mut self, job: NpuJob) -> Result<u32, NpuError> {
        if !self.initialized {
            return Err(NpuError::NotInitialized);
        }

        if self.simulate_queue_full {
            return Err(NpuError::QueueFull);
        }

        // Validate job parameters
        if job.input_size == 0 || job.output_size == 0 {
            return Err(NpuError::InvalidJob("Zero size buffer".to_string()));
        }

        if job.model_addr == 0 || job.input_addr == 0 || job.output_addr == 0 {
            return Err(NpuError::InvalidJob("Null address".to_string()));
        }

        // Check queue capacity
        {
            let queue = self.job_queue.lock().unwrap();
            if queue.len() >= self.config.max_queue_depth {
                return Err(NpuError::QueueFull);
            }
        }

        // Process job immediately (mock backend is synchronous)
        let result = self.process_job(job);
        let job_id = result.job_id;

        // Store result
        {
            let mut completed = self.completed_jobs.lock().unwrap();
            completed.insert(job_id, result.clone());
        }

        // Update statistics
        self.update_statistics(&result);

        Ok(job_id)
    }

    async fn poll_completed(&mut self) -> Result<Vec<NpuResult>, NpuError> {
        if !self.initialized {
            return Err(NpuError::NotInitialized);
        }

        let mut completed = self.completed_jobs.lock().unwrap();
        let results: Vec<NpuResult> = completed.values().cloned().collect();
        completed.clear();

        Ok(results)
    }

    async fn wait_for_job(&mut self, job_id: u32, timeout: Duration) -> Result<NpuResult, NpuError> {
        if !self.initialized {
            return Err(NpuError::NotInitialized);
        }

        let start = Instant::now();
        loop {
            // Check if job is completed
            {
                let completed = self.completed_jobs.lock().unwrap();
                if let Some(result) = completed.get(&job_id) {
                    return Ok(result.clone());
                }
            }

            // Check timeout
            if start.elapsed() >= timeout {
                return Err(NpuError::Timeout);
            }

            // Small sleep to avoid busy waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn get_statistics(&self) -> Result<NpuStatistics, NpuError> {
        if !self.initialized {
            return Err(NpuError::NotInitialized);
        }

        let stats = self.statistics.lock().unwrap();
        Ok(stats.clone())
    }

    async fn reset(&mut self) -> Result<(), NpuError> {
        if !self.initialized {
            return Err(NpuError::NotInitialized);
        }

        // Clear all queues and reset statistics
        {
            let mut queue = self.job_queue.lock().unwrap();
            queue.clear();
        }

        {
            let mut completed = self.completed_jobs.lock().unwrap();
            completed.clear();
        }

        {
            let mut stats = self.statistics.lock().unwrap();
            *stats = NpuStatistics {
                total_jobs: 0,
                completed_jobs: 0,
                failed_jobs: 0,
                total_cycles: 0,
                queue_depth: 0,
                average_latency_us: 0.0,
                p95_latency_us: 0.0,
                p99_latency_us: 0.0,
                utilization_percent: 0.0,
            };
        }

        {
            let mut next_id = self.next_job_id.lock().unwrap();
            *next_id = 1;
        }

        self.start_time = Instant::now();
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), NpuError> {
        self.initialized = false;
        self.reset().await
    }

    fn name(&self) -> &'static str {
        "MockNpuBackend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_npu_initialization() {
        let config = NpuBackendConfig::default();
        let mut backend = MockNpuBackend::new(config);

        assert!(!backend.is_alive().await);
        assert!(backend.initialize().await.is_ok());
        assert!(backend.is_alive().await);
    }

    #[tokio::test]
    async fn test_mock_npu_job_submission() {
        let config = NpuBackendConfig::default();
        let mut backend = MockNpuBackend::new(config);
        backend.initialize().await.unwrap();

        let job = NpuJob {
            job_id: 0,  // Will be assigned
            model_id: 42,
            model_addr: 0x1000_0000,
            input_addr: 0x2000_0000,
            output_addr: 0x3000_0000,
            input_size: 1000,
            output_size: 100,
            priority: NpuPriority::High,
            max_cycles: 50_000,
        };

        let job_id = backend.submit_job(job).await.unwrap();
        assert_eq!(job_id, 1);

        let results = backend.poll_completed().await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].job_id, 1);
        assert_eq!(results[0].status, NpuJobStatus::Completed);
    }

    #[tokio::test]
    async fn test_mock_npu_failure_simulation() {
        let config = NpuBackendConfig::default();
        let mut backend = MockNpuBackend::new(config);
        backend.set_simulate_failure(true);

        assert!(backend.initialize().await.is_err());
    }

    #[tokio::test]
    async fn test_mock_npu_queue_full() {
        let config = NpuBackendConfig::default();
        let mut backend = MockNpuBackend::new(config);
        backend.initialize().await.unwrap();
        backend.set_simulate_queue_full(true);

        let job = NpuJob {
            job_id: 0,
            model_id: 42,
            model_addr: 0x1000_0000,
            input_addr: 0x2000_0000,
            output_addr: 0x3000_0000,
            input_size: 1000,
            output_size: 100,
            priority: NpuPriority::High,
            max_cycles: 50_000,
        };

        let result = backend.submit_job(job).await;
        assert!(matches!(result, Err(NpuError::QueueFull)));
    }

    #[tokio::test]
    async fn test_mock_npu_statistics() {
        let config = NpuBackendConfig::default();
        let mut backend = MockNpuBackend::new(config);
        backend.initialize().await.unwrap();

        let job = NpuJob {
            job_id: 0,
            model_id: 42,
            model_addr: 0x1000_0000,
            input_addr: 0x2000_0000,
            output_addr: 0x3000_0000,
            input_size: 1000,
            output_size: 100,
            priority: NpuPriority::High,
            max_cycles: 50_000,
        };

        backend.submit_job(job.clone()).await.unwrap();
        backend.submit_job(job).await.unwrap();

        let stats = backend.get_statistics().await.unwrap();
        assert_eq!(stats.total_jobs, 2);
        assert_eq!(stats.completed_jobs, 2);
        assert_eq!(stats.failed_jobs, 0);
        assert!(stats.average_latency_us > 0.0);
    }
}
