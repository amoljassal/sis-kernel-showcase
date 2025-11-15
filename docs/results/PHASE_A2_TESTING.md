# Phase A2 Testing Guide

**Status**: Ready for Testing (requires network access to build)
**Date**: 2025-11-05
**Branch**: `claude/os-impl-phase-a-011CUpm4M4bDUrf6TDy9ZFaG`

---

## Build Status

**Current Issue**: Cannot build due to crates.io access restriction (HTTP 403)

**When network access is restored**, build with:
```bash
cd /home/user/sis-kernel-showcase
cargo build --release --target aarch64-unknown-none
```

---

## Phase A2 Features to Test

### 1. PTY (Pseudo-Terminal) Support

#### 1.1 Basic PTY Creation
```bash
# Boot kernel with QEMU
qemu-system-aarch64 -machine virt -cpu cortex-a72 \
  -kernel target/aarch64-unknown-none/release/sis_kernel \
  -nographic -serial mon:stdio

# In kernel shell, test /dev/ptmx exists
/ # ls -l /dev/ptmx
crw-rw-rw- 1 root root 0, 0 /dev/ptmx

# Test /dev/pts directory exists
/ # ls -l /dev/pts/
total 0
```

#### 1.2 PTY Pair Creation (C test program)
Create test program `test_pty.c`:
```c
#include <fcntl.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/ioctl.h>

int main() {
    // Open /dev/ptmx
    int master_fd = open("/dev/ptmx", O_RDWR);
    if (master_fd < 0) {
        printf("FAIL: Cannot open /dev/ptmx\n");
        return 1;
    }
    printf("PASS: Opened /dev/ptmx, fd=%d\n", master_fd);

    // Get PTY number
    unsigned int pty_num;
    if (ioctl(master_fd, 0x80045430, &pty_num) < 0) {  // TIOCGPTN
        printf("FAIL: TIOCGPTN failed\n");
        return 1;
    }
    printf("PASS: PTY number = %u\n", pty_num);

    // Verify /dev/pts/N exists
    char slave_path[32];
    snprintf(slave_path, sizeof(slave_path), "/dev/pts/%u", pty_num);
    printf("Slave path: %s\n", slave_path);

    // Open slave
    int slave_fd = open(slave_path, O_RDWR);
    if (slave_fd < 0) {
        printf("FAIL: Cannot open %s\n", slave_path);
        return 1;
    }
    printf("PASS: Opened slave, fd=%d\n", slave_fd);

    // Test master -> slave write/read
    const char *msg = "hello pty\n";
    if (write(master_fd, msg, 10) != 10) {
        printf("FAIL: Master write failed\n");
        return 1;
    }
    printf("PASS: Master wrote 10 bytes\n");

    char buf[128];
    int n = read(slave_fd, buf, sizeof(buf));
    if (n <= 0) {
        printf("FAIL: Slave read failed\n");
        return 1;
    }
    buf[n] = '\0';
    printf("PASS: Slave read %d bytes: %s", n, buf);

    close(master_fd);
    close(slave_fd);
    printf("PASS: All PTY tests passed!\n");
    return 0;
}
```

**Expected Output**:
```
PASS: Opened /dev/ptmx, fd=3
PASS: PTY number = 0
Slave path: /dev/pts/0
PASS: Opened slave, fd=4
PASS: Master wrote 10 bytes
PASS: Slave read 11 bytes: hello pty
PASS: All PTY tests passed!
```

#### 1.3 Termios IOCTL Testing
```c
#include <termios.h>

struct termios tios;
if (ioctl(master_fd, TCGETS, &tios) < 0) {
    printf("FAIL: TCGETS failed\n");
}
printf("PASS: TCGETS succeeded\n");
printf("c_lflag = 0x%x (ICANON=%d, ECHO=%d)\n",
       tios.c_lflag,
       !!(tios.c_lflag & ICANON),
       !!(tios.c_lflag & ECHO));

// Disable echo
tios.c_lflag &= ~ECHO;
if (ioctl(master_fd, TCSETS, &tios) < 0) {
    printf("FAIL: TCSETS failed\n");
}
printf("PASS: TCSETS succeeded (echo disabled)\n");
```

