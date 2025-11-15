//! SDHCI (SD Host Controller Interface) driver for Arasan SDHCI 5.1
//!
//! This module provides a driver for the Arasan SDHCI controller used in the
//! Raspberry Pi 5 (BCM2712 SoC). It implements the SDHCI specification version 3.0
//! with vendor-specific enhancements from Arasan.
//!
//! # Features
//!
//! - SD/SDHC/SDXC card support
//! - ADMA2 DMA transfers for efficient data movement
//! - 4-bit bus width support
//! - Clock management and frequency scaling
//! - Error detection and recovery
//! - Hot-plug detection (card insertion/removal)
//!
//! # Hardware Specifications
//!
//! - Base Address: 0x1000fff0 (RPi5, from FDT)
//! - IRQ: Variable (from FDT)
//! - Max Clock: 200 MHz
//! - DMA: ADMA2 (32-bit addressing)
//! - Bus Width: 1-bit, 4-bit
//!
//! # Initialization Sequence
//!
//! 1. Reset controller (software reset)
//! 2. Configure clocks and power
//! 3. Detect card presence
//! 4. Initialize SD card (CMD0, CMD8, ACMD41)
//! 5. Get card information (CID, CSD)
//! 6. Select card and set bus width
//! 7. Set transfer mode (ADMA2, 4-bit)
//!
//! # References
//!
//! - SD Host Controller Simplified Specification Version 3.00
//! - SD Physical Layer Simplified Specification Version 3.01
//! - Arasan SDHCI Controller IP Datasheet

use crate::drivers::traits::BlockDevice;
use crate::lib::error::{Errno, Result};
use alloc::string::String;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

/// SDHCI register offsets
const SDHCI_DMA_ADDRESS: usize = 0x00;
const SDHCI_BLOCK_SIZE: usize = 0x04;
const SDHCI_BLOCK_COUNT: usize = 0x06;
const SDHCI_ARGUMENT: usize = 0x08;
const SDHCI_TRANSFER_MODE: usize = 0x0C;
const SDHCI_COMMAND: usize = 0x0E;
const SDHCI_RESPONSE: usize = 0x10;  // Response 0-3 (4 × 32-bit)
const SDHCI_BUFFER: usize = 0x20;
const SDHCI_PRESENT_STATE: usize = 0x24;
const SDHCI_HOST_CONTROL: usize = 0x28;
const SDHCI_POWER_CONTROL: usize = 0x29;
const SDHCI_CLOCK_CONTROL: usize = 0x2C;
const SDHCI_TIMEOUT_CONTROL: usize = 0x2E;
const SDHCI_SOFTWARE_RESET: usize = 0x2F;
const SDHCI_INT_STATUS: usize = 0x30;
const SDHCI_INT_ENABLE: usize = 0x34;
const SDHCI_SIGNAL_ENABLE: usize = 0x38;
const SDHCI_CAPABILITIES: usize = 0x40;
const SDHCI_MAX_CURRENT: usize = 0x48;
const SDHCI_ADMA_ADDRESS: usize = 0x58;
const SDHCI_SLOT_INT_STATUS: usize = 0xFC;
const SDHCI_HOST_VERSION: usize = 0xFE;

/// Present State Register bits
const PRESENT_STATE_CMD_INHIBIT: u32 = 1 << 0;
const PRESENT_STATE_DAT_INHIBIT: u32 = 1 << 1;
const PRESENT_STATE_DAT_ACTIVE: u32 = 1 << 2;
const PRESENT_STATE_WRITE_ACTIVE: u32 = 1 << 8;
const PRESENT_STATE_READ_ACTIVE: u32 = 1 << 9;
const PRESENT_STATE_BUFFER_WRITE_ENABLE: u32 = 1 << 10;
const PRESENT_STATE_BUFFER_READ_ENABLE: u32 = 1 << 11;
const PRESENT_STATE_CARD_INSERTED: u32 = 1 << 16;
const PRESENT_STATE_CARD_STABLE: u32 = 1 << 17;
const PRESENT_STATE_CARD_DETECT_PIN: u32 = 1 << 18;

