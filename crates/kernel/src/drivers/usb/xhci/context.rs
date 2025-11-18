//! XHCI Device and Endpoint Contexts
//!
//! Defines the data structures used by the XHCI controller to track device state.

use crate::drivers::{DriverError, DriverResult};
use alloc::vec::Vec;

/// Device Context Base Address Array (DCBAA)
///
/// The DCBAA is an array of 64-bit pointers to device context structures.
/// Entry 0 is reserved for scratchpad buffers.
/// Entries 1-255 point to device contexts for slots 1-255.
pub struct DeviceContextArray {
    /// Array of device context pointers
    array: Vec<u64>,

    /// Physical address of array
    phys_addr: usize,

    /// Maximum number of device slots
    max_slots: usize,
}

impl DeviceContextArray {
    /// Create new DCBAA
    ///
    /// # Arguments
    /// * `max_slots` - Maximum number of device slots (from HCSPARAMS1)
    pub fn new(max_slots: usize) -> DriverResult<Self> {
        if max_slots == 0 || max_slots > 255 {
            return Err(DriverError::InvalidParameter);
        }

        // Allocate array (256 entries: 1 scratchpad + 255 device slots)
        let mut array = alloc::vec![0u64; 256];
        let phys_addr = array.as_ptr() as usize;

        Ok(Self {
            array,
            phys_addr,
            max_slots,
        })
    }

    /// Get physical address of DCBAA
    pub fn physical_addr(&self) -> usize {
        self.phys_addr
    }

    /// Set device context pointer for a slot
    pub fn set_device_context(&mut self, slot_id: u8, context_addr: u64) -> DriverResult<()> {
        if slot_id == 0 || slot_id as usize > self.max_slots {
            return Err(DriverError::InvalidParameter);
        }

        self.array[slot_id as usize] = context_addr;
        Ok(())
    }

    /// Get device context pointer for a slot
    pub fn get_device_context(&self, slot_id: u8) -> Option<u64> {
        if slot_id == 0 || slot_id as usize > self.max_slots {
            return None;
        }

        let addr = self.array[slot_id as usize];
        if addr != 0 {
            Some(addr)
        } else {
            None
        }
    }

    /// Clear device context pointer
    pub fn clear_device_context(&mut self, slot_id: u8) -> DriverResult<()> {
        if slot_id == 0 || slot_id as usize > self.max_slots {
            return Err(DriverError::InvalidParameter);
        }

        self.array[slot_id as usize] = 0;
        Ok(())
    }
}

/// Slot Context (32 bytes)
///
/// Contains device-level information.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(32))]
pub struct SlotContext {
    /// DWord 0: Route String, Speed, Context Entries
    pub dw0: u32,

    /// DWord 1: Max Exit Latency, Root Hub Port Number, Number of Ports
    pub dw1: u32,

    /// DWord 2: Parent Hub Slot ID, Parent Port Number, TT Think Time, Interrupter Target
    pub dw2: u32,

    /// DWord 3: USB Device Address, Slot State
    pub dw3: u32,

    /// Reserved
    _reserved: [u32; 4],
}

impl SlotContext {
    /// Create new slot context
    pub fn new() -> Self {
        Self {
            dw0: 0,
            dw1: 0,
            dw2: 0,
            dw3: 0,
            _reserved: [0; 4],
        }
    }

    /// Set route string
    pub fn set_route_string(&mut self, route_string: u32) {
        self.dw0 = (self.dw0 & !0xFFFFF) | (route_string & 0xFFFFF);
    }

    /// Set speed
    pub fn set_speed(&mut self, speed: u8) {
        self.dw0 = (self.dw0 & !(0xF << 20)) | ((speed as u32 & 0xF) << 20);
    }

    /// Set context entries (number of valid endpoint contexts)
    pub fn set_context_entries(&mut self, entries: u8) {
        self.dw0 = (self.dw0 & !(0x1F << 27)) | ((entries as u32 & 0x1F) << 27);
    }

    /// Set root hub port number
    pub fn set_root_hub_port(&mut self, port: u8) {
        self.dw1 = (self.dw1 & !(0xFF << 16)) | ((port as u32) << 16);
    }

