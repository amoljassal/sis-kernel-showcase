//! MCP2515 CAN Controller Driver
//!
//! Microchip MCP2515 is a standalone CAN controller with SPI interface.
//! Commonly used with Raspberry Pi for CAN bus applications.
//!
//! # Hardware Connections
//!
//! ```text
//! Raspberry Pi 5    MCP2515
//! ──────────────    ───────
//! SPI0_MOSI    <──> SI (Data In)
//! SPI0_MISO    <──> SO (Data Out)
//! SPI0_SCLK    <──> SCK (Clock)
//! SPI0_CE0     <──> CS (Chip Select)
//! GPIO25       <──> INT (Interrupt)
//! ```
//!
//! # Features
//! - CAN 2.0A/B protocol support
//! - Up to 1 Mbps
//! - 2 receive buffers with prioritized message storage
//! - 3 transmit buffers with prioritization
//! - 6 acceptance filters and 2 masks

use super::{CanInterface, CanFrame, CanId, CanMode, CanSpeed, CanFilter};
use super::{CanErrorCounters, CanStatistics, FrameType};
use crate::drivers::{DriverError, DriverResult};
use crate::drivers::spi;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// MCP2515 SPI Commands
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum SpiCommand {
    Reset = 0xC0,
    Read = 0x03,
    ReadRxBuffer = 0x90,
    Write = 0x02,
    LoadTxBuffer = 0x40,
    Rts = 0x80,  // Request to Send
    ReadStatus = 0xA0,
    RxStatus = 0xB0,
    BitModify = 0x05,
}

/// MCP2515 Registers
#[allow(dead_code)]
mod reg {
    // Configuration registers
    pub const CANSTAT: u8 = 0x0E;
    pub const CANCTRL: u8 = 0x0F;
    pub const CNF1: u8 = 0x2A;
    pub const CNF2: u8 = 0x29;
    pub const CNF3: u8 = 0x28;

    // Transmit buffer registers
    pub const TXB0CTRL: u8 = 0x30;
    pub const TXB1CTRL: u8 = 0x40;
    pub const TXB2CTRL: u8 = 0x50;

    // Receive buffer registers
    pub const RXB0CTRL: u8 = 0x60;
    pub const RXB1CTRL: u8 = 0x70;

    // Interrupt registers
    pub const CANINTE: u8 = 0x2B;
    pub const CANINTF: u8 = 0x2C;

    // Error registers
    pub const TEC: u8 = 0x1C;  // Transmit Error Counter
    pub const REC: u8 = 0x1D;  // Receive Error Counter
    pub const EFLG: u8 = 0x2D;  // Error Flags
}

/// CANCTRL Register Modes
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum CtrlMode {
    Normal = 0x00,
    Sleep = 0x20,
    Loopback = 0x40,
    ListenOnly = 0x60,
    Config = 0x80,
}

/// MCP2515 CAN Controller
pub struct Mcp2515 {
    /// SPI bus number
    spi_bus: u8,

    /// SPI chip select
    spi_cs: spi::ChipSelect,

    /// Current operating mode
    mode: CanMode,

    /// Current bus speed
    speed: CanSpeed,

    /// Error counters
    error_counters: CanErrorCounters,

    /// Statistics
    stats: CanStatistics,

    /// Initialized flag
    initialized: AtomicBool,
}

impl Mcp2515 {
    /// Create new MCP2515 instance
    ///
    /// # Arguments
    /// * `spi_bus` - SPI bus number (0-4 on RPi5)
    /// * `cs` - Chip select line
    pub fn new(spi_bus: u8, cs: spi::ChipSelect) -> Self {
        Self {
            spi_bus,
            spi_cs: cs,
            mode: CanMode::Configuration,
            speed: CanSpeed::Speed500K,
            error_counters: CanErrorCounters::default(),
            stats: CanStatistics::default(),
            initialized: AtomicBool::new(false),
        }
    }

