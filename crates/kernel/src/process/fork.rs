//! Process Fork Implementation (Phase 8 Scaffolding)
//!
//! This module provides basic fork() scaffolding for Phase 8. It implements
//! the core process duplication logic but is intentionally incomplete to serve
//! as a foundation for full fork/exec in Phase 9.
//!
//! # What's Implemented (Phase 8)
//!
//! - Page table duplication with COW
//! - Memory manager cloning
//! - PID allocation
//! - Process table insertion
//! - Basic file descriptor inheritance
//!
//! # What's NOT Implemented (deferred to Phase 9)
//!
//! - CPU context save/restore (registers, PC, SP)
//! - Signal handler duplication
//! - Complete FD table cloning
//! - Robust error handling with cleanup
//! - Return value differentiation (parent gets child PID, child gets 0)
//!
//! # Example Usage
//!
//! ```rust
//! // From syscall handler:
//! let parent_pid = current_pid();
//! let child_pid = do_fork(parent_pid)?;
//! // Returns child PID to parent
//! // (Child will see 0 when context is restored - Phase 9)
//! ```

use crate::lib::error::{Errno, KernelError};
use super::{Pid, Task, ProcessState, MemoryManager, alloc_pid, insert_task, get_process_table};
use alloc::sync::Arc;

/// Fork the current process
///
/// Creates a new child process that is a copy of the parent process.
/// The child process has a new PID and its own address space, but initially
/// shares physical pages with the parent via copy-on-write.
///
/// # Arguments
/// * `parent_pid` - PID of the parent process
///
/// # Returns
/// * `Ok(child_pid)` - PID of the newly created child process
/// * `Err` - Error if fork fails
///
/// # Phase 8 Limitations
///
/// This is scaffolding code that:
/// - Does NOT set up CPU context properly (child won't actually run yet)
/// - Does NOT handle return value differentiation
/// - Does NOT fully clone file descriptors
/// - Serves as foundation for Phase 9 complete implementation
///
/// # Implementation Notes
///
/// The fork process:
/// 1. Allocate new PID for child
/// 2. Duplicate parent's memory manager (page tables + VMAs)
/// 3. Set up COW for both parent and child
/// 4. Clone other process state (credentials, etc.)
/// 5. Insert child into process table
/// 6. Return child PID to parent
pub fn do_fork(parent_pid: Pid) -> Result<Pid, Errno> {
    crate::debug!("do_fork: forking process {}", parent_pid);

    // Get parent process from process table
    let mut table = get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let parent = table.get_mut(parent_pid).ok_or(Errno::ESRCH)?;

    // Allocate PID for child
    let child_pid = alloc_pid().map_err(|e| {
        crate::error!("do_fork: failed to allocate child PID: {:?}", e);
        Errno::EAGAIN
    })?;

    crate::debug!("do_fork: allocated child PID {}", child_pid);

    // Clone memory manager (page tables + VMAs)
    let child_mm = clone_memory_manager(&mut parent.mm).map_err(|e| {
        crate::error!("do_fork: failed to clone memory manager: {:?}", e);
        // TODO: Free child PID on error
        Errno::ENOMEM
    })?;

    crate::debug!("do_fork: cloned memory manager (child PT={:#x})", child_mm.page_table);

    // Clone file descriptor table (shallow copy for MVP)
    let child_files = clone_file_table(&parent.files);

    // Create child task
    let child = Task {
        pid: child_pid,
        ppid: parent_pid,
        state: ProcessState::Ready,
        exit_code: 0,
        mm: child_mm,
        files: child_files,
        cred: parent.cred,
        // TODO Phase 9: Set up child to return 0, parent to return child_pid
        trap_frame: parent.trap_frame.clone(),
        cpu_context: parent.cpu_context.clone(),
        kstack: parent.kstack, // TODO Phase 9: Allocate separate kernel stack
        name: parent.name.clone(),
        children: alloc::vec::Vec::new(),
        signals: crate::process::signal::SignalQueue::new(),
        cwd: parent.cwd.clone(),
    };

    // Insert child into process table
    insert_task(child).map_err(|e| {
        crate::error!("do_fork: failed to insert child task: {:?}", e);
        // TODO: Clean up allocated resources
        Errno::EAGAIN
    })?;

    crate::info!("do_fork: created child process {} from parent {}", child_pid, parent_pid);

    // TODO Phase 9: Set up child's CPU context to return 0
    // TODO Phase 9: Parent continues here and returns child_pid
    // TODO Phase 9: Child resumes at same point but returns 0

    Ok(child_pid)
}

