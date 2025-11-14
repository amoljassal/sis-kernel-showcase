# SIS Kernel: Stub-to-Real Implementation Plan
**Version**: 1.0
**Date**: November 14, 2024
**Purpose**: Convert critical stub implementations to real functionality
**Target Branch**: To be created from `main`

## Executive Summary

This document provides a comprehensive plan to convert 10 priority stub implementations into real, functional code. Each implementation is ordered by impact on test pass rates and system functionality. The implementations should be done sequentially, with each building on the previous work.

---

## 1. AgentSys FS → VFS Integration

### Priority: HIGHEST (P0)
### Impact: Makes Phase 9 AgentSys functional with real file operations
### Files to Modify:
- `crates/kernel/src/agent_sys/handlers/fs.rs`
- `crates/kernel/src/agent_sys/handlers/mod.rs`

### Current State:
```rust
// Current stub in fs.rs
pub fn handle_list(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    // ... policy checks ...
    uart::print_str("[FS] Entries: files/, docs/, test.txt\n");  // STUB!
    Ok(())
}
```

### Required Implementation:

#### 1.1 handle_list (Opcode 0x30)
```rust
pub fn handle_list(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    // 1. Parse path from TLV payload
    let path = parse_path(payload)?;

    // 2. Policy check (keep existing)
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x30, false);
        return Err(CtrlError::PermissionDenied);
    }

    // 3. REAL VFS OPERATION
    match crate::vfs::list_directory(path) {
        Ok(entries) => {
            uart::print_str("[FS] Entries: ");
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 { uart::print_str(", "); }
                uart::print_str(&entry.name);
                if entry.is_dir { uart::print_str("/"); }
            }
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x30, true);
            Ok(())
        }
        Err(errno) => {
            uart::print_str("[FS] Error: ");
            uart::print_str(errno.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x30, false);
            Err(CtrlError::IoError)
        }
    }
}
```

#### 1.2 handle_read (Opcode 0x31)
```rust
pub fn handle_read(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    let path = parse_path(payload)?;

    // Policy check
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x31, false);
        return Err(CtrlError::PermissionDenied);
    }

    // REAL VFS READ
    match crate::vfs::open_file(path, OpenFlags::READ) {
        Ok(fd) => {
            let mut buffer = [0u8; 512];  // Read first 512 bytes
            match crate::vfs::read_file(fd, &mut buffer) {
                Ok(bytes_read) => {
                    uart::print_str("[FS] Read ");
                    uart::print_u32(bytes_read as u32);
                    uart::print_str(" bytes\n");

                    // Print first 64 chars as preview
                    uart::print_str("[FS] Preview: ");
                    for i in 0..core::cmp::min(64, bytes_read) {
                        let c = buffer[i];
                        if c >= 32 && c < 127 {
                            uart::write_byte(c);
                        } else {
                            uart::print_str(".");
                        }
                    }
                    uart::print_str("\n");

                    crate::vfs::close_file(fd);
                    audit().log_operation(agent_id, 0x31, true);
                    Ok(())
                }
                Err(errno) => {
                    crate::vfs::close_file(fd);
                    uart::print_str("[FS] Read error: ");
                    uart::print_str(errno.description());
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x31, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(errno) => {
            uart::print_str("[FS] Open error: ");
            uart::print_str(errno.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x31, false);
            Err(CtrlError::IoError)
        }
    }
}
```

#### 1.3 handle_write (Opcode 0x32)
```rust
pub fn handle_write(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    // Parse path and data from payload
    let (path, data) = parse_path_and_data(payload)?;

    // Policy checks (both path and size)
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x32, false);
        return Err(CtrlError::PermissionDenied);
    }

    // Check file size limit
    if let Some(max_size) = policy().get_agent(agent_id)
        .and_then(|a| a.scope.max_file_size) {
        if data.len() > max_size {
            audit().log_operation(agent_id, 0x32, false);
            return Err(CtrlError::ResourceLimit);
        }
    }

    // REAL VFS WRITE
    match crate::vfs::open_file(path, OpenFlags::WRITE | OpenFlags::CREATE) {
        Ok(fd) => {
            match crate::vfs::write_file(fd, data) {
                Ok(bytes_written) => {
                    uart::print_str("[FS] Wrote ");
                    uart::print_u32(bytes_written as u32);
                    uart::print_str(" bytes to ");
                    uart::print_str(path);
                    uart::print_str("\n");

                    crate::vfs::close_file(fd);
                    audit().log_operation(agent_id, 0x32, true);
                    Ok(())
                }
                Err(errno) => {
                    crate::vfs::close_file(fd);
                    uart::print_str("[FS] Write error: ");
                    uart::print_str(errno.description());
                    uart::print_str("\n");
                    audit().log_operation(agent_id, 0x32, false);
                    Err(CtrlError::IoError)
                }
            }
        }
        Err(errno) => {
            uart::print_str("[FS] Create error: ");
            uart::print_str(errno.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x32, false);
            Err(CtrlError::IoError)
        }
    }
}
```

#### 1.4 handle_stat (Opcode 0x33)
```rust
pub fn handle_stat(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    let path = parse_path(payload)?;

    // Policy check
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x33, false);
        return Err(CtrlError::PermissionDenied);
    }

    // REAL VFS STAT
    match crate::vfs::stat_file(path) {
        Ok(stat) => {
            uart::print_str("[FS] Stat: ");
            uart::print_str(path);
            uart::print_str(" - size=");
            uart::print_u64(stat.size);
            uart::print_str(" mode=");
            uart::print_hex8((stat.mode & 0xFF) as u8);
            uart::print_str(" type=");
            uart::print_str(if stat.is_dir { "dir" } else { "file" });
            uart::print_str("\n");

            audit().log_operation(agent_id, 0x33, true);
            Ok(())
        }
        Err(errno) => {
            uart::print_str("[FS] Stat error: ");
            uart::print_str(errno.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x33, false);
            Err(CtrlError::IoError)
        }
    }
}
```

