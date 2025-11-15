//! Lightweight tracing and METRIC emission utilities.
//! Designed for no_std, low-overhead serial output.
//!
//! Runtime metrics capture: disabled by default, enabled via metricsctl command.

use core::sync::atomic::{AtomicBool, Ordering};

/// Runtime toggle for metric emission (on by default for visibility)
static METRICS_ENABLED: AtomicBool = AtomicBool::new(true);

/// Enable or disable metric emission at runtime
pub fn metrics_set_enabled(enabled: bool) {
    METRICS_ENABLED.store(enabled, Ordering::Release);
}

/// Check if metrics emission is currently enabled
pub fn metrics_enabled() -> bool {
    METRICS_ENABLED.load(Ordering::Acquire)
}

#[inline(always)]
pub fn metric_kv(name: &str, value: usize) {
    // Conditionally print to UART based on runtime toggle
    if METRICS_ENABLED.load(Ordering::Relaxed) {
        unsafe {
            crate::uart_print(b"METRIC ");
            print_str(name);
            crate::uart_print(b"=");
            print_usize(value);
            crate::uart_print(b"\n");
        }
    }
}

#[inline(always)]
pub fn trace(tag: &str) {
    unsafe {
        crate::uart_print(b"[TRACE] ");
        print_str(tag);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub fn op_start(op_id: u32) {
    unsafe {
        crate::uart_print(b"[TRACE] op_start id=");
        print_usize(op_id as usize);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub fn op_queued(op_id: u32) {
    unsafe {
        crate::uart_print(b"[TRACE] op_queued id=");
        print_usize(op_id as usize);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub fn op_end_ns(op_id: u32, ns: u64) {
    unsafe {
        crate::uart_print(b"[TRACE] op_end id=");
        print_usize(op_id as usize);
        crate::uart_print(b" ns=");
        print_usize(ns as usize);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub fn ch_depth(ch_id: usize, depth: usize) {
    unsafe {
        crate::uart_print(b"[TRACE] ch_depth id=");
        print_usize(ch_id);
        crate::uart_print(b" depth=");
        print_usize(depth);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub unsafe fn print_str(s: &str) {
    crate::uart_print(s.as_bytes());
}

#[inline(always)]
pub unsafe fn print_usize(mut num: usize) {
    if num == 0 {
        crate::uart_print(b"0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while num > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        crate::uart_print(&[buf[i]]);
    }
}

/// Snapshot recent ctx_switch_ns values (stub: always returns 0)
pub fn metrics_snapshot_ctx_switch(_out: &mut [usize]) -> usize {
    0
}

/// Snapshot recent memory_alloc_ns values (stub: always returns 0)
pub fn metrics_snapshot_memory_alloc(_out: &mut [usize]) -> usize {
    0
}

/// Snapshot recent real_ctx_switch_ns values (stub: always returns 0)
pub fn metrics_snapshot_real_ctx(_out: &mut [usize]) -> usize {
    0
}

// ========== Decision Trace + OTel Integration ==========

/// Telemetry snapshot at decision time
pub struct DecisionTelemetry {
    pub mem_pressure: u32,
    pub deadline_misses: u32,
}

impl Default for DecisionTelemetry {
    fn default() -> Self {
        Self {
            mem_pressure: 0,
            deadline_misses: 0,
        }
    }
}

/// Policy check result
pub struct PolicyCheck {
    pub check_name: alloc::string::String,
    pub passed: bool,
    pub value: f32,
}

/// Simplified decision trace for autonomous system decisions
pub struct DecisionTrace {
    pub trace_id: u64,
    pub timestamp_us: u64,
    pub model_version: alloc::string::String,
    pub chosen_action: u32,
    pub confidence: u32,
    pub was_executed: bool,
    pub override_reason: Option<alloc::string::String>,
    pub telemetry: DecisionTelemetry,
    pub policy_checks: alloc::vec::Vec<PolicyCheck>,
}

impl DecisionTrace {
    /// Create a new decision trace
    pub fn new(action: u32, confidence: u32) -> Self {
        Self {
            trace_id: crate::time::get_uptime_ms(),  // Use timestamp as trace ID
            timestamp_us: crate::time::current_time_us(),
            model_version: alloc::string::String::from("v1.0"),
            chosen_action: action,
            confidence,
            was_executed: true,
            override_reason: None,
            telemetry: DecisionTelemetry::default(),
            policy_checks: alloc::vec::Vec::new(),
        }
    }

    /// Mark as overridden with reason
    pub fn override_with(mut self, reason: &str) -> Self {
        self.was_executed = false;
        self.override_reason = Some(alloc::string::String::from(reason));
        self
    }

    /// Add policy check result
    pub fn add_policy_check(mut self, name: &str, passed: bool, value: f32) -> Self {
        self.policy_checks.push(PolicyCheck {
            check_name: alloc::string::String::from(name),
            passed,
            value,
        });
        self
    }

    /// Set telemetry data
    pub fn with_telemetry(mut self, mem_pressure: u32, deadline_misses: u32) -> Self {
        self.telemetry = DecisionTelemetry {
            mem_pressure,
            deadline_misses,
        };
        self
    }
}

/// Log a decision trace and export to OTel if feature enabled
pub fn log_decision_trace(trace: DecisionTrace) {
    // Log to UART
    unsafe {
        crate::uart_print(b"[DECISION] id=");
        print_usize(trace.trace_id as usize);
        crate::uart_print(b" action=");
        print_usize(trace.chosen_action as usize);
        crate::uart_print(b" conf=");
        print_usize(trace.confidence as usize);
        crate::uart_print(b" exec=");
        crate::uart_print(if trace.was_executed { b"yes" } else { b"no" });
        crate::uart_print(b"\n");
    }

    // Export to OTel if decision-traces feature enabled
    #[cfg(feature = "decision-traces")]
    {
        crate::otel::exporter::OTEL_EXPORTER.record_decision_span(&trace);
    }

    // Write to decision log file
    write_decision_log(&trace);
}

/// Write decision trace to VFS log file
fn write_decision_log(trace: &DecisionTrace) {
    use alloc::format;

    let mut json = alloc::string::String::from("{");
    json.push_str(&format!("\"trace_id\":{},", trace.trace_id));
    json.push_str(&format!("\"timestamp_us\":{},", trace.timestamp_us));
    json.push_str(&format!("\"model\":\"{}\"", trace.model_version));
    json.push_str(&format!(",\"action\":{}", trace.chosen_action));
    json.push_str(&format!(",\"confidence\":{}", trace.confidence));
    json.push_str(&format!(",\"executed\":{}", trace.was_executed));

    if let Some(reason) = &trace.override_reason {
        json.push_str(&format!(",\"override\":\"{}\"", reason));
    }

    json.push_str("}\n");

    // Append to decision log
    let _ = crate::vfs::mkdir("/var", 0o755);
    let _ = crate::vfs::mkdir("/var/log", 0o755);

    if let Ok(fd) = crate::vfs::open(
        "/var/log/decisions.json",
        crate::vfs::OpenFlags::O_WRONLY | crate::vfs::OpenFlags::O_APPEND | crate::vfs::OpenFlags::O_CREAT
    ) {
        let _ = fd.write(json.as_bytes());
    }
}

/// Cross-check decision trace with policy
pub fn verify_decision_policy(trace: &DecisionTrace, agent_id: u32) -> bool {
    use crate::agent_sys::policy;
    use crate::security::agent_policy::{Capability, Resource, PolicyDecision};

    // Check if agent has permission for the action
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,  // Placeholder - should map action to capability
        &Resource::NoResource,
    );

    matches!(decision, PolicyDecision::Allow { .. })
}
