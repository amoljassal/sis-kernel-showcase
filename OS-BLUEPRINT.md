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

**Timeline**: 2–3 weeks (Heaviest lift)

### Scope

Transform the kernel from single-binary shell to multi-process userspace environment running standard POSIX utilities.

### Implementation Details

#### 1. Process Model

**Current State**: Single kernel-space shell with no process isolation.

**Target State**: Per-process address spaces with full isolation.

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

#### 2. Syscall Interface (MVP)

**Required Syscalls** (30 total):

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
| poll/select | 73/82 | I/O multiplexing | P1 |
| brk | 214 | Heap management | P0 |
| mmap | 222 | Memory mapping | P0 |
| munmap | 215 | Unmap memory | P1 |
| fork | 220 | Create process | P0 |
| execve | 221 | Execute program | P0 |
| wait4 | 260 | Wait for child | P0 |
| exit | 93 | Terminate process | P0 |
| getpid | 172 | Get process ID | P0 |
| clock_gettime | 113 | Get time | P1 |
| nanosleep | 101 | Sleep | P1 |

**Implementation Structure**:

```rust
// crates/kernel/src/syscall/mod.rs
pub fn syscall_dispatcher(nr: usize, args: &[u64; 6]) -> Result<isize, Errno> {
    match nr {
        63 => sys_read(args[0] as i32, args[1] as *mut u8, args[2]),
        64 => sys_write(args[0] as i32, args[1] as *const u8, args[2]),
        220 => sys_fork(),
        221 => sys_execve(args[0] as *const i8, args[1], args[2]),
        260 => sys_wait4(args[0] as i32, args[1] as *mut i32, args[2] as i32),
        // ... etc
        _ => Err(Errno::ENOSYS),
    }
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

#### 5. TTY/PTY Subsystem

**Architecture**:
```
┌──────────────┐
│  User App    │
└──────┬───────┘
       │ read/write/ioctl
       ▼
┌──────────────┐
│   TTY Line   │ ← Line discipline (cooked/raw)
│  Discipline  │   termios, echo, signals
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Console Drv  │ ← virtio-console (existing)
└──────────────┘
```

**TTY Features**:
- Line discipline: canonical (cooked) vs raw mode
- termios: c_iflag, c_oflag, c_cflag, c_lflag
- Echo/Erase/Kill line editing
- Signal generation: Ctrl-C (SIGINT), Ctrl-Z (SIGTSTP), Ctrl-\ (SIGQUIT)
- Job control (optional Phase A, full in later phases)

**PTY (Pseudo-Terminal)**:
```rust
// crates/kernel/src/tty/pty.rs
pub struct PtyMaster {
    pub index: usize,
    pub slave: Arc<PtySlave>,
    pub buffer: RingBuffer,
}

pub struct PtySlave {
    pub index: usize,
    pub master: Weak<PtyMaster>,
    pub termios: Termios,
}
```

**Device Nodes**:
- /dev/console - System console (virtio-console)
- /dev/tty - Controlling terminal
- /dev/ptmx - PTY master clone device
- /dev/pts/0, /dev/pts/1, ... - PTY slaves

#### 6. Initramfs

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

# Test 6: TTY/PTY
test_pty() {
    expect << 'EOF'
spawn $QEMU_CMD
expect "/ #"
send "script -qc 'echo PTY OK' /dev/null\r"
expect "PTY OK" { exit 0 }
exit 1
EOF
}

# Run all tests
test_boot
test_commands
test_fork_exec
test_pipes
test_signals
test_pty

echo "Phase A: All tests passed ✓"
```

### Artifacts

1. **Kernel Binary**: KERNEL.ELF with userspace support
2. **Initramfs**: initramfs.cpio.gz (BusyBox + init)
3. **Build Scripts**:
   - scripts/build_initramfs.sh
   - scripts/build_kernel_userspace.sh
4. **CI Job**: .github/workflows/phase_a.yml

### Exit Criteria

- ✅ Reproducibly boots to BusyBox prompt on virtio-console
- ✅ All 30 MVP syscalls implemented and functional
- ✅ fork/exec/wait create and reap processes correctly
- ✅ Pipes, signals, TTY/PTY work as expected
- ✅ /proc, /sys, /dev filesystems mounted and accessible
- ✅ All acceptance tests pass in CI
- ✅ No kernel panics or deadlocks under normal workload

---

## Phase B — Persistent Storage (Block + ext2)

**Objective**: Read/write a persistent filesystem; survive reboot.

**Timeline**: 1–2 weeks

### Scope

Add block device support and ext2 filesystem for persistent storage across reboots.

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

#### 4. ext2 Filesystem

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

## Phase C — Networking (virtio-net + Sockets + TCP/IP)

**Objective**: Bring up network, get IP via DHCP, resolve DNS, and fetch HTTP.

**Timeline**: 2 weeks

### Scope

Add full networking stack with virtio-net driver, sockets API, and TCP/IP implementation.

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

**Stack Options**:

**Option A**: Integrate smoltcp (Rust embedded TCP/IP stack)
```toml
[dependencies]
smoltcp = { version = "0.11", default-features = false, features = ["proto-ipv4", "socket-tcp", "socket-udp"] }
```

**Option B**: Minimal in-kernel stack (recommended for learning)

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

## Execution Timeline

### Phase Sequencing

```
Weeks 0-3:   Phase A (Userspace)      ████████░░░░
Weeks 3-5:   Phase B (Storage)              ████░░
Weeks 5-7:   Phase C (Networking)              ████
Weeks 7-9:   Phase D (Security)                  ████
Weeks 9-11:  Phase E (SMP)                         ████
Weeks 11-13: Phase F (Resilience)                    ████
Week 14+:    Phase G (Graphics) [Optional]             ██

Total: ~14 weeks to complete OS
```

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
