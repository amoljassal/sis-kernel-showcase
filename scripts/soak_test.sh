#!/usr/bin/env bash
# Soak Testing Infrastructure for SIS Kernel
# Phase 2.3 - Production Readiness Plan
#
# Runs kernel repeatedly over extended period to detect:
# - Memory leaks
# - Performance degradation
# - Intermittent failures
# - Resource exhaustion

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

# Configuration
DURATION=${DURATION:-86400}  # 24 hours default (in seconds)
INTERVAL=${INTERVAL:-60}     # 1 minute between runs (in seconds)
TIMEOUT=${TIMEOUT:-45}       # Timeout per test run (in seconds)
OUTPUT_DIR=${OUTPUT_DIR:-/tmp/soak-test}

# Calculate number of runs
RUNS=$((DURATION / INTERVAL))

echo "=================================================="
echo "SIS Kernel Soak Test"
echo "=================================================="
echo "Duration:       ${DURATION}s ($(echo "scale=2; $DURATION/3600" | bc -l)h)"
echo "Runs:           $RUNS"
echo "Interval:       ${INTERVAL}s"
echo "Timeout/run:    ${TIMEOUT}s"
echo "Output:         $OUTPUT_DIR"
echo "=================================================="
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Counters
PASS=0
FAIL=0
TIMEOUT_COUNT=0
START_TIME=$(date +%s)

# Results files
SUMMARY_FILE="$OUTPUT_DIR/summary.txt"
METRICS_FILE="$OUTPUT_DIR/metrics.log"
FAILURES_FILE="$OUTPUT_DIR/failures.txt"

# Initialize files
echo "# SIS Kernel Soak Test Results" > "$SUMMARY_FILE"
echo "# Started: $(date)" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

echo "# SIS Kernel Soak Test Metrics" > "$METRICS_FILE"
echo "# Format: run_num,timestamp,result,boot_time_ms" >> "$METRICS_FILE"

echo "# Failed Test Runs" > "$FAILURES_FILE"

# Trap to show summary on exit
trap 'show_summary' EXIT

show_summary() {
    local end_time=$(date +%s)
    local elapsed=$((end_time - START_TIME))
    local completed=$((PASS + FAIL + TIMEOUT_COUNT))

    echo ""
    echo "=================================================="
    echo "Soak Test Summary"
    echo "=================================================="
    echo "Completed:      $completed / $RUNS runs"
    echo "Passed:         $PASS"
    echo "Failed:         $FAIL"
    echo "Timeouts:       $TIMEOUT_COUNT"
    echo "Elapsed:        ${elapsed}s ($(echo "scale=2; $elapsed/3600" | bc -l)h)"
    echo "=================================================="

    # Calculate rates
    if [[ $completed -gt 0 ]]; then
        local pass_rate=$((PASS * 100 / completed))
        local fail_rate=$(((FAIL + TIMEOUT_COUNT) * 100 / completed))
        echo "Pass rate:      ${pass_rate}%"
        echo "Fail rate:      ${fail_rate}%"

        # Append to summary file
        echo "" >> "$SUMMARY_FILE"
        echo "## Final Results" >> "$SUMMARY_FILE"
        echo "Completed: $completed / $RUNS" >> "$SUMMARY_FILE"
        echo "Passed:    $PASS" >> "$SUMMARY_FILE"
        echo "Failed:    $FAIL" >> "$SUMMARY_FILE"
        echo "Timeouts:  $TIMEOUT_COUNT" >> "$SUMMARY_FILE"
        echo "Pass rate: ${pass_rate}%" >> "$SUMMARY_FILE"
        echo "Fail rate: ${fail_rate}%" >> "$SUMMARY_FILE"

        # Check if failure rate is too high
        if [[ $fail_rate -gt 5 ]]; then
            echo ""
            echo "[!] SOAK TEST FAILED: Failure rate too high (${fail_rate}% > 5%)"
            exit 1
        fi
    fi
}

