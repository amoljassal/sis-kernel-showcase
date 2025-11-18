//! CAN Bus Driver for Raspberry Pi 5
//!
//! Implements Controller Area Network (CAN) bus support for automotive
//! and industrial applications. Supports CAN 2.0A/B and CAN FD protocols.
//!
//! # CAN Bus Overview
//!
//! CAN (Controller Area Network) is a robust vehicle bus standard designed
//! to allow microcontrollers and devices to communicate without a host computer.
//!
//! ## Features
//! - Multi-master broadcast serial bus
//! - Message-based protocol
//! - Automatic arbitration and error detection
//! - Real-time capable (deterministic timing)
//!
//! # Hardware Support
//!
//! On Raspberry Pi 5, CAN controllers are typically connected via SPI:
//! - MCP2515 CAN controller (classic CAN 2.0)
//! - MCP2517/2518 CAN FD controllers
//! - TJA1050/1051 CAN transceivers
//!
//! # CAN Frame Structure
//!
//! ```text
//! Standard CAN 2.0A (11-bit ID):
//! ┌─────┬─────┬───┬───┬────┬────────────┬─────┬────┬────┐
//! │ SOF │ ID  │RTR│IDE│ r0 │    DLC     │ DATA│ CRC│ EOF│
//! └─────┴─────┴───┴───┴────┴────────────┴─────┴────┴────┘
//!
//! Extended CAN 2.0B (29-bit ID):
//! ┌─────┬──────┬───┬───┬──────┬───┬────┬────────┬────┬────┬────┐
//! │ SOF │ ID-A │SRR│IDE│ ID-B │RTR│ r0 │  DLC   │DATA│CRC │EOF │
//! └─────┴──────┴───┴───┴──────┴───┴────┴────────┴────┴────┴────┘
//! ```

pub mod mcp2515;  // MCP2515 CAN controller driver

use crate::drivers::{DriverError, DriverResult};
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

/// CAN Bus Speed (baud rate)
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CanSpeed {
    /// 10 kbps
    Speed10K = 10_000,
    /// 20 kbps
    Speed20K = 20_000,
    /// 50 kbps
    Speed50K = 50_000,
    /// 100 kbps
    Speed100K = 100_000,
    /// 125 kbps (common automotive)
    Speed125K = 125_000,
    /// 250 kbps (common automotive)
    Speed250K = 250_000,
    /// 500 kbps (common automotive/industrial)
    Speed500K = 500_000,
    /// 800 kbps
    Speed800K = 800_000,
    /// 1 Mbps (maximum for classic CAN)
    Speed1M = 1_000_000,
}

impl CanSpeed {
    pub fn as_bps(&self) -> u32 {
        *self as u32
    }

    pub fn from_bps(bps: u32) -> Option<Self> {
        match bps {
            10_000 => Some(Self::Speed10K),
            20_000 => Some(Self::Speed20K),
            50_000 => Some(Self::Speed50K),
            100_000 => Some(Self::Speed100K),
            125_000 => Some(Self::Speed125K),
            250_000 => Some(Self::Speed250K),
            500_000 => Some(Self::Speed500K),
            800_000 => Some(Self::Speed800K),
            1_000_000 => Some(Self::Speed1M),
            _ => None,
        }
    }
}

/// CAN Frame ID (11-bit or 29-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanId {
    /// Standard 11-bit ID (0x000 - 0x7FF)
    Standard(u16),
    /// Extended 29-bit ID (0x00000000 - 0x1FFFFFFF)
    Extended(u32),
}

impl CanId {
    /// Get raw ID value
    pub fn raw(&self) -> u32 {
        match self {
            CanId::Standard(id) => *id as u32,
            CanId::Extended(id) => *id,
        }
    }

    /// Check if ID is standard (11-bit)
    pub fn is_standard(&self) -> bool {
        matches!(self, CanId::Standard(_))
    }

    /// Check if ID is extended (29-bit)
    pub fn is_extended(&self) -> bool {
        matches!(self, CanId::Extended(_))
    }
}

/// CAN Frame Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// Data frame (carries data payload)
    Data,
    /// Remote Transmission Request (requests data from another node)
    Remote,
    /// Error frame
    Error,
    /// Overload frame
    Overload,
}

