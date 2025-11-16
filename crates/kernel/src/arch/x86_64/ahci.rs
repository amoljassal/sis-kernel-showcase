//! AHCI (Advanced Host Controller Interface) Driver for SATA
//!
//! This module implements an AHCI driver for accessing SATA disks on real x86_64 hardware.
//! AHCI is the standard interface for SATA controllers and is widely supported across
//! various hardware platforms including Intel ICH9 (used in MacBook Pro Mid 2012).
//!
//! Key features:
//! - HBA (Host Bus Adapter) initialization
//! - Port detection and enumeration
//! - SATA device identification
//! - Basic read/write operations
//!
//! Reference: Intel AHCI Specification 1.3.1

use crate::arch::x86_64::pci::PciDevice;
use core::ptr;

/// AHCI HBA Memory Registers (ABAR - AHCI Base Address Register)
#[repr(C)]
pub struct AhciHba {
    // Generic Host Control
    pub cap: u32,      // 0x00: Host Capabilities
    pub ghc: u32,      // 0x04: Global Host Control
    pub is: u32,       // 0x08: Interrupt Status
    pub pi: u32,       // 0x0C: Ports Implemented
    pub vs: u32,       // 0x10: Version
    pub ccc_ctl: u32,  // 0x14: Command Completion Coalescing Control
    pub ccc_ports: u32,// 0x18: Command Completion Coalescing Ports
    pub em_loc: u32,   // 0x1C: Enclosure Management Location
    pub em_ctl: u32,   // 0x20: Enclosure Management Control
    pub cap2: u32,     // 0x24: Host Capabilities Extended
    pub bohc: u32,     // 0x28: BIOS/OS Handoff Control and Status

    _rsv: [u8; 0x74],  // 0x2C - 0x9F: Reserved

    _vendor: [u8; 0x60], // 0xA0 - 0xFF: Vendor Specific

    // Port Control Registers (0x100 - 0x10FF)
    // Each port is 0x80 bytes, supports up to 32 ports
}

/// AHCI Port Registers (offset 0x100 + port_num * 0x80)
#[repr(C)]
pub struct AhciPort {
    pub clb: u64,       // 0x00: Command List Base Address (1K-aligned)
    pub fb: u64,        // 0x08: FIS Base Address (256-byte aligned)
    pub is: u32,        // 0x10: Interrupt Status
    pub ie: u32,        // 0x14: Interrupt Enable
    pub cmd: u32,       // 0x18: Command and Status
    _rsv0: u32,         // 0x1C: Reserved
    pub tfd: u32,       // 0x20: Task File Data
    pub sig: u32,       // 0x24: Signature
    pub ssts: u32,      // 0x28: SATA Status (SCR0: SStatus)
    pub sctl: u32,      // 0x2C: SATA Control (SCR2: SControl)
    pub serr: u32,      // 0x30: SATA Error (SCR1: SError)
    pub sact: u32,      // 0x34: SATA Active (SCR3: SActive)
    pub ci: u32,        // 0x38: Command Issue
    pub sntf: u32,      // 0x3C: SATA Notification (SCR4: SNotification)
    pub fbs: u32,       // 0x40: FIS-based Switching Control
    _rsv1: [u32; 11],   // 0x44 - 0x6F: Reserved
    _vendor: [u32; 4],  // 0x70 - 0x7F: Vendor Specific
}

// AHCI HBA Capabilities (CAP) register bits
pub const CAP_S64A: u32 = 1 << 31;     // Supports 64-bit Addressing
pub const CAP_SNCQ: u32 = 1 << 30;     // Supports Native Command Queuing
pub const CAP_NCS_MASK: u32 = 0x1F << 8; // Number of Command Slots

// AHCI Global Host Control (GHC) register bits
pub const GHC_AE: u32 = 1 << 31;       // AHCI Enable
pub const GHC_IE: u32 = 1 << 1;        // Interrupt Enable
pub const GHC_HR: u32 = 1 << 0;        // HBA Reset

// AHCI Port Command and Status (PxCMD) register bits
pub const PCMD_ST: u32 = 1 << 0;       // Start
pub const PCMD_FRE: u32 = 1 << 4;      // FIS Receive Enable
pub const PCMD_FR: u32 = 1 << 14;      // FIS Receive Running
pub const PCMD_CR: u32 = 1 << 15;      // Command List Running

// SATA Status (PxSSTS) register bits
#[allow(dead_code)]
pub const SSTS_DET_MASK: u32 = 0xF;    // Device Detection
pub const SSTS_DET_PRESENT: u32 = 0x3; // Device present and communication established

/// AHCI Controller state
pub struct AhciController {
    abar: usize,           // Physical address of AHCI registers
    ports_implemented: u32, // Bitmask of implemented ports
    num_ports: u8,          // Number of ports
    num_cmd_slots: u8,      // Number of command slots per port
}

impl AhciController {
    /// Initialize AHCI controller from PCI device
    pub fn new(pci_dev: &PciDevice) -> Result<Self, &'static str> {
        // Read BAR5 (ABAR - AHCI Base Address Register)
        let abar = match pci_dev.bar5_address() {
            Some(addr) => addr as usize,
            None => return Err("AHCI BAR5 not configured"),
        };

        crate::arch::x86_64::serial::serial_write(b"[AHCI] Initializing controller at BAR5: 0x");
        print_hex_u64(abar as u64);
        crate::arch::x86_64::serial::serial_write(b"\n");

