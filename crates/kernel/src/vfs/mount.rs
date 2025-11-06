/// Mount table and filesystem mounting
///
/// Manages mounted filesystems and provides root access.

use super::inode::Inode;
use crate::lib::error::Errno;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use spin::RwLock;

/// Mount point
#[derive(Clone)]
pub struct Mount {
    pub fs_type: &'static str,
    pub root: Arc<Inode>,
    pub mountpoint: String,
}

impl Mount {
    pub fn new(fs_type: &'static str, root: Arc<Inode>, mountpoint: String) -> Self {
        Self {
            fs_type,
            root,
            mountpoint,
        }
    }
}

/// Global mount table
pub struct MountTable {
    mounts: Vec<Mount>,
    root: Option<Arc<Inode>>,
}

impl MountTable {
    fn new() -> Self {
        Self {
            mounts: Vec::new(),
            root: None,
        }
    }

    /// Mount a filesystem
    pub fn mount(&mut self, mount: Mount) -> Result<(), Errno> {
        // Special case: mounting at "/" sets the root
        if mount.mountpoint == "/" {
            self.root = Some(mount.root.clone());
            crate::info!("VFS: mounted {} at {}", mount.fs_type, mount.mountpoint);
        } else {
            // For non-root mounts, just add to list
            // In a full implementation, we'd create a mountpoint inode
            crate::info!("VFS: mounted {} at {}", mount.fs_type, mount.mountpoint);
        }

        self.mounts.push(mount);
        Ok(())
    }

    /// Get root inode
    pub fn root(&self) -> Option<Arc<Inode>> {
        self.root.clone()
    }

    /// List all mounts
    pub fn list(&self) -> Vec<Mount> {
        self.mounts.clone()
    }
}

/// Global VFS state
static VFS: RwLock<Option<MountTable>> = RwLock::new(None);

/// Initialize VFS
pub fn init_vfs() {
    let mut vfs = VFS.write();
    *vfs = Some(MountTable::new());
    crate::info!("VFS initialized");
}

/// Mount a filesystem
pub fn mount(fs_type: &'static str, root: Arc<Inode>, mountpoint: &str) -> Result<(), Errno> {
    let mut vfs = VFS.write();
    if let Some(ref mut table) = *vfs {
        let mount = Mount::new(fs_type, root, mountpoint.into());
        table.mount(mount)
    } else {
        Err(Errno::ENODEV)
    }
}

/// Get root inode
pub fn get_root() -> Option<Arc<Inode>> {
    let vfs = VFS.read();
    vfs.as_ref().and_then(|t| t.root())
}

/// Get mount table for /proc/mounts
pub fn get_mounts() -> Vec<Mount> {
    let vfs = VFS.read();
    vfs.as_ref().map(|t| t.list()).unwrap_or_default()
}

/// Set root inode (used during initialization)
pub fn set_root(root: Arc<Inode>) {
    let mut vfs = VFS.write();
    if let Some(ref mut table) = *vfs {
        table.root = Some(root);
    }
}

/// Path lookup - resolve a path to an inode
///
/// Starts from the given root inode and walks the path components
/// to find the target inode.
pub fn path_lookup(root: &Arc<Inode>, path: &str) -> Result<Arc<Inode>, Errno> {
    use super::InodeType;

    // Handle empty or root path
    if path.is_empty() || path == "/" {
        return Ok(root.clone());
    }

    // Remove leading slash and split into components
    let path = path.strip_prefix('/').unwrap_or(path);
    let components: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();

    let mut current = root.clone();

    // Walk each path component
    for component in components {
        // Ensure current is a directory
        let meta = current.metadata()?;
        if meta.inode_type != InodeType::Directory {
            return Err(Errno::ENOTDIR);
        }

        // Look up the next component
        current = current.lookup(component)?;
    }

    Ok(current)
}
