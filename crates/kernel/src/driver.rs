//! Driver framework for SIS kernel
//!
//! Provides a unified interface for device drivers with support for:
//! - Device discovery and enumeration
//! - Driver registration and binding
//! - Interrupt handling integration
//! - VirtIO transport layer support

use core::fmt;
use heapless::Vec;

/// Maximum number of registered drivers
const MAX_DRIVERS: usize = 32;

/// Driver framework errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// No suitable driver found for device
    NoDriver,
    /// Driver initialization failed
    InitFailed,
    /// Invalid device configuration
    InvalidDevice,
    /// Resource allocation failed
    ResourceError,
    /// Operation not supported
    NotSupported,
    /// Driver registry is full
    RegistryFull,
    /// Invalid or unavailable virtqueue
    InvalidQueue,
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverError::NoDriver => write!(f, "No suitable driver found"),
            DriverError::InitFailed => write!(f, "Driver initialization failed"),
            DriverError::InvalidDevice => write!(f, "Invalid device configuration"),
            DriverError::ResourceError => write!(f, "Resource allocation failed"),
            DriverError::NotSupported => write!(f, "Operation not supported"),
            DriverError::RegistryFull => write!(f, "Driver registry is full"),
            DriverError::InvalidQueue => write!(f, "Invalid or unavailable virtqueue"),
        }
    }
}

/// Device identification and metadata
#[derive(Debug, Clone, Copy)]
pub struct DeviceId {
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID  
    pub device_id: u16,
    /// Device class
    pub class: u8,
    /// Device subclass
    pub subclass: u8,
}

/// Physical device information
#[derive(Debug, Clone, Copy)]
pub struct DeviceInfo {
    /// Device identification
    pub id: DeviceId,
    /// Base memory address
    pub base_addr: u64,
    /// Memory region size
    pub size: u64,
    /// IRQ line (if applicable)
    pub irq: Option<u32>,
    /// Device-specific data
    pub device_data: u64,
}

/// Driver capabilities and metadata
#[derive(Debug, Clone, Copy)]
pub struct DriverInfo {
    /// Driver name
    pub name: &'static str,
    /// Driver version
    pub version: &'static str,
    /// Supported device IDs
    pub supported_devices: &'static [DeviceId],
}

/// Driver operation results
pub type DriverResult<T = ()> = Result<T, DriverError>;

/// Core driver trait that all drivers must implement
pub trait Driver {
    /// Get driver information
    fn info(&self) -> DriverInfo;

    /// Check if this driver supports the given device
    fn probe(&self, device: &DeviceInfo) -> bool;

    /// Initialize the driver for a specific device
    fn init(&mut self, device: &DeviceInfo) -> DriverResult<()>;

    /// Start the driver (called after successful init)
    fn start(&mut self) -> DriverResult<()> {
        Ok(())
    }

    /// Stop the driver
    fn stop(&mut self) -> DriverResult<()> {
        Ok(())
    }

    /// Handle device interrupt (if applicable)
    fn handle_irq(&mut self) -> DriverResult<()> {
        Ok(())
    }

    /// Driver-specific I/O operations
    fn read(&mut self, offset: u64, buffer: &mut [u8]) -> DriverResult<usize> {
        let _ = (offset, buffer);
        Err(DriverError::NotSupported)
    }

    fn write(&mut self, offset: u64, data: &[u8]) -> DriverResult<usize> {
        let _ = (offset, data);
        Err(DriverError::NotSupported)
    }

    /// Device control operations
    fn ioctl(&mut self, cmd: u32, arg: u64) -> DriverResult<u64> {
        let _ = (cmd, arg);
        Err(DriverError::NotSupported)
    }
}

/// Registered driver instance
struct DriverInstance {
    driver: &'static mut dyn Driver,
    device: Option<DeviceInfo>,
    active: bool,
}

/// Global driver registry
static mut DRIVER_REGISTRY: Option<DriverRegistry> = None;

