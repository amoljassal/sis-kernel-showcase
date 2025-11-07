#!/usr/bin/env bash
# Build SIS Kernel Docker image with reproducibility
# Phase 2.2 - Production Readiness Plan

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"

# Get git commit hash for tagging
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")

# Configuration
RUST_VERSION=${RUST_VERSION:-1.75}
RUST_NIGHTLY_DATE=${RUST_NIGHTLY_DATE:-2025-09-08}
IMAGE_NAME=${IMAGE_NAME:-sis-kernel}

echo "[*] Building SIS Kernel Docker image"
echo "    Rust version: $RUST_VERSION"
echo "    Nightly date: $RUST_NIGHTLY_DATE"
echo "    Git commit:   $GIT_COMMIT"
echo "    Git branch:   $GIT_BRANCH"

# Build image
docker build \
    --build-arg RUST_NIGHTLY_DATE="$RUST_NIGHTLY_DATE" \
    --tag "${IMAGE_NAME}:${GIT_COMMIT}" \
    --tag "${IMAGE_NAME}:latest-${GIT_BRANCH}" \
    .

echo "[*] Build complete!"
echo ""
echo "Images tagged:"
echo "  - ${IMAGE_NAME}:${GIT_COMMIT}"
echo "  - ${IMAGE_NAME}:latest-${GIT_BRANCH}"

# Tag as latest if on main branch
if [[ "$GIT_BRANCH" == "main" ]]; then
    echo "[*] Tagging as latest (main branch)"
    docker tag "${IMAGE_NAME}:${GIT_COMMIT}" "${IMAGE_NAME}:latest"
    echo "  - ${IMAGE_NAME}:latest"
fi

echo ""
echo "Usage examples:"
echo "  # Run interactive shell in container"
echo "  docker run --rm -it ${IMAGE_NAME}:${GIT_COMMIT} bash"
echo ""
echo "  # Run tests"
echo "  docker run --rm ${IMAGE_NAME}:${GIT_COMMIT} bash ./scripts/capture_baseline.sh"
echo ""
echo "  # Build kernel"
echo "  docker run --rm ${IMAGE_NAME}:${GIT_COMMIT} cargo +nightly build -p sis_kernel"
echo ""
echo "  # Use docker-compose"
echo "  docker-compose up sis-kernel"
echo "  docker-compose --profile test up sis-kernel-test"
echo "  docker-compose --profile ci up sis-kernel-ci"