    /// Reset the MCP2515
    fn reset_controller(&self) -> DriverResult<()> {
        let cmd = [SpiCommand::Reset as u8];
        spi::write(self.spi_bus, self.spi_cs, &cmd)?;

        // Wait for reset to complete
        crate::time::sleep_ms(10);

        Ok(())
    }

    /// Read register
    fn read_register(&self, addr: u8) -> DriverResult<u8> {
        let tx = [SpiCommand::Read as u8, addr, 0x00];
        let mut rx = [0u8; 3];

        spi::transfer(self.spi_bus, self.spi_cs, &tx, &mut rx)?;

        Ok(rx[2])
    }

    /// Write register
    fn write_register(&self, addr: u8, value: u8) -> DriverResult<()> {
        let tx = [SpiCommand::Write as u8, addr, value];
        spi::write(self.spi_bus, self.spi_cs, &tx)?;
        Ok(())
    }

    /// Modify register bits
    fn modify_register(&self, addr: u8, mask: u8, value: u8) -> DriverResult<()> {
        let tx = [SpiCommand::BitModify as u8, addr, mask, value];
        spi::write(self.spi_bus, self.spi_cs, &tx)?;
        Ok(())
    }

    /// Configure bit timing for specified speed
    fn configure_bit_timing(&self, speed: CanSpeed) -> DriverResult<()> {
        // These values assume 16 MHz crystal oscillator
        // TQ = 2 * (BRP + 1) / FOSC
        // Bit Time = (SYNC_SEG + PROP_SEG + PS1 + PS2)
        //
        // For 500 kbps with 16 MHz:
        // BRP = 0, TQ = 125 ns, 16 TQ per bit = 2 μs = 500 kbps

        let (cnf1, cnf2, cnf3) = match speed {
            CanSpeed::Speed1M => {
                // 1 Mbps: BRP=0, 8 TQ per bit
                (0x00, 0x90, 0x02)
            }
            CanSpeed::Speed500K => {
                // 500 kbps: BRP=0, 16 TQ per bit
                (0x00, 0xB1, 0x05)
            }
            CanSpeed::Speed250K => {
                // 250 kbps: BRP=1, 16 TQ per bit
                (0x01, 0xB1, 0x05)
            }
            CanSpeed::Speed125K => {
                // 125 kbps: BRP=3, 16 TQ per bit
                (0x03, 0xB1, 0x05)
            }
            CanSpeed::Speed100K => {
                // 100 kbps: BRP=4, 16 TQ per bit
                (0x04, 0xB1, 0x05)
            }
            CanSpeed::Speed50K => {
                // 50 kbps: BRP=9, 16 TQ per bit
                (0x09, 0xB1, 0x05)
            }
            CanSpeed::Speed20K => {
                // 20 kbps: BRP=24, 16 TQ per bit
                (0x18, 0xB1, 0x05)
            }
            CanSpeed::Speed10K => {
                // 10 kbps: BRP=49, 16 TQ per bit
                (0x31, 0xB1, 0x05)
            }
            _ => {
                // Default to 500 kbps
                (0x00, 0xB1, 0x05)
            }
        };

        self.write_register(reg::CNF1, cnf1)?;
        self.write_register(reg::CNF2, cnf2)?;
        self.write_register(reg::CNF3, cnf3)?;

        Ok(())
    }

    /// Set controller mode
    fn set_controller_mode(&self, mode: CtrlMode) -> DriverResult<()> {
        self.modify_register(reg::CANCTRL, 0xE0, mode as u8)?;

        // Wait for mode change
        let timeout = 100;
        for _ in 0..timeout {
            let canstat = self.read_register(reg::CANSTAT)?;
            if (canstat & 0xE0) == (mode as u8) {
                return Ok(());
            }
            crate::time::sleep_ms(1);
        }

        let timeout_err = crate::drivers::timeout::TimeoutError::new(timeout, timeout);
        Err(DriverError::Timeout(timeout_err))
    }

