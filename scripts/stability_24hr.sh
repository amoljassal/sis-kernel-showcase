#!/bin/bash
# 24-Hour Stability Test for Phase 4 Solidification
# Continuous operation test with periodic monitoring
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/stability_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TEST_DURATION=86400  # 24 hours in seconds

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
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

log_metric() {
    echo -e "${MAGENTA}[METRIC]${NC} $1"
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Monitoring data files
OUTPUT_FILE="$RESULTS_DIR/stability_24hr_${TIMESTAMP}.log"
METRICS_FILE="$RESULTS_DIR/stability_24hr_metrics_${TIMESTAMP}.csv"
SUMMARY_FILE="$RESULTS_DIR/stability_24hr_summary_${TIMESTAMP}.txt"

# Initialize metrics CSV
init_metrics_csv() {
    echo "Timestamp,Elapsed_Sec,Decisions,Inferences,OOM_Events,Memory_Pressure,Heap_Used,Crashes" > "$METRICS_FILE"
}

# Collect metrics snapshot
collect_metrics() {
    local elapsed=$1
    local timestamp=$(date +%Y-%m-%d_%H:%M:%S)

    # Extract current metrics from log
    local decisions=$(grep "Total AI decisions:" "$OUTPUT_FILE" | tail -1 | awk '{print $4}' || echo "0")
    local inferences=$(grep "AI inferences:" "$OUTPUT_FILE" | tail -1 | awk '{print $3}' || echo "0")
    local oom_events=$(grep -c "OOM" "$OUTPUT_FILE" 2>/dev/null || echo "0")
    local mem_pressure=$(grep "Memory pressure:" "$OUTPUT_FILE" | tail -1 | awk '{print $3}' | tr -d '%' || echo "0")
    local heap_used=$(grep "Heap used:" "$OUTPUT_FILE" | tail -1 | awk '{print $3}' || echo "0")
    local crashes=$(grep -c "panic\|crash\|fault" "$OUTPUT_FILE" 2>/dev/null || echo "0")

    # Append to CSV
    echo "$timestamp,$elapsed,$decisions,$inferences,$oom_events,$mem_pressure,$heap_used,$crashes" >> "$METRICS_FILE"
}

# Generate summary report
generate_summary() {
    local total_elapsed=$1

    log_info "Generating stability test summary..."

    {
        echo "========================================"
        echo "  24-Hour Stability Test Summary"
        echo "========================================"
        echo "Start Time:    $(head -2 "$METRICS_FILE" | tail -1 | cut -d',' -f1)"
        echo "End Time:      $(tail -1 "$METRICS_FILE" | cut -d',' -f1)"
        echo "Duration:      ${total_elapsed}s ($((total_elapsed / 3600))h)"
        echo ""

        # Final metrics
        local final_decisions=$(tail -1 "$METRICS_FILE" | cut -d',' -f3)
        local final_inferences=$(tail -1 "$METRICS_FILE" | cut -d',' -f4)
        local final_ooms=$(tail -1 "$METRICS_FILE" | cut -d',' -f5)
        local final_crashes=$(tail -1 "$METRICS_FILE" | cut -d',' -f8)

        echo "Final Metrics:"
        echo "  Total AI Decisions:   $final_decisions"
        echo "  Total AI Inferences:  $final_inferences"
        echo "  Total OOM Events:     $final_ooms"
        echo "  Total Crashes:        $final_crashes"
        echo ""

        # Calculate rates
        if [ "$total_elapsed" -gt 0 ]; then
            local decisions_per_hour=$((final_decisions * 3600 / total_elapsed))
            local inferences_per_hour=$((final_inferences * 3600 / total_elapsed))
            local ooms_per_hour=$((final_ooms * 3600 / total_elapsed))

            echo "Average Rates:"
            echo "  Decisions/hour:       $decisions_per_hour"
            echo "  Inferences/hour:      $inferences_per_hour"
            echo "  OOM events/hour:      $ooms_per_hour"
            echo ""
        fi

        # Memory leak analysis
        local initial_heap=$(head -2 "$METRICS_FILE" | tail -1 | cut -d',' -f7)
        local final_heap=$(tail -1 "$METRICS_FILE" | cut -d',' -f7)

        if [ "$initial_heap" != "0" ] && [ "$final_heap" != "0" ]; then
            local heap_growth=$((final_heap - initial_heap))
            local heap_growth_mb=$((heap_growth / 1048576))

            echo "Memory Analysis:"
            echo "  Initial Heap:         $initial_heap bytes"
            echo "  Final Heap:           $final_heap bytes"
            echo "  Heap Growth:          $heap_growth bytes ($heap_growth_mb MB)"
            echo ""

            if [ $heap_growth_mb -gt 100 ]; then
                echo "  [WARN] Significant heap growth - potential memory leak"
            else
                echo "  [PASS] Heap growth within acceptable range"
            fi
            echo ""
        fi

        # Stability assessment
        echo "Stability Assessment:"

        if [ "$final_crashes" -eq 0 ]; then
            echo "  [PASS] No crashes - excellent stability"
        else
            echo "  [FAIL] $final_crashes crashes detected"
        fi

        if [ "$final_inferences" -gt 100 ]; then
            echo "  [PASS] AI neural network active"
        else
            echo "  [WARN] Low AI activity - $final_inferences inferences"
        fi

        if [ "$final_ooms" -lt 100 ]; then
            echo "  [PASS] Low OOM event rate"
        else
            echo "  [WARN] High OOM event rate - $final_ooms events"
        fi

        echo ""
        echo "Full log:        $OUTPUT_FILE"
        echo "Metrics CSV:     $METRICS_FILE"
        echo "========================================"

    } | tee "$SUMMARY_FILE"
}

# Main test execution
main() {
    log_info "========================================"
    log_info "  24-Hour Stability Test"
    log_info "========================================"
    log_info "Duration: ${TEST_DURATION}s (24 hours)"
    log_info "Start Time: $(date)"
    log_info "Output: $OUTPUT_FILE"
    log_info "Metrics: $METRICS_FILE"
    echo ""

    # Initialize metrics tracking
    init_metrics_csv

    # Build kernel
    log_info "Building kernel..."
    cd "$PROJECT_ROOT"
    cargo build --release --target x86_64-unknown-uefi || {
        log_error "Kernel build failed"
        exit 1
    }

    # Start QEMU
    log_info "Starting QEMU for 24-hour stability test..."
    SIS_FEATURES="llm,crypto-real" BRINGUP=1 "$SCRIPT_DIR/uefi_run.sh" build > "$OUTPUT_FILE" 2>&1 &
    local qemu_pid=$!

    log_info "QEMU running with PID $qemu_pid"
    log_info "Test will run for 24 hours. You can safely detach from this terminal."
    log_info "Progress updates every 30 minutes..."
    echo ""

    # Monitoring loop
    local elapsed=0
    local check_interval=300      # Check every 5 minutes
    local report_interval=1800    # Report every 30 minutes
    local last_report=0

    while [ $elapsed -lt $TEST_DURATION ]; do
        # Check if QEMU is still running
        if ! ps -p $qemu_pid > /dev/null 2>&1; then
            log_error "QEMU process died unexpectedly at ${elapsed}s"
            log_error "Elapsed: $((elapsed / 3600))h $((elapsed % 3600 / 60))m"
            break
        fi

        # Wait for check interval
        sleep $check_interval
        elapsed=$((elapsed + check_interval))

        # Collect metrics
        collect_metrics $elapsed

        # Periodic reporting
        if [ $((elapsed - last_report)) -ge $report_interval ]; then
            local hours=$((elapsed / 3600))
            local mins=$(((elapsed % 3600) / 60))
            local remaining=$((TEST_DURATION - elapsed))
            local remaining_hours=$((remaining / 3600))
            local remaining_mins=$(((remaining % 3600) / 60))

            log_test "Progress: ${hours}h${mins}m elapsed, ${remaining_hours}h${remaining_mins}m remaining"

            # Extract latest metrics
            local decisions=$(grep "Total AI decisions:" "$OUTPUT_FILE" | tail -1 | awk '{print $4}' || echo "0")
            local inferences=$(grep "AI inferences:" "$OUTPUT_FILE" | tail -1 | awk '{print $3}' || echo "0")
            local oom_count=$(grep -c "OOM" "$OUTPUT_FILE" 2>/dev/null || echo "0")

            log_metric "Decisions: $decisions | Inferences: $inferences | OOMs: $oom_count"

            last_report=$elapsed
            echo ""
        fi
    done

    # Test complete - terminate QEMU
    log_info "Test duration complete. Terminating QEMU..."
    kill -TERM $qemu_pid 2>/dev/null || true
    wait $qemu_pid 2>/dev/null || true

    # Generate final summary
    echo ""
    generate_summary $elapsed

    # Final message
    echo ""
    log_info "24-hour stability test complete"
    log_info "End Time: $(date)"
}

main "$@"
