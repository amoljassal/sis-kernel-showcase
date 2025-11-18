//! Transfer Request Blocks (TRBs)
//!
//! TRBs are the basic unit of communication between software and the XHCI controller.
//! They describe data transfers, commands, and events.

/// TRB Type codes
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum TrbType {
    // Transfer TRBs
    Normal = 1,
    SetupStage = 2,
    DataStage = 3,
    StatusStage = 4,
    Isoch = 5,
    Link = 6,
    EventData = 7,
    NoOp = 8,

    // Command TRBs
    EnableSlot = 9,
    DisableSlot = 10,
    AddressDevice = 11,
    ConfigureEndpoint = 12,
    EvaluateContext = 13,
    ResetEndpoint = 14,
    StopEndpoint = 15,
    SetTrDequeuePointer = 16,
    ResetDevice = 17,
    ForceEvent = 18,
    NegotiateBandwidth = 19,
    SetLatencyToleranceValue = 20,
    GetPortBandwidth = 21,
    ForceHeader = 22,
    NoOpCommand = 23,

    // Event TRBs
    TransferEvent = 32,
    CommandCompletion = 33,
    PortStatusChange = 34,
    BandwidthRequest = 35,
    Doorbell = 36,
    HostController = 37,
    DeviceNotification = 38,
    MfindexWrap = 39,
}

impl TrbType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::Normal),
            2 => Some(Self::SetupStage),
            3 => Some(Self::DataStage),
            4 => Some(Self::StatusStage),
            5 => Some(Self::Isoch),
            6 => Some(Self::Link),
            7 => Some(Self::EventData),
            8 => Some(Self::NoOp),
            9 => Some(Self::EnableSlot),
            10 => Some(Self::DisableSlot),
            11 => Some(Self::AddressDevice),
            12 => Some(Self::ConfigureEndpoint),
            13 => Some(Self::EvaluateContext),
            14 => Some(Self::ResetEndpoint),
            15 => Some(Self::StopEndpoint),
            16 => Some(Self::SetTrDequeuePointer),
            17 => Some(Self::ResetDevice),
            18 => Some(Self::ForceEvent),
            19 => Some(Self::NegotiateBandwidth),
            20 => Some(Self::SetLatencyToleranceValue),
            21 => Some(Self::GetPortBandwidth),
            22 => Some(Self::ForceHeader),
            23 => Some(Self::NoOpCommand),
            32 => Some(Self::TransferEvent),
            33 => Some(Self::CommandCompletion),
            34 => Some(Self::PortStatusChange),
            35 => Some(Self::BandwidthRequest),
            36 => Some(Self::Doorbell),
            37 => Some(Self::HostController),
            38 => Some(Self::DeviceNotification),
            39 => Some(Self::MfindexWrap),
            _ => None,
        }
    }
}

/// Generic TRB structure (16 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct Trb {
    /// Parameter or data buffer pointer (depending on TRB type)
    pub parameter: u64,

    /// Status field (varies by TRB type)
    pub status: u32,

    /// Control field (contains TRB type, cycle bit, flags)
    pub control: u32,
}

impl Trb {
    /// Create new TRB with all fields zeroed
    pub fn new() -> Self {
        Self {
            parameter: 0,
            status: 0,
            control: 0,
        }
    }

    /// Get TRB type
    pub fn trb_type(&self) -> Option<TrbType> {
        let type_val = ((self.control >> 10) & 0x3F) as u8;
        TrbType::from_u8(type_val)
    }

    /// Set TRB type
    pub fn set_trb_type(&mut self, trb_type: TrbType) {
        self.control = (self.control & !(0x3F << 10)) | ((trb_type as u32) << 10);
    }

    /// Get cycle bit
    pub fn cycle(&self) -> bool {
        (self.control & 0x1) != 0
    }

