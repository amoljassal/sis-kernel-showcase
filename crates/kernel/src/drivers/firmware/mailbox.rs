//! Raspberry Pi Mailbox Interface
//!
//! This driver provides communication with the VideoCore firmware on
//! Raspberry Pi via the mailbox property interface. This allows the
//! kernel to query hardware information, manage clocks, and configure
//! various system parameters.
//!
//! ## Protocol
//! The mailbox uses a request/response protocol with tagged messages.
//! Each message contains a sequence of property tags with request/response data.
//!
//! ## M6 Implementation (GPIO/Mailbox)
//! ## M8 Hardening Applied: Timeout framework, error handling, alignment validation

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::drivers::{DriverError, DriverResult, Timeout, Validator};

/// Mailbox register offsets
const MAILBOX_READ: usize = 0x00;
const MAILBOX_POLL: usize = 0x10;
const MAILBOX_SENDER: usize = 0x14;
const MAILBOX_STATUS: usize = 0x18;
const MAILBOX_CONFIG: usize = 0x1C;
const MAILBOX_WRITE: usize = 0x20;

/// Mailbox status register bits
const MAILBOX_FULL: u32 = 0x80000000;
const MAILBOX_EMPTY: u32 = 0x40000000;

/// Mailbox channels
const MAILBOX_CHANNEL_PROPERTY: u32 = 8;

/// Request/response codes
const MAILBOX_REQUEST: u32 = 0x00000000;
const MAILBOX_RESPONSE_SUCCESS: u32 = 0x80000000;
const MAILBOX_RESPONSE_ERROR: u32 = 0x80000001;

/// Property tag IDs
pub mod tags {
    /// Get board serial number
    pub const GET_BOARD_SERIAL: u32 = 0x00010004;

    /// Get ARM memory region
    pub const GET_ARM_MEMORY: u32 = 0x00010005;

    /// Get VC memory region
    pub const GET_VC_MEMORY: u32 = 0x00010006;

    /// Get board model
    pub const GET_BOARD_MODEL: u32 = 0x00010001;

    /// Get board revision
    pub const GET_BOARD_REVISION: u32 = 0x00010002;

    /// Get firmware revision
    pub const GET_FIRMWARE_REVISION: u32 = 0x00000001;

    /// Get temperature
    pub const GET_TEMPERATURE: u32 = 0x00030006;

    /// Get max temperature
    pub const GET_MAX_TEMPERATURE: u32 = 0x0003000A;

    /// Get clock rate
    pub const GET_CLOCK_RATE: u32 = 0x00030002;

    /// Get voltage
    pub const GET_VOLTAGE: u32 = 0x00030003;

    /// Set clock rate
    pub const SET_CLOCK_RATE: u32 = 0x00038002;

    /// Set voltage
    pub const SET_VOLTAGE: u32 = 0x00038003;
}

/// Temperature sensor IDs
pub mod temp_id {
    pub const SOC: u32 = 0;  // SoC temperature
}

/// Clock IDs
pub mod clock_id {
    pub const EMMC: u32 = 1;
    pub const UART: u32 = 2;
    pub const ARM: u32 = 3;
    pub const CORE: u32 = 4;
    pub const V3D: u32 = 5;
    pub const H264: u32 = 6;
    pub const ISP: u32 = 7;
    pub const SDRAM: u32 = 8;
    pub const PIXEL: u32 = 9;
    pub const PWM: u32 = 10;
}

/// Voltage IDs
pub mod voltage_id {
    pub const CORE: u32 = 1;
    pub const SDRAM_C: u32 = 2;
    pub const SDRAM_P: u32 = 3;
    pub const SDRAM_I: u32 = 4;
}

/// Buffer alignment requirement (16 bytes for DMA)
const MAILBOX_BUFFER_ALIGNMENT: usize = 16;

/// Mailbox timeout (5 seconds for slow firmware operations)
const MAILBOX_TIMEOUT_US: u64 = 5_000_000;

/// Mailbox controller
pub struct Mailbox {
    base: usize,
}

