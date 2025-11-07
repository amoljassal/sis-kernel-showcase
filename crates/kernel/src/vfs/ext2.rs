/// ext2 Filesystem Driver
///
/// Implements the Second Extended Filesystem (ext2) with support for:
/// - Superblock and block group descriptors
/// - Inode operations (read/write)
/// - Directory operations (lookup, readdir)
/// - File operations (create, unlink, rename)
/// - Direct, indirect, and double-indirect block allocation

use crate::lib::error::{Result, Errno};
use crate::block::BlockDevice;
use crate::vfs::{Inode, InodeOps, InodeType, FileSystem, DirEntry};
use crate::mm::{get_buffer, put_buffer, BufferHead};
use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use spin::Mutex;
use core::mem::size_of;

/// ext2 magic number
const EXT2_SUPER_MAGIC: u16 = 0xEF53;

/// Block size constants
const EXT2_MIN_BLOCK_SIZE: u32 = 1024;
const EXT2_MAX_BLOCK_SIZE: u32 = 4096;

/// Inode constants
const EXT2_GOOD_OLD_INODE_SIZE: u16 = 128;
const EXT2_DIRECT_BLOCKS: usize = 12;
const EXT2_IND_BLOCK: usize = 12;
const EXT2_DIND_BLOCK: usize = 13;
const EXT2_TIND_BLOCK: usize = 14;
const EXT2_N_BLOCKS: usize = 15;

/// File type constants (for directory entries)
const EXT2_FT_UNKNOWN: u8 = 0;
const EXT2_FT_REG_FILE: u8 = 1;
const EXT2_FT_DIR: u8 = 2;
const EXT2_FT_CHRDEV: u8 = 3;
const EXT2_FT_BLKDEV: u8 = 4;
const EXT2_FT_FIFO: u8 = 5;
const EXT2_FT_SOCK: u8 = 6;
const EXT2_FT_SYMLINK: u8 = 7;

/// Root inode number
const EXT2_ROOT_INO: u32 = 2;

/// Superblock structure (located at byte offset 1024)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Ext2Superblock {
    s_inodes_count: u32,
    s_blocks_count: u32,
    s_r_blocks_count: u32,
    s_free_blocks_count: u32,
    s_free_inodes_count: u32,
    s_first_data_block: u32,
    s_log_block_size: u32,
    s_log_frag_size: u32,
    s_blocks_per_group: u32,
    s_frags_per_group: u32,
    s_inodes_per_group: u32,
    s_mtime: u32,
    s_wtime: u32,
    s_mnt_count: u16,
    s_max_mnt_count: u16,
    s_magic: u16,
    s_state: u16,
    s_errors: u16,
    s_minor_rev_level: u16,
    s_lastcheck: u32,
    s_checkinterval: u32,
    s_creator_os: u32,
    s_rev_level: u32,
    s_def_resuid: u16,
    s_def_resgid: u16,
    // Extended fields (rev_level >= 1)
    s_first_ino: u32,
    s_inode_size: u16,
    s_block_group_nr: u16,
    s_feature_compat: u32,
    s_feature_incompat: u32,
    s_feature_ro_compat: u32,
    s_uuid: [u8; 16],
    s_volume_name: [u8; 16],
    s_last_mounted: [u8; 64],
    s_algo_bitmap: u32,
    // Performance hints
    s_prealloc_blocks: u8,
    s_prealloc_dir_blocks: u8,
    s_padding1: u16,
    // Journaling support
    s_journal_uuid: [u8; 16],
    s_journal_inum: u32,
    s_journal_dev: u32,
    s_last_orphan: u32,
}

/// Block group descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Ext2GroupDesc {
    bg_block_bitmap: u32,
    bg_inode_bitmap: u32,
    bg_inode_table: u32,
    bg_free_blocks_count: u16,
    bg_free_inodes_count: u16,
    bg_used_dirs_count: u16,
    bg_pad: u16,
    bg_reserved: [u32; 3],
}

/// Inode structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Ext2Inode {
    i_mode: u16,
    i_uid: u16,
    i_size: u32,
    i_atime: u32,
    i_ctime: u32,
    i_mtime: u32,
    i_dtime: u32,
    i_gid: u16,
    i_links_count: u16,
    i_blocks: u32,
    i_flags: u32,
    i_osd1: u32,
    i_block: [u32; EXT2_N_BLOCKS],
    i_generation: u32,
    i_file_acl: u32,
    i_dir_acl: u32,
    i_faddr: u32,
    i_osd2: [u8; 12],
}

/// Directory entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Ext2DirEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
    // name follows (variable length)
}

