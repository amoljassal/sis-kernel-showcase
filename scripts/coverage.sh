#!/usr/bin/env bash
set -euo pipefail

# Host-side coverage using cargo-llvm-cov

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "[*] Installing cargo-llvm-cov"
  cargo install cargo-llvm-cov >/dev/null 2>&1 || true
fi

echo "[*] Ensuring llvm-tools-preview is installed"
rustup component add llvm-tools-preview >/dev/null 2>&1 || true

# 1) Try workspace coverage (host-testable crates only). Exclude sis-testing to run separately.
echo "[*] Cleaning previous coverage"
cargo llvm-cov clean --workspace || true

echo "[*] Running workspace coverage (excluding sis-testing)"
pushd "$ROOT_DIR" >/dev/null
set +e
cargo llvm-cov --workspace --exclude sis-testing --no-report
WS_STATUS=$?
set -e
if [[ $WS_STATUS -ne 0 ]]; then
  echo "[!] Workspace coverage had failures (expected for no_std crates). Continuing."
fi
popd >/dev/null

# 2) Run coverage for crates/testing (host-run)
echo "[*] Running coverage for crates/testing"
pushd "$ROOT_DIR/crates/testing" >/dev/null
cargo llvm-cov --html --output-path target/llvm-cov/html --ignore-filename-regex 'ai_benchmark\.rs|benchmark\.rs'
cargo llvm-cov --lcov --output-path lcov.info --ignore-filename-regex 'ai_benchmark\.rs|benchmark\.rs'
echo "[*] Coverage HTML: crates/testing/target/llvm-cov/html/index.html"
popd >/dev/null

# 3) Merge artifacts into a single folder for convenience
MERGE_DIR="$ROOT_DIR/target/coverage"
mkdir -p "$MERGE_DIR"
if [[ -d "$ROOT_DIR/target/llvm-cov" ]]; then
  cp -R "$ROOT_DIR/target/llvm-cov" "$MERGE_DIR/workspace" 2>/dev/null || true
fi
cp -R "$ROOT_DIR/crates/testing/target/llvm-cov/html" "$MERGE_DIR/testing-html" 2>/dev/null || true
cp "$ROOT_DIR/crates/testing/lcov.info" "$MERGE_DIR/lcov.info" 2>/dev/null || true
echo "[*] Merged coverage under: target/coverage"

# Optional: MODE=embedded placeholder for future QEMU-instrumented coverage
if [[ "${MODE:-}" == "embedded" ]]; then
  echo "[i] Embedded coverage mode is a future enhancement (QEMU-instrumented).\n    Current script completes host-side coverage and merges artifacts."
fi
