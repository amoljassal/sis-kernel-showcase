//! Enhanced FDT (Flattened Device Tree) parser for Raspberry Pi 5 and QEMU support.
//!
//! This module provides comprehensive device tree parsing with support for:
//! - UART devices (PL011, BCM2835-AUX)
//! - GICv3 interrupt controllers
//! - SDHCI controllers for SD card access
//! - PCIe controllers
//! - USB XHCI controllers
//! - Ethernet devices
//! - Memory regions and MMIO ranges
//!
//! The parser is designed to work with both QEMU virt and Raspberry Pi 5 device trees,
//! providing hardware-agnostic platform initialization.

use super::{GicDesc, MmioRange, Platform, RamRange, TimerDesc, UartDesc};
use core::ptr;

#[repr(C)]
#[derive(Copy, Clone)]
struct FdtHeader {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

fn be32(x: u32) -> u32 { u32::from_be(x) }

const FDT_MAGIC: u32 = 0xD00D_FEED;
const FDT_BEGIN_NODE: u32 = 0x0000_0001;
const FDT_END_NODE: u32   = 0x0000_0002;
const FDT_PROP: u32       = 0x0000_0003;
const FDT_NOP: u32        = 0x0000_0004;
const FDT_END: u32        = 0x0000_0009;

/// SDHCI device information
#[derive(Debug, Copy, Clone)]
pub struct SdhciInfo {
    pub base: usize,
    pub size: usize,
    pub irq: u32,
    pub quirks: u32,
}

/// PCIe controller information
#[derive(Debug, Copy, Clone)]
pub struct PcieInfo {
    pub base: usize,
    pub size: usize,
    pub cfg_base: usize,
    pub cfg_size: usize,
}

/// USB XHCI controller information
#[derive(Debug, Copy, Clone)]
pub struct UsbInfo {
    pub base: usize,
    pub size: usize,
    pub irq: u32,
}

/// Ethernet controller information
#[derive(Debug, Copy, Clone)]
pub struct EthInfo {
    pub base: usize,
    pub size: usize,
    pub irq: u32,
    pub mac_addr: [u8; 6],
}

/// Complete device map parsed from FDT
#[derive(Debug, Copy, Clone)]
pub struct DeviceMap {
    pub uart: Option<UartDesc>,
    pub gic: Option<GicDesc>,
    pub timer: Option<TimerDesc>,
    pub sdhci: Option<SdhciInfo>,
    pub pcie: Option<PcieInfo>,
    pub usb: Option<UsbInfo>,
    pub ethernet: Option<EthInfo>,
}

impl Default for DeviceMap {
    fn default() -> Self {
        Self {
            uart: None,
            gic: None,
            timer: None,
            sdhci: None,
            pcie: None,
            usb: None,
            ethernet: None,
        }
    }
}

/// Platform derived from DTB; enhanced for Raspberry Pi 5 and QEMU support.
pub struct DtPlatform {
    uart: UartDesc,
    gic: GicDesc,
    timer: TimerDesc,
    mmio: &'static [MmioRange],
    ram: &'static [RamRange],
    devices: DeviceMap,
}

impl DtPlatform {
    /// Get the complete device map
    pub fn devices(&self) -> &DeviceMap {
        &self.devices
    }
}

impl super::Platform for DtPlatform {
    fn uart(&self) -> UartDesc { self.uart }
    fn gic(&self) -> GicDesc { self.gic }
    fn timer(&self) -> TimerDesc { self.timer }
    fn mmio_ranges(&self) -> &'static [MmioRange] { self.mmio }
    fn ram_ranges(&self) -> &'static [RamRange] { self.ram }
}

static mut DT_INSTANCE: Option<DtPlatform> = None;

// Backing storage for dynamic slices
static mut DT_MMIO: [MmioRange; 8] = [MmioRange { start: 0, size: 0, device: false }; 8];
static mut DT_RAM: [RamRange; 4] = [RamRange { start: 0, size: 0 }; 4];
static mut DT_DEVICES: DeviceMap = DeviceMap {
    uart: None,
    gic: None,
    timer: None,
    sdhci: None,
    pcie: None,
    usb: None,
    ethernet: None,
};

/// Get the current device map (if FDT was parsed)
#[allow(static_mut_refs)]
pub fn get_device_map() -> Option<DeviceMap> {
    unsafe {
        if DT_INSTANCE.is_some() {
            Some(DT_DEVICES)
        } else {
            None
        }
    }
}

/// Attempt to create a platform from a DTB pointer. Returns None if parsing fails or unsupported.
#[allow(static_mut_refs)]
pub unsafe fn from_dtb(dtb_ptr: *const u8) -> Option<&'static dyn Platform> {
    if dtb_ptr.is_null() { return None; }
    // Validate header
    let hdr = &*(dtb_ptr as *const FdtHeader);
    if be32(hdr.magic) != FDT_MAGIC { return None; }