    /// Set number of ports
    pub fn set_num_ports(&mut self, num_ports: u8) {
        self.dw1 = (self.dw1 & !(0xFF << 24)) | ((num_ports as u32) << 24);
    }

    /// Set interrupter target
    pub fn set_interrupter_target(&mut self, target: u16) {
        self.dw2 = (self.dw2 & !(0x3FF << 22)) | ((target as u32 & 0x3FF) << 22);
    }

    /// Set USB device address
    pub fn set_device_address(&mut self, address: u8) {
        self.dw3 = (self.dw3 & !0xFF) | (address as u32);
    }

    /// Set slot state
    pub fn set_slot_state(&mut self, state: u8) {
        self.dw3 = (self.dw3 & !(0x1F << 27)) | ((state as u32 & 0x1F) << 27);
    }

    /// Get device address
    pub fn device_address(&self) -> u8 {
        (self.dw3 & 0xFF) as u8
    }

    /// Get slot state
    pub fn slot_state(&self) -> u8 {
        ((self.dw3 >> 27) & 0x1F) as u8
    }
}

impl Default for SlotContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Endpoint Context (32 bytes)
///
/// Contains endpoint-level information.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(32))]
pub struct EndpointContext {
    /// DWord 0: Endpoint State, Mult, Max Primary Streams, LSA, Interval
    pub dw0: u32,

    /// DWord 1: Max ESIT Payload High, Error Count, Endpoint Type, HID, Max Burst Size, Max Packet Size
    pub dw1: u32,

    /// DWord 2: TR Dequeue Pointer Low, DCS
    pub dw2: u32,

    /// DWord 3: TR Dequeue Pointer High
    pub dw3: u32,

    /// DWord 4: Average TRB Length, Max ESIT Payload Low
    pub dw4: u32,

    /// Reserved
    _reserved: [u32; 3],
}

impl EndpointContext {
    /// Create new endpoint context
    pub fn new() -> Self {
        Self {
            dw0: 0,
            dw1: 0,
            dw2: 0,
            dw3: 0,
            dw4: 0,
            _reserved: [0; 3],
        }
    }

    /// Set endpoint state
    pub fn set_ep_state(&mut self, state: u8) {
        self.dw0 = (self.dw0 & !(0x7)) | (state as u32 & 0x7);
    }

    /// Set interval
    pub fn set_interval(&mut self, interval: u8) {
        self.dw0 = (self.dw0 & !(0xFF << 16)) | ((interval as u32) << 16);
    }

    /// Set error count
    pub fn set_error_count(&mut self, count: u8) {
        self.dw1 = (self.dw1 & !(0x3 << 1)) | ((count as u32 & 0x3) << 1);
    }

    /// Set endpoint type
    pub fn set_ep_type(&mut self, ep_type: u8) {
        self.dw1 = (self.dw1 & !(0x7 << 3)) | ((ep_type as u32 & 0x7) << 3);
    }

    /// Set max burst size
    pub fn set_max_burst_size(&mut self, size: u8) {
        self.dw1 = (self.dw1 & !(0xFF << 8)) | ((size as u32) << 8);
    }

    /// Set max packet size
    pub fn set_max_packet_size(&mut self, size: u16) {
        self.dw1 = (self.dw1 & !(0xFFFF << 16)) | ((size as u32) << 16);
    }

    /// Set transfer ring dequeue pointer
    pub fn set_tr_dequeue_pointer(&mut self, pointer: u64, dcs: bool) {
        self.dw2 = ((pointer & 0xFFFFFFF0) as u32) | if dcs { 0x1 } else { 0x0 };
        self.dw3 = (pointer >> 32) as u32;
    }

    /// Set average TRB length
    pub fn set_average_trb_length(&mut self, length: u16) {
        self.dw4 = (self.dw4 & !0xFFFF) | (length as u32);
    }

    /// Get endpoint state
    pub fn ep_state(&self) -> u8 {
        (self.dw0 & 0x7) as u8
    }

    /// Get transfer ring dequeue pointer
    pub fn tr_dequeue_pointer(&self) -> u64 {
        ((self.dw3 as u64) << 32) | ((self.dw2 & !0xF) as u64)
    }
}

impl Default for EndpointContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Device Context
///
/// Contains one slot context and up to 31 endpoint contexts.
#[derive(Debug)]
#[repr(C, align(64))]
pub struct DeviceContext {
    /// Slot context
    pub slot: SlotContext,