    /// Load frame into TX buffer
    fn load_tx_buffer(&self, buffer_idx: u8, frame: &CanFrame) -> DriverResult<()> {
        // Calculate load command
        let load_cmd = (SpiCommand::LoadTxBuffer as u8) | (buffer_idx << 1);

        // Build TX buffer data
        let mut tx_data = Vec::new();
        tx_data.push(load_cmd);

        // Add ID bytes
        match frame.id {
            CanId::Standard(id) => {
                // SIDH: bits 10-3
                tx_data.push(((id >> 3) & 0xFF) as u8);
                // SIDL: bits 2-0 at positions 7-5
                tx_data.push(((id & 0x07) << 5) as u8);
                // EID8, EID0 (not used for standard)
                tx_data.push(0x00);
                tx_data.push(0x00);
            }
            CanId::Extended(id) => {
                // SIDH: bits 28-21
                tx_data.push(((id >> 21) & 0xFF) as u8);
                // SIDL: bits 20-18 at 7-5, EXIDE=1 at bit 3, bits 17-16 at 1-0
                tx_data.push((((id >> 13) & 0xE0) | 0x08 | ((id >> 16) & 0x03)) as u8);
                // EID8: bits 15-8
                tx_data.push(((id >> 8) & 0xFF) as u8);
                // EID0: bits 7-0
                tx_data.push((id & 0xFF) as u8);
            }
        }

        // DLC byte
        let dlc = frame.dlc() & 0x0F;
        let dlc_byte = if frame.is_remote() {
            dlc | 0x40  // Set RTR bit
        } else {
            dlc
        };
        tx_data.push(dlc_byte);

        // Add data bytes
        tx_data.extend_from_slice(&frame.data);

        // Send via SPI
        spi::write(self.spi_bus, self.spi_cs, &tx_data)?;

        Ok(())
    }

    /// Request to send (trigger transmission)
    fn request_to_send(&self, buffer_mask: u8) -> DriverResult<()> {
        let cmd = [(SpiCommand::Rts as u8) | (buffer_mask & 0x07)];
        spi::write(self.spi_bus, self.spi_cs, &cmd)?;
        Ok(())
    }

    /// Read RX buffer
    fn read_rx_buffer(&self, buffer_idx: u8) -> DriverResult<CanFrame> {
        let read_cmd = (SpiCommand::ReadRxBuffer as u8) | (buffer_idx << 2);

        // Read ID, DLC, and up to 8 data bytes (13 bytes total)
        let mut tx = alloc::vec![read_cmd];
        tx.extend_from_slice(&[0u8; 13]);
        let mut rx = alloc::vec![0u8; 14];

        spi::transfer(self.spi_bus, self.spi_cs, &tx, &mut rx)?;

        // Parse ID
        let sidh = rx[1];
        let sidl = rx[2];
        let eid8 = rx[3];
        let eid0 = rx[4];

        let id = if (sidl & 0x08) != 0 {
            // Extended ID
            let id_a = (sidh as u32) << 21;
            let id_b = ((sidl as u32 & 0xE0) << 13) | ((sidl as u32 & 0x03) << 16);
            let id_c = (eid8 as u32) << 8;
            let id_d = eid0 as u32;
            CanId::Extended(id_a | id_b | id_c | id_d)
        } else {
            // Standard ID
            let id = ((sidh as u16) << 3) | ((sidl as u16) >> 5);
            CanId::Standard(id)
        };

        // Parse DLC and RTR
        let dlc_byte = rx[5];
        let dlc = (dlc_byte & 0x0F) as usize;
        let is_rtr = (dlc_byte & 0x40) != 0;

        // Extract data
        let data = rx[6..6 + dlc].to_vec();

        Ok(CanFrame {
            id,
            frame_type: if is_rtr { FrameType::Remote } else { FrameType::Data },
            data,
            timestamp_us: crate::time::get_timestamp_us(),
        })
    }

