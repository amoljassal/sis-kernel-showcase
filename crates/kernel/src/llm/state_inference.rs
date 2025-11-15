//! Real-Time LLM Inference on OS State
//!
//! This module enables natural language queries about system state using an LLM.
//! It captures current OS metrics, formats them as natural language context,
//! and runs inference to produce actionable recommendations or explanations.
//!
//! # Architecture
//!
//! 1. **State Capture**: Gather memory, CPU, I/O, scheduler metrics
//! 2. **Serialization**: Format state as natural language prompt
//! 3. **Inference**: Run through LLM (reuses existing inference infrastructure)
//! 4. **Parsing**: Extract commands or explanations from output
//! 5. **Execution**: Optionally execute suggested shell commands
//!
//! # Performance Targets
//!
//! - 70%+ of queries produce actionable commands
//! - Inference latency <500ms (target <1s)
//! - Command execution success rate >80%

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// System state snapshot for LLM context
#[derive(Debug, Clone)]
pub struct SystemStateSnapshot {
    pub timestamp_ms: u64,
    pub memory_free_mb: usize,
    pub memory_used_mb: usize,
    pub fragmentation: f32,
    pub cpu_load: f32,
    pub num_tasks: usize,
    pub io_reads: u64,
    pub io_writes: u64,
}

impl SystemStateSnapshot {
    /// Capture current system state
    pub fn capture() -> Self {
        // Get memory stats
        let (memory_free_mb, memory_used_mb, fragmentation) =
            if let Some(stats) = crate::mm::buddy::get_stats() {
                let free_mb = (stats.free_pages * 4096) / (1024 * 1024);
                let used_mb = (stats.allocated_pages * 4096) / (1024 * 1024);
                let frag = if stats.total_pages > 0 {
                    stats.allocated_pages as f32 / stats.total_pages as f32
                } else {
                    0.0
                };
                (free_mb, used_mb, frag)
            } else {
                (0, 0, 0.0)
            };

        // Simplified CPU load and task count (would be real in production)
        let cpu_load = 0.5; // Placeholder
        let num_tasks = 10; // Placeholder

        // I/O stats (placeholders)
        let io_reads = 0;
        let io_writes = 0;

        Self {
            timestamp_ms: crate::time::get_uptime_ms(),
            memory_free_mb,
            memory_used_mb,
            fragmentation,
            cpu_load,
            num_tasks,
            io_reads,
            io_writes,
        }
    }
}

/// System state encoder for LLM
pub struct SystemStateEncoder {
    auto_execute: AtomicBool,
    query_count: AtomicU64,
    successful_executions: AtomicU64,
}

impl SystemStateEncoder {
    /// Create a new encoder
    pub fn new() -> Self {
        Self {
            auto_execute: AtomicBool::new(false),
            query_count: AtomicU64::new(0),
            successful_executions: AtomicU64::new(0),
        }
    }

    /// Encode system state and user query into LLM prompt
    pub fn encode_prompt(&self, state: &SystemStateSnapshot, user_query: &str) -> String {
        format!(
            "System State Analysis\n\
             ====================\n\
             Timestamp: {} ms uptime\n\
             \n\
             Memory Status:\n\
             - Free: {} MB\n\
             - Used: {} MB\n\
             - Fragmentation: {:.2}%\n\
             \n\
             CPU Status:\n\
             - Load: {:.2}%\n\
             - Active Tasks: {}\n\
             \n\
             I/O Status:\n\
             - Total Reads: {}\n\
             - Total Writes: {}\n\
             \n\
             User Query: {}\n\
             \n\
             Please analyze the system state and provide:\n\
             1. A brief explanation of the current situation\n\
             2. Recommended action (if any)\n\
             3. Shell command to execute (if applicable)\n\
             \n\
             Response:",
            state.timestamp_ms,
            state.memory_free_mb,
            state.memory_used_mb,
            state.fragmentation * 100.0,
            state.cpu_load * 100.0,
            state.num_tasks,
            state.io_reads,
            state.io_writes,
            user_query
        )
    }

    /// Parse LLM output to extract suggested command
    ///
    /// Looks for patterns like:
    /// - "Run: memctl compact"
    /// - "Execute: crashctl status"
    /// - "Command: schedctl transformer on"
    pub fn parse_command(&self, llm_output: &str) -> Option<String> {
        // Look for command markers
        let markers = ["Run:", "Execute:", "Command:", "run:", "execute:", "command:"];

        for line in llm_output.lines() {
            for marker in &markers {
                if let Some(cmd_start) = line.find(marker) {
                    let cmd = &line[cmd_start + marker.len()..].trim();
                    if !cmd.is_empty() {
                        return Some(cmd.to_string());
                    }
                }
            }
        }

        None
    }

    /// Set auto-execute mode
    pub fn set_auto_execute(&mut self, enabled: bool) {
        self.auto_execute.store(enabled, Ordering::Relaxed);
        if enabled {
            crate::info!("state_inference: auto-execute ENABLED");
        } else {
            crate::info!("state_inference: auto-execute DISABLED");
        }
    }

    /// Check if auto-execute is enabled
    pub fn is_auto_execute(&self) -> bool {
        self.auto_execute.load(Ordering::Relaxed)
    }

