//! Phase 7: OpenTelemetry Integration and Drift Detection
//!
//! This module provides OpenTelemetry export and drift detection:
//! - OTelExporter: Export decision traces as OTel spans
//! - DriftMonitor: Detect model drift from baseline
//! - Automatic safe mode on excessive drift

#[cfg(feature = "otel")]
pub mod exporter;

#[cfg(feature = "otel")]
pub mod drift;

#[cfg(feature = "otel")]
pub use exporter::{OTelExporter, OTelSpan, SpanKind, SpanStatus};

#[cfg(feature = "otel")]
pub use drift::{DriftMonitor, DriftStatus};
