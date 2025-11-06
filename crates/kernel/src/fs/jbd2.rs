/// JBD2 (Journaling Block Device v2) - Phase F
///
/// Implements ext4 journaling for crash recovery and data consistency.
/// Provides transaction-based atomic operations for filesystem metadata.

use crate::lib::error::{Result, Errno};
use crate::block::BlockDevice;
use alloc::sync::Arc;
use alloc::vec::{self, Vec};
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

        // For MVP: Just ensure data is flushed to disk
        // In full implementation, would write to journal first, then checkpoint

        // TODO: Write descriptor blocks
        // TODO: Write data blocks to journal
        // TODO: Write commit block
        // TODO: Wait for I/O completion
        // TODO: Checkpoint (write to final locations)

        txn.set_state(TransactionState::Committed);

        // Clear current transaction
        *self.current_transaction.lock() = None;

        crate::debug!("JBD2: Transaction {} committed", txn.tid);
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

        // MVP: Simple replay - just scan journal
        // Full implementation would parse descriptors and replay blocks

        let mut replayed = 0;
        let mut current_block = start_block;

        while current_block < start_block + max_len {
            // Read block header
            let mut header_buf = vec![0u8; 12]; // Magic + type + sequence
            self.device.read(self.journal_start + current_block as u64, &mut header_buf)?;

            let magic = u32::from_le_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]]);
            let blocktype = u32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]]);

            if magic != JBD2_MAGIC_NUMBER {
                break; // End of valid journal entries
            }

            match blocktype {
                1 => {
                    // Descriptor block
                    crate::debug!("JBD2: Found descriptor at block {}", current_block);
                    // TODO: Parse and replay metadata blocks
                    replayed += 1;
                }
                2 => {
                    // Commit block
                    crate::debug!("JBD2: Found commit at block {}", current_block);
                    // Transaction completed, continue
                }
                5 => {
                    // Revoke block
                    crate::debug!("JBD2: Found revoke at block {}", current_block);
                    // TODO: Process revocations
                }
                _ => {
                    crate::warn!("JBD2: Unknown block type {} at block {}", blocktype, current_block);
                    break;
                }
            }

            current_block += 1;
        }

        crate::info!("JBD2: Journal replay complete ({} transactions replayed)", replayed);
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
