/// Virtual File System (VFS) layer
///
/// Phase A1 minimal implementation supporting:
/// - tmpfs (in-memory filesystem)
/// - devfs (device nodes)
/// - procfs (process information)
/// Phase A2 additions:
/// - ptmx (/dev/ptmx for PTY allocation)
/// - ptsfs (/dev/pts for PTY slave devices)
/// Phase B additions:
/// - ext2 (Second Extended Filesystem)
/// Phase F:
/// - ext4 with journaling (see crate::fs module for JBD2 and ext4)

pub mod inode;
pub mod file;
pub mod mount;
pub mod tmpfs;
pub mod devfs;
pub mod pipe;
pub mod procfs;
pub mod ptmx;
pub mod ptsfs;
pub mod ext2;

pub use inode::{Inode, InodeType, InodeOps, Ino, DirEntry, alloc_ino, InodeMeta};
pub use file::{File, FileOps, OpenFlags, PipeEnd, PtyEnd};
pub use mount::{Mount, init_vfs, mount, get_root, get_mounts, set_root, path_lookup};
pub use pipe::{create_pipe, PipeReader, PipeWriter};
pub use procfs::mount_procfs;
pub use ptmx::open_ptmx;
pub use ptsfs::mount_ptsfs;
pub use ext2::mount_ext2;

/// Initialize VFS subsystem (Phase A)
/// Wrapper for init_vfs() that returns Result for error handling
pub fn init() -> Result<(), Errno> {
    init_vfs();
    Ok(())
}

/// Filesystem trait - marker trait for filesystem implementations (Phase B)
///
/// Implemented by specific filesystems like Ext2FileSystem, TmpFS, etc.
/// Used to identify filesystem-specific structures and provide common interface.
pub trait FileSystem: Send + Sync {
    /// Get filesystem name
    fn name(&self) -> &str;

    /// Get root inode of this filesystem
    fn root_inode(&self) -> Result<Arc<Inode>, Errno>;
}

use crate::lib::error::Errno;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;

/// File mode bits
pub const S_IFMT: u32 = 0o170000;   // File type mask
pub const S_IFREG: u32 = 0o100000;  // Regular file
pub const S_IFDIR: u32 = 0o040000;  // Directory
pub const S_IFCHR: u32 = 0o020000;  // Character device
pub const S_IFBLK: u32 = 0o060000;  // Block device
pub const S_IRUSR: u32 = 0o000400;  // Owner read
pub const S_IWUSR: u32 = 0o000200;  // Owner write
pub const S_IXUSR: u32 = 0o000100;  // Owner execute
pub const S_IRGRP: u32 = 0o000040;  // Group read
pub const S_IWGRP: u32 = 0o000020;  // Group write
pub const S_IXGRP: u32 = 0o000010;  // Group execute
pub const S_IROTH: u32 = 0o000004;  // Other read
pub const S_IWOTH: u32 = 0o000002;  // Other write
pub const S_IXOTH: u32 = 0o000001;  // Other execute

/// Open a file by path
pub fn open(path: &str, flags: OpenFlags) -> Result<Arc<File>, Errno> {
    // Get root inode
    let root = get_root().ok_or(Errno::ENOENT)?;

    // Simple path resolution (absolute paths only)
    let inode = if path == "/" {
        root
    } else {
        // Walk path components
        path_walk(root, path)?
    };

    // Handle O_DIRECTORY flag
    if flags.contains(OpenFlags::O_DIRECTORY) && !inode.is_dir() {
        return Err(Errno::ENOTDIR);
    }

    // Handle O_TRUNC flag
    if flags.contains(OpenFlags::O_TRUNC) && flags.is_writable() {
        inode.set_size(0);
    }

    // Create File object
    let file = File::new(inode, flags);
    Ok(Arc::new(file))
}

/// Walk a path and return the inode
fn path_walk(mut current: Arc<Inode>, path: &str) -> Result<Arc<Inode>, Errno> {
    if !path.starts_with('/') {
        return Err(Errno::EINVAL);
    }

    let components: Vec<&str> = path.trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();

    for component in components {
        // Check if current is a directory
        if !current.is_dir() {
            return Err(Errno::ENOTDIR);
        }

        // Handle special components
        if component == "." {
            continue;
        }
        if component == ".." {
            // For A1, just stay at current (no parent traversal yet)
            continue;
        }

        // Look up component in current directory
        current = current.lookup(component)?;
    }

    Ok(current)
}

/// Create a new file or directory
pub fn create(path: &str, mode: u32, flags: OpenFlags) -> Result<Arc<File>, Errno> {
    let root = get_root().ok_or(Errno::ENOENT)?;

    // Split path into parent and name
    let (parent_path, name) = split_path(path)?;

    // Walk to parent
    let parent = if parent_path == "/" {
        root
    } else {
        path_walk(root, parent_path)?
    };

    // Create file in parent
    let inode = parent.create(name, mode)?;

    // Handle O_TRUNC if needed
    if flags.contains(OpenFlags::O_TRUNC) {
        inode.set_size(0);
    }

    // Open the newly created file
    let file = File::new(inode, flags);
    Ok(Arc::new(file))
}

/// Split path into (parent, name)
fn split_path(path: &str) -> Result<(&str, &str), Errno> {
    if !path.starts_with('/') {
        return Err(Errno::EINVAL);
    }

    if path == "/" {
        return Err(Errno::EISDIR);
    }

    if let Some(pos) = path.rfind('/') {
        let parent = if pos == 0 { "/" } else { &path[..pos] };
        let name = &path[pos + 1..];
        if name.is_empty() {
            return Err(Errno::EINVAL);
        }
        Ok((parent, name))
    } else {
        Err(Errno::EINVAL)
    }
}

/// Create a directory
pub fn mkdir(path: &str, mode: u32) -> Result<(), Errno> {
    let root = get_root().ok_or(Errno::ENOENT)?;

    // Split path into parent and name
    let (parent_path, name) = split_path(path)?;

    // Walk to parent
    let parent = if parent_path == "/" {
        root
    } else {
        path_walk(root, parent_path)?
    };

    // Create directory in parent (mode with S_IFDIR bit set)
    parent.create(name, mode | S_IFDIR)?;
    Ok(())
}

/// Remove a directory
pub fn rmdir(path: &str) -> Result<(), Errno> {
    let root = get_root().ok_or(Errno::ENOENT)?;

    // Split path into parent and name
    let (parent_path, name) = split_path(path)?;

    // Walk to parent
    let parent = if parent_path == "/" {
        root
    } else {
        path_walk(root, parent_path)?
    };

    // Get the directory inode
    let dir_inode = parent.lookup(name)?;

    // Verify it's a directory
    if !dir_inode.is_dir() {
        return Err(Errno::ENOTDIR);
    }

    // Remove directory from parent
    parent.unlink(name)
}

/// Remove a file
pub fn unlink(path: &str) -> Result<(), Errno> {
    let root = get_root().ok_or(Errno::ENOENT)?;

    // Split path into parent and name
    let (parent_path, name) = split_path(path)?;

    // Walk to parent
    let parent = if parent_path == "/" {
        root
    } else {
        path_walk(root, parent_path)?
    };

    // Remove file from parent
    parent.unlink(name)
}
