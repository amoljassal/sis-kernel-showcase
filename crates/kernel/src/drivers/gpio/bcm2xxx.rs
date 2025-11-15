//! BCM2xxx GPIO Driver for Raspberry Pi 5 (BCM2712)
//!
//! This driver provides GPIO control for the BCM2712 SoC on Raspberry Pi 5.
//! It supports basic GPIO operations: pin function selection, set/clear pins,
//! and read pin state.
//!
//! ## Hardware Details
//! - BCM2712 has 54 GPIO pins (GPIO0-GPIO53)
//! - Each pin can be configured for different functions (GPIO, ALT0-ALT5)
//! - Pins are controlled via memory-mapped registers
//!
//! ## Register Layout
//! - GPFSEL0-GPFSEL5: Function select (3 bits per pin, 10 pins per register)
//! - GPSET0-GPSET1: Output set registers (write 1 to set pin high)
//! - GPCLR0-GPCLR1: Output clear registers (write 1 to set pin low)
//! - GPLEV0-GPLEV1: Pin level registers (read current pin state)
//!
//! ## M6 Implementation (GPIO/Mailbox)

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// GPIO register offsets from base address
const GPFSEL0: usize = 0x00;    // Function Select 0 (GPIO 0-9)
const GPFSEL1: usize = 0x04;    // Function Select 1 (GPIO 10-19)
const GPFSEL2: usize = 0x08;    // Function Select 2 (GPIO 20-29)
const GPFSEL3: usize = 0x0C;    // Function Select 3 (GPIO 30-39)
const GPFSEL4: usize = 0x10;    // Function Select 4 (GPIO 40-49)
const GPFSEL5: usize = 0x14;    // Function Select 5 (GPIO 50-53)

const GPSET0: usize = 0x1C;     // Output Set 0 (GPIO 0-31)
const GPSET1: usize = 0x20;     // Output Set 1 (GPIO 32-53)

const GPCLR0: usize = 0x28;     // Output Clear 0 (GPIO 0-31)
const GPCLR1: usize = 0x2C;     // Output Clear 1 (GPIO 32-53)

const GPLEV0: usize = 0x34;     // Pin Level 0 (GPIO 0-31)
const GPLEV1: usize = 0x38;     // Pin Level 1 (GPIO 32-53)

const GPPUD: usize = 0x94;      // Pull-up/down control
const GPPUDCLK0: usize = 0x98;  // Pull-up/down clock 0 (GPIO 0-31)
const GPPUDCLK1: usize = 0x9C;  // Pull-up/down clock 1 (GPIO 32-53)

/// GPIO pin function modes
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GpioFunction {
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010,
}

/// GPIO pull-up/down modes
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GpioPull {
    Off = 0b00,
    Down = 0b01,
    Up = 0b10,
}

/// BCM2xxx GPIO controller
pub struct BcmGpio {
    base: usize,
}

impl BcmGpio {
    /// Create a new GPIO controller at the given base address
    ///
    /// # Safety
    /// The caller must ensure that `base` points to a valid GPIO controller
    /// memory region and that access to this region is safe.
    pub const fn new(base: usize) -> Self {
        Self { base }
    }

    /// Set the function of a GPIO pin
    ///
    /// # Arguments
    /// * `pin` - GPIO pin number (0-53)
    /// * `func` - Function mode to set
    ///
    /// # Example
    /// ```
    /// gpio.set_function(42, GpioFunction::Output);  // Set GPIO 42 as output
    /// ```
    pub fn set_function(&self, pin: u32, func: GpioFunction) {
        if pin >= 54 {
            return; // Invalid pin number
        }

        let reg_offset = GPFSEL0 + ((pin / 10) * 4) as usize;
        let bit_offset = (pin % 10) * 3;

        unsafe {
            let mut val = self.read_reg(reg_offset);
            val &= !(0b111 << bit_offset);  // Clear function bits
            val |= (func as u32) << bit_offset;  // Set new function
            self.write_reg(reg_offset, val);
        }
    }

    /// Set a GPIO pin high (output mode)
    ///
    /// # Arguments
    /// * `pin` - GPIO pin number (0-53)
    ///
    /// # Example
    /// ```
    /// gpio.set_pin(42);  // Set GPIO 42 high
    /// ```
    pub fn set_pin(&self, pin: u32) {
        if pin >= 54 {
            return;
        }

        let (reg_offset, bit) = if pin < 32 {
            (GPSET0, pin)
        } else {
            (GPSET1, pin - 32)
        };

        unsafe {
            self.write_reg(reg_offset, 1 << bit);
        }
    }

