/// PID allocation and process table
///
/// Manages the allocation and deallocation of PIDs and maintains
/// a global table of all tasks in the system.

use super::task::{Pid, Task};
use crate::lib::error::{KernelError, Errno};
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

/// Maximum number of processes
const MAX_PIDS: usize = 32768;

/// Global PID counter
static NEXT_PID: AtomicU32 = AtomicU32::new(2); // Start at 2 (PID 1 is init)

/// Process table entry
enum PidEntry {
    Free,
    Used(Box<Task>),
}

/// Global process table
pub struct PidTable {
    entries: Vec<PidEntry>,
}

impl PidTable {
    /// Create a new empty PID table
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(MAX_PIDS);
        // Initialize with Free entries
        for _ in 0..MAX_PIDS {
            entries.push(PidEntry::Free);
        }
        Self { entries }
    }

    /// Allocate a new PID and return it
    pub fn alloc_pid(&self) -> Result<Pid, KernelError> {
        let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);
        if pid >= MAX_PIDS as u32 {
            // PID space exhausted
            return Err(KernelError::OutOfMemory);
        }
        Ok(pid)
    }

    /// Insert a task into the table
    pub fn insert(&mut self, task: Task) -> Result<(), KernelError> {
        let pid = task.pid as usize;
        if pid >= MAX_PIDS {
            return Err(KernelError::InvalidArgument);
        }
        self.entries[pid] = PidEntry::Used(Box::new(task));
        Ok(())
    }

    /// Get a reference to a task by PID
    pub fn get(&self, pid: Pid) -> Option<&Task> {
        let idx = pid as usize;
        if idx >= self.entries.len() {
            return None;
        }
        match &self.entries[idx] {
            PidEntry::Used(task) => Some(task),
            PidEntry::Free => None,
        }
    }

    /// Get a mutable reference to a task by PID
    pub fn get_mut(&mut self, pid: Pid) -> Option<&mut Task> {
        let idx = pid as usize;
        if idx >= self.entries.len() {
            return None;
        }
        match &mut self.entries[idx] {
            PidEntry::Used(task) => Some(task),
            PidEntry::Free => None,
        }
    }

    /// Remove and return a task from the table
    pub fn remove(&mut self, pid: Pid) -> Option<Task> {
        let idx = pid as usize;
        if idx >= self.entries.len() {
            return None;
        }
        let entry = core::mem::replace(&mut self.entries[idx], PidEntry::Free);
        match entry {
            PidEntry::Used(task) => Some(*task),
            PidEntry::Free => None,
        }
    }

    /// Free a PID (marks the entry as free but doesn't deallocate)
    pub fn free(&mut self, pid: Pid) {
        let idx = pid as usize;
        if idx < self.entries.len() {
            self.entries[idx] = PidEntry::Free;
        }
    }

    /// Find children of a given parent PID
    pub fn find_children(&self, ppid: Pid) -> Vec<Pid> {
        let mut children = Vec::new();
        for (idx, entry) in self.entries.iter().enumerate() {
            if let PidEntry::Used(task) = entry {
                if task.ppid == ppid {
                    children.push(idx as Pid);
                }
            }
        }
        children
    }

    /// Reparent all children of a dying process to init (PID 1)
    pub fn reparent_to_init(&mut self, dying_pid: Pid) {
        for entry in self.entries.iter_mut() {
            if let PidEntry::Used(task) = entry {
                if task.ppid == dying_pid {
                    task.ppid = 1; // Reparent to init
                }
            }
        }
    }

    /// Count active processes
    pub fn count(&self) -> usize {
        self.entries.iter().filter(|e| matches!(e, PidEntry::Used(_))).count()
    }
}

/// Global process table (protected by mutex)
static PROCESS_TABLE: Mutex<Option<PidTable>> = Mutex::new(None);

/// Initialize the process table
pub fn init_process_table() {
    let mut table = PROCESS_TABLE.lock();
    *table = Some(PidTable::new());
    crate::info!("Process table initialized (max {} PIDs)", MAX_PIDS);
}

/// Get a reference to the process table
pub fn get_process_table() -> spin::MutexGuard<'static, Option<PidTable>> {
    PROCESS_TABLE.lock()
}

/// Allocate a new PID
pub fn alloc_pid() -> Result<Pid, KernelError> {
    let table = PROCESS_TABLE.lock();
    if let Some(ref tbl) = *table {
        tbl.alloc_pid()
    } else {
        Err(KernelError::NotInitialized)
    }
}

/// Insert a task into the global table
pub fn insert_task(task: Task) -> Result<(), KernelError> {
    let mut table = PROCESS_TABLE.lock();
    if let Some(ref mut tbl) = *table {
        tbl.insert(task)
    } else {
        Err(KernelError::NotInitialized)
    }
}

/// Get the current task (stub - will use per-CPU in future)
pub fn get_current_task() -> Option<Pid> {
    // For now, return PID 1 (init) as a stub
    Some(1)
}