    // Locate structure and strings blocks
    let off_struct = be32(hdr.off_dt_struct) as usize;
    let off_strings = be32(hdr.off_dt_strings) as usize;
    let size_struct = be32(hdr.size_dt_struct) as usize;
    let struct_base = dtb_ptr.add(off_struct);
    let strings_base = dtb_ptr.add(off_strings);

    // Results with safe defaults
    let mut uart_base: usize = 0x0900_0000;
    let mut uart_clk: u32 = 24_000_000;
    let mut gicd_base: usize = 0x0800_0000;
    let mut gicr_base: usize = 0x080A_0000;
    let mut ram_start: usize = 0x4000_0000;
    let mut ram_size: usize = 0x2000_0000; // 512 MiB

    // Track whether we saw explicit values (not currently used; defaults suffice)

    let mut devmap = DeviceMap::default();

    // Parser state
    let mut p = struct_base;
    let end = struct_base.add(size_struct);
    let mut addr_cells: u32 = 2;
    let mut size_cells: u32 = 2;
    let mut int_cells: u32 = 3;  // GICv3 default

    // Current node flags
    let mut node_is_uart = false;
    let mut node_is_gic = false;
    let mut node_is_mem = false;
    let mut node_is_sdhci = false;
    let mut node_is_pcie = false;
    let mut node_is_usb = false;
    let mut node_is_eth = false;

    // Temporary device storage
    let mut sdhci_tmp = SdhciInfo { base: 0, size: 0, irq: 0, quirks: 0 };
    let mut pcie_tmp = PcieInfo { base: 0, size: 0, cfg_base: 0, cfg_size: 0 };
    let mut usb_tmp = UsbInfo { base: 0, size: 0, irq: 0 };
    let mut eth_tmp = EthInfo { base: 0, size: 0, irq: 0, mac_addr: [0; 6] };