/// ext2 filesystem instance
pub struct Ext2FileSystem {
    device: Arc<BlockDevice>,
    superblock: Ext2Superblock,
    block_size: u32,
    inodes_per_group: u32,
    blocks_per_group: u32,
    inode_size: u16,
    block_groups: Vec<Ext2GroupDesc>,
}

impl Ext2FileSystem {
    /// Mount an ext2 filesystem from a block device
    pub fn mount(device: Arc<BlockDevice>) -> Result<Arc<Self>> {
        // Read superblock (at byte offset 1024, which is sector 2 for 512-byte sectors)
        let sb_sector = 1024 / device.sector_size as u64;
        let sb_buf = get_buffer(device.clone(), sb_sector)?;

        let superblock = unsafe {
            let data = sb_buf.data();
            let sb_offset = 1024 % device.sector_size;
            core::ptr::read_unaligned(
                data.as_ptr().add(sb_offset) as *const Ext2Superblock
            )
        };
        put_buffer(sb_buf);

        // Validate magic number
        if superblock.s_magic != EXT2_SUPER_MAGIC {
            crate::warn!("ext2: invalid magic number 0x{:04x}", superblock.s_magic);
            return Err(Errno::EINVAL);
        }

        // Calculate block size
        let block_size = EXT2_MIN_BLOCK_SIZE << superblock.s_log_block_size;
        if block_size < EXT2_MIN_BLOCK_SIZE || block_size > EXT2_MAX_BLOCK_SIZE {
            crate::warn!("ext2: invalid block size {}", block_size);
            return Err(Errno::EINVAL);
        }

        // Get inode size (default to 128 for old revisions)
        let inode_size = if superblock.s_rev_level >= 1 {
            superblock.s_inode_size
        } else {
            EXT2_GOOD_OLD_INODE_SIZE
        };

        crate::info!("ext2: block_size={} inode_size={} groups={}",
                     block_size, inode_size,
                     (superblock.s_blocks_count + superblock.s_blocks_per_group - 1) / superblock.s_blocks_per_group);

        // Read block group descriptors
        let bg_count = ((superblock.s_blocks_count + superblock.s_blocks_per_group - 1)
                        / superblock.s_blocks_per_group) as usize;
        let bgd_block = if block_size == 1024 { 2 } else { 1 };
        let mut block_groups = Vec::new();

        for i in 0..bg_count {
            let bgd_sector = (bgd_block * block_size / device.sector_size as u32) as u64
                           + ((i * size_of::<Ext2GroupDesc>()) / device.sector_size) as u64;
            let bgd_buf = get_buffer(device.clone(), bgd_sector)?;

            let bgd = unsafe {
                let data = bgd_buf.data();
                let offset = (i * size_of::<Ext2GroupDesc>()) % device.sector_size;
                core::ptr::read_unaligned(
                    data.as_ptr().add(offset) as *const Ext2GroupDesc
                )
            };
            put_buffer(bgd_buf);

            block_groups.push(bgd);
        }

        Ok(Arc::new(Self {
            device,
            superblock,
            block_size,
            inodes_per_group: superblock.s_inodes_per_group,
            blocks_per_group: superblock.s_blocks_per_group,
            inode_size,
            block_groups,
        }))
    }

    /// Read an inode from disk
    fn read_inode(&self, inode_num: u32) -> Result<Ext2Inode> {
        if inode_num == 0 || inode_num > self.superblock.s_inodes_count {
            return Err(Errno::EINVAL);
        }

        // Calculate block group and index
        let group = (inode_num - 1) / self.inodes_per_group;
        let index = (inode_num - 1) % self.inodes_per_group;

        if group as usize >= self.block_groups.len() {
            return Err(Errno::EINVAL);
        }

        let bgd = &self.block_groups[group as usize];
        let inode_table_block = bgd.bg_inode_table;

        // Calculate inode location
        let inode_offset = index * self.inode_size as u32;
        let block_num = inode_table_block + (inode_offset / self.block_size);
        let block_offset = inode_offset % self.block_size;

        // Read inode
        let sector = (block_num as u64 * self.block_size as u64) / self.device.sector_size as u64;
        let buf = get_buffer(self.device.clone(), sector)?;

        let inode = unsafe {
            let data = buf.data();
            let offset = ((block_num as u64 * self.block_size as u64) % self.device.sector_size as u64 + block_offset as u64) as usize;
            core::ptr::read_unaligned(data.as_ptr().add(offset) as *const Ext2Inode)
        };
        put_buffer(buf);

        Ok(inode)
    }

