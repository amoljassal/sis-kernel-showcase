#!/usr/bin/env bash
# Automated Shell Test Harness
# Phase 1.2 - Production Readiness Plan
#
# Runs kernel, waits for shell, injects commands, validates output

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

# Configuration
TIMEOUT=${TIMEOUT:-60}
QMP_SOCK=${QMP_SOCK:-/tmp/sis-test-qmp.sock}
TEST_LOG=${TEST_LOG:-/tmp/sis-test.log}
TEST_DIR=${TEST_DIR:-$ROOT_DIR/tests/shell}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================================="
echo "SIS Kernel Automated Shell Tests"
echo "=================================================="
echo "Timeout:    ${TIMEOUT}s"
echo "QMP Socket: $QMP_SOCK"
echo "Test Log:   $TEST_LOG"
echo "Test Dir:   $TEST_DIR"
echo "=================================================="
echo ""

# Clean up old files
rm -f "$QMP_SOCK" "$TEST_LOG"

# Start QEMU with QMP in background
echo "[*] Starting kernel with QMP..."
export QMP=1
export QMP_SOCK
export BRINGUP=1
export SIS_FEATURES="${SIS_FEATURES:-llm,crypto-real}"

"$SCRIPT_DIR/uefi_run.sh" > "$TEST_LOG" 2>&1 &
QEMU_PID=$!

echo "[*] QEMU PID: $QEMU_PID"

# Cleanup function
cleanup() {
    echo ""
    echo "[*] Cleaning up..."
    if [[ -n "${QEMU_PID:-}" ]] && kill -0 "$QEMU_PID" 2>/dev/null; then
        # Try graceful shutdown via QMP
        "$SCRIPT_DIR/qmp_input.py" --socket "$QMP_SOCK" quit 2>/dev/null || true
        sleep 1

        # Force kill if still running
        if kill -0 "$QEMU_PID" 2>/dev/null; then
            echo "[*] Force killing QEMU..."
            kill -9 "$QEMU_PID" 2>/dev/null || true
        fi
    fi
    rm -f "$QMP_SOCK"
}

trap cleanup EXIT INT TERM

# Wait for shell prompt with timeout
echo "[*] Waiting for shell prompt..."
START_TIME=$(date +%s)

while true; do
    ELAPSED=$(($(date +%s) - START_TIME))
    if [[ $ELAPSED -gt $TIMEOUT ]]; then
        echo -e "${RED}[✗] TIMEOUT: Shell prompt not detected after ${TIMEOUT}s${NC}"
        echo ""
        echo "Recent log output:"
        tail -50 "$TEST_LOG"
        exit 1
    fi

    if grep -q "LAUNCHING SHELL\|Shell ready\|sis>" "$TEST_LOG"; then
        echo -e "${GREEN}[✓] Shell prompt detected (${ELAPSED}s)${NC}"
        break
    fi

    # Check for kernel panic
    if grep -qi "KERNEL PANIC" "$TEST_LOG"; then
        echo -e "${RED}[✗] KERNEL PANIC detected${NC}"
        tail -100 "$TEST_LOG"
        exit 1
    fi

    sleep 0.5
done

# Wait a bit more for shell to be fully ready
sleep 2

# Helper functions
send_command() {
    local cmd="$1"
    echo "[*] Sending command: $cmd"
    "$SCRIPT_DIR/qmp_input.py" --socket "$QMP_SOCK" send-command "$cmd" || {
        echo -e "${RED}[✗] Failed to send command${NC}"
        return 1
    }
    sleep 1  # Wait for command to execute
}

expect_output() {
    local pattern="$1"
    local timeout="${2:-5}"
    local start=$(date +%s)

    echo "    Expecting: $pattern"

    while true; do
        if grep -qE "$pattern" "$TEST_LOG"; then
            echo -e "    ${GREEN}✓ Found${NC}"
            return 0
        fi

        if [[ $(($(date +%s) - start)) -gt $timeout ]]; then
            echo -e "    ${RED}✗ Not found (timeout: ${timeout}s)${NC}"
            return 1
        fi

        sleep 0.2
    done
}

expect_no_output() {
    local pattern="$1"
    echo "    Expecting NOT: $pattern"

    if grep -qE "$pattern" "$TEST_LOG"; then
        echo -e "    ${RED}✗ Found (should not be present)${NC}"
        return 1
    else
        echo -e "    ${GREEN}✓ Not found${NC}"
        return 0
    fi
}

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name="$1"
    local test_func="$2"

    echo ""
    echo "=================================================="
    echo "Test: $test_name"
    echo "=================================================="

    ((TESTS_RUN++)) || true

    if $test_func; then
        echo -e "${GREEN}[✓] PASS: $test_name${NC}"
        ((TESTS_PASSED++)) || true
        return 0
    else
        echo -e "${RED}[✗] FAIL: $test_name${NC}"
        ((TESTS_FAILED++)) || true
        return 1
    fi
}

# Built-in test functions
test_help() {
    send_command "help" || return 1
    expect_output "Available commands" || return 1
    return 0
}

test_version() {
    send_command "version" || return 1
    expect_output "SIS Kernel" || return 1
    return 0
}

test_memstats() {
    send_command "memstats" || return 1
    expect_output "Heap.*MiB" || return 1
    return 0
}

test_netstat() {
    send_command "netstat" || return 1
    expect_output "Interface.*UP" || return 1
    return 0
}

test_uptime() {
    send_command "uptime" || return 1
    expect_output "Uptime|uptime" || return 1
    return 0
}

# Run built-in tests
echo ""
echo "[*] Running built-in test suite..."

run_test "help command" test_help || true
run_test "version command" test_version || true
run_test "memstats command" test_memstats || true
run_test "netstat command" test_netstat || true
run_test "uptime command" test_uptime || true

# Run modular tests from test directory
if [[ -d "$TEST_DIR" ]]; then
    echo ""
    echo "[*] Running modular tests from: $TEST_DIR"

    for test_file in "$TEST_DIR"/test_*.sh; do
        if [[ -f "$test_file" ]]; then
            test_name=$(basename "$test_file" .sh)

            # Source the test file to get its test function
            # shellcheck disable=SC1090
            if source "$test_file"; then
                # Test files should define a function named "run_test"
                if declare -f run_test_impl >/dev/null; then
                    run_test "$test_name" run_test_impl || true
                else
                    echo -e "${YELLOW}[!] SKIP: $test_file (no run_test_impl function)${NC}"
                fi
            else
                echo -e "${YELLOW}[!] SKIP: $test_file (source failed)${NC}"
            fi
        fi
    done
else
    echo "[*] No modular tests directory found (expected: $TEST_DIR)"
fi

# Check for panics during testing
if grep -qi "KERNEL PANIC" "$TEST_LOG"; then
    echo ""
    echo -e "${RED}[!] WARNING: Kernel panic detected during tests${NC}"
    ((TESTS_FAILED++)) || true
fi

# Summary
echo ""
echo "=================================================="
echo "Test Summary"
echo "=================================================="
echo "Tests run:    $TESTS_RUN"
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"
echo "=================================================="

if [[ $TESTS_FAILED -eq 0 ]]; then
    echo -e "${GREEN}[✓] ALL TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}[✗] SOME TESTS FAILED${NC}"
    echo ""
    echo "Log file: $TEST_LOG"
    exit 1
fi