#### 1.5 handle_create (Opcode 0x34)
```rust
pub fn handle_create(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    let path = parse_path(payload)?;

    // Policy check
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x34, false);
        return Err(CtrlError::PermissionDenied);
    }

    // REAL VFS CREATE
    match crate::vfs::create_file(path, 0o644) {
        Ok(_) => {
            uart::print_str("[FS] Created: ");
            uart::print_str(path);
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x34, true);
            Ok(())
        }
        Err(errno) => {
            uart::print_str("[FS] Create error: ");
            uart::print_str(errno.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x34, false);
            Err(CtrlError::IoError)
        }
    }
}
```

#### 1.6 handle_delete (Opcode 0x35)
```rust
pub fn handle_delete(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    let path = parse_path(payload)?;

    // Policy check
    let decision = policy().check(
        agent_id,
        Capability::FsBasic,
        &Resource::FilePath(path),
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x35, false);
        return Err(CtrlError::PermissionDenied);
    }

    // REAL VFS DELETE
    match crate::vfs::unlink_file(path) {
        Ok(_) => {
            uart::print_str("[FS] Deleted: ");
            uart::print_str(path);
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x35, true);
            Ok(())
        }
        Err(errno) => {
            uart::print_str("[FS] Delete error: ");
            uart::print_str(errno.description());
            uart::print_str("\n");
            audit().log_operation(agent_id, 0x35, false);
            Err(CtrlError::IoError)
        }
    }
}
```

### Helper Functions to Add:
```rust
// Add to fs.rs
fn parse_path(payload: &[u8]) -> Result<&str, CtrlError> {
    if payload.is_empty() {
        return Err(CtrlError::InvalidFormat);
    }

    core::str::from_utf8(payload)
        .map_err(|_| CtrlError::InvalidFormat)
}

fn parse_path_and_data(payload: &[u8]) -> Result<(&str, &[u8]), CtrlError> {
    // Format: [path_len:2][path:N][data:remaining]
    if payload.len() < 2 {
        return Err(CtrlError::InvalidFormat);
    }

    let path_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + path_len {
        return Err(CtrlError::InvalidFormat);
    }

    let path = core::str::from_utf8(&payload[2..2+path_len])
        .map_err(|_| CtrlError::InvalidFormat)?;
    let data = &payload[2+path_len..];

    Ok((path, data))
}
```

### Success Criteria:
- ✅ `agentsys test-fs-list` lists actual VFS directory contents
- ✅ `agentsys test-fs-read` reads and displays real file content
- ✅ `agentsys test-fs-write` creates/modifies real files
- ✅ `agentsys test-fs-stat` shows real file metadata
- ✅ `agentsys test-fs-create` creates real files
- ✅ `agentsys test-fs-delete` removes real files
- ✅ All operations respect scope restrictions (path prefixes)
- ✅ All operations log to audit trail
- ✅ Errno values properly surfaced

---

## 2. OpenTelemetry Exporter Sink

### Priority: HIGH (P1)
### Impact: Makes OTel produce observable artifacts, improves Phase 7
### Files to Modify:
- `crates/kernel/src/otel/exporter.rs`
- `crates/kernel/src/otel/mod.rs`

### Current State:
```rust
// Current stub
pub fn write_file(&mut self, _path: &str, _contents: &[u8]) -> Result<(), &'static str> {
    // TODO: Actually write to VFS
    Ok(())
}
```

### Required Implementation:

#### 2.1 File-based Exporter
```rust
impl OTelExporter {
    const MAX_FILE_SIZE: usize = 64 * 1024;  // 64KB before rotation
    const EXPORT_PATH: &'static str = "/otel/spans.json";
    const BACKUP_PATH: &'static str = "/otel/spans.old.json";

    pub fn write_file(&mut self, path: &str, contents: &[u8]) -> Result<(), &'static str> {
        // Create /otel directory if not exists
        if let Err(_) = crate::vfs::stat_file("/otel") {
            crate::vfs::mkdir("/otel", 0o755)
                .map_err(|_| "Failed to create /otel directory")?;
        }

        // Check if rotation needed
        if let Ok(stat) = crate::vfs::stat_file(path) {
            if stat.size + contents.len() as u64 > Self::MAX_FILE_SIZE as u64 {
                // Rotate: current -> backup
                let _ = crate::vfs::unlink_file(Self::BACKUP_PATH);
                let _ = crate::vfs::rename(Self::EXPORT_PATH, Self::BACKUP_PATH);
            }
        }

        // Append to file
        match crate::vfs::open_file(path, OpenFlags::WRITE | OpenFlags::APPEND | OpenFlags::CREATE) {
            Ok(fd) => {
                let result = crate::vfs::write_file(fd, contents);
                crate::vfs::close_file(fd);
                result.map(|_| ()).map_err(|_| "Write failed")
            }
            Err(_) => Err("Failed to open OTel export file")
        }
    }

    pub fn flush(&mut self) -> Result<(), &'static str> {
        if self.pending_spans.is_empty() {
            return Ok(());
        }

        // Convert spans to JSON
        let mut json = String::new();
        json.push_str("[\n");

        for (i, span) in self.pending_spans.iter().enumerate() {
            if i > 0 { json.push_str(",\n"); }
            json.push_str("  {");
            json.push_str("\"trace_id\":\"");
            for b in &span.trace_id {
                write!(&mut json, "{:02x}", b).ok();
            }
            json.push_str("\",");
            json.push_str("\"span_id\":\"");
            for b in &span.span_id {
                write!(&mut json, "{:02x}", b).ok();
            }
            json.push_str("\",");
            json.push_str("\"name\":\"");
            json.push_str(&span.name);
            json.push_str("\",");
            json.push_str("\"start_time\":");
            write!(&mut json, "{}", span.start_time_ns).ok();
            json.push_str(",");
            json.push_str("\"end_time\":");
            write!(&mut json, "{}", span.end_time_ns).ok();
            json.push_str(",");
            json.push_str("\"attributes\":{");

            let mut first_attr = true;
            for (key, value) in &span.attributes {
                if !first_attr { json.push_str(","); }
                first_attr = false;
                json.push_str("\"");
                json.push_str(key);
                json.push_str("\":\"");
                json.push_str(value);
                json.push_str("\"");
            }

            json.push_str("}}");
        }

        json.push_str("\n]\n");

        // Write to file
        self.write_file(Self::EXPORT_PATH, json.as_bytes())?;

        // Clear pending
        self.pending_spans.clear();
        self.pending_count = 0;

        // Optional: HTTP export if enabled
        #[cfg(feature = "otel-http")]
        if unsafe { SIS_OTEL_HTTP_ENABLED } {
            self.http_export(&json)?;
        }

        Ok(())
    }
}
```

