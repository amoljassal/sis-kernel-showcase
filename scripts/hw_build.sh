#!/usr/bin/env bash
set -euo pipefail

# Hardware-lean build helper (uses hw-minimal marker + bringup). BOARD env is accepted for future use.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"

FEATURES="bringup,hw-minimal"

if [[ -n "${EXTRA_FEATURES:-}" ]]; then
  FEATURES+="${FEATURES:+,}${EXTRA_FEATURES}"
fi

echo "[*] Building kernel (hw-minimal preset) with features: $FEATURES"
rustup target add aarch64-unknown-none >/dev/null 2>&1 || true
export RUSTFLAGS="-C link-arg=-T$ROOT_DIR/crates/kernel/src/arch/aarch64/aarch64-qemu.ld"
cargo +nightly build -p sis_kernel -Z build-std=core,alloc --target aarch64-unknown-none --features "$FEATURES"

echo "[*] Done. Kernel ELF: target/aarch64-unknown-none/debug/sis_kernel"
