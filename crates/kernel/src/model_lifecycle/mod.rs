//! Phase 7: Model Registry and Lifecycle Management
//!
//! This module provides infrastructure for managing AI model lifecycle,
//! including loading, verification, hot-swapping, and rollback capabilities.
//!
//! Key features:
//! - Model registry backed by ext4 filesystem
//! - Atomic model hot-swap with RCU semantics
//! - SHA-256 + Ed25519 signature verification
//! - Health checks (latency, memory, accuracy)
//! - Rollback to last-known-good model

#[cfg(feature = "model-lifecycle")]
pub mod registry;

#[cfg(feature = "model-lifecycle")]
pub mod lifecycle;

#[cfg(feature = "model-lifecycle")]
pub mod health;

#[cfg(feature = "model-lifecycle")]
pub use registry::{ModelRegistry, ModelMetadata, ModelStatus, HealthMetrics};

#[cfg(feature = "model-lifecycle")]
pub use lifecycle::{ModelLifecycle, Model};

#[cfg(feature = "model-lifecycle")]
pub use health::HealthChecker;
