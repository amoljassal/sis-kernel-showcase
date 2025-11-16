// Procfs implementation for Phase A1
// Provides /proc filesystem for process and system information

use super::inode::{Inode, InodeType, InodeOps, Ino, DirEntry, alloc_ino};
use crate::lib::error::{Errno, Result};
use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use spin::Mutex;

/// Procfs root inode
pub struct ProcfsRoot;

impl InodeOps for ProcfsRoot {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 1,
            itype: InodeType::Directory,
            mode: crate::vfs::S_IFDIR | 0o555,
            uid: 0,
            gid: 0,
            nlink: 2,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn lookup(&self, name: &str) -> Result<Arc<Inode>> {
        match name {
            "cpuinfo" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(CpuInfoFile)) as &'static dyn InodeOps,
            ))),
            "meminfo" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(MemInfoFile)) as &'static dyn InodeOps,
            ))),
            "uptime" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(UptimeFile)) as &'static dyn InodeOps,
            ))),
            "mounts" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(MountsFile)) as &'static dyn InodeOps,
            ))),
            #[cfg(feature = "agentsys")]
            "agentsys" => Ok(Arc::new(Inode::new(
                InodeType::Directory,
                0o555,
                Box::leak(Box::new(AgentSysDir)) as &'static dyn InodeOps,
            ))),
            "self" => {
                // /proc/self symlink to current process (Phase A2)
                let pid = crate::process::current_pid();
                Ok(Arc::new(Inode::new(
                    InodeType::Directory,
                    0o555,
                    Box::leak(Box::new(ProcPidDir { pid })) as &'static dyn InodeOps,
                )))
            }
            _ => {
                // Check if it's a PID directory (numeric)
                if let Ok(pid) = name.parse::<u32>() {
                    // Verify PID exists
                    let table = crate::process::get_process_table();
                    if let Some(ref table) = *table {
                        if table.get(pid).is_some() {
                            return Ok(Arc::new(Inode::new(
                                InodeType::Directory,
                                0o555,
                                Box::leak(Box::new(ProcPidDir { pid })) as &'static dyn InodeOps,
                            )));
                        }
                    }
                }
                Err(Errno::ENOENT)
            }
        }
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::EROFS)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        let mut entries = Vec::new();

        // Add . and ..
        entries.push(DirEntry {
            ino: 1,
            name: ".".to_string(),
            itype: InodeType::Directory,
        });
        entries.push(DirEntry {
            ino: 1,
            name: "..".to_string(),
            itype: InodeType::Directory,
        });

        // Add static files
        entries.push(DirEntry {
            ino: 2,
            name: "cpuinfo".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 3,
            name: "meminfo".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 4,
            name: "uptime".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 5,
            name: "mounts".to_string(),
            itype: InodeType::Regular,
        });

        // Add agentsys directory if feature is enabled
        #[cfg(feature = "agentsys")]
        entries.push(DirEntry {
            ino: 6,
            name: "agentsys".to_string(),
            itype: InodeType::Directory,
        });

        // TODO: Add PID directories dynamically

        Ok(entries)
    }

    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize> {
        Err(Errno::EISDIR)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EISDIR)
    }
}

/// /proc/cpuinfo file
struct CpuInfoFile;

impl InodeOps for CpuInfoFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 2,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let content = format!(
            "processor\t: 0\n\
             model name\t: ARM Cortex-A72\n\
             BogoMIPS\t: 100.00\n\
             Features\t: fp asimd\n\
             CPU implementer\t: 0x41\n\
             CPU architecture: 8\n\
             CPU variant\t: 0x0\n\
             CPU part\t: 0xd08\n\
             CPU revision\t: 3\n\n"
        );

        let bytes = content.as_bytes();
        if offset >= bytes.len() as u64 {
            return Ok(0);
        }

        let start = offset as usize;
        let to_copy = (bytes.len() - start).min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
        Ok(to_copy)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

/// /proc/meminfo file
struct MemInfoFile;

