//! Kernel Tests and Benchmarks
//!
//! This module contains test suites and performance benchmarks for various
//! kernel subsystems.

#[cfg(feature = "benchmarks")]
pub mod slab_bench;

#[cfg(feature = "benchmarks")]
pub mod virtio_bench;
