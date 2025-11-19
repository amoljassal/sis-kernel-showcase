// Minimal FDT parser (host-side) to exercise DT parsing logic similar to kernel platform/dt.rs
// This is intentionally small and self-contained for coverage purposes.

#[repr(C)]
#[derive(Clone, Copy)]
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

const FDT_MAGIC: u32 = 0xD00D_FEEDu32;
const FDT_BEGIN_NODE: u32 = 0x0000_0001;
const FDT_END_NODE: u32 = 0x0000_0002;
const FDT_PROP: u32 = 0x0000_0003;
const FDT_NOP: u32 = 0x0000_0004;
const FDT_END: u32 = 0x0000_0009;

fn be32(x: u32) -> u32 { u32::from_be(x) }
fn be64(x: u64) -> u64 { u64::from_be(x) }

pub fn parse_uart_base_and_irq(dtb: &[u8]) -> Option<(usize, u32)> {
    if dtb.len() < core::mem::size_of::<FdtHeader>() { return None; }
    let hdr = unsafe { &*(dtb.as_ptr() as *const FdtHeader) };
    if be32(hdr.magic) != FDT_MAGIC { return None; }
    let off_struct = be32(hdr.off_dt_struct) as usize;
    let off_strings = be32(hdr.off_dt_strings) as usize;
    let size_struct = be32(hdr.size_dt_struct) as usize;
    let mut p = dtb.get(off_struct..)?.as_ptr();
    let end = unsafe { p.add(size_struct) };
    let strings = dtb.get(off_strings..)?;

    let mut node_is_uart = false;
    let mut uart_base: usize = 0;
    let mut uart_irq: u32 = 0;

    unsafe {
        while p < end {
            let token = u32::from_be(*(p as *const u32)); p = p.add(4);
            match token {
                FDT_BEGIN_NODE => {
                    // read name
                    let mut q = p; while q < end && *q != 0 { q = q.add(1); }
                    // skip name + NUL, align
                    q = q.add(1);
                    p = ((q as usize + 3) & !3) as *const u8;
                    node_is_uart = false;
                }
                FDT_END_NODE => { node_is_uart = false; }
                FDT_PROP => {
                    let len = u32::from_be(*(p as *const u32)) as usize; p = p.add(4);
                    let nameoff = u32::from_be(*(p as *const u32)) as usize; p = p.add(4);
                    let val = p; let next = ((p as usize + len + 3) & !3) as *const u8; p = next;
                    // property name
                    let sname = {
                        let mut s = strings.as_ptr().add(nameoff);
                        while *s != 0 { s = s.add(1); }
                        let n = s as usize - (strings.as_ptr().add(nameoff) as usize);
                        let bytes = core::slice::from_raw_parts(strings.as_ptr().add(nameoff), n);
                        core::str::from_utf8_unchecked(bytes)
                    };
                    match sname {
                        "compatible" => {
                            let bytes = core::slice::from_raw_parts(val, len);
                            if bytes.windows(11).any(|w| w == b"arm,pl011\0") {
                                node_is_uart = true;
                            }
                        }
                        "reg" if node_is_uart => {
                            // read addr,size as be32 pairs (simple 32-bit cells)
                            if len >= 8 {
                                let addr = u32::from_be(*(val as *const u32)) as usize;
                                // skip size
                                let _size = u32::from_be(*(val.add(4) as *const u32)) as usize;
                                uart_base = addr;
                            }
                        }
                        "interrupts" if node_is_uart => {
                            // type, num, flags (3 cells)
                            if len >= 12 {
                                // let _ty = u32::from_be(*(val as *const u32));
                                let num = u32::from_be(*(val.add(4) as *const u32));
                                uart_irq = num;
                            }
                        }
                        _ => {}
                    }
                }
                FDT_NOP => {}
                FDT_END => break,
                _ => break,
            }
        }
    }
    if uart_base != 0 { Some((uart_base, uart_irq)) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    // helper to append u32 BE
    fn be32b(v: u32, out: &mut Vec<u8>) { out.extend_from_slice(&v.to_be_bytes()); }
    fn align4(out: &mut Vec<u8>) { while out.len() % 4 != 0 { out.push(0); } }

    #[test]
    fn parse_minimal_uart_node() {
        // Build a minimal FDT with a uart node compatible arm,pl011, reg, interrupts
        let mut strings = Vec::new();
        // strings: "compatible\0reg\0interrupts\0"
        let off_compatible = 0usize;
        strings.extend_from_slice(b"compatible\0");
        let off_reg = strings.len();
        strings.extend_from_slice(b"reg\0");
        let off_interrupts = strings.len();
        strings.extend_from_slice(b"interrupts\0");
        align4(&mut strings);

        let mut structblk = Vec::new();
        be32b(FDT_BEGIN_NODE, &mut structblk);
        structblk.extend_from_slice(b"uart@0\0"); align4(&mut structblk);
        // compatible: "arm,pl011\0"
        be32b(FDT_PROP, &mut structblk);
        be32b(11, &mut structblk);
        be32b(off_compatible as u32, &mut structblk);
        structblk.extend_from_slice(b"arm,pl011\0"); align4(&mut structblk);
        // reg: base=0x09000000 size=0x1000
        be32b(FDT_PROP, &mut structblk);
        be32b(8, &mut structblk);
        be32b(off_reg as u32, &mut structblk);
        be32b(0x0900_0000, &mut structblk); // base
        be32b(0x0000_1000, &mut structblk); // size
        // interrupts: type=0, num=33, flags=0x4
        be32b(FDT_PROP, &mut structblk);
        be32b(12, &mut structblk);
        be32b(off_interrupts as u32, &mut structblk);
        be32b(0, &mut structblk);
        be32b(33, &mut structblk);
        be32b(4, &mut structblk);
        be32b(FDT_END_NODE, &mut structblk);
        be32b(FDT_END, &mut structblk);
        align4(&mut structblk);

        // header
        let mut dtb = Vec::new(); dtb.resize(40, 0); // placeholder
        let off_struct = dtb.len(); dtb.extend_from_slice(&structblk);
        let off_strings = dtb.len(); dtb.extend_from_slice(&strings);
        let totalsize = dtb.len() as u32;
        let size_dt_struct = structblk.len() as u32;
        // fill header
        let hdr = FdtHeader {
            magic: FDT_MAGIC.to_be(), totalsize: totalsize.to_be(), off_dt_struct: (off_struct as u32).to_be(),
            off_dt_strings: (off_strings as u32).to_be(), off_mem_rsvmap: 0u32.to_be(), version: 17u32.to_be(),
            last_comp_version: 16u32.to_be(), boot_cpuid_phys: 0, size_dt_strings: (strings.len() as u32).to_be(), size_dt_struct: size_dt_struct.to_be()
        };
        let hdr_bytes = unsafe {
            core::slice::from_raw_parts(&hdr as *const _ as *const u8, core::mem::size_of::<FdtHeader>())
        };
        dtb[..hdr_bytes.len()].copy_from_slice(hdr_bytes);

        let p = parse_uart_base_and_irq(&dtb).expect("parse");
        assert_eq!(p.0, 0x0900_0000usize);
        assert_eq!(p.1, 33u32);
    }
}

