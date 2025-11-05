//! Tauri backend for SIS Kernel Desktop App
//!
//! Provides bridge commands for the frontend to interact with:
//! - Local daemon (auto-launch if not running)
//! - System operations (file dialogs, etc.)

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use std::process::Command as StdCommand;
use tauri::Manager;

const DAEMON_URL: &str = "http://localhost:8871";

#[derive(Debug, Serialize, Deserialize)]
struct DaemonStatus {
    healthy: bool,
    version: Option<String>,
}

/// Check if daemon is running
#[tauri::command]
async fn check_daemon() -> Result<DaemonStatus, String> {
    let client = reqwest::Client::new();
    match client
        .get(format!("{}/health", DAEMON_URL))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                let health: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
                Ok(DaemonStatus {
                    healthy: true,
                    version: health
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                })
            } else {
                Ok(DaemonStatus {
                    healthy: false,
                    version: None,
                })
            }
        }
        Err(_) => Ok(DaemonStatus {
            healthy: false,
            version: None,
        }),
    }
}

/// Launch daemon if not running
#[tauri::command]
async fn launch_daemon() -> Result<String, String> {
    // Check if already running
    let status = check_daemon().await?;
    if status.healthy {
        return Ok("Daemon already running".to_string());
    }

    // Find daemon binary
    // In development: cargo run -p sisctl
    // In production: bundled binary or system-installed sisctl

    // For development, spawn daemon via cargo
    #[cfg(debug_assertions)]
    {
        let child = StdCommand::new("cargo")
            .args(["run", "-p", "sisctl"])
            .spawn()
            .map_err(|e| format!("Failed to spawn daemon: {}", e))?;

        // Give it time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(format!("Daemon launched (PID: {})", child.id()))
    }

    #[cfg(not(debug_assertions))]
    {
        // In production, try to find sisctl binary
        let child = StdCommand::new("sisctl")
            .spawn()
            .map_err(|e| format!("Failed to spawn daemon: {}. Make sure sisctl is installed.", e))?;

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(format!("Daemon launched (PID: {})", child.id()))
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![check_daemon, launch_daemon])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
