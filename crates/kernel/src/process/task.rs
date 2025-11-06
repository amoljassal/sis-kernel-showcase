/// Process task structure
///
/// Represents a single process/task in the system with all necessary state
/// for scheduling, memory management, file descriptors, and credentials.

use crate::lib::error::{KernelError, Errno};
use crate::arch::TrapFrame;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU32, Ordering};

pub type Pid = u32;

/// Process states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is runnable (ready or running)
    Running,
    /// Process is waiting (interruptible)
    Sleeping,
    /// Process has exited but not yet reaped
    Zombie,
    /// Process is stopped (for signals/debugging)
    Stopped,
}

/// Process credentials
#[derive(Debug, Clone, Copy)]
pub struct Credentials {
    pub uid: u32,
    pub gid: u32,
    pub euid: u32,
    pub egid: u32,
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            uid: 0,
            gid: 0,
            euid: 0,
            egid: 0,
        }
    }
}

/// Memory management structure (stub for now, expanded in mm module)
#[derive(Debug)]
pub struct MemoryManager {
    /// Page table base address (TTBR0_EL1) - physical address
    pub page_table: u64,
    /// Break pointer for heap (brk syscall)
    pub brk: u64,
    /// Start of heap
    pub brk_start: u64,
    /// Stack top
    pub stack_top: u64,
    /// Mmap base address (for ASLR, Phase D)
    pub mmap_base: u64,
    /// List of VMAs (will be implemented in mm/address_space.rs)
    pub vmas: Vec<Vma>,
}

impl MemoryManager {
    /// Allocate a new user address space with page table
    pub fn new_user() -> Result<Self, KernelError> {
        let page_table = crate::mm::alloc_user_page_table()?;

        // Use ASLR if enabled (Phase D)
        let (stack_top, heap_start, mmap_base) = if crate::mm::is_aslr_enabled() {
            crate::mm::randomize_address_space()
        } else {
            (crate::mm::USER_STACK_TOP, crate::mm::USER_HEAP_START, crate::mm::USER_MMAP_BASE)
        };

        Ok(Self {
            page_table,
            brk: heap_start,
            brk_start: heap_start,
            stack_top,
            mmap_base,
            vmas: Vec::new(),
        })
    }
}

/// Virtual Memory Area
#[derive(Debug, Clone)]
pub struct Vma {
    pub start: u64,
    pub end: u64,
    pub flags: VmaFlags,
    pub offset: u64,
}

bitflags::bitflags! {
    /// VMA protection and mapping flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VmaFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC = 1 << 2;
        const SHARED = 1 << 3;
        const ANONYMOUS = 1 << 4;
        const COW = 1 << 5;  // Copy-on-Write
    }
}

/// File descriptor table
pub struct FileTable {
    /// File descriptors (Arc<File> for shared references)
    pub fds: Vec<Option<alloc::sync::Arc<crate::vfs::File>>>,
}

impl FileTable {
    /// Create a new empty FD table
    pub fn new() -> Self {
        Self {
            fds: vec![None; 256], // Start with 256 FD slots
        }
    }

    /// Allocate a new FD
    pub fn alloc_fd(&mut self, file: alloc::sync::Arc<crate::vfs::File>) -> Result<i32, Errno> {
        for (i, slot) in self.fds.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(file);
                return Ok(i as i32);
            }
        }
        Err(Errno::EMFILE) // Too many open files
    }

    /// Get file by FD
    pub fn get(&self, fd: i32) -> Result<alloc::sync::Arc<crate::vfs::File>, Errno> {
        if fd < 0 || fd as usize >= self.fds.len() {
            return Err(Errno::EBADF);
        }
        self.fds[fd as usize].clone().ok_or(Errno::EBADF)
    }

    /// Close an FD
    pub fn close(&mut self, fd: i32) -> Result<(), Errno> {
        if fd < 0 || fd as usize >= self.fds.len() {
            return Err(Errno::EBADF);
        }
        if self.fds[fd as usize].is_none() {
            return Err(Errno::EBADF);
        }
        self.fds[fd as usize] = None;
        Ok(())
    }

    /// Duplicate FD (for dup/dup2)
    pub fn dup(&mut self, oldfd: i32) -> Result<i32, Errno> {
        let file = self.get(oldfd)?;
        self.alloc_fd(file)
    }
}

impl core::fmt::Debug for FileTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let open_fds: Vec<i32> = self.fds.iter().enumerate()
            .filter_map(|(i, slot)| if slot.is_some() { Some(i as i32) } else { None })
            .collect();
        f.debug_struct("FileTable")
            .field("open_fds", &open_fds)
            .finish()
    }
}

