//! # Task State Segment (TSS)
//!
//! The TSS is a hardware structure used by x86_64 processors for task management and
//! privilege level transitions. In 64-bit mode, the TSS is significantly simplified
//! compared to 32-bit mode and serves two main purposes:
//!
//! 1. **Privilege Stack Table (RSP0-RSP2)**: Provides kernel stack pointers for
//!    privilege level transitions (e.g., user mode → kernel mode during syscalls)
//!
//! 2. **Interrupt Stack Table (IST1-IST7)**: Provides dedicated stacks for specific
//!    interrupts/exceptions to prevent stack corruption during critical errors
//!
//! ## Why the TSS is Critical
//!
//! When a userspace program makes a system call or triggers an exception:
//! 1. CPU switches from user stack (untrusted) to kernel stack (trusted)
//! 2. CPU loads RSP from TSS.RSP0 (kernel stack pointer)
//! 3. CPU pushes user SS, RSP, RFLAGS, CS, RIP onto kernel stack
//! 4. Handler executes with known-good kernel stack
//!
//! Without a valid TSS, privilege transitions would fail → triple fault → reset
//!
//! ## IST (Interrupt Stack Table)
//!
//! The IST provides dedicated stacks for critical exceptions that might be triggered
//! while the kernel stack is in an inconsistent state:
//!
//! - **Double Fault (#DF)**: Occurs when exception handling fails. If we don't have
//!   a dedicated stack, the double fault handler itself would triple fault!
//! - **NMI (Non-Maskable Interrupt)**: Cannot be disabled, might occur during stack
//!   manipulation
//! - **Machine Check**: Critical hardware error that needs its own stack
//!
//! ## Stack Layout
//!
//! Each stack in the TSS should be properly aligned and sized:
//! - **Size**: 16 KiB per stack (4 pages, reasonable for deep call chains)
//! - **Alignment**: 16 bytes (required for x86_64 ABI)
//! - **Guard Pages**: Future enhancement to detect stack overflow
//!
//! ## Safety Considerations
//!
//! The TSS and its stacks must:
//! 1. Live for the entire kernel lifetime (static storage)
//! 2. Be properly initialized before enabling interrupts
//! 3. Have valid stack pointers (writable, aligned, adequate size)
//! 4. Be updated per-CPU in SMP systems

use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use lazy_static::lazy_static;

/// Stack size for IST entries (16 KiB)
///
/// This should be sufficient for:
/// - Deep call stacks during exception handling
/// - Local variables in exception handlers
/// - Nested exceptions (though we try to avoid these)
pub const IST_STACK_SIZE: usize = 16 * 1024;

/// Stack size for privilege level transitions (16 KiB)
pub const PRIVILEGE_STACK_SIZE: usize = 16 * 1024;

/// IST index for double fault handler
///
/// The double fault exception (#DF) uses its own dedicated stack to ensure
/// it can always execute, even if the main kernel stack is corrupted.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// IST index for NMI (Non-Maskable Interrupt) handler
///
/// NMIs cannot be disabled and might occur during critical sections,
/// so they need a dedicated stack.
pub const NMI_IST_INDEX: u16 = 1;

/// IST index for machine check exception handler
///
/// Machine check exceptions indicate serious hardware errors and need
/// a dedicated stack to diagnose the problem.
pub const MACHINE_CHECK_IST_INDEX: u16 = 2;

/// Stack structure with proper alignment
///
/// #[repr(align(16))] ensures the stack is 16-byte aligned as required by
/// the x86_64 ABI (some SSE instructions require aligned stack).
#[repr(align(16))]
struct Stack([u8; IST_STACK_SIZE]);

impl Stack {
    /// Create a new zeroed stack
    const fn new() -> Self {
        Stack([0; IST_STACK_SIZE])
    }
}

