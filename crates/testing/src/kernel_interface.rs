// SIS Kernel Command Interface
// Enables external testing suite to execute real kernel shell commands

use crate::{TestError, TestResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::process::{Child, Command};
use tokio::time::sleep;

/// Results from executing Phase 3 AI validation commands in kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealAIValidationResults {
    pub real_time_ai_results: RealTimeAIMetrics,
    pub temporal_isolation_results: TemporalIsolationMetrics,
    pub phase3_validation_results: Phase3ValidationMetrics,
    pub command_execution_time: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeAIMetrics {
    pub ai_inference_latency_us: Option<f64>,
    pub ai_deadline_misses: Option<u32>,
    pub ai_budget_utilization: Option<f64>,
    pub neural_engine_utilization: Option<f64>,
    pub pmu_cycle_measurements: Option<HashMap<String, u64>>,
    pub deterministic_scheduler_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalIsolationMetrics {
    pub ai_workload_latency_us: Option<f64>,
    pub traditional_workload_latency_us: Option<f64>,
    pub concurrent_workload_latency_us: Option<f64>,
    pub interference_overhead_percent: Option<f64>,
    pub isolation_verified: bool,
    pub temporal_guarantee_violations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase3ValidationMetrics {
    pub cbs_edf_scheduler_validated: bool,
    pub npu_driver_functional: bool,
    pub real_time_guarantees_verified: bool,
    pub ai_native_kernel_operational: bool,
    pub validation_tests_passed: u32,
    pub validation_tests_total: u32,
    pub overall_phase3_score: f64,
}

/// Command output parser for structured kernel shell responses
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub raw_output: String,
    pub parsed_metrics: HashMap<String, String>,
    pub success: bool,
    pub execution_time: Duration,
}

/// Interface to execute commands in running SIS kernel via QEMU serial connection
pub struct KernelCommandInterface {
    #[allow(dead_code)]
    qemu_process: Option<Child>,
    serial_log_path: String,
    serial_port: u16,
    #[allow(dead_code)]
    monitor_port: u16,
    command_timeout: Duration,
    last_log_position: u64,
}

impl KernelCommandInterface {
    pub fn new(serial_log_path: String, monitor_port: u16) -> Self {
        // Extract serial port from monitor port (serial port = monitor_port - 100)
        let serial_port = monitor_port - 100;
        Self {
            qemu_process: None,
            serial_log_path,
            serial_port,
            monitor_port,
            command_timeout: Duration::from_secs(30), // Increased for Phase 3 validation commands
            last_log_position: 0,
        }
    }

    /// Execute a shell command in the running kernel and parse structured output
    pub async fn execute_command(&mut self, command: &str) -> TestResult<CommandOutput> {
        self.execute_command_with_timeout(command, self.command_timeout).await
    }
    
    /// Execute command with custom timeout
    pub async fn execute_command_with_timeout(&mut self, command: &str, timeout: Duration) -> TestResult<CommandOutput> {
        let start_time = Instant::now();
        let old_timeout = self.command_timeout;
        self.command_timeout = timeout;
        
        // Wait for shell prompt to be ready
        self.wait_for_shell_prompt().await?;
        
        // Mark current position in serial log before sending command
        self.update_log_position().await?;
        
        // Send command via direct serial socket connection  
        self.send_command_via_serial(command).await?;
        
        // Wait for command completion and parse output
        let raw_output = self.wait_for_command_completion(command).await?;
        let parsed_metrics = self.parse_command_output(&raw_output);
        
        let execution_time = start_time.elapsed();
        let success = self.determine_command_success(&raw_output, command);
        
        // Restore original timeout
        self.command_timeout = old_timeout;
        
        Ok(CommandOutput {
            raw_output,
            parsed_metrics,
            success,
            execution_time,
        })
    }

    /// Test basic command execution with help command
    pub async fn test_basic_command_execution(&mut self) -> TestResult<CommandOutput> {
        log::info!("Testing basic command execution with 'help' command");
        self.execute_command("help").await
    }

    /// Execute the full Phase 3 AI validation command suite
    /// 
    /// This method orchestrates the complete Phase 3 AI-native kernel validation
    /// by executing the three primary validation commands and parsing their structured output.
    /// It's designed to be called by the external SIS Industry-Grade Test Suite.
    pub async fn run_phase3_ai_validation(&mut self) -> TestResult<RealAIValidationResults> {
        let start_time = Instant::now();
        
        log::info!("Starting Phase 3 AI validation command suite execution");
        
        // First test basic command execution
        match self.test_basic_command_execution().await {
            Ok(output) => {
                log::info!("Basic command execution successful: {}", 
                          output.raw_output.chars().take(100).collect::<String>());
            },
            Err(e) => {
                return Err(TestError::ExecutionFailed {
                    message: format!("Basic command execution failed: {}", e)
                });
            }
        }
        
        // Execute real-time AI validation with extended timeout
        log::info!("Executing rtaivalidation command in kernel");
        let rtai_output = self.execute_command_with_timeout("rtaivalidation", Duration::from_secs(60)).await?;
        let real_time_ai_results = self.parse_rtai_output(&rtai_output);
        
        // Execute temporal isolation validation with extended timeout  
        log::info!("Executing temporaliso command in kernel");
        let temporal_output = self.execute_command_with_timeout("temporaliso", Duration::from_secs(60)).await?;
        let temporal_isolation_results = self.parse_temporal_output(&temporal_output);
        
        // Execute comprehensive Phase 3 validation with extended timeout (increased for complex validation)
        log::info!("Executing phase3validation command in kernel");
        let phase3_output = self.execute_command_with_timeout("phase3validation", Duration::from_secs(180)).await?;
        let phase3_validation_results = self.parse_phase3_output(&phase3_output);
        
        let execution_time = start_time.elapsed();
        
        log::info!("Phase 3 AI validation commands completed in {:?}", execution_time);
        
        Ok(RealAIValidationResults {
            real_time_ai_results,
            temporal_isolation_results,
            phase3_validation_results,
            command_execution_time: execution_time,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Send command to kernel shell via direct serial socket connection
    async fn send_command_via_serial(&self, command: &str) -> TestResult<()> {
        // Send command directly to serial socket using netcat with proper TCP options
        let nc_command = format!(
            "printf '{}\\n' | nc -w 3 localhost {}", 
            command.replace("'", "'\"'\"'"), // Escape single quotes properly
            self.serial_port
        );
        
        let output = Command::new("sh")
            .args(["-c", &nc_command])
            .output()
            .await
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to connect to serial socket: {}", e)
            })?;
        
        if !output.status.success() {
            return Err(TestError::QEMUError {
                message: format!("Serial socket command failed: {}", 
                               String::from_utf8_lossy(&output.stderr))
            });
        }
        
        log::debug!("Successfully sent command '{}' to kernel via serial socket", command);
        Ok(())
    }

    /// Wait for shell prompt to be ready
    async fn wait_for_shell_prompt(&mut self) -> TestResult<()> {
        let deadline = Instant::now() + Duration::from_secs(10);
        
        while Instant::now() < deadline {
            let content = fs::read_to_string(&self.serial_log_path).await
                .map_err(|e| TestError::IoError(e))?;
            
            if content.contains("sis>") {
                log::debug!("Shell prompt detected, ready for commands");
                return Ok(());
            }
            
            sleep(Duration::from_millis(200)).await;
        }
        
        Err(TestError::ExecutionFailed {
            message: "Shell prompt not detected within timeout".to_string()
        })
    }

    /// Wait for command completion by monitoring serial log for expected patterns
    async fn wait_for_command_completion(&mut self, command: &str) -> TestResult<String> {
        let expected_completion_patterns = match command {
            "rtaivalidation" => vec![
                "[RT-AI VALIDATION] Real-time AI validation complete",
                "sis>" // Shell prompt indicates command completed
            ],
            "temporaliso" => vec![
                "[TEMPORAL ISOLATION] Temporal isolation validation complete",
                "sis>"
            ],
            "phase3validation" => vec![
                "[PHASE 3 VALIDATION] Phase 3 validation complete", 
                "Phase 3 validation complete - AI-native kernel operational",
                "[NPU] Processing test inference job",
                "[NPU PERF] NPU driver performance validation complete", // Earlier completion point
                "sis>" // Primary completion indicator
            ],
            _ => vec!["sis>"], // Default shell prompt
        };
        
        let deadline = Instant::now() + self.command_timeout;
        let mut accumulated_output = String::new();
        
        while Instant::now() < deadline {
            // Read new content from serial log
            let new_content = self.read_new_log_content().await?;
            accumulated_output.push_str(&new_content);
            
            // Check if any completion pattern is found
            for pattern in &expected_completion_patterns {
                if accumulated_output.contains(pattern) {
                    log::debug!("Command '{}' completed with pattern: '{}'", command, pattern);
                    return Ok(accumulated_output);
                }
            }
            
            // Brief delay before next check
            sleep(Duration::from_millis(100)).await;
        }
        
        Err(TestError::ExecutionFailed {
            message: format!("Command '{}' timed out after {:?}. Output: {}", 
                           command, self.command_timeout, accumulated_output.chars().take(200).collect::<String>())
        })
    }

    /// Read new content from serial log since last position
    async fn read_new_log_content(&mut self) -> TestResult<String> {
        let file_content = fs::read_to_string(&self.serial_log_path).await
            .map_err(|e| TestError::IoError(e))?;
        
        if file_content.len() as u64 > self.last_log_position {
            let new_content = file_content.chars()
                .skip(self.last_log_position as usize)
                .collect::<String>();
            self.last_log_position = file_content.len() as u64;
            Ok(new_content)
        } else {
            Ok(String::new())
        }
    }

    /// Update current position in serial log file
    async fn update_log_position(&mut self) -> TestResult<()> {
        if let Ok(metadata) = fs::metadata(&self.serial_log_path).await {
            self.last_log_position = metadata.len();
        }
        Ok(())
    }

    /// Parse generic command output for key-value metrics
    fn parse_command_output(&self, output: &str) -> HashMap<String, String> {
        let mut metrics = HashMap::new();
        
        for line in output.lines() {
            // Parse METRIC lines: "METRIC key=value"
            if let Some(metric_line) = line.strip_prefix("METRIC ") {
                if let Some((key, value)) = metric_line.split_once('=') {
                    metrics.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
            
            // Parse structured output patterns from Phase 3 commands
            if line.contains("AI inference latency:") {
                if let Some(latency) = self.extract_numeric_value(line, "μs") {
                    metrics.insert("ai_inference_latency_us".to_string(), latency);
                }
            }
            
            if line.contains("Neural Engine utilization:") {
                if let Some(util) = self.extract_numeric_value(line, "%") {
                    metrics.insert("neural_engine_utilization".to_string(), util);
                }
            }
        }
        
        metrics
    }

    /// Extract numeric value from text line with specified unit
    fn extract_numeric_value(&self, line: &str, unit: &str) -> Option<String> {
        // Look for pattern like "value: 123.45μs" or "value: 95.0%"
        if let Some(unit_pos) = line.find(unit) {
            let before_unit = &line[..unit_pos];
            if let Some(last_space) = before_unit.rfind(' ') {
                let number_part = &before_unit[last_space + 1..];
                if number_part.chars().all(|c| c.is_numeric() || c == '.') {
                    return Some(number_part.to_string());
                }
            }
        }
        None
    }

    /// Parse real-time AI validation command output
    fn parse_rtai_output(&self, output: &CommandOutput) -> RealTimeAIMetrics {
        let metrics = &output.parsed_metrics;
        
        RealTimeAIMetrics {
            ai_inference_latency_us: metrics.get("ai_inference_latency_us")
                .and_then(|s| s.parse().ok()),
            ai_deadline_misses: metrics.get("ai_deadline_misses")
                .and_then(|s| s.parse().ok()),
            ai_budget_utilization: metrics.get("ai_budget_utilization")
                .and_then(|s| s.parse().ok()),
            neural_engine_utilization: metrics.get("neural_engine_utilization")
                .and_then(|s| s.parse().ok()),
            pmu_cycle_measurements: None, // Could be expanded to parse PMU data
            deterministic_scheduler_active: output.raw_output.contains("CBS+EDF scheduler active"),
        }
    }

    /// Parse temporal isolation validation command output
    fn parse_temporal_output(&self, output: &CommandOutput) -> TemporalIsolationMetrics {
        let metrics = &output.parsed_metrics;
        
        TemporalIsolationMetrics {
            ai_workload_latency_us: metrics.get("ai_workload_latency_us")
                .and_then(|s| s.parse().ok()),
            traditional_workload_latency_us: metrics.get("traditional_workload_latency_us")
                .and_then(|s| s.parse().ok()),
            concurrent_workload_latency_us: metrics.get("concurrent_workload_latency_us")
                .and_then(|s| s.parse().ok()),
            interference_overhead_percent: metrics.get("interference_overhead_percent")
                .and_then(|s| s.parse().ok()),
            isolation_verified: output.raw_output.contains("OK Temporal isolation validated"),
            temporal_guarantee_violations: metrics.get("temporal_guarantee_violations")
                .and_then(|s| s.parse().ok()),
        }
    }

    /// Parse Phase 3 validation command output  
    fn parse_phase3_output(&self, output: &CommandOutput) -> Phase3ValidationMetrics {
        let raw = &output.raw_output;
        
        Phase3ValidationMetrics {
            cbs_edf_scheduler_validated: raw.contains("CBS+EDF scheduler integration validated"),
            npu_driver_functional: raw.contains("NPU driver operational"),
            real_time_guarantees_verified: raw.contains("Real-time guarantees verified"),
            ai_native_kernel_operational: raw.contains("AI-native kernel operational"),
            validation_tests_passed: self.count_pattern_occurrences(raw, "OK"),
            validation_tests_total: self.count_pattern_occurrences(raw, "OK") + self.count_pattern_occurrences(raw, "FAIL"),
            overall_phase3_score: if raw.contains("Phase 3 validation complete - AI-native kernel operational") { 100.0 } else { 0.0 },
        }
    }

    /// Count occurrences of a pattern in text
    fn count_pattern_occurrences(&self, text: &str, pattern: &str) -> u32 {
        text.matches(pattern).count() as u32
    }

    /// Determine if command executed successfully based on output patterns
    fn determine_command_success(&self, output: &str, command: &str) -> bool {
        match command {
            "rtaivalidation" => output.contains("[RT-AI VALIDATION] Real-time AI validation complete"),
            "temporaliso" => output.contains("[TEMPORAL ISOLATION] Temporal isolation validation complete"),
            "phase3validation" => output.contains("[PHASE 3 VALIDATION] Phase 3 validation complete"),
            _ => !output.contains("ERROR") && !output.contains("FAIL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numeric_value() {
        let interface = KernelCommandInterface::new("test.log".to_string(), 7100);
        
        assert_eq!(
            interface.extract_numeric_value("AI inference latency: 3.25μs", "μs"),
            Some("3.25".to_string())
        );
        
        assert_eq!(
            interface.extract_numeric_value("Neural Engine utilization: 95.0%", "%"),
            Some("95.0".to_string())
        );
    }

    #[test]
    fn test_parse_command_output() {
        let interface = KernelCommandInterface::new("test.log".to_string(), 7100);
        let output = "Test output\nMETRIC ai_inference_us=3.25\nMETRIC deadline_misses=0\nEnd";
        
        let metrics = interface.parse_command_output(output);
        assert_eq!(metrics.get("ai_inference_us"), Some(&"3.25".to_string()));
        assert_eq!(metrics.get("deadline_misses"), Some(&"0".to_string()));
    }
}