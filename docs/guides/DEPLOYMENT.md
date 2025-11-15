# Deployment Guide - M8

Comprehensive guide for building, testing, and deploying the SIS Kernel Desktop Application.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Local Development](#local-development)
3. [Testing](#testing)
4. [Building for Production](#building-for-production)
5. [Packaging](#packaging)
6. [CI/CD](#cicd)
7. [Deployment](#deployment)

## Prerequisites

### Development Tools

- **Node.js** 18+ and pnpm 8+
- **Rust** stable toolchain
- **Tauri CLI** 2.0+
- **Platform-specific dependencies**:
  - **Linux**: `libgtk-3-dev`, `libwebkit2gtk-4.0-dev`, `librsvg2-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools

### Installation

```bash
# Install pnpm globally
npm install -g pnpm

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli
```

## Local Development

```bash
# Install dependencies
pnpm install

# Run daemon (in one terminal)
cd apps/daemon
cargo run --release

# Run desktop app (in another terminal)
cd apps/desktop
pnpm tauri dev
```

## Testing

### Unit Tests

```bash
# Daemon unit tests
cd apps/daemon
cargo test

# Frontend unit tests (if configured)
cd apps/desktop
pnpm test
```

### E2E Tests

```bash
# Install Playwright
cd apps/desktop
pnpm exec playwright install --with-deps

# Run E2E tests
pnpm test:e2e

# Run with UI
pnpm test:e2e:ui

# Generate test report
pnpm exec playwright show-report
```

### Linting

```bash
# Lint desktop app
cd apps/desktop
pnpm lint
pnpm format:check

# Fix lint issues
pnpm lint:fix
pnpm format
```

## Building for Production

### Daemon

```bash
cd apps/daemon
cargo build --release

# Binary location:
# - Linux/macOS: target/release/sisctl
# - Windows: target/release/sisctl.exe
```

### Desktop App

```bash
cd apps/desktop

# Build frontend only
pnpm build

# Build full Tauri app
pnpm tauri build
```

## Packaging

### macOS (.dmg)

```bash
cd apps/desktop
pnpm tauri build --target universal-apple-darwin

# Output: src-tauri/target/release/bundle/dmg/
```

**Code Signing (macOS)**:
```bash
# Set environment variables
export APPLE_CERTIFICATE="..."
export APPLE_CERTIFICATE_PASSWORD="..."
export APPLE_ID="developer@example.com"
export APPLE_PASSWORD="app-specific-password"

# Build with signing
pnpm tauri build --target universal-apple-darwin
```

### Linux (.AppImage, .deb)

```bash
cd apps/desktop
pnpm tauri build

# Outputs:
# - src-tauri/target/release/bundle/appimage/
# - src-tauri/target/release/bundle/deb/
```

### Windows (.msi, .exe)

```bash
cd apps/desktop
pnpm tauri build

# Outputs:
# - src-tauri/target/release/bundle/msi/
# - src-tauri/target/release/bundle/nsis/
```

## CI/CD

### GitHub Actions

The project includes a comprehensive CI/CD workflow at `.github/workflows/desktop-ci.yml`:

**Triggered on**:
- Push to `main` or `develop`
- Pull requests to `main`
- Tag pushes (for releases)

**Jobs**:
1. **Lint and Test**: Linting, unit tests, and builds
2. **E2E Tests**: Playwright end-to-end tests
3. **Build Tauri**: Multi-platform Tauri builds (Linux/macOS/Windows)
4. **Release**: Automated GitHub releases for tagged versions

### Manual Workflow Dispatch

```yaml
# Add to workflow for manual triggers
on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to build'
        required: true
        default: 'latest'
```

## Deployment

### Local Installation

After building, install the appropriate package for your platform:

**macOS**:
```bash
open apps/desktop/src-tauri/target/release/bundle/dmg/SIS-Kernel_*.dmg
```

**Linux**:
```bash
# AppImage
chmod +x apps/desktop/src-tauri/target/release/bundle/appimage/sis-kernel_*.AppImage
./sis-kernel_*.AppImage

# Debian
sudo dpkg -i apps/desktop/src-tauri/target/release/bundle/deb/sis-kernel_*.deb
```

**Windows**:
```powershell
# Run installer
.\apps\desktop\src-tauri\target\release\bundle\msi\SIS-Kernel_*.msi
```

### GitHub Releases

Automated releases are created for tagged versions:

```bash
# Create and push tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# GitHub Actions will:
# 1. Run all tests
# 2. Build for all platforms
# 3. Create GitHub release
# 4. Upload binaries as release assets
```

### Update Distribution

For distributing updates to users:

1. **GitHub Releases**: Users download from GitHub Releases page
2. **Tauri Updater** (future): In-app update notifications
3. **Package Managers**: Submit to Homebrew, winget, snap, etc.

## Troubleshooting

### Build Failures

**Linux - WebKit errors**:
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-dev
```

**macOS - Code signing**:
```bash
# List available identities
security find-identity -v -p codesigning

# Use specific identity
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
```

**Windows - MSVC not found**:
Install Visual Studio Build Tools 2022 with "Desktop development with C++" workload.

### Test Failures

**Playwright timeouts**:
```bash
# Increase timeout in playwright.config.ts
timeout: 60000  # 60 seconds
```

**WebSocket connection issues**:
Ensure daemon is running before E2E tests:
```bash
cd apps/daemon
cargo run --release &
# Wait for startup
sleep 5
cd ../desktop
pnpm test:e2e
```

## Security Considerations

- **Code Signing**: Always sign production releases
- **Secrets Management**: Use GitHub Secrets for credentials
- **Dependency Auditing**: Run `pnpm audit` and `cargo audit` regularly
- **CSP**: Content Security Policy configured in `tauri.conf.json`
- **Localhost Only**: Daemon binds to `127.0.0.1` only

## Performance Optimization

### Production Build Optimizations

**Tauri config** (`tauri.conf.json`):
```json
{
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist",
    "withGlobalTauri": true
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "appimage", "msi"],
    "resources": [],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

**Rust optimizations** (`Cargo.toml`):
```toml
[profile.release]
opt-level = "z"  # Optimize for size
lto = true       # Link-time optimization
codegen-units = 1
strip = true     # Strip symbols
```

## Monitoring and Analytics

### Error Tracking

Consider integrating:
- **Sentry**: For error tracking
- **PostHog**: For product analytics
- **Prometheus**: For metrics (daemon-side)

### Logging

Production logs:
```bash
# Daemon logs
RUST_LOG=info cargo run --release

# Tauri logs (check platform-specific locations)
# macOS: ~/Library/Logs/com.sis-kernel.app/
# Linux: ~/.local/share/sis-kernel/logs/
# Windows: %APPDATA%\sis-kernel\logs\
```

## References

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Playwright Testing](https://playwright.dev/)
- [GitHub Actions](https://docs.github.com/en/actions)
- [Code Signing Guide](https://tauri.app/v1/guides/distribution/sign-windows)

## Changelog

- **2025-11-05**: Initial deployment guide (M8 completion)
