# Phase A1 Status and Runtime Documentation

## Overview

Phase A1 implements a fully preemptive multitasking kernel with virtual memory, process management, and a basic userspace environment. This phase builds upon the Phase A0 syscall infrastructure to provide:

- **Virtual Memory Management**: 4-level page tables (39-bit VA), demand paging with COW
- **Process Management**: Fork, execve, wait, exit with proper parent-child relationships
- **Preemptive Scheduler**: Round-robin scheduler with 10ms timeslices, timer IRQ-driven preemption
- **File Systems**: tmpfs (/) and devfs (/dev) with VFS layer
- **Userspace**: BusyBox shell with common utilities (ls, cat, echo, pipes, etc.)

## Implementation Status

### ✅ Completed Components

1. **Memory Management** (`crates/kernel/src/mm/`)
   - Buddy allocator for physical page allocation
   - 4-level page table implementation with W^X enforcement
   - COW (copy-on-write) for efficient fork()
   - Lazy allocation: pages allocated on first access
   - Per-process address spaces with TTBR0_EL1 switching

2. **Context Switching** (`crates/kernel/src/arch/aarch64/switch.S`)
   - Assembly routine saving/restoring callee-saved registers (x19-x30, SP)
   - Per-task 16KB kernel stacks
   - EL0 context management (ELR_EL1, SPSR_EL1, SP_EL0)

3. **Scheduler** (`crates/kernel/src/process/scheduler.rs`)
   - Round-robin algorithm with VecDeque run queue
   - Timeslice accounting (1 tick = 10ms at 100Hz)
   - Timer IRQ integration with preemption
   - Address space switching on context switch

4. **Interrupt Handling** (`crates/kernel/src/arch/aarch64/trap.rs`)
   - GICv3 interrupt controller support
   - Proper EOI/Deactivate sequence for interrupts
   - Timer IRQ → scheduler tick → preemption flow
   - Spurious interrupt detection

5. **Syscalls** (`crates/kernel/src/syscall/`)
   - fork: Creates child process with COW page tables
   - execve: Loads ELF binaries, sets up stack, jumps to entry point
   - wait/waitpid: Parent waits for child exit
   - exit: Zombie state, signals parent, cleanup
   - read/write: Console and file I/O
   - open/close: File descriptor management
   - getcwd/chdir: Working directory support (absolute paths only)

6. **Virtual File System** (`crates/kernel/src/vfs/`)
   - VFS layer with inode abstraction
   - tmpfs: In-memory filesystem mounted at /
   - devfs: Device filesystem at /dev
   - Console device: /dev/console for stdin/stdout/stderr

7. **Boot Sequence** (`crates/kernel/src/main.rs`)
   - Buddy allocator initialization (112MB RAM)
   - Process table initialization
   - Scheduler initialization
   - VFS and filesystem mounting
   - PID 1 creation and scheduling
   - Initramfs unpacking (when ready)
   - /sbin/init execution (when initramfs available)

### 🚧 Pending for Complete A1

1. **Initramfs Build**: Run `bash scripts/build_initramfs.sh` to create BusyBox root filesystem
2. **Boot Wiring**: Uncomment initramfs unpacking and /sbin/init execution in `main.rs`
3. **Testing**: Run acceptance tests with BusyBox shell

## Building and Running

### Prerequisites

```bash
# Install Rust nightly and AArch64 target
rustup default nightly
rustup target add aarch64-unknown-none

# Install QEMU
sudo apt-get install qemu-system-aarch64  # Ubuntu/Debian
brew install qemu                         # macOS
```

### Build Kernel

```bash
# From project root
cargo build --release --target aarch64-unknown-none

# Kernel binary location:
# target/aarch64-unknown-none/release/sis_kernel
```

### Build Initramfs (BusyBox Root Filesystem)

```bash
# Create initramfs with BusyBox
bash scripts/build_initramfs.sh

# This creates:
# - build/initramfs.cpio: Uncompressed CPIO archive
# - build/rootfs/: Root filesystem contents
# - crates/kernel/initramfs_data.rs: Rust include file (auto-generated)
```

### Boot with QEMU

```bash
# Standard boot command for Phase A1
qemu-system-aarch64 \
  -machine virt,gic-version=3 \
  -cpu cortex-a72 \
  -smp 1 \
  -m 128M \
  -nographic \
  -kernel target/aarch64-unknown-none/release/sis_kernel \
  -serial mon:stdio

# Alternative: Use helper script if available
bash scripts/run_qemu.sh
```

### Expected Boot Sequence

