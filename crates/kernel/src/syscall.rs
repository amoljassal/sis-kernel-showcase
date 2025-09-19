//! System call interface for SIS kernel
//!
//! Implements ARM64 system call handling with EL0 -> EL1 transitions.
//! Provides POSIX-compatible system calls for userspace applications.

use core::arch::asm;

// Fast syscall implementation - normally would be in vDSO
#[no_mangle]
pub extern "C" fn vdso_fast_syscall(syscall_num: i64, _arg0: u64, _arg1: u64, _arg2: u64) -> u64 {
    // Simple implementation for kernel testing
    match syscall_num {
        1 => 0x42, // Dummy successful response
        _ => 0,
    }
}

/// System call numbers (following Linux ARM64 convention)
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    Read = 63,
    Write = 64,
    Exit = 93,
    Fork = 220,
    Exec = 221,
    Open = 56,
    Close = 57,
    Mmap = 222,
    Munmap = 215,
    Brk = 214,
    GetPid = 172,
    GetPpid = 173,
    Wait4 = 260,
    /// Invalid system call number
    Invalid = u64::MAX,
}

impl From<u64> for SyscallNumber {
    fn from(num: u64) -> Self {
        match num {
            63 => SyscallNumber::Read,
            64 => SyscallNumber::Write,
            93 => SyscallNumber::Exit,
            220 => SyscallNumber::Fork,
            221 => SyscallNumber::Exec,
            56 => SyscallNumber::Open,
            57 => SyscallNumber::Close,
            222 => SyscallNumber::Mmap,
            215 => SyscallNumber::Munmap,
            214 => SyscallNumber::Brk,
            172 => SyscallNumber::GetPid,
            173 => SyscallNumber::GetPpid,
            260 => SyscallNumber::Wait4,
            _ => SyscallNumber::Invalid,
        }
    }
}

/// System call arguments passed in ARM64 registers
#[derive(Debug, Clone, Copy)]
pub struct SyscallArgs {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
}

/// System call result
pub type SyscallResult = Result<u64, SyscallError>;

/// System call errors (negative errno values)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SyscallError {
    /// Invalid argument
    EINVAL = -22,
    /// Permission denied  
    EACCES = -13,
    /// No such file or directory
    ENOENT = -2,
    /// Bad file descriptor
    EBADF = -9,
    /// Out of memory
    ENOMEM = -12,
    /// Function not implemented
    ENOSYS = -38,
    /// No such process
    ESRCH = -3,
    /// Resource temporarily unavailable
    EAGAIN = -11,
    /// No child processes
    ECHILD = -10,
}

impl From<SyscallError> for u64 {
    fn from(err: SyscallError) -> u64 {
        (err as i32) as u64
    }
}

/// Saved processor state during system call
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallFrame {
    /// General purpose registers x0-x30
    pub gpr: [u64; 31],
    /// Stack pointer (EL0)
    pub sp_el0: u64,
    /// Exception link register
    pub elr_el1: u64,
    /// Saved program status register
    pub spsr_el1: u64,
}

impl SyscallFrame {
    /// Get system call number from x8 register
    pub fn syscall_number(&self) -> SyscallNumber {
        SyscallNumber::from(self.gpr[8])
    }

    /// Get system call arguments from registers
    pub fn args(&self) -> SyscallArgs {
        SyscallArgs {
            x0: self.gpr[0],
            x1: self.gpr[1],
            x2: self.gpr[2],
            x3: self.gpr[3],
            x4: self.gpr[4],
            x5: self.gpr[5],
        }
    }

    /// Set return value in x0 register
    pub fn set_return_value(&mut self, value: u64) {
        self.gpr[0] = value;
    }
}

