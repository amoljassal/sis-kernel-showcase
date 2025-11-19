// Hardware backend implementations for SIS Kernel testing

pub mod qemu_backend;
pub mod serial_backend;
pub mod mock_npu_backend;

// Re-export for convenience
pub use qemu_backend::QemuBackend;
pub use serial_backend::SerialBackend;
pub use mock_npu_backend::MockNpuBackend;
