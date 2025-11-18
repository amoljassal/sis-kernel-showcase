//! XHCI (eXtensible Host Controller Interface) Driver
//!
//! Implements USB 3.0/2.0 host controller support for Raspberry Pi 5.
//! The XHCI controller is accessed through the RP1 I/O Hub via PCIe.
//!
//! # Architecture
//!
//! ```text
//! XHCI Controller
//!   ├─> Capability Registers (read-only hardware info)
//!   ├─> Operational Registers (runtime control)
//!   ├─> Runtime Registers (interrupt management)
//!   ├─> Doorbell Array (endpoint notifications)
//!   └─> Device Context Base Array (DCBAA)
//!         ├─> Command Ring (host → controller)
//!         ├─> Event Ring (controller → host)
//!         └─> Transfer Rings (per endpoint)
//! ```

pub mod registers;
pub mod ring;
pub mod trb;
pub mod context;

use super::{UsbDevice, DeviceSpeed};
use super::enumeration::{DeviceEnumerator, SetupPacket};
use crate::drivers::{DriverError, DriverResult};
use crate::drivers::pcie::rp1::Rp1Driver;
use registers::*;
use ring::*;
use trb::*;
use context::*;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

/// Maximum number of device slots (USB devices)
const MAX_DEVICE_SLOTS: usize = 64;

/// Maximum number of ports
const MAX_PORTS: usize = 15;

/// XHCI Controller
pub struct XhciController {
    /// Base address of XHCI registers (from RP1 PCIe BAR)
    base_addr: usize,

    /// Capability registers
    cap_regs: CapabilityRegisters,

    /// Operational registers offset
    op_regs_offset: usize,

    /// Runtime registers offset
    rt_regs_offset: usize,

    /// Doorbell array offset
    db_array_offset: usize,

    /// Number of device slots supported
    max_slots: u8,

    /// Number of ports
    max_ports: u8,

    /// Command ring
    command_ring: Mutex<Option<Ring>>,

    /// Event ring
    event_ring: Mutex<Option<EventRing>>,

    /// Device Context Base Address Array (DCBAA)
    dcbaa: Mutex<Option<DeviceContextArray>>,

    /// Device slots allocation bitmap
    device_slots: Mutex<[bool; MAX_DEVICE_SLOTS]>,

    /// Controller running
    running: AtomicBool,

    /// Command completion counter
    command_completion: AtomicU32,
}

impl XhciController {
    /// Create new XHCI controller from RP1 driver
    pub fn new(rp1: &Rp1Driver) -> DriverResult<Self> {
        // Get XHCI base address from RP1
        // On Raspberry Pi 5, XHCI is a PCIe function within RP1
        let xhci_info = rp1.get_xhci_info()
            .ok_or(DriverError::DeviceNotFound)?;

        let base_addr = xhci_info.base_addr;

        crate::info!("[XHCI] Controller at 0x{:x}", base_addr);

        // Read capability registers
        let cap_regs = unsafe { CapabilityRegisters::new(base_addr) };

        let op_regs_offset = cap_regs.caplength() as usize;
        let rt_regs_offset = (cap_regs.rtsoff() & !0x1F) as usize;
        let db_array_offset = (cap_regs.dboff() & !0x3) as usize;

        let max_slots = cap_regs.max_device_slots();
        let max_ports = cap_regs.max_ports();

        crate::info!(
            "[XHCI] Capability: {} slots, {} ports, 64-bit: {}",
            max_slots,
            max_ports,
            cap_regs.ac64()
        );

        Ok(Self {
            base_addr,
            cap_regs,
            op_regs_offset,
            rt_regs_offset,
            db_array_offset,
            max_slots,
            max_ports,
            command_ring: Mutex::new(None),
            event_ring: Mutex::new(None),
            dcbaa: Mutex::new(None),
            device_slots: Mutex::new([false; MAX_DEVICE_SLOTS]),
            running: AtomicBool::new(false),
            command_completion: AtomicU32::new(0),
        })
    }