        // Map AHCI registers (identity mapping for now)
        let hba = unsafe { &*(abar as *const AhciHba) };

        // Read capabilities
        let cap = unsafe { ptr::read_volatile(&hba.cap) };
        let ports_implemented = unsafe { ptr::read_volatile(&hba.pi) };
        let version = unsafe { ptr::read_volatile(&hba.vs) };

        crate::arch::x86_64::serial::serial_write(b"[AHCI] Version: ");
        print_hex_u32(version);
        crate::arch::x86_64::serial::serial_write(b"\n");

        crate::arch::x86_64::serial::serial_write(b"[AHCI] Capabilities: 0x");
        print_hex_u32(cap);
        crate::arch::x86_64::serial::serial_write(b"\n");

        crate::arch::x86_64::serial::serial_write(b"[AHCI] Ports implemented: 0x");
        print_hex_u32(ports_implemented);
        crate::arch::x86_64::serial::serial_write(b"\n");

        // Extract number of command slots
        let num_cmd_slots = ((cap & CAP_NCS_MASK) >> 8) as u8 + 1;
        crate::arch::x86_64::serial::serial_write(b"[AHCI] Command slots per port: ");
        print_decimal_u8(num_cmd_slots);
        crate::arch::x86_64::serial::serial_write(b"\n");

        // Count implemented ports
        let num_ports = ports_implemented.count_ones() as u8;
        crate::arch::x86_64::serial::serial_write(b"[AHCI] Number of ports: ");
        print_decimal_u8(num_ports);
        crate::arch::x86_64::serial::serial_write(b"\n");

        Ok(Self {
            abar,
            ports_implemented,
            num_ports,
            num_cmd_slots,
        })
    }

    /// Enumerate SATA ports and detect attached devices
    pub fn enumerate_ports(&self) {
        crate::arch::x86_64::serial::serial_write(b"[AHCI] Enumerating ports...\n");

        for port_num in 0..32 {
            if (self.ports_implemented & (1 << port_num)) == 0 {
                continue; // Port not implemented
            }

            // Calculate port register address
            let port_addr = self.abar + 0x100 + (port_num * 0x80);
            let port = unsafe { &*(port_addr as *const AhciPort) };

            // Read SATA status
            let ssts = unsafe { ptr::read_volatile(&port.ssts) };
            let det = ssts & SSTS_DET_MASK;

            if det == SSTS_DET_PRESENT {
                crate::arch::x86_64::serial::serial_write(b"[AHCI] Port ");
                print_decimal_u8(port_num as u8);
                crate::arch::x86_64::serial::serial_write(b": Device detected\n");

                // Read device signature
                let sig = unsafe { ptr::read_volatile(&port.sig) };
                crate::arch::x86_64::serial::serial_write(b"[AHCI]   Signature: 0x");
                print_hex_u32(sig);
                crate::arch::x86_64::serial::serial_write(b"\n");

                match sig {
                    0x00000101 => crate::arch::x86_64::serial::serial_write(b"[AHCI]   Type: SATA drive\n"),
                    0xEB140101 => crate::arch::x86_64::serial::serial_write(b"[AHCI]   Type: ATAPI drive\n"),
                    0xC33C0101 => crate::arch::x86_64::serial::serial_write(b"[AHCI]   Type: SEMB\n"),
                    0x96690101 => crate::arch::x86_64::serial::serial_write(b"[AHCI]   Type: Port Multiplier\n"),
                    _ => {
                        crate::arch::x86_64::serial::serial_write(b"[AHCI]   Type: Unknown\n");
                    }
                }
            } else {
                crate::arch::x86_64::serial::serial_write(b"[AHCI] Port ");
                print_decimal_u8(port_num as u8);
                crate::arch::x86_64::serial::serial_write(b": No device (DET=");
                print_decimal_u8(det as u8);
                crate::arch::x86_64::serial::serial_write(b")\n");
            }
        }
    }

    /// Get AHCI BAR address
    pub fn abar(&self) -> usize {
        self.abar
    }
}

/// Helper function to print u64 as hexadecimal
fn print_hex_u64(n: u64) {
    let hex_chars = b"0123456789abcdef";
    let mut buf = [0u8; 16];

    for i in 0..16 {
        let shift = (15 - i) * 4;
        let nibble = ((n >> shift) & 0xF) as usize;
        buf[i] = hex_chars[nibble];
    }

    crate::arch::x86_64::serial::serial_write(&buf);
}

/// Helper function to print u32 as hexadecimal
fn print_hex_u32(n: u32) {
    let hex_chars = b"0123456789abcdef";
    let mut buf = [0u8; 8];

    for i in 0..8 {
        let shift = (7 - i) * 4;
        let nibble = ((n >> shift) & 0xF) as usize;
        buf[i] = hex_chars[nibble];
    }

    crate::arch::x86_64::serial::serial_write(&buf);
}

/// Helper function to print u8 as decimal
fn print_decimal_u8(mut n: u8) {
    if n == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 3]; // u8 max is 255 (3 digits)
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10);
        n /= 10;
        i += 1;
    }

    // Print in reverse order (we built the number backwards)
    while i > 0 {
        i -= 1;
        crate::arch::x86_64::serial::serial_write(&[buf[i]]);
    }
}
