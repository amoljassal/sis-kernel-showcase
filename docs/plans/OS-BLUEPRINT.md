# SIS Kernel → Complete OS - Blueprint

**Version**: 1.0.0
**Status**: Planning Phase
**Target Platform**: QEMU virt (ARM64 AArch64)
**Last Updated**: 2025-11-05

---

## Vision

Transform the **SIS Kernel** from an AI-native research platform into a **complete, production-ready operating system** with:

1. **POSIX Userspace**: Full process model, syscalls, signals, pipes, TTY, ELF execution
2. **Persistent Storage**: Block devices, ext2/ext4 filesystems, journaling
3. **Networking**: TCP/IP stack, sockets API, DHCP/DNS/HTTP
4. **Security**: Permissions, memory protections (NX/W^X), ASLR, COW, entropy
5. **Multi-core**: SMP scheduling, load balancing, PSCI CPU bring-up
6. **Resilience**: Crash recovery, journaling, integrity guarantees

### Core Goals

- **Lowest-risk path**: Build incrementally with testable milestones
- **Highest leverage**: Prioritize features that unlock ecosystem compatibility
- **Production quality**: CI-tested, documented, reproducible
- **Ecosystem ready**: Run standard Linux userspace (BusyBox, musl, POSIX tools)

---

## Architecture Overview

### System Evolution Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Phase G (Optional)                           │
│              Graphics (virtio-gpu) + Audio                      │
└─────────────────────────────────────────────────────────────────┘
         ▲
         │
┌─────────────────────────────────────────────────────────────────┐
│                    Phase F - Resilience                         │
│         ext4 Journaling + Crash Recovery + NTP                  │
├─────────────────────────────────────────────────────────────────┤
│  Journal Replay │  Barriers │  fsck │  Panic Handling          │
└─────────────────────────────────────────────────────────────────┘
         ▲
         │
┌─────────────────────────────────────────────────────────────────┐
│                    Phase E - SMP & Performance                  │
│      Multi-core Scheduling + Load Balancing + Timers            │
├─────────────────────────────────────────────────────────────────┤
│  PSCI │ Per-CPU Runqueues │ Affinity │ High-res Timers         │
└─────────────────────────────────────────────────────────────────┘
         ▲
         │
┌─────────────────────────────────────────────────────────────────┐
│              Phase D - Security & Memory Protections            │
│      UID/GID + NX/W^X + ASLR + COW + /dev/urandom              │
├─────────────────────────────────────────────────────────────────┤
│  Credentials │ mprotect │ Guard Pages │ COW Faults │ Entropy   │
└─────────────────────────────────────────────────────────────────┘
         ▲
         │
┌─────────────────────────────────────────────────────────────────┐
│              Phase C - Networking (TCP/IP Stack)                │
│       virtio-net + Sockets + DHCP/DNS/HTTP                      │
├─────────────────────────────────────────────────────────────────┤
│  Driver │ Socket ABI │ TCP/IP │ ARP/ICMP │ DNS Resolver        │
└─────────────────────────────────────────────────────────────────┘
         ▲
         │
┌─────────────────────────────────────────────────────────────────┐
│          Phase B - Persistent Storage (Block + ext2)            │
│         virtio-blk + Block Layer + ext2 Filesystem             │
├─────────────────────────────────────────────────────────────────┤
│  Block Driver │ Request Queues │ Partitioning │ ext2 VFS       │
└─────────────────────────────────────────────────────────────────┘
         ▲
         │
┌─────────────────────────────────────────────────────────────────┐
│         Phase A - Userspace Bring-Up (Initramfs + BusyBox)     │
│      Process Model + Syscalls + VFS + TTY + ELF Loader         │
├─────────────────────────────────────────────────────────────────┤
│  fork/exec │ Signals │ Pipes │ TTY/PTY │ tmpfs/devfs/procfs    │
└─────────────────────────────────────────────────────────────────┘
         ▲
         │
┌─────────────────────────────────────────────────────────────────┐
│                    Current SIS Kernel (Phase 6)                 │
│     AI-Native Features + Meta-Agent + Explainability           │
├─────────────────────────────────────────────────────────────────┤
│  Virtio Console │ Shell │ Caps │ Memory │ Timers │ NN Infer    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase A — Userspace Bring-Up (Initramfs + BusyBox)

**Objective**: Boot to a BusyBox shell on TTY with a usable POSIX-ish core (fork/exec/wait/signals/pipe/tty/ELF).

**Timeline**: 4 weeks (split into A0/A1/A2 to de-risk scope)

### Scope

Transform the kernel from single-binary shell to multi-process userspace environment running standard POSIX utilities. Split into three sub-phases to manage complexity:

- **A0** (1 week): Syscall infrastructure foundation
- **A1** (2 weeks): Minimal userspace with COW fork
- **A2** (1 week): PTY and full procfs entries

### Implementation Details

### Phase A0 — Syscall Infrastructure (Week 1)

**Objective**: Establish trap path, basic syscall dispatch, and preemptive scheduling foundation.

#### 1. AArch64 Trap Path (EL0 → EL1)

**Exception Vector Setup**:
```rust
// crates/kernel/src/arch/aarch64/trap.rs

#[repr(C, align(2048))]
pub struct ExceptionVectors {
    // Each vector is 128 bytes (32 instructions max)
    curr_el_sp0_sync: [u32; 32],
    curr_el_sp0_irq: [u32; 32],
    curr_el_sp0_fiq: [u32; 32],
    curr_el_sp0_serror: [u32; 32],

    curr_el_spx_sync: [u32; 32],
    curr_el_spx_irq: [u32; 32],
    curr_el_spx_fiq: [u32; 32],
    curr_el_spx_serror: [u32; 32],

    lower_el_aarch64_sync: [u32; 32],   // ← SVC from EL0 enters here
    lower_el_aarch64_irq: [u32; 32],
    lower_el_aarch64_fiq: [u32; 32],
    lower_el_aarch64_serror: [u32; 32],

    lower_el_aarch32_sync: [u32; 32],
    lower_el_aarch32_irq: [u32; 32],
    lower_el_aarch32_fiq: [u32; 32],
    lower_el_aarch32_serror: [u32; 32],
}

#[repr(C)]
pub struct TrapFrame {
    pub x0: u64,  pub x1: u64,  pub x2: u64,  pub x3: u64,
    pub x4: u64,  pub x5: u64,  pub x6: u64,  pub x7: u64,
    pub x8: u64,  pub x9: u64,  pub x10: u64, pub x11: u64,
    pub x12: u64, pub x13: u64, pub x14: u64, pub x15: u64,
    pub x16: u64, pub x17: u64, pub x18: u64, pub x19: u64,
    pub x20: u64, pub x21: u64, pub x22: u64, pub x23: u64,
    pub x24: u64, pub x25: u64, pub x26: u64, pub x27: u64,
    pub x28: u64, pub x29: u64, pub x30: u64, // LR
    pub sp: u64,  pub pc: u64,  pub pstate: u64,
}

pub unsafe fn handle_sync_exception(frame: &mut TrapFrame) {
    let esr = read_esr_el1();
    let ec = (esr >> 26) & 0x3F;  // Exception Class

    match ec {
        0x15 => handle_syscall(frame),  // SVC from AArch64 EL0
        0x20 | 0x24 => handle_page_fault(frame, esr),  // Instruction/Data Abort
        _ => panic!("Unhandled exception: EC={:#x}", ec),
    }
}
```

**Syscall ABI**:
- Arguments: x0..x5 (6 args max)
- Syscall number: x8
- Return value: x0 (negative for errno)
- SVC #0 instruction transitions to EL1

#### 2. Syscall Dispatcher (Minimal Set for A0)

**A0 Minimal Syscalls** (4 only):

| Syscall | Number | Purpose |
|---------|--------|---------|
| read | 63 | Read from FD (console only) |
| write | 64 | Write to FD (console only) |
| exit | 93 | Terminate process |
| getpid | 172 | Get process ID |

**Implementation**:

```rust
// crates/kernel/src/syscall/mod.rs

pub fn syscall_dispatcher(frame: &mut TrapFrame) -> isize {
    let nr = frame.x8 as usize;
    let args = [frame.x0, frame.x1, frame.x2, frame.x3, frame.x4, frame.x5];

    let result = match nr {
        63 => sys_read(args[0] as i32, args[1] as *mut u8, args[2] as usize),
        64 => sys_write(args[0] as i32, args[1] as *const u8, args[2] as usize),
        93 => sys_exit(args[0] as i32),
        172 => sys_getpid(),
        _ => Err(Errno::ENOSYS),
    };

    match result {
        Ok(ret) => ret,
        Err(e) => -(e as isize),  // Negative errno
    }
}

pub fn sys_read(fd: i32, buf: *mut u8, count: usize) -> Result<isize, Errno> {
    if fd != 0 { return Err(Errno::EBADF); }  // Only stdin for now
    let kernel_buf = copy_from_user(buf, count)?;
    let n = CONSOLE.lock().read(&mut kernel_buf)?;
    copy_to_user(buf, &kernel_buf[..n])?;
    Ok(n as isize)
}

pub fn sys_write(fd: i32, buf: *const u8, count: usize) -> Result<isize, Errno> {
    if fd != 1 && fd != 2 { return Err(Errno::EBADF); }  // Only stdout/stderr
    let kernel_buf = copy_from_user(buf as *mut u8, count)?;
    CONSOLE.lock().write(&kernel_buf)?;
    Ok(count as isize)
}

pub fn sys_exit(code: i32) -> Result<isize, Errno> {
    current_process().exit_code = Some(code);
    current_process().state = ProcessState::Dead;
    schedule();  // Never returns
    unreachable!()
}

pub fn sys_getpid() -> Result<isize, Errno> {
    Ok(current_process().pid as isize)
}
```

#### 3. uaccess Helpers (Safe User Memory Access)

```rust
// crates/kernel/src/syscall/uaccess.rs

/// Copy data from user space to kernel space with safety checks
pub fn copy_from_user<T>(user_ptr: *const T, count: usize) -> Result<Vec<T>, Errno>
where T: Copy {
    // 1. Validate pointer is in user address space (< KERNEL_BASE)
    if (user_ptr as usize) >= KERNEL_BASE {
        return Err(Errno::EFAULT);
    }

    // 2. Check for overflow
    let total_size = count.checked_mul(size_of::<T>())
        .ok_or(Errno::EINVAL)?;

    // 3. Verify range is mapped and accessible
    let current = current_process();
    if !current.address_space.is_user_readable(user_ptr as usize, total_size) {
        return Err(Errno::EFAULT);
    }

    // 4. Perform copy with fault handling
    let mut buf = Vec::with_capacity(count);
    unsafe {
        let result = catch_page_fault(|| {
            ptr::copy_nonoverlapping(user_ptr, buf.as_mut_ptr(), count);
            buf.set_len(count);
        });
        result?;
    }

    Ok(buf)
}

/// Copy data from kernel space to user space
pub fn copy_to_user<T>(user_ptr: *mut T, data: &[T]) -> Result<(), Errno>
where T: Copy {
    if (user_ptr as usize) >= KERNEL_BASE {
        return Err(Errno::EFAULT);
    }

    let current = current_process();
    if !current.address_space.is_user_writable(user_ptr as usize, data.len() * size_of::<T>()) {
        return Err(Errno::EFAULT);
    }

    unsafe {
        catch_page_fault(|| {
            ptr::copy_nonoverlapping(data.as_ptr(), user_ptr, data.len());
        })?;
    }

    Ok(())
}

/// Execute closure and catch page faults
unsafe fn catch_page_fault<F, R>(f: F) -> Result<R, Errno>
where F: FnOnce() -> R {
    // Set fault handler flag
    current_cpu().in_uaccess = true;
    let result = f();
    current_cpu().in_uaccess = false;
    Ok(result)
}
```

#### 4. Kernel Logging (printk + dmesg)

```rust
// crates/kernel/src/lib/printk.rs

pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

pub struct LogBuffer {
    buffer: RingBuffer<LogEntry, 4096>,
    lock: SpinLock<()>,
}

struct LogEntry {
    timestamp_us: u64,
    level: LogLevel,
    message: [u8; 256],
    len: usize,
}

static KERNEL_LOG: LogBuffer = LogBuffer::new();

#[macro_export]
macro_rules! printk {
    ($level:expr, $($arg:tt)*) => {
        $crate::lib::printk::log($level, format_args!($($arg)*))
    };
}

pub fn log(level: LogLevel, args: core::fmt::Arguments) {
    let timestamp = timer::get_time_us();
    let message = format!("{}", args);

    KERNEL_LOG.lock().push(LogEntry {
        timestamp_us: timestamp,
        level,
        message: message.as_bytes(),
        len: message.len().min(256),
    });

    // Also output to console if early boot or level >= Warn
    if level <= LogLevel::Warn || !CONSOLE.is_initialized() {
        CONSOLE.write_str(&message);
    }
}

// Syscall to read dmesg
pub fn sys_dmesg(buf: *mut u8, count: usize) -> Result<isize, Errno> {
    let entries = KERNEL_LOG.lock().drain_all();
    let mut written = 0;

    for entry in entries {
        if written + entry.len > count {
            break;
        }
        copy_to_user(buf.add(written), &entry.message[..entry.len])?;
        written += entry.len;
    }

    Ok(written as isize)
}
```

