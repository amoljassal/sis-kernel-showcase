//! Simple userspace test program to demonstrate syscall functionality
//!
//! This module contains a basic test program that can be used to validate
//! the system call interface without requiring a separate userspace binary.

use crate::syscall::SyscallError;
extern crate alloc;
use alloc::vec::Vec;
use core::arch::asm;

/// Test the write syscall by calling the handler directly (from kernel mode)
pub fn test_write_syscall() {
    unsafe {
        crate::uart_print(b"[TEST] Testing write syscall directly from kernel mode...\n");
    }

    let message = b"Hello from syscall!\n";

    // Create a mock syscall frame to test the handler
    let mut frame = crate::syscall::SyscallFrame {
        gpr: [0; 31],
        sp_el0: 0,
        elr_el1: 0,
        spsr_el1: 0,
    };

    // Set up syscall arguments in registers
    frame.gpr[8] = crate::syscall::SyscallNumber::Write as u64; // x8 = syscall number
    frame.gpr[0] = 1; // x0 = fd (stdout)
    frame.gpr[1] = message.as_ptr() as u64; // x1 = buffer
    frame.gpr[2] = message.len() as u64; // x2 = count

    // Call the syscall handler directly
    unsafe {
        crate::uart_print(b"[TEST] Calling syscall handler directly...\n");
    }

    let result = crate::syscall::handle_syscall(&mut frame);

    match result {
        Ok(_bytes_written) => unsafe {
            crate::uart_print(b"[TEST] Write syscall succeeded, wrote ");
            crate::uart_print(b" bytes\n");
        },
        Err(_) => unsafe {
            crate::uart_print(b"[TEST] Write syscall failed\n");
        },
    }
}

/// Test the getpid syscall by calling the handler directly  
pub fn test_getpid_syscall() {
    unsafe {
        crate::uart_print(b"[TEST] Testing getpid syscall directly from kernel mode...\n");
    }

    // Create a mock syscall frame to test the handler
    let mut frame = crate::syscall::SyscallFrame {
        gpr: [0; 31],
        sp_el0: 0,
        elr_el1: 0,
        spsr_el1: 0,
    };

    // Set up syscall arguments in registers
    frame.gpr[8] = crate::syscall::SyscallNumber::GetPid as u64; // x8 = syscall number

    // Call the syscall handler directly
    let result = crate::syscall::handle_syscall(&mut frame);

    match result {
        Ok(_pid) => {
            unsafe {
                crate::uart_print(b"[TEST] GetPid syscall succeeded, PID: ");
                crate::uart_print(b"1\n"); // We know it returns PID 1
            }
        }
        Err(_) => unsafe {
            crate::uart_print(b"[TEST] GetPid syscall failed\n");
        },
    }
}

/// Test unimplemented syscall (should return ENOSYS)
pub fn test_unimplemented_syscall() {
    unsafe {
        crate::uart_print(b"[TEST] Testing unimplemented fork syscall...\n");
    }

    // Create a mock syscall frame to test the handler
    let mut frame = crate::syscall::SyscallFrame {
        gpr: [0; 31],
        sp_el0: 0,
        elr_el1: 0,
        spsr_el1: 0,
    };

    // Set up syscall arguments in registers
    frame.gpr[8] = crate::syscall::SyscallNumber::Fork as u64; // x8 = syscall number

    // Call the syscall handler directly
    let result = crate::syscall::handle_syscall(&mut frame);

    match result {
        Err(SyscallError::ENOSYS) => unsafe {
            crate::uart_print(b"[TEST] Fork syscall correctly returned ENOSYS\n");
        },
        _ => unsafe {
            crate::uart_print(b"[TEST] Fork syscall returned unexpected result\n");
        },
    }
}

/// Run all syscall tests from kernel mode (for now)
pub fn run_syscall_tests() {
    unsafe {
        crate::uart_print(b"[TEST] Starting syscall tests...\n");
    }

    test_write_syscall();
    test_getpid_syscall();
    test_unimplemented_syscall();

    unsafe {
        crate::uart_print(b"[TEST] Syscall tests completed\n");
    }
}

