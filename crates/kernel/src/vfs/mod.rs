/// Virtual File System (VFS) layer
///
/// Phase A1 minimal implementation supporting:
/// - tmpfs (in-memory filesystem)
/// - devfs (device nodes)
/// - procfs (process information)

pub mod inode;
pub mod file;
pub mod mount;

pub use inode::{Inode, InodeType, InodeOps};
pub use file::{File, FileOps, OpenFlags};
pub use mount::{Mount, MountTable, init_vfs, mount, get_root};

use crate::lib::error::Errno;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;

/// File mode bits
pub const S_IFMT: u32 = 0o170000;   // File type mask
pub const S_IFREG: u32 = 0o100000;  // Regular file
pub const S_IFDIR: u32 = 0o040000;  // Directory
pub const S_IFCHR: u32 = 0o020000;  // Character device
pub const S_IRUSR: u32 = 0o000400;  // Owner read
pub const S_IWUSR: u32 = 0o000200;  // Owner write
pub const S_IXUSR: u32 = 0o000100;  // Owner execute

/// Open a file by path
pub fn open(path: &str, flags: OpenFlags) -> Result<Arc<File>, Errno> {
    // Get root inode
    let root = get_root().ok_or(Errno::ENOENT)?;

    // For now, simple path resolution (absolute paths only)
    let inode = if path == "/" {
        root
    } else {
        // Walk path components
        path_walk(root, path)?
    };

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
        // Look up component in current directory
        current = current.lookup(component)?;
    }

    Ok(current)
}

/// Create a new regular file
pub fn create(path: &str, mode: u32) -> Result<Arc<Inode>, Errno> {
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
    parent.create(name, mode)
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