/// Main system call dispatcher with vDSO fast path support
pub fn handle_syscall(frame: &mut SyscallFrame) -> SyscallResult {
    // Check for vDSO fast syscalls (negative syscall numbers)
    let raw_syscall = frame.gpr[8] as i64;
    if raw_syscall < 0 {
        // Fast path: vDSO syscalls avoid full kernel transition
        let result = vdso_fast_syscall(raw_syscall, frame.gpr[0], frame.gpr[1], frame.gpr[2]);

        // Fast path succeeded
        if result != u64::MAX {
            frame.set_return_value(result);
            return Ok(result);
        }

        // Fall through to regular syscall if vDSO failed
    }
    unsafe {
        crate::uart_print(b"[SYSCALL] Raw x8 register: ");
        let raw_x8 = frame.gpr[8];
        // Print the raw x8 register value in hex
        crate::uart_print(b"0x");
        for i in (0..16).rev() {
            let nibble = (raw_x8 >> (i * 4)) & 0xF;
            let c = if nibble < 10 {
                b'0' + nibble as u8
            } else {
                b'A' + (nibble - 10) as u8
            };
            crate::uart_print(&[c]);
        }
        crate::uart_print(b"\n");
    }

    let syscall_num = frame.syscall_number();
    let args = frame.args();

    // Performance measurement start
    let start_cycles = read_cycle_counter();

    unsafe {
        crate::uart_print(b"[SYSCALL] Dispatching syscall number: ");
        // Print the actual syscall number for debugging
        let num = syscall_num as u64;
        if num == 64 {
            crate::uart_print(b"WRITE(64)");
        } else if num == 63 {
            crate::uart_print(b"READ(63)");
        } else if num == 93 {
            crate::uart_print(b"EXIT(93)");
        } else if num == 172 {
            crate::uart_print(b"GETPID(172)");
        } else if num == 220 {
            crate::uart_print(b"FORK(220)");
        } else if num == u64::MAX {
            crate::uart_print(b"INVALID(MAX)");
        } else {
            crate::uart_print(b"UNKNOWN(");
            // Print raw number in a simple way
            if num < 10 {
                crate::uart_print(&[b'0' + num as u8]);
            } else if num < 100 {
                crate::uart_print(&[b'0' + (num / 10) as u8, b'0' + (num % 10) as u8]);
            } else {
                crate::uart_print(b"XXX");
            }
            crate::uart_print(b")");
        }
        crate::uart_print(b"\n");
    }

    let result = match syscall_num {
        SyscallNumber::Read => sys_read(args.x0 as i32, args.x1 as *mut u8, args.x2),
        SyscallNumber::Write => {
            unsafe {
                crate::uart_print(b"[SYSCALL] Calling sys_write with fd=");
                crate::uart_print(if args.x0 == 1 { b"1(stdout)" } else { b"OTHER" });
                crate::uart_print(b"\n");
            }
            sys_write(args.x0 as i32, args.x1 as *const u8, args.x2)
        }
        SyscallNumber::Exit => sys_exit(args.x0 as i32),
        SyscallNumber::Fork => sys_fork(),
        SyscallNumber::Exec => sys_exec(args.x0 as *const u8, args.x1 as *const *const u8),
        SyscallNumber::Open => sys_open(args.x0 as *const u8, args.x1 as i32, args.x2 as u32),
        SyscallNumber::Close => sys_close(args.x0 as i32),
        SyscallNumber::Mmap => sys_mmap(
            args.x0,
            args.x1,
            args.x2 as i32,
            args.x3 as i32,
            args.x4 as i32,
            args.x5 as i64,
        ),
        SyscallNumber::Munmap => sys_munmap(args.x0, args.x1),
        SyscallNumber::Brk => sys_brk(args.x0),
        SyscallNumber::GetPid => sys_getpid(),
        SyscallNumber::GetPpid => sys_getppid(),
        SyscallNumber::Wait4 => sys_wait4(args.x0 as i32, args.x1 as *mut i32, args.x2 as i32),
        SyscallNumber::Invalid => Err(SyscallError::ENOSYS),
    };

    // Performance measurement end
    let end_cycles = read_cycle_counter();
    let latency_cycles = end_cycles.wrapping_sub(start_cycles);

    // Record syscall performance metrics
    record_syscall_metrics(syscall_num, latency_cycles, result.is_ok());

    result
}

/// Read system call - read from file descriptor
fn sys_read(fd: i32, buf: *mut u8, count: u64) -> SyscallResult {
    // Validate file descriptor
    if fd < 0 {
        return Err(SyscallError::EBADF);
    }

    // Validate buffer pointer and size
    if buf.is_null() || count == 0 {
        return Err(SyscallError::EINVAL);
    }

    // For now, implement basic UART read for stdin (fd 0)
    if fd == 0 {
        // NOTE: UART input buffering not implemented - using direct read
        let bytes_read = uart_read_bytes(buf, count as usize)?;
        Ok(bytes_read as u64)
    } else {
        // NOTE: SIS filesystem module integration pending
        Err(SyscallError::ENOSYS)
    }
}