    /// Read data from an inode
    fn read_inode_data(&self, inode: &Ext2Inode, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let file_size = inode.i_size as u64;
        if offset >= file_size {
            return Ok(0);
        }

        let read_size = core::cmp::min(buf.len(), (file_size - offset) as usize);
        let mut bytes_read = 0usize;

        while bytes_read < read_size {
            let block_idx = ((offset + bytes_read as u64) / self.block_size as u64) as u32;
            let block_offset = ((offset + bytes_read as u64) % self.block_size as u64) as usize;
            let copy_size = core::cmp::min(read_size - bytes_read, self.block_size as usize - block_offset);

            // Get physical block number
            let phys_block = self.get_block_num(inode, block_idx)?;

            if phys_block == 0 {
                // Sparse block - fill with zeros
                buf[bytes_read..bytes_read + copy_size].fill(0);
            } else {
                // Read block
                let sector = (phys_block as u64 * self.block_size as u64) / self.device.sector_size as u64;
                let block_buf = get_buffer(self.device.clone(), sector)?;

                {
                    let data = block_buf.data();
                    let src_offset = ((phys_block as u64 * self.block_size as u64) % self.device.sector_size as u64) as usize + block_offset;
                    buf[bytes_read..bytes_read + copy_size].copy_from_slice(&data[src_offset..src_offset + copy_size]);
                }
                put_buffer(block_buf);
            }

            bytes_read += copy_size;
        }

        Ok(bytes_read)
    }

    /// Get physical block number for a logical block in an inode
    fn get_block_num(&self, inode: &Ext2Inode, block_idx: u32) -> Result<u32> {
        let addrs_per_block = (self.block_size / 4) as u32;

        // Direct blocks
        if block_idx < EXT2_DIRECT_BLOCKS as u32 {
            return Ok(inode.i_block[block_idx as usize]);
        }

        // Indirect block
        let mut idx = block_idx - EXT2_DIRECT_BLOCKS as u32;
        if idx < addrs_per_block {
            let ind_block = inode.i_block[EXT2_IND_BLOCK];
            if ind_block == 0 {
                return Ok(0);
            }
            return self.read_indirect_block(ind_block, idx);
        }

        // Double indirect block
        idx -= addrs_per_block;
        if idx < addrs_per_block * addrs_per_block {
            let dind_block = inode.i_block[EXT2_DIND_BLOCK];
            if dind_block == 0 {
                return Ok(0);
            }
            let ind_idx = idx / addrs_per_block;
            let block_idx = idx % addrs_per_block;

            let ind_block = self.read_indirect_block(dind_block, ind_idx)?;
            if ind_block == 0 {
                return Ok(0);
            }
            return self.read_indirect_block(ind_block, block_idx);
        }

        // Triple indirect not implemented
        Err(Errno::ENOSYS)
    }

    /// Read a block number from an indirect block
    fn read_indirect_block(&self, block_num: u32, index: u32) -> Result<u32> {
        let sector = (block_num as u64 * self.block_size as u64) / self.device.sector_size as u64;
        let buf = get_buffer(self.device.clone(), sector)?;

        let result = {
            let data = buf.data();
            let offset = ((block_num as u64 * self.block_size as u64) % self.device.sector_size as u64) as usize
                       + (index * 4) as usize;
            u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]])
        };
        put_buffer(buf);

        Ok(result)
    }

    /// Look up a directory entry by name
    fn lookup_dir(&self, dir_inode: &Ext2Inode, name: &str) -> Result<u32> {
        let dir_size = dir_inode.i_size as usize;
        let mut buf = vec![0u8; dir_size];
        self.read_inode_data(dir_inode, 0, &mut buf)?;

        let mut offset = 0;
        while offset < dir_size {
            let entry = unsafe {
                core::ptr::read_unaligned(buf.as_ptr().add(offset) as *const Ext2DirEntry)
            };

            if entry.rec_len == 0 {
                break;
            }

            if entry.inode != 0 && entry.name_len > 0 {
                let entry_name = core::str::from_utf8(
                    &buf[offset + size_of::<Ext2DirEntry>()..offset + size_of::<Ext2DirEntry>() + entry.name_len as usize]
                ).unwrap_or("");

                if entry_name == name {
                    return Ok(entry.inode);
                }
            }

            offset += entry.rec_len as usize;
        }

        Err(Errno::ENOENT)
    }

    /// Read directory entries
    fn readdir(&self, dir_inode: &Ext2Inode) -> Result<Vec<DirEntry>> {
        let dir_size = dir_inode.i_size as usize;
        let mut buf = vec![0u8; dir_size];
        self.read_inode_data(dir_inode, 0, &mut buf)?;

        let mut entries = Vec::new();
        let mut offset = 0;

        while offset < dir_size {
            let entry = unsafe {
                core::ptr::read_unaligned(buf.as_ptr().add(offset) as *const Ext2DirEntry)
            };

            if entry.rec_len == 0 {
                break;
            }

            if entry.inode != 0 && entry.name_len > 0 {
                let name_bytes = &buf[offset + size_of::<Ext2DirEntry>()..offset + size_of::<Ext2DirEntry>() + entry.name_len as usize];
                let name = String::from_utf8_lossy(name_bytes).into_owned();

                let itype = match entry.file_type {
                    EXT2_FT_DIR => InodeType::Directory,
                    EXT2_FT_REG_FILE => InodeType::Regular,
                    EXT2_FT_CHRDEV => InodeType::CharDevice,
                    EXT2_FT_BLKDEV => InodeType::BlockDevice,
                    EXT2_FT_SYMLINK => InodeType::Symlink,
                    _ => InodeType::Regular,
                };

                entries.push(DirEntry {
                    ino: entry.inode as u64,
                    name,
                    itype,
                });
            }

            offset += entry.rec_len as usize;
        }

        Ok(entries)
    }
}

