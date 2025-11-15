//! Neural Processing Unit (NPU) device driver and emulation framework
//! 
//! This module provides:
//! - QEMU NPU device emulation interface
//! - MMIO register definitions and operations
//! - Interrupt-driven asynchronous inference
//! - Job queue management with priority scheduling
//! - Hardware acceleration simulation

use crate::ml::{ModelId, MLError};
use crate::trace::metric_kv;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::collections::VecDeque;

/// NPU MMIO register offsets
pub mod registers {
    pub const NPU_REG_STATUS: u64 = 0x00;
    pub const NPU_REG_COMMAND: u64 = 0x04;
    pub const NPU_REG_MODEL_ADDR: u64 = 0x08;
    pub const NPU_REG_INPUT_ADDR: u64 = 0x10;
    pub const NPU_REG_OUTPUT_ADDR: u64 = 0x18;
    pub const NPU_REG_INPUT_SIZE: u64 = 0x20;
    pub const NPU_REG_OUTPUT_SIZE: u64 = 0x24;
    pub const NPU_REG_CYCLES: u64 = 0x28;
    pub const NPU_REG_IRQ_STATUS: u64 = 0x2C;
    pub const NPU_REG_IRQ_ENABLE: u64 = 0x30;
    pub const NPU_REG_JOB_ID: u64 = 0x34;
    pub const NPU_REG_PRIORITY: u64 = 0x38;
    pub const NPU_REG_VERSION: u64 = 0x3C;
}

/// NPU status register bits
pub mod status {
    pub const NPU_STATUS_IDLE: u32 = 0x00;
    pub const NPU_STATUS_BUSY: u32 = 0x01;
    pub const NPU_STATUS_COMPLETE: u32 = 0x02;
    pub const NPU_STATUS_ERROR: u32 = 0x04;
    pub const NPU_STATUS_QUEUE_FULL: u32 = 0x08;
    pub const NPU_STATUS_QUEUE_EMPTY: u32 = 0x10;
}

/// NPU command register values
pub mod commands {
    pub const NPU_CMD_NOP: u32 = 0x00;
    pub const NPU_CMD_INFERENCE: u32 = 0x01;
    pub const NPU_CMD_RESET: u32 = 0x02;
    pub const NPU_CMD_QUEUE_FLUSH: u32 = 0x03;
    pub const NPU_CMD_POWER_DOWN: u32 = 0x04;
    pub const NPU_CMD_POWER_UP: u32 = 0x05;
}

/// NPU interrupt status bits
pub mod interrupts {
    pub const NPU_IRQ_COMPLETE: u32 = 0x01;
    pub const NPU_IRQ_ERROR: u32 = 0x02;
    pub const NPU_IRQ_QUEUE_READY: u32 = 0x04;
    pub const NPU_IRQ_OVERFLOW: u32 = 0x08;
}

/// NPU job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NpuPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// NPU job status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NpuJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// NPU inference job descriptor
#[derive(Debug, Clone)]
pub struct NpuJob {
    pub job_id: u32,
    pub model_id: ModelId,
    pub model_addr: u64,
    pub input_addr: u64,
    pub output_addr: u64,
    pub input_size: u32,
    pub output_size: u32,
    pub priority: NpuPriority,
    pub submit_time: u64,
    pub max_cycles: u64,
}

/// NPU job completion result
#[derive(Debug, Clone)]
pub struct NpuResult {
    pub job_id: u32,
    pub status: NpuJobStatus,
    pub cycles_used: u64,
    pub completion_time: u64,
    pub error_code: Option<u32>,
}

/// NPU device MMIO interface
pub struct NpuDevice {
    // Base address for MMIO registers
    #[allow(dead_code)]
    mmio_base: u64,
    
    // Current device state
    status: AtomicU32,
    command: AtomicU32,
    
    // Job management
    job_queue: VecDeque<NpuJob>,
    completion_queue: VecDeque<NpuResult>,
    current_job: Option<NpuJob>,
    next_job_id: AtomicU32,
    
    // Performance counters
    total_jobs: AtomicU64,
    completed_jobs: AtomicU64,
    failed_jobs: AtomicU64,
    total_cycles: AtomicU64,
    
    // Interrupt management
    irq_status: AtomicU32,
    irq_enable: AtomicU32,
    #[allow(dead_code)]
    irq_number: u32,
}

