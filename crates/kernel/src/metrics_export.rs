// Metrics Export Module
// Phase 1.3 - Production Readiness Plan
//
// Provides structured metrics export in JSON and Prometheus formats
// for observability and monitoring

use alloc::string::String;
use alloc::format;

/// Metrics snapshot for export
pub struct MetricsSnapshot {
    // Context switching metrics
    pub ctx_switch_p50_ns: u64,
    pub ctx_switch_p95_ns: u64,
    pub ctx_switch_p99_ns: u64,

    // Memory metrics
    pub heap_allocs: u64,
    pub heap_deallocs: u64,
    pub heap_current_bytes: u64,
    pub heap_peak_bytes: u64,
    pub heap_failures: u64,

    // System metrics
    pub panic_count: u64,
    pub uptime_ms: u64,

    // Additional metrics
    pub network_tx_packets: u64,
    pub network_rx_packets: u64,
    pub disk_read_ops: u64,
    pub disk_write_ops: u64,
}

impl MetricsSnapshot {
    /// Create a metrics snapshot from current system state
    pub fn capture() -> Self {
        // Get context switch percentiles
        let mut buf = [0usize; 8];
        let n = crate::trace::metrics_snapshot_ctx_switch(&mut buf);

        // Calculate percentiles (simple approximation)
        let (p50, p95, p99) = if n > 0 {
            let sorted = {
                let mut sorted_buf = buf;
                sorted_buf[..n].sort_unstable();
                sorted_buf
            };
            let p50 = sorted[n / 2] as u64;
            let p95 = sorted[(n * 95) / 100] as u64;
            let p99 = sorted[(n * 99) / 100] as u64;
            (p50, p95, p99)
        } else {
            (0, 0, 0)
        };

        // Get heap stats (stub - will integrate with actual heap)
        let (heap_allocs, heap_deallocs, heap_current, heap_peak, heap_failures) =
            get_heap_stats();

        // Get uptime
        let uptime_ms = crate::time::get_time_since_boot_ms();

        Self {
            ctx_switch_p50_ns: p50,
            ctx_switch_p95_ns: p95,
            ctx_switch_p99_ns: p99,
            heap_allocs,
            heap_deallocs,
            heap_current_bytes: heap_current,
            heap_peak_bytes: heap_peak,
            heap_failures,
            panic_count: 0, // TODO: Track panics
            uptime_ms,
            network_tx_packets: 0, // TODO: Integrate with network stats
            network_rx_packets: 0,
            disk_read_ops: 0,
            disk_write_ops: 0,
        }
    }

