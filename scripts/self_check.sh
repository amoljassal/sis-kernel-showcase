#!/usr/bin/env bash
set -euo pipefail

# Minimal self-check: assert bring-up banners in QEMU logs.
# Usage:
#   Non-streaming (post-run):
#     ./scripts/uefi_run.sh 2>&1 | tee /tmp/sis_qemu.log
#     ./scripts/self_check.sh /tmp/sis_qemu.log
#   Streaming (live during run; exits once all markers seen):
#     ./scripts/uefi_run.sh 2>&1 | ./scripts/self_check.sh -s
#     ./scripts/self_check.sh -s /tmp/sis_qemu.log   # tails the file

STREAM=0
QUIET=0
TIMEOUT=0
LOGFILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    -s|--stream) STREAM=1; shift ;;
    -q|--quiet) QUIET=1; shift ;;
    --timeout) TIMEOUT=${2:-0}; shift 2 ;;
    -h|--help) echo "Usage: $0 [-s|--stream] [--timeout N] [-q|--quiet] [LOGFILE]"; exit 2 ;;
    *) if [[ -z "$LOGFILE" ]]; then LOGFILE="$1"; shift; else echo "[ERROR] Unexpected arg: $1" >&2; exit 2; fi ;;
  esac
done

markers=(
  "KERNEL(U)"
  "STACK OK"
  "MMU: SCTLR"
  "MMU ON"
  "UART: READY"
  "GIC: INIT"
  "LAUNCHING SHELL"
  "GIC: ENABLE PPI27"
)

if [[ $STREAM -eq 1 ]]; then
  # Streaming mode: print PASS as markers appear; exit as soon as all are seen.
  # Track which markers we have seen.
  remain=${#markers[@]}
  # Use a parallel array of seen flags (0/1) for portability (Bash 3.x compatible).
  seen=()
  for _ in "${markers[@]}"; do seen+=(0); done

  handle_line() {
    local line="$1"
    local i=0
    while [[ $i -lt ${#markers[@]} ]]; do
      if [[ ${seen[$i]} -eq 0 ]]; then
        local m="${markers[$i]}"
        if [[ "$line" == *"$m"* ]]; then
          if [[ $QUIET -eq 0 ]]; then
            echo "[CHECK] PASS: $m"
          fi
          seen[$i]=1
          remain=$((remain-1))
          if [[ $remain -le 0 ]]; then
            echo "[CHECK] ALL MARKERS SEEN"
            exit 0
          fi
        fi
      fi
      i=$((i+1))
    done
  }

  # Install optional timeout handler
  if [[ $TIMEOUT -gt 0 ]]; then
    trap '
      echo "[CHECK] TIMEOUT after '${TIMEOUT}'s" >&2
      i=0; fail=0
      while [[ $i -lt ${#markers[@]} ]]; do
        if [[ ${seen[$i]} -eq 0 ]]; then
          echo "[CHECK] FAIL: ${markers[$i]}"
          fail=1
        fi
        i=$((i+1))
      done
      exit ${fail}
    ' ALRM
    (
      sleep "$TIMEOUT"
      # Only signal if main process still alive
      if kill -0 $$ 2>/dev/null; then
        kill -ALRM $$ 2>/dev/null || true
      fi
    ) &
    TIMER_PID=$!
    trap 'kill "${TIMER_PID}" 2>/dev/null || true' EXIT
  else
    TIMER_PID=""
  fi

  if [[ -n "$LOGFILE" ]]; then
    if [[ ! -f "$LOGFILE" ]]; then
      echo "[ERROR] Log file not found: $LOGFILE" >&2
      exit 2
    fi
    # Tail the file from the beginning and follow updates.
    # Use process substitution to keep the while loop in the main shell.
    while IFS= read -r line; do
      handle_line "$line"
    done < <(tail -n +1 -f "$LOGFILE")
  else
    # Read from stdin; require a pipe.
    if [[ -t 0 ]]; then
      echo "Usage: $0 -s [LOGFILE]  (pipe or provide a log file)" >&2
      exit 2
    fi
    while IFS= read -r line; do
      handle_line "$line"
    done
  fi

  # If we reach here, input ended before all markers were seen.
  # Print missing markers and fail.
  i=0; fail=0
  while [[ $i -lt ${#markers[@]} ]]; do
    if [[ ${seen[$i]} -eq 0 ]]; then
      echo "[CHECK] FAIL: ${markers[$i]}"
      fail=1
    fi
    i=$((i+1))
  done
  # Cancel timer if running
  if [[ -n "${TIMER_PID}" ]]; then kill "${TIMER_PID}" 2>/dev/null || true; fi
  exit $fail
else
  # Non-streaming: read the whole input and check markers.
  content=""
  if [[ -n "$LOGFILE" ]]; then
    if [[ -f "$LOGFILE" ]]; then
      content=$(cat "$LOGFILE")
    else
      echo "[ERROR] Log file not found: $LOGFILE" >&2
      echo "Usage: $0 [LOGFILE] | $0 < LOGFILE" >&2
      exit 2
    fi
  else
    # No file arg: read stdin if piped, otherwise print usage
    if [[ -t 0 ]]; then
      echo "Usage: $0 [LOGFILE] | $0 < LOGFILE" >&2
      exit 2
    fi
    content=$(cat)
  fi

  fail=0
  for m in "${markers[@]}"; do
    if echo "$content" | grep -q "$m"; then
      if [[ $QUIET -eq 0 ]]; then
        echo "[CHECK] PASS: $m"
      fi
    else
      echo "[CHECK] FAIL: $m"
      fail=1
    fi
  done
  if [[ $fail -eq 0 ]]; then
    echo "[CHECK] ALL MARKERS SEEN"
  fi
  exit $fail
fi