/// Write system call - write to file descriptor  
fn sys_write(fd: i32, buf: *const u8, count: u64) -> SyscallResult {
    // Validate file descriptor
    if fd < 0 {
        return Err(SyscallError::EBADF);
    }

    // Validate buffer pointer and size
    if buf.is_null() || count == 0 {
        return Ok(0);
    }

    // Implement UART write for stdout/stderr (fd 1, 2)
    if fd == 1 || fd == 2 {
        unsafe {
            crate::uart_print(
                b"[SYSCALL] sys_write: fd is stdout/stderr, calling uart_write_bytes\n",
            );
        }
        let bytes_written = uart_write_bytes(buf, count as usize)?;
        unsafe {
            crate::uart_print(b"[SYSCALL] sys_write: uart_write_bytes succeeded\n");
        }
        Ok(bytes_written as u64)
    } else {
        unsafe {
            crate::uart_print(b"[SYSCALL] sys_write: fd is not stdout/stderr, returning ENOSYS\n");
        }
        // NOTE: SIS filesystem module integration pending
        Err(SyscallError::ENOSYS)
    }
}

/// Exit system call - terminate current process
fn sys_exit(_status: i32) -> SyscallResult {
    // Use existing uart_print function
    unsafe {
        crate::uart_print(b"[SYSCALL] Process exit with status: ");
        // Convert status to string and print (simplified)
        crate::uart_print(b"\n");
    }

    // For single process system, halt
    loop {}
}

/// Fork system call - create new process
fn sys_fork() -> SyscallResult {
    // NOTE: Process management integration pending - returning ENOSYS
    Err(SyscallError::ENOSYS)
}

/// Exec system call - replace current process image
fn sys_exec(_path: *const u8, _argv: *const *const u8) -> SyscallResult {
    // NOTE: ELF loader integration pending - returning ENOSYS
    Err(SyscallError::ENOSYS)
}

/// Open system call - open file
fn sys_open(_path: *const u8, _flags: i32, _mode: u32) -> SyscallResult {
    // NOTE: Filesystem integration pending - returning ENOSYS
    Err(SyscallError::ENOSYS)
}

/// Close system call - close file descriptor
fn sys_close(_fd: i32) -> SyscallResult {
    // NOTE: File descriptor table integration pending - returning ENOSYS
    Err(SyscallError::ENOSYS)
}

/// Memory map system call
fn sys_mmap(
    _addr: u64,
    _length: u64,
    _prot: i32,
    _flags: i32,
    _fd: i32,
    _offset: i64,
) -> SyscallResult {
    // NOTE: Memory management integration pending - returning ENOSYS
    Err(SyscallError::ENOSYS)
}

/// Memory unmap system call
fn sys_munmap(_addr: u64, _length: u64) -> SyscallResult {
    // NOTE: Memory management integration pending - returning ENOSYS
    Err(SyscallError::ENOSYS)
}

/// Program break system call - adjust heap size
fn sys_brk(_addr: u64) -> SyscallResult {
    // NOTE: Heap management integration pending - returning ENOSYS
    Err(SyscallError::ENOSYS)
}

/// Get process ID
fn sys_getpid() -> SyscallResult {
    // NOTE: Process manager integration pending - returning hardcoded PID
    Ok(1) // Temporary: return PID 1 for init process
}

/// Get parent process ID
fn sys_getppid() -> SyscallResult {
    // NOTE: Process manager integration pending - returning hardcoded PID
    Ok(0) // Temporary: return PID 0 for kernel
}

/// Wait for child process
fn sys_wait4(_pid: i32, _status: *mut i32, _options: i32) -> SyscallResult {
    // NOTE: Process scheduler integration pending - returning ENOSYS
    Err(SyscallError::ENOSYS)
}

/// UART read implementation
fn uart_read_bytes(_buf: *mut u8, _count: usize) -> Result<usize, SyscallError> {
    // NOTE: UART input buffering not implemented - returning zero bytes
    Ok(0)
}

/// UART write implementation using existing uart_print
fn uart_write_bytes(buf: *const u8, count: usize) -> Result<usize, SyscallError> {
    unsafe {
        let slice = core::slice::from_raw_parts(buf, count);
        crate::uart_print(slice);
    }
    Ok(count)
}

