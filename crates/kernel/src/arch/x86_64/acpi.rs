//! # ACPI (Advanced Configuration and Power Interface) Support
//!
//! This module provides ACPI table parsing for x86_64 systems. ACPI tables provide
//! critical information about the hardware platform including:
//! - Processor topology (MADT)
//! - High Precision Event Timer location (HPET)
//! - PCI Express configuration space (MCFG)
//! - Power management configuration (FADT)
//!
//! ## ACPI Table Discovery
//!
//! ACPI tables are discovered via the Root System Description Pointer (RSDP):
//!
//! ```text
//! RSDP (Root System Description Pointer)
//!     ↓
//! RSDT/XSDT (Root/Extended System Description Table)
//!     ↓
//! ┌───────────┬─────────┬──────────┬──────────┬─────────┐
//! │   MADT    │  HPET   │   MCFG   │   FADT   │  Others │
//! └───────────┴─────────┴──────────┴──────────┴─────────┘
//! ```
//!
//! ## MADT (Multiple APIC Description Table)
//!
//! The MADT describes interrupt controllers and processors:
//! - Local APIC address
//! - I/O APIC configurations
//! - Processor local APICs (one per CPU core)
//! - Interrupt source overrides
//! - NMI sources
//!
//! ## HPET (High Precision Event Timer)
//!
//! The HPET table provides the physical address of the HPET MMIO region.
//!
//! ## MCFG (Memory Mapped Configuration)
//!
//! The MCFG table describes PCI Express Enhanced Configuration Access Mechanism (ECAM).
//!
//! ## FADT (Fixed ACPI Description Table)
//!
//! The FADT contains power management information including:
//! - PM1a/PM1b control registers
//! - Reset register
//! - Sleep state information

use x86_64::PhysAddr;
use core::ptr::read_volatile;
use crate::arch::x86_64::serial;

/// ACPI Root System Description Pointer (RSDP)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Rsdp {
    signature: [u8; 8],      // "RSD PTR "
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,       // Physical address of RSDT
}

/// ACPI Extended System Description Pointer (for ACPI 2.0+)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct RsdpExtended {
    rsdp: Rsdp,
    length: u32,
    xsdt_address: u64,       // Physical address of XSDT
    extended_checksum: u8,
    reserved: [u8; 3],
}

/// ACPI System Description Table Header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct SdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

/// ACPI initialization state
struct AcpiInfo {
    rsdp_addr: PhysAddr,
    rsdt_addr: Option<PhysAddr>,
    xsdt_addr: Option<PhysAddr>,
    madt_addr: Option<PhysAddr>,
    hpet_addr: Option<PhysAddr>,
    mcfg_addr: Option<PhysAddr>,
    fadt_addr: Option<PhysAddr>,
}

static mut ACPI_INFO: Option<AcpiInfo> = None;

