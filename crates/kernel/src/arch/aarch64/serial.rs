//! # ARM64 Serial Driver Compatibility Layer
//!
//! This module provides a compatibility layer for the aarch64 architecture
//! to match the x86_64 serial interface. On ARM64, we use the existing UART
//! infrastructure (`uart_print`) which is platform-agnostic.

/// Write a single byte to the serial port
///
/// On ARM64, this uses the global `uart_print` function.
pub fn serial_write_byte(byte: u8) {
    unsafe {
        crate::uart_print(&[byte]);
    }
}

/// Write a byte slice to the serial port
///
/// On ARM64, this uses the global `uart_print` function.
pub fn serial_write(s: &[u8]) {
    unsafe {
        crate::uart_print(s);
    }
}

/// Read bytes from the serial port (non-blocking)
///
/// On ARM64, serial input is not yet implemented.
/// Returns 0 (no bytes read).
pub fn serial_read_bytes(_buf: &mut [u8]) -> usize {
    0
}

/// Read a single byte from the serial port (non-blocking)
///
/// On ARM64, serial input is not yet implemented.
/// Returns None.
pub fn serial_read() -> Option<u8> {
    None
}

/// Get the number of bytes available to read
///
/// On ARM64, serial input is not yet implemented.
/// Returns 0.
pub fn serial_available() -> usize {
    0
}

/// Initialize the serial port driver
///
/// On ARM64, the UART is initialized in the boot sequence.
/// This is a no-op for compatibility with x86_64.
pub unsafe fn init_serial() -> Result<(), &'static str> {
    Ok(())
}

/// Enable interrupt-driven I/O for the serial port
///
/// On ARM64, this is not yet implemented.
/// This is a no-op for compatibility with x86_64.
pub unsafe fn enable_interrupts() {
    // No-op on ARM64 for now
}

/// Check if serial interrupts are enabled
///
/// On ARM64, interrupts are not yet implemented.
pub fn interrupts_enabled() -> bool {
    false
}

/// Handle serial port interrupt
///
/// On ARM64, this is not yet implemented.
pub unsafe fn handle_interrupt() -> usize {
    0
}