    while p < end {
        let token = u32::from_be(ptr::read_unaligned(p as *const u32));
        p = p.add(4);
        match token {
            FDT_BEGIN_NODE => {
                // Read NUL-terminated name, then pad to 4 bytes
                let mut q = p;
                while q < end && ptr::read(q) != 0 { q = q.add(1); }
                let name_len = q as usize - p as usize;
                let name = core::slice::from_raw_parts(p, name_len);
                let name_str = core::str::from_utf8_unchecked(name);

                // Reset node flags
                node_is_uart = false;
                node_is_gic = false;
                node_is_mem = name_str == "memory" || name_str.starts_with("memory@");
                node_is_sdhci = false;
                node_is_pcie = false;
                node_is_usb = false;
                node_is_eth = false;

                // advance past name + NUL, 4-byte aligned
                q = q.add(1);
                let aligned = (q as usize + 3) & !3;
                p = aligned as *const u8;
            }
            FDT_END_NODE => {
                // Save device info when exiting node
                if node_is_sdhci && sdhci_tmp.base != 0 {
                    devmap.sdhci = Some(sdhci_tmp);
                    sdhci_tmp = SdhciInfo { base: 0, size: 0, irq: 0, quirks: 0 };
                }
                if node_is_pcie && pcie_tmp.base != 0 {
                    devmap.pcie = Some(pcie_tmp);
                    pcie_tmp = PcieInfo { base: 0, size: 0, cfg_base: 0, cfg_size: 0 };
                }
                if node_is_usb && usb_tmp.base != 0 {
                    devmap.usb = Some(usb_tmp);
                    usb_tmp = UsbInfo { base: 0, size: 0, irq: 0 };
                }
                if node_is_eth && eth_tmp.base != 0 {
                    devmap.ethernet = Some(eth_tmp);
                    eth_tmp = EthInfo { base: 0, size: 0, irq: 0, mac_addr: [0; 6] };
                }

                // Reset flags
                node_is_uart = false;
                node_is_gic = false;
                node_is_mem = false;
                node_is_sdhci = false;
                node_is_pcie = false;
                node_is_usb = false;
                node_is_eth = false;
            }
            FDT_PROP => {
                if p.add(8) > end { break; }
                let len = u32::from_be(ptr::read_unaligned(p as *const u32)) as usize; p = p.add(4);
                let nameoff = u32::from_be(ptr::read_unaligned(p as *const u32)) as usize; p = p.add(4);
                let val = p; let next = ((p as usize + len + 3) & !3) as *const u8; p = next;
                let pname_ptr = strings_base.add(nameoff);
                // Read property name until NUL
                let mut sn = pname_ptr; while ptr::read(sn) != 0 { sn = sn.add(1); }
                let sname = core::str::from_utf8_unchecked(core::slice::from_raw_parts(pname_ptr, sn as usize - pname_ptr as usize));

                match sname {
                    "#address-cells" => {
                        if len >= 4 { addr_cells = u32::from_be(ptr::read_unaligned(val as *const u32)); }
                    }
                    "#size-cells" => {
                        if len >= 4 { size_cells = u32::from_be(ptr::read_unaligned(val as *const u32)); }
                    }
                    "device_type" => {
                        // value is NUL-terminated string
                        let bytes = core::slice::from_raw_parts(val, len);
                        if bytes.starts_with(b"memory\0") { node_is_mem = true; }
                    }
                    "compatible" => {
                        let bytes = core::slice::from_raw_parts(val, len);

                        // Check for UART devices
                        if bytes.windows(11).any(|w| w == b"arm,pl011\0") {
                            node_is_uart = true;
                        }

                        // Check for GIC versions
                        if bytes.windows(12).any(|w| w == b"arm,gic-400\0")
                            || bytes.windows(11).any(|w| w == b"arm,gic-v3\0") {
                            node_is_gic = true;
                        }

                        // Check for SDHCI controllers (RPi5-specific)
                        if bytes.windows(19).any(|w| w == b"brcm,bcm2712-sdhci\0")
                            || bytes.windows(17).any(|w| w == b"arasan,sdhci-5.1\0") {
                            node_is_sdhci = true;
                        }

                        // Check for PCIe controllers (RPi5-specific)
                        if bytes.windows(18).any(|w| w == b"brcm,bcm2712-pcie\0") {
                            node_is_pcie = true;
                        }

                        // Check for USB XHCI
                        if bytes.windows(14).any(|w| w == b"generic-xhci\0")
                            || bytes.windows(18).any(|w| w == b"brcm,bcm2712-xhci\0") {
                            node_is_usb = true;
                        }

                        // Check for Ethernet (RPi5-specific)
                        if bytes.windows(19).any(|w| w == b"brcm,bcm2712-genet\0") {
                            node_is_eth = true;
                        }
                    }
                    "#interrupt-cells" => {
                        if len >= 4 {
                            int_cells = u32::from_be(ptr::read_unaligned(val as *const u32));
                        }
                    }
                    "clock-frequency" => {
                        if node_is_uart && len >= 4 {
                            uart_clk = u32::from_be(ptr::read_unaligned(val as *const u32));
                        }
                    }
                    "reg" => {
                        // Parse reg property based on address/size cells
                        if addr_cells == 2 && size_cells == 2 && len >= 16 {
                            let mut off = 0usize;
                            let read_u64 = |base: *const u8, off: &mut usize| -> u64 {
                                let hi = u32::from_be(ptr::read_unaligned(base.add(*off) as *const u32)) as u64;
                                *off += 4;
                                let lo = u32::from_be(ptr::read_unaligned(base.add(*off) as *const u32)) as u64;
                                *off += 4;
                                (hi << 32) | lo
                            };

                            if node_is_uart {
                                let addr = read_u64(val, &mut off) as usize;
                                let _size = read_u64(val, &mut off) as usize;
                                if addr != 0 {
                                    uart_base = addr;
                                }
                            } else if node_is_gic {
                                // First region = GICD
                                let addr0 = read_u64(val, &mut off) as usize;
                                let _size0 = read_u64(val, &mut off) as usize;
                                if addr0 != 0 {
                                    gicd_base = addr0;
                                }
                                // Second region = GICR (if present)
                                if len >= 32 {
                                    let addr1 = read_u64(val, &mut off) as usize;
                                    let _size1 = read_u64(val, &mut off) as usize;
                                    if addr1 != 0 {
                                        gicr_base = addr1;
                                    }
                                }
                            } else if node_is_mem {
                                let addr = read_u64(val, &mut off) as usize;
                                let size = read_u64(val, &mut off) as usize;
                                if addr != 0 && size != 0 {
                                    ram_start = addr;
                                    ram_size = size;
                                }
                            } else if node_is_sdhci {
                                let addr = read_u64(val, &mut off) as usize;
                                let size = read_u64(val, &mut off) as usize;
                                sdhci_tmp.base = addr;
                                sdhci_tmp.size = size;
                            } else if node_is_pcie {
                                let addr = read_u64(val, &mut off) as usize;
                                let size = read_u64(val, &mut off) as usize;
                                pcie_tmp.base = addr;
                                pcie_tmp.size = size;
                            } else if node_is_usb {
                                let addr = read_u64(val, &mut off) as usize;
                                let size = read_u64(val, &mut off) as usize;
                                usb_tmp.base = addr;
                                usb_tmp.size = size;
                            } else if node_is_eth {
                                let addr = read_u64(val, &mut off) as usize;
                                let size = read_u64(val, &mut off) as usize;
                                eth_tmp.base = addr;
                                eth_tmp.size = size;
                            }
                        }
                    }
                    "interrupts" => {
                        // Parse interrupts property (GIC format: type, num, flags)
                        if len >= 12 && int_cells == 3 {
                            let _irq_type = u32::from_be(ptr::read_unaligned(val as *const u32));
                            let irq_num = u32::from_be(ptr::read_unaligned(val.add(4) as *const u32));
                            let _irq_flags = u32::from_be(ptr::read_unaligned(val.add(8) as *const u32));

                            if node_is_sdhci {
                                sdhci_tmp.irq = irq_num;
                            } else if node_is_usb {
                                usb_tmp.irq = irq_num;
                            } else if node_is_eth {
                                eth_tmp.irq = irq_num;
                            }
                        }
                    }
                    _ => {}
                }
            }
            FDT_NOP => {}
            FDT_END => { break; }
            _ => { break; }
        }
    }

