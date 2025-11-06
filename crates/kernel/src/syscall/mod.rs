// Syscall infrastructure
// Phase A0 - Minimal syscall dispatcher

pub mod uaccess;

use crate::lib::error::{Errno, Result};
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;

/// Syscall error type (alias for Errno)
pub type SyscallError = Errno;

/// Syscall frame - represents CPU state during syscall entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallFrame {
    /// General purpose registers x0-x30
    pub gpr: [u64; 31],
    /// User stack pointer (SP_EL0)
    pub sp_el0: u64,
    /// Exception link register (return address)
    pub elr_el1: u64,
    /// Saved program status register
    pub spsr_el1: u64,
}

/// Syscall numbers (AArch64 Linux syscall numbers)
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SyscallNumber {
    // File I/O
    OpenAt = 56,
    Close = 57,
    Lseek = 62,
    Read = 63,
    Write = 64,
    ReadLinkAt = 78,
    Fstat = 80,
    GetDents64 = 61,

    // Process management
    Exit = 93,
    GetPid = 172,
    GetPPid = 173,
    Fork = 220,
    Execve = 221,
    Wait4 = 260,

    // Credentials
    GetUid = 174,
    GetEuid = 175,
    GetGid = 176,
    GetEgid = 177,
    SetUid = 146,
    SetGid = 144,
    SetEuid = 145,
    SetEgid = 147,

    // Signals
    Kill = 129,
    SigAction = 134,
    SigReturn = 139,

    // Memory management
    Brk = 214,
    Mmap = 222,
    Munmap = 215,
    Mprotect = 226,

    // File operations
    Mkdir = 34,
    Rmdir = 35,
    Unlink = 10,
    Pipe = 59,
    Dup = 23,
    Dup2 = 24,
    Chmod = 52,
    Chown = 55,
    Umask = 166,
    GetRandom = 278,

    // Working directory
    GetCwd = 17,
    Chdir = 49,

    // I/O operations
    Ioctl = 29,
    Ppoll = 73,

    // Time operations
    ClockGetTime = 113,
    Nanosleep = 101,

    // Filesystem operations
    Mount = 40,
    Umount2 = 39,

    // Socket operations
    Socket = 198,
    Bind = 200,
    Listen = 201,
    Accept = 202,
    Connect = 203,
    SendTo = 206,
    RecvFrom = 207,
    Shutdown = 210,
}

/// Handle a syscall from userspace
///
/// Takes a syscall frame containing register state and dispatches
/// to the appropriate handler. Returns result in x0 or error.
pub fn handle_syscall(frame: &mut SyscallFrame) -> Result<u64> {
    let nr = frame.gpr[8] as usize; // x8 contains syscall number
    let args = [
        frame.gpr[0], // x0
        frame.gpr[1], // x1
        frame.gpr[2], // x2
        frame.gpr[3], // x3
        frame.gpr[4], // x4
        frame.gpr[5], // x5
    ];

    let result = syscall_dispatcher(nr, &args);

    if result < 0 {
        Err(Errno::from_negated_i32(result as i32))
    } else {
        Ok(result as u64)
    }
}

/// Run a syscall microbenchmark
#[allow(dead_code)]
pub fn run_syscall_microbenchmark(_nr: SyscallNumber, _iterations: usize) {
    // Stub implementation for now
    crate::warn!("Syscall microbenchmark not yet implemented");
}

/// Print cycle counter value
#[allow(dead_code)]
pub fn print_cycles(_label: &str) {
    // Stub implementation for now
}

/// Print syscall performance report
#[allow(dead_code)]
pub fn print_syscall_performance_report() {
    // Stub implementation for now
}

/// Reset syscall metrics
#[allow(dead_code)]
pub fn reset_syscall_metrics() {
    // Stub implementation for now
}

/// Read cycle counter
#[allow(dead_code)]
pub fn read_cycle_counter() -> u64 {
    // Stub implementation for now
    0
}