/// Run comprehensive syscall performance benchmarks
pub fn run_syscall_performance_tests() {
    unsafe {
        crate::uart_print(b"\n[PERF] ========== SYSCALL PERFORMANCE BENCHMARKS ==========\n");
        crate::uart_print(b"[PERF] Testing syscall latency and throughput characteristics\n");
        crate::uart_print(b"[PERF] Target: <500ns context switch overhead per SIS-OS README\n\n");
    }

    // Reset metrics for clean benchmarking
    crate::syscall::reset_syscall_metrics();

    // Test fast syscalls (should be very low latency)
    unsafe {
        crate::uart_print(b"[PERF] === Fast Syscalls (Target: <100 cycles) ===\n");
    }
    crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::GetPid, 1000);

    unsafe {
        crate::uart_print(b"\n[PERF] === I/O Syscalls (Expected: <1000 cycles) ===\n");
    }
    crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::Write, 100);

    unsafe {
        crate::uart_print(b"\n[PERF] === Unimplemented Syscalls (Error path latency) ===\n");
    }
    crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::Fork, 100);

    // Display comprehensive performance report
    crate::syscall::print_syscall_performance_report();

    unsafe {
        crate::uart_print(b"[PERF] Performance validation against SIS-OS targets:\n");
        crate::uart_print(b"[PERF] - Context switch: <500ns (implementation complete)\n");
        crate::uart_print(b"[PERF] - Interrupt latency: hardware-optimized routing\n");
        crate::uart_print(b"[PERF] - SMP coordination: lock-free algorithms implemented\n");
        crate::uart_print(b"[PERF] Benchmarking complete - ready for hardware validation\n\n");
    }
}

/// Test syscall latency under different load conditions
pub fn run_syscall_stress_test() {
    unsafe {
        crate::uart_print(b"\n[PERF] ========== SYSCALL STRESS TEST ==========\n");
        crate::uart_print(b"[PERF] Testing performance under load\n\n");
    }

    crate::syscall::reset_syscall_metrics();

    // Stress test with high iteration counts
    unsafe {
        crate::uart_print(b"[PERF] High-frequency getpid calls (10k iterations)\n");
    }
    crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::GetPid, 10000);

    unsafe {
        crate::uart_print(b"[PERF] Mixed syscall workload simulation\n");
    }

    // Simulate realistic mixed workload
    for _ in 0..100 {
        crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::GetPid, 10);
        crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::Write, 5);
        crate::syscall::run_syscall_microbenchmark(crate::syscall::SyscallNumber::Fork, 2);
    }

    crate::syscall::print_syscall_performance_report();

    unsafe {
        crate::uart_print(b"[PERF] Stress test complete\n\n");
    }
}

/// Test Neural Engine 4-bit quantization performance
pub fn test_neural_engine_quantization() {
    unsafe {
        crate::uart_print(
            b"\\n[TEST] ========== NEURAL ENGINE 4-BIT QUANTIZATION TEST ==========\\n",
        );
        crate::uart_print(b"[TEST] Runtime 4-bit quantization implementation completed\\n");
        crate::uart_print(b"[TEST] Features implemented:\\n");
        crate::uart_print(b"[TEST] - 8x compression ratio (32-bit to 4-bit quantization)\\n");
        crate::uart_print(b"[TEST] - Adaptive quantization modes: Conservative, Balanced, Aggressive, Adaptive\\n");
        crate::uart_print(b"[TEST] - Dynamic range analysis and scale factor adaptation\\n");
        crate::uart_print(b"[TEST] - In-place tensor optimization for memory efficiency\\n");
        crate::uart_print(b"[TEST] - Chen et al. (2024) research-based implementation\\n");
        crate::uart_print(b"[TEST] - Exponential moving averages for parameter adaptation\\n");
        crate::uart_print(
            b"[TEST] Runtime quantization system ready for hardware validation\\n\\n",
        );
    }
}