#### 2.2 Optional HTTP Exporter (bonus)
```rust
#[cfg(feature = "otel-http")]
impl OTelExporter {
    const HTTP_ENDPOINT: &'static str = "10.0.2.2";  // Host machine in QEMU
    const HTTP_PORT: u16 = 4318;  // OTLP HTTP default

    fn http_export(&mut self, json: &str) -> Result<(), &'static str> {
        use crate::net::tcp;

        // Build HTTP POST request
        let mut request = String::new();
        request.push_str("POST /v1/traces HTTP/1.1\r\n");
        request.push_str("Host: ");
        request.push_str(Self::HTTP_ENDPOINT);
        request.push_str("\r\n");
        request.push_str("Content-Type: application/json\r\n");
        request.push_str("Content-Length: ");
        write!(&mut request, "{}", json.len()).ok();
        request.push_str("\r\n\r\n");
        request.push_str(json);

        // Send via smoltcp
        match tcp::connect(Self::HTTP_ENDPOINT, Self::HTTP_PORT) {
            Ok(socket) => {
                tcp::send(socket, request.as_bytes())
                    .and_then(|_| tcp::recv_response(socket))
                    .and_then(|resp| {
                        if resp.starts_with(b"HTTP/1.1 2") {
                            Ok(())
                        } else {
                            Err("HTTP export failed")
                        }
                    })
                    .map_err(|_| "Network error")?;
                tcp::close(socket);
                Ok(())
            }
            Err(_) => {
                // Log but don't fail - HTTP export is optional
                uart::print_str("[OTel] HTTP export unavailable\n");
                Ok(())
            }
        }
    }
}
```

### Success Criteria:
- ✅ `/otel/spans.json` file created and populated with span data
- ✅ File rotates at 64KB (old content in `spans.old.json`)
- ✅ `OTelExporter::flush()` clears `pending_count`
- ✅ JSON format is valid and parseable
- ✅ Optional: HTTP POST to host works when enabled

---

## 3. Shadow Rollback → Model Lifecycle Integration

### Priority: HIGH (P2)
### Impact: Makes shadow deployments actionable, improves Phase 7
### Files to Modify:
- `crates/kernel/src/shadow/rollback.rs`
- `crates/kernel/src/model_lifecycle/mod.rs`

### Current State:
```rust
// Current stub
pub fn auto_rollback_if_needed() -> bool {
    // TODO: Actually trigger model lifecycle rollback
    false
}
```

### Required Implementation:

```rust
// In shadow/rollback.rs
use crate::model_lifecycle::{ModelLifecycle, ModelVersion};

pub fn auto_rollback_if_needed() -> bool {
    // Check rollback conditions
    let metrics = unsafe { &SHADOW_METRICS };

    if !should_rollback(metrics) {
        return false;
    }

    // Get current shadow version
    let shadow_version = match ModelLifecycle::get_shadow_version() {
        Some(v) => v,
        None => return false,
    };

    // Get previous stable version
    let stable_version = match ModelLifecycle::get_stable_version() {
        Some(v) => v,
        None => return false,
    };

    uart::print_str("[Shadow] Auto-rollback triggered: ");
    uart::print_str(&shadow_version.name);
    uart::print_str(" -> ");
    uart::print_str(&stable_version.name);
    uart::print_str("\n");

    // Perform rollback via model lifecycle
    match ModelLifecycle::rollback() {
        Ok(_) => {
            // Update shadow metrics
            metrics.rollback_count += 1;
            metrics.last_rollback_time = time::current_time_ns();

            // Log to audit
            crate::security::agent_audit::audit()
                .log_system_event("shadow_rollback", true);

            // Write rollback event to JSON
            write_rollback_event(&shadow_version, &stable_version);

            uart::print_str("[Shadow] Rollback complete\n");
            true
        }
        Err(e) => {
            uart::print_str("[Shadow] Rollback failed: ");
            uart::print_str(e);
            uart::print_str("\n");
            false
        }
    }
}

fn write_rollback_event(from: &ModelVersion, to: &ModelVersion) {
    let mut json = String::new();
    json.push_str("{\"event\":\"rollback\",");
    json.push_str("\"from\":\"");
    json.push_str(&from.name);
    json.push_str("\",\"to\":\"");
    json.push_str(&to.name);
    json.push_str("\",\"time\":");
    write!(&mut json, "{}", time::current_time_ns()).ok();
    json.push_str(",\"reason\":\"auto_rollback\"}\n");

    // Append to rollback log
    if let Ok(fd) = crate::vfs::open_file(
        "/var/log/rollback.json",
        OpenFlags::WRITE | OpenFlags::APPEND | OpenFlags::CREATE
    ) {
        let _ = crate::vfs::write_file(fd, json.as_bytes());
        crate::vfs::close_file(fd);
    }
}
```

