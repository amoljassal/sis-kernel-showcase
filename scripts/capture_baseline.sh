#!/usr/bin/env bash
# Capture baseline JSON logs for regression testing
# Phase 1.1 - Production Readiness Plan

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
BASELINE_DIR="$ROOT_DIR/tests/baselines"

# Configuration
TIMEOUT=${TIMEOUT:-60}
CONFIG_NAME=${CONFIG_NAME:-boot-default}

echo "[*] Capturing baseline for configuration: $CONFIG_NAME"
echo "[*] Timeout: ${TIMEOUT}s"

# Ensure baseline directory exists
mkdir -p "$BASELINE_DIR"

# Set LOG_FORMAT=json to enable structured logging
export LOG_FORMAT=json
export BRINGUP=1

# Optional: allow caller to override features
if [[ -z "${SIS_FEATURES:-}" ]]; then
    export SIS_FEATURES="llm,crypto-real"
fi

echo "[*] Features: $SIS_FEATURES"
echo "[*] Starting kernel with JSON logging..."

# Capture logs
LOG_FILE="$BASELINE_DIR/${CONFIG_NAME}.json"
TEMP_LOG="/tmp/sis-baseline-capture-$$.log"

# Run kernel and capture output
timeout "${TIMEOUT}s" "$SCRIPT_DIR/uefi_run.sh" > "$TEMP_LOG" 2>&1 || true

# Extract JSON-formatted log lines
grep -E '^\{' "$TEMP_LOG" > "$LOG_FILE" || true

# Count captured events
EVENT_COUNT=$(wc -l < "$LOG_FILE")

if [[ "$EVENT_COUNT" -eq 0 ]]; then
    echo "[!] WARNING: No JSON events captured!"
    echo "[!] Check if structured logging is properly enabled"
    echo "[!] Recent output:"
    tail -20 "$TEMP_LOG"
    exit 1
fi

echo "[*] Baseline captured: $EVENT_COUNT events"
echo "[*] Saved to: $LOG_FILE"

# Generate summary
echo "[*] Event summary:"
if command -v jq >/dev/null 2>&1; then
    echo "  By subsystem:"
    jq -r '.subsystem' "$LOG_FILE" 2>/dev/null | sort | uniq -c | sort -rn || true
    echo "  By level:"
    jq -r '.level' "$LOG_FILE" 2>/dev/null | sort | uniq -c | sort -rn || true
else
    echo "  (Install jq for detailed analysis)"
    grep -o '"subsystem":"[^"]*"' "$LOG_FILE" | sort | uniq -c | sort -rn | head -10 || true
fi

# Clean up temp file
rm -f "$TEMP_LOG"

echo "[*] Done!"
