//! Common driver error types
//!
//! Part of M8 Driver Hardening - provides comprehensive error handling
//! for all driver operations.

use super::timeout::TimeoutError;

/// Common driver error type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DriverError {
    /// Operation timed out
    Timeout(TimeoutError),

    /// Hardware not initialized
    NotInitialized,

    /// Invalid parameter or argument
    InvalidParameter,

    /// Invalid address or out of bounds
    InvalidAddress,

    /// Hardware error or fault
    HardwareError,

    /// Operation not supported
    NotSupported,

    /// Resource busy or in use
    Busy,

    /// I/O error
    IoError,

    /// Buffer too small
    BufferTooSmall,

    /// Invalid state for operation
    InvalidState,

    /// Permission denied
    PermissionDenied,

    /// Device not found
    DeviceNotFound,

    /// Alignment error
    AlignmentError,

    /// Checksum or verification failed
    VerificationFailed,
}

impl DriverError {
    /// Get error code for logging
    pub fn code(&self) -> u32 {
        match self {
            Self::Timeout(_) => 1,
            Self::NotInitialized => 2,
            Self::InvalidParameter => 3,
            Self::InvalidAddress => 4,
            Self::HardwareError => 5,
            Self::NotSupported => 6,
            Self::Busy => 7,
            Self::IoError => 8,
            Self::BufferTooSmall => 9,
            Self::InvalidState => 10,
            Self::PermissionDenied => 11,
            Self::DeviceNotFound => 12,
            Self::AlignmentError => 13,
            Self::VerificationFailed => 14,
        }
    }

    /// Get error name for logging
    pub fn name(&self) -> &'static str {
        match self {
            Self::Timeout(_) => "Timeout",
            Self::NotInitialized => "NotInitialized",
            Self::InvalidParameter => "InvalidParameter",
            Self::InvalidAddress => "InvalidAddress",
            Self::HardwareError => "HardwareError",
            Self::NotSupported => "NotSupported",
            Self::Busy => "Busy",
            Self::IoError => "IoError",
            Self::BufferTooSmall => "BufferTooSmall",
            Self::InvalidState => "InvalidState",
            Self::PermissionDenied => "PermissionDenied",
            Self::DeviceNotFound => "DeviceNotFound",
            Self::AlignmentError => "AlignmentError",
            Self::VerificationFailed => "VerificationFailed",
        }
    }
}

impl From<TimeoutError> for DriverError {
    fn from(err: TimeoutError) -> Self {
        DriverError::Timeout(err)
    }
}

/// Result type for driver operations
pub type DriverResult<T> = Result<T, DriverError>;

/// Input validation helper
pub struct Validator;

impl Validator {
    /// Validate address alignment
    ///
    /// # Arguments
    /// * `addr` - Address to check
    /// * `alignment` - Required alignment (must be power of 2)
    ///
    /// # Returns
    /// `Ok(())` if aligned, `Err(DriverError::AlignmentError)` otherwise
    pub fn check_alignment(addr: usize, alignment: usize) -> DriverResult<()> {
        if alignment == 0 || (alignment & (alignment - 1)) != 0 {
            return Err(DriverError::InvalidParameter);
        }
        if addr & (alignment - 1) != 0 {
            return Err(DriverError::AlignmentError);
        }
        Ok(())
    }

    /// Validate buffer size
    pub fn check_buffer_size(buffer_size: usize, required_size: usize) -> DriverResult<()> {
        if buffer_size < required_size {
            return Err(DriverError::BufferTooSmall);
        }
        Ok(())
    }

    /// Validate range is within bounds
    pub fn check_bounds(value: usize, min: usize, max: usize) -> DriverResult<()> {
        if value < min || value > max {
            return Err(DriverError::InvalidAddress);
        }
        Ok(())
    }

    /// Validate non-null pointer
    pub fn check_non_null<T>(ptr: *const T) -> DriverResult<()> {
        if ptr.is_null() {
            return Err(DriverError::InvalidParameter);
        }
        Ok(())
    }

    /// Validate GPIO pin number
    pub fn check_gpio_pin(pin: u32, max_pin: u32) -> DriverResult<()> {
        if pin >= max_pin {
            return Err(DriverError::InvalidParameter);
        }
        Ok(())
    }

    /// Validate register offset
    pub fn check_register_offset(offset: usize, max_offset: usize) -> DriverResult<()> {
        if offset > max_offset {
            return Err(DriverError::InvalidAddress);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment_check() {
        assert!(Validator::check_alignment(0x1000, 16).is_ok());
        assert!(Validator::check_alignment(0x1001, 16).is_err());
    }

    #[test]
    fn test_buffer_size_check() {
        assert!(Validator::check_buffer_size(1024, 512).is_ok());
        assert!(Validator::check_buffer_size(256, 512).is_err());
    }

    #[test]
    fn test_bounds_check() {
        assert!(Validator::check_bounds(50, 0, 100).is_ok());
        assert!(Validator::check_bounds(150, 0, 100).is_err());
    }
}