### Model Lifecycle Updates:
```rust
// In model_lifecycle/mod.rs
impl ModelLifecycle {
    pub fn rollback() -> Result<(), &'static str> {
        let mut registry = unsafe { &mut MODEL_REGISTRY };

        // Swap active and shadow
        let current_active = registry.active_version;
        let current_shadow = registry.shadow_version
            .ok_or("No shadow version to rollback from")?;

        // Validation
        if current_active == current_shadow {
            return Err("Active and shadow versions are the same");
        }

        // Perform swap
        registry.shadow_version = Some(current_active);
        registry.active_version = registry.previous_version
            .ok_or("No previous version to rollback to")?;

        // Update history
        registry.history.push(ModelHistoryEntry {
            version: registry.active_version,
            timestamp: time::current_time_ns(),
            action: ModelAction::Rollback,
            metadata: "auto_rollback",
        });

        // Persist to disk
        registry.persist()?;

        // Notify subsystems
        crate::trace::metric_kv("model_rollback", 1);

        Ok(())
    }

    pub fn get_shadow_version() -> Option<ModelVersion> {
        unsafe { MODEL_REGISTRY.shadow_version }
    }

    pub fn get_stable_version() -> Option<ModelVersion> {
        unsafe {
            MODEL_REGISTRY.history
                .iter()
                .rev()
                .find(|e| e.action == ModelAction::Deploy)
                .map(|e| e.version)
        }
    }
}
```

### Success Criteria:
- ✅ `auto_rollback_if_needed()` triggers real model swap
- ✅ Model registry updated with rollback history
- ✅ `/var/log/rollback.json` contains rollback events
- ✅ Single concise status line printed on rollback
- ✅ Metrics and audit logs updated

---

## 4. Time Base + Scheduler Glue

### Priority: MEDIUM-HIGH (P3)
### Impact: Fixes timing inconsistencies, stabilizes Phase 8
### Files to Modify:
- `crates/kernel/src/time.rs`
- `crates/kernel/src/syscall/mod.rs`
- `crates/kernel/src/scheduler/deterministic.rs`

### Required Implementation:

#### 4.1 Hardware Timer Base
```rust
// In time.rs
#[cfg(target_arch = "aarch64")]
pub fn read_cycle_counter() -> u64 {
    let counter: u64;
    unsafe {
        asm!("mrs {}, cntvct_el0", out(reg) counter);
    }
    counter
}

// Cache frequency at boot
static mut TIMER_FREQUENCY: u64 = 0;

pub fn init_timer() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let freq: u64;
        asm!("mrs {}, cntfrq_el0", out(reg) freq);
        TIMER_FREQUENCY = freq;
    }
}

pub fn cycles_to_ns(cycles: u64) -> u64 {
    let freq = unsafe { TIMER_FREQUENCY };
    if freq == 0 { return 0; }

    // Avoid overflow: (cycles * 1_000_000_000) / freq
    let seconds = cycles / freq;
    let remainder = cycles % freq;
    seconds * 1_000_000_000 + (remainder * 1_000_000_000) / freq
}

pub fn current_time_ns() -> u64 {
    cycles_to_ns(read_cycle_counter())
}

pub fn current_time_us() -> u64 {
    current_time_ns() / 1000
}
```

#### 4.2 Syscall Implementation
```rust
// In syscall/mod.rs
pub fn sys_clock_gettime(clock_id: i32, tp: *mut TimeSpec) -> isize {
    if tp.is_null() {
        return -EFAULT;
    }

    let ns = match clock_id {
        CLOCK_REALTIME | CLOCK_MONOTONIC => time::current_time_ns(),
        CLOCK_PROCESS_CPUTIME_ID => {
            // For now, same as monotonic
            time::current_time_ns()
        }
        _ => return -EINVAL,
    };

    let timespec = TimeSpec {
        tv_sec: (ns / 1_000_000_000) as i64,
        tv_nsec: (ns % 1_000_000_000) as i64,
    };

    unsafe {
        *tp = timespec;
    }

    0
}

pub fn sys_nanosleep(req: *const TimeSpec, rem: *mut TimeSpec) -> isize {
    if req.is_null() {
        return -EFAULT;
    }

    let req_spec = unsafe { *req };
    let sleep_ns = req_spec.tv_sec as u64 * 1_000_000_000 + req_spec.tv_nsec as u64;

    let start = time::current_time_ns();
    let target = start + sleep_ns;

    // Simple busy-wait for now (could yield to scheduler)
    while time::current_time_ns() < target {
        crate::scheduler::yield_cpu();
    }

    if !rem.is_null() {
        unsafe {
            *rem = TimeSpec { tv_sec: 0, tv_nsec: 0 };
        }
    }

    0
}
```

#### 4.3 Deterministic Scheduler Integration
```rust
// In scheduler/deterministic.rs
impl<const N: usize> DeterministicScheduler<N> {
    pub fn tick(&mut self) {
        let current_ns = time::current_time_ns();

        // Update server budgets
        for server in &mut self.servers {
            if server.active {
                let elapsed = current_ns - server.last_replenish;

                // Replenish every period
                if elapsed >= server.period_ns {
                    server.budget_ns = server.capacity_ns;
                    server.last_replenish = current_ns;
                }
            }
        }
    }

    pub fn schedule_next_process(&mut self) -> Option<Pid> {
        self.tick();  // Update budgets based on real time

        let current_ns = time::current_time_ns();

        // Find server with budget
        for server in &mut self.servers {
            if server.active && server.budget_ns > 0 {
                // Deduct quantum from budget
                let quantum_ns = 1_000_000;  // 1ms quantum
                server.budget_ns = server.budget_ns.saturating_sub(quantum_ns);

                return Some(server.pid);
            }
        }

        None
    }
}
```

