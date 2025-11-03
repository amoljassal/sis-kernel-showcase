#!/bin/bash
# Autonomous Control Validation Script for Phase 4 Solidification
# Tests autonomous operation for 1hr and 4hr durations
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/autonomous_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

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

# Function to run autonomous validation test
run_autonomous_test() {
    local duration_name=$1
    local duration_seconds=$2
    local output_file="$RESULTS_DIR/autonomous_${duration_name}_${TIMESTAMP}.log"

    log_info "======================================"
    log_info "  Autonomous Control Test - $duration_name"
    log_info "======================================"
    log_info "Duration: ${duration_seconds}s"
    log_info "Output: $output_file"
    echo ""

    # Build kernel
    log_info "Building kernel..."
    cd "$PROJECT_ROOT"
    cargo build --release --target x86_64-unknown-uefi || {
        log_error "Kernel build failed"
        return 1
    }

    log_info "Starting QEMU for autonomous validation..."
    SIS_FEATURES="llm,crypto-real" BRINGUP=1 "$SCRIPT_DIR/uefi_run.sh" build > "$output_file" 2>&1 &
    local qemu_pid=$!

    log_info "QEMU running with PID $qemu_pid"
    log_info "Monitoring autonomous operation for ${duration_seconds}s..."
    echo ""

    # Monitoring loop
    local elapsed=0
    local check_interval=60  # Check every minute
    local last_decisions=0
    local last_inferences=0

    while [ $elapsed -lt $duration_seconds ]; do
        if ! ps -p $qemu_pid > /dev/null 2>&1; then
            log_error "QEMU process died unexpectedly at ${elapsed}s"
            return 1
        fi

        sleep $check_interval
        elapsed=$((elapsed + check_interval))

        # Extract latest metrics
        local decisions=$(grep "Total AI decisions:" "$output_file" | tail -1 | awk '{print $4}' || echo "0")
        local inferences=$(grep "AI inferences:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")
        local memory_freed=$(grep "Memory freed:" "$output_file" | tail -1 | awk '{print $3}' || echo "0")
        local autonomy_level=$(grep "Autonomy level:" "$output_file" | tail -1 | awk '{print $3}' || echo "N/A")

        # Calculate deltas
        local new_decisions=$((decisions - last_decisions))
        local new_inferences=$((inferences - last_inferences))
        last_decisions=$decisions
        last_inferences=$inferences

        # Progress report
        local remaining=$((duration_seconds - elapsed))
        local hours=$((elapsed / 3600))
        local mins=$(((elapsed % 3600) / 60))
        local secs=$((elapsed % 60))

        log_test "T+${hours}h${mins}m${secs}s (${remaining}s remaining)"
        echo "    AI Decisions:     $decisions total (+$new_decisions)"
        echo "    AI Inferences:    $inferences total (+$new_inferences)"
        echo "    Memory Freed:     $memory_freed"
        echo "    Autonomy Level:   $autonomy_level"

        # Warnings
        if [ "$new_inferences" -eq 0 ]; then
            log_warn "No new AI inferences in last ${check_interval}s - AI may be idle"
        fi

        if [ "$autonomy_level" = "0" ]; then
            log_warn "Autonomy level is 0 - autonomous control may be disabled"
        fi

        echo ""
    done

    # Terminate QEMU
    log_info "Test duration complete. Terminating QEMU..."
    kill -TERM $qemu_pid 2>/dev/null || true
    wait $qemu_pid 2>/dev/null || true

    # Final analysis
    analyze_autonomous_results "$output_file" "$duration_name" "$duration_seconds"

    return 0
}