    /// Set cycle bit
    pub fn set_cycle(&mut self, cycle: bool) {
        if cycle {
            self.control |= 0x1;
        } else {
            self.control &= !0x1;
        }
    }

    /// Create Enable Slot command TRB
    pub fn enable_slot() -> Self {
        let mut trb = Self::new();
        trb.set_trb_type(TrbType::EnableSlot);
        trb.set_cycle(true);
        trb
    }

    /// Create Disable Slot command TRB
    pub fn disable_slot(slot_id: u8) -> Self {
        let mut trb = Self::new();
        trb.set_trb_type(TrbType::DisableSlot);
        trb.control |= (slot_id as u32) << 24;
        trb.set_cycle(true);
        trb
    }

    /// Create Address Device command TRB
    pub fn address_device(slot_id: u8, input_context_addr: u64, bsr: bool) -> Self {
        let mut trb = Self::new();
        trb.parameter = input_context_addr;
        trb.set_trb_type(TrbType::AddressDevice);
        trb.control |= (slot_id as u32) << 24;
        if bsr {
            trb.control |= 1 << 9;  // Block Set Address Request
        }
        trb.set_cycle(true);
        trb
    }

    /// Create Configure Endpoint command TRB
    pub fn configure_endpoint(slot_id: u8, input_context_addr: u64) -> Self {
        let mut trb = Self::new();
        trb.parameter = input_context_addr;
        trb.set_trb_type(TrbType::ConfigureEndpoint);
        trb.control |= (slot_id as u32) << 24;
        trb.set_cycle(true);
        trb
    }

    /// Create Setup Stage TRB
    pub fn setup_stage(setup_data: u64, trt: u8, idt: bool) -> Self {
        let mut trb = Self::new();
        trb.parameter = setup_data;
        trb.status = 8;  // Transfer length: 8 bytes
        trb.set_trb_type(TrbType::SetupStage);
        trb.control |= ((trt as u32) & 0x3) << 16;  // Transfer Type
        if idt {
            trb.control |= 1 << 6;  // Immediate Data
        }
        trb.set_cycle(true);
        trb
    }

    /// Create Data Stage TRB
    pub fn data_stage(buffer: u64, length: u32, dir_in: bool) -> Self {
        let mut trb = Self::new();
        trb.parameter = buffer;
        trb.status = length & 0x1FFFF;  // Transfer length (17 bits)
        trb.set_trb_type(TrbType::DataStage);
        if dir_in {
            trb.control |= 1 << 16;  // Direction: IN
        }
        trb.set_cycle(true);
        trb
    }

    /// Create Status Stage TRB
    pub fn status_stage(dir_in: bool) -> Self {
        let mut trb = Self::new();
        trb.set_trb_type(TrbType::StatusStage);
        if dir_in {
            trb.control |= 1 << 16;  // Direction: IN
        }
        trb.control |= 1 << 5;  // Interrupt On Completion
        trb.set_cycle(true);
        trb
    }

    /// Create Normal TRB
    pub fn normal(buffer: u64, length: u32, chain: bool, ioc: bool) -> Self {
        let mut trb = Self::new();
        trb.parameter = buffer;
        trb.status = length & 0x1FFFF;
        trb.set_trb_type(TrbType::Normal);
        if chain {
            trb.control |= 1 << 4;  // Chain bit
        }
        if ioc {
            trb.control |= 1 << 5;  // Interrupt On Completion
        }
        trb.set_cycle(true);
        trb
    }

    /// Create Link TRB
    pub fn link(ring_segment_ptr: u64, toggle_cycle: bool) -> Self {
        let mut trb = Self::new();
        trb.parameter = ring_segment_ptr;
        trb.set_trb_type(TrbType::Link);
        if toggle_cycle {
            trb.control |= 1 << 1;  // Toggle Cycle
        }
        trb.set_cycle(true);
        trb
    }