impl Mailbox {
    /// Create a new mailbox controller at the given base address
    ///
    /// # Safety
    /// The caller must ensure that `base` points to a valid mailbox
    /// memory region and that access to this region is safe.
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    /// Send a mailbox message and wait for response
    ///
    /// # Arguments
    /// * `buffer` - Message buffer (must be 16-byte aligned)
    /// * `channel` - Mailbox channel (typically MAILBOX_CHANNEL_PROPERTY)
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err(DriverError)` on failure
    ///
    /// # Safety
    /// The buffer must be properly aligned and contain a valid message
    ///
    /// # M8 Hardening
    /// - Uses Timeout framework (5s timeout for firmware operations)
    /// - Validates 16-byte buffer alignment
    /// - Proper error types via DriverError
    unsafe fn call(&self, buffer: &mut [u32], channel: u32) -> DriverResult<()> {
        let addr = buffer.as_ptr() as usize;

        // M8: Validate buffer alignment (16-byte required for DMA)
        Validator::check_alignment(addr, MAILBOX_BUFFER_ALIGNMENT)?;

        // M8: Use Timeout framework instead of raw counter
        let timeout = Timeout::new(MAILBOX_TIMEOUT_US);

        // Wait for mailbox to be not full
        timeout.wait(|| (self.read_reg(MAILBOX_STATUS) & MAILBOX_FULL) == 0)?;

        // Write message address with channel in low 4 bits
        let msg = (addr & !0xF) as u32 | (channel & 0xF);
        self.write_reg(MAILBOX_WRITE, msg);

        // Wait for response with timeout
        let response_timeout = Timeout::new(MAILBOX_TIMEOUT_US);
        loop {
            // Wait for mailbox to be not empty
            if (self.read_reg(MAILBOX_STATUS) & MAILBOX_EMPTY) == 0 {
                // Read response
                let resp = self.read_reg(MAILBOX_READ);

                // Check if this is our response
                if (resp & 0xF) == channel {
                    // Check response code
                    if buffer[1] == MAILBOX_RESPONSE_SUCCESS {
                        return Ok(());
                    } else {
                        return Err(DriverError::HardwareError); // Firmware rejected request
                    }
                }
            }

            // Check timeout
            if response_timeout.is_expired() {
                return Err(DriverError::Timeout(crate::drivers::TimeoutError::new(
                    response_timeout.elapsed_us(),
                    MAILBOX_TIMEOUT_US,
                )));
            }

            core::hint::spin_loop();
        }
    }

    #[inline]
    unsafe fn read_reg(&self, offset: usize) -> u32 {
        read_volatile((self.base + offset) as *const u32)
    }

    #[inline]
    unsafe fn write_reg(&self, offset: usize, value: u32) {
        write_volatile((self.base + offset) as *mut u32, value)
    }
}

// Global mailbox instance
static MAILBOX_BASE: AtomicUsize = AtomicUsize::new(0);
static MAILBOX_INITIALIZED: AtomicBool = AtomicBool::new(false);

// Aligned buffer for mailbox messages (must be 16-byte aligned)
#[repr(align(16))]
struct AlignedBuffer {
    data: [u32; 256],
}

static mut MAILBOX_BUFFER: AlignedBuffer = AlignedBuffer { data: [0; 256] };

/// Initialize the mailbox controller
///
/// # Arguments
/// * `base` - Base address of the mailbox controller
///
/// # Safety
/// The caller must ensure that the base address is valid and accessible
pub unsafe fn init(base: usize) {
    MAILBOX_BASE.store(base, Ordering::Release);
    MAILBOX_INITIALIZED.store(true, Ordering::Release);

    crate::info!("[MAILBOX] Initialized at {:#x}", base);
}

/// Check if mailbox is initialized
pub fn is_initialized() -> bool {
    MAILBOX_INITIALIZED.load(Ordering::Acquire)
}

/// Get board serial number
///
/// # M8 Hardening: Returns DriverResult with proper error handling
pub fn get_board_serial() -> DriverResult<u64> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 8 * 4;                      // Total size
        buffer[1] = MAILBOX_REQUEST;            // Request code
        buffer[2] = tags::GET_BOARD_SERIAL;     // Tag ID
        buffer[3] = 8;                          // Value buffer size
        buffer[4] = 0;                          // Request/response size
        buffer[5] = 0;                          // Serial low
        buffer[6] = 0;                          // Serial high
        buffer[7] = 0;                          // End tag

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        let serial = buffer[5] as u64 | ((buffer[6] as u64) << 32);
        Ok(serial)
    }
}

/// Get SoC temperature in millidegrees Celsius
///
/// # Example
/// ```
/// let temp = mailbox::get_temperature()?;
/// // temp is in millidegrees, so 45123 = 45.123°C
/// ```
///
/// # M8 Hardening: Returns DriverResult with proper error handling
pub fn get_temperature() -> DriverResult<u32> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 8 * 4;                      // Total size
        buffer[1] = MAILBOX_REQUEST;            // Request code
        buffer[2] = tags::GET_TEMPERATURE;      // Tag ID
        buffer[3] = 8;                          // Value buffer size
        buffer[4] = 4;                          // Request size (4 bytes)
        buffer[5] = temp_id::SOC;               // Temperature ID (0 = SoC)
        buffer[6] = 0;                          // Response: temperature
        buffer[7] = 0;                          // End tag

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        Ok(buffer[6])  // Temperature in millidegrees
    }
}

