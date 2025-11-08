//! OpenTelemetry Exporter
//!
//! Exports decision traces as OpenTelemetry spans for
//! integration with observability platforms.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Serialize, Deserialize};
use spin::Mutex;
use crate::lib::error::Result;

#[cfg(feature = "decision-traces")]
use crate::trace_decision::DecisionTrace;

/// OpenTelemetry span
#[derive(Debug, Serialize, Deserialize)]
pub struct OTelSpan {
    pub trace_id: String,           // Hex trace ID
    pub span_id: String,            // Hex span ID
    pub parent_span_id: Option<String>,
    pub name: String,               // "autonomous_decision"
    pub kind: SpanKind,
    pub start_time_us: u64,
    pub end_time_us: u64,
    pub attributes: Vec<Attribute>,
    pub events: Vec<Event>,
    pub status: SpanStatus,
}

/// Span kind
#[derive(Debug, Serialize, Deserialize)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
}

/// Span attribute
#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeValue,
}

/// Attribute value types
#[derive(Debug, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Span event
#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub timestamp_us: u64,
    pub name: String,
    pub attributes: Vec<Attribute>,
}

/// Span status
#[derive(Debug, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error { message: String },
}

/// OpenTelemetry exporter
pub struct OTelExporter {
    export_endpoint: &'static str,
    batch_size: usize,
    spans: Mutex<Vec<OTelSpan>>,
}

impl OTelExporter {
    /// Create new OTel exporter
    pub const fn new() -> Self {
        Self {
            export_endpoint: "/otel/spans.json",
            batch_size: 100,
            spans: Mutex::new(Vec::new()),
        }
    }

    /// Record decision trace as OTel span
    #[cfg(feature = "decision-traces")]
    pub fn record_decision_span(&self, trace: &DecisionTrace) {
        let span = OTelSpan {
            trace_id: alloc::format!("{:016x}", trace.trace_id),
            span_id: alloc::format!("{:016x}", trace.trace_id),
            parent_span_id: None,
            name: String::from("autonomous_decision"),
            kind: SpanKind::Internal,
            start_time_us: trace.timestamp_us,
            end_time_us: trace.timestamp_us + 1000,  // Assume 1ms duration
            attributes: self.build_attributes(trace),
            events: self.build_events(trace),
            status: if trace.was_executed {
                SpanStatus::Ok
            } else {
                SpanStatus::Error {
                    message: trace.override_reason.clone()
                        .unwrap_or_else(|| String::from("Not executed"))
                }
            },
        };

        let mut spans = self.spans.lock();
        spans.push(span);

        // Export batch if full
        if spans.len() >= self.batch_size {
            let _ = self.flush_batch(&mut spans);
        }
    }

    #[cfg(feature = "decision-traces")]
    fn build_attributes(&self, trace: &DecisionTrace) -> Vec<Attribute> {
        alloc::vec![
            Attribute {
                key: String::from("model.version"),
                value: AttributeValue::String(trace.model_version.clone()),
            },
            Attribute {
                key: String::from("action"),
                value: AttributeValue::Int(trace.chosen_action as i64),
            },
            Attribute {
                key: String::from("confidence"),
                value: AttributeValue::Int(trace.confidence as i64),
            },
            Attribute {
                key: String::from("mem_pressure"),
                value: AttributeValue::Int(trace.telemetry.mem_pressure as i64),
            },
            Attribute {
                key: String::from("deadline_misses"),
                value: AttributeValue::Int(trace.telemetry.deadline_misses as i64),
            },
        ]
    }

    #[cfg(feature = "decision-traces")]
    fn build_events(&self, trace: &DecisionTrace) -> Vec<Event> {
        let mut events = Vec::new();

        // Policy check events
        for check in &trace.policy_checks {
            events.push(Event {
                timestamp_us: trace.timestamp_us,
                name: alloc::format!("policy_check.{}", check.check_name),
                attributes: alloc::vec![
                    Attribute {
                        key: String::from("passed"),
                        value: AttributeValue::Bool(check.passed),
                    },
                    Attribute {
                        key: String::from("value"),
                        value: AttributeValue::Float(check.value as f64),
                    },
                ],
            });
        }

        events
    }

    fn flush_batch(&self, spans: &mut Vec<OTelSpan>) -> Result<()> {
        // Serialize to JSON
        let json = serde_json::to_string(&spans)
            .map_err(|_| crate::lib::error::Errno::EINVAL)?;

        // TODO: Write to ext4 via VFS
        let _ = self.write_file(self.export_endpoint, json.as_bytes());

        // Clear batch
        spans.clear();

        Ok(())
    }

    fn write_file(&self, _path: &str, _data: &[u8]) -> Result<()> {
        // TODO: Implement VFS write
        Ok(())
    }

    /// Force flush all pending spans
    pub fn flush(&self) -> Result<()> {
        let mut spans = self.spans.lock();
        if !spans.is_empty() {
            self.flush_batch(&mut spans)?;
        }
        Ok(())
    }

    /// Get pending span count
    pub fn pending_count(&self) -> usize {
        self.spans.lock().len()
    }
}

/// Global OTel exporter instance
pub static OTEL_EXPORTER: OTelExporter = OTelExporter::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otel_exporter() {
        let exporter = OTelExporter::new();
        assert_eq!(exporter.pending_count(), 0);
    }
}