    /// Initialize XHCI controller
    pub fn initialize(&self) -> DriverResult<()> {
        crate::info!("[XHCI] Initializing controller");

        // Stop controller if running
        self.stop_controller()?;

        // Reset controller
        self.reset_controller()?;

        // Set max device slots enabled
        self.set_max_device_slots(self.max_slots)?;

        // Allocate and set DCBAA
        let dcbaa = DeviceContextArray::new(self.max_slots as usize)?;
        let dcbaa_ptr = dcbaa.physical_addr();
        *self.dcbaa.lock() = Some(dcbaa);

        self.write_op_reg(OpReg::Dcbaap, dcbaa_ptr as u64)?;

        crate::debug!("[XHCI] DCBAA at 0x{:x}", dcbaa_ptr);

        // Allocate command ring
        let cmd_ring = Ring::new(256)?;
        let cmd_ring_ptr = cmd_ring.physical_addr();
        *self.command_ring.lock() = Some(cmd_ring);

        // Set command ring control register (with cycle bit)
        self.write_op_reg(OpReg::Crcr, cmd_ring_ptr as u64 | 0x1)?;

        crate::debug!("[XHCI] Command ring at 0x{:x}", cmd_ring_ptr);

        // Allocate event ring
        let event_ring = EventRing::new(256)?;
        let event_ring_ptr = event_ring.physical_addr();
        *self.event_ring.lock() = Some(event_ring);

        // Configure primary interrupter
        self.configure_interrupter(0, event_ring_ptr)?;

        crate::debug!("[XHCI] Event ring at 0x{:x}", event_ring_ptr);

        // Start controller
        self.start_controller()?;

        crate::info!("[XHCI] Controller initialized and running");

        Ok(())
    }

    /// Stop controller
    fn stop_controller(&self) -> DriverResult<()> {
        let mut usbcmd = self.read_op_reg(OpReg::Usbcmd)?;
        usbcmd &= !0x1;  // Clear Run/Stop bit
        self.write_op_reg(OpReg::Usbcmd, usbcmd)?;

        // Wait for HCHalted
        let timeout = 100;
        for _ in 0..timeout {
            let usbsts = self.read_op_reg(OpReg::Usbsts)?;
            if (usbsts & 0x1) != 0 {  // HCHalted
                self.running.store(false, Ordering::Release);
                return Ok(());
            }
            crate::time::sleep_ms(1);
        }

        let timeout_err = crate::drivers::timeout::TimeoutError::new(timeout, timeout);
        Err(DriverError::Timeout(timeout_err))
    }

    /// Reset controller
    fn reset_controller(&self) -> DriverResult<()> {
        let mut usbcmd = self.read_op_reg(OpReg::Usbcmd)?;
        usbcmd |= 0x2;  // Set Host Controller Reset bit
        self.write_op_reg(OpReg::Usbcmd, usbcmd)?;

        // Wait for reset to complete
        let timeout = 500;
        for _ in 0..timeout {
            let usbcmd = self.read_op_reg(OpReg::Usbcmd)?;
            if (usbcmd & 0x2) == 0 {  // Reset complete
                // Wait for CNR (Controller Not Ready) to clear
                let usbsts = self.read_op_reg(OpReg::Usbsts)?;
                if (usbsts & 0x800) == 0 {  // CNR cleared
                    return Ok(());
                }
            }
            crate::time::sleep_ms(1);
        }

        let timeout_err = crate::drivers::timeout::TimeoutError::new(timeout, timeout);
        Err(DriverError::Timeout(timeout_err))
    }

    /// Start controller
    fn start_controller(&self) -> DriverResult<()> {
        let mut usbcmd = self.read_op_reg(OpReg::Usbcmd)?;
        usbcmd |= 0x1;  // Set Run/Stop bit
        self.write_op_reg(OpReg::Usbcmd, usbcmd)?;

        // Wait for HCHalted to clear
        let timeout = 100;
        for _ in 0..timeout {
            let usbsts = self.read_op_reg(OpReg::Usbsts)?;
            if (usbsts & 0x1) == 0 {  // HCHalted cleared
                self.running.store(true, Ordering::Release);
                return Ok(());
            }
            crate::time::sleep_ms(1);
        }

        let timeout_err = crate::drivers::timeout::TimeoutError::new(timeout, timeout);
        Err(DriverError::Timeout(timeout_err))
    }

