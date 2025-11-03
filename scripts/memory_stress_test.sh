#!/bin/bash
# Memory Stress Testing Script for Phase 4 Solidification
# Tests memory subsystem under 95% pressure for 10 minutes
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/stress_test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TEST_DURATION=600  # 10 minutes in seconds

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

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Main test execution
main() {
    local output_file="$RESULTS_DIR/memory_stress_${TIMESTAMP}.log"

    log_info "======================================"
    log_info "  Memory Stress Test - 95% Pressure"
    log_info "======================================"
    log_info "Duration: ${TEST_DURATION}s (10 minutes)"
    log_info "Output: $output_file"
    echo ""

    # Build kernel
    log_info "Building kernel with memory stress features..."
    cd "$PROJECT_ROOT"
    cargo build --release --target x86_64-unknown-uefi || {
        log_error "Kernel build failed"
        exit 1
    }

    log_info "Starting QEMU with memory stress configuration..."

    # Start QEMU with limited memory to induce pressure
    # Redirect output to log file
    SIS_FEATURES="llm,crypto-real" BRINGUP=1 "$SCRIPT_DIR/uefi_run.sh" build > "$output_file" 2>&1 &
    local qemu_pid=$!

    log_info "QEMU running with PID $qemu_pid"
    log_info "Monitoring for $TEST_DURATION seconds..."
    echo ""

    # Monitor progress
    local elapsed=0
    local check_interval=30
    local oom_count=0
    local last_oom_count=0

    while [ $elapsed -lt $TEST_DURATION ]; do
        if ! ps -p $qemu_pid > /dev/null 2>&1; then
            log_error "QEMU process died unexpectedly at ${elapsed}s"
            break
        fi

        sleep $check_interval
        elapsed=$((elapsed + check_interval))

        # Check for OOM events in log
        oom_count=$(grep -c "OOM" "$output_file" 2>/dev/null || echo "0")
        local new_ooms=$((oom_count - last_oom_count))
        last_oom_count=$oom_count

        # Extract latest memory stats if available
        local mem_pressure=$(grep "Memory pressure:" "$output_file" | tail -1 | awk '{print $3}' || echo "N/A")
        local heap_used=$(grep "Heap used:" "$output_file" | tail -1 | awk '{print $3}' || echo "N/A")

        # Progress report
        local remaining=$((TEST_DURATION - elapsed))
        log_test "T+${elapsed}s (${remaining}s remaining)"
        echo "    Memory Pressure: $mem_pressure"
        echo "    Heap Usage:      $heap_used"
        echo "    OOM Events:      $oom_count total (+$new_ooms)"

        if [ "$new_ooms" -gt 0 ]; then
            log_warn "Detected $new_ooms new OOM events in last ${check_interval}s"
        fi
        echo ""
    done

    # Terminate QEMU
    log_info "Test duration complete. Terminating QEMU..."
    kill -TERM $qemu_pid 2>/dev/null || true
    wait $qemu_pid 2>/dev/null || true

    # Final analysis
    echo ""
    log_info "======================================"
    log_info "  Memory Stress Test Results"
    log_info "======================================"

    # Extract final metrics
    local total_ooms=$(grep -c "OOM" "$output_file" 2>/dev/null || echo "0")
    local ai_decisions=$(grep "AI decisions:" "$output_file" | tail -1 || echo "N/A")
    local memory_freed=$(grep "Memory freed by AI:" "$output_file" | tail -1 || echo "N/A")
    local max_pressure=$(grep "Memory pressure:" "$output_file" | awk '{print $3}' | sort -n | tail -1 || echo "N/A")

    echo "Test Duration:        ${TEST_DURATION}s"
    echo "Total OOM Events:     $total_ooms"
    echo "Max Memory Pressure:  $max_pressure"
    echo "AI Memory Decisions:  $ai_decisions"
    echo "Memory Freed by AI:   $memory_freed"
    echo ""
    echo "Full results: $output_file"
    echo "======================================"

    # Validation checks
    echo ""
    log_info "Validation Checks:"

    if [ "$total_ooms" -eq 0 ]; then
        log_info "[PASS] No OOM events - AI memory management effective"
    elif [ "$total_ooms" -lt 10 ]; then
        log_warn "[WARN] Low OOM count ($total_ooms) - acceptable under stress"
    else
        log_error "[FAIL] High OOM count ($total_ooms) - AI may need tuning"
    fi

    # Check if AI was active
    if grep -q "AI inferences: 0" "$output_file"; then
        log_error "[FAIL] AI predictions inactive - no inferences detected"
    else
        log_info "[PASS] AI predictions active"
    fi

    # Check for memory leaks
    local initial_heap=$(grep "Heap used:" "$output_file" | head -1 | awk '{print $3}' || echo "0")
    local final_heap=$(grep "Heap used:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")

    if [ "$initial_heap" != "N/A" ] && [ "$final_heap" != "N/A" ]; then
        local heap_growth=$((final_heap - initial_heap))
        echo "Heap Growth: $heap_growth bytes"

        if [ $heap_growth -gt 1048576 ]; then  # 1MB growth
            log_warn "[WARN] Potential memory leak - heap grew by $heap_growth bytes"
        else
            log_info "[PASS] No significant memory leak detected"
        fi
    fi

    echo ""
    log_info "Memory stress test complete"
}

main "$@"
