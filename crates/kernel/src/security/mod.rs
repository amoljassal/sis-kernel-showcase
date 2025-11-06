/// Security subsystem (Phase D)
///
/// Provides credentials, permissions, and access control

pub mod cred;
pub mod perm;

pub use cred::{Credentials, init_credentials, current_cred, set_current_cred};
pub use perm::{Permission, inode_permission, check_permission};
