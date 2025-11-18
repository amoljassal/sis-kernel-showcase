//! XHCI Register Definitions
//!
//! Defines the memory-mapped registers for the XHCI host controller.

/// XHCI Capability Registers
pub struct CapabilityRegisters {
    base: usize,
}

impl CapabilityRegisters {
    /// Create new capability registers accessor
    ///
    /// # Safety
    /// Caller must ensure base address is valid XHCI MMIO region
    pub unsafe fn new(base: usize) -> Self {
        Self { base }
    }

    /// Read CAPLENGTH register (offset to operational registers)
    pub fn caplength(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.base as *const u8) }
    }

    /// Read HCIVERSION register
    pub fn hciversion(&self) -> u16 {
        unsafe { core::ptr::read_volatile((self.base + 0x02) as *const u16) }
    }

    /// Read HCSPARAMS1 register (structural parameters 1)
    pub fn hcsparams1(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x04) as *const u32) }
    }

    /// Get maximum device slots from HCSPARAMS1
    pub fn max_device_slots(&self) -> u8 {
        (self.hcsparams1() & 0xFF) as u8
    }

    /// Get maximum interrupters from HCSPARAMS1
    pub fn max_interrupters(&self) -> u16 {
        ((self.hcsparams1() >> 8) & 0x7FF) as u16
    }

    /// Get maximum ports from HCSPARAMS1
    pub fn max_ports(&self) -> u8 {
        ((self.hcsparams1() >> 24) & 0xFF) as u8
    }

    /// Read HCSPARAMS2 register (structural parameters 2)
    pub fn hcsparams2(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x08) as *const u32) }
    }

    /// Read HCSPARAMS3 register (structural parameters 3)
    pub fn hcsparams3(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x0C) as *const u32) }
    }

    /// Read HCCPARAMS1 register (capability parameters 1)
    pub fn hccparams1(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x10) as *const u32) }
    }

    /// Check if 64-bit addressing is supported
    pub fn ac64(&self) -> bool {
        (self.hccparams1() & 0x1) != 0
    }

    /// Check if Context Size is 64 bytes (vs 32 bytes)
    pub fn csz(&self) -> bool {
        (self.hccparams1() & 0x4) != 0
    }

    /// Read DBOFF register (doorbell offset)
    pub fn dboff(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x14) as *const u32) }
    }

    /// Read RTSOFF register (runtime register space offset)
    pub fn rtsoff(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x18) as *const u32) }
    }

    /// Read HCCPARAMS2 register (capability parameters 2)
    pub fn hccparams2(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x1C) as *const u32) }
    }
}

/// Operational Register offsets
pub struct OpRegisters {
    base: usize,
}

impl OpRegisters {
    /// Create new operational registers accessor
    pub fn new(base: usize) -> Self {
        Self { base }
    }

    /// Read USBCMD register
    pub fn usbcmd(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x00) as *const u32) }
    }

    /// Write USBCMD register
    pub fn set_usbcmd(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x00) as *mut u32, value) }
    }

    /// Read USBSTS register
    pub fn usbsts(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x04) as *const u32) }
    }

    /// Write USBSTS register (to clear bits)
    pub fn set_usbsts(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x04) as *mut u32, value) }
    }

    /// Read PAGESIZE register
    pub fn pagesize(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x08) as *const u32) }
    }

    /// Read DNCTRL register
    pub fn dnctrl(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x14) as *const u32) }
    }

    /// Write DNCTRL register
    pub fn set_dnctrl(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x14) as *mut u32, value) }
    }

    /// Read CRCR register (Command Ring Control)
    pub fn crcr(&self) -> u64 {
        unsafe { core::ptr::read_volatile((self.base + 0x18) as *const u64) }
    }

    /// Write CRCR register
    pub fn set_crcr(&self, value: u64) {
        unsafe { core::ptr::write_volatile((self.base + 0x18) as *mut u64, value) }
    }

    /// Read DCBAAP register (Device Context Base Address Array Pointer)
    pub fn dcbaap(&self) -> u64 {
        unsafe { core::ptr::read_volatile((self.base + 0x30) as *const u64) }
    }

    /// Write DCBAAP register
    pub fn set_dcbaap(&self, value: u64) {
        unsafe { core::ptr::write_volatile((self.base + 0x30) as *mut u64, value) }
    }

    /// Read CONFIG register
    pub fn config(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x38) as *const u32) }
    }

    /// Write CONFIG register
    pub fn set_config(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x38) as *mut u32, value) }
    }
}