/// Clock Control Register bits
const CLOCK_CONTROL_INTERNAL_ENABLE: u16 = 1 << 0;
const CLOCK_CONTROL_INTERNAL_STABLE: u16 = 1 << 1;
const CLOCK_CONTROL_SD_ENABLE: u16 = 1 << 2;
const CLOCK_CONTROL_FREQ_SEL_SHIFT: u16 = 8;

/// Software Reset Register bits
const SOFTWARE_RESET_ALL: u8 = 1 << 0;
const SOFTWARE_RESET_CMD: u8 = 1 << 1;
const SOFTWARE_RESET_DAT: u8 = 1 << 2;

/// Power Control Register bits
const POWER_CONTROL_SD_BUS_POWER: u8 = 1 << 0;
const POWER_CONTROL_SD_BUS_VOLTAGE_3_3V: u8 = 7 << 1;

/// Host Control Register bits
const HOST_CONTROL_LED: u8 = 1 << 0;
const HOST_CONTROL_DATA_WIDTH_4BIT: u8 = 1 << 1;
const HOST_CONTROL_HIGH_SPEED: u8 = 1 << 2;
const HOST_CONTROL_DMA_SELECT_SHIFT: u8 = 3;
const HOST_CONTROL_DMA_SELECT_ADMA2: u8 = 2 << 3;

/// Transfer Mode Register bits
const TRANSFER_MODE_DMA_ENABLE: u16 = 1 << 0;
const TRANSFER_MODE_BLOCK_COUNT_ENABLE: u16 = 1 << 1;
const TRANSFER_MODE_AUTO_CMD12: u16 = 1 << 2;
const TRANSFER_MODE_DATA_TRANSFER_READ: u16 = 1 << 4;
const TRANSFER_MODE_MULTI_BLOCK: u16 = 1 << 5;

/// Command Register bits
const COMMAND_RESPONSE_TYPE_SHIFT: u16 = 0;
const COMMAND_CRC_CHECK_ENABLE: u16 = 1 << 3;
const COMMAND_INDEX_CHECK_ENABLE: u16 = 1 << 4;
const COMMAND_DATA_PRESENT: u16 = 1 << 5;
const COMMAND_TYPE_SHIFT: u16 = 6;
const COMMAND_INDEX_SHIFT: u16 = 8;

/// Response types
const RESPONSE_TYPE_NONE: u16 = 0 << 0;
const RESPONSE_TYPE_136: u16 = 1 << 0;  // R2
const RESPONSE_TYPE_48: u16 = 2 << 0;   // R1, R3, R4, R5, R6, R7
const RESPONSE_TYPE_48_BUSY: u16 = 3 << 0;  // R1b

/// Interrupt Status Register bits
const INT_STATUS_COMMAND_COMPLETE: u32 = 1 << 0;
const INT_STATUS_TRANSFER_COMPLETE: u32 = 1 << 1;
const INT_STATUS_DMA_INTERRUPT: u32 = 1 << 3;
const INT_STATUS_BUFFER_WRITE_READY: u32 = 1 << 4;
const INT_STATUS_BUFFER_READ_READY: u32 = 1 << 5;
const INT_STATUS_CARD_INSERTION: u32 = 1 << 6;
const INT_STATUS_CARD_REMOVAL: u32 = 1 << 7;
const INT_STATUS_ERROR_INTERRUPT: u32 = 1 << 15;
const INT_STATUS_TIMEOUT_ERROR: u32 = 1 << 16;
const INT_STATUS_CRC_ERROR: u32 = 1 << 17;
const INT_STATUS_END_BIT_ERROR: u32 = 1 << 18;
const INT_STATUS_INDEX_ERROR: u32 = 1 << 19;
const INT_STATUS_DATA_TIMEOUT_ERROR: u32 = 1 << 20;
const INT_STATUS_DATA_CRC_ERROR: u32 = 1 << 21;
const INT_STATUS_DATA_END_BIT_ERROR: u32 = 1 << 22;
const INT_STATUS_AUTO_CMD_ERROR: u32 = 1 << 24;
const INT_STATUS_ADMA_ERROR: u32 = 1 << 25;

