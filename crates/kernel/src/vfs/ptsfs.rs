// /dev/pts filesystem - PTY slave device nodes
// Provides access to slave PTY devices as /dev/pts/N

use super::{Inode, InodeType, InodeOps, Ino, DirEntry, alloc_ino, S_IFDIR, S_IFCHR};
use crate::lib::error::Errno;
use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// /dev/pts root directory
pub struct PtsfsRoot {
    ino: Ino,
}

impl PtsfsRoot {
    pub fn new() -> Self {
        Self {
            ino: alloc_ino(),
        }
    }
}

impl InodeOps for PtsfsRoot {
    fn lookup(&self, name: &str) -> Result<Arc<Inode>, Errno> {
        // Parse name as PTY number (e.g., "0", "1", "2", etc.)
        let pty_num = name.parse::<usize>().map_err(|_| Errno::ENOENT)?;

        // Check if PTY slave exists in registry
        let slave = super::ptmx::get_pty_slave(pty_num).ok_or(Errno::ENOENT)?;

        // Create a PTS device node
        let pts_node = PtsDevice::new(pty_num, slave);
        let pts_ops: &'static PtsDevice = Box::leak(Box::new(pts_node));

        Ok(Arc::new(Inode::new(
            InodeType::CharDevice,
            S_IFCHR | 0o620, // crw--w----
            pts_ops as &'static dyn InodeOps,
        )))
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>, Errno> {
        // /dev/pts is read-only (slaves are created via /dev/ptmx)
        Err(Errno::EROFS)
    }

    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize, Errno> {
        Err(Errno::EISDIR)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, Errno> {
        Err(Errno::EISDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, Errno> {
        let mut entries = Vec::new();

        // Add . and ..
        entries.push(DirEntry {
            ino: self.ino,
            name: ".".into(),
            itype: InodeType::Directory,
        });
        entries.push(DirEntry {
            ino: self.ino,
            name: "..".into(),
            itype: InodeType::Directory,
        });

        // Add all registered PTY slaves
        for pty_num in super::ptmx::list_pty_slaves() {
            entries.push(DirEntry {
                ino: 1000 + pty_num as u64,
                name: pty_num.to_string(),
                itype: InodeType::CharDevice,
            });
        }

        Ok(entries)
    }

    fn getattr(&self) -> Result<super::inode::InodeMeta, Errno> {
        Ok(super::inode::InodeMeta {
            ino: self.ino,
            itype: InodeType::Directory,
            mode: S_IFDIR | 0o755,
            uid: 0,
            gid: 0,
            nlink: 2,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}

/// PTY slave device node (/dev/pts/N)
struct PtsDevice {
    ino: Ino,
    pty_num: usize,
    slave: crate::drivers::char::pty::PtySlave,
}

impl PtsDevice {
    fn new(pty_num: usize, slave: crate::drivers::char::pty::PtySlave) -> Self {
        Self {
            ino: alloc_ino(),
            pty_num,
            slave,
        }
    }
}

impl InodeOps for PtsDevice {
    fn lookup(&self, _name: &str) -> Result<Arc<Inode>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn read(&self, _offset: u64, buf: &mut [u8]) -> Result<usize, Errno> {
        // Read from PTY slave
        self.slave.read(buf)
    }

    fn write(&self, _offset: u64, buf: &[u8]) -> Result<usize, Errno> {
        // Write to PTY slave
        self.slave.write(buf)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, Errno> {
        Err(Errno::ENOTDIR)
    }

    fn getattr(&self) -> Result<super::inode::InodeMeta, Errno> {
        Ok(super::inode::InodeMeta {
            ino: self.ino,
            itype: InodeType::CharDevice,
            mode: S_IFCHR | 0o620, // crw--w----
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }
}

/// Mount ptsfs at /dev/pts
pub fn mount_ptsfs() -> Result<Arc<Inode>, Errno> {
    let root = PtsfsRoot::new();
    let root_ops: &'static PtsfsRoot = Box::leak(Box::new(root));

    let root_inode = Arc::new(Inode::new(
        InodeType::Directory,
        0o755,
        root_ops as &'static dyn InodeOps,
    ));

    crate::info!("ptsfs: mounted at /dev/pts");

    Ok(root_inode)
}
