// Split from shell.rs: metricsctl and metrics commands

impl super::Shell {
    /// Runtime toggle for metric capture
    pub(crate) fn cmd_metricsctl(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: metricsctl <on|off|status>\n"); }
            return;
        }
        match args[0] {
            "on" => {
                crate::trace::metrics_set_enabled(true);
                unsafe { crate::uart_print(b"[METRICSCTL] capture enabled\n"); }
            }
            "off" => {
                crate::trace::metrics_set_enabled(false);
                unsafe { crate::uart_print(b"[METRICSCTL] capture disabled\n"); }
            }
            "status" => {
                let enabled = crate::trace::metrics_enabled();
                unsafe {
                    crate::uart_print(b"[METRICSCTL] capture: ");
                    crate::uart_print(if enabled { b"ON\n" } else { b"OFF\n" });
                }
            }
            _ => {
                unsafe { crate::uart_print(b"Usage: metricsctl <on|off|status>\n"); }
            }
        }
    }

    /// Show recent metrics captured into small rings
    pub(crate) fn cmd_metrics(&self, args: &[&str]) {
        // Phase 1.3 enhancement: Support JSON and Prometheus export
        if let Some(format) = args.get(0) {
            match *format {
                "json" => {
                    // Export metrics as JSON
                    let json = crate::metrics_export::export_json();
                    unsafe {
                        crate::uart_print(json.as_bytes());
                        crate::uart_print(b"\n");
                    }
                    return;
                }
                "prometheus" | "prom" => {
                    // Export metrics as Prometheus format
                    let prom = crate::metrics_export::export_prometheus();
                    unsafe {
                        crate::uart_print(prom.as_bytes());
                    }
                    return;
                }
                "simple" => {
                    // Export metrics in simple key=value format
                    let simple = crate::metrics_export::export_simple();
                    unsafe {
                        crate::uart_print(simple.as_bytes());
                        crate::uart_print(b"\n");
                    }
                    return;
                }
                "help" => {
                    unsafe {
                        crate::uart_print(b"Usage: metrics [format]\n");
                        crate::uart_print(b"Formats:\n");
                        crate::uart_print(b"  json        - Export all metrics as JSON\n");
                        crate::uart_print(b"  prometheus  - Export all metrics as Prometheus format\n");
                        crate::uart_print(b"  simple      - Export all metrics as key=value\n");
                        crate::uart_print(b"  ctx         - Show context switch metrics\n");
                        crate::uart_print(b"  mem         - Show memory allocation metrics\n");
                        crate::uart_print(b"  real        - Show real context switch metrics\n");
                        crate::uart_print(b"  (default)   - Show all raw metrics\n");
                    }
                    return;
                }
                "ctx" => {
                    let mut buf = [0usize; 8];
                    let n = crate::trace::metrics_snapshot_ctx_switch(&mut buf);
                    unsafe { crate::uart_print(b"[METRICS] ctx_switch_ns:"); }
                    for i in 0..n { unsafe { crate::uart_print(b" "); } self.print_number_simple(buf[i] as u64); }
                    unsafe { crate::uart_print(b"\n"); }
                    return;
                }
                "mem" => {
                    let mut buf = [0usize; 8];
                    let n = crate::trace::metrics_snapshot_memory_alloc(&mut buf);
                    unsafe { crate::uart_print(b"[METRICS] memory_alloc_ns:"); }
                    for i in 0..n { unsafe { crate::uart_print(b" "); } self.print_number_simple(buf[i] as u64); }
                    unsafe { crate::uart_print(b"\n"); }
                    return;
                }
                "real" => {
                    let mut buf = [0usize; 8];
                    let n = crate::trace::metrics_snapshot_real_ctx(&mut buf);
                    unsafe { crate::uart_print(b"[METRICS] real_ctx_switch_ns:"); }
                    for i in 0..n { unsafe { crate::uart_print(b" "); } self.print_number_simple(buf[i] as u64); }
                    unsafe { crate::uart_print(b"\n"); }
                    return;
                }
                _ => {}
            }
        }
        let n1 = crate::trace::metrics_snapshot_ctx_switch(&mut buf);
        unsafe { crate::uart_print(b"[METRICS] ctx_switch_ns:"); }
        for i in 0..n1 { unsafe { crate::uart_print(b" "); } self.print_number_simple(buf[i] as u64); }
        unsafe { crate::uart_print(b"\n"); }
        let n2 = crate::trace::metrics_snapshot_memory_alloc(&mut buf);
        unsafe { crate::uart_print(b"[METRICS] memory_alloc_ns:"); }
        for i in 0..n2 { unsafe { crate::uart_print(b" "); } self.print_number_simple(buf[i] as u64); }
        unsafe { crate::uart_print(b"\n"); }
        let n3 = crate::trace::metrics_snapshot_real_ctx(&mut buf);
        unsafe { crate::uart_print(b"[METRICS] real_ctx_switch_ns:"); }
        for i in 0..n3 { unsafe { crate::uart_print(b" "); } self.print_number_simple(buf[i] as u64); }
        unsafe { crate::uart_print(b"\n"); }
    }
}

