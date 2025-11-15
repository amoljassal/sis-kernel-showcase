use crate::ml::VerifiedMLModel;
use crate::npu::{NpuDevice, NpuJob, NpuPriority};
use crate::npu::registers::*;
use crate::interrupts::InterruptHandler;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

pub const NPU_IRQ_NUMBER: u32 = 33; // ARM Generic Timer IRQ + 1
pub const MAX_PENDING_JOBS: usize = 64;
pub const MAX_COMPLETED_JOBS: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NpuDriverStats {
    pub total_jobs_submitted: u64,
    pub total_jobs_completed: u64,
    pub total_jobs_failed: u64,
    pub current_pending_jobs: u32,
    pub average_completion_time_cycles: u64,
    pub peak_queue_depth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NpuDriverError {
    DeviceNotFound,
    QueueFull,
    InvalidModel,
    DmaError,
    TimeoutError,
    InterruptError,
    HardwareError,
}

pub struct NpuDriver {
    device: Mutex<Option<NpuDevice>>,
    mmio_base: u64,
    interrupt_enabled: AtomicBool,
    pending_jobs: Mutex<VecDeque<NpuDriverJob>>,
    completed_jobs: Mutex<VecDeque<NpuDriverResult>>,
    stats: Mutex<NpuDriverStats>,
    next_job_id: AtomicU32,
}

#[derive(Debug, Clone)]
pub struct NpuDriverJob {
    pub job_id: u32,
    pub model_id: u32,
    pub input_buffer: Vec<f32>,
    pub output_size: usize,
    pub priority: NpuPriority,
    pub submitted_at: u64,
}

#[derive(Debug, Clone)]
pub struct NpuDriverResult {
    pub job_id: u32,
    pub output_buffer: Vec<f32>,
    pub completion_time_cycles: u64,
    pub peak_memory_usage: usize,
    pub success: bool,
    pub error: Option<NpuDriverError>,
}

impl NpuDriver {
    pub const fn new(mmio_base: u64) -> Self {
        Self {
            device: Mutex::new(None),
            mmio_base,
            interrupt_enabled: AtomicBool::new(false),
            pending_jobs: Mutex::new(VecDeque::new()),
            completed_jobs: Mutex::new(VecDeque::new()),
            stats: Mutex::new(NpuDriverStats {
                total_jobs_submitted: 0,
                total_jobs_completed: 0,
                total_jobs_failed: 0,
                current_pending_jobs: 0,
                average_completion_time_cycles: 0,
                peak_queue_depth: 0,
            }),
            next_job_id: AtomicU32::new(1),
        }
    }

    pub fn initialize(&self) -> Result<(), NpuDriverError> {
        // Initialize NPU device
        let npu_device = NpuDevice::new(self.mmio_base, NPU_IRQ_NUMBER);
        *self.device.lock() = Some(npu_device);

        // Reset NPU device
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_COMMAND as u64) as *mut u32, 0x1); // Reset
        }

        // Wait for reset completion
        let mut timeout = 1000;
        while timeout > 0 {
            let status = unsafe {
                ptr::read_volatile((self.mmio_base + NPU_REG_STATUS as u64) as *const u32)
            };
            if status & 0x1 == 0 { // Reset complete
                break;
            }
            timeout -= 1;
        }

        if timeout == 0 {
            return Err(NpuDriverError::TimeoutError);
        }

        // Enable NPU device
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_COMMAND as u64) as *mut u32, 0x2); // Enable
        }

        // Enable interrupts
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_IRQ_ENABLE as u64) as *mut u32, 0x1);
        }

        self.interrupt_enabled.store(true, Ordering::SeqCst);

        // Register interrupt handler
        self.register_interrupt_handler()?;

        Ok(())
    }

    pub fn submit_inference_job(
        &self,
        model: &VerifiedMLModel,
        input: &[f32],
        output_size: usize,
        priority: NpuPriority,
    ) -> Result<u32, NpuDriverError> {
        let mut pending = self.pending_jobs.lock();
        
        if pending.len() >= MAX_PENDING_JOBS {
            return Err(NpuDriverError::QueueFull);
        }

        let job_id = self.next_job_id.fetch_add(1, Ordering::SeqCst);
        
        let driver_job = NpuDriverJob {
            job_id,
            model_id: model.id.0,
            input_buffer: input.to_vec(),
            output_size,
            priority,
            submitted_at: self.read_cycle_counter(),
        };

        pending.push_back(driver_job.clone());

        // Update stats
        {
            let mut stats = self.stats.lock();
            stats.total_jobs_submitted += 1;
            stats.current_pending_jobs = pending.len() as u32;
            if pending.len() as u32 > stats.peak_queue_depth {
                stats.peak_queue_depth = pending.len() as u32;
            }
        }

        // Try to submit job to hardware immediately
        self.try_submit_next_job()?;

        Ok(job_id)
    }

    fn try_submit_next_job(&self) -> Result<(), NpuDriverError> {
        let mut pending = self.pending_jobs.lock();
        
        if pending.is_empty() {
            return Ok(());
        }

        // Check if NPU is ready for new job
        let status = unsafe {
            ptr::read_volatile((self.mmio_base + NPU_REG_STATUS as u64) as *const u32)
        };

        if status & 0x4 != 0 { // NPU busy
            return Ok(());
        }

        let driver_job = pending.pop_front().unwrap();

        // Setup DMA for input data
        self.setup_input_dma(&driver_job)?;

        // Submit job to NPU
        let npu_job = NpuJob {
            job_id: driver_job.job_id,
            model_id: crate::ml::ModelId(driver_job.model_id),
            model_addr: 0x50000000, // Placeholder model address
            input_addr: self.mmio_base + 0x10000, // Input buffer offset
            output_addr: self.mmio_base + 0x20000, // Output buffer offset
            input_size: driver_job.input_buffer.len() as u32,
            output_size: driver_job.output_size as u32,
            priority: driver_job.priority,
            submit_time: driver_job.submitted_at,
            max_cycles: 1000000, // 1M cycle budget
        };

        if let Some(ref mut device) = *self.device.lock() {
            let _ = device.submit_inference_job(npu_job);
        }

        // Trigger NPU execution
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_COMMAND as u64) as *mut u32, 0x4); // Start
        }

        Ok(())
    }

    fn setup_input_dma(&self, job: &NpuDriverJob) -> Result<(), NpuDriverError> {
        // In a real implementation, this would setup DMA transfer
        // For simulation, we write directly to NPU memory
        let input_addr = self.mmio_base + 0x10000; // Input buffer offset
        
        for (i, &value) in job.input_buffer.iter().enumerate() {
            unsafe {
                let addr = (input_addr + (i * 4) as u64) as *mut f32;
                ptr::write_volatile(addr, value);
            }
        }

        // Set input buffer address and size
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_INPUT_ADDR as u64) as *mut u64, input_addr);
            ptr::write_volatile((self.mmio_base + NPU_REG_INPUT_SIZE as u64) as *mut u32, 
                               job.input_buffer.len() as u32);
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn setup_output_dma(&self, output_size: usize) -> Result<u64, NpuDriverError> {
        let output_addr = self.mmio_base + 0x20000; // Output buffer offset

        // Set output buffer address and size
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_OUTPUT_ADDR as u64) as *mut u64, output_addr);
            ptr::write_volatile((self.mmio_base + NPU_REG_OUTPUT_SIZE as u64) as *mut u32, 
                               output_size as u32);
        }

        Ok(output_addr)
    }

    pub fn poll_completed_jobs(&self) -> Vec<NpuDriverResult> {
        let mut completed = self.completed_jobs.lock();
        let mut results = Vec::new();

        while let Some(result) = completed.pop_front() {
            results.push(result);
        }

        results
    }

    pub fn get_job_result(&self, job_id: u32) -> Option<NpuDriverResult> {
        let mut completed = self.completed_jobs.lock();
        
        if let Some(pos) = completed.iter().position(|r| r.job_id == job_id) {
            Some(completed.remove(pos).unwrap())
        } else {
            None
        }
    }

    pub fn get_stats(&self) -> NpuDriverStats {
        *self.stats.lock()
    }

    fn register_interrupt_handler(&self) -> Result<(), NpuDriverError> {
        // In a real kernel, this would register with the interrupt controller
        // For simulation, we'll handle this in the polling mechanism
        Ok(())
    }

    pub fn handle_interrupt(&self) {
        if !self.interrupt_enabled.load(Ordering::SeqCst) {
            return;
        }

        // Read interrupt status
        let irq_status = unsafe {
            ptr::read_volatile((self.mmio_base + NPU_REG_IRQ_STATUS as u64) as *const u32)
        };

        if irq_status & 0x1 != 0 { // Job completion interrupt
            self.handle_job_completion();
        }

        if irq_status & 0x2 != 0 { // Error interrupt
            self.handle_error_interrupt();
        }

        // Clear interrupt status
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_IRQ_STATUS as u64) as *mut u32, irq_status);
        }

        // Try to submit next job
        let _ = self.try_submit_next_job();
    }

    fn handle_job_completion(&self) {
        // Read job result from NPU
        let job_id = unsafe {
            ptr::read_volatile((self.mmio_base + NPU_REG_JOB_ID as u64) as *const u32)
        };

        let cycles_taken = unsafe {
            ptr::read_volatile((self.mmio_base + NPU_REG_CYCLES as u64) as *const u64)
        };

        let output_size = unsafe {
            ptr::read_volatile((self.mmio_base + NPU_REG_OUTPUT_SIZE as u64) as *const u32)
        } as usize;

        let memory_usage = unsafe {
            ptr::read_volatile((self.mmio_base + 0x44) as *const u32) // Memory usage register
        } as usize;

        // Read output data
        let output_addr = self.mmio_base + 0x20000; // Output buffer offset
        let mut output_buffer = Vec::with_capacity(output_size);
        
        for i in 0..output_size {
            let value = unsafe {
                let addr = (output_addr + (i * 4) as u64) as *const f32;
                ptr::read_volatile(addr)
            };
            output_buffer.push(value);
        }

        let result = NpuDriverResult {
            job_id,
            output_buffer,
            completion_time_cycles: cycles_taken,
            peak_memory_usage: memory_usage,
            success: true,
            error: None,
        };

        // Add to completed jobs queue
        {
            let mut completed = self.completed_jobs.lock();
            if completed.len() < MAX_COMPLETED_JOBS {
                completed.push_back(result);
            }
        }

        // Update stats
        {
            let mut stats = self.stats.lock();
            stats.total_jobs_completed += 1;
            stats.current_pending_jobs = stats.current_pending_jobs.saturating_sub(1);
            
            // Update rolling average completion time
            let total_completed = stats.total_jobs_completed;
            let current_avg = stats.average_completion_time_cycles;
            stats.average_completion_time_cycles = 
                (current_avg * (total_completed - 1) + cycles_taken) / total_completed;
        }
    }

    fn handle_error_interrupt(&self) {
        let error_code = unsafe {
            ptr::read_volatile((self.mmio_base + 0x40) as *const u32) // Error code register
        };

        let job_id = unsafe {
            ptr::read_volatile((self.mmio_base + NPU_REG_JOB_ID as u64) as *const u32)
        };

        let error = match error_code {
            1 => NpuDriverError::InvalidModel,
            2 => NpuDriverError::DmaError,
            3 => NpuDriverError::TimeoutError,
            _ => NpuDriverError::HardwareError,
        };

        let result = NpuDriverResult {
            job_id,
            output_buffer: Vec::new(),
            completion_time_cycles: 0,
            peak_memory_usage: 0,
            success: false,
            error: Some(error),
        };

        // Add to completed jobs queue
        {
            let mut completed = self.completed_jobs.lock();
            if completed.len() < MAX_COMPLETED_JOBS {
                completed.push_back(result);
            }
        }

        // Update stats
        {
            let mut stats = self.stats.lock();
            stats.total_jobs_failed += 1;
            stats.current_pending_jobs = stats.current_pending_jobs.saturating_sub(1);
        }
    }

    fn read_cycle_counter(&self) -> u64 {
        // Read ARM PMU cycle counter
        let mut cycles: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, pmccntr_el0",
                out(reg) cycles,
                options(nostack, nomem)
            );
        }
        cycles
    }

    pub fn shutdown(&self) -> Result<(), NpuDriverError> {
        // Disable interrupts
        self.interrupt_enabled.store(false, Ordering::SeqCst);
        
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_IRQ_ENABLE as u64) as *mut u32, 0x0);
        }

        // Disable NPU device
        unsafe {
            ptr::write_volatile((self.mmio_base + NPU_REG_COMMAND as u64) as *mut u32, 0x0);
        }

        // Clear device reference
        *self.device.lock() = None;

        Ok(())
    }
}

// Global NPU driver instance
pub static NPU_DRIVER: NpuDriver = NpuDriver::new(0x40000000); // NPU MMIO base address

impl InterruptHandler for NpuDriver {
    fn handle_interrupt(&self, _irq_number: u32) {
        self.handle_interrupt();
    }
}

pub fn initialize_npu_driver() -> Result<(), NpuDriverError> {
    NPU_DRIVER.initialize()
}

pub fn submit_ai_inference(
    model: &VerifiedMLModel,
    input: &[f32],
    output_size: usize,
    priority: NpuPriority,
) -> Result<u32, NpuDriverError> {
    NPU_DRIVER.submit_inference_job(model, input, output_size, priority)
}

pub fn get_inference_result(job_id: u32) -> Option<NpuDriverResult> {
    NPU_DRIVER.get_job_result(job_id)
}

pub fn poll_inference_results() -> Vec<NpuDriverResult> {
    NPU_DRIVER.poll_completed_jobs()
}

pub fn get_npu_stats() -> NpuDriverStats {
    NPU_DRIVER.get_stats()
}