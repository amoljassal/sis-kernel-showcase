// Serial Backend Implementation
// Communicates with real hardware (Raspberry Pi 5) via UART/serial port

use crate::hardware_backend::{HardwareBackend, BackendType, BackendResult, BackendConfig};
use crate::TestError;
use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use nix::sys::termios::{self, SetArg};

/// Serial port backend for real hardware testing
pub struct SerialBackend {
    /// Unique ID for this backend instance
    id: String,

    /// Serial device path (e.g., /dev/ttyUSB0, /dev/ttyACM0)
    device_path: String,

    /// Baud rate for serial communication
    baud_rate: u32,

    /// Serial port file descriptor (when open)
    serial_fd: Option<tokio::fs::File>,

    /// Buffer for accumulated output
    output_buffer: String,

    /// Configuration
    config: BackendConfig,

    /// Whether backend is initialized
    initialized: bool,
}

impl SerialBackend {
    /// Create a new serial backend
    pub fn new(
        device_path: String,
        baud_rate: u32,
        config: BackendConfig,
    ) -> Self {
        let id = format!("serial-{}", device_path.replace('/', "-"));

        Self {
            id,
            device_path,
            baud_rate,
            serial_fd: None,
            output_buffer: String::new(),
            config,
            initialized: false,
        }
    }

    /// Configure serial port for raw mode communication
    fn configure_serial_port(file: &tokio::fs::File, baud_rate: u32) -> Result<(), TestError> {
        // Get current terminal settings
        let mut termios = termios::tcgetattr(file).map_err(|e| TestError::SerialError {
            message: format!("Failed to get terminal attributes: {}", e),
        })?;

        // Set baud rate
        let speed = match baud_rate {
            9600 => termios::BaudRate::B9600,
            19200 => termios::BaudRate::B19200,
            38400 => termios::BaudRate::B38400,
            57600 => termios::BaudRate::B57600,
            115200 => termios::BaudRate::B115200,
            230400 => termios::BaudRate::B230400,
            _ => {
                return Err(TestError::SerialError {
                    message: format!("Unsupported baud rate: {}", baud_rate),
                });
            }
        };

        termios::cfsetispeed(&mut termios, speed).map_err(|e| TestError::SerialError {
            message: format!("Failed to set input baud rate: {}", e),
        })?;

        termios::cfsetospeed(&mut termios, speed).map_err(|e| TestError::SerialError {
            message: format!("Failed to set output baud rate: {}", e),
        })?;

        // Configure for raw mode (8N1, no flow control)
        termios::cfmakeraw(&mut termios);

        // Apply settings
        termios::tcsetattr(file, SetArg::TCSANOW, &termios).map_err(|e| TestError::SerialError {
            message: format!("Failed to set terminal attributes: {}", e),
        })?;

        Ok(())
    }

    /// Read available data from serial port (non-blocking)
    async fn read_available(&mut self) -> BackendResult<String> {
        if let Some(ref mut file) = self.serial_fd {
            let mut buf = vec![0u8; 4096];

            match tokio::time::timeout(
                Duration::from_millis(100),
                file.read(&mut buf)
            ).await {
                Ok(Ok(n)) if n > 0 => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    self.output_buffer.push_str(&data);
                    Ok(data)
                }
                Ok(Ok(_)) => Ok(String::new()), // No data available
                Ok(Err(e)) => Err(TestError::SerialError {
                    message: format!("Serial read error: {}", e),
                }),
                Err(_) => Ok(String::new()), // Timeout - no data available
            }
        } else {
            Err(TestError::SerialError {
                message: "Serial port not open".to_string(),
            })
        }
    }
}

#[async_trait]
impl HardwareBackend for SerialBackend {
    fn backend_id(&self) -> String {
        self.id.clone()
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Serial
    }

