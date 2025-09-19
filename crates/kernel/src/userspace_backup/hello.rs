//! Simple userspace test program
//!
//! Demonstrates basic syscall functionality including write and exit.

#![no_std]
#![no_main]

use core::arch::asm;

// System call numbers from our kernel implementation
const SYS_WRITE: u64 = 64;
const SYS_EXIT: u64 = 93;

/// Entry point for userspace program
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Write "Hello from userspace!" to stdout (fd 1)
    let message = b"Hello from userspace!\n";
    sys_write(1, message.as_ptr(), message.len());
    
    // Write system info message
    let info = b"SIS Kernel userspace is working!\n";
    sys_write(1, info.as_ptr(), info.len());
    
    // Exit with status 0
    sys_exit(0);
}

/// Write system call wrapper
fn sys_write(fd: i32, buf: *const u8, count: usize) -> i64 {
    let result: i64;
    unsafe {
        asm!(
            "mov x8, {syscall_num}",    // System call number
            "mov x0, {fd}",              // File descriptor
            "mov x1, {buf}",             // Buffer pointer
            "mov x2, {count}",           // Count
            "svc #0",                    // Supervisor call
            "mov {result}, x0",          // Get return value
            syscall_num = in(reg) SYS_WRITE,
            fd = in(reg) fd as u64,
            buf = in(reg) buf as u64,
            count = in(reg) count as u64,
            result = out(reg) result,
            out("x8") _,
            out("x0") _,
            out("x1") _,
            out("x2") _,
            options(nostack)
        );
    }
    result
}

/// Exit system call wrapper
fn sys_exit(status: i32) -> ! {
    unsafe {
        asm!(
            "mov x8, {syscall_num}",    // System call number
            "mov x0, {status}",         // Exit status
            "svc #0",                   // Supervisor call
            syscall_num = in(reg) SYS_EXIT,
            status = in(reg) status as u64,
            options(noreturn, nostack)
        )
    }
}

// Note: No panic handler needed when compiled as part of kernel