/// Read cycle counter for performance measurement
#[inline(always)]
pub fn read_cycle_counter() -> u64 {
    unsafe {
        let mut count: u64;
        #[cfg(target_arch = "aarch64")]
        asm!("mrs {}, cntvct_el0", out(reg) count);

        #[cfg(target_arch = "x86_64")]
        asm!("rdtsc", "shl rdx, 32", "or rax, rdx", out("rax") count, out("rdx") _);

        #[cfg(target_arch = "riscv64")]
        asm!("rdcycle {}", out(reg) count);

        count
    }
}

/// Performance metrics for syscall microbenchmarking
#[derive(Debug, Clone, Copy)]
struct SyscallMetrics {
    call_count: u64,
    total_cycles: u64,
    min_cycles: u64,
    max_cycles: u64,
    avg_cycles: u64,
}

impl SyscallMetrics {
    const fn new() -> Self {
        Self {
            call_count: 0,
            total_cycles: 0,
            min_cycles: u64::MAX,
            max_cycles: 0,
            avg_cycles: 0,
        }
    }

    fn update(&mut self, cycles: u64) {
        self.call_count += 1;
        self.total_cycles += cycles;
        self.min_cycles = self.min_cycles.min(cycles);
        self.max_cycles = self.max_cycles.max(cycles);
        self.avg_cycles = self.total_cycles / self.call_count;
    }
}

/// Global performance metrics storage
static mut SYSCALL_METRICS: [SyscallMetrics; 16] = [SyscallMetrics::new(); 16];

fn syscall_to_index(syscall: SyscallNumber) -> usize {
    match syscall {
        SyscallNumber::Read => 0,
        SyscallNumber::Write => 1,
        SyscallNumber::Exit => 2,
        SyscallNumber::Fork => 3,
        SyscallNumber::Exec => 4,
        SyscallNumber::Open => 5,
        SyscallNumber::Close => 6,
        SyscallNumber::Mmap => 7,
        SyscallNumber::Munmap => 8,
        SyscallNumber::Brk => 9,
        SyscallNumber::GetPid => 10,
        SyscallNumber::GetPpid => 11,
        SyscallNumber::Wait4 => 12,
        SyscallNumber::Invalid => 13,
        // Reserve slots 14-15 for future syscalls
    }
}

/// Record system call performance metrics
fn record_syscall_metrics(syscall: SyscallNumber, cycles: u64, success: bool) {
    // Ensure `success` is observed even when perf-verbose is off
    let _ = success;
    unsafe {
        let index = syscall_to_index(syscall);
        SYSCALL_METRICS[index].update(cycles);

        // Performance warnings (verbose only)
        #[cfg(feature = "perf-verbose")]
        {
            const HIGH_LATENCY_THRESHOLD: u64 = 1000; // cycles
            const VERY_HIGH_LATENCY_THRESHOLD: u64 = 5000; // cycles
            if cycles > VERY_HIGH_LATENCY_THRESHOLD {
                crate::uart_print(b"[PERF] CRITICAL latency: ");
                print_syscall_name(syscall);
                crate::uart_print(b" took ");
                print_cycles(cycles);
                crate::uart_print(b" cycles\n");
            } else if cycles > HIGH_LATENCY_THRESHOLD {
                crate::uart_print(b"[PERF] High latency: ");
                print_syscall_name(syscall);
                crate::uart_print(b" took ");
                print_cycles(cycles);
                crate::uart_print(b" cycles\n");
            }
        }

        // Success/failure tracking
        #[cfg(feature = "perf-verbose")]
        {
            if !success {
                crate::uart_print(b"[PERF] Failed syscall: ");
                print_syscall_name(syscall);
                crate::uart_print(b"\n");
            }
        }
    }
}

/// Helper function to print syscall name
fn print_syscall_name(syscall: SyscallNumber) {
    unsafe {
        match syscall {
            SyscallNumber::Read => crate::uart_print(b"read"),
            SyscallNumber::Write => crate::uart_print(b"write"),
            SyscallNumber::Exit => crate::uart_print(b"exit"),
            SyscallNumber::Fork => crate::uart_print(b"fork"),
            SyscallNumber::Exec => crate::uart_print(b"exec"),
            SyscallNumber::Open => crate::uart_print(b"open"),
            SyscallNumber::Close => crate::uart_print(b"close"),
            SyscallNumber::Mmap => crate::uart_print(b"mmap"),
            SyscallNumber::Munmap => crate::uart_print(b"munmap"),
            SyscallNumber::Brk => crate::uart_print(b"brk"),
            SyscallNumber::GetPid => crate::uart_print(b"getpid"),
            SyscallNumber::GetPpid => crate::uart_print(b"getppid"),
            SyscallNumber::Wait4 => crate::uart_print(b"wait4"),
            SyscallNumber::Invalid => crate::uart_print(b"invalid"),
        }
    }
}

