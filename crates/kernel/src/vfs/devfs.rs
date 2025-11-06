/// devfs - device filesystem for /dev
///
/// Character device nodes with custom FileOps.

use super::{Inode, InodeType, InodeOps, Ino, DirEntry, alloc_ino, S_IFCHR, S_IFDIR};
use super::file::FileOps;
use crate::lib::error::Errno;
use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::RwLock;

/// devfs root directory
pub struct DevfsRoot {
    meta: RwLock<DevfsRootMeta>,
}

struct DevfsRootMeta {
    ino: Ino,
    children: BTreeMap<String, Arc<Inode>>,
}

impl DevfsRoot {
    fn new() -> Self {
        Self {
            meta: RwLock::new(DevfsRootMeta {
                ino: alloc_ino(),
                children: BTreeMap::new(),
            }),
        }
    }

    /// Add a character device node
    pub fn add_char_device(&self, name: &str, fops: &'static dyn FileOps, mode: u32) -> Result<(), Errno> {
        let mut meta = self.meta.write();

        // Check if already exists
        if meta.children.contains_key(name) {
            return Err(Errno::EEXIST);
        }

        // Create character device inode
        let dev_node = DevfsCharDev::new(fops, mode);
        let dev_ops: &'static DevfsCharDev = Box::leak(Box::new(dev_node));

        let inode = Arc::new(Inode::new(
            InodeType::CharDevice,
            mode,
            dev_ops as &'static dyn InodeOps,
        ));

        meta.children.insert(name.into(), inode);

        crate::debug!("devfs: added char device '{}'", name);

        Ok(())
    }

    /// Add a subdirectory
    pub fn add_directory(&self, name: &str, inode: Arc<Inode>) -> Result<(), Errno> {
        let mut meta = self.meta.write();

        // Check if already exists
        if meta.children.contains_key(name) {
            return Err(Errno::EEXIST);
        }

        meta.children.insert(name.into(), inode);

        crate::debug!("devfs: added subdirectory '{}'", name);

        Ok(())
    }
}

impl InodeOps for DevfsRoot {
    fn lookup(&self, name: &str) -> Result<Arc<Inode>, Errno> {
        let meta = self.meta.read();
        meta.children.get(name).cloned().ok_or(Errno::ENOENT)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>, Errno> {
        // devfs is read-only
        Err(Errno::EROFS)
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
            ino: meta.ino,
            name: "..".into(),
            itype: InodeType::Directory,
        });

        // Add device nodes
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
        Ok(super::inode::InodeMeta {
            ino: meta.ino,
            itype: InodeType::Directory,
            mode: S_IFDIR | 0o755,
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}

/// Character device node
pub struct DevfsCharDev {
    ino: Ino,
    mode: u32,
    fops: &'static dyn FileOps,
}

impl DevfsCharDev {
    fn new(fops: &'static dyn FileOps, mode: u32) -> Self {
        Self {
            ino: alloc_ino(),
            mode: S_IFCHR | (mode & 0o777),
            fops,
        }
    }
}

impl InodeOps for DevfsCharDev {
    fn lookup(&self, _name: &str) -> Result<Arc<Inode>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Errno> {
        // For character devices, offset is typically ignored
        let _ = offset;

        // Create a temporary File for the FileOps call
        // This is a bit awkward but maintains the FileOps interface
        let file = crate::vfs::File::new_with_ops(
            Arc::new(Inode::new(InodeType::CharDevice, self.mode, self)),
            crate::vfs::OpenFlags::O_RDONLY,
            self.fops,
        );

        self.fops.read(&file, buf)
    }

    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize, Errno> {
        // For character devices, offset is typically ignored
        let _ = offset;

        let file = crate::vfs::File::new_with_ops(
            Arc::new(Inode::new(InodeType::CharDevice, self.mode, self)),
            crate::vfs::OpenFlags::O_WRONLY,
            self.fops,
        );

        self.fops.write(&file, buf)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn getattr(&self) -> Result<super::inode::InodeMeta, Errno> {
        Ok(super::inode::InodeMeta {
            ino: self.ino,
            itype: InodeType::CharDevice,
            mode: self.mode,
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}

/// Mount devfs and populate with standard devices
pub fn mount_devfs() -> Result<Arc<Inode>, Errno> {
    let root = DevfsRoot::new();

    // Add standard character devices
    root.add_char_device("console", &crate::drivers::char::console::CONSOLE_OPS, 0o600)?;
    root.add_char_device("tty", &crate::drivers::char::console::CONSOLE_OPS, 0o666)?; // Alias to console
    root.add_char_device("null", &crate::drivers::char::console::NULL_OPS, 0o666)?;
    root.add_char_device("zero", &crate::drivers::char::console::ZERO_OPS, 0o666)?;
    root.add_char_device("random", &crate::drivers::char::console::RANDOM_OPS, 0o444)?;
    root.add_char_device("urandom", &crate::drivers::char::console::RANDOM_OPS, 0o444)?;

    // Add PTY master multiplexer (Phase A2)
    root.add_char_device("ptmx", &super::ptmx::PTMX_OPS, 0o666)?;

    // Add /dev/pts directory for PTY slaves (Phase A2)
    let pts_inode = super::ptsfs::mount_ptsfs()?;
    root.add_directory("pts", pts_inode)?;

    let root_ops: &'static DevfsRoot = Box::leak(Box::new(root));

    let root_inode = Arc::new(Inode::new(
        InodeType::Directory,
        0o755,
        root_ops as &'static dyn InodeOps,
    ));

    crate::info!("devfs: mounted at /dev with console, tty, null, zero, random, urandom, ptmx, pts/");

    Ok(root_inode)
}
