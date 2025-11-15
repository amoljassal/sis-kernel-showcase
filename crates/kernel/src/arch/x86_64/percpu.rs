//! # Per-CPU Data Structures
//!
//! This module implements per-CPU (processor-local) data structures for x86_64.
//! Each CPU has its own private data that can be accessed quickly without locks.
//!
//! ## Overview
//!
//! Per-CPU data is essential for SMP (Symmetric Multiprocessing) systems where
//! multiple CPUs run kernel code simultaneously. Each CPU needs its own:
//! - Kernel stack for syscalls and interrupts
//! - Task State Segment (TSS)
//! - Current process/thread pointer
//! - CPU-local statistics
//! - Temporary storage for context switching
//!
//! ## GS Segment Register
//!
//! On x86_64, the GS segment register is used to access per-CPU data:
//! - **GS.base** points to the `CpuLocal` structure for the current CPU
//! - Assembly: `mov rax, gs:[offset]` reads from CPU-local memory
//! - No locks needed - each CPU has its own copy
//!
//! ### Setting GS Base
//!
//! Two methods to set GS base:
//!
//! 1. **WRGSBASE instruction** (modern CPUs with FSGSBASE feature):
//!    ```asm
//!    wrgsbase rax  ; Set GS.base to value in RAX
//!    ```
//!
//! 2. **MSR write** (older CPUs or when FSGSBASE not available):
//!    ```rust
//!    wrmsr(IA32_GS_BASE, cpu_data_addr);
//!    ```
//!
//! ## Memory Layout
//!
//! ```text
//! CpuLocal structure (per CPU):
//! +0x00: self_ptr      (pointer to this structure)
//! +0x08: cpu_id        (logical CPU number)
//! +0x0C: apic_id       (Local APIC ID)
//! +0x10: kernel_stack  (top of kernel stack for syscalls)
//! +0x18: user_rsp_tmp  (temporary storage for user RSP during syscall)
//! +0x20: tss_rsp0      (kernel stack for interrupts, stored in TSS)
//! +0x28: current_task  (pointer to current process, if any)
//! +0x30: stats         (CPU statistics)
//! ```
//!
//! ## Usage
//!
//! ### Initialization (BSP - Boot Processor)
//!
//! ```rust
//! // During early boot
//! percpu::init_bsp();
//! ```
//!
//! ### Accessing Per-CPU Data
//!
//! ```rust
//! // Get current CPU's data (read-only)
//! let cpu = CpuLocal::current();
//! println!("Running on CPU {}", cpu.cpu_id);
//!
//! // Get mutable access
//! let cpu = CpuLocal::current_mut();
//! cpu.stats.syscalls += 1;
//! ```
//!
//! ### In Assembly (syscall entry)
//!
//! ```asm
//! # Get kernel stack pointer
//! mov rsp, gs:[0x10]  # Load kernel_stack field
//!
//! # Save user RSP temporarily
//! mov gs:[0x18], rsp  # Store in user_rsp_tmp field
//! ```
//!
//! ## Safety Considerations
//!
//! - GS base must be set before accessing per-CPU data
//! - Each CPU must initialize its own per-CPU area
//! - The `self_ptr` field must always point to the structure itself
//! - Stack pointers must be valid and properly aligned
//! - Context switches must save/restore GS base if needed

use core::ptr::NonNull;
use x86_64::VirtAddr;

/// MSR for GS base (kernel GS)
const IA32_GS_BASE: u32 = 0xC0000101;

/// MSR for kernel GS base (swapped with GS on SWAPGS)
const IA32_KERNEL_GS_BASE: u32 = 0xC0000102;

/// Size of per-CPU kernel stack (64 KiB)
///
/// This is larger than the M4 static stack (16 KiB) to handle
/// deep call chains and large stack frames.
pub const KERNEL_STACK_SIZE: usize = 65536;

