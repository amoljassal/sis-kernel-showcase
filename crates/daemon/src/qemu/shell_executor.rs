//! Shell command execution with queue and prompt handling

#![allow(dead_code)]

use crate::parser::ParsedEvent;
use crate::qemu::shell::{ShellCommandRequest, ShellCommandResponse};
use anyhow::{Context, Result};
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Duration};
use tracing::debug;

const MAX_RESPONSE_BYTES: usize = 1_048_576; // 1MB cap
const DEFAULT_TIMEOUT_MS: u64 = 30000;

/// Command execution request (internal)
struct CommandRequest {
    command: String,
    timeout_ms: u64,
    response_tx: oneshot::Sender<Result<ShellCommandResponse>>,
}

/// Shell executor with command queue
#[derive(Debug)]
pub struct ShellExecutor {
    command_tx: mpsc::UnboundedSender<CommandRequest>,
}

impl ShellExecutor {
    /// Create a new shell executor
    pub fn new(
        mut stdin: ChildStdin,
        mut event_rx: mpsc::UnboundedReceiver<ParsedEvent>,
    ) -> Self {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel::<CommandRequest>();

        // Spawn executor task
        tokio::spawn(async move {
            let mut shell_ready = false;

            while let Some(req) = command_rx.recv().await {
                let start = Instant::now();

                // Wait for shell ready
                if !shell_ready {
                    // Check for prompt in event stream
                    loop {
                        match timeout(Duration::from_secs(30), event_rx.recv()).await {
                            Ok(Some(ParsedEvent::Prompt { .. })) => {
                                shell_ready = true;
                                break;
                            }
                            Ok(Some(ParsedEvent::Marker { .. })) => {
                                // Continue waiting
                            }
                            Ok(None) => {
                                let _ = req.response_tx.send(Err(anyhow::anyhow!(
                                    "Event stream closed"
                                )));
                                return;
                            }
                            Err(_) => {
                                let _ = req.response_tx.send(Err(anyhow::anyhow!(
                                    "Timeout waiting for shell prompt"
                                )));
                                return;
                            }
                            _ => {}
                        }
                    }
                }

                debug!("Executing command: {}", req.command);

                // Write command to stdin
                let cmd_line = format!("{}\n", req.command);
                if let Err(e) = stdin.write_all(cmd_line.as_bytes()).await {
                    let _ = req
                        .response_tx
                        .send(Err(anyhow::anyhow!("Failed to write command: {}", e)));
                    continue;
                }

                if let Err(e) = stdin.flush().await {
                    let _ = req
                        .response_tx
                        .send(Err(anyhow::anyhow!("Failed to flush stdin: {}", e)));
                    continue;
                }

                // Collect response until next prompt
                let timeout_duration = Duration::from_millis(req.timeout_ms);
                let mut output = Vec::new();
                let mut total_bytes = 0;
                let mut skip_first_echo = true; // Skip echoed command

                let result = timeout(timeout_duration, async {
                    while let Some(event) = event_rx.recv().await {
                        match event {
                            ParsedEvent::Shell { text, .. } => {
                                // Skip the echoed command line (case-insensitive, trim CR)
                                let text_normalized = text.trim_end_matches(|c| c == '\r' || c == '\n').trim();
                                let cmd_normalized = req.command.trim_end_matches(|c| c == '\r' || c == '\n').trim();

                                if skip_first_echo && text_normalized.eq_ignore_ascii_case(cmd_normalized) {
                                    skip_first_echo = false;
                                    continue;
                                }
                                skip_first_echo = false;

                                // Check byte cap
                                total_bytes += text.len();
                                if total_bytes > MAX_RESPONSE_BYTES {
                                    output.push(format!(
                                        "[truncated: exceeded {} bytes]",
                                        MAX_RESPONSE_BYTES
                                    ));
                                    break;
                                }

                                if !text.is_empty() {
                                    output.push(text);
                                }
                            }
                            ParsedEvent::Prompt { .. } => {
                                // End of response
                                break;
                            }
                            _ => {
                                // Ignore other events during command execution
                            }
                        }
                    }
                    Ok::<_, anyhow::Error>(())
                })
                .await;

                let execution_time_ms = start.elapsed().as_millis() as u64;

                let response = match result {
                    Ok(Ok(())) => Ok(ShellCommandResponse {
                        command: req.command.clone(),
                        output,
                        success: true,
                        error: None,
                        execution_time_ms,
                    }),
                    Ok(Err(e)) => Err(e),
                    Err(_) => Err(anyhow::anyhow!("Command execution timed out")),
                };

                let _ = req.response_tx.send(response);
            }
        });

        Self { command_tx }
    }

    /// Execute a shell command
    pub async fn execute(&self, request: ShellCommandRequest) -> Result<ShellCommandResponse> {
        let (response_tx, response_rx) = oneshot::channel();

        let req = CommandRequest {
            command: request.command.clone(),
            timeout_ms: request.timeout_ms,
            response_tx,
        };

        self.command_tx
            .send(req)
            .context("Failed to queue command")?;

        response_rx
            .await
            .context("Command executor dropped")?
    }

    /// Check if executor is available
    pub fn is_available(&self) -> bool {
        !self.command_tx.is_closed()
    }
}