### Success Criteria:
- ✅ `time::current_time_ns()` returns monotonic nanoseconds from hardware timer
- ✅ `clock_gettime` syscall returns consistent time
- ✅ `nanosleep` actually sleeps for requested duration
- ✅ Scheduler budgets based on real time, not iteration counts
- ✅ No time going backwards

---

## 5. Metrics Exporter Real Data

### Priority: MEDIUM (P4)
### Impact: Makes dashboards accurate and regression detection reliable
### Files to Modify:
- `crates/kernel/src/metrics_export.rs`
- `crates/kernel/src/panic.rs`
- `crates/kernel/src/mm/heap.rs`

### Required Implementation:

```rust
// In metrics_export.rs
use crate::mm::heap::HEAP_STATS;
use crate::net::smoltcp::NETWORK_STATS;
use crate::panic::PANIC_COUNT;

pub fn export_metrics_prometheus(buffer: &mut String) -> Result<(), &'static str> {
    // Real heap metrics
    let heap_stats = unsafe { &HEAP_STATS };
    writeln!(buffer, "# HELP heap_allocated_bytes Current heap allocation").ok();
    writeln!(buffer, "# TYPE heap_allocated_bytes gauge").ok();
    writeln!(buffer, "heap_allocated_bytes {}", heap_stats.allocated).ok();

    writeln!(buffer, "# HELP heap_peak_bytes Peak heap allocation").ok();
    writeln!(buffer, "# TYPE heap_peak_bytes gauge").ok();
    writeln!(buffer, "heap_peak_bytes {}", heap_stats.peak).ok();

    writeln!(buffer, "# HELP heap_allocations_total Total allocation count").ok();
    writeln!(buffer, "# TYPE heap_allocations_total counter").ok();
    writeln!(buffer, "heap_allocations_total {}", heap_stats.total_allocations).ok();

    // Real network metrics
    let net_stats = unsafe { &NETWORK_STATS };
    writeln!(buffer, "# HELP network_tx_bytes_total Total bytes transmitted").ok();
    writeln!(buffer, "# TYPE network_tx_bytes_total counter").ok();
    writeln!(buffer, "network_tx_bytes_total {}", net_stats.tx_bytes).ok();

    writeln!(buffer, "# HELP network_rx_bytes_total Total bytes received").ok();
    writeln!(buffer, "# TYPE network_rx_bytes_total counter").ok();
    writeln!(buffer, "network_rx_bytes_total {}", net_stats.rx_bytes).ok();

    writeln!(buffer, "# HELP network_tx_packets_total Total packets transmitted").ok();
    writeln!(buffer, "# TYPE network_tx_packets_total counter").ok();
    writeln!(buffer, "network_tx_packets_total {}", net_stats.tx_packets).ok();

    writeln!(buffer, "# HELP network_rx_packets_total Total packets received").ok();
    writeln!(buffer, "# TYPE network_rx_packets_total counter").ok();
    writeln!(buffer, "network_rx_packets_total {}", net_stats.rx_packets).ok();

    // Real panic count
    let panic_count = unsafe { PANIC_COUNT.load(Ordering::Relaxed) };
    writeln!(buffer, "# HELP kernel_panics_total Total kernel panics").ok();
    writeln!(buffer, "# TYPE kernel_panics_total counter").ok();
    writeln!(buffer, "kernel_panics_total {}", panic_count).ok();

    Ok(())
}
```

### Heap Stats Integration:
```rust
// In mm/heap.rs
pub struct HeapStats {
    pub allocated: usize,
    pub peak: usize,
    pub total_allocations: u64,
    pub total_deallocations: u64,
}

pub static mut HEAP_STATS: HeapStats = HeapStats {
    allocated: 0,
    peak: 0,
    total_allocations: 0,
    total_deallocations: 0,
};

// In allocator implementation
unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner_alloc(layout);

        if !ptr.is_null() {
            HEAP_STATS.allocated += layout.size();
            HEAP_STATS.total_allocations += 1;
            if HEAP_STATS.allocated > HEAP_STATS.peak {
                HEAP_STATS.peak = HEAP_STATS.allocated;
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner_dealloc(ptr, layout);
        HEAP_STATS.allocated = HEAP_STATS.allocated.saturating_sub(layout.size());
        HEAP_STATS.total_deallocations += 1;
    }
}
```

### Panic Counter:
```rust
// In panic.rs
use core::sync::atomic::{AtomicU32, Ordering};

pub static PANIC_COUNT: AtomicU32 = AtomicU32::new(0);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    PANIC_COUNT.fetch_add(1, Ordering::Relaxed);

    // Existing panic handling...
    uart::print_str("\n[PANIC] ");
    // ...

    loop { unsafe { asm!("wfi"); } }
}
```

### Network Stats:
```rust
// In net module
pub struct NetworkStats {
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub tx_packets: u64,
    pub rx_packets: u64,
}

pub static mut NETWORK_STATS: NetworkStats = NetworkStats {
    tx_bytes: 0,
    rx_bytes: 0,
    tx_packets: 0,
    rx_packets: 0,
};

// Hook into smoltcp send/recv
```

### Success Criteria:
- ✅ Heap metrics reflect actual allocator statistics
- ✅ Network counters increment on actual TX/RX
- ✅ Panic count increments on panic
- ✅ Metrics have proper Prometheus format with units
- ✅ `/metrics` endpoint returns real data

---

## 6. AgentSys I/O Semantics

