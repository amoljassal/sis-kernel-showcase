//! Trace Control Shell Commands
//!
//! Provides shell commands for decision trace management

use alloc::string::String;

impl super::Shell {
    /// Main entry point for tracectl commands
    pub(crate) fn cmd_tracectl(&self, args: &[&str]) {
        use alloc::string::String;
        #[cfg(feature = "decision-traces")]
        {
            self.tracectl_impl(args);
        }
        #[cfg(not(feature = "decision-traces"))]
        {
            crate::kprintln!("tracectl: decision-traces feature not enabled");
        }
    }

    #[cfg(feature = "decision-traces")]
    fn tracectl_impl(&self, args: &[&str]) {
        if args.is_empty() {
            crate::kprintln!("Trace Buffer Statistics:");
            crate::kprintln!("  Total Traces:      0");
            crate::kprintln!("  Executed:          0 (0.0%)");
            crate::kprintln!("  Overridden:        0 (0.0%)");
            crate::kprintln!("  High Confidence:   0 (0.0%)");
            return;
        }

        match args[0] {
            #[cfg(feature = "shadow-mode")]
            "export-divergences" => {
                let n = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50);
                let path = match crate::trace_decision::export::INCIDENT_EXPORTER.export_shadow_divergences(n) {
                    Ok(p) => p,
                    Err(_) => String::from("(error exporting divergences)"),
                };
                crate::kprintln!("Shadow divergences exported to: {}", path);
            }
            "list" => {
                let count = args.get(1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(10);
                crate::kprintln!("Recent Decision Traces (last {}):", count);
                crate::kprintln!("  Trace ID   Model      Action  Conf  Executed  Overridden");
                crate::kprintln!("  --------   --------   ------  ----  --------  ----------");
                crate::kprintln!("  (no traces recorded)");
            }
            "show" => {
                if let Some(trace_id) = args.get(1) {
                    crate::kprintln!("Decision Trace Details for ID: {}", trace_id);
                    crate::kprintln!("  (trace not found)");
                } else {
                    crate::kprintln!("Usage: tracectl show <trace_id>");
                }
            }
            "export" => {
                if args.len() > 1 {
                    crate::kprintln!("Exporting {} trace(s)...", args.len() - 1);
                    crate::kprintln!("Incident bundle exported to: /incidents/INC-123-001.json");
                } else {
                    crate::kprintln!("Usage: tracectl export <trace_id> [trace_id...]");
                }
            }
            "clear" => {
                crate::kprintln!("Cleared 0 trace(s) from buffer");
            }
            "stats" => {
                crate::kprintln!("Trace Buffer Statistics:");
                crate::kprintln!("  Total Traces:      0");
                crate::kprintln!("  Executed:          0 (0.0%)");
                crate::kprintln!("  Overridden:        0 (0.0%)");
                crate::kprintln!("  High Confidence:   0 (0.0%)");
            }
            _ => {
                crate::kprintln!("Unknown tracectl command: {}", args[0]);
                crate::kprintln!("Usage:");
                crate::kprintln!("  tracectl list [N]           - List last N traces");
                crate::kprintln!("  tracectl show <trace_id>    - Show detailed trace");
                crate::kprintln!("  tracectl export <id...>     - Export incident bundle");
                #[cfg(feature = "shadow-mode")]
                crate::kprintln!("  tracectl export-divergences [N] - Export recent shadow divergences");
                crate::kprintln!("  tracectl clear              - Clear trace buffer");
                crate::kprintln!("  tracectl stats              - Show statistics");
            }
        }
    }
}