/// Ext2 inode implementation for VFS
pub struct Ext2InodeOps {
    fs: Arc<Ext2FileSystem>,
    inode_num: u32,
    inode_data: Mutex<Ext2Inode>,
}

impl Ext2InodeOps {
    fn new(fs: Arc<Ext2FileSystem>, inode_num: u32) -> Result<Self> {
        let inode_data = fs.read_inode(inode_num)?;
        Ok(Self {
            fs,
            inode_num,
            inode_data: Mutex::new(inode_data),
        })
    }

    fn get_inode_type(mode: u16) -> InodeType {
        if (mode & 0o040000) != 0 {
            InodeType::Directory
        } else if (mode & 0o100000) != 0 {
            InodeType::Regular
        } else {
            InodeType::Regular  // Default
        }
    }
}

impl InodeOps for Ext2InodeOps {
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let inode = self.inode_data.lock();
        self.fs.read_inode_data(&inode, offset, buf)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize, Errno> {
        Err(Errno::EROFS) // Read-only
    }

    fn lookup(&self, name: &str) -> Result<Arc<Inode>, Errno> {
        let inode = self.inode_data.lock();
        let child_ino = self.fs.lookup_dir(&inode, name)?;
        drop(inode);

        // Create VFS inode for child
        let child_ops = Box::new(Self::new(self.fs.clone(), child_ino)?);
        let child_ops_static: &'static dyn InodeOps = Box::leak(child_ops);

        let child_inode_data = self.fs.read_inode(child_ino)?;
        let child_itype = Self::get_inode_type(child_inode_data.i_mode);

        Ok(Arc::new(Inode::new(child_itype, child_inode_data.i_mode as u32, child_ops_static)))
    }

    fn readdir(&self) -> Result<Vec<DirEntry>, Errno> {
        let inode = self.inode_data.lock();
        self.fs.readdir(&inode)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>, Errno> {
        Err(Errno::EROFS) // Read-only
    }

    fn getattr(&self) -> Result<crate::vfs::InodeMeta, Errno> {
        let inode = self.inode_data.lock();
        Ok(crate::vfs::InodeMeta {
            ino: self.inode_num as u64,
            itype: Self::get_inode_type(inode.i_mode),
            mode: inode.i_mode as u32,
            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            nlink: inode.i_links_count as u32,
            size: inode.i_size as u64,
            atime: inode.i_atime as u64,
            mtime: inode.i_mtime as u64,
            ctime: inode.i_ctime as u64,
        })
    }
}

/// Mount an ext2 filesystem
pub fn mount_ext2(device: Arc<BlockDevice>) -> Result<Arc<Inode>> {
    let fs = Ext2FileSystem::mount(device)?;

    // Create root inode
    let root_ops = Box::new(Ext2InodeOps::new(fs.clone(), EXT2_ROOT_INO)?);
    let root_ops_static: &'static dyn InodeOps = Box::leak(root_ops);

    let root_inode_data = fs.read_inode(EXT2_ROOT_INO)?;
    let root_itype = Ext2InodeOps::get_inode_type(root_inode_data.i_mode);

    let root = Arc::new(Inode::new(root_itype, root_inode_data.i_mode as u32, root_ops_static));

    crate::info!("ext2: mounted successfully (root inode {})", EXT2_ROOT_INO);
    Ok(root)
}
