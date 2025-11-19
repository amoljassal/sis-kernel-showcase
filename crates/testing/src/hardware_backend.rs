// Hardware Backend Abstraction for SIS Kernel Testing
// Enables testing on both QEMU (emulated) and real hardware (RPi5 via serial)

use crate::TestError;
use async_trait::async_trait;
use std::time::Duration;

/// Result type for backend operations
pub type BackendResult<T> = Result<T, TestError>;

/// Hardware backend trait - abstracts QEMU vs. real hardware communication
#[async_trait]
pub trait HardwareBackend: Send + Sync {
    /// Get a unique identifier for this backend instance
    fn backend_id(&self) -> String;

    /// Get the type of backend (for logging/debugging)
    fn backend_type(&self) -> BackendType;

    /// Initialize the backend (start QEMU, open serial port, etc.)
    async fn initialize(&mut self) -> BackendResult<()>;

    /// Shutdown/cleanup the backend
    async fn shutdown(&mut self) -> BackendResult<()>;

    /// Wait for kernel to reach ready state (shell prompt available)
    async fn wait_for_ready(&mut self, timeout: Duration) -> BackendResult<()>;

    /// Send a command to the kernel shell
    async fn send_command(&mut self, command: &str) -> BackendResult<()>;

    /// Read output from the kernel (since last read or from start)
    async fn read_output(&mut self) -> BackendResult<String>;

    /// Read output with timeout
    async fn read_output_with_timeout(&mut self, timeout: Duration) -> BackendResult<String>;

    /// Execute a command and wait for output (convenience method)
    async fn execute_command(&mut self, command: &str, timeout: Duration) -> BackendResult<String> {
        self.send_command(command).await?;

        // Wait a bit for command to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.read_output_with_timeout(timeout).await
    }

    /// Check if backend is still alive/responsive
    async fn is_alive(&self) -> bool;

    /// Get the path to serial/console log (for debugging)
    fn log_path(&self) -> Option<String>;

    /// Reset the backend to initial state (reboot kernel)
    async fn reset(&mut self) -> BackendResult<()>;
}

/// Backend type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// QEMU emulation (fast, reproducible, no hardware needed)
    Qemu,
    /// Real hardware via serial port (RPi5, accurate timing)
    Serial,
    /// Mock backend for unit testing
    Mock,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::Qemu => write!(f, "QEMU"),
            BackendType::Serial => write!(f, "Serial"),
            BackendType::Mock => write!(f, "Mock"),
        }
    }
}

/// Configuration for creating hardware backends
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Timeout for boot/initialization
    pub boot_timeout: Duration,

    /// Timeout for individual commands
    pub command_timeout: Duration,

    /// Path to kernel binary (for QEMU)
    pub kernel_path: Option<String>,

    /// Serial device path (for Serial backend)
    pub serial_device: Option<String>,

    /// Serial baud rate (for Serial backend)
    pub serial_baud_rate: u32,

    /// QEMU instance ID (for multi-node QEMU clusters)
    pub qemu_node_id: Option<usize>,

    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            boot_timeout: Duration::from_secs(90),
            command_timeout: Duration::from_secs(30),
            kernel_path: None,
            serial_device: None,
            serial_baud_rate: 115200,
            qemu_node_id: None,
            verbose: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_display() {
        assert_eq!(BackendType::Qemu.to_string(), "QEMU");
        assert_eq!(BackendType::Serial.to_string(), "Serial");
        assert_eq!(BackendType::Mock.to_string(), "Mock");
    }

    #[test]
    fn test_backend_config_default() {
        let config = BackendConfig::default();
        assert_eq!(config.boot_timeout, Duration::from_secs(90));
        assert_eq!(config.command_timeout, Duration::from_secs(30));
        assert_eq!(config.serial_baud_rate, 115200);
        assert!(!config.verbose);
    }
}
