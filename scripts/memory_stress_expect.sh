#!/bin/bash
# Memory Stress Test with Expect
# Tests memory subsystem under high pressure (95%) for extended duration
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/stress_test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$RESULTS_DIR/memory_stress_${TIMESTAMP}.log"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Check if expect is installed
if ! command -v expect &> /dev/null; then
    log_fail "expect is not installed. Install with: brew install expect"
    exit 1
fi

# Parse arguments
DURATION_MS=${1:-600000}  # Default 10 minutes (600,000 ms)
TARGET_PRESSURE=${2:-95}   # Default 95% memory pressure

# Main test execution
main() {
    log_info "========================================"
    log_info "  Memory Stress Test (Expect)"
    log_info "========================================"
    log_info "Duration: ${DURATION_MS}ms ($((DURATION_MS / 1000))s)"
    log_info "Target Pressure: ${TARGET_PRESSURE}%"
    log_info "Output: $OUTPUT_FILE"
    echo ""

    log_info "Starting QEMU with expect automation..."

    # Create expect script
    cat > /tmp/memory_stress_$$.exp <<EXPECT_EOF
#!/usr/bin/expect -f
set timeout 900

# Start QEMU
spawn env SIS_FEATURES=llm,crypto-real BRINGUP=1 ./scripts/uefi_run.sh build

# Wait for shell prompt
expect {
    "sis>" {
        send_user "\n\[EXPECT\] Shell ready\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Timeout waiting for shell\n"
        exit 1
    }
}

# Run memory stress test
send_user "\n\[EXPECT\] Starting memory stress test (${DURATION_MS}ms at ${TARGET_PRESSURE}% pressure)...\n"
send "stresstest memory --duration ${DURATION_MS} --target-pressure ${TARGET_PRESSURE}\r"

expect {
    "STRESSTEST" {
        send_user "\n\[EXPECT\] Memory stress test completed\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Memory stress timeout\n"
    }
}

expect "sis>"

# Get stress test report
send_user "\n\[EXPECT\] Getting stress test report...\n"
send "stresstest report\r"
expect "sis>"

# Exit QEMU
send_user "\n\[EXPECT\] Exiting QEMU...\n"
send "\x01"
send "x"

expect eof
EXPECT_EOF

    chmod +x /tmp/memory_stress_$$.exp

    # Run expect script
    cd "$PROJECT_ROOT"
    /tmp/memory_stress_$$.exp 2>&1 | tee "$OUTPUT_FILE"
    local exit_code=${PIPESTATUS[0]}

    # Cleanup
    rm -f /tmp/memory_stress_$$.exp

    # Analysis
    echo ""
    log_info "========================================"
    log_info "  Memory Stress Test Results"
    log_info "========================================"

    # Extract metrics (trim whitespace)
    local peak_pressure=$(grep "peak_pressure=" "$OUTPUT_FILE" | tail -1 | grep -o "peak_pressure=[0-9]\+" | cut -d'=' -f2 | tr -d '[:space:]' || echo "0")
    local oom_events=$(grep "oom_events=" "$OUTPUT_FILE" | tail -1 | grep -o "oom_events=[0-9]\+" | cut -d'=' -f2 | tr -d '[:space:]' || echo "0")
    local compactions=$(grep "compactions=" "$OUTPUT_FILE" | tail -1 | grep -o "compactions=[0-9]\+" | cut -d'=' -f2 | tr -d '[:space:]' || echo "0")
    local duration=$(grep "duration_ms=" "$OUTPUT_FILE" | tail -1 | grep -o "duration_ms=[0-9]\+" | cut -d'=' -f2 | tr -d '[:space:]' || echo "0")

    # Ensure we have valid numbers
    peak_pressure=${peak_pressure:-0}
    oom_events=${oom_events:-0}
    compactions=${compactions:-0}
    duration=${duration:-0}

    echo "Peak Memory Pressure:     $peak_pressure%"
    echo "OOM Events:               $oom_events"
    echo "Compaction Triggers:      $compactions"
    echo "Actual Duration:          ${duration}ms"
    echo ""

    # Validation checks
    log_info "Validation Checks:"
    echo ""

    local pass_count=0
    local fail_count=0

    # Check 1: Test completed
    local test_complete=$(grep -c "\[EXPECT\] Memory stress test completed" "$OUTPUT_FILE" || echo "0")
    if [ "$test_complete" -gt 0 ]; then
        log_pass "Memory stress test completed"
        pass_count=$((pass_count + 1))
    else
        log_fail "Memory stress test did not complete"
        fail_count=$((fail_count + 1))
    fi

    # Check 2: Achieved target pressure (within 10%)
    local pressure_target=$((TARGET_PRESSURE - 10))
    if [ "$peak_pressure" -ge "$pressure_target" ] 2>/dev/null; then
        log_pass "Reached target pressure: $peak_pressure% (target: ${TARGET_PRESSURE}%)"
        pass_count=$((pass_count + 1))
    else
        log_fail "Did not reach target pressure: $peak_pressure% (target: ${TARGET_PRESSURE}%)"
        fail_count=$((fail_count + 1))
    fi

    # Check 3: No excessive OOM events (allow up to 5 for 95% pressure)
    local max_oom=5
    if [ "$TARGET_PRESSURE" -ge 95 ]; then
        max_oom=10  # Allow more OOMs at very high pressure
    fi

    if [ "$oom_events" -le "$max_oom" ] 2>/dev/null; then
        log_pass "OOM events within acceptable range: $oom_events (<= $max_oom)"
        pass_count=$((pass_count + 1))
    else
        log_fail "Excessive OOM events: $oom_events (> $max_oom)"
        fail_count=$((fail_count + 1))
    fi

    # Check 4: Test duration reasonable (within 20% of target)
    local duration_target_min=$((DURATION_MS * 80 / 100))
    local duration_target_max=$((DURATION_MS * 120 / 100))

    if [ "$duration" -ge "$duration_target_min" ] && [ "$duration" -le "$duration_target_max" ] 2>/dev/null; then
        log_pass "Test duration reasonable: ${duration}ms (target: ${DURATION_MS}ms)"
        pass_count=$((pass_count + 1))
    else
        log_fail "Test duration out of range: ${duration}ms (target: ${DURATION_MS}ms)"
        fail_count=$((fail_count + 1))
    fi

    # Check 5: System stability (no crashes)
    local crashes=$(grep -i "kernel panic\|PANIC" "$OUTPUT_FILE" 2>/dev/null | wc -l | tr -d '[:space:]')
    crashes=${crashes:-0}
    if ! [[ "$crashes" =~ ^[0-9]+$ ]]; then
        crashes=0
    fi

    if [ "$crashes" -eq 0 ] 2>/dev/null; then
        log_pass "System stable (no crashes)"
        pass_count=$((pass_count + 1))
    else
        log_fail "System crashes detected: $crashes"
        fail_count=$((fail_count + 1))
    fi

    # Overall result
    echo ""
    echo "========================================"
    echo "Overall Result: $pass_count passed, $fail_count failed"
    echo ""

    if [ $fail_count -eq 0 ] && [ $pass_count -ge 4 ]; then
        log_pass "MEMORY STRESS TEST SUCCESSFUL"
        echo "========================================"
        echo ""
        echo "Key Results:"
        echo "  - Peak Pressure: $peak_pressure%"
        echo "  - OOM Events: $oom_events"
        echo "  - Compactions: $compactions"
        echo "  - Duration: ${duration}ms"
        echo ""
        echo "Full results: $OUTPUT_FILE"
        exit 0
    else
        log_fail "MEMORY STRESS TEST FAILED"
        echo "========================================"
        echo ""
        echo "Review full log: $OUTPUT_FILE"
        exit 1
    fi
}

main "$@"
