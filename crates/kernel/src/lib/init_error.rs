use core::fmt;

#[derive(Debug)]
pub enum InitError {
    NotFound(&'static str),
    Timeout(&'static str),
    Invalid(&'static str),
    MmioMap { base: usize, size: usize },
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InitError::NotFound(what) => write!(f, "device not found: {}", what),
            InitError::Timeout(what) => write!(f, "timeout waiting for {}", what),
            InitError::Invalid(what) => write!(f, "invalid parameter: {}", what),
            InitError::MmioMap { base, size } => write!(f, "mmio mapping failed: base={:#x} size={:#x}", base, size),
        }
    }
}

impl From<InitError> for crate::lib::error::Errno {
    fn from(e: InitError) -> Self {
        match e {
            InitError::NotFound(_) => Self::ENOENT,
            InitError::Timeout(_) => Self::ETIMEDOUT,
            InitError::Invalid(_) => Self::EINVAL,
            InitError::MmioMap { .. } => Self::EFAULT,
        }
    }
}
