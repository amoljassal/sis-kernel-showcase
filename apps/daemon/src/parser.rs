//! UART line parser for SIS kernel output
//!
//! Parses kernel output lines into structured events:
//! - METRIC lines: `METRIC name=value`
//! - Boot markers: `KERNEL(U)`, `STACK OK`, `MMU ON`, etc.
//! - Banner lines: informational output
//! - Shell lines: interactive shell I/O

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Boot markers that indicate kernel initialization progress
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BootMarker {
    KernelU,        // KERNEL(U) - Kernel entered
    StackOk,        // STACK OK
    MmuSctlr,       // MMU: SCTLR
    MmuOn,          // MMU ON
    UartReady,      // UART: READY
    GicInit,        // GIC: INIT
    VectorsOk,      // VECTORS OK
    LaunchingShell, // LAUNCHING SHELL
    ShellReady,     // sis> prompt appeared
}

impl BootMarker {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::KernelU => "Kernel entry point reached",
            Self::StackOk => "Stack initialized",
            Self::MmuSctlr => "MMU control register configured",
            Self::MmuOn => "Memory management unit enabled",
            Self::UartReady => "UART driver initialized",
            Self::GicInit => "Generic Interrupt Controller initialized",
            Self::VectorsOk => "Exception vectors installed",
            Self::LaunchingShell => "Shell launching",
            Self::ShellReady => "Shell prompt ready",
        }
    }

    /// Try to parse a boot marker from a line
    pub fn from_line(line: &str) -> Option<Self> {
        // Check for specific patterns
        if line.contains("KERNEL(U)") {
            Some(Self::KernelU)
        } else if line.contains("STACK OK") {
            Some(Self::StackOk)
        } else if line.contains("MMU: SCTLR") {
            Some(Self::MmuSctlr)
        } else if line.contains("MMU ON") {
            Some(Self::MmuOn)
        } else if line.contains("UART: READY") {
            Some(Self::UartReady)
        } else if line.contains("GIC: INIT") {
            Some(Self::GicInit)
        } else if line.contains("VECTORS OK") {
            Some(Self::VectorsOk)
        } else if line.contains("LAUNCHING SHELL") {
            Some(Self::LaunchingShell)
        } else if line.contains("sis>") {
            Some(Self::ShellReady)
        } else {
            None
        }
    }

    /// Get ordered list of all boot markers
    pub fn all_markers() -> Vec<Self> {
        vec![
            Self::KernelU,
            Self::StackOk,
            Self::MmuSctlr,
            Self::MmuOn,
            Self::UartReady,
            Self::GicInit,
            Self::VectorsOk,
            Self::LaunchingShell,
            Self::ShellReady,
        ]
    }
}

/// Test result marker from self_check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TestResult {
    Pass,
    Fail,
}