/// Helper function to print cycle count
pub fn print_cycles(cycles: u64) {
    unsafe {
        if cycles < 1000 {
            // Print as decimal
            let mut buf = [0u8; 20];
            let mut idx = 19;
            let mut n = cycles;

            if n == 0 {
                crate::uart_print(b"0");
                return;
            }

            while n > 0 && idx > 0 {
                buf[idx] = b'0' + (n % 10) as u8;
                n /= 10;
                idx -= 1;
            }

            crate::uart_print(&buf[idx + 1..]);
        } else if cycles < 1000000 {
            // Print in K format
            let k_cycles = cycles / 1000;
            let mut buf = [0u8; 20];
            let mut idx = 19;
            let mut n = k_cycles;

            while n > 0 && idx > 0 {
                buf[idx] = b'0' + (n % 10) as u8;
                n /= 10;
                idx -= 1;
            }

            crate::uart_print(&buf[idx + 1..]);
            crate::uart_print(b"K");
        } else {
            // Print in M format
            let m_cycles = cycles / 1000000;
            let mut buf = [0u8; 20];
            let mut idx = 19;
            let mut n = m_cycles;

            while n > 0 && idx > 0 {
                buf[idx] = b'0' + (n % 10) as u8;
                n /= 10;
                idx -= 1;
            }

            crate::uart_print(&buf[idx + 1..]);
            crate::uart_print(b"M");
        }
    }
}

/// Display comprehensive performance metrics report
pub fn print_syscall_performance_report() {
    #[cfg(feature = "perf-verbose")]
    unsafe {
        crate::uart_print(b"\n[PERF] ========== SYSCALL PERFORMANCE REPORT ==========\n");
        crate::uart_print(b"[PERF] Syscall        | Calls |  Min  |  Max  |  Avg  |\n");
        crate::uart_print(b"[PERF] -------------- | ----- | ----- | ----- | ----- |\n");

        let syscalls = [
            (SyscallNumber::Read, "read          "),
            (SyscallNumber::Write, "write         "),
            (SyscallNumber::Exit, "exit          "),
            (SyscallNumber::Fork, "fork          "),
            (SyscallNumber::Exec, "exec          "),
            (SyscallNumber::Open, "open          "),
            (SyscallNumber::Close, "close         "),
            (SyscallNumber::Mmap, "mmap          "),
            (SyscallNumber::Munmap, "munmap        "),
            (SyscallNumber::Brk, "brk           "),
            (SyscallNumber::GetPid, "getpid        "),
            (SyscallNumber::GetPpid, "getppid       "),
            (SyscallNumber::Wait4, "wait4         "),
            (SyscallNumber::Invalid, "invalid       "),
        ];

        for (syscall, name) in &syscalls {
            let index = syscall_to_index(*syscall);
            let metrics = &SYSCALL_METRICS[index];

            if metrics.call_count > 0 {
                crate::uart_print(b"[PERF] ");
                crate::uart_print(name.as_bytes());
                crate::uart_print(b" | ");
                print_padded_number(metrics.call_count, 5);
                crate::uart_print(b" | ");
                print_padded_cycles(metrics.min_cycles, 5);
                crate::uart_print(b" | ");
                print_padded_cycles(metrics.max_cycles, 5);
                crate::uart_print(b" | ");
                print_padded_cycles(metrics.avg_cycles, 5);
                crate::uart_print(b" |\n");
            }
        }

        crate::uart_print(b"[PERF] ================================================\n\n");
    }
}

/// Helper function to print a number with padding
fn print_padded_number(num: u64, width: usize) {
    unsafe {
        let mut buf = [b' '; 20];
        let mut idx = 19;
        let mut n = num;

        if n == 0 {
            buf[19] = b'0';
            idx = 19;
        } else {
            while n > 0 && idx > 0 {
                buf[idx] = b'0' + (n % 10) as u8;
                n /= 10;
                idx -= 1;
            }
            idx += 1;
        }

        let len = 20 - idx;
        let padding = if width > len { width - len } else { 0 };

        // Print padding spaces
        for _ in 0..padding {
            crate::uart_print(b" ");
        }

        crate::uart_print(&buf[idx..]);
    }
}

