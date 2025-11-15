# GUI-Kernel Integration Plan

**Date:** November 6, 2025
**Status:** 📋 PLANNING
**Phase:** Post-Phase 6 - Live Kernel Integration

## Executive Summary

This document outlines the implementation plan for connecting the Phase 6 web GUI to a live running kernel instance, transitioning from replay-based demonstration to real-time kernel control and monitoring.

**Current State:** Web GUI successfully demonstrates all features using pre-recorded sample logs via replay system.

**Target State:** Web GUI connects to live SIS kernel running in QEMU, with bidirectional communication for real-time control and monitoring.

**Estimated Effort:** 2-3 weeks (medium complexity, building on existing infrastructure)

---

## Table of Contents

1. [Current Architecture](#current-architecture)
2. [Target Architecture](#target-architecture)
3. [Gap Analysis](#gap-analysis)
4. [Implementation Phases](#implementation-phases)
5. [Technical Specifications](#technical-specifications)
6. [File Modifications](#file-modifications)
7. [Testing Strategy](#testing-strategy)
8. [Success Criteria](#success-criteria)
9. [Risks and Mitigations](#risks-and-mitigations)
10. [Rollout Plan](#rollout-plan)

---

## Current Architecture

### How It Works Today (Replay Mode)

```
┌─────────────┐                  ┌──────────────┐                ┌────────────┐
│  Web GUI    │   HTTP/WS        │ sisctl daemon│   File I/O     │  Sample    │
│ (Browser)   │ ←──────────────→ │   (Rust)     │ ←────────────→ │  Logs      │
│             │  localhost:8871  │              │                │  (Static)  │
└─────────────┘                  └──────────────┘                └────────────┘
                                        ↓
                                  ReplayManager
                                  • Reads .log files
                                  • Parses lines
                                  • Emits QemuEvents
                                  • Simulates timing
```

**Key Components:**
- **ReplayManager** (`crates/daemon/src/qemu/replay.rs`)
  - Reads sample log files from `samples/` directory
  - Parses lines using LineParser
  - Emits events via broadcast channel
  - Simulates realistic timing (100ms, 10ms, or instant)
  - No actual kernel interaction

- **QemuSupervisor** (`crates/daemon/src/qemu/supervisor.rs`)
  - Event broadcasting infrastructure (already in place)
  - Metrics batching (100ms intervals, 1000 points/batch)
  - WebSocket streaming to GUI
  - Shell command execution framework (partially implemented)

- **LineParser** (`crates/daemon/src/parser.rs`)
  - Parses kernel output into structured events
  - Handles metrics, autonomy decisions, LLM output, graphs, etc.
  - Already works with both replay and live modes

**Limitations:**
1. ❌ No live kernel interaction
2. ❌ Shell commands are simulated (not sent to real kernel)
3. ❌ Cannot test autonomous approval workflows
4. ❌ Metrics are historical, not real-time
5. ❌ No bidirectional communication

---

## Target Architecture

### How It Will Work (Live Mode)

```
┌─────────────┐                  ┌──────────────┐                ┌────────────┐
│  Web GUI    │   HTTP/WS        │ sisctl daemon│   Stdio/PTY    │ SIS Kernel │
│ (Browser)   │ ←──────────────→ │   (Rust)     │ ←────────────→ │  (QEMU)    │
│             │  localhost:8871  │              │  bidirectional │            │
└─────────────┘                  └──────────────┘                └────────────┘
        │                               │                               │
        │  1. User clicks "Start"       │                               │
        │───────────────────────────────→                               │
        │                               │  2. Spawn QEMU process        │
        │                               │──────────────────────────────→│
        │                               │                               │
        │                               │  3. Read serial output (stdout)
        │                               │←──────────────────────────────│
        │                               │  4. Parse & broadcast events  │
        │  5. Receive real-time events  │                               │
        │←──────────────────────────────│                               │
        │                               │                               │
        │  6. Execute shell command     │                               │
        │───────────────────────────────→                               │
        │                               │  7. Write to stdin            │
        │                               │──────────────────────────────→│
        │                               │  8. Kernel processes command  │
        │                               │  9. Output response           │
        │                               │←──────────────────────────────│
        │  10. Stream response to GUI   │                               │
        │←──────────────────────────────│                               │
```

**New Components:**
- **LiveTransport** (new module)
  - Spawns QEMU process with stdio/PTY
  - Reads stdout line-by-line
  - Writes stdin for shell commands
  - Handles process lifecycle

- **Enhanced QemuSupervisor**
  - Manages both replay and live modes
  - Routes shell commands to appropriate transport
  - Unified event streaming regardless of source

- **Bidirectional Shell Executor**
  - Queues commands to send to kernel
  - Tracks command-response correlation
  - Timeout handling (5s default)
  - Concurrent command support (with queueing)

**Capabilities Unlocked:**
1. ✅ Real-time kernel metrics
2. ✅ Live autonomous decision monitoring
3. ✅ Interactive shell commands from GUI
4. ✅ Autonomous approval workflows (real votes)
5. ✅ LLM inference triggered from GUI
6. ✅ Memory/scheduling operations from GUI
7. ✅ Dual control (shell + GUI simultaneously)

---

## Gap Analysis

### What Exists Today

✅ **Event Broadcasting Infrastructure**
- `QemuEvent` enum with all event types
- `broadcast::Sender` for event distribution
- WebSocket handlers in `api/ws.rs`
- Metrics batching and streaming

✅ **Parsing Infrastructure**
- `LineParser` handles all kernel output formats
- Structured event extraction (metrics, autonomy, LLM, graphs, etc.)
- Works with any line source (replay or live)

✅ **Shell Command Framework**
- `ShellCommandRequest`/`ShellCommandResponse` types
- `ShellExecutor` skeleton (basic structure exists)
- API handlers in `api/shell_handlers.rs`

✅ **QEMU Configuration**
- `QemuConfig` struct with paths and features
- Environment variable support (`SIS_RUN_SCRIPT`)
- Feature flag mapping

✅ **Web GUI**
- All dashboard panels implemented
- Real-time updates via WebSocket
- Shell command input components
- Ready for live data

### What Needs to Be Built

❌ **QEMU Process Management**
- Spawn QEMU with appropriate arguments
- Capture stdout/stderr
- Provide stdin for command injection
- Handle process lifecycle (start/stop/crash detection)
- PTY vs stdio decision

❌ **Bidirectional Communication**
- Read kernel output line-by-line (non-blocking)
- Write shell commands to kernel stdin
- Command-response correlation
- Timeout and error handling

❌ **Live Transport Implementation**
- New `LiveTransport` module
- Integration with existing supervisor
- Graceful fallback to replay mode
- Process monitoring and auto-restart

❌ **Command Queueing**
- Handle concurrent command requests
- Fair queueing (FIFO)
- Command timeout tracking
- Cancellation support

❌ **Mode Switching**
- API to switch between replay and live
- State preservation during switch
- UI indication of current mode

❌ **Integration Testing**
- Test with actual kernel running
- Verify all GUI panels work live
- Load testing (command throughput, event rate)
- Failure scenarios (QEMU crash, timeout)

---

## Implementation Phases

### Phase 1: QEMU Process Management (Week 1, Days 1-3)

**Goal:** Spawn QEMU and capture serial output

**Tasks:**
1. Create `LiveTransport` module (`crates/daemon/src/qemu/live.rs`)
2. Implement QEMU process spawning
   - Use `SIS_RUN_SCRIPT` environment variable (points to `uefi_run.sh`)
   - Build command: `bash $SIS_RUN_SCRIPT <build|run>`
   - Capture stdout/stderr as lines
   - Store process handle for lifecycle management
3. Implement output reading
   - Async line-by-line reading with `tokio::io::BufReader`
   - Non-blocking to prevent deadlocks
   - Buffer management (max 50k lines)
4. Implement process monitoring
   - Detect when QEMU exits
   - Emit `QemuExited` event
   - Cleanup resources
5. Unit tests
   - Test process spawning
   - Test output capture
   - Test process exit detection

**Files Created:**
- `crates/daemon/src/qemu/live.rs` (~300 lines)

**Files Modified:**
- `crates/daemon/src/qemu/mod.rs` (add `pub mod live;`)

**Success Criteria:**
- ✅ QEMU process spawns successfully
- ✅ Stdout is captured line-by-line
- ✅ Process exit is detected
- ✅ No resource leaks (file descriptors, memory)

---

### Phase 2: Bidirectional Communication (Week 1, Days 4-5)

**Goal:** Send commands to kernel and receive responses

**Tasks:**
1. Implement stdin writing in `LiveTransport`
   - Send commands to QEMU stdin
   - Add newline termination
   - Non-blocking writes
2. Implement command-response correlation
   - Track pending commands (HashMap<id, command>)
   - Match responses to commands via timing/heuristics
   - Timeout after 5 seconds
3. Enhance `ShellExecutor` (`crates/daemon/src/qemu/shell_executor.rs`)
   - Route commands to `LiveTransport` if live mode
   - Handle concurrent commands (queue internally)
   - Return `ShellCommandResponse` with output/error
4. Integration with supervisor
   - `QemuSupervisor::execute_command()` calls `ShellExecutor`
   - Already used by API handlers
5. Unit and integration tests
   - Test command sending
   - Test response parsing
   - Test timeout handling
   - Test concurrent commands

**Files Modified:**
- `crates/daemon/src/qemu/live.rs` (+150 lines)
- `crates/daemon/src/qemu/shell_executor.rs` (rewrite, ~400 lines)
- `crates/daemon/src/qemu/supervisor.rs` (minor updates)

**Success Criteria:**
- ✅ Commands sent to kernel successfully
- ✅ Responses captured and returned
- ✅ Timeouts handled gracefully
- ✅ Concurrent commands work correctly

---

### Phase 3: Supervisor Integration (Week 2, Days 1-2)

**Goal:** Unified supervisor supports both replay and live modes

**Tasks:**
1. Add mode enum to `QemuSupervisor`
   ```rust
   pub enum QemuMode {
       Replay { source: String, speed: ReplaySpeed },
       Live { config: QemuConfig },
   }
   ```
2. Implement mode switching
   - `supervisor.start_live(config)` - spawn QEMU
   - `supervisor.start_replay(source, speed)` - load log file
   - `supervisor.stop()` - cleanup current mode
   - Preserve event subscribers across switches
3. Update API handlers
   - `POST /qemu/start` - detect mode from request
   - `GET /qemu/status` - include mode in response
   - New: `POST /qemu/switch-mode` - switch between modes
4. Update metrics store
   - Real-time metrics from live kernel
   - Historical metrics preserved
   - Downsampling for long-running sessions
5. Integration tests
   - Test mode switching
   - Test event streaming in both modes
   - Test command execution in both modes

**Files Modified:**
- `crates/daemon/src/qemu/supervisor.rs` (+200 lines)
- `crates/daemon/src/api/handlers.rs` (+50 lines)
- `crates/daemon/src/qemu/types.rs` (add `QemuMode`)

**Success Criteria:**
- ✅ Supervisor supports both modes
- ✅ Mode switching works without restarts
- ✅ Events stream correctly in both modes
- ✅ API reflects current mode

---

### Phase 4: GUI Updates (Week 2, Days 3-4)

**Goal:** GUI indicates mode and handles live interactions

**Tasks:**
1. Update `QemuStatus` to include mode
   ```typescript
   interface QemuStatus {
     state: "idle" | "running" | "error";
     mode?: "replay" | "live";
     source?: string;  // replay only
     pid?: number;      // live only
     uptime_sec?: number;
   }
   ```
2. Add mode indicator to Dashboard
   - Badge showing "REPLAY" or "LIVE"
   - Different colors (replay: blue, live: green)
3. Update shell command input
   - Disable when QEMU not running
   - Show "connecting..." during command execution
   - Display command history
4. Add mode switcher control (optional)
   - Toggle between replay/live
   - Confirm before switching
5. Update E2E tests
   - Test with live mode (requires QEMU running)
   - Mock live mode for CI
   - Add `@live` tag for tests requiring kernel

**Files Modified:**
- `gui/desktop/src/lib/api.ts` (update types)
- `gui/desktop/src/components/Dashboard.tsx` (+30 lines)
- `gui/desktop/src/components/ShellCommandInput.tsx` (+20 lines)
- `gui/desktop/e2e/*.spec.ts` (add live mode tests)

**Success Criteria:**
- ✅ GUI shows current mode
- ✅ Shell commands work in live mode
- ✅ UI feedback for command execution
- ✅ E2E tests pass with live mode

---

### Phase 5: Integration Testing & Validation (Week 2-3, Days 5-7)

**Goal:** Comprehensive testing with live kernel

**Test Scenarios:**

1. **Basic Kernel Interaction**
   - Start QEMU from GUI
   - Execute simple commands (`help`, `metrics`)
   - Verify output displayed in GUI
   - Stop QEMU from GUI

2. **Real-Time Metrics**
   - Start kernel with metrics enabled
   - Verify metrics update in real-time
   - Check downsampling after 10k points
   - Verify no memory leaks

3. **Autonomous Decisions**
   - Enable autonomy: `autoctl on`
   - Monitor decisions in real-time
   - Test approval workflow: `memctl approval on`
   - Approve/reject from GUI
   - Verify actions execute in kernel

4. **LLM Integration**
   - Load LLM model: `llmctl load`
   - Execute inference from GUI
   - Stream results to GUI
   - Verify token-by-token display

5. **Graph Visualization**
   - Start graph operations: `graphdemo`
   - Verify graph updates in real-time
   - Check operator statistics
   - Verify channel stalls displayed

6. **Concurrent Operations**
   - Execute 10 commands in rapid succession
   - Verify all complete successfully
   - Check command queueing works
   - Verify no race conditions

7. **Failure Scenarios**
   - Kill QEMU process externally
   - Verify daemon detects exit
   - Check GUI shows error state
   - Test recovery (restart QEMU)

8. **Long-Running Sessions**
   - Run kernel for 1 hour
   - Execute 100+ commands
   - Verify memory usage stable
   - Check WebSocket stays connected

**Performance Targets:**
- Command latency: <500ms (GUI → kernel → GUI)
- Event streaming: 100+ events/sec without drops
- Memory usage: <500MB daemon (1hr session)
- QEMU startup: <5 seconds

**Files Created:**
- `crates/daemon/tests/integration_live.rs` (~500 lines)
- `gui/desktop/e2e/live-mode.spec.ts` (~300 lines)

**Success Criteria:**
- ✅ All test scenarios pass
- ✅ Performance targets met
- ✅ No crashes or hangs
- ✅ Resource usage acceptable

---

## Technical Specifications

### QEMU Process Spawning

```rust
// Spawn QEMU using uefi_run.sh script
pub async fn spawn_qemu(config: &QemuConfig) -> Result<LiveProcess> {
    let script = std::env::var("SIS_RUN_SCRIPT")
        .context("SIS_RUN_SCRIPT not set")?;

    let mut cmd = Command::new("bash");
    cmd.arg(&script)
       .arg("run")  // Use 'run' command (builds + runs)
       .env("SIS_FEATURES", &config.features)
       .env("BRINGUP", "1")  // Enable bringup mode
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .kill_on_drop(true);

    let mut child = cmd.spawn()
        .context("Failed to spawn QEMU")?;

    let stdin = child.stdin.take()
        .context("Failed to get stdin")?;
    let stdout = child.stdout.take()
        .context("Failed to get stdout")?;
    let stderr = child.stderr.take()
        .context("Failed to get stderr")?;

    Ok(LiveProcess {
        child,
        stdin: Arc::new(Mutex::new(stdin)),
        stdout: BufReader::new(stdout),
        stderr: BufReader::new(stderr),
    })
}
```

### Command-Response Correlation

```rust
// Track pending commands with correlation IDs
pub struct CommandTracker {
    pending: Arc<RwLock<HashMap<Uuid, PendingCommand>>>,
}

struct PendingCommand {
    command: String,
    sent_at: Instant,
    tx: oneshot::Sender<ShellCommandResponse>,
}

impl CommandTracker {
    pub async fn send_command(&self, cmd: String, stdin: &mut ChildStdin)
        -> Result<oneshot::Receiver<ShellCommandResponse>>
    {
        let id = Uuid::new_v4();
        let (tx, rx) = oneshot::channel();

        // Store pending command
        self.pending.write().await.insert(id, PendingCommand {
            command: cmd.clone(),
            sent_at: Instant::now(),
            tx,
        });

        // Write to stdin (newline-terminated)
        stdin.write_all(cmd.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        Ok(rx)
    }

    pub async fn handle_output(&self, line: &str) {
        // Heuristic: match output to most recent pending command
        // In practice, kernel should echo command or use IDs
        if let Some((id, pending)) = self.pending.write().await.iter_mut().next() {
            // Send response back to caller
            let _ = pending.tx.send(ShellCommandResponse {
                success: true,
                output: vec![line.to_string()],
                error: None,
                execution_time_ms: pending.sent_at.elapsed().as_millis() as u64,
            });
            self.pending.write().await.remove(&id);
        }
    }
}
```

### Event Streaming (Unchanged)

```rust
// Existing event streaming works with live mode
// LineParser extracts events from any source
pub async fn process_output_line(
    line: String,
    parser: &LineParser,
    event_tx: &broadcast::Sender<QemuEvent>,
) {
    if let Some(event) = parser.parse(&line) {
        let _ = event_tx.send(QemuEvent::Parsed { event });
    }

    // Also emit raw line for debugging
    let _ = event_tx.send(QemuEvent::RawLine {
        line,
        timestamp: chrono::Utc::now(),
    });
}
```

### Mode Switching

```rust
impl QemuSupervisor {
    pub async fn start_live(&self, config: QemuConfig) -> Result<()> {
        // Stop current mode if any
        self.stop().await?;

        // Spawn QEMU process
        let process = spawn_qemu(&config).await?;

        // Start output processor
        let event_tx = self.event_tx.clone();
        let parser = LineParser::new();

        tokio::spawn(async move {
            let mut lines = process.stdout.lines();
            while let Some(line) = lines.next_line().await? {
                process_output_line(line, &parser, &event_tx).await;
            }
            Ok::<_, anyhow::Error>(())
        });

        // Update state
        *self.mode.write().await = QemuMode::Live { config };
        *self.state.write().await = QemuState::Running;

        Ok(())
    }

    pub async fn start_replay(&self, source: String, speed: ReplaySpeed) -> Result<()> {
        // Stop current mode
        self.stop().await?;

        // Start replay manager
        self.replay.start(source.clone(), speed, self.event_tx.clone()).await?;

        // Update state
        *self.mode.write().await = QemuMode::Replay { source, speed };
        *self.state.write().await = QemuState::Running;

        Ok(())
    }
}
```

---

## File Modifications

### New Files

1. **`crates/daemon/src/qemu/live.rs`** (~450 lines)
   - `LiveProcess` struct (process handle, stdin/stdout/stderr)
   - `spawn_qemu()` function
   - `CommandTracker` for command-response correlation
   - `process_output_lines()` async stream handler
   - Unit tests

2. **`crates/daemon/tests/integration_live.rs`** (~500 lines)
   - Integration tests for live mode
   - Requires QEMU available

3. **`gui/desktop/e2e/live-mode.spec.ts`** (~300 lines)
   - E2E tests for live kernel interaction
   - Tagged with `@live` for conditional execution

### Modified Files

1. **`crates/daemon/src/qemu/supervisor.rs`** (+200 lines, ~1000 total)
   - Add `QemuMode` enum
   - Add `start_live()` method
   - Update `start_replay()` to use mode enum
   - Add mode switching logic

2. **`crates/daemon/src/qemu/shell_executor.rs`** (rewrite, ~400 lines)
   - Implement actual command execution (not stub)
   - Route to live transport when in live mode
   - Command queueing and timeout handling

3. **`crates/daemon/src/qemu/mod.rs`** (+2 lines)
   - Add `pub mod live;`
   - Re-export `LiveProcess`

4. **`crates/daemon/src/qemu/types.rs`** (+30 lines)
   - Add `QemuMode` enum
   - Update `QemuStatus` to include mode and pid

5. **`crates/daemon/src/api/handlers.rs`** (+50 lines)
   - Update `POST /qemu/start` to detect mode from request
   - Add mode field to responses

6. **`gui/desktop/src/lib/api.ts`** (+20 lines)
   - Update `QemuStatus` type
   - Add mode field

7. **`gui/desktop/src/components/Dashboard.tsx`** (+30 lines)
   - Add mode indicator badge
   - Update styling for live vs replay

8. **`gui/desktop/src/components/ShellCommandInput.tsx`** (+20 lines)
   - Disable when not running
   - Show execution feedback

### Unchanged Files (Work as-is)

✅ `crates/daemon/src/parser.rs` - Already supports any line source
✅ `crates/daemon/src/api/ws.rs` - Event streaming unchanged
✅ `crates/daemon/src/metrics/` - Works with live or replay
✅ `gui/desktop/src/components/*` - Most components unchanged
✅ All other GUI components

---

## Testing Strategy

### Unit Tests

**Location:** `crates/daemon/src/qemu/live.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_qemu() {
        std::env::set_var("SIS_RUN_SCRIPT", "/path/to/uefi_run.sh");
        let config = QemuConfig::default();
        let process = spawn_qemu(&config).await.unwrap();
        assert!(process.child.id().is_some());
    }

    #[tokio::test]
    async fn test_command_tracker() {
        let tracker = CommandTracker::new();
        // Test send/receive cycle
    }

    #[tokio::test]
    async fn test_command_timeout() {
        // Verify commands timeout after 5 seconds
    }
}
```

### Integration Tests

**Location:** `crates/daemon/tests/integration_live.rs`

```rust
#[tokio::test]
#[ignore = "requires QEMU"]
async fn test_live_kernel_commands() {
    let supervisor = QemuSupervisor::new().await;
    supervisor.start_live(QemuConfig::default()).await.unwrap();

    // Wait for boot
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Execute command
    let resp = supervisor.execute_command(ShellCommandRequest {
        command: "help".to_string(),
        timeout_ms: 5000,
    }).await.unwrap();

    assert!(resp.success);
    assert!(!resp.output.is_empty());
}
```

### E2E Tests

**Location:** `gui/desktop/e2e/live-mode.spec.ts`

```typescript
import { test, expect } from '@playwright/test';

test.describe('Live Mode @live', () => {
  test('should start QEMU and execute commands', async ({ page }) => {
    await page.goto('http://localhost:1420');

    // Start QEMU
    await page.click('[data-testid="start-qemu"]');
    await expect(page.locator('[data-testid="qemu-status"]'))
      .toHaveText('Running', { timeout: 15000 });

    // Wait for kernel boot
    await page.waitForTimeout(10000);

    // Execute command
    await page.fill('[data-testid="shell-input"]', 'help');
    await page.press('[data-testid="shell-input"]', 'Enter');

    // Verify output
    await expect(page.locator('[data-testid="shell-output"]'))
      .toContainText('Available commands');
  });
});
```

### Performance Tests

```rust
#[tokio::test]
#[ignore = "long running"]
async fn test_live_session_1hour() {
    let supervisor = QemuSupervisor::new().await;
    supervisor.start_live(QemuConfig::default()).await.unwrap();

    let start_mem = get_process_memory();

    // Run for 1 hour, executing commands periodically
    for _ in 0..360 {
        supervisor.execute_command(/* ... */).await.unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    let end_mem = get_process_memory();
    assert!(end_mem < start_mem + 100_000_000); // <100MB growth
}
```

---

## Success Criteria

### Functional Requirements

- ✅ **QEMU Process Management**
  - Daemon can spawn QEMU using `uefi_run.sh`
  - Process stdout/stderr captured
  - Process exit detected and handled
  - Resources cleaned up on stop

- ✅ **Bidirectional Communication**
  - Commands sent to kernel stdin
  - Responses captured from stdout
  - Command-response correlation works
  - Timeout handling (5s default)

- ✅ **Real-Time Event Streaming**
  - Kernel output parsed into events
  - Events broadcast to WebSocket clients
  - Metrics batched (100ms intervals)
  - No event drops under normal load

- ✅ **Mode Switching**
  - Supervisor supports replay and live modes
  - API reflects current mode
  - Mode can be switched without restart
  - State preserved across switches

- ✅ **GUI Integration**
  - GUI shows current mode
  - Shell commands work from GUI
  - Real-time metrics displayed
  - Autonomous decisions monitored

### Non-Functional Requirements

- ✅ **Performance**
  - Command latency: <500ms (p95)
  - Event throughput: 100+ events/sec
  - Memory usage: <500MB (1hr session)
  - QEMU startup: <5 seconds

- ✅ **Reliability**
  - No crashes during 1-hour sessions
  - Graceful handling of QEMU crashes
  - WebSocket reconnection works
  - No resource leaks (FDs, memory)

- ✅ **Maintainability**
  - Code well-documented
  - Unit test coverage >80%
  - Integration tests for all scenarios
  - Clear error messages

---

## Risks and Mitigations

### Risk 1: Command-Response Correlation Ambiguity

**Risk:** Kernel doesn't echo commands, hard to match responses to requests

**Likelihood:** High
**Impact:** High
**Mitigation:**
1. Add command echoing to kernel shell (best solution)
2. Use timing heuristics (sent_at → first output)
3. Add unique IDs to shell prompt
4. Queue commands (only one in-flight at a time)

**Status:** Mitigated by queueing + timing

### Risk 2: QEMU Startup Failures

**Risk:** QEMU fails to start (missing deps, bad config, etc.)

**Likelihood:** Medium
**Impact:** High
**Mitigation:**
1. Validate `SIS_RUN_SCRIPT` before spawning
2. Check QEMU binary exists
3. Capture stderr for error messages
4. Timeout after 10 seconds if no output
5. Provide clear error messages to GUI

**Status:** Mitigated by validation + timeouts

### Risk 3: Performance Degradation Under Load

**Risk:** High event rate causes drops or latency

**Likelihood:** Medium
**Impact:** Medium
**Mitigation:**
1. Backpressure: limit buffer to 50k lines
2. Metrics batching (already implemented)
3. Event subscriber limit (100 max)
4. Downsampling after 10k metric points
5. Performance tests in CI

**Status:** Mitigated by existing infrastructure

### Risk 4: WebSocket Disconnections

**Risk:** Long-running sessions cause WebSocket drops

**Likelihood:** Low
**Impact:** Medium
**Mitigation:**
1. WebSocket keepalive (ping every 30s)
2. Automatic reconnection in GUI
3. Event replay after reconnect
4. Graceful degradation (polling fallback)

**Status:** Mitigated by keepalive + reconnect

### Risk 5: Concurrent Command Race Conditions

**Risk:** Multiple clients send commands simultaneously

**Likelihood:** Medium
**Impact:** Medium
**Mitigation:**
1. Internal command queue (FIFO)
2. Mutex around stdin writes
3. Command IDs for tracking
4. Timeout per command (5s)
5. Clear "busy" state in GUI

**Status:** Mitigated by queueing

---

## Rollout Plan

### Phase 1: Internal Testing (Week 3, Day 1-2)

**Goal:** Validate with developers

1. Deploy to dev environment
2. Test all scenarios manually
3. Run performance tests
4. Collect feedback
5. Fix critical bugs

**Success:** No blockers, performance acceptable

### Phase 2: Beta Testing (Week 3, Day 3-4)

**Goal:** Broader testing with real workflows

1. Enable live mode in GUI (beta flag)
2. Test with actual use cases
3. Monitor logs for errors
4. Collect user feedback
5. Fix non-critical bugs

**Success:** Users can complete workflows, no data loss

### Phase 3: Production Release (Week 3, Day 5)

**Goal:** Full rollout

1. Update README with live mode docs
2. Add section to Quick Start
3. Record demo video (live mode)
4. Announce in changelog
5. Monitor for issues

**Success:** Documentation complete, users can self-serve

### Post-Release

1. **Week 4:** Monitor error rates, collect feedback
2. **Week 5:** Performance tuning based on real usage
3. **Week 6:** Add advanced features (command history, auto-complete)

---

## Appendix

### Environment Variables

- `SIS_RUN_SCRIPT` - Path to `uefi_run.sh` script (required)
- `SIS_FEATURES` - Kernel features to enable (e.g., "llm,deterministic")
- `BRINGUP` - Set to "1" for bringup mode

### API Changes

**New Endpoint:**
```
POST /qemu/switch-mode
{
  "mode": "live" | "replay",
  "replay_source": "boot_minimal.log"  // if replay
}
```

**Updated Response:**
```json
{
  "state": "running",
  "mode": "live",
  "pid": 12345,
  "uptime_sec": 120
}
```

### Related Documents

- `docs/DIRECTORY_RESTRUCTURE.md` - Directory layout
- `docs/INTEGRATION_TEST_REPORT.md` - Testing strategy
- `docs/E2E_TEST_ISSUES_SUMMARY.md` - E2E test results
- `gui/desktop/README.md` - Web GUI documentation (if exists)

---

**Next Steps:** Review this plan, approve, and begin Phase 1 implementation.