#### 1.4 Line Discipline Testing
Test canonical mode (line buffering):
```c
// Write partial line (no newline)
write(master_fd, "hello", 5);

// Slave read should block (canonical mode waits for newline)
// Try non-blocking read
fcntl(slave_fd, F_SETFL, O_NONBLOCK);
int n = read(slave_fd, buf, sizeof(buf));
if (n < 0 && errno == EAGAIN) {
    printf("PASS: Canonical mode blocks on partial line\n");
}

// Complete the line
write(master_fd, " world\n", 7);

// Now slave should read full line
n = read(slave_fd, buf, sizeof(buf));
if (n == 12) {  // "hello world\n"
    printf("PASS: Read complete line after newline\n");
}
```

---

### 2. Extended Procfs

#### 2.1 /proc/self Symlink
```bash
/ # cat /proc/self/cmdline
/bin/sh

/ # cat /proc/self/stat
1 (sh) R 0 0 0...

/ # ls -l /proc/self/
total 0
-r--r--r-- 1 root root 0 cmdline
-r--r--r-- 1 root root 0 stat
-r--r--r-- 1 root root 0 status
-r--r--r-- 1 root root 0 maps
```

**Expected**: `/proc/self` resolves to current process's PID directory

#### 2.2 /proc/[pid]/maps
```bash
/ # cat /proc/self/maps
00400000-00500000 r-xp 00000000 00:00 0          [text]
00600000-00700000 rw-p 00000000 00:00 0          [data]
00800000-00900000 rw-p 00000000 00:00 0          [heap]
007ffffff00000-007fffff00000 rw-p 00000000 00:00 0          [stack]

/ # cat /proc/1/maps  # Same as above for PID 1
```

**Expected**: Shows memory mapping information (Phase A2: static entries)

#### 2.3 Existing Procfs Entries (Regression Test)
```bash
/ # cat /proc/cpuinfo
processor       : 0
BogoMIPS        : 125.00
Features        : fp asimd
CPU implementer : 0x41
CPU architecture: 8
CPU variant     : 0x0
CPU part        : 0xd08

/ # cat /proc/meminfo
MemTotal:     32768 kB
MemFree:      28672 kB
MemAvailable: 28672 kB

/ # cat /proc/uptime
123.45 120.00

/ # cat /proc/mounts
tmpfs / tmpfs rw 0 0
devfs /dev devfs rw 0 0
proc /proc proc rw 0 0

/ # cat /proc/1/cmdline
/bin/sh

/ # cat /proc/1/stat
1 (sh) R 0 0 0...

/ # cat /proc/1/status
Name:   sh
State:  R (running)
Pid:    1
PPid:   0
```

---

### 3. CWD (Current Working Directory) Tracking

#### 3.1 getcwd() Basic Test
```bash
/ # pwd
/

/ # mkdir /tmp
/ # cd /tmp
/tmp # pwd
/tmp

/tmp # cd /
/ # pwd
/
```

**Expected**: `pwd` returns current working directory

#### 3.2 Relative Path Resolution
```bash
/ # mkdir -p /tmp/test/subdir
/ # cd /tmp/test
/tmp/test # pwd
/tmp/test

/tmp/test # cd subdir
/tmp/test/subdir # pwd
/tmp/test/subdir

/tmp/test/subdir # cd ..
/tmp/test # pwd
/tmp/test

/tmp/test # cd ../..
/ # pwd
/
```

**Expected**: Relative paths resolve correctly

#### 3.3 chdir() with Relative Paths (C test)
```c
#include <unistd.h>
#include <stdio.h>

int main() {
    char cwd[256];

    // Start at root
    if (getcwd(cwd, sizeof(cwd)) == NULL) {
        printf("FAIL: getcwd failed\n");
        return 1;
    }
    printf("Initial CWD: %s\n", cwd);

    // Change to /tmp
    if (chdir("/tmp") < 0) {
        printf("FAIL: chdir /tmp failed\n");
        return 1;
    }
    getcwd(cwd, sizeof(cwd));
    printf("After 'cd /tmp': %s\n", cwd);

    // Relative: cd test
    if (chdir("test") < 0) {
        printf("FAIL: chdir test failed\n");
        return 1;
    }
    getcwd(cwd, sizeof(cwd));
    printf("After 'cd test': %s\n", cwd);

    // Relative: cd ..
    if (chdir("..") < 0) {
        printf("FAIL: chdir .. failed\n");
        return 1;
    }
    getcwd(cwd, sizeof(cwd));
    printf("After 'cd ..': %s\n", cwd);

    printf("PASS: All CWD tests passed\n");
    return 0;
}
```