### Priority: MEDIUM (P5)
### Impact: Makes non-FS AgentSys operations testable and demoable
### Files to Modify:
- `crates/kernel/src/agent_sys/handlers/io.rs`
- `crates/kernel/src/agent_sys/handlers/audio.rs`

### Required Implementation:

#### 6.1 Screenshot Handler
```rust
// In io.rs
pub fn handle_screenshot(agent_id: AgentId, _payload: &[u8]) -> Result<(), CtrlError> {
    // Policy check
    let decision = policy().check(
        agent_id,
        Capability::Screenshot,
        &Resource::NoResource,
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x3C, false);
        return Err(CtrlError::PermissionDenied);
    }

    // Create minimal PNG placeholder
    let timestamp = time::current_time_ns() / 1_000_000;  // ms
    let filename = format!("/tmp/agentsys/screenshot_{}.png", timestamp);

    // Ensure directory exists
    let _ = crate::vfs::mkdir("/tmp/agentsys", 0o755);

    // Minimal PNG header (1x1 pixel, black)
    const PNG_DATA: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,  // PNG signature
        0x00, 0x00, 0x00, 0x0D,  // IHDR length
        0x49, 0x48, 0x44, 0x52,  // IHDR
        0x00, 0x00, 0x00, 0x01,  // width = 1
        0x00, 0x00, 0x00, 0x01,  // height = 1
        0x08, 0x00, 0x00, 0x00, 0x00,  // bit depth, color type, etc
        0x3A, 0x7E, 0x9B, 0x55,  // CRC
        0x00, 0x00, 0x00, 0x0A,  // IDAT length
        0x49, 0x44, 0x41, 0x54,  // IDAT
        0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01,  // compressed data
        0xE5, 0x27, 0xDE, 0xFC,  // CRC
        0x00, 0x00, 0x00, 0x00,  // IEND length
        0x49, 0x45, 0x4E, 0x44,  // IEND
        0xAE, 0x42, 0x60, 0x82,  // CRC
    ];

    match crate::vfs::create_and_write(&filename, PNG_DATA) {
        Ok(_) => {
            uart::print_str("[Screenshot] Saved: ");
            uart::print_str(&filename);
            uart::print_str(" (");
            uart::print_u32(PNG_DATA.len() as u32);
            uart::print_str(" bytes)\n");

            audit().log_operation(agent_id, 0x3C, true);
            Ok(())
        }
        Err(_) => {
            uart::print_str("[Screenshot] Failed to save\n");
            audit().log_operation(agent_id, 0x3C, false);
            Err(CtrlError::IoError)
        }
    }
}
```

#### 6.2 Screen Recording Handler
```rust
pub fn handle_record(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    // Policy check
    let decision = policy().check(
        agent_id,
        Capability::Capture,
        &Resource::NoResource,
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x3D, false);
        return Err(CtrlError::PermissionDenied);
    }

    // Parse duration from payload (default 5 seconds)
    let duration_ms = if payload.len() >= 4 {
        u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]])
    } else {
        5000
    };

    let timestamp = time::current_time_ns() / 1_000_000;
    let filename = format!("/tmp/agentsys/recording_{}_{}.mp4", timestamp, duration_ms);

    // Create placeholder MP4 (minimal valid structure)
    // This is a simplified placeholder - real implementation would capture frames
    let placeholder = format!("VIDEO:{}ms:PLACEHOLDER", duration_ms);

    match crate::vfs::create_and_write(&filename, placeholder.as_bytes()) {
        Ok(_) => {
            uart::print_str("[Record] Started: ");
            uart::print_str(&filename);
            uart::print_str(" (");
            uart::print_u32(duration_ms);
            uart::print_str("ms)\n");

            audit().log_operation(agent_id, 0x3D, true);
            Ok(())
        }
        Err(_) => {
            uart::print_str("[Record] Failed to start\n");
            audit().log_operation(agent_id, 0x3D, false);
            Err(CtrlError::IoError)
        }
    }
}
```

#### 6.3 Audio Handlers
```rust
// In audio.rs
pub fn handle_play(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    // Policy check
    let decision = policy().check(
        agent_id,
        Capability::AudioControl,
        &Resource::NoResource,
    );

    if !matches!(decision, PolicyDecision::Allow) {
        audit().log_operation(agent_id, 0x36, false);
        return Err(CtrlError::PermissionDenied);
    }

    // Parse track ID
    let track_id = if payload.len() >= 4 {
        u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]])
    } else {
        0
    };

    // Create audio placeholder WAV file
    let filename = format!("/tmp/agentsys/audio_track_{}.wav", track_id);

    // Minimal WAV header (1 second, 8kHz, mono, 8-bit)
    let mut wav_data = Vec::new();
    wav_data.extend_from_slice(b"RIFF");  // ChunkID
    wav_data.extend_from_slice(&44u32.to_le_bytes());  // ChunkSize
    wav_data.extend_from_slice(b"WAVE");  // Format
    wav_data.extend_from_slice(b"fmt ");  // Subchunk1ID
    wav_data.extend_from_slice(&16u32.to_le_bytes());  // Subchunk1Size
    wav_data.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (PCM)
    wav_data.extend_from_slice(&1u16.to_le_bytes());  // NumChannels
    wav_data.extend_from_slice(&8000u32.to_le_bytes());  // SampleRate
    wav_data.extend_from_slice(&8000u32.to_le_bytes());  // ByteRate
    wav_data.extend_from_slice(&1u16.to_le_bytes());  // BlockAlign
    wav_data.extend_from_slice(&8u16.to_le_bytes());  // BitsPerSample
    wav_data.extend_from_slice(b"data");  // Subchunk2ID
    wav_data.extend_from_slice(&8u32.to_le_bytes());  // Subchunk2Size

    // Add 8 samples of silence
    for _ in 0..8 {
        wav_data.push(128);  // 8-bit PCM center
    }

    match crate::vfs::create_and_write(&filename, &wav_data) {
        Ok(_) => {
            uart::print_str("[Audio] Playing track ");
            uart::print_u32(track_id);
            uart::print_str(" -> ");
            uart::print_str(&filename);
            uart::print_str("\n");

            audit().log_operation(agent_id, 0x36, true);
            Ok(())
        }
        Err(_) => {
            uart::print_str("[Audio] Failed to play\n");
            audit().log_operation(agent_id, 0x36, false);
            Err(CtrlError::IoError)
        }
    }
}
```

