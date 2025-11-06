/// Permission checking
///
/// Unix-style permission checks for inodes

use crate::lib::error::{Result, Errno};
use crate::security::Credentials;
use crate::vfs::Inode;
use alloc::vec::Vec;

/// Permission bits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Read = 0o4,
    Write = 0o2,
    Execute = 0o1,
}

impl Permission {
    /// Get all permissions
    pub fn all() -> Vec<Self> {
        vec![Self::Read, Self::Write, Self::Execute]
    }

    /// Convert to mode bits
    pub fn to_bits(self) -> u32 {
        self as u32
    }
}

/// Check if credentials have permission to access inode
///
/// Uses Unix permission model:
/// 1. If owner matches, use owner bits
/// 2. Else if group matches, use group bits
/// 3. Else use other bits
///
/// Root (UID 0) bypasses all checks except execute (needs at least one +x)
pub fn inode_permission(cred: &Credentials, inode: &Inode, req: Permission) -> bool {
    let meta = inode.meta.read();

    // Root bypasses permission checks (except execute)
    if cred.euid == 0 {
        if req == Permission::Execute {
            // Root needs at least one execute bit set
            let mode = meta.mode;
            return (mode & 0o111) != 0;
        }
        return true;
    }

    let mode = meta.mode;
    let uid = meta.uid;
    let gid = meta.gid;

    // Owner check
    if cred.euid == uid {
        let owner_bits = (mode >> 6) & 0o7;
        return (owner_bits & req.to_bits()) != 0;
    }

    // Group check
    if cred.in_group(gid) {
        let group_bits = (mode >> 3) & 0o7;
        return (group_bits & req.to_bits()) != 0;
    }

    // Other check
    let other_bits = mode & 0o7;
    (other_bits & req.to_bits()) != 0
}

/// Check permission and return error if denied
pub fn check_permission(cred: &Credentials, inode: &Inode, req: Permission) -> Result<()> {
    if inode_permission(cred, inode, req) {
        Ok(())
    } else {
        Err(Errno::EACCES)
    }
}

/// Check if credentials can modify inode ownership
pub fn can_chown(cred: &Credentials, inode: &Inode, new_uid: u32, new_gid: u32) -> Result<()> {
    let meta = inode.meta.read();

    // Root can always chown
    if cred.euid == 0 {
        return Ok(());
    }

    // Owner can change group to one they're a member of
    if cred.euid == meta.uid {
        // Can't change UID (only root can do that)
        if new_uid != meta.uid && new_uid != u32::MAX {
            return Err(Errno::EPERM);
        }

        // Can change to a group we're in
        if new_gid != u32::MAX && !cred.in_group(new_gid) {
            return Err(Errno::EPERM);
        }

        return Ok(());
    }

    Err(Errno::EPERM)
}

/// Check if credentials can modify inode mode
pub fn can_chmod(cred: &Credentials, inode: &Inode) -> Result<()> {
    let meta = inode.meta.read();

    // Root can always chmod
    if cred.euid == 0 {
        return Ok(());
    }

    // Owner can chmod
    if cred.euid == meta.uid {
        return Ok(());
    }

    Err(Errno::EPERM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_owner() {
        let cred = Credentials::new(1000, 1000);
        let inode = create_test_inode(1000, 1000, 0o644);

        assert!(inode_permission(&cred, &inode, Permission::Read));
        assert!(inode_permission(&cred, &inode, Permission::Write));
        assert!(!inode_permission(&cred, &inode, Permission::Execute));
    }

    #[test]
    fn test_permission_group() {
        let mut cred = Credentials::new(1001, 1000);
        let inode = create_test_inode(1000, 1000, 0o640);

        assert!(inode_permission(&cred, &inode, Permission::Read));
        assert!(!inode_permission(&cred, &inode, Permission::Write));
    }

    #[test]
    fn test_permission_other() {
        let cred = Credentials::new(1001, 1001);
        let inode = create_test_inode(1000, 1000, 0o604);

        assert!(!inode_permission(&cred, &inode, Permission::Read));
        assert!(!inode_permission(&cred, &inode, Permission::Write));
    }

    #[test]
    fn test_permission_root() {
        let cred = Credentials::root();
        let inode = create_test_inode(1000, 1000, 0o000);

        assert!(inode_permission(&cred, &inode, Permission::Read));
        assert!(inode_permission(&cred, &inode, Permission::Write));
    }
}