**Expected Output**:
```
Initial CWD: /
After 'cd /tmp': /tmp
After 'cd test': /tmp/test
After 'cd ..': /tmp
PASS: All CWD tests passed
```

#### 3.4 open() with Relative Paths
```bash
/ # cd /tmp
/tmp # touch file.txt
/tmp # echo "hello" > file.txt
/tmp # cat file.txt
hello

/tmp # cd /
/ # cat /tmp/file.txt
hello

/ # cd /tmp
/tmp # cat ./file.txt
hello
```

**Expected**: Files created with relative paths are accessible

#### 3.5 CWD Inheritance (fork test)
```c
#include <unistd.h>
#include <stdio.h>
#include <sys/wait.h>

int main() {
    char cwd[256];

    // Parent changes to /tmp
    chdir("/tmp");
    getcwd(cwd, sizeof(cwd));
    printf("Parent CWD: %s\n", cwd);

    pid_t pid = fork();
    if (pid == 0) {
        // Child - should inherit parent's CWD
        getcwd(cwd, sizeof(cwd));
        printf("Child CWD: %s\n", cwd);

        // Child changes to /
        chdir("/");
        getcwd(cwd, sizeof(cwd));
        printf("Child after cd /: %s\n", cwd);
        return 0;
    }

    // Parent waits
    wait(NULL);

    // Parent's CWD should be unchanged
    getcwd(cwd, sizeof(cwd));
    printf("Parent CWD after child: %s\n", cwd);

    return 0;
}
```

**Expected Output**:
```
Parent CWD: /tmp
Child CWD: /tmp
Child after cd /: /
Parent CWD after child: /tmp
PASS: Child inherits CWD, changes don't affect parent
```

---

## Automated Test Suite

Create `tests/phase_a2_automated.sh`:
```bash
#!/bin/bash
set -e

echo "=== Phase A2 Automated Test Suite ==="

# Compile test programs
aarch64-linux-gnu-gcc -static test_pty.c -o test_pty
aarch64-linux-gnu-gcc -static test_cwd.c -o test_cwd

# Create initramfs with test programs
mkdir -p initramfs/bin
cp test_pty test_cwd initramfs/bin/
(cd initramfs && find . | cpio -o -H newc | gzip) > initramfs.cpio.gz

# Boot kernel with test initramfs
qemu-system-aarch64 -machine virt -cpu cortex-a72 \
  -kernel target/aarch64-unknown-none/release/sis_kernel \
  -initrd initramfs.cpio.gz \
  -nographic -serial mon:stdio \
  -append "console=ttyAMA0" &

QEMU_PID=$!

# Wait for boot
sleep 3

# Run tests via expect
expect << 'EOF'
set timeout 30
expect "/ #"

# Test 1: PTY
send "/bin/test_pty\r"
expect {
    "PASS: All PTY tests passed!" { puts "✓ PTY tests passed" }
    timeout { puts "✗ PTY tests failed"; exit 1 }
}
expect "/ #"

# Test 2: Procfs
send "cat /proc/self/cmdline\r"
expect {
    "/bin/sh" { puts "✓ /proc/self works" }
    timeout { puts "✗ /proc/self failed"; exit 1 }
}
expect "/ #"

send "cat /proc/self/maps | wc -l\r"
expect {
    "4" { puts "✓ /proc/[pid]/maps works" }
    timeout { puts "✗ /proc/[pid]/maps failed"; exit 1 }
}
expect "/ #"

# Test 3: CWD
send "/bin/test_cwd\r"
expect {
    "PASS: All CWD tests passed" { puts "✓ CWD tests passed" }
    timeout { puts "✗ CWD tests failed"; exit 1 }
}
expect "/ #"

# Test 4: Relative paths
send "mkdir -p /tmp/test\r"
expect "/ #"
send "cd /tmp/test\r"
expect "/ #"
send "pwd\r"
expect {
    "/tmp/test" { puts "✓ Relative path resolution works" }
    timeout { puts "✗ Relative path resolution failed"; exit 1 }
}

puts "\n=== All Phase A2 tests passed! ==="
exit 0
EOF

RESULT=$?

# Cleanup
kill $QEMU_PID 2>/dev/null || true

exit $RESULT
```

