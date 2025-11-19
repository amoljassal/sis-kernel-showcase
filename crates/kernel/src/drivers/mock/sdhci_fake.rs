//! Fake SDHCI MMIO bus for testing the driver state machine (very simple)
//!
//! This implementation returns values that let the command path and single
//! block PIO read path proceed without timeouts.

#[cfg(feature = "mock-devices")]
pub struct SimpleSdhciOk;

#[cfg(feature = "mock-devices")]
impl SimpleSdhciOk {
    pub const fn new() -> Self { Self }
}

#[cfg(feature = "mock-devices")]
impl super::super::block::sdhci::SdhciBus for SimpleSdhciOk {
    fn read_u32(&self, offset: usize) -> u32 {
        // INT_STATUS: always report requested events complete
        const SDHCI_INT_STATUS: usize = 0x30;
        const SDHCI_PRESENT_STATE: usize = 0x24;
        const INT_STATUS_COMMAND_COMPLETE: u32 = 1 << 0;
        const INT_STATUS_TRANSFER_COMPLETE: u32 = 1 << 1;
        const INT_STATUS_BUFFER_READ_READY: u32 = 1 << 5;
        const PRESENT_STATE_CMD_INHIBIT: u32 = 1 << 0;

        match offset {
            SDHCI_PRESENT_STATE => {
                // Command/data lines ready
                0 & !PRESENT_STATE_CMD_INHIBIT
            }
            SDHCI_INT_STATUS => {
                INT_STATUS_COMMAND_COMPLETE | INT_STATUS_BUFFER_READ_READY | INT_STATUS_TRANSFER_COMPLETE
            }
            _ => 0
        }
    }

    fn write_u32(&self, _offset: usize, _value: u32) { /* ignore */ }
    fn read_u16(&self, _offset: usize) -> u16 { 0 }
    fn write_u16(&self, _offset: usize, _value: u16) {}
    fn read_u8(&self, _offset: usize) -> u8 { 0 }
    fn write_u8(&self, _offset: usize, _value: u8) {}
}

#[cfg(feature = "mock-devices")]
pub fn install_simple_fake_sdhci_ok() {
    static BUS: SimpleSdhciOk = SimpleSdhciOk::new();
    super::super::block::sdhci::set_fake_bus(&BUS);
}