    async fn initialize(&mut self) -> BackendResult<()> {
        if self.initialized {
            return Ok(());
        }

        if self.config.verbose {
            log::info!("[{}] Opening serial port {} at {} baud",
                      self.id, self.device_path, self.baud_rate);
        }

        // Open serial device
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.device_path)
            .await
            .map_err(|e| TestError::SerialError {
                message: format!("Failed to open {}: {}", self.device_path, e),
            })?;

        // Configure serial port
        Self::configure_serial_port(&file, self.baud_rate)?;

        self.serial_fd = Some(file);
        self.initialized = true;
        self.output_buffer.clear();

        if self.config.verbose {
            log::info!("[{}] Serial port configured successfully", self.id);
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> BackendResult<()> {
        if !self.initialized {
            return Ok(());
        }

        if self.config.verbose {
            log::info!("[{}] Closing serial port", self.id);
        }

        self.serial_fd = None;
        self.initialized = false;

        Ok(())
    }

    async fn wait_for_ready(&mut self, timeout: Duration) -> BackendResult<()> {
        if !self.initialized {
            return Err(TestError::SerialError {
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
            "boot:",  // RPi5 bootloader prompt
        ];

        let deadline = Instant::now() + timeout;

        while Instant::now() < deadline {
            // Read available data
            self.read_available().await?;

            // Check for patterns in accumulated buffer
            for pattern in &patterns {
                if self.output_buffer.contains(pattern) {
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
            return Err(TestError::SerialError {
                message: "Backend not initialized".to_string(),
            });
        }

        if self.config.verbose {
            log::debug!("[{}] Sending command: {}", self.id, command);
        }

        if let Some(ref mut file) = self.serial_fd {
            // Send command with newline
            let cmd_with_newline = format!("{}\n", command);
            file.write_all(cmd_with_newline.as_bytes())
                .await
                .map_err(|e| TestError::SerialError {
                    message: format!("Failed to write command: {}", e),
                })?;

            file.flush().await.map_err(|e| TestError::SerialError {
                message: format!("Failed to flush command: {}", e),
            })?;

            Ok(())
        } else {
            Err(TestError::SerialError {
                message: "Serial port not open".to_string(),
            })
        }
    }

    async fn read_output(&mut self) -> BackendResult<String> {
        if !self.initialized {
            return Err(TestError::SerialError {
                message: "Backend not initialized".to_string(),
            });
        }

        // Read and return new data
        self.read_available().await
    }

    async fn read_output_with_timeout(&mut self, timeout: Duration) -> BackendResult<String> {
        if !self.initialized {
            return Err(TestError::SerialError {
                message: "Backend not initialized".to_string(),
            });
        }

        let deadline = Instant::now() + timeout;
        let mut accumulated = String::new();

        while Instant::now() < deadline {
            let data = self.read_available().await?;
            accumulated.push_str(&data);

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Ok(accumulated)
    }

    async fn is_alive(&self) -> bool {
        self.initialized && self.serial_fd.is_some()
    }

    fn log_path(&self) -> Option<String> {
        // Serial backend doesn't log to file by default
        // Could be enhanced to optionally log to file
        None
    }

    async fn reset(&mut self) -> BackendResult<()> {
        // For real hardware, reset would require:
        // 1. Sending reboot command to kernel
        // 2. Or toggling GPIO reset line
        // 3. Or power cycling via smart PDU

        if self.config.verbose {
            log::info!("[{}] Hardware reset not implemented", self.id);
        }

        // Send reboot command as best-effort
        self.send_command("reboot").await?;

        // Wait for reboot
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Re-initialize to handle any serial port changes
        self.shutdown().await?;
        self.initialize().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_backend_creation() {
        let config = BackendConfig::default();
        let backend = SerialBackend::new(
            "/dev/ttyUSB0".to_string(),
            115200,
            config,
        );

        assert_eq!(backend.backend_type(), BackendType::Serial);
        assert!(!backend.initialized);
        assert_eq!(backend.baud_rate, 115200);
    }
}