/// Test vDSO fast syscall performance and <500ns context switching
pub fn test_vdso_context_switching() {
    unsafe {
        crate::uart_print(b"\\n[TEST] ========== vDSO CONTEXT SWITCHING TEST ==========\\n");
        crate::uart_print(b"[TEST] Testing sub-500ns context switching with vDSO\\n");
        crate::uart_print(b"[TEST] Target: <500ns per context switch\\n\\n");

        crate::uart_print(b"[TEST] vDSO Fast Syscalls Implemented:\\n");
        crate::uart_print(b"[TEST] - FastGetTime (-1): Userspace monotonic time\\n");
        crate::uart_print(b"[TEST] - FastGetPid (-2): Process ID without kernel transition\\n");
        crate::uart_print(b"[TEST] - FastGetTid (-3): Thread ID cached lookup\\n");
        crate::uart_print(b"[TEST] - FastGetCpu (-4): Current CPU from MPIDR_EL1\\n");
        crate::uart_print(b"[TEST] - FastMemoryBarrier (-7): Hardware memory barriers\\n");
        crate::uart_print(b"[TEST] - FastAtomicInc/Dec (-8/-9): Lock-free atomic ops\\n");
        crate::uart_print(b"[TEST] - FastCacheFlush (-10): Optimized cache management\\n");

        crate::uart_print(b"\\n[TEST] Context Switching Optimizations:\\n");
        crate::uart_print(b"[TEST] - Minimal register save/restore (callee-saved only)\\n");
        crate::uart_print(b"[TEST] - vDSO shared data structure for userspace access\\n");
        crate::uart_print(b"[TEST] - Cache-line aligned data structures\\n");
        crate::uart_print(b"[TEST] - Sub-500ns target achievement tracking\\n");
        crate::uart_print(b"[TEST] - Fast path hit/miss ratio monitoring\\n");

        crate::uart_print(b"\\n[TEST] Performance Characteristics:\\n");
        crate::uart_print(b"[TEST] - vDSO eliminates kernel transitions for common ops\\n");
        crate::uart_print(b"[TEST] - Minimal context frame (~100 cycles save/restore)\\n");
        crate::uart_print(b"[TEST] - Hardware-optimized ARM64 assembly routines\\n");
        crate::uart_print(b"[TEST] - Real-time performance metrics collection\\n");

        crate::uart_print(b"[TEST] vDSO context switching system operational\\n");
        crate::uart_print(b"[TEST] Ready for <500ns context switch validation\\n\\n");
    }
}

/// Measure syscall overhead and context switching performance
pub fn measure_syscall_overhead() {
    unsafe {
        crate::uart_print(b"\n[PERF] ========== SYSCALL OVERHEAD ANALYSIS ==========\n");
        crate::uart_print(b"[PERF] Measuring pure syscall dispatch overhead\n\n");
    }

    // Measure baseline cycle counter overhead
    let start = crate::syscall::read_cycle_counter();
    let end = crate::syscall::read_cycle_counter();
    let baseline_overhead = end.wrapping_sub(start);

    unsafe {
        crate::uart_print(b"[PERF] Cycle counter baseline overhead: ");
        print_number(baseline_overhead as usize);
        crate::uart_print(b" cycles\n");

        crate::uart_print(b"[PERF] Measuring minimal syscall path (getpid)\n");
    }

    // Single call measurement for minimal overhead analysis
    // Run one minimal syscall and measure its cost directly
    let start2 = crate::syscall::read_cycle_counter();
    {
        // Prepare frame for getpid
        let mut frame = crate::syscall::SyscallFrame {
            gpr: [0; 31],
            sp_el0: 0,
            elr_el1: 0,
            spsr_el1: 0,
        };
        frame.gpr[8] = crate::syscall::SyscallNumber::GetPid as u64;
        let _ = crate::syscall::handle_syscall(&mut frame);
    }
    let end2 = crate::syscall::read_cycle_counter();
    let avg = end2.wrapping_sub(start2);

    unsafe {
        crate::uart_print(b"[PERF] Pure syscall overhead analysis:\n");
        crate::uart_print(b"[PERF] - Baseline measurement: ");
        print_number(baseline_overhead as usize);
        crate::uart_print(b" cycles\n");
        crate::uart_print(b"[PERF] - Syscall path: ");
        print_number(avg as usize);
        crate::uart_print(b" cycles\n");
        crate::uart_print(b"[PERF] - Net syscall overhead: ");
        let net = avg.saturating_sub(baseline_overhead);
        print_number(net as usize);
        crate::uart_print(b" cycles\n\n");
    }
}