# Function to analyze autonomous test results
analyze_autonomous_results() {
    local result_file=$1
    local duration_name=$2
    local duration_seconds=$3

    echo ""
    log_info "======================================"
    log_info "  Autonomous Test Results - $duration_name"
    log_info "======================================"

    # Extract metrics
    local total_decisions=$(grep "Total AI decisions:" "$result_file" | tail -1 | awk '{print $4}' || echo "0")
    local total_inferences=$(grep "AI inferences:" "$result_file" | tail -1 | awk '{print $3}' || echo "0")
    local oom_events=$(grep -c "OOM" "$result_file" 2>/dev/null || echo "0")
    local crashes=$(grep -c "panic\|crash\|fault" "$result_file" 2>/dev/null || echo "0")
    local memory_decisions=$(grep "Memory management decisions:" "$result_file" | tail -1 | awk '{print $4}' || echo "0")
    local schedule_decisions=$(grep "Scheduling decisions:" "$result_file" | tail -1 | awk '{print $3}' || echo "0")

    echo "Test Duration:           ${duration_seconds}s"
    echo "Total AI Decisions:      $total_decisions"
    echo "Total AI Inferences:     $total_inferences"
    echo "Memory Decisions:        $memory_decisions"
    echo "Scheduling Decisions:    $schedule_decisions"
    echo "OOM Events:              $oom_events"
    echo "System Crashes:          $crashes"
    echo ""

    # Calculate decision rate
    if [ "$duration_seconds" -gt 0 ] && [ "$total_decisions" -gt 0 ]; then
        local decisions_per_min=$((total_decisions * 60 / duration_seconds))
        echo "Decision Rate:           $decisions_per_min decisions/min"
    fi

    # Calculate inference rate
    if [ "$duration_seconds" -gt 0 ] && [ "$total_inferences" -gt 0 ]; then
        local inferences_per_min=$((total_inferences * 60 / duration_seconds))
        echo "Inference Rate:          $inferences_per_min inferences/min"
    fi

    echo ""
    log_info "Validation Checks:"

    # Check 1: AI activity
    if [ "$total_inferences" -eq 0 ]; then
        log_error "[FAIL] No AI inferences - neural network inactive"
    elif [ "$total_inferences" -lt 10 ]; then
        log_warn "[WARN] Low AI activity - only $total_inferences inferences"
    else
        log_info "[PASS] AI neural network active - $total_inferences inferences"
    fi

    # Check 2: Autonomous decisions
    if [ "$total_decisions" -eq 0 ]; then
        log_error "[FAIL] No autonomous decisions made"
    elif [ "$total_decisions" -lt 5 ]; then
        log_warn "[WARN] Low decision count - only $total_decisions decisions"
    else
        log_info "[PASS] Autonomous control active - $total_decisions decisions"
    fi

    # Check 3: System stability
    if [ "$crashes" -gt 0 ]; then
        log_error "[FAIL] System crashes detected: $crashes"
    else
        log_info "[PASS] No crashes detected - system stable"
    fi

    # Check 4: Memory management
    if [ "$oom_events" -gt 50 ]; then
        log_error "[FAIL] Excessive OOM events: $oom_events"
    elif [ "$oom_events" -gt 10 ]; then
        log_warn "[WARN] Moderate OOM events: $oom_events"
    else
        log_info "[PASS] Low OOM events: $oom_events"
    fi

    echo ""
    echo "Full results: $result_file"
    echo "======================================"
    echo ""
}

# Main execution
main() {
    log_info "======================================"
    log_info "  Autonomous Control Validation Suite"
    log_info "======================================"
    log_info "Timestamp: $TIMESTAMP"
    log_info "Results Directory: $RESULTS_DIR"
    echo ""

    # Determine which test to run
    if [ "$1" = "1hr" ]; then
        run_autonomous_test "1hr" 3600
    elif [ "$1" = "4hr" ]; then
        run_autonomous_test "4hr" 14400
    elif [ "$1" = "both" ]; then
        # Run 1hr test
        run_autonomous_test "1hr" 3600

        # Wait between tests
        log_info "Waiting 120s before 4hr test..."
        sleep 120

        # Run 4hr test
        run_autonomous_test "4hr" 14400
    else
        log_error "Usage: $0 {1hr|4hr|both}"
        echo ""
        echo "Examples:"
        echo "  $0 1hr       Run 1-hour autonomous test"
        echo "  $0 4hr       Run 4-hour autonomous test"
        echo "  $0 both      Run both tests sequentially"
        exit 1
    fi

    echo ""
    log_info "======================================"
    log_info "  Autonomous Validation Complete"
    log_info "======================================"
    log_info "All results saved in: $RESULTS_DIR"
}

main "$@"