impl InodeOps for MemInfoFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 3,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let stats = crate::mm::get_stats().unwrap_or_else(|| crate::mm::buddy::AllocStats {
            total_pages: 0,
            free_pages: 0,
            allocated_pages: 0,
            allocation_failures: 0,
        });
        let total_kb = (stats.total_pages * 4) as u64; // 4KB pages
        let free_kb = (stats.free_pages * 4) as u64;
        let used_kb = total_kb - free_kb;

        let content = format!(
            "MemTotal:     {} kB\n\
             MemFree:      {} kB\n\
             MemAvailable: {} kB\n\
             Buffers:      0 kB\n\
             Cached:       0 kB\n\
             SwapTotal:    0 kB\n\
             SwapFree:     0 kB\n",
            total_kb, free_kb, free_kb
        );

        let bytes = content.as_bytes();
        if offset >= bytes.len() as u64 {
            return Ok(0);
        }

        let start = offset as usize;
        let to_copy = (bytes.len() - start).min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
        Ok(to_copy)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

/// /proc/uptime file
struct UptimeFile;

impl InodeOps for UptimeFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 4,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // TODO: Get actual uptime from timer
        // For now, return dummy value
        let content = "100.00 100.00\n";

        let bytes = content.as_bytes();
        if offset >= bytes.len() as u64 {
            return Ok(0);
        }

        let start = offset as usize;
        let to_copy = (bytes.len() - start).min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
        Ok(to_copy)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

/// /proc/mounts file
struct MountsFile;

impl InodeOps for MountsFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 5,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let content = "tmpfs / tmpfs rw,relatime 0 0\n\
                      devfs /dev devfs rw,relatime 0 0\n\
                      proc /proc proc rw,relatime 0 0\n";

        let bytes = content.as_bytes();
        if offset >= bytes.len() as u64 {
            return Ok(0);
        }

        let start = offset as usize;
        let to_copy = (bytes.len() - start).min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
        Ok(to_copy)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

/// /proc/[pid] directory
struct ProcPidDir {
    pid: u32,
}

impl InodeOps for ProcPidDir {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 100 + self.pid as u64,
            itype: InodeType::Directory,
            mode: crate::vfs::S_IFDIR | 0o555,
            uid: 0,
            gid: 0,
            nlink: 2,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn lookup(&self, name: &str) -> Result<Arc<Inode>> {
        match name {
            "cmdline" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(ProcPidCmdline { pid: self.pid })) as &'static dyn InodeOps,
            ))),
            "stat" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(ProcPidStat { pid: self.pid })) as &'static dyn InodeOps,
            ))),
            "status" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(ProcPidStatus { pid: self.pid })) as &'static dyn InodeOps,
            ))),
            "maps" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(ProcPidMaps { pid: self.pid })) as &'static dyn InodeOps,
            ))),
            _ => Err(Errno::ENOENT),
        }
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::EROFS)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        let mut entries = Vec::new();

        // Add . and ..
        entries.push(DirEntry {
            ino: 100 + self.pid as u64,
            name: ".".to_string(),
            itype: InodeType::Directory,
        });
        entries.push(DirEntry {
            ino: 1,
            name: "..".to_string(),
            itype: InodeType::Directory,
        });

        // Add per-process files
        entries.push(DirEntry {
            ino: 200 + self.pid as u64,
            name: "cmdline".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 300 + self.pid as u64,
            name: "stat".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 400 + self.pid as u64,
            name: "status".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 500 + self.pid as u64,
            name: "maps".to_string(),
            itype: InodeType::Regular,
        });

        Ok(entries)
    }

    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize> {
        Err(Errno::EISDIR)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EISDIR)
    }
}

/// /proc/[pid]/cmdline file
struct ProcPidCmdline {
    pid: u32,
}

impl InodeOps for ProcPidCmdline {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 200 + self.pid as u64,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get process name
        let table = crate::process::get_process_table();
        let table = table.as_ref().ok_or(Errno::ESRCH)?;
        let task = table.get(self.pid).ok_or(Errno::ESRCH)?;

        // Format: command\0
        let mut content = task.name.clone();
        content.push('\0');

        let bytes = content.as_bytes();
        if offset >= bytes.len() as u64 {
            return Ok(0);
        }

        let start = offset as usize;
        let to_copy = (bytes.len() - start).min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
        Ok(to_copy)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

/// /proc/[pid]/stat file
struct ProcPidStat {
    pid: u32,
}

impl InodeOps for ProcPidStat {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 300 + self.pid as u64,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get process info
        let table = crate::process::get_process_table();
        let table = table.as_ref().ok_or(Errno::ESRCH)?;
        let task = table.get(self.pid).ok_or(Errno::ESRCH)?;