```
SIS KERNEL PHASE A1
MM: BUDDY ALLOCATOR
MM: BUDDY READY (28672 pages)
PROCESS: INIT TABLE
PROCESS: TABLE READY
SCHEDULER: INIT
SCHEDULER: READY
VFS: INIT
VFS: MOUNT TMPFS AT /
VFS: MOUNT DEVFS AT /dev
VFS: READY
INIT: CREATING PID 1
INIT: PID 1 CREATED
SCHEDULER: ENQUEUE PID 1
SCHEDULER: PID 1 RUNNING
PHASE A1: BOOT WIRING COMPLETE
GIC: INIT
[... GIC initialization messages ...]
[IRQ latency benchmark output if enabled]

SIS Kernel Phase A1 - Init starting
Init complete - starting shell
/ #
```

### Expected Shell Prompt

```bash
/ # ls /
bin  dev  etc  proc  sbin  sys  tmp

/ # echo hello
hello

/ # cat /dev/console
[Type some text and press Enter to see echo]

/ # yes | head -n 1
y

/ # touch /tmp/test.txt
/ # ls /tmp
test.txt

/ # sh -c 'exit 42'
/ # echo $?
42
```

## Known Limitations (Phase A1)

### Filesystem Limitations

1. **Absolute Paths Only**: All syscalls require absolute paths
   - ❌ `open("file.txt")` will fail
   - ✅ `open("/tmp/file.txt")` works
   - Relative path resolution with CWD is Phase A2

2. **No Procfs**: `/proc` filesystem not yet implemented
   - No `/proc/cpuinfo`, `/proc/meminfo`, `/proc/[pid]/` entries
   - Planned for Phase A2

3. **No PTY/TTY**: `/dev/ptmx` and pseudo-terminals not available
   - Only `/dev/console` for I/O
   - Background jobs and job control unavailable
   - Planned for Phase A2

### Process Management Limitations

1. **Blocking Console Reads**: `read()` on console blocks indefinitely
   - No non-blocking I/O or select/poll
   - Ctrl+C not handled (no signal support yet)

2. **No Signals**: Signal delivery not implemented
   - No SIGCHLD, SIGKILL, SIGTERM, etc.
   - Process can only exit voluntarily
   - Planned for Phase A2+

3. **No Pipes/FIFOs Persistence**: Pipes work but are anonymous
   - Named pipes (FIFOs) not supported
   - Pipe buffers limited by kernel memory

### Memory Limitations

1. **Fixed User Address Space**: 39-bit VA (512GB max)
   - User space: 0x0000_0000_0000_0000 to 0x0000_007F_FFFF_FFFF
   - Kernel space: 0xFFFF_8000_0000_0000 to 0xFFFF_FFFF_FFFF_FFFF

2. **No Swap**: All memory is physical RAM
   - Out-of-memory kills are not graceful
   - No memory overcommit

3. **W^X Enforced**: Pages cannot be both writable and executable
   - Writable pages automatically get UXN (execute never) flag
   - Protects against code injection but prevents JIT

## Testing

### Phase A1 Acceptance Tests

```bash
# Run automated acceptance tests
bash tests/phase_a1/run_tests.sh

# Manual tests in shell:
/ # ls /                    # List root directory
/ # echo test               # Echo to console
/ # cat /etc/passwd         # Read file
/ # touch /tmp/newfile      # Create file
/ # ls /tmp                 # Verify file created
/ # yes | head -n 1         # Pipe test
/ # sh -c 'exit 42'         # Exit code test
/ # echo $?                 # Should print 42
```

### Stress Testing

```bash
# Fork/exec stress test (from shell)
/ # for i in 1 2 3 4 5; do sh -c 'echo child $i' & done

# Pipe stress test
/ # yes > /dev/null &       # Background process (blocks without job control)

# Scheduler stress (create multiple children)
/ # sh -c 'for i in 1 2 3; do sleep 1 & done; wait'
```

## Troubleshooting

### Kernel Panics on Boot

1. **Buddy allocator failure**: Check RAM size (-m 128M) and ram_start/ram_size in main.rs
2. **GIC initialization failure**: Verify `-machine virt,gic-version=3` flag
3. **Page table allocation failure**: Increase RAM or reduce kernel image size

### Timer IRQ Not Firing

1. Check GIC initialization messages for PPI 30 enablement
2. Verify `ICC_PMR_EL1` is 0xFF or interrupt priority < PMR value
3. Check `ICC_IGRPEN1_EL1` is 1 (Group 1 interrupts enabled)
4. Verify timer is initialized with non-zero expiry value

### Scheduler Not Preempting

1. Verify `handle_irq()` is being called (add debug print)
2. Check `timer_tick()` is decrementing timeslice
3. Verify `need_resched()` returns true after timeslice expires
4. Check `schedule()` is actually switching tasks (print current_pid)

