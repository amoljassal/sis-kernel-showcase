// QEMU Backend Implementation
// Wraps existing QEMURuntimeManager to conform to HardwareBackend trait

use crate::hardware_backend::{HardwareBackend, BackendType, BackendResult, BackendConfig};
use crate::{TestError, qemu_runtime::QEMURuntimeManager};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// QEMU-based hardware backend
pub struct QemuBackend {
    /// Unique ID for this backend instance
    id: String,

    /// Node ID in QEMU cluster
    node_id: usize,

    /// Shared QEMU runtime manager
    manager: Arc<Mutex<QEMURuntimeManager>>,

    /// Path to serial log file
    serial_log_path: String,

    /// Last position in serial log (for incremental reads)
    last_log_position: u64,

    /// Configuration
    config: BackendConfig,

    /// Whether backend is initialized
    initialized: bool,
}

impl QemuBackend {
    /// Create a new QEMU backend
    pub fn new(
        node_id: usize,
        manager: Arc<Mutex<QEMURuntimeManager>>,
        serial_log_path: String,
        config: BackendConfig,
    ) -> Self {
        let id = format!("qemu-node{}", node_id);

        Self {
            id,
            node_id,
            manager,
            serial_log_path,
            last_log_position: 0,
            config,
            initialized: false,
        }
    }

    /// Read serial log from last position
    async fn read_log_incremental(&mut self) -> BackendResult<String> {
        let content = fs::read_to_string(&self.serial_log_path)
            .await
            .map_err(TestError::IoError)?;

        let bytes = content.as_bytes();
        let start = self.last_log_position as usize;

        if start >= bytes.len() {
            return Ok(String::new());
        }

        let new_content = String::from_utf8_lossy(&bytes[start..]).to_string();
        self.last_log_position = bytes.len() as u64;

        Ok(new_content)
    }

    /// Wait for specific pattern in serial log
    #[allow(dead_code)]
    async fn wait_for_pattern(&mut self, pattern: &str, timeout: Duration) -> BackendResult<()> {
        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline {
            let content = fs::read_to_string(&self.serial_log_path)
                .await
                .map_err(TestError::IoError)?;

            if content.contains(pattern) {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(TestError::TimeoutError {
            message: format!("Timeout waiting for pattern: {}", pattern),
        })
    }
}

#[async_trait]
impl HardwareBackend for QemuBackend {
    fn backend_id(&self) -> String {
        self.id.clone()
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Qemu
    }

    async fn initialize(&mut self) -> BackendResult<()> {
        if self.initialized {
            return Ok(());
        }

        if self.config.verbose {
            log::info!("[{}] Initializing QEMU backend", self.id);
        }

        // QEMU manager handles the actual instance launch
        // We just mark ourselves as initialized
        self.initialized = true;
        self.last_log_position = 0;

        Ok(())
    }

    async fn shutdown(&mut self) -> BackendResult<()> {
        if !self.initialized {
            return Ok(());
        }

        if self.config.verbose {
            log::info!("[{}] Shutting down QEMU backend", self.id);
        }

        // QEMU manager handles cluster shutdown
        // We just mark ourselves as uninitialized
        self.initialized = false;

        Ok(())
    }

    async fn wait_for_ready(&mut self, timeout: Duration) -> BackendResult<()> {
        if !self.initialized {
            return Err(TestError::QEMUError {
                message: "Backend not initialized".to_string(),
            });
        }

        if self.config.verbose {
            log::info!("[{}] Waiting for kernel ready (timeout: {:?})", self.id, timeout);
        }

        // Wait for shell prompt patterns
        let patterns = [
            "[MAIN] STARTING FULL SHELL",
            "LAUNCHING SHELL",
            "sis>",
            "Shell ready",
        ];

        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline {
            let content = fs::read_to_string(&self.serial_log_path)
                .await
                .map_err(TestError::IoError)?;

            for pattern in &patterns {
                if content.contains(pattern) {
                    if self.config.verbose {
                        log::info!("[{}] Kernel ready (found pattern: {})", self.id, pattern);
                    }
                    return Ok(());
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(TestError::TimeoutError {
            message: format!("Timeout waiting for kernel ready on {}", self.id),
        })
    }

    async fn send_command(&mut self, command: &str) -> BackendResult<()> {
        if !self.initialized {
            return Err(TestError::QEMUError {
                message: "Backend not initialized".to_string(),
            });
        }

        if self.config.verbose {
            log::debug!("[{}] Sending command: {}", self.id, command);
        }

        // Get stdin writer from manager
        let manager = self.manager.lock().await;
        let writers = manager.serial_writers();
        let mut writers_lock = writers.lock().await;

        if let Some(stdin) = writers_lock.get_mut(&self.node_id) {
            // Send command with newline
            let cmd_with_newline = format!("{}\n", command);
            stdin.write_all(cmd_with_newline.as_bytes())
                .await
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to write command: {}", e),
                })?;

            stdin.flush().await.map_err(|e| TestError::QEMUError {
                message: format!("Failed to flush command: {}", e),
            })?;

            Ok(())
        } else {
            Err(TestError::QEMUError {
                message: format!("No stdin writer for node {}", self.node_id),
            })
        }
    }

    async fn read_output(&mut self) -> BackendResult<String> {
        if !self.initialized {
            return Err(TestError::QEMUError {
                message: "Backend not initialized".to_string(),
            });
        }

        self.read_log_incremental().await
    }

    async fn read_output_with_timeout(&mut self, timeout: Duration) -> BackendResult<String> {
        if !self.initialized {
            return Err(TestError::QEMUError {
                message: "Backend not initialized".to_string(),
            });
        }

        // Wait a bit for output to accumulate
        tokio::time::sleep(timeout).await;

        self.read_log_incremental().await
    }

    async fn is_alive(&self) -> bool {
        if !self.initialized {
            return false;
        }

        // Check if serial log file exists and is being updated
        if let Ok(metadata) = fs::metadata(&self.serial_log_path).await {
            metadata.is_file()
        } else {
            false
        }
    }

    fn log_path(&self) -> Option<String> {
        Some(self.serial_log_path.clone())
    }

    async fn reset(&mut self) -> BackendResult<()> {
        // For QEMU, we need to restart the instance
        // This is a placeholder - full implementation would require
        // shutting down and restarting the QEMU instance

        if self.config.verbose {
            log::info!("[{}] Resetting QEMU instance (not yet implemented)", self.id);
        }

        Err(TestError::QEMUError {
            message: "QEMU reset not yet implemented".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestSuiteConfig;

    #[tokio::test]
    async fn test_qemu_backend_creation() {
        let config = BackendConfig::default();
        let test_config = TestSuiteConfig::default();
        let manager = Arc::new(Mutex::new(QEMURuntimeManager::new(&test_config)));

        let backend = QemuBackend::new(
            0,
            manager,
            "/tmp/test-serial.log".to_string(),
            config,
        );

        assert_eq!(backend.backend_id(), "qemu-node0");
        assert_eq!(backend.backend_type(), BackendType::Qemu);
        assert!(!backend.initialized);
    }

    #[tokio::test]
    async fn test_qemu_backend_initialize() {
        let config = BackendConfig::default();
        let test_config = TestSuiteConfig::default();
        let manager = Arc::new(Mutex::new(QEMURuntimeManager::new(&test_config)));

        let mut backend = QemuBackend::new(
            0,
            manager,
            "/tmp/test-serial.log".to_string(),
            config,
        );

        assert!(backend.initialize().await.is_ok());
        assert!(backend.initialized);
    }
}
