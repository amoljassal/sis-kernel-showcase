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
        use crate::trace_decision::buffer::TRACE_BUFFER;
        if args.is_empty() {
            let s = TRACE_BUFFER.stats();
            let exec_pct = if s.total > 0 { (s.executed as f32 * 100.0 / s.total as f32) } else { 0.0 };
            let over_pct = if s.total > 0 { (s.overridden as f32 * 100.0 / s.total as f32) } else { 0.0 };
            let hi_pct = if s.total > 0 { (s.high_confidence as f32 * 100.0 / s.total as f32) } else { 0.0 };
            crate::kprintln!("Trace Buffer Statistics:");
            crate::kprintln!("  Total Traces:      {}", s.total);
            crate::kprintln!("  Executed:          {} ({:.1}%)", s.executed, exec_pct);
            crate::kprintln!("  Overridden:        {} ({:.1}%)", s.overridden, over_pct);
            crate::kprintln!("  High Confidence:   {} ({:.1}%)", s.high_confidence, hi_pct);
            return;
        }

        match args[0] {
            "demo" => {
                // Generate N synthetic decision traces for demo purposes
                let n = args.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(10);
                let mut id = 1000u64;
                for _ in 0..n {
                    let mut t = crate::trace_decision::DecisionTrace::new(id);
                    t.model_version = alloc::string::String::from("v_demo");
                    t.predictions = alloc::vec![0.1, 0.2, 0.7];
                    t.chosen_action = 2;
                    t.confidence = 700;
                    t.was_executed = true;
                    crate::trace_decision::buffer::TRACE_BUFFER.record(t);
                    id += 1;
                }
                crate::kprintln!("Generated {} demo traces", n);
            }
            #[cfg(feature = "shadow-mode")]
            "export-divergences" => {
                // Syntax: tracectl export-divergences [N] [--path <file>]
                let mut n: Option<usize> = None;
                let mut out_path: Option<&str> = None;

                let mut i = 1;
                while i < args.len() {
                    match args[i] {
                        "--path" | "-o" => {
                            if let Some(p) = args.get(i + 1) { out_path = Some(p); i += 1; }
                        }
                        token => {
                            if let Ok(v) = token.parse::<usize>() { n = Some(v); }
                        }
                    }
                    i += 1;
                }

                let count = n.unwrap_or(50);
                let res = if let Some(p) = out_path {
                    crate::trace_decision::export::INCIDENT_EXPORTER
                        .export_shadow_divergences_to_path(count, p)
                } else {
                    crate::trace_decision::export::INCIDENT_EXPORTER
                        .export_shadow_divergences(count)
                };

                match res {
                    Ok(p) => crate::kprintln!("Shadow divergences exported to: {}", p),
                    Err(e) => crate::kprintln!("export-divergences: failed ({:?})", e),
                }
            }
            "list" => {
                let count = args.get(1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(10);
                crate::kprintln!("Recent Decision Traces (last {}):", count);
                crate::kprintln!("  Trace ID   Model      Action  Conf  Executed  Overridden");
                crate::kprintln!("  --------   --------   ------  ----  --------  ----------");
                let traces = TRACE_BUFFER.get_last_n(count);
                if traces.is_empty() {
                    crate::kprintln!("  (no traces recorded)");
                } else {
                    for t in traces {
                        crate::kprintln!("  {:<10} {:<10} {:<6} {:<4} {:<8} {:<10}",
                            t.trace_id,
                            if t.model_version.is_empty() { "-" } else { &t.model_version },
                            t.chosen_action,
                            t.confidence,
                            if t.was_executed { "Y" } else { "N" },
                            if t.was_overridden { "Y" } else { "N" },
                        );
                    }
                }
            }
            "show" => {
                if let Some(trace_id) = args.get(1).and_then(|s| s.parse::<u64>().ok()) {
                    if let Some(t) = TRACE_BUFFER.find_by_trace_id(trace_id) {
                        crate::kprintln!("Decision Trace Details for ID: {}", trace_id);
                        crate::kprintln!("  model={} action={} conf={}", t.model_version, t.chosen_action, t.confidence);
                        crate::kprintln!("  executed={} overridden={} reason={}", t.was_executed, t.was_overridden, t.override_reason.unwrap_or_default());
                    } else {
                        crate::kprintln!("  (trace not found)");
                    }
                } else {
                    crate::kprintln!("Usage: tracectl show <trace_id>");
                }
            }
            "export" => {
                // Syntax: tracectl export [--path <file>] [--all | --recent N | <id...>]
                use crate::trace_decision::buffer::TRACE_BUFFER;
                let mut out_path: Option<&str> = None;
                let mut all = false;
                let mut recent: Option<usize> = None;
                let mut ids: alloc::vec::Vec<u64> = alloc::vec::Vec::new();

                let mut i = 1;
                while i < args.len() {
                    match args[i] {
                        "--path" | "-o" => {
                            if let Some(p) = args.get(i + 1) { out_path = Some(p); i += 1; }
                        }
                        "--all" => { all = true; }
                        "--recent" => {
                            if let Some(n) = args.get(i + 1).and_then(|s| s.parse::<usize>().ok()) {
                                recent = Some(n);
                                i += 1;
                            }
                        }
                        token => {
                            if let Ok(id) = token.parse::<u64>() { ids.push(id); }
                        }
                    }
                    i += 1;
                }

                // Default behavior: if nothing specified, export last 10 traces
                if !all && recent.is_none() && ids.is_empty() {
                    recent = Some(10);
                }

                // Resolve selection into IDs
                if all {
                    let all_traces = TRACE_BUFFER.get_last_n(usize::MAX);
                    ids = all_traces.iter().map(|t| t.trace_id).collect();
                } else if let Some(n) = recent {
                    let last = TRACE_BUFFER.get_last_n(n);
                    ids = last.iter().map(|t| t.trace_id).collect();
                }

                if ids.is_empty() {
                    crate::kprintln!("export: no traces selected (buffer empty?)");
                } else {
                    crate::kprintln!("Exporting {} trace(s)...", ids.len());
                    let res = if let Some(p) = out_path {
                        crate::trace_decision::export::INCIDENT_EXPORTER
                            .export_traces_to_path(&ids, p)
                    } else {
                        crate::trace_decision::export::INCIDENT_EXPORTER.export_bundle(&ids)
                    };
                    match res {
                        Ok(path) => crate::kprintln!("Incident bundle exported to: {}", path),
                        Err(e) => crate::kprintln!("export: failed ({:?})", e),
                    }
                }
            }
            "clear" => {
                use crate::trace_decision::buffer::TRACE_BUFFER;
                let count = TRACE_BUFFER.len();
                TRACE_BUFFER.clear();
                crate::kprintln!("Cleared {} trace(s) from buffer", count);
            }
            "stats" => {
                let s = TRACE_BUFFER.stats();
                let exec_pct = if s.total > 0 { (s.executed as f32 * 100.0 / s.total as f32) } else { 0.0 };
                let over_pct = if s.total > 0 { (s.overridden as f32 * 100.0 / s.total as f32) } else { 0.0 };
                let hi_pct = if s.total > 0 { (s.high_confidence as f32 * 100.0 / s.total as f32) } else { 0.0 };
                crate::kprintln!("Trace Buffer Statistics:");
                crate::kprintln!("  Total Traces:      {}", s.total);
                crate::kprintln!("  Executed:          {} ({:.1}%)", s.executed, exec_pct);
                crate::kprintln!("  Overridden:        {} ({:.1}%)", s.overridden, over_pct);
                crate::kprintln!("  High Confidence:   {} ({:.1}%)", s.high_confidence, hi_pct);
            }
            _ => {
                crate::kprintln!("Unknown tracectl command: {}", args[0]);
                crate::kprintln!("Usage:");
                crate::kprintln!("  tracectl list [N]           - List last N traces");
                crate::kprintln!("  tracectl show <trace_id>    - Show detailed trace");
                crate::kprintln!("  tracectl export [--path <file>] [--all | --recent N | <id...>] - Export traces");
                crate::kprintln!("  tracectl demo [N]           - Generate N synthetic traces");
                #[cfg(feature = "shadow-mode")]
                crate::kprintln!("  tracectl export-divergences [N] [--path <file>] - Export shadow divergences");
                crate::kprintln!("  tracectl clear              - Clear trace buffer");
                crate::kprintln!("  tracectl stats              - Show statistics");
            }
        }
    }
}
