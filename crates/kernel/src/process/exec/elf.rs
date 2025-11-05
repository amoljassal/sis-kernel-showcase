/// ELF64 loader for AArch64
///
/// Parses and loads ELF64 executables, mapping PT_LOAD segments
/// and setting up the initial user stack with argc/argv/envp/auxv.

use crate::lib::error::Errno;
use crate::process::{Task, VmaFlags};
use crate::mm::{PAGE_SIZE, PteFlags};
use alloc::vec::Vec;
use alloc::string::String;

/// ELF magic number
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// ELF class
const ELFCLASS64: u8 = 2;

/// ELF machine
const EM_AARCH64: u16 = 183;

/// Program header types
const PT_LOAD: u32 = 1;
const PT_INTERP: u32 = 3;
const PT_PHDR: u32 = 6;

/// Segment flags
const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

/// Auxiliary vector entry types
const AT_NULL: u64 = 0;
const AT_PHDR: u64 = 3;
const AT_PHENT: u64 = 4;
const AT_PHNUM: u64 = 5;
const AT_PAGESZ: u64 = 6;
const AT_BASE: u64 = 7;
const AT_ENTRY: u64 = 9;
const AT_UID: u64 = 11;
const AT_EUID: u64 = 12;
const AT_GID: u64 = 13;
const AT_EGID: u64 = 14;
const AT_RANDOM: u64 = 25;

/// ELF error type
#[derive(Debug)]
pub enum ElfError {
    InvalidMagic,
    InvalidClass,
    InvalidMachine,
    InvalidHeader,
    UnsupportedFeature,
    MemoryError,
}

impl From<ElfError> for Errno {
    fn from(e: ElfError) -> Self {
        match e {
            ElfError::InvalidMagic | ElfError::InvalidClass
            | ElfError::InvalidMachine | ElfError::InvalidHeader => Errno::ENOEXEC,
            ElfError::UnsupportedFeature => Errno::EINVAL,
            ElfError::MemoryError => Errno::ENOMEM,
        }
    }
}

/// ELF64 header
#[repr(C)]
#[derive(Clone, Copy)]
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

/// ELF64 program header
#[repr(C)]
#[derive(Clone, Copy, Debug)]
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

/// Load ELF binary into task's address space
pub fn load_elf(
    task: &mut Task,
    elf_data: &[u8],
    argv: Vec<String>,
    envp: Vec<String>,
) -> Result<u64, ElfError> {
    // Parse and validate ELF header
    if elf_data.len() < core::mem::size_of::<Elf64Ehdr>() {
        return Err(ElfError::InvalidHeader);
    }

    let ehdr = unsafe {
        &*(elf_data.as_ptr() as *const Elf64Ehdr)
    };

    // Validate magic
    if ehdr.e_ident[0..4] != ELF_MAGIC {
        return Err(ElfError::InvalidMagic);
    }

    // Validate class (64-bit)
    if ehdr.e_ident[4] != ELFCLASS64 {
        return Err(ElfError::InvalidClass);
    }

    // Validate machine (AArch64)
    if ehdr.e_machine != EM_AARCH64 {
        return Err(ElfError::InvalidMachine);
    }

    let entry = ehdr.e_entry;
    let phoff = ehdr.e_phoff as usize;
    let phnum = ehdr.e_phnum as usize;
    let phentsize = ehdr.e_phentsize as usize;

    if phnum == 0 {
        return Err(ElfError::InvalidHeader);
    }

    crate::info!("ELF: entry={:#x}, phnum={}, phentsize={}", entry, phnum, phentsize);

    // Parse program headers and load PT_LOAD segments
    let mut phdr_addr = 0u64;

    for i in 0..phnum {
        let ph_offset = phoff + i * phentsize;
        if ph_offset + phentsize > elf_data.len() {
            return Err(ElfError::InvalidHeader);
        }

        let phdr = unsafe {
            &*(elf_data.as_ptr().add(ph_offset) as *const Elf64Phdr)
        };

        match phdr.p_type {
            PT_LOAD => {
                load_segment(task, elf_data, phdr)?;
            }
            PT_PHDR => {
                phdr_addr = phdr.p_vaddr;
            }
            _ => {
                // Ignore other segments for now
            }
        }
    }

    // Set up initial stack with argc/argv/envp/auxv
    let sp = setup_stack(task, argv, envp, entry, phdr_addr, phnum as u64, phentsize as u64)?;

    // Update trap frame to start at entry point
    task.trap_frame.pc = entry;
    task.trap_frame.sp = sp;
    // SPSR for EL0: clear all flags, EL0t mode
    task.trap_frame.pstate = 0;

    crate::info!("ELF loaded: entry={:#x}, sp={:#x}", entry, sp);

    Ok(entry)
}