    /// Create No-Op TRB
    pub fn noop() -> Self {
        let mut trb = Self::new();
        trb.set_trb_type(TrbType::NoOp);
        trb.set_cycle(true);
        trb
    }

    /// Create No-Op Command TRB
    pub fn noop_command() -> Self {
        let mut trb = Self::new();
        trb.set_trb_type(TrbType::NoOpCommand);
        trb.set_cycle(true);
        trb
    }
}

impl Default for Trb {
    fn default() -> Self {
        Self::new()
    }
}

/// Command Completion Event TRB parser
pub struct CommandCompletionEvent {
    trb: Trb,
}

impl CommandCompletionEvent {
    /// Create from TRB
    pub fn from_trb(trb: Trb) -> Option<Self> {
        if trb.trb_type() == Some(TrbType::CommandCompletion) {
            Some(Self { trb })
        } else {
            None
        }
    }

    /// Get completion code
    pub fn completion_code(&self) -> u8 {
        ((self.trb.status >> 24) & 0xFF) as u8
    }

    /// Get slot ID
    pub fn slot_id(&self) -> u8 {
        ((self.trb.control >> 24) & 0xFF) as u8
    }

    /// Get command TRB pointer
    pub fn command_trb_pointer(&self) -> u64 {
        self.trb.parameter
    }

    /// Check if command succeeded
    pub fn is_success(&self) -> bool {
        self.completion_code() == 1  // Success code
    }
}

/// Transfer Event TRB parser
pub struct TransferEvent {
    trb: Trb,
}

impl TransferEvent {
    /// Create from TRB
    pub fn from_trb(trb: Trb) -> Option<Self> {
        if trb.trb_type() == Some(TrbType::TransferEvent) {
            Some(Self { trb })
        } else {
            None
        }
    }

    /// Get completion code
    pub fn completion_code(&self) -> u8 {
        ((self.trb.status >> 24) & 0xFF) as u8
    }

    /// Get slot ID
    pub fn slot_id(&self) -> u8 {
        ((self.trb.control >> 24) & 0xFF) as u8
    }

    /// Get endpoint ID
    pub fn endpoint_id(&self) -> u8 {
        ((self.trb.control >> 16) & 0x1F) as u8
    }

    /// Get transfer length
    pub fn transfer_length(&self) -> u32 {
        self.trb.status & 0xFFFFFF
    }

    /// Get TRB pointer
    pub fn trb_pointer(&self) -> u64 {
        self.trb.parameter
    }

    /// Check if transfer succeeded
    pub fn is_success(&self) -> bool {
        let code = self.completion_code();
        code == 1 || code == 13  // Success or Short Packet
    }
}

/// Port Status Change Event TRB parser
pub struct PortStatusChangeEvent {
    trb: Trb,
}

impl PortStatusChangeEvent {
    /// Create from TRB
    pub fn from_trb(trb: Trb) -> Option<Self> {
        if trb.trb_type() == Some(TrbType::PortStatusChange) {
            Some(Self { trb })
        } else {
            None
        }
    }

    /// Get port ID
    pub fn port_id(&self) -> u8 {
        ((self.trb.parameter >> 24) & 0xFF) as u8
    }

    /// Get completion code
    pub fn completion_code(&self) -> u8 {
        ((self.trb.status >> 24) & 0xFF) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trb_size() {
        assert_eq!(core::mem::size_of::<Trb>(), 16);
        assert_eq!(core::mem::align_of::<Trb>(), 16);
    }

    #[test]
    fn test_trb_type() {
        let mut trb = Trb::new();
        trb.set_trb_type(TrbType::EnableSlot);
        assert_eq!(trb.trb_type(), Some(TrbType::EnableSlot));
    }

    #[test]
    fn test_cycle_bit() {
        let mut trb = Trb::new();
        assert!(!trb.cycle());
        trb.set_cycle(true);
        assert!(trb.cycle());
        trb.set_cycle(false);
        assert!(!trb.cycle());
    }
}
