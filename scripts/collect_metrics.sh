#!/usr/bin/env bash
# Metrics Collection Script
# Phase 1.3 - Production Readiness Plan
#
# Collects metrics from running kernel at regular intervals

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

# Configuration
INTERVAL=${INTERVAL:-5}         # Collection interval in seconds
DURATION=${DURATION:-300}        # Total duration in seconds (5 minutes default)
QMP_SOCK=${QMP_SOCK:-/tmp/sis-metrics-qmp.sock}
OUTPUT_FILE=${OUTPUT_FILE:-metrics.jsonl}  # JSON Lines format
FORMAT=${FORMAT:-json}            # json, prometheus, or simple

echo "=================================================="
echo "SIS Kernel Metrics Collector"
echo "=================================================="
echo "Interval:    ${INTERVAL}s"
echo "Duration:    ${DURATION}s"
echo "Format:      $FORMAT"
echo "Output:      $OUTPUT_FILE"
echo "QMP Socket:  $QMP_SOCK"
echo "=================================================="
echo ""

# Clean up old files
rm -f "$QMP_SOCK" "$OUTPUT_FILE"

# Start kernel with QMP in background
echo "[*] Starting kernel with QMP..."
export QMP=1
export QMP_SOCK
export BRINGUP=1
export SIS_FEATURES="${SIS_FEATURES:-llm,crypto-real}"

# Start kernel in background, redirect to log
KERNEL_LOG="/tmp/sis-metrics-kernel.log"
"$SCRIPT_DIR/uefi_run.sh" > "$KERNEL_LOG" 2>&1 &
QEMU_PID=$!

echo "[*] QEMU PID: $QEMU_PID"

# Cleanup function
cleanup() {
    echo ""
    echo "[*] Stopping metrics collection..."
    if [[ -n "${QEMU_PID:-}" ]] && kill -0 "$QEMU_PID" 2>/dev/null; then
        # Graceful shutdown
        "$SCRIPT_DIR/qmp_input.py" --socket "$QMP_SOCK" quit 2>/dev/null || true
        sleep 1

        # Force kill if still running
        if kill -0 "$QEMU_PID" 2>/dev/null; then
            kill -9 "$QEMU_PID" 2>/dev/null || true
        fi
    fi
    rm -f "$QMP_SOCK"

    echo "[*] Metrics saved to: $OUTPUT_FILE"
    echo "[*] Kernel log saved to: $KERNEL_LOG"
}

trap cleanup EXIT INT TERM

# Wait for shell prompt
echo "[*] Waiting for kernel to boot..."
timeout 60s bash -c "while ! grep -q 'LAUNCHING SHELL\|Shell ready' '$KERNEL_LOG'; do sleep 0.5; done" || {
    echo "[!] ERROR: Kernel did not boot in time"
    exit 1
}

echo "[*] Kernel ready!"

# Wait a bit for QMP to be ready
sleep 3

# Calculate number of collections
COLLECTIONS=$((DURATION / INTERVAL))

echo "[*] Starting metrics collection ($COLLECTIONS samples)"
echo ""

# Collect metrics in a loop
for i in $(seq 1 $COLLECTIONS); do
    TIMESTAMP=$(date +%s)
    echo -n "[$i/$COLLECTIONS] $(date '+%H:%M:%S') - "

    # Inject metrics command via QMP
    case "$FORMAT" in
        json)
            CMD="metrics json"
            ;;
        prometheus|prom)
            CMD="metrics prometheus"
            ;;
        simple)
            CMD="metrics simple"
            ;;
        *)
            CMD="metrics"
            ;;
    esac

    # Send command
    if "$SCRIPT_DIR/qmp_input.py" --socket "$QMP_SOCK" send-command "$CMD" 2>/dev/null; then
        echo "✓ collected"

        # Wait a bit for output to appear
        sleep 1

        # Extract metrics from kernel log
        case "$FORMAT" in
            json)
                # Extract last JSON line
                grep '^\{' "$KERNEL_LOG" | tail -1 >> "$OUTPUT_FILE" || true
                ;;
            prometheus)
                # Extract Prometheus metrics (last batch)
                # TODO: Better extraction logic
                tail -50 "$KERNEL_LOG" | grep '^[a-z_]' | grep -v '^\[' >> "$OUTPUT_FILE" || true
                ;;
            simple)
                # Extract simple format
                grep '^[a-z_].*=' "$KERNEL_LOG" | tail -1 >> "$OUTPUT_FILE" || true
                ;;
        esac
    else
        echo "✗ failed"
    fi

    # Sleep until next collection
    if [[ $i -lt $COLLECTIONS ]]; then
        sleep "$INTERVAL"
    fi

    # Check if kernel is still running
    if ! kill -0 "$QEMU_PID" 2>/dev/null; then
        echo ""
        echo "[!] WARNING: Kernel stopped unexpectedly"
        break
    fi
done

echo ""
echo "[*] Collection complete!"
echo "[*] Collected $(wc -l < "$OUTPUT_FILE") metric samples"

# Generate summary if JSON format
if [[ "$FORMAT" == "json" ]] && command -v jq >/dev/null 2>&1; then
    echo ""
    echo "=== Metrics Summary ==="
    echo "Uptime range:"
    jq -r '.uptime_ms' "$OUTPUT_FILE" 2>/dev/null | awk '{print "  " $0 " ms"}' | head -1
    jq -r '.uptime_ms' "$OUTPUT_FILE" 2>/dev/null | awk '{print "  " $0 " ms"}' | tail -1

    echo "Context switch P50:"
    jq -r '.ctx_switch_p50_ns' "$OUTPUT_FILE" 2>/dev/null | awk '{sum+=$1; count++} END {print "  avg: " sum/count " ns"}'

    echo "Heap usage:"
    jq -r '.heap_current_bytes' "$OUTPUT_FILE" 2>/dev/null | awk '{print "  " $0/1024/1024 " MiB"}' | tail -1
fi
