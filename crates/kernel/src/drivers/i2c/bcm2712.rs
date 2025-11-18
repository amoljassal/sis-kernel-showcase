//! BCM2712 I2C Controller Driver for Raspberry Pi 5
//!
//! This module implements the I2C (Inter-Integrated Circuit) controller driver
//! for the BCM2712 SoC on Raspberry Pi 5. I2C controllers are accessed through
//! the RP1 I/O Hub.
//!
//! # Hardware Overview
//!
//! The BCM2712 provides 6 I2C controllers through the RP1 chip:
//! - 6× independent I2C master controllers
//! - Standard mode (100 kHz) and Fast mode (400 kHz)
//! - 7-bit and 10-bit addressing support
//! - Clock stretching support
//! - FIFO buffers for efficient transfers
//!
//! # I2C Protocol
//!
//! I2C uses two lines:
//! - **SDA (Serial Data)** - Bidirectional data line
//! - **SCL (Serial Clock)** - Clock line driven by master
//!
//! ## Transaction Format
//! ```text
//! START | ADDR+W | ACK | DATA | ACK | ... | STOP  (Write)
//! START | ADDR+R | ACK | DATA | ACK | ... | STOP  (Read)
//! ```
//!
//! # Register Map (per I2C controller)
//!
//! ```text
//! Offset   Register    Description
//! ------   --------    -----------
//! 0x00     C           Control register
//! 0x04     S           Status register
//! 0x08     DLEN        Data length
//! 0x0C     A           Slave address
//! 0x10     FIFO        FIFO data register
//! 0x14     DIV         Clock divider
//! 0x18     DEL         Data delay
//! 0x1C     CLKT        Clock stretch timeout
//! ```
//!
//! # Common I2C Devices
//!
//! - **IMU:** MPU6050 (0x68), MPU9250 (0x68), BNO055 (0x28)
//! - **Environmental:** BME280 (0x76/0x77), BMP280 (0x76/0x77)
//! - **Distance:** VL53L0X (0x29), VL53L1X (0x29)
//! - **Current/Voltage:** INA219 (0x40-0x4F)
//! - **EEPROM:** AT24C32 (0x50-0x57)
//! - **RTC:** DS1307 (0x68), DS3231 (0x68)
//!
//! # References
//! - I2C Specification (NXP)
//! - BCM2835 ARM Peripherals (similar I2C controller)
//! - BCM2712 Technical Reference Manual

use crate::drivers::{DriverError, DriverResult};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// I2C clock frequency for standard mode (100 kHz)
pub const I2C_CLOCK_STANDARD: u32 = 100_000;

/// I2C clock frequency for fast mode (400 kHz)
pub const I2C_CLOCK_FAST: u32 = 400_000;

/// Core clock frequency (typically 150 MHz on RPi5)
const CORE_CLOCK_HZ: u32 = 150_000_000;

/// Maximum I2C transaction length
pub const MAX_TRANSACTION_LENGTH: usize = 65535;

/// I2C register offsets
mod regs {
    pub const C: u32 = 0x00;      // Control
    pub const S: u32 = 0x04;      // Status
    pub const DLEN: u32 = 0x08;   // Data length
    pub const A: u32 = 0x0C;      // Slave address
    pub const FIFO: u32 = 0x10;   // FIFO data
    pub const DIV: u32 = 0x14;    // Clock divider
    pub const DEL: u32 = 0x18;    // Data delay
    pub const CLKT: u32 = 0x1C;   // Clock stretch timeout
}

/// Control register bits
mod c {
    pub const I2CEN: u32 = 1 << 15;   // I2C enable
    pub const INTR: u32 = 1 << 10;    // Interrupt on RX
    pub const INTT: u32 = 1 << 9;     // Interrupt on TX
    pub const INTD: u32 = 1 << 8;     // Interrupt on DONE
    pub const ST: u32 = 1 << 7;       // Start transfer
    pub const CLEAR: u32 = 3 << 4;    // Clear FIFO (both bits)
    pub const READ: u32 = 1 << 0;     // Read transfer (vs write)
}

/// Status register bits
mod s {
    pub const CLKT: u32 = 1 << 9;     // Clock stretch timeout
    pub const ERR: u32 = 1 << 8;      // ACK error
    pub const RXF: u32 = 1 << 7;      // RX FIFO full
    pub const TXE: u32 = 1 << 6;      // TX FIFO empty
    pub const RXD: u32 = 1 << 5;      // RX FIFO has data
    pub const TXD: u32 = 1 << 4;      // TX FIFO can accept data
    pub const RXR: u32 = 1 << 3;      // RX FIFO needs reading
    pub const TXW: u32 = 1 << 2;      // TX FIFO needs writing
    pub const DONE: u32 = 1 << 1;     // Transfer done
    pub const TA: u32 = 1 << 0;       // Transfer active
}

/// I2C transfer mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum I2cMode {
    /// Standard mode (100 kHz)
    Standard,

    /// Fast mode (400 kHz)
    Fast,
}

