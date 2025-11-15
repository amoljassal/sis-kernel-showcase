#!/usr/bin/env bash
set -euo pipefail

# Simple helper to demo the kernel LLM service via the shell.
# Usage:
#   ./scripts/llm_demo.sh            # LLM only
#   DET=1 ./scripts/llm_demo.sh      # LLM + deterministic budgeting

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

echo "== SIS Kernel LLM Demo =="
if [[ "${DET:-}" != "" ]]; then
  echo "Mode: LLM + deterministic budgeting"
  echo
  echo "In the SIS shell, run:"
  echo "  llmctl load --wcet-cycles 25000"
  echo "  llmctl budget --period-ns 1000000000 --max-tokens-per-period 8"
  echo "  llminfer \"why was op B slower than op A?\" --max-tokens 8"
  echo "  llmstream \"why was op B slower than op A?\" --max-tokens 8 --chunk 2"
  echo "  llmstats"
  echo
  SIS_FEATURES="llm,deterministic" BRINGUP=1 "$SCRIPT_DIR/uefi_run.sh"
else
  echo "Mode: LLM"
  echo
  echo "In the SIS shell, run:"
  echo "  llmctl load --wcet-cycles 25000"
  echo "  llminfer \"why was op B slower than op A?\" --max-tokens 8"
  echo "  llmstream \"why was op B slower than op A?\" --max-tokens 8 --chunk 2"
  echo "  llmstats"
  echo
  SIS_FEATURES="llm" BRINGUP=1 "$SCRIPT_DIR/uefi_run.sh"
fi

