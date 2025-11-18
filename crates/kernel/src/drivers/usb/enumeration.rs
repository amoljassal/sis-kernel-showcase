//! USB Core - Device Enumeration and Management
//!
//! Handles USB device detection, enumeration, descriptor reading, and
//! driver matching for devices connected to the XHCI host controller.

use super::{UsbDevice, DeviceClass, DeviceSpeed};
use super::descriptor::{
    DeviceDescriptor, ConfigDescriptor, InterfaceDescriptor,
    EndpointDescriptor, StringDescriptor, DescriptorIterator, DescriptorType,
};
use crate::drivers::{DriverError, DriverResult};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// USB request type
#[derive(Debug, Copy, Clone)]
pub struct RequestType(pub u8);

impl RequestType {
    /// Standard device request
    pub const STANDARD_DEVICE_OUT: u8 = 0x00;
    pub const STANDARD_DEVICE_IN: u8 = 0x80;
    pub const STANDARD_INTERFACE_IN: u8 = 0x81;
    pub const STANDARD_ENDPOINT_IN: u8 = 0x82;

    /// Class-specific requests
    pub const CLASS_INTERFACE_OUT: u8 = 0x21;
    pub const CLASS_INTERFACE_IN: u8 = 0xA1;
}

/// USB standard requests
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Request {
    GetStatus = 0,
    ClearFeature = 1,
    SetFeature = 3,
    SetAddress = 5,
    GetDescriptor = 6,
    SetDescriptor = 7,
    GetConfiguration = 8,
    SetConfiguration = 9,
    GetInterface = 10,
    SetInterface = 11,
    SynchFrame = 12,
}

