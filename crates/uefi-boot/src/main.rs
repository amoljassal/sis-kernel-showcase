#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::fmt::Write;
use core::mem;
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileMode, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryType, SearchType};
use uefi::table::cfg::ACPI2_GUID;
use uefi::{Handle, Identify};

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).ok();
    let _ = st.stdout().reset(false);
    let _ = st
        .stdout()
        .write_str("BOOT-ARM64 (UEFI)\r\nSIS UEFI loader v2 (VERBOSE)\r\n");

    match chainload_kernel(handle, st) {
        Ok(_) => unreachable!(),
        Err(_) => loop {},
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum ChainloadError {
    LoadedImage,
    OpenSfs,
    OpenRoot,
    OpenKernel,
    FileInfo,
    AllocatePages,
    Read,
    ExitBoot,
}

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

const PT_LOAD: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

#[repr(C)]
struct Elf64Sym {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: u64,
    st_size: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct BootInfo {
    rsdp_addr: u64,
}

// Keep BootInfo in static storage so the kernel can read it after we exit boot services.
static mut BOOT_INFO: BootInfo = BootInfo { rsdp_addr: 0 };

fn chainload_kernel(
    handle: Handle,
    mut st: SystemTable<Boot>,
) -> Result<Infallible, ChainloadError> {
    let _ = st.stdout().write_str("Opening LoadedImage...\r\n");
    st.boot_services().stall(100_000);
    let device = {
        let loaded = st
            .boot_services()
            .open_protocol_exclusive::<LoadedImage>(handle)
            .map_err(|_| ChainloadError::LoadedImage)?;
        loaded.device()
    };

    let _ = st.stdout().write_str("Opening SimpleFileSystem...\r\n");
    st.boot_services().stall(100_000);
    let mut root: Directory = {
        let mut sfs = st
            .boot_services()
            .open_protocol_exclusive::<SimpleFileSystem>(device)
            .map_err(|_| ChainloadError::OpenSfs)?;
        sfs.open_volume().map_err(|_| ChainloadError::OpenRoot)?
    };

    let _ = st.stdout().write_str("Opening root volume...\r\n");
    st.boot_services().stall(100_000);
    // Try multiple path variants
    let candidates = [
        (
            cstr16!(r"\EFI\SIS\KERNEL.ELF"),
            r"Trying path1 \EFI\SIS\KERNEL.ELF\r\n",
        ),
        (
            cstr16!(r"EFI\SIS\KERNEL.ELF"),
            r"Trying path2 EFI\SIS\KERNEL.ELF\r\n",
        ),
        (
            cstr16!(r"\efi\sis\kernel.elf"),
            r"Trying path3 \efi\sis\kernel.elf\r\n",
        ),
        (
            cstr16!(r"efi\sis\kernel.elf"),
            r"Trying path4 efi\sis\kernel.elf\r\n",
        ),
    ];
    let mut file_opt: Option<RegularFile> = None;
    for (c, label) in candidates {
        let _ = st.stdout().write_str(label);
        st.boot_services().stall(50_000);
        if let Ok(f) = root.open(c, FileMode::Read, FileAttribute::empty()) {
            if let Ok(uefi::proto::media::file::FileType::Regular(r)) = f.into_type() {
                file_opt = Some(r);
                break;
            }
        }
    }
    // Hierarchical attempt on image device: EFI/efi -> SIS/sis -> KERNEL.ELF/kernel.elf
    if file_opt.is_none() {
        let _ = st
            .stdout()
            .write_str("Hierarchical open on image device...\r\n");
        st.boot_services().stall(50_000);
        let efi_names = [cstr16!(r"EFI"), cstr16!(r"efi")];
        let sis_names = [cstr16!(r"SIS"), cstr16!(r"sis")];
        let file_names = [cstr16!(r"KERNEL.ELF"), cstr16!(r"kernel.elf")];
        for en in &efi_names {
            if let Ok(efi_file) = root.open(*en, FileMode::Read, FileAttribute::empty()) {
                if let Ok(uefi::proto::media::file::FileType::Dir(mut efi_dir)) =
                    efi_file.into_type()
                {
                    for sn in &sis_names {
                        if let Ok(sis_file) =
                            efi_dir.open(*sn, FileMode::Read, FileAttribute::empty())
                        {
                            if let Ok(uefi::proto::media::file::FileType::Dir(mut sis_dir)) =
                                sis_file.into_type()
                            {
                                for fnm in &file_names {
                                    if let Ok(kf) =
                                        sis_dir.open(*fnm, FileMode::Read, FileAttribute::empty())
                                    {
                                        if let Ok(uefi::proto::media::file::FileType::Regular(r)) =
                                            kf.into_type()
                                        {
                                            file_opt = Some(r);
                                            break;
                                        }
                                    }
                                }
                                if file_opt.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                    if file_opt.is_some() {
                        break;
                    }
                }
            }
        }
    }
    // Scan all SimpleFileSystem handles if not found
    if file_opt.is_none() {
        let _ = st
            .stdout()
            .write_str("Scanning all SimpleFileSystem handles...\r\n");
        st.boot_services().stall(50_000);
        // Collect handles first to avoid borrowing st while logging
        let handles_vec: alloc::vec::Vec<Handle> = {
            if let Ok(buf) = st
                .boot_services()
                .locate_handle_buffer(SearchType::ByProtocol(&SimpleFileSystem::GUID))
            {
                let mut v = alloc::vec::Vec::new();
                for &h in buf.iter() {
                    v.push(h);
                }
                v
            } else {
                alloc::vec::Vec::new()
            }
        };
        for (i, h) in handles_vec.iter().enumerate() {
            let _ = st.stdout().write_fmt(format_args!("FS handle {}\r\n", i));
            st.boot_services().stall(20_000);

            // Avoid logging while protocol is open to prevent borrow conflicts
            let mut matched_idx: Option<usize> = None;
            let mut found_file: Option<RegularFile> = None;
            {
                if let Ok(mut sfs) = st
                    .boot_services()
                    .open_protocol_exclusive::<SimpleFileSystem>(*h)
                {
                    if let Ok(mut root) = sfs.open_volume() {
                        for (idx, (c, _label)) in candidates.iter().enumerate() {
                            if let Ok(f) = root.open(*c, FileMode::Read, FileAttribute::empty()) {
                                if let Ok(uefi::proto::media::file::FileType::Regular(r)) =
                                    f.into_type()
                                {
                                    matched_idx = Some(idx);
                                    found_file = Some(r);
                                    break;
                                }
                            }
                        }
                        // Hierarchical attempt on this FS
                        if found_file.is_none() {
                            let efi_names = [cstr16!(r"EFI"), cstr16!(r"efi")];
                            let sis_names = [cstr16!(r"SIS"), cstr16!(r"sis")];
                            let file_names = [cstr16!(r"KERNEL.ELF"), cstr16!(r"kernel.elf")];
                            for en in &efi_names {
                                if let Ok(efi_file) =
                                    root.open(*en, FileMode::Read, FileAttribute::empty())
                                {
                                    if let Ok(uefi::proto::media::file::FileType::Dir(
                                        mut efi_dir,
                                    )) = efi_file.into_type()
                                    {
                                        for sn in &sis_names {
                                            if let Ok(sis_file) = efi_dir.open(
                                                *sn,
                                                FileMode::Read,
                                                FileAttribute::empty(),
                                            ) {
                                                if let Ok(
                                                    uefi::proto::media::file::FileType::Dir(
                                                        mut sis_dir,
                                                    ),
                                                ) = sis_file.into_type()
                                                {
                                                    for fnm in &file_names {
                                                        if let Ok(kf) = sis_dir.open(
                                                            *fnm,
                                                            FileMode::Read,
                                                            FileAttribute::empty(),
                                                        ) {
                                                            if let Ok(uefi::proto::media::file::FileType::Regular(r)) = kf.into_type() {
                                                                matched_idx = Some(10);
                                                                found_file = Some(r);
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    if found_file.is_some() {
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        if found_file.is_some() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(r) = found_file {
                let idx = matched_idx.unwrap_or(0);
                let _ = st.stdout().write_fmt(format_args!(
                    "Opened on FS handle {} with path{}\r\n",
                    i,
                    idx + 1
                ));
                st.boot_services().stall(20_000);
                file_opt = Some(r);
                break;
            }
        }
    }
    let mut file = match file_opt {
        Some(r) => r,
        None => {
            let _ = st
                .stdout()
                .write_str("OpenKernel failed (all candidates, all FS)\r\n");
            st.boot_services().stall(500_000);
            loop {}
        }
    };
    let _ = st.stdout().write_str("Reading ELF header...\r\n");
    st.boot_services().stall(50_000);

    let mut ehdr_buf = [0u8; core::mem::size_of::<Elf64Ehdr>()];
    let _ = file
        .read(&mut ehdr_buf[..])
        .map_err(|_| ChainloadError::Read)?;
    let ehdr: Elf64Ehdr = unsafe { core::ptr::read_unaligned(ehdr_buf.as_ptr() as *const _) };
    if &ehdr.e_ident[0..4] != b"\x7FELF" || ehdr.e_ident[4] != 2 {
        let _ = st.stdout().write_str("Not an ELF64 file\r\n");
        return Err(ChainloadError::Read);
    }

    let phoff = ehdr.e_phoff as usize;
    let phentsize = ehdr.e_phentsize as usize;
    let phnum = ehdr.e_phnum as usize;
    let _ = st.stdout().write_fmt(format_args!(
        "PH num: {} ent_size: {} off: 0x{:x}\r\n",
        phnum, phentsize, phoff
    ));
    st.boot_services().stall(50_000);
    let mut pht = vec![0u8; phentsize * phnum];
    file.set_position(phoff as u64)
        .map_err(|_| ChainloadError::Read)?;
    let rd = file.read(&mut pht[..]).map_err(|_| ChainloadError::Read)?;
    if rd < pht.len() {
        return Err(ChainloadError::Read);
    }

    let mut min_vaddr = usize::MAX;
    let mut max_vaddr = 0usize;
    for i in 0..phnum {
        let off = i * phentsize;
        let ph: Elf64Phdr = unsafe { core::ptr::read_unaligned(pht.as_ptr().add(off) as *const _) };
        if ph.p_type == PT_LOAD {
            let start = ph.p_vaddr as usize;
            let end = start + ph.p_memsz as usize;
            if start < min_vaddr {
                min_vaddr = start;
            }
            if end > max_vaddr {
                max_vaddr = end;
            }
        }
    }
    if min_vaddr == usize::MAX {
        let _ = st.stdout().write_str("No PT_LOAD segments found\r\n");
        return Err(ChainloadError::Read);
    }

    let map_base: usize = min_vaddr;
    let span = max_vaddr - min_vaddr;
    let pages = ((span + 0xFFF) & !0xFFF) / 0x1000;
    let _ = st.stdout().write_fmt(format_args!(
        "Allocating pages at 0x{:x} for PT_LOAD span ({} pages) ...\r\n",
        map_base,
        pages.max(1)
    ));
    st.boot_services().stall(100_000);

    // Try to allocate at the requested address first
    let _ = st.stdout().write_str("Attempting AllocateType::Address...\r\n");
    st.boot_services().stall(50_000);

    let alloc_result = st
        .boot_services()
        .allocate_pages(
            AllocateType::Address(map_base as u64),
            MemoryType::LOADER_CODE,
            pages.max(1),
        );

    match alloc_result {
        Ok(_) => {
            let _ = st.stdout().write_str("Address allocation succeeded!\r\n");
            st.boot_services().stall(50_000);
        }
        Err(e) => {
            let _ = st.stdout().write_fmt(format_args!(
                "Address allocation FAILED: {:?}\r\n",
                e
            ));
            let _ = st.stdout().write_str("Trying AllocateType::AnyPages as fallback...\r\n");
            st.boot_services().stall(100_000);

            // Fallback: Let UEFI choose any available address
            let actual_addr = st
                .boot_services()
                .allocate_pages(
                    AllocateType::AnyPages,
                    MemoryType::LOADER_CODE,
                    pages.max(1),
                )
                .map_err(|_| ChainloadError::AllocatePages)?;

            let _ = st.stdout().write_fmt(format_args!(
                "FALLBACK: Allocated at 0x{:x} (requested 0x{:x})\r\n",
                actual_addr, map_base
            ));
            st.boot_services().stall(100_000);

            // This is a critical issue: kernel expects to run at map_base but we allocated elsewhere
            // For now, we'll proceed but the kernel MUST be position-independent or this will crash
            let _ = st.stdout().write_str("WARNING: Kernel must be position-independent!\r\n");
            st.boot_services().stall(100_000);

            return Err(ChainloadError::AllocatePages);
        }
    }

    for i in 0..phnum {
        let off = i * phentsize;
        let ph: Elf64Phdr = unsafe { core::ptr::read_unaligned(pht.as_ptr().add(off) as *const _) };
        if ph.p_type != PT_LOAD || ph.p_filesz == 0 {
            continue;
        }
        let dst = ph.p_vaddr as usize;
        let filesz = ph.p_filesz as usize;
        let memsz = ph.p_memsz as usize;
        let _ = st.stdout().write_fmt(format_args!(
            "Segment: off=0x{:x} vaddr=0x{:x} filesz={} memsz={} -> dst=0x{:x}\r\n",
            ph.p_offset, ph.p_vaddr, filesz, memsz, dst
        ));
        st.boot_services().stall(10_000);

        file.set_position(ph.p_offset)
            .map_err(|_| ChainloadError::Read)?;
        let mut remaining = filesz;
        let mut written = 0usize;
        while remaining > 0 {
            let chunk = remaining.min(128 * 1024);
            let buf = unsafe { core::slice::from_raw_parts_mut((dst + written) as *mut u8, chunk) };
            let r = file.read(buf).map_err(|_| ChainloadError::Read)?;
            if r == 0 {
                break;
            }
            remaining -= r;
            written += r;
        }
        if memsz > filesz {
            unsafe {
                core::ptr::write_bytes((dst + filesz) as *mut u8, 0, memsz - filesz);
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "dsb ish",
            "ic iallu",
            "dsb ish",
            "isb",
            options(nostack, preserves_flags)
        );
    }

    // Determine entry: prefer ELF e_entry; if absent, try _start symbol; else fallback to map_base
    let mut entry_addr = map_base;
    let e_entry = ehdr.e_entry as usize;
    if e_entry >= min_vaddr && e_entry < max_vaddr {
        entry_addr = e_entry;
    } else {
        // Attempt to resolve _start from symbol table
        let shoff = ehdr.e_shoff as usize;
        let shentsize = ehdr.e_shentsize as usize;
        let shnum = ehdr.e_shnum as usize;
        if shoff > 0 && shentsize >= core::mem::size_of::<Elf64Shdr>() && shnum > 0 {
            let _ = st.stdout().write_str("Resolving _start from symtab...\r\n");
            st.boot_services().stall(50_000);
            // Read section headers
            let mut sht = vec![0u8; shentsize * shnum];
            file.set_position(shoff as u64).ok();
            if let Ok(r) = file.read(&mut sht[..]) {
                if r >= sht.len() {
                    // Find symtab and linked strtab
                    let mut symtab_off = 0usize;
                    let mut symtab_size = 0usize;
                    let mut symtab_entsize = 0usize;
                    let mut strtab_off = 0usize;
                    let mut strtab_size = 0usize;
                    for i in 0..shnum {
                        let off = i * shentsize;
                        let sh: Elf64Shdr =
                            unsafe { core::ptr::read_unaligned(sht.as_ptr().add(off) as *const _) };
                        if sh.sh_type == SHT_STRTAB && strtab_off == 0 {
                            // Keep first strtab; a better approach is to match by index, but suffices here
                            strtab_off = sh.sh_offset as usize;
                            strtab_size = sh.sh_size as usize;
                        }
                        if sh.sh_type == SHT_SYMTAB {
                            symtab_off = sh.sh_offset as usize;
                            symtab_size = sh.sh_size as usize;
                            symtab_entsize = sh.sh_entsize as usize;
                        }
                    }
                    if symtab_off > 0
                        && symtab_entsize >= core::mem::size_of::<Elf64Sym>()
                        && strtab_off > 0
                    {
                        // Read strtab
                        let mut strtab = vec![0u8; strtab_size];
                        file.set_position(strtab_off as u64).ok();
                        let _ = file.read(&mut strtab[..]);
                        // Iterate symbols
                        let count = symtab_size / symtab_entsize;
                        for i in 0..count {
                            let off = symtab_off + i * symtab_entsize;
                            let mut buf = [0u8; core::mem::size_of::<Elf64Sym>()];
                            file.set_position(off as u64).ok();
                            if file.read(&mut buf).ok().unwrap_or(0) < buf.len() {
                                break;
                            }
                            let sym: Elf64Sym =
                                unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const _) };
                            let name_off = sym.st_name as usize;
                            if name_off < strtab.len() {
                                // Compare with "_start"
                                let mut matches = true;
                                let pat = b"_start\0";
                                if name_off + pat.len() <= strtab.len() {
                                    for j in 0..pat.len() {
                                        if strtab[name_off + j] != pat[j] {
                                            matches = false;
                                            break;
                                        }
                                    }
                                } else {
                                    matches = false;
                                }
                                if matches {
                                    let sv = sym.st_value as usize;
                                    if sv >= min_vaddr && sv < max_vaddr {
                                        entry_addr = sv;
                                        let _ =
                                            st.stdout().write_str("Resolved _start symbol.\r\n");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let _ = st.stdout().write_fmt(format_args!(
        "Entry vaddr=0x{:x} mapped @ 0x{:x}\r\n",
        e_entry, entry_addr
    ));
    st.boot_services().stall(100_000);

    // Hexdump first 16 bytes at entry to verify instructions
    let mut bytes = [0u8; 16];
    unsafe {
        let src = entry_addr as *const u8;
        for i in 0..16 {
            bytes[i] = core::ptr::read_volatile(src.add(i));
        }
    }
    let _ = st.stdout().write_fmt(format_args!(
        "Entry bytes: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} \\ \\ {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}\r\n",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    ));
    st.boot_services().stall(100_000);

    #[cfg(feature = "dt-override")]
    {
    // Attempt to find a Device Tree (DTB) pointer in UEFI configuration tables
    let dtb_ptr_opt: Option<*const u8> = {
        let mut found: Option<*const u8> = None;
        for t in st.config_table() {
            let ptr = t.address as *const u8;
            if !ptr.is_null() {
                unsafe {
                    let magic = core::ptr::read(ptr as *const u32).to_be();
                    if magic == 0xD00D_FEED { found = Some(ptr); break; }
                }
            }
        }
        if let Some(p) = found {
            let _ = st.stdout().write_fmt(format_args!("Found DTB at 0x{:x}\r\n", p as usize));
            st.boot_services().stall(50_000);
        }
        found
    };

    // If we found a DTB pointer, try to locate DTB_PTR symbol in kernel and patch it
    if let Some(dtb_ptr) = dtb_ptr_opt {
        // Reuse or re-parse symtab/strtab to find DTB_PTR
        let shoff = ehdr.e_shoff as usize;
        let shentsize = ehdr.e_shentsize as usize;
        let shnum = ehdr.e_shnum as usize;
        if shoff > 0 && shentsize >= core::mem::size_of::<Elf64Shdr>() && shnum > 0 {
            let _ = st.stdout().write_str("Patching DTB_PTR symbol...\r\n");
            st.boot_services().stall(20_000);
            let mut sht = vec![0u8; shentsize * shnum];
            file.set_position(shoff as u64).ok();
            if let Ok(r) = file.read(&mut sht[..]) {
                if r >= sht.len() {
                    // Find symtab and strtab
                    let mut symtab_off = 0usize;
                    let mut symtab_size = 0usize;
                    let mut symtab_entsize = 0usize;
                    let mut strtab_off = 0usize;
                    let mut strtab_size = 0usize;
                    for i in 0..shnum {
                        let off = i * shentsize;
                        let sh: Elf64Shdr =
                            unsafe { core::ptr::read_unaligned(sht.as_ptr().add(off) as *const _) };
                        if sh.sh_type == SHT_STRTAB && strtab_off == 0 {
                            strtab_off = sh.sh_offset as usize;
                            strtab_size = sh.sh_size as usize;
                        }
                        if sh.sh_type == SHT_SYMTAB {
                            symtab_off = sh.sh_offset as usize;
                            symtab_size = sh.sh_size as usize;
                            symtab_entsize = sh.sh_entsize as usize;
                        }
                    }
                    if symtab_off > 0 && symtab_entsize >= core::mem::size_of::<Elf64Sym>() && strtab_off > 0 {
                        // Read strtab
                        let mut strtab = vec![0u8; strtab_size];
                        file.set_position(strtab_off as u64).ok();
                        let _ = file.read(&mut strtab[..]);
                        // Iterate symbols and find DTB_PTR / BOOT_RSDP_PHYS
                        let count = symtab_size / symtab_entsize;
                        let mut patched_dtb = false;
                        let mut patched_rsdp = false;
                        for i in 0..count {
                            let off = symtab_off + i * symtab_entsize;
                            let mut buf = [0u8; core::mem::size_of::<Elf64Sym>()];
                            file.set_position(off as u64).ok();
                            if file.read(&mut buf).ok().unwrap_or(0) < buf.len() { break; }
                            let sym: Elf64Sym = unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const _) };
                            let name_off = sym.st_name as usize;

                            // Compare with "DTB_PTR\0"
                            let pat_dtb = b"DTB_PTR\0";
                            if name_off + pat_dtb.len() <= strtab.len() {
                                if &strtab[name_off..name_off + pat_dtb.len()] == pat_dtb {
                                    let addr = sym.st_value as usize;
                                    unsafe { core::ptr::write_volatile(addr as *mut usize, dtb_ptr as usize); }
                                    let _ = st.stdout().write_fmt(format_args!("DTB_PTR patched at 0x{:x}\r\n", addr));
                                    st.boot_services().stall(20_000);
                                    patched_dtb = true;
                                }
                            }

                            // Compare with "BOOT_RSDP_PHYS\0"
                            if BOOT_INFO.rsdp_addr != 0 {
                                let pat_rsdp = b"BOOT_RSDP_PHYS\0";
                                if name_off + pat_rsdp.len() <= strtab.len() {
                                    if &strtab[name_off..name_off + pat_rsdp.len()] == pat_rsdp {
                                        let addr = sym.st_value as usize;
                                        unsafe { core::ptr::write_volatile(addr as *mut u64, BOOT_INFO.rsdp_addr); }
                                        let _ = st.stdout().write_fmt(format_args!("BOOT_RSDP_PHYS patched at 0x{:x}\r\n", addr));
                                        st.boot_services().stall(20_000);
                                        patched_rsdp = true;
                                    }
                                }
                            }

                            if patched_dtb && patched_rsdp {
                                break;
                            }
                        }
                        if !patched_dtb {
                            let _ = st.stdout().write_str("DTB_PTR symbol not found in kernel\r\n");
                            st.boot_services().stall(20_000);
                        }
                        if BOOT_INFO.rsdp_addr != 0 && !patched_rsdp {
                            let _ = st.stdout().write_str("BOOT_RSDP_PHYS symbol not found in kernel\r\n");
                            st.boot_services().stall(20_000);
                        }
                    }
                }
            }
        }
    }
    }

    // Collect boot info (ACPI RSDP) before exiting boot services
    // Populate BootInfo (static) before exiting boot services
    let rsdp_addr: u64;
    unsafe {
        BOOT_INFO = BootInfo::default();
        for table in st.config_table() {
            if table.guid == ACPI2_GUID {
                BOOT_INFO.rsdp_addr = table.address as u64;
                break;
            }
        }
        rsdp_addr = BOOT_INFO.rsdp_addr;
    }

    if rsdp_addr != 0 {
        let _ = st.stdout().write_fmt(format_args!(
            "Found ACPI RSDP at 0x{:x}\r\n",
            rsdp_addr as usize
        ));
        st.boot_services().stall(20_000);
    } else {
        let _ = st.stdout().write_str("ACPI RSDP not found in config tables\r\n");
        st.boot_services().stall(20_000);
    }

    // (No symbol patching needed; RSDP is passed via BootInfo)

    let _ = st.stdout().write_str("Exiting boot services...\r\n");
    st.boot_services().stall(100_000);
    let (_rt, _mmap) = st.exit_boot_services();

    // Optional: disable MMU and caches to ensure identity execution at physical addresses
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            // Read SCTLR_EL1
            "mrs x1, SCTLR_EL1",
            // Clear M (bit 0), C (bit 2), I (bit 12)
            "bic x1, x1, #1",
            "bic x1, x1, #(1 << 2)",
            "bic x1, x1, #(1 << 12)",
            "msr SCTLR_EL1, x1",
            "isb",
            options(nostack)
        );
        // Small delay
        for _ in 0..1000 {
            core::arch::asm!("nop");
        }
        // Try a test write to PL011 to see a marker
        let uart = 0x0900_0000 as *mut u8;
        core::ptr::write_volatile(uart, b'!');
    }

    // Call kernel entry with BootInfo pointer
    let entry: extern "C" fn(*const BootInfo) -> ! = unsafe { mem::transmute(entry_addr as usize) };
    let boot_info_ptr: *const BootInfo = unsafe { &BOOT_INFO };
    entry(boot_info_ptr)
}
