/// JBD2 (Journaling Block Device v2) - Phase F
///
/// Implements ext4 journaling for crash recovery and data consistency.
/// Provides transaction-based atomic operations for filesystem metadata.

use crate::lib::error::{Result, Errno};
use crate::block::BlockDevice;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use spin::Mutex;

/// JBD2 magic numbers
pub const JBD2_MAGIC_NUMBER: u32 = 0xC03B3998;

/// Journal block types
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalBlockType {
    /// Descriptor block (metadata blocks follow)
    Descriptor = 1,
    /// Commit block (marks end of transaction)
    Commit = 2,
    /// Superblock (v1)
    SuperblockV1 = 3,
    /// Superblock (v2)
    SuperblockV2 = 4,
    /// Revoke block (invalidates previous blocks)
    Revoke = 5,
}

/// Journal superblock (first block of journal)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JournalSuperblock {
    /// Magic number (JBD2_MAGIC_NUMBER)
    pub s_magic: u32,
    /// Block type (SuperblockV1 or SuperblockV2)
    pub s_blocktype: u32,
    /// Sequence number
    pub s_sequence: u32,

    /// Journal block size
    pub s_blocksize: u32,
    /// Total number of blocks in journal
    pub s_maxlen: u32,
    /// First block of log
    pub s_first: u32,

    /// Sequence number of first transaction
    pub s_sequence_start: u32,
    /// Block number of start of log
    pub s_start: u32,
    /// Error code
    pub s_errno: i32,

    /// Feature compatibility flags
    pub s_feature_compat: u32,
    /// Feature incompatibility flags
    pub s_feature_incompat: u32,
    /// Read-only compatible feature flags
    pub s_feature_ro_compat: u32,

    /// UUID of journal
    pub s_uuid: [u8; 16],

    /// Number of filesystem blocks per journal block
    pub s_nr_users: u32,
    /// Block number of dynamic superblock copy
    pub s_dynsuper: u32,

    /// Limit of journal blocks per transaction
    pub s_max_transaction: u32,
    /// Limit of filesystem blocks per transaction
    pub s_max_trans_data: u32,

    /// Padding to 1024 bytes
    pub s_padding: [u32; 176],
}

/// Journal descriptor block header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JournalDescriptor {
    /// Magic number
    pub h_magic: u32,
    /// Block type (Descriptor)
    pub h_blocktype: u32,
    /// Sequence number
    pub h_sequence: u32,
}

/// Journal descriptor block tag
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JournalBlockTag {
    /// Filesystem block number
    pub t_blocknr: u32,
    /// Tag flags
    pub t_flags: u32,
}

/// Tag flags
pub const JBD2_FLAG_ESCAPE: u32 = 1;      // Block had to be escaped
pub const JBD2_FLAG_SAME_UUID: u32 = 2;   // UUID is same as previous
pub const JBD2_FLAG_DELETED: u32 = 4;     // Block deleted
pub const JBD2_FLAG_LAST_TAG: u32 = 8;    // Last tag in descriptor

/// Journal commit block
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JournalCommit {
    /// Magic number
    pub h_magic: u32,
    /// Block type (Commit)
    pub h_blocktype: u32,
    /// Sequence number
    pub h_sequence: u32,
    /// Commit time (seconds)
    pub h_commit_sec: u32,
    /// Commit time (nanoseconds)
    pub h_commit_nsec: u32,
}

/// Journal revoke block
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JournalRevoke {
    /// Magic number
    pub h_magic: u32,
    /// Block type (Revoke)
    pub h_blocktype: u32,
    /// Sequence number
    pub h_sequence: u32,
    /// Number of revoked blocks
    pub r_count: u32,
}

/// Transaction handle
pub struct TransactionHandle {
    /// Transaction ID
    pub tid: u32,
    /// Blocks modified in this transaction
    pub blocks: Mutex<Vec<u64>>,
    /// Transaction state
    pub state: Mutex<TransactionState>,
}

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction is being built
    Running,
    /// Transaction is locked (committing)
    Locked,
    /// Transaction commit in progress
    Committing,
    /// Transaction committed
    Committed,
    /// Transaction aborted
    Aborted,
}

impl TransactionHandle {
    /// Create a new transaction
    pub fn new(tid: u32) -> Self {
        Self {
            tid,
            blocks: Mutex::new(Vec::new()),
            state: Mutex::new(TransactionState::Running),
        }
    }

