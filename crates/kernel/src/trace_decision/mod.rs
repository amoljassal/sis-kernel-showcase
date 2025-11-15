//! Phase 7: Decision Trace Infrastructure
//!
//! This module provides comprehensive decision trace recording and export
//! for AI/ML decision observability and incident investigation.
//!
//! Key features:
//! - DecisionTrace: Captures full decision context
//! - TraceBuffer: Ring buffer for recent traces
//! - IncidentExporter: Export bundles for forensics

#[cfg(feature = "decision-traces")]
pub mod decision;

#[cfg(feature = "decision-traces")]
pub mod buffer;

#[cfg(feature = "decision-traces")]
pub mod export;

#[cfg(feature = "decision-traces")]
pub use decision::{DecisionTrace, Telemetry, SystemState, PolicyCheck, Alternative};

#[cfg(feature = "decision-traces")]
pub use buffer::TraceBuffer;

#[cfg(feature = "decision-traces")]
pub use export::IncidentExporter;
