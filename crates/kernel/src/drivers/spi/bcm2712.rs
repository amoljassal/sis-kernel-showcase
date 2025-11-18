//! BCM2712 SPI Controller Driver
//!
//! Hardware driver for the BCM2712 SPI controllers on Raspberry Pi 5.
//! Provides SPI master mode with configurable speeds and modes.

use crate::drivers::{DriverError, DriverResult};
use core::ptr::{read_volatile, write_volatile};

/// SPI controller registers (BCM2712 SPI)
#[repr(C)]
struct SpiRegs {
    cs: u32,          // 0x00: Control and Status
    fifo: u32,        // 0x04: TX and RX FIFOs
    clk: u32,         // 0x08: Clock Divider
    dlen: u32,        // 0x0C: Data Length
    ltoh: u32,        // 0x10: LOSSI mode TOH
    dc: u32,          // 0x14: DMA DREQ Controls
}

/// Control and Status register bits
const CS_LEN_LONG: u32 = 1 << 25;  // Enable Long data word
const CS_DMA_LEN: u32 = 1 << 24;   // Enable DMA mode in Lossi mode
const CS_CSPOL2: u32 = 1 << 23;    // Chip Select 2 Polarity
const CS_CSPOL1: u32 = 1 << 22;    // Chip Select 1 Polarity
const CS_CSPOL0: u32 = 1 << 21;    // Chip Select 0 Polarity
const CS_RXF: u32 = 1 << 20;       // RXF - Receive FIFO Full
const CS_RXR: u32 = 1 << 19;       // RXR - Receive FIFO needs Reading
const CS_TXD: u32 = 1 << 18;       // TXD - TX FIFO can accept Data
const CS_RXD: u32 = 1 << 17;       // RXD - RX FIFO contains Data
const CS_DONE: u32 = 1 << 16;      // Done transfer Done
const CS_LEN: u32 = 1 << 13;       // LEN LoSSI enable
const CS_REN: u32 = 1 << 12;       // REN Read Enable
const CS_ADCS: u32 = 1 << 11;      // ADCS Automatically Deassert Chip Select
const CS_INTR: u32 = 1 << 10;      // INTR Interrupt on RXR
const CS_INTD: u32 = 1 << 9;       // INTD Interrupt on Done
const CS_DMAEN: u32 = 1 << 8;      // DMAEN DMA Enable
const CS_TA: u32 = 1 << 7;         // Transfer Active
const CS_CSPOL: u32 = 1 << 6;      // Chip Select Polarity
const CS_CLEAR_RX: u32 = 1 << 5;   // Clear RX FIFO
const CS_CLEAR_TX: u32 = 1 << 4;   // Clear TX FIFO
const CS_CPOL: u32 = 1 << 3;       // Clock Polarity
const CS_CPHA: u32 = 1 << 2;       // Clock Phase
const CS_CS_MASK: u32 = 0x3;       // Chip Select bits

/// SPI Mode (CPOL/CPHA configuration)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    /// Mode 0: CPOL=0, CPHA=0
    Mode0,
    /// Mode 1: CPOL=0, CPHA=1
    Mode1,
    /// Mode 2: CPOL=1, CPHA=0
    Mode2,
    /// Mode 3: CPOL=1, CPHA=1
    Mode3,
}

impl SpiMode {
    /// Convert to CS register bits
    fn to_cs_bits(self) -> u32 {
        match self {
            SpiMode::Mode0 => 0,
            SpiMode::Mode1 => CS_CPHA,
            SpiMode::Mode2 => CS_CPOL,
            SpiMode::Mode3 => CS_CPOL | CS_CPHA,
        }
    }
}

/// Chip select line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipSelect {
    /// Chip select 0
    Cs0 = 0,
    /// Chip select 1
    Cs1 = 1,
    /// Chip select 2
    Cs2 = 2,
}

/// BCM2712 SPI controller
pub struct Bcm2712Spi {
    base: usize,
    controller: u8,
    clock_hz: u32,  // Input clock frequency
}

impl Bcm2712Spi {
    /// Create a new SPI controller instance
    pub const fn new(base: usize, controller: u8) -> Self {
        Self {
            base,
            controller,
            clock_hz: 250_000_000,  // 250MHz core clock
        }
    }

    /// Get pointer to SPI registers
    fn regs(&self) -> *mut SpiRegs {
        self.base as *mut SpiRegs
    }

    /// Read CS register
    fn read_cs(&self) -> u32 {
        unsafe { read_volatile(&(*self.regs()).cs) }
    }

    /// Write CS register
    fn write_cs(&self, val: u32) {
        unsafe { write_volatile(&mut (*self.regs()).cs, val) }
    }

    /// Read FIFO register
    fn read_fifo(&self) -> u32 {
        unsafe { read_volatile(&(*self.regs()).fifo) }
    }

    /// Write FIFO register
    fn write_fifo(&self, val: u32) {
        unsafe { write_volatile(&mut (*self.regs()).fifo, val) }
    }

