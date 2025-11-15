// Syscall Input Validation
// Phase 4.1 - Production Readiness Plan
//
// Comprehensive input validation for all syscalls to prevent security vulnerabilities

use crate::lib::error::{Errno, Result};

/// Maximum syscall number supported
pub const MAX_SYSCALL_NUM: usize = 512;

/// Maximum file descriptor value
pub const MAX_FD: i32 = 1024;

/// Maximum write/read size (prevent overflow attacks)
pub const MAX_IO_SIZE: usize = 0x7ffff000; // ~2GB

/// Maximum path length
pub const MAX_PATH_LEN: usize = 4096;

/// Maximum buffer size for various operations
pub const MAX_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// Userspace memory range (simplified check)
pub const USER_SPACE_START: u64 = 0x1000;
pub const USER_SPACE_END: u64 = 0x0000_ffff_ffff_ffff;

/// Kernel memory range (must not be accessed from userspace)
pub const KERNEL_SPACE_START: u64 = 0xffff_0000_0000_0000;

/// Validation result type
pub type ValidationResult<T> = Result<T>;

/// Syscall input validator
pub struct SyscallValidator;

impl SyscallValidator {
    /// Validate syscall number
    #[inline]
    pub fn validate_syscall_number(nr: usize) -> ValidationResult<usize> {
        if nr > MAX_SYSCALL_NUM {
            return Err(Errno::ENOSYS);
        }
        Ok(nr)
    }

    /// Validate file descriptor
    #[inline]
    pub fn validate_fd(fd: i32) -> ValidationResult<i32> {
        if fd < 0 {
            return Err(Errno::EBADF);
        }
        if fd >= MAX_FD {
            return Err(Errno::EBADF);
        }
        Ok(fd)
    }

    /// Validate user pointer (basic check)
    #[inline]
    pub fn validate_user_ptr<T>(ptr: *const T, len: usize) -> ValidationResult<*const T> {
        let addr = ptr as u64;

        // Null pointer check
        if addr == 0 {
            return Err(Errno::EFAULT);
        }

        // Check for kernel space addresses
        if addr >= KERNEL_SPACE_START {
            return Err(Errno::EFAULT);
        }

        // Check for underflow in user space
        if addr < USER_SPACE_START {
            return Err(Errno::EFAULT);
        }

        // Check for overflow
        let end_addr = addr.checked_add(len as u64)
            .ok_or(Errno::EFAULT)?;

        // Ensure end address is still in user space
        if end_addr > USER_SPACE_END {
            return Err(Errno::EFAULT);
        }

        // Check for wrap-around
        if end_addr < addr {
            return Err(Errno::EFAULT);
        }

        Ok(ptr)
    }

    /// Validate mutable user pointer
    #[inline]
    pub fn validate_user_ptr_mut<T>(ptr: *mut T, len: usize) -> ValidationResult<*mut T> {
        Self::validate_user_ptr(ptr as *const T, len)?;
        Ok(ptr)
    }

    /// Validate read buffer
    #[inline]
    pub fn validate_read_buffer(ptr: *const u8, len: usize) -> ValidationResult<(*const u8, usize)> {
        if len > MAX_IO_SIZE {
            return Err(Errno::EINVAL);
        }

        Self::validate_user_ptr(ptr, len)?;
        Ok((ptr, len))
    }

    /// Validate write buffer
    #[inline]
    pub fn validate_write_buffer(ptr: *mut u8, len: usize) -> ValidationResult<(*mut u8, usize)> {
        if len > MAX_IO_SIZE {
            return Err(Errno::EINVAL);
        }

        Self::validate_user_ptr_mut(ptr, len)?;
        Ok((ptr, len))
    }

    /// Validate string pointer (null-terminated)
    pub fn validate_string_ptr(ptr: *const u8, max_len: usize) -> ValidationResult<*const u8> {
        if ptr.is_null() {
            return Err(Errno::EFAULT);
        }

        Self::validate_user_ptr(ptr, max_len)?;

        // Try to find null terminator within max_len
        unsafe {
            for i in 0..max_len {
                let byte = ptr.add(i).read_volatile();
                if byte == 0 {
                    return Ok(ptr);
                }
            }
        }

        // No null terminator found within max_len
        Err(Errno::ENAMETOOLONG)
    }

    /// Validate path string
    #[inline]
    pub fn validate_path(ptr: *const u8) -> ValidationResult<*const u8> {
        Self::validate_string_ptr(ptr, MAX_PATH_LEN)
    }

    /// Validate flags value (ensure only valid bits are set)
    #[inline]
    pub fn validate_flags(flags: u32, valid_mask: u32) -> ValidationResult<u32> {
        if flags & !valid_mask != 0 {
            return Err(Errno::EINVAL);
        }
        Ok(flags)
    }