impl NpuDevice {
    /// Create new NPU device with specified MMIO base address
    pub fn new(mmio_base: u64, irq_number: u32) -> Self {
        Self {
            mmio_base,
            status: AtomicU32::new(status::NPU_STATUS_IDLE),
            command: AtomicU32::new(commands::NPU_CMD_NOP),
            job_queue: VecDeque::with_capacity(16),
            completion_queue: VecDeque::with_capacity(16),
            current_job: None,
            next_job_id: AtomicU32::new(1),
            total_jobs: AtomicU64::new(0),
            completed_jobs: AtomicU64::new(0),
            failed_jobs: AtomicU64::new(0),
            total_cycles: AtomicU64::new(0),
            irq_status: AtomicU32::new(0),
            irq_enable: AtomicU32::new(0),
            irq_number,
        }
    }
    
    /// Read from NPU MMIO register
    pub fn mmio_read(&self, offset: u64) -> u32 {
        match offset {
            registers::NPU_REG_STATUS => self.status.load(Ordering::Acquire),
            registers::NPU_REG_COMMAND => self.command.load(Ordering::Acquire),
            registers::NPU_REG_CYCLES => {
                self.current_job.as_ref()
                    .map(|job| self.estimate_job_cycles(job))
                    .unwrap_or(0) as u32
            }
            registers::NPU_REG_IRQ_STATUS => self.irq_status.load(Ordering::Acquire),
            registers::NPU_REG_IRQ_ENABLE => self.irq_enable.load(Ordering::Acquire),
            registers::NPU_REG_JOB_ID => {
                self.current_job.as_ref()
                    .map(|job| job.job_id)
                    .unwrap_or(0)
            }
            registers::NPU_REG_VERSION => 0x01000000, // Version 1.0.0.0
            _ => {
                metric_kv("npu_invalid_read", offset as usize);
                0xDEADBEEF // Invalid register
            }
        }
    }
    
    /// Write to NPU MMIO register
    pub fn mmio_write(&mut self, offset: u64, value: u32) {
        match offset {
            registers::NPU_REG_COMMAND => {
                self.command.store(value, Ordering::Release);
                self.handle_command(value);
            }
            registers::NPU_REG_MODEL_ADDR => {
                // Store model address for next job
                // In real implementation, would validate address
            }
            registers::NPU_REG_INPUT_ADDR => {
                // Store input buffer address
            }
            registers::NPU_REG_OUTPUT_ADDR => {
                // Store output buffer address
            }
            registers::NPU_REG_INPUT_SIZE => {
                // Store input size
            }
            registers::NPU_REG_OUTPUT_SIZE => {
                // Store output size
            }
            registers::NPU_REG_IRQ_ENABLE => {
                self.irq_enable.store(value, Ordering::Release);
            }
            registers::NPU_REG_PRIORITY => {
                // Set priority for next job
            }
            _ => {
                metric_kv("npu_invalid_write", offset as usize);
            }
        }
    }
    
    /// Handle NPU command execution
    fn handle_command(&mut self, command: u32) {
        match command {
            commands::NPU_CMD_NOP => {
                // No operation
            }
            commands::NPU_CMD_INFERENCE => {
                self.start_inference_job();
            }
            commands::NPU_CMD_RESET => {
                self.reset_device();
            }
            commands::NPU_CMD_QUEUE_FLUSH => {
                self.flush_job_queue();
            }
            commands::NPU_CMD_POWER_DOWN => {
                self.power_down();
            }
            commands::NPU_CMD_POWER_UP => {
                self.power_up();
            }
            _ => {
                // Invalid command
                self.set_error_status();
            }
        }
    }
    
    /// Submit inference job to NPU
    pub fn submit_inference_job(&mut self, job: NpuJob) -> Result<u32, MLError> {
        // Check queue capacity
        if self.job_queue.len() >= 16 {
            self.status.fetch_or(status::NPU_STATUS_QUEUE_FULL, Ordering::AcqRel);
            return Err(MLError::ArenaExhausted); // Reuse for queue full
        }
        
        // Validate job parameters
        if job.input_size == 0 || job.output_size == 0 {
            return Err(MLError::InvalidInput);
        }
        
        if job.model_addr == 0 || job.input_addr == 0 || job.output_addr == 0 {
            return Err(MLError::InvalidInput);
        }
        
        // Assign job ID
        let job_id = self.next_job_id.fetch_add(1, Ordering::AcqRel);
        let mut job_with_id = job;
        job_with_id.job_id = job_id;
        job_with_id.submit_time = self.get_current_time();
        
        // Add to priority queue
        self.insert_job_by_priority(job_with_id.clone());
        
        // Update statistics
        self.total_jobs.fetch_add(1, Ordering::Relaxed);
        
        // Update status
        if self.job_queue.len() == 1 && self.current_job.is_none() {
            self.status.store(status::NPU_STATUS_IDLE, Ordering::Release);
        }
        
        metric_kv("npu_jobs_submitted", 1);
        Ok(job_id)
    }
    