        // Format stat file (simplified for Phase A1)
        let state = match task.state {
            crate::process::ProcessState::Running => 'R',
            crate::process::ProcessState::Ready => 'R',
            crate::process::ProcessState::Sleeping => 'S',
            crate::process::ProcessState::Stopped => 'T',
            crate::process::ProcessState::Zombie => 'Z',
        };

        let content = format!(
            "{} ({}) {} {} 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
            self.pid,
            task.name,
            state,
            task.ppid,
        );

        let bytes = content.as_bytes();
        if offset >= bytes.len() as u64 {
            return Ok(0);
        }

        let start = offset as usize;
        let to_copy = (bytes.len() - start).min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
        Ok(to_copy)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

/// /proc/[pid]/status file
struct ProcPidStatus {
    pid: u32,
}

impl InodeOps for ProcPidStatus {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 400 + self.pid as u64,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get process info
        let table = crate::process::get_process_table();
        let table = table.as_ref().ok_or(Errno::ESRCH)?;
        let task = table.get(self.pid).ok_or(Errno::ESRCH)?;

        let state = match task.state {
            crate::process::ProcessState::Running => "running",
            crate::process::ProcessState::Ready => "ready",
            crate::process::ProcessState::Sleeping => "sleeping",
            crate::process::ProcessState::Stopped => "stopped (signal)",
            crate::process::ProcessState::Zombie => "zombie",
        };

        let content = format!(
            "Name:\t{}\n\
             State:\t{}\n\
             Pid:\t{}\n\
             PPid:\t{}\n\
             Uid:\t{}\t{}\t{}\t{}\n\
             Gid:\t{}\t{}\t{}\t{}\n",
            task.name,
            state,
            self.pid,
            task.ppid,
            task.cred.uid, task.cred.euid, task.cred.euid, task.cred.euid,
            task.cred.gid, task.cred.egid, task.cred.egid, task.cred.egid,
        );

        let bytes = content.as_bytes();
        if offset >= bytes.len() as u64 {
            return Ok(0);
        }

        let start = offset as usize;
        let to_copy = (bytes.len() - start).min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
        Ok(to_copy)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

/// /proc/[pid]/maps file
struct ProcPidMaps {
    pid: u32,
}

impl InodeOps for ProcPidMaps {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 500 + self.pid as u64,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get process VMAs
        let table = crate::process::get_process_table();
        let table = table.as_ref().ok_or(Errno::ESRCH)?;
        let task = table.get(self.pid).ok_or(Errno::ESRCH)?;

        // Format maps (Phase A2: simplified, no actual VMA iteration yet)
        // Format: address perms offset dev:inode pathname
        // Example: 00400000-00500000 r-xp 00000000 00:00 0  /bin/sh

        let mut content = String::new();

        // Text segment (executable)
        content.push_str(&format!(
            "00400000-00500000 r-xp 00000000 00:00 0          [text]\n"
        ));

        // Data segment (read-write)
        content.push_str(&format!(
            "00600000-00700000 rw-p 00000000 00:00 0          [data]\n"
        ));

        // Heap (if allocated)
        content.push_str(&format!(
            "00800000-00900000 rw-p 00000000 00:00 0          [heap]\n"
        ));

        // Stack
        content.push_str(&format!(
            "007ffffff00000-007fffff00000 rw-p 00000000 00:00 0          [stack]\n"
        ));

        // TODO Phase B: Iterate actual VMAs from task.mm

        let bytes = content.as_bytes();
        if offset >= bytes.len() as u64 {
            return Ok(0);
        }