/// Helper function to print cycle count with padding
#[cfg(feature = "perf-verbose")]
fn print_padded_cycles(cycles: u64, width: usize) {
    unsafe {
        if cycles == u64::MAX {
            // Handle uninitialized min values
            let padding = if width > 1 { width - 1 } else { 0 };
            for _ in 0..padding {
                crate::uart_print(b" ");
            }
            crate::uart_print(b"-");
        } else if cycles < 1000 {
            print_padded_number(cycles, width);
        } else if cycles < 1000000 {
            let k_cycles = cycles / 1000;
            print_padded_number(k_cycles, width - 1);
            crate::uart_print(b"K");
        } else {
            let m_cycles = cycles / 1000000;
            print_padded_number(m_cycles, width - 1);
            crate::uart_print(b"M");
        }
    }
}

/// Reset performance metrics (useful for focused testing)
pub fn reset_syscall_metrics() {
    unsafe {
        let metrics_ptr = &raw mut SYSCALL_METRICS;
        for metrics in (*metrics_ptr).iter_mut() {
            *metrics = SyscallMetrics::new();
        }
        crate::uart_print(b"[PERF] Performance metrics reset\n");
    }
}

/// Syscall microbenchmark runner
pub fn run_syscall_microbenchmark(syscall: SyscallNumber, iterations: u32) -> (u64, u64, u64) {
    unsafe {
        crate::uart_print(b"[PERF] Running microbenchmark for ");
        print_syscall_name(syscall);
        crate::uart_print(b" (");
        print_padded_number(iterations as u64, 0);
        crate::uart_print(b" iterations)\n");

        let mut min_cycles = u64::MAX;
        let mut max_cycles = 0;
        let mut total_cycles = 0;

        for _ in 0..iterations {
            // Create a mock syscall frame
            let mut frame = SyscallFrame {
                gpr: [0; 31],
                sp_el0: 0,
                elr_el1: 0,
                spsr_el1: 0,
            };

            // Setup basic syscall arguments
            frame.gpr[8] = syscall as u64;

            match syscall {
                SyscallNumber::Write => {
                    let test_msg = b"benchmark\n";
                    frame.gpr[0] = 1; // stdout
                    frame.gpr[1] = test_msg.as_ptr() as u64;
                    frame.gpr[2] = test_msg.len() as u64;
                }
                SyscallNumber::GetPid => {
                    // No additional args needed
                }
                _ => {
                    // Default args for other syscalls
                    frame.gpr[0] = 0;
                    frame.gpr[1] = 0;
                    frame.gpr[2] = 0;
                }
            }

            let start_cycles = read_cycle_counter();
            let _ = handle_syscall(&mut frame);
            let end_cycles = read_cycle_counter();

            let cycles = end_cycles.wrapping_sub(start_cycles);
            min_cycles = min_cycles.min(cycles);
            max_cycles = max_cycles.max(cycles);
            total_cycles += cycles;
        }

        let avg_cycles = total_cycles / iterations as u64;

        crate::uart_print(b"[PERF] Benchmark results: min=");
        print_cycles(min_cycles);
        crate::uart_print(b", max=");
        print_cycles(max_cycles);
        crate::uart_print(b", avg=");
        print_cycles(avg_cycles);
        crate::uart_print(b" cycles\n");

        (min_cycles, max_cycles, avg_cycles)
    }
}

/// System call exception handler (called from assembly)
///
/// # Safety
/// This function dereferences a raw pointer to SyscallFrame.
/// The caller must ensure the pointer is valid and properly aligned.
#[no_mangle]
pub unsafe extern "C" fn syscall_handler(frame: *mut SyscallFrame) {
    crate::uart_print(b"[SYSCALL] Handler called\n");
    let frame_ref = &mut *frame;
    crate::uart_print(b"[SYSCALL] About to handle syscall\n");
    match handle_syscall(frame_ref) {
        Ok(result) => {
            crate::uart_print(b"[SYSCALL] Success, setting return value\n");
            frame_ref.set_return_value(result);
        }
        Err(error) => {
            crate::uart_print(b"[SYSCALL] Error, setting error value\n");
            frame_ref.set_return_value(error.into());
        }
    }
    crate::uart_print(b"[SYSCALL] Handler returning\n");
}