    /// Endpoint contexts (0-30, where 0 is control endpoint)
    pub endpoints: [EndpointContext; 31],
}

impl DeviceContext {
    /// Create new device context
    pub fn new() -> Self {
        Self {
            slot: SlotContext::new(),
            endpoints: [EndpointContext::new(); 31],
        }
    }

    /// Get endpoint context by DCI (Device Context Index)
    ///
    /// DCI 0 is reserved, DCI 1 is control endpoint OUT, DCI 2 is control endpoint IN, etc.
    pub fn get_endpoint(&self, dci: u8) -> Option<&EndpointContext> {
        if dci == 0 || dci > 31 {
            return None;
        }
        Some(&self.endpoints[(dci - 1) as usize])
    }

    /// Get mutable endpoint context by DCI
    pub fn get_endpoint_mut(&mut self, dci: u8) -> Option<&mut EndpointContext> {
        if dci == 0 || dci > 31 {
            return None;
        }
        Some(&mut self.endpoints[(dci - 1) as usize])
    }
}

impl Default for DeviceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Input Context
///
/// Used to communicate device context changes to the XHCI controller.
#[derive(Debug)]
#[repr(C, align(64))]
pub struct InputContext {
    /// Input Control Context (32 bytes)
    pub control: InputControlContext,

    /// Slot context
    pub slot: SlotContext,

    /// Endpoint contexts
    pub endpoints: [EndpointContext; 31],
}

impl InputContext {
    /// Create new input context
    pub fn new() -> Self {
        Self {
            control: InputControlContext::new(),
            slot: SlotContext::new(),
            endpoints: [EndpointContext::new(); 31],
        }
    }

    /// Get endpoint context by DCI
    pub fn get_endpoint_mut(&mut self, dci: u8) -> Option<&mut EndpointContext> {
        if dci == 0 || dci > 31 {
            return None;
        }
        Some(&mut self.endpoints[(dci - 1) as usize])
    }
}

impl Default for InputContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Input Control Context (32 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, align(32))]
pub struct InputControlContext {
    /// Drop Context flags (bit 0 is reserved, bits 1-31 for endpoints 1-31)
    pub drop_flags: u32,

    /// Add Context flags (bit 0 is slot, bits 1-31 for endpoints 1-31)
    pub add_flags: u32,

    /// Reserved
    _reserved: [u32; 5],

    /// Configuration Value
    pub config_value: u8,

    /// Interface Number
    pub interface_number: u8,

    /// Alternate Setting
    pub alternate_setting: u8,

    /// Reserved
    _reserved2: u8,
}

impl InputControlContext {
    /// Create new input control context
    pub fn new() -> Self {
        Self {
            drop_flags: 0,
            add_flags: 0,
            _reserved: [0; 5],
            config_value: 0,
            interface_number: 0,
            alternate_setting: 0,
            _reserved2: 0,
        }
    }

    /// Set add flag for slot context
    pub fn add_slot(&mut self) {
        self.add_flags |= 0x1;
    }

    /// Set add flag for endpoint context
    pub fn add_endpoint(&mut self, dci: u8) {
        if dci > 0 && dci <= 31 {
            self.add_flags |= 1 << dci;
        }
    }

    /// Set drop flag for endpoint context
    pub fn drop_endpoint(&mut self, dci: u8) {
        if dci > 0 && dci <= 31 {
            self.drop_flags |= 1 << dci;
        }
    }
}

impl Default for InputControlContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_context_size() {
        assert_eq!(core::mem::size_of::<SlotContext>(), 32);
        assert_eq!(core::mem::align_of::<SlotContext>(), 32);
    }

    #[test]
    fn test_endpoint_context_size() {
        assert_eq!(core::mem::size_of::<EndpointContext>(), 32);
        assert_eq!(core::mem::align_of::<EndpointContext>(), 32);
    }

    #[test]
    fn test_device_context_size() {
        // Slot context (32) + 31 endpoint contexts (31 * 32) = 1024 bytes
        assert_eq!(core::mem::size_of::<DeviceContext>(), 1024);
        assert_eq!(core::mem::align_of::<DeviceContext>(), 64);
    }
}