/// Initialize ACPI by discovering and parsing tables
///
/// # Arguments
/// * `rsdp_addr` - Physical address of the RSDP (from UEFI)
///
/// # Safety
/// Must be called during early boot with valid RSDP address from UEFI
pub unsafe fn init(rsdp_addr: PhysAddr) -> Result<(), &'static str> {
    serial::serial_write(b"[ACPI] Initializing ACPI subsystem\n");
    serial::serial_write(b"[ACPI] RSDP at: 0x");
    print_hex_u64(rsdp_addr.as_u64());
    serial::serial_write(b"\n");

    // Map RSDP (20 bytes for ACPI 1.0, 36 bytes for ACPI 2.0+)
    // For x86_64 QEMU, the RSDP is in low memory that should be identity mapped
    // Try identity mapping first (physical address = virtual address)
    serial::serial_write(b"[ACPI] Attempting to access RSDP with identity mapping\n");
    let rsdp_virt = rsdp_addr.as_u64() as *const Rsdp;

    serial::serial_write(b"[ACPI] Reading RSDP from virtual address: 0x");
    print_hex_u64(rsdp_virt as u64);
    serial::serial_write(b"\n");

    // Read and validate RSDP signature
    serial::serial_write(b"[ACPI] Reading RSDP signature...\n");
    let rsdp = read_volatile(rsdp_virt);
    serial::serial_write(b"[ACPI] RSDP signature read complete\n");

    if &rsdp.signature != b"RSD PTR " {
        serial::serial_write(b"[ACPI] ERROR: Invalid RSDP signature: ");
        for i in 0..8 {
            serial::serial_write(&[rsdp.signature[i]]);
        }
        serial::serial_write(b"\n");
        return Err("Invalid RSDP signature");
    }
    serial::serial_write(b"[ACPI] RSDP signature valid\n");

    serial::serial_write(b"[ACPI] RSDP revision: ");
    print_u64(rsdp.revision as u64);
    serial::serial_write(b"\n");

    let mut acpi_info = AcpiInfo {
        rsdp_addr,
        rsdt_addr: None,
        xsdt_addr: None,
        madt_addr: None,
        hpet_addr: None,
        mcfg_addr: None,
        fadt_addr: None,
    };

    // ACPI 2.0+ uses XSDT, ACPI 1.0 uses RSDT
    if rsdp.revision >= 2 {
        // Read extended RSDP for XSDT address
        let rsdp_ext_virt = rsdp_virt as *const RsdpExtended;
        let rsdp_ext = read_volatile(rsdp_ext_virt);

        if rsdp_ext.xsdt_address != 0 {
            acpi_info.xsdt_addr = Some(PhysAddr::new(rsdp_ext.xsdt_address));
            serial::serial_write(b"[ACPI] XSDT at: 0x");
            print_hex_u64(rsdp_ext.xsdt_address);
            serial::serial_write(b"\n");

            parse_xsdt(&mut acpi_info)?;
        }
    }

    // Fall back to RSDT if no XSDT
    if acpi_info.xsdt_addr.is_none() && rsdp.rsdt_address != 0 {
        acpi_info.rsdt_addr = Some(PhysAddr::new(rsdp.rsdt_address as u64));
        serial::serial_write(b"[ACPI] RSDT at: 0x");
        print_hex_u64(rsdp.rsdt_address as u64);
        serial::serial_write(b"\n");

        parse_rsdt(&mut acpi_info)?;
    }

    // Report discovered tables
    if acpi_info.madt_addr.is_some() {
        serial::serial_write(b"[ACPI] Found MADT (Multiple APIC Description Table)\n");
    }
    if acpi_info.hpet_addr.is_some() {
        serial::serial_write(b"[ACPI] Found HPET table\n");
    }
    if acpi_info.mcfg_addr.is_some() {
        serial::serial_write(b"[ACPI] Found MCFG (PCI Express Configuration)\n");
    }
    if acpi_info.fadt_addr.is_some() {
        serial::serial_write(b"[ACPI] Found FADT (Fixed ACPI Description Table)\n");
    }

    ACPI_INFO = Some(acpi_info);

    serial::serial_write(b"[ACPI] ACPI initialization complete\n");
    Ok(())
}

/// Parse RSDT (Root System Description Table) for ACPI 1.0
unsafe fn parse_rsdt(acpi_info: &mut AcpiInfo) -> Result<(), &'static str> {
    let rsdt_addr = acpi_info.rsdt_addr.ok_or("No RSDT address")?;

    // Use identity mapping for low memory ACPI tables
    let rsdt_virt = rsdt_addr.as_u64() as *const SdtHeader;
    let header = read_volatile(rsdt_virt);

    // Validate signature
    if &header.signature != b"RSDT" {
        return Err("Invalid RSDT signature");
    }

    // Calculate number of entries (each is a 32-bit physical address)
    let header_size = core::mem::size_of::<SdtHeader>();
    let entry_count = (header.length as usize - header_size) / 4;

    serial::serial_write(b"[ACPI] RSDT contains ");
    print_u64(entry_count as u64);
    serial::serial_write(b" entries\n");

    // Parse entry pointers
    let entries = (rsdt_virt as usize + header_size) as *const u32;
    for i in 0..entry_count {
        let table_addr = PhysAddr::new(read_volatile(entries.add(i)) as u64);
        parse_table_at(acpi_info, table_addr);
    }

    Ok(())
}