/// Driver registry for managing all system drivers
pub struct DriverRegistry {
    drivers: Vec<DriverInstance, MAX_DRIVERS>,
    initialized: bool,
}

impl DriverRegistry {
    /// Create new driver registry
    pub const fn new() -> Self {
        Self {
            drivers: Vec::new(),
            initialized: false,
        }
    }

    /// Initialize the driver registry
    pub fn init(&mut self) -> DriverResult<()> {
        if self.initialized {
            return Ok(());
        }

        unsafe {
            crate::uart_print(b"[DRIVER] Initializing driver registry\n");
        }

        self.initialized = true;
        Ok(())
    }

    /// Register a new driver
    pub fn register_driver(&mut self, driver: &'static mut dyn Driver) -> DriverResult<()> {
        if self.drivers.len() >= MAX_DRIVERS {
            return Err(DriverError::RegistryFull);
        }

        let info = driver.info();
        unsafe {
            crate::uart_print(b"[DRIVER] Registering driver: ");
            crate::uart_print(info.name.as_bytes());
            crate::uart_print(b" v");
            crate::uart_print(info.version.as_bytes());
            crate::uart_print(b"\n");
        }

        self.drivers
            .push(DriverInstance {
                driver,
                device: None,
                active: false,
            })
            .map_err(|_| DriverError::RegistryFull)?;

        Ok(())
    }

    /// Discover and bind drivers to devices using VirtIO
    pub fn discover_devices(&mut self) -> DriverResult<usize> {
        unsafe {
            crate::uart_print(b"[DRIVER] Starting device discovery\n");
        }

        let mut bound_count = 0;

        // Use VirtIO device discovery
        match crate::virtio::VirtIODiscovery::discover_devices() {
            Ok(devices) => {
                for device in &devices {
                    unsafe {
                        crate::uart_print(b"[DRIVER] Found device: vendor=");
                        self.print_hex16(device.id.vendor_id);
                        crate::uart_print(b" device=");
                        self.print_hex16(device.id.device_id);
                        crate::uart_print(b" class=");
                        self.print_hex8(device.id.class);
                        crate::uart_print(b"\n");
                    }

                    if let Ok(_) = self.bind_device(device) {
                        bound_count += 1;
                    }
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(
                        b"[DRIVER] VirtIO discovery failed, using fallback devices\n",
                    );
                }

                // Fallback to a single hinted device if platform provides virtio-mmio hint
                if let Some((base, slot_size, irq_base)) = crate::platform::active().virtio_mmio_hint() {
                    let test_devices = [DeviceInfo {
                        id: DeviceId {
                            vendor_id: 0x1AF4, // Red Hat (VirtIO)
                            device_id: 0x0003, // VirtIO Console
                            class: 0x07,       // Communication controller
                            subclass: 0x80,    // Other
                        },
                        base_addr: base as u64,
                        size: slot_size as u64,
                        irq: Some(irq_base as u32),
                        device_data: 2, // VirtIO 1.0+ version
                    }];

                    for device in &test_devices {
                        unsafe {
                            crate::uart_print(b"[DRIVER] Fallback device: vendor=");
                            self.print_hex16(device.id.vendor_id);
                            crate::uart_print(b" device=");
                            self.print_hex16(device.id.device_id);
                            crate::uart_print(b"\n");
                        }

                        if let Ok(_) = self.bind_device(device) {
                            bound_count += 1;
                        }
                    }
                } else {
                    unsafe { crate::uart_print(b"[DRIVER] No virtio-mmio hint; skipping fallback device\n"); }
                }
            }
        }

        unsafe {
            crate::uart_print(b"[DRIVER] Device discovery complete, bound ");
            self.print_number(bound_count);
            crate::uart_print(b" devices\n");
        }

        Ok(bound_count)
    }