    /// Clear a GPIO pin low (output mode)
    ///
    /// # Arguments
    /// * `pin` - GPIO pin number (0-53)
    ///
    /// # Example
    /// ```
    /// gpio.clear_pin(42);  // Set GPIO 42 low
    /// ```
    pub fn clear_pin(&self, pin: u32) {
        if pin >= 54 {
            return;
        }

        let (reg_offset, bit) = if pin < 32 {
            (GPCLR0, pin)
        } else {
            (GPCLR1, pin - 32)
        };

        unsafe {
            self.write_reg(reg_offset, 1 << bit);
        }
    }

    /// Read the current level of a GPIO pin
    ///
    /// # Arguments
    /// * `pin` - GPIO pin number (0-53)
    ///
    /// # Returns
    /// `true` if pin is high, `false` if pin is low
    pub fn read_pin(&self, pin: u32) -> bool {
        if pin >= 54 {
            return false;
        }

        let (reg_offset, bit) = if pin < 32 {
            (GPLEV0, pin)
        } else {
            (GPLEV1, pin - 32)
        };

        unsafe {
            let val = self.read_reg(reg_offset);
            (val & (1 << bit)) != 0
        }
    }

    /// Toggle a GPIO pin
    ///
    /// # Arguments
    /// * `pin` - GPIO pin number (0-53)
    pub fn toggle_pin(&self, pin: u32) {
        if self.read_pin(pin) {
            self.clear_pin(pin);
        } else {
            self.set_pin(pin);
        }
    }

    /// Set pull-up/down resistor for a GPIO pin
    ///
    /// # Arguments
    /// * `pin` - GPIO pin number (0-53)
    /// * `pull` - Pull mode (Off, Down, Up)
    pub fn set_pull(&self, pin: u32, pull: GpioPull) {
        if pin >= 54 {
            return;
        }

        unsafe {
            // Set pull mode in GPPUD register
            self.write_reg(GPPUD, pull as u32);

            // Wait 150 cycles for control signal to settle
            for _ in 0..150 {
                core::hint::spin_loop();
            }

            // Clock the control signal into the pin
            let (clk_offset, bit) = if pin < 32 {
                (GPPUDCLK0, pin)
            } else {
                (GPPUDCLK1, pin - 32)
            };
            self.write_reg(clk_offset, 1 << bit);

            // Wait 150 cycles
            for _ in 0..150 {
                core::hint::spin_loop();
            }

            // Remove control signal and clock
            self.write_reg(GPPUD, 0);
            self.write_reg(clk_offset, 0);
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

// Global GPIO instance
static GPIO_BASE: AtomicUsize = AtomicUsize::new(0);
static GPIO_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the GPIO controller
///
/// # Arguments
/// * `base` - Base address of the GPIO controller
///
/// # Safety
/// The caller must ensure that the base address is valid and accessible
pub unsafe fn init(base: usize) {
    GPIO_BASE.store(base, Ordering::Release);
    GPIO_INITIALIZED.store(true, Ordering::Release);

    crate::info!("[GPIO] BCM GPIO initialized at {:#x}", base);
}

/// Get a reference to the global GPIO controller
///
/// # Returns
/// `Some(BcmGpio)` if initialized, `None` otherwise
pub fn get_gpio() -> Option<BcmGpio> {
    if GPIO_INITIALIZED.load(Ordering::Acquire) {
        let base = GPIO_BASE.load(Ordering::Acquire);
        Some(BcmGpio::new(base))
    } else {
        None
    }
}

/// Check if GPIO is initialized
pub fn is_initialized() -> bool {
    GPIO_INITIALIZED.load(Ordering::Acquire)
}

/// Set a GPIO pin function (convenience wrapper)
pub fn set_function(pin: u32, func: GpioFunction) {
    if let Some(gpio) = get_gpio() {
        gpio.set_function(pin, func);
    }
}

/// Set a GPIO pin high (convenience wrapper)
pub fn set_pin(pin: u32) {
    if let Some(gpio) = get_gpio() {
        gpio.set_pin(pin);
    }
}

/// Clear a GPIO pin low (convenience wrapper)
pub fn clear_pin(pin: u32) {
    if let Some(gpio) = get_gpio() {
        gpio.clear_pin(pin);
    }
}

/// Read a GPIO pin level (convenience wrapper)
pub fn read_pin(pin: u32) -> bool {
    if let Some(gpio) = get_gpio() {
        gpio.read_pin(pin)
    } else {
        false
    }
}

/// Toggle a GPIO pin (convenience wrapper)
pub fn toggle_pin(pin: u32) {
    if let Some(gpio) = get_gpio() {
        gpio.toggle_pin(pin);
    }
}

/// Set pull resistor (convenience wrapper)
pub fn set_pull(pin: u32, pull: GpioPull) {
    if let Some(gpio) = get_gpio() {
        gpio.set_pull(pin, pull);
    }
}
