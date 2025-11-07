#!/usr/bin/env bash
# Chaos Testing Runner
# Phase 3.1 - Production Readiness Plan
#
# Runs all chaos test scenarios to verify graceful failure handling

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

# Configuration
TIMEOUT=${TIMEOUT:-120}
QMP_SOCK=${QMP_SOCK:-/tmp/sis-chaos-qmp.sock}
TEST_LOG=${TEST_LOG:-/tmp/sis-chaos.log}
TEST_DIR="$ROOT_DIR/tests/chaos"

echo "=================================================="
echo "SIS Kernel Chaos Testing Suite"
echo "=================================================="
echo "Test Dir:   $TEST_DIR"
echo "Timeout:    ${TIMEOUT}s"
echo "=================================================="
echo ""

# Check if chaos feature is enabled
if ! grep -q 'chaos' "$ROOT_DIR/Cargo.toml" 2>/dev/null; then
    echo "[!] WARNING: 'chaos' feature may not be enabled in Cargo.toml"
    echo "[*] Build with: SIS_FEATURES=\"llm,crypto-real,chaos\" ./scripts/uefi_run.sh"
fi

# Clean up old files
rm -f "$QMP_SOCK" "$TEST_LOG"

# Start QEMU with QMP and chaos feature
echo "[*] Starting kernel with chaos feature enabled..."
export QMP=1
export QMP_SOCK
export BRINGUP=1
export SIS_FEATURES="llm,crypto-real,chaos"

"$SCRIPT_DIR/uefi_run.sh" > "$TEST_LOG" 2>&1 &
QEMU_PID=$!

echo "[*] QEMU PID: $QEMU_PID"

# Cleanup function
cleanup() {
    echo ""
    echo "[*] Cleaning up..."
    if [[ -n "${QEMU_PID:-}" ]] && kill -0 "$QEMU_PID" 2>/dev/null; then
        "$SCRIPT_DIR/qmp_input.py" --socket "$QMP_SOCK" quit 2>/dev/null || true
        sleep 1
        if kill -0 "$QEMU_PID" 2>/dev/null; then
            kill -9 "$QEMU_PID" 2>/dev/null || true
        fi
    fi
    rm -f "$QMP_SOCK"
}

trap cleanup EXIT INT TERM

# Wait for shell prompt
echo "[*] Waiting for shell prompt..."
START_TIME=$(date +%s)

while true; do
    ELAPSED=$(($(date +%s) - START_TIME))
    if [[ $ELAPSED -gt $TIMEOUT ]]; then
        echo "[!] ERROR: Timeout waiting for shell"
        exit 1
    fi

    if grep -q "LAUNCHING SHELL\|Shell ready\|sis>" "$TEST_LOG"; then
        echo "[*] Shell ready!"
        break
    fi

    if grep -qi "KERNEL PANIC" "$TEST_LOG"; then
        echo "[!] ERROR: Kernel panic during boot"
        tail -100 "$TEST_LOG"
        exit 1
    fi

    sleep 0.5
done

sleep 2

# Helper functions (same as automated_shell_tests.sh)
send_command() {
    local cmd="$1"
    echo "    > $cmd"
    "$SCRIPT_DIR/qmp_input.py" --socket "$QMP_SOCK" send-command "$cmd" 2>/dev/null || {
        echo "    [!] Failed to send command"
        return 1
    }
    sleep 1
}

expect_output() {
    local pattern="$1"
    local timeout="${2:-5}"
    local start=$(date +%s)

    while true; do
        if grep -qE "$pattern" "$TEST_LOG"; then
            return 0
        fi

        if [[ $(($(date +%s) - start)) -gt $timeout ]]; then
            echo "    [!] Expected pattern not found: $pattern"
            return 1
        fi

        sleep 0.2
    done
}

expect_no_output() {
    local pattern="$1"
    if grep -qE "$pattern" "$TEST_LOG"; then
        echo "    [!] Unexpected pattern found: $pattern"
        return 1
    fi
    return 0
}

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name="$1"
    local test_file="$2"

    echo ""
    echo "=================================================="
    echo "Test: $test_name"
    echo "=================================================="

    ((TESTS_RUN++)) || true

    # Source the test file to get run_test_impl
    if source "$test_file"; then
        if declare -f run_test_impl >/dev/null; then
            if run_test_impl; then
                echo "[✓] PASS: $test_name"
                ((TESTS_PASSED++)) || true
                return 0
            fi
        fi
    fi

    echo "[✗] FAIL: $test_name"
    ((TESTS_FAILED++)) || true
    return 1
}

# Verify chaos command is available
echo "[*] Verifying chaos feature..."
send_command "chaos" || {
    echo "[!] ERROR: 'chaos' command not found"
    echo "[!] Kernel may not be built with --features chaos"
    exit 1
}

expect_output "Usage: chaos" || {
    echo "[!] ERROR: Chaos command not working properly"
    exit 1
}

echo "[*] Chaos feature verified!"

# Run all chaos tests
echo ""
echo "[*] Running chaos test scenarios..."

for test_file in "$TEST_DIR"/test_*.sh; do
    if [[ -f "$test_file" ]]; then
        test_name=$(basename "$test_file" .sh)
        run_test "$test_name" "$test_file" || true

        # Give system time to recover between tests
        sleep 2
    fi
done

# Final panic check
if grep -qi "KERNEL PANIC" "$TEST_LOG"; then
    echo ""
    echo "[!] WARNING: Kernel panic detected during chaos tests"
    ((TESTS_FAILED++)) || true
fi

# Summary
echo ""
echo "=================================================="
echo "Chaos Testing Summary"
echo "=================================================="
echo "Tests run:    $TESTS_RUN"
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $TESTS_FAILED"
echo "=================================================="

if [[ $TESTS_FAILED -eq 0 ]]; then
    echo "[✓] ALL CHAOS TESTS PASSED"
    echo "[*] System handles failures gracefully!"
    exit 0
else
    echo "[✗] SOME CHAOS TESTS FAILED"
    echo "[*] Log file: $TEST_LOG"
    exit 1
fi