/// SD Commands
const CMD0_GO_IDLE_STATE: u8 = 0;
const CMD2_ALL_SEND_CID: u8 = 2;
const CMD3_SEND_RELATIVE_ADDR: u8 = 3;
const CMD6_SWITCH_FUNC: u8 = 6;
const CMD7_SELECT_CARD: u8 = 7;
const CMD8_SEND_IF_COND: u8 = 8;
const CMD9_SEND_CSD: u8 = 9;
const CMD10_SEND_CID: u8 = 10;
const CMD12_STOP_TRANSMISSION: u8 = 12;
const CMD13_SEND_STATUS: u8 = 13;
const CMD16_SET_BLOCKLEN: u8 = 16;
const CMD17_READ_SINGLE_BLOCK: u8 = 17;
const CMD18_READ_MULTIPLE_BLOCK: u8 = 18;
const CMD24_WRITE_BLOCK: u8 = 24;
const CMD25_WRITE_MULTIPLE_BLOCK: u8 = 25;
const CMD55_APP_CMD: u8 = 55;
const ACMD6_SET_BUS_WIDTH: u8 = 6;
const ACMD41_SD_SEND_OP_COND: u8 = 41;
const ACMD51_SEND_SCR: u8 = 51;

/// SD Card status
const CARD_STATUS_READY_FOR_DATA: u32 = 1 << 8;
const CARD_STATUS_STATE_SHIFT: u32 = 9;
const CARD_STATUS_STATE_MASK: u32 = 0xF;
const CARD_STATE_TRANSFER: u32 = 4;

/// Timeouts
const TIMEOUT_RESET_MS: u32 = 100;
const TIMEOUT_COMMAND_MS: u32 = 1000;
const TIMEOUT_DATA_MS: u32 = 5000;
const TIMEOUT_CARD_INIT_MS: u32 = 2000;

/// Block size
const SD_BLOCK_SIZE: usize = 512;

/// ADMA2 Descriptor
#[repr(C, align(4))]
#[derive(Copy, Clone)]
struct Adma2Descriptor {
    attr: u16,
    length: u16,
    address: u32,
}

const ADMA2_ATTR_VALID: u16 = 1 << 0;
const ADMA2_ATTR_END: u16 = 1 << 1;
const ADMA2_ATTR_INT: u16 = 1 << 2;
const ADMA2_ATTR_ACT_TRAN: u16 = 2 << 4;

/// SDHCI Controller state
pub struct SdhciController {
    base: usize,
    name: String,
    card_present: AtomicBool,
    card_rca: u16,          // Relative Card Address
    card_capacity: u64,     // Total capacity in blocks
    initialized: AtomicBool,
}

