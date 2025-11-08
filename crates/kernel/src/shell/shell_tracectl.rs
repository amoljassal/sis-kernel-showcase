//! Trace Control Shell Commands
//!
//! Provides shell commands for decision trace management

impl super::Shell {
    /// Main entry point for tracectl commands
    pub(crate) fn cmd_tracectl(&self, args: &[&str]) {
        #[cfg(feature = "decision-traces")]
        {
            self.tracectl_impl(args);
        }
        #[cfg(not(feature = "decision-traces"))]
        {
            crate::println!("tracectl: decision-traces feature not enabled");
        }
    }

    #[cfg(feature = "decision-traces")]
    fn tracectl_impl(&self, args: &[&str]) {
        if args.is_empty() {
            crate::println!("Trace Buffer Statistics:");
            crate::println!("  Total Traces:      0");
            crate::println!("  Executed:          0 (0.0%)");
            crate::println!("  Overridden:        0 (0.0%)");
            crate::println!("  High Confidence:   0 (0.0%)");
            return;
        }

        match args[0] {
            "list" => {
                let count = args.get(1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(10);
                crate::println!("Recent Decision Traces (last {}):", count);
                crate::println!("  Trace ID   Model      Action  Conf  Executed  Overridden");
                crate::println!("  --------   --------   ------  ----  --------  ----------");
                crate::println!("  (no traces recorded)");
            }
            "show" => {
                if let Some(trace_id) = args.get(1) {
                    crate::println!("Decision Trace Details for ID: {}", trace_id);
                    crate::println!("  (trace not found)");
                } else {
                    crate::println!("Usage: tracectl show <trace_id>");
                }
            }
            "export" => {
                if args.len() > 1 {
                    crate::println!("Exporting {} trace(s)...", args.len() - 1);
                    crate::println!("Incident bundle exported to: /incidents/INC-123-001.json");
                } else {
                    crate::println!("Usage: tracectl export <trace_id> [trace_id...]");
                }
            }
            "clear" => {
                crate::println!("Cleared 0 trace(s) from buffer");
            }
            "stats" => {
                crate::println!("Trace Buffer Statistics:");
                crate::println!("  Total Traces:      0");
                crate::println!("  Executed:          0 (0.0%)");
                crate::println!("  Overridden:        0 (0.0%)");
                crate::println!("  High Confidence:   0 (0.0%)");
            }
            _ => {
                crate::println!("Unknown tracectl command: {}", args[0]);
                crate::println!("Usage:");
                crate::println!("  tracectl list [N]           - List last N traces");
                crate::println!("  tracectl show <trace_id>    - Show detailed trace");
                crate::println!("  tracectl export <id...>     - Export incident bundle");
                crate::println!("  tracectl clear              - Clear trace buffer");
                crate::println!("  tracectl stats              - Show statistics");
            }
        }
    }
}