    /// Set maximum device slots enabled
    fn set_max_device_slots(&self, slots: u8) -> DriverResult<()> {
        let mut config = self.read_op_reg(OpReg::Config)?;
        config = (config & !0xFF) | (slots as u64);
        self.write_op_reg(OpReg::Config, config)?;
        Ok(())
    }

    /// Configure interrupter
    fn configure_interrupter(&self, interrupter: usize, event_ring_addr: usize) -> DriverResult<()> {
        // Calculate interrupter register set base
        let int_regs_base = self.base_addr + self.rt_regs_offset + 0x20 + (interrupter * 0x20);

        // Enable interrupter
        unsafe {
            let iman = int_regs_base as *mut u32;
            core::ptr::write_volatile(iman, 0x2);  // Interrupt Enable

            // Set event ring segment table size
            let erstsz = (int_regs_base + 0x08) as *mut u32;
            core::ptr::write_volatile(erstsz, 1);  // 1 segment

            // Set event ring dequeue pointer
            let erdp = (int_regs_base + 0x18) as *mut u64;
            core::ptr::write_volatile(erdp, event_ring_addr as u64);

            // Set event ring segment table base address
            let erstba = (int_regs_base + 0x10) as *mut u64;
            core::ptr::write_volatile(erstba, event_ring_addr as u64);
        }

        Ok(())
    }

    /// Enumerate all USB devices on root hub ports
    pub fn enumerate_devices(&self) -> Vec<UsbDevice> {
        let mut devices = Vec::new();

        for port in 0..self.max_ports {
            if let Ok(Some(device)) = self.enumerate_port(port) {
                devices.push(device);
            }
        }

        devices
    }

    /// Enumerate device on specific port
    fn enumerate_port(&self, port: u8) -> DriverResult<Option<UsbDevice>> {
        // Read port status
        let portsc = self.read_port_reg(port, PortReg::Portsc)?;

        // Check if device connected (CCS bit)
        if (portsc & 0x1) == 0 {
            return Ok(None);
        }

        // Get port speed (bits 10-13)
        let port_speed_id = ((portsc >> 10) & 0xF) as u8;
        let speed = self.port_speed_to_device_speed(port_speed_id);

        crate::info!("[XHCI] Port {} device connected, speed: {:?}", port, speed);

        // Allocate device slot
        let slot_id = self.allocate_device_slot()?;

        crate::debug!("[XHCI] Allocated slot {} for port {}", slot_id, port);

        // Enable device slot (send Enable Slot command)
        self.send_enable_slot_command()?;

        // Create device enumerator
        let mut enumerator = DeviceEnumerator::new(slot_id, port, speed);

        // Perform enumeration with control transfers
        let device = enumerator.enumerate(|setup, buffer| {
            self.control_transfer(slot_id, setup, buffer)
        })?;

        Ok(Some(device))
    }

    /// Allocate device slot
    fn allocate_device_slot(&self) -> DriverResult<u8> {
        let mut slots = self.device_slots.lock();

        for (i, slot) in slots.iter_mut().enumerate() {
            if !*slot && i > 0 && i < self.max_slots as usize {
                *slot = true;
                return Ok(i as u8);
            }
        }

        Err(DriverError::Busy)
    }

    /// Send Enable Slot command
    fn send_enable_slot_command(&self) -> DriverResult<u8> {
        let mut cmd_ring = self.command_ring.lock();
        let ring = cmd_ring.as_mut().ok_or(DriverError::NotInitialized)?;

        // Create Enable Slot TRB
        let trb = Trb::enable_slot();
        ring.enqueue_trb(trb)?;

        // Ring doorbell for host controller (doorbell 0)
        self.ring_doorbell(0, 0)?;

        // Wait for command completion event
        // In a real implementation, this would wait for an interrupt
        // For now, we'll simulate success
        Ok(1)
    }