/// Port Register Set
pub struct PortRegisters {
    base: usize,
}

impl PortRegisters {
    /// Create new port registers accessor for specific port
    pub fn new(op_base: usize, port: u8) -> Self {
        let base = op_base + 0x400 + (port as usize * 0x10);
        Self { base }
    }

    /// Read PORTSC register (Port Status and Control)
    pub fn portsc(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x00) as *const u32) }
    }

    /// Write PORTSC register
    pub fn set_portsc(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x00) as *mut u32, value) }
    }

    /// Read PORTPMSC register (Port Power Management Status and Control)
    pub fn portpmsc(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x04) as *const u32) }
    }

    /// Write PORTPMSC register
    pub fn set_portpmsc(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x04) as *mut u32, value) }
    }

    /// Read PORTLI register (Port Link Info)
    pub fn portli(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x08) as *const u32) }
    }

    /// Read PORTHLPMC register (Port Hardware LPM Control)
    pub fn porthlpmc(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x0C) as *const u32) }
    }

    /// Write PORTHLPMC register
    pub fn set_porthlpmc(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x0C) as *mut u32, value) }
    }

    /// Check if device is connected (Current Connect Status)
    pub fn is_connected(&self) -> bool {
        (self.portsc() & 0x1) != 0
    }

    /// Get port speed
    pub fn port_speed(&self) -> u8 {
        ((self.portsc() >> 10) & 0xF) as u8
    }

    /// Check if port is enabled
    pub fn is_enabled(&self) -> bool {
        (self.portsc() & 0x2) != 0
    }

    /// Reset port
    pub fn reset(&self) {
        let mut portsc = self.portsc();
        portsc |= 0x10;  // Port Reset bit
        self.set_portsc(portsc);
    }
}

/// Runtime Register offsets
pub struct RuntimeRegisters {
    base: usize,
}

impl RuntimeRegisters {
    /// Create new runtime registers accessor
    pub fn new(base: usize) -> Self {
        Self { base }
    }

    /// Read MFINDEX register (Microframe Index)
    pub fn mfindex(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x00) as *const u32) }
    }
}

/// Interrupter Register Set
pub struct InterrupterRegisters {
    base: usize,
}

impl InterrupterRegisters {
    /// Create new interrupter registers accessor
    pub fn new(rt_base: usize, interrupter: usize) -> Self {
        let base = rt_base + 0x20 + (interrupter * 0x20);
        Self { base }
    }

    /// Read IMAN register (Interrupter Management)
    pub fn iman(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x00) as *const u32) }
    }

    /// Write IMAN register
    pub fn set_iman(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x00) as *mut u32, value) }
    }

    /// Read IMOD register (Interrupter Moderation)
    pub fn imod(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x04) as *const u32) }
    }

    /// Write IMOD register
    pub fn set_imod(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x04) as *mut u32, value) }
    }

    /// Read ERSTSZ register (Event Ring Segment Table Size)
    pub fn erstsz(&self) -> u32 {
        unsafe { core::ptr::read_volatile((self.base + 0x08) as *const u32) }
    }

    /// Write ERSTSZ register
    pub fn set_erstsz(&self, value: u32) {
        unsafe { core::ptr::write_volatile((self.base + 0x08) as *mut u32, value) }
    }

    /// Read ERSTBA register (Event Ring Segment Table Base Address)
    pub fn erstba(&self) -> u64 {
        unsafe { core::ptr::read_volatile((self.base + 0x10) as *const u64) }
    }

    /// Write ERSTBA register
    pub fn set_erstba(&self, value: u64) {
        unsafe { core::ptr::write_volatile((self.base + 0x10) as *mut u64, value) }
    }

    /// Read ERDP register (Event Ring Dequeue Pointer)
    pub fn erdp(&self) -> u64 {
        unsafe { core::ptr::read_volatile((self.base + 0x18) as *const u64) }
    }

    /// Write ERDP register
    pub fn set_erdp(&self, value: u64) {
        unsafe { core::ptr::write_volatile((self.base + 0x18) as *mut u64, value) }
    }
}