    /// Add a block to this transaction
    pub fn add_block(&self, block: u64) {
        let mut blocks = self.blocks.lock();
        if !blocks.contains(&block) {
            blocks.push(block);
        }
    }

    /// Get transaction state
    pub fn state(&self) -> TransactionState {
        *self.state.lock()
    }

    /// Set transaction state
    pub fn set_state(&self, state: TransactionState) {
        *self.state.lock() = state;
    }
}

/// Journal handle
pub struct Journal {
    /// Block device
    pub device: Arc<BlockDevice>,
    /// Journal superblock
    pub superblock: Mutex<JournalSuperblock>,
    /// Current transaction
    pub current_transaction: Mutex<Option<Arc<TransactionHandle>>>,
    /// Next transaction ID
    pub next_tid: Mutex<u32>,
    /// Journal block offset (start of journal on device)
    pub journal_start: u64,
}

impl Journal {
    /// Load journal from device
    pub fn load(device: Arc<BlockDevice>, journal_block: u64) -> Result<Arc<Self>> {
        // Read journal superblock (first block of journal)
        let mut sb_buf = vec![0u8; 4096];
        device.read(journal_block, &mut sb_buf)?;

        // Parse superblock
        let sb: JournalSuperblock = unsafe {
            core::ptr::read(sb_buf.as_ptr() as *const JournalSuperblock)
        };

        // Verify magic number
        if sb.s_magic != JBD2_MAGIC_NUMBER {
            crate::error!("JBD2: Invalid journal magic: {:#x}", sb.s_magic);
            return Err(Errno::EINVAL);
        }

        crate::info!("JBD2: Loaded journal (size={} blocks, seq={}, start={})",
                     sb.s_maxlen, sb.s_sequence, sb.s_start);

        Ok(Arc::new(Self {
            device,
            superblock: Mutex::new(sb),
            current_transaction: Mutex::new(None),
            next_tid: Mutex::new(sb.s_sequence + 1),
            journal_start: journal_block,
        }))
    }

    /// Begin a new transaction
    pub fn begin_transaction(&self) -> Arc<TransactionHandle> {
        let mut current = self.current_transaction.lock();
        let mut next_tid = self.next_tid.lock();

        // If there's already a running transaction, return it
        if let Some(ref txn) = *current {
            if txn.state() == TransactionState::Running {
                return txn.clone();
            }
        }

        // Create new transaction
        let tid = *next_tid;
        *next_tid += 1;

        let txn = Arc::new(TransactionHandle::new(tid));
        *current = Some(txn.clone());

        crate::debug!("JBD2: Started transaction {}", tid);
        txn
    }