/// Per-CPU data structure
///
/// This structure contains all CPU-local data. It is accessed via the GS
/// segment register, allowing fast, lock-free access to CPU-specific data.
///
/// # Memory Layout
///
/// The `self_ptr` field **must** be the first field. This allows:
/// ```asm
/// mov rax, gs:[0]  # Get pointer to CpuLocal structure
/// ```
///
/// # Alignment
///
/// The structure is aligned to cache line size (64 bytes) to avoid false
/// sharing between CPUs.
#[repr(C, align(64))]
pub struct CpuLocal {
    /// Pointer to this structure (must be first field)
    ///
    /// This allows `mov rax, gs:[0]` to get the CpuLocal pointer.
    /// Always points to the structure itself.
    self_ptr: *const CpuLocal,

    /// Logical CPU ID (0-based)
    ///
    /// This is the CPU number assigned by the kernel, not the APIC ID.
    /// BSP (Boot Processor) is always CPU 0.
    pub cpu_id: u32,

    /// Local APIC ID
    ///
    /// Hardware identifier for this CPU's APIC. Used for sending IPIs.
    pub apic_id: u32,

    /// Kernel stack pointer (top of stack)
    ///
    /// This is the stack used for syscalls and interrupts.
    /// Points to the **top** of the stack (stack grows downward).
    pub kernel_stack: u64,

    /// Temporary storage for user RSP during syscall entry
    ///
    /// The syscall entry code stores the user's RSP here before
    /// switching to the kernel stack.
    pub user_rsp_tmp: u64,

    /// Kernel stack for TSS (interrupts)
    ///
    /// This is stored in TSS.RSP0 and used when interrupts occur in user mode.
    /// May be different from kernel_stack in future implementations.
    pub tss_rsp0: u64,

    /// Pointer to current task/process (null if idle)
    ///
    /// Points to the currently running process on this CPU.
    /// Null when running the idle task or during early boot.
    pub current_task: u64,  // Will be Arc<Process> in future

    /// CPU statistics
    pub stats: CpuStats,

    /// Padding to next cache line
    _padding: [u64; 1],
}

/// CPU statistics
///
/// Tracks CPU usage and events for this processor.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct CpuStats {
    /// Number of syscalls executed
    pub syscalls: u64,

    /// Number of interrupts handled
    pub interrupts: u64,

    /// Number of context switches
    pub context_switches: u64,

    /// Idle time in TSC ticks
    pub idle_ticks: u64,
}

impl CpuLocal {
    /// Create a new per-CPU data structure
    ///
    /// # Arguments
    ///
    /// * `cpu_id` - Logical CPU number (0 for BSP)
    /// * `apic_id` - Local APIC ID for this CPU
    /// * `kernel_stack` - Top of kernel stack
    ///
    /// # Returns
    ///
    /// A new `CpuLocal` structure with fields initialized.
    pub fn new(cpu_id: u32, apic_id: u32, kernel_stack: VirtAddr) -> Self {
        Self {
            self_ptr: core::ptr::null(),  // Set by init_bsp/init_ap
            cpu_id,
            apic_id,
            kernel_stack: kernel_stack.as_u64(),
            user_rsp_tmp: 0,
            tss_rsp0: kernel_stack.as_u64(),
            current_task: 0,
            stats: CpuStats::default(),
            _padding: [0; 1],
        }
    }

    /// Create an empty per-CPU data structure (for static initialization)
    ///
    /// # Returns
    ///
    /// A zeroed `CpuLocal` structure.
    pub const fn empty() -> Self {
        Self {
            self_ptr: core::ptr::null(),
            cpu_id: 0,
            apic_id: 0,
            kernel_stack: 0,
            user_rsp_tmp: 0,
            tss_rsp0: 0,
            current_task: 0,
            stats: CpuStats {
                syscalls: 0,
                interrupts: 0,
                context_switches: 0,
            },
            _padding: [0; 1],
        }
    }