/// Main task structure
pub struct Task {
    /// Process ID
    pub pid: Pid,
    /// Parent process ID
    pub ppid: Pid,
    /// Process state
    pub state: ProcessState,
    /// Exit code (valid when state is Zombie)
    pub exit_code: i32,
    /// Memory manager
    pub mm: MemoryManager,
    /// File descriptor table
    pub files: FileTable,
    /// Credentials
    pub cred: Credentials,
    /// Saved trap frame (for context switching)
    pub trap_frame: TrapFrame,
    /// CPU context (callee-saved registers + SP)
    pub cpu_context: crate::arch::CpuContext,
    /// Kernel stack base (physical address) - 16KB
    pub kstack: u64,
    /// Process name
    pub name: String,
    /// Children PIDs
    pub children: Vec<Pid>,
    /// Signal queue (pending signals and handlers)
    pub signals: crate::process::signal::SignalQueue,
    /// Current working directory (Phase A2)
    pub cwd: String,
}

impl Task {
    /// Allocate kernel stack for task (16KB = 4 pages)
    fn alloc_kstack() -> Result<u64, KernelError> {
        // Allocate 4 contiguous pages (16KB)
        crate::mm::alloc_pages(2).ok_or(KernelError::OutOfMemory)
    }
}

impl Task {
    /// Create a new task (for PID 1 / init)
    pub fn new_init() -> Self {
        let mm = MemoryManager::new_user().expect("Failed to allocate page table for init");
        let kstack = Self::alloc_kstack().expect("Failed to allocate kernel stack for init");

        Self {
            pid: 1,
            ppid: 0,
            state: ProcessState::Running,
            exit_code: 0,
            mm,
            files: FileTable::new(),
            cred: Credentials::default(),
            trap_frame: TrapFrame::default(),
            cpu_context: crate::arch::CpuContext::new(),
            kstack,
            name: String::from("init"),
            children: Vec::new(),
            signals: crate::process::signal::SignalQueue::new(),
            cwd: String::from("/"), // Start at root
        }
    }

    /// Create a new task as a fork of another
    pub fn fork_from(parent: &Task, child_pid: Pid) -> Self {
        // Allocate new page table and kernel stack for child
        let mm = MemoryManager::new_user().expect("Failed to allocate page table for child");
        let kstack = Self::alloc_kstack().expect("Failed to allocate kernel stack for child");

        // Copy parent's address space layout
        let mut child_mm = mm;
        child_mm.brk = parent.mm.brk;
        child_mm.brk_start = parent.mm.brk_start;
        child_mm.stack_top = parent.mm.stack_top;
        child_mm.mmap_base = parent.mm.mmap_base;
        child_mm.vmas = parent.mm.vmas.clone();

        // Copy parent's page tables with COW (if parent has a page table)
        if parent.mm.page_table != 0 {
            let parent_pt = parent.mm.page_table as *mut crate::mm::paging::PageTable;
            let child_pt = child_mm.page_table as *mut crate::mm::paging::PageTable;

            // Copy page tables for each VMA
            for vma in &parent.mm.vmas {
                let is_writable = vma.flags.contains(VmaFlags::WRITE);
                let _ = crate::mm::paging::copy_page_table_for_fork(
                    parent_pt,
                    child_pt,
                    vma.start,
                    vma.end,
                    is_writable,
                );
            }

            // Flush TLB to ensure COW permissions take effect
            crate::mm::paging::flush_tlb_all();
        }

        // Child gets same trap frame but x0 (return value) will be set to 0 in fork syscall
        let mut child_tf = parent.trap_frame;
        child_tf.x0 = 0; // Child returns 0 from fork

        Self {
            pid: child_pid,
            ppid: parent.pid,
            state: ProcessState::Running,
            exit_code: 0,
            mm: child_mm,
            files: FileTable {
                fds: parent.files.fds.clone(),
            },
            cred: parent.cred,
            trap_frame: child_tf,
            cpu_context: crate::arch::CpuContext::new(), // Will be initialized in scheduler
            kstack,
            name: parent.name.clone(),
            children: Vec::new(),
            signals: parent.signals.clone(), // Inherit signal handlers from parent
            cwd: parent.cwd.clone(), // Inherit cwd from parent
        }
    }

    /// Mark task as zombie and set exit code
    pub fn exit(&mut self, code: i32) {
        self.state = ProcessState::Zombie;
        self.exit_code = code;
    }

    /// Check if task is a zombie
    pub fn is_zombie(&self) -> bool {
        self.state == ProcessState::Zombie
    }
}

impl core::fmt::Debug for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Task")
            .field("pid", &self.pid)
            .field("ppid", &self.ppid)
            .field("state", &self.state)
            .field("name", &self.name)
            .finish()
    }
}