    /// Bind a device to an appropriate driver
    pub fn bind_device(&mut self, device: &DeviceInfo) -> DriverResult<()> {
        // Find a driver that supports this device
        for instance in &mut self.drivers {
            if !instance.active && instance.driver.probe(device) {
                unsafe {
                    crate::uart_print(b"[DRIVER] Binding device to driver: ");
                    crate::uart_print(instance.driver.info().name.as_bytes());
                    crate::uart_print(b"\n");
                }

                // Initialize the driver with this device
                if let Err(e) = instance.driver.init(device) {
                    unsafe {
                        crate::uart_print(b"[DRIVER] Driver init failed\n");
                    }
                    return Err(e);
                }

                // Start the driver
                if let Err(e) = instance.driver.start() {
                    unsafe {
                        crate::uart_print(b"[DRIVER] Driver start failed\n");
                    }
                    return Err(e);
                }

                instance.device = Some(*device);
                instance.active = true;

                unsafe {
                    crate::uart_print(b"[DRIVER] Device binding successful\n");
                }
                return Ok(());
            }
        }

        unsafe {
            crate::uart_print(b"[DRIVER] No suitable driver found for device\n");
        }
        Err(DriverError::NoDriver)
    }

    /// Get active driver count
    pub fn active_driver_count(&self) -> usize {
        self.drivers.iter().filter(|d| d.active).count()
    }

    /// Handle interrupt for a specific device
    pub fn handle_device_irq(&mut self, irq: u32) -> DriverResult<()> {
        for instance in &mut self.drivers {
            if instance.active {
                if let Some(device) = &instance.device {
                    if device.irq == Some(irq) {
                        return instance.driver.handle_irq();
                    }
                }
            }
        }
        Err(DriverError::NoDriver)
    }

    /// Helper function to print hex numbers
    unsafe fn print_hex16(&self, num: u16) {
        crate::uart_print(b"0x");
        for i in (0..4).rev() {
            let nibble = (num >> (i * 4)) & 0xF;
            let c = if nibble < 10 {
                b'0' + nibble as u8
            } else {
                b'A' + (nibble - 10) as u8
            };
            crate::uart_print(&[c]);
        }
    }

    /// Helper function to print 8-bit hex numbers
    unsafe fn print_hex8(&self, num: u8) {
        crate::uart_print(b"0x");
        for i in (0..2).rev() {
            let nibble = (num >> (i * 4)) & 0xF;
            let c = if nibble < 10 {
                b'0' + nibble as u8
            } else {
                b'A' + (nibble - 10) as u8
            };
            crate::uart_print(&[c]);
        }
    }

    /// Helper function to print numbers
    unsafe fn print_number(&self, mut num: usize) {
        if num == 0 {
            crate::uart_print(b"0");
            return;
        }

        let mut digits = [0u8; 20];
        let mut i = 0;

        while num > 0 {
            digits[i] = b'0' + (num % 10) as u8;
            num /= 10;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            crate::uart_print(&[digits[i]]);
        }
    }
}

/// Initialize the global driver registry
pub fn init_driver_framework() -> DriverResult<()> {
    unsafe {
        DRIVER_REGISTRY = Some(DriverRegistry::new());
        let registry_ptr = &raw mut DRIVER_REGISTRY;
        if let Some(registry) = (*registry_ptr).as_mut() {
            registry.init()
        } else {
            Err(DriverError::InitFailed)
        }
    }
}

/// Get a mutable reference to the global driver registry
pub fn get_driver_registry() -> Option<&'static mut DriverRegistry> {
    unsafe {
        let registry_ptr = &raw mut DRIVER_REGISTRY;
        (*registry_ptr).as_mut()
    }
}

/// Register a driver with the global registry
pub fn register_driver(driver: &'static mut dyn Driver) -> DriverResult<()> {
    if let Some(registry) = get_driver_registry() {
        registry.register_driver(driver)
    } else {
        Err(DriverError::InitFailed)
    }
}

/// Discover and bind all devices
pub fn discover_devices() -> DriverResult<usize> {
    if let Some(registry) = get_driver_registry() {
        registry.discover_devices()
    } else {
        Err(DriverError::InitFailed)
    }
}