/// Get maximum temperature in millidegrees Celsius
///
/// # M8 Hardening: Returns DriverResult
pub fn get_max_temperature() -> DriverResult<u32> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 8 * 4;
        buffer[1] = MAILBOX_REQUEST;
        buffer[2] = tags::GET_MAX_TEMPERATURE;
        buffer[3] = 8;
        buffer[4] = 4;
        buffer[5] = temp_id::SOC;
        buffer[6] = 0;
        buffer[7] = 0;

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        Ok(buffer[6])
    }
}

/// Get firmware revision
///
/// # M8 Hardening: Returns DriverResult
pub fn get_firmware_revision() -> DriverResult<u32> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 7 * 4;
        buffer[1] = MAILBOX_REQUEST;
        buffer[2] = tags::GET_FIRMWARE_REVISION;
        buffer[3] = 4;
        buffer[4] = 0;
        buffer[5] = 0;
        buffer[6] = 0;

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        Ok(buffer[5])
    }
}

/// Get board model
///
/// # M8 Hardening: Returns DriverResult
pub fn get_board_model() -> DriverResult<u32> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 7 * 4;
        buffer[1] = MAILBOX_REQUEST;
        buffer[2] = tags::GET_BOARD_MODEL;
        buffer[3] = 4;
        buffer[4] = 0;
        buffer[5] = 0;
        buffer[6] = 0;

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        Ok(buffer[5])
    }
}

/// Get board revision
///
/// # M8 Hardening: Returns DriverResult
pub fn get_board_revision() -> DriverResult<u32> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 7 * 4;
        buffer[1] = MAILBOX_REQUEST;
        buffer[2] = tags::GET_BOARD_REVISION;
        buffer[3] = 4;
        buffer[4] = 0;
        buffer[5] = 0;
        buffer[6] = 0;

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        Ok(buffer[5])
    }
}

/// Get ARM memory region (base, size)
///
/// # M8 Hardening: Returns DriverResult
pub fn get_arm_memory() -> DriverResult<(u32, u32)> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 8 * 4;
        buffer[1] = MAILBOX_REQUEST;
        buffer[2] = tags::GET_ARM_MEMORY;
        buffer[3] = 8;
        buffer[4] = 0;
        buffer[5] = 0;  // Base address
        buffer[6] = 0;  // Size
        buffer[7] = 0;

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        Ok((buffer[5], buffer[6]))
    }
}

/// Get clock rate in Hz
///
/// # Arguments
/// * `clock_id` - Clock ID (use `clock_id::*` constants)
///
/// # M8 Hardening: Returns DriverResult
pub fn get_clock_rate(clock_id: u32) -> DriverResult<u32> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 8 * 4;
        buffer[1] = MAILBOX_REQUEST;
        buffer[2] = tags::GET_CLOCK_RATE;
        buffer[3] = 8;
        buffer[4] = 4;
        buffer[5] = clock_id;
        buffer[6] = 0;  // Rate
        buffer[7] = 0;

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        Ok(buffer[6])
    }
}

/// Get voltage in microvolts
///
/// # Arguments
/// * `voltage_id` - Voltage ID (use `voltage_id::*` constants)
///
/// # M8 Hardening: Returns DriverResult
pub fn get_voltage(voltage_id: u32) -> DriverResult<u32> {
    if !is_initialized() {
        return Err(DriverError::NotInitialized);
    }

    unsafe {
        let buffer = &mut MAILBOX_BUFFER.data;
        buffer[0] = 8 * 4;
        buffer[1] = MAILBOX_REQUEST;
        buffer[2] = tags::GET_VOLTAGE;
        buffer[3] = 8;
        buffer[4] = 4;
        buffer[5] = voltage_id;
        buffer[6] = 0;  // Voltage
        buffer[7] = 0;

        let base = MAILBOX_BASE.load(Ordering::Acquire);
        let mailbox = Mailbox::new(base);
        mailbox.call(buffer, MAILBOX_CHANNEL_PROPERTY)?;

        Ok(buffer[6])
    }
}
