// Enhanced Panic Handler
// Phase 3.2 - Production Readiness Plan
//
// Provides detailed panic information for debugging and forensics

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::fmt::Write;
use alloc::string::String;
use alloc::format;

/// Global panic state - prevents recursive panics
static PANICKING: AtomicBool = AtomicBool::new(false);

/// Panic counter for tracking multiple panics
static PANIC_COUNT: AtomicU64 = AtomicU64::new(0);

/// Maximum number of recent log entries to display
const MAX_RECENT_LOGS: usize = 20;

/// Enhanced panic handler with comprehensive diagnostics
pub fn panic_handler(info: &PanicInfo) -> ! {
    // Check for recursive panic
    if PANICKING.swap(true, Ordering::SeqCst) {
        // Recursive panic - minimal output and halt
        unsafe {
            crate::uart_print(b"\n!!! RECURSIVE PANIC !!!\n");
        }
        halt();
    }

    // Increment panic counter
    let panic_num = PANIC_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

    // Disable interrupts to prevent further issues
    disable_interrupts();

    // Print panic header
    print_panic_header(panic_num);

    // Print panic location and message
    print_panic_info(info);

    // Print register dump (architecture-specific)
    print_registers();

    // Print system state
    print_system_state();

    // Print recent logs
    print_recent_logs();

    // Print call stack (if available)
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    print_stack_trace();

    // Print next steps
    print_next_steps();

    // Write crash dump if feature enabled
    #[cfg(feature = "crash-dump")]
    write_crash_dump(info);

    // Log panic in structured format
    log_panic_structured(info);

    // Halt the system
    halt();
}

fn print_panic_header(panic_num: u64) {
    unsafe {
        crate::uart_print(b"\n");
        crate::uart_print(b"================================================================================\n");
        crate::uart_print(b"!!!                        KERNEL PANIC                                      !!!\n");
        crate::uart_print(b"================================================================================\n");

        if panic_num > 1 {
            let msg = alloc::format!("Panic #{}\n", panic_num);
            crate::uart_print(msg.as_bytes());
        }
        crate::uart_print(b"\n");
    }
}

fn print_panic_info(info: &PanicInfo) {
    unsafe {
        crate::uart_print(b"PANIC INFORMATION:\n");
        crate::uart_print(b"------------------\n");

        // Location
        if let Some(location) = info.location() {
            let loc = alloc::format!("  Location: {}:{}:{}\n",
                location.file(), location.line(), location.column());
            crate::uart_print(loc.as_bytes());
        } else {
            crate::uart_print(b"  Location: <unknown>\n");
        }

        // Message (best-effort from payload)
        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            let msg = alloc::format!("  Message:  {}\n", payload);
            crate::uart_print(msg.as_bytes());
        } else if let Some(payload) = info.payload().downcast_ref::<alloc::string::String>() {
            let msg = alloc::format!("  Message:  {}\n", payload);
            crate::uart_print(msg.as_bytes());
        } else {
            crate::uart_print(b"  Message:  <no message>\n");
        }

        crate::uart_print(b"\n");
    }
}

fn print_registers() {
    unsafe {
        crate::uart_print(b"REGISTER DUMP:\n");
        crate::uart_print(b"--------------\n");

        #[cfg(target_arch = "aarch64")]
        print_aarch64_registers();

        #[cfg(target_arch = "x86_64")]
        print_x86_64_registers();

        #[cfg(target_arch = "riscv64")]
        print_riscv64_registers();

        crate::uart_print(b"\n");
    }
}