    /// Get reference to current CPU's data (read-only)
    ///
    /// This is safe to call from any context (syscall, interrupt, kernel thread).
    ///
    /// # Returns
    ///
    /// Immutable reference to the current CPU's `CpuLocal` structure.
    ///
    /// # Safety
    ///
    /// Assumes GS base has been initialized by `init_bsp()` or `init_ap()`.
    #[inline]
    pub fn current() -> &'static Self {
        unsafe {
            let ptr: *const CpuLocal;
            core::arch::asm!(
                "mov {}, gs:[0]",
                out(reg) ptr,
                options(pure, nomem, nostack, preserves_flags)
            );
            &*ptr
        }
    }

    /// Get mutable reference to current CPU's data
    ///
    /// This allows modifying CPU-local statistics and state.
    ///
    /// # Returns
    ///
    /// Mutable reference to the current CPU's `CpuLocal` structure.
    ///
    /// # Safety
    ///
    /// Assumes GS base has been initialized. Caller must ensure no other
    /// code on this CPU is simultaneously accessing the same fields.
    #[inline]
    pub fn current_mut() -> &'static mut Self {
        unsafe {
            let ptr: *mut CpuLocal;
            core::arch::asm!(
                "mov {}, gs:[0]",
                out(reg) ptr,
                options(pure, nomem, nostack, preserves_flags)
            );
            &mut *ptr
        }
    }

    /// Get kernel stack pointer for current CPU
    ///
    /// Returns the top of the kernel stack. This is used by syscall entry.
    ///
    /// # Returns
    ///
    /// Virtual address of kernel stack top.
    #[inline]
    pub fn kernel_stack_top() -> u64 {
        unsafe {
            let stack_top: u64;
            core::arch::asm!(
                "mov {}, gs:[0x10]",  // Offset 0x10 = kernel_stack field
                out(reg) stack_top,
                options(pure, nomem, nostack, preserves_flags)
            );
            stack_top
        }
    }
}

/// Global storage for BSP (Boot Processor) per-CPU data
///
/// The BSP's CpuLocal structure is statically allocated since it's needed
/// before the heap is available. APs allocate theirs dynamically.
static mut BSP_CPU_DATA: CpuLocal = CpuLocal {
    self_ptr: core::ptr::null(),
    cpu_id: 0,
    apic_id: 0,
    kernel_stack: 0,
    user_rsp_tmp: 0,
    tss_rsp0: 0,
    current_task: 0,
    stats: CpuStats {
        syscalls: 0,
        interrupts: 0,
        context_switches: 0,
        idle_ticks: 0,
    },
    _padding: [0; 1],
};

/// BSP kernel stack (64 KiB)
///
/// Statically allocated for the boot processor. Stack grows downward.
static mut BSP_KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

/// Initialize per-CPU data for BSP (Boot Processor)
///
/// This must be called early in the boot process, before any code
/// tries to access per-CPU data.
///
/// # Safety
///
/// - Must be called exactly once during boot
/// - Must be called before enabling interrupts or syscalls
/// - Must be called on the BSP only
pub unsafe fn init_bsp() {
    crate::arch::x86_64::serial::serial_write(b"[PERCPU] Initializing BSP per-CPU data...\n");

    // Get APIC ID for BSP
    let apic_id = if let Some(apic) = crate::arch::x86_64::apic::get() {
        apic.id()
    } else {
        0  // Fallback if APIC not available
    };

    // Calculate kernel stack top (stack grows downward)
    let stack_bottom = BSP_KERNEL_STACK.as_ptr() as u64;
    let stack_top = stack_bottom + KERNEL_STACK_SIZE as u64;

    // Initialize BSP CPU data
    BSP_CPU_DATA = CpuLocal::new(0, apic_id, VirtAddr::new(stack_top));
    BSP_CPU_DATA.self_ptr = &BSP_CPU_DATA as *const _;

    // Set GS base to point to BSP_CPU_DATA
    set_gs_base(&BSP_CPU_DATA as *const _ as u64);

    crate::arch::x86_64::serial::serial_write(b"[PERCPU] BSP CPU ID: 0, APIC ID: ");
    print_u32(apic_id);
    crate::arch::x86_64::serial::serial_write(b"\n[PERCPU] Kernel stack: 0x");
    print_hex(stack_top);
    crate::arch::x86_64::serial::serial_write(b"\n[PERCPU] Per-CPU data at: 0x");
    print_hex(&BSP_CPU_DATA as *const _ as u64);
    crate::arch::x86_64::serial::serial_write(b"\n");

    // Verify GS base was set correctly
    let gs_base = get_gs_base();
    if gs_base != &BSP_CPU_DATA as *const _ as u64 {
        crate::arch::x86_64::serial::serial_write(b"[PERCPU] ERROR: GS base mismatch!\n");
        panic!("Per-CPU initialization failed");
    }

    crate::arch::x86_64::serial::serial_write(b"[PERCPU] BSP per-CPU data initialized\n");
}

