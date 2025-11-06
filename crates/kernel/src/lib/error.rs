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
    EBADF = 9,      // Bad file descriptor
    EAGAIN = 11,    // Try again / Would block
    ENOMEM = 12,    // Out of memory
    EACCES = 13,    // Permission denied
    EFAULT = 14,    // Bad address
    EBUSY = 16,     // Device or resource busy
    EEXIST = 17,    // File exists
    ENODEV = 19,    // No such device
    EINVAL = 22,    // Invalid argument
    ENOSYS = 38,    // Function not implemented
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
        }
    }
}

impl Errno {
    pub fn as_isize(self) -> isize {
        -(self as i32 as isize)
    }
}

pub type Result<T> = core::result::Result<T, Errno>;