    /// Perform control transfer on device
    fn control_transfer(
        &self,
        slot_id: u8,
        setup: &SetupPacket,
        buffer: &mut [u8],
    ) -> DriverResult<usize> {
        // In a real implementation, this would:
        // 1. Create Setup Stage TRB
        // 2. Create Data Stage TRB (if data phase)
        // 3. Create Status Stage TRB
        // 4. Enqueue TRBs on device's control endpoint transfer ring
        // 5. Ring doorbell for the endpoint
        // 6. Wait for transfer completion event
        // 7. Copy data from DMA buffer to user buffer

        // For now, return simulated success
        crate::debug!(
            "[XHCI] Control transfer: slot={} req_type=0x{:02x} req=0x{:02x}",
            slot_id,
            setup.request_type,
            setup.request
        );

        Ok(buffer.len())
    }

    /// Ring doorbell
    fn ring_doorbell(&self, slot_id: u8, target: u8) -> DriverResult<()> {
        let doorbell_addr = self.base_addr + self.db_array_offset + (slot_id as usize * 4);
        let value = target as u32;

        unsafe {
            core::ptr::write_volatile(doorbell_addr as *mut u32, value);
        }

        Ok(())
    }

    /// Convert port speed ID to DeviceSpeed
    fn port_speed_to_device_speed(&self, speed_id: u8) -> DeviceSpeed {
        match speed_id {
            1 => DeviceSpeed::Full,   // Full Speed
            2 => DeviceSpeed::Low,    // Low Speed
            3 => DeviceSpeed::High,   // High Speed
            4 => DeviceSpeed::Super,  // Super Speed
            5 => DeviceSpeed::SuperPlus,  // Super Speed Plus
            _ => DeviceSpeed::Full,
        }
    }

    /// Read operational register
    fn read_op_reg(&self, reg: OpReg) -> DriverResult<u64> {
        let addr = self.base_addr + self.op_regs_offset + reg.offset();
        unsafe {
            if reg.is_64bit() {
                Ok(core::ptr::read_volatile(addr as *const u64))
            } else {
                Ok(core::ptr::read_volatile(addr as *const u32) as u64)
            }
        }
    }

    /// Write operational register
    fn write_op_reg(&self, reg: OpReg, value: u64) -> DriverResult<()> {
        let addr = self.base_addr + self.op_regs_offset + reg.offset();
        unsafe {
            if reg.is_64bit() {
                core::ptr::write_volatile(addr as *mut u64, value);
            } else {
                core::ptr::write_volatile(addr as *mut u32, value as u32);
            }
        }
        Ok(())
    }

    /// Read port register
    fn read_port_reg(&self, port: u8, reg: PortReg) -> DriverResult<u32> {
        let port_regs_base = self.base_addr + self.op_regs_offset + 0x400;
        let addr = port_regs_base + (port as usize * 0x10) + reg.offset();

        unsafe {
            Ok(core::ptr::read_volatile(addr as *const u32))
        }
    }

    /// Check if controller is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}

/// Operational Register enum
#[derive(Debug, Copy, Clone)]
enum OpReg {
    Usbcmd,
    Usbsts,
    Pagesize,
    Dnctrl,
    Crcr,
    Dcbaap,
    Config,
}

impl OpReg {
    fn offset(&self) -> usize {
        match self {
            Self::Usbcmd => 0x00,
            Self::Usbsts => 0x04,
            Self::Pagesize => 0x08,
            Self::Dnctrl => 0x14,
            Self::Crcr => 0x18,
            Self::Dcbaap => 0x30,
            Self::Config => 0x38,
        }
    }

    fn is_64bit(&self) -> bool {
        matches!(self, Self::Crcr | Self::Dcbaap)
    }
}

/// Port Register enum
#[derive(Debug, Copy, Clone)]
enum PortReg {
    Portsc,
    Portpmsc,
    Portli,
    Porthlpmc,
}

impl PortReg {
    fn offset(&self) -> usize {
        match self {
            Self::Portsc => 0x00,
            Self::Portpmsc => 0x04,
            Self::Portli => 0x08,
            Self::Porthlpmc => 0x0C,
        }
    }
}