    /// Get query statistics
    pub fn stats(&self) -> InferenceStats {
        let total = self.query_count.load(Ordering::Relaxed);
        let successful = self.successful_executions.load(Ordering::Relaxed);
        let success_rate = if total > 0 {
            (successful as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        InferenceStats {
            total_queries: total,
            successful_executions: successful,
            success_rate,
        }
    }

    /// Record query execution
    pub fn record_query(&mut self, success: bool) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_executions.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl Default for SystemStateEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Inference statistics
#[derive(Debug, Clone, Copy)]
pub struct InferenceStats {
    pub total_queries: u64,
    pub successful_executions: u64,
    pub success_rate: f32,
}

/// Inference result
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub explanation: String,
    pub suggested_command: Option<String>,
    pub confidence: f32,
    pub latency_ms: u64,
}

/// Run inference on current system state
pub fn infer_on_state(user_query: &str) -> Result<InferenceResult, &'static str> {
    let start_time = crate::time::get_uptime_ms();

    // Capture current state
    let state = SystemStateSnapshot::capture();

    // Get encoder
    let encoder = STATE_ENCODER.lock();
    let encoder_ref = encoder.as_ref().ok_or("State encoder not initialized")?;

    // Encode prompt
    let prompt = encoder_ref.encode_prompt(&state, user_query);

    // Run LLM inference (simplified - would use real LLM in production)
    // For now, we generate deterministic responses based on state
    let (explanation, suggested_command) = generate_response(&state, user_query);

    let latency_ms = crate::time::get_uptime_ms() - start_time;

    Ok(InferenceResult {
        explanation,
        suggested_command,
        confidence: 0.8, // Placeholder
        latency_ms,
    })
}

/// Generate response based on state (simplified heuristic)
fn generate_response(state: &SystemStateSnapshot, query: &str) -> (String, Option<String>) {
    let query_lower = query.to_lowercase();

    // Memory-related queries
    if query_lower.contains("memory") || query_lower.contains("mem") {
        if state.fragmentation > 0.7 {
            (
                format!(
                    "Memory fragmentation is high at {:.1}%. This can cause allocation failures. \
                     Consider running memory compaction.",
                    state.fragmentation * 100.0
                ),
                Some("memctl compact".to_string()),
            )
        } else if state.memory_free_mb < 100 {
            (
                format!(
                    "Available memory is low at {} MB. Monitor for potential OOM conditions.",
                    state.memory_free_mb
                ),
                Some("crashctl status".to_string()),
            )
        } else {
            (
                format!(
                    "Memory status is healthy. {} MB free with {:.1}% fragmentation.",
                    state.memory_free_mb,
                    state.fragmentation * 100.0
                ),
                None,
            )
        }
    }
    // Performance queries
    else if query_lower.contains("performance") || query_lower.contains("slow") {
        (
            "For better performance, consider enabling the transformer scheduler for \
             intelligent task prioritization."
                .to_string(),
            Some("schedctl transformer on".to_string()),
        )
    }
    // Crash prediction queries
    else if query_lower.contains("crash") || query_lower.contains("stable") {
        (
            "Check crash prediction status to assess system stability.".to_string(),
            Some("crashctl status".to_string()),
        )
    }
    // General status
    else {
        (
            format!(
                "System is running normally. Memory: {} MB free, CPU load: {:.1}%, {} active tasks.",
                state.memory_free_mb,
                state.cpu_load * 100.0,
                state.num_tasks
            ),
            None,
        )
    }
}

/// Global state encoder
static STATE_ENCODER: Mutex<Option<SystemStateEncoder>> = Mutex::new(None);

/// Initialize state inference
pub fn init() {
    let mut encoder = STATE_ENCODER.lock();
    *encoder = Some(SystemStateEncoder::new());
    crate::info!("state_inference: initialized");
}

/// Set auto-execute mode
pub fn set_auto_execute(enabled: bool) {
    if let Some(encoder) = STATE_ENCODER.lock().as_mut() {
        encoder.set_auto_execute(enabled);
    }
}

/// Check if auto-execute is enabled
pub fn is_auto_execute() -> bool {
    STATE_ENCODER.lock()
        .as_ref()
        .map(|e| e.is_auto_execute())
        .unwrap_or(false)
}

/// Get inference statistics
pub fn get_stats() -> Option<InferenceStats> {
    STATE_ENCODER.lock()
        .as_ref()
        .map(|e| e.stats())
}

/// Record query execution result
pub fn record_query(success: bool) {
    if let Some(encoder) = STATE_ENCODER.lock().as_mut() {
        encoder.record_query(success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_snapshot() {
        let snapshot = SystemStateSnapshot::capture();
        assert!(snapshot.timestamp_ms > 0);
    }

    #[test]
    fn test_prompt_encoding() {
        let encoder = SystemStateEncoder::new();
        let state = SystemStateSnapshot {
            timestamp_ms: 1000,
            memory_free_mb: 512,
            memory_used_mb: 512,
            fragmentation: 0.5,
            cpu_load: 0.3,
            num_tasks: 5,
            io_reads: 100,
            io_writes: 50,
        };

        let prompt = encoder.encode_prompt(&state, "What's the memory status?");
        assert!(prompt.contains("Memory Status"));
        assert!(prompt.contains("512 MB"));
    }

    #[test]
    fn test_command_parsing() {
        let encoder = SystemStateEncoder::new();

        let output1 = "Analysis shows high fragmentation. Run: memctl compact";
        assert_eq!(
            encoder.parse_command(output1),
            Some("memctl compact".to_string())
        );

        let output2 = "System is healthy. No action needed.";
        assert_eq!(encoder.parse_command(output2), None);
    }
}