/// Parse XSDT (Extended System Description Table) for ACPI 2.0+
unsafe fn parse_xsdt(acpi_info: &mut AcpiInfo) -> Result<(), &'static str> {
    let xsdt_addr = acpi_info.xsdt_addr.ok_or("No XSDT address")?;

    // Use identity mapping for low memory ACPI tables
    let xsdt_virt = xsdt_addr.as_u64() as *const SdtHeader;
    let header = read_volatile(xsdt_virt);

    // Validate signature
    if &header.signature != b"XSDT" {
        return Err("Invalid XSDT signature");
    }

    // Calculate number of entries (each is a 64-bit physical address)
    let header_size = core::mem::size_of::<SdtHeader>();
    let entry_count = (header.length as usize - header_size) / 8;

    serial::serial_write(b"[ACPI] XSDT contains ");
    print_u64(entry_count as u64);
    serial::serial_write(b" entries\n");

    // Parse entry pointers
    serial::serial_write(b"[ACPI] XSDT base: 0x");
    print_hex_u64(xsdt_virt as u64);
    serial::serial_write(b", header size: ");
    print_u64(header_size as u64);
    serial::serial_write(b"\n");

    let entries_ptr = (xsdt_virt as usize + header_size);
    serial::serial_write(b"[ACPI] Entries start at: 0x");
    print_hex_u64(entries_ptr as u64);
    serial::serial_write(b"\n");

    let entries = entries_ptr as *const u64;

    for i in 0..entry_count {
        serial::serial_write(b"[ACPI] Reading XSDT entry ");
        print_u64(i as u64);
        serial::serial_write(b" at address 0x");
        print_hex_u64(entries.add(i) as u64);
        serial::serial_write(b"...\n");

        let entry_addr = entries.add(i);

        // Try to read the 64-bit entry using byte-by-byte access to avoid alignment issues
        let mut table_addr_raw: u64 = 0;
        let entry_bytes = entry_addr as *const u8;

        serial::serial_write(b"[ACPI]   Reading entry bytes...\n");
        for j in 0..8 {
            let byte_addr = entry_bytes.add(j);
            let byte_val = read_volatile(byte_addr);
            table_addr_raw |= (byte_val as u64) << (j * 8);

            // Debug: show each byte read
            if j == 0 {
                serial::serial_write(b"[ACPI]   Bytes: ");
            }
            let hex_chars = b"0123456789abcdef";
            serial::serial_write(&[hex_chars[(byte_val >> 4) as usize]]);
            serial::serial_write(&[hex_chars[(byte_val & 0xF) as usize]]);
            serial::serial_write(b" ");
        }
        serial::serial_write(b"\n");

        serial::serial_write(b"[ACPI]   Table address: 0x");
        print_hex_u64(table_addr_raw);
        serial::serial_write(b"\n");

        // Validate the address
        if table_addr_raw == 0 {
            serial::serial_write(b"[ACPI]   Skipping null table address\n");
            continue;
        }

        if table_addr_raw > 0x100000000 {
            serial::serial_write(b"[ACPI]   Skipping invalid table address (too high)\n");
            continue;
        }

        let table_addr = PhysAddr::new(table_addr_raw);
        serial::serial_write(b"[ACPI]   Parsing table...\n");
        parse_table_at(acpi_info, table_addr);
        serial::serial_write(b"[ACPI]   Table parsed successfully\n");
    }

    Ok(())
}

/// Parse a single ACPI table at the given address
unsafe fn parse_table_at(acpi_info: &mut AcpiInfo, table_addr: PhysAddr) {
    // Use identity mapping for low memory ACPI tables
    let table_virt = table_addr.as_u64() as *const SdtHeader;

    serial::serial_write(b"[ACPI]     Reading table header at 0x");
    print_hex_u64(table_virt as u64);
    serial::serial_write(b"\n");

    let header = read_volatile(table_virt);

    serial::serial_write(b"[ACPI]     Table signature: ");
    for i in 0..4 {
        serial::serial_write(&[header.signature[i]]);
    }
    serial::serial_write(b"\n");

    // Match known table signatures
    match &header.signature {
        b"APIC" => {
            // MADT (Multiple APIC Description Table)
            acpi_info.madt_addr = Some(table_addr);
        }
        b"HPET" => {
            // High Precision Event Timer
            acpi_info.hpet_addr = Some(table_addr);
        }
        b"MCFG" => {
            // PCI Express Memory Mapped Configuration
            acpi_info.mcfg_addr = Some(table_addr);
        }
        b"FACP" => {
            // Fixed ACPI Description Table (FADT)
            acpi_info.fadt_addr = Some(table_addr);
        }
        _ => {
            // Other tables we don't care about yet
        }
    }
}

/// Get the MADT (Multiple APIC Description Table) address
///
/// The MADT contains information about interrupt controllers and processors.
pub fn get_madt_address() -> Option<PhysAddr> {
    unsafe { ACPI_INFO.as_ref()?.madt_addr }
}

/// Get the HPET table address
pub fn get_hpet_address() -> Option<PhysAddr> {
    unsafe { ACPI_INFO.as_ref()?.hpet_addr }
}

/// Get the MCFG (PCI Express Configuration) table address
pub fn get_mcfg_address() -> Option<PhysAddr> {
    unsafe { ACPI_INFO.as_ref()?.mcfg_addr }
}

/// Get the FADT (Fixed ACPI Description Table) address
pub fn get_fadt_address() -> Option<PhysAddr> {
    unsafe { ACPI_INFO.as_ref()?.fadt_addr }
}

// Helper functions for debug output
fn print_hex_u64(n: u64) {
    let hex_chars = b"0123456789abcdef";
    let mut buf = [0u8; 16];

    for i in 0..16 {
        let shift = (15 - i) * 4;
        let nibble = ((n >> shift) & 0xF) as usize;
        buf[i] = hex_chars[nibble];
    }

    serial::serial_write(&buf);
}

fn print_u64(mut n: u64) {
    if n == 0 {
        serial::serial_write(b"0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        serial::serial_write_byte(buf[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsdp_size() {
        assert_eq!(core::mem::size_of::<Rsdp>(), 20);
        assert_eq!(core::mem::size_of::<RsdpExtended>(), 36);
    }

    #[test]
    fn test_sdt_header_size() {
        assert_eq!(core::mem::size_of::<SdtHeader>(), 36);
    }
}