/// CAN Frame
#[derive(Debug, Clone)]
pub struct CanFrame {
    /// Frame identifier
    pub id: CanId,

    /// Frame type
    pub frame_type: FrameType,

    /// Data payload (0-8 bytes for CAN 2.0, 0-64 for CAN FD)
    pub data: Vec<u8>,

    /// Timestamp in microseconds
    pub timestamp_us: u64,
}

impl CanFrame {
    /// Create new data frame
    pub fn new_data(id: CanId, data: Vec<u8>) -> DriverResult<Self> {
        if data.len() > 8 {
            return Err(DriverError::InvalidParameter);
        }

        Ok(Self {
            id,
            frame_type: FrameType::Data,
            data,
            timestamp_us: crate::time::get_timestamp_us(),
        })
    }

    /// Create new remote frame
    pub fn new_remote(id: CanId, dlc: u8) -> DriverResult<Self> {
        if dlc > 8 {
            return Err(DriverError::InvalidParameter);
        }

        Ok(Self {
            id,
            frame_type: FrameType::Remote,
            data: alloc::vec![0; dlc as usize],
            timestamp_us: crate::time::get_timestamp_us(),
        })
    }

    /// Get data length code (DLC)
    pub fn dlc(&self) -> u8 {
        self.data.len() as u8
    }

    /// Check if frame is a data frame
    pub fn is_data(&self) -> bool {
        self.frame_type == FrameType::Data
    }

    /// Check if frame is a remote frame
    pub fn is_remote(&self) -> bool {
        self.frame_type == FrameType::Remote
    }

    /// Check if frame is an error frame
    pub fn is_error(&self) -> bool {
        self.frame_type == FrameType::Error
    }
}

/// CAN Bus Operating Mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CanMode {
    /// Normal operating mode
    Normal,
    /// Listen-only mode (no acknowledgment, no error frames)
    ListenOnly,
    /// Loopback mode (for testing)
    Loopback,
    /// Sleep mode (low power)
    Sleep,
    /// Configuration mode
    Configuration,
}

/// CAN Bus Error Counters
#[derive(Debug, Clone, Copy, Default)]
pub struct CanErrorCounters {
    pub tx_errors: u8,
    pub rx_errors: u8,
}

impl CanErrorCounters {
    pub fn is_error_active(&self) -> bool {
        self.tx_errors < 128 && self.rx_errors < 128
    }

    pub fn is_error_passive(&self) -> bool {
        self.tx_errors >= 128 || self.rx_errors >= 128
    }

    pub fn is_bus_off(&self) -> bool {
        // TEC reaching 256 means bus-off, but u8 saturates at 255
        // In CAN spec, TEC=255 indicates bus-off state
        self.tx_errors == 255
    }
}

/// CAN Bus Statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct CanStatistics {
    pub frames_tx: u32,
    pub frames_rx: u32,
    pub errors_tx: u32,
    pub errors_rx: u32,
    pub bus_errors: u32,
    pub arb_lost: u32,
}

/// CAN Bus Filter
#[derive(Debug, Clone, Copy)]
pub struct CanFilter {
    pub id: u32,
    pub mask: u32,
    pub extended: bool,
}

impl CanFilter {
    /// Create filter that accepts all IDs
    pub fn accept_all() -> Self {
        Self {
            id: 0,
            mask: 0,
            extended: false,
        }
    }

    /// Create filter for specific standard ID
    pub fn standard(id: u16) -> Self {
        Self {
            id: id as u32,
            mask: 0x7FF,
            extended: false,
        }
    }

    /// Create filter for specific extended ID
    pub fn extended(id: u32) -> Self {
        Self {
            id: id & 0x1FFFFFFF,
            mask: 0x1FFFFFFF,
            extended: true,
        }
    }

    /// Check if frame matches filter
    pub fn matches(&self, frame: &CanFrame) -> bool {
        let frame_id = frame.id.raw();
        let frame_ext = frame.id.is_extended();

        if self.extended != frame_ext {
            return false;
        }

        (frame_id & self.mask) == (self.id & self.mask)
    }
}