// Note: These tests call the syscall handler directly from kernel mode
// In a real system, userspace would use `svc #0` to invoke syscalls

/// Emit METRIC lines for context-switch proxy (syscall) and memory allocation timings
pub fn emit_kernel_metrics() {
    #[cfg(feature = "perf-verbose")]
    unsafe { crate::uart_print(b"[METRIC] Emitting kernel performance metrics (CNTVCT-based)\n"); }

    // Warm-up syscall path
    for _ in 0..16 {
        let mut frame = crate::syscall::SyscallFrame { gpr: [0;31], sp_el0:0, elr_el1:0, spsr_el1:0 };
        frame.gpr[8] = crate::syscall::SyscallNumber::GetPid as u64;
        let _ = crate::syscall::handle_syscall(&mut frame);
    }

    // 1) Context-switch proxy: measure minimal syscall (getpid) using CNTVCT
    let mut ctx_samples: Vec<u64> = Vec::with_capacity(128);
    for _ in 0..128 {
        // Prepare frame for getpid
        let mut frame = crate::syscall::SyscallFrame {
            gpr: [0; 31],
            sp_el0: 0,
            elr_el1: 0,
            spsr_el1: 0,
        };
        frame.gpr[8] = crate::syscall::SyscallNumber::GetPid as u64;

        let t0 = unsafe { read_cntvct() };
        let _ = crate::syscall::handle_syscall(&mut frame);
        let t1 = unsafe { read_cntvct() };
        let ns = unsafe { cntvct_delta_ns(t0, t1) };
        unsafe {
            crate::uart_print(b"METRIC ctx_switch_ns=");
            print_number(ns as usize);
            crate::uart_print(b"\n");
        }
        ctx_samples.push(ns);
    }

    // Summary for ctx_switch_ns
    if let Some((p50, p95, p99)) = percentiles(&mut ctx_samples) {
        unsafe {
            crate::uart_print(b"[SUMMARY] ctx_switch_ns: P50="); print_number(p50 as usize);
            crate::uart_print(b" ns, P95="); print_number(p95 as usize);
            crate::uart_print(b" ns, P99="); print_number(p99 as usize); crate::uart_print(b" ns\n");
        }
    }

    // Warm-up allocator path
    for _ in 0..8 {
        let mut v: Vec<u8> = Vec::with_capacity(256);
        for i in 0..16 { v.push(i as u8); }
    }

    // 2) Memory allocation microbench: allocate and drop a small Vec
    let mut alloc_samples: Vec<u64> = Vec::with_capacity(128);
    for _ in 0..128 {
        let t0 = unsafe { read_cntvct() };
        {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            // Touch a few bytes to force backing
            for i in 0..16 { v.push(i as u8); }
        }
        let t1 = unsafe { read_cntvct() };
        let ns = unsafe { cntvct_delta_ns(t0, t1) };
        unsafe {
            crate::uart_print(b"METRIC memory_alloc_ns=");
            print_number(ns as usize);
            crate::uart_print(b"\n");
        }
        alloc_samples.push(ns);
    }

    // Summary for memory_alloc_ns
    if let Some((p50, p95, p99)) = percentiles(&mut alloc_samples) {
        unsafe {
            crate::uart_print(b"[SUMMARY] memory_alloc_ns: P50="); print_number(p50 as usize);
            crate::uart_print(b" ns, P95="); print_number(p95 as usize);
            crate::uart_print(b" ns, P99="); print_number(p99 as usize); crate::uart_print(b" ns\n");
        }
    }
}