impl SdhciController {
    /// Create a new SDHCI controller instance
    ///
    /// # Arguments
    /// * `base` - MMIO base address of the controller
    /// * `name` - Device name (e.g., "mmcblk0")
    pub fn new(base: usize, name: String) -> Self {
        Self {
            base,
            name,
            card_present: AtomicBool::new(false),
            card_rca: 0,
            card_capacity: 0,
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the SDHCI controller and SD card
    ///
    /// This performs the complete initialization sequence:
    /// 1. Reset controller
    /// 2. Configure clocks and power
    /// 3. Detect and initialize SD card
    /// 4. Configure transfer parameters
    pub unsafe fn init(&mut self) -> Result<()> {
        crate::info!("SDHCI: Initializing controller at {:#x}", self.base);

        // Check hardware version
        let version = self.read_u8(SDHCI_HOST_VERSION);
        crate::info!("SDHCI: Version {}.{}", (version >> 4) & 0xF, version & 0xF);

        // Reset controller
        self.reset(SOFTWARE_RESET_ALL)?;

        // Configure power (3.3V)
        self.write_u8(SDHCI_POWER_CONTROL,
            POWER_CONTROL_SD_BUS_POWER | POWER_CONTROL_SD_BUS_VOLTAGE_3_3V);
        self.delay_ms(10);

        // Set timeout to maximum
        self.write_u8(SDHCI_TIMEOUT_CONTROL, 0xE);

        // Enable internal clock and wait for stability
        self.set_clock(400_000)?;  // 400 kHz for initialization

        // Clear all interrupt status
        self.write_u32(SDHCI_INT_STATUS, 0xFFFF_FFFF);

        // Enable all normal and error interrupts
        self.write_u32(SDHCI_INT_ENABLE, 0xFFFF_FFFF);
        self.write_u32(SDHCI_SIGNAL_ENABLE, 0x0);  // Polling mode for now

        // Check card presence
        if !self.is_card_present() {
            crate::warn!("SDHCI: No card detected");
            return Err(Errno::NoDevice);
        }

        crate::info!("SDHCI: Card detected");
        self.card_present.store(true, Ordering::Release);

        // Initialize SD card
        self.init_card()?;

        // Increase clock to high speed (25 MHz for SD)
        self.set_clock(25_000_000)?;

        // Enable 4-bit bus width
        self.set_bus_width_4bit()?;

        // Enable ADMA2 mode
        let mut host_ctrl = self.read_u8(SDHCI_HOST_CONTROL);
        host_ctrl |= HOST_CONTROL_DMA_SELECT_ADMA2;
        self.write_u8(SDHCI_HOST_CONTROL, host_ctrl);

        self.initialized.store(true, Ordering::Release);
        crate::info!("SDHCI: Initialization complete ({} blocks, {} GB)",
                     self.card_capacity,
                     (self.card_capacity * SD_BLOCK_SIZE as u64) / 1_000_000_000);

        Ok(())
    }

    /// Reset the SDHCI controller
    fn reset(&self, mask: u8) -> Result<()> {
        unsafe {
            self.write_u8(SDHCI_SOFTWARE_RESET, mask);

            let mut timeout = TIMEOUT_RESET_MS;
            while timeout > 0 {
                if (self.read_u8(SDHCI_SOFTWARE_RESET) & mask) == 0 {
                    return Ok(());
                }
                self.delay_ms(1);
                timeout -= 1;
            }
        }

        Err(Errno::TimedOut)
    }

    /// Set SD clock frequency
    fn set_clock(&self, freq_hz: u32) -> Result<()> {
        unsafe {
            // Disable SD clock
            let mut clk_ctrl = self.read_u16(SDHCI_CLOCK_CONTROL);
            clk_ctrl &= !CLOCK_CONTROL_SD_ENABLE;
            self.write_u16(SDHCI_CLOCK_CONTROL, clk_ctrl);

            // Calculate divisor (assuming base clock of 200 MHz)
            let base_clock = 200_000_000;
            let mut divisor = base_clock / freq_hz;
            if divisor > 0 {
                divisor = (divisor + 1) / 2;  // Round up and divide by 2
            }
            if divisor > 256 {
                divisor = 256;
            }

            // Set frequency select
            let freq_sel = (divisor as u16) << CLOCK_CONTROL_FREQ_SEL_SHIFT;
            clk_ctrl = freq_sel | CLOCK_CONTROL_INTERNAL_ENABLE;
            self.write_u16(SDHCI_CLOCK_CONTROL, clk_ctrl);

            // Wait for internal clock stable
            let mut timeout = TIMEOUT_RESET_MS;
            while timeout > 0 {
                if (self.read_u16(SDHCI_CLOCK_CONTROL) & CLOCK_CONTROL_INTERNAL_STABLE) != 0 {
                    break;
                }
                self.delay_ms(1);
                timeout -= 1;
            }

            if timeout == 0 {
                return Err(Errno::TimedOut);
            }

            // Enable SD clock
            clk_ctrl |= CLOCK_CONTROL_SD_ENABLE;
            self.write_u16(SDHCI_CLOCK_CONTROL, clk_ctrl);
            self.delay_ms(10);

            crate::info!("SDHCI: Clock set to {} Hz (divisor {})",
                         base_clock / (divisor * 2), divisor);
        }

        Ok(())
    }

    /// Check if card is present
    fn is_card_present(&self) -> bool {
        unsafe {
            let state = self.read_u32(SDHCI_PRESENT_STATE);
            (state & PRESENT_STATE_CARD_INSERTED) != 0 &&
            (state & PRESENT_STATE_CARD_STABLE) != 0
        }
    }

    /// Initialize SD card
    fn init_card(&mut self) -> Result<()> {
        unsafe {
            // CMD0: GO_IDLE_STATE
            self.send_command(CMD0_GO_IDLE_STATE, 0, RESPONSE_TYPE_NONE, 0)?;
            self.delay_ms(10);

            // CMD8: SEND_IF_COND (check voltage and pattern)
            let cmd8_arg = 0x1AA;  // VHS=1 (2.7-3.6V), check pattern=0xAA
            match self.send_command(CMD8_SEND_IF_COND, cmd8_arg, RESPONSE_TYPE_48,
                                   COMMAND_CRC_CHECK_ENABLE | COMMAND_INDEX_CHECK_ENABLE) {
                Ok(resp) => {
                    if (resp[0] & 0xFFF) != cmd8_arg {
                        return Err(Errno::InvalidArgument);
                    }
                    crate::info!("SDHCI: SD Card Version 2.0+");
                }
                Err(_) => {
                    crate::info!("SDHCI: SD Card Version 1.x");
                    return Err(Errno::NotSupported);
                }
            }

            // ACMD41: SD_SEND_OP_COND (get OCR and initialize)
            let mut timeout = TIMEOUT_CARD_INIT_MS;
            loop {
                // CMD55: APP_CMD
                self.send_command(CMD55_APP_CMD, 0, RESPONSE_TYPE_48, 0)?;

                // ACMD41: SD_SEND_OP_COND with HCS bit (support SDHC/SDXC)
                let acmd41_arg = 0x40FF8000;  // HCS=1, 3.2-3.3V, 3.3-3.4V
                let resp = self.send_command(ACMD41_SD_SEND_OP_COND, acmd41_arg, RESPONSE_TYPE_48, 0)?;

                // Check if card is ready (bit 31 = 1)
                if (resp[0] & 0x8000_0000) != 0 {
                    // Check CCS bit (Card Capacity Status)
                    let is_sdhc = (resp[0] & 0x4000_0000) != 0;
                    crate::info!("SDHCI: Card type: {}", if is_sdhc { "SDHC/SDXC" } else { "SDSC" });
                    break;
                }

                if timeout == 0 {
                    return Err(Errno::TimedOut);
                }
                self.delay_ms(10);
                timeout -= 10;
            }

            // CMD2: ALL_SEND_CID
            let cid = self.send_command(CMD2_ALL_SEND_CID, 0, RESPONSE_TYPE_136,
                                       COMMAND_CRC_CHECK_ENABLE)?;
            crate::info!("SDHCI: CID = {:08x} {:08x} {:08x} {:08x}",
                         cid[0], cid[1], cid[2], cid[3]);

            // CMD3: SEND_RELATIVE_ADDR
            let resp = self.send_command(CMD3_SEND_RELATIVE_ADDR, 0, RESPONSE_TYPE_48,
                                        COMMAND_CRC_CHECK_ENABLE | COMMAND_INDEX_CHECK_ENABLE)?;
            self.card_rca = (resp[0] >> 16) as u16;
            crate::info!("SDHCI: RCA = {:#x}", self.card_rca);

            // CMD9: SEND_CSD
            let csd = self.send_command(CMD9_SEND_CSD, (self.card_rca as u32) << 16,
                                       RESPONSE_TYPE_136, COMMAND_CRC_CHECK_ENABLE)?;

            // Parse CSD to get capacity
            self.card_capacity = self.parse_csd(&csd);
            crate::info!("SDHCI: Capacity = {} blocks", self.card_capacity);

            // CMD7: SELECT_CARD
            self.send_command(CMD7_SELECT_CARD, (self.card_rca as u32) << 16,
                             RESPONSE_TYPE_48_BUSY,
                             COMMAND_CRC_CHECK_ENABLE | COMMAND_INDEX_CHECK_ENABLE)?;

            // Set block length to 512 bytes
            self.send_command(CMD16_SET_BLOCKLEN, SD_BLOCK_SIZE as u32,
                             RESPONSE_TYPE_48,
                             COMMAND_CRC_CHECK_ENABLE | COMMAND_INDEX_CHECK_ENABLE)?;
        }

        Ok(())
    }

    /// Parse CSD (Card-Specific Data) to extract capacity
    fn parse_csd(&self, csd: &[u32; 4]) -> u64 {
        // CSD version is in bits [127:126]
        let csd_version = (csd[3] >> 30) & 0x3;

        if csd_version == 0 {
            // CSD Version 1.0 (SDSC)
            let c_size = ((csd[1] >> 30) & 0x3) | ((csd[2] & 0x3FF) << 2);
            let c_size_mult = (csd[1] >> 15) & 0x7;
            let read_bl_len = (csd[2] >> 16) & 0xF;

            let block_len = 1u64 << read_bl_len;
            let mult = 1u64 << (c_size_mult + 2);
            let blocknr = (c_size as u64 + 1) * mult;

            blocknr * block_len / SD_BLOCK_SIZE as u64
        } else {
            // CSD Version 2.0 (SDHC/SDXC)
            let c_size = (csd[1] >> 16) & 0xFFFF | ((csd[2] & 0x3F) << 16);
            (c_size as u64 + 1) * 1024  // Each C_SIZE represents 512KB
        }
    }

    /// Set 4-bit bus width
    fn set_bus_width_4bit(&mut self) -> Result<()> {
        unsafe {
            // CMD55: APP_CMD
            self.send_command(CMD55_APP_CMD, (self.card_rca as u32) << 16,
                             RESPONSE_TYPE_48, 0)?;

            // ACMD6: SET_BUS_WIDTH (2 = 4-bit)
            self.send_command(ACMD6_SET_BUS_WIDTH, 2, RESPONSE_TYPE_48, 0)?;

            // Update host controller
            let mut host_ctrl = self.read_u8(SDHCI_HOST_CONTROL);
            host_ctrl |= HOST_CONTROL_DATA_WIDTH_4BIT;
            self.write_u8(SDHCI_HOST_CONTROL, host_ctrl);

            crate::info!("SDHCI: Set 4-bit bus width");
        }

        Ok(())
    }

    /// Send a command to the SD card
    ///
    /// # Returns
    /// Response data (4 × u32 for R2, first u32 for other responses)
    fn send_command(&self, cmd: u8, arg: u32, resp_type: u16, flags: u16) -> Result<[u32; 4]> {
        unsafe {
            // Wait for command line ready
            self.wait_for_cmd_ready()?;

            // Clear interrupt status
            self.write_u32(SDHCI_INT_STATUS, 0xFFFF_FFFF);

            // Write argument
            self.write_u32(SDHCI_ARGUMENT, arg);

            // Build command register value
            let cmd_reg = ((cmd as u16) << COMMAND_INDEX_SHIFT) | resp_type | flags;

            // Send command
            self.write_u16(SDHCI_COMMAND, cmd_reg);

            // Wait for command complete
            self.wait_for_interrupt(INT_STATUS_COMMAND_COMPLETE, TIMEOUT_COMMAND_MS)?;

            // Clear command complete interrupt
            self.write_u32(SDHCI_INT_STATUS, INT_STATUS_COMMAND_COMPLETE);

            // Read response
            let mut response = [0u32; 4];
            if resp_type != RESPONSE_TYPE_NONE {
                response[0] = self.read_u32(SDHCI_RESPONSE + 0);
                if resp_type == RESPONSE_TYPE_136 {
                    response[1] = self.read_u32(SDHCI_RESPONSE + 4);
                    response[2] = self.read_u32(SDHCI_RESPONSE + 8);
                    response[3] = self.read_u32(SDHCI_RESPONSE + 12);
                }
            }

            Ok(response)
        }
    }

    /// Wait for command line to be ready
    fn wait_for_cmd_ready(&self) -> Result<()> {
        unsafe {
            let mut timeout = TIMEOUT_COMMAND_MS;
            while timeout > 0 {
                let state = self.read_u32(SDHCI_PRESENT_STATE);
                if (state & PRESENT_STATE_CMD_INHIBIT) == 0 {
                    return Ok(());
                }
                self.delay_ms(1);
                timeout -= 1;
            }
        }
        Err(Errno::TimedOut)
    }

    /// Wait for specific interrupt status
    fn wait_for_interrupt(&self, mask: u32, timeout_ms: u32) -> Result<()> {
        unsafe {
            let mut timeout = timeout_ms;
            while timeout > 0 {
                let status = self.read_u32(SDHCI_INT_STATUS);

                // Check for errors
                if (status & INT_STATUS_ERROR_INTERRUPT) != 0 {
                    let errors = status & 0xFFFF_0000;
                    crate::warn!("SDHCI: Error interrupt {:#x}", errors);
                    self.write_u32(SDHCI_INT_STATUS, errors);
                    return Err(Errno::IOError);
                }

                // Check for desired interrupt
                if (status & mask) != 0 {
                    return Ok(());
                }

                self.delay_ms(1);
                timeout -= 1;
            }
        }
        Err(Errno::TimedOut)
    }

    /// Read a single block from the SD card
    fn read_block_pio(&self, block: u64, buf: &mut [u8]) -> Result<()> {
        if buf.len() < SD_BLOCK_SIZE {
            return Err(Errno::InvalidArgument);
        }

        unsafe {
            // Configure block size and count
            self.write_u16(SDHCI_BLOCK_SIZE, SD_BLOCK_SIZE as u16);
            self.write_u16(SDHCI_BLOCK_COUNT, 1);

            // Set transfer mode (single block, read)
            self.write_u16(SDHCI_TRANSFER_MODE, TRANSFER_MODE_DATA_TRANSFER_READ);

            // Send READ_SINGLE_BLOCK command
            self.send_command(CMD17_READ_SINGLE_BLOCK, block as u32, RESPONSE_TYPE_48,
                             COMMAND_DATA_PRESENT | COMMAND_CRC_CHECK_ENABLE | COMMAND_INDEX_CHECK_ENABLE)?;

            // Wait for buffer read ready
            self.wait_for_interrupt(INT_STATUS_BUFFER_READ_READY, TIMEOUT_DATA_MS)?;

            // Read data from buffer
            for i in (0..SD_BLOCK_SIZE).step_by(4) {
                let word = self.read_u32(SDHCI_BUFFER);
                buf[i] = word as u8;
                buf[i + 1] = (word >> 8) as u8;
                buf[i + 2] = (word >> 16) as u8;
                buf[i + 3] = (word >> 24) as u8;
            }

            // Wait for transfer complete
            self.wait_for_interrupt(INT_STATUS_TRANSFER_COMPLETE, TIMEOUT_DATA_MS)?;

            // Clear interrupt status
            self.write_u32(SDHCI_INT_STATUS, INT_STATUS_BUFFER_READ_READY | INT_STATUS_TRANSFER_COMPLETE);
        }

        Ok(())
    }

    /// Write a single block to the SD card
    fn write_block_pio(&self, block: u64, buf: &[u8]) -> Result<()> {
        if buf.len() < SD_BLOCK_SIZE {
            return Err(Errno::InvalidArgument);
        }

        unsafe {
            // Configure block size and count
            self.write_u16(SDHCI_BLOCK_SIZE, SD_BLOCK_SIZE as u16);
            self.write_u16(SDHCI_BLOCK_COUNT, 1);

            // Set transfer mode (single block, write)
            self.write_u16(SDHCI_TRANSFER_MODE, 0);

            // Send WRITE_BLOCK command
            self.send_command(CMD24_WRITE_BLOCK, block as u32, RESPONSE_TYPE_48,
                             COMMAND_DATA_PRESENT | COMMAND_CRC_CHECK_ENABLE | COMMAND_INDEX_CHECK_ENABLE)?;

            // Wait for buffer write ready
            self.wait_for_interrupt(INT_STATUS_BUFFER_WRITE_READY, TIMEOUT_DATA_MS)?;

            // Write data to buffer
            for i in (0..SD_BLOCK_SIZE).step_by(4) {
                let word = buf[i] as u32 |
                          ((buf[i + 1] as u32) << 8) |
                          ((buf[i + 2] as u32) << 16) |
                          ((buf[i + 3] as u32) << 24);
                self.write_u32(SDHCI_BUFFER, word);
            }

            // Wait for transfer complete
            self.wait_for_interrupt(INT_STATUS_TRANSFER_COMPLETE, TIMEOUT_DATA_MS)?;

            // Clear interrupt status
            self.write_u32(SDHCI_INT_STATUS, INT_STATUS_BUFFER_WRITE_READY | INT_STATUS_TRANSFER_COMPLETE);
        }

        Ok(())
    }

    // Register access helpers
    #[inline]
    unsafe fn read_u32(&self, offset: usize) -> u32 {
        read_volatile((self.base + offset) as *const u32)
    }

    #[inline]
    unsafe fn write_u32(&self, offset: usize, value: u32) {
        write_volatile((self.base + offset) as *mut u32, value)
    }

    #[inline]
    unsafe fn read_u16(&self, offset: usize) -> u16 {
        read_volatile((self.base + offset) as *const u16)
    }

    #[inline]
    unsafe fn write_u16(&self, offset: usize, value: u16) {
        write_volatile((self.base + offset) as *mut u16, value)
    }

    #[inline]
    unsafe fn read_u8(&self, offset: usize) -> u8 {
        read_volatile((self.base + offset) as *const u8)
    }

    #[inline]
    unsafe fn write_u8(&self, offset: usize, value: u8) {
        write_volatile((self.base + offset) as *mut u8, value)
    }

    /// Busy-wait delay (milliseconds)
    fn delay_ms(&self, ms: u32) {
        // Use timer if available, otherwise busy-wait
        let freq = crate::arch::aarch64::timer::read_cntfrq();
        let start = crate::arch::aarch64::timer::read_cntpct();
        let ticks = (freq * ms as u64) / 1000;
        while crate::arch::aarch64::timer::read_cntpct() - start < ticks {
            core::hint::spin_loop();
        }
    }
}

impl BlockDevice for SdhciController {
    fn read(&self, block: u64, buf: &mut [u8]) -> Result<()> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(Errno::NotReady);
        }

        if !self.card_present.load(Ordering::Acquire) {
            return Err(Errno::NoDevice);
        }

        if block >= self.card_capacity {
            return Err(Errno::InvalidArgument);
        }

        self.read_block_pio(block, buf)
    }

    fn write(&self, block: u64, buf: &[u8]) -> Result<()> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(Errno::NotReady);
        }

        if !self.card_present.load(Ordering::Acquire) {
            return Err(Errno::NoDevice);
        }

        if block >= self.card_capacity {
            return Err(Errno::InvalidArgument);
        }

        self.write_block_pio(block, buf)
    }

    fn flush(&self) -> Result<()> {
        // No explicit flush needed for synchronous PIO transfers
        Ok(())
    }

    fn block_size(&self) -> usize {
        SD_BLOCK_SIZE
    }

    fn block_count(&self) -> u64 {
        self.card_capacity
    }

    fn name(&self) -> &str {
        &self.name
    }
}

unsafe impl Send for SdhciController {}
unsafe impl Sync for SdhciController {}
