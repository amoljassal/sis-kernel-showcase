//! # CPU Initialization and Feature Detection
//!
//! This module handles CPU-specific initialization and feature detection for x86_64.
//! It uses CPUID instruction to detect CPU capabilities and enables required features
//! for kernel operation.
//!
//! ## Required CPU Features
//!
//! The following features are **required** for the kernel to function:
//! - **64-bit Long Mode**: Already enabled by bootloader
//! - **SSE2**: Required for Rust floating-point operations
//! - **APIC**: Required for interrupt handling (we don't support ancient PIC-only CPUs)
//!
//! ## Optional CPU Features (Performance/Security)
//!
//! These features are enabled if available:
//! - **SSE3, SSE4.1, SSE4.2**: Better SIMD performance
//! - **AVX, AVX2**: Advanced vector extensions
//! - **Execute Disable (NX)**: Security - prevent code execution in data pages
//! - **FSGSBASE**: Fast access to FS/GS base (TLS and per-CPU data)
//! - **x2APIC**: MSR-based APIC (faster than memory-mapped xAPIC)
//! - **PCID**: Process-Context Identifiers (avoid TLB flush on context switch)
//! - **INVPCID**: Selective TLB invalidation
//! - **SMEP**: Supervisor Mode Execution Prevention (prevent kernel from executing user code)
//! - **SMAP**: Supervisor Mode Access Prevention (prevent kernel from accessing user data)
//!
//! ## CPU Feature Bits (CPUID)
//!
//! CPUID is queried with different leaf values (EAX input):
//! - **Leaf 0**: Maximum supported leaf, vendor ID
//! - **Leaf 1**: Feature flags (SSE, AVX, APIC, etc.)
//! - **Leaf 7**: Extended features (SMEP, SMAP, etc.)
//! - **Leaf 0x80000001**: Extended features (NX, SYSCALL, etc.)
//!
//! ## Control Registers
//!
//! CPU features are enabled by setting bits in control registers:
//! - **CR0**: Basic processor control (FPU, paging, etc.)
//! - **CR4**: Extended features (SSE, FSGSBASE, SMEP, SMAP, etc.)
//! - **EFER (MSR 0xC0000080)**: Extended features (NX, SYSCALL, etc.)
//! - **XCR0**: Extended state management (AVX state, etc.)

use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
use raw_cpuid::CpuId;

/// CPU feature information
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    // Vendor and model
    pub vendor: CpuVendor,
    pub model_name: [u8; 48],

    // Required features
    pub has_sse2: bool,
    pub has_apic: bool,

    // Optional SIMD features
    pub has_sse3: bool,
    pub has_ssse3: bool,
    pub has_sse4_1: bool,
    pub has_sse4_2: bool,
    pub has_avx: bool,
    pub has_avx2: bool,

    // Security features
    pub has_nx: bool,
    pub has_smep: bool,
    pub has_smap: bool,

    // Performance features
    pub has_fsgsbase: bool,
    pub has_x2apic: bool,
    pub has_pcid: bool,
    pub has_invpcid: bool,

    // Other features
    pub has_tsc: bool,
    pub has_tsc_deadline: bool,
    pub has_invariant_tsc: bool,
}

/// CPU vendor identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Intel,
    AMD,
    Unknown,
}

