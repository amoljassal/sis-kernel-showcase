//! # Global Descriptor Table (GDT)
//!
//! The GDT is a data structure used by x86_64 CPUs to define memory segments and their
//! protection attributes. While segmentation is largely legacy in 64-bit mode, the GDT
//! is still required for:
//!
//! 1. **Code/Data Segments**: Kernel and user mode execution requires valid CS/DS entries
//! 2. **Task State Segment (TSS)**: Required for privilege level switching and IST stacks
//! 3. **System Call Entry**: SYSCALL/SYSRET instructions use GDT segment selectors
//!
//! ## GDT Structure
//!
//! ```text
//! Index  Segment          DPL   Type      Usage
//! -----  ---------------  ---   -------   ---------------------------
//! 0      Null Descriptor  -     -         Required by CPU (unused)
//! 1      Kernel Code      0     Code      Kernel mode code segment
//! 2      Kernel Data      0     Data      Kernel mode data segment
//! 3      User Data        3     Data      User mode data segment
//! 4      User Code        3     Code      User mode code segment
//! 5      TSS              0     System    Task State Segment
//! ```
//!
//! ## Segment Selectors
//!
//! Segment selectors are 16-bit values loaded into segment registers:
//! ```text
//! Bits 15-3: Index into GDT (0-8191)
//! Bit 2:     Table Indicator (0=GDT, 1=LDT)
//! Bits 1-0:  Requested Privilege Level (RPL) (0=kernel, 3=user)
//! ```
//!
//! ## 64-bit Mode Differences
//!
//! In 64-bit long mode:
//! - Segmentation is flat (base=0, limit=0xFFFFFFFF)
//! - CS must be valid (for CPL checks and SYSCALL/SYSRET)
//! - DS, ES, SS are mostly ignored (except for SYSCALL/SYSRET)
//! - FS and GS can still be used (via MSRs) for TLS and per-CPU data
//!
//! ## Safety
//!
//! Loading a new GDT is a critical operation that can cause crashes if done incorrectly:
//! - The GDT must remain valid and accessible at all times
//! - Segment selectors must point to valid entries
//! - TSS descriptor must be properly configured
//! - Code must not be interrupted during GDT reload

use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::instructions::tables::load_tss;
use x86_64::instructions::segmentation::{Segment, CS, DS, ES, FS, GS, SS};
use lazy_static::lazy_static;
use spin::Mutex;

/// Global GDT instance
///
/// Uses lazy_static to ensure it's initialized exactly once and lives for the
/// entire program lifetime (required since CPU always accesses it).
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // Entry 0: Null descriptor (required by CPU, never loaded)
        // The CPU requires the first entry to be null

        // Entry 1: Kernel code segment (Ring 0)
        // Used for kernel mode execution
        let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());

        // Entry 2: Kernel data segment (Ring 0)
        // Used for kernel mode data access
        let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());

        // Entry 3: User data segment (Ring 3)
        // Note: User data must come before user code for SYSRET compatibility
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());

        // Entry 4: User code segment (Ring 3)
        // Used for user mode execution
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());

        // Entry 5: TSS (Task State Segment)
        // Will be initialized later, placeholder for now
        // TSS is required for privilege level changes and interrupt stacks
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(unsafe {
            &*(&super::tss::TSS as *const _)
        }));

        (gdt, Selectors {
            kernel_code_selector,
            kernel_data_selector,
            user_code_selector,
            user_data_selector,
            tss_selector,
        })
    };
}

/// Segment selectors for all GDT entries
///
/// These are the values that get loaded into segment registers (CS, DS, etc.)
/// and used by SYSCALL/SYSRET instructions.
#[derive(Debug)]
struct Selectors {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Initialize and load the Global Descriptor Table
///
/// This function must be called during early boot, before any segmentation-related
/// operations. It loads the GDT and sets up all segment registers.
///
/// # Safety
///
/// Must be called exactly once during boot, with interrupts disabled.
/// After calling this function:
/// - CS will point to kernel code segment
/// - DS, ES, SS will point to kernel data segment
/// - FS, GS will point to kernel data segment (can be changed later for TLS/per-CPU)
/// - TSS will be loaded (providing interrupt stack switching)
pub unsafe fn init_gdt() {
    // Load the GDT into the CPU's GDTR register
    GDT.0.load();

    // Set all segment registers to their correct values
    // CS (Code Segment) - must be updated via a far jump/return
    CS::set_reg(GDT.1.kernel_code_selector);

    // Data segments - can be set directly
    DS::set_reg(GDT.1.kernel_data_selector);
    ES::set_reg(GDT.1.kernel_data_selector);
    SS::set_reg(GDT.1.kernel_data_selector);

    // FS and GS - initially kernel data, but can be changed later
    // FS: typically used for thread-local storage (TLS)
    // GS: typically used for per-CPU data
    FS::set_reg(GDT.1.kernel_data_selector);
    GS::set_reg(GDT.1.kernel_data_selector);

    // Load TSS (Task State Segment)
    // This is required for privilege level changes (user ↔ kernel)
    // and for using IST (Interrupt Stack Table) for critical exceptions
    load_tss(GDT.1.tss_selector);
}

/// Get the kernel code segment selector
///
/// Used by SYSCALL/SYSRET MSR setup
pub fn kernel_code_selector() -> SegmentSelector {
    GDT.1.kernel_code_selector
}

/// Get the kernel data segment selector
///
/// Used by SYSCALL/SYSRET MSR setup
pub fn kernel_data_selector() -> SegmentSelector {
    GDT.1.kernel_data_selector
}

/// Get the user code segment selector
///
/// Used by SYSCALL/SYSRET MSR setup and for returning to userspace
pub fn user_code_selector() -> SegmentSelector {
    GDT.1.user_code_selector
}

/// Get the user data segment selector
///
/// Used by SYSCALL/SYSRET MSR setup and for returning to userspace
pub fn user_data_selector() -> SegmentSelector {
    GDT.1.user_data_selector
}

/// Get the TSS segment selector
///
/// Used for loading the TSS
pub fn tss_selector() -> SegmentSelector {
    GDT.1.tss_selector
}

/// Update the TSS descriptor in the GDT
///
/// This is called after the TSS is properly initialized (in tss.rs)
/// to update the GDT entry with the correct TSS address.
///
/// # Safety
///
/// The TSS must be valid and live for the entire program lifetime.
pub unsafe fn update_tss_descriptor(tss: &'static TaskStateSegment) {
    // Note: This is a simplified version. In reality, the TSS is already
    // set up correctly by the lazy_static initialization. This function
    // is provided for completeness and future use.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdt_selectors() {
        // Verify segment selectors have correct RPL (Requested Privilege Level)
        // Kernel segments should have RPL=0
        assert_eq!(kernel_code_selector().rpl(), 0);
        assert_eq!(kernel_data_selector().rpl(), 0);

        // User segments should have RPL=3
        assert_eq!(user_code_selector().rpl(), 3);
        assert_eq!(user_data_selector().rpl(), 3);

        // TSS should have RPL=0
        assert_eq!(tss_selector().rpl(), 0);
    }

    #[test]
    fn test_segment_order() {
        // Verify that segments are in the correct order for SYSCALL/SYSRET
        // SYSRET requires: user_data_selector + 8 = user_code_selector
        let user_data_idx = user_data_selector().index();
        let user_code_idx = user_code_selector().index();
        assert_eq!(user_code_idx, user_data_idx + 1);
    }
}
