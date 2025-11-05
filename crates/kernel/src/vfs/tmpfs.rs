/// tmpfs - temporary filesystem in RAM
///
/// Simple in-memory filesystem for root and /tmp.
/// Files store content in Vec<u8>, directories track children in HashMap.

use super::{Inode, InodeType, InodeOps, Ino, DirEntry, alloc_ino, S_IFDIR, S_IFREG};
use crate::lib::error::Errno;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::RwLock;

/// tmpfs directory node
pub struct TmpfsDir {
    meta: RwLock<TmpfsDirMeta>,
}

struct TmpfsDirMeta {
    ino: Ino,
    mode: u32,
    children: BTreeMap<String, Arc<Inode>>,
}

impl TmpfsDir {
    pub fn new(mode: u32) -> Self {
        Self {
            meta: RwLock::new(TmpfsDirMeta {
                ino: alloc_ino(),
                mode: S_IFDIR | (mode & 0o777),
                children: BTreeMap::new(),
            }),
        }
    }

    pub fn new_with_ino(ino: Ino, mode: u32) -> Self {
        Self {
            meta: RwLock::new(TmpfsDirMeta {
                ino,
                mode: S_IFDIR | (mode & 0o777),
                children: BTreeMap::new(),
            }),
        }
    }
}

impl InodeOps for TmpfsDir {
    fn lookup(&self, name: &str) -> Result<Arc<Inode>, Errno> {
        let meta = self.meta.read();
        meta.children.get(name).cloned().ok_or(Errno::ENOENT)
    }

    fn create(&self, name: &str, mode: u32) -> Result<Arc<Inode>, Errno> {
        let mut meta = self.meta.write();

        // Check if already exists
        if meta.children.contains_key(name) {
            return Err(Errno::EEXIST);
        }

        // Create new inode based on mode
        let inode = if (mode & S_IFDIR) != 0 {
            // Create directory
            let dir = TmpfsDir::new(mode);
            Arc::new(Inode::new(InodeType::Directory, mode, &dir as &'static dyn InodeOps))
        } else {
            // Create regular file
            let file = TmpfsFile::new(mode);
            Arc::new(Inode::new(InodeType::Regular, mode, &file as &'static dyn InodeOps))
        };

        meta.children.insert(name.into(), inode.clone());

        Ok(inode)
    }

    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize, Errno> {
        Err(Errno::EISDIR)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, Errno> {
        Err(Errno::EISDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, Errno> {
        let meta = self.meta.read();
        let mut entries = Vec::new();

        // Add . and ..
        entries.push(DirEntry {
            ino: meta.ino,
            name: ".".into(),
            itype: InodeType::Directory,
        });
        entries.push(DirEntry {
            ino: meta.ino, // For now, .. is same as .
            name: "..".into(),
            itype: InodeType::Directory,
        });

        // Add children
        for (name, inode) in meta.children.iter() {
            entries.push(DirEntry {
                ino: inode.ino(),
                name: name.clone(),
                itype: inode.itype(),
            });
        }

        Ok(entries)
    }

    fn getattr(&self) -> Result<super::inode::InodeMeta, Errno> {
        let meta = self.meta.read();
        let num_children = meta.children.len();
        Ok(super::inode::InodeMeta {
            ino: meta.ino,
            itype: InodeType::Directory,
            mode: meta.mode,
            uid: 0,
            gid: 0,
            size: (num_children * 64) as u64, // Fake size
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}

/// tmpfs regular file node
pub struct TmpfsFile {
    meta: RwLock<TmpfsFileMeta>,
}

struct TmpfsFileMeta {
    ino: Ino,
    mode: u32,
    content: Vec<u8>,
}

impl TmpfsFile {
    pub fn new(mode: u32) -> Self {
        Self {
            meta: RwLock::new(TmpfsFileMeta {
                ino: alloc_ino(),
                mode: S_IFREG | (mode & 0o777),
                content: Vec::new(),
            }),
        }
    }
}

impl InodeOps for TmpfsFile {
    fn lookup(&self, _name: &str) -> Result<Arc<Inode>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Errno> {
        let meta = self.meta.read();
        let offset = offset as usize;

        if offset >= meta.content.len() {
            return Ok(0); // EOF
        }

        let available = meta.content.len() - offset;
        let to_read = available.min(buf.len());

        buf[..to_read].copy_from_slice(&meta.content[offset..offset + to_read]);

        Ok(to_read)
    }

    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize, Errno> {
        let mut meta = self.meta.write();
        let offset = offset as usize;

        // Extend content if necessary
        if offset + buf.len() > meta.content.len() {
            meta.content.resize(offset + buf.len(), 0);
        }

        meta.content[offset..offset + buf.len()].copy_from_slice(buf);

        Ok(buf.len())
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn getattr(&self) -> Result<super::inode::InodeMeta, Errno> {
        let meta = self.meta.read();
        Ok(super::inode::InodeMeta {
            ino: meta.ino,
            itype: InodeType::Regular,
            mode: meta.mode,
            uid: 0,
            gid: 0,
            size: meta.content.len() as u64,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn truncate(&self, size: u64) -> Result<(), Errno> {
        let mut meta = self.meta.write();
        meta.content.resize(size as usize, 0);
        Ok(())
    }
}

/// Create a tmpfs mount
pub fn mount_tmpfs() -> Result<Arc<Inode>, Errno> {
    // Create root directory with inode 1
    let root = TmpfsDir::new_with_ino(1, 0o755);
    let root_ops: &'static TmpfsDir = Box::leak(Box::new(root));

    let root_inode = Arc::new(Inode::new(
        InodeType::Directory,
        0o755,
        root_ops as &'static dyn InodeOps,
    ));

    crate::info!("tmpfs: mounted at / (root inode=1)");

    Ok(root_inode)
}

/// Helper to create directory nodes for mount points
pub fn create_dir(parent: &Arc<Inode>, name: &str, mode: u32) -> Result<Arc<Inode>, Errno> {
    parent.create(name, S_IFDIR | mode)
}
