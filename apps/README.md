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

### Milestone 1-3 Deliverables ✅

- [x] Shell command execution API (`POST /shell/exec`)
- [x] Self-check automation (`POST /shell/selfcheck`)
- [x] Metrics ingestion with downsampling (LTTB algorithm)
- [x] Autonomy control panel (`autoctl` wrapper)
- [x] Memory approvals UI (`memctl` wrapper)
- [x] What-if scenario simulator
- [x] Replay controls for offline testing
- [x] Live log tailing with run history

### Milestone 4 (M4) Deliverables ✅

**Graph/Scheduling/LLM/Logs - Production Systems**

- [x] **GraphPanel** - Graph computation visualization
  - Create/manage computational graphs via API
  - Add channels and operators dynamically
  - Live graph state updates via WebSocket
  - Prediction/feedback loop UI
  - Export graph states (JSON)

- [x] **SchedPanel** - Workload scheduling control
  - View active workloads with priorities and affinity
  - Set priorities (0-139) for workloads
  - Configure CPU affinity masks
  - Toggle scheduler features (GENTLE_FAIR_SLEEPERS, etc.)
  - Circuit breaker monitoring and reset

- [x] **LlmPanel** - LLM model management
  - Load models with path and format selection
  - Inference requests with prompt input
  - Token streaming via WebSocket
  - Audit trail with request history
  - Model status and configuration display

- [x] **LogsPanel** - Advanced log management
  - Live log tailing with level/source filtering
  - Run history tracking with profiles
  - Start/stop runs with feature configurations
  - Export run logs for troubleshooting
  - WebSocket backpressure detection with droppedCount badges

### Milestone 5 (M5) Deliverables ✅

**Crash Capture & Incident Management**

- [x] **CrashPanel** - Crash capture and incident workflow
  - Live crash feed from WebSocket
  - Crash detail modal with stack traces and registers
  - Severity filtering (critical/high/medium/low)
  - Incident creation from crashes (title + description)
  - Crash/incident correlation and tracking
  - Auto-deduplication by crashId

- [x] **Crash API** - Backend crash ingestion
  - `POST /api/v1/crash` - Ingest crash reports
  - `GET /api/v1/crashes` - List with pagination and filters
  - `POST /api/v1/incidents` - Create incidents
  - `GET /api/v1/incidents` - List incidents
  - WebSocket streaming of crash events

### UX Polish (Option A) ✅

- [x] **Copy-to-clipboard** for JSON exports (GraphPanel, LogsPanel)
- [x] **Problem+json CTA hints** with actionable error banners
- [x] **droppedCount badges** with auto-reset for backpressure visibility
- [x] **QEMU profile save/load** with localStorage persistence

### Dev Tools (Option B) ✅

- [x] **X-Request-Id tracer** for API debugging
  - Middleware generates/accepts X-Request-Id (UUIDv4)
  - Request ID in error responses (Problem+json format)
  - Request ID in log entries (WebSocket + REST)
  - Frontend captures and displays request IDs in error banners
  - Axios interceptor for automatic request ID extraction

### Milestone 3 (M3) Deliverables ✅

**API Explorer - Enhanced Testing & Development**

- [x] **API Explorer Panel** - Comprehensive API testing interface
  - Interactive endpoint browser with 50+ REST endpoints
  - Endpoint search and filtering by name, tag, or method
  - Grouping by API tags (qemu, replay, metrics, graph, etc.)
  - Custom request builder with method/URL/headers/body
  - Request history with localStorage persistence (max 100 entries)
  - Response inspection with JSON formatting
  - One-click copy to clipboard for responses
  - Request timing and status display
  - Integration with Swagger UI (`/swagger-ui`)
  - Support for GET, POST, PUT, PATCH, DELETE methods
  - Load historical requests for replay

### Milestone 4 (M4) Deliverables ✅

**Log Viewer Enhancements & Boot Timeline**

- [x] **Boot Timeline View** - Visual boot performance analysis
  - Horizontal timeline visualization with event markers
  - Real-time boot event tracking (9 markers)
  - Performance metrics (total duration, avg/min/max deltas)
  - Interactive timeline scrubbing and event selection
  - Event sequence list with timing details
  - Performance insights (detects slow boot steps)
  - Color-coded performance indicators
  - Formatted duration display (ms/s/m)

