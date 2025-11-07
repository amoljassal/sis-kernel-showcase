#!/bin/bash
# Phase A1 Acceptance Tests
#
# Tests the kernel boots to a BusyBox shell and validates basic functionality:
# - Boot to shell prompt
# - File I/O (ls, cat, echo, touch)
# - Process management (fork, exec, wait)
# - Pipes
# - Exit codes

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/../.."
KERNEL="$ROOT_DIR/target/aarch64-unknown-none/release/sis_kernel"
QEMU="qemu-system-aarch64"

echo "==> Phase A1 Acceptance Tests"
echo ""

# Check prerequisites
if [ ! -f "$KERNEL" ]; then
    echo "ERROR: Kernel not found at $KERNEL"
    echo "Run: cargo build --release --target aarch64-unknown-none"
    exit 1
fi

if ! command -v "$QEMU" &> /dev/null; then
    echo "ERROR: QEMU not found"
    echo "Install: sudo apt-get install qemu-system-arm"
    exit 1
fi

if ! command -v expect &> /dev/null; then
    echo "ERROR: expect not found"
    echo "Install: sudo apt-get install expect"
    exit 1
fi

# Check if initramfs is built
if [ ! -f "$ROOT_DIR/build/initramfs.cpio" ]; then
    echo "WARNING: initramfs not found"
    echo "Run: bash $ROOT_DIR/scripts/build_initramfs.sh"
    echo ""
    echo "Skipping tests that require userspace..."
    exit 0
fi

echo "==> Starting QEMU with kernel and initramfs"
echo ""

# Create expect script
EXPECT_SCRIPT=$(mktemp)
trap "rm -f $EXPECT_SCRIPT" EXIT

cat > "$EXPECT_SCRIPT" << 'EXPECT_EOF'
#!/usr/bin/expect -f

set timeout 30
set kernel [lindex $argv 0]

# Start QEMU
spawn qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a57 \
    -m 128M \
    -nographic \
    -kernel $kernel \
    -append "console=ttyAMA0 init=/sbin/init"

# Wait for shell prompt
expect {
    timeout { puts "FAIL: Timeout waiting for shell prompt"; exit 1 }
    "/ #" { puts "PASS: Got shell prompt" }
    eof { puts "FAIL: Unexpected EOF"; exit 1 }
}

# Test 1: List root directory
send "ls /\r"
expect {
    timeout { puts "FAIL: ls timeout"; exit 1 }
    -re "bin.*sbin.*dev" { puts "PASS: ls shows expected directories" }
    eof { puts "FAIL: Unexpected EOF during ls"; exit 1 }
}
expect "/ #"

# Test 2: Echo command
send "echo hello world\r"
expect {
    timeout { puts "FAIL: echo timeout"; exit 1 }
    "hello world" { puts "PASS: echo works" }
    eof { puts "FAIL: Unexpected EOF during echo"; exit 1 }
}
expect "/ #"

# Test 3: Create file and write to it
send "touch /tmp/testfile\r"
expect "/ #"
send "echo test123 > /tmp/testfile\r"
expect "/ #"
send "cat /tmp/testfile\r"
expect {
    timeout { puts "FAIL: file I/O timeout"; exit 1 }
    "test123" { puts "PASS: File I/O works (touch/echo/cat)" }
    eof { puts "FAIL: Unexpected EOF during file I/O"; exit 1 }
}
expect "/ #"

# Test 4: Exit code
send "sh -c 'exit 42'\r"
expect "/ #"
send "echo \$?\r"
expect {
    timeout { puts "FAIL: exit code timeout"; exit 1 }
    "42" { puts "PASS: Exit codes work" }
    eof { puts "FAIL: Unexpected EOF during exit code test"; exit 1 }
}
expect "/ #"

# Test 5: Pipe
send "echo hello | cat\r"
expect {
    timeout { puts "FAIL: pipe timeout"; exit 1 }
    "hello" { puts "PASS: Pipes work" }
    eof { puts "FAIL: Unexpected EOF during pipe test"; exit 1 }
}
expect "/ #"

# Test 6: Write to /dev/console
send "echo direct_console > /dev/console\r"
expect {
    timeout { puts "FAIL: /dev/console timeout"; exit 1 }
    "direct_console" { puts "PASS: /dev/console works" }
    eof { puts "FAIL: Unexpected EOF during console test"; exit 1 }
}
expect "/ #"

# All tests passed!
puts ""
puts "======================================="
puts "ALL TESTS PASSED - Phase A1 COMPLETE!"
puts "======================================="

# Shutdown
send "poweroff\r"
expect eof
EXPECT_EOF

chmod +x "$EXPECT_SCRIPT"

# Run expect script
"$EXPECT_SCRIPT" "$KERNEL"

EXIT_CODE=$?
if [ $EXIT_CODE -eq 0 ]; then
    echo ""
    echo "✓ Phase A1 acceptance tests PASSED"
else
    echo ""
    echo "✗ Phase A1 acceptance tests FAILED (exit code: $EXIT_CODE)"
fi

exit $EXIT_CODE
