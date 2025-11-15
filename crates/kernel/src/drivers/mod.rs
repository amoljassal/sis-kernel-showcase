/// Device drivers

// M8: Driver hardening infrastructure
pub mod timeout;   // Timeout utilities for hardware operations
pub mod error;     // Common driver error types

pub mod char;
pub mod block;  // Block device drivers (SDHCI, etc.) - M1
pub mod virtio_blk;
pub mod virtio_net;
pub mod virtio_gpu;
pub mod watchdog;  // Watchdog timer driver - M2
pub mod gpio;      // GPIO drivers - M6
pub mod firmware;  // Firmware interface (mailbox) - M6

// Phase 6 - Production Readiness: Mock drivers and trait abstractions
pub mod traits;
#[cfg(feature = "mock-devices")]
pub mod mock;

// Re-export common types for convenience
pub use error::{DriverError, DriverResult, Validator};
pub use timeout::{Timeout, TimeoutError};
