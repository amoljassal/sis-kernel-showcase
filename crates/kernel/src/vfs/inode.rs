/// Inode - represents a file system object
///
/// Core VFS abstraction for files, directories, and device nodes.

use crate::lib::error::Errno;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::RwLock;

/// Inode number type
pub type Ino = u64;

/// Global inode number allocator
static NEXT_INO: AtomicU64 = AtomicU64::new(2); // 1 is root

/// Allocate a new inode number
pub fn alloc_ino() -> Ino {
    NEXT_INO.fetch_add(1, Ordering::SeqCst)
}

/// Inode type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    Regular,
    Directory,
    CharDevice,
    BlockDevice,
    Symlink,
}

impl InodeType {
    /// Convert to mode bits
    pub fn to_mode_bits(&self) -> u32 {
        match self {
            InodeType::Regular => crate::vfs::S_IFREG,
            InodeType::Directory => crate::vfs::S_IFDIR,
            InodeType::CharDevice => crate::vfs::S_IFCHR,
            InodeType::BlockDevice => crate::vfs::S_IFBLK,
            InodeType::Symlink => 0o120000,
        }
    }
}

/// Inode metadata
pub struct InodeMeta {
    pub ino: Ino,
    pub itype: InodeType,
    pub mode: u32,      // Permission bits
    pub uid: u32,
    pub gid: u32,
    pub nlink: u32,     // Number of hard links
    pub size: u64,
    pub atime: u64,     // Access time
    pub mtime: u64,     // Modification time
    pub ctime: u64,     // Change time
}

impl InodeMeta {
    pub fn new(itype: InodeType, mode: u32) -> Self {
        Self {
            ino: alloc_ino(),
            itype,
            mode: itype.to_mode_bits() | (mode & 0o777),
            uid: 0,
            gid: 0,
            nlink: if matches!(itype, InodeType::Directory) { 2 } else { 1 },
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
}

/// Directory entry returned by readdir
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub ino: Ino,
    pub name: String,
    pub itype: InodeType,
}

/// Inode operations trait
pub trait InodeOps: Send + Sync {
    /// Lookup a child by name (for directories)
    fn lookup(&self, name: &str) -> Result<Arc<Inode>, Errno>;

    /// Create a new file/directory (for directories)
    fn create(&self, name: &str, mode: u32) -> Result<Arc<Inode>, Errno>;

    /// Read from inode
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Errno>;

    /// Write to inode
    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize, Errno>;

    /// Read directory entries
    fn readdir(&self) -> Result<Vec<DirEntry>, Errno>;

    /// Get attributes
    fn getattr(&self) -> Result<InodeMeta, Errno>;

    /// Truncate to size
    fn truncate(&self, size: u64) -> Result<(), Errno> {
        let _ = size;
        Err(Errno::ENOSYS)
    }
    /// Unlink (remove) a child entry (for directories)
    fn unlink(&self, name: &str) -> Result<(), Errno> {
        let _ = name;
        Err(Errno::ENOSYS)
    }
}

/// Inode structure
pub struct Inode {
    pub meta: RwLock<InodeMeta>,
    pub ops: &'static dyn InodeOps,
}

impl Inode {
    /// Create a new inode
    pub fn new(itype: InodeType, mode: u32, ops: &'static dyn InodeOps) -> Self {
        Self {
            meta: RwLock::new(InodeMeta::new(itype, mode)),
            ops,
        }
    }

    /// Lookup child (for directories)
    pub fn lookup(&self, name: &str) -> Result<Arc<Inode>, Errno> {
        self.ops.lookup(name)
    }

    /// Create child (for directories)
    pub fn create(&self, name: &str, mode: u32) -> Result<Arc<Inode>, Errno> {
        self.ops.create(name, mode)
    }

    /// Read from inode
    pub fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Errno> {
        self.ops.read(offset, buf)
    }

    /// Write to inode
    pub fn write(&self, offset: u64, buf: &[u8]) -> Result<usize, Errno> {
        self.ops.write(offset, buf)
    }

    /// Read directory entries
    pub fn readdir(&self) -> Result<Vec<DirEntry>, Errno> {
        self.ops.readdir()
    }

    /// Get metadata
    pub fn getattr(&self) -> Result<InodeMeta, Errno> {
        self.ops.getattr()
    }

    /// Get inode number
    pub fn ino(&self) -> Ino {
        self.meta.read().ino
    }

    /// Get inode type
    pub fn itype(&self) -> InodeType {
        self.meta.read().itype
    }

    /// Get size
    pub fn size(&self) -> u64 {
        self.meta.read().size
    }

    /// Set size
    pub fn set_size(&self, size: u64) {
        self.meta.write().size = size;
    }

    /// Is directory?
    pub fn is_dir(&self) -> bool {
        self.itype() == InodeType::Directory
    }

    /// Unlink a child (for directories)
    pub fn unlink(&self, name: &str) -> Result<(), Errno> {
        self.ops.unlink(name)
    }
}

impl core::fmt::Debug for Inode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let meta = self.meta.read();
        f.debug_struct("Inode")
            .field("ino", &meta.ino)
            .field("type", &meta.itype)
            .field("mode", &format_args!("{:#o}", meta.mode))
            .field("size", &meta.size)
            .finish()
    }
}