- [x] **Enhanced LogsPanel** - Production-grade log viewer (existing)
  - Virtualized scrolling with @tanstack/react-virtual
  - Ring buffer (10,000 log limit) for memory efficiency
  - Multi-level filtering (level, source, search text)
  - Real-time log tailing via WebSocket
  - Run history with start/stop/export
  - JSON and CSV export capabilities
  - Backpressure detection with visual indicators
  - Self-check integration with pass/fail banners
  - Color-coded log levels
  - Timestamp display

### Milestone 5 (M5) Deliverables ✅

**Metrics Enhancements - Storage, Alerts, and Analysis**

- [x] **IndexedDB Persistent Storage** - Client-side metrics persistence
  - IndexedDB-based storage utility (metricsDB.ts)
  - Automatic metrics persistence across sessions
  - 24-hour data retention with auto-cleanup
  - Efficient querying by series name
  - Stale data detection and removal
  - Clear all / delete series operations
  - Size-aware storage management

- [x] **Performance Alert System** - Threshold-based monitoring
  - Configurable threshold rules with operators (>, <, =)
  - Multi-level severity (critical, warning, info)
  - Default rules for common metrics (IRQ latency, memory, CPU)
  - Real-time alert checking on incoming metrics
  - Alert history with localStorage persistence (max 100)
  - Toast notifications for critical alerts
  - Enable/disable rules individually
  - Custom rule creation and management

- [x] **Metrics Alerts Panel** - Alert visualization and management
  - Split view: alert rules and alert history
  - Visual severity indicators (critical/warning/info)
  - Rule configuration (metric, threshold, operator, severity)
  - Alert history with timestamps and values
  - Enable/disable/delete rules
  - Clear alert history
  - Real-time alert feed
  - Formatted value display (K/M suffixes)

- [x] **Enhanced Metrics Panel** - Time-series controls (existing)
  - Time range selection (5m, 30m, 1h) for zoom capability
  - Pause/resume live updates
  - Sparkline visualizations
  - CSV/JSON export
  - Backpressure detection

### Milestone 6 (M6) Deliverables ✅

**Settings & Customization**

- [x] **Settings Panel** - Application configuration interface
  - Multi-tab layout (Appearance, Shortcuts, Daemon)
  - Theme selection (light/dark/system) with live preview
  - System theme detection and auto-switching
  - Keyboard shortcuts reference display
  - Daemon connection configuration (REST + WebSocket URLs)
  - Auto-reconnect settings with interval control
  - Settings persistence with localStorage
  - Save/reset functionality with change detection
  - Warning messages for critical changes
  - Responsive two-column layout

### Milestone 7 (M7) Deliverables ✅

**Polish & UX Enhancements**

- [x] **Loading States** - Comprehensive loading indicators
  - LoadingState component with size variants (sm/md/lg)
  - Full-screen loading overlay option
  - LoadingSkeleton for content placeholders
  - LoadingSpinner for inline use
  - ARIA live regions for screen reader announcements
  - Smooth animations with Tailwind

- [x] **Empty States** - User-friendly empty data displays
  - EmptyState component with optional icon
  - Title, description, and action button support
  - Centered layout with proper spacing
  - ARIA role="status" for accessibility
  - Reusable across all panels

- [x] **Accessibility Utilities** - ARIA and keyboard navigation
  - Metric value ARIA label generator
  - Status badge ARIA label helper
  - Keyboard navigation handler (Enter/Space/Escape/Arrows)
  - Focus trap for modals
  - Screen reader announcement utility
  - Keyboard-friendly components throughout

- [x] **Existing UX Features** - Already implemented
  - Smooth transitions on all interactive elements
  - Hover states for buttons and links
  - Color-coded status indicators
  - Toast notifications for user feedback
  - Error boundaries with graceful fallbacks
  - Responsive design for all screen sizes

## Next Steps (Milestone 8+)

- [x] Replay authoring guide documentation (see docs/guides/REPLAY-AUTHORING-GUIDE.md)
- [ ] i18n support
- [ ] Playwright E2E tests
- [ ] CI/CD packaging (macOS/Linux/Windows)
- [ ] Hardware deployment workflows

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