#[cfg(target_arch = "aarch64")]
fn print_aarch64_registers() {
    use core::arch::asm;

    unsafe {
        // Read general-purpose registers
        let mut x0: u64; let mut x1: u64; let mut x2: u64; let mut x3: u64;
        let mut x4: u64; let mut x5: u64; let mut x6: u64; let mut x7: u64;
        let mut x8: u64; let mut x9: u64; let mut x10: u64; let mut x11: u64;
        let mut x12: u64; let mut x13: u64; let mut x14: u64; let mut x15: u64;
        let mut x16: u64; let mut x17: u64; let mut x18: u64; let mut x19: u64;
        let mut x20: u64; let mut x21: u64; let mut x22: u64; let mut x23: u64;
        let mut x24: u64; let mut x25: u64; let mut x26: u64; let mut x27: u64;
        let mut x28: u64; let mut x29: u64; let mut x30: u64;
        let mut sp: u64; let mut pc: u64;

        asm!("mov {}, x0", out(reg) x0);
        asm!("mov {}, x1", out(reg) x1);
        asm!("mov {}, x2", out(reg) x2);
        asm!("mov {}, x3", out(reg) x3);
        asm!("mov {}, x4", out(reg) x4);
        asm!("mov {}, x5", out(reg) x5);
        asm!("mov {}, x6", out(reg) x6);
        asm!("mov {}, x7", out(reg) x7);
        asm!("mov {}, x8", out(reg) x8);
        asm!("mov {}, x9", out(reg) x9);
        asm!("mov {}, x10", out(reg) x10);
        asm!("mov {}, x11", out(reg) x11);
        asm!("mov {}, x12", out(reg) x12);
        asm!("mov {}, x13", out(reg) x13);
        asm!("mov {}, x14", out(reg) x14);
        asm!("mov {}, x15", out(reg) x15);
        asm!("mov {}, x16", out(reg) x16);
        asm!("mov {}, x17", out(reg) x17);
        asm!("mov {}, x18", out(reg) x18);
        asm!("mov {}, x19", out(reg) x19);
        asm!("mov {}, x20", out(reg) x20);
        asm!("mov {}, x21", out(reg) x21);
        asm!("mov {}, x22", out(reg) x22);
        asm!("mov {}, x23", out(reg) x23);
        asm!("mov {}, x24", out(reg) x24);
        asm!("mov {}, x25", out(reg) x25);
        asm!("mov {}, x26", out(reg) x26);
        asm!("mov {}, x27", out(reg) x27);
        asm!("mov {}, x28", out(reg) x28);
        asm!("mov {}, x29", out(reg) x29);  // Frame pointer
        asm!("mov {}, x30", out(reg) x30);  // Link register
        asm!("mov {}, sp", out(reg) sp);

        // PC is harder to get, use a label
        asm!("adr {}, .", out(reg) pc);

        // Print in columns
        let regs = alloc::format!(
            "  x0:  {:016x}  x1:  {:016x}  x2:  {:016x}  x3:  {:016x}\n\
             x4:  {:016x}  x5:  {:016x}  x6:  {:016x}  x7:  {:016x}\n\
             x8:  {:016x}  x9:  {:016x}  x10: {:016x}  x11: {:016x}\n\
             x12: {:016x}  x13: {:016x}  x14: {:016x}  x15: {:016x}\n\
             x16: {:016x}  x17: {:016x}  x18: {:016x}  x19: {:016x}\n\
             x20: {:016x}  x21: {:016x}  x22: {:016x}  x23: {:016x}\n\
             x24: {:016x}  x25: {:016x}  x26: {:016x}  x27: {:016x}\n\
             x28: {:016x}  x29: {:016x}  x30: {:016x}\n\
             sp:  {:016x}  pc:  {:016x}\n",
            x0, x1, x2, x3, x4, x5, x6, x7,
            x8, x9, x10, x11, x12, x13, x14, x15,
            x16, x17, x18, x19, x20, x21, x22, x23,
            x24, x25, x26, x27, x28, x29, x30,
            sp, pc
        );
        crate::uart_print(regs.as_bytes());
    }
}

