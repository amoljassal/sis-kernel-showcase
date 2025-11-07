#!/bin/bash
# Phase A0 Acceptance Tests
# Tests basic syscall infrastructure, exception vectors, and error handling

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "===== Phase A0 Acceptance Tests ====="
echo "Project root: $PROJECT_ROOT"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass_count=0
fail_count=0

test_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((pass_count++))
}

test_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((fail_count++))
}

test_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Test 1: QEMU boot with VBAR verification
echo "Test 1: QEMU boot with exception vector initialization"
test_info "Building kernel..."

cd "$PROJECT_ROOT"
if cargo build --release 2>&1 | tee /tmp/phase_a0_build.log; then
    test_info "Build successful"
else
    test_fail "Kernel build failed"
    echo "Build log:"
    cat /tmp/phase_a0_build.log
    exit 1
fi

# Start QEMU in background and capture output
test_info "Starting QEMU..."
timeout 10s qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a57 \
    -smp 1 \
    -m 128M \
    -nographic \
    -kernel target/aarch64-unknown-none/release/kernel \
    > /tmp/phase_a0_qemu.log 2>&1 || true

# Check for VBAR initialization message
if grep -q "VBAR_EL1 set to" /tmp/phase_a0_qemu.log; then
    test_pass "Exception vectors initialized (VBAR_EL1 set)"
else
    test_fail "VBAR_EL1 initialization message not found in boot log"
    echo "Boot log excerpt:"
    head -20 /tmp/phase_a0_qemu.log
fi

# Check that boot completes without panic
if grep -q "PANIC" /tmp/phase_a0_qemu.log; then
    test_fail "Kernel panic detected during boot"
    echo "Panic context:"
    grep -A 5 "PANIC" /tmp/phase_a0_qemu.log
else
    test_pass "Boot completed without panic"
fi

# Test 2: Syscall dispatcher integration
echo ""
echo "Test 2: Syscall dispatcher integration"
test_info "Checking syscall infrastructure..."

# Verify syscall module exports are present
if grep -rq "pub fn syscall_dispatcher" "$PROJECT_ROOT/crates/kernel/src/syscall/"; then
    test_pass "Syscall dispatcher function present"
else
    test_fail "Syscall dispatcher function not found"
fi

# Verify negative errno mapping
if grep -rq "as_isize.*negative" "$PROJECT_ROOT/crates/kernel/src/lib/error.rs"; then
    test_pass "Negative errno mapping implemented"
else
    test_fail "Negative errno mapping not found"
fi

# Test 3: ENOSYS error handling (theoretical - would need userspace)
echo ""
echo "Test 3: Error handling verification"
test_info "Checking ENOSYS handling for unimplemented syscalls..."

# Verify ENOSYS is defined
if grep -q "ENOSYS" "$PROJECT_ROOT/crates/kernel/src/lib/error.rs"; then
    test_pass "ENOSYS error code defined"
else
    test_fail "ENOSYS error code not found"
fi

# Verify default case in dispatcher returns ENOSYS
if grep -A 2 "_ =>" "$PROJECT_ROOT/crates/kernel/src/syscall/mod.rs" | grep -q "ENOSYS"; then
    test_pass "Dispatcher returns ENOSYS for unimplemented syscalls"
else
    test_fail "ENOSYS not returned for unimplemented syscalls"
fi

# Test 4: TrapFrame structure verification
echo ""
echo "Test 4: TrapFrame and exception handling"
test_info "Checking TrapFrame structure..."

if grep -q "pub struct TrapFrame" "$PROJECT_ROOT/crates/kernel/src/arch/aarch64/trap.rs"; then
    test_pass "TrapFrame structure defined"
else
    test_fail "TrapFrame structure not found"
fi

# Verify vectors.S exists and has proper alignment
if [ -f "$PROJECT_ROOT/crates/kernel/src/arch/aarch64/vectors.S" ]; then
    if grep -q ".align 11" "$PROJECT_ROOT/crates/kernel/src/arch/aarch64/vectors.S"; then
        test_pass "Exception vector table has correct alignment (2048 bytes)"
    else
        test_fail "Exception vector table alignment incorrect"
    fi
else
    test_fail "vectors.S file not found"
fi

# Test 5: MVP syscalls present
echo ""
echo "Test 5: MVP syscall implementations"
test_info "Checking for read, write, exit, getpid syscalls..."

mvp_syscalls=("sys_read" "sys_write" "sys_exit" "sys_getpid")
for syscall in "${mvp_syscalls[@]}"; do
    if grep -q "fn $syscall" "$PROJECT_ROOT/crates/kernel/src/syscall/mod.rs"; then
        test_pass "MVP syscall: $syscall"
    else
        test_fail "MVP syscall missing: $syscall"
    fi
done

# Summary
echo ""
echo "====================================="
echo "Phase A0 Test Summary"
echo "====================================="
echo -e "${GREEN}Passed: $pass_count${NC}"
if [ $fail_count -gt 0 ]; then
    echo -e "${RED}Failed: $fail_count${NC}"
else
    echo -e "Failed: $fail_count"
fi
echo "====================================="

if [ $fail_count -eq 0 ]; then
    echo -e "${GREEN}All Phase A0 acceptance tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some Phase A0 tests failed.${NC}"
    exit 1
fi