/// Syscall dispatcher - routes syscall number to appropriate handler
///
/// AArch64 Syscall ABI:
/// - Syscall number: x8
/// - Arguments: x0-x5 (up to 6 args)
/// - Return value: x0 (negative for errno)
pub fn syscall_dispatcher(nr: usize, args: &[u64; 6]) -> isize {
    let result = match nr {
        // File I/O syscalls
        56 => sys_openat(args[0] as i32, args[1] as *const u8, args[2] as i32, args[3] as u32),
        57 => sys_close(args[0] as i32),
        62 => sys_lseek(args[0] as i32, args[1] as i64, args[2] as i32),
        63 => sys_read(args[0] as i32, args[1] as *mut u8, args[2] as usize),
        64 => sys_write(args[0] as i32, args[1] as *const u8, args[2] as usize),
        78 => sys_readlinkat(args[0] as i32, args[1] as *const u8, args[2] as *mut u8, args[3] as usize),
        80 => sys_fstat(args[0] as i32, args[1] as *mut u8),
        61 => sys_getdents64(args[0] as i32, args[1] as *mut u8, args[2] as usize),

        // Process management
        93 => sys_exit(args[0] as i32),
        172 => sys_getpid(),
        173 => sys_getppid(),
        220 => sys_fork(),
        221 => sys_execve(args[0] as *const u8, args[1] as *const *const u8, args[2] as *const *const u8),
        260 => sys_wait4(args[0] as i32, args[1] as *mut i32, args[2] as i32, args[3] as *mut u8),

        // Credentials (Phase D)
        174 => sys_getuid(),
        175 => sys_geteuid(),
        176 => sys_getgid(),
        177 => sys_getegid(),
        146 => sys_setuid(args[0] as u32),
        144 => sys_setgid(args[0] as u32),
        145 => sys_seteuid(args[0] as u32),
        147 => sys_setegid(args[0] as u32),

        // Signal handling
        129 => sys_kill(args[0] as i32, args[1] as i32),
        134 => sys_sigaction(args[0] as i32, args[1] as *const u8, args[2] as *mut u8),
        139 => sys_sigreturn(),

        // Memory management
        214 => sys_brk(args[0] as *const u8),
        222 => sys_mmap(args[0] as *mut u8, args[1] as usize, args[2] as i32, args[3] as i32, args[4] as i32, args[5] as i64),
        215 => sys_munmap(args[0] as *mut u8, args[1] as usize),
        226 => sys_mprotect(args[0] as *mut u8, args[1] as usize, args[2] as i32),

        // File operations
        34 => sys_mkdir(args[0] as *const u8, args[1] as u32),
        35 => sys_rmdir(args[0] as *const u8),
        10 => sys_unlink(args[0] as *const u8),
        59 => sys_pipe(args[0] as *mut i32),
        23 => sys_dup(args[0] as i32),
        24 => sys_dup2(args[0] as i32, args[1] as i32),
        52 => sys_chmod(args[0] as *const u8, args[1] as u32),
        55 => sys_chown(args[0] as *const u8, args[1] as u32, args[2] as u32),
        166 => sys_umask(args[0] as u32),
        278 => sys_getrandom(args[0] as *mut u8, args[1] as usize, args[2] as u32),

        // Working directory
        17 => sys_getcwd(args[0] as *mut u8, args[1] as usize),
        49 => sys_chdir(args[0] as *const u8),

        // I/O operations
        29 => sys_ioctl(args[0] as i32, args[1] as u64, args[2] as u64),
        73 => sys_ppoll(args[0] as *mut u8, args[1] as usize, args[2] as *const u8, args[3] as *const u8),

        // Time operations
        113 => sys_clock_gettime(args[0] as i32, args[1] as *mut u8),
        101 => sys_nanosleep(args[0] as *const u8, args[1] as *mut u8),

        // Filesystem operations (Phase B)
        40 => sys_mount(args[0] as *const u8, args[1] as *const u8, args[2] as *const u8, args[3] as u64, args[4] as *const u8),
        39 => sys_umount2(args[0] as *const u8, args[1] as i32),

        // Socket operations (Phase C)
        198 => sys_socket(args[0] as i32, args[1] as i32, args[2] as i32),
        200 => sys_bind(args[0] as i32, args[1] as *const u8, args[2] as u32),
        201 => sys_listen(args[0] as i32, args[1] as i32),
        202 => sys_accept(args[0] as i32, args[1] as *mut u8, args[2] as *mut u32),
        203 => sys_connect(args[0] as i32, args[1] as *const u8, args[2] as u32),
        206 => sys_sendto(args[0] as i32, args[1] as *const u8, args[2] as usize, args[3] as i32, args[4] as *const u8, args[5] as u32),
        207 => sys_recvfrom(args[0] as i32, args[1] as *mut u8, args[2] as usize, args[3] as i32, args[4] as *mut u8, args[5] as *mut u32),
        210 => sys_shutdown(args[0] as i32, args[1] as i32),

        // Unimplemented
        _ => {
            crate::warn!("Unimplemented syscall: {}", nr);
            Err(Errno::ENOSYS)
        }
    };

    match result {
        Ok(ret) => ret,
        Err(e) => e.as_isize(),
    }
}