/// USB control transfer setup packet
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct SetupPacket {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupPacket {
    /// Create GET_DESCRIPTOR request
    pub fn get_descriptor(desc_type: u8, desc_index: u8, lang_id: u16, length: u16) -> Self {
        Self {
            request_type: RequestType::STANDARD_DEVICE_IN,
            request: Request::GetDescriptor as u8,
            value: ((desc_type as u16) << 8) | (desc_index as u16),
            index: lang_id,
            length,
        }
    }

    /// Create SET_ADDRESS request
    pub fn set_address(address: u8) -> Self {
        Self {
            request_type: RequestType::STANDARD_DEVICE_OUT,
            request: Request::SetAddress as u8,
            value: address as u16,
            index: 0,
            length: 0,
        }
    }

    /// Create SET_CONFIGURATION request
    pub fn set_configuration(config_value: u8) -> Self {
        Self {
            request_type: RequestType::STANDARD_DEVICE_OUT,
            request: Request::SetConfiguration as u8,
            value: config_value as u16,
            index: 0,
            length: 0,
        }
    }

    /// Create GET_CONFIGURATION request
    pub fn get_configuration() -> Self {
        Self {
            request_type: RequestType::STANDARD_DEVICE_IN,
            request: Request::GetConfiguration as u8,
            value: 0,
            index: 0,
            length: 1,
        }
    }
}

/// USB device enumerator
pub struct DeviceEnumerator {
    /// Device being enumerated
    device: UsbDevice,

    /// Raw device descriptor
    device_desc: Option<DeviceDescriptor>,

    /// Raw configuration descriptor
    config_desc: Option<ConfigDescriptor>,

    /// Full configuration data (includes interfaces and endpoints)
    config_data: Vec<u8>,
}

impl DeviceEnumerator {
    /// Create new device enumerator
    pub fn new(slot_id: u8, port: u8, speed: DeviceSpeed) -> Self {
        let mut device = UsbDevice::new(slot_id, port);
        device.speed = speed;

        Self {
            device,
            device_desc: None,
            config_desc: None,
            config_data: Vec::new(),
        }
    }

    /// Enumerate device (full enumeration sequence)
    ///
    /// This performs the complete USB enumeration sequence:
    /// 1. Read device descriptor (initial 8 bytes to get max packet size)
    /// 2. Reset device
    /// 3. Set device address
    /// 4. Read full device descriptor
    /// 5. Read configuration descriptor
    /// 6. Read string descriptors
    /// 7. Set configuration
    ///
    /// Returns the fully enumerated device information.
    pub fn enumerate<F>(
        &mut self,
        mut control_transfer: F,
    ) -> DriverResult<UsbDevice>
    where
        F: FnMut(&SetupPacket, &mut [u8]) -> DriverResult<usize>,
    {
        // Step 1: Read first 8 bytes of device descriptor (to get max packet size)
        let mut buf = [0u8; 8];
        let setup = SetupPacket::get_descriptor(
            DescriptorType::Device as u8,
            0,
            0,
            8,
        );

        control_transfer(&setup, &mut buf)?;

        // Parse partial descriptor to get max packet size
        if buf.len() >= 8 {
            let max_packet_size = buf[7];
            crate::debug!("[USB] Device max packet size: {}", max_packet_size);
        }

        // Step 2: Set device address
        let address = self.device.id;  // Use slot ID as address
        let setup = SetupPacket::set_address(address);
        control_transfer(&setup, &mut [])?;
        self.device.address = address;

        crate::time::sleep_ms(10);  // Give device time to process

        // Step 3: Read full device descriptor
        let mut desc_buf = [0u8; 18];
        let setup = SetupPacket::get_descriptor(
            DescriptorType::Device as u8,
            0,
            0,
            18,
        );

        control_transfer(&setup, &mut desc_buf)?;

        if let Some(dev_desc) = DeviceDescriptor::parse(&desc_buf) {
            self.device_desc = Some(dev_desc);
            self.device.vendor_id = dev_desc.vendor_id;
            self.device.product_id = dev_desc.product_id;
            self.device.class = dev_desc.class();
            self.device.sub_class = dev_desc.device_sub_class;
            self.device.protocol = dev_desc.device_protocol;

            crate::info!(
                "[USB] Device descriptor: VID={:04x} PID={:04x} Class={}",
                self.device.vendor_id,
                self.device.product_id,
                self.device.class.name()
            );

            // Step 4: Read configuration descriptor (header first)
            let mut config_header = [0u8; 9];
            let setup = SetupPacket::get_descriptor(
                DescriptorType::Configuration as u8,
                0,
                0,
                9,
            );

            control_transfer(&setup, &mut config_header)?;

            if let Some(cfg_desc) = ConfigDescriptor::parse(&config_header) {
                let total_len = cfg_desc.total_length as usize;

                // Read full configuration descriptor (all interfaces and endpoints)
                self.config_data = alloc::vec![0u8; total_len];
                let setup = SetupPacket::get_descriptor(
                    DescriptorType::Configuration as u8,
                    0,
                    0,
                    total_len as u16,
                );

                control_transfer(&setup, &mut self.config_data)?;

                // Parse configuration
                if let Some(cfg) = ConfigDescriptor::parse(&self.config_data) {
                    self.config_desc = Some(cfg);

                    crate::debug!(
                        "[USB] Configuration: {} interfaces, max power: {} mA",
                        cfg.num_interfaces,
                        cfg.max_power_ma()
                    );
                }
            }

            // Step 5: Read string descriptors
            self.read_string_descriptors(&mut control_transfer, &dev_desc)?;

            // Step 6: Set configuration
            if let Some(cfg) = self.config_desc {
                let setup = SetupPacket::set_configuration(cfg.configuration_value);
                control_transfer(&setup, &mut [])?;

                crate::debug!("[USB] Configuration set to {}", cfg.configuration_value);
            }

            // Build device name
            if !self.device.product.is_empty() {
                self.device.name = self.device.product.clone();
            } else if !self.device.manufacturer.is_empty() {
                self.device.name = format!(
                    "{} Device",
                    self.device.manufacturer
                );
            } else {
                self.device.name = format!(
                    "{} Device",
                    self.device.class.name()
                );
            }

            Ok(self.device.clone())
        } else {
            Err(DriverError::VerificationFailed)
        }
    }

    /// Read string descriptors (manufacturer, product, serial)
    fn read_string_descriptors<F>(
        &mut self,
        control_transfer: &mut F,
        dev_desc: &DeviceDescriptor,
    ) -> DriverResult<()>
    where
        F: FnMut(&SetupPacket, &mut [u8]) -> DriverResult<usize>,
    {
        // Get language IDs
        let mut lang_buf = [0u8; 255];
        let setup = SetupPacket::get_descriptor(
            DescriptorType::String as u8,
            0,
            0,
            255,
        );

        if control_transfer(&setup, &mut lang_buf).is_ok() {
            let lang_ids = StringDescriptor::parse_language_ids(&lang_buf);
            let lang_id = lang_ids.first().copied().unwrap_or(0x0409);  // Default to English (US)

            // Read manufacturer string
            if dev_desc.manufacturer_index != 0 {
                let mut str_buf = [0u8; 255];
                let setup = SetupPacket::get_descriptor(
                    DescriptorType::String as u8,
                    dev_desc.manufacturer_index,
                    lang_id,
                    255,
                );

                if control_transfer(&setup, &mut str_buf).is_ok() {
                    if let Some(s) = StringDescriptor::parse(&str_buf) {
                        self.device.manufacturer = s;
                    }
                }
            }

            // Read product string
            if dev_desc.product_index != 0 {
                let mut str_buf = [0u8; 255];
                let setup = SetupPacket::get_descriptor(
                    DescriptorType::String as u8,
                    dev_desc.product_index,
                    lang_id,
                    255,
                );

                if control_transfer(&setup, &mut str_buf).is_ok() {
                    if let Some(s) = StringDescriptor::parse(&str_buf) {
                        self.device.product = s;
                    }
                }
            }

            // Read serial number string
            if dev_desc.serial_index != 0 {
                let mut str_buf = [0u8; 255];
                let setup = SetupPacket::get_descriptor(
                    DescriptorType::String as u8,
                    dev_desc.serial_index,
                    lang_id,
                    255,
                );

                if control_transfer(&setup, &mut str_buf).is_ok() {
                    if let Some(s) = StringDescriptor::parse(&str_buf) {
                        self.device.serial = s;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get parsed configuration data
    pub fn get_config_data(&self) -> &[u8] {
        &self.config_data
    }

    /// Get interfaces from configuration
    pub fn get_interfaces(&self) -> Vec<InterfaceDescriptor> {
        let mut interfaces = Vec::new();
        let mut iter = DescriptorIterator::new(&self.config_data);

        // Skip configuration descriptor
        iter.next_descriptor();

        while let Some(desc_data) = iter.next_descriptor() {
            if let Some(iface) = InterfaceDescriptor::parse(desc_data) {
                interfaces.push(iface);
            }
        }

        interfaces
    }

    /// Get endpoints for a specific interface
    pub fn get_endpoints(&self, interface_num: u8) -> Vec<EndpointDescriptor> {
        let mut endpoints = Vec::new();
        let mut iter = DescriptorIterator::new(&self.config_data);
        let mut in_interface = false;

        while let Some(desc_data) = iter.next_descriptor() {
            if desc_data.len() >= 2 {
                match desc_data[1] {
                    4 => {
                        // Interface descriptor
                        if let Some(iface) = InterfaceDescriptor::parse(desc_data) {
                            in_interface = iface.interface_number == interface_num;
                        }
                    }
                    5 if in_interface => {
                        // Endpoint descriptor
                        if let Some(ep) = EndpointDescriptor::parse(desc_data) {
                            endpoints.push(ep);
                        }
                    }
                    _ => {}
                }
            }
        }

        endpoints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_packet_get_descriptor() {
        let setup = SetupPacket::get_descriptor(1, 0, 0, 18);
        assert_eq!(setup.request_type, RequestType::STANDARD_DEVICE_IN);
        assert_eq!(setup.request, Request::GetDescriptor as u8);
        assert_eq!(setup.value, 0x0100);  // Device descriptor
        assert_eq!(setup.length, 18);
    }

    #[test]
    fn test_setup_packet_set_address() {
        let setup = SetupPacket::set_address(5);
        assert_eq!(setup.request_type, RequestType::STANDARD_DEVICE_OUT);
        assert_eq!(setup.request, Request::SetAddress as u8);
        assert_eq!(setup.value, 5);
        assert_eq!(setup.length, 0);
    }
}