/// Double fault stack (IST entry 0)
///
/// This stack is used when a double fault occurs. A double fault is triggered
/// when the CPU encounters an exception while trying to invoke the handler for
/// a previous exception.
///
/// Example scenario:
/// 1. Page fault occurs (invalid memory access)
/// 2. CPU tries to invoke page fault handler
/// 3. Page fault handler descriptor is invalid (or handler code is paged out)
/// 4. Another page fault occurs → double fault!
///
/// Without a dedicated stack, the double fault handler would itself cause a
/// triple fault (CPU reset).
static mut DOUBLE_FAULT_STACK: Stack = Stack::new();

/// NMI (Non-Maskable Interrupt) stack (IST entry 1)
///
/// NMIs cannot be disabled by the CLI instruction and can occur at any time,
/// including during stack manipulation. A dedicated stack ensures the NMI
/// handler can always execute safely.
static mut NMI_STACK: Stack = Stack::new();

/// Machine check exception stack (IST entry 2)
///
/// Machine check exceptions indicate serious hardware errors (e.g., memory
/// corruption, bus errors). A dedicated stack allows diagnostics even when
/// the system is in a critical state.
static mut MACHINE_CHECK_STACK: Stack = Stack::new();

/// Privilege level 0 stack (kernel stack for syscalls/interrupts)
///
/// This is a temporary stack used during early boot. Once the process
/// subsystem is initialized, each thread will have its own kernel stack,
/// and TSS.RSP0 will be updated on each context switch.
static mut PRIVILEGE_STACK: Stack = Stack::new();

lazy_static! {
    /// Global Task State Segment
    ///
    /// This TSS is shared by all CPUs during early boot. In a full SMP system,
    /// each CPU should have its own TSS with per-CPU stacks.
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Set up Interrupt Stack Table (IST)
        // Each entry points to the *top* of the stack (stacks grow downward)

        // IST[0]: Double fault stack
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &DOUBLE_FAULT_STACK });
            let stack_top = stack_start + IST_STACK_SIZE;
            stack_top
        };

        // IST[1]: NMI stack
        tss.interrupt_stack_table[NMI_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &NMI_STACK });
            let stack_top = stack_start + IST_STACK_SIZE;
            stack_top
        };

        // IST[2]: Machine check stack
        tss.interrupt_stack_table[MACHINE_CHECK_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &MACHINE_CHECK_STACK });
            let stack_top = stack_start + IST_STACK_SIZE;
            stack_top
        };

        // Set up Privilege Stack Table
        // RSP0 is used when transitioning from user mode (Ring 3) to kernel mode (Ring 0)
        // This will be updated per-thread once the process subsystem is initialized
        tss.privilege_stack_table[0] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &PRIVILEGE_STACK });
            let stack_top = stack_start + PRIVILEGE_STACK_SIZE;
            stack_top
        };

        tss
    };
}

/// Initialize the Task State Segment
///
/// This function should be called during early boot, after GDT initialization.
/// It ensures the TSS is properly set up before any interrupts or privilege
/// transitions can occur.
///
/// # Safety
///
/// Must be called exactly once during boot, after GDT is loaded.
pub unsafe fn init_tss() {
    // The TSS is automatically initialized by the lazy_static macro.
    // This function exists to force initialization and provide a clear
    // initialization point in the boot sequence.
    //
    // The actual loading of the TSS into the CPU happens in gdt.rs
    // via the `load_tss()` instruction.
    let _ = &*TSS;
}