    /// Start processing next inference job
    fn start_inference_job(&mut self) {
        if self.current_job.is_some() {
            // Already processing a job
            return;
        }
        
        if let Some(job) = self.job_queue.pop_front() {
            self.status.store(status::NPU_STATUS_BUSY, Ordering::Release);
            self.current_job = Some(job.clone());
            
            // Simulate inference execution
            self.execute_inference_simulation(job);
        } else {
            self.status.store(status::NPU_STATUS_IDLE, Ordering::Release);
        }
    }
    
    /// Simulate NPU inference execution
    fn execute_inference_simulation(&mut self, job: NpuJob) {
        // Simulate realistic NPU execution time based on model complexity
        let estimated_cycles = self.estimate_job_cycles(&job);
        
        // Check if job exceeds budget
        if estimated_cycles > job.max_cycles {
            self.complete_job_with_error(job, 0xE001); // Budget exceeded
            return;
        }
        
        // Simulate successful execution
        let completion_time = self.get_current_time() + estimated_cycles;
        let result = NpuResult {
            job_id: job.job_id,
            status: NpuJobStatus::Completed,
            cycles_used: estimated_cycles,
            completion_time,
            error_code: None,
        };
        
        // Add to completion queue
        self.completion_queue.push_back(result);
        self.current_job = None;
        
        // Update statistics
        self.completed_jobs.fetch_add(1, Ordering::Relaxed);
        self.total_cycles.fetch_add(estimated_cycles, Ordering::Relaxed);
        
        // Set completion status and trigger interrupt
        self.status.store(status::NPU_STATUS_COMPLETE, Ordering::Release);
        self.trigger_completion_interrupt();
        
        // Start next job if available
        self.start_inference_job();
    }
    
    /// Estimate execution cycles for a job
    fn estimate_job_cycles(&self, job: &NpuJob) -> u64 {
        // Simulate realistic cycle estimation based on data size
        let base_cycles = 1000; // Base overhead
        let input_cycles = (job.input_size as u64) / 4; // ~1 cycle per 4 bytes
        let output_cycles = (job.output_size as u64) / 8; // Outputs are cheaper
        
        // Priority affects scheduling but not execution time
        let total_cycles = base_cycles + input_cycles + output_cycles;
        
        // Add some variability based on model complexity (simulated)
        let complexity_factor = match job.model_id.0 % 4 {
            0 => 1.0,   // Simple model
            1 => 1.5,   // Medium model
            2 => 2.0,   // Complex model
            3 => 3.0,   // Very complex model
            _ => 1.0,
        };
        
        (total_cycles as f64 * complexity_factor) as u64
    }
    
    /// Insert job into queue sorted by priority
    fn insert_job_by_priority(&mut self, job: NpuJob) {
        // Find insertion point for priority queue
        let mut insert_index = self.job_queue.len();
        
        for (i, existing_job) in self.job_queue.iter().enumerate() {
            if job.priority > existing_job.priority {
                insert_index = i;
                break;
            }
        }
        
        self.job_queue.insert(insert_index, job);
    }
    
    /// Complete job with error
    fn complete_job_with_error(&mut self, job: NpuJob, error_code: u32) {
        let result = NpuResult {
            job_id: job.job_id,
            status: NpuJobStatus::Failed,
            cycles_used: 0,
            completion_time: self.get_current_time(),
            error_code: Some(error_code),
        };
        
        self.completion_queue.push_back(result);
        self.current_job = None;
        self.failed_jobs.fetch_add(1, Ordering::Relaxed);
        
        self.status.store(status::NPU_STATUS_ERROR, Ordering::Release);
        self.trigger_error_interrupt();
    }
    
