//! Custom tracing layer for emitting LogLine WebSocket events

use crate::qemu::{QemuEvent, QemuSupervisor};
use std::sync::Arc;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

/// Custom tracing layer that emits LogLine events to WebSocket subscribers
pub struct WebSocketLayer {
    supervisor: Arc<QemuSupervisor>,
}

impl WebSocketLayer {
    /// Create a new WebSocket tracing layer
    pub fn new(supervisor: Arc<QemuSupervisor>) -> Self {
        Self { supervisor }
    }
}

impl<S> Layer<S> for WebSocketLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Convert tracing level to string
        let level = match *event.metadata().level() {
            Level::ERROR => "error",
            Level::WARN => "warn",
            Level::INFO => "info",
            Level::DEBUG => "debug",
            Level::TRACE => "debug", // Map TRACE to debug
        };

        // Extract message from event
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        if let Some(msg) = visitor.message {
            // Determine source based on target
            let source = if event.metadata().target().contains("qemu") {
                "qemu"
            } else if event.metadata().target().contains("kernel") {
                "kernel"
            } else {
                "daemon"
            };

            // Emit LogLine event
            let event = QemuEvent::LogLine {
                level: level.to_string(),
                source: source.to_string(),
                msg,
                ts: chrono::Utc::now().timestamp_millis(),
                request_id: None,
            };

            self.supervisor.broadcast_event(event);
        }
    }
}

/// Visitor to extract message from event
#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        }
    }
}