    /// Validate mode bits
    #[inline]
    pub fn validate_mode(mode: u32) -> ValidationResult<u32> {
        const VALID_MODE_MASK: u32 = 0o7777; // rwxrwxrwx + sticky/setuid/setgid
        if mode & !VALID_MODE_MASK != 0 {
            return Err(Errno::EINVAL);
        }
        Ok(mode)
    }

    /// Validate offset for seek operations
    #[inline]
    pub fn validate_offset(offset: i64) -> ValidationResult<i64> {
        // Negative offsets are only valid for SEEK_CUR/SEEK_END
        // For absolute offsets, must be >= 0
        if offset < 0 && offset != -1 {
            // -1 is sometimes used as a sentinel value
            // Real validation depends on whence parameter
            return Err(Errno::EINVAL);
        }
        Ok(offset)
    }

    /// Validate whence parameter for lseek
    #[inline]
    pub fn validate_whence(whence: i32) -> ValidationResult<i32> {
        const SEEK_SET: i32 = 0;
        const SEEK_CUR: i32 = 1;
        const SEEK_END: i32 = 2;
        const SEEK_DATA: i32 = 3;
        const SEEK_HOLE: i32 = 4;

        match whence {
            SEEK_SET | SEEK_CUR | SEEK_END | SEEK_DATA | SEEK_HOLE => Ok(whence),
            _ => Err(Errno::EINVAL),
        }
    }

    /// Validate signal number
    #[inline]
    pub fn validate_signal(sig: i32) -> ValidationResult<i32> {
        const MAX_SIGNAL: i32 = 64;
        if sig < 0 || sig > MAX_SIGNAL {
            return Err(Errno::EINVAL);
        }
        Ok(sig)
    }

    /// Validate PID
    #[inline]
    pub fn validate_pid(pid: i32) -> ValidationResult<i32> {
        // PID 0 is valid for certain operations
        // Negative PIDs are valid for process groups
        // PID -1 has special meaning (all processes)
        if pid < -1 {
            return Err(Errno::EINVAL);
        }
        Ok(pid)
    }

    /// Validate UID/GID
    #[inline]
    pub fn validate_uid(uid: u32) -> ValidationResult<u32> {
        // UIDs are typically 0-65535, but can be higher
        // No specific validation needed for now
        Ok(uid)
    }

    /// Validate size parameter (ensure no overflow)
    #[inline]
    pub fn validate_size(size: usize, max_size: usize) -> ValidationResult<usize> {
        if size > max_size {
            return Err(Errno::EINVAL);
        }
        Ok(size)
    }

    /// Validate count parameter
    #[inline]
    pub fn validate_count(count: usize, max_count: usize) -> ValidationResult<usize> {
        if count > max_count {
            return Err(Errno::EINVAL);
        }
        Ok(count)
    }

    /// Validate timeout value
    #[inline]
    pub fn validate_timeout(timeout_ms: i64) -> ValidationResult<i64> {
        if timeout_ms < -1 {
            // -1 means infinite timeout
            return Err(Errno::EINVAL);
        }
        Ok(timeout_ms)
    }

    /// Validate dirfd (directory file descriptor for *at syscalls)
    #[inline]
    pub fn validate_dirfd(dirfd: i32) -> ValidationResult<i32> {
        const AT_FDCWD: i32 = -100; // Special value for current working directory

        if dirfd == AT_FDCWD {
            return Ok(dirfd);
        }

        Self::validate_fd(dirfd)
    }

    /// Validate array of pointers (e.g., argv, envp)
    pub fn validate_ptr_array(ptr_array: *const *const u8, max_count: usize) -> ValidationResult<usize> {
        if ptr_array.is_null() {
            // Empty array is valid
            return Ok(0);
        }

        Self::validate_user_ptr(ptr_array, max_count * core::mem::size_of::<*const u8>())?;

        unsafe {
            for i in 0..max_count {
                let ptr = ptr_array.add(i).read_volatile();
                if ptr.is_null() {
                    // Null terminator for array
                    return Ok(i);
                }

                // Validate each string in the array
                Self::validate_string_ptr(ptr, MAX_PATH_LEN)?;
            }
        }

        // Array too long or not null-terminated
        Err(Errno::E2BIG)
    }

    /// Validate socket domain
    #[inline]
    pub fn validate_socket_domain(domain: i32) -> ValidationResult<i32> {
        const AF_UNIX: i32 = 1;
        const AF_INET: i32 = 2;
        const AF_INET6: i32 = 10;

        match domain {
            AF_UNIX | AF_INET | AF_INET6 => Ok(domain),
            _ => Err(Errno::EAFNOSUPPORT),
        }
    }