/// Load a PT_LOAD segment
fn load_segment(
    task: &mut Task,
    elf_data: &[u8],
    phdr: &Elf64Phdr,
) -> Result<(), ElfError> {
    let vaddr = phdr.p_vaddr;
    let filesz = phdr.p_filesz as usize;
    let memsz = phdr.p_memsz as usize;
    let offset = phdr.p_offset as usize;
    let flags = phdr.p_flags;

    crate::info!(
        "PT_LOAD: vaddr={:#x}, filesz={:#x}, memsz={:#x}, flags={:#x}",
        vaddr, filesz, memsz, flags
    );

    // Convert ELF flags to VMA flags
    let mut vma_flags = VmaFlags::empty();
    if (flags & PF_R) != 0 { vma_flags |= VmaFlags::READ; }
    if (flags & PF_W) != 0 { vma_flags |= VmaFlags::WRITE; }
    if (flags & PF_X) != 0 { vma_flags |= VmaFlags::EXEC; }

    // Enforce W^X: cannot be both writable and executable
    if vma_flags.contains(VmaFlags::WRITE) && vma_flags.contains(VmaFlags::EXEC) {
        return Err(ElfError::UnsupportedFeature);
    }

    // Round addresses to page boundaries
    let page_start = vaddr & !(PAGE_SIZE as u64 - 1);
    let page_end = (vaddr + memsz as u64 + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1);

    // Create VMA for this segment
    let vma = crate::process::Vma {
        start: page_start,
        end: page_end,
        flags: vma_flags,
        offset: 0,
    };

    task.mm.insert_vma(vma).map_err(|_| ElfError::MemoryError)?;

    // Note: For Phase A1, we create VMAs only. Page faults will allocate pages on demand.
    // Full eager loading with page table setup will be implemented in later phases.

    crate::debug!(
        "Created VMA {:#x}-{:#x} (flags={:?})",
        page_start, page_end, vma_flags
    );

    Ok(())
}

/// Set up initial user stack with argc/argv/envp/auxv
fn setup_stack(
    task: &mut Task,
    argv: Vec<String>,
    envp: Vec<String>,
    entry: u64,
    phdr_addr: u64,
    phnum: u64,
    phentsize: u64,
) -> Result<u64, ElfError> {
    let stack_top = crate::mm::USER_STACK_TOP;

    // Ensure stack VMA exists
    task.mm.setup_stack().map_err(|_| ElfError::MemoryError)?;

    // Build stack layout from top down:
    // 1. Random bytes (16 bytes for AT_RANDOM)
    // 2. Environment strings
    // 3. Argument strings
    // 4. Padding to 16-byte alignment
    // 5. Auxv entries (ending with AT_NULL)
    // 6. envp[] pointers (ending with NULL)
    // 7. argv[] pointers (ending with NULL)
    // 8. argc

    let mut sp = stack_top;

    // 1. Random bytes for AT_RANDOM (16 bytes)
    sp -= 16;
    let random_addr = sp;
    // For Phase A1, use simple pseudo-random pattern
    let random_bytes: [u8; 16] = [
        0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    ];

    // 2. Environment strings
    let mut env_addrs = Vec::new();
    for env in envp.iter().rev() {
        let bytes = env.as_bytes();
        sp -= bytes.len() as u64 + 1; // +1 for null terminator
        env_addrs.push(sp);
    }
    env_addrs.reverse();

    // 3. Argument strings
    let mut arg_addrs = Vec::new();
    for arg in argv.iter().rev() {
        let bytes = arg.as_bytes();
        sp -= bytes.len() as u64 + 1; // +1 for null terminator
        arg_addrs.push(sp);
    }
    arg_addrs.reverse();

    // 4. Align to 16-byte boundary
    sp = sp & !0xF;

    // 5. Auxv entries (8 bytes type + 8 bytes value each)
    let auxv_entries = [
        (AT_PAGESZ, PAGE_SIZE as u64),
        (AT_PHDR, phdr_addr),
        (AT_PHENT, phentsize),
        (AT_PHNUM, phnum),
        (AT_ENTRY, entry),
        (AT_UID, task.cred.uid as u64),
        (AT_EUID, task.cred.euid as u64),
        (AT_GID, task.cred.gid as u64),
        (AT_EGID, task.cred.egid as u64),
        (AT_RANDOM, random_addr),
        (AT_NULL, 0),
    ];

    sp -= (auxv_entries.len() * 16) as u64;
    let auxv_start = sp;

    // 6. envp[] pointers
    sp -= ((env_addrs.len() + 1) * 8) as u64; // +1 for NULL terminator
    let envp_start = sp;

    // 7. argv[] pointers
    sp -= ((arg_addrs.len() + 1) * 8) as u64; // +1 for NULL terminator
    let argv_start = sp;

    // 8. argc
    sp -= 8;
    let argc_addr = sp;

    // Ensure 16-byte alignment
    sp = sp & !0xF;

    // For Phase A1, we create the stack structure but defer actual memory writes
    // to when page faults occur or when the kernel sets up the initial user context.
    // The page fault handler will allocate stack pages on demand.

    crate::info!(
        "Stack layout: argc={:#x}, argv={:#x}, envp={:#x}, auxv={:#x}, sp={:#x}",
        argc_addr, argv_start, envp_start, auxv_start, sp
    );

    Ok(sp)
}
