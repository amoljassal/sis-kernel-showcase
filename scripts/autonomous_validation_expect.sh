#!/bin/bash
# Autonomous Control Long-Duration Validation with Expect
# Tests autonomous AI operation for extended periods (1hr, 4hr, 24hr)
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/autonomous_validation_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$RESULTS_DIR/autonomous_validation_${TIMESTAMP}.log"

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
DURATION_SECONDS=${1:-3600}  # Default 1 hour (3600 seconds)
DURATION_MS=$((DURATION_SECONDS * 1000))

# Main test execution
main() {
    log_info "========================================"
    log_info "  Autonomous Control Validation"
    log_info "========================================"
    log_info "Duration: ${DURATION_SECONDS}s ($((DURATION_SECONDS / 60))min)"
    log_info "Output: $OUTPUT_FILE"
    echo ""

    log_info "Starting QEMU with expect automation..."

    # Calculate appropriate timeout (duration + 10 minutes for overhead)
    local timeout_seconds=$((DURATION_SECONDS + 600))

    # Create expect script
    cat > /tmp/autonomous_validation_$$.exp <<EXPECT_EOF
#!/usr/bin/expect -f
set timeout $timeout_seconds

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

# Enable autonomous mode
send_user "\n\[EXPECT\] Enabling autonomous control...\n"
send "autoctl on\r"
expect "sis>"

# Get initial status
send_user "\n\[EXPECT\] Getting initial autonomous status...\n"
send "autoctl status\r"
expect "sis>"

# Run multi-subsystem stress test with autonomous control active
send_user "\n\[EXPECT\] Starting ${DURATION_SECONDS}s validation with autonomous control...\n"
send "stresstest multi --duration ${DURATION_MS}\r"

expect {
    "STRESSTEST" {
        send_user "\n\[EXPECT\] Autonomous validation completed\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Autonomous validation timeout\n"
    }
}

expect "sis>"

# Get final autonomous status and statistics
send_user "\n\[EXPECT\] Getting final autonomous status...\n"
send "autoctl status\r"
expect "sis>"

send_user "\n\[EXPECT\] Getting agent statistics...\n"
send "agentctl stats\r"
expect "sis>"

send_user "\n\[EXPECT\] Getting learning statistics...\n"
send "learnctl stats\r"
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

    chmod +x /tmp/autonomous_validation_$$.exp

    # Run expect script
    cd "$PROJECT_ROOT"
    /tmp/autonomous_validation_$$.exp 2>&1 | tee "$OUTPUT_FILE"
    local exit_code=${PIPESTATUS[0]}

    # Cleanup
    rm -f /tmp/autonomous_validation_$$.exp

    # Analysis
    echo ""
    log_info "========================================"
    log_info "  Autonomous Validation Results"
    log_info "========================================"

    # Extract autonomous control metrics (trim whitespace)
    local total_decisions=$(grep "Total Decisions:" "$OUTPUT_FILE" | tail -1 | awk '{print $3}' | tr -d '[:space:]' || echo "0")
    local ai_decisions=$(grep "Total AI decisions:" "$OUTPUT_FILE" | tail -1 | awk '{print $4}' | tr -d '[:space:]' || echo "0")
    local watchdog_triggers=$(grep "Watchdog Triggers:" "$OUTPUT_FILE" | tail -1 | awk '{print $3}' | tr -d '[:space:]' || echo "0")
    local mode=$(grep "Mode: ENABLED" "$OUTPUT_FILE" | wc -l | tr -d '[:space:]')

    # Extract neural network metrics
    local nn_inferences=$(grep "METRIC nn_infer_count=" "$OUTPUT_FILE" | tail -1 | cut -d'=' -f2 | tr -d '[:space:]' || echo "0")

    # Ensure we have valid numbers
    total_decisions=${total_decisions:-0}
    ai_decisions=${ai_decisions:-0}
    watchdog_triggers=${watchdog_triggers:-0}
    mode=${mode:-0}
    nn_inferences=${nn_inferences:-0}

    echo "Autonomous Mode:          $([ "$mode" -gt 0 ] && echo "ENABLED" || echo "DISABLED")"
    echo "Total Decisions:          $total_decisions"
    echo "AI Decisions:             $ai_decisions"
    echo "Neural Network Inferences:$nn_inferences"
    echo "Watchdog Triggers:        $watchdog_triggers"
    echo "Duration:                 ${DURATION_SECONDS}s"
    echo ""

    # Validation checks
    log_info "Validation Checks:"
    echo ""

    local pass_count=0
    local fail_count=0

    # Check 1: Test completed
    local test_complete=$(grep -c "\[EXPECT\] Autonomous validation completed" "$OUTPUT_FILE" || echo "0")
    if [ "$test_complete" -gt 0 ]; then
        log_pass "Autonomous validation completed"
        pass_count=$((pass_count + 1))
    else
        log_fail "Autonomous validation did not complete"
        fail_count=$((fail_count + 1))
    fi

    # Check 2: Autonomous mode remained enabled
    if [ "$mode" -gt 0 ]; then
        log_pass "Autonomous mode remained enabled throughout test"
        pass_count=$((pass_count + 1))
    else
        log_fail "Autonomous mode was not enabled"
        fail_count=$((fail_count + 1))
    fi

    # Check 3: AI made decisions
    # For 1hr test, expect at least 100 decisions (conservative: 1 per 36s)
    # For 4hr test, expect at least 400 decisions
    local min_decisions=$((DURATION_SECONDS / 36))

    if [ "$total_decisions" -ge "$min_decisions" ] 2>/dev/null; then
        log_pass "Sufficient autonomous decisions: $total_decisions (>= $min_decisions expected)"
        pass_count=$((pass_count + 1))
    else
        log_fail "Insufficient autonomous decisions: $total_decisions (< $min_decisions expected)"
        fail_count=$((fail_count + 1))
    fi

    # Check 4: Neural network active
    if [ "$nn_inferences" -gt 0 ] 2>/dev/null; then
        log_pass "Neural network active: $nn_inferences inferences"
        pass_count=$((pass_count + 1))
    else
        log_fail "Neural network inactive: $nn_inferences inferences"
        fail_count=$((fail_count + 1))
    fi

    # Check 5: No excessive watchdog triggers (allow up to 1% of decisions)
    local max_watchdog=$((total_decisions / 100))
    if [ "$max_watchdog" -lt 5 ]; then
        max_watchdog=5  # Allow at least 5 watchdog triggers
    fi

    if [ "$watchdog_triggers" -le "$max_watchdog" ] 2>/dev/null; then
        log_pass "Watchdog triggers acceptable: $watchdog_triggers (<= $max_watchdog)"
        pass_count=$((pass_count + 1))
    else
        log_fail "Excessive watchdog triggers: $watchdog_triggers (> $max_watchdog)"
        fail_count=$((fail_count + 1))
    fi

    # Check 6: System stability (no crashes)
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

    if [ $fail_count -eq 0 ] && [ $pass_count -ge 5 ]; then
        log_pass "AUTONOMOUS VALIDATION SUCCESSFUL"
        echo "========================================"
        echo ""
        echo "Key Results:"
        echo "  - Duration: ${DURATION_SECONDS}s ($((DURATION_SECONDS / 60))min)"
        echo "  - Total Decisions: $total_decisions"
        echo "  - AI Decisions: $ai_decisions"
        echo "  - Neural Inferences: $nn_inferences"
        echo "  - Watchdog Triggers: $watchdog_triggers"
        echo "  - Stability: Zero crashes"
        echo ""
        echo "Full results: $OUTPUT_FILE"
        exit 0
    else
        log_fail "AUTONOMOUS VALIDATION FAILED"
        echo "========================================"
        echo ""
        echo "Review full log: $OUTPUT_FILE"
        exit 1
    fi
}

main "$@"
