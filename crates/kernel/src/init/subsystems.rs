//! Phase 3: Core subsystems initialization
//!
//! This phase initializes the core kernel subsystems required for process management and
//! file system operations:
//! - Process table
//! - Scheduler (single-core and SMP)
//! - VFS (Virtual File System)
//! - tmpfs root mount
//! - devfs at /dev
//! - procfs at /proc
//! - Page cache

use super::{InitError, InitResult};

/// Initialize all core subsystems in the correct dependency order
///
/// # Safety
/// Must be called after memory management initialization (Phase 2)
/// Must be called before device drivers (Phase 4)
pub unsafe fn init_core_subsystems() -> InitResult<()> {
    // Initialize process table
    init_process_table()?;

    // Initialize scheduler
    init_scheduler()?;

    // Initialize VFS and mount file systems
    init_vfs()?;

    // Initialize page cache
    init_page_cache()?;

    Ok(())
}

/// Initialize process table for task management
unsafe fn init_process_table() -> InitResult<()> {
    crate::process::init_process_table();
    Ok(())
}

/// Initialize scheduler for both single-core and SMP
unsafe fn init_scheduler() -> InitResult<()> {
    crate::process::scheduler::init();
    crate::process::scheduler_smp::init();
    Ok(())
}

/// Initialize VFS and mount file systems
unsafe fn init_vfs() -> InitResult<()> {
    // Initialize VFS core
    crate::vfs::init().map_err(|_| InitError::VfsFailed)?;

    // Mount tmpfs at /
    let root = crate::vfs::tmpfs::mount_tmpfs()
        .map_err(|_| InitError::MountFailed)?;
    crate::vfs::set_root(root.clone());

    // Optionally unpack embedded initramfs (models for integration)
    #[cfg(all(feature = "initramfs-models", have_initramfs_models))]
    {
        if let Err(e) = crate::initramfs::unpack_initramfs(crate::embedded_models_initramfs::data) {
            crate::warn!("initramfs: unpack failed: {:?}", e);
        }
    }

    // Mount devfs at /dev
    let dev_inode = crate::vfs::devfs::mount_devfs()
        .map_err(|_| InitError::MountFailed)?;
    root.create("dev", crate::vfs::S_IFDIR | 0o755)
        .map_err(|_| InitError::MountFailed)?;
    crate::vfs::set_root(root.clone());

    // Mount procfs at /proc
    let proc_inode = crate::vfs::mount_procfs()
        .map_err(|_| InitError::MountFailed)?;
    root.create("proc", crate::vfs::S_IFDIR | 0o555)
        .map_err(|_| InitError::MountFailed)?;
    crate::vfs::set_root(root.clone());

    // Create /tmp directory for temporary files (needed by AgentSys tests)
    root.create("tmp", crate::vfs::S_IFDIR | 0o777)
        .map_err(|_| InitError::MountFailed)?;

    Ok(())
}

/// Initialize page cache for file system caching
unsafe fn init_page_cache() -> InitResult<()> {
    // Cache up to 1024 blocks (512KB)
    crate::mm::init_page_cache(1024);
    Ok(())
}
