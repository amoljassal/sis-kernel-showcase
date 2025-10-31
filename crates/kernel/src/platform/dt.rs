//! Minimal DT (FDT) intake. Parses a few key nodes/properties to derive UART/GIC/RAM,
//! with safe fallbacks to QEMU defaults. Optional and only used when enabled by feature.

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

/// Platform derived from DTB; currently a thin wrapper with defaults.
pub struct DtPlatform {
    uart: UartDesc,
    gic: GicDesc,
    timer: TimerDesc,
    mmio: &'static [MmioRange],
    ram: &'static [RamRange],
}

impl super::Platform for DtPlatform {
    fn uart(&self) -> UartDesc { self.uart }
    fn gic(&self) -> GicDesc { self.gic }
    fn timer(&self) -> TimerDesc { self.timer }
    fn mmio_ranges(&self) -> &'static [MmioRange] { self.mmio }
    fn ram_ranges(&self) -> &'static [RamRange] { self.ram }
}

const DT_MMIO_EMPTY: &[MmioRange] = &[];
const DT_RAM_EMPTY: &[RamRange] = &[];
static mut DT_INSTANCE: Option<DtPlatform> = None;

// Backing storage for dynamic slices
static mut DT_MMIO: [MmioRange; 4] = [MmioRange { start: 0, size: 0, device: false }; 4];
static mut DT_RAM: [RamRange; 4] = [RamRange { start: 0, size: 0 }; 4];

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

    // Minimal walker state
    let mut p = struct_base;
    let end = struct_base.add(size_struct);
    // Assume 64-bit address/size cells
    let mut addr_cells: u32 = 2;
    let mut size_cells: u32 = 2;
    // Current node flags
    let mut node_is_uart = false;
    let mut node_is_gic = false;
    let mut node_is_mem = false;

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
                node_is_mem = name_str == "memory"; // hint; also check device_type later
                node_is_uart = false;
                node_is_gic = false;
                // advance past name + NUL, 4-byte aligned
                q = q.add(1);
                let aligned = (q as usize + 3) & !3;
                p = aligned as *const u8;
            }
            FDT_END_NODE => {
                node_is_uart = false;
                node_is_gic = false;
                node_is_mem = false;
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
                        // check for pl011 and gic
                        let bytes = core::slice::from_raw_parts(val, len);
                        if bytes.windows(9).any(|w| w == b"arm,pl011") { node_is_uart = true; }
                        if bytes.windows(10).any(|w| w == b"arm,gic-400") || bytes.windows(9).any(|w| w == b"arm,gic-v3") {
                            node_is_gic = true;
                        }
                    }
                    "clock-frequency" => {
                        if node_is_uart && len >= 4 {
                            uart_clk = u32::from_be(ptr::read_unaligned(val as *const u32));
                        }
                    }
                    "reg" => {
                        // Interpret reg as pairs of (address,size) according to cells; assume 64-bit when 2/2
                        if addr_cells == 2 && size_cells == 2 && len >= 16 {
                            // Helper to read u64 BE from two u32
                            let mut off = 0usize;
                            let read_u64 = |base: *const u8, off: &mut usize| -> u64 {
                                let hi = u32::from_be(unsafe { ptr::read_unaligned(base.add(*off) as *const u32) }) as u64; *off += 4;
                                let lo = u32::from_be(unsafe { ptr::read_unaligned(base.add(*off) as *const u32) }) as u64; *off += 4;
                                (hi << 32) | lo
                            };
                            if node_is_uart {
                                let addr = read_u64(val, &mut off) as usize; let _size = read_u64(val, &mut off) as usize;
                                if addr != 0 { uart_base = addr; }
                            } else if node_is_gic {
                                // First region = GICD, second = GICR (if present)
                                let addr0 = read_u64(val, &mut off) as usize; let _size0 = read_u64(val, &mut off) as usize;
                                if len >= 32 {
                                    let addr1 = read_u64(val, &mut off) as usize; let _size1 = read_u64(val, &mut off) as usize;
                                    if addr1 != 0 { gicr_base = addr1; }
                                }
                                if addr0 != 0 { gicd_base = addr0; }
                            } else if node_is_mem {
                                let addr = read_u64(val, &mut off) as usize; let size = read_u64(val, &mut off) as usize;
                                if addr != 0 && size != 0 { ram_start = addr; ram_size = size; }
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

    // Construct dynamic slices for mmio/ram (best-effort)
    let mut mmio_len = 0usize;
    // GIC region: cover up to 2MiB by default to avoid overlaps
    DT_MMIO[mmio_len] = MmioRange { start: gicd_base, size: 0x0020_0000, device: true }; mmio_len += 1;
    // UART region: 4 KiB
    DT_MMIO[mmio_len] = MmioRange { start: uart_base, size: 0x1000, device: true }; mmio_len += 1;
    let mmio: &'static [MmioRange] = &DT_MMIO[..mmio_len];

    let mut ram_len = 0usize;
    DT_RAM[ram_len] = RamRange { start: ram_start, size: ram_size }; ram_len += 1;
    let ram: &'static [RamRange] = &DT_RAM[..ram_len];

    let uart = UartDesc { base: uart_base, clock_hz: uart_clk };
    let gic = GicDesc { gicd: gicd_base, gicr: gicr_base };
    let timer = TimerDesc { freq_hz: 0 }; // prefer CNTFRQ_EL0 at runtime; 0 indicates fallback
    DT_INSTANCE = Some(DtPlatform { uart, gic, timer, mmio, ram });
    // Return a stable trait object reference to the instance
    DT_INSTANCE.as_ref().map(|inst| inst as &dyn Platform)
}