        let start = offset as usize;
        let to_copy = (bytes.len() - start).min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
        Ok(to_copy)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

// =============================================================================
// Agent Supervision Module /proc entries
// =============================================================================

#[cfg(feature = "agentsys")]
/// /proc/agentsys directory
struct AgentSysDir;

#[cfg(feature = "agentsys")]
impl InodeOps for AgentSysDir {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 6,
            itype: InodeType::Directory,
            mode: crate::vfs::S_IFDIR | 0o555,
            uid: 0,
            gid: 0,
            nlink: 2,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn lookup(&self, name: &str) -> Result<Arc<Inode>> {
        match name {
            "status" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(AgentStatusFile)) as &'static dyn InodeOps,
            ))),
            "telemetry" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(AgentTelemetryFile)) as &'static dyn InodeOps,
            ))),
            "cloud_gateway" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(CloudGatewayFile)) as &'static dyn InodeOps,
            ))),
            "compliance" => Ok(Arc::new(Inode::new(
                InodeType::Regular,
                0o444,
                Box::leak(Box::new(ComplianceFile)) as &'static dyn InodeOps,
            ))),
            _ => Err(Errno::ENOENT),
        }
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::EROFS)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        let mut entries = Vec::new();

        entries.push(DirEntry {
            ino: 6,
            name: ".".to_string(),
            itype: InodeType::Directory,
        });
        entries.push(DirEntry {
            ino: 1,
            name: "..".to_string(),
            itype: InodeType::Directory,
        });
        entries.push(DirEntry {
            ino: 7,
            name: "status".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 8,
            name: "telemetry".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 9,
            name: "cloud_gateway".to_string(),
            itype: InodeType::Regular,
        });
        entries.push(DirEntry {
            ino: 10,
            name: "compliance".to_string(),
            itype: InodeType::Regular,
        });

        Ok(entries)
    }

    fn read(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize> {
        Err(Errno::EISDIR)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EISDIR)
    }
}

#[cfg(feature = "agentsys")]
/// /proc/agentsys/status file - human-readable telemetry
struct AgentStatusFile;

#[cfg(feature = "agentsys")]
impl InodeOps for AgentStatusFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 7,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get telemetry data from ASM
        let telemetry = crate::agent_sys::supervisor::TELEMETRY.lock();

        if let Some(ref telem) = *telemetry {
            // Export to buffer
            let written = telem.export_proc(buf);

            if offset >= written as u64 {
                return Ok(0);
            }

            let start = offset as usize;
            let remaining = written - start;

            // The data is already in buf, just return the length from offset
            Ok(remaining.min(buf.len()))
        } else {
            let msg = b"Agent Supervision Module not initialized\n";
            if offset >= msg.len() as u64 {
                return Ok(0);
            }

            let start = offset as usize;
            let to_copy = (msg.len() - start).min(buf.len());
            buf[..to_copy].copy_from_slice(&msg[start..start + to_copy]);
            Ok(to_copy)
        }
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

#[cfg(feature = "agentsys")]
/// /proc/agentsys/telemetry file - JSON telemetry snapshot
struct AgentTelemetryFile;

#[cfg(feature = "agentsys")]
impl InodeOps for AgentTelemetryFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 8,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get telemetry snapshot
        if let Some(snapshot) = crate::agent_sys::supervisor::hooks::get_telemetry_snapshot() {
            // Serialize to JSON
            if let Ok(json) = serde_json::to_string(&snapshot) {
                let bytes = json.as_bytes();

                if offset >= bytes.len() as u64 {
                    return Ok(0);
                }

                let start = offset as usize;
                let to_copy = (bytes.len() - start).min(buf.len());
                buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
                Ok(to_copy)
            } else {
                let msg = b"Failed to serialize telemetry\n";
                if offset == 0 {
                    let to_copy = msg.len().min(buf.len());
                    buf[..to_copy].copy_from_slice(&msg[..to_copy]);
                    Ok(to_copy)
                } else {
                    Ok(0)
                }
            }
        } else {
            let msg = b"Agent Supervision Module not initialized\n";
            if offset >= msg.len() as u64 {
                return Ok(0);
            }

            let start = offset as usize;
            let to_copy = (msg.len() - start).min(buf.len());
            buf[..to_copy].copy_from_slice(&msg[start..start + to_copy]);
            Ok(to_copy)
        }
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

#[cfg(feature = "agentsys")]
/// /proc/agentsys/cloud_gateway file - Cloud Gateway metrics
struct CloudGatewayFile;

