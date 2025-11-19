#!/usr/bin/env bash
set -euo pipefail

# Simple smoke wrapper that runs the automated shell tests with a short timeout

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export TIMEOUT=${TIMEOUT:-60}
export SIS_FEATURES=${SIS_FEATURES:-"ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys"}

echo "[SMOKE] Running QEMU smoke with TIMEOUT=${TIMEOUT}s"
"${SCRIPT_DIR}/automated_shell_tests.sh"

echo "[SMOKE] Done"