    /// Write CLK register (clock divider)
    fn write_clk(&self, val: u32) {
        unsafe { write_volatile(&mut (*self.regs()).clk, val) }
    }

    /// Initialize SPI controller
    pub fn initialize(&self, mode: SpiMode, speed_hz: u32) -> DriverResult<()> {
        // Clear FIFOs
        self.write_cs(CS_CLEAR_RX | CS_CLEAR_TX);

        // Set clock divider
        let divider = self.clock_hz / speed_hz;
        let divider = if divider < 2 { 2 } else { divider & !1 };  // Must be even, minimum 2
        self.write_clk(divider);

        // Configure mode
        let cs = mode.to_cs_bits();
        self.write_cs(cs);

        Ok(())
    }

    /// Transfer data (full duplex)
    ///
    /// Sends `tx_data` and receives data into `rx_data`.
    /// Both buffers must have the same length.
    pub fn transfer(
        &self,
        cs: ChipSelect,
        tx_data: &[u8],
        rx_data: &mut [u8],
    ) -> DriverResult<usize> {
        if tx_data.len() != rx_data.len() {
            return Err(DriverError::InvalidParameter);
        }

        let len = tx_data.len();
        if len == 0 {
            return Ok(0);
        }

        // Select chip select
        let mut cs_reg = self.read_cs();
        cs_reg &= !CS_CS_MASK;
        cs_reg |= cs as u32;
        cs_reg &= !(CS_CPOL | CS_CPHA);  // Preserve mode bits
        self.write_cs(cs_reg);

        // Clear FIFOs
        cs_reg |= CS_CLEAR_RX | CS_CLEAR_TX;
        self.write_cs(cs_reg);

        // Start transfer
        cs_reg |= CS_TA;
        self.write_cs(cs_reg);

        let mut tx_idx = 0;
        let mut rx_idx = 0;

        while rx_idx < len {
            // Write to TX FIFO
            while tx_idx < len && (self.read_cs() & CS_TXD) != 0 {
                self.write_fifo(tx_data[tx_idx] as u32);
                tx_idx += 1;
            }

            // Read from RX FIFO
            while rx_idx < len && (self.read_cs() & CS_RXD) != 0 {
                rx_data[rx_idx] = self.read_fifo() as u8;
                rx_idx += 1;
            }

            // Check for timeout (simple counter-based)
            if rx_idx < len && tx_idx >= len {
                // Wait for remaining RX data
                let mut timeout = 100000;
                while (self.read_cs() & CS_DONE) == 0 && timeout > 0 {
                    timeout -= 1;
                }
                if timeout == 0 {
                    // Clear TA bit and return error
                    let mut cs_reg = self.read_cs();
                    cs_reg &= !CS_TA;
                    self.write_cs(cs_reg);
                    let timeout_err = crate::drivers::timeout::TimeoutError::new(100000, 100000);
                    return Err(DriverError::Timeout(timeout_err));
                }
            }
        }

        // Wait for transfer to complete
        let mut timeout = 100000;
        while (self.read_cs() & CS_DONE) == 0 && timeout > 0 {
            timeout -= 1;
        }

        // Clear TA bit
        let mut cs_reg = self.read_cs();
        cs_reg &= !CS_TA;
        self.write_cs(cs_reg);

        if timeout == 0 {
            let timeout_err = crate::drivers::timeout::TimeoutError::new(100000, 100000);
            return Err(DriverError::Timeout(timeout_err));
        }

        Ok(len)
    }

    /// Write-only transfer
    pub fn write(&self, cs: ChipSelect, data: &[u8]) -> DriverResult<usize> {
        let mut dummy_rx = alloc::vec![0u8; data.len()];
        self.transfer(cs, data, &mut dummy_rx)
    }

    /// Read-only transfer (sends zeros)
    pub fn read(&self, cs: ChipSelect, data: &mut [u8]) -> DriverResult<usize> {
        let dummy_tx = alloc::vec![0u8; data.len()];
        self.transfer(cs, &dummy_tx, data)
    }

    /// Set SPI speed
    pub fn set_speed(&self, speed_hz: u32) -> DriverResult<u32> {
        let divider = self.clock_hz / speed_hz;
        let divider = if divider < 2 { 2 } else { divider & !1 };
        self.write_clk(divider);
        Ok(self.clock_hz / divider)
    }

    /// Get current status
    pub fn get_status(&self) -> u32 {
        self.read_cs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spi_mode_bits() {
        assert_eq!(SpiMode::Mode0.to_cs_bits(), 0);
        assert_eq!(SpiMode::Mode1.to_cs_bits(), CS_CPHA);
        assert_eq!(SpiMode::Mode2.to_cs_bits(), CS_CPOL);
        assert_eq!(SpiMode::Mode3.to_cs_bits(), CS_CPOL | CS_CPHA);
    }

    #[test]
    fn test_chip_select_values() {
        assert_eq!(ChipSelect::Cs0 as u32, 0);
        assert_eq!(ChipSelect::Cs1 as u32, 1);
        assert_eq!(ChipSelect::Cs2 as u32, 2);
    }
}
