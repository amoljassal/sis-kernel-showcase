// Syscall infrastructure
// Phase A0 - Minimal syscall dispatcher

pub mod uaccess;

use crate::lib::error::{Errno, Result};

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
        220 => sys_fork(),
        221 => sys_execve(args[0] as *const u8, args[1] as *const *const u8, args[2] as *const *const u8),
        260 => sys_wait4(args[0] as i32, args[1] as *mut i32, args[2] as i32, args[3] as *mut u8),

        // Memory management
        214 => sys_brk(args[0] as *const u8),
        222 => sys_mmap(args[0] as *mut u8, args[1] as usize, args[2] as i32, args[3] as i32, args[4] as i32, args[5] as i64),
        215 => sys_munmap(args[0] as *mut u8, args[1] as usize),

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

/// sys_openat - Open a file (Phase A1: treat as open for absolute paths)
pub fn sys_openat(dirfd: i32, pathname: *const u8, flags: i32, mode: u32) -> Result<isize> {
    // For Phase A1, only support absolute paths (dirfd is ignored if path is absolute)
    let _ = dirfd;

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

    // Convert flags to OpenFlags
    let open_flags = crate::vfs::OpenFlags::from_bits_truncate(flags as u32);

    // Open or create file
    let file = if open_flags.contains(crate::vfs::OpenFlags::O_CREAT) {
        // Create new file if doesn't exist
        match crate::vfs::open(path, open_flags) {
            Ok(f) => f,
            Err(Errno::ENOENT) => crate::vfs::create(path, mode, open_flags)?,
            Err(e) => return Err(e),
        }
    } else {
        crate::vfs::open(path, open_flags)?
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
    let meta = file.inode.getattr()?;

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
    if !file.inode.is_dir() {
        return Err(Errno::ENOTDIR);
    }

    // Read directory entries
    let entries = file.inode.readdir()?;

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

    // Get parent task and create child
    let mut table = crate::process::get_process_table();
    let table = table.as_mut().ok_or(Errno::ESRCH)?;

    let parent = table.get(parent_pid).ok_or(Errno::ESRCH)?;
    let mut child = crate::process::Task::fork_from(parent, child_pid);

    // Set up COW for parent and child
    crate::mm::setup_cow_for_fork(&mut child.mm)
        .map_err(|_| Errno::ENOMEM)?;

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
            crate::vfs::OpenFlags::RDWR,
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
