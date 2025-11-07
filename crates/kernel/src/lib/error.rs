// Kernel error handling and errno definitions
// Phase A0 - Basic error types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    OutOfMemory,
    InvalidArgument,
    PermissionDenied,
    NotFound,
    AlreadyExists,
    IoError,
    Interrupted,
    WouldBlock,
    TimedOut,
    NotSupported,
    BadFileDescriptor,
    BadAddress,
    SecurityViolation,  // Phase D: W^X and security policy violations
    NotInitialized,     // Component not initialized
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Errno {
    EPERM = 1,      // Operation not permitted
    ENOENT = 2,     // No such file or directory
    ESRCH = 3,      // No such process
    EINTR = 4,      // Interrupted system call
    EIO = 5,        // I/O error
    ENXIO = 6,      // No such device or address
    E2BIG = 7,      // Argument list too long
    ENOEXEC = 8,    // Exec format error
    EBADF = 9,      // Bad file descriptor
    ECHILD = 10,    // No child processes
    EAGAIN = 11,    // Try again / Would block
    ENOMEM = 12,    // Out of memory
    EACCES = 13,    // Permission denied
    EFAULT = 14,    // Bad address
    EBUSY = 16,     // Device or resource busy
    EEXIST = 17,    // File exists
    ENODEV = 19,    // No such device
    ENOTDIR = 20,   // Not a directory
    EISDIR = 21,    // Is a directory
    EINVAL = 22,    // Invalid argument
    EMFILE = 24,    // Too many open files
    ENOTTY = 25,    // Not a typewriter
    ENOSPC = 28,    // No space left on device
    ESPIPE = 29,    // Illegal seek
    EROFS = 30,     // Read-only file system
    EPIPE = 32,     // Broken pipe
    ERANGE = 34,    // Math result not representable
    ENOSYS = 38,    // Function not implemented
    ENAMETOOLONG = 36, // File name too long
    EMSGSIZE = 90,  // Message too long
    ENOTSUP = 95,   // Operation not supported
    EAFNOSUPPORT = 97, // Address family not supported
    EADDRNOTAVAIL = 99, // Cannot assign requested address
    ENOTSOCK = 88,  // Socket operation on non-socket
    ETIMEDOUT = 110, // Connection timed out
}

impl From<KernelError> for Errno {
    fn from(err: KernelError) -> Self {
        match err {
            KernelError::OutOfMemory => Errno::ENOMEM,
            KernelError::InvalidArgument => Errno::EINVAL,
            KernelError::PermissionDenied => Errno::EACCES,
            KernelError::NotFound => Errno::ENOENT,
            KernelError::AlreadyExists => Errno::EEXIST,
            KernelError::IoError => Errno::EIO,
            KernelError::Interrupted => Errno::EINTR,
            KernelError::WouldBlock => Errno::EAGAIN,
            KernelError::TimedOut => Errno::ETIMEDOUT,
            KernelError::NotSupported => Errno::ENOSYS,
            KernelError::BadFileDescriptor => Errno::EBADF,
            KernelError::BadAddress => Errno::EFAULT,
            KernelError::SecurityViolation => Errno::EACCES,
            KernelError::NotInitialized => Errno::EINVAL,
        }
    }
}

impl Errno {
    pub fn as_isize(self) -> isize {
        -(self as i32 as isize)
    }

    /// Convert a negated errno value (e.g., -2) to an Errno variant
    pub fn from_negated_i32(value: i32) -> Self {
        match -value {
            1 => Errno::EPERM,
            2 => Errno::ENOENT,
            3 => Errno::ESRCH,
            4 => Errno::EINTR,
            5 => Errno::EIO,
            6 => Errno::ENXIO,
            7 => Errno::E2BIG,
            8 => Errno::ENOEXEC,
            9 => Errno::EBADF,
            10 => Errno::ECHILD,
            11 => Errno::EAGAIN,
            12 => Errno::ENOMEM,
            13 => Errno::EACCES,
            14 => Errno::EFAULT,
            16 => Errno::EBUSY,
            17 => Errno::EEXIST,
            19 => Errno::ENODEV,
            20 => Errno::ENOTDIR,
            21 => Errno::EISDIR,
            22 => Errno::EINVAL,
            24 => Errno::EMFILE,
            25 => Errno::ENOTTY,
            28 => Errno::ENOSPC,
            29 => Errno::ESPIPE,
            30 => Errno::EROFS,
            32 => Errno::EPIPE,
            34 => Errno::ERANGE,
            36 => Errno::ENAMETOOLONG,
            38 => Errno::ENOSYS,
            90 => Errno::EMSGSIZE,
            95 => Errno::ENOTSUP,
            99 => Errno::EADDRNOTAVAIL,
            110 => Errno::ETIMEDOUT,
            _ => Errno::EINVAL, // Default to EINVAL for unknown values
        }
    }
}

pub type Result<T> = core::result::Result<T, Errno>;