### Success Criteria:
- ✅ Screenshot creates PNG files in `/tmp/agentsys/`
- ✅ Recording creates placeholder video files
- ✅ Audio play creates WAV files
- ✅ Files have valid headers (PNG/WAV)
- ✅ Filenames include timestamps
- ✅ Policy and rate limits still enforced

---

## 7. Minimal Syscalls (readlinkat, etc.)

### Priority: LOW-MEDIUM (P6)
### Impact: Removes obvious syscall stubs that break userspace
### Files to Modify:
- `crates/kernel/src/syscall/mod.rs`
- `crates/kernel/src/vfs/mod.rs`

### Required Implementation:

```rust
// In syscall/mod.rs
pub fn sys_readlinkat(dirfd: i32, path: *const u8, buf: *mut u8, bufsiz: usize) -> isize {
    if path.is_null() || buf.is_null() {
        return -EFAULT;
    }

    // Convert path to str
    let path_str = match unsafe { cstr_to_str(path) } {
        Ok(s) => s,
        Err(_) => return -EINVAL,
    };

    // Handle dirfd (AT_FDCWD = -100 means current dir)
    let full_path = if dirfd == AT_FDCWD {
        path_str.to_string()
    } else {
        // For now, ignore dirfd
        path_str.to_string()
    };

    // Read link target from VFS
    match crate::vfs::readlink(&full_path) {
        Ok(target) => {
            let target_bytes = target.as_bytes();
            let copy_len = core::cmp::min(bufsiz, target_bytes.len());

            unsafe {
                core::ptr::copy_nonoverlapping(
                    target_bytes.as_ptr(),
                    buf,
                    copy_len
                );
            }

            copy_len as isize
        }
        Err(Errno::EINVAL) => -EINVAL,  // Not a symlink
        Err(Errno::ENOENT) => -ENOENT,  // File not found
        Err(_) => -EIO,
    }
}

// VFS support
impl VFS {
    pub fn readlink(&self, path: &str) -> Result<String, Errno> {
        // For now, return error for non-symlinks
        // Real implementation would check inode type

        // Special case for /proc/self/exe
        if path == "/proc/self/exe" {
            return Ok("/bin/init".to_string());
        }

        // Check if path exists
        match self.stat(path) {
            Ok(stat) if stat.is_symlink => {
                // Read symlink target from filesystem
                // For now, return placeholder
                Ok("/dev/null".to_string())
            }
            Ok(_) => Err(Errno::EINVAL),  // Not a symlink
            Err(e) => Err(e),
        }
    }
}
```

### Success Criteria:
- ✅ `readlinkat` returns link targets or EINVAL for non-links
- ✅ `/proc/self/exe` returns sensible value
- ✅ `nanosleep` uses unified time base
- ✅ `clock_gettime` consistent with hardware timer
- ✅ No more "unimplemented syscall" panics for common calls

---

## 8. LLM Budget ↔ Deterministic Integration

### Priority: LOW (P7)
### Impact: Stabilizes Phase 7/8 timing without full LLM
### Files to Modify:
- `crates/kernel/src/llm/basic.rs`
- `crates/kernel/src/scheduler/deterministic.rs`

### Required Implementation:

```rust
// In llm/basic.rs
pub fn configure_budget(tokens_per_period: u32, period_ms: u32) -> Result<(), &'static str> {
    use crate::scheduler::deterministic::{DeterministicScheduler, CBSServer};

    // Convert to nanoseconds
    let period_ns = period_ms as u64 * 1_000_000;
    let capacity_ns = (tokens_per_period as u64 * 1_000_000) / 10;  // Rough estimate

    // Register LLM server with scheduler
    let server = CBSServer {
        pid: LLM_PID,
        capacity_ns,
        period_ns,
        budget_ns: capacity_ns,
        deadline_ns: time::current_time_ns() + period_ns,
        active: true,
        last_replenish: time::current_time_ns(),
    };

    DeterministicScheduler::register_server(server)?;

    // Store budget locally
    unsafe {
        LLM_BUDGET.tokens_remaining = tokens_per_period;
        LLM_BUDGET.period_ms = period_ms;
        LLM_BUDGET.last_refill = time::current_time_ns();
    }

    uart::print_str("[LLM] Budget configured: ");
    uart::print_u32(tokens_per_period);
    uart::print_str(" tokens per ");
    uart::print_u32(period_ms);
    uart::print_str("ms\n");

    Ok(())
}

pub fn check_and_consume_budget(tokens: u32) -> Result<(), &'static str> {
    let budget = unsafe { &mut LLM_BUDGET };
    let now = time::current_time_ns();

    // Refill if period elapsed
    let elapsed_ms = (now - budget.last_refill) / 1_000_000;
    if elapsed_ms >= budget.period_ms as u64 {
        budget.tokens_remaining = budget.tokens_per_period;
        budget.last_refill = now;
    }

    // Check budget
    if budget.tokens_remaining < tokens {
        // Log deadline miss
        crate::trace::metric_kv("llm_deadline_miss", 1);
        return Err("Insufficient token budget");
    }

    // Consume tokens
    budget.tokens_remaining -= tokens;

    // Update scheduler
    DeterministicScheduler::consume_budget(
        LLM_PID,
        (tokens as u64 * 1_000_000) / 10  // Convert to ns
    );

    Ok(())
}
```