#### 5. Minimal Preemptive Scheduler (Single CPU)

```rust
// crates/kernel/src/process/scheduler.rs

pub struct Scheduler {
    runqueue: VecDeque<Arc<Mutex<Process>>>,
    current: Option<Arc<Mutex<Process>>>,
    timeslice_ticks: u64,
}

static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

impl Scheduler {
    const TIMESLICE_MS: u64 = 10;  // 10ms per process

    pub fn new() -> Self {
        Self {
            runqueue: VecDeque::new(),
            current: None,
            timeslice_ticks: 0,
        }
    }

    pub fn add_process(&mut self, proc: Arc<Mutex<Process>>) {
        self.runqueue.push_back(proc);
    }

    pub fn schedule(&mut self) {
        // Save current process context if still running
        if let Some(current) = &self.current {
            let mut proc = current.lock();
            if proc.state == ProcessState::Running {
                proc.state = ProcessState::Ready;
                self.runqueue.push_back(current.clone());
            }
        }

        // Pick next process (simple round-robin)
        while let Some(next) = self.runqueue.pop_front() {
            let mut proc = next.lock();
            if proc.state == ProcessState::Ready {
                proc.state = ProcessState::Running;
                self.current = Some(next.clone());
                self.timeslice_ticks = Self::TIMESLICE_MS * TICKS_PER_MS;

                // Switch to process address space and restore context
                unsafe {
                    switch_to_process(&proc);
                }
                return;
            }
        }

        // No runnable process - idle
        self.current = None;
        idle();
    }

    pub fn timer_tick(&mut self) {
        if self.timeslice_ticks > 0 {
            self.timeslice_ticks -= 1;
        }

        if self.timeslice_ticks == 0 {
            self.schedule();  // Preempt current process
        }
    }

    pub fn block_current(&mut self) {
        if let Some(current) = &self.current {
            current.lock().state = ProcessState::Sleeping;
        }
        self.schedule();
    }

    pub fn wake_process(&mut self, pid: Pid) {
        // Find process and mark as Ready
        for proc in &self.runqueue {
            let mut p = proc.lock();
            if p.pid == pid && p.state == ProcessState::Sleeping {
                p.state = ProcessState::Ready;
                break;
            }
        }
    }
}

// Called from timer interrupt
pub fn schedule_tick() {
    SCHEDULER.lock().timer_tick();
}

// Yield CPU voluntarily
pub fn yield_now() {
    SCHEDULER.lock().schedule();
}
```

#### 6. A0 Acceptance Tests

```bash
# tests/phase_a0/run_tests.sh

# Test 1: Trap and return
test_trap() {
    # Load minimal test program that does: write(1, "OK\n", 3); exit(0);
    expect "OK" from console
    expect clean exit
}

# Test 2: Syscall dispatch
test_syscall() {
    # Call read/write/getpid from test program
    expect correct return values
    expect errno on invalid fd
}

# Test 3: dmesg readable
test_dmesg() {
    expect kernel log messages present
    expect no corruption
}

# Test 4: Timer preemption
test_preempt() {
    # Load two CPU-bound processes
    expect both make progress (interleaved output)
    expect ~10ms timeslice behavior
}
```

---

### Phase A1 — Minimal Userspace with COW (Weeks 2-3)

**Objective**: Boot to BusyBox shell with fork/exec/wait working correctly.

#### 1. Process Model

```rust
// crates/kernel/src/process/mod.rs
pub struct Process {
    pub pid: Pid,
    pub parent: Option<Pid>,
    pub children: Vec<Pid>,
    pub state: ProcessState,
    pub address_space: AddressSpace,
    pub open_files: Vec<FileDescriptor>,
    pub signals: SignalQueue,
    pub credentials: Credentials,
    pub exit_code: Option<i32>,
}

pub enum ProcessState {
    Ready,
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Dead,
}

pub struct PidTable {
    processes: HashMap<Pid, Arc<Mutex<Process>>>,
    next_pid: AtomicU32,
}
```

**Key Features**:
- PID allocation (1 = init, 2+ = user processes)
- Parent/child relationships (PPID tracking)
- Zombie collection (wait4 reaping)
- Orphan reaper (reparent to PID 1)

#### 2. Copy-on-Write (COW) Fork **[CRITICAL - Moved from Phase D]**

**Rationale**: Fork without COW will cause OOM with BusyBox. Must be in A1.

```rust
// crates/kernel/src/mm/cow.rs

pub struct Page {
    phys_addr: PhysAddr,
    refcount: AtomicU32,
    flags: PageFlags,
}

bitflags! {
    pub struct PageFlags: u32 {
        const COW = 1 << 0;
        const DIRTY = 1 << 1;
        const ACCESSED = 1 << 2;
    }
}

// Fork implementation with COW
pub fn sys_fork() -> Result<isize, Errno> {
    let parent = current_process();
    let mut child = Process::new();

    // Copy process metadata
    child.pid = alloc_pid();
    child.parent = Some(parent.pid);
    child.credentials = parent.credentials.clone();
    child.cwd = parent.cwd.clone();

    // Clone address space with COW
    child.address_space = parent.address_space.clone_with_cow()?;

    // Clone file descriptor table
    for (fd, file) in parent.open_files.iter().enumerate() {
        child.open_files[fd] = file.clone();
        file.refcount.fetch_add(1, Ordering::SeqCst);
    }

    // Add to process table
    PROCESS_TABLE.lock().insert(child.pid, Arc::new(Mutex::new(child)));

    // Return 0 to child, child_pid to parent
    Ok(child.pid as isize)
}

impl AddressSpace {
    /// Clone address space with COW semantics
    pub fn clone_with_cow(&self) -> Result<Self, Errno> {
        let mut new_space = AddressSpace::new()?;

        for vma in &self.vmas {
            // 1. Mark parent pages as read-only + COW
            for page in vma.pages() {
                let pte = self.page_table.lookup_mut(page.vaddr)?;
                if pte.is_writable() {
                    pte.set_read_only();
                    pte.set_cow();
                    inc_refcount(pte.phys_addr());
                }
            }

            // 2. Clone VMA and PTEs (point to same physical pages)
            let new_vma = vma.clone();
            new_space.vmas.push(new_vma);
            self.page_table.clone_ptes_into(&mut new_space.page_table, vma.start, vma.end)?;
        }

        // Flush TLB on parent CPU
        flush_tlb_all();

        Ok(new_space)
    }
}

/// Handle write fault on COW page
pub fn handle_cow_fault(fault_addr: VAddr) -> Result<(), Errno> {
    let current = current_process();
    let pte = current.address_space.page_table.lookup_mut(fault_addr)?;

    if !pte.is_cow() {
        return Err(Errno::EFAULT);  // Not a COW fault
    }

    let old_phys = pte.phys_addr();
    let refcount = get_refcount(old_phys);

    if refcount == 1 {
        // Last reference - just make writable
        pte.set_writable();
        pte.clear_cow();
    } else {
        // Multiple references - copy page
        let new_page = alloc_page()?;
        unsafe {
            copy_page(old_phys, new_page);
        }

        pte.set_phys_addr(new_page);
        pte.set_writable();
        pte.clear_cow();

        // Decrement refcount on old page
        dec_refcount(old_phys);
    }

    flush_tlb_page(fault_addr);
    Ok(())
}

// Page fault handler (in arch/aarch64/fault.rs)
pub fn handle_page_fault(frame: &mut TrapFrame, esr: u64) {
    let fault_addr = VAddr::new(read_far_el1());
    let write_fault = (esr & (1 << 6)) != 0;  // WnR bit

    if write_fault {
        // Try COW handler first
        if handle_cow_fault(fault_addr).is_ok() {
            return;  // COW resolved, continue execution
        }
    }

    // Not COW or COW failed - check other fault types
    // ... guard page, unmapped, permissions, etc.
}
```

**Refcounting**:
```rust
// Global page frame database
static PAGE_DB: Mutex<HashMap<PhysAddr, Page>> = Mutex::new(HashMap::new());

pub fn inc_refcount(phys: PhysAddr) {
    PAGE_DB.lock().get_mut(&phys).unwrap().refcount.fetch_add(1, Ordering::SeqCst);
}

pub fn dec_refcount(phys: PhysAddr) {
    let db = PAGE_DB.lock();
    let page = db.get_mut(&phys).unwrap();
    let old_count = page.refcount.fetch_sub(1, Ordering::SeqCst);

    if old_count == 1 {
        // Last reference - free page
        free_page(phys);
        db.remove(&phys);
    }
}

pub fn get_refcount(phys: PhysAddr) -> u32 {
    PAGE_DB.lock().get(&phys).unwrap().refcount.load(Ordering::SeqCst)
}
```

#### 3. ELF Loader

**Format**: ELF64 (AArch64)

**Parsing Requirements**:
- ELF header validation (magic, class, machine)
- Program headers (PT_LOAD, PT_INTERP, PT_GNU_STACK)
- Section headers (optional for dynamic linking)

**Memory Mapping**:
```rust
// crates/kernel/src/exec/elf.rs
pub struct ElfLoader {
    pub entry_point: u64,
    pub base_addr: u64,
    pub brk_start: u64,
}

impl ElfLoader {
    pub fn load(&mut self, elf_data: &[u8]) -> Result<(), ElfError> {
        // 1. Parse ELF header
        // 2. Map PT_LOAD segments (respecting p_flags)
        // 3. Zero BSS sections
        // 4. Setup auxv on stack
        // 5. Return entry point
    }
}
```

**Stack Layout** (on exec):
```
High address
┌────────────────┐
│  envp strings  │
├────────────────┤
│  argv strings  │
├────────────────┤
│  auxv[]        │  AT_PHDR, AT_ENTRY, AT_PAGESZ, etc.
├────────────────┤
│  envp[]        │  NULL-terminated
├────────────────┤
│  argv[]        │  NULL-terminated
├────────────────┤
│  argc          │  8 bytes
└────────────────┘
Low address (SP)
```

#### 4. VFS (Virtual Filesystem)

**Architecture**:
```
        VFS Layer
     ┌──────┴──────┐
  tmpfs       devfs      procfs
    │           │           │
 (RAM)     /dev nodes   /proc
```

**Key Structures**:

```rust
// crates/kernel/src/vfs/mod.rs
pub struct Inode {
    pub ino: u64,
    pub mode: FileMode,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime: Timespec,
    pub mtime: Timespec,
    pub ctime: Timespec,
    pub ops: &'static InodeOps,
}

pub trait InodeOps {
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize>;
    fn lookup(&self, name: &str) -> Result<Arc<Inode>>;
    fn create(&self, name: &str, mode: FileMode) -> Result<Arc<Inode>>;
}

pub struct Dentry {
    pub name: String,
    pub inode: Arc<Inode>,
    pub parent: Weak<Dentry>,
    pub children: HashMap<String, Arc<Dentry>>,
}

pub struct MountPoint {
    pub path: String,
    pub fs_type: &'static str,
    pub root: Arc<Dentry>,
}
```

**Filesystems**:

1. **tmpfs** (in-RAM):
   - Block allocator for file data
   - Inode cache
   - Directory operations

2. **devfs** (/dev):
   - Character devices: console, tty, null, zero, random, urandom
   - Block devices: (Phase B)
   - PTY nodes: ptmx, pts/N

3. **procfs** (/proc):
   - /proc/cpuinfo - CPU information
   - /proc/meminfo - Memory stats
   - /proc/[pid]/cmdline - Process command line
   - /proc/[pid]/stat - Process status
   - /proc/[pid]/status - Human-readable status

4. **sysfs** (/sys):
   - /sys/devices - Device tree
   - /sys/fs - Filesystem info
   - /sys/kernel - Kernel parameters

#### 5. TTY Subsystem (Console Only - A1)

**Architecture**:
```
┌──────────────┐
│  User App    │
└──────┬───────┘
       │ read/write/ioctl
       ▼
┌──────────────┐
│   TTY Line   │ ← Minimal line discipline
│  Discipline  │   echo, backspace, Ctrl-C
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Console Drv  │ ← virtio-console (existing)
└──────────────┘
```

