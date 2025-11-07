/// PSCI (Power State Coordination Interface) for ARM64
///
/// Provides interface for CPU power management and secondary CPU bring-up.
/// PSCI is a standard ARM interface implemented by firmware (TF-A, ATF, or QEMU).

/// PSCI function IDs (SMC calling convention)
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PsciFunction {
    /// Get PSCI version
    Version = 0x8400_0000,
    /// Bring a CPU online
    CpuOn = 0xC400_0003,
    /// Take current CPU offline
    CpuOff = 0x8400_0002,
    /// Suspend current CPU
    CpuSuspend = 0xC400_0001,
    /// Reset system
    SystemReset = 0x8400_0009,
    /// Power off system
    SystemOff = 0x8400_0008,
    /// Query CPU features
    Features = 0x8400_000A,
}

/// PSCI return codes
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsciError {
    Success = 0,
    NotSupported = -1,
    InvalidParameters = -2,
    Denied = -3,
    AlreadyOn = -4,
    OnPending = -5,
    InternalFailure = -6,
    NotPresent = -7,
    Disabled = -8,
    InvalidAddress = -9,
}

impl PsciError {
    pub fn from_i32(code: i32) -> Result<(), Self> {
        match code {
            0 => Ok(()),
            -1 => Err(Self::NotSupported),
            -2 => Err(Self::InvalidParameters),
            -3 => Err(Self::Denied),
            -4 => Err(Self::AlreadyOn),
            -5 => Err(Self::OnPending),
            -6 => Err(Self::InternalFailure),
            -7 => Err(Self::NotPresent),
            -8 => Err(Self::Disabled),
            -9 => Err(Self::InvalidAddress),
            _ => Err(Self::InternalFailure),
        }
    }
}

/// Make a PSCI SMC call
///
/// Uses the ARM SMC (Secure Monitor Call) instruction to invoke PSCI functions.
#[inline]
unsafe fn psci_call(function: u32, arg0: u64, arg1: u64, arg2: u64) -> i32 {
    let result: i32;

    core::arch::asm!(
        "mov x0, {function}",
        "mov x1, {arg0}",
        "mov x2, {arg1}",
        "mov x3, {arg2}",
        "smc #0",
        "sxtw {result}, w0",
        function = in(reg) function as u64,
        arg0 = in(reg) arg0,
        arg1 = in(reg) arg1,
        arg2 = in(reg) arg2,
        result = out(reg) result,
        options(nomem, nostack)
    );

    result
}

/// Get PSCI version
pub fn psci_version() -> u32 {
    unsafe {
        psci_call(PsciFunction::Version as u32, 0, 0, 0) as u32
    }
}

/// Bring a secondary CPU online
///
/// # Arguments
/// * `target_cpu` - MPIDR value of CPU to bring up (e.g., CPU 1 = 0x1, CPU 2 = 0x2)
/// * `entry_point` - Physical address of entry point function
/// * `context_id` - Context ID passed to entry point (e.g., CPU ID)
pub fn cpu_on(target_cpu: u64, entry_point: u64, context_id: u64) -> Result<(), PsciError> {
    let result = unsafe {
        psci_call(PsciFunction::CpuOn as u32, target_cpu, entry_point, context_id)
    };

    PsciError::from_i32(result)
}

/// Power off the current CPU
pub fn cpu_off() -> ! {
    unsafe {
        psci_call(PsciFunction::CpuOff as u32, 0, 0, 0);
    }

    // Should never return, but just in case
    loop {
        core::hint::spin_loop();
    }
}

/// Reset the system
pub fn system_reset() -> ! {
    crate::info!("PSCI: System reset requested");

    unsafe {
        psci_call(PsciFunction::SystemReset as u32, 0, 0, 0);
    }

    // Should never return
    loop {
        core::hint::spin_loop();
    }
}

/// Power off the system
pub fn system_off() -> ! {
    crate::info!("PSCI: System power off requested");

    unsafe {
        psci_call(PsciFunction::SystemOff as u32, 0, 0, 0);
    }

    // Should never return
    loop {
        core::hint::spin_loop();
    }
}

/// Check if a PSCI function is supported
pub fn is_feature_supported(function: PsciFunction) -> bool {
    let result = unsafe {
        psci_call(PsciFunction::Features as u32, function as u64, 0, 0)
    };

    result >= 0
}

/// Get current CPU's MPIDR (Multiprocessor Affinity Register)
#[inline]
pub fn get_mpidr() -> u64 {
    let mpidr: u64;
    unsafe {
        core::arch::asm!(
            "mrs {}, mpidr_el1",
            out(reg) mpidr,
            options(nomem, nostack, preserves_flags)
        );
    }
    mpidr & 0xFF // Extract CPU ID from bits [7:0]
}

/// Get CPU ID from MPIDR
#[inline]
pub fn cpu_id_from_mpidr(mpidr: u64) -> usize {
    (mpidr & 0xFF) as usize
}

/// Get current CPU ID
#[inline]
pub fn current_cpu_id() -> usize {
    cpu_id_from_mpidr(get_mpidr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psci_error_conversion() {
        assert_eq!(PsciError::from_i32(0), Ok(()));
        assert_eq!(PsciError::from_i32(-1), Err(PsciError::NotSupported));
        assert_eq!(PsciError::from_i32(-2), Err(PsciError::InvalidParameters));
    }
}
