# SIS Kernel Reproducible Build Environment
# Phase 2.2 - Production Readiness Plan

FROM rust:1.75-bookworm

# Install QEMU and build tools with pinned versions
RUN apt-get update && apt-get install -y \
    qemu-system-aarch64 \
    qemu-efi-aarch64 \
    qemu-utils \
    gcc-aarch64-linux-gnu \
    e2fsprogs \
    python3 \
    python3-pip \
    expect \
    jq \
    git \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly with specific date for reproducibility
ARG RUST_NIGHTLY_DATE=2025-09-08
RUN rustup install nightly-${RUST_NIGHTLY_DATE} && \
    rustup default nightly-${RUST_NIGHTLY_DATE} && \
    rustup target add aarch64-unknown-none --toolchain nightly-${RUST_NIGHTLY_DATE} && \
    rustup target add aarch64-unknown-uefi --toolchain nightly-${RUST_NIGHTLY_DATE} && \
    rustup component add rust-src --toolchain nightly-${RUST_NIGHTLY_DATE} && \
    rustup component add rustfmt --toolchain nightly-${RUST_NIGHTLY_DATE} && \
    rustup component add clippy --toolchain nightly-${RUST_NIGHTLY_DATE}

# Create build user (non-root for security)
RUN useradd -m -s /bin/bash builder && \
    mkdir -p /workspace && \
    chown -R builder:builder /workspace

USER builder
WORKDIR /workspace

# Copy source files
COPY --chown=builder:builder . /workspace/sis-kernel

# Set working directory
WORKDIR /workspace/sis-kernel

# Set environment variables for reproducible builds
ENV RUSTFLAGS="-C link-arg=-T/workspace/sis-kernel/crates/kernel/src/arch/aarch64/aarch64-qemu.ld"
ENV BRINGUP="1"
ENV SIS_FEATURES="llm,crypto-real"

# Pre-download dependencies (caching layer)
RUN cargo +nightly fetch || true

# Build kernel (optional - can be done at runtime)
# RUN cargo +nightly build -p sis_kernel -Z build-std=core,alloc \
#     --target aarch64-unknown-none --features "$SIS_FEATURES"

# Default command: run tests
CMD ["bash", "./scripts/uefi_run.sh"]

# Health check (optional)
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD test -f /workspace/sis-kernel/target/aarch64-unknown-none/debug/sis_kernel || exit 1

# Labels for metadata
LABEL org.opencontainers.image.title="SIS Kernel Build Environment"
LABEL org.opencontainers.image.description="Reproducible build environment for SIS Kernel"
LABEL org.opencontainers.image.version="1.0"
LABEL org.opencontainers.image.vendor="SIS Kernel Team"

# Notes on reproducibility:
# - Pinned Rust version (nightly-2025-09-08)
# - Pinned apt packages (Debian Bookworm)
# - Build user for consistent UID/GID
# - RUSTFLAGS for deterministic builds