    /// Reset NPU device
    fn reset_device(&mut self) {
        self.job_queue.clear();
        self.completion_queue.clear();
        self.current_job = None;
        
        self.status.store(status::NPU_STATUS_IDLE, Ordering::Release);
        self.irq_status.store(0, Ordering::Release);
        
        metric_kv("npu_device_resets", 1);
    }
    
    /// Flush job queue
    fn flush_job_queue(&mut self) {
        let flushed_count = self.job_queue.len();
        self.job_queue.clear();
        
        if self.current_job.is_none() {
            self.status.store(status::NPU_STATUS_IDLE, Ordering::Release);
        }
        
        metric_kv("npu_jobs_flushed", flushed_count);
    }
    
    /// Power down NPU
    fn power_down(&mut self) {
        // Cancel current job if any
        if let Some(job) = self.current_job.take() {
            self.complete_job_with_error(job, 0xE002); // Power down
        }
        
        self.flush_job_queue();
        self.status.store(status::NPU_STATUS_IDLE, Ordering::Release);
        
        metric_kv("npu_power_down", 1);
    }
    
    /// Power up NPU
    fn power_up(&mut self) {
        self.status.store(status::NPU_STATUS_IDLE, Ordering::Release);
        metric_kv("npu_power_up", 1);
    }
    
    /// Set error status
    fn set_error_status(&mut self) {
        self.status.fetch_or(status::NPU_STATUS_ERROR, Ordering::AcqRel);
        self.trigger_error_interrupt();
    }
    
    /// Trigger completion interrupt
    fn trigger_completion_interrupt(&mut self) {
        if self.irq_enable.load(Ordering::Acquire) & interrupts::NPU_IRQ_COMPLETE != 0 {
            self.irq_status.fetch_or(interrupts::NPU_IRQ_COMPLETE, Ordering::AcqRel);
            self.send_interrupt();
        }
    }
    
    /// Trigger error interrupt
    fn trigger_error_interrupt(&mut self) {
        if self.irq_enable.load(Ordering::Acquire) & interrupts::NPU_IRQ_ERROR != 0 {
            self.irq_status.fetch_or(interrupts::NPU_IRQ_ERROR, Ordering::AcqRel);
            self.send_interrupt();
        }
    }
    
    /// Send interrupt to CPU
    fn send_interrupt(&self) {
        // In real implementation, would trigger GIC interrupt
        // For simulation, just emit metric
        metric_kv("npu_interrupts_sent", 1);
    }
    
    /// Handle interrupt acknowledgment
    pub fn handle_interrupt(&mut self) {
        let irq_status = self.irq_status.load(Ordering::Acquire);
        
        if irq_status & interrupts::NPU_IRQ_COMPLETE != 0 {
            // Job completed
            if let Some(_result) = self.completion_queue.pop_front() {
                metric_kv("npu_jobs_completed", 1);
                // Notify completion to higher level
            }
        }
        
        if irq_status & interrupts::NPU_IRQ_ERROR != 0 {
            // Error occurred
            metric_kv("npu_errors_handled", 1);
        }
        
        // Clear interrupt status
        self.irq_status.store(0, Ordering::Release);
    }
    
    /// Get current system time (simulation)
    fn get_current_time(&self) -> u64 {
        // In real implementation, would read system timer
        // For simulation, use cycle counter
        crate::inference::ArmPmu::read_cycle_counter()
    }
    
    /// Get device statistics
    pub fn get_statistics(&self) -> NpuStatistics {
        NpuStatistics {
            total_jobs: self.total_jobs.load(Ordering::Relaxed),
            completed_jobs: self.completed_jobs.load(Ordering::Relaxed),
            failed_jobs: self.failed_jobs.load(Ordering::Relaxed),
            total_cycles: self.total_cycles.load(Ordering::Relaxed),
            queue_depth: self.job_queue.len(),
            current_status: self.status.load(Ordering::Relaxed),
        }
    }
    
    /// Emit NPU performance metrics
    pub fn emit_metrics(&self) {
        let stats = self.get_statistics();
        
        metric_kv("npu_total_jobs", stats.total_jobs as usize);
        metric_kv("npu_completed_jobs", stats.completed_jobs as usize);
        metric_kv("npu_failed_jobs", stats.failed_jobs as usize);
        metric_kv("npu_total_cycles", stats.total_cycles as usize);
        metric_kv("npu_queue_depth", stats.queue_depth);
        metric_kv("npu_status", stats.current_status as usize);
        
        // Calculate utilization and throughput
        if stats.total_jobs > 0 {
            let success_rate = (stats.completed_jobs * 100) / stats.total_jobs;
            metric_kv("npu_success_rate_pct", success_rate as usize);
            
            if stats.completed_jobs > 0 {
                let avg_cycles = stats.total_cycles / stats.completed_jobs;
                metric_kv("npu_avg_cycles_per_job", avg_cycles as usize);
            }
        }
    }
}

