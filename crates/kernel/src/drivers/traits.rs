// Device Trait Abstractions
// Phase 6 - Production Readiness Plan
//
// Trait-based abstractions for hardware devices to enable mocking and testing

use crate::lib::error::{Errno, Result};
use alloc::vec::Vec;

/// Block device trait - represents block-based storage
pub trait BlockDevice: Send + Sync {
    /// Read a block from the device
    fn read(&self, block: u64, buf: &mut [u8]) -> Result<()>;

    /// Write a block to the device
    fn write(&self, block: u64, buf: &[u8]) -> Result<()>;

    /// Flush any pending writes to the device
    fn flush(&self) -> Result<()>;

    /// Get the block size in bytes
    fn block_size(&self) -> usize;

    /// Get the total number of blocks
    fn block_count(&self) -> u64;

    /// Get total capacity in bytes
    fn capacity(&self) -> u64 {
        self.block_count() * (self.block_size() as u64)
    }

    /// Check if device is read-only
    fn is_readonly(&self) -> bool {
        false
    }

    /// Get device name/identifier
    fn name(&self) -> &str;

    /// Sync the device (ensure all writes are persisted)
    fn sync(&self) -> Result<()> {
        self.flush()
    }
}

/// Network device trait - represents network interface
pub trait NetworkDevice: Send + Sync {
    /// Send a packet
    fn send(&self, packet: &[u8]) -> Result<()>;

    /// Receive a packet (non-blocking)
    fn recv(&self, buf: &mut [u8]) -> Result<usize>;

    /// Get MAC address
    fn mac_address(&self) -> [u8; 6];

    /// Get MTU (Maximum Transmission Unit)
    fn mtu(&self) -> usize {
        1500 // Default Ethernet MTU
    }

    /// Check if link is up
    fn link_up(&self) -> bool;

    /// Get device name
    fn name(&self) -> &str;

    /// Get statistics
    fn stats(&self) -> NetworkStats;
}

/// Network statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkStats {
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
}

/// Character device trait - represents byte-oriented devices
pub trait CharDevice: Send + Sync {
    /// Read bytes from the device
    fn read(&self, buf: &mut [u8]) -> Result<usize>;

    /// Write bytes to the device
    fn write(&self, buf: &[u8]) -> Result<usize>;

    /// Perform device-specific control operation
    fn ioctl(&self, cmd: u64, arg: u64) -> Result<u64>;

    /// Check if device is ready for reading
    fn can_read(&self) -> bool;

    /// Check if device is ready for writing
    fn can_write(&self) -> bool;

    /// Get device name
    fn name(&self) -> &str;
}

/// Timer device trait - represents hardware timer
pub trait TimerDevice: Send + Sync {
    /// Get current timer value (ticks since boot)
    fn read(&self) -> u64;

    /// Get timer frequency in Hz
    fn frequency(&self) -> u64;

    /// Set timer to fire after given ticks
    fn set_timeout(&self, ticks: u64) -> Result<()>;

    /// Cancel pending timeout
    fn cancel_timeout(&self) -> Result<()>;

    /// Get nanoseconds since boot
    fn nanos(&self) -> u64 {
        let ticks = self.read();
        let freq = self.frequency();
        (ticks * 1_000_000_000) / freq
    }

    /// Get microseconds since boot
    fn micros(&self) -> u64 {
        self.nanos() / 1000
    }

    /// Get milliseconds since boot
    fn millis(&self) -> u64 {
        self.micros() / 1000
    }
}

/// Display device trait - represents framebuffer/display
pub trait DisplayDevice: Send + Sync {
    /// Get display width in pixels
    fn width(&self) -> usize;

    /// Get display height in pixels
    fn height(&self) -> usize;

    /// Get bytes per pixel
    fn bpp(&self) -> usize;

    /// Get framebuffer address
    fn framebuffer(&self) -> *mut u8;

    /// Write pixel at (x, y) with color
    fn write_pixel(&mut self, x: usize, y: usize, color: u32) -> Result<()>;

    /// Fill rectangle with color
    fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) -> Result<()>;

    /// Blit buffer to display
    fn blit(&mut self, src: &[u8], x: usize, y: usize, w: usize, h: usize) -> Result<()>;

    /// Present/flip buffers
    fn present(&mut self) -> Result<()>;
}

/// Input device trait - represents keyboard, mouse, etc.
pub trait InputDevice: Send + Sync {
    /// Read next input event
    fn read_event(&self) -> Result<InputEvent>;

    /// Check if events are available
    fn has_events(&self) -> bool;

    /// Get device type
    fn device_type(&self) -> InputDeviceType;

    /// Get device name
    fn name(&self) -> &str;
}

/// Input device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDeviceType {
    Keyboard,
    Mouse,
    Touchscreen,
    Gamepad,
}

/// Input events
#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    KeyPress(u32),
    KeyRelease(u32),
    MouseMove { x: i32, y: i32 },
    MouseButton { button: u8, pressed: bool },
    MouseWheel { delta: i8 },
}

/// RTC (Real-Time Clock) device trait
pub trait RtcDevice: Send + Sync {
    /// Read current time (Unix timestamp in seconds)
    fn read_time(&self) -> Result<u64>;

    /// Write time (Unix timestamp in seconds)
    fn write_time(&self, timestamp: u64) -> Result<()>;

    /// Get date/time components
    fn read_datetime(&self) -> Result<DateTime>;

    /// Set date/time components
    fn write_datetime(&self, datetime: &DateTime) -> Result<()>;
}

/// Date/Time representation
#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,     // 1-12
    pub day: u8,       // 1-31
    pub hour: u8,      // 0-23
    pub minute: u8,    // 0-59
    pub second: u8,    // 0-59
}

/// Random number generator device trait
pub trait RngDevice: Send + Sync {
    /// Fill buffer with random bytes
    fn fill_bytes(&self, buf: &mut [u8]) -> Result<()>;

    /// Get random u32
    fn next_u32(&self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.fill_bytes(&mut buf)?;
        Ok(u32::from_ne_bytes(buf))
    }

    /// Get random u64
    fn next_u64(&self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.fill_bytes(&mut buf)?;
        Ok(u64::from_ne_bytes(buf))
    }
}

/// DMA (Direct Memory Access) buffer
pub struct DmaBuffer {
    pub virt_addr: *mut u8,
    pub phys_addr: u64,
    pub size: usize,
}

unsafe impl Send for DmaBuffer {}
unsafe impl Sync for DmaBuffer {}

impl DmaBuffer {
    /// Create a new DMA buffer
    pub fn new(virt_addr: *mut u8, phys_addr: u64, size: usize) -> Self {
        Self {
            virt_addr,
            phys_addr,
            size,
        }
    }

    /// Get as mutable slice
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        core::slice::from_raw_parts_mut(self.virt_addr, self.size)
    }

    /// Get as slice
    pub unsafe fn as_slice(&self) -> &[u8] {
        core::slice::from_raw_parts(self.virt_addr, self.size)
    }
}

/// GPIO pin trait
pub trait GpioPin: Send + Sync {
    /// Set pin as output
    fn set_output(&mut self) -> Result<()>;

    /// Set pin as input
    fn set_input(&mut self) -> Result<()>;

    /// Set pin high
    fn set_high(&mut self) -> Result<()>;

    /// Set pin low
    fn set_low(&mut self) -> Result<()>;

    /// Read pin value
    fn read(&self) -> Result<bool>;

    /// Toggle pin
    fn toggle(&mut self) -> Result<()> {
        if self.read()? {
            self.set_low()
        } else {
            self.set_high()
        }
    }
}