#[cfg(target_arch = "x86_64")]
fn print_x86_64_registers() {
    use core::arch::asm;

    unsafe {
        let mut rax: u64; let mut rbx: u64; let mut rcx: u64; let mut rdx: u64;
        let mut rsi: u64; let mut rdi: u64; let mut rbp: u64; let mut rsp: u64;
        let mut r8: u64; let mut r9: u64; let mut r10: u64; let mut r11: u64;
        let mut r12: u64; let mut r13: u64; let mut r14: u64; let mut r15: u64;
        let mut rip: u64;

        asm!("mov {}, rax", out(reg) rax);
        asm!("mov {}, rbx", out(reg) rbx);
        asm!("mov {}, rcx", out(reg) rcx);
        asm!("mov {}, rdx", out(reg) rdx);
        asm!("mov {}, rsi", out(reg) rsi);
        asm!("mov {}, rdi", out(reg) rdi);
        asm!("mov {}, rbp", out(reg) rbp);
        asm!("mov {}, rsp", out(reg) rsp);
        asm!("mov {}, r8", out(reg) r8);
        asm!("mov {}, r9", out(reg) r9);
        asm!("mov {}, r10", out(reg) r10);
        asm!("mov {}, r11", out(reg) r11);
        asm!("mov {}, r12", out(reg) r12);
        asm!("mov {}, r13", out(reg) r13);
        asm!("mov {}, r14", out(reg) r14);
        asm!("mov {}, r15", out(reg) r15);
        asm!("lea {}, [rip]", out(reg) rip);

        let regs = alloc::format!(
            "  rax: {:016x}  rbx: {:016x}  rcx: {:016x}  rdx: {:016x}\n\
             rsi: {:016x}  rdi: {:016x}  rbp: {:016x}  rsp: {:016x}\n\
             r8:  {:016x}  r9:  {:016x}  r10: {:016x}  r11: {:016x}\n\
             r12: {:016x}  r13: {:016x}  r14: {:016x}  r15: {:016x}\n\
             rip: {:016x}\n",
            rax, rbx, rcx, rdx, rsi, rdi, rbp, rsp,
            r8, r9, r10, r11, r12, r13, r14, r15, rip
        );
        crate::uart_print(regs.as_bytes());
    }
}

#[cfg(target_arch = "riscv64")]
fn print_riscv64_registers() {
    unsafe {
        crate::uart_print(b"  [RISC-V register dump not yet implemented]\n");
        // TODO: Implement RISC-V register dump
    }
}

fn print_system_state() {
    unsafe {
        crate::uart_print(b"SYSTEM STATE:\n");
        crate::uart_print(b"-------------\n");

        // Uptime
        let uptime_ms = crate::time::get_uptime_ms();
        let uptime_sec = uptime_ms / 1000;
        let msg = alloc::format!("  Uptime:       {} seconds ({} ms)\n", uptime_sec, uptime_ms);
        crate::uart_print(msg.as_bytes());

        // Heap statistics
        {
            let stats = crate::heap::get_heap_stats();
            let current_mb = stats.current_allocated() / (1024 * 1024);
            let peak_mb = stats.peak_allocated() / (1024 * 1024);
            let allocs = stats.total_allocations();
            let deallocs = stats.total_deallocations();
            let failures = stats.allocation_failures();
            let msg = alloc::format!(
                "  Heap usage:   {} MB current, {} MB peak\n\
                 Allocations: {} allocs, {} deallocs, {} active\n\
                 Failures:    {}\n",
                current_mb, peak_mb, allocs, deallocs,
                allocs.saturating_sub(deallocs), failures
            );
            crate::uart_print(msg.as_bytes());
        }

        // Build information
        let build_info = crate::build_info::get_version_string();
        let msg = alloc::format!("  Version:      {}\n", build_info);
        crate::uart_print(msg.as_bytes());

        crate::uart_print(b"\n");
    }
}

fn print_recent_logs() {
    unsafe {
        crate::uart_print(b"RECENT LOGS:\n");
        crate::uart_print(b"------------\n");
        crate::uart_print(b"  [Log buffer not yet implemented]\n");
        // TODO: Implement circular log buffer to capture recent log entries
        crate::uart_print(b"\n");
    }
}

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
fn print_stack_trace() {
    unsafe {
        crate::uart_print(b"STACK TRACE:\n");
        crate::uart_print(b"------------\n");

        #[cfg(target_arch = "aarch64")]
        {
            use core::arch::asm;
            let mut fp: u64;
            asm!("mov {}, x29", out(reg) fp);  // Frame pointer

            crate::uart_print(b"  [Stack unwinding requires frame pointers]\n");
            crate::uart_print(b"  [Build with RUSTFLAGS=\"-C force-frame-pointers=yes\"]\n");

            // Basic frame pointer walk (may not work without proper setup)
            for i in 0..10 {
                if fp == 0 || fp < 0x40000000 {
                    break;
                }

                // Check if pointer looks valid (very basic check)
                if fp > 0x1000_0000_0000 {
                    break;
                }

                // Try to read frame
                let frame_ptr = fp as *const u64;
                let lr = frame_ptr.offset(1).read_volatile();

                let msg = alloc::format!("  #{}: {:016x}\n", i, lr);
                crate::uart_print(msg.as_bytes());

                // Get next frame
                fp = frame_ptr.read_volatile();
            }
        }

        #[cfg(target_arch = "x86_64")]
        {
            use core::arch::asm;
            let mut rbp: u64;
            asm!("mov {}, rbp", out(reg) rbp);

            crate::uart_print(b"  [Stack unwinding requires frame pointers]\n");
            crate::uart_print(b"  [Build with RUSTFLAGS=\"-C force-frame-pointers=yes\"]\n");

            // Basic frame pointer walk
            for i in 0..10 {
                if rbp == 0 || rbp < 0x1000 {
                    break;
                }

                if rbp > 0x1000_0000_0000 {
                    break;
                }

                let frame_ptr = rbp as *const u64;
                let ret_addr = frame_ptr.offset(1).read_volatile();

                let msg = alloc::format!("  #{}: {:016x}\n", i, ret_addr);
                crate::uart_print(msg.as_bytes());

                rbp = frame_ptr.read_volatile();
            }
        }

        crate::uart_print(b"\n");
    }
}

