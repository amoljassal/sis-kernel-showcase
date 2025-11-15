//! AgentSys Audit Logger
//!
//! Records all AgentSys operations for security auditing and compliance.
//! Uses circular buffer to prevent unbounded memory growth.

use crate::agent_sys::AgentId;
use crate::uart;
use crate::trace::metric_kv;

const AUDIT_BUFFER_SIZE: usize = 200;

/// Audit record
#[derive(Copy, Clone, Debug)]
pub struct AuditRecord {
    pub agent_id: AgentId,
    pub opcode: u8,
    pub timestamp_us: u64,
    pub allowed: bool,
}

/// Circular audit logger
pub struct AuditLogger {
    buffer: [Option<AuditRecord>; AUDIT_BUFFER_SIZE],
    write_pos: usize,
    total_ops: u64,
}

impl AuditLogger {
    pub fn new() -> Self {
        AuditLogger {
            buffer: [None; AUDIT_BUFFER_SIZE],
            write_pos: 0,
            total_ops: 0,
        }
    }

    /// Log an operation
    pub fn log_operation(&mut self, agent_id: AgentId, opcode: u8, allowed: bool) {
        let timestamp_us = crate::time::get_timestamp_us();

        let record = AuditRecord {
            agent_id,
            opcode,
            timestamp_us,
            allowed,
        };

        // Write to circular buffer
        self.buffer[self.write_pos] = Some(record);
        self.write_pos = (self.write_pos + 1) % AUDIT_BUFFER_SIZE;
        self.total_ops += 1;

        // Emit to serial (for test harness parsing)
        let result_str = if allowed { "ALLOW" } else { "DENY" };
        uart::print_str("[AUDIT] agent=");
        uart::print_u32(agent_id);
        uart::print_str(" op=0x");
        uart::print_hex8(opcode);
        uart::print_str(" result=");
        uart::print_str(result_str);
        uart::print_str("\n");

        // Emit metric
        metric_kv("agentsys_audit_events", 1);
        if !allowed {
            metric_kv("agentsys_denies_total", 1);
        }
    }

    /// Get total operations audited
    pub fn total_operations(&self) -> u64 {
        self.total_ops
    }

    /// Dump recent audit records (for debugging)
    pub fn dump_recent(&self, count: usize) {
        uart::print_str("[AUDIT] Recent operations:\n");
        let mut shown = 0;
        let mut pos = if self.write_pos == 0 { AUDIT_BUFFER_SIZE - 1 } else { self.write_pos - 1 };

        while shown < count && shown < AUDIT_BUFFER_SIZE {
            if let Some(record) = self.buffer[pos] {
                uart::print_str("  agent=");
                uart::print_u32(record.agent_id);
                uart::print_str(" op=0x");
                uart::print_hex8(record.opcode);
                uart::print_str(" ts=");
                uart::print_u64(record.timestamp_us);
                uart::print_str(" allowed=");
                if record.allowed { uart::print_str("true"); } else { uart::print_str("false"); }
                uart::print_str("\n");
                shown += 1;
            }
            if pos == 0 {
                pos = AUDIT_BUFFER_SIZE - 1;
            } else {
                pos -= 1;
            }
        }
    }
}
