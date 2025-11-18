//! USB Driver Infrastructure for Raspberry Pi 5
//!
//! This module provides USB support via the XHCI (eXtensible Host Controller Interface)
//! host controller, which is accessed through the RP1 I/O Hub on Raspberry Pi 5.
//!
//! # Architecture
//!
//! ```text
//! RP1 I/O Hub (PCIe)
//!   └─> USB 3.0 XHCI Controller
//!         ├─> Root Hub Ports
//!         │     ├─> USB 3.0 Devices (SuperSpeed)
//!         │     └─> USB 2.0 Devices (High/Full/Low Speed)
//!         ├─> Device Enumeration
//!         ├─> Descriptor Parsing
//!         └─> Class Drivers
//!               ├─> HID (Keyboard, Mouse)
//!               ├─> Mass Storage (USB drives)
//!               ├─> UVC (Cameras) - Phase 5
//!               └─> Audio - Phase 6
//! ```
//!
//! # USB Device Classes
//!
//! - **HID (0x03):** Human Interface Devices (keyboard, mouse, gamepad)
//! - **Mass Storage (0x08):** USB drives, SD card readers
//! - **Video (0x0E):** Webcams, capture devices (UVC)
//! - **Audio (0x01):** Microphones, speakers, headsets
//! - **Hub (0x09):** USB hubs for port expansion
//!
//! # References
//! - USB 3.2 Specification
//! - XHCI Specification Rev 1.2
//! - USB Device Class Specifications

pub mod xhci;
pub mod core;
pub mod descriptor;

use crate::drivers::{DriverError, DriverResult};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Once;

/// USB device class codes
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceClass {
    /// Per-interface class
    PerInterface = 0x00,

    /// Audio device
    Audio = 0x01,

    /// Communications device
    Communications = 0x02,

    /// Human Interface Device (HID)
    Hid = 0x03,

    /// Physical device
    Physical = 0x05,

    /// Still imaging device
    Image = 0x06,

    /// Printer
    Printer = 0x07,

    /// Mass storage device
    MassStorage = 0x08,

    /// Hub
    Hub = 0x09,

    /// CDC-Data
    CdcData = 0x0A,

    /// Smart Card
    SmartCard = 0x0B,

    /// Content Security
    ContentSecurity = 0x0D,

    /// Video device
    Video = 0x0E,

    /// Personal Healthcare
    PersonalHealthcare = 0x0F,

    /// Audio/Video device
    AudioVideo = 0x10,

    /// Diagnostic device
    Diagnostic = 0xDC,

    /// Wireless controller
    Wireless = 0xE0,

    /// Miscellaneous
    Miscellaneous = 0xEF,

    /// Application specific
    ApplicationSpecific = 0xFE,

    /// Vendor specific
    VendorSpecific = 0xFF,
}

impl DeviceClass {
    /// Create from raw u8 value
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::PerInterface,
            0x01 => Self::Audio,
            0x02 => Self::Communications,
            0x03 => Self::Hid,
            0x05 => Self::Physical,
            0x06 => Self::Image,
            0x07 => Self::Printer,
            0x08 => Self::MassStorage,
            0x09 => Self::Hub,
            0x0A => Self::CdcData,
            0x0B => Self::SmartCard,
            0x0D => Self::ContentSecurity,
            0x0E => Self::Video,
            0x0F => Self::PersonalHealthcare,
            0x10 => Self::AudioVideo,
            0xDC => Self::Diagnostic,
            0xE0 => Self::Wireless,
            0xEF => Self::Miscellaneous,
            0xFE => Self::ApplicationSpecific,
            0xFF => Self::VendorSpecific,
            _ => Self::VendorSpecific,
        }
    }

    /// Get class name
    pub fn name(&self) -> &'static str {
        match self {
            Self::PerInterface => "Per-Interface",
            Self::Audio => "Audio",
            Self::Communications => "Communications",
            Self::Hid => "HID",
            Self::Physical => "Physical",
            Self::Image => "Image",
            Self::Printer => "Printer",
            Self::MassStorage => "Mass Storage",
            Self::Hub => "Hub",
            Self::CdcData => "CDC-Data",
            Self::SmartCard => "Smart Card",
            Self::ContentSecurity => "Content Security",
            Self::Video => "Video",
            Self::PersonalHealthcare => "Personal Healthcare",
            Self::AudioVideo => "Audio/Video",
            Self::Diagnostic => "Diagnostic",
            Self::Wireless => "Wireless",
            Self::Miscellaneous => "Miscellaneous",
            Self::ApplicationSpecific => "Application Specific",
            Self::VendorSpecific => "Vendor Specific",
        }
    }
}

