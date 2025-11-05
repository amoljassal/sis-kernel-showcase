/// Process task structure
///
/// Represents a single process/task in the system with all necessary state
/// for scheduling, memory management, file descriptors, and credentials.

use crate::lib::error::{KernelError, Errno};
use crate::arch::TrapFrame;
use alloc::string::String;
use alloc::vec::Vec;
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
    /// Page table base address (TTBR0_EL0)
    pub page_table: u64,
    /// Break pointer for heap (brk syscall)
    pub brk: u64,
    /// Start of heap
    pub brk_start: u64,
    /// Stack top
    pub stack_top: u64,
    /// List of VMAs (will be implemented in mm/address_space.rs)
    pub vmas: Vec<Vma>,
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

/// File descriptor table (stub)
#[derive(Debug)]
pub struct FileTable {
    /// File descriptors (stub - will be expanded with VFS)
    pub fds: Vec<Option<FileDescriptor>>,
}

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub flags: u32,
    pub offset: u64,
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
    /// Process name
    pub name: String,
    /// Children PIDs
    pub children: Vec<Pid>,
}

impl Task {
    /// Create a new task (for PID 1 / init)
    pub fn new_init() -> Self {
        Self {
            pid: 1,
            ppid: 0,
            state: ProcessState::Running,
            exit_code: 0,
            mm: MemoryManager {
                page_table: 0,
                brk: 0,
                brk_start: 0,
                stack_top: 0x0000_7FFF_FFFF_F000, // User stack top
                vmas: Vec::new(),
            },
            files: FileTable {
                fds: vec![None; 256], // Start with 256 fd slots
            },
            cred: Credentials::default(),
            trap_frame: TrapFrame::default(),
            name: String::from("init"),
            children: Vec::new(),
        }
    }

    /// Create a new task as a fork of another
    pub fn fork_from(parent: &Task, child_pid: Pid) -> Self {
        Self {
            pid: child_pid,
            ppid: parent.pid,
            state: ProcessState::Running,
            exit_code: 0,
            mm: MemoryManager {
                page_table: 0, // Will be allocated during fork
                brk: parent.mm.brk,
                brk_start: parent.mm.brk_start,
                stack_top: parent.mm.stack_top,
                vmas: parent.mm.vmas.clone(), // COW will mark pages RO
            },
            files: FileTable {
                fds: parent.files.fds.clone(),
            },
            cred: parent.cred,
            trap_frame: parent.trap_frame,
            name: parent.name.clone(),
            children: Vec::new(),
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