**TTY Features (A1 Minimal)**:
- Basic line discipline: echo, backspace
- Minimal termios support
- Ctrl-C generates SIGINT (basic signal support)
- /dev/console and /dev/tty devices

**Device Nodes**:
- /dev/console - System console (virtio-console)
- /dev/tty - Controlling terminal (same as console for A1)

#### 6. Full Syscall Set for A1 (MVP - 30 syscalls)

| Syscall | Number | Purpose | Priority |
|---------|--------|---------|----------|
| read | 63 | Read from FD | P0 |
| write | 64 | Write to FD | P0 |
| open | 56 | Open file/device | P0 |
| close | 57 | Close FD | P0 |
| fstat | 80 | File metadata | P0 |
| lseek | 62 | Seek in file | P1 |
| ioctl | 29 | TTY control | P0 |
| pipe | 59 | Create pipe | P0 |
| dup/dup2 | 23/24 | Duplicate FD | P0 |
| poll | 73 | I/O multiplexing | P1 |
| brk | 214 | Heap management | P0 |
| mmap | 222 | Memory mapping | P0 |
| munmap | 215 | Unmap memory | P1 |
| mprotect | 226 | Change protections | P1 |
| fork | 220 | Create process | P0 |
| execve | 221 | Execute program | P0 |
| wait4 | 260 | Wait for child | P0 |
| exit | 93 | Terminate process | P0 |
| getpid | 172 | Get process ID | P0 |
| getppid | 173 | Get parent PID | P1 |
| kill | 129 | Send signal | P0 |
| sigaction | 134 | Set signal handler | P0 |
| sigreturn | 139 | Return from signal | P0 |
| clock_gettime | 113 | Get time | P1 |
| nanosleep | 101 | Sleep | P1 |
| getcwd | 17 | Get CWD | P1 |
| chdir | 49 | Change directory | P1 |
| mkdir | 34 | Create directory | P1 |
| rmdir | 35 | Remove directory | P1 |
| unlink | 10 | Remove file | P1 |

#### 7. Initramfs (BusyBox + musl static)

**Structure**:
```
initramfs.cpio.gz
├── bin/
│   └── busybox → (musl-static)
├── sbin/
│   └── init → (simple PID 1)
├── dev/
│   ├── console
│   └── null
├── proc/
├── sys/
└── etc/
    └── passwd (optional)
```

**PID 1 (init)**:
```sh
#!/bin/sh
# /sbin/init

# Mount virtual filesystems
mount -t proc none /proc
mount -t sysfs none /sys
mount -t devtmpfs none /dev

# Start getty on console
/sbin/getty -L ttyAMA0 115200 vt100 &

# Wait for children (reaper)
while true; do
    wait -n || true
done
```

**Building Initramfs**:
```bash
# scripts/build_initramfs.sh
#!/bin/bash
set -e

INITRAMFS_DIR=build/initramfs
BUSYBOX_VERSION=1.36.1

# 1. Build musl-static BusyBox
wget https://busybox.net/downloads/busybox-${BUSYBOX_VERSION}.tar.bz2
tar xf busybox-${BUSYBOX_VERSION}.tar.bz2
cd busybox-${BUSYBOX_VERSION}
make defconfig
sed -i 's/# CONFIG_STATIC is not set/CONFIG_STATIC=y/' .config
make -j$(nproc)

# 2. Create directory structure
mkdir -p ${INITRAMFS_DIR}/{bin,sbin,dev,proc,sys,etc}
cp busybox ${INITRAMFS_DIR}/bin/
ln -s busybox ${INITRAMFS_DIR}/bin/sh

# 3. Create init script
cat > ${INITRAMFS_DIR}/sbin/init << 'EOF'
#!/bin/sh
mount -t proc none /proc
mount -t sysfs none /sys
mount -t devtmpfs none /dev
exec /bin/sh
EOF
chmod +x ${INITRAMFS_DIR}/sbin/init

# 4. Pack into cpio
cd ${INITRAMFS_DIR}
find . | cpio -o -H newc | gzip > ../initramfs.cpio.gz
```

### Interfaces

#### Boot Arguments

```bash
# QEMU command line
qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a57 \
  -m 1G \
  -kernel KERNEL.ELF \
  -initrd initramfs.cpio.gz \
  -append "init=/sbin/init earlycon console=ttyAMA0" \
  -nographic
```

#### /proc Filesystem

Essential entries:
- /proc/cpuinfo - CPU model, cores, features
- /proc/meminfo - MemTotal, MemFree, MemAvailable
- /proc/[pid]/cmdline - Command line (NULL-separated)
- /proc/[pid]/stat - Process statistics (space-separated)
- /proc/[pid]/status - Human-readable status
- /proc/mounts - Mounted filesystems
- /proc/uptime - System uptime

### Acceptance Tests

#### Test Suite Structure

```bash
# tests/phase_a/run_tests.sh
#!/bin/bash
set -e

QEMU_CMD="qemu-system-aarch64 -machine virt -m 1G -kernel KERNEL.ELF -initrd initramfs.cpio.gz -nographic"

# Test 1: Boot to Shell
test_boot() {
    expect << 'EOF'
spawn $QEMU_CMD
set timeout 30
expect "/ #" { send "echo BOOT_OK\r" }
expect "BOOT_OK" { exit 0 }
exit 1
EOF
}

# Test 2: Basic Commands
test_commands() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "ls /\r"
expect "bin" { }
send "echo hi\r"
expect "hi" { }
send "cat /proc/cpuinfo | head -n 5\r"
expect "processor" { }
send "ps\r"
expect "init" { }
exit 0
EOF
}

# Test 3: Fork/Exec
test_fork_exec() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "sh -c 'exit 42'; echo $?\r"
expect "42" { exit 0 }
exit 1
EOF
}

# Test 4: Pipes
test_pipes() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "yes | head -n 1 | wc -c\r"
expect "2" { exit 0 }
exit 1
EOF
}

# Test 5: Signals
test_signals() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "sleep 10 &\r"
expect {
    -re "\\[\\d+\\] (\\d+)" {
        set pid $expect_out(1,string)
        send "kill -TERM $pid\r"
    }
}
send "wait; echo $?\r"
expect "143" { exit 0 }  # 128 + 15 (SIGTERM)
exit 1
EOF
}

# Run all A1 tests
test_boot
test_commands
test_fork_exec
test_pipes
test_signals

echo "Phase A1: All tests passed ✓"
```

### A1 Exit Criteria