    /// Commit current transaction
    pub fn commit_transaction(&self, txn: Arc<TransactionHandle>) -> Result<()> {
        if txn.state() != TransactionState::Running {
            return Err(Errno::EINVAL);
        }

        txn.set_state(TransactionState::Locked);

        let blocks = txn.blocks.lock();
        let num_blocks = blocks.len();

        crate::info!("JBD2: Committing transaction {} ({} blocks)", txn.tid, num_blocks);

        if num_blocks == 0 {
            // Empty transaction, just mark as committed
            txn.set_state(TransactionState::Committed);
            *self.current_transaction.lock() = None;
            return Ok(());
        }

        let mut sb = self.superblock.lock();
        let block_size = sb.s_blocksize as usize;

        // Find next free journal block
        let mut journal_block = sb.s_start;
        if journal_block >= sb.s_maxlen {
            journal_block = sb.s_first; // Wrap around
        }

        // 1. Write descriptor block
        let descriptor_block = journal_block;
        journal_block += 1;

        let mut desc_buf = vec![0u8; block_size];
        let desc = JournalDescriptor {
            h_magic: JBD2_MAGIC_NUMBER,
            h_blocktype: JournalBlockType::Descriptor as u32,
            h_sequence: txn.tid,
        };

        // Write descriptor header
        let desc_bytes = unsafe {
            core::slice::from_raw_parts(&desc as *const JournalDescriptor as *const u8, 12)
        };
        desc_buf[0..12].copy_from_slice(desc_bytes);

        // Write block tags after descriptor header
        let mut tag_offset = 12;
        for (i, &block_num) in blocks.iter().enumerate() {
            if tag_offset + 8 > block_size {
                break; // No more space for tags (simplified - should chain descriptors)
            }

            let is_last = i == blocks.len() - 1;
            let tag = JournalBlockTag {
                t_blocknr: block_num as u32,
                t_flags: if is_last { JBD2_FLAG_LAST_TAG } else { 0 },
            };

            let tag_bytes = unsafe {
                core::slice::from_raw_parts(&tag as *const JournalBlockTag as *const u8, 8)
            };
            desc_buf[tag_offset..tag_offset + 8].copy_from_slice(tag_bytes);
            tag_offset += 8;
        }

        self.device.write(self.journal_start + descriptor_block as u64, &desc_buf)?;

        // 2. Write data blocks to journal
        for &block_num in blocks.iter() {
            if journal_block >= sb.s_maxlen {
                journal_block = sb.s_first; // Wrap around
            }

            // Read metadata block from filesystem
            let mut block_data = vec![0u8; block_size];
            self.device.read(block_num, &mut block_data)?;

            // Write to journal
            self.device.write(self.journal_start + journal_block as u64, &block_data)?;
            journal_block += 1;
        }

        // 3. Write commit block
        let commit_block = journal_block;
        let mut commit_buf = vec![0u8; block_size];
        let commit = JournalCommit {
            h_magic: JBD2_MAGIC_NUMBER,
            h_blocktype: JournalBlockType::Commit as u32,
            h_sequence: txn.tid,
            h_commit_sec: 0,   // Would use real timestamp
            h_commit_nsec: 0,
        };

        let commit_bytes = unsafe {
            core::slice::from_raw_parts(&commit as *const JournalCommit as *const u8, 20)
        };
        commit_buf[0..20].copy_from_slice(commit_bytes);

        self.device.write(self.journal_start + commit_block as u64, &commit_buf)?;

        // 4. Update journal superblock
        sb.s_start = commit_block + 1;
        sb.s_sequence = txn.tid + 1;

        let sb_buf = unsafe {
            core::slice::from_raw_parts(&*sb as *const JournalSuperblock as *const u8, core::mem::size_of::<JournalSuperblock>())
        };
        let mut sb_write_buf = vec![0u8; block_size];
        sb_write_buf[0..sb_buf.len()].copy_from_slice(sb_buf);
        self.device.write(self.journal_start, &sb_write_buf)?;

        drop(sb);
        drop(blocks);

        txn.set_state(TransactionState::Committed);

        // Clear current transaction
        *self.current_transaction.lock() = None;

        crate::debug!("JBD2: Transaction {} committed (desc={}, commit={})", txn.tid, descriptor_block, commit_block);
        Ok(())
    }

