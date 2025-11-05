//! Shell command execution logic

use super::shell::{ShellCommandRequest, ShellCommandResponse};
use crate::parser::{ParsedEvent, TestResult};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{timeout, Duration};
use tracing::{debug, warn};

/// Command executor handles shell command execution
pub struct CommandExecutor {
    stdin: Arc<RwLock<Option<ChildStdin>>>,
    response_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<ParsedEvent>>>>,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new(
        stdin: ChildStdin,
        response_rx: mpsc::UnboundedReceiver<ParsedEvent>,
    ) -> Self {
        Self {
            stdin: Arc::new(RwLock::new(Some(stdin))),
            response_rx: Arc::new(RwLock::new(Some(response_rx))),
        }
    }

    /// Execute a shell command and collect response
    pub async fn execute(&self, request: ShellCommandRequest) -> Result<ShellCommandResponse> {
        let start = Instant::now();

        debug!("Executing command: {}", request.command);

        // Write command to stdin
        {
            let mut stdin_guard = self.stdin.write().await;
            let stdin = stdin_guard
                .as_mut()
                .context("Stdin not available")?;

            // Send command with newline
            let cmd_line = format!("{}\n", request.command);
            stdin
                .write_all(cmd_line.as_bytes())
                .await
                .context("Failed to write command")?;
            stdin.flush().await.context("Failed to flush stdin")?;
        }

        // Collect response until next prompt or timeout
        let output = self
            .collect_response(Duration::from_millis(request.timeout_ms))
            .await?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(ShellCommandResponse {
            command: request.command.clone(),
            output,
            success: true,
            error: None,
            execution_time_ms,
        })
    }

    /// Collect response lines until prompt appears
    async fn collect_response(&self, timeout_duration: Duration) -> Result<Vec<String>> {
        let mut output = Vec::new();

        let result = timeout(timeout_duration, async {
            let mut rx_guard = self.response_rx.write().await;
            let rx = rx_guard
                .as_mut()
                .context("Response receiver not available")?;

            while let Some(event) = rx.recv().await {
                match event {
                    ParsedEvent::Shell { text, .. } => {
                        // Skip echoed command and empty lines
                        if !text.is_empty() {
                            output.push(text);
                        }
                    }
                    ParsedEvent::Prompt { .. } => {
                        // Prompt indicates end of response
                        break;
                    }
                    _ => {
                        // Ignore other event types during command execution
                    }
                }
            }

            Ok::<_, anyhow::Error>(())
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(output),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                warn!("Command execution timed out");
                Err(anyhow::anyhow!("Command execution timed out"))
            }
        }
    }

    /// Check if executor is ready
    pub async fn is_ready(&self) -> bool {
        self.stdin.read().await.is_some() && self.response_rx.read().await.is_some()
    }
}

/// Test result collector for self_check
pub struct TestResultCollector {
    results: Vec<(String, TestResult)>,
}

impl TestResultCollector {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, name: String, result: TestResult) {
        self.results.push((name, result));
    }

    pub fn get_results(&self) -> &[(String, TestResult)] {
        &self.results
    }

    pub fn passed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|(_, r)| matches!(r, TestResult::Pass))
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|(_, r)| matches!(r, TestResult::Fail))
            .count()
    }
}
