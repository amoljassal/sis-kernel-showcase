/// VFS adapter for ext4 (write + journaling)
///
/// Wraps `crate::fs::ext4::Ext4FileSystem` to expose VFS `InodeOps`.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::block::BlockDevice;
use crate::fs::ext4::{Ext4FileSystem, Ext4Inode};
use crate::lib::error::{Errno, Result};
use super::{DirEntry, FileSystem, Inode, InodeMeta, InodeOps, InodeType};

const EXT4_ROOT_INO: u32 = 2;

pub struct Ext4InodeOps {
    fs: Arc<Ext4FileSystem>,
    ino: u32,
}

impl Ext4InodeOps {
    fn new(fs: Arc<Ext4FileSystem>, ino: u32) -> Self {
        Self { fs, ino }
    }

    fn inode_type(mode: u16) -> InodeType {
        match mode & 0xF000 {
            0x4000 => InodeType::Directory,
            0x8000 => InodeType::Regular,
            0x2000 => InodeType::CharDevice,
            0x6000 => InodeType::BlockDevice,
            0xA000 => InodeType::Symlink,
            _ => InodeType::Regular,
        }
    }
}

impl InodeOps for Ext4InodeOps {
    fn lookup(&self, name: &str) -> Result<Arc<Inode>> {
        let child = self.fs.find_dir_entry(self.ino, name)?;
        let c = self.fs.read_inode(child)?;
        let itype = Self::inode_type(c.i_mode);
        let ops: &'static dyn InodeOps = Box::leak(Box::new(Ext4InodeOps::new(self.fs.clone(), child)));
        Ok(Arc::new(Inode::new(itype, c.i_mode as u32, ops)))
    }

    fn create(&self, name: &str, mode: u32) -> Result<Arc<Inode>> {
        let ino = self.fs.create_file(self.ino, name, mode)?;
        let meta = self.fs.read_inode(ino)?;
        let itype = Self::inode_type(meta.i_mode);
        let ops: &'static dyn InodeOps = Box::leak(Box::new(Ext4InodeOps::new(self.fs.clone(), ino)));
        Ok(Arc::new(Inode::new(itype, meta.i_mode as u32, ops)))
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let inode = self.fs.read_inode(self.ino)?;
        self.fs.read_data(&inode, offset, buf)
    }

    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize> {
        self.fs.write_data_direct(self.ino, offset, buf)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        let inode = self.fs.read_inode(self.ino)?;
        let entries = self.fs.list_dir(&inode)?;
        Ok(entries
            .into_iter()
            .map(|(ino, name, ftype)| DirEntry {
                ino: ino as u64,
                name,
                itype: match ftype {
                    crate::fs::ext4::Ext4FileType::Directory => InodeType::Directory,
                    crate::fs::ext4::Ext4FileType::RegularFile => InodeType::Regular,
                    crate::fs::ext4::Ext4FileType::CharDevice => InodeType::CharDevice,
                    crate::fs::ext4::Ext4FileType::BlockDevice => InodeType::BlockDevice,
                    crate::fs::ext4::Ext4FileType::Symlink => InodeType::Symlink,
                    _ => InodeType::Regular,
                },
            })
            .collect())
    }

    fn getattr(&self) -> Result<InodeMeta> {
        let inode = self.fs.read_inode(self.ino)?;
        Ok(InodeMeta {
            ino: self.ino as u64,
            itype: Self::inode_type(inode.i_mode),
            mode: inode.i_mode as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            nlink: inode.i_links_count as u32,
            size: inode.size(),
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }

    fn unlink(&self, name: &str) -> Result<()> {
        self.fs.delete_file(self.ino, name)
    }

    fn truncate(&self, size: u64) -> Result<()> {
        self.fs.truncate_inode(self.ino, size)
    }
}

/// Mount an ext4 filesystem on a block device, returning the root VFS inode.
pub fn mount_ext4(device: Arc<BlockDevice>) -> Result<Arc<Inode>> {
    let fs = Ext4FileSystem::mount(device)?;
    // Only support 1024-byte block size for now to match block layer addressing
    if fs.block_size_bytes() != 1024 {
        return Err(Errno::ENOTSUP);
    }

    let root = fs.read_inode(EXT4_ROOT_INO)?;
    let itype = Ext4InodeOps::inode_type(root.i_mode);
    let ops: &'static dyn InodeOps = Box::leak(Box::new(Ext4InodeOps::new(fs.clone(), EXT4_ROOT_INO)));
    Ok(Arc::new(Inode::new(itype, root.i_mode as u32, ops)))
}