/// USB device speed
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeviceSpeed {
    /// Low Speed (1.5 Mbps) - USB 1.0
    Low,

    /// Full Speed (12 Mbps) - USB 1.1
    Full,

    /// High Speed (480 Mbps) - USB 2.0
    High,

    /// SuperSpeed (5 Gbps) - USB 3.0
    Super,

    /// SuperSpeed+ (10 Gbps) - USB 3.1
    SuperPlus,
}

/// USB device information
#[derive(Debug, Clone)]
pub struct UsbDevice {
    /// Device ID (slot number)
    pub id: u8,

    /// Port number (0-based)
    pub port: u8,

    /// Device address
    pub address: u8,

    /// Vendor ID
    pub vendor_id: u16,

    /// Product ID
    pub product_id: u16,

    /// Device class
    pub class: DeviceClass,

    /// Device sub-class
    pub sub_class: u8,

    /// Device protocol
    pub protocol: u8,

    /// Device speed
    pub speed: DeviceSpeed,

    /// Manufacturer string
    pub manufacturer: alloc::string::String,

    /// Product string
    pub product: alloc::string::String,

    /// Serial number string
    pub serial: alloc::string::String,

    /// Device name (for convenience)
    pub name: alloc::string::String,
}

impl UsbDevice {
    /// Create a new USB device
    pub fn new(id: u8, port: u8) -> Self {
        Self {
            id,
            port,
            address: 0,
            vendor_id: 0,
            product_id: 0,
            class: DeviceClass::PerInterface,
            sub_class: 0,
            protocol: 0,
            speed: DeviceSpeed::Full,
            manufacturer: alloc::string::String::new(),
            product: alloc::string::String::new(),
            serial: alloc::string::String::new(),
            name: alloc::string::String::new(),
        }
    }
}

/// Global USB subsystem state
struct UsbState {
    /// XHCI controller
    xhci: xhci::XhciController,

    /// Initialization complete
    initialized: AtomicBool,
}

/// Global USB state instance
static USB_STATE: Once<UsbState> = Once::new();

/// Initialize USB subsystem
///
/// This function should be called after RP1 initialization to set up
/// the USB XHCI host controller.
///
/// # Returns
/// Ok(()) if initialization succeeds, or an error if:
/// - RP1 not initialized
/// - XHCI controller not found
/// - XHCI initialization fails
pub fn initialize() -> DriverResult<()> {
    // Get RP1 driver
    let rp1 = crate::drivers::pcie::get_rp1()
        .ok_or(DriverError::NotInitialized)?;

    crate::info!("[USB] Initializing USB subsystem");

    // Initialize XHCI controller
    let xhci = xhci::XhciController::new(rp1)?;
    xhci.initialize()?;

    // Store global state
    USB_STATE.call_once(|| UsbState {
        xhci,
        initialized: AtomicBool::new(true),
    });

    crate::info!("[USB] USB subsystem initialized");

    Ok(())
}

/// Check if USB subsystem is initialized
pub fn is_initialized() -> bool {
    USB_STATE
        .get()
        .map(|state| state.initialized.load(Ordering::Acquire))
        .unwrap_or(false)
}

/// Get XHCI controller reference
pub fn get_xhci() -> Option<&'static xhci::XhciController> {
    USB_STATE.get().map(|state| &state.xhci)
}

/// Enumerate all USB devices
///
/// Scans all root hub ports and returns a list of connected devices.
///
/// # Returns
/// Vector of USB device information
pub fn enumerate_devices() -> alloc::vec::Vec<UsbDevice> {
    if let Some(xhci) = get_xhci() {
        xhci.enumerate_devices()
    } else {
        alloc::vec::Vec::new()
    }
}

/// Find devices by class
///
/// Returns all devices matching the specified class code.
pub fn find_devices_by_class(class: DeviceClass) -> alloc::vec::Vec<UsbDevice> {
    enumerate_devices()
        .into_iter()
        .filter(|dev| dev.class == class)
        .collect()
}

/// Find device by vendor and product ID
pub fn find_device_by_vid_pid(vendor_id: u16, product_id: u16) -> Option<UsbDevice> {
    enumerate_devices()
        .into_iter()
        .find(|dev| dev.vendor_id == vendor_id && dev.product_id == product_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_class_conversion() {
        assert_eq!(DeviceClass::from_u8(0x03), DeviceClass::Hid);
        assert_eq!(DeviceClass::from_u8(0x08), DeviceClass::MassStorage);
        assert_eq!(DeviceClass::from_u8(0x0E), DeviceClass::Video);
    }

    #[test]
    fn test_device_class_names() {
        assert_eq!(DeviceClass::Hid.name(), "HID");
        assert_eq!(DeviceClass::Video.name(), "Video");
        assert_eq!(DeviceClass::MassStorage.name(), "Mass Storage");
    }
}