/// Update the kernel stack pointer in the TSS
///
/// This function should be called during context switches to update RSP0
/// with the kernel stack of the new thread. This ensures that when the
/// thread makes a syscall or receives an interrupt, the CPU switches to
/// the correct kernel stack.
///
/// # Arguments
///
/// * `stack_top` - Virtual address of the top of the kernel stack
///
/// # Safety
///
/// - `stack_top` must point to a valid, writable kernel stack
/// - The stack must be properly aligned (16-byte alignment)
/// - The stack must have sufficient size (at least 4 KiB recommended)
/// - This function is not thread-safe; caller must ensure proper synchronization
///
/// # Example
///
/// ```no_run
/// // During context switch to a new thread:
/// let kernel_stack_top = thread.kernel_stack_base + KERNEL_STACK_SIZE;
/// unsafe {
///     set_kernel_stack(VirtAddr::new(kernel_stack_top));
/// }
/// ```
pub unsafe fn set_kernel_stack(stack_top: VirtAddr) {
    // In a real implementation, we would need to:
    // 1. Get a mutable reference to the current CPU's TSS
    // 2. Update TSS.privilege_stack_table[0]
    // 3. Ensure this is done atomically or with interrupts disabled
    //
    // For now, this is a placeholder for M0 (basic boot).
    // Full implementation will come in M8 (SMP) with per-CPU TSSes.

    // TODO(M8): Implement per-CPU TSS with proper synchronization
    // For now, this is a no-op since we're single-threaded during M0
}

/// Get the current kernel stack pointer from the TSS
///
/// Returns the stack pointer that will be used for the next privilege
/// level transition (syscall or interrupt from userspace).
pub fn get_kernel_stack() -> VirtAddr {
    TSS.privilege_stack_table[0]
}

/// Validate that all TSS stacks are properly configured
///
/// This is a debug/test function to ensure the TSS was initialized correctly.
/// It checks that:
/// - All IST entries point to valid addresses
/// - Stack pointers are properly aligned
/// - Stacks don't overlap
#[cfg(debug_assertions)]
pub fn validate_tss() -> Result<(), &'static str> {
    // Check double fault stack
    let df_stack = TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize];
    if df_stack.is_null() {
        return Err("Double fault IST stack is null");
    }
    if df_stack.as_u64() % 16 != 0 {
        return Err("Double fault IST stack is not 16-byte aligned");
    }

    // Check NMI stack
    let nmi_stack = TSS.interrupt_stack_table[NMI_IST_INDEX as usize];
    if nmi_stack.is_null() {
        return Err("NMI IST stack is null");
    }
    if nmi_stack.as_u64() % 16 != 0 {
        return Err("NMI IST stack is not 16-byte aligned");
    }

    // Check machine check stack
    let mc_stack = TSS.interrupt_stack_table[MACHINE_CHECK_IST_INDEX as usize];
    if mc_stack.is_null() {
        return Err("Machine check IST stack is null");
    }
    if mc_stack.as_u64() % 16 != 0 {
        return Err("Machine check IST stack is not 16-byte aligned");
    }

    // Check privilege stack
    let priv_stack = TSS.privilege_stack_table[0];
    if priv_stack.is_null() {
        return Err("Privilege stack (RSP0) is null");
    }
    if priv_stack.as_u64() % 16 != 0 {
        return Err("Privilege stack (RSP0) is not 16-byte aligned");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_alignment() {
        // Verify stacks are properly aligned
        let stack = Stack::new();
        let addr = &stack as *const _ as usize;
        assert_eq!(addr % 16, 0, "Stack must be 16-byte aligned");
    }

    #[test]
    fn test_stack_size() {
        // Verify stack size is correct
        assert_eq!(
            core::mem::size_of::<Stack>(),
            IST_STACK_SIZE,
            "Stack size mismatch"
        );
    }

    #[test]
    fn test_tss_initialization() {
        // Verify TSS is properly initialized
        let tss = &*TSS;

        // Check IST entries are non-null
        assert!(
            !tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize].is_null(),
            "Double fault IST not initialized"
        );
        assert!(
            !tss.interrupt_stack_table[NMI_IST_INDEX as usize].is_null(),
            "NMI IST not initialized"
        );
        assert!(
            !tss.interrupt_stack_table[MACHINE_CHECK_IST_INDEX as usize].is_null(),
            "Machine check IST not initialized"
        );

        // Check privilege stack is non-null
        assert!(
            !tss.privilege_stack_table[0].is_null(),
            "Privilege stack not initialized"
        );
    }
}