/// Set GS base to given address
///
/// Uses WRGSBASE if available (faster), otherwise uses WRMSR.
///
/// # Arguments
///
/// * `base` - Physical or virtual address to set as GS base
///
/// # Safety
///
/// - Must point to valid memory
/// - Should point to a `CpuLocal` structure
unsafe fn set_gs_base(base: u64) {
    // Check if WRGSBASE is supported
    if cpu_has_fsgsbase() {
        // Use WRGSBASE instruction (faster)
        core::arch::asm!(
            "wrgsbase {}",
            in(reg) base,
            options(nostack, preserves_flags)
        );
    } else {
        // Use MSR (slower but works on all CPUs)
        wrmsr(IA32_GS_BASE, base);
    }
}

/// Get current GS base
///
/// # Returns
///
/// Current value of GS.base
unsafe fn get_gs_base() -> u64 {
    if cpu_has_fsgsbase() {
        let base: u64;
        core::arch::asm!(
            "rdgsbase {}",
            out(reg) base,
            options(pure, nomem, nostack, preserves_flags)
        );
        base
    } else {
        rdmsr(IA32_GS_BASE)
    }
}

/// Check if CPU supports FSGSBASE instructions
///
/// # Returns
///
/// `true` if RDGSBASE/WRGSBASE are available
fn cpu_has_fsgsbase() -> bool {
    // Check CPUID for FSGSBASE support
    use raw_cpuid::CpuId;
    let cpuid = CpuId::new();
    if let Some(features) = cpuid.get_extended_feature_info() {
        features.has_fsgsbase()
    } else {
        false
    }
}

/// Write to Model-Specific Register
///
/// # Arguments
///
/// * `msr` - MSR number
/// * `value` - 64-bit value to write
///
/// # Safety
///
/// Writing to MSRs can change CPU behavior. Only safe for known MSRs.
unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nostack, preserves_flags)
    );
}

/// Read from Model-Specific Register
///
/// # Arguments
///
/// * `msr` - MSR number
///
/// # Returns
///
/// 64-bit value from MSR
unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags)
    );
    ((high as u64) << 32) | (low as u64)
}

