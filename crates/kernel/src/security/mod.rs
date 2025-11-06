/// Security subsystem (Phase D)
///
/// Provides credentials, permissions, and access control

pub mod cred;
pub mod perm;
pub mod random;

pub use cred::{
    Credentials, init_credentials, current_cred, set_current_cred,
    current_uid, current_euid, current_gid, current_egid,
    set_uid, set_euid, set_gid, set_egid,
};
pub use perm::{Permission, inode_permission, check_permission};
pub use random::{init as init_random, fill_random_bytes, random_u64, random_u32, random_range};