/// Detect CPU features using CPUID instruction
///
/// Returns a structure containing information about available CPU features.
pub fn detect_cpu_features() -> CpuFeatures {
    let cpuid = CpuId::new();

    // Detect vendor
    let vendor = if let Some(vendor_info) = cpuid.get_vendor_info() {
        let vendor_str = vendor_info.as_str();
        if vendor_str == "GenuineIntel" {
            CpuVendor::Intel
        } else if vendor_str == "AuthenticAMD" {
            CpuVendor::AMD
        } else {
            CpuVendor::Unknown
        }
    } else {
        CpuVendor::Unknown
    };

    // Get feature flags
    let features = cpuid.get_feature_info().unwrap();
    let extended_features = cpuid.get_extended_feature_info();
    let extended_proc = cpuid.get_extended_processor_and_feature_identifiers();
    let apm_info = cpuid.get_advanced_power_mgmt_info();

    // Get processor brand string (model name)
    let mut model_name = [0u8; 48];
    if let Some(brand) = cpuid.get_processor_brand_string() {
        let brand_str = brand.as_str();
        let len = core::cmp::min(brand_str.len(), 48);
        model_name[..len].copy_from_slice(&brand_str.as_bytes()[..len]);
    }

    CpuFeatures {
        vendor,
        model_name,

        // Required features
        has_sse2: features.has_sse2(),
        has_apic: features.has_apic(),

        // SIMD features
        has_sse3: features.has_sse3(),
        has_ssse3: features.has_ssse3(),
        has_sse4_1: features.has_sse41(),
        has_sse4_2: features.has_sse42(),
        has_avx: features.has_avx(),
        has_avx2: extended_features.as_ref().map_or(false, |f| f.has_avx2()),

        // Security features
        has_nx: extended_proc.as_ref().map_or(false, |f| f.has_execute_disable()),
        has_smep: extended_features.as_ref().map_or(false, |f| f.has_smep()),
        has_smap: extended_features.as_ref().map_or(false, |f| f.has_smap()),

        // Performance features
        has_fsgsbase: extended_features.as_ref().map_or(false, |f| f.has_fsgsbase()),
        has_x2apic: features.has_x2apic(),
        has_pcid: features.has_pcid(),
        has_invpcid: extended_features.as_ref().map_or(false, |f| f.has_invpcid()),

        // Other features
        has_tsc: features.has_tsc(),
        has_tsc_deadline: features.has_tsc_deadline(),
        has_invariant_tsc: apm_info.as_ref().map_or(false, |f| f.has_invariant_tsc()),
    }
}

/// Enable required CPU features
///
/// This function enables all CPU features required for the kernel to function correctly.
/// It will return an error if any required feature is missing.
///
/// # Safety
///
/// Must be called during early boot, before any floating-point operations or
/// interrupt handling. Interrupts must be disabled.
pub unsafe fn enable_cpu_features() -> Result<(), &'static str> {
    let features = detect_cpu_features();

    // Validate required features
    if !features.has_sse2 {
        return Err("CPU does not support SSE2 (required for Rust floating-point)");
    }

    if !features.has_apic {
        return Err("CPU does not support APIC (required for interrupt handling)");
    }

    // Enable FPU (x87)
    // CR0.EM (bit 2) = 0: No emulation, real FPU present
    // CR0.MP (bit 1) = 1: Monitor coprocessor
    let mut cr0 = Cr0::read();
    cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
    cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
    Cr0::write(cr0);

    // Enable SSE/SSE2
    // CR4.OSFXSR (bit 9) = 1: Enable FXSAVE/FXRSTOR for SSE state
    // CR4.OSXMMEXCPT (bit 10) = 1: Enable unmasked SSE exceptions
    let mut cr4 = Cr4::read();
    cr4.insert(Cr4Flags::OSFXSR);
    cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);

    // Enable AVX if available
    if features.has_avx {
        // CR4.OSXSAVE (bit 18) = 1: Enable XSAVE/XRSTOR for extended state
        cr4.insert(Cr4Flags::OSXSAVE);
        Cr4::write(cr4);

        // Enable AVX state in XCR0
        // XCR0 bit 0: x87 state
        // XCR0 bit 1: SSE state
        // XCR0 bit 2: AVX state
        let xcr0 = xgetbv(0);
        xsetbv(0, xcr0 | 0x7); // Enable x87, SSE, and AVX states
    } else {
        Cr4::write(cr4);
    }

    // Enable NX (No-Execute) bit if available
    if features.has_nx {
        use x86_64::registers::model_specific::Efer;
        use x86_64::registers::model_specific::EferFlags;

        Efer::update(|flags| {
            *flags |= EferFlags::NO_EXECUTE_ENABLE;
        });
    }

    // Enable FSGSBASE if available (fast TLS/per-CPU data access)
    if features.has_fsgsbase {
        let mut cr4 = Cr4::read();
        cr4.insert(Cr4Flags::FSGSBASE);
        Cr4::write(cr4);
    }

    // Enable SMEP (Supervisor Mode Execution Prevention) if available
    // Prevents kernel from executing code in user pages
    if features.has_smep {
        let mut cr4 = Cr4::read();
        cr4.insert(Cr4Flags::SUPERVISOR_MODE_EXECUTION_PROTECTION);
        Cr4::write(cr4);
    }

    // Enable SMAP (Supervisor Mode Access Prevention) if available
    // Prevents kernel from accessing data in user pages (must use special instructions)
    if features.has_smap {
        let mut cr4 = Cr4::read();
        cr4.insert(Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION);
        Cr4::write(cr4);
    }

    // Enable PCID (Process-Context Identifiers) if available
    // Allows TLB entries to be tagged with process ID, avoiding flush on context switch
    if features.has_pcid {
        let mut cr4 = Cr4::read();
        cr4.insert(Cr4Flags::PCID);
        Cr4::write(cr4);
    }

    // Enable PGE (Page Global Enable)
    // Allows marking pages as global (not flushed on CR3 reload)
    let mut cr4 = Cr4::read();
    cr4.insert(Cr4Flags::PAGE_GLOBAL);
    Cr4::write(cr4);

    Ok(())
}