    /// Validate socket type
    #[inline]
    pub fn validate_socket_type(sock_type: i32) -> ValidationResult<i32> {
        const SOCK_STREAM: i32 = 1;
        const SOCK_DGRAM: i32 = 2;
        const SOCK_RAW: i32 = 3;

        let base_type = sock_type & 0xf; // Remove flags
        match base_type {
            SOCK_STREAM | SOCK_DGRAM | SOCK_RAW => Ok(sock_type),
            _ => Err(Errno::EINVAL),
        }
    }

    /// Validate protocol
    #[inline]
    pub fn validate_protocol(protocol: i32) -> ValidationResult<i32> {
        // Protocol 0 means default protocol for the socket type
        // Specific protocols are validated by socket implementation
        if protocol < 0 {
            return Err(Errno::EINVAL);
        }
        Ok(protocol)
    }

    /// Validate mmap protection flags
    #[inline]
    pub fn validate_mmap_prot(prot: i32) -> ValidationResult<i32> {
        const PROT_READ: i32 = 1;
        const PROT_WRITE: i32 = 2;
        const PROT_EXEC: i32 = 4;
        const VALID_PROT_MASK: i32 = PROT_READ | PROT_WRITE | PROT_EXEC;

        if prot & !VALID_PROT_MASK != 0 {
            return Err(Errno::EINVAL);
        }
        Ok(prot)
    }

    /// Validate mmap flags
    #[inline]
    pub fn validate_mmap_flags(flags: i32) -> ValidationResult<i32> {
        const MAP_SHARED: i32 = 1;
        const MAP_PRIVATE: i32 = 2;
        const MAP_FIXED: i32 = 0x10;
        const MAP_ANONYMOUS: i32 = 0x20;
        const VALID_MAP_MASK: i32 = MAP_SHARED | MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS;

        // Must have either MAP_SHARED or MAP_PRIVATE (but not both)
        let sharing_flags = flags & (MAP_SHARED | MAP_PRIVATE);
        if sharing_flags == 0 || sharing_flags == (MAP_SHARED | MAP_PRIVATE) {
            return Err(Errno::EINVAL);
        }

        if flags & !VALID_MAP_MASK != 0 {
            return Err(Errno::EINVAL);
        }

        Ok(flags)
    }

    /// Validate memory address alignment
    #[inline]
    pub fn validate_alignment(addr: u64, alignment: usize) -> ValidationResult<u64> {
        if alignment == 0 || !alignment.is_power_of_two() {
            return Err(Errno::EINVAL);
        }

        if addr as usize & (alignment - 1) != 0 {
            return Err(Errno::EINVAL);
        }

        Ok(addr)
    }
}

/// Validation statistics for monitoring
pub struct ValidationStats {
    pub total_validations: u64,
    pub failed_validations: u64,
    pub null_pointer_errors: u64,
    pub invalid_fd_errors: u64,
    pub buffer_overflow_errors: u64,
    pub invalid_flags_errors: u64,
}

impl ValidationStats {
    pub const fn new() -> Self {
        Self {
            total_validations: 0,
            failed_validations: 0,
            null_pointer_errors: 0,
            invalid_fd_errors: 0,
            buffer_overflow_errors: 0,
            invalid_flags_errors: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_syscall_number() {
        assert!(SyscallValidator::validate_syscall_number(0).is_ok());
        assert!(SyscallValidator::validate_syscall_number(511).is_ok());
        assert!(SyscallValidator::validate_syscall_number(512).is_err());
        assert!(SyscallValidator::validate_syscall_number(9999).is_err());
    }

    #[test]
    fn test_validate_fd() {
        assert!(SyscallValidator::validate_fd(0).is_ok());
        assert!(SyscallValidator::validate_fd(1023).is_ok());
        assert!(SyscallValidator::validate_fd(-1).is_err());
        assert!(SyscallValidator::validate_fd(1024).is_err());
    }

    #[test]
    fn test_validate_whence() {
        assert!(SyscallValidator::validate_whence(0).is_ok()); // SEEK_SET
        assert!(SyscallValidator::validate_whence(1).is_ok()); // SEEK_CUR
        assert!(SyscallValidator::validate_whence(2).is_ok()); // SEEK_END
        assert!(SyscallValidator::validate_whence(99).is_err());
    }

    #[test]
    fn test_validate_signal() {
        assert!(SyscallValidator::validate_signal(1).is_ok());
        assert!(SyscallValidator::validate_signal(64).is_ok());
        assert!(SyscallValidator::validate_signal(0).is_ok());
        assert!(SyscallValidator::validate_signal(-1).is_err());
        assert!(SyscallValidator::validate_signal(65).is_err());
    }

    #[test]
    fn test_validate_pid() {
        assert!(SyscallValidator::validate_pid(0).is_ok());
        assert!(SyscallValidator::validate_pid(1).is_ok());
        assert!(SyscallValidator::validate_pid(-1).is_ok()); // All processes
        assert!(SyscallValidator::validate_pid(-2).is_err());
    }
}