fn print_next_steps() {
    unsafe {
        crate::uart_print(b"DEBUGGING STEPS:\n");
        crate::uart_print(b"----------------\n");
        crate::uart_print(b"  1. Check panic location and message above\n");
        crate::uart_print(b"  2. Examine register values for invalid pointers\n");
        crate::uart_print(b"  3. Check heap usage for memory exhaustion\n");
        crate::uart_print(b"  4. Review recent logs for error patterns\n");
        crate::uart_print(b"  5. If stack trace available, identify call chain\n");
        crate::uart_print(b"  6. Check system uptime for timing-related issues\n");
        crate::uart_print(b"\n");
        crate::uart_print(b"COMMON CAUSES:\n");
        crate::uart_print(b"--------------\n");
        crate::uart_print(b"  - Null or invalid pointer dereference\n");
        crate::uart_print(b"  - Array out of bounds access\n");
        crate::uart_print(b"  - Heap corruption or exhaustion\n");
        crate::uart_print(b"  - Stack overflow\n");
        crate::uart_print(b"  - Assertion failure\n");
        crate::uart_print(b"  - Unhandled error condition\n");
        crate::uart_print(b"\n");
    }
}

#[cfg(feature = "crash-dump")]
fn write_crash_dump(_info: &PanicInfo) {
    unsafe {
        crate::uart_print(b"CRASH DUMP:\n");
        crate::uart_print(b"-----------\n");
        crate::uart_print(b"  [Crash dump to disk not yet implemented]\n");
        // TODO: Write crash dump to virtio-blk device if available
        crate::uart_print(b"\n");
    }
}

fn log_panic_structured(info: &PanicInfo) {
    // Emit structured log for automated parsing
    #[cfg(feature = "structured-logging")]
    unsafe {
        let location = info.location()
            .map(|l| alloc::format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let message = if let Some(msg) = info.message() {
            alloc::format!("{}", msg)
        } else if let Some(payload) = info.payload().downcast_ref::<&str>() {
            payload.to_string()
        } else {
            "no message".to_string()
        };

        let timestamp = crate::time::get_timestamp_us();
        let log = alloc::format!(
            "{{\"ts\":{},\"subsystem\":\"PANIC\",\"status\":\"kernel_panic\",\"level\":\"FATAL\",\"location\":\"{}\",\"message\":\"{}\"}}\n",
            timestamp, location, message.replace("\"", "\\\"")
        );
        crate::uart_print(log.as_bytes());
    }
}

/// Disable interrupts (architecture-specific)
#[inline(always)]
fn disable_interrupts() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("msr daifset, #0xf");
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("cli");
    }

    #[cfg(target_arch = "riscv64")]
    unsafe {
        // Disable interrupts on RISC-V
        core::arch::asm!("csrci mstatus, 8");
    }
}

/// Halt the system (architecture-specific)
#[inline(always)]
fn halt() -> ! {
    unsafe {
        crate::uart_print(b"================================================================================\n");
        crate::uart_print(b"System halted.\n");
        crate::uart_print(b"================================================================================\n");
    }

    loop {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe");
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }

        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

/// Get current panic count
pub fn get_panic_count() -> u64 {
    PANIC_COUNT.load(Ordering::Relaxed)
}

/// Check if currently panicking
pub fn is_panicking() -> bool {
    PANICKING.load(Ordering::Relaxed)
}
