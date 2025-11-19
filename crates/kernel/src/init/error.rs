/// Kernel initialization errors with detailed context
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// Early initialization failed (MMU, UART, heap)
    EarlyInit(&'static str),

    /// Platform detection/initialization failed
    PlatformInit(&'static str),

    /// Memory subsystem initialization failed
    MemoryInit(&'static str),

    /// Driver initialization failed
    DriverInit(&'static str),

    /// Subsystem initialization failed (VFS, network, etc.)
    SubsystemInit(&'static str),

    /// Late initialization failed (scheduler, shell)
    LateInit(&'static str),

    /// Wrong exception level
    InvalidExceptionLevel(u8),
}

impl core::fmt::Display for KernelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EarlyInit(msg) => write!(f, "early init failed: {}", msg),
            Self::PlatformInit(msg) => write!(f, "platform init failed: {}", msg),
            Self::MemoryInit(msg) => write!(f, "memory init failed: {}", msg),
            Self::DriverInit(msg) => write!(f, "driver init failed: {}", msg),
            Self::SubsystemInit(msg) => write!(f, "subsystem init failed: {}", msg),
            Self::LateInit(msg) => write!(f, "late init failed: {}", msg),
            Self::InvalidExceptionLevel(el) => write!(f, "invalid exception level: EL{}", el),
        }
    }
}

pub type KernelResult<T> = core::result::Result<T, KernelError>;