    /// Replay journal on mount (crash recovery)
    pub fn replay(&self) -> Result<()> {
        let sb = self.superblock.lock();

        crate::info!("JBD2: Replaying journal from sequence {}", sb.s_sequence);

        let start_block = sb.s_start;
        let max_len = sb.s_maxlen;

        if start_block == 0 {
            crate::info!("JBD2: Journal is clean, no replay needed");
            return Ok(());
        }

        // Full journal replay: parse descriptors and replay metadata blocks
        let block_size = sb.s_blocksize as usize;
        let mut replayed_txns = 0;
        let mut replayed_blocks = 0;
        let mut current_block = start_block;

        // Revoke list (blocks that should not be replayed)
        let mut revoked: Vec<u64> = Vec::new();

        while current_block < start_block + max_len {
            // Read block header
            let mut header_buf = vec![0u8; 12]; // Magic + type + sequence
            self.device.read(self.journal_start + current_block as u64, &mut header_buf)?;

            let magic = u32::from_le_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]]);
            let blocktype = u32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]]);
            let sequence = u32::from_le_bytes([header_buf[8], header_buf[9], header_buf[10], header_buf[11]]);

            if magic != JBD2_MAGIC_NUMBER {
                break; // End of valid journal entries
            }

            match blocktype {
                1 => {
                    // Descriptor block - parse tags and replay
                    crate::debug!("JBD2: Replaying descriptor at block {} (seq={})", current_block, sequence);

                    // Read full descriptor block
                    let mut desc_buf = vec![0u8; block_size];
                    self.device.read(self.journal_start + current_block as u64, &mut desc_buf)?;

                    // Parse tags (starting at offset 12 after header)
                    let mut tag_offset = 12;
                    let mut journal_data_block = current_block + 1;

                    loop {
                        if tag_offset + 8 > block_size {
                            break;
                        }

                        // Parse tag
                        let tag_blocknr = u32::from_le_bytes([
                            desc_buf[tag_offset],
                            desc_buf[tag_offset + 1],
                            desc_buf[tag_offset + 2],
                            desc_buf[tag_offset + 3],
                        ]) as u64;

                        let tag_flags = u32::from_le_bytes([
                            desc_buf[tag_offset + 4],
                            desc_buf[tag_offset + 5],
                            desc_buf[tag_offset + 6],
                            desc_buf[tag_offset + 7],
                        ]);

                        // Check if block is revoked
                        if !revoked.contains(&tag_blocknr) {
                            // Read data block from journal
                            let mut data_buf = vec![0u8; block_size];
                            self.device.read(self.journal_start + journal_data_block as u64, &mut data_buf)?;

                            // Write to final filesystem location
                            self.device.write(tag_blocknr, &data_buf)?;

                            replayed_blocks += 1;
                            crate::debug!("JBD2: Replayed block {} -> {}", journal_data_block, tag_blocknr);
                        } else {
                            crate::debug!("JBD2: Skipped revoked block {}", tag_blocknr);
                        }

                        journal_data_block += 1;
                        tag_offset += 8;

                        // Check for last tag
                        if (tag_flags & JBD2_FLAG_LAST_TAG) != 0 {
                            break;
                        }
                    }

                    current_block = journal_data_block;
                }
                2 => {
                    // Commit block - transaction completed successfully
                    crate::debug!("JBD2: Found commit at block {} (seq={})", current_block, sequence);
                    replayed_txns += 1;
                    current_block += 1;
                }
                5 => {
                    // Revoke block - add blocks to revoke list
                    crate::debug!("JBD2: Processing revoke block at {}", current_block);

                    let mut revoke_buf = vec![0u8; block_size];
                    self.device.read(self.journal_start + current_block as u64, &mut revoke_buf)?;

                    // Parse revoke count (at offset 12)
                    let r_count = u32::from_le_bytes([
                        revoke_buf[12],
                        revoke_buf[13],
                        revoke_buf[14],
                        revoke_buf[15],
                    ]);

                    // Parse revoked block numbers (starting at offset 16)
                    let mut offset = 16;
                    for _ in 0..r_count {
                        if offset + 8 > block_size {
                            break;
                        }

                        let revoked_block = u64::from_le_bytes([
                            revoke_buf[offset],
                            revoke_buf[offset + 1],
                            revoke_buf[offset + 2],
                            revoke_buf[offset + 3],
                            revoke_buf[offset + 4],
                            revoke_buf[offset + 5],
                            revoke_buf[offset + 6],
                            revoke_buf[offset + 7],
                        ]);

                        revoked.push(revoked_block);
                        offset += 8;
                    }

                    crate::debug!("JBD2: Added {} blocks to revoke list", r_count);
                    current_block += 1;
                }
                _ => {
                    crate::warn!("JBD2: Unknown block type {} at block {}", blocktype, current_block);
                    break;
                }
            }
        }

        // Clear journal after successful replay
        drop(sb);
        let mut sb = self.superblock.lock();
        sb.s_start = 0;
        sb.s_sequence_start = sb.s_sequence + 1;

        let sb_buf = unsafe {
            core::slice::from_raw_parts(&*sb as *const JournalSuperblock as *const u8, core::mem::size_of::<JournalSuperblock>())
        };
        let mut sb_write_buf = vec![0u8; block_size];
        sb_write_buf[0..sb_buf.len()].copy_from_slice(sb_buf);
        self.device.write(self.journal_start, &sb_write_buf)?;

        crate::info!("JBD2: Journal replay complete ({} transactions, {} blocks replayed)", replayed_txns, replayed_blocks);
        Ok(())
    }

    /// Abort transaction
    pub fn abort_transaction(&self, txn: Arc<TransactionHandle>) {
        txn.set_state(TransactionState::Aborted);
        *self.current_transaction.lock() = None;
        crate::warn!("JBD2: Transaction {} aborted", txn.tid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_transaction() {
        let txn = TransactionHandle::new(1);
        assert_eq!(txn.state(), TransactionState::Running);

        txn.add_block(100);
        txn.add_block(200);
        txn.add_block(100); // Duplicate

        let blocks = txn.blocks.lock();
        assert_eq!(blocks.len(), 2); // No duplicates
    }
}
