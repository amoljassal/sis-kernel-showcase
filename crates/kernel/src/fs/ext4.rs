/// ext4 Filesystem Driver with Journaling - Phase F
///
/// Extends ext2 with ext4 features and JBD2 journaling for:
/// - Crash recovery via journal replay
/// - Atomic metadata operations
/// - Ordered data mode (data-before-metadata)
/// - Transaction-based writes

use crate::lib::error::{Result, Errno};
use crate::block::BlockDevice;
use crate::vfs::{Inode, InodeOps, InodeType, DirEntry};
use super::jbd2::{Journal, TransactionHandle, JBD2_MAGIC_NUMBER};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

/// ext4 magic number (same as ext2)
const EXT4_SUPER_MAGIC: u16 = 0xEF53;

/// ext4 feature flags
pub const EXT4_FEATURE_COMPAT_HAS_JOURNAL: u32 = 0x0004;
pub const EXT4_FEATURE_INCOMPAT_FILETYPE: u32 = 0x0002;
pub const EXT4_FEATURE_INCOMPAT_EXTENTS: u32 = 0x0040;
pub const EXT4_FEATURE_INCOMPAT_64BIT: u32 = 0x0080;

/// ext4 superblock (extends ext2)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4Superblock {
    // Base ext2 fields
    pub s_inodes_count: u32,
    pub s_blocks_count: u32,
    pub s_r_blocks_count: u32,
    pub s_free_blocks_count: u32,
    pub s_free_inodes_count: u32,
    pub s_first_data_block: u32,
    pub s_log_block_size: u32,
    pub s_log_cluster_size: u32,
    pub s_blocks_per_group: u32,
    pub s_clusters_per_group: u32,
    pub s_inodes_per_group: u32,
    pub s_mtime: u32,
    pub s_wtime: u32,
    pub s_mnt_count: u16,
    pub s_max_mnt_count: u16,
    pub s_magic: u16,
    pub s_state: u16,
    pub s_errors: u16,
    pub s_minor_rev_level: u16,
    pub s_lastcheck: u32,
    pub s_checkinterval: u32,
    pub s_creator_os: u32,
    pub s_rev_level: u32,
    pub s_def_resuid: u16,
    pub s_def_resgid: u16,

    // Extended fields
    pub s_first_ino: u32,
    pub s_inode_size: u16,
    pub s_block_group_nr: u16,
    pub s_feature_compat: u32,
    pub s_feature_incompat: u32,
    pub s_feature_ro_compat: u32,
    pub s_uuid: [u8; 16],
    pub s_volume_name: [u8; 16],
    pub s_last_mounted: [u8; 64],
    pub s_algorithm_usage_bitmap: u32,

    // Performance hints
    pub s_prealloc_blocks: u8,
    pub s_prealloc_dir_blocks: u8,
    pub s_reserved_gdt_blocks: u16,

    // Journaling support
    pub s_journal_uuid: [u8; 16],
    pub s_journal_inum: u32,
    pub s_journal_dev: u32,
    pub s_last_orphan: u32,
    pub s_hash_seed: [u32; 4],
    pub s_def_hash_version: u8,
    pub s_jnl_backup_type: u8,
    pub s_desc_size: u16,
    pub s_default_mount_opts: u32,
    pub s_first_meta_bg: u32,
    pub s_mkfs_time: u32,
    pub s_jnl_blocks: [u32; 17],
}

impl Ext4Superblock {
    /// Get block size in bytes
    pub fn block_size(&self) -> u32 {
        1024 << self.s_log_block_size
    }

    /// Check if journaling is enabled
    pub fn has_journal(&self) -> bool {
        (self.s_feature_compat & EXT4_FEATURE_COMPAT_HAS_JOURNAL) != 0
    }

    /// Get journal inode number
    pub fn journal_inum(&self) -> u32 {
        self.s_journal_inum
    }

    /// Get number of block groups
    pub fn block_groups_count(&self) -> u32 {
        (self.s_blocks_count + self.s_blocks_per_group - 1) / self.s_blocks_per_group
    }

    /// Get inode size
    pub fn inode_size(&self) -> u16 {
        if self.s_rev_level == 0 {
            128 // Old format
        } else {
            self.s_inode_size
        }
    }
}

/// ext4 inode structure (128+ bytes on disk)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4Inode {
    pub i_mode: u16,        // File mode
    pub i_uid: u16,         // Owner UID (lower 16 bits)
    pub i_size_lo: u32,     // Size in bytes (lower 32 bits)
    pub i_atime: u32,       // Access time
    pub i_ctime: u32,       // Change time
    pub i_mtime: u32,       // Modification time
    pub i_dtime: u32,       // Deletion time
    pub i_gid: u16,         // Group ID (lower 16 bits)
    pub i_links_count: u16, // Hard links count
    pub i_blocks_lo: u32,   // Blocks count (lower 32 bits)
    pub i_flags: u32,       // File flags
    pub i_osd1: u32,        // OS dependent
    pub i_block: [u32; 15], // Block pointers (or extent tree root)
    pub i_generation: u32,  // File version (for NFS)
    pub i_file_acl_lo: u32, // File ACL (lower 32 bits)
    pub i_size_high: u32,   // Size high 32 bits (for large files)
    pub i_obso_faddr: u32,  // Obsolete fragment address
    pub i_osd2: [u8; 12],   // OS dependent
    pub i_extra_isize: u16, // Extra inode size
    pub i_checksum_hi: u16, // Inode checksum high bits
    pub i_ctime_extra: u32, // Extra change time
    pub i_mtime_extra: u32, // Extra modification time
    pub i_atime_extra: u32, // Extra access time
    pub i_crtime: u32,      // Creation time
    pub i_crtime_extra: u32, // Extra creation time
    pub i_version_hi: u32,  // High 32 bits of version
    pub i_projid: u32,      // Project ID
}

impl Ext4Inode {
    /// Get full file size (64-bit)
    pub fn size(&self) -> u64 {
        (self.i_size_high as u64) << 32 | self.i_size_lo as u64
    }

    /// Set file size
    pub fn set_size(&mut self, size: u64) {
        self.i_size_lo = size as u32;
        self.i_size_high = (size >> 32) as u32;
    }

    /// Check if inode uses extents
    pub fn uses_extents(&self) -> bool {
        (self.i_flags & 0x80000) != 0 // EXT4_EXTENTS_FL
    }

