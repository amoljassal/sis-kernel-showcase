/// Filesystem Layer - Phase F
///
/// Provides journaling filesystem support with ext4 and JBD2.

pub mod jbd2;
pub mod ext4;

pub use jbd2::{Journal, TransactionHandle, JBD2_MAGIC_NUMBER};
pub use ext4::{Ext4FileSystem, Ext4Superblock, Ext4Stats, get_stats};