/// CAN Bus Interface
pub trait CanInterface {
    /// Initialize CAN controller
    fn initialize(&mut self, speed: CanSpeed) -> DriverResult<()>;

    /// Set operating mode
    fn set_mode(&mut self, mode: CanMode) -> DriverResult<()>;

    /// Get current mode
    fn get_mode(&self) -> CanMode;

    /// Send CAN frame
    fn send_frame(&mut self, frame: &CanFrame) -> DriverResult<()>;

    /// Receive CAN frame (non-blocking)
    fn receive_frame(&mut self) -> DriverResult<Option<CanFrame>>;

    /// Set acceptance filter
    fn set_filter(&mut self, filter: CanFilter) -> DriverResult<()>;

    /// Get error counters
    fn get_error_counters(&self) -> CanErrorCounters;

    /// Get statistics
    fn get_statistics(&self) -> CanStatistics;

    /// Clear statistics
    fn clear_statistics(&mut self);

    /// Reset controller
    fn reset(&mut self) -> DriverResult<()>;
}

/// Global CAN Bus Manager
pub struct CanManager {
    interfaces: Mutex<Vec<Box<dyn CanInterface + Send>>>,
}

impl CanManager {
    pub fn new() -> Self {
        Self {
            interfaces: Mutex::new(Vec::new()),
        }
    }

    /// Register a CAN interface
    pub fn register_interface(&self, interface: Box<dyn CanInterface + Send>) {
        self.interfaces.lock().push(interface);
    }

    /// Get number of interfaces
    pub fn interface_count(&self) -> usize {
        self.interfaces.lock().len()
    }

    /// Send frame on specific interface
    pub fn send_frame(&self, interface_id: usize, frame: &CanFrame) -> DriverResult<()> {
        let mut interfaces = self.interfaces.lock();
        if let Some(interface) = interfaces.get_mut(interface_id) {
            interface.send_frame(frame)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }

    /// Receive frame from specific interface
    pub fn receive_frame(&self, interface_id: usize) -> DriverResult<Option<CanFrame>> {
        let mut interfaces = self.interfaces.lock();
        if let Some(interface) = interfaces.get_mut(interface_id) {
            interface.receive_frame()
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
}

impl Default for CanManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global CAN manager instance
static CAN_MANAGER: spin::Once<CanManager> = spin::Once::new();

/// Initialize CAN subsystem
pub fn initialize() -> DriverResult<()> {
    CAN_MANAGER.call_once(|| CanManager::new());
    crate::info!("[CAN] CAN subsystem initialized");
    Ok(())
}

/// Get global CAN manager
pub fn get_manager() -> Option<&'static CanManager> {
    CAN_MANAGER.get()
}

/// Register a CAN interface
pub fn register_interface(interface: Box<dyn CanInterface + Send>) {
    if let Some(manager) = CAN_MANAGER.get() {
        manager.register_interface(interface);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_id() {
        let std_id = CanId::Standard(0x123);
        assert!(std_id.is_standard());
        assert!(!std_id.is_extended());
        assert_eq!(std_id.raw(), 0x123);

        let ext_id = CanId::Extended(0x12345678);
        assert!(!ext_id.is_standard());
        assert!(ext_id.is_extended());
        assert_eq!(ext_id.raw(), 0x12345678);
    }

    #[test]
    fn test_can_filter() {
        let filter = CanFilter::standard(0x123);
        let frame1 = CanFrame::new_data(CanId::Standard(0x123), alloc::vec![1, 2, 3]).unwrap();
        let frame2 = CanFrame::new_data(CanId::Standard(0x456), alloc::vec![4, 5, 6]).unwrap();

        assert!(filter.matches(&frame1));
        assert!(!filter.matches(&frame2));
    }

    #[test]
    fn test_error_state() {
        let mut errors = CanErrorCounters::default();
        assert!(errors.is_error_active());

        errors.tx_errors = 128;
        assert!(errors.is_error_passive());

        errors.tx_errors = 255;  // Bus-off state (TEC saturates at 255 for u8)
        assert!(errors.is_bus_off());
    }
}
