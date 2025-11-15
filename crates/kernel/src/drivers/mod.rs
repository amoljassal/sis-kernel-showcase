/// Device drivers

pub mod char;
pub mod block;  // Block device drivers (SDHCI, etc.) - M1
pub mod virtio_blk;
pub mod virtio_net;
pub mod virtio_gpu;

// Phase 6 - Production Readiness: Mock drivers and trait abstractions
pub mod traits;
#[cfg(feature = "mock-devices")]
pub mod mock;
