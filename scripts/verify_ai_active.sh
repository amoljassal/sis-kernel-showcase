#!/bin/bash
# AI Neural Network Activity Verification Script
# Verifies that AI predictions are active and generating inferences
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/ai_verification_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TEST_DURATION=300  # 5 minutes for quick verification

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

# Main verification test
main() {
    local output_file="$RESULTS_DIR/ai_verification_${TIMESTAMP}.log"

    log_info "========================================"
    log_info "  AI Neural Network Activity Verification"
    log_info "========================================"
    log_info "Duration: ${TEST_DURATION}s (5 minutes)"
    log_info "Output: $output_file"
    echo ""

    # Start QEMU (uefi_run.sh will build the kernel)
    log_info "Starting QEMU for AI verification (kernel will be built automatically)..."
    SIS_FEATURES="llm,crypto-real" BRINGUP=1 "$SCRIPT_DIR/uefi_run.sh" build > "$output_file" 2>&1 &
    local qemu_pid=$!

    log_info "QEMU running with PID $qemu_pid"
    log_info "Monitoring AI activity for ${TEST_DURATION}s..."
    echo ""

    # Monitoring loop
    local elapsed=0
    local check_interval=30
    local inference_checks=0
    local inference_active_checks=0

    while [ $elapsed -lt $TEST_DURATION ]; do
        if ! ps -p $qemu_pid > /dev/null 2>&1; then
            log_error "QEMU process died unexpectedly"
            exit 1
        fi

        sleep $check_interval
        elapsed=$((elapsed + check_interval))
        inference_checks=$((inference_checks + 1))

        # Check for AI activity
        local inferences=$(grep "AI inferences:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")
        local decisions=$(grep "Total AI decisions:" "$output_file" | tail -1 | awk '{print $4}' || echo "0")
        local memory_pred=$(grep "Memory predictions:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")
        local schedule_pred=$(grep "Schedule predictions:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")

        # Log current status
        local remaining=$((TEST_DURATION - elapsed))
        echo "T+${elapsed}s (${remaining}s remaining):"
        echo "  AI Inferences:        $inferences"
        echo "  AI Decisions:         $decisions"
        echo "  Memory Predictions:   $memory_pred"
        echo "  Schedule Predictions: $schedule_pred"

        # Track if AI is active
        if [ "$inferences" -gt 0 ]; then
            inference_active_checks=$((inference_active_checks + 1))
            echo "  Status: [ACTIVE]"
        else
            echo "  Status: [INACTIVE]"
            log_warn "No AI inferences detected yet"
        fi

        echo ""
    done

    # Terminate QEMU
    log_info "Verification complete. Terminating QEMU..."
    kill -TERM $qemu_pid 2>/dev/null || true
    wait $qemu_pid 2>/dev/null || true

    # Analysis
    echo ""
    log_info "========================================"
    log_info "  AI Verification Results"
    log_info "========================================"

    # Extract final metrics
    local final_inferences=$(grep "AI inferences:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")
    local final_decisions=$(grep "Total AI decisions:" "$output_file" | tail -1 | awk '{print $4}' || echo "0")
    local memory_mgmt=$(grep "Memory management decisions:" "$output_file" | tail -1 | awk '{print $4}' || echo "0")
    local schedule_mgmt=$(grep "Scheduling decisions:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")
    local network_mgmt=$(grep "Network decisions:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")

    echo "Test Duration:           ${TEST_DURATION}s"
    echo "Total Checks:            $inference_checks"
    echo "Active Checks:           $inference_active_checks"
    echo ""
    echo "Final AI Metrics:"
    echo "  Total Inferences:      $final_inferences"
    echo "  Total Decisions:       $final_decisions"
    echo "  Memory Decisions:      $memory_mgmt"
    echo "  Scheduling Decisions:  $schedule_mgmt"
    echo "  Network Decisions:     $network_mgmt"
    echo ""

    # Validation checks
    log_info "Validation Checks:"
    echo ""

    local pass_count=0
    local fail_count=0

    # Check 1: AI inferences > 0
    if [ "$final_inferences" -gt 0 ]; then
        log_pass "AI neural network generating inferences ($final_inferences total)"
        pass_count=$((pass_count + 1))
    else
        log_fail "AI neural network NOT generating inferences"
        fail_count=$((fail_count + 1))
    fi

    # Check 2: AI decisions > 0
    if [ "$final_decisions" -gt 0 ]; then
        log_pass "AI making autonomous decisions ($final_decisions total)"
        pass_count=$((pass_count + 1))
    else
        log_fail "AI NOT making autonomous decisions"
        fail_count=$((fail_count + 1))
    fi

    # Check 3: Memory management active
    if [ "$memory_mgmt" -gt 0 ]; then
        log_pass "AI memory management active ($memory_mgmt decisions)"
        pass_count=$((pass_count + 1))
    else
        log_warn "AI memory management inactive"
    fi

    # Check 4: Scheduling active
    if [ "$schedule_mgmt" -gt 0 ]; then
        log_pass "AI scheduling active ($schedule_mgmt decisions)"
        pass_count=$((pass_count + 1))
    else
        log_warn "AI scheduling inactive"
    fi

    # Check 5: Network management active
    if [ "$network_mgmt" -gt 0 ]; then
        log_pass "AI network management active ($network_mgmt decisions)"
        pass_count=$((pass_count + 1))
    else
        log_warn "AI network management inactive"
    fi

    # Check 6: Consistency - AI active in most checks
    local activity_rate=$((inference_active_checks * 100 / inference_checks))
    echo ""
    echo "Activity Rate: $activity_rate% ($inference_active_checks/$inference_checks checks)"

    if [ $activity_rate -ge 80 ]; then
        log_pass "AI consistently active (>80% of checks)"
        pass_count=$((pass_count + 1))
    elif [ $activity_rate -ge 50 ]; then
        log_warn "AI intermittently active ($activity_rate% of checks)"
    else
        log_fail "AI mostly inactive (<50% of checks)"
        fail_count=$((fail_count + 1))
    fi

    # Overall result
    echo ""
    echo "========================================"
    echo "Overall Result: $pass_count passed, $fail_count failed"

    if [ $fail_count -eq 0 ] && [ $pass_count -ge 4 ]; then
        log_pass "AI VERIFICATION SUCCESSFUL"
        echo "========================================"
        exit 0
    elif [ $fail_count -le 1 ]; then
        log_warn "AI VERIFICATION PASSED WITH WARNINGS"
        echo "========================================"
        exit 0
    else
        log_fail "AI VERIFICATION FAILED"
        echo "========================================"
        echo ""
        echo "Troubleshooting:"
        echo "  1. Check that SIS_FEATURES=llm is set"
        echo "  2. Verify LLM model is loaded correctly"
        echo "  3. Check autonomy level (should be >0)"
        echo "  4. Review full log: $output_file"
        exit 1
    fi
}

main "$@"
