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
echo "[CI] HW-first guard (no hardcoded MMIO outside platform)"
if command -v rg >/dev/null 2>&1; then
  if command -v jq >/dev/null 2>&1; then
    ./scripts/ci_guard_hwfirst.sh || true
  else
    echo "[CI] jq not installed; skipping HW-first guard"
  fi
else
  echo "[CI] ripgrep not installed; skipping HW-first guard"
fi
echo "[CI] Optional build with graphctl-framed (best effort)"
if command -v cargo >/dev/null 2>&1; then
  if cargo +nightly -V >/dev/null 2>&1; then
    rustup target add aarch64-unknown-none >/dev/null 2>&1 || true
    (cargo +nightly build -p sis_kernel -Z build-std=core,alloc --target aarch64-unknown-none --features graphctl-framed || true)
  else
    echo "[CI] nightly toolchain not available; skipping framed build"
  fi
else
  echo "[CI] cargo not found; skipping framed build"
fi