    /// Export metrics as JSON
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"ctx_switch_p50_ns":{},"ctx_switch_p95_ns":{},"ctx_switch_p99_ns":{},\
            "heap_allocs":{},"heap_deallocs":{},"heap_current_bytes":{},\
            "heap_peak_bytes":{},"heap_failures":{},\
            "panic_count":{},"uptime_ms":{},\
            "network_tx_packets":{},"network_rx_packets":{},\
            "disk_read_ops":{},"disk_write_ops":{}}}"#,
            self.ctx_switch_p50_ns,
            self.ctx_switch_p95_ns,
            self.ctx_switch_p99_ns,
            self.heap_allocs,
            self.heap_deallocs,
            self.heap_current_bytes,
            self.heap_peak_bytes,
            self.heap_failures,
            self.panic_count,
            self.uptime_ms,
            self.network_tx_packets,
            self.network_rx_packets,
            self.disk_read_ops,
            self.disk_write_ops
        )
    }

    /// Export metrics in Prometheus format
    pub fn to_prometheus(&self) -> String {
        let mut output = String::new();

        // Context switch metrics
        output.push_str("# HELP ctx_switch_ns Context switch time in nanoseconds\n");
        output.push_str("# TYPE ctx_switch_ns summary\n");
        output.push_str(&format!("ctx_switch_ns{{quantile=\"0.5\"}} {}\n", self.ctx_switch_p50_ns));
        output.push_str(&format!("ctx_switch_ns{{quantile=\"0.95\"}} {}\n", self.ctx_switch_p95_ns));
        output.push_str(&format!("ctx_switch_ns{{quantile=\"0.99\"}} {}\n", self.ctx_switch_p99_ns));

        // Heap metrics
        output.push_str("# HELP heap_bytes Heap memory usage in bytes\n");
        output.push_str("# TYPE heap_bytes gauge\n");
        output.push_str(&format!("heap_bytes{{state=\"current\"}} {}\n", self.heap_current_bytes));
        output.push_str(&format!("heap_bytes{{state=\"peak\"}} {}\n", self.heap_peak_bytes));

        output.push_str("# HELP heap_operations_total Total heap operations\n");
        output.push_str("# TYPE heap_operations_total counter\n");
        output.push_str(&format!("heap_operations_total{{operation=\"alloc\"}} {}\n", self.heap_allocs));
        output.push_str(&format!("heap_operations_total{{operation=\"dealloc\"}} {}\n", self.heap_deallocs));
        output.push_str(&format!("heap_operations_total{{operation=\"failure\"}} {}\n", self.heap_failures));

        // System metrics
        output.push_str("# HELP panic_count_total Total kernel panics\n");
        output.push_str("# TYPE panic_count_total counter\n");
        output.push_str(&format!("panic_count_total {}\n", self.panic_count));

        output.push_str("# HELP uptime_ms Uptime in milliseconds\n");
        output.push_str("# TYPE uptime_ms counter\n");
        output.push_str(&format!("uptime_ms {}\n", self.uptime_ms));

        // Network metrics
        output.push_str("# HELP network_packets_total Total network packets\n");
        output.push_str("# TYPE network_packets_total counter\n");
        output.push_str(&format!("network_packets_total{{direction=\"tx\"}} {}\n", self.network_tx_packets));
        output.push_str(&format!("network_packets_total{{direction=\"rx\"}} {}\n", self.network_rx_packets));

        // Disk metrics
        output.push_str("# HELP disk_operations_total Total disk operations\n");
        output.push_str("# TYPE disk_operations_total counter\n");
        output.push_str(&format!("disk_operations_total{{operation=\"read\"}} {}\n", self.disk_read_ops));
        output.push_str(&format!("disk_operations_total{{operation=\"write\"}} {}\n", self.disk_write_ops));

        output
    }

    /// Export metrics in simple key=value format
    pub fn to_simple(&self) -> String {
        format!(
            "ctx_switch_p50_ns={} ctx_switch_p95_ns={} ctx_switch_p99_ns={} \
            heap_allocs={} heap_deallocs={} heap_current_bytes={} \
            heap_peak_bytes={} heap_failures={} \
            panic_count={} uptime_ms={} \
            network_tx_packets={} network_rx_packets={} \
            disk_read_ops={} disk_write_ops={}",
            self.ctx_switch_p50_ns,
            self.ctx_switch_p95_ns,
            self.ctx_switch_p99_ns,
            self.heap_allocs,
            self.heap_deallocs,
            self.heap_current_bytes,
            self.heap_peak_bytes,
            self.heap_failures,
            self.panic_count,
            self.uptime_ms,
            self.network_tx_packets,
            self.network_rx_packets,
            self.disk_read_ops,
            self.disk_write_ops
        )
    }
}

/// Get heap statistics (stub - integrate with actual heap allocator)
fn get_heap_stats() -> (u64, u64, u64, u64, u64) {
    // TODO: Integrate with crate::heap module
    // For now, return placeholder values
    (0, 0, 0, 0, 0)
}

/// Export current metrics as JSON string
pub fn export_json() -> String {
    let metrics = MetricsSnapshot::capture();
    metrics.to_json()
}

/// Export current metrics as Prometheus string
pub fn export_prometheus() -> String {
    let metrics = MetricsSnapshot::capture();
    metrics.to_prometheus()
}

/// Export current metrics in simple format
pub fn export_simple() -> String {
    let metrics = MetricsSnapshot::capture();
    metrics.to_simple()
}
