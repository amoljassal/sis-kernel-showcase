#!/bin/bash
# Extended Benchmark Test Suite for Phase 4 Solidification
# Runs benchmarks for 5min, 15min, and 1hr durations
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Color codes for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to run benchmark for specified duration
run_timed_benchmark() {
    local duration_name=$1
    local duration_seconds=$2
    local output_file="$RESULTS_DIR/benchmark_${duration_name}_${TIMESTAMP}.log"

    log_info "Starting $duration_name benchmark (${duration_seconds}s)..."
    log_info "Output: $output_file"

    # Start QEMU in background with serial output
    "$SCRIPT_DIR/uefi_run.sh" build > "$output_file" 2>&1 &
    local qemu_pid=$!

    # Wait for QEMU to start
    sleep 5

    # Send benchmark command through QEMU monitor or serial
    # For now, we'll let it run and capture output
    log_info "QEMU running with PID $qemu_pid"

    # Wait for specified duration
    local elapsed=0
    while [ $elapsed -lt $duration_seconds ]; do
        if ! ps -p $qemu_pid > /dev/null 2>&1; then
            log_error "QEMU process died unexpectedly"
            return 1
        fi
        sleep 10
        elapsed=$((elapsed + 10))
        local remaining=$((duration_seconds - elapsed))
        if [ $((elapsed % 60)) -eq 0 ]; then
            log_info "Progress: ${elapsed}s elapsed, ${remaining}s remaining"
        fi
    done

    # Terminate QEMU gracefully
    log_info "Benchmark duration complete. Terminating QEMU..."
    kill -TERM $qemu_pid 2>/dev/null || true
    wait $qemu_pid 2>/dev/null || true

    log_info "Completed $duration_name benchmark"
    log_info "Results saved to: $output_file"

    return 0
}

# Function to analyze benchmark results
analyze_results() {
    local result_file=$1
    local duration_name=$2

    log_info "Analyzing $duration_name results..."

    # Extract key metrics
    local oom_events=$(grep -c "OOM" "$result_file" 2>/dev/null || echo "0")
    local ai_inferences=$(grep "AI inferences" "$result_file" | tail -1 || echo "N/A")
    local memory_pressure=$(grep "Memory pressure" "$result_file" | tail -1 || echo "N/A")
    local packets_sent=$(grep "packets sent" "$result_file" | tail -1 || echo "N/A")

    echo ""
    echo "=========================================="
    echo "  $duration_name Benchmark Summary"
    echo "=========================================="
    echo "OOM Events:       $oom_events"
    echo "AI Inferences:    $ai_inferences"
    echo "Memory Pressure:  $memory_pressure"
    echo "Network Activity: $packets_sent"
    echo "=========================================="
    echo ""

    # Check for anomalies
    if [ "$oom_events" -gt 10 ]; then
        log_warn "High OOM count detected: $oom_events events"
    fi
}

# Main execution
main() {
    log_info "======================================"
    log_info "  Phase 4 Extended Benchmark Suite"
    log_info "======================================"
    log_info "Timestamp: $TIMESTAMP"
    log_info "Results Directory: $RESULTS_DIR"
    echo ""

    # Build the kernel first
    log_info "Building kernel..."
    cd "$PROJECT_ROOT"
    cargo build --release --target x86_64-unknown-uefi || {
        log_error "Kernel build failed"
        exit 1
    }

    # Run benchmarks
    log_info "Starting benchmark sequence..."
    echo ""

    # 5-minute benchmark
    run_timed_benchmark "5min" 300

    # 15-minute benchmark
    log_info "Waiting 30s before next benchmark..."
    sleep 30
    run_timed_benchmark "15min" 900

    # 1-hour benchmark
    log_info "Waiting 60s before long-duration benchmark..."
    sleep 60
    run_timed_benchmark "1hr" 3600

    # Analyze all results
    echo ""
    log_info "======================================"
    log_info "  Benchmark Suite Complete"
    log_info "======================================"
    echo ""

    for result in "$RESULTS_DIR"/benchmark_*_${TIMESTAMP}.log; do
        if [ -f "$result" ]; then
            duration=$(basename "$result" | cut -d'_' -f2)
            analyze_results "$result" "$duration"
        fi
    done

    log_info "All benchmark results saved in: $RESULTS_DIR"
}

# Run main function
main "$@"
