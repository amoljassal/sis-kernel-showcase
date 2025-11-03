#!/bin/bash
# Comprehensive Benchmark Suite with Expect
# Runs all Week 12 benchmarks programmatically
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$RESULTS_DIR/benchmark_suite_${TIMESTAMP}.log"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
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
    log_error "expect is not installed. Install with: brew install expect"
    exit 1
fi

# Parse arguments
DURATION=${1:-15}  # Default 15 seconds for full benchmark

# Main test execution
main() {
    log_info "========================================"
    log_info "  Comprehensive Benchmark Suite"
    log_info "========================================"
    log_info "Duration: ${DURATION}s per test"
    log_info "Output: $OUTPUT_FILE"
    echo ""

    log_info "Starting QEMU with expect automation..."

    # Create expect script
    cat > /tmp/benchmark_expect_$$.exp <<EXPECT_EOF
#!/usr/bin/expect -f
set timeout 300

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

# Test 1: Memory benchmark
send_user "\n\[EXPECT\] ========== TEST 1: Memory Benchmark ==========\n"
send "benchmark memory 10\r"
expect {
    "SUMMARY" {
        send_user "\n\[EXPECT\] Memory benchmark completed\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Memory benchmark timeout\n"
    }
}
expect "sis>"

# Test 2: Commands benchmark
send_user "\n\[EXPECT\] ========== TEST 2: Commands Benchmark ==========\n"
send "benchmark commands 5\r"
expect {
    "SUMMARY" {
        send_user "\n\[EXPECT\] Commands benchmark completed\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Commands benchmark timeout\n"
    }
}
expect "sis>"

# Test 3: Network benchmark
send_user "\n\[EXPECT\] ========== TEST 3: Network Benchmark ==========\n"
send "benchmark network 10\r"
expect {
    "SUMMARY" {
        send_user "\n\[EXPECT\] Network benchmark completed\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Network benchmark timeout\n"
    }
}
expect "sis>"

# Test 4: Full benchmark
send_user "\n\[EXPECT\] ========== TEST 4: Full Benchmark ==========\n"
send "benchmark full $DURATION\r"
expect {
    "SUMMARY" {
        send_user "\n\[EXPECT\] Full benchmark completed\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Full benchmark timeout\n"
    }
}
expect "sis>"

# Test 5: Benchmark report
send_user "\n\[EXPECT\] ========== TEST 5: Benchmark Report ==========\n"
send "benchmark report\r"
expect {
    "SUMMARY" {
        send_user "\n\[EXPECT\] Report generated\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Report timeout\n"
    }
}
expect "sis>"

# Exit QEMU
send_user "\n\[EXPECT\] All benchmarks complete. Exiting...\n"
send "\x01"
send "x"

expect eof
EXPECT_EOF

    chmod +x /tmp/benchmark_expect_$$.exp

    # Run expect script
    cd "$PROJECT_ROOT"
    /tmp/benchmark_expect_$$.exp 2>&1 | tee "$OUTPUT_FILE"
    local exit_code=${PIPESTATUS[0]}

    # Cleanup
    rm -f /tmp/benchmark_expect_$$.exp

    # Analysis
    echo ""
    log_info "========================================"
    log_info "  Benchmark Results Analysis"
    log_info "========================================"

    # Extract key metrics (trim whitespace)
    local nn_inferences=$(grep "METRIC nn_infer_count=" "$OUTPUT_FILE" | tail -1 | cut -d'=' -f2 | tr -d '[:space:]' || echo "0")
    local commands_executed=$(grep "Commands Executed:" "$OUTPUT_FILE" | tail -1 | awk '{print $3}' | tr -d '[:space:]' || echo "0")
    local packets_baseline=$(grep "Baseline (no AI):" -A 1 "$OUTPUT_FILE" | grep "Packets Sent:" | tail -1 | awk '{print $4}' | tr -d '[:space:]' || echo "0")
    local packets_ai=$(grep "With AI Enabled:" -A 1 "$OUTPUT_FILE" | grep "Packets Sent:" | tail -1 | awk '{print $4}' | tr -d '[:space:]' || echo "0")
    local oom_baseline=$(grep "Baseline (no AI):" -A 3 "$OUTPUT_FILE" | grep "OOM Events:" | tail -1 | awk '{print $4}' | tr -d '[:space:]' || echo "0")
    local oom_ai=$(grep "With AI Enabled:" -A 3 "$OUTPUT_FILE" | grep "OOM Events:" | tail -1 | awk '{print $4}' | tr -d '[:space:]' || echo "0")

    # Ensure we have valid numbers (default to 0 if empty)
    nn_inferences=${nn_inferences:-0}
    commands_executed=${commands_executed:-0}
    packets_baseline=${packets_baseline:-0}
    packets_ai=${packets_ai:-0}
    oom_baseline=${oom_baseline:-0}
    oom_ai=${oom_ai:-0}

    echo "Neural Network Inferences:    $nn_inferences"
    echo "Commands Executed:            $commands_executed"
    echo ""
    echo "Network Performance:"
    echo "  Baseline packets:           $packets_baseline"
    echo "  With AI packets:            $packets_ai"
    if [ "$packets_baseline" -gt 0 ] 2>/dev/null && [ "$packets_ai" -gt 0 ] 2>/dev/null; then
        local improvement=$((100 * (packets_ai - packets_baseline) / packets_baseline))
        echo "  Improvement:                +$improvement%"
    fi
    echo ""
    echo "Memory Management:"
    echo "  Baseline OOM events:        $oom_baseline"
    echo "  With AI OOM events:         $oom_ai"
    echo ""

    # Validation checks
    log_info "Validation Checks:"
    echo ""

    local pass_count=0
    local fail_count=0

    # Check 1: All benchmarks completed
    local test_count=$(grep -c "\[EXPECT\] .*benchmark completed" "$OUTPUT_FILE" || echo "0")
    if [ "$test_count" -ge 4 ]; then
        log_pass "All benchmark tests completed ($test_count/5)"
        pass_count=$((pass_count + 1))
    else
        log_warn "Some benchmarks incomplete ($test_count/5)"
    fi

    # Check 2: Neural network active
    if [ "$nn_inferences" -gt 0 ] 2>/dev/null; then
        log_pass "Neural network active ($nn_inferences inferences)"
        pass_count=$((pass_count + 1))
    else
        log_fail "Neural network inactive (count: $nn_inferences)"
        fail_count=$((fail_count + 1))
    fi

    # Check 3: Commands processed
    if [ "$commands_executed" -gt 100 ] 2>/dev/null; then
        log_pass "Commands processed ($commands_executed commands)"
        pass_count=$((pass_count + 1))
    else
        log_warn "Low command count ($commands_executed)"
    fi

    # Check 4: Network activity
    if [ "$packets_ai" -gt 10000 ] 2>/dev/null; then
        log_pass "Network throughput good ($packets_ai packets)"
        pass_count=$((pass_count + 1))
    else
        log_warn "Low network throughput ($packets_ai packets)"
    fi

    # Check 5: System stability
    # Look for actual kernel panics (not just the word "crash" in normal output)
    local crashes=$(grep -i "kernel panic\|PANIC" "$OUTPUT_FILE" 2>/dev/null | wc -l | tr -d '[:space:]')
    crashes=${crashes:-0}
    # Ensure it's a valid single number
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
        log_pass "BENCHMARK SUITE SUCCESSFUL"
        echo "========================================"
        echo ""
        echo "Key Achievements:"
        echo "  - Neural network: $nn_inferences inferences"
        echo "  - Commands: $commands_executed processed"
        echo "  - Network: $packets_ai packets (AI enabled)"
        echo "  - Stability: Zero crashes"
        echo ""
        echo "Full results: $OUTPUT_FILE"
        exit 0
    else
        log_fail "BENCHMARK SUITE FAILED"
        echo "========================================"
        echo ""
        echo "Review full log: $OUTPUT_FILE"
        exit 1
    fi
}

main "$@"
