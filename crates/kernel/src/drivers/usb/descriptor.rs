//! USB Descriptor Parsing
//!
//! Implements parsing of USB descriptors from raw data returned by devices
//! during enumeration. Supports standard descriptors and class-specific
//! descriptors for HID, Mass Storage, Video (UVC), and Audio.

use super::{DeviceClass, DeviceSpeed};
use core::mem;

/// Descriptor types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DescriptorType {
    Device = 1,
    Configuration = 2,
    String = 3,
    Interface = 4,
    Endpoint = 5,
    DeviceQualifier = 6,
    OtherSpeedConfiguration = 7,
    InterfacePower = 8,
    Otg = 9,
    Debug = 10,
    InterfaceAssociation = 11,
    Bos = 15,
    DeviceCapability = 16,
    SuperSpeedUsbEndpointCompanion = 48,
    SuperSpeedPlusIsochronousEndpointCompanion = 49,
}

impl DescriptorType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::Device),
            2 => Some(Self::Configuration),
            3 => Some(Self::String),
            4 => Some(Self::Interface),
            5 => Some(Self::Endpoint),
            6 => Some(Self::DeviceQualifier),
            7 => Some(Self::OtherSpeedConfiguration),
            8 => Some(Self::InterfacePower),
            9 => Some(Self::Otg),
            10 => Some(Self::Debug),
            11 => Some(Self::InterfaceAssociation),
            15 => Some(Self::Bos),
            16 => Some(Self::DeviceCapability),
            48 => Some(Self::SuperSpeedUsbEndpointCompanion),
            49 => Some(Self::SuperSpeedPlusIsochronousEndpointCompanion),
            _ => None,
        }
    }
}

/// USB Device Descriptor (18 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub usb_version: u16,
    pub device_class: u8,
    pub device_sub_class: u8,
    pub device_protocol: u8,
    pub max_packet_size0: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: u16,
    pub manufacturer_index: u8,
    pub product_index: u8,
    pub serial_index: u8,
    pub num_configurations: u8,
}

impl DeviceDescriptor {
    /// Parse device descriptor from raw bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 18 || data[0] != 18 || data[1] != 1 {
            return None;
        }

        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }

    /// Get USB version as (major, minor)
    pub fn usb_version(&self) -> (u8, u8) {
        let major = (self.usb_version >> 8) as u8;
        let minor = (self.usb_version & 0xFF) as u8;
        (major, minor)
    }

    /// Get device class
    pub fn class(&self) -> DeviceClass {
        DeviceClass::from_u8(self.device_class)
    }
}

/// USB Configuration Descriptor (9 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ConfigDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_index: u8,
    pub attributes: u8,
    pub max_power: u8,
}

impl ConfigDescriptor {
    /// Parse configuration descriptor from raw bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 9 || data[0] != 9 || data[1] != 2 {
            return None;
        }

        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }

    /// Is self-powered
    pub fn is_self_powered(&self) -> bool {
        (self.attributes & 0x40) != 0
    }

    /// Supports remote wakeup
    pub fn remote_wakeup(&self) -> bool {
        (self.attributes & 0x20) != 0
    }

    /// Maximum power consumption in mA
    pub fn max_power_ma(&self) -> u16 {
        (self.max_power as u16) * 2
    }
}

/// USB Interface Descriptor (9 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct InterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_sub_class: u8,
    pub interface_protocol: u8,
    pub interface_index: u8,
}

impl InterfaceDescriptor {
    /// Parse interface descriptor from raw bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 9 || data[0] != 9 || data[1] != 4 {
            return None;
        }

        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }

    /// Get interface class
    pub fn class(&self) -> DeviceClass {
        DeviceClass::from_u8(self.interface_class)
    }
}

/// Endpoint direction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EndpointDirection {
    Out = 0,
    In = 1,
}

/// Endpoint transfer type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransferType {
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

/// USB Endpoint Descriptor (7+ bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EndpointDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

impl EndpointDescriptor {
    /// Parse endpoint descriptor from raw bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 7 || data[0] < 7 || data[1] != 5 {
            return None;
        }

        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }

    /// Get endpoint number (0-15)
    pub fn number(&self) -> u8 {
        self.endpoint_address & 0x0F
    }

    /// Get endpoint direction
    pub fn direction(&self) -> EndpointDirection {
        if (self.endpoint_address & 0x80) != 0 {
            EndpointDirection::In
        } else {
            EndpointDirection::Out
        }
    }

    /// Get transfer type
    pub fn transfer_type(&self) -> TransferType {
        match self.attributes & 0x03 {
            0 => TransferType::Control,
            1 => TransferType::Isochronous,
            2 => TransferType::Bulk,
            3 => TransferType::Interrupt,
            _ => TransferType::Control,
        }
    }

    /// Is isochronous endpoint
    pub fn is_isochronous(&self) -> bool {
        matches!(self.transfer_type(), TransferType::Isochronous)
    }

    /// Is bulk endpoint
    pub fn is_bulk(&self) -> bool {
        matches!(self.transfer_type(), TransferType::Bulk)
    }

    /// Is interrupt endpoint
    pub fn is_interrupt(&self) -> bool {
        matches!(self.transfer_type(), TransferType::Interrupt)
    }
}

