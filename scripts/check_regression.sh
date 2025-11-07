#!/usr/bin/env bash
# Check for log regressions by comparing normalized logs
# Phase 1.1 - Production Readiness Plan

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
BASELINE_DIR="$ROOT_DIR/tests/baselines"

# Configuration
CONFIG_NAME=${CONFIG_NAME:-boot-default}
BASELINE_FILE="$BASELINE_DIR/${CONFIG_NAME}.json"
NEW_LOG=${NEW_LOG:-/tmp/sis-new-boot.log}
ALLOW_ADDITIONS=${ALLOW_ADDITIONS:-1}  # Allow new log entries by default
SHOW_DIFF=${SHOW_DIFF:-1}  # Show diff by default

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Check for log regressions by comparing with baseline."
    echo ""
    echo "Options:"
    echo "  CONFIG_NAME=name     Configuration name (default: boot-default)"
    echo "  NEW_LOG=path         Path to new log file (default: /tmp/sis-new-boot.log)"
    echo "  ALLOW_ADDITIONS=0|1  Allow new log entries (default: 1)"
    echo "  SHOW_DIFF=0|1        Show diff output (default: 1)"
    echo ""
    echo "Environment:"
    echo "  Set LOG_FORMAT=json before running the kernel"
    echo ""
    echo "Example:"
    echo "  # Capture new log"
    echo "  LOG_FORMAT=json BRINGUP=1 ./scripts/uefi_run.sh 2>&1 | grep '^{' > /tmp/new.log"
    echo "  # Check for regressions"
    echo "  NEW_LOG=/tmp/new.log ./scripts/check_regression.sh"
    exit 1
}

if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
    usage
fi

# Check if baseline exists
if [[ ! -f "$BASELINE_FILE" ]]; then
    echo "[!] ERROR: Baseline not found: $BASELINE_FILE"
    echo "[!] Run: ./scripts/capture_baseline.sh"
    exit 1
fi

# Check if new log exists
if [[ ! -f "$NEW_LOG" ]]; then
    echo "[!] ERROR: New log not found: $NEW_LOG"
    echo "[!] Capture a new log first with LOG_FORMAT=json"
    exit 1
fi

echo "[*] Checking for regressions..."
echo "[*] Baseline: $BASELINE_FILE ($(wc -l < "$BASELINE_FILE") events)"
echo "[*] New log:  $NEW_LOG ($(wc -l < "$NEW_LOG") events)"

# Normalize both logs
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

NORMALIZED_BASELINE="$TEMP_DIR/baseline.normalized"
NORMALIZED_NEW="$TEMP_DIR/new.normalized"

echo "[*] Normalizing logs..."
"$SCRIPT_DIR/normalize_log.py" < "$BASELINE_FILE" > "$NORMALIZED_BASELINE"
"$SCRIPT_DIR/normalize_log.py" < "$NEW_LOG" > "$NORMALIZED_NEW"

echo "[*] Normalized baseline: $(wc -l < "$NORMALIZED_BASELINE") events"
echo "[*] Normalized new:      $(wc -l < "$NORMALIZED_NEW") events"

# Sort for comparison (makes diff more meaningful)
sort "$NORMALIZED_BASELINE" > "$TEMP_DIR/baseline.sorted"
sort "$NORMALIZED_NEW" > "$TEMP_DIR/new.sorted"

# Perform diff
DIFF_OUTPUT="$TEMP_DIR/diff.txt"
if diff -u "$TEMP_DIR/baseline.sorted" "$TEMP_DIR/new.sorted" > "$DIFF_OUTPUT" 2>&1; then
    echo "[✓] PASS: No regressions detected"
    echo "[*] Logs are identical (after normalization)"
    exit 0
fi

# Analyze differences
ADDITIONS=$(grep -c '^+{' "$DIFF_OUTPUT" || true)
REMOVALS=$(grep -c '^-{' "$DIFF_OUTPUT" || true)

echo ""
echo "[!] DIFFERENCES DETECTED:"
echo "    Additions: $ADDITIONS events"
echo "    Removals:  $REMOVALS events"

if [[ "$SHOW_DIFF" == "1" ]]; then
    echo ""
    echo "=== Diff Output (normalized, sorted) ==="
    cat "$DIFF_OUTPUT" | head -100
    echo ""
    if [[ $(wc -l < "$DIFF_OUTPUT") -gt 100 ]]; then
        echo "(... diff truncated, full output in $DIFF_OUTPUT)"
    fi
fi

# Decide if this is a regression
REGRESSION=0

if [[ "$REMOVALS" -gt 0 ]]; then
    echo "[!] REGRESSION: $REMOVALS log entries disappeared"
    echo "[!] This may indicate missing functionality"
    REGRESSION=1
fi

if [[ "$ALLOW_ADDITIONS" == "0" ]] && [[ "$ADDITIONS" -gt 0 ]]; then
    echo "[!] REGRESSION: $ADDITIONS new log entries (additions not allowed)"
    REGRESSION=1
elif [[ "$ADDITIONS" -gt 0 ]]; then
    echo "[*] INFO: $ADDITIONS new log entries (allowed)"
fi

# Summary
echo ""
if [[ "$REGRESSION" == "1" ]]; then
    echo "[✗] FAIL: Log regression detected"
    echo "[!] Action required: Review changes or update baseline"
    echo ""
    echo "To update baseline (if changes are intentional):"
    echo "  cp $NEW_LOG $BASELINE_FILE"
    exit 1
else
    echo "[✓] PASS: No regressions (additions allowed)"
    exit 0
fi