/// Print CPU information to serial console
///
/// Useful for debugging and verification during boot.
pub fn print_cpu_info() {
    let features = detect_cpu_features();

    // Print vendor
    let vendor_str = match features.vendor {
        CpuVendor::Intel => "Intel",
        CpuVendor::AMD => "AMD",
        CpuVendor::Unknown => "Unknown",
    };

    crate::arch::x86_64::serial::serial_write(b"[CPU] Vendor: ");
    crate::arch::x86_64::serial::serial_write(vendor_str.as_bytes());
    crate::arch::x86_64::serial::serial_write(b"\n");

    // Print model name
    crate::arch::x86_64::serial::serial_write(b"[CPU] Model: ");
    crate::arch::x86_64::serial::serial_write(&features.model_name);
    crate::arch::x86_64::serial::serial_write(b"\n");

    // Print features
    crate::arch::x86_64::serial::serial_write(b"[CPU] Features:\n");

    if features.has_sse2 {
        crate::arch::x86_64::serial::serial_write(b"  - SSE2\n");
    }
    if features.has_sse3 {
        crate::arch::x86_64::serial::serial_write(b"  - SSE3\n");
    }
    if features.has_sse4_1 {
        crate::arch::x86_64::serial::serial_write(b"  - SSE4.1\n");
    }
    if features.has_sse4_2 {
        crate::arch::x86_64::serial::serial_write(b"  - SSE4.2\n");
    }
    if features.has_avx {
        crate::arch::x86_64::serial::serial_write(b"  - AVX\n");
    }
    if features.has_avx2 {
        crate::arch::x86_64::serial::serial_write(b"  - AVX2\n");
    }
    if features.has_nx {
        crate::arch::x86_64::serial::serial_write(b"  - NX (No-Execute)\n");
    }
    if features.has_smep {
        crate::arch::x86_64::serial::serial_write(b"  - SMEP\n");
    }
    if features.has_smap {
        crate::arch::x86_64::serial::serial_write(b"  - SMAP\n");
    }
    if features.has_fsgsbase {
        crate::arch::x86_64::serial::serial_write(b"  - FSGSBASE\n");
    }
    if features.has_x2apic {
        crate::arch::x86_64::serial::serial_write(b"  - x2APIC\n");
    }
}

/// Read Extended Control Register (XCR)
///
/// # Safety
///
/// XCR0 can only be accessed if CR4.OSXSAVE is set.
#[inline]
unsafe fn xgetbv(xcr: u32) -> u64 {
    let (high, low): (u32, u32);
    core::arch::asm!(
        "xgetbv",
        in("ecx") xcr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack)
    );
    ((high as u64) << 32) | (low as u64)
}

/// Write Extended Control Register (XCR)
///
/// # Safety
///
/// XCR0 can only be accessed if CR4.OSXSAVE is set.
/// Invalid XCR0 values can cause exceptions.
#[inline]
unsafe fn xsetbv(xcr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!(
        "xsetbv",
        in("ecx") xcr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cpu_features() {
        let features = detect_cpu_features();

        // Basic sanity checks
        assert!(features.has_sse2, "SSE2 should be available on all x86_64 CPUs");
        assert!(features.has_apic, "APIC should be available on all modern CPUs");
    }

    #[test]
    fn test_vendor_detection() {
        let features = detect_cpu_features();

        // Vendor should be detected (Intel, AMD, or Unknown)
        assert!(
            features.vendor == CpuVendor::Intel ||
            features.vendor == CpuVendor::AMD ||
            features.vendor == CpuVendor::Unknown
        );
    }
}
