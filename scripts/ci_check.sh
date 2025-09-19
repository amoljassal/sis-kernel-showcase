#!/usr/bin/env bash
set -euo pipefail

echo "[CI] Clippy lint (best effort)"
if command -v cargo >/dev/null 2>&1; then
  if cargo clippy -V >/dev/null 2>&1; then
    cargo clippy --all-targets --all-features -q || true
  else
    echo "[CI] cargo-clippy not installed; skipping"
  fi
else
  echo "[CI] cargo not found; skipping clippy"
fi

echo "[CI] Validate schemas (if python present)"
if command -v python3 >/dev/null 2>&1; then
  if python3 -c 'import jsonschema' 2>/dev/null; then
    ./scripts/validate-metrics.sh || true
  else
    echo "[CI] jsonschema not installed; skipping"
  fi
else
  echo "[CI] python3 not found; skipping schema validation"
fi

echo "[CI] Done"