/// NPU device statistics
#[derive(Debug, Clone)]
pub struct NpuStatistics {
    pub total_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub total_cycles: u64,
    pub queue_depth: usize,
    pub current_status: u32,
}

/// Global NPU device instance
pub static mut KERNEL_NPU_DEVICE: Option<NpuDevice> = None;

/// Initialize NPU device
pub fn init_npu_device(mmio_base: u64, irq_number: u32) {
    unsafe {
        KERNEL_NPU_DEVICE = Some(NpuDevice::new(mmio_base, irq_number));
    }
    metric_kv("npu_device_initialized", 1);
}

/// Get reference to NPU device
pub fn get_npu_device() -> Option<&'static mut NpuDevice> {
    unsafe { 
        let device_ptr = &raw mut KERNEL_NPU_DEVICE;
        (*device_ptr).as_mut()
    }
}

/// NPU demo function
pub fn npu_demo() {
    use crate::trace::trace;
    
    trace("NPU DEMO: Starting NPU device demonstration");
    
    // Initialize NPU device
    let npu_mmio_base = 0x1000_0000; // Simulated MMIO base
    let npu_irq = 42; // Simulated IRQ number
    
    init_npu_device(npu_mmio_base, npu_irq);
    
    if let Some(npu) = get_npu_device() {
        trace(&alloc::format!("NPU DEMO: Device initialized at MMIO 0x{:x}, IRQ {}", 
            npu_mmio_base, npu_irq));
        
        // Test MMIO reads
        let version = npu.mmio_read(registers::NPU_REG_VERSION);
        let status = npu.mmio_read(registers::NPU_REG_STATUS);
        
        trace(&alloc::format!("NPU DEMO: Version: 0x{:x}, Status: 0x{:x}", version, status));
        
        // Create and submit test jobs
        for i in 0..3 {
            let job = NpuJob {
                job_id: 0, // Will be assigned
                model_id: crate::ml::ModelId(i),
                model_addr: 0x2000_0000 + (i as u64 * 0x10000),
                input_addr: 0x3000_0000 + (i as u64 * 0x1000),
                output_addr: 0x4000_0000 + (i as u64 * 0x1000),
                input_size: 224 * 224 * 3 * 4, // Standard image input
                output_size: 1000 * 4, // Classification output
                priority: match i {
                    0 => NpuPriority::High,
                    1 => NpuPriority::Normal,
                    2 => NpuPriority::Low,
                    _ => NpuPriority::Normal,
                },
                submit_time: 0,
                max_cycles: 50_000, // 50k cycle budget
            };
            
            match npu.submit_inference_job(job) {
                Ok(job_id) => {
                    trace(&alloc::format!("NPU DEMO: Submitted job {} with ID {}", i, job_id));
                }
                Err(error) => {
                    trace(&alloc::format!("NPU DEMO: Failed to submit job {}: {:?}", i, error));
                }
            }
        }
        
        // Process jobs
        npu.mmio_write(registers::NPU_REG_COMMAND, commands::NPU_CMD_INFERENCE);
        
        // Simulate some processing time
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
        
        // Check completion
        let final_status = npu.mmio_read(registers::NPU_REG_STATUS);
        trace(&alloc::format!("NPU DEMO: Final status: 0x{:x}", final_status));
        
        // Handle interrupts
        npu.handle_interrupt();
        
        // Show statistics
        let stats = npu.get_statistics();
        trace(&alloc::format!("NPU DEMO: Statistics - Total: {}, Completed: {}, Failed: {}", 
            stats.total_jobs, stats.completed_jobs, stats.failed_jobs));
        trace(&alloc::format!("NPU DEMO: Total cycles: {}, Queue depth: {}", 
            stats.total_cycles, stats.queue_depth));
        
        // Emit metrics
        npu.emit_metrics();
    } else {
        trace("NPU DEMO: Failed to initialize NPU device");
    }
    
    trace("NPU DEMO: NPU device demonstration complete");
}