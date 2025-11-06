//! VirtIO transport layer for SIS kernel
//!
//! Implements VirtIO 1.0+ specification for paravirtualized I/O devices
//! Supports MMIO transport with device discovery and management

pub mod virtqueue;

use crate::driver::{DeviceId, DeviceInfo, DriverError, DriverResult};
use core::ptr;

/// VirtIO MMIO register offsets (VirtIO 1.0+ spec)
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum VirtIOMMIOOffset {
    MagicValue = 0x000,        // 0x74726976 "virt"
    Version = 0x004,           // Version (should be 2+ for VirtIO 1.0)
    DeviceID = 0x008,          // Device type identifier
    VendorID = 0x00c,          // Vendor identifier (0x554D4551 for QEMU)
    DeviceFeatures = 0x010,    // Device feature bits 31:0
    DeviceFeaturesSel = 0x014, // Device feature selection
    DriverFeatures = 0x020,    // Driver feature bits 31:0
    DriverFeaturesSel = 0x024, // Driver feature selection
    GuestPageSize = 0x028,     // Legacy (VirtIO 0.9.5)
    QueueSel = 0x030,          // Queue selection
    QueueNumMax = 0x034,       // Maximum queue size
    QueueNum = 0x038,          // Actual queue size
    QueueAlign = 0x03c,        // Legacy (VirtIO 0.9.5)
    QueuePFN = 0x040,          // Legacy queue address (page number)
    QueueReady = 0x044,        // Queue ready status (VirtIO 1.0+)
    QueueNotify = 0x050,       // Queue notification
    InterruptStatus = 0x060,   // Interrupt status
    InterruptACK = 0x064,      // Interrupt acknowledge
    Status = 0x070,            // Device status
    QueueDescLow = 0x080,      // Queue descriptor area (low 32 bits)
    QueueDescHigh = 0x084,     // Queue descriptor area (high 32 bits)
    QueueAvailLow = 0x090,     // Queue available ring (low 32 bits)
    QueueAvailHigh = 0x094,    // Queue available ring (high 32 bits)
    QueueUsedLow = 0x0a0,      // Queue used ring (low 32 bits)
    QueueUsedHigh = 0x0a4,     // Queue used ring (high 32 bits)
    ConfigGeneration = 0x0fc,  // Configuration generation
    Config = 0x100,            // Device-specific configuration
}

/// VirtIO device types
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtIODeviceType {
    Reserved = 0,
    NetworkCard = 1,
    BlockDevice = 2,
    Console = 3,
    EntropySource = 4,
    MemoryBallon = 5,
    IOMemory = 6,
    RPMSG = 7,
    SCSIHost = 8,
    Transport9P = 9,
    Mac80211WLAN = 10,
    RprocSerial = 11,
    VirtIOCAIF = 12,
    MemoryBalloon = 13,
    GPU = 16,
    Timer = 17,
    Input = 18,
    Socket = 19,
    Crypto = 20,
    SignalDistModule = 21,
    PSTORE = 22,
    IOMMU = 23,
    Memory = 24,
    Unknown = u32::MAX,
}

impl From<u32> for VirtIODeviceType {
    fn from(val: u32) -> Self {
        match val {
            0 => VirtIODeviceType::Reserved,
            1 => VirtIODeviceType::NetworkCard,
            2 => VirtIODeviceType::BlockDevice,
            3 => VirtIODeviceType::Console,
            4 => VirtIODeviceType::EntropySource,
            5 => VirtIODeviceType::MemoryBallon,
            6 => VirtIODeviceType::IOMemory,
            7 => VirtIODeviceType::RPMSG,
            8 => VirtIODeviceType::SCSIHost,
            9 => VirtIODeviceType::Transport9P,
            10 => VirtIODeviceType::Mac80211WLAN,
            11 => VirtIODeviceType::RprocSerial,
            12 => VirtIODeviceType::VirtIOCAIF,
            13 => VirtIODeviceType::MemoryBalloon,
            16 => VirtIODeviceType::GPU,
            17 => VirtIODeviceType::Timer,
            18 => VirtIODeviceType::Input,
            19 => VirtIODeviceType::Socket,
            20 => VirtIODeviceType::Crypto,
            21 => VirtIODeviceType::SignalDistModule,
            22 => VirtIODeviceType::PSTORE,
            23 => VirtIODeviceType::IOMMU,
            24 => VirtIODeviceType::Memory,
            _ => VirtIODeviceType::Unknown,
        }
    }
}

/// VirtIO device status flags
#[repr(u32)]
pub enum VirtIOStatus {
    Reset = 0,
    Acknowledge = 1,
    Driver = 2,
    DriverOK = 4,
    FeaturesOK = 8,
    DeviceNeedsReset = 64,
    Failed = 128,
}

/// VirtIO MMIO transport
#[derive(Debug)]
pub struct VirtIOMMIOTransport {
    base_addr: u64,
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    irq: Option<u32>,
    device_type: VirtIODeviceType,
    version: u32,
    #[allow(dead_code)]
    vendor_id: u32,
}

impl VirtIOMMIOTransport {
    /// Create new VirtIO MMIO transport from device info
    pub fn new(base_addr: u64, size: u64, irq: Option<u32>) -> Result<Self, DriverError> {
        unsafe {
            // Verify VirtIO magic value
            let magic = ptr::read_volatile((base_addr + VirtIOMMIOOffset::MagicValue as u64) as *const u32);
            if magic != 0x74726976 {
                // Debug: print the actual magic for troubleshooting
                if magic != 0x00000000 && magic != 0xFFFFFFFF {
                    crate::uart_print(b"[VIRTIO-DEBUG] Wrong magic 0x");
                    crate::virtio::VirtIODiscovery::print_hex(magic as u64);
                    crate::uart_print(b" at 0x");
                    crate::virtio::VirtIODiscovery::print_hex(base_addr);
                    crate::uart_print(b"\n");
                }
                return Err(DriverError::InvalidDevice);
            }

            // Check version (accept 1+ for VirtIO 0.9.5+ compatibility)
            let version = ptr::read_volatile((base_addr + VirtIOMMIOOffset::Version as u64) as *const u32);
            if version < 1 {
                // Debug: print the actual version for troubleshooting
                crate::uart_print(b"[VIRTIO-DEBUG] Found version ");
                crate::virtio::VirtIODiscovery::print_number(version);
                crate::uart_print(b" at 0x");
                crate::virtio::VirtIODiscovery::print_hex(base_addr);
                crate::uart_print(b"\n");
                return Err(DriverError::NotSupported);
            }

            // Read device type and vendor
            let device_type = VirtIODeviceType::from(ptr::read_volatile((base_addr + VirtIOMMIOOffset::DeviceID as u64) as *const u32));
            let vendor_id = ptr::read_volatile((base_addr + VirtIOMMIOOffset::VendorID as u64) as *const u32);

            Ok(VirtIOMMIOTransport { base_addr, size, irq, device_type, version, vendor_id })
        }
    }

    /// Read from VirtIO MMIO register
    pub fn read_reg(&self, offset: VirtIOMMIOOffset) -> u32 {
        unsafe { ptr::read_volatile((self.base_addr + offset as u64) as *const u32) }
    }

    /// Write to VirtIO MMIO register
    pub fn write_reg(&self, offset: VirtIOMMIOOffset, value: u32) {
        unsafe {
            ptr::write_volatile((self.base_addr + offset as u64) as *mut u32, value);
        }
    }

    /// Get device type
    pub fn device_type(&self) -> VirtIODeviceType {
        self.device_type
    }

    /// Reset device
    pub fn reset_device(&self) -> DriverResult<()> {
        self.write_reg(VirtIOMMIOOffset::Status, VirtIOStatus::Reset as u32);

        // Wait for reset to complete
        for _ in 0..1000 {
            if self.read_reg(VirtIOMMIOOffset::Status) == 0 {
                break;
            }
        }

        if self.read_reg(VirtIOMMIOOffset::Status) != 0 {
            return Err(DriverError::InitFailed);
        }

        Ok(())
    }

    /// Initialize device (status negotiation)
    pub fn init_device(&self, features: u32) -> DriverResult<()> {
        // 1. Reset device
        self.reset_device()?;

        // 2. Acknowledge device
        self.write_reg(VirtIOMMIOOffset::Status, VirtIOStatus::Acknowledge as u32);

        // 3. Set DRIVER status flag
        let mut status = self.read_reg(VirtIOMMIOOffset::Status);
        status |= VirtIOStatus::Driver as u32;
        self.write_reg(VirtIOMMIOOffset::Status, status);

        // 4. Negotiate features
        self.write_reg(VirtIOMMIOOffset::DriverFeaturesSel, 0);
        self.write_reg(VirtIOMMIOOffset::DriverFeatures, features);

        // 5. Set FEATURES_OK
        status |= VirtIOStatus::FeaturesOK as u32;
        self.write_reg(VirtIOMMIOOffset::Status, status);

        // 6. Verify features are accepted
        if (self.read_reg(VirtIOMMIOOffset::Status) & VirtIOStatus::FeaturesOK as u32) == 0 {
            return Err(DriverError::NotSupported);
        }

        // 7. Device-specific setup happens in device drivers

        // 8. Set DRIVER_OK (will be done by specific driver)

        Ok(())
    }

    /// Mark driver as ready
    pub fn driver_ready(&self) {
        let mut status = self.read_reg(VirtIOMMIOOffset::Status);
        status |= VirtIOStatus::DriverOK as u32;
        self.write_reg(VirtIOMMIOOffset::Status, status);
    }

    /// Check if device failed
    pub fn is_failed(&self) -> bool {
        (self.read_reg(VirtIOMMIOOffset::Status) & VirtIOStatus::Failed as u32) != 0
    }
}

/// VirtIO device discovery for QEMU ARM64 virt machine
pub struct VirtIODiscovery;

impl VirtIODiscovery {
    const MAX_VIRTIO_DEVICES: usize = 32;

    /// Discover VirtIO devices
    pub fn discover_devices() -> DriverResult<heapless::Vec<DeviceInfo, 32>> {
        let mut devices = heapless::Vec::new();

        unsafe {
            crate::uart_print(b"[VIRTIO] Starting VirtIO device discovery...\n");
        }

        let hint = crate::platform::active().virtio_mmio_hint();
        let (virtio_base, slot_size, irq_base) = match hint {
            Some((b, s, irq)) => (b as u64, s as u64, irq),
            None => {
                unsafe { crate::uart_print(b"[VIRTIO] No virtio-mmio hint from platform; skipping discovery\n"); }
                return Ok(devices);
            }
        };

        for i in 0..Self::MAX_VIRTIO_DEVICES {
            let base_addr = virtio_base + (i as u64 * slot_size);

            unsafe {
                crate::uart_print(b"[VIRTIO] Probing device slot ");
                Self::print_number(i as u32);
                crate::uart_print(b" at 0x");
                Self::print_hex(base_addr);
                crate::uart_print(b"\n");
            }

            // Try to create VirtIO transport for this address
            match VirtIOMMIOTransport::new(base_addr, slot_size, Some(irq_base + i as u32)) {
                Ok(transport) => {
                    let device_type = transport.device_type();

                    unsafe {
                        crate::uart_print(b"[VIRTIO] Found device type ");
                        Self::print_number(device_type as u32);
                        crate::uart_print(b" (");
                        Self::print_device_type_name(device_type);
                        crate::uart_print(b") at slot ");
                        Self::print_number(i as u32);
                        crate::uart_print(b"\n");
                    }

                    // Skip reserved/unknown devices
                    if device_type == VirtIODeviceType::Reserved
                        || device_type == VirtIODeviceType::Unknown
                    {
                        unsafe {
                            crate::uart_print(b"[VIRTIO] Skipping reserved/unknown device\n");
                        }
                        continue;
                    }

                    unsafe {
                        crate::uart_print(b"[VIRTIO] Found device at 0x");
                        Self::print_hex(base_addr);
                        crate::uart_print(b" type=");
                        Self::print_number(device_type as u32);
                        crate::uart_print(b" (");
                        Self::print_device_type_name(device_type);
                        crate::uart_print(b")\n");
                    }

                    // Convert to generic DeviceInfo
                    let device_info = DeviceInfo {
                        id: DeviceId {
                            vendor_id: 0x1AF4, // Red Hat (VirtIO)
                            device_id: device_type as u16,
                            class: Self::get_device_class(device_type),
                            subclass: Self::get_device_subclass(device_type),
                        },
                        base_addr,
                        size: slot_size,
                        irq: Some(irq_base + i as u32),
                        device_data: transport.version as u64,
                    };

                    if devices.push(device_info).is_err() {
                        break; // Vec is full
                    }
                }
                Err(e) => {
                    unsafe {
                        crate::uart_print(b"[VIRTIO] Probe failed: ");
                        match e {
                            DriverError::InvalidDevice => {
                                crate::uart_print(b"invalid magic/version\n")
                            }
                            DriverError::NotSupported => {
                                crate::uart_print(b"unsupported version\n")
                            }
                            _ => crate::uart_print(b"unknown error\n"),
                        }
                    }
                    // Continue to next slot
                }
            }
        }

        unsafe {
            crate::uart_print(b"[VIRTIO] Discovery complete, found ");
            Self::print_number(devices.len() as u32);
            crate::uart_print(b" devices\n");
        }

        Ok(devices)
    }

