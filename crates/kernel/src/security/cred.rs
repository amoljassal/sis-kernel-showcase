/// Credentials subsystem
///
/// Manages user/group IDs and process credentials

use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};

/// Process credentials
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Real user ID
    pub uid: u32,
    /// Effective user ID (for permission checks)
    pub euid: u32,
    /// Saved set-user-ID
    pub suid: u32,
    /// Real group ID
    pub gid: u32,
    /// Effective group ID (for permission checks)
    pub egid: u32,
    /// Saved set-group-ID
    pub sgid: u32,
    /// Supplementary groups
    pub groups: Vec<u32>,
}

impl Credentials {
    /// Create root credentials (UID/GID = 0)
    pub fn root() -> Self {
        Self {
            uid: 0,
            euid: 0,
            suid: 0,
            gid: 0,
            egid: 0,
            sgid: 0,
            groups: Vec::new(),
        }
    }

    /// Create unprivileged credentials
    pub fn new(uid: u32, gid: u32) -> Self {
        Self {
            uid,
            euid: uid,
            suid: uid,
            gid,
            egid: gid,
            sgid: gid,
            groups: Vec::new(),
        }
    }

    /// Check if credentials have root privileges
    pub fn is_root(&self) -> bool {
        self.euid == 0
    }

    /// Check if effective UID matches
    pub fn euid_matches(&self, uid: u32) -> bool {
        self.euid == uid
    }

    /// Check if effective GID matches or in supplementary groups
    pub fn in_group(&self, gid: u32) -> bool {
        self.egid == gid || self.groups.contains(&gid)
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Self::root()
    }
}

/// Global credentials for current process (simplified for Phase D)
/// In production, this would be per-process in the task struct
static CURRENT_CRED: Mutex<Credentials> = Mutex::new(Credentials {
    uid: 0,
    euid: 0,
    suid: 0,
    gid: 0,
    egid: 0,
    sgid: 0,
    groups: Vec::new(),
});

/// Initialize credentials subsystem
pub fn init_credentials() {
    *CURRENT_CRED.lock() = Credentials::root();
    crate::info!("security: Credentials subsystem initialized");
}

/// Get current process credentials
pub fn current_cred() -> Credentials {
    CURRENT_CRED.lock().clone()
}

/// Set current process credentials
pub fn set_current_cred(cred: Credentials) {
    *CURRENT_CRED.lock() = cred;
}

/// Get current UID
pub fn current_uid() -> u32 {
    CURRENT_CRED.lock().uid
}

/// Get current effective UID
pub fn current_euid() -> u32 {
    CURRENT_CRED.lock().euid
}

/// Get current GID
pub fn current_gid() -> u32 {
    CURRENT_CRED.lock().gid
}

/// Get current effective GID
pub fn current_egid() -> u32 {
    CURRENT_CRED.lock().egid
}

/// Set UID (requires root or matching UID)
pub fn set_uid(uid: u32) -> crate::lib::error::Result<()> {
    let mut cred = CURRENT_CRED.lock();

    // Root can set to any UID
    if cred.euid == 0 {
        cred.uid = uid;
        cred.euid = uid;
        cred.suid = uid;
        return Ok(());
    }

    // Non-root can only set to real UID or saved UID
    if uid == cred.uid || uid == cred.suid {
        cred.euid = uid;
        return Ok(());
    }

    Err(crate::lib::error::Errno::EPERM)
}

/// Set GID (requires root or matching GID)
pub fn set_gid(gid: u32) -> crate::lib::error::Result<()> {
    let mut cred = CURRENT_CRED.lock();

    // Root can set to any GID
    if cred.euid == 0 {
        cred.gid = gid;
        cred.egid = gid;
        cred.sgid = gid;
        return Ok(());
    }

    // Non-root can only set to real GID or saved GID
    if gid == cred.gid || gid == cred.sgid {
        cred.egid = gid;
        return Ok(());
    }

    Err(crate::lib::error::Errno::EPERM)
}

/// Set effective UID
pub fn set_euid(euid: u32) -> crate::lib::error::Result<()> {
    let mut cred = CURRENT_CRED.lock();

    // Root can set to any EUID
    if cred.euid == 0 {
        cred.euid = euid;
        return Ok(());
    }

    // Non-root can set to real UID or saved UID
    if euid == cred.uid || euid == cred.suid {
        cred.euid = euid;
        return Ok(());
    }

    Err(crate::lib::error::Errno::EPERM)
}

/// Set effective GID
pub fn set_egid(egid: u32) -> crate::lib::error::Result<()> {
    let mut cred = CURRENT_CRED.lock();

    // Root can set to any EGID
    if cred.euid == 0 {
        cred.egid = egid;
        return Ok(());
    }

    // Non-root can set to real GID or saved GID
    if egid == cred.gid || egid == cred.sgid {
        cred.egid = egid;
        return Ok(());
    }

    Err(crate::lib::error::Errno::EPERM)
}
