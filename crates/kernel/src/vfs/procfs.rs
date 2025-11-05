// Procfs implementation for Phase A1
// Provides /proc filesystem for process and system information

use super::inode::{Inode, InodeType, InodeOps, Ino, DirEntry, alloc_ino};
use crate::lib::error::{Errno, Result};
use alloc::sync::Arc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;

/// Procfs root inode
pub struct ProcfsRoot;

impl InodeOps for ProcfsRoot {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 1,
            size: 0,
            mode: crate::vfs::S_IFDIR | 0o555,
            nlink: 2,
        })
    }

    fn lookup(&self, name: &str) -> Result<Arc<Inode>> {
        match name {
            "cpuinfo" => Ok(Arc::new(Inode::new(
                alloc_ino(),
                InodeType::File,
                Box::new(CpuInfoFile),
            ))),
            "meminfo" => Ok(Arc::new(Inode::new(
                alloc_ino(),
                InodeType::File,
                Box::new(MemInfoFile),
            ))),
            "uptime" => Ok(Arc::new(Inode::new(
                alloc_ino(),
                InodeType::File,
                Box::new(UptimeFile),
            ))),
            "mounts" => Ok(Arc::new(Inode::new(
                alloc_ino(),
                InodeType::File,
                Box::new(MountsFile),
            ))),
            "self" => {
                // /proc/self symlink to current process (Phase A2)
                let pid = crate::process::current_pid();
                Ok(Arc::new(Inode::new(
                    alloc_ino(),
                    InodeType::Directory,
                    Box::new(ProcPidDir { pid }),
                )))
            }
            _ => {
                // Check if it's a PID directory (numeric)
                if let Ok(pid) = name.parse::<u32>() {
                    // Verify PID exists
                    let table = crate::process::get_process_table();
                    if let Some(ref table) = *table {
                        if table.get(&pid).is_some() {
                            return Ok(Arc::new(Inode::new(
                                alloc_ino(),
                                InodeType::Directory,
                                Box::new(ProcPidDir { pid }),
                            )));
                        }
                    }
                }
                Err(Errno::ENOENT)
            }
        }
    }

    fn readdir(&self, offset: usize) -> Result<Option<DirEntry>> {
        let entries = vec![
            (".", 1),
            ("..", 1),
            ("cpuinfo", 2),
            ("meminfo", 3),
            ("uptime", 4),
            ("mounts", 5),
        ];

        if offset >= entries.len() {
            // TODO: Add PID directories dynamically
            return Ok(None);
        }

        let (name, ino) = entries[offset];
        Ok(Some(DirEntry {
            ino,
            name: name.to_string(),
            typ: if offset < 2 { InodeType::Directory } else { InodeType::File },
        }))
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
            size: 0,
            mode: crate::vfs::S_IFREG | 0o444,
            nlink: 1,
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
}

/// /proc/meminfo file
struct MemInfoFile;

impl InodeOps for MemInfoFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 3,
            size: 0,
            mode: crate::vfs::S_IFREG | 0o444,
            nlink: 1,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let stats = crate::mm::get_stats();
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
}

/// /proc/uptime file
struct UptimeFile;

impl InodeOps for UptimeFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 4,
            size: 0,
            mode: crate::vfs::S_IFREG | 0o444,
            nlink: 1,
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
}

/// /proc/mounts file
struct MountsFile;

impl InodeOps for MountsFile {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 5,
            size: 0,
            mode: crate::vfs::S_IFREG | 0o444,
            nlink: 1,
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
}

/// /proc/[pid] directory
struct ProcPidDir {
    pid: u32,
}

impl InodeOps for ProcPidDir {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 100 + self.pid as u64,
            size: 0,
            mode: crate::vfs::S_IFDIR | 0o555,
            nlink: 2,
        })
    }

    fn lookup(&self, name: &str) -> Result<Arc<Inode>> {
        match name {
            "cmdline" => Ok(Arc::new(Inode::new(
                alloc_ino(),
                InodeType::File,
                Box::new(ProcPidCmdline { pid: self.pid }),
            ))),
            "stat" => Ok(Arc::new(Inode::new(
                alloc_ino(),
                InodeType::File,
                Box::new(ProcPidStat { pid: self.pid }),
            ))),
            "status" => Ok(Arc::new(Inode::new(
                alloc_ino(),
                InodeType::File,
                Box::new(ProcPidStatus { pid: self.pid }),
            ))),
            "maps" => Ok(Arc::new(Inode::new(
                alloc_ino(),
                InodeType::File,
                Box::new(ProcPidMaps { pid: self.pid }),
            ))),
            _ => Err(Errno::ENOENT),
        }
    }

    fn readdir(&self, offset: usize) -> Result<Option<DirEntry>> {
        let entries = vec![
            (".", 100 + self.pid as u64),
            ("..", 1),
            ("cmdline", 200 + self.pid as u64),
            ("stat", 300 + self.pid as u64),
            ("status", 400 + self.pid as u64),
            ("maps", 500 + self.pid as u64),
        ];

        if offset >= entries.len() {
            return Ok(None);
        }

        let (name, ino) = entries[offset];
        Ok(Some(DirEntry {
            ino,
            name: name.to_string(),
            typ: if offset < 2 { InodeType::Directory } else { InodeType::File },
        }))
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
            size: 0,
            mode: crate::vfs::S_IFREG | 0o444,
            nlink: 1,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get process name
        let table = crate::process::get_process_table();
        let table = table.as_ref().ok_or(Errno::ESRCH)?;
        let task = table.get(&self.pid).ok_or(Errno::ESRCH)?;

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
}

/// /proc/[pid]/stat file
struct ProcPidStat {
    pid: u32,
}

impl InodeOps for ProcPidStat {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 300 + self.pid as u64,
            size: 0,
            mode: crate::vfs::S_IFREG | 0o444,
            nlink: 1,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get process info
        let table = crate::process::get_process_table();
        let table = table.as_ref().ok_or(Errno::ESRCH)?;
        let task = table.get(&self.pid).ok_or(Errno::ESRCH)?;

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
}

/// /proc/[pid]/status file
struct ProcPidStatus {
    pid: u32,
}

impl InodeOps for ProcPidStatus {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 400 + self.pid as u64,
            size: 0,
            mode: crate::vfs::S_IFREG | 0o444,
            nlink: 1,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get process info
        let table = crate::process::get_process_table();
        let table = table.as_ref().ok_or(Errno::ESRCH)?;
        let task = table.get(&self.pid).ok_or(Errno::ESRCH)?;

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
            task.cred.uid, task.cred.euid, task.cred.suid, task.cred.fsuid,
            task.cred.gid, task.cred.egid, task.cred.sgid, task.cred.fsgid,
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
}

/// /proc/[pid]/maps file
struct ProcPidMaps {
    pid: u32,
}

impl InodeOps for ProcPidMaps {
    fn getattr(&self) -> Result<super::inode::InodeMeta> {
        Ok(super::inode::InodeMeta {
            ino: 500 + self.pid as u64,
            size: 0,
            mode: crate::vfs::S_IFREG | 0o444,
            nlink: 1,
        })
    }

    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // Get process VMAs
        let table = crate::process::get_process_table();
        let table = table.as_ref().ok_or(Errno::ESRCH)?;
        let task = table.get(&self.pid).ok_or(Errno::ESRCH)?;

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
}

/// Mount procfs at /proc
pub fn mount_procfs() -> Result<Arc<Inode>> {
    Ok(Arc::new(Inode::new(
        alloc_ino(),
        InodeType::Directory,
        Box::new(ProcfsRoot),
    )))
}