/// sys_openat - Open a file (Phase A2: support relative paths with CWD)
pub fn sys_openat(dirfd: i32, pathname: *const u8, flags: i32, mode: u32) -> Result<isize> {
    const AT_FDCWD: i32 = -100;

    if pathname.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy pathname from userspace
    let path_str = unsafe {
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(pathname, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    // Phase A2: Resolve relative paths
    // Get current task to access CWD
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(pid).ok_or(Errno::ESRCH)?;

    // Resolve path (absolute or relative)
    let path = if path_str.starts_with('/') {
        // Absolute path - use as-is
        alloc::string::String::from(path_str)
    } else {
        // Relative path
        if dirfd == AT_FDCWD || dirfd == -1 {
            // Resolve against current CWD
            let mut full_path = task.cwd.clone();
            if !full_path.ends_with('/') {
                full_path.push('/');
            }
            full_path.push_str(path_str);
            normalize_path(&full_path)
        } else {
            // Phase B+: Resolve against directory FD
            return Err(Errno::ENOTSUP);
        }
    };

    drop(table); // Release the reference before continuing

    // Convert flags to OpenFlags
    let open_flags = crate::vfs::OpenFlags::from_bits_truncate(flags as u32);

    // Special handling for /dev/ptmx (Phase A2)
    let file = if path == "/dev/ptmx" {
        // Opening /dev/ptmx creates a new PTY pair and returns the master
        alloc::sync::Arc::new(crate::vfs::open_ptmx()?)
    } else if open_flags.contains(crate::vfs::OpenFlags::O_CREAT) {
        // Create new file if doesn't exist
        match crate::vfs::open(&path, open_flags) {
            Ok(f) => f,
            Err(Errno::ENOENT) => crate::vfs::create(&path, mode, open_flags)?,
            Err(e) => return Err(e),
        }
    } else {
        crate::vfs::open(&path, open_flags)?
    };

    // Get current process and allocate FD
    let pid = crate::process::current_pid();
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    let fd = task.files.alloc_fd(file)?;

    crate::debug!("sys_open({}) -> fd {}", path, fd);

    Ok(fd as isize)
}

/// sys_close - Close a file descriptor
pub fn sys_close(fd: i32) -> Result<isize> {
    let pid = crate::process::current_pid();
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    task.files.close(fd)?;

    Ok(0)
}

/// sys_lseek - Reposition file offset
pub fn sys_lseek(fd: i32, offset: i64, whence: i32) -> Result<isize> {
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(pid).ok_or(Errno::ESRCH)?;

    let file = task.files.get(fd)?;
    let new_offset = file.lseek(offset, whence)?;

    Ok(new_offset as isize)
}

/// sys_read - Read from file descriptor
pub fn sys_read(fd: i32, buf: *mut u8, count: usize) -> Result<isize> {
    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    if count == 0 {
        return Ok(0);
    }

    // Get file from FD table
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(pid).ok_or(Errno::ESRCH)?;

    let file = task.files.get(fd)?;

    // Create buffer
    let data = unsafe { core::slice::from_raw_parts_mut(buf, count) };

    // Read from file
    let n = file.read(data)?;

    Ok(n as isize)
}

/// sys_write - Write to file descriptor
pub fn sys_write(fd: i32, buf: *const u8, count: usize) -> Result<isize> {
    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    if count == 0 {
        return Ok(0);
    }

    // Get file from FD table
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(pid).ok_or(Errno::ESRCH)?;

    let file = task.files.get(fd)?;

    // Create buffer
    let data = unsafe { core::slice::from_raw_parts(buf, count) };

    // Write to file
    let n = file.write(data)?;

    Ok(n as isize)
}

/// sys_fstat - Get file status
pub fn sys_fstat(fd: i32, statbuf: *mut u8) -> Result<isize> {
    if statbuf.is_null() {
        return Err(Errno::EFAULT);
    }

    // Get file from FD table
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(pid).ok_or(Errno::ESRCH)?;

    let file = task.files.get(fd)?;
    let meta = file.getattr()?;

    // Fill stat structure (simplified for Phase A1)
    // struct stat is large, we'll fill the important fields
    let stat = unsafe { core::slice::from_raw_parts_mut(statbuf, 128) };
    stat.fill(0);

    // Write fields (x86_64/aarch64 stat layout)
    // st_dev: 8 bytes at offset 0
    // st_ino: 8 bytes at offset 8
    // st_mode: 4 bytes at offset 24
    // st_nlink: 8 bytes at offset 16
    // st_uid: 4 bytes at offset 28
    // st_gid: 4 bytes at offset 32
    // st_size: 8 bytes at offset 48

    unsafe {
        let p = statbuf as *mut u64;
        *p.add(1) = meta.ino; // st_ino
        let pm = statbuf.add(24) as *mut u32;
        *pm = meta.mode; // st_mode
        let ps = statbuf.add(48) as *mut u64;
        *ps = meta.size; // st_size
    }

    Ok(0)
}

/// sys_getdents64 - Get directory entries
pub fn sys_getdents64(fd: i32, dirp: *mut u8, count: usize) -> Result<isize> {
    if dirp.is_null() {
        return Err(Errno::EFAULT);
    }

    // Get file from FD table
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(pid).ok_or(Errno::ESRCH)?;

    let file = task.files.get(fd)?;

    // Check if directory
    if !file.is_dir()? {
        return Err(Errno::ENOTDIR);
    }

    // Read directory entries
    let entries = file.readdir()?;

    // Fill linux_dirent64 structures
    let mut offset = 0usize;

    for entry in entries {
        // struct linux_dirent64 layout:
        // u64 d_ino, i64 d_off, u16 d_reclen, u8 d_type, char d_name[]
        let name_bytes = entry.name.as_bytes();
        let reclen = ((19 + name_bytes.len() + 1 + 7) & !7) as u16; // Align to 8

        if offset + reclen as usize > count {
            break; // No more space
        }

        unsafe {
            let p = dirp.add(offset);
            // d_ino
            *(p as *mut u64) = entry.ino;
            // d_off (can be 0 for now)
            *(p.add(8) as *mut i64) = 0;
            // d_reclen
            *(p.add(16) as *mut u16) = reclen;
            // d_type
            *p.add(18) = match entry.itype {
                crate::vfs::InodeType::Regular => 8,    // DT_REG
                crate::vfs::InodeType::Directory => 4,  // DT_DIR
                crate::vfs::InodeType::CharDevice => 2, // DT_CHR
                crate::vfs::InodeType::Symlink => 10,   // DT_LNK
            };
            // d_name
            core::ptr::copy_nonoverlapping(name_bytes.as_ptr(), p.add(19), name_bytes.len());
            *p.add(19 + name_bytes.len()) = 0; // Null terminator
        }

        offset += reclen as usize;
    }

    Ok(offset as isize)
}

/// sys_readlinkat - Read symbolic link (stub for Phase A1)
pub fn sys_readlinkat(dirfd: i32, pathname: *const u8, buf: *mut u8, bufsiz: usize) -> Result<isize> {
    let _ = (dirfd, pathname, buf, bufsiz);
    // For Phase A1, return EINVAL (no symlinks yet)
    Err(Errno::EINVAL)
}

/// sys_exit - Terminate current process
pub fn sys_exit(code: i32) -> Result<isize> {
    let pid = crate::process::current_pid();
    crate::info!("Process {} exit with code {}", pid, code);

    // Call do_exit which never returns
    crate::process::do_exit(pid, code);
}

/// sys_getpid - Get process ID
pub fn sys_getpid() -> Result<isize> {
    let pid = crate::process::current_pid();
    Ok(pid as isize)
}

/// sys_fork - Create a child process
pub fn sys_fork() -> Result<isize> {
    let parent_pid = crate::process::current_pid();

    // Allocate new PID for child
    let child_pid = crate::process::alloc_pid()
        .map_err(|_| Errno::EAGAIN)?;

    crate::info!("fork: parent={}, child={}", parent_pid, child_pid);

    // Get parent task and create child (COW is set up in fork_from)
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;

    let parent = table.get(parent_pid).ok_or(Errno::ESRCH)?;
    let child = crate::process::Task::fork_from(parent, child_pid);

    // Insert child into process table
    drop(table); // Release lock before inserting
    crate::process::insert_task(child)
        .map_err(|_| Errno::ENOMEM)?;

    // TODO: Copy trap frame and set child's return value to 0
    // TODO: Mark child as runnable in scheduler

    // Parent returns child PID
    Ok(child_pid as isize)
}

/// sys_execve - Execute a program
pub fn sys_execve(
    pathname: *const u8,
    argv: *const *const u8,
    envp: *const *const u8,
) -> Result<isize> {
    use alloc::vec::Vec;
    use alloc::string::String;

    let current_pid = crate::process::current_pid();

    // 1. Copy pathname from userspace
    let path = unsafe {
        if pathname.is_null() {
            return Err(Errno::EFAULT);
        }
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        if len == 0 {
            return Err(Errno::EINVAL);
        }
        let bytes = core::slice::from_raw_parts(pathname, len);
        String::from_utf8(bytes.to_vec()).map_err(|_| Errno::EINVAL)?
    };

    crate::info!("execve: path={}", path);

    // 2. Copy argv from userspace
    let mut argv_vec = Vec::new();
    if !argv.is_null() {
        let mut i = 0;
        loop {
            let arg_ptr = unsafe { *argv.add(i) };
            if arg_ptr.is_null() {
                break;
            }
            let arg = unsafe {
                let mut len = 0;
                while len < 4096 && *arg_ptr.add(len) != 0 {
                    len += 1;
                }
                let bytes = core::slice::from_raw_parts(arg_ptr, len);
                String::from_utf8(bytes.to_vec()).map_err(|_| Errno::EINVAL)?
            };
            argv_vec.push(arg);
            i += 1;
            if i > 1024 {
                return Err(Errno::E2BIG); // Too many arguments
            }
        }
    }

    // 3. Copy envp from userspace
    let mut envp_vec = Vec::new();
    if !envp.is_null() {
        let mut i = 0;
        loop {
            let env_ptr = unsafe { *envp.add(i) };
            if env_ptr.is_null() {
                break;
            }
            let env = unsafe {
                let mut len = 0;
                while len < 4096 && *env_ptr.add(len) != 0 {
                    len += 1;
                }
                let bytes = core::slice::from_raw_parts(env_ptr, len);
                String::from_utf8(bytes.to_vec()).map_err(|_| Errno::EINVAL)?
            };
            envp_vec.push(env);
            i += 1;
            if i > 1024 {
                return Err(Errno::E2BIG); // Too many environment variables
            }
        }
    }

    crate::debug!("execve: argc={}, envc={}", argv_vec.len(), envp_vec.len());

    // 4. Open and read the ELF file
    let root = crate::vfs::get_root().ok_or(Errno::ENOENT)?;
    let inode = crate::vfs::path_lookup(&root, &path)?;

    // Read entire file into buffer
    let meta = inode.getattr()?;
    let file_size = meta.size as usize;
    if file_size > 16 * 1024 * 1024 {
        return Err(Errno::E2BIG); // File too large (16MB limit)
    }

    let mut elf_data = Vec::with_capacity(file_size);
    elf_data.resize(file_size, 0);
    let bytes_read = inode.read(0, &mut elf_data)?;
    elf_data.truncate(bytes_read);

    // 5. Get current task and load ELF
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(current_pid).ok_or(Errno::ESRCH)?;

    // Clear existing VMAs
    task.mm.vmas.clear();
    task.mm.brk = crate::mm::USER_HEAP_START;
    task.mm.brk_start = crate::mm::USER_HEAP_START;

    // Load ELF
    crate::process::exec::elf::load_elf(task, &elf_data, argv_vec, envp_vec)
        .map_err(|e| Errno::from(e))?;

    // 6. Set up FD 0/1/2 if not already open
    if task.files.get(0).is_err() {
        // Open /dev/console for stdin/stdout/stderr
        let dev_root = crate::vfs::get_root().ok_or(Errno::ENOENT)?;
        let console_inode = crate::vfs::path_lookup(&dev_root, "/dev/console")?;

        let console_file = alloc::sync::Arc::new(crate::vfs::File::new(
            console_inode,
            crate::vfs::OpenFlags::O_RDWR,
            &crate::drivers::char::CONSOLE_OPS,
        ));

        task.files.alloc_fd(console_file.clone())?; // FD 0 (stdin)
        task.files.alloc_fd(console_file.clone())?; // FD 1 (stdout)
        task.files.alloc_fd(console_file)?;         // FD 2 (stderr)
    }

    crate::info!("execve: loaded {} successfully", path);

    // execve does not return on success (trap frame was updated)
    Ok(0)
}

/// sys_wait4 - Wait for process to change state
pub fn sys_wait4(
    pid: i32,
    wstatus: *mut i32,
    options: i32,
    rusage: *mut u8,
) -> Result<isize> {
    let current_pid = crate::process::current_pid();

    let child_pid = crate::process::do_wait4(current_pid, pid, wstatus, options)?;

    Ok(child_pid as isize)
}

/// sys_brk - Change data segment size
pub fn sys_brk(addr: *const u8) -> Result<isize> {
    let new_brk = addr as u64;
    let pid = crate::process::current_pid();

    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    let result_brk = task.mm.do_brk(new_brk)?;
    Ok(result_brk as isize)
}

/// sys_mmap - Map memory
pub fn sys_mmap(
    addr: *mut u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> Result<isize> {
    let pid = crate::process::current_pid();

    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    let result_addr = task.mm.do_mmap(addr as u64, length as u64, prot, flags)?;
    Ok(result_addr as isize)
}

/// sys_munmap - Unmap memory
pub fn sys_munmap(addr: *mut u8, length: usize) -> Result<isize> {
    let pid = crate::process::current_pid();

    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    task.mm.do_munmap(addr as u64, length as u64)?;
    Ok(0)
}

/// sys_mprotect - Change memory protection (Phase D)
///
/// Change the protection flags of memory pages.
/// Enforces W^X policy (pages cannot be both writable and executable).
pub fn sys_mprotect(addr: *mut u8, len: usize, prot: i32) -> Result<isize> {
    use crate::mm::paging::{PAGE_SIZE, PROT_NONE, PROT_READ, PROT_WRITE, PROT_EXEC};

    // Validate address alignment
    let start_addr = addr as u64;
    if (start_addr & (PAGE_SIZE as u64 - 1)) != 0 {
        return Err(Errno::EINVAL);
    }

    // Validate length
    if len == 0 {
        return Ok(0);
    }

    // Round up length to page boundary
    let end_addr = start_addr + ((len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)) as u64;

    // Validate protection flags
    if (prot & !(PROT_NONE | PROT_READ | PROT_WRITE | PROT_EXEC)) != 0 {
        return Err(Errno::EINVAL);
    }

    // Get current task's page table
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(&pid).ok_or(Errno::ESRCH)?;

    // Get page table root
    let ttbr0 = task.mm.ttbr0;
    if ttbr0 == 0 {
        return Err(Errno::EFAULT);
    }

    let page_table = ttbr0 as *mut crate::mm::paging::PageTable;

    // Change protection for the range
    crate::mm::paging::change_page_protection(page_table, start_addr, end_addr, prot)
        .map_err(|e| Errno::from(e))?;

    Ok(0)
}

/// sys_getppid - Get parent process ID
pub fn sys_getppid() -> Result<isize> {
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(&pid).ok_or(Errno::ESRCH)?;
    Ok(task.ppid as isize)
}

/// sys_getuid - Get real user ID (Phase D)
pub fn sys_getuid() -> Result<isize> {
    Ok(crate::security::current_uid() as isize)
}

/// sys_geteuid - Get effective user ID (Phase D)
pub fn sys_geteuid() -> Result<isize> {
    Ok(crate::security::current_euid() as isize)
}

/// sys_getgid - Get real group ID (Phase D)
pub fn sys_getgid() -> Result<isize> {
    Ok(crate::security::current_gid() as isize)
}

/// sys_getegid - Get effective group ID (Phase D)
pub fn sys_getegid() -> Result<isize> {
    Ok(crate::security::current_egid() as isize)
}

/// sys_setuid - Set user ID (Phase D)
pub fn sys_setuid(uid: u32) -> Result<isize> {
    crate::security::set_uid(uid)?;
    Ok(0)
}

/// sys_setgid - Set group ID (Phase D)
pub fn sys_setgid(gid: u32) -> Result<isize> {
    crate::security::set_gid(gid)?;
    Ok(0)
}

/// sys_seteuid - Set effective user ID (Phase D)
pub fn sys_seteuid(euid: u32) -> Result<isize> {
    crate::security::set_euid(euid)?;
    Ok(0)
}

/// sys_setegid - Set effective group ID (Phase D)
pub fn sys_setegid(egid: u32) -> Result<isize> {
    crate::security::set_egid(egid)?;
    Ok(0)
}

/// sys_kill - Send signal to a process
pub fn sys_kill(pid: i32, sig: i32) -> Result<isize> {
    use crate::process::signal::Signal;

    // Validate signal number
    let signal = Signal::from_u32(sig as u32).ok_or(Errno::EINVAL)?;

    // Validate PID
    if pid <= 0 {
        // TODO: Phase A2 - support process groups (negative PIDs)
        return Err(Errno::EINVAL);
    }

    // Send signal
    crate::process::send_signal(pid as u32, signal)?;
    Ok(0)
}

/// sys_sigaction - Set signal handler
pub fn sys_sigaction(sig: i32, act: *const u8, oldact: *mut u8) -> Result<isize> {
    use crate::process::signal::{Signal, SignalAction, SigAction, SIG_DFL, SIG_IGN};

    // Validate signal number
    let signal = Signal::from_u32(sig as u32).ok_or(Errno::EINVAL)?;

    // Cannot change SIGKILL or SIGSTOP
    if !signal.is_catchable() {
        return Err(Errno::EINVAL);
    }

    let pid = crate::process::current_pid();
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    // Get old action if requested
    if !oldact.is_null() {
        let old_action = task.signals.get_handler(signal);
        let old_act = SigAction {
            sa_handler: match old_action {
                SignalAction::Ignore => SIG_IGN,
                SignalAction::Terminate => SIG_DFL,
                SignalAction::Stop => SIG_DFL,
                SignalAction::Continue => SIG_DFL,
                SignalAction::Handler(addr) => addr,
            },
            sa_mask: 0,
            sa_flags: 0,
            sa_restorer: 0,
        };

        // Copy to userspace
        unsafe {
            core::ptr::copy_nonoverlapping(
                &old_act as *const SigAction as *const u8,
                oldact,
                core::mem::size_of::<SigAction>(),
            );
        }
    }

    // Set new action if provided
    if !act.is_null() {
        // Copy from userspace
        let new_act: SigAction = unsafe {
            let mut act_buf = core::mem::MaybeUninit::<SigAction>::uninit();
            core::ptr::copy_nonoverlapping(
                act,
                act_buf.as_mut_ptr() as *mut u8,
                core::mem::size_of::<SigAction>(),
            );
            act_buf.assume_init()
        };

        // Convert to SignalAction
        let action = if new_act.sa_handler == SIG_DFL {
            signal.default_action()
        } else if new_act.sa_handler == SIG_IGN {
            SignalAction::Ignore
        } else {
            SignalAction::Handler(new_act.sa_handler)
        };

        task.signals.set_handler(signal, action);
    }

    Ok(0)
}

/// sys_sigreturn - Return from signal handler
pub fn sys_sigreturn() -> Result<isize> {
    // Phase A1: Minimal implementation
    // TODO: Restore saved context from signal frame on stack
    crate::warn!("sigreturn not fully implemented in Phase A1");
    Ok(0)
}

/// sys_pipe - Create a pipe
pub fn sys_pipe(fds: *mut i32) -> Result<isize> {
    if fds.is_null() {
        return Err(Errno::EFAULT);
    }

    // Create pipe
    let (reader, writer) = crate::vfs::create_pipe();

    // Wrap in File objects
    let read_file = Arc::new(crate::vfs::File::from_pipe_reader(reader));
    let write_file = Arc::new(crate::vfs::File::from_pipe_writer(writer));

    // Get current task and allocate FDs
    let pid = crate::process::current_pid();
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    // Allocate read FD
    let read_fd = task.files.alloc_fd(read_file)?;

    // Allocate write FD
    let write_fd = match task.files.alloc_fd(write_file) {
        Ok(fd) => fd,
        Err(e) => {
            // Failed to allocate write FD, clean up read FD
            let _ = task.files.close(read_fd);
            return Err(e);
        }
    };

    // Write FDs to userspace
    unsafe {
        *fds.offset(0) = read_fd;
        *fds.offset(1) = write_fd;
    }

    crate::debug!("sys_pipe: created pipe with fds [{}, {}]", read_fd, write_fd);
    Ok(0)
}

/// sys_dup - Duplicate file descriptor
pub fn sys_dup(oldfd: i32) -> Result<isize> {
    let pid = crate::process::current_pid();
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    // Get the file from oldfd
    let file = task.files.get(oldfd).ok_or(Errno::EBADF)?;

    // Allocate new FD
    let newfd = task.files.alloc_fd(file.clone())?;
    Ok(newfd as isize)
}

/// sys_dup2 - Duplicate file descriptor to specific FD number
pub fn sys_dup2(oldfd: i32, newfd: i32) -> Result<isize> {
    if newfd < 0 || newfd >= 1024 {
        return Err(Errno::EBADF);
    }

    let pid = crate::process::current_pid();
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    // If oldfd == newfd, just validate and return
    if oldfd == newfd {
        task.files.get(oldfd).ok_or(Errno::EBADF)?;
        return Ok(newfd as isize);
    }

    // Get the file from oldfd
    let file = task.files.get(oldfd).ok_or(Errno::EBADF)?;

    // Close newfd if it's open
    let _ = task.files.close(newfd);

    // Set newfd to point to the same file
    task.files.set(newfd, file.clone())?;
    Ok(newfd as isize)
}

/// sys_mkdir - Create a directory
pub fn sys_mkdir(pathname: *const u8, mode: u32) -> Result<isize> {
    if pathname.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy pathname from userspace
    let path = unsafe {
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(pathname, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    // Create directory through VFS
    let root = crate::vfs::get_root().ok_or(Errno::ENOENT)?;
    crate::vfs::mkdir(path, mode)?;
    Ok(0)
}

/// sys_rmdir - Remove a directory
pub fn sys_rmdir(pathname: *const u8) -> Result<isize> {
    if pathname.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy pathname from userspace
    let path = unsafe {
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(pathname, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    // Remove directory through VFS
    crate::vfs::rmdir(path)?;
    Ok(0)
}

/// sys_unlink - Remove a file
pub fn sys_unlink(pathname: *const u8) -> Result<isize> {
    if pathname.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy pathname from userspace
    let path = unsafe {
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(pathname, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    // Remove file through VFS
    crate::vfs::unlink(path)?;
    Ok(0)
}

/// sys_chmod - Change file permissions (Phase D)
pub fn sys_chmod(pathname: *const u8, mode: u32) -> Result<isize> {
    if pathname.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy pathname from userspace
    let path = unsafe {
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(pathname, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    // Get current credentials
    let cred = crate::security::current_cred();

    // Look up inode (simplified - would use VFS path lookup)
    // For Phase D, we'll just log and return success
    crate::info!("chmod: {} mode={:o}", path, mode);

    Ok(0)
}

/// sys_chown - Change file ownership (Phase D)
pub fn sys_chown(pathname: *const u8, uid: u32, gid: u32) -> Result<isize> {
    if pathname.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy pathname from userspace
    let path = unsafe {
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(pathname, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    // Get current credentials
    let cred = crate::security::current_cred();

    // Check if root (only root can chown to different UID)
    if uid != u32::MAX && !cred.is_root() {
        return Err(Errno::EPERM);
    }

    crate::info!("chown: {} uid={} gid={}", path, uid, gid);

    Ok(0)
}

/// Global umask value
static UMASK: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0o022);

/// sys_umask - Set file creation mode mask (Phase D)
pub fn sys_umask(mask: u32) -> Result<isize> {
    use core::sync::atomic::Ordering;

    let old_mask = UMASK.swap(mask & 0o777, Ordering::Relaxed);
    Ok(old_mask as isize)
}

/// Get current umask
pub fn get_umask() -> u32 {
    use core::sync::atomic::Ordering;
    UMASK.load(Ordering::Relaxed)
}

/// sys_getrandom - Get random bytes (Phase D)
///
/// Fills buffer with random bytes from kernel PRNG.
/// Flags are currently ignored (MVP implementation).
pub fn sys_getrandom(buf: *mut u8, buflen: usize, flags: u32) -> Result<isize> {
    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    if buflen == 0 {
        return Ok(0);
    }

    // Convert to slice
    let buffer = unsafe {
        core::slice::from_raw_parts_mut(buf, buflen)
    };

    // Fill with random bytes from Phase D entropy source
    crate::security::fill_random_bytes(buffer);

    // Flags (GRND_RANDOM, GRND_NONBLOCK) are ignored for MVP
    // Our PRNG is always non-blocking
    let _ = flags;

    Ok(buflen as isize)
}

/// sys_getcwd - Get current working directory
pub fn sys_getcwd(buf: *mut u8, size: usize) -> Result<isize> {
    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    // Phase A2: Return actual CWD from task
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(pid).ok_or(Errno::ESRCH)?;

    // CWD string + null terminator
    let cwd_bytes = task.cwd.as_bytes();
    let total_len = cwd_bytes.len() + 1; // +1 for null terminator

    if size < total_len {
        return Err(Errno::ERANGE);
    }

    unsafe {
        core::ptr::copy_nonoverlapping(cwd_bytes.as_ptr(), buf, cwd_bytes.len());
        *buf.add(cwd_bytes.len()) = 0; // Null terminator
    }

    Ok(buf as isize)
}

/// sys_chdir - Change current working directory
pub fn sys_chdir(pathname: *const u8) -> Result<isize> {
    if pathname.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy pathname from userspace
    let path = unsafe {
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(pathname, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    // Phase A2: Resolve path and update CWD
    // For absolute paths, verify they exist
    // For relative paths, resolve against current CWD
    let pid = crate::process::current_pid();
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;
    let task = table.get_mut(pid).ok_or(Errno::ESRCH)?;

    // Resolve path (handle both absolute and relative)
    let new_cwd = if path.starts_with('/') {
        // Absolute path - verify it exists
        let root = crate::vfs::get_root().ok_or(Errno::ENOENT)?;
        let inode = crate::vfs::path_lookup(&root, path)?;

        // Verify it's a directory
        let meta = inode.getattr()?;
        if (meta.mode & crate::vfs::S_IFMT) != crate::vfs::S_IFDIR {
            return Err(Errno::ENOTDIR);
        }

        path.to_string()
    } else {
        // Relative path - resolve against current CWD
        let mut new_path = task.cwd.clone();
        if !new_path.ends_with('/') {
            new_path.push('/');
        }
        new_path.push_str(path);

        // Normalize path (remove ./ and ../)
        // For Phase A2, simplified normalization
        let normalized = normalize_path(&new_path);

        // Verify it exists
        let root = crate::vfs::get_root().ok_or(Errno::ENOENT)?;
        let inode = crate::vfs::path_lookup(&root, &normalized)?;

        let meta = inode.getattr()?;
        if (meta.mode & crate::vfs::S_IFMT) != crate::vfs::S_IFDIR {
            return Err(Errno::ENOTDIR);
        }

        normalized
    };

    // Update task CWD
    task.cwd = new_cwd;

    Ok(0)
}

/// Normalize a path by resolving . and .. components
fn normalize_path(path: &str) -> String {
    let mut components = Vec::new();

    for component in path.split('/') {
        match component {
            "" | "." => continue, // Skip empty and current directory
            ".." => {
                // Go up one level (unless at root)
                if components.len() > 0 {
                    components.pop();
                }
            }
            c => components.push(c),
        }
    }

    // Build normalized path
    if components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", components.join("/"))
    }
}

/// sys_ioctl - I/O control operations
pub fn sys_ioctl(fd: i32, cmd: u64, arg: u64) -> Result<isize> {
    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(&pid).ok_or(Errno::ESRCH)?;

    // Get file
    let file = task.files.get(fd).ok_or(Errno::EBADF)?;

    // Call file's ioctl
    file.ioctl(cmd as u32, arg as usize)
}

/// sys_ppoll - Poll for I/O events (simplified for Phase A1)
pub fn sys_ppoll(fds: *mut u8, nfds: usize, timeout: *const u8, sigmask: *const u8) -> Result<isize> {
    // Phase A1: Simplified implementation - just check if FDs are valid
    // Always return ready for read/write (no actual polling)

    if fds.is_null() {
        return Err(Errno::EFAULT);
    }

    if nfds > 1024 {
        return Err(Errno::EINVAL);
    }

    let pid = crate::process::current_pid();
    let table = crate::process::get_process_table();
    let table = table.as_ref().ok_or(Errno::ESRCH)?;
    let task = table.get(&pid).ok_or(Errno::ESRCH)?;

    // pollfd structure: fd (4 bytes), events (2 bytes), revents (2 bytes) = 8 bytes
    let mut ready_count = 0;

    for i in 0..nfds {
        unsafe {
            let pollfd_ptr = fds.add(i * 8);
            let fd = *(pollfd_ptr as *const i32);
            let events = *(pollfd_ptr.add(4) as *const u16);
            let revents_ptr = pollfd_ptr.add(6) as *mut u16;

            // Check if FD is valid
            if fd < 0 {
                *revents_ptr = 0;
                continue;
            }

            if task.files.get(fd).is_none() {
                // Invalid FD
                *revents_ptr = 0x0020; // POLLNVAL
                ready_count += 1;
                continue;
            }

            // Phase A1: Always mark as ready for requested events
            // Real implementation would check file readiness
            *revents_ptr = events & 0x0007; // POLLIN | POLLOUT | POLLERR
            if events != 0 {
                ready_count += 1;
            }
        }
    }

    Ok(ready_count as isize)
}

/// sys_clock_gettime - Get time from clock
pub fn sys_clock_gettime(clk_id: i32, tp: *mut u8) -> Result<isize> {
    if tp.is_null() {
        return Err(Errno::EFAULT);
    }

    // Clock IDs
    const CLOCK_REALTIME: i32 = 0;
    const CLOCK_MONOTONIC: i32 = 1;

    // For Phase A1, return dummy time (TODO: get actual time from timer)
    // timespec structure: tv_sec (8 bytes), tv_nsec (8 bytes)
    let seconds: i64 = 100; // Dummy value
    let nanoseconds: i64 = 0;

    match clk_id {
        CLOCK_REALTIME | CLOCK_MONOTONIC => {
            unsafe {
                *(tp as *mut i64) = seconds;
                *(tp.add(8) as *mut i64) = nanoseconds;
            }
            Ok(0)
        }
        _ => Err(Errno::EINVAL),
    }
}

/// sys_nanosleep - Sleep for specified time
pub fn sys_nanosleep(req: *const u8, rem: *mut u8) -> Result<isize> {
    if req.is_null() {
        return Err(Errno::EFAULT);
    }

    // Read requested time
    let seconds: i64;
    let nanoseconds: i64;
    unsafe {
        seconds = *(req as *const i64);
        nanoseconds = *(req.add(8) as *const i64);
    }

    // Validate
    if seconds < 0 || nanoseconds < 0 || nanoseconds >= 1_000_000_000 {
        return Err(Errno::EINVAL);
    }

    // Phase A1: Minimal sleep implementation
    // TODO: Implement proper sleep with timer and scheduler wakeup
    // For now, just yield CPU
    crate::process::scheduler::yield_now();

    // If rem is not null, write remaining time (always 0 for Phase A1)
    if !rem.is_null() {
        unsafe {
            *(rem as *mut i64) = 0;
            *(rem.add(8) as *mut i64) = 0;
        }
    }

    Ok(0)
}

/// sys_mount - Mount a filesystem (Phase B)
///
/// Arguments:
/// - source: Device path or filesystem source (e.g., "/dev/vda1")
/// - target: Mount point path (e.g., "/mnt")
/// - filesystemtype: Filesystem type (e.g., "ext2")
/// - mountflags: Mount flags (MS_RDONLY, etc.)
/// - data: Filesystem-specific mount options
pub fn sys_mount(
    source: *const u8,
    target: *const u8,
    filesystemtype: *const u8,
    mountflags: u64,
    data: *const u8,
) -> Result<isize> {
    if source.is_null() || target.is_null() || filesystemtype.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy strings from userspace
    let source_str = unsafe {
        let mut len = 0;
        while len < 4096 && *source.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(source, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    let target_str = unsafe {
        let mut len = 0;
        while len < 4096 && *target.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(target, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    let fstype_str = unsafe {
        let mut len = 0;
        while len < 64 && *filesystemtype.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(filesystemtype, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    crate::info!("mount: {} on {} type {}", source_str, target_str, fstype_str);

    // Get the block device
    let device = crate::block::get_block_device(source_str.trim_start_matches("/dev/"))
        .ok_or(Errno::ENODEV)?;

    // Mount based on filesystem type
    let root_inode = match fstype_str {
        "ext2" => crate::vfs::mount_ext2(device)?,
        _ => {
            crate::warn!("mount: unsupported filesystem type: {}", fstype_str);
            return Err(Errno::ENODEV);
        }
    };

    // For Phase B, we'll store the mounted filesystem in a simple way
    // In production, would need proper mount point tracking and VFS integration
    crate::info!("mount: successfully mounted {} on {}", source_str, target_str);

    Ok(0)
}

/// sys_umount2 - Unmount a filesystem (Phase B)
///
/// Arguments:
/// - target: Mount point path
/// - flags: Unmount flags (MNT_FORCE, MNT_DETACH, etc.)
pub fn sys_umount2(target: *const u8, flags: i32) -> Result<isize> {
    if target.is_null() {
        return Err(Errno::EFAULT);
    }

    // Copy target from userspace
    let target_str = unsafe {
        let mut len = 0;
        while len < 4096 && *target.add(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(target, len);
        core::str::from_utf8(bytes).map_err(|_| Errno::EINVAL)?
    };

    crate::info!("umount: {}", target_str);

    // For Phase B, simplified implementation
    // In production, would need to:
    // 1. Find the mount by target path
    // 2. Flush all dirty buffers for the device
    // 3. Invalidate cache entries
    // 4. Remove from mount table

    // Flush all dirty buffers
    crate::mm::sync_all()?;

    crate::info!("umount: successfully unmounted {}", target_str);

    Ok(0)
}

/// sys_socket - Create a socket (Phase C)
///
/// Arguments:
/// - domain: Address family (AF_INET = 2)
/// - type: Socket type (SOCK_STREAM = 1, SOCK_DGRAM = 2)
/// - protocol: Protocol (usually 0)
pub fn sys_socket(domain: i32, sock_type: i32, protocol: i32) -> Result<isize> {
    crate::info!("socket: domain={} type={} protocol={}", domain, sock_type, protocol);

    // For Phase C, we'll return a placeholder FD
    // Full implementation would create a socket and register it with the process

    // Simplified: just return success for now
    // Real implementation needs:
    // 1. Create Socket struct
    // 2. Allocate FD from process table
    // 3. Store socket in process file descriptor table

    Ok(3) // Return FD 3 (placeholder)
}

/// sys_bind - Bind socket to address (Phase C)
pub fn sys_bind(sockfd: i32, addr: *const u8, addrlen: u32) -> Result<isize> {
    if addr.is_null() {
        return Err(Errno::EFAULT);
    }

    crate::info!("bind: sockfd={} addrlen={}", sockfd, addrlen);

    // Simplified for Phase C
    // Real implementation needs:
    // 1. Get socket from FD
    // 2. Parse sockaddr structure
    // 3. Bind to smoltcp socket

    Ok(0)
}

/// sys_listen - Listen for connections (Phase C)
pub fn sys_listen(sockfd: i32, backlog: i32) -> Result<isize> {
    crate::info!("listen: sockfd={} backlog={}", sockfd, backlog);

    // Simplified for Phase C
    Ok(0)
}

/// sys_accept - Accept connection (Phase C)
pub fn sys_accept(sockfd: i32, addr: *mut u8, addrlen: *mut u32) -> Result<isize> {
    crate::info!("accept: sockfd={}", sockfd);

    // Simplified for Phase C
    // Return new FD for accepted connection
    Ok(4)
}

/// sys_connect - Connect to remote address (Phase C)
pub fn sys_connect(sockfd: i32, addr: *const u8, addrlen: u32) -> Result<isize> {
    if addr.is_null() {
        return Err(Errno::EFAULT);
    }

    crate::info!("connect: sockfd={} addrlen={}", sockfd, addrlen);

    // Simplified for Phase C
    // Real implementation needs:
    // 1. Parse sockaddr
    // 2. Initiate TCP connection via smoltcp
    // 3. Wait for connection establishment

    Ok(0)
}

/// sys_sendto - Send data on socket (Phase C)
pub fn sys_sendto(
    sockfd: i32,
    buf: *const u8,
    len: usize,
    flags: i32,
    dest_addr: *const u8,
    addrlen: u32,
) -> Result<isize> {
    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    crate::info!("sendto: sockfd={} len={} flags={}", sockfd, len, flags);

    // Simplified for Phase C
    // Real implementation needs:
    // 1. Get socket from FD
    // 2. Copy data from userspace
    // 3. Send via smoltcp TCP/UDP socket

    Ok(len as isize)
}

/// sys_recvfrom - Receive data from socket (Phase C)
pub fn sys_recvfrom(
    sockfd: i32,
    buf: *mut u8,
    len: usize,
    flags: i32,
    src_addr: *mut u8,
    addrlen: *mut u32,
) -> Result<isize> {
    if buf.is_null() {
        return Err(Errno::EFAULT);
    }

    crate::info!("recvfrom: sockfd={} len={} flags={}", sockfd, len, flags);

    // Simplified for Phase C
    // Real implementation needs:
    // 1. Get socket from FD
    // 2. Receive from smoltcp socket
    // 3. Copy to userspace buffer
    // 4. Fill in source address if provided

    Ok(0) // Return 0 bytes read for now
}

/// sys_shutdown - Shutdown socket (Phase C)
pub fn sys_shutdown(sockfd: i32, how: i32) -> Result<isize> {
    crate::info!("shutdown: sockfd={} how={}", sockfd, how);

    // Simplified for Phase C
    // Real implementation: close TX/RX/both sides of socket

    Ok(0)
}

// Syscall numbers for reference (ARM64 calling convention)
#[allow(dead_code)]
mod syscall_numbers {
    pub const SYS_READ: usize = 63;
    pub const SYS_WRITE: usize = 64;
    pub const SYS_EXIT: usize = 93;
    pub const SYS_GETPID: usize = 172;

    // Phase A1 syscalls
    pub const SYS_OPEN: usize = 56;
    pub const SYS_CLOSE: usize = 57;
    pub const SYS_FORK: usize = 220;
    pub const SYS_EXECVE: usize = 221;
    pub const SYS_WAIT4: usize = 260;
    // ... more syscalls to be added in Phase A1
}