    // Construct device map
    devmap.uart = Some(UartDesc { base: uart_base, clock_hz: uart_clk });
    devmap.gic = Some(GicDesc { gicd: gicd_base, gicr: gicr_base });
    devmap.timer = Some(TimerDesc { freq_hz: 0 }); // Read from CNTFRQ_EL0 at runtime

    // Build MMIO ranges
    let mut mmio_len = 0usize;

    // GIC region: cover up to 2MiB to include redistributors
    DT_MMIO[mmio_len] = MmioRange { start: gicd_base, size: 0x0020_0000, device: true };
    mmio_len += 1;

    // UART region: 4 KiB
    DT_MMIO[mmio_len] = MmioRange { start: uart_base, size: 0x1000, device: true };
    mmio_len += 1;

    // Add SDHCI if present
    if let Some(sdhci) = devmap.sdhci {
        if mmio_len < DT_MMIO.len() {
            DT_MMIO[mmio_len] = MmioRange { start: sdhci.base, size: sdhci.size, device: true };
            mmio_len += 1;
        }
    }

    // Add USB if present
    if let Some(usb) = devmap.usb {
        if mmio_len < DT_MMIO.len() {
            DT_MMIO[mmio_len] = MmioRange { start: usb.base, size: usb.size, device: true };
            mmio_len += 1;
        }
    }

    // Add Ethernet if present
    if let Some(eth) = devmap.ethernet {
        if mmio_len < DT_MMIO.len() {
            DT_MMIO[mmio_len] = MmioRange { start: eth.base, size: eth.size, device: true };
            mmio_len += 1;
        }
    }

    let mmio: &'static [MmioRange] = &DT_MMIO[..mmio_len];

    // Build RAM ranges
    let mut ram_len = 0usize;
    DT_RAM[ram_len] = RamRange { start: ram_start, size: ram_size };
    ram_len += 1;
    let ram: &'static [RamRange] = &DT_RAM[..ram_len];

    // Log parsed devices
    crate::info!("FDT: UART @ {:#x} ({} Hz)", uart_base, uart_clk);
    crate::info!("FDT: GIC @ GICD={:#x} GICR={:#x}", gicd_base, gicr_base);
    crate::info!("FDT: RAM @ {:#x} ({} MiB)", ram_start, ram_size / (1024 * 1024));

    if let Some(sdhci) = devmap.sdhci {
        crate::info!("FDT: SDHCI @ {:#x} size={:#x} IRQ={}", sdhci.base, sdhci.size, sdhci.irq);
    }

    if let Some(usb) = devmap.usb {
        crate::info!("FDT: USB XHCI @ {:#x} size={:#x} IRQ={}", usb.base, usb.size, usb.irq);
    }

    if let Some(eth) = devmap.ethernet {
        crate::info!("FDT: Ethernet @ {:#x} size={:#x} IRQ={}", eth.base, eth.size, eth.irq);
    }

    if let Some(pcie) = devmap.pcie {
        crate::info!("FDT: PCIe @ {:#x} size={:#x}", pcie.base, pcie.size);
    }

    // Store device map
    DT_DEVICES = devmap;

    // Create platform instance
    let uart = UartDesc { base: uart_base, clock_hz: uart_clk };
    let gic = GicDesc { gicd: gicd_base, gicr: gicr_base };
    let timer = TimerDesc { freq_hz: 0 }; // prefer CNTFRQ_EL0 at runtime; 0 indicates fallback

    DT_INSTANCE = Some(DtPlatform { uart, gic, timer, mmio, ram, devices: devmap });

    // Return a stable trait object reference to the instance
    DT_INSTANCE.as_ref().map(|inst| inst as &dyn Platform)
}