    /// Check if this is a directory
    pub fn is_dir(&self) -> bool {
        (self.i_mode & 0xF000) == 0x4000 // S_IFDIR
    }

    /// Check if this is a regular file
    pub fn is_file(&self) -> bool {
        (self.i_mode & 0xF000) == 0x8000 // S_IFREG
    }
}

/// ext4 extent header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4ExtentHeader {
    pub eh_magic: u16,      // Magic number (0xF30A)
    pub eh_entries: u16,    // Number of valid entries
    pub eh_max: u16,        // Capacity of store
    pub eh_depth: u16,      // Depth of tree (0 = leaf level)
    pub eh_generation: u32, // Generation of the tree
}

/// ext4 extent (leaf node - maps logical to physical blocks)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4Extent {
    pub ee_block: u32,  // First logical block
    pub ee_len: u16,    // Number of blocks covered
    pub ee_start_hi: u16, // High 16 bits of physical block
    pub ee_start_lo: u32, // Low 32 bits of physical block
}

impl Ext4Extent {
    /// Get full physical block number
    pub fn start(&self) -> u64 {
        (self.ee_start_hi as u64) << 32 | self.ee_start_lo as u64
    }

    /// Set physical block number
    pub fn set_start(&mut self, block: u64) {
        self.ee_start_lo = block as u32;
        self.ee_start_hi = (block >> 32) as u16;
    }
}

/// ext4 extent index (internal node - points to next level)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4ExtentIdx {
    pub ei_block: u32,  // Logical block covered
    pub ei_leaf_lo: u32, // Pointer to physical block of next level
    pub ei_leaf_hi: u16, // High 16 bits of physical block
    pub ei_unused: u16, // Reserved
}

impl Ext4ExtentIdx {
    /// Get physical block of next level
    pub fn leaf(&self) -> u64 {
        (self.ei_leaf_hi as u64) << 32 | self.ei_leaf_lo as u64
    }

    /// Set physical block of next level
    pub fn set_leaf(&mut self, block: u64) {
        self.ei_leaf_lo = block as u32;
        self.ei_leaf_hi = (block >> 32) as u16;
    }
}

/// ext4 block group descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4BlockGroupDesc {
    pub bg_block_bitmap_lo: u32,      // Block bitmap block (lower 32 bits)
    pub bg_inode_bitmap_lo: u32,      // Inode bitmap block (lower 32 bits)
    pub bg_inode_table_lo: u32,       // Inode table block (lower 32 bits)
    pub bg_free_blocks_count_lo: u16, // Free blocks count (lower 16 bits)
    pub bg_free_inodes_count_lo: u16, // Free inodes count (lower 16 bits)
    pub bg_used_dirs_count_lo: u16,   // Directories count (lower 16 bits)
    pub bg_flags: u16,                // Flags
    pub bg_exclude_bitmap_lo: u32,    // Snapshot exclusion bitmap
    pub bg_block_bitmap_csum_lo: u16, // Block bitmap checksum
    pub bg_inode_bitmap_csum_lo: u16, // Inode bitmap checksum
    pub bg_itable_unused_lo: u16,     // Unused inodes count
    pub bg_checksum: u16,             // Group descriptor checksum
    // 64-bit fields (if INCOMPAT_64BIT is set)
    pub bg_block_bitmap_hi: u32,      // Block bitmap block (upper 32 bits)
    pub bg_inode_bitmap_hi: u32,      // Inode bitmap block (upper 32 bits)
    pub bg_inode_table_hi: u32,       // Inode table block (upper 32 bits)
    pub bg_free_blocks_count_hi: u16, // Free blocks count (upper 16 bits)
    pub bg_free_inodes_count_hi: u16, // Free inodes count (upper 16 bits)
    pub bg_used_dirs_count_hi: u16,   // Directories count (upper 16 bits)
    pub bg_itable_unused_hi: u16,     // Unused inodes count (upper 16 bits)
    pub bg_exclude_bitmap_hi: u32,    // Snapshot exclusion bitmap (upper 32 bits)
    pub bg_block_bitmap_csum_hi: u16, // Block bitmap checksum (upper 16 bits)
    pub bg_inode_bitmap_csum_hi: u16, // Inode bitmap checksum (upper 16 bits)
    pub bg_reserved: u32,             // Reserved
}

impl Ext4BlockGroupDesc {
    /// Get block bitmap block number (64-bit)
    pub fn block_bitmap(&self) -> u64 {
        (self.bg_block_bitmap_hi as u64) << 32 | self.bg_block_bitmap_lo as u64
    }

    /// Get inode bitmap block number (64-bit)
    pub fn inode_bitmap(&self) -> u64 {
        (self.bg_inode_bitmap_hi as u64) << 32 | self.bg_inode_bitmap_lo as u64
    }

    /// Get inode table block number (64-bit)
    pub fn inode_table(&self) -> u64 {
        (self.bg_inode_table_hi as u64) << 32 | self.bg_inode_table_lo as u64
    }

    /// Get free blocks count (32-bit)
    pub fn free_blocks_count(&self) -> u32 {
        (self.bg_free_blocks_count_hi as u32) << 16 | self.bg_free_blocks_count_lo as u32
    }

    /// Get free inodes count (32-bit)
    pub fn free_inodes_count(&self) -> u32 {
        (self.bg_free_inodes_count_hi as u32) << 16 | self.bg_free_inodes_count_lo as u32
    }

    /// Decrement free blocks count
    pub fn dec_free_blocks(&mut self) {
        let count = self.free_blocks_count();
        if count > 0 {
            let new_count = count - 1;
            self.bg_free_blocks_count_lo = new_count as u16;
            self.bg_free_blocks_count_hi = (new_count >> 16) as u16;
        }
    }

    /// Decrement free inodes count
    pub fn dec_free_inodes(&mut self) {
        let count = self.free_inodes_count();
        if count > 0 {
            let new_count = count - 1;
            self.bg_free_inodes_count_lo = new_count as u16;
            self.bg_free_inodes_count_hi = (new_count >> 16) as u16;
        }
    }
}

/// ext4 directory entry (version 2 with file type)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext4DirEntry2 {
    pub inode: u32,      // Inode number
    pub rec_len: u16,    // Directory entry length
    pub name_len: u8,    // File name length
    pub file_type: u8,   // File type
    // name follows (variable length)
}

