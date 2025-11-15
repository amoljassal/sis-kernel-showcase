# SIS Kernel Build Guide

## Reproducible Builds

### Using Docker (Recommended for CI)

Docker provides a consistent, reproducible build environment with pinned dependencies.

#### Quick Start

```bash
# Build the Docker image
./scripts/docker_build.sh

# Or use docker-compose
docker-compose build
```

#### Running Tests

```bash
# Run all tests
docker-compose --profile test up sis-kernel-test

# Run CI pipeline
docker-compose --profile ci up sis-kernel-ci

# Interactive development
docker run --rm -it sis-kernel:latest bash
```

### Pinned Versions

For reproducibility, all dependencies are pinned:

- **Rust**: nightly-2025-09-08
- **Debian**: Bookworm (stable)
- **QEMU**: 8.0+ (from Debian stable)
- **GCC**: 12.2.0 (aarch64-linux-gnu)

### Local Build (Without Docker)

#### Prerequisites

1. Install Rust nightly:
```bash
rustup install nightly-2025-09-08
rustup default nightly-2025-09-08
rustup target add aarch64-unknown-none aarch64-unknown-uefi
rustup component add rust-src
```

2. Install system dependencies:
```bash
# Ubuntu/Debian
sudo apt-get install -y \
    qemu-system-aarch64 \
    qemu-efi-aarch64 \
    gcc-aarch64-linux-gnu \
    e2fsprogs \
    python3 \
    expect
```

#### Build and Run

```bash
# Build and run kernel
BRINGUP=1 SIS_FEATURES="llm,crypto-real" ./scripts/uefi_run.sh

# Build only
cargo +nightly build -p sis_kernel \
    -Z build-std=core,alloc \
    --target aarch64-unknown-none \
    --features llm,crypto-real
```

### Feature Flags

The kernel supports various feature flags:

- `llm` - LLM/AI features
- `crypto-real` - Real cryptographic implementations
- `bringup` - Early bringup/debug features
- `sntp` - SNTP time synchronization
- `chaos` - Chaos engineering/failure injection

Example:
```bash
SIS_FEATURES="llm,crypto-real,sntp" ./scripts/uefi_run.sh
```

## Deterministic Builds

### Environment Variables

For reproducible builds, set:

```bash
export QEMU_RNG_SEED=12345
export QEMU_CLOCK=vm
export SOURCE_DATE_EPOCH=0
```

### Verification

To verify build reproducibility:

```bash
# Build 1
./scripts/docker_build.sh
docker run --rm sis-kernel:latest cargo +nightly build -p sis_kernel > /tmp/build1.log

# Build 2 (clean)
docker rmi sis-kernel:latest
./scripts/docker_build.sh
docker run --rm sis-kernel:latest cargo +nightly build -p sis_kernel > /tmp/build2.log

# Compare
diff /tmp/build1.log /tmp/build2.log
```

## CI/CD Integration

### GitHub Actions

The project includes CI workflows:

- `.github/workflows/ci.yml` - Main CI pipeline
- `.github/workflows/soak-test.yml` - Weekend soak tests

### Running CI Locally

```bash
# Simulate CI environment
docker-compose --profile ci up sis-kernel-ci
```

## Troubleshooting

### Build Errors

**Error: "failed to get crates.io index"**
- Solution: Check network connectivity, or use offline mode with vendored dependencies

**Error: "QEMU not found"**
- Solution: Install QEMU: `sudo apt-get install qemu-system-aarch64`

**Error: "target not found"**
- Solution: Add target: `rustup target add aarch64-unknown-none`

### Docker Issues

**Error: "permission denied"**
- Solution: Add user to docker group: `sudo usermod -aG docker $USER`

**Error: "out of disk space"**
- Solution: Clean up: `docker system prune -a`

## Performance Tips

### Faster Builds

1. Use sccache for caching:
```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
```

2. Enable parallel builds:
```bash
export CARGO_BUILD_JOBS=8
```

3. Use Docker BuildKit:
```bash
export DOCKER_BUILDKIT=1
docker build .
```

## Additional Resources

- [Production Readiness Plan](../docs/plans/PRODUCTION-READINESS-PLAN.md)
- [Testing Guide](../docs/TESTING.md) (to be created)
- [Contributing Guide](../CONTRIBUTING.md) (to be created)
