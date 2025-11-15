// /dev/ptmx - PTY master multiplexer
// Opens a new PTY pair and returns the master FD

use super::file::{File, FileOps, OpenFlags};
use crate::lib::error::Errno;
use crate::drivers::char::pty::create_pty_pair;

/// /dev/ptmx device operations
/// This is NOT used for normal read/write - it's only used for open()
/// When sys_openat opens /dev/ptmx, it should call create_pty_pair()
/// and return a File wrapping the PtyMaster.
pub struct PtmxOps;

impl FileOps for PtmxOps {
    fn read(&self, _file: &File, _buf: &mut [u8]) -> Result<usize, Errno> {
        // /dev/ptmx is not meant to be read directly
        // It should only be opened, which creates a new PTY pair
        Err(Errno::EINVAL)
    }

    fn write(&self, _file: &File, _buf: &[u8]) -> Result<usize, Errno> {
        // /dev/ptmx is not meant to be written directly
        Err(Errno::EINVAL)
    }
}

pub static PTMX_OPS: PtmxOps = PtmxOps;

/// Handle open of /dev/ptmx - creates a new PTY pair
/// Returns a File wrapping the PtyMaster
/// The slave can be accessed via /dev/pts/N where N is the PTY number
pub fn open_ptmx() -> Result<File, Errno> {
    let (master, _slave) = create_pty_pair()?;

    // Store slave in global registry for /dev/pts/N access
    register_pty_slave(_slave.pty_num(), _slave)?;

    // Return master as a File
    Ok(File::from_pty_master(master))
}

/// Global PTY slave registry for /dev/pts/N access
use alloc::collections::BTreeMap;
use spin::RwLock;
use crate::drivers::char::pty::PtySlave;

static PTY_SLAVES: RwLock<BTreeMap<usize, PtySlave>> = RwLock::new(BTreeMap::new());

fn register_pty_slave(pty_num: usize, slave: PtySlave) -> Result<(), Errno> {
    let mut slaves = PTY_SLAVES.write();
    slaves.insert(pty_num, slave);
    Ok(())
}

pub fn get_pty_slave(pty_num: usize) -> Option<PtySlave> {
    let slaves = PTY_SLAVES.read();
    slaves.get(&pty_num).cloned()
}

pub fn list_pty_slaves() -> alloc::vec::Vec<usize> {
    let slaves = PTY_SLAVES.read();
    slaves.keys().copied().collect()
}