# Main test loop
for i in $(seq 1 $RUNS); do
    RUN_START=$(date +%s)
    echo "[$i/$RUNS] $(date) - Starting test run..."

    # Run test
    TEST_LOG="$OUTPUT_DIR/run-$i.log"
    TEST_START_MS=$(date +%s%3N)

    if timeout "${TIMEOUT}s" "$SCRIPT_DIR/uefi_run.sh" > "$TEST_LOG" 2>&1; then
        # Test completed without timeout
        TEST_END_MS=$(date +%s%3N)
        BOOT_TIME=$((TEST_END_MS - TEST_START_MS))

        # Check for successful boot
        if grep -q "LAUNCHING SHELL\|Shell ready" "$TEST_LOG"; then
            ((PASS++))
            echo "  ✓ PASS (${BOOT_TIME}ms)"
            echo "$i,$(date +%s),PASS,$BOOT_TIME" >> "$METRICS_FILE"

            # Extract metrics if available
            if grep -q "METRIC\|heap" "$TEST_LOG"; then
                grep -E "METRIC|heap|memory" "$TEST_LOG" >> "$METRICS_FILE" || true
            fi
        else
            ((FAIL++))
            echo "  ✗ FAIL - No shell prompt"
            echo "$i,$(date +%s),FAIL_NO_SHELL,$BOOT_TIME" >> "$METRICS_FILE"
            echo "Run $i: No shell prompt (boot time: ${BOOT_TIME}ms)" >> "$FAILURES_FILE"
        fi

        # Check for panics
        if grep -qi "KERNEL PANIC" "$TEST_LOG"; then
            echo "  [!] WARNING: Kernel panic detected"
            echo "Run $i: Kernel panic" >> "$FAILURES_FILE"
            tail -50 "$TEST_LOG" >> "$FAILURES_FILE"
        fi

    else
        # Timeout occurred
        exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            ((TIMEOUT_COUNT++))
            echo "  ⏱ TIMEOUT"
            echo "$i,$(date +%s),TIMEOUT,0" >> "$METRICS_FILE"
            echo "Run $i: Timeout (${TIMEOUT}s)" >> "$FAILURES_FILE"
        else
            ((FAIL++))
            echo "  ✗ FAIL (exit code: $exit_code)"
            echo "$i,$(date +%s),FAIL_EXIT_$exit_code,0" >> "$METRICS_FILE"
            echo "Run $i: Failed with exit code $exit_code" >> "$FAILURES_FILE"
        fi
    fi

    # Calculate sleep time (to maintain interval)
    RUN_END=$(date +%s)
    RUN_DURATION=$((RUN_END - RUN_START))
    SLEEP_TIME=$((INTERVAL - RUN_DURATION))

    if [[ $SLEEP_TIME -gt 0 ]]; then
        echo "  Sleeping ${SLEEP_TIME}s until next run..."
        sleep "$SLEEP_TIME"
    else
        echo "  [!] WARNING: Run took longer than interval (${RUN_DURATION}s > ${INTERVAL}s)"
    fi

    # Print progress every 10 runs
    if [[ $((i % 10)) -eq 0 ]]; then
        echo ""
        echo "  Progress: $i/$RUNS ($(echo "scale=1; $i*100/$RUNS" | bc -l)%)"
        echo "  Stats: $PASS passed, $FAIL failed, $TIMEOUT_COUNT timeouts"
        echo ""
    fi
done

echo ""
echo "[*] Soak test complete!"
echo "[*] Generating analysis report..."

# Generate report
if [[ -x "$SCRIPT_DIR/soak_report.py" ]]; then
    python3 "$SCRIPT_DIR/soak_report.py" "$METRICS_FILE" > "$OUTPUT_DIR/report.html"
    echo "[*] Report saved to: $OUTPUT_DIR/report.html"
else
    echo "[!] Report generator not found (soak_report.py)"
fi

echo "[*] Results saved to: $OUTPUT_DIR/"
