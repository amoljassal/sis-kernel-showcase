//! Metric parsing from kernel output
//!
//! Parses lines matching: METRIC name=value
//! Example: "METRIC nn_infer_us=1234"

use regex::Regex;

/// Parsed metric data
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMetric {
    /// Metric name (normalized: lowercase, trimmed)
    pub name: String,
    /// Metric value (signed integer)
    pub value: i64,
    /// Timestamp when parsed
    pub timestamp: i64,
}

/// Metric line parser
pub struct MetricParser {
    pattern: Regex,
    ansi_pattern: Regex,
}

impl MetricParser {
    /// Create a new metric parser
    pub fn new() -> Self {
        Self {
            // METRIC name=value
            // name: [A-Za-z0-9_:\-\.]+
            // value: -?[0-9]+
            pattern: Regex::new(r"^METRIC\s+([A-Za-z0-9_:\-\.]+)=(-?[0-9]+)\s*$")
                .expect("valid metric regex"),
            // ANSI escape sequences: ESC[...m
            ansi_pattern: Regex::new(r"\x1B\[[0-9;]*[mGKHf]").expect("valid ANSI regex"),
        }
    }

    /// Parse a line, returning metric if valid
    pub fn parse_line(&self, line: &str, timestamp: i64) -> Option<ParsedMetric> {
        // Strip ANSI escape sequences
        let clean = self.ansi_pattern.replace_all(line, "");

        // Match pattern
        let captures = self.pattern.captures(&clean)?;

        // Extract name and value
        let name = captures.get(1)?.as_str();
        let value_str = captures.get(2)?.as_str();

        // Parse value
        let value = value_str.parse::<i64>().ok()?;

        // Normalize name: trim + lowercase, keep ':'
        let normalized_name = name.trim().to_lowercase();

        Some(ParsedMetric {
            name: normalized_name,
            value,
            timestamp,
        })
    }
}

impl Default for MetricParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_metric() {
        let parser = MetricParser::new();
        let result = parser.parse_line("METRIC nn_infer_us=1234", 1000);
        assert_eq!(
            result,
            Some(ParsedMetric {
                name: "nn_infer_us".to_string(),
                value: 1234,
                timestamp: 1000,
            })
        );
    }

    #[test]
    fn test_parse_with_colon() {
        let parser = MetricParser::new();
        let result = parser.parse_line("METRIC nn:infer_us=5678", 2000);
        assert_eq!(
            result,
            Some(ParsedMetric {
                name: "nn:infer_us".to_string(),
                value: 5678,
                timestamp: 2000,
            })
        );
    }

    #[test]
    fn test_parse_negative_value() {
        let parser = MetricParser::new();
        let result = parser.parse_line("METRIC temperature_c=-15", 3000);
        assert_eq!(
            result,
            Some(ParsedMetric {
                name: "temperature_c".to_string(),
                value: -15,
                timestamp: 3000,
            })
        );
    }

    #[test]
    fn test_parse_with_ansi() {
        let parser = MetricParser::new();
        let result = parser.parse_line("\x1B[32mMETRIC\x1B[0m foo_bar=999", 4000);
        assert_eq!(
            result,
            Some(ParsedMetric {
                name: "foo_bar".to_string(),
                value: 999,
                timestamp: 4000,
            })
        );
    }

    #[test]
    fn test_parse_uppercase_normalized() {
        let parser = MetricParser::new();
        let result = parser.parse_line("METRIC FOO_BAR=123", 5000);
        assert_eq!(
            result,
            Some(ParsedMetric {
                name: "foo_bar".to_string(),
                value: 123,
                timestamp: 5000,
            })
        );
    }

    #[test]
    fn test_parse_with_whitespace() {
        let parser = MetricParser::new();
        let result = parser.parse_line("METRIC  test_metric  =  456  ", 6000);
        // This should NOT match because of spaces around =
        assert_eq!(result, None);

        // But this should work (trailing space is OK)
        let result2 = parser.parse_line("METRIC test_metric=456  ", 6000);
        assert_eq!(
            result2,
            Some(ParsedMetric {
                name: "test_metric".to_string(),
                value: 456,
                timestamp: 6000,
            })
        );
    }

    #[test]
    fn test_parse_invalid_lines() {
        let parser = MetricParser::new();

        // Missing value
        assert_eq!(parser.parse_line("METRIC foo", 1000), None);

        // Invalid characters in name
        assert_eq!(parser.parse_line("METRIC foo@bar=123", 1000), None);

        // No METRIC prefix
        assert_eq!(parser.parse_line("foo=123", 1000), None);

        // Float value (not supported)
        assert_eq!(parser.parse_line("METRIC foo=12.34", 1000), None);
    }
}
