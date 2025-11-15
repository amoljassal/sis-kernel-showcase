# Kernel Core - Module Integration Guide

## Overview

This is the heart of SIS Kernel. This README provides a **module-by-module map** showing how major subsystems integrate, what traits they expose, and where boundaries lie.

## Directory Structure

```
crates/kernel/src/
├── main.rs              # Boot sequence, initialization, main loop
├── shell.rs             # Interactive shell (see shell/README.md)
│
├── Core Library (lib/)
│   ├── panic.rs         # Enhanced panic handler (Phase 4)
│   ├── printk.rs        # Structured logging (Human/JSON formats)
│   ├── ringbuf.rs       # Bounded ring buffers
│   ├── debug.rs         # Debug utilities
│   └── error.rs         # Error handling
│
├── OS Foundation (Phase A)
│   ├── vfs/             # Virtual File System (see vfs/README.md)
│   ├── mm/              # Memory management
│   ├── syscall/         # System call interface
│   ├── drivers/         # Device drivers (see drivers/README.md)
│   ├── net/             # Network stack
│   ├── security/        # Security subsystem
│   ├── process/         # Process management
│   └── (more...)
│
├── AI/ML/Neural (Phases 1-3, 5-6)
│   ├── graph.rs         # Dataflow graph (Phase 1)
│   ├── deterministic.rs # CBS+EDF scheduler (Phase 2)
│   ├── llm.rs           # LLM service (Phase 2)
│   ├── npu.rs           # NPU emulation (Phase 3)
│   ├── neural.rs        # Neural coordinator (Phase 3)
│   ├── autonomy.rs      # Autonomous control (Phase 3)
│   └── (more...)
│
└── Production (Phase 4)
    ├── build_info.rs    # Build metadata tracking
    ├── chaos.rs         # Chaos engineering
    ├── metrics_export.rs# Metrics export
    └── syscall/validation.rs # Security hardening
```

## Key Integration Points

### 1. VFS → Everything
**What:** Unified file system abstraction
**Trait:** `InodeOps` (defines read/write/readdir/lookup/create/etc.)
**Who uses it:**
- Syscall layer (`sys_open`, `sys_read`, `sys_write`)
- Shell commands (`cat`, `ls`, `mkdir`)
- Block devices (mount ext2/ext4 filesystems)

**See:** [`vfs/README.md`](vfs/README.md) for full integration guide

---

### 2. Drivers → VirtIO Framework
**What:** Device driver abstraction
**Traits:** `Device`, `BlockDevice`, `NetworkDevice`
**Who uses it:**
- VFS (block devices for filesystems)
- Network stack (network devices for TCP/IP)
- Graphics (GPU devices for rendering)

**See:** [`drivers/README.md`](drivers/README.md) for driver implementation guide

---