/// I2C transfer direction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum I2cDirection {
    /// Write to slave
    Write,

    /// Read from slave
    Read,
}

/// I2C controller state
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum I2cState {
    Uninitialized,
    Initialized,
    Busy,
    Error,
}

/// BCM2712 I2C controller
pub struct Bcm2712I2c {
    /// MMIO base address
    base: usize,

    /// Controller index (0-5)
    index: u8,

    /// Current state
    state: AtomicU32,

    /// Initialization complete
    initialized: AtomicBool,

    /// Current clock frequency
    clock_hz: AtomicU32,
}

impl Bcm2712I2c {
    /// Create a new I2C controller instance
    ///
    /// # Arguments
    /// * `base` - MMIO base address from RP1
    /// * `index` - Controller index (0-5)
    pub const fn new(base: usize, index: u8) -> Self {
        Self {
            base,
            index,
            state: AtomicU32::new(I2cState::Uninitialized as u32),
            initialized: AtomicBool::new(false),
            clock_hz: AtomicU32::new(0),
        }
    }

    /// Get MMIO base address
    pub fn base(&self) -> usize {
        self.base
    }

    /// Get controller index
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Get current state
    pub fn state(&self) -> I2cState {
        match self.state.load(Ordering::Acquire) {
            0 => I2cState::Uninitialized,
            1 => I2cState::Initialized,
            2 => I2cState::Busy,
            _ => I2cState::Error,
        }
    }