    /// Get device class for VirtIO device type
    fn get_device_class(device_type: VirtIODeviceType) -> u8 {
        match device_type {
            VirtIODeviceType::NetworkCard => 0x02,   // Network controller
            VirtIODeviceType::BlockDevice => 0x01,   // Storage controller
            VirtIODeviceType::Console => 0x07,       // Communication controller
            VirtIODeviceType::EntropySource => 0x10, // Encryption controller
            VirtIODeviceType::GPU => 0x03,           // Display controller
            VirtIODeviceType::Input => 0x09,         // Input controller
            _ => 0xFF,                               // Other
        }
    }

    /// Get device subclass for VirtIO device type
    fn get_device_subclass(device_type: VirtIODeviceType) -> u8 {
        match device_type {
            VirtIODeviceType::NetworkCard => 0x00,   // Ethernet
            VirtIODeviceType::BlockDevice => 0x08,   // Other storage
            VirtIODeviceType::Console => 0x80,       // Other communication
            VirtIODeviceType::EntropySource => 0x00, // Other encryption
            VirtIODeviceType::GPU => 0x80,           // Other display
            VirtIODeviceType::Input => 0x80,         // Other input
            _ => 0x80,                               // Other
        }
    }

    /// Print device type name
    unsafe fn print_device_type_name(device_type: VirtIODeviceType) {
        match device_type {
            VirtIODeviceType::NetworkCard => crate::uart_print(b"Network"),
            VirtIODeviceType::BlockDevice => crate::uart_print(b"Block"),
            VirtIODeviceType::Console => crate::uart_print(b"Console"),
            VirtIODeviceType::EntropySource => crate::uart_print(b"RNG"),
            VirtIODeviceType::MemoryBallon => crate::uart_print(b"Balloon"),
            VirtIODeviceType::GPU => crate::uart_print(b"GPU"),
            VirtIODeviceType::Input => crate::uart_print(b"Input"),
            VirtIODeviceType::Socket => crate::uart_print(b"Socket"),
            _ => crate::uart_print(b"Unknown"),
        }
    }

    /// Helper to print hex numbers
    unsafe fn print_hex(num: u64) {
        for i in (0..16).rev() {
            let nibble = (num >> (i * 4)) & 0xF;
            let c = if nibble < 10 {
                b'0' + nibble as u8
            } else {
                b'A' + (nibble - 10) as u8
            };
            crate::uart_print(&[c]);
        }
    }

    /// Helper to print numbers
    unsafe fn print_number(mut num: u32) {
        if num == 0 {
            crate::uart_print(b"0");
            return;
        }

        let mut digits = [0u8; 10];
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