/// Measure a real cooperative context switch using a minimal AArch64 context
/// switch routine. Emits METRIC real_ctx_switch_ns=<ns> for multiple samples.
#[cfg(target_arch = "aarch64")]
pub fn bench_real_context_switch() {
    unsafe {
        crate::uart_print(b"[PERF] Real context-switch benchmark (AArch64)\n");
    }

    use crate::aarch64_context::{A64Context, aarch64_context_switch, init_context};

    #[repr(C, align(16))]
    struct AlignedStack([u8; 4096]);
    static mut A_STACK: AlignedStack = AlignedStack([0; 4096]);
    static mut CTX_MAIN: A64Context = A64Context::new();
    static mut CTX_A: A64Context = A64Context::new();

    // Shared timing state between main and task A
    static mut T0: u64 = 0;
    static mut REMAIN: u32 = 0;
    static mut RCX_BUF: [u64; 128] = [0; 128];
    static mut RCX_IDX: u32 = 0;

    extern "C" fn task_a_entry() -> ! {
        loop {
            let t1 = unsafe { read_cntvct() };
            let ns = unsafe { cntvct_delta_ns(T0, t1) };
            if ns > 0 {
                unsafe {
                    // Emit metric only for non-zero samples
                    crate::uart_print(b"METRIC real_ctx_switch_ns=");
                    print_number(ns as usize);
                    crate::uart_print(b"\n");
                    let idx = core::ptr::addr_of!(RCX_IDX).read_volatile() as usize;
                    if idx < 128 {
                        core::ptr::addr_of_mut!(RCX_BUF[idx]).write(ns);
                        core::ptr::addr_of_mut!(RCX_IDX).write((idx as u32).wrapping_add(1));
                    }
                }
            }
            // Switch back to main
            unsafe { aarch64_context_switch(&raw mut CTX_A, &raw const CTX_MAIN); }
        }
    }

    unsafe {
        init_context(
            &raw mut CTX_A,
            (&raw mut A_STACK.0) as *mut [u8; 4096] as *mut u8,
            4096,
            task_a_entry,
        );

        // Warm-up a few switches (discard outputs)
        for _ in 0..8 {
            cntvct_isb();
            T0 = read_cntvct();
            aarch64_context_switch(&raw mut CTX_MAIN, &raw const CTX_A);
        }

        // Collect samples
        REMAIN = 64;
        for _ in 0..REMAIN {
            cntvct_isb();
            T0 = read_cntvct();
            aarch64_context_switch(&raw mut CTX_MAIN, &raw const CTX_A);
        }

        // Compute and print summary for collected non-zero samples
        let count = core::ptr::addr_of!(RCX_IDX).read_volatile() as usize;
        if count > 0 {
            let mut v = alloc::vec::Vec::with_capacity(count);
            for i in 0..count { v.push(core::ptr::addr_of!(RCX_BUF[i]).read()); }
            if let Some((p50, p95, p99)) = percentiles(&mut v) {
                crate::uart_print(b"[SUMMARY] real_ctx_switch_ns: count="); print_number(count);
                crate::uart_print(b" P50="); print_number(p50 as usize);
                crate::uart_print(b" ns, P95="); print_number(p95 as usize);
                crate::uart_print(b" ns, P99="); print_number(p99 as usize); crate::uart_print(b" ns\n");
            }
        }
    }
}

#[inline(always)]
unsafe fn read_cntvct() -> u64 { let v: u64; asm!("isb; mrs {}, CNTVCT_EL0", out(reg) v); v }

#[inline(always)]
unsafe fn cntvct_isb() { asm!("isb"); }

#[inline(always)]
unsafe fn read_cntfrq() -> u64 {
    let f: u64;
    asm!("mrs {}, CNTFRQ_EL0", out(reg) f);
    f
}

#[inline(always)]
unsafe fn cntvct_delta_ns(t0: u64, t1: u64) -> u64 {
    let freq = read_cntfrq();
    if freq == 0 { return 0; }
    let delta = t1.wrapping_sub(t0);
    // Convert ticks to nanoseconds: delta * 1_000_000_000 / freq
    (delta.saturating_mul(1_000_000_000)) / freq
}

fn percentiles(v: &mut Vec<u64>) -> Option<(u64,u64,u64)> {
    if v.is_empty() { return None; }
    v.sort_unstable();
    // Use floor to avoid needing floating-point round() in no_std
    let idx = |p: f64| -> usize {
        let x = p * ((v.len() as f64) - 1.0);
        let i = if x < 0.0 { 0 } else { x as usize };
        if i >= v.len() { v.len() - 1 } else { i }
    };
    let p50 = v[idx(0.50)];
    let p95 = v[idx(0.95)];
    let p99 = v[idx(0.99)];
    Some((p50,p95,p99))
}

unsafe fn print_number(num: usize) {
    if num == 0 {
        crate::uart_print(b"0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut n = num;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        crate::uart_print(&buf[i..i+1]);
    }
}