---

## Manual Testing Checklist

- [ ] Kernel builds successfully (`cargo build --release --target aarch64-unknown-none`)
- [ ] Kernel boots to shell
- [ ] `/dev/ptmx` exists and is accessible
- [ ] `/dev/pts/` directory exists
- [ ] PTY pair creation works (open /dev/ptmx, get PTY number)
- [ ] PTY master/slave I/O works bidirectionally
- [ ] TCGETS/TCSETS IOCTLs work
- [ ] Line discipline (canonical mode) works
- [ ] `/proc/self` symlink resolves correctly
- [ ] `/proc/self/maps` shows memory mappings
- [ ] All existing procfs entries still work (cpuinfo, meminfo, uptime, mounts, [pid]/*)
- [ ] `pwd` returns current working directory
- [ ] `cd` with absolute paths works
- [ ] `cd` with relative paths works
- [ ] `cd ..` and `cd .` work correctly
- [ ] Path normalization handles `.` and `..`
- [ ] `open()` with relative paths works
- [ ] CWD is inherited by child processes
- [ ] Child CWD changes don't affect parent

---

## Expected Issues / Known Limitations

### Phase A2 Scope
1. **PTY Window Size**: TIOCGWINSZ/TIOCSWINSZ not implemented (Phase C+)
2. **PTY Process Groups**: TIOCGPGRP/TIOCSPGRP not implemented (Phase C+)
3. **Procfs /proc/[pid]/maps**: Shows static entries, not actual VMAs (Phase B+ for real VMA tracking)
4. **Symlinks**: Not implemented yet (Phase B+), so /proc/self is implemented as directory lookup
5. **openat() with dirfd**: Only AT_FDCWD (-100) supported (Phase B+)

---

## Regression Testing

Ensure all Phase A1 features still work:

### Phase A1 Syscalls (30/30)
```bash
/ # ls /                  # sys_getdents64
/ # cat /proc/cpuinfo     # sys_read
/ # echo test             # sys_write
/ # mkdir /tmp/test       # sys_mkdirat
/ # touch /tmp/file       # sys_openat + sys_close
/ # cat /tmp/file         # sys_read
/ # rm /tmp/file          # sys_unlinkat
/ # ps                    # sys_getpid, sys_getppid
/ # ls /dev/              # sys_getdents64 on devfs
```

### Phase A1 Features
- [ ] Process creation (fork/exec) works
- [ ] Signals (SIGKILL, SIGCHLD, etc.) work
- [ ] Pipes work (`ls | wc -l`)
- [ ] Console I/O works
- [ ] VFS (tmpfs, devfs, procfs) works
- [ ] BusyBox shell works

---

## Build and Run Instructions

### Prerequisites
```bash
# Install Rust nightly
rustup install nightly
rustup default nightly

# Install target
rustup target add aarch64-unknown-none

# Install QEMU (if testing locally)
sudo apt-get install qemu-system-aarch64

# Install cross-compiler (for test programs)
sudo apt-get install gcc-aarch64-linux-gnu
```

### Build Kernel
```bash
cd /home/user/sis-kernel-showcase
cargo build --release --target aarch64-unknown-none
```

### Run Kernel
```bash
qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a72 \
  -smp 1 \
  -m 128M \
  -kernel target/aarch64-unknown-none/release/sis_kernel \
  -nographic \
  -serial mon:stdio
```

---

## Success Criteria

Phase A2 is considered **COMPLETE** when:

1. ✅ All code committed and pushed
2. ✅ Kernel builds without errors
3. ✅ Kernel boots to shell
4. ✅ All PTY tests pass
5. ✅ All procfs tests pass
6. ✅ All CWD tests pass
7. ✅ No regressions in Phase A1 features
8. ✅ BusyBox tools requiring PTY work (script, vi, top)

---

## Contact

For issues or questions:
- Check PHASE_A2_COMPLETION.md for implementation details
- Review OS-BLUEPRINT.md for Phase A2 specification
- Examine commit history: de23baa, 36777d3, 97e8f31

**Phase A2 Status**: ✅ IMPLEMENTATION COMPLETE, AWAITING TESTING
**Next Phase**: Phase B - Persistent Storage