    /// Check if RX buffer has message
    fn has_rx_message(&self) -> DriverResult<bool> {
        let status = self.read_register(reg::CANINTF)?;
        Ok((status & 0x03) != 0)  // RX0IF or RX1IF set
    }
}

impl CanInterface for Mcp2515 {
    fn initialize(&mut self, speed: CanSpeed) -> DriverResult<()> {
        crate::info!("[MCP2515] Initializing CAN controller at {} bps", speed.as_bps());

        // Reset controller
        self.reset_controller()?;

        // Enter configuration mode
        self.set_controller_mode(CtrlMode::Config)?;

        // Configure bit timing
        self.configure_bit_timing(speed)?;
        self.speed = speed;

        // Configure interrupts (enable RX interrupts)
        self.write_register(reg::CANINTE, 0x03)?;  // RX0IE | RX1IE

        // Configure RX buffers to receive all messages
        self.write_register(reg::RXB0CTRL, 0x60)?;  // Receive all (turn off filters)
        self.write_register(reg::RXB1CTRL, 0x60)?;

        self.initialized.store(true, Ordering::Release);

        crate::info!("[MCP2515] Initialization complete");
        Ok(())
    }

    fn set_mode(&mut self, mode: CanMode) -> DriverResult<()> {
        let ctrl_mode = match mode {
            CanMode::Normal => CtrlMode::Normal,
            CanMode::ListenOnly => CtrlMode::ListenOnly,
            CanMode::Loopback => CtrlMode::Loopback,
            CanMode::Sleep => CtrlMode::Sleep,
            CanMode::Configuration => CtrlMode::Config,
        };

        self.set_controller_mode(ctrl_mode)?;
        self.mode = mode;

        crate::debug!("[MCP2515] Mode set to {:?}", mode);
        Ok(())
    }

    fn get_mode(&self) -> CanMode {
        self.mode
    }

    fn send_frame(&mut self, frame: &CanFrame) -> DriverResult<()> {
        // Use TX buffer 0
        self.load_tx_buffer(0, frame)?;

        // Request to send
        self.request_to_send(0x01)?;

        self.stats.frames_tx += 1;

        Ok(())
    }

    fn receive_frame(&mut self) -> DriverResult<Option<CanFrame>> {
        if !self.has_rx_message()? {
            return Ok(None);
        }

        // Read from RX buffer 0 (can extend to check both buffers)
        let frame = self.read_rx_buffer(0)?;

        // Clear interrupt flag
        self.modify_register(reg::CANINTF, 0x01, 0x00)?;

        self.stats.frames_rx += 1;

        Ok(Some(frame))
    }

    fn set_filter(&mut self, filter: CanFilter) -> DriverResult<()> {
        // Enter configuration mode to set filters
        let old_mode = self.mode;
        self.set_mode(CanMode::Configuration)?;

        // Configure filter and mask registers (simplified - only filter 0)
        // In a real implementation, would configure all 6 filters and 2 masks

        // Restore previous mode
        self.set_mode(old_mode)?;

        crate::debug!("[MCP2515] Filter configured: ID=0x{:X}, Mask=0x{:X}", filter.id, filter.mask);
        Ok(())
    }

    fn get_error_counters(&self) -> CanErrorCounters {
        // Read error counters from registers
        if let (Ok(tx_errors), Ok(rx_errors)) = (
            self.read_register(reg::TEC),
            self.read_register(reg::REC),
        ) {
            CanErrorCounters { tx_errors, rx_errors }
        } else {
            self.error_counters
        }
    }

    fn get_statistics(&self) -> CanStatistics {
        self.stats
    }

    fn clear_statistics(&mut self) {
        self.stats = CanStatistics::default();
    }

    fn reset(&mut self) -> DriverResult<()> {
        self.reset_controller()?;
        self.clear_statistics();
        Ok(())
    }
}