#[cfg(feature = "agentsys")]
impl InodeOps for CloudGatewayFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 9,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        use alloc::format;
        use alloc::string::ToString;

        // Get gateway metrics
        let gateway_guard = crate::agent_sys::cloud_gateway::CLOUD_GATEWAY.lock();
        if let Some(ref gateway) = *gateway_guard {
            let metrics = gateway.metrics();
            let health = gateway.all_backend_health();

            // Format metrics as human-readable text
            let mut output = alloc::vec::Vec::new();

            output.extend_from_slice(b"Cloud Gateway Metrics\n");
            output.extend_from_slice(b"====================\n\n");

            // System metrics
            output.extend_from_slice(b"Request Statistics:\n");
            output.extend_from_slice(format!("  Total Requests:       {}\n", metrics.total_requests).as_bytes());
            output.extend_from_slice(format!("  Successful:           {}\n", metrics.successful_requests).as_bytes());
            output.extend_from_slice(format!("  Failed:               {}\n", metrics.failed_requests).as_bytes());
            output.extend_from_slice(format!("  Rate Limited:         {}\n", metrics.rate_limited_requests).as_bytes());
            output.extend_from_slice(format!("  Fallback Used:        {}\n", metrics.fallback_requests).as_bytes());
            output.extend_from_slice(b"\n");

            // Per-provider stats
            output.extend_from_slice(b"Provider Statistics:\n");
            output.extend_from_slice(format!("  Claude:   Success: {}  Failures: {}\n",
                metrics.claude_successes, metrics.claude_failures).as_bytes());
            output.extend_from_slice(format!("  GPT-4:    Success: {}  Failures: {}\n",
                metrics.gpt4_successes, metrics.gpt4_failures).as_bytes());
            output.extend_from_slice(format!("  Gemini:   Success: {}  Failures: {}\n",
                metrics.gemini_successes, metrics.gemini_failures).as_bytes());
            output.extend_from_slice(format!("  Local:    Success: {}  Failures: {}\n",
                metrics.local_successes, metrics.local_failures).as_bytes());
            output.extend_from_slice(b"\n");

            // Backend health
            output.extend_from_slice(b"Backend Health:\n");
            for (provider, health_val) in health.iter() {
                let status = if *health_val > 0.8 { "HEALTHY" } else if *health_val > 0.5 { "DEGRADED" } else { "DOWN" };
                output.extend_from_slice(format!("  {:12} {:.1}%  [{}]\n",
                    provider.as_str(), health_val * 100.0, status).as_bytes());
            }
            output.extend_from_slice(b"\n");

            // Performance
            output.extend_from_slice(b"Performance:\n");
            output.extend_from_slice(format!("  Total Tokens:         {}\n", metrics.total_tokens).as_bytes());
            output.extend_from_slice(format!("  Avg Response Time:    {} μs\n", metrics.avg_response_time_us).as_bytes());
            output.extend_from_slice(b"\n");

            // Active agents
            output.extend_from_slice(format!("Active Agents (rate limiters): {}\n", gateway.active_agents()).as_bytes());

            let bytes = &output;
            if offset >= bytes.len() as u64 {
                return Ok(0);
            }

            let start = offset as usize;
            let to_copy = (bytes.len() - start).min(buf.len());
            buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
            Ok(to_copy)
        } else {
            let msg = b"Cloud Gateway not initialized\n";
            if offset >= msg.len() as u64 {
                return Ok(0);
            }

            let start = offset as usize;
            let to_copy = (msg.len() - start).min(buf.len());
            buf[..to_copy].copy_from_slice(&msg[start..start + to_copy]);
            Ok(to_copy)
        }
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

#[cfg(feature = "agentsys")]
/// /proc/agentsys/compliance file - EU AI Act compliance report
struct ComplianceFile;