/// Initialize Application Processor per-CPU data
///
/// Allocates and configures per-CPU data structures for an AP.
///
/// # Arguments
///
/// * `cpu_id` - Sequential CPU ID (1+ for APs)
/// * `apic_id` - Local APIC ID for this AP
///
/// # Safety
///
/// Must be called exactly once per AP during startup, after heap initialization.
pub unsafe fn init_ap(cpu_id: u32, apic_id: u32) {
    crate::arch::x86_64::serial::serial_write(b"[PERCPU] Initializing AP ");
    print_u32(cpu_id);
    crate::arch::x86_64::serial::serial_write(b" per-CPU data...\n");

    // For M8 Part 2, we'll allocate AP kernel stacks and per-CPU data statically
    // A full implementation would use the heap allocator

    // For now, create per-CPU data in static arrays (limited to 16 CPUs)
    const MAX_APS: usize = 15; // BSP + 15 APs = 16 total

    static mut AP_CPU_DATA: [CpuLocal; MAX_APS] = [CpuLocal::empty(); MAX_APS];
    static mut AP_KERNEL_STACKS: [[u8; KERNEL_STACK_SIZE]; MAX_APS] =
        [[0; KERNEL_STACK_SIZE]; MAX_APS];

    if cpu_id == 0 || cpu_id as usize > MAX_APS {
        crate::arch::x86_64::serial::serial_write(b"[PERCPU] Invalid CPU ID: ");
        print_u32(cpu_id);
        crate::arch::x86_64::serial::serial_write(b"\n");
        return;
    }

    let ap_index = (cpu_id - 1) as usize; // AP 1 -> index 0, AP 2 -> index 1, etc.

    // Calculate kernel stack top
    let stack_bottom = AP_KERNEL_STACKS[ap_index].as_ptr() as u64;
    let stack_top = stack_bottom + KERNEL_STACK_SIZE as u64;

    // Initialize AP CPU data
    AP_CPU_DATA[ap_index] = CpuLocal::new(cpu_id, apic_id, VirtAddr::new(stack_top));
    AP_CPU_DATA[ap_index].self_ptr = &AP_CPU_DATA[ap_index] as *const _;

    // Set GS base to point to this AP's CPU data
    set_gs_base(&AP_CPU_DATA[ap_index] as *const _ as u64);

    crate::arch::x86_64::serial::serial_write(b"[PERCPU] AP ");
    print_u32(cpu_id);
    crate::arch::x86_64::serial::serial_write(b" CPU ID: ");
    print_u32(cpu_id);
    crate::arch::x86_64::serial::serial_write(b", APIC ID: ");
    print_u32(apic_id);
    crate::arch::x86_64::serial::serial_write(b"\n[PERCPU] Kernel stack: 0x");
    print_hex(stack_top);
    crate::arch::x86_64::serial::serial_write(b"\n[PERCPU] Per-CPU data at: 0x");
    print_hex(&AP_CPU_DATA[ap_index] as *const _ as u64);
    crate::arch::x86_64::serial::serial_write(b"\n");

    // Verify GS base was set correctly
    let gs_base = get_gs_base();
    if gs_base != &AP_CPU_DATA[ap_index] as *const _ as u64 {
        crate::arch::x86_64::serial::serial_write(b"[PERCPU] WARNING: GS base mismatch!\n");
        crate::arch::x86_64::serial::serial_write(b"[PERCPU] Expected: 0x");
        print_hex(&AP_CPU_DATA[ap_index] as *const _ as u64);
        crate::arch::x86_64::serial::serial_write(b"\n[PERCPU] Got: 0x");
        print_hex(gs_base);
        crate::arch::x86_64::serial::serial_write(b"\n");
    } else {
        crate::arch::x86_64::serial::serial_write(b"[PERCPU] GS base verified OK\n");
    }
}

/// Helper to print u32 to serial
fn print_u32(n: u32) {
    let mut buf = [0u8; 10];
    let mut i = 0;
    let mut num = n;

    if num == 0 {
        crate::arch::x86_64::serial::serial_write(b"0");
        return;
    }

    while num > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    for j in 0..i/2 {
        buf.swap(j, i - 1 - j);
    }

    crate::arch::x86_64::serial::serial_write(&buf[..i]);
}

/// Helper to print hex value
fn print_hex(n: u64) {
    for i in (0..16).rev() {
        let nibble = ((n >> (i * 4)) & 0xF) as u8;
        let ch = if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + (nibble - 10)
        };
        crate::arch::x86_64::serial::serial_write(&[ch]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_local_layout() {
        // Verify self_ptr is at offset 0
        let cpu = CpuLocal::new(0, 0, VirtAddr::zero());
        let base = &cpu as *const _ as usize;
        let self_ptr_addr = &cpu.self_ptr as *const _ as usize;
        assert_eq!(self_ptr_addr, base);

        // Verify kernel_stack is at offset 0x10
        let stack_addr = &cpu.kernel_stack as *const _ as usize;
        assert_eq!(stack_addr - base, 0x10);
    }

    #[test]
    fn test_cpu_local_size() {
        // Ensure structure fits in cache line multiples
        assert!(core::mem::size_of::<CpuLocal>() <= 128);
    }
}