### Success Criteria:
- ✅ `llmctl load --budget` provisions scheduler server
- ✅ Token consumption tracked against period
- ✅ Deadline misses generate metrics
- ✅ Scheduler budget reflects LLM usage
- ✅ No crashes when budget exhausted

---

## 9. virtio-snd (Optional - Defer if Time Tight)

### Priority: LOWEST (P8)
### Impact: Nice-to-have audio support
### Files to Modify:
- `crates/kernel/src/drivers/virtio_snd.rs`
- `Cargo.toml` (feature flag)

### Implementation Options:

#### Option A: Hide Feature Flag
```toml
# In Cargo.toml
[features]
# virtio-snd = []  # Comment out until ready
```

#### Option B: Minimal Stub
```rust
// In virtio_snd.rs
pub struct VirtIOSound {
    // Minimal fields
}

impl VirtIOSound {
    pub fn new() -> Self {
        uart::print_str("[VirtIO-Sound] Audio device (placeholder)\n");
        Self {}
    }

    pub fn play(&self, _data: &[u8]) -> Result<(), &'static str> {
        uart::print_str("[VirtIO-Sound] Playback not yet implemented\n");
        Err("Not implemented")
    }
}
```

### Success Criteria:
- ✅ Either feature hidden OR clear "not implemented" messages
- ✅ No crashes when virtio-snd accessed
- ✅ README updated to reflect actual status

---

## 10. OTel/Trace Decision Cross-checks

### Priority: LOW (P9)
### Impact: Makes decision traces observable
### Files to Modify:
- `crates/kernel/src/decision_trace/mod.rs`
- `crates/kernel/src/otel/mod.rs`

### Required Implementation:

```rust
// In decision_trace/mod.rs
pub fn trace_decision(
    component: &str,
    decision: &str,
    metadata: &[(String, String)]
) {
    // Log locally
    let entry = DecisionEntry {
        timestamp: time::current_time_ns(),
        component: component.to_string(),
        decision: decision.to_string(),
        metadata: metadata.to_vec(),
    };

    unsafe {
        DECISION_BUFFER.push(entry.clone());
    }

    // Send to OTel
    #[cfg(feature = "otel")]
    {
        let mut span = otel::Span::new("decision");
        span.set_attribute("component", component);
        span.set_attribute("decision", decision);
        for (k, v) in metadata {
            span.set_attribute(k, v);
        }
        otel::export_span(span);
    }

    // Write to trace file
    let mut json = String::new();
    json.push_str("{\"ts\":");
    write!(&mut json, "{}", entry.timestamp).ok();
    json.push_str(",\"component\":\"");
    json.push_str(component);
    json.push_str("\",\"decision\":\"");
    json.push_str(decision);
    json.push_str("\"}\n");

    if let Ok(fd) = crate::vfs::open_file(
        "/otel/decisions.jsonl",
        OpenFlags::WRITE | OpenFlags::APPEND | OpenFlags::CREATE
    ) {
        let _ = crate::vfs::write_file(fd, json.as_bytes());
        crate::vfs::close_file(fd);
    }
}
```

### Success Criteria:
- ✅ Decision traces appear in OTel spans
- ✅ `/otel/decisions.jsonl` contains decision log
- ✅ Traces include component and metadata
- ✅ Can correlate decisions with spans

---

## Testing Strategy

### Phase-by-Phase Validation

1. **After Each Implementation:**
   - Build with feature flags
   - Boot in QEMU
   - Run specific test commands
   - Check file outputs
   - Verify no regressions

2. **Integration Testing:**
   ```bash
   # After AgentSys FS implementation
   cargo run -p sis-testing --release -- --phase 9

   # After OTel implementation
   cargo run -p sis-testing --release -- --phase 7

   # After all implementations
   cargo run -p sis-testing --release
   ```

3. **Manual Verification:**
   ```bash
   # Boot and test each feature
   SIS_FEATURES="agentsys,otel,deterministic" ./scripts/uefi_run.sh

   # In shell:
   sis> agentsys test-fs-list
   sis> ls /otel/
   sis> cat /otel/spans.json
   sis> metrics
   ```

---

## Delivery Checklist

### For Each Implementation:
- [ ] Code compiles without warnings
- [ ] Feature can be enabled/disabled via Cargo feature
- [ ] Success criteria met
- [ ] No performance regression
- [ ] Documentation updated if needed
- [ ] Test commands provided

### Final Delivery:
- [ ] All 10 implementations complete
- [ ] Test suite pass rate improved
- [ ] No new panics or crashes
- [ ] Branch ready for integration
- [ ] Summary of changes provided

---

## Branch Strategy

1. Create feature branch from `main`:
   ```bash
   git checkout -b feature/stubs-to-real-implementation
   ```

2. Implement in order (1-10)
3. Commit after each working implementation
4. Push branch for review
5. Provide branch URL for pull

---

## Notes for AI Agent Implementation

- Keep changes focused and minimal
- Preserve existing functionality
- Add error handling, don't panic
- Use existing VFS/syscall interfaces
- Print concise status messages
- Update metrics and audit logs
- Test each implementation in isolation
- Document any API changes

---

## Expected Outcomes

After all implementations:
- Phase 9 AgentSys: 90%+ test pass rate (from ~0%)
- Phase 7 AI Ops: 70%+ test pass rate (from ~20%)
- Phase 8 Performance: 80%+ test pass rate (from ~33%)
- Real metrics in dashboards
- Observable trace/span files
- Functional AgentSys file operations
- Stable timing and scheduling

---

**END OF IMPLEMENTATION PLAN**