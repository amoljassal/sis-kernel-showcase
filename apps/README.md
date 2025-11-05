# SIS Kernel Desktop App - Milestone 0

Cross-platform desktop application for managing and controlling SIS Kernel QEMU instances.

## Architecture

### Monorepo Structure

```
apps/
├── daemon/          # Rust control daemon (sisctl)
│   ├── src/
│   │   ├── api/     # REST + WebSocket handlers
│   │   ├── qemu/    # QEMU supervisor
│   │   └── parser/  # UART output parser
│   └── Cargo.toml
│
└── desktop/         # Tauri desktop app
    ├── src/         # React + TypeScript frontend
    │   ├── components/
    │   ├── lib/
    │   └── App.tsx
    ├── src-tauri/   # Tauri Rust backend
    └── package.json

packages/
├── protos/          # OpenAPI schemas (future)
├── client/          # Generated TypeScript client (future)
└── ui/              # Shared design system (future)
```

### Component Responsibilities

#### Daemon (sisctl)
- Launches and supervises QEMU via `scripts/uefi_run.sh`
- Parses UART output (stdout/stderr)
- Detects boot markers (KERNEL(U), STACK OK, MMU ON, etc.)
- Parses METRIC lines
- Exposes REST API on `127.0.0.1:8871`
- Streams events via WebSocket at `/events`

#### Desktop App
- Tauri app with React frontend
- Auto-launches daemon if not running
- Connects to daemon REST API + WebSocket
- Displays terminal output (xterm.js)
- Shows boot progress checklist
- Visualizes metrics with sparklines
- Provides QEMU profile selector

## API Endpoints

### REST API (Base: `/api/v1`)

- **POST `/qemu/run`** - Start QEMU with configuration
  ```json
  {
    "features": ["llm", "graph-demo"],
    "env": {
      "BRINGUP": "1",
      "SIS_FEATURES": "llm,graph-demo"
    }
  }
  ```

- **POST `/qemu/stop`** - Stop QEMU

- **GET `/qemu/status`** - Get QEMU status
  ```json
  {
    "state": "running",
    "pid": 12345,
    "uptime_secs": 60,
    "features": ["llm"],
    "lines_processed": 1234,
    "events_emitted": 567
  }
  ```

- **GET `/health`** - Health check

### WebSocket (`/events`)

Streams events in JSON format:

```json
{
  "type": "raw_line",
  "line": "KERNEL(U) entry...",
  "timestamp": 1234567890
}
```

```json
{
  "type": "parsed",
  "event": {
    "type": "marker",
    "marker": "KernelU",
    "timestamp": 1234567890
  }
}
```

```json
{
  "type": "parsed",
  "event": {
    "type": "metric",
    "name": "irq_latency_ns",
    "value": 1234.5,
    "timestamp": 1234567890
  }
}
```

## Development Setup

### Prerequisites

- **Rust** (stable) - for daemon and Tauri backend
- **Node.js 18+** - for frontend
- **pnpm 8+** - package manager
- **QEMU** (aarch64-system) - for running SIS kernel
- **EDK2 UEFI firmware** - for QEMU boot

### Installation

1. **Install pnpm dependencies:**
   ```bash
   pnpm install
   ```

2. **Build daemon:**
   ```bash
   cd apps/daemon
   cargo build --release
   ```

3. **Run daemon (development):**
   ```bash
   cargo run -p sisctl
   # or
   pnpm -F daemon dev
   ```

4. **Run desktop app (development):**
   ```bash
   cd apps/desktop
   pnpm tauri dev
   # or from root
   pnpm -F desktop tauri:dev
   ```

### Running Both Together

From repository root:

```bash
pnpm dev
```

This launches both daemon and desktop app concurrently.

## Features

### Milestone 0 Deliverables ✅

- [x] Monorepo structure with pnpm workspace
- [x] Daemon (sisctl) with QEMU supervisor
- [x] UART parser for metrics, banners, and boot markers
- [x] REST API with OpenAPI schema
- [x] WebSocket event streaming
- [x] Tauri desktop app with React + TypeScript
- [x] Terminal component (xterm.js)
- [x] Boot markers checklist (9 markers)
- [x] Metrics sparkline visualization
- [x] QEMU profile selector with feature flags
- [x] Auto-launch daemon from GUI

### Boot Markers

The app tracks 9 boot markers:

1. **KERNEL(U)** - Kernel entry point
2. **STACK OK** - Stack initialized
3. **MMU: SCTLR** - MMU control register configured
4. **MMU ON** - Memory management enabled
5. **UART: READY** - UART driver initialized
6. **GIC: INIT** - Interrupt controller initialized
7. **VECTORS OK** - Exception vectors installed
8. **LAUNCHING SHELL** - Shell starting
9. **sis>** - Shell prompt ready

### Supported Feature Flags

- `llm` - LLM subsystem
- `graph-demo` - Graph computation demos
- `perf-verbose` - Verbose performance metrics
- `virtio-console` - VirtIO console driver
- `deterministic` - CBS+EDF scheduler
- `demos` - All demo commands

## Configuration

### Daemon

Environment variables:
- `SISCTL_BIND` - Bind address (default: `127.0.0.1:8871`)
- `RUST_LOG` - Log level (default: `info,sisctl=debug`)

### Desktop App

Vite environment variables:
- `VITE_DAEMON_URL` - Daemon REST URL (default: `http://localhost:8871`)
- `VITE_WS_URL` - WebSocket URL (default: `ws://localhost:8871/events`)

## Testing

### Daemon Tests

```bash
cd apps/daemon
cargo test
```

### Desktop App Linting

```bash
cd apps/desktop
pnpm lint
pnpm format:check
```

## Next Steps (Milestone 1+)

- [ ] Shell command execution API (`POST /shell/exec`)
- [ ] Self-check automation (`GET /selfcheck`)
- [ ] Metrics ingestion with downsampling
- [ ] Autonomy control panel (`autoctl` wrapper)
- [ ] Memory approvals UI (`memctl` wrapper)
- [ ] What-if scenario simulator
- [ ] Graph/LLM/Scheduling panels
- [ ] i18n support
- [ ] Playwright E2E tests
- [ ] CI/CD packaging (macOS/Linux/Windows)

## Troubleshooting

### Daemon won't start

1. Check if port 8871 is available:
   ```bash
   lsof -i :8871
   ```

2. Check logs:
   ```bash
   RUST_LOG=debug cargo run -p sisctl
   ```

### QEMU fails to start

1. Verify `scripts/uefi_run.sh` is executable:
   ```bash
   chmod +x scripts/uefi_run.sh
   ```

2. Check EDK2 firmware path (macOS Homebrew):
   ```bash
   ls /opt/homebrew/share/qemu/edk2-aarch64-code.fd
   ```

3. Check QEMU installation:
   ```bash
   qemu-system-aarch64 --version
   ```

### Desktop app can't connect to daemon

1. Ensure daemon is running:
   ```bash
   curl http://localhost:8871/health
   ```

2. Check CSP settings in `tauri.conf.json` allow `localhost:8871`

3. Check browser console for CORS errors

## License

See repository root LICENSE file.

## Contributing

1. Follow TypeScript `strict` mode
2. Use ESLint + Prettier
3. Write unit tests for parser logic
4. Update OpenAPI schema when adding endpoints
5. Keep components accessible (ARIA, keyboard navigation)