### Shell Not Appearing

1. Verify initramfs was unpacked successfully
2. Check `/sbin/init` exists and is executable (0755)
3. Verify PID 1 was created and enqueued
4. Check `execve()` succeeded loading /sbin/init
5. Verify fd 0/1/2 are bound to /dev/console for PID 1

### Console I/O Not Working

1. Check UART initialization succeeded
2. Verify `/dev/console` device exists in devfs
3. Check file descriptors 0/1/2 are open in the task
4. Verify sys_read/sys_write are being called correctly

### Page Faults in Userspace

1. **Lazy allocation**: Normal for first access to stack/heap pages
2. **COW fault**: Normal for writes to shared (forked) pages
3. **Segmentation fault**: Access outside VMA bounds
   - Check VMA list for the task
   - Verify ELF loading created proper VMAs
   - Check stack VMA is large enough (8MB default)

## Architecture Details

### Memory Layout

```
User Space (TTBR0_EL1):
  0x0000_0000_0000_0000 - 0x0000_007F_FFFF_FFFF  (512GB)
    0x0000_0000_0040_0000: ELF text/data segments (loaded by execve)
    0x0000_007F_FFF0_0000: User stack (grows down, 8MB)

Kernel Space (TTBR1_EL1):
  0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF  (512GB)
    0xFFFF_8000_4000_0000: Physical RAM mapped (identity)
    Kernel stacks: 16KB per task (4 pages)
```

### Page Table Structure

- **Levels**: 4 (L0, L1, L2, L3)
- **Page Size**: 4KB
- **VA Bits**: 39 (512GB addressable per TTBR)
- **Granule**: 4KB
- **PTE Flags**: VALID, USER, READONLY, ACCESS, UXN, PXN, COW (software bit)

### Context Switch Flow

1. Timer IRQ fires → `handle_irq()`
2. Read `ICC_IAR1_EL1` to get interrupt ID
3. Call `timer_tick()` → decrement timeslice
4. Check `need_resched()` → true when timeslice expires
5. Save current task's trap frame
6. Call `schedule()`:
   - Pick next task from run queue (round-robin)
   - Switch TTBR0_EL1 to new task's page table
   - Set ELR_EL1 (PC), SPSR_EL1 (PSTATE), SP_EL0 (stack)
7. Write `ICC_EOIR1_EL1` and `ICC_DIR_EL1` to acknowledge IRQ
8. Return via ERET → enter new task at PC

### Syscall Flow

1. User calls `svc #0` instruction
2. CPU traps to `vector_table` → `sync_current_elx_sp0`
3. Trap frame saved on kernel stack
4. `handle_sync()` decodes ESR_EL1 → syscall number
5. Dispatch to syscall handler via `crate::syscall::handle_syscall()`
6. Handler performs operation, returns result in x0
7. Return via ERET → resume user mode

## Next Steps

### Immediate (to complete A1)

1. **Build initramfs**: `bash scripts/build_initramfs.sh`
2. **Uncomment boot wiring**: Enable initramfs unpack and execve in `main.rs`
3. **Rebuild kernel**: `cargo build --release --target aarch64-unknown-none`
4. **Test**: Boot with QEMU and verify BusyBox shell
5. **Run acceptance tests**: `bash tests/phase_a1/run_tests.sh`

### Phase A2 (next milestone)

1. **Procfs**: `/proc/cpuinfo`, `/proc/meminfo`, `/proc/[pid]/cmdline`
2. **PTY**: `/dev/ptmx`, `/dev/pts/N`, job control
3. **Relative Paths**: CWD resolution, `openat()` syscall
4. **Extended Attributes**: File metadata, permissions

### Phase B (future)

1. **Persistent Storage**: ext2 filesystem, virtio-blk driver
2. **Block Cache**: Page cache for disk I/O
3. **Network**: virtio-net driver, TCP/IP stack
4. **SMP**: Multi-core support, per-CPU scheduler, spinlocks

## References

- **OS-BLUEPRINT.md**: Overall project roadmap and milestones
- **docs/syscall_abi.md**: Syscall ABI and calling conventions
- **docs/memory_layout.md**: Virtual memory layout and page table format
- **tests/phase_a1/**: Acceptance test suite

## Changelog

- **2024-01-XX**: Phase A1 boot wiring complete (scheduler + PID 1 + IRQ handling)
- **2024-01-XX**: Context switching and scheduler implementation
- **2024-01-XX**: Page tables and fault handlers (COW + lazy allocation)
- **2024-01-XX**: Initramfs build scripts and BusyBox integration
- **2024-01-XX**: Phase A0 complete (syscall infrastructure, trap handling)
