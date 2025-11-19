#!/usr/bin/env bash
set -euo pipefail

echo "[CI] rustfmt --check"
cargo fmt --all -- --check

echo "[CI] cargo clippy -D warnings (workspace)"
cargo clippy --workspace -D warnings

echo "[CI] Done"