/// String descriptor parsing
pub struct StringDescriptor;

impl StringDescriptor {
    /// Parse UTF-16 string descriptor to UTF-8 String
    pub fn parse(data: &[u8]) -> Option<alloc::string::String> {
        if data.len() < 2 || data[1] != 3 {
            return None;
        }

        let len = data[0] as usize;
        if data.len() < len || len < 2 {
            return None;
        }

        // Skip header (2 bytes), read UTF-16LE characters
        let mut result = alloc::string::String::new();
        let mut i = 2;

        while i + 1 < len {
            let code_unit = u16::from_le_bytes([data[i], data[i + 1]]);

            // Simple UTF-16 to UTF-8 conversion (only BMP characters)
            if code_unit < 0x80 {
                result.push(code_unit as u8 as char);
            } else if code_unit < 0x800 {
                let byte1 = 0xC0 | ((code_unit >> 6) as u8);
                let byte2 = 0x80 | ((code_unit & 0x3F) as u8);
                result.push(byte1 as char);
                result.push(byte2 as char);
            } else {
                let byte1 = 0xE0 | ((code_unit >> 12) as u8);
                let byte2 = 0x80 | (((code_unit >> 6) & 0x3F) as u8);
                let byte3 = 0x80 | ((code_unit & 0x3F) as u8);
                result.push(byte1 as char);
                result.push(byte2 as char);
                result.push(byte3 as char);
            }

            i += 2;
        }

        Some(result)
    }

    /// Get language ID list from string descriptor 0
    pub fn parse_language_ids(data: &[u8]) -> alloc::vec::Vec<u16> {
        if data.len() < 2 || data[1] != 3 {
            return alloc::vec::Vec::new();
        }

        let len = data[0] as usize;
        if data.len() < len || len < 2 {
            return alloc::vec::Vec::new();
        }

        let mut langs = alloc::vec::Vec::new();
        let mut i = 2;

        while i + 1 < len {
            let lang_id = u16::from_le_bytes([data[i], data[i + 1]]);
            langs.push(lang_id);
            i += 2;
        }

        langs
    }
}

/// HID Descriptor (9+ bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct HidDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub hid_version: u16,
    pub country_code: u8,
    pub num_descriptors: u8,
    pub report_descriptor_type: u8,
    pub report_descriptor_length: u16,
}

impl HidDescriptor {
    /// Parse HID descriptor from raw bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 9 || data[0] < 9 || data[1] != 0x21 {
            return None;
        }

        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }
}

/// Descriptor iterator for parsing configuration descriptors
pub struct DescriptorIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> DescriptorIterator<'a> {
    /// Create new descriptor iterator
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Get next descriptor
    pub fn next_descriptor(&mut self) -> Option<&'a [u8]> {
        if self.offset >= self.data.len() {
            return None;
        }

        let length = self.data[self.offset] as usize;
        if length == 0 || self.offset + length > self.data.len() {
            return None;
        }

        let desc = &self.data[self.offset..self.offset + length];
        self.offset += length;

        Some(desc)
    }

    /// Find next descriptor of specific type
    pub fn find_descriptor(&mut self, desc_type: DescriptorType) -> Option<&'a [u8]> {
        while let Some(desc) = self.next_descriptor() {
            if desc.len() >= 2 && desc[1] == desc_type as u8 {
                return Some(desc);
            }
        }
        None
    }

    /// Reset iterator to beginning
    pub fn reset(&mut self) {
        self.offset = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_direction() {
        // OUT endpoint
        let ep_out = EndpointDescriptor {
            length: 7,
            descriptor_type: 5,
            endpoint_address: 0x01,  // EP1 OUT
            attributes: 0x02,         // Bulk
            max_packet_size: 512,
            interval: 0,
        };
        assert_eq!(ep_out.direction(), EndpointDirection::Out);
        assert_eq!(ep_out.number(), 1);

        // IN endpoint
        let ep_in = EndpointDescriptor {
            length: 7,
            descriptor_type: 5,
            endpoint_address: 0x81,  // EP1 IN
            attributes: 0x02,         // Bulk
            max_packet_size: 512,
            interval: 0,
        };
        assert_eq!(ep_in.direction(), EndpointDirection::In);
        assert_eq!(ep_in.number(), 1);
    }

    #[test]
    fn test_transfer_type() {
        let ep = EndpointDescriptor {
            length: 7,
            descriptor_type: 5,
            endpoint_address: 0x81,
            attributes: 0x02,  // Bulk
            max_packet_size: 512,
            interval: 0,
        };
        assert_eq!(ep.transfer_type(), TransferType::Bulk);
        assert!(ep.is_bulk());
    }
}