impl Ext4DirEntry2 {
    /// Minimum entry size (without name)
    pub const MIN_SIZE: usize = 8;

    /// Calculate required record length for given name
    pub fn required_len(name_len: u8) -> u16 {
        let len = Self::MIN_SIZE + name_len as usize;
        // Align to 4 bytes
        ((len + 3) & !3) as u16
    }

    /// Check if this is a valid entry
    pub fn is_valid(&self) -> bool {
        self.inode != 0 && self.rec_len >= Self::MIN_SIZE as u16
    }
}

/// File types for directory entries
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ext4FileType {
    Unknown = 0,
    RegularFile = 1,
    Directory = 2,
    CharDevice = 3,
    BlockDevice = 4,
    Fifo = 5,
    Socket = 6,
    Symlink = 7,
}

/// ext4 filesystem structure
pub struct Ext4FileSystem {
    /// Block device
    pub device: Arc<BlockDevice>,
    /// Superblock
    pub superblock: Mutex<Ext4Superblock>,
    /// Block group descriptors cache
    pub block_groups: Mutex<Vec<Ext4BlockGroupDesc>>,
    /// Journal (if enabled)
    pub journal: Option<Arc<Journal>>,
    /// Mount state
    pub is_mounted: Mutex<bool>,
}

impl Ext4FileSystem {
    /// Mount ext4 filesystem
    pub fn mount(device: Arc<BlockDevice>) -> Result<Arc<Self>> {
        crate::info!("ext4: Mounting filesystem...");

        // Read superblock (at offset 1024)
        let mut sb_buf = vec![0u8; 1024];
        device.read(2, &mut sb_buf)?; // Block 2 (1024 bytes offset)

        // Parse superblock
        let sb: Ext4Superblock = unsafe {
            core::ptr::read(sb_buf.as_ptr() as *const Ext4Superblock)
        };

        // Verify magic number
        if sb.s_magic != EXT4_SUPER_MAGIC {
            crate::error!("ext4: Invalid magic number: {:#x}", sb.s_magic);
            return Err(Errno::EINVAL);
        }

        crate::info!("ext4: Valid filesystem (blocks={}, inodes={}, block_size={})",
                     sb.s_blocks_count, sb.s_inodes_count, sb.block_size());

        // Load journal if enabled
        let journal = if sb.has_journal() {
            crate::info!("ext4: Journal enabled (inode={})", sb.journal_inum());

            // For MVP, assume journal is at fixed location
            // In full implementation, would read journal inode to get blocks
            let journal_block = 1024; // Placeholder

            match Journal::load(device.clone(), journal_block) {
                Ok(j) => {
                    // Replay journal for crash recovery
                    if let Err(e) = j.replay() {
                        crate::error!("ext4: Journal replay failed: {:?}", e);
                        return Err(e);
                    }
                    Some(j)
                }
                Err(e) => {
                    crate::warn!("ext4: Failed to load journal: {:?}", e);
                    None
                }
            }
        } else {
            crate::info!("ext4: Journal not enabled");
            None
        };

        // Load block group descriptors
        let bg_count = sb.block_groups_count();
        let mut block_groups = Vec::with_capacity(bg_count as usize);

        // Block group descriptor table starts after superblock
        let bg_table_block = if sb.block_size() == 1024 { 2 } else { 1 };
        let desc_size = if (sb.s_feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0 {
            sb.s_desc_size as usize
        } else {
            32 // 32-byte descriptors for 32-bit filesystems
        };

        for i in 0..bg_count {
            let offset = i as usize * desc_size;
            let block = bg_table_block + (offset / sb.block_size() as usize) as u64;
            let block_offset = offset % sb.block_size() as usize;

            let mut buf = vec![0u8; sb.block_size() as usize];
            device.read(block, &mut buf)?;

            let bg_desc: Ext4BlockGroupDesc = unsafe {
                core::ptr::read((buf.as_ptr().add(block_offset)) as *const Ext4BlockGroupDesc)
            };
            block_groups.push(bg_desc);
        }

        crate::info!("ext4: Loaded {} block group descriptors", bg_count);

        let fs = Arc::new(Self {
            device,
            superblock: Mutex::new(sb),
            block_groups: Mutex::new(block_groups),
            journal,
            is_mounted: Mutex::new(true),
        });

        crate::info!("ext4: Mount successful");
        Ok(fs)
    }

    /// Unmount filesystem
    pub fn unmount(&self) -> Result<()> {
        let mut mounted = self.is_mounted.lock();
        if !*mounted {
            return Err(Errno::EINVAL);
        }

        crate::info!("ext4: Unmounting filesystem...");

        // Commit any pending transactions
        if let Some(ref journal) = self.journal {
            let txn_opt = { journal.current_transaction.lock().as_ref().cloned() };
            if let Some(txn) = txn_opt {
                journal.commit_transaction(txn)?;
            }
        }

        // Sync all buffers
        let _ = crate::mm::sync_all();

        *mounted = false;
        crate::info!("ext4: Unmount successful");
        Ok(())
    }

    /// Begin a transaction (if journaling enabled)
    pub fn begin_transaction(&self) -> Option<Arc<TransactionHandle>> {
        self.journal.as_ref().map(|j| j.begin_transaction())
    }

    /// Commit a transaction
    pub fn commit_transaction(&self, txn: Arc<TransactionHandle>) -> Result<()> {
        if let Some(ref journal) = self.journal {
            journal.commit_transaction(txn)
        } else {
            // No journal, just sync
            let _ = crate::mm::sync_all();
            Ok(())
        }
    }

    /// Create a file (with transaction)
    pub fn create_file(&self, parent_ino: u32, name: &str, mode: u32) -> Result<u32> {
        if name.len() > 255 {
            return Err(Errno::EINVAL);
        }

        // Begin transaction
        let txn = self.begin_transaction();

        // Check if parent is a directory
        let parent_inode = self.read_inode(parent_ino)?;
        if !parent_inode.is_dir() {
            if let Some(txn) = txn {
                if let Some(ref journal) = self.journal {
                    journal.abort_transaction(txn);
                }
            }
            return Err(Errno::ENOTDIR);
        }

        // Check if file already exists
        if self.find_dir_entry(parent_ino, name).is_ok() {
            if let Some(txn) = txn {
                if let Some(ref journal) = self.journal {
                    journal.abort_transaction(txn);
                }
            }
            return Err(Errno::EEXIST);
        }

        // 1. Allocate inode
        let inode_num = self.allocate_inode(0, mode as u16)?;

        // Track inode bitmap and inode table blocks in transaction
        if let Some(ref txn) = txn {
            // Calculate inode bitmap block and inode table block
            let sb = self.superblock.lock();
            let block_groups = self.block_groups.lock();
            let group_idx = ((inode_num - 1) / sb.s_inodes_per_group) as usize;
            if group_idx < block_groups.len() {
                let bg = &block_groups[group_idx];
                txn.add_block(bg.inode_bitmap());
                txn.add_block(bg.inode_table());
            }
            drop(block_groups);
            drop(sb);
        }

        // 2. Add directory entry
        let file_type = if mode & 0xF000 == 0x4000 {
            Ext4FileType::Directory
        } else {
            Ext4FileType::RegularFile
        };

        if let Err(e) = self.add_dir_entry(parent_ino, name, inode_num, file_type) {
            // Rollback: free the allocated inode
            let _ = self.free_inode(inode_num);
            if let Some(txn) = txn {
                if let Some(ref journal) = self.journal {
                    journal.abort_transaction(txn);
                }
            }
            return Err(e);
        }

        // Track directory blocks in transaction
        if let Some(ref txn) = txn {
            let parent_inode = self.read_inode(parent_ino)?;
            let sb = self.superblock.lock();
            let block_size = sb.block_size() as usize;
            let num_blocks = (parent_inode.size() as usize + block_size - 1) / block_size;
            drop(sb);

            // Add all directory blocks (simplified - would track specific modified blocks)
            for block_idx in 0..num_blocks {
                if let Ok(block_num) = self.get_inode_block(&parent_inode, block_idx as u64) {
                    txn.add_block(block_num);
                }
            }
        }

        // 3. Commit transaction
        if let Some(txn) = txn {
            if let Err(e) = self.commit_transaction(txn) {
                // Transaction commit failed - try to clean up
                let _ = self.remove_dir_entry(parent_ino, name);
                let _ = self.free_inode(inode_num);
                return Err(e);
            }
        }

        crate::info!("ext4: Created file '{}' with inode {}", name, inode_num);
        Ok(inode_num)
    }

    /// Delete a file (with transaction)
    pub fn delete_file(&self, parent_ino: u32, name: &str) -> Result<()> {
        // Begin transaction
        let txn = self.begin_transaction();

        // Check if parent is a directory
        let parent_inode = self.read_inode(parent_ino)?;
        if !parent_inode.is_dir() {
            if let Some(txn) = txn {
                if let Some(ref journal) = self.journal {
                    journal.abort_transaction(txn);
                }
            }
            return Err(Errno::ENOTDIR);
        }

        // 1. Remove directory entry and get inode number
        let inode_num = match self.remove_dir_entry(parent_ino, name) {
            Ok(ino) => ino,
            Err(e) => {
                if let Some(txn) = txn {
                    if let Some(ref journal) = self.journal {
                        journal.abort_transaction(txn);
                    }
                }
                return Err(e);
            }
        };

        // Track directory blocks in transaction
        if let Some(ref txn) = txn {
            let parent_inode = self.read_inode(parent_ino)?;
            let sb = self.superblock.lock();
            let block_size = sb.block_size() as usize;
            let num_blocks = (parent_inode.size() as usize + block_size - 1) / block_size;
            drop(sb);

            // Add all directory blocks
            for block_idx in 0..num_blocks {
                if let Ok(block_num) = self.get_inode_block(&parent_inode, block_idx as u64) {
                    txn.add_block(block_num);
                }
            }
        }

        // 2. Read inode and decrement link count
        let mut inode = self.read_inode(inode_num)?;
        if inode.i_links_count > 0 {
            inode.i_links_count -= 1;
        }

        // 3. If link count is 0, free inode and data blocks
        if inode.i_links_count == 0 {
            crate::debug!("ext4: Link count is 0, freeing inode {} and data blocks", inode_num);

            // Set deletion time
            inode.i_dtime = 0; // Would use real timestamp

            // Free all data blocks
            let sb = self.superblock.lock();
            let block_size = sb.block_size() as usize;
            let num_blocks = (inode.size() as usize + block_size - 1) / block_size;
            drop(sb);

            for block_idx in 0..num_blocks {
                if let Ok(block_num) = self.get_inode_block(&inode, block_idx as u64) {
                    if let Err(e) = self.free_block(block_num) {
                        crate::warn!("ext4: Failed to free block {}: {:?}", block_num, e);
                    }

                    // Track block bitmap in transaction
                    if let Some(ref txn) = txn {
                        let sb = self.superblock.lock();
                        let block_groups = self.block_groups.lock();
                        let relative_block = block_num - sb.s_first_data_block as u64;
                        let group_idx = (relative_block / sb.s_blocks_per_group as u64) as usize;
                        if group_idx < block_groups.len() {
                            let bg = &block_groups[group_idx];
                            txn.add_block(bg.block_bitmap());
                        }
                        drop(block_groups);
                        drop(sb);
                    }
                }
            }

            // Write updated inode (with dtime set)
            self.write_inode(inode_num, &inode)?;

            // Free inode
            self.free_inode(inode_num)?;

            // Track inode bitmap and inode table in transaction
            if let Some(ref txn) = txn {
                let sb = self.superblock.lock();
                let block_groups = self.block_groups.lock();
                let group_idx = ((inode_num - 1) / sb.s_inodes_per_group) as usize;
                if group_idx < block_groups.len() {
                    let bg = &block_groups[group_idx];
                    txn.add_block(bg.inode_bitmap());
                    txn.add_block(bg.inode_table());
                }
                drop(block_groups);
                drop(sb);
            }
        } else {
            // Just update link count
            self.write_inode(inode_num, &inode)?;

            // Track inode table in transaction
            if let Some(ref txn) = txn {
                let sb = self.superblock.lock();
                let block_groups = self.block_groups.lock();
                let group_idx = ((inode_num - 1) / sb.s_inodes_per_group) as usize;
                if group_idx < block_groups.len() {
                    let bg = &block_groups[group_idx];
                    txn.add_block(bg.inode_table());
                }
                drop(block_groups);
                drop(sb);
            }
        }

        // 4. Commit transaction
        if let Some(txn) = txn {
            self.commit_transaction(txn)?;
        }

        crate::info!("ext4: Deleted file '{}' (inode {})", name, inode_num);
        Ok(())
    }

    /// Write data with ordered mode
    ///
    /// Ordered mode: Write data blocks before committing metadata changes
    pub fn write_ordered(&self, inode_num: u32, offset: u64, data: &[u8]) -> Result<usize> {
        // 1. Begin transaction
        let txn = self.begin_transaction();

        // 2. Write data blocks to disk (NOT in journal)
        // TODO: Allocate blocks and write data

        // 3. Update metadata (inode) in journal
        if let Some(ref txn) = txn {
            txn.add_block(inode_num as u64);
        }

        // 4. Commit transaction (metadata)
        if let Some(txn) = txn {
            self.commit_transaction(txn)?;
        }

        // Placeholder
        Ok(data.len())
    }

    /// Sync filesystem (force commit)
    pub fn sync(&self) -> Result<()> {
        crate::info!("ext4: Syncing filesystem...");

        // Commit current transaction if any
        if let Some(ref journal) = self.journal {
            let txn_opt = { journal.current_transaction.lock().as_ref().cloned() };
            if let Some(txn) = txn_opt {
                journal.commit_transaction(txn)?;
            }
        }

        // Sync all buffers
        let _ = crate::mm::sync_all();

        crate::info!("ext4: Sync complete");
        Ok(())
    }

    /// Check filesystem consistency (fsck)
    pub fn fsck(&self) -> Result<()> {
        crate::info!("ext4: Checking filesystem consistency...");

        let sb = self.superblock.lock();

        // Check superblock state
        if sb.s_state != 1 {
            crate::warn!("ext4: Filesystem not cleanly unmounted (state={})", sb.s_state);
        }

        // Check journal if present
        if let Some(ref journal) = self.journal {
            let jsb = journal.superblock.lock();
            if jsb.s_start != 0 {
                crate::warn!("ext4: Journal not empty, may need replay");
            }
        }

        // TODO: Full consistency checks
        // - Block bitmap consistency
        // - Inode bitmap consistency
        // - Directory structure
        // - Link counts

        crate::info!("ext4: Filesystem check complete");
        Ok(())
    }

    /// Allocate a data block
    ///
    /// Searches block groups for a free block, marks it used in the bitmap,
    /// and updates the block group descriptor and superblock.
    pub fn allocate_block(&self, preferred_group: u32) -> Result<u64> {
        let sb = self.superblock.lock();
        let mut block_groups = self.block_groups.lock();

        if sb.s_free_blocks_count == 0 {
            return Err(Errno::ENOSPC);
        }

        let bg_count = sb.block_groups_count();
        let block_size = sb.block_size();

        // Start search from preferred group (for locality)
        for i in 0..bg_count {
            let group_idx = ((preferred_group + i) % bg_count) as usize;
            let bg = &mut block_groups[group_idx];

            if bg.free_blocks_count() == 0 {
                continue;
            }

            // Read block bitmap
            let bitmap_block = bg.block_bitmap();
            let mut bitmap = vec![0u8; block_size as usize];
            self.device.read(bitmap_block, &mut bitmap)?;

            // Find first free bit
            if let Some(bit_offset) = find_first_zero_bit(&bitmap) {
                // Mark as used
                set_bit(&mut bitmap, bit_offset);

                // Write bitmap back
                self.device.write(bitmap_block, &bitmap)?;

                // Calculate absolute block number
                let block_num = sb.s_first_data_block +
                                (group_idx as u32 * sb.s_blocks_per_group) +
                                bit_offset;

                // Update block group descriptor
                bg.dec_free_blocks();

                // Write back block group descriptor
                self.write_block_group_desc(group_idx, bg)?;

                // Update superblock (will be done on unmount/sync)
                drop(block_groups);
                drop(sb);

                crate::debug!("ext4: Allocated block {} from group {}", block_num, group_idx);
                return Ok(block_num as u64);
            }
        }

        Err(Errno::ENOSPC)
    }

    /// Free a data block
    ///
    /// Marks the block as free in the bitmap and updates counters.
    pub fn free_block(&self, block_num: u64) -> Result<()> {
        let sb = self.superblock.lock();
        let mut block_groups = self.block_groups.lock();

        let block_size = sb.block_size();
        let blocks_per_group = sb.s_blocks_per_group;

        // Calculate group and offset
        let relative_block = block_num - sb.s_first_data_block as u64;
        let group_idx = (relative_block / blocks_per_group as u64) as usize;
        let bit_offset = (relative_block % blocks_per_group as u64) as u32;

        if group_idx >= block_groups.len() {
            return Err(Errno::EINVAL);
        }

        let bg = &mut block_groups[group_idx];

        // Read block bitmap
        let bitmap_block = bg.block_bitmap();
        let mut bitmap = vec![0u8; block_size as usize];
        self.device.read(bitmap_block, &mut bitmap)?;

        // Check if already free
        if !test_bit(&bitmap, bit_offset) {
            crate::warn!("ext4: Attempt to free already-free block {}", block_num);
            return Err(Errno::EINVAL);
        }

        // Mark as free
        clear_bit(&mut bitmap, bit_offset);

        // Write bitmap back
        self.device.write(bitmap_block, &bitmap)?;

        // Update block group descriptor
        let count = bg.free_blocks_count();
        bg.bg_free_blocks_count_lo = (count + 1) as u16;
        bg.bg_free_blocks_count_hi = ((count + 1) >> 16) as u16;

        // Write back block group descriptor
        self.write_block_group_desc(group_idx, bg)?;

        crate::debug!("ext4: Freed block {}", block_num);
        Ok(())
    }

    /// Allocate an inode
    ///
    /// Searches block groups for a free inode, marks it used in the bitmap,
    /// returns the inode number and initializes the inode structure.
    pub fn allocate_inode(&self, preferred_group: u32, mode: u16) -> Result<u32> {
        let sb = self.superblock.lock();
        let mut block_groups = self.block_groups.lock();

        if sb.s_free_inodes_count == 0 {
            return Err(Errno::ENOSPC);
        }

        let bg_count = sb.block_groups_count();
        let block_size = sb.block_size();

        // Start search from preferred group (for locality)
        for i in 0..bg_count {
            let group_idx = ((preferred_group + i) % bg_count) as usize;
            let bg = &mut block_groups[group_idx];

            if bg.free_inodes_count() == 0 {
                continue;
            }

            // Read inode bitmap
            let bitmap_block = bg.inode_bitmap();
            let mut bitmap = vec![0u8; block_size as usize];
            self.device.read(bitmap_block, &mut bitmap)?;

            // Find first free bit
            if let Some(bit_offset) = find_first_zero_bit(&bitmap) {
                // Check if exceeds inodes_per_group
                if bit_offset >= sb.s_inodes_per_group {
                    continue;
                }

                // Mark as used
                set_bit(&mut bitmap, bit_offset);

                // Write bitmap back
                self.device.write(bitmap_block, &bitmap)?;

                // Calculate absolute inode number (1-indexed in ext4)
                let inode_num = 1 + (group_idx as u32 * sb.s_inodes_per_group) + bit_offset;

                // Update block group descriptor
                bg.dec_free_inodes();

                // Write back block group descriptor
                self.write_block_group_desc(group_idx, bg)?;

                // Initialize the inode on disk
                self.write_inode(inode_num, &Self::new_inode(mode))?;

                crate::debug!("ext4: Allocated inode {} from group {}", inode_num, group_idx);
                return Ok(inode_num);
            }
        }

        Err(Errno::ENOSPC)
    }

    /// Free an inode
    ///
    /// Marks the inode as free in the bitmap and updates counters.
    pub fn free_inode(&self, inode_num: u32) -> Result<()> {
        let sb = self.superblock.lock();
        let mut block_groups = self.block_groups.lock();

        if inode_num < 1 {
            return Err(Errno::EINVAL);
        }

        let block_size = sb.block_size();
        let inodes_per_group = sb.s_inodes_per_group;

        // Calculate group and offset (inode numbers are 1-indexed)
        let relative_inode = inode_num - 1;
        let group_idx = (relative_inode / inodes_per_group) as usize;
        let bit_offset = relative_inode % inodes_per_group;

        if group_idx >= block_groups.len() {
            return Err(Errno::EINVAL);
        }

        let bg = &mut block_groups[group_idx];

        // Read inode bitmap
        let bitmap_block = bg.inode_bitmap();
        let mut bitmap = vec![0u8; block_size as usize];
        self.device.read(bitmap_block, &mut bitmap)?;

        // Check if already free
        if !test_bit(&bitmap, bit_offset) {
            crate::warn!("ext4: Attempt to free already-free inode {}", inode_num);
            return Err(Errno::EINVAL);
        }

        // Mark as free
        clear_bit(&mut bitmap, bit_offset);

        // Write bitmap back
        self.device.write(bitmap_block, &bitmap)?;

        // Update block group descriptor
        let count = bg.free_inodes_count();
        bg.bg_free_inodes_count_lo = (count + 1) as u16;
        bg.bg_free_inodes_count_hi = ((count + 1) >> 16) as u16;

        // Write back block group descriptor
        self.write_block_group_desc(group_idx, bg)?;

        crate::debug!("ext4: Freed inode {}", inode_num);
        Ok(())
    }

    /// Create a new inode structure with given mode
    fn new_inode(mode: u16) -> Ext4Inode {
        let mut inode: Ext4Inode = unsafe { core::mem::zeroed() };
        inode.i_mode = mode;
        inode.i_links_count = 1;

        // Set current timestamps (would use real time in production)
        let now = 0; // Placeholder - would get from system clock
        inode.i_atime = now;
        inode.i_ctime = now;
        inode.i_mtime = now;
        inode.i_crtime = now;

        // Enable extents by default
        inode.i_flags = 0x80000; // EXT4_EXTENTS_FL

        inode
    }

    /// Read inode from disk
    pub fn read_inode(&self, inode_num: u32) -> Result<Ext4Inode> {
        let sb = self.superblock.lock();
        let block_groups = self.block_groups.lock();

        if inode_num < 1 {
            return Err(Errno::EINVAL);
        }

        let inodes_per_group = sb.s_inodes_per_group;
        let inode_size = sb.inode_size() as usize;

        // Calculate location
        let relative_inode = inode_num - 1;
        let group_idx = (relative_inode / inodes_per_group) as usize;
        let inode_offset = (relative_inode % inodes_per_group) as usize;

        if group_idx >= block_groups.len() {
            return Err(Errno::EINVAL);
        }

        let bg = &block_groups[group_idx];
        let inode_table = bg.inode_table();

        // Calculate block and offset within block
        let block_size = sb.block_size() as usize;
        let byte_offset = inode_offset * inode_size;
        let block = inode_table + (byte_offset / block_size) as u64;
        let block_offset = byte_offset % block_size;

        // Read block containing inode
        let mut buf = vec![0u8; block_size];
        self.device.read(block, &mut buf)?;

        // Parse inode
        let inode: Ext4Inode = unsafe {
            core::ptr::read((buf.as_ptr().add(block_offset)) as *const Ext4Inode)
        };

        Ok(inode)
    }

    /// Write inode to disk
    pub fn write_inode(&self, inode_num: u32, inode: &Ext4Inode) -> Result<()> {
        let sb = self.superblock.lock();
        let block_groups = self.block_groups.lock();

        if inode_num < 1 {
            return Err(Errno::EINVAL);
        }

        let inodes_per_group = sb.s_inodes_per_group;
        let inode_size = sb.inode_size() as usize;

        // Calculate location
        let relative_inode = inode_num - 1;
        let group_idx = (relative_inode / inodes_per_group) as usize;
        let inode_offset = (relative_inode % inodes_per_group) as usize;

        if group_idx >= block_groups.len() {
            return Err(Errno::EINVAL);
        }

        let bg = &block_groups[group_idx];
        let inode_table = bg.inode_table();

        // Calculate block and offset within block
        let block_size = sb.block_size() as usize;
        let byte_offset = inode_offset * inode_size;
        let block = inode_table + (byte_offset / block_size) as u64;
        let block_offset = byte_offset % block_size;

        // Read block, modify inode, write back
        let mut buf = vec![0u8; block_size];
        self.device.read(block, &mut buf)?;

        let inode_bytes = unsafe {
            core::slice::from_raw_parts(inode as *const Ext4Inode as *const u8, inode_size)
        };
        buf[block_offset..block_offset + inode_size].copy_from_slice(inode_bytes);

        self.device.write(block, &buf)?;
        Ok(())
    }

    /// Add directory entry to a directory inode
    ///
    /// Finds space in existing directory blocks or allocates new block.
    pub fn add_dir_entry(&self, dir_inode_num: u32, entry_name: &str, entry_inode: u32, file_type: Ext4FileType) -> Result<()> {
        if entry_name.len() > 255 {
            return Err(Errno::EINVAL);
        }

        let mut dir_inode = self.read_inode(dir_inode_num)?;
        if !dir_inode.is_dir() {
            return Err(Errno::ENOTDIR);
        }

        let sb = self.superblock.lock();
        let block_size = sb.block_size() as usize;
        drop(sb);

        let required_len = Ext4DirEntry2::required_len(entry_name.len() as u8);

        // Try to find space in existing blocks
        let num_blocks = (dir_inode.size() as usize + block_size - 1) / block_size;

        for block_idx in 0..num_blocks {
            let block_num = self.get_inode_block(&dir_inode, block_idx as u64)?;
            let mut block_data = vec![0u8; block_size];
            self.device.read(block_num, &mut block_data)?;

            // Scan directory entries in this block
            let mut offset = 0;
            while offset < block_size {
                if offset + Ext4DirEntry2::MIN_SIZE > block_size {
                    break;
                }

                let entry: &Ext4DirEntry2 = unsafe {
                    &*(block_data.as_ptr().add(offset) as *const Ext4DirEntry2)
                };

                if entry.rec_len == 0 {
                    break;
                }

                // Check if this is last entry with extra space
                let actual_len = Ext4DirEntry2::required_len(entry.name_len);
                let available_space = entry.rec_len - actual_len;

                if entry.inode == 0 && entry.rec_len >= required_len {
                    // Empty entry, reuse it
                    self.write_dir_entry(&mut block_data, offset, entry_inode, entry_name, file_type, required_len)?;
                    self.device.write(block_num, &block_data)?;
                    return Ok(());
                } else if available_space >= required_len {
                    // Split this entry
                    // Update current entry to its actual size
                    let current_entry: &mut Ext4DirEntry2 = unsafe {
                        &mut *(block_data.as_mut_ptr().add(offset) as *mut Ext4DirEntry2)
                    };
                    current_entry.rec_len = actual_len;

                    // Write new entry after it
                    let new_offset = offset + actual_len as usize;
                    self.write_dir_entry(&mut block_data, new_offset, entry_inode, entry_name, file_type, available_space)?;
                    self.device.write(block_num, &block_data)?;
                    return Ok(());
                }

                offset += entry.rec_len as usize;
            }
        }

        // No space found, allocate new block
        let new_block = self.allocate_block(0)?;
        let mut block_data = vec![0u8; block_size];

        // Write entry at start of new block, consuming entire block
        self.write_dir_entry(&mut block_data, 0, entry_inode, entry_name, file_type, block_size as u16)?;
        self.device.write(new_block, &block_data)?;

        // Update directory inode
        dir_inode.set_size(dir_inode.size() + block_size as u64);
        dir_inode.i_blocks_lo += (block_size / 512) as u32; // 512-byte sectors

        // TODO: Add block to inode's extent tree (for now, assume linear mapping)
        self.write_inode(dir_inode_num, &dir_inode)?;

        Ok(())
    }

    /// Remove directory entry from a directory inode
    pub fn remove_dir_entry(&self, dir_inode_num: u32, entry_name: &str) -> Result<u32> {
        let dir_inode = self.read_inode(dir_inode_num)?;
        if !dir_inode.is_dir() {
            return Err(Errno::ENOTDIR);
        }

        let sb = self.superblock.lock();
        let block_size = sb.block_size() as usize;
        drop(sb);

        let num_blocks = (dir_inode.size() as usize + block_size - 1) / block_size;

        for block_idx in 0..num_blocks {
            let block_num = self.get_inode_block(&dir_inode, block_idx as u64)?;
            let mut block_data = vec![0u8; block_size];
            self.device.read(block_num, &mut block_data)?;

            let mut offset = 0;
            let mut prev_offset: Option<usize> = None;

            while offset < block_size {
                if offset + Ext4DirEntry2::MIN_SIZE > block_size {
                    break;
                }

                let entry: &Ext4DirEntry2 = unsafe {
                    &*(block_data.as_ptr().add(offset) as *const Ext4DirEntry2)
                };

                if entry.rec_len == 0 {
                    break;
                }

                // Get entry name
                let name_offset = offset + Ext4DirEntry2::MIN_SIZE;
                let name_slice = &block_data[name_offset..name_offset + entry.name_len as usize];

                if entry.inode != 0 && name_slice == entry_name.as_bytes() {
                    // Found the entry to remove
                    let removed_inode = entry.inode;

                    // Mark as deleted by setting inode to 0
                    // In real implementation, could merge with previous entry
                    let entry_mut: &mut Ext4DirEntry2 = unsafe {
                        &mut *(block_data.as_mut_ptr().add(offset) as *mut Ext4DirEntry2)
                    };
                    entry_mut.inode = 0;

                    // If there's a previous entry, extend it to consume this space
                    if let Some(prev_off) = prev_offset {
                        let prev_entry: &mut Ext4DirEntry2 = unsafe {
                            &mut *(block_data.as_mut_ptr().add(prev_off) as *mut Ext4DirEntry2)
                        };
                        prev_entry.rec_len += entry.rec_len;
                    }

                    self.device.write(block_num, &block_data)?;
                    return Ok(removed_inode);
                }

                prev_offset = Some(offset);
                offset += entry.rec_len as usize;
            }
        }

        Err(Errno::ENOENT)
    }

    /// Find directory entry by name
    pub fn find_dir_entry(&self, dir_inode_num: u32, entry_name: &str) -> Result<u32> {
        let dir_inode = self.read_inode(dir_inode_num)?;
        if !dir_inode.is_dir() {
            return Err(Errno::ENOTDIR);
        }

        let sb = self.superblock.lock();
        let block_size = sb.block_size() as usize;
        drop(sb);

        let num_blocks = (dir_inode.size() as usize + block_size - 1) / block_size;

        for block_idx in 0..num_blocks {
            let block_num = self.get_inode_block(&dir_inode, block_idx as u64)?;
            let mut block_data = vec![0u8; block_size];
            self.device.read(block_num, &mut block_data)?;

            let mut offset = 0;

            while offset < block_size {
                if offset + Ext4DirEntry2::MIN_SIZE > block_size {
                    break;
                }

                let entry: &Ext4DirEntry2 = unsafe {
                    &*(block_data.as_ptr().add(offset) as *const Ext4DirEntry2)
                };

                if entry.rec_len == 0 {
                    break;
                }

                // Get entry name
                let name_offset = offset + Ext4DirEntry2::MIN_SIZE;
                let name_slice = &block_data[name_offset..name_offset + entry.name_len as usize];

                if entry.inode != 0 && name_slice == entry_name.as_bytes() {
                    return Ok(entry.inode);
                }

                offset += entry.rec_len as usize;
            }
        }

        Err(Errno::ENOENT)
    }

    /// Helper: Write directory entry at given offset
    fn write_dir_entry(&self, block_data: &mut [u8], offset: usize, inode: u32, name: &str, file_type: Ext4FileType, rec_len: u16) -> Result<()> {
        if offset + Ext4DirEntry2::MIN_SIZE + name.len() > block_data.len() {
            return Err(Errno::ENOSPC);
        }

        let entry = Ext4DirEntry2 {
            inode,
            rec_len,
            name_len: name.len() as u8,
            file_type: file_type as u8,
        };

        // Write entry header
        let entry_bytes = unsafe {
            core::slice::from_raw_parts(&entry as *const Ext4DirEntry2 as *const u8, Ext4DirEntry2::MIN_SIZE)
        };
        block_data[offset..offset + Ext4DirEntry2::MIN_SIZE].copy_from_slice(entry_bytes);

        // Write name
        let name_offset = offset + Ext4DirEntry2::MIN_SIZE;
        block_data[name_offset..name_offset + name.len()].copy_from_slice(name.as_bytes());

        Ok(())
    }

    /// Helper: Get physical block number for inode's logical block
    ///
    /// Simplified version - assumes direct blocks only (no extents yet)
    fn get_inode_block(&self, inode: &Ext4Inode, logical_block: u64) -> Result<u64> {
        if logical_block < 12 {
            // Direct blocks
            let block = inode.i_block[logical_block as usize] as u64;
            if block == 0 {
                return Err(Errno::EINVAL);
            }
            Ok(block)
        } else {
            // TODO: Handle indirect blocks / extent tree
            Err(Errno::ENOSYS)
        }
    }

    /// Write block group descriptor back to disk
    fn write_block_group_desc(&self, group_idx: usize, desc: &Ext4BlockGroupDesc) -> Result<()> {
        let sb = self.superblock.lock();
        let block_size = sb.block_size();

        let bg_table_block = if block_size == 1024 { 2 } else { 1 };
        let desc_size = if (sb.s_feature_incompat & EXT4_FEATURE_INCOMPAT_64BIT) != 0 {
            sb.s_desc_size as usize
        } else {
            32
        };

        let offset = group_idx * desc_size;
        let block = bg_table_block + (offset / block_size as usize) as u64;
        let block_offset = offset % block_size as usize;

        // Read block, modify descriptor, write back
        let mut buf = vec![0u8; block_size as usize];
        self.device.read(block, &mut buf)?;

        let desc_bytes = unsafe {
            core::slice::from_raw_parts(desc as *const Ext4BlockGroupDesc as *const u8, desc_size)
        };
        buf[block_offset..block_offset + desc_size].copy_from_slice(desc_bytes);

        self.device.write(block, &buf)?;
        Ok(())
    }
}

/// Bitmap helper: Find first zero bit (free block/inode)
fn find_first_zero_bit(bitmap: &[u8]) -> Option<u32> {
    for (byte_idx, &byte) in bitmap.iter().enumerate() {
        if byte != 0xFF {
            // Found a byte with at least one zero bit
            for bit_idx in 0..8 {
                if (byte & (1 << bit_idx)) == 0 {
                    return Some((byte_idx * 8 + bit_idx) as u32);
                }
            }
        }
    }
    None
}

/// Bitmap helper: Set bit (mark as used)
fn set_bit(bitmap: &mut [u8], bit_offset: u32) {
    let byte_idx = (bit_offset / 8) as usize;
    let bit_idx = bit_offset % 8;
    if byte_idx < bitmap.len() {
        bitmap[byte_idx] |= 1 << bit_idx;
    }
}

/// Bitmap helper: Clear bit (mark as free)
fn clear_bit(bitmap: &mut [u8], bit_offset: u32) {
    let byte_idx = (bit_offset / 8) as usize;
    let bit_idx = bit_offset % 8;
    if byte_idx < bitmap.len() {
        bitmap[byte_idx] &= !(1 << bit_idx);
    }
}

/// Bitmap helper: Test bit (check if used)
fn test_bit(bitmap: &[u8], bit_offset: u32) -> bool {
    let byte_idx = (bit_offset / 8) as usize;
    let bit_idx = bit_offset % 8;
    if byte_idx < bitmap.len() {
        (bitmap[byte_idx] & (1 << bit_idx)) != 0
    } else {
        false
    }
}

/// Get filesystem statistics
pub fn get_stats(fs: &Ext4FileSystem) -> Ext4Stats {
    let sb = fs.superblock.lock();

    Ext4Stats {
        total_blocks: sb.s_blocks_count,
        free_blocks: sb.s_free_blocks_count,
        total_inodes: sb.s_inodes_count,
        free_inodes: sb.s_free_inodes_count,
        block_size: sb.block_size(),
        has_journal: sb.has_journal(),
        mount_count: sb.s_mnt_count,
    }
}

/// Filesystem statistics
#[derive(Debug, Clone, Copy)]
pub struct Ext4Stats {
    pub total_blocks: u32,
    pub free_blocks: u32,
    pub total_inodes: u32,
    pub free_inodes: u32,
    pub block_size: u32,
    pub has_journal: bool,
    pub mount_count: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superblock_block_size() {
        let mut sb: Ext4Superblock = unsafe { core::mem::zeroed() };
        sb.s_log_block_size = 0;
        assert_eq!(sb.block_size(), 1024);

        sb.s_log_block_size = 1;
        assert_eq!(sb.block_size(), 2048);

        sb.s_log_block_size = 2;
        assert_eq!(sb.block_size(), 4096);
    }

    #[test]
    fn test_has_journal() {
        let mut sb: Ext4Superblock = unsafe { core::mem::zeroed() };
        assert!(!sb.has_journal());

        sb.s_feature_compat = EXT4_FEATURE_COMPAT_HAS_JOURNAL;
        assert!(sb.has_journal());
    }
}