/// Parsed event from kernel output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedEvent {
    /// Metric line: METRIC name=value
    Metric {
        name: String,
        value: f64,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Boot marker detected
    Marker {
        marker: BootMarker,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Banner or informational line
    Banner {
        text: String,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Shell I/O line
    Shell {
        text: String,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Shell prompt detected (sis>)
    Prompt {
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Test result (PASS/FAIL from self_check)
    TestResult {
        test_name: String,
        result: TestResult,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

// Regex patterns (compiled once)
static METRIC_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"METRIC\s+([a-zA-Z_][a-zA-Z0-9_]*)=([0-9.eE+-]+)").unwrap());

static PROMPT_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^\s*sis>\s*$").unwrap());

static TEST_RESULT_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[(PASS|FAIL)\]\s+(.+)").unwrap());

static ANSI_ESCAPE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap());

/// Strip ANSI escape sequences from a string
fn strip_ansi(text: &str) -> String {
    ANSI_ESCAPE_PATTERN.replace_all(text, "").to_string()
}

/// Line parser for SIS kernel UART output
#[derive(Debug)]
pub struct LineParser {
    shell_active: bool,
    saw_launching_shell: bool,
}

impl Default for LineParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LineParser {
    /// Create a new line parser
    pub fn new() -> Self {
        Self {
            shell_active: false,
            saw_launching_shell: false,
        }
    }

    /// Parse a single line of kernel output
    pub fn parse_line(&mut self, line: &str) -> Option<ParsedEvent> {
        let line = line.trim_end_matches(|c| c == '\r' || c == '\n').trim();
        if line.is_empty() {
            return None;
        }

        let timestamp = chrono::Utc::now();

        // Strip ANSI escapes before checking prompt
        let clean_line = strip_ansi(line);

        // Check for shell prompt (sis>) - only activate if saw LAUNCHING SHELL first
        if PROMPT_PATTERN.is_match(&clean_line) {
            if self.saw_launching_shell {
                self.shell_active = true;
                return Some(ParsedEvent::Prompt { timestamp });
            }
        }

        // Check for boot markers first (highest priority)
        if let Some(marker) = BootMarker::from_line(&clean_line) {
            if marker == BootMarker::LaunchingShell {
                self.saw_launching_shell = true;
            }
            if marker == BootMarker::ShellReady {
                self.shell_active = true;
            }
            return Some(ParsedEvent::Marker { marker, timestamp });
        }

        // Check for test results [PASS]/[FAIL]
        if let Some(captures) = TEST_RESULT_PATTERN.captures(line) {
            let result_str = captures.get(1)?.as_str();
            let test_name = captures.get(2)?.as_str().to_string();
            let result = match result_str {
                "PASS" => TestResult::Pass,
                "FAIL" => TestResult::Fail,
                _ => return None,
            };
            return Some(ParsedEvent::TestResult {
                test_name,
                result,
                timestamp,
            });
        }

        // Check for METRIC lines
        if let Some(captures) = METRIC_PATTERN.captures(line) {
            let name = captures.get(1)?.as_str().to_string();
            let value_str = captures.get(2)?.as_str();
            let value = value_str.parse::<f64>().ok()?;
            return Some(ParsedEvent::Metric {
                name,
                value,
                timestamp,
            });
        }

        // After shell is ready, treat lines as shell output
        if self.shell_active {
            return Some(ParsedEvent::Shell {
                text: line.to_string(),
                timestamp,
            });
        }

        // Otherwise, treat as banner
        Some(ParsedEvent::Banner {
            text: line.to_string(),
            timestamp,
        })
    }

    /// Check if shell is ready for commands
    pub fn is_shell_ready(&self) -> bool {
        self.shell_active
    }

    /// Parse multiple metrics from a line (supports multiple METRIC entries)
    pub fn parse_metrics(&self, line: &str) -> Vec<(String, f64)> {
        let mut metrics = Vec::new();
        for captures in METRIC_PATTERN.captures_iter(line) {
            if let (Some(name), Some(value_str)) = (captures.get(1), captures.get(2)) {
                if let Ok(value) = value_str.as_str().parse::<f64>() {
                    metrics.push((name.as_str().to_string(), value));
                }
            }
        }
        metrics
    }

    /// Reset parser state (e.g., when QEMU restarts)
    pub fn reset(&mut self) {
        self.shell_active = false;
        self.saw_launching_shell = false;
    }
}

/// Aggregate boot marker status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootStatus {
    pub markers: HashMap<String, bool>,
    pub complete: bool,
}

impl BootStatus {
    /// Create new boot status with all markers pending
    pub fn new() -> Self {
        let mut markers = HashMap::new();
        for marker in BootMarker::all_markers() {
            markers.insert(format!("{:?}", marker), false);
        }
        Self {
            markers,
            complete: false,
        }
    }

    /// Mark a boot marker as seen
    pub fn mark_seen(&mut self, marker: BootMarker) {
        self.markers.insert(format!("{:?}", marker), true);
        // Check if all markers are complete
        self.complete = self.markers.values().all(|&seen| seen);
    }

    /// Check if a specific marker was seen
    pub fn is_seen(&self, marker: BootMarker) -> bool {
        self.markers
            .get(&format!("{:?}", marker))
            .copied()
            .unwrap_or(false)
    }
}

impl Default for BootStatus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metric() {
        let mut parser = LineParser::new();
        let event = parser.parse_line("METRIC irq_latency_ns=1234.5");
        assert!(matches!(event, Some(ParsedEvent::Metric { .. })));

        if let Some(ParsedEvent::Metric { name, value, .. }) = event {
            assert_eq!(name, "irq_latency_ns");
            assert_eq!(value, 1234.5);
        }
    }

    #[test]
    fn test_parse_boot_marker() {
        let mut parser = LineParser::new();

        // Test each boot marker
        assert!(matches!(
            parser.parse_line("KERNEL(U)"),
            Some(ParsedEvent::Marker {
                marker: BootMarker::KernelU,
                ..
            })
        ));

        assert!(matches!(
            parser.parse_line("[INFO] STACK OK"),
            Some(ParsedEvent::Marker {
                marker: BootMarker::StackOk,
                ..
            })
        ));

        assert!(matches!(
            parser.parse_line("MMU: SCTLR configured"),
            Some(ParsedEvent::Marker {
                marker: BootMarker::MmuSctlr,
                ..
            })
        ));
    }

    #[test]
    fn test_shell_activation() {
        let mut parser = LineParser::new();

        // Before shell ready, should be banner
        assert!(matches!(
            parser.parse_line("Some boot message"),
            Some(ParsedEvent::Banner { .. })
        ));

        // Shell prompt activates shell mode
        parser.parse_line("sis>");

        // After shell ready, should be shell output
        assert!(matches!(
            parser.parse_line("help"),
            Some(ParsedEvent::Shell { .. })
        ));
    }

    #[test]
    fn test_multiple_metrics() {
        let parser = LineParser::new();
        let metrics = parser.parse_metrics("METRIC cpu_util=45.2 METRIC mem_used=1024");
        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].0, "cpu_util");
        assert_eq!(metrics[0].1, 45.2);
        assert_eq!(metrics[1].0, "mem_used");
        assert_eq!(metrics[1].1, 1024.0);
    }

    #[test]
    fn test_boot_status() {
        let mut status = BootStatus::new();
        assert!(!status.complete);

        // Mark all markers as seen
        for marker in BootMarker::all_markers() {
            status.mark_seen(marker);
        }

        assert!(status.complete);
        assert!(status.is_seen(BootMarker::ShellReady));
    }
}