#[cfg(feature = "agentsys")]
impl InodeOps for ComplianceFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 10,
            itype: InodeType::Regular,
            mode: crate::vfs::S_IFREG | 0o444,
            uid: 0,
            gid: 0,
            nlink: 1,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        use alloc::format;
        use alloc::string::ToString;

        // Get compliance report
        if let Some(report) = crate::agent_sys::supervisor::hooks::get_compliance_report() {
            // Format compliance report as human-readable text
            let mut output = alloc::vec::Vec::new();

            output.extend_from_slice(b"EU AI Act Compliance Report\n");
            output.extend_from_slice(b"===========================\n\n");

            // System-wide compliance
            output.extend_from_slice(format!("Timestamp:            {}\n", report.timestamp).as_bytes());
            output.extend_from_slice(format!("Total Agents:         {}\n", report.total_agents).as_bytes());
            output.extend_from_slice(format!("Total Events:         {}\n", report.total_events).as_bytes());
            output.extend_from_slice(format!("Policy Violations:    {}\n", report.policy_violations).as_bytes());
            output.extend_from_slice(format!("System Compliance:    {:.1}%\n\n", report.system_compliance_score * 100.0).as_bytes());

            // Risk level distribution
            output.extend_from_slice(b"Risk Level Distribution:\n");
            output.extend_from_slice(format!("  Minimal:            {}\n", report.minimal_risk_agents).as_bytes());
            output.extend_from_slice(format!("  Limited:            {}\n", report.limited_risk_agents).as_bytes());
            output.extend_from_slice(format!("  High:               {}\n", report.high_risk_agents).as_bytes());
            output.extend_from_slice(format!("  Unacceptable:       {}\n", report.unacceptable_risk_agents).as_bytes());
            output.extend_from_slice(b"\n");

            // Per-agent compliance
            output.extend_from_slice(b"Agent Compliance Details:\n");
            output.extend_from_slice(b"-------------------------\n");

            for agent_record in &report.agent_records {
                output.extend_from_slice(format!("\nAgent ID: {}\n", agent_record.agent_id).as_bytes());
                output.extend_from_slice(format!("  Risk Level:         {}\n", agent_record.risk_level.as_str()).as_bytes());
                output.extend_from_slice(format!("  Events Logged:      {}\n", agent_record.events_logged).as_bytes());
                output.extend_from_slice(format!("  Violations:         {}\n", agent_record.policy_violations).as_bytes());
                output.extend_from_slice(format!("  Human Oversight:    {}\n", agent_record.human_oversight_count).as_bytes());
                output.extend_from_slice(format!("  Compliance Score:   {:.1}%\n", agent_record.compliance_score * 100.0).as_bytes());

                let status = if agent_record.compliance_score >= 0.9 {
                    "COMPLIANT"
                } else if agent_record.compliance_score >= 0.7 {
                    "REVIEW_NEEDED"
                } else {
                    "NON_COMPLIANT"
                };
                output.extend_from_slice(format!("  Status:             {}\n", status).as_bytes());
            }

            output.extend_from_slice(b"\n");
            output.extend_from_slice(b"Compliance Requirements (EU AI Act):\n");
            output.extend_from_slice(b"- Transparency: All operations logged\n");
            output.extend_from_slice(b"- Risk Assessment: Agents classified by risk level\n");
            output.extend_from_slice(b"- Human Oversight: Available via compliance events\n");
            output.extend_from_slice(b"- Audit Trail: Complete event history maintained\n");
            output.extend_from_slice(b"- Robustness: Fault detection and recovery active\n");

            let bytes = &output;
            if offset >= bytes.len() as u64 {
                return Ok(0);
            }

            let start = offset as usize;
            let to_copy = (bytes.len() - start).min(buf.len());
            buf[..to_copy].copy_from_slice(&bytes[start..start + to_copy]);
            Ok(to_copy)
        } else {
            let msg = b"Compliance tracking not initialized\n";
            if offset >= msg.len() as u64 {
                return Ok(0);
            }

            let start = offset as usize;
            let to_copy = (msg.len() - start).min(buf.len());
            buf[..to_copy].copy_from_slice(&msg[start..start + to_copy]);
            Ok(to_copy)
        }
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        Err(Errno::EACCES)
    }

    fn lookup(&self, _name: &str) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn create(&self, _name: &str, _mode: u32) -> Result<Arc<Inode>> {
        Err(Errno::ENOTDIR)
    }

    fn readdir(&self) -> Result<Vec<DirEntry>> {
        Err(Errno::ENOTDIR)
    }
}

// =============================================================================
// Mount function
// =============================================================================

/// Mount procfs at /proc
pub fn mount_procfs() -> Result<Arc<Inode>> {
    Ok(Arc::new(Inode::new(
        InodeType::Directory,
        0o555,
        Box::leak(Box::new(ProcfsRoot)) as &'static dyn InodeOps,
    )))
}