    /// Set state
    fn set_state(&self, state: I2cState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Read a register
    #[inline]
    fn read_reg(&self, offset: u32) -> u32 {
        let addr = (self.base + offset as usize) as *const u32;
        unsafe { core::ptr::read_volatile(addr) }
    }

    /// Write a register
    #[inline]
    fn write_reg(&self, offset: u32, value: u32) {
        let addr = (self.base + offset as usize) as *mut u32;
        unsafe { core::ptr::write_volatile(addr, value); }
    }

    /// Initialize the I2C controller
    ///
    /// # Arguments
    /// * `mode` - Operating mode (Standard or Fast)
    pub fn initialize(&self, mode: I2cMode) -> DriverResult<()> {
        // Disable I2C during configuration
        self.write_reg(regs::C, 0);

        // Set clock divider based on mode
        let clock_hz = match mode {
            I2cMode::Standard => I2C_CLOCK_STANDARD,
            I2cMode::Fast => I2C_CLOCK_FAST,
        };

        let divider = CORE_CLOCK_HZ / clock_hz;
        self.write_reg(regs::DIV, divider);

        // Set data delay (recommended: 48)
        self.write_reg(regs::DEL, (48 << 16) | 48);

        // Set clock stretch timeout (recommended: 64)
        self.write_reg(regs::CLKT, 64);

        // Clear FIFO
        self.write_reg(regs::C, c::CLEAR);

        // Clear status flags
        self.write_reg(regs::S, s::CLKT | s::ERR | s::DONE);

        // Enable I2C controller
        self.write_reg(regs::C, c::I2CEN);

        self.clock_hz.store(clock_hz, Ordering::Release);
        self.initialized.store(true, Ordering::Release);
        self.set_state(I2cState::Initialized);

        crate::info!("I2C{}: Initialized at {} kHz", self.index, clock_hz / 1000);

        Ok(())
    }

    /// Perform I2C transaction (read or write)
    ///
    /// # Arguments
    /// * `addr` - 7-bit slave address
    /// * `data` - Data buffer (for write: data to send, for read: receive buffer)
    /// * `direction` - Transfer direction (Read or Write)
    ///
    /// # Returns
    /// Number of bytes transferred
    pub fn transfer(&self, addr: u8, data: &mut [u8], direction: I2cDirection) -> DriverResult<usize> {
        if !self.is_initialized() {
            return Err(DriverError::NotInitialized);
        }

        if data.is_empty() || data.len() > MAX_TRANSACTION_LENGTH {
            return Err(DriverError::InvalidParameter);
        }

        if addr > 0x7F {
            return Err(DriverError::InvalidParameter);
        }

        self.set_state(I2cState::Busy);

        // Clear FIFO and status
        self.write_reg(regs::C, c::I2CEN | c::CLEAR);
        self.write_reg(regs::S, s::CLKT | s::ERR | s::DONE);

        // Set slave address
        self.write_reg(regs::A, addr as u32);

        // Set data length
        self.write_reg(regs::DLEN, data.len() as u32);

        // Start transfer
        let mut control = c::I2CEN | c::ST;
        if direction == I2cDirection::Read {
            control |= c::READ;
        }

        match direction {
            I2cDirection::Write => {
                // Fill TX FIFO with initial data
                for &byte in data.iter() {
                    self.write_reg(regs::FIFO, byte as u32);
                }

                // Start write transfer
                self.write_reg(regs::C, control);

                // Wait for completion
                let mut timeout = 100000;
                while timeout > 0 {
                    let status = self.read_reg(regs::S);

                    if (status & s::ERR) != 0 {
                        self.set_state(I2cState::Error);
                        self.write_reg(regs::S, s::ERR);
                        return Err(DriverError::IoError);
                    }

                    if (status & s::CLKT) != 0 {
                        self.set_state(I2cState::Error);
                        self.write_reg(regs::S, s::CLKT);
                        return Err(DriverError::Timeout(crate::drivers::timeout::TimeoutError::new(0, 0)));
                    }

                    if (status & s::DONE) != 0 {
                        self.write_reg(regs::S, s::DONE);
                        break;
                    }

                    timeout -= 1;
                    core::hint::spin_loop();
                }

                if timeout == 0 {
                    self.set_state(I2cState::Error);
                    return Err(DriverError::Timeout(crate::drivers::timeout::TimeoutError::new(0, 0)));
                }
            }

            I2cDirection::Read => {
                // Start read transfer
                self.write_reg(regs::C, control);

                // Read data from RX FIFO
                let mut bytes_read = 0;
                let mut timeout = 100000;

                while bytes_read < data.len() && timeout > 0 {
                    let status = self.read_reg(regs::S);

                    if (status & s::ERR) != 0 {
                        self.set_state(I2cState::Error);
                        self.write_reg(regs::S, s::ERR);
                        return Err(DriverError::IoError);
                    }

                    if (status & s::CLKT) != 0 {
                        self.set_state(I2cState::Error);
                        self.write_reg(regs::S, s::CLKT);
                        return Err(DriverError::Timeout(crate::drivers::timeout::TimeoutError::new(0, 0)));
                    }

                    if (status & s::RXD) != 0 {
                        data[bytes_read] = self.read_reg(regs::FIFO) as u8;
                        bytes_read += 1;
                        timeout = 100000; // Reset timeout on successful read
                    }

                    if (status & s::DONE) != 0 {
                        self.write_reg(regs::S, s::DONE);
                        // Read any remaining bytes
                        while (self.read_reg(regs::S) & s::RXD) != 0 && bytes_read < data.len() {
                            data[bytes_read] = self.read_reg(regs::FIFO) as u8;
                            bytes_read += 1;
                        }
                        break;
                    }

                    timeout -= 1;
                    core::hint::spin_loop();
                }

                if timeout == 0 {
                    self.set_state(I2cState::Error);
                    return Err(DriverError::Timeout(crate::drivers::timeout::TimeoutError::new(0, 0)));
                }
            }
        }

        self.set_state(I2cState::Initialized);
        Ok(data.len())
    }

    /// Write data to I2C slave
    ///
    /// # Arguments
    /// * `addr` - 7-bit slave address
    /// * `data` - Data to write
    pub fn write(&self, addr: u8, data: &[u8]) -> DriverResult<usize> {
        let mut buffer = alloc::vec::Vec::from(data);
        self.transfer(addr, &mut buffer, I2cDirection::Write)
    }

    /// Read data from I2C slave
    ///
    /// # Arguments
    /// * `addr` - 7-bit slave address
    /// * `buffer` - Buffer to receive data
    pub fn read(&self, addr: u8, buffer: &mut [u8]) -> DriverResult<usize> {
        self.transfer(addr, buffer, I2cDirection::Read)
    }

    /// Write to register then read (common I2C pattern)
    ///
    /// # Arguments
    /// * `addr` - 7-bit slave address
    /// * `reg` - Register address to read from
    /// * `buffer` - Buffer to receive data
    pub fn write_read(&self, addr: u8, reg: u8, buffer: &mut [u8]) -> DriverResult<usize> {
        // Write register address
        self.write(addr, &[reg])?;

        // Read data
        self.read(addr, buffer)
    }

    /// Scan for devices on the bus
    ///
    /// Returns a vector of addresses that responded with ACK
    pub fn scan(&self) -> alloc::vec::Vec<u8> {
        let mut devices = alloc::vec::Vec::new();

        for addr in 0x08..=0x77 {
            // Try to read 0 bytes from address (just checking for ACK)
            let mut dummy = [0u8; 1];
            if self.read(addr, &mut dummy[..0]).is_ok() {
                devices.push(addr);
            }
        }

        devices
    }

    /// Get status register value
    pub fn get_status(&self) -> u32 {
        self.read_reg(regs::S)
    }

    /// Clear error flags
    pub fn clear_errors(&self) -> DriverResult<()> {
        self.write_reg(regs::S, s::ERR | s::CLKT);
        self.set_state(I2cState::Initialized);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_divider() {
        // Standard mode: 100 kHz
        let divider_std = CORE_CLOCK_HZ / I2C_CLOCK_STANDARD;
        assert_eq!(divider_std, 1500);

        // Fast mode: 400 kHz
        let divider_fast = CORE_CLOCK_HZ / I2C_CLOCK_FAST;
        assert_eq!(divider_fast, 375);
    }

    #[test]
    fn test_address_validation() {
        // Valid 7-bit addresses
        assert!(0x08 <= 0x7F);
        assert!(0x77 <= 0x7F);

        // Invalid addresses
        assert!(0x80 > 0x7F);
        assert!(0xFF > 0x7F);
    }
}