- ✅ Reproducibly boots to BusyBox prompt on virtio-console
- ✅ All 30 MVP syscalls implemented and functional
- ✅ fork/exec/wait create and reap processes correctly with COW
- ✅ Pipes, signals, basic TTY work as expected
- ✅ /proc (basic), /sys, /dev (console, null, zero, random) accessible
- ✅ All A1 acceptance tests pass in CI
- ✅ No kernel panics or deadlocks under normal workload
- ✅ Memory usage reasonable (fork doesn't OOM due to COW)

---

### Phase A2 — PTY + Full /proc (Week 4)

**Objective**: Add PTY support and complete procfs entries for full BusyBox compatibility.

#### 1. PTY (Pseudo-Terminal) Implementation

```rust
// crates/kernel/src/drivers/char/pty.rs

pub struct PtyMaster {
    pub index: usize,
    pub slave: Arc<Mutex<PtySlave>>,
    pub input_buffer: RingBuffer<u8, 4096>,  // From slave to master
    pub output_buffer: RingBuffer<u8, 4096>, // From master to slave
    pub lock: SpinLock<()>,
}

pub struct PtySlave {
    pub index: usize,
    pub master: Weak<Mutex<PtyMaster>>,
    pub termios: Termios,
    pub winsize: Winsize,
    pub pgrp: Option<Pid>,  // Controlling process group
}

impl PtyMaster {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, Errno> {
        self.input_buffer.read(buf)
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, Errno> {
        self.output_buffer.write(buf)
    }
}

impl PtySlave {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, Errno> {
        let master = self.master.upgrade().ok_or(Errno::EIO)?;
        master.lock().output_buffer.read(buf)
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, Errno> {
        let master = self.master.upgrade().ok_or(Errno::EIO)?;

        // Apply line discipline (echo, etc.)
        let processed = self.line_discipline(buf)?;

        master.lock().input_buffer.write(&processed)
    }
}

// Syscall: open /dev/ptmx
pub fn sys_open_ptmx() -> Result<(Arc<PtyMaster>, Arc<PtySlave>), Errno> {
    let index = alloc_pty_index();

    let slave = Arc::new(Mutex::new(PtySlave {
        index,
        master: Weak::new(),
        termios: Termios::default(),
        winsize: Winsize::default(),
        pgrp: None,
    }));

    let master = Arc::new(Mutex::new(PtyMaster {
        index,
        slave: slave.clone(),
        input_buffer: RingBuffer::new(),
        output_buffer: RingBuffer::new(),
        lock: SpinLock::new(()),
    }));

    // Set up bidirectional link
    slave.lock().master = Arc::downgrade(&master);

    // Create /dev/pts/N device node
    create_pts_device(index, slave.clone())?;

    Ok((master, slave))
}
```

**Device Nodes**:
- /dev/ptmx - PTY master clone device (major 5, minor 2)
- /dev/pts/0, /dev/pts/1, ... - PTY slaves (major 136, minor N)

#### 2. Full /proc Entries

```rust
// crates/kernel/src/vfs/procfs.rs

// /proc/[pid]/cmdline
pub fn proc_cmdline(pid: Pid) -> Result<Vec<u8>, Errno> {
    let proc = get_process(pid)?;
    let mut result = Vec::new();

    for arg in &proc.argv {
        result.extend_from_slice(arg.as_bytes());
        result.push(0);  // NULL-separated
    }

    Ok(result)
}

// /proc/[pid]/stat
pub fn proc_stat(pid: Pid) -> Result<String, Errno> {
    let proc = get_process(pid)?;

    Ok(format!(
        "{} ({}) {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}\n",
        proc.pid,
        proc.name,
        proc.state.as_char(),
        proc.parent.unwrap_or(0),
        proc.pgrp,
        proc.session,
        proc.tty_nr,
        proc.tpgid,
        proc.flags,
        proc.minflt,
        proc.cminflt,
        proc.majflt,
        proc.cmajflt,
        proc.utime,
        proc.stime,
        proc.cutime,
        proc.cstime,
        proc.priority,
        proc.nice,
        proc.num_threads,
        proc.itrealvalue,
        proc.starttime,
        proc.vsize,
        proc.rss,
    ))
}

// /proc/[pid]/maps
pub fn proc_maps(pid: Pid) -> Result<String, Errno> {
    let proc = get_process(pid)?;
    let mut result = String::new();

    for vma in &proc.address_space.vmas {
        result.push_str(&format!(
            "{:016x}-{:016x} {} {:08x} {:02x}:{:02x} {} {}\n",
            vma.start,
            vma.end,
            vma.perms_string(),
            vma.offset,
            vma.dev_major,
            vma.dev_minor,
            vma.inode,
            vma.name,
        ));
    }

    Ok(result)
}

// /proc/mounts
pub fn proc_mounts() -> Result<String, Errno> {
    let mounts = get_all_mounts();
    let mut result = String::new();

    for mnt in mounts {
        result.push_str(&format!(
            "{} {} {} {} 0 0\n",
            mnt.device,
            mnt.mountpoint,
            mnt.fstype,
            mnt.options,
        ));
    }

    Ok(result)
}

// /proc/uptime
pub fn proc_uptime() -> Result<String, Errno> {
    let uptime_secs = timer::get_uptime_secs();
    let idle_secs = timer::get_idle_secs();

    Ok(format!("{}.{:02} {}.{:02}\n",
        uptime_secs / 100, uptime_secs % 100,
        idle_secs / 100, idle_secs % 100))
}
```

#### 3. Enhanced Line Discipline

```rust
// Full termios support with all flags
pub struct Termios {
    pub c_iflag: tcflag_t,   // Input modes
    pub c_oflag: tcflag_t,   // Output modes
    pub c_cflag: tcflag_t,   // Control modes
    pub c_lflag: tcflag_t,   // Local modes
    pub c_cc: [cc_t; NCCS],  // Control characters
}

// LFLAG bits
const ECHO: tcflag_t = 0x00000008;
const ECHOE: tcflag_t = 0x00000010;
const ECHOK: tcflag_t = 0x00000020;
const ICANON: tcflag_t = 0x00000002;
const ISIG: tcflag_t = 0x00000001;

pub fn process_input(&mut self, byte: u8) -> Option<Vec<u8>> {
    if self.termios.c_lflag & ICANON != 0 {
        // Canonical mode (line buffered)
        match byte {
            b'\r' | b'\n' => {
                self.line_buffer.push(b'\n');
                let line = self.line_buffer.clone();
                self.line_buffer.clear();
                Some(line)
            }
            127 | 8 => {  // Backspace/DEL
                if !self.line_buffer.is_empty() {
                    self.line_buffer.pop();
                    if self.termios.c_lflag & ECHOE != 0 {
                        // Echo backspace sequence
                        return Some(vec![8, b' ', 8]);
                    }
                }
                None
            }
            3 => {  // Ctrl-C
                if self.termios.c_lflag & ISIG != 0 {
                    send_signal_to_fg_process_group(SIGINT);
                }
                None
            }
            _ => {
                self.line_buffer.push(byte);
                if self.termios.c_lflag & ECHO != 0 {
                    Some(vec![byte])
                } else {
                    None
                }
            }
        }
    } else {
        // Raw mode (immediate)
        Some(vec![byte])
    }
}
```

### A2 Acceptance Tests

```bash
# tests/phase_a2/run_tests.sh

# Test 1: PTY creation
test_pty_create() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "ls -l /dev/ptmx\r"
expect "crw" { exit 0 }
exit 1
EOF
}

# Test 2: PTY communication
test_pty() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "script -qc 'echo PTY OK' /dev/null\r"
expect "PTY OK" { exit 0 }
exit 1
EOF
}

# Test 3: /proc entries readable
test_proc_entries() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "cat /proc/self/cmdline | od -c\r"
expect "/bin/sh" { }
send "cat /proc/self/stat | awk '{print $1}'\r"
expect -re "\\d+" { }
send "cat /proc/self/maps | head -1\r"
expect -re "[0-9a-f]+-[0-9a-f]+" { }
send "cat /proc/mounts\r"
expect "proc /proc" { }
send "cat /proc/uptime\r"
expect -re "\\d+\\.\\d+" { exit 0 }
exit 1
EOF
}

# Test 4: BusyBox tools requiring PTY
test_busybox_tools() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "top -bn1 | head -5\r"
expect "Mem:" { }
send "vi -c ':q!' /tmp/test 2>&1\r"
expect "/ #" { exit 0 }
exit 1
EOF
}

test_pty_create
test_pty
test_proc_entries
test_busybox_tools

echo "Phase A2: All tests passed ✓"
```

### A2 Exit Criteria

- ✅ PTY creation and communication works (open /dev/ptmx, access /dev/pts/N)
- ✅ BusyBox tools requiring PTY work (script, top, vi)
- ✅ All /proc/[pid]/* entries readable and accurate
- ✅ /proc/mounts, /proc/uptime work correctly
- ✅ Line discipline handles all termios modes correctly
- ✅ All A2 acceptance tests pass in CI

### Phase A Artifacts (Combined A0+A1+A2)

1. **Kernel Binary**: KERNEL.ELF with full userspace support
2. **Initramfs**: initramfs.cpio.gz (BusyBox + init)
3. **Build Scripts**:
   - scripts/build_initramfs.sh
   - scripts/build_kernel_userspace.sh
4. **CI Jobs**:
   - .github/workflows/phase_a0.yml
   - .github/workflows/phase_a1.yml
   - .github/workflows/phase_a2.yml

---

## Phase B — Persistent Storage (Block + ext2 + Page Cache)

**Objective**: Read/write a persistent filesystem with caching; survive reboot.

**Timeline**: 2 weeks

### Scope

Add block device support, ext2 filesystem, and page/buffer cache for performant persistent storage across reboots. **Critical**: Add page cache in Phase B, not later—ext2 without caching is unusably slow.

### Implementation Details

#### 1. Block Layer

**Architecture**:
```
┌──────────────┐
│  VFS Layer   │
└──────┬───────┘
       │ read/write inodes
       ▼
┌──────────────┐
│ ext2 Driver  │
└──────┬───────┘
       │ block I/O
       ▼
┌──────────────┐
│ Block Layer  │ ← Request queues, BIO
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ virtio-blk   │ ← Driver
└──────────────┘
```

**Key Structures**:

```rust
// crates/kernel/src/block/mod.rs
pub struct BlockDevice {
    pub name: String,
    pub major: u32,
    pub minor: u32,
    pub capacity_sectors: u64,
    pub sector_size: usize,
    pub ops: &'static BlockDeviceOps,
}

pub trait BlockDeviceOps {
    fn read_sectors(&self, sector: u64, buf: &mut [u8]) -> Result<()>;
    fn write_sectors(&self, sector: u64, buf: &[u8]) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

pub struct BlockRequest {
    pub device: Arc<BlockDevice>,
    pub operation: BlockOp,
    pub sector: u64,
    pub buffer: Vec<u8>,
    pub completion: Option<Waker>,
}

pub enum BlockOp {
    Read,
    Write,
    Flush,
}
```

**Request Queue** (simple FIFO for Phase B):
```rust
pub struct RequestQueue {
    pending: VecDeque<BlockRequest>,
    in_flight: Option<BlockRequest>,
    wakers: Vec<Waker>,
}
```

#### 2. virtio-blk Driver

**Virtio Spec**: Version 1.0, Device ID 2

**Configuration Space**:
```rust
#[repr(C)]
pub struct VirtioBlkConfig {
    capacity: u64,      // Sectors (512 bytes each)
    size_max: u32,      // Max segment size
    seg_max: u32,       // Max segments
    // ...
}
```

**Request Format**:
```rust
#[repr(C)]
pub struct VirtioBlkReq {
    req_type: u32,      // VIRTIO_BLK_T_IN (0) or T_OUT (1)
    reserved: u32,
    sector: u64,
}
```

**Driver Implementation**:
```rust
// crates/kernel/src/drivers/virtio_blk.rs
pub struct VirtioBlkDevice {
    common: VirtioCommonCfg,
    queue: VirtQueue,
    config: *mut VirtioBlkConfig,
    capacity: u64,
}

impl VirtioBlkDevice {
    pub fn new(base: usize) -> Result<Self> {
        // 1. Probe PCI device
        // 2. Initialize virtqueues
        // 3. Read capacity from config space
        // 4. Register as block device
    }

    fn submit_request(&self, req: BlockRequest) -> Result<()> {
        // 1. Allocate descriptor chain
        // 2. Fill virtio_blk_req header
        // 3. Attach data buffer
        // 4. Kick queue
        // 5. Wait for interrupt
    }
}
```

#### 3. Partitioning (MBR/GPT)

**MBR** (Master Boot Record):
```rust
#[repr(C)]
pub struct MbrEntry {
    status: u8,
    first_chs: [u8; 3],
    partition_type: u8,
    last_chs: [u8; 3],
    first_lba: u32,
    sector_count: u32,
}
```

**GPT** (GUID Partition Table):
```rust
pub struct GptHeader {
    signature: [u8; 8],     // "EFI PART"
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
    current_lba: u64,
    backup_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    disk_guid: [u8; 16],
    partition_entries_lba: u64,
    num_partition_entries: u32,
    partition_entry_size: u32,
    partition_entries_crc32: u32,
}
```

**Partition Detection**:
```rust
pub fn probe_partitions(dev: &BlockDevice) -> Result<Vec<Partition>> {
    let mut sector0 = vec![0u8; 512];
    dev.read_sectors(0, &mut sector0)?;

    // Check GPT signature first
    if &sector0[0..8] == b"EFI PART" {
        return parse_gpt(dev);
    }

    // Fallback to MBR
    if sector0[510] == 0x55 && sector0[511] == 0xAA {
        return parse_mbr(dev, &sector0);
    }

    Err(Error::NoPartitionTable)
}
```

#### 4. Page/Buffer Cache **[NEW - Critical for Performance]**

**Rationale**: ext2 without caching makes every read/write hit the disk—unbearably slow. Add simple LRU cache now.

```rust
// crates/kernel/src/mm/page_cache.rs

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

pub struct PageCache {
    cache: Mutex<LruCache<CacheKey, Arc<BufferHead>>>,
    total_pages: AtomicU64,
    dirty_pages: Mutex<Vec<Arc<BufferHead>>>,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct CacheKey {
    pub device_id: DeviceId,
    pub block_num: u64,
}

pub struct BufferHead {
    pub device_id: DeviceId,
    pub block_num: u64,
    pub data: Box<[u8]>,     // 4KB block
    pub dirty: AtomicBool,
    pub locked: AtomicBool,
    pub uptodate: AtomicBool,
}

static PAGE_CACHE: Once<PageCache> = Once::new();

impl PageCache {
    const MAX_CACHED_PAGES: usize = 16384;  // 64MB cache (16K * 4KB)

    pub fn global() -> &'static PageCache {
        PAGE_CACHE.call_once(|| PageCache {
            cache: Mutex::new(LruCache::new(Self::MAX_CACHED_PAGES)),
            total_pages: AtomicU64::new(0),
            dirty_pages: Mutex::new(Vec::new()),
        })
    }

    /// Read block (from cache or disk)
    pub fn read_block(&self, dev: DeviceId, block: u64) -> Result<Arc<BufferHead>, Errno> {
        let key = CacheKey { device_id: dev, block_num: block };

        // Try cache first
        {
            let mut cache = self.cache.lock();
            if let Some(bh) = cache.get(&key) {
                return Ok(bh.clone());
            }
        }

        // Cache miss - read from disk
        let device = get_block_device(dev)?;
        let mut data = vec![0u8; device.block_size()];
        device.read_blocks(block, &mut data)?;

        let bh = Arc::new(BufferHead {
            device_id: dev,
            block_num: block,
            data: data.into_boxed_slice(),
            dirty: AtomicBool::new(false),
            locked: AtomicBool::new(false),
            uptodate: AtomicBool::new(true),
        });

        // Insert into cache
        let mut cache = self.cache.lock();
        cache.insert(key, bh.clone());
        self.total_pages.fetch_add(1, Ordering::Relaxed);

        Ok(bh)
    }

    /// Write block (mark dirty, defer actual write)
    pub fn write_block(&self, dev: DeviceId, block: u64, data: &[u8]) -> Result<(), Errno> {
        let key = CacheKey { device_id: dev, block_num: block };

        let bh = {
            let mut cache = self.cache.lock();
            if let Some(bh) = cache.get(&key) {
                bh.clone()
            } else {
                // Not in cache - allocate new
                let bh = Arc::new(BufferHead {
                    device_id: dev,
                    block_num: block,
                    data: data.to_vec().into_boxed_slice(),
                    dirty: AtomicBool::new(true),
                    locked: AtomicBool::new(false),
                    uptodate: AtomicBool::new(true),
                });
                cache.insert(key, bh.clone());
                bh
            }
        };

        // Update data and mark dirty
        unsafe {
            let bh_data = &bh.data as *const [u8] as *mut [u8];
            (*bh_data).copy_from_slice(data);
        }
        bh.dirty.store(true, Ordering::Release);

        // Add to dirty list
        self.dirty_pages.lock().push(bh);

        Ok(())
    }

    /// Flush all dirty blocks to disk
    pub fn sync_all(&self) -> Result<(), Errno> {
        let dirty = {
            let mut list = self.dirty_pages.lock();
            core::mem::take(&mut *list)
        };

        for bh in dirty {
            if !bh.dirty.load(Ordering::Acquire) {
                continue;
            }

            let device = get_block_device(bh.device_id)?;
            device.write_blocks(bh.block_num, &bh.data)?;
            bh.dirty.store(false, Ordering::Release);
        }

        Ok(())
    }

    /// Flush dirty blocks for specific device
    pub fn sync_device(&self, dev: DeviceId) -> Result<(), Errno> {
        let dirty = self.dirty_pages.lock();

        for bh in dirty.iter() {
            if bh.device_id != dev {
                continue;
            }

            if !bh.dirty.load(Ordering::Acquire) {
                continue;
            }

            let device = get_block_device(bh.device_id)?;
            device.write_blocks(bh.block_num, &bh.data)?;
            bh.dirty.store(false, Ordering::Release);
        }

        Ok(())
    }
}

// Syscall: sync (flush all dirty pages)
pub fn sys_sync() -> Result<isize, Errno> {
    PageCache::global().sync_all()?;

    // Also sync open file metadata
    for proc in all_processes() {
        for fd in &proc.lock().open_files {
            fd.sync()?;
        }
    }

    Ok(0)
}
```

**LRU Implementation** (simple CLOCK or proper LRU):
```rust
use alloc::collections::VecDeque;

pub struct LruCache<K, V> {
    map: HashMap<K, V>,
    order: VecDeque<K>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            // Move to back (most recently used)
            self.order.retain(|k| k != key);
            self.order.push_back(key.clone());
            self.map.get(key)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.map.len() >= self.capacity {
            // Evict LRU (front of queue)
            if let Some(old_key) = self.order.pop_front() {
                self.map.remove(&old_key);
            }
        }

        self.map.insert(key.clone(), value);
        self.order.push_back(key);
    }
}
```

**Integration with ext2**:
```rust
impl Ext2Filesystem {
    pub fn read_block(&self, block_num: u64) -> Result<Arc<BufferHead>, Errno> {
        PageCache::global().read_block(self.device.id(), block_num)
    }

    pub fn write_block(&self, block_num: u64, data: &[u8]) -> Result<(), Errno> {
        PageCache::global().write_block(self.device.id(), block_num, data)
    }
}
```

#### 5. ext2 Filesystem

**On-Disk Layout**:
```
Block 0:     Boot sector (unused)
Block 1:     Superblock
Block 2:     Block Group Descriptor Table
Block 3+:    Block bitmap, Inode bitmap, Inode table, Data blocks
```

**Superblock**:
```rust
#[repr(C)]
pub struct Ext2Superblock {
    inodes_count: u32,
    blocks_count: u32,
    r_blocks_count: u32,
    free_blocks_count: u32,
    free_inodes_count: u32,
    first_data_block: u32,
    log_block_size: u32,        // Block size = 1024 << log_block_size
    log_frag_size: u32,
    blocks_per_group: u32,
    frags_per_group: u32,
    inodes_per_group: u32,
    mtime: u32,
    wtime: u32,
    mnt_count: u16,
    max_mnt_count: u16,
    magic: u16,                 // 0xEF53
    state: u16,
    errors: u16,
    // ... more fields
}
```

**Inode**:
```rust
#[repr(C)]
pub struct Ext2Inode {
    mode: u16,
    uid: u16,
    size: u32,
    atime: u32,
    ctime: u32,
    mtime: u32,
    dtime: u32,
    gid: u16,
    links_count: u16,
    blocks: u32,
    flags: u32,
    block: [u32; 15],           // Direct + indirect pointers
    generation: u32,
    file_acl: u32,
    dir_acl: u32,
    faddr: u32,
}
```

**Directory Entry**:
```rust
#[repr(C)]
pub struct Ext2DirEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: u8,
    // name follows (variable length)
}
```

**Operations**:

```rust
// crates/kernel/src/fs/ext2/mod.rs
pub struct Ext2Filesystem {
    device: Arc<BlockDevice>,
    superblock: Ext2Superblock,
    block_size: usize,
    groups: Vec<BlockGroupDescriptor>,
}

impl Ext2Filesystem {
    pub fn mount(dev: Arc<BlockDevice>) -> Result<Self> {
        // 1. Read superblock at block 1
        // 2. Validate magic 0xEF53
        // 3. Calculate block size
        // 4. Read block group descriptors
        // 5. Load root inode (inode 2)
    }

    pub fn read_inode(&self, ino: u32) -> Result<Ext2Inode> {
        // 1. Calculate block group
        // 2. Find inode table offset
        // 3. Read inode from disk
    }

    pub fn read_file(&self, inode: &Ext2Inode, offset: u64, buf: &mut [u8]) -> Result<usize> {
        // 1. Calculate block index
        // 2. Handle direct/indirect/double-indirect blocks
        // 3. Read data blocks
        // 4. Copy to buffer
    }

    pub fn write_file(&mut self, inode: &mut Ext2Inode, offset: u64, buf: &[u8]) -> Result<usize> {
        // 1. Allocate blocks if needed
        // 2. Update block pointers
        // 3. Write data blocks
        // 4. Update inode size/mtime
    }

    pub fn lookup(&self, dir_inode: &Ext2Inode, name: &str) -> Result<u32> {
        // 1. Read directory blocks
        // 2. Parse directory entries
        // 3. Match name
        // 4. Return inode number
    }
}
```

#### 5. Root Mount

**Boot Sequence**:
```
1. Kernel boots with initramfs
2. initramfs init mounts ext2 root:
   mount -t ext2 /dev/vda1 /mnt
3. Switch root:
   exec switch_root /mnt /sbin/init
4. Real init continues boot
```

**Alternative**: Direct root mount (no initramfs):
```bash
# Boot args:
root=/dev/vda1 rootfstype=ext2 rw init=/sbin/init
```

### Interfaces

#### mount(2) Syscall

```rust
pub fn sys_mount(
    source: *const u8,      // "/dev/vda1"
    target: *const u8,      // "/mnt"
    fstype: *const u8,      // "ext2"
    flags: u64,             // MS_RDONLY, MS_NOATIME, etc.
    data: *const u8,        // Options
) -> Result<isize>;
```

#### /proc/mounts

```
/dev/vda1 / ext2 rw,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec 0 0
sysfs /sys sysfs rw,nosuid,nodev,noexec 0 0
devtmpfs /dev devtmpfs rw,nosuid 0 0
```

### Acceptance Tests

```bash
# tests/phase_b/run_tests.sh
#!/bin/bash
set -e

# Create ext2 disk image (from host)
dd if=/dev/zero of=disk.img bs=1M count=128
mkfs.ext2 -F disk.img
mkdir -p mnt
sudo mount -o loop disk.img mnt
echo "hello persistent world" > mnt/test.txt
sudo umount mnt

# Test 1: Mount and read
test_mount_read() {
    expect << 'EOF'
spawn qemu-system-aarch64 ... -drive file=disk.img,format=raw,if=none,id=hd -device virtio-blk,drive=hd
expect "/ #"
send "mount -t ext2 /dev/vda /mnt\r"
expect "/ #"
send "cat /mnt/test.txt\r"
expect "hello persistent world" { exit 0 }
exit 1
EOF
}

# Test 2: Write and persist
test_write_persist() {
    # Boot 1: Write
    expect << 'EOF'
spawn qemu...
expect "/ #"
send "mount -t ext2 /dev/vda /mnt\r"
send "echo 'boot 1' > /mnt/persist.txt\r"
send "sync\r"
send "poweroff\r"
expect eof
EOF

    # Boot 2: Verify
    expect << 'EOF'
spawn qemu...
expect "/ #"
send "mount -t ext2 /dev/vda /mnt\r"
send "cat /mnt/persist.txt\r"
expect "boot 1" { exit 0 }
exit 1
EOF
}

# Test 3: Large file
test_large_file() {
    expect << 'EOF'
spawn qemu...
expect "/ #"
send "mount -t ext2 /dev/vda /mnt\r"
send "dd if=/dev/zero of=/mnt/big bs=1M count=64\r"
send "sync\r"
send "ls -lh /mnt/big\r"
expect "64M" { exit 0 }
exit 1
EOF
}

# Test 4: File operations
test_file_ops() {
    expect << 'EOF'
spawn qemu...
expect "/ #"
send "mount -t ext2 /dev/vda /mnt\r"
send "mkdir /mnt/testdir\r"
send "touch /mnt/testdir/file1\r"
send "ln -s file1 /mnt/testdir/link1\r"
send "ls -la /mnt/testdir/\r"
expect "link1 -> file1" { }
send "mv /mnt/testdir/file1 /mnt/testdir/file2\r"
send "rm /mnt/testdir/link1\r"
send "ls /mnt/testdir/\r"
expect "file2" { exit 0 }
exit 1
EOF
}

test_mount_read
test_write_persist
test_large_file
test_file_ops

echo "Phase B: All tests passed ✓"
```

### Artifacts

1. **Kernel**: virtio-blk driver + ext2 filesystem
2. **Disk Image**: scripts/create_ext2_disk.sh
3. **CI Job**: .github/workflows/phase_b.yml
4. **Documentation**: docs/filesystems/ext2.md

### Exit Criteria

- ✅ virtio-blk driver initializes and detects capacity
- ✅ MBR/GPT partitioning detected correctly
- ✅ ext2 filesystem mounts successfully
- ✅ Files persist across reboot
- ✅ Large files (128MB+) work correctly
- ✅ File operations (create/unlink/rename/symlink) work
- ✅ No data corruption or panics
- ✅ All acceptance tests pass in CI

---

## Phase C — Networking (virtio-net + smoltcp + Sockets)

**Objective**: Bring up network, get IP via DHCP, resolve DNS, and fetch HTTP.

**Timeline**: 1-2 weeks (using smoltcp saves 2-3 weeks vs custom TCP/IP)

### Scope

Add full networking stack with virtio-net driver, **smoltcp integration**, sockets API, and network protocols.

**Decision**: Use smoltcp (embedded Rust TCP/IP stack) to avoid 4-6 weeks of custom TCP implementation and subtle bugs. Integrate via custom PHY layer over virtio-net. **No kernel async/await**—use blocking I/O with state machines.

### Implementation Details

#### 1. virtio-net Driver

**Virtio Spec**: Version 1.0, Device ID 1

**Configuration Space**:
```rust
#[repr(C)]
pub struct VirtioNetConfig {
    mac: [u8; 6],
    status: u16,
    max_virtqueue_pairs: u16,
    mtu: u16,
}
```

**Virtqueues**:
- RX queue (index 0): Receive packets
- TX queue (index 1): Transmit packets

**Packet Format**:
```rust
#[repr(C)]
pub struct VirtioNetHdr {
    flags: u8,
    gso_type: u8,
    hdr_len: u16,
    gso_size: u16,
    csum_start: u16,
    csum_offset: u16,
    num_buffers: u16,
}
```

**Driver Implementation**:
```rust
// crates/kernel/src/drivers/virtio_net.rs
pub struct VirtioNetDevice {
    common: VirtioCommonCfg,
    rx_queue: VirtQueue,
    tx_queue: VirtQueue,
    mac: [u8; 6],
    mtu: u16,
    rx_buffers: VecDeque<Vec<u8>>,
}

impl VirtioNetDevice {
    pub fn new(base: usize) -> Result<Self> {
        // 1. Initialize virtqueues
        // 2. Read MAC address
        // 3. Pre-fill RX queue with buffers
        // 4. Enable interrupts
    }

    pub fn transmit(&self, packet: &[u8]) -> Result<()> {
        // 1. Allocate TX descriptor
        // 2. Copy packet data
        // 3. Add virtio_net_hdr
        // 4. Kick TX queue
    }

    pub fn receive(&mut self) -> Option<Vec<u8>> {
        // 1. Check RX queue for used descriptors
        // 2. Extract packet data
        // 3. Refill RX buffer
        // 4. Return packet
    }
}
```

#### 2. Network Stack Architecture

**Layers**:
```
┌──────────────────┐
│  Socket Layer    │ ← BSD sockets API
└────────┬─────────┘
         │
┌────────▼─────────┐
│   TCP / UDP      │ ← Transport layer
└────────┬─────────┘
         │
┌────────▼─────────┐
│      IPv4        │ ← Network layer
└────────┬─────────┘
         │
┌────────▼─────────┐
│ ARP / ICMP       │ ← Helper protocols
└────────┬─────────┘
         │
┌────────▼─────────┐
│   Ethernet       │ ← Link layer
└────────┬─────────┘
         │
┌────────▼─────────┐
│   virtio-net     │ ← Driver
└──────────────────┘
```

**Selected Approach**: **smoltcp Integration** (saves 3-4 weeks, battle-tested)

```toml
# Cargo.toml
[dependencies]
smoltcp = { version = "0.11", default-features = false, features = [
    "proto-ipv4",
    "proto-dhcpv4",
    "socket-tcp",
    "socket-udp",
    "socket-icmp",
    "socket-raw",
    "medium-ethernet",
] }
```

**Integration via Custom PHY**:
```rust
// crates/kernel/src/net/smoltcp_iface.rs

use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken};
use smoltcp::time::Instant;

pub struct VirtioNetPhy {
    device: Arc<Mutex<VirtioNetDevice>>,
}

impl Device for VirtioNetPhy {
    type RxToken<'a> = VirtioRxToken<'a>;
    type TxToken<'a> = VirtioTxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut dev = self.device.lock();

        if let Some(packet) = dev.receive() {
            Some((
                VirtioRxToken { packet },
                VirtioTxToken { device: &mut dev },
            ))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(VirtioTxToken { device: &mut *self.device.lock() })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;  // Ethernet MTU + header
        caps.max_burst_size = Some(256);
        caps
    }
}

pub struct VirtioRxToken { packet: Vec<u8> }
pub struct VirtioTxToken<'a> { device: &'a mut VirtioNetDevice }

impl RxToken for VirtioRxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where F: FnOnce(&mut [u8]) -> R {
        f(&mut self.packet)
    }
}

impl TxToken for VirtioTxToken<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where F: FnOnce(&mut [u8]) -> R {
        let mut buffer = vec![0u8; len];
        let result = f(&mut buffer);
        self.device.transmit(&buffer).expect("TX failed");
        result
    }
}

// Create smoltcp interface
pub fn create_interface() -> Result<smoltcp::iface::Interface, Errno> {
    let device = VirtioNetPhy { device: VIRTIO_NET_DEVICE.clone() };

    let config = smoltcp::iface::Config::new(
        smoltcp::wire::HardwareAddress::Ethernet(
            smoltcp::wire::EthernetAddress(VIRTIO_NET_DEVICE.lock().mac)
        )
    );

    let mut interface = smoltcp::iface::Interface::new(config, &mut device);

    // Add IPv4 address (will be replaced by DHCP)
    interface.update_ip_addrs(|addrs| {
        addrs.push(smoltcp::wire::IpCidr::new(
            smoltcp::wire::IpAddress::v4(0, 0, 0, 0),
            0
        )).ok();
    });

    Ok(interface)
}
```

#### 3. Socket API (BSD Sockets)

**Syscalls** (16 total):

| Syscall | Number | Purpose |
|---------|--------|---------|
| socket | 198 | Create socket |
| bind | 200 | Bind address |
| listen | 201 | Listen for connections |
| accept | 202 | Accept connection |
| connect | 203 | Connect to address |
| send/sendto | 206/207 | Send data |
| recv/recvfrom | 207/208 | Receive data |
| shutdown | 210 | Shutdown socket |
| getsockopt | 209 | Get socket option |
| setsockopt | 208 | Set socket option |

**Socket Structure**:
```rust
// crates/kernel/src/net/socket.rs
pub struct Socket {
    pub domain: AddressFamily,      // AF_INET, AF_INET6
    pub sock_type: SocketType,      // SOCK_STREAM, SOCK_DGRAM
    pub protocol: Protocol,         // IPPROTO_TCP, IPPROTO_UDP
    pub state: SocketState,
    pub local_addr: Option<SocketAddr>,
    pub remote_addr: Option<SocketAddr>,
    pub recv_queue: VecDeque<Vec<u8>>,
    pub send_queue: VecDeque<Vec<u8>>,
    pub backlog: Option<VecDeque<Socket>>,  // For listening sockets
}

pub enum SocketState {
    Unbound,
    Bound,
    Listening,
    Connecting,
    Connected,
    Closing,
    Closed,
}
```

**Usage Example** (from userspace):
```c
// TCP client
int sock = socket(AF_INET, SOCK_STREAM, 0);
struct sockaddr_in addr = {
    .sin_family = AF_INET,
    .sin_port = htons(80),
    .sin_addr.s_addr = inet_addr("93.184.216.34"),  // example.com
};
connect(sock, (struct sockaddr*)&addr, sizeof(addr));
send(sock, "GET / HTTP/1.0\r\n\r\n", 18, 0);
char buf[1024];
recv(sock, buf, sizeof(buf), 0);
close(sock);
```

#### 4. TCP/IP Implementation (Minimal)

**IPv4**:
```rust
#[repr(C)]
pub struct Ipv4Header {
    version_ihl: u8,        // Version (4 bits) + IHL (4 bits)
    dscp_ecn: u8,
    total_length: u16be,
    identification: u16be,
    flags_fragment: u16be,
    ttl: u8,
    protocol: u8,           // 1=ICMP, 6=TCP, 17=UDP
    checksum: u16be,
    src_addr: [u8; 4],
    dst_addr: [u8; 4],
}
```

**TCP**:
```rust
#[repr(C)]
pub struct TcpHeader {
    src_port: u16be,
    dst_port: u16be,
    seq_num: u32be,
    ack_num: u32be,
    data_offset_flags: u16be,   // Data offset (4 bits) + flags (12 bits)
    window_size: u16be,
    checksum: u16be,
    urgent_pointer: u16be,
}

pub struct TcpConnection {
    state: TcpState,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    send_seq: u32,
    recv_seq: u32,
    window: u16,
    // Retransmission queue, timers, etc.
}

pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}
```

**UDP**:
```rust
#[repr(C)]
pub struct UdpHeader {
    src_port: u16be,
    dst_port: u16be,
    length: u16be,
    checksum: u16be,
}
```

**ARP** (Address Resolution Protocol):
```rust
#[repr(C)]
pub struct ArpPacket {
    hardware_type: u16be,   // 1 = Ethernet
    protocol_type: u16be,   // 0x0800 = IPv4
    hardware_len: u8,       // 6 (MAC)
    protocol_len: u8,       // 4 (IPv4)
    operation: u16be,       // 1 = Request, 2 = Reply
    sender_mac: [u8; 6],
    sender_ip: [u8; 4],
    target_mac: [u8; 6],
    target_ip: [u8; 4],
}

pub struct ArpCache {
    entries: HashMap<Ipv4Addr, MacAddr>,
    pending: HashMap<Ipv4Addr, Vec<Waker>>,
}
```

**ICMP** (Internet Control Message Protocol):
```rust
#[repr(C)]
pub struct IcmpHeader {
    icmp_type: u8,          // 0 = Echo Reply, 8 = Echo Request
    code: u8,
    checksum: u16be,
    rest_of_header: [u8; 4],
}
```

#### 5. DHCP Client

**DHCP Message**:
```rust
#[repr(C)]
pub struct DhcpMessage {
    op: u8,                 // 1 = Request, 2 = Reply
    htype: u8,              // 1 = Ethernet
    hlen: u8,               // 6 (MAC length)
    hops: u8,
    xid: u32be,             // Transaction ID
    secs: u16be,
    flags: u16be,
    ciaddr: [u8; 4],        // Client IP
    yiaddr: [u8; 4],        // Your IP
    siaddr: [u8; 4],        // Server IP
    giaddr: [u8; 4],        // Gateway IP
    chaddr: [u8; 16],       // Client MAC
    sname: [u8; 64],
    file: [u8; 128],
    options: [u8; 312],     // DHCP options
}
```

**DHCP Process** (DORA):
1. **Discover**: Client broadcasts DHCPDISCOVER
2. **Offer**: Server responds with DHCPOFFER (IP address)
3. **Request**: Client requests offered IP with DHCPREQUEST
4. **Acknowledge**: Server confirms with DHCPACK

**Implementation**:
```rust
pub async fn dhcp_acquire() -> Result<NetworkConfig> {
    // 1. Send DHCPDISCOVER to 255.255.255.255:67
    // 2. Wait for DHCPOFFER
    // 3. Send DHCPREQUEST
    // 4. Wait for DHCPACK
    // 5. Configure interface with assigned IP
    // 6. Set default gateway and DNS servers
}
```

#### 6. DNS Resolver

**DNS Query**:
```rust
#[repr(C)]
pub struct DnsHeader {
    id: u16be,
    flags: u16be,
    qdcount: u16be,         // Questions
    ancount: u16be,         // Answers
    nscount: u16be,         // Authority RRs
    arcount: u16be,         // Additional RRs
}

pub async fn dns_resolve(hostname: &str) -> Result<Ipv4Addr> {
    // 1. Parse /etc/resolv.conf for nameserver
    // 2. Encode DNS query (type A, class IN)
    // 3. Send UDP packet to nameserver:53
    // 4. Parse response
    // 5. Return first A record
}
```

### Interfaces

#### Network Configuration

**ifconfig** (BusyBox):
```bash
ifconfig eth0 up
dhclient eth0  # Or built-in DHCP
ifconfig eth0
# eth0      Link encap:Ethernet  HWaddr 52:54:00:12:34:56
#           inet addr:10.0.2.15  Bcast:10.0.2.255  Mask:255.255.255.0
```

**/etc/resolv.conf**:
```
nameserver 1.1.1.1
nameserver 8.8.8.8
```

**Routing**:
```bash
route -n
# Kernel IP routing table
# Destination     Gateway         Genmask         Flags Metric Ref    Use Iface
# 0.0.0.0         10.0.2.2        0.0.0.0         UG    0      0        0 eth0
# 10.0.2.0        0.0.0.0         255.255.255.0   U     0      0        0 eth0
```

### Acceptance Tests

```bash
# tests/phase_c/run_tests.sh
#!/bin/bash
set -e

QEMU_CMD="qemu-system-aarch64 ... -netdev user,id=net0 -device virtio-net,netdev=net0"

# Test 1: Link Up + DHCP
test_link_dhcp() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "ip link set eth0 up\r"
send "dhclient eth0\r"
expect "Lease of" { }
send "ip addr show eth0\r"
expect -re "inet (\\d+\\.\\d+\\.\\d+\\.\\d+)" { exit 0 }
exit 1
EOF
}

# Test 2: ICMP Ping
test_ping() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "ping -c1 1.1.1.1\r"
expect "1 packets received" { exit 0 }
exit 1
EOF
}

# Test 3: DNS Resolution
test_dns() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "ping -c1 example.com\r"
expect "PING example.com" { }
expect "1 packets received" { exit 0 }
exit 1
EOF
}

# Test 4: HTTP Fetch
test_http() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "wget -O - http://example.com 2>/dev/null | head -n 1\r"
expect "<!doctype html>" { exit 0 }
exit 1
EOF
}

# Test 5: Socket API (echo server/client)
test_socket() {
    # Start echo server in background
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "nc -l -p 1234 &\r"
send "echo 'test' | nc 127.0.0.1 1234\r"
expect "test" { exit 0 }
exit 1
EOF
}

test_link_dhcp
test_ping
test_dns
test_http
test_socket

echo "Phase C: All tests passed ✓"
```

### Artifacts

1. **Kernel**: virtio-net + TCP/IP stack + sockets
2. **Network Tools**: wget, ping, nc (from BusyBox)
3. **CI Job**: .github/workflows/phase_c.yml
4. **Documentation**: docs/networking/tcpip.md

### Exit Criteria

- ✅ virtio-net driver initializes and detects link
- ✅ DHCP client acquires IP address
- ✅ Ping to IP address works (ICMP)
- ✅ DNS resolution works
- ✅ HTTP fetch returns HTML
- ✅ Socket API (connect/send/recv) functional
- ✅ No packet loss or stack panics
- ✅ All acceptance tests pass in CI

---

## Phase D — Security & Memory Protections

**Objective**: Enforce permissions and memory safety basics; provide entropy.

**Timeline**: 1–2 weeks

### Scope
Implement a minimal Unix security model (UID/GID/perms) and core memory protections (NX/W^X, ASLR, COW), plus an entropy source.

### Implementation Details

#### 1. Credentials & Permission Checks

```rust
// crates/kernel/src/security/cred.rs
pub struct Credentials { pub uid: u32, pub gid: u32, pub groups: SmallVec<[u32; 8]> }

pub fn inode_permission(cred: &Credentials, inode: &Inode, req: Perm) -> bool {
    // 1) Owner match → use owner bits, else 2) group match → group bits, else 3) other bits
}
```

Syscalls to complete: chmod/chown/umask; getuid/geteuid/getgid/getegid; (setuid/setgid optional).

#### 2. Entropy: /dev/urandom
Kernel PRNG seeded from jitter/time counters; expose as /dev/urandom; nonblocking for MVP.

#### 3. Memory Protections
- NX/W^X in PT_LOAD and `mmap/mprotect`.
- `mprotect(PROT_*)` support; TLB flush.
- ASLR: randomize stack/mmap base; PIE optional later.
- Guard pages below/above stack.

#### 4. Copy‑on‑Write (COW)
Fork copies PTEs RO; on write fault allocate new page, copy, update PTE, manage refcounts.

#### 5. AArch64 Syscall ABI & uaccess
Entry via SVC to EL1; args in x0..x5, nr in x8, retval in x0 (negative errno). `copy_{from,to}_user` with fault handling; validate user pointers.

### Acceptance Tests
- Perms: other user cannot write 0644 file; chmod 0666 enables; umask applied.
- NX/W^X: RWX mapping exec blocked.
- ASLR: varying heap/stack/exe base.
- COW: fork cost low; parent data unchanged after child writes.
- /dev/urandom yields random bytes.

### Exit Criteria
- ✅ Permissions enforced; NX/W^X active; ASLR/COW working; CI green.

---

## Phase E — SMP & Performance

**Objective**: Multi‑core execution with preemptive scheduling and basic perf observability.

**Timeline**: 1–2 weeks

### Scope
- PSCI secondary bring‑up; per‑CPU data; IPIs.
- Scheduler: per‑CPU runqueues; timeslice; load balancing; IRQ affinity.
- Timers: per‑CPU EL1 physical timer PPI 30.

### Acceptance Tests
- Boot with 2–4 vCPUs; `stress -c 4` uses all cores; system responsive.
- Affinity respected; context switch latency within budget.

### Exit Criteria
- ✅ SMP stable; scheduler balances; CI soak passes.

---

## Phase F — Journaling & Resilience (ext4)

**Objective**: ext4 journaling (ordered mode) + crash recovery.

**Timeline**: 1–2 weeks

### Acceptance Tests
- Forced crash → journal replay on mount; data intact.
- Throughput consistent (no extreme variance) on sequential writes.

### Exit Criteria
- ✅ ext4 mounts and replays journal; crash/recovery CI passes.

---

---

## Cross-Cutting Engineering

**Note**: These components must be implemented early (during or before Phase A0) as they're used throughout all phases.

### Memory Allocator

#### Physical Memory (Buddy Allocator)

```rust
// crates/kernel/src/mm/buddy.rs

pub struct BuddyAllocator {
    free_lists: [LinkedList<*mut PageFrame>; MAX_ORDER],  // Order 0-11 (4KB to 8MB)
    zones: Vec<Zone>,
}

pub struct Zone {
    start_pfn: usize,
    end_pfn: usize,
    free_pages: usize,
}

pub struct PageFrame {
    pfn: usize,
    order: u8,
    flags: PageFlags,
    refcount: AtomicU32,
    next: Option<*mut PageFrame>,
}

impl BuddyAllocator {
    pub fn alloc_pages(&mut self, order: u8) -> Result<PhysAddr, Errno> {
        // 1. Try to find free block of requested order
        if let Some(page) = self.free_lists[order as usize].pop_front() {
            return Ok(pfn_to_phys(unsafe { (*page).pfn }));
        }

        // 2. Split larger block
        for higher_order in (order + 1)..MAX_ORDER {
            if let Some(page) = self.free_lists[higher_order as usize].pop_front() {
                return Ok(self.split_block(page, higher_order, order));
            }
        }

        Err(Errno::ENOMEM)
    }

    pub fn free_pages(&mut self, addr: PhysAddr, order: u8) {
        let pfn = phys_to_pfn(addr);
        let buddy_pfn = pfn ^ (1 << order);

        // Try to coalesce with buddy
        if let Some(buddy) = self.find_buddy(buddy_pfn, order) {
            self.free_lists[order as usize].remove(buddy);
            let merged_pfn = pfn.min(buddy_pfn);
            self.free_pages(pfn_to_phys(merged_pfn), order + 1);
        } else {
            // Add to free list
            let page = create_page_frame(pfn, order);
            self.free_lists[order as usize].push_back(page);
        }
    }
}
```

#### Kernel Object Allocator (Slab)

```rust
// crates/kernel/src/mm/slab.rs

pub struct SlabAllocator {
    caches: Vec<KmemCache>,
}

pub struct KmemCache {
    name: &'static str,
    object_size: usize,
    align: usize,
    slabs: LinkedList<Slab>,
    partial: LinkedList<Slab>,
    free: LinkedList<Slab>,
}

pub struct Slab {
    page: PhysAddr,
    num_objects: usize,
    free_count: usize,
    free_list: *mut u8,
}

impl KmemCache {
    pub fn alloc(&mut self) -> Result<*mut u8, Errno> {
        // 1. Try partial slab
        if let Some(slab) = self.partial.front_mut() {
            return slab.alloc_object();
        }

        // 2. Try free slab
        if let Some(slab) = self.free.pop_front() {
            self.partial.push_back(slab);
            return slab.alloc_object();
        }

        // 3. Allocate new slab
        let page = BUDDY_ALLOCATOR.lock().alloc_pages(0)?;
        let slab = Slab::new(page, self.object_size, self.align);
        self.partial.push_back(slab);
        slab.alloc_object()
    }

    pub fn free(&mut self, ptr: *mut u8) {
        let slab = self.find_slab(ptr);
        slab.free_object(ptr);

        if slab.is_empty() {
            // Return to free list
            self.partial.remove(slab);
            self.free.push_back(slab);
        }
    }
}

// Per-CPU quick caches (for hot paths)
pub struct PerCpuCache {
    entries: [Option<*mut u8>; 16],
    count: usize,
}
```

### Interrupt Handling (GICv3 for ARM64)

```rust
// crates/kernel/src/arch/aarch64/gicv3.rs

pub struct Gicv3 {
    distributor_base: usize,  // GICD
    redistributor_base: usize, // GICR (per-CPU)
}

impl Gicv3 {
    pub fn init(&self) {
        // 1. Disable distributor
        self.write_dist(GICD_CTLR, 0);

        // 2. Configure SPIs (32-1019)
        for intid in 32..1020 {
            self.disable_interrupt(intid);
            self.set_priority(intid, 0xa0);
            self.set_target(intid, 0);  // CPU 0
        }

        // 3. Enable distributor
        self.write_dist(GICD_CTLR, GICD_CTLR_ENABLE);

        // 4. Configure redistributor (per-CPU)
        self.init_redistributor();

        // 5. Enable CPU interface
        unsafe {
            // Enable Group 1 interrupts
            asm!("msr ICC_IGRPEN1_EL1, {}", in(reg) 1);
            // Set priority mask
            asm!("msr ICC_PMR_EL1, {}", in(reg) 0xf0);
        }
    }

    pub fn enable_interrupt(&self, intid: u32) {
        let reg = intid / 32;
        let bit = intid % 32;
        self.write_dist(GICD_ISENABLER(reg), 1 << bit);
    }

    pub fn disable_interrupt(&self, intid: u32) {
        let reg = intid / 32;
        let bit = intid % 32;
        self.write_dist(GICD_ICENABLER(reg), 1 << bit);
    }

    pub fn set_affinity(&self, intid: u32, cpu: u8) {
        let aff = (cpu as u64) << 8;  // Aff0
        self.write_dist(GICD_IROUTER(intid), aff);
    }

    pub fn eoi(&self, intid: u32) {
        unsafe {
            asm!("msr ICC_EOIR1_EL1, {}", in(reg) intid);
        }
    }
}

// IRQ handler
pub fn handle_irq() {
    // 1. Read interrupt ID
    let intid: u32;
    unsafe {
        asm!("mrs {}, ICC_IAR1_EL1", out(reg) intid);
    }

    if intid >= 1020 {
        return;  // Spurious
    }

    // 2. Dispatch to handler
    if let Some(handler) = IRQ_HANDLERS.lock().get(&intid) {
        handler();
    }

    // 3. End of interrupt
    GIC.eoi(intid);
}

// Bottom-half/Softirq for deferred work
pub struct Softirq {
    pending: AtomicU32,
    handlers: [Option<fn()>; 32],
}

impl Softirq {
    pub fn raise(&self, nr: u32) {
        self.pending.fetch_or(1 << nr, Ordering::Release);
    }

    pub fn process(&self) {
        let pending = self.pending.swap(0, Ordering::Acquire);

        for i in 0..32 {
            if pending & (1 << i) != 0 {
                if let Some(handler) = self.handlers[i as usize] {
                    handler();
                }
            }
        }
    }
}
```

### Error Handling

```rust
// crates/kernel/src/lib/error.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    OutOfMemory,
    InvalidArgument,
    PermissionDenied,
    NotFound,
    AlreadyExists,
    IoError,
    Interrupted,
    WouldBlock,
    TimedOut,
    NotSupported,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum Errno {
    EPERM = 1,
    ENOENT = 2,
    ESRCH = 3,
    EINTR = 4,
    EIO = 5,
    ENXIO = 6,
    E2BIG = 7,
    EBADF = 9,
    EAGAIN = 11,
    ENOMEM = 12,
    EACCES = 13,
    EFAULT = 14,
    EBUSY = 16,
    EEXIST = 17,
    ENODEV = 19,
    EINVAL = 22,
    ENOSYS = 38,
    // ... more errno values
}

impl From<KernelError> for Errno {
    fn from(err: KernelError) -> Self {
        match err {
            KernelError::OutOfMemory => Errno::ENOMEM,
            KernelError::InvalidArgument => Errno::EINVAL,
            KernelError::PermissionDenied => Errno::EACCES,
            KernelError::NotFound => Errno::ENOENT,
            KernelError::AlreadyExists => Errno::EEXIST,
            KernelError::IoError => Errno::EIO,
            KernelError::Interrupted => Errno::EINTR,
            KernelError::WouldBlock => Errno::EAGAIN,
            KernelError::TimedOut => Errno::ETIMEDOUT,
            KernelError::NotSupported => Errno::ENOSYS,
        }
    }
}

// Panic handler (no panics on normal errors!)
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let location = info.location().unwrap();
    printk!(ERROR, "KERNEL PANIC at {}:{}: {}\n",
        location.file(), location.line(), info.message());

    // Print stack trace
    print_stack_trace();

    // Dump registers
    print_cpu_state();

    // Halt all CPUs
    halt_all_cpus();
    loop { unsafe { asm!("wfi"); } }
}
```

### Debugging Support

```rust
// crates/kernel/src/lib/debug.rs

pub fn print_stack_trace() {
    let mut fp: u64;
    unsafe { asm!("mov {}, x29", out(reg) fp); }

    printk!(ERROR, "Stack trace:\n");

    for i in 0..32 {
        if fp == 0 || !is_valid_kernel_addr(fp) {
            break;
        }

        let lr = unsafe { *(fp.add(8) as *const u64) };
        let symbol = resolve_symbol(lr);

        printk!(ERROR, "  #{}: {:#018x} - {}\n", i, lr, symbol);

        fp = unsafe { *(fp as *const u64) };
    }
}

pub fn resolve_symbol(addr: u64) -> &'static str {
    // Look up in kernel symbol table (generated at build time)
    KERNEL_SYMBOLS.iter()
        .filter(|s| s.addr <= addr && addr < s.addr + s.size)
        .map(|s| s.name)
        .next()
        .unwrap_or("<unknown>")
}

// GDB stub support (via QEMU)
#[cfg(feature = "gdb-stub")]
pub mod gdb {
    pub fn init() {
        // Set up GDB remote protocol over serial
        // Breakpoint handling
        // Memory read/write
        // Register access
    }
}
```

### Performance Targets (Initial)

| Metric | Target | Phase | Notes |
|--------|--------|-------|-------|
| Syscall latency (p99) | ≤ 1 μs | A | ARM64 virt, simple syscalls |
| Context switch | ≤ 10 μs | A/E | Including TLB flush |
| Sequential write | ≥ 100 MB/s | B | virtio-blk with cache |
| Random IOPS (4KB) | ≥ 10K | B | Cached |
| Network throughput | ≥ 300 Mbps | C | TCP, large transfers |
| Memory allocation | ≤ 100 ns | All | Slab, per-CPU cache hit |

### CI & Test Harness

**CI Pipeline** (.github/workflows/os_integration.yml):
```yaml
name: OS Integration Tests

on: [push, pull_request]

jobs:
  phase-a:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y qemu-system-aarch64 expect
      - name: Build kernel + initramfs
        run: |
          cargo build --release
          ./scripts/build_initramfs.sh
      - name: Run Phase A tests
        run: ./tests/phase_a/run_tests.sh
        timeout-minutes: 10

  phase-b:
    needs: phase-a
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build kernel + disk
        run: |
          cargo build --release
          ./scripts/create_ext2_disk.sh
      - name: Run Phase B tests
        run: ./tests/phase_b/run_tests.sh

  # ... phases C-G
```

### Observability

**Kernel Metrics** (/proc/stats):
- Context switches
- Syscall counts per type
- Page faults (major/minor)
- Network packets (RX/TX)
- Block I/O operations

**dmesg Ring Buffer**:
- Ratelimited printk (1000 msgs/sec)
- Timestamps (microseconds)
- Log levels (ERROR, WARN, INFO, DEBUG)

### Documentation Structure

```
docs/
├── phases/
│   ├── phase-a-userspace.md
│   ├── phase-b-storage.md
│   ├── phase-c-networking.md
│   └── ...
├── syscalls/
│   ├── fork.md
│   ├── execve.md
│   └── socket.md
├── drivers/
│   ├── virtio-blk.md
│   └── virtio-net.md
├── filesystems/
│   ├── ext2.md
│   └── ext4.md
└── development/
    ├── build-images.md
    ├── qemu-profiles.md
    └── testing-guide.md
```

---

## Directory & Module Scaffold

**Create these directories and module stubs NOW before starting Phase A0**:

```
crates/kernel/src/
├── arch/
│   └── aarch64/
│       ├── mod.rs
│       ├── trap.rs              # Exception vectors, trap handler
│       ├── psci.rs              # SMP CPU bring-up
│       ├── gicv3.rs             # Interrupt controller
│       ├── timer.rs             # EL1 physical timer (PPI 30)
│       └── mmu.rs               # Page tables, TLB operations
│
├── syscall/
│   ├── mod.rs                   # Dispatcher, syscall table
│   ├── table.rs                 # Syscall number → function mapping
│   └── uaccess.rs               # copy_from_user, copy_to_user
│
├── process/
│   ├── mod.rs                   # Process struct, PID table
│   ├── scheduler.rs             # Runqueue, context switch
│   └── exec/
│       └── elf.rs               # ELF64 loader
│
├── mm/
│   ├── mod.rs
│   ├── paging.rs                # Address space, page tables
│   ├── fault.rs                 # Page fault handler
│   ├── buddy.rs                 # Physical page allocator
│   ├── slab.rs                  # Kernel object allocator
│   ├── cow.rs                   # Copy-on-write fork
│   ├── mprotect.rs              # Memory protection changes
│   └── page_cache.rs            # Block/page cache (Phase B)
│
├── vfs/
│   ├── mod.rs                   # VFS core, mount table
│   ├── inode.rs                 # Inode operations
│   ├── dentry.rs                # Directory entry cache
│   ├── mount.rs                 # Mount/umount
│   ├── tmpfs.rs                 # In-RAM filesystem
│   ├── devfs.rs                 # Device nodes
│   ├── procfs.rs                # /proc entries
│   └── sysfs.rs                 # /sys entries
│
├── fs/
│   └── ext2/
│       ├── mod.rs               # ext2 superblock, mount
│       ├── inode.rs             # Inode operations
│       ├── dir.rs               # Directory traversal
│       └── block.rs             # Block I/O with cache
│
├── drivers/
│   ├── virtio/
│   │   ├── mod.rs               # Virtio common (queues, config)
│   │   ├── blk.rs               # virtio-blk driver
│   │   └── net.rs               # virtio-net driver
│   └── char/
│       ├── tty.rs               # TTY line discipline
│       ├── pty.rs               # PTY master/slave
│       ├── console.rs           # virtio-console
│       └── urandom.rs           # /dev/urandom entropy
│
├── net/
│   ├── mod.rs                   # Network subsystem init
│   ├── smoltcp_iface.rs         # smoltcp PHY integration
│   ├── socket.rs                # BSD sockets ABI
│   ├── dhcp.rs                  # DHCP client
│   └── dns.rs                   # DNS resolver
│
├── security/
│   ├── mod.rs
│   ├── cred.rs                  # Credentials (UID/GID)
│   └── perms.rs                 # Permission checks
│
└── lib/
    ├── mod.rs
    ├── printk.rs                # Kernel logging
    ├── ringbuf.rs               # Ring buffer utility
    ├── error.rs                 # KernelError, Errno
    └── debug.rs                 # Stack traces, symbol resolution

tests/
├── phase_a0/
│   └── run_tests.sh             # A0 acceptance tests
├── phase_a1/
│   └── run_tests.sh             # A1 acceptance tests
├── phase_a2/
│   └── run_tests.sh             # A2 acceptance tests
├── phase_b/
│   └── run_tests.sh
├── phase_c/
│   └── run_tests.sh
├── phase_d/
│   └── run_tests.sh
├── phase_e/
│   └── run_tests.sh
└── phase_f/
    └── run_tests.sh

scripts/
├── build_initramfs.sh           # Build BusyBox + musl initramfs
├── create_ext2_disk.sh          # Create ext2 disk image
└── qemu_profiles.sh             # QEMU launch configs per phase
```

---

## Risk Controls & Guardrails

### Technical Guardrails

1. **No kernel async/await runtimes**
   - Prefer blocking I/O with kernel threads or explicit state machines
   - Avoid tokio, async-std, or any heavyweight async runtime
   - Rationale: Kernel async is immature, adds complexity, harder to debug

2. **Avoid deep filesystem features initially**
   - Skip ext2/ext4 extents, ACLs, xattrs in early phases
   - Implement basic read/write/directory ops first
   - Add advanced features only after basic functionality proven

3. **Use smoltcp for networking**
   - Don't write TCP/IP from scratch (saves 3-4 weeks)
   - Vendor dependency if concerned about network outages during dev
   - Rationale: TCP has ~30 edge cases, battle-tested code wins

4. **Feature flags and phase gates**
   - Use Cargo features to gate incomplete phases
   - Example: `features = ["phase-a", "phase-b"]`
   - Prevents accidentally using unimplemented features

5. **Locking discipline**
   - Document lock ordering in mm/locking.md before Phase E
   - Use lockdep-style assertions (in dev builds)
   - Never hold spinlock across blocking operations

### Development Practices

1. **Incremental testing**
   - Every syscall gets a unit test
   - Integration tests run in CI on every commit
   - Manual testing on real QEMU before marking phase complete

2. **No premature optimization**
   - Get correctness first, then profile
   - Exception: Page cache is needed in Phase B (not later)
   - Performance targets are guidance, not gates

3. **Documentation as you go**
   - Update OS-BLUEPRINT.md when reality diverges
   - Document QEMU quirks immediately when found
   - Syscall reference updated with each new syscall

4. **Code review**
   - All kernel code reviewed before merge
   - Focus on safety (unsafe blocks), locking, error paths
   - Panics only for truly unrecoverable errors

---

## Definition of Done (Per Deliverable)

### Code Quality

- ✅ Compiles with `cargo build --release` with zero warnings
- ✅ Clippy passes with `clippy::pedantic` (or documented exceptions)
- ✅ Unit tests for core logic (e.g., buddy allocator, COW, page cache)
- ✅ Phase acceptance tests pass in CI

### Documentation

- ✅ All new syscalls documented in syscalls/ with:
  - Purpose and behavior
  - Arguments and return values
  - Error codes (errno) and conditions
- ✅ Complex subsystems have architecture docs (e.g., mm/COW.md)
- ✅ README updated with build/run instructions

### Quality Gates

- ✅ dmesg clean of WARN/ERROR under acceptance tests
- ✅ No memory leaks in basic runs (track via allocator stats)
- ✅ No panics or deadlocks under normal workload
- ✅ Passes 10-minute stress test (after Phase A)

### Phase-Specific Gates

**Phase A0**:
- Trap path works, syscalls dispatch correctly
- printk outputs to console
- dmesg readable

**Phase A1**:
- Boots to BusyBox shell
- fork/exec/wait work with COW
- No OOM under moderate process creation

**Phase A2**:
- PTY creation/communication works
- All /proc entries accurate
- BusyBox tools requiring PTY functional

**Phase B**:
- Files persist across reboot
- sync(2) completes without corruption
- Large file (128MB+) operations work

**Phase C**:
- DHCP acquires IP
- DNS resolves hostnames
- HTTP fetch returns valid HTML
- Socket send/recv functional

**Phase D**:
- Permission checks enforced
- NX/W^X active (no exec from RW pages)
- ASLR shows variance across boots
- /dev/urandom yields random bytes

**Phase E**:
- All vCPUs active (nproc shows correct count)
- Load distributes across cores
- No race conditions in 24h soak test

**Phase F**:
- Journal replay after crash works
- Data intact after forced power-off test
- fsck clean after recovery

---

## Execution Timeline

### Phase Sequencing (Revised with A0/A1/A2 Split)

```
Week 1:      Phase A0 (Syscall infra)     ██
Weeks 2-3:   Phase A1 (Minimal userspace) ████
Week 4:      Phase A2 (PTY + full /proc)  ██
Weeks 5-6:   Phase B (Storage + cache)      ████
Weeks 7-8:   Phase C (Networking/smoltcp)    ████
Weeks 9-10:  Phase D (Security)                ████
Weeks 11-13: Phase E (SMP)                       ██████
Weeks 14-15: Phase F (Resilience)                     ████
Week 16+:    Phase G (Graphics) [Optional]               ██

Total: ~16 weeks to complete OS (was 14, adjusted for realism)
       Phase A expanded from 3→4 weeks (de-risked with A0/A1/A2)
       Phase E expanded from 2→3 weeks (SMP debugging takes time)
```

### Week-by-Week Breakdown

**Week 1 - A0**: Trap path, syscall dispatch, uaccess, printk, basic scheduler
**Week 2-3 - A1**: Process model, COW fork, ELF, VFS (tmpfs/devfs/procfs minimal), TTY, initramfs, BusyBox boot
**Week 4 - A2**: PTY implementation, full /proc entries, line discipline
**Week 5-6 - B**: virtio-blk, block layer, page cache, ext2, partitioning
**Week 7-8 - C**: virtio-net, smoltcp integration, sockets, DHCP, DNS
**Week 9-10 - D**: Credentials, /dev/urandom, NX/W^X, ASLR, mprotect (COW already done)
**Week 11-13 - E**: PSCI, per-CPU data, scheduler SMP, IRQ affinity, IPIs, locking docs
**Week 14-15 - F**: ext4 journaling OR ext2 metadata journal, crash recovery
**Week 16+ - G**: virtio-gpu (optional)

### Resource Allocation

- **Core Kernel Team**: 2-3 engineers
- **Testing/CI**: 1 engineer
- **Documentation**: Shared responsibility
- **Code Review**: All engineers participate

---

## Success Metrics

### Per-Phase Metrics

| Phase | Tests Pass | Boot Time | Memory Usage | Stability |
|-------|-----------|-----------|--------------|-----------|
| A | ✅ 6/6 | < 5s | < 128MB | No panics 1h |
| B | ✅ 4/4 | < 5s | < 150MB | No corruption |
| C | ✅ 5/5 | < 7s | < 200MB | No packet loss |
| D | ✅ 6/6 | < 7s | < 200MB | Perms enforced |
| E | ✅ 4/4 | < 7s | < 256MB | SMP stable 24h |
| F | ✅ 2/2 | < 8s | < 256MB | Crash recovery |

### Final OS Acceptance

- ✅ Boot to multi-user system
- ✅ Run standard POSIX tools (BusyBox full suite)
- ✅ Persistent storage with journaling
- ✅ Network connectivity (HTTP/HTTPS)
- ✅ Multi-core scheduling
- ✅ Memory and security protections
- ✅ 99.9% uptime over 7 days
- ✅ No data corruption under stress
- ✅ All 50+ acceptance tests pass

---

## Risk Mitigation

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| TCP/IP stack bugs | High | Use smoltcp (battle-tested) OR extensive fuzzing |
| ext2 corruption | High | Thorough testing; add fsck; consider ext4 earlier |
| SMP race conditions | Medium | Lockdep, KASAN, stress testing |
| Memory leaks | Medium | Automated leak detection in CI |
| QEMU quirks | Low | Test on multiple QEMU versions |

### Schedule Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Phase A overrun | Medium | Start early; allocate 4 weeks if needed |
| Networking complexity | Medium | Use smoltcp to save 1-2 weeks |
| Testing gaps | Low | Automate tests from day 1 |

---

## Future Phases (Post-MVP)

### Phase H — Advanced Filesystems
- ext4 extents
- Btrfs / F2FS
- FUSE support

### Phase I — Containers
- Namespaces (PID, mount, network, IPC, UTS)
- Cgroups (CPU, memory, I/O limits)
- seccomp-bpf

### Phase J — Real Hardware
- Raspberry Pi 4 support
- UART drivers
- GPIO / I2C / SPI

### Phase K — GUI Desktop
- DRM/KMS
- Wayland compositor
- Desktop environment (lightweight)

---

## Appendix

### A. QEMU Command Reference

```bash
# Basic boot (Phase A)
qemu-system-aarch64 -machine virt -cpu cortex-a57 -m 1G \
  -kernel KERNEL.ELF -initrd initramfs.cpio.gz -nographic

# With storage (Phase B)
qemu-system-aarch64 -machine virt -cpu cortex-a57 -m 1G \
  -kernel KERNEL.ELF -initrd initramfs.cpio.gz \
  -drive file=disk.img,format=raw,if=none,id=hd \
  -device virtio-blk,drive=hd \
  -nographic

# With networking (Phase C)
qemu-system-aarch64 -machine virt -cpu cortex-a57 -m 1G \
  -kernel KERNEL.ELF -initrd initramfs.cpio.gz \
  -netdev user,id=net0 -device virtio-net,netdev=net0 \
  -nographic

# SMP (Phase E)
qemu-system-aarch64 -machine virt -cpu cortex-a57 -smp 4 -m 2G \
  -kernel KERNEL.ELF -initrd initramfs.cpio.gz \
  -nographic

# With graphics (Phase G)
qemu-system-aarch64 -machine virt -cpu cortex-a57 -m 2G \
  -kernel KERNEL.ELF -initrd initramfs.cpio.gz \
  -device virtio-gpu-pci \
  -device virtio-keyboard-pci \
  -device virtio-mouse-pci
```

### B. Syscall Number Reference (ARM64)

See: https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/unistd.h

### C. Related Documents

- GUI/BLUEPRINT.md - Desktop app specification
- docs/PHASES-5-6-COMPLETION.md - Current kernel state
- docs/EXPLAINABILITY-GUIDE.md - AI-native features
- docs/EU-AI-ACT-COMPLIANCE.md - Compliance documentation

---

**Document Status**: ✅ Complete
**Next Action**: Begin Phase A implementation (fork/exec/syscalls)
**Approval Required**: Architecture review + resource allocation