### 3. Shell → Module APIs
**What:** Command routing to subsystems
**Pattern:** Thin dispatch (shell.rs) → Fat helpers (shell/*_helpers.rs) → Module APIs
**Boundary rule:** Shell NEVER modifies kernel state directly

**See:** [`shell/README.md`](shell/README.md) for command implementation guide

---

### 4. Dataflow Graph → All AI Modules
**What:** Observable computation graph for AI/ML
**API:** `graph::add_channel()`, `graph::add_operator()`, `graph::emit_sample()`
**Who uses it:**
- Neural coordinator (`neural.rs`)
- Autonomous control (`autonomy.rs`)
- All AI prediction modules (memory, scheduling, network)

**Integration pattern:**
```rust
// Add graph channel for observability
graph::add_channel(ChannelId::MemoryPredictions, ChannelType::F32);

// Emit samples from AI module
graph::emit_sample(ChannelId::MemoryPredictions, prediction_value);

// Shell queries graph
graphctl status  // Shows all channels and operators
```

**See:** `graph.rs` header docs for full API

---

### 5. LLM Service → Model Security
**What:** Kernel-resident LLM with signed model packages
**Feature:** `llm` (feature-gated)
**Security:** SHA-256 + Ed25519 signature verification (when `crypto-real` enabled)
**Who uses it:**
- Shell (`llmctl`, `llminfer`, `llmstream` commands)
- Autonomous control (decision explanations)

**Integration pattern:**
```rust
// Load signed model package
llm::load_model("/models/my_model.pkg")?;

// Run inference
let output = llm::infer(&input_tokens)?;

// Stream responses
llm::stream_response(&prompt, |chunk| {
    println!("{}", chunk);
});
```

**See:** `llm.rs` header docs and `docs/guides/LLM-KERNEL-INTEGRATION.md`

---

### 6. Deterministic Scheduler → LLM Budgeting
**What:** CBS+EDF scheduler with time budgets
**Feature:** `deterministic`
**Who uses it:**
- LLM service (enforce inference time limits)
- Real-time tasks (guaranteed scheduling)

**Integration pattern:**
```rust
// Register task with CBS budget
det::register_task(task_id, budget_us, period_us)?;

// Scheduler enforces budget automatically
// Task is preempted if budget exhausted
```

**See:** `deterministic.rs` header docs

---

### 7. Chaos Engineering → All Subsystems
**What:** Failure injection for resilience testing
**Feature:** `chaos`
**Modes:** DiskFull, NetworkFail, MemoryPressure, CorruptedData, SlowIO, RandomFaults, AllFaults
**Who uses it:**
- VFS (disk full simulation)
- Network stack (packet drops)
- Memory allocator (allocation failures)

**Integration pattern:**
```rust
// Check if chaos mode is active
if chaos::should_inject_fault() {
    return Err(Error::DiskFull);  // Simulated failure
}

// Shell controls chaos
chaos DiskFull    # Enable disk full simulation
chaos none        # Disable chaos
```

**See:** `chaos.rs` and `docs/guides/CHAOS_TESTING.md`

---

### 8. Structured Logging → Automation
**What:** Dual-format logging (Human / JSON)
**Atomic switching:** Runtime switchable with zero overhead
**Who uses it:**
- All kernel modules (via `printk!()`, `log_info!()`, etc.)
- CI/CD pipelines (parse JSON logs)
- Shell (switch formats with `set_log_format`)

**Integration pattern:**
```rust
// Human-readable
log_info!("GPU", "READY");  // Output: [INFO] GPU: READY

// Switch to JSON
set_log_format(LogFormat::Json);
log_info!("GPU", "READY");  // Output: {"ts":123,"subsystem":"GPU","status":"READY","level":"INFO"}
```

**See:** `lib/printk.rs` header docs

---

## Module Dependency Map

```
┌─────────────────────────────────────────────────────────────┐
│                        Shell (UI Layer)                     │
├─────────────────────────────────────────────────────────────┤
│  graphctl │ llmctl │ autoctl │ det │ chaos │ metricsctl    │
└────┬───────────┬─────────┬─────────┬────────┬──────────┬───┘
     │           │         │         │        │          │
     ▼           ▼         ▼         ▼        ▼          ▼
┌─────────────────────────────────────────────────────────────┐
│                     Kernel Modules (API Layer)              │
├─────────────────────────────────────────────────────────────┤
│ graph.rs │ llm.rs │ autonomy.rs │ det.rs │ chaos.rs │ ...  │
└────┬───────────┬─────────┬─────────┬────────┬──────────┬───┘
     │           │         │         │        │          │
     └───────────┴─────────┴─────────┴────────┴──────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  Core Services (OS Layer)                   │
├─────────────────────────────────────────────────────────────┤
│       VFS  │  Drivers  │  Syscalls  │  Memory  │  Net      │
└─────────────────────────────────────────────────────────────┘
```

## Cross-Module Communication Patterns

### Pattern 1: Direct API Call (Preferred)
```rust
// Module A calls Module B's public API
let result = module_b::do_something(args);
```

**When to use:** Module B exports a stable, documented API

---

### Pattern 2: Agent Bus (Decoupled Communication)
```rust
// Module A sends message to agent bus
agent_bus::send(AgentId::Scheduler, Message::UpdatePriority(priority));

// Module B listens on agent bus
match agent_bus::receive() {
    Some(Message::UpdatePriority(p)) => handle_priority(p),
    // ...
}
```

**When to use:** Modules should not directly depend on each other

---

### Pattern 3: Dataflow Graph (Observable Data Flow)
```rust
// Producer emits sample
graph::emit_sample(ChannelId::CpuLoad, load_percentage);

// Consumer reads from channel (optional)
let samples = graph::read_channel(ChannelId::CpuLoad, count);

// Observer views graph
graphctl status  // Shell command shows all data flow
```

**When to use:** Data flow should be observable and debuggable

---

### Pattern 4: Global State with Atomics (Lock-Free)
```rust
// Module publishes state via atomic
static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}
```

**When to use:** Simple boolean flags or counters

---

## Boundary Rules (MUST FOLLOW)

1. **Platform abstraction for hardware access**
   - NO hardcoded MMIO addresses
   - Use `platform::get_uart_base()`, not `0x9000000`

2. **Module APIs are narrow and documented**
   - Export minimal public interface
   - Internal details are `pub(crate)` or `pub(super)`

3. **Feature flags control subsystem compilation**
   - Heavy features default to OFF
   - Use `#[cfg(feature = "...")]` liberally

4. **No panics in production paths**
   - Use `Result<T, E>` for errors
   - Panic only in `debug_assert!()` or truly impossible cases

5. **Bounded data structures**
   - Use ring buffers, not `Vec` for unbounded growth
   - Preallocate at init time when possible

6. **Interrupt handlers are minimal**
   - Set atomic flags, then defer work
   - No heap allocation or heavy computation

7. **Shell commands don't modify kernel state directly**
   - Always route through module APIs
   - Helpers parse args and validate, modules execute

## Testing Integration Points

### Unit Tests (Per-Module)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_api() {
        assert_eq!(my_function(42), 84);
    }
}
```

### Integration Tests (Cross-Module)
```rust
// In crates/testing/
#[test]
fn test_vfs_with_block_device() {
    let block_dev = MockBlockDevice::new();
    vfs::mount("/mnt", "ext4", block_dev).unwrap();
    // ...
}
```

### Shell Tests (End-to-End)
```bash
# scripts/automated_shell_tests.sh
send_command "graphctl add-channel cpu_load f32"
expect_output "Channel added: cpu_load"
```

## Performance Optimization Patterns

1. **Atomic operations for hot paths**
   - Use `Ordering::Relaxed` for counters
   - Use `Ordering::Acquire`/`Release` for synchronization

2. **Ring buffers for bounded growth**
   - Preallocate fixed-size buffer at init
   - Wrap around when full (oldest data lost)

3. **Lazy initialization**
   - Defer expensive setup until first use
   - Use `Once` or `Lazy` for one-time init

4. **Cache-friendly data structures**
   - Keep hot data together (struct of arrays > array of structs)
   - Align to cache line boundaries (64 bytes)

## Related Documentation

- **Subsystem READMEs:**
  - [`vfs/README.md`](vfs/README.md) - Virtual File System
  - [`drivers/README.md`](drivers/README.md) - Device Drivers
  - [`shell/README.md`](shell/README.md) - Shell Commands

- **Guides:**
  - `docs/guides/LLM-KERNEL-INTEGRATION.md` - LLM service integration
  - `docs/guides/CHAOS_TESTING.md` - Chaos engineering guide
  - `docs/guides/BUILD.md` - Build instructions
  - `docs/guides/SECURITY.md` - Security hardening

- **Architecture:**
  - `docs/architecture/ARCHITECTURE.md` - System-wide architecture
  - `docs/architecture/kernel-neural-net.md` - Neural network design

- **Main README:**
  - `../../README.md` - High-level project overview

## Future Subsystem READMEs (TODO)

- [ ] `mm/README.md` - Memory Management
- [ ] `syscall/README.md` - System Call Interface
- [ ] `net/README.md` - Network Stack
- [ ] `security/README.md` - Security Subsystem
- [ ] `process/README.md` - Process Management
- [ ] `virtio/README.md` - VirtIO Framework
- [ ] `platform/README.md` - Platform Abstraction

## Questions?

For questions about module integration or kernel architecture:
1. Check this README and subsystem READMEs first
2. Refer to `docs/architecture/ARCHITECTURE.md` for system-wide design
3. Look at existing modules for patterns
4. When in doubt, follow the boundary rules above