/// Clone memory manager for fork
///
/// Duplicates the parent's page tables and VMAs, setting up copy-on-write
/// for all writable pages.
///
/// # Arguments
/// * `parent_mm` - Parent's memory manager
///
/// # Returns
/// * New memory manager for child with duplicated page tables
fn clone_memory_manager(parent_mm: &mut MemoryManager) -> Result<MemoryManager, KernelError> {
    // Duplicate page table with COW setup
    let child_pt = crate::mm::clone_page_table_with_cow(parent_mm)?;

    // Clone VMAs (deep copy of VMA list)
    let child_vmas = parent_mm.vmas.clone();

    // Create child's memory manager
    Ok(MemoryManager {
        page_table: child_pt,
        brk: parent_mm.brk,
        brk_start: parent_mm.brk_start,
        stack_top: parent_mm.stack_top,
        mmap_base: parent_mm.mmap_base,
        vmas: child_vmas,
    })
}

/// Clone file descriptor table
///
/// For Phase 8, this does a shallow clone (shares File objects via Arc).
/// Phase 9 will implement proper FD duplication with separate file pointers.
///
/// # Arguments
/// * `parent_files` - Parent's file table
///
/// # Returns
/// * Cloned file table for child
fn clone_file_table(parent_files: &super::task::FileTable) -> super::task::FileTable {
    use super::task::FileTable;

    // For Phase 8: shallow copy
    // Each Arc<File> is cloned, so parent and child share file objects
    // This means they share file offsets (not POSIX-compliant but OK for MVP)
    let mut child_files = FileTable::new();

    for (i, fd) in parent_files.fds.iter().enumerate() {
        if let Some(file) = fd {
            child_files.fds[i] = Some(file.clone());
        }
    }

    child_files
}

/// Stub for exec (Phase 9)
///
/// Placeholder for process replacement via exec().
/// Will be fully implemented in Phase 9 with ELF loader integration.
pub fn do_exec(_pid: Pid, _path: &str, _args: &[&str]) -> Result<(), Errno> {
    crate::warn!("do_exec: not implemented (Phase 9)");
    Err(Errno::ENOSYS)
}

/// Get fork statistics (for debugging)
#[derive(Debug, Clone, Copy)]
pub struct ForkStats {
    pub total_forks: usize,
    pub failed_forks: usize,
    pub active_children: usize,
}

use core::sync::atomic::{AtomicUsize, Ordering};

static TOTAL_FORKS: AtomicUsize = AtomicUsize::new(0);
static FAILED_FORKS: AtomicUsize = AtomicUsize::new(0);

/// Increment fork counter (called on successful fork)
pub fn record_fork_success() {
    TOTAL_FORKS.fetch_add(1, Ordering::Relaxed);
}

/// Increment failed fork counter
pub fn record_fork_failure() {
    FAILED_FORKS.fetch_add(1, Ordering::Relaxed);
}

/// Get fork statistics
pub fn get_fork_stats() -> ForkStats {
    ForkStats {
        total_forks: TOTAL_FORKS.load(Ordering::Relaxed),
        failed_forks: FAILED_FORKS.load(Ordering::Relaxed),
        active_children: 0, // TODO: Calculate from process table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_stats() {
        record_fork_success();
        record_fork_success();
        record_fork_failure();

        let stats = get_fork_stats();
        assert!(stats.total_forks >= 2);
        assert!(stats.failed_forks >= 1);
    }
}
