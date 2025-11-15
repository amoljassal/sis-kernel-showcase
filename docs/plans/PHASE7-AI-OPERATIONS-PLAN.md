# Phase 7: AI Operations Platform - Implementation Plan

**Status:** PLANNING
**Target:** P0 - Production AI/ML Operations (2-4 weeks)
**Dependencies:** Phase 4 Production Readiness COMPLETE ✅

---

## Executive Summary

Phase 7 transforms SIS Kernel from "AI-ready OS" to "AI-operable OS at scale" by implementing enterprise-grade model lifecycle management, decision observability, canary deployments, and OpenTelemetry integration.

**P0 Deliverables (2-4 weeks):**
1. Model registry with atomic hot-swap and rollback
2. Decision-trace ring buffer with incident bundle export
3. Shadow/canary deployment mode with automatic rollback
4. OpenTelemetry exporter with drift detection
5. Build system improvements and cleanup

**Key Architectural Principle:**
> Keep inference out of the kernel where feasible. Kernel provides deterministic hooks (buffers, metrics, policy gates) while agent/model complexity lives in hardened userspace service.

---

## Build Environment Requirements

### Critical: Toolchain Compatibility

**Learned from Phase 4 implementation:** Bootloader and Rust version compatibility is critical. Specify exact versions.

#### Rust Toolchain (MUST USE)
```toml
# rust-toolchain.toml (already exists, DO NOT MODIFY)
[toolchain]
channel = "nightly-2025-01-15"
components = ["rust-src", "rustfmt", "clippy"]
targets = ["aarch64-unknown-none", "aarch64-unknown-uefi"]
profile = "minimal"
```

**Version:** `rustc 1.91.0-nightly (6c699a372 2025-09-05)` (managed by rust-toolchain.toml)
**Why pinned:** Newer nightlies have breaking changes in target spec format
**DO NOT:** Attempt to use latest nightly without testing bootloader compatibility

#### Bootloader Version (CURRENT)
```toml
# crates/kernel/Cargo.toml
[dependencies]
bootloader_api = { version = "0.11.12", default-features = false }

[build-dependencies]
bootloader = { version = "0.11.12", default-features = false, features = ["bios"] }
bootloader_api = { version = "0.11.12" }
```

**Version:** `0.11.12` (DO NOT upgrade to 0.12+ - does not exist yet)
**Why this version:** Compatible with Rust 2021 edition and nightly-2025-01-15
**Known issue:** Bootloader 0.11.11 has target-pointer-width incompatibility with modern Rust

#### Build System (build.rs)
**Current approach:** Environment variable-based (no generated file dependencies)
```rust
// build.rs exports via env vars only:
println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);
println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);

// build_info.rs reads with option_env!()
pub const GIT_COMMIT: &str = option_env!("GIT_COMMIT").unwrap_or("unknown");
```

**Why:** Eliminates Rust 2021 string literal errors with `include!()` macro
**DO NOT:** Attempt to use `include!(concat!(env!("OUT_DIR"), "/build_info.rs"))` pattern

#### Required Dependencies (Already in Cargo.toml)
```toml
# Phase 4 dependencies (DO NOT REMOVE)
sha2 = { version = "0.10", default-features = false, features = ["force-soft"] }
ed25519-dalek = { version = "2", default-features = false, features = ["alloc"] }
signature = { version = "2", default-features = false }
heapless = { version = "0.8", default-features = false }
libm = { version = "0.2", default-features = false }
```

**New dependencies for Phase 7:**
```toml
# Add to crates/kernel/Cargo.toml [dependencies]
serde = { version = "1.0", default-features = false, features = ["derive", "alloc"] }
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }
```

**Why serde/serde_json:** Model registry metadata, decision trace serialization, incident bundle export

---

## Architecture Overview

### Component Layout

```
┌─────────────────────────────────────────────────────────────┐
│                     SIS Kernel (no_std)                     │
├─────────────────────────────────────────────────────────────┤
│  Model Lifecycle          Decision Traces      Shadow Mode  │
│  ┌──────────────┐        ┌──────────────┐    ┌──────────┐  │
│  │ ModelRegistry│        │ TraceBuffer  │    │ ShadowCtl│  │
│  │ (ext4-backed)│        │ (ring)       │    │          │  │
│  │              │        │              │    │          │  │
│  │ - load()     │        │ - record()   │    │ - enable │  │
│  │ - swap()     │        │ - export()   │    │ - promote│  │
│  │ - rollback() │        │ - bundle()   │    │ - compare│  │
│  └──────────────┘        └──────────────┘    └──────────┘  │
├─────────────────────────────────────────────────────────────┤
│  OpenTelemetry Exporter      Drift Detection                │
│  ┌──────────────┐            ┌──────────────┐              │
│  │ OTelExporter │            │ DriftMonitor │              │
│  │              │            │              │              │
│  │ - span()     │            │ - check()    │              │
│  │ - trace()    │            │ - alert()    │              │
│  └──────────────┘            └──────────────┘              │
└─────────────────────────────────────────────────────────────┘

        ▲ syscalls                    ▲ metrics/traces
        │                             │
        ▼                             ▼
┌─────────────────────────────────────────────────────────────┐
│              Agent Daemon (userspace, future)               │
│  - Heavyweight inference                                    │
│  - TensorFlow/PyTorch integration (P2)                      │
│  - Lives outside kernel TCB                                 │
└─────────────────────────────────────────────────────────────┘
```

### File Structure (New Files)

```
crates/kernel/src/
├── model/
│   ├── mod.rs                  # Model module root
│   ├── registry.rs             # Model registry (NEW - ~400 lines)
│   ├── lifecycle.rs            # Load/swap/rollback (NEW - ~300 lines)
│   ├── health.rs               # Health checks (NEW - ~150 lines)
│   └── signature.rs            # Extend existing crypto-real
├── trace/
│   ├── mod.rs                  # Trace module root
│   ├── decision.rs             # DecisionTrace struct (NEW - ~200 lines)
│   ├── buffer.rs               # Ring buffer for traces (NEW - ~250 lines)
│   └── export.rs               # Bundle export to ext4 (NEW - ~200 lines)
├── shadow/
│   ├── mod.rs                  # Shadow agent module
│   ├── agent.rs                # Shadow agent logic (NEW - ~300 lines)
│   ├── compare.rs              # Decision comparison (NEW - ~200 lines)
│   └── rollback.rs             # Automatic rollback (NEW - ~150 lines)
├── otel/
│   ├── mod.rs                  # OpenTelemetry module
│   ├── exporter.rs             # OTel span/trace export (NEW - ~350 lines)
│   └── drift.rs                # Drift detection (NEW - ~200 lines)
└── shell/
    ├── shell_modelctl.rs       # Model control commands (NEW - ~250 lines)
    ├── shell_shadowctl.rs      # Shadow control commands (NEW - ~200 lines)
    └── shell_tracectl.rs       # Trace control commands (NEW - ~150 lines)

docs/
└── PHASE7-AI-OPERATIONS.md     # Implementation docs (NEW - ~800 lines)

tests/
├── model/
│   ├── test_lifecycle.sh       # Model swap/rollback tests (NEW)
│   └── test_health.sh          # Health check tests (NEW)
└── shadow/
    ├── test_canary.sh          # Canary deployment tests (NEW)
    └── test_rollback.sh        # Automatic rollback tests (NEW)
```

**Estimated new code:** ~3,000 lines across 20+ files

---

## Phase 7.1: Model Registry and Lifecycle (Week 1)

### 7.1.1 Model Registry Structure

**File:** `crates/kernel/src/model/registry.rs`

**Ext4 Directory Structure:**
```
/models/                        # Root registry (ext4-backed)
├── registry.json               # Model metadata
├── active -> v1.2.3/          # Symlink for atomic swap
├── shadow -> v1.3.0/          # Shadow model (canary)
├── v1.2.3/                    # Production model
│   ├── model.bin              # Model weights
│   ├── model.sig              # Ed25519 signature
│   ├── manifest.json          # Model metadata
│   └── health.json            # Health check results
├── v1.3.0/                    # Shadow/canary model
│   ├── model.bin
│   ├── model.sig
│   ├── manifest.json
│   └── health.json
└── rollback/                   # Last-known-good backup
    └── v1.2.2/
        ├── model.bin
        ├── model.sig
        └── manifest.json
```

**registry.json Schema:**
```json
{
  "models": [
    {
      "version": "v1.2.3",
      "hash": "sha256:abc123...",
      "signature": "ed25519:def456...",
      "status": "active",
      "loaded_at": 1762546874,
      "health": {
        "inference_latency_p99_us": 850,
        "memory_footprint_bytes": 8388608,
        "test_accuracy": 0.98
      }
    },
    {
      "version": "v1.3.0",
      "status": "shadow",
      "loaded_at": 1762550000,
      "divergence_count": 5
    }
  ],
  "active": "v1.2.3",
  "shadow": "v1.3.0",
  "rollback": "v1.2.2"
}
```

**Implementation:**

```rust
// crates/kernel/src/model/registry.rs

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::lib::error::{Result, Errno};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub version: String,
    pub hash: [u8; 32],              // SHA-256
    pub signature: [u8; 64],         // Ed25519
    pub status: ModelStatus,
    pub loaded_at: u64,              // UNIX timestamp
    pub health: Option<HealthMetrics>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModelStatus {
    Active,
    Shadow,
    Rollback,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub inference_latency_p99_us: u64,
    pub memory_footprint_bytes: usize,
    pub test_accuracy: f32,
}

#[derive(Debug)]
pub struct ModelRegistry {
    registry_path: &'static str,
    models: Vec<ModelMetadata>,
    active: Option<String>,
    shadow: Option<String>,
    rollback: Option<String>,
}

impl ModelRegistry {
    pub const REGISTRY_PATH: &'static str = "/models/registry.json";

    pub fn new() -> Self {
        Self {
            registry_path: Self::REGISTRY_PATH,
            models: Vec::new(),
            active: None,
            shadow: None,
            rollback: None,
        }
    }

    /// Load registry from ext4
    pub fn load(&mut self) -> Result<()> {
        // Read /models/registry.json via VFS
        // Parse with serde_json
        // Populate self.models
        todo!("Implement ext4 read + JSON parse")
    }

    /// Save registry to ext4 (journaled)
    pub fn save(&self) -> Result<()> {
        // Serialize to JSON with serde_json
        // Write to /models/registry.json via VFS
        // ext4 journal ensures atomicity
        todo!("Implement JSON serialize + ext4 write")
    }

    /// List all models
    pub fn list(&self) -> &[ModelMetadata] {
        &self.models
    }

    /// Get active model
    pub fn active(&self) -> Option<&ModelMetadata> {
        self.active.as_ref().and_then(|v| {
            self.models.iter().find(|m| &m.version == v)
        })
    }

    /// Get shadow model
    pub fn shadow(&self) -> Option<&ModelMetadata> {
        self.shadow.as_ref().and_then(|v| {
            self.models.iter().find(|m| &m.version == v)
        })
    }
}
```

### 7.1.2 Model Lifecycle Operations

**File:** `crates/kernel/src/model/lifecycle.rs`

**Atomic Hot-Swap Algorithm:**
1. Load new model to `/models/v1.3.0/`
2. Verify signature (SHA-256 + Ed25519)
3. Run health checks
4. If healthy: Atomically update symlink `/models/active -> v1.3.0/`
5. Save old active to `/models/rollback/`
6. Update registry.json

**RCU-Style Double Buffer:**
```rust
// crates/kernel/src/model/lifecycle.rs

use spin::Mutex;
use alloc::sync::Arc;

pub struct ModelLifecycle {
    active_model: Arc<Mutex<Option<Model>>>,
    shadow_model: Arc<Mutex<Option<Model>>>,
    registry: Arc<Mutex<ModelRegistry>>,
}

impl ModelLifecycle {
    /// Load and verify model from disk
    pub fn load_model(&self, version: &str) -> Result<Model> {
        let path = format!("/models/{}/model.bin", version);

        // 1. Read model file via VFS
        let model_data = self.read_file(&path)?;

        // 2. Verify signature
        self.verify_signature(&model_data, version)?;

        // 3. Deserialize model
        let model = Model::from_bytes(&model_data)?;

        Ok(model)
    }

    /// Atomic hot-swap with RCU
    pub fn swap_model(&mut self, new_version: &str) -> Result<()> {
        // 1. Load new model
        let new_model = self.load_model(new_version)?;

        // 2. Health check
        let health = self.health_check(&new_model)?;
        if !health.is_healthy() {
            return Err(Errno::EINVAL);
        }

        // 3. RCU swap (readers can still access old model)
        let old_model = {
            let mut active = self.active_model.lock();
            active.replace(new_model)
        };

        // 4. Update registry (ext4 journaled)
        {
            let mut reg = self.registry.lock();
            reg.set_active(new_version);
            if let Some(old) = old_model {
                reg.set_rollback(&old.version);
            }
            reg.save()?;
        }

        // 5. Update symlink atomically
        self.update_symlink("active", new_version)?;

        Ok(())
    }

    /// Rollback to last known good
    pub fn rollback(&mut self) -> Result<()> {
        let rollback_version = {
            let reg = self.registry.lock();
            reg.rollback.clone().ok_or(Errno::ENOENT)?
        };

        self.swap_model(&rollback_version)
    }

    fn verify_signature(&self, data: &[u8], version: &str) -> Result<()> {
        // Use existing crypto-real infrastructure
        // Read /models/{version}/model.sig
        // Verify with Ed25519
        todo!("Use existing signature verification from model.rs")
    }

    fn health_check(&self, model: &Model) -> Result<HealthMetrics> {
        // See section 7.1.3
        todo!("Run health checks")
    }

    fn update_symlink(&self, link_name: &str, target: &str) -> Result<()> {
        // Use VFS to update symlink atomically
        // ln -sf /models/{target} /models/{link_name}
        todo!("Implement via VFS symlink API")
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub version: String,
    pub weights: Vec<f32>,  // Simplified
    pub loaded_at: u64,
}

impl Model {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // Deserialize model from binary format
        todo!("Implement model deserialization")
    }
}
```

### 7.1.3 Model Health Checks

**File:** `crates/kernel/src/model/health.rs`

**Health Check Requirements:**
1. **Inference Latency P99:** Must be < 1ms (critical for 500ms autonomous tick)
2. **Memory Footprint:** Must be < 10MB (kernel heap is 8MB)
3. **Test Accuracy:** Must be > 95% on known test inputs

```rust
// crates/kernel/src/model/health.rs

pub struct HealthChecker {
    test_inputs: Vec<TestCase>,
}

#[derive(Debug)]
pub struct TestCase {
    pub input: Vec<f32>,
    pub expected_output: usize,
}

impl HealthChecker {
    pub fn check(&self, model: &Model) -> Result<HealthMetrics> {
        // 1. Latency test
        let latency_p99 = self.measure_latency_p99(model)?;
        if latency_p99 > 1000 {  // 1ms in microseconds
            return Err(Errno::ETIMEDOUT);
        }

        // 2. Memory test
        let mem_footprint = self.measure_memory(model)?;
        if mem_footprint > 10 * 1024 * 1024 {  // 10MB
            return Err(Errno::ENOMEM);
        }

        // 3. Accuracy test
        let accuracy = self.measure_accuracy(model)?;
        if accuracy < 0.95 {
            return Err(Errno::EINVAL);
        }

        Ok(HealthMetrics {
            inference_latency_p99_us: latency_p99,
            memory_footprint_bytes: mem_footprint,
            test_accuracy: accuracy,
        })
    }

    fn measure_latency_p99(&self, model: &Model) -> Result<u64> {
        let mut latencies = Vec::with_capacity(100);

        for _ in 0..100 {
            let start = crate::time::get_timestamp_us();
            let _ = model.predict(&self.test_inputs[0].input);
            let end = crate::time::get_timestamp_us();
            latencies.push(end - start);
        }

        latencies.sort_unstable();
        Ok(latencies[99])  // P99
    }

    fn measure_memory(&self, model: &Model) -> Result<usize> {
        // Size of model weights + metadata
        Ok(model.weights.len() * core::mem::size_of::<f32>() +
           core::mem::size_of::<Model>())
    }

    fn measure_accuracy(&self, model: &Model) -> Result<f32> {
        let mut correct = 0;
        for test_case in &self.test_inputs {
            let output = model.predict(&test_case.input);
            let predicted = output.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(idx, _)| idx)
                .unwrap();

            if predicted == test_case.expected_output {
                correct += 1;
            }
        }

        Ok(correct as f32 / self.test_inputs.len() as f32)
    }
}
```

### 7.1.4 Shell Commands - modelctl

**File:** `crates/kernel/src/shell/shell_modelctl.rs`

**Commands:**
```bash
modelctl list                    # List all models
modelctl load <version>          # Load model (no activation)
modelctl swap <version>          # Hot-swap to new model
modelctl rollback                # Rollback to last known good
modelctl health [version]        # Run health checks
modelctl status                  # Show active/shadow/rollback
modelctl remove <version>        # Remove model from registry
```

**Implementation:**
```rust
// crates/kernel/src/shell/shell_modelctl.rs

pub fn cmd_modelctl(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        return cmd_modelctl_status();
    }

    match args[0] {
        "list" => cmd_modelctl_list(),
        "load" => cmd_modelctl_load(args.get(1).ok_or(Errno::EINVAL)?),
        "swap" => cmd_modelctl_swap(args.get(1).ok_or(Errno::EINVAL)?),
        "rollback" => cmd_modelctl_rollback(),
        "health" => cmd_modelctl_health(args.get(1)),
        "status" => cmd_modelctl_status(),
        "remove" => cmd_modelctl_remove(args.get(1).ok_or(Errno::EINVAL)?),
        _ => {
            println!("Unknown modelctl command: {}", args[0]);
            Err(Errno::EINVAL)
        }
    }
}

fn cmd_modelctl_list() -> Result<()> {
    let registry = MODEL_REGISTRY.lock();

    println!("Model Registry:");
    println!("  Version      Status     Loaded At           Health");
    println!("  -------      ------     ---------           ------");

    for model in registry.list() {
        let health_str = if let Some(h) = &model.health {
            format!("P99={}μs Mem={}KB Acc={:.2}%",
                h.inference_latency_p99_us,
                h.memory_footprint_bytes / 1024,
                h.test_accuracy * 100.0)
        } else {
            "Not checked".to_string()
        };

        println!("  {:12} {:10} {:19} {}",
            model.version,
            format!("{:?}", model.status),
            model.loaded_at,
            health_str);
    }

    Ok(())
}

fn cmd_modelctl_swap(version: &str) -> Result<()> {
    println!("Swapping to model version: {}", version);

    let mut lifecycle = MODEL_LIFECYCLE.lock();
    lifecycle.swap_model(version)?;

    println!("Model swap complete: {}", version);
    println!("Previous model saved to rollback");

    Ok(())
}

fn cmd_modelctl_rollback() -> Result<()> {
    println!("Rolling back to last known good model...");

    let mut lifecycle = MODEL_LIFECYCLE.lock();
    lifecycle.rollback()?;

    println!("Rollback complete");

    Ok(())
}
```

**Shell integration:**
```rust
// Add to crates/kernel/src/shell.rs
mod shell_modelctl;

// In command dispatch
"modelctl" => shell_modelctl::cmd_modelctl(&args[1..]),
```

---

## Phase 7.2: Decision Traces and Export (Week 2)

### 7.2.1 Decision Trace Structure

**File:** `crates/kernel/src/trace/decision.rs`

**Trace Schema:**
```rust
use alloc::vec::Vec;
use alloc::string::String;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTrace {
    // Context
    pub trace_id: u64,              // Unique trace ID
    pub timestamp_us: u64,           // UNIX microseconds
    pub model_version: String,       // "v1.2.3"
    pub model_hash: [u8; 32],       // SHA-256 of model

    // Inputs
    pub telemetry: Telemetry,
    pub features: Vec<f32>,          // Extracted features
    pub system_state: SystemState,

    // Processing
    pub hidden_activations: Vec<Vec<f32>>,  // NN layer outputs
    pub policy_checks: Vec<PolicyCheck>,     // Safety checks

    // Outputs
    pub predictions: Vec<f32>,       // All output neurons
    pub chosen_action: usize,        // Index of chosen action
    pub confidence: u32,             // 0-1000
    pub alternatives: Vec<Alternative>,  // Top 3 alternatives

    // Outcome
    pub was_executed: bool,
    pub was_overridden: bool,
    pub override_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    pub mem_pressure: u32,
    pub deadline_misses: u32,
    pub cpu_usage: u32,
    pub network_latency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub uptime_ms: u64,
    pub heap_used: usize,
    pub processes_running: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheck {
    pub check_name: String,
    pub passed: bool,
    pub value: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub action_idx: usize,
    pub confidence: u32,
}
```

### 7.2.2 Trace Ring Buffer

**File:** `crates/kernel/src/trace/buffer.rs`

**Extend existing RingBuffer from printk.rs:**
```rust
use crate::lib::ringbuf::RingBuffer;
use spin::Mutex;

pub struct TraceBuffer {
    buffer: Mutex<RingBuffer<DecisionTrace, 1024>>,  // Last 1024 decisions
}

impl TraceBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: Mutex::new(RingBuffer::new()),
        }
    }

    pub fn record(&self, trace: DecisionTrace) {
        let mut buffer = self.buffer.lock();
        buffer.push(trace);
    }

    pub fn get_last_n(&self, n: usize) -> Vec<DecisionTrace> {
        let buffer = self.buffer.lock();
        buffer.iter().rev().take(n).cloned().collect()
    }

    pub fn find_by_trace_id(&self, trace_id: u64) -> Option<DecisionTrace> {
        let buffer = self.buffer.lock();
        buffer.iter().find(|t| t.trace_id == trace_id).cloned()
    }

    pub fn drain_all(&self) -> Vec<DecisionTrace> {
        let mut buffer = self.buffer.lock();
        buffer.drain_all()
    }
}

// Global trace buffer
pub static TRACE_BUFFER: TraceBuffer = TraceBuffer::new();
```

**Integration with autonomous agent:**
```rust
// In crates/kernel/src/autonomy.rs - autonomous_decision_tick()

pub fn autonomous_decision_tick() {
    // Existing code...
    let telemetry = gather_telemetry();
    let prediction = meta_agent.predict(&telemetry);

    // NEW: Record decision trace
    let trace = DecisionTrace {
        trace_id: generate_trace_id(),
        timestamp_us: crate::time::get_timestamp_us(),
        model_version: get_active_model_version(),
        model_hash: get_active_model_hash(),
        telemetry: telemetry.clone(),
        features: extract_features(&telemetry),
        system_state: gather_system_state(),
        hidden_activations: meta_agent.get_hidden_states(),
        policy_checks: run_policy_checks(&prediction),
        predictions: prediction.all_outputs.clone(),
        chosen_action: prediction.action,
        confidence: prediction.confidence,
        alternatives: get_top_alternatives(&prediction, 3),
        was_executed: true,  // Will be updated
        was_overridden: false,
        override_reason: None,
    };

    TRACE_BUFFER.record(trace);

    // Existing execution code...
}
```

### 7.2.3 Incident Bundle Export

**File:** `crates/kernel/src/trace/export.rs`

**Bundle Format:**
```json
{
  "incident_id": "INC-20250115-001",
  "exported_at": 1762550000,
  "traces": [
    { /* DecisionTrace */ },
    { /* DecisionTrace */ },
    { /* DecisionTrace */ }
  ],
  "model_info": {
    "version": "v1.2.3",
    "hash": "sha256:abc123...",
    "loaded_at": 1762546874
  },
  "system_snapshot": {
    "uptime_ms": 3600000,
    "heap_stats": { "allocated": 2097152, "peak": 3145728 },
    "logs": [ /* Last 100 log entries */ ]
  },
  "config": {
    "features": "bringup,llm,crypto-real",
    "git_commit": "6c4b7bbe",
    "build_timestamp": 1762546874
  }
}
```

**Implementation:**
```rust
use serde_json;

pub struct IncidentExporter {
    export_dir: &'static str,
}

impl IncidentExporter {
    pub const EXPORT_DIR: &'static str = "/incidents";

    pub fn new() -> Self {
        Self {
            export_dir: Self::EXPORT_DIR,
        }
    }

    pub fn export_bundle(&self, trace_ids: &[u64]) -> Result<String> {
        // 1. Gather traces
        let traces: Vec<DecisionTrace> = trace_ids.iter()
            .filter_map(|id| TRACE_BUFFER.find_by_trace_id(*id))
            .collect();

        if traces.is_empty() {
            return Err(Errno::ENOENT);
        }

        // 2. Build bundle
        let bundle = IncidentBundle {
            incident_id: self.generate_incident_id(),
            exported_at: crate::time::get_timestamp_us(),
            traces,
            model_info: self.get_model_info()?,
            system_snapshot: self.get_system_snapshot()?,
            config: self.get_config(),
        };

        // 3. Serialize to JSON
        let json = serde_json::to_string_pretty(&bundle)
            .map_err(|_| Errno::EINVAL)?;

        // 4. Write to ext4
        let filename = format!("{}/{}.json",
            self.export_dir, bundle.incident_id);
        self.write_file(&filename, json.as_bytes())?;

        Ok(filename)
    }

    fn generate_incident_id(&self) -> String {
        let timestamp = crate::time::get_timestamp_us();
        let date = timestamp / 1_000_000 / 86400;  // Days since epoch
        let counter = self.get_next_counter();
        format!("INC-{}-{:03}", date, counter)
    }

    fn get_model_info(&self) -> Result<ModelInfo> {
        let registry = MODEL_REGISTRY.lock();
        let active = registry.active().ok_or(Errno::ENOENT)?;
        Ok(ModelInfo {
            version: active.version.clone(),
            hash: active.hash,
            loaded_at: active.loaded_at,
        })
    }

    fn get_system_snapshot(&self) -> Result<SystemSnapshot> {
        Ok(SystemSnapshot {
            uptime_ms: crate::time::uptime_ms(),
            heap_stats: crate::heap::get_heap_stats(),
            logs: crate::lib::printk::KERNEL_LOG.drain_all(),
        })
    }

    fn get_config(&self) -> ConfigInfo {
        ConfigInfo {
            features: crate::build_info::FEATURES.to_string(),
            git_commit: crate::build_info::GIT_COMMIT.to_string(),
            build_timestamp: crate::build_info::BUILD_TIMESTAMP,
        }
    }
}

#[derive(Serialize)]
struct IncidentBundle {
    incident_id: String,
    exported_at: u64,
    traces: Vec<DecisionTrace>,
    model_info: ModelInfo,
    system_snapshot: SystemSnapshot,
    config: ConfigInfo,
}
```

### 7.2.4 Shell Commands - tracectl

**File:** `crates/kernel/src/shell/shell_tracectl.rs`

**Commands:**
```bash
tracectl list [--last N]         # List recent traces
tracectl show <trace_id>         # Show detailed trace
tracectl export <trace_id...>    # Export incident bundle
tracectl clear                   # Clear trace buffer
tracectl stats                   # Show trace statistics
```

**Implementation:**
```rust
pub fn cmd_tracectl(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        return cmd_tracectl_stats();
    }

    match args[0] {
        "list" => cmd_tracectl_list(args.get(1)),
        "show" => cmd_tracectl_show(args.get(1).ok_or(Errno::EINVAL)?),
        "export" => cmd_tracectl_export(&args[1..]),
        "clear" => cmd_tracectl_clear(),
        "stats" => cmd_tracectl_stats(),
        _ => Err(Errno::EINVAL),
    }
}

fn cmd_tracectl_list(count_arg: Option<&str>) -> Result<()> {
    let count = count_arg
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    let traces = TRACE_BUFFER.get_last_n(count);

    println!("Recent Decision Traces (last {}):", count);
    println!("  Trace ID   Timestamp         Model    Action  Conf  Executed");
    println!("  --------   ---------         -----    ------  ----  --------");

    for trace in traces {
        println!("  {:8}   {:16}  {:8}  {:6}  {:4}  {}",
            trace.trace_id,
            trace.timestamp_us,
            trace.model_version,
            trace.chosen_action,
            trace.confidence,
            if trace.was_executed { "YES" } else { "NO" });
    }

    Ok(())
}

fn cmd_tracectl_export(trace_ids_str: &[&str]) -> Result<()> {
    if trace_ids_str.is_empty() {
        println!("Usage: tracectl export <trace_id> [trace_id...]");
        return Err(Errno::EINVAL);
    }

    let trace_ids: Vec<u64> = trace_ids_str.iter()
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    let exporter = IncidentExporter::new();
    let filename = exporter.export_bundle(&trace_ids)?;

    println!("Incident bundle exported to: {}", filename);

    Ok(())
}
```

---

## Phase 7.3: Shadow/Canary Mode (Week 3)

### 7.3.1 Shadow Agent Architecture

**File:** `crates/kernel/src/shadow/agent.rs`

**Shadow Mode States:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowMode {
    Disabled,          // No shadow agent
    LogOnly,           // Shadow runs, logs only
    Compare,           // Shadow runs, compares with production
    CanaryPartial,     // Shadow executes for 10% of decisions
    CanaryFull,        // Shadow promoted to production
}
```

**Shadow Agent:**
```rust
use spin::Mutex;
use alloc::sync::Arc;

pub struct ShadowAgent {
    mode: Mutex<ShadowMode>,
    model: Arc<Mutex<Option<Model>>>,
    divergence_count: AtomicU64,
    divergence_threshold: u32,
    decision_count: AtomicU64,
}

impl ShadowAgent {
    pub fn new() -> Self {
        Self {
            mode: Mutex::new(ShadowMode::Disabled),
            model: Arc::new(Mutex::new(None)),
            divergence_count: AtomicU64::new(0),
            divergence_threshold: 50,  // Rollback after 50 divergences
            decision_count: AtomicU64::new(0),
        }
    }

    pub fn enable(&self, model: Model, mode: ShadowMode) -> Result<()> {
        *self.model.lock() = Some(model);
        *self.mode.lock() = mode;
        self.divergence_count.store(0, Ordering::Relaxed);
        self.decision_count.store(0, Ordering::Relaxed);
        Ok(())
    }

    pub fn disable(&self) {
        *self.mode.lock() = ShadowMode::Disabled;
        *self.model.lock() = None;
    }

    /// Run shadow prediction (parallel to production)
    pub fn predict_shadow(&self, telemetry: &Telemetry) -> Option<Prediction> {
        let mode = *self.mode.lock();
        if mode == ShadowMode::Disabled {
            return None;
        }

        let model = self.model.lock();
        let model = model.as_ref()?;

        // Run shadow prediction
        let prediction = model.predict(telemetry);

        self.decision_count.fetch_add(1, Ordering::Relaxed);

        Some(prediction)
    }

    /// Compare shadow with production prediction
    pub fn compare(&self, prod: &Prediction, shadow: &Prediction) -> ComparisonResult {
        let confidence_delta = (prod.confidence as i32 - shadow.confidence as i32).abs() as u32;
        let action_matches = prod.action == shadow.action;

        let diverged = !action_matches || confidence_delta > 200;  // 20% confidence delta

        if diverged {
            let div_count = self.divergence_count.fetch_add(1, Ordering::Relaxed) + 1;

            // Check if we should rollback
            if div_count >= self.divergence_threshold as u64 {
                return ComparisonResult::Rollback;
            }
        }

        ComparisonResult::Ok {
            diverged,
            confidence_delta,
            action_matches,
        }
    }

    pub fn get_stats(&self) -> ShadowStats {
        ShadowStats {
            mode: *self.mode.lock(),
            decision_count: self.decision_count.load(Ordering::Relaxed),
            divergence_count: self.divergence_count.load(Ordering::Relaxed),
            divergence_rate: self.get_divergence_rate(),
        }
    }

    fn get_divergence_rate(&self) -> f32 {
        let total = self.decision_count.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let divs = self.divergence_count.load(Ordering::Relaxed);
        (divs as f32 / total as f32) * 100.0
    }
}

#[derive(Debug)]
pub enum ComparisonResult {
    Ok {
        diverged: bool,
        confidence_delta: u32,
        action_matches: bool,
    },
    Rollback,
}

#[derive(Debug)]
pub struct ShadowStats {
    pub mode: ShadowMode,
    pub decision_count: u64,
    pub divergence_count: u64,
    pub divergence_rate: f32,
}

// Global shadow agent
pub static SHADOW_AGENT: ShadowAgent = ShadowAgent::new();
```

### 7.3.2 Automatic Rollback Logic

**File:** `crates/kernel/src/shadow/rollback.rs`

**Rollback Triggers:**
```rust
pub struct RollbackTrigger {
    divergence_threshold: u32,     // Max divergences before rollback
    confidence_drop_threshold: u32, // Max confidence drop
    error_rate_threshold: f32,     // Max error rate (%)
}

impl RollbackTrigger {
    pub fn check(&self, stats: &ShadowStats) -> RollbackDecision {
        // 1. Check divergence rate
        if stats.divergence_count >= self.divergence_threshold as u64 {
            return RollbackDecision::Rollback {
                reason: "Divergence threshold exceeded".into(),
                metric: format!("{} divergences", stats.divergence_count),
            };
        }

        // 2. Check divergence rate
        if stats.divergence_rate > self.error_rate_threshold {
            return RollbackDecision::Rollback {
                reason: "Divergence rate too high".into(),
                metric: format!("{:.2}%", stats.divergence_rate),
            };
        }

        // 3. All checks passed
        RollbackDecision::Continue
    }
}

#[derive(Debug)]
pub enum RollbackDecision {
    Continue,
    Rollback {
        reason: String,
        metric: String,
    },
}

/// Automatic rollback on trigger
pub fn auto_rollback_if_needed() -> Result<()> {
    let stats = SHADOW_AGENT.get_stats();
    let trigger = RollbackTrigger {
        divergence_threshold: 50,
        confidence_drop_threshold: 300,  // 30%
        error_rate_threshold: 20.0,      // 20%
    };

    match trigger.check(&stats) {
        RollbackDecision::Continue => Ok(()),
        RollbackDecision::Rollback { reason, metric } => {
            println!("[ROLLBACK] Triggered: {} ({})", reason, metric);

            // Disable shadow
            SHADOW_AGENT.disable();

            // Rollback model
            let mut lifecycle = MODEL_LIFECYCLE.lock();
            lifecycle.rollback()?;

            println!("[ROLLBACK] Complete - reverted to production model");

            Ok(())
        }
    }
}
```

### 7.3.3 Integration with Autonomous Agent

**Modified:** `crates/kernel/src/autonomy.rs`

```rust
pub fn autonomous_decision_tick() {
    let telemetry = gather_telemetry();

    // Production prediction
    let prod_prediction = meta_agent.predict(&telemetry);

    // Shadow prediction (if enabled)
    if let Some(shadow_prediction) = SHADOW_AGENT.predict_shadow(&telemetry) {
        // Compare predictions
        let comparison = SHADOW_AGENT.compare(&prod_prediction, &shadow_prediction);

        match comparison {
            ComparisonResult::Ok { diverged, confidence_delta, action_matches } => {
                // Log comparison
                log_kv!("SHADOW", LogLevel::Info,
                    "diverged" => if diverged { "true" } else { "false" },
                    "conf_delta" => &format!("{}", confidence_delta),
                    "action_match" => if action_matches { "true" } else { "false" }
                );

                // In canary mode, use shadow prediction sometimes
                let mode = *SHADOW_AGENT.mode.lock();
                if mode == ShadowMode::CanaryPartial {
                    // 10% canary traffic
                    if (telemetry.timestamp_us % 10) == 0 {
                        execute_action(&shadow_prediction);
                        return;
                    }
                } else if mode == ShadowMode::CanaryFull {
                    execute_action(&shadow_prediction);
                    return;
                }
            }
            ComparisonResult::Rollback => {
                // Trigger automatic rollback
                auto_rollback_if_needed().ok();
            }
        }
    }

    // Execute production prediction
    execute_action(&prod_prediction);
}
```

### 7.3.4 Shell Commands - shadowctl

**File:** `crates/kernel/src/shell/shell_shadowctl.rs`

**Commands:**
```bash
shadowctl enable <version>       # Enable shadow with model version
shadowctl disable                # Disable shadow mode
shadowctl promote                # Promote shadow to production
shadowctl status                 # Show shadow statistics
shadowctl threshold <N>          # Set divergence threshold
shadowctl mode <MODE>            # Set shadow mode (log/compare/canary10/canary100)
```

**Implementation:**
```rust
pub fn cmd_shadowctl(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        return cmd_shadowctl_status();
    }

    match args[0] {
        "enable" => cmd_shadowctl_enable(args.get(1).ok_or(Errno::EINVAL)?),
        "disable" => cmd_shadowctl_disable(),
        "promote" => cmd_shadowctl_promote(),
        "status" => cmd_shadowctl_status(),
        "threshold" => cmd_shadowctl_threshold(args.get(1).ok_or(Errno::EINVAL)?),
        "mode" => cmd_shadowctl_mode(args.get(1).ok_or(Errno::EINVAL)?),
        _ => Err(Errno::EINVAL),
    }
}

fn cmd_shadowctl_enable(version: &str) -> Result<()> {
    println!("Enabling shadow agent with model: {}", version);

    // Load shadow model
    let lifecycle = MODEL_LIFECYCLE.lock();
    let shadow_model = lifecycle.load_model(version)?;

    // Enable shadow in LogOnly mode
    SHADOW_AGENT.enable(shadow_model, ShadowMode::LogOnly)?;

    println!("Shadow agent enabled (LogOnly mode)");
    println!("Use 'shadowctl mode compare' to enable comparison");

    Ok(())
}

fn cmd_shadowctl_promote() -> Result<()> {
    let stats = SHADOW_AGENT.get_stats();

    println!("Shadow Agent Statistics:");
    println!("  Decisions: {}", stats.decision_count);
    println!("  Divergences: {} ({:.2}%)",
        stats.divergence_count, stats.divergence_rate);

    if stats.divergence_rate > 10.0 {
        println!("\nWARNING: Divergence rate > 10%");
        println!("Are you sure you want to promote? (y/n)");
        // In real implementation, would wait for user confirmation
        return Err(Errno::EINVAL);
    }

    println!("\nPromoting shadow to production...");

    // Get shadow model version
    let shadow_model = SHADOW_AGENT.model.lock();
    let shadow_version = shadow_model.as_ref()
        .ok_or(Errno::ENOENT)?
        .version.clone();

    // Swap to shadow model
    let mut lifecycle = MODEL_LIFECYCLE.lock();
    lifecycle.swap_model(&shadow_version)?;

    // Disable shadow (it's now production)
    SHADOW_AGENT.disable();

    println!("Promotion complete: {} is now active", shadow_version);

    Ok(())
}

fn cmd_shadowctl_status() -> Result<()> {
    let stats = SHADOW_AGENT.get_stats();

    println!("Shadow Agent Status:");
    println!("  Mode: {:?}", stats.mode);
    println!("  Decisions: {}", stats.decision_count);
    println!("  Divergences: {} ({:.2}%)",
        stats.divergence_count, stats.divergence_rate);

    if stats.mode != ShadowMode::Disabled {
        let model = SHADOW_AGENT.model.lock();
        if let Some(m) = model.as_ref() {
            println!("  Shadow Model: {}", m.version);
        }
    }

    Ok(())
}
```

---

## Phase 7.4: OpenTelemetry and Drift Detection (Week 4)

### 7.4.1 OpenTelemetry Exporter

**File:** `crates/kernel/src/otel/exporter.rs`

**OTel Span Format:**
```rust
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OTelSpan {
    pub trace_id: String,           // Hex trace ID
    pub span_id: String,            // Hex span ID
    pub parent_span_id: Option<String>,
    pub name: String,               // "autonomous_decision"
    pub kind: SpanKind,
    pub start_time_us: u64,
    pub end_time_us: u64,
    pub attributes: Vec<Attribute>,
    pub events: Vec<Event>,
    pub status: SpanStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub key: String,
    pub value: AttributeValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub timestamp_us: u64,
    pub name: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error { message: String },
}
```

**Exporter Implementation:**
```rust
pub struct OTelExporter {
    export_endpoint: &'static str,
    batch_size: usize,
    spans: Mutex<Vec<OTelSpan>>,
}

impl OTelExporter {
    pub fn new() -> Self {
        Self {
            export_endpoint: "/otel/spans.json",
            batch_size: 100,
            spans: Mutex::new(Vec::new()),
        }
    }

    /// Create span from decision trace
    pub fn record_decision_span(&self, trace: &DecisionTrace) {
        let span = OTelSpan {
            trace_id: format!("{:016x}", trace.trace_id),
            span_id: format!("{:016x}", trace.trace_id),  // Same for root span
            parent_span_id: None,
            name: "autonomous_decision".into(),
            kind: SpanKind::Internal,
            start_time_us: trace.timestamp_us,
            end_time_us: trace.timestamp_us + 1000,  // Assume 1ms duration
            attributes: self.build_attributes(trace),
            events: self.build_events(trace),
            status: if trace.was_executed {
                SpanStatus::Ok
            } else {
                SpanStatus::Error {
                    message: trace.override_reason.clone()
                        .unwrap_or_else(|| "Not executed".into())
                }
            },
        };

        let mut spans = self.spans.lock();
        spans.push(span);

        // Export batch if full
        if spans.len() >= self.batch_size {
            self.flush_batch(&mut spans).ok();
        }
    }

    fn build_attributes(&self, trace: &DecisionTrace) -> Vec<Attribute> {
        vec![
            Attribute {
                key: "model.version".into(),
                value: AttributeValue::String(trace.model_version.clone()),
            },
            Attribute {
                key: "model.hash".into(),
                value: AttributeValue::String(format!("{:x}", trace.model_hash[0])),
            },
            Attribute {
                key: "action".into(),
                value: AttributeValue::Int(trace.chosen_action as i64),
            },
            Attribute {
                key: "confidence".into(),
                value: AttributeValue::Int(trace.confidence as i64),
            },
            Attribute {
                key: "mem_pressure".into(),
                value: AttributeValue::Int(trace.telemetry.mem_pressure as i64),
            },
            Attribute {
                key: "deadline_misses".into(),
                value: AttributeValue::Int(trace.telemetry.deadline_misses as i64),
            },
        ]
    }

    fn build_events(&self, trace: &DecisionTrace) -> Vec<Event> {
        let mut events = Vec::new();

        // Policy check events
        for check in &trace.policy_checks {
            events.push(Event {
                timestamp_us: trace.timestamp_us,
                name: format!("policy_check.{}", check.check_name),
                attributes: vec![
                    Attribute {
                        key: "passed".into(),
                        value: AttributeValue::Bool(check.passed),
                    },
                    Attribute {
                        key: "value".into(),
                        value: AttributeValue::Float(check.value as f64),
                    },
                ],
            });
        }

        events
    }

    fn flush_batch(&self, spans: &mut Vec<OTelSpan>) -> Result<()> {
        // Serialize to JSON
        let json = serde_json::to_string(&spans)
            .map_err(|_| Errno::EINVAL)?;

        // Write to ext4
        self.write_file(self.export_endpoint, json.as_bytes())?;

        // Clear batch
        spans.clear();

        Ok(())
    }

    fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        // Use VFS to write file
        todo!("Write via VFS")
    }
}

// Global exporter
pub static OTEL_EXPORTER: OTelExporter = OTelExporter::new();
```

**Integration:**
```rust
// In autonomous_decision_tick() after recording trace
OTEL_EXPORTER.record_decision_span(&trace);
```

### 7.4.2 Drift Detection

**File:** `crates/kernel/src/otel/drift.rs`

**Drift Metrics:**
```rust
pub struct DriftMonitor {
    confidence_baseline: f32,
    confidence_window: RingBuffer<u32, 100>,  // Last 100 confidences
    reward_baseline: f32,
    reward_window: RingBuffer<f32, 100>,
}

impl DriftMonitor {
    pub fn new() -> Self {
        Self {
            confidence_baseline: 800.0,  // 80% baseline
            confidence_window: RingBuffer::new(),
            reward_baseline: 0.5,
            reward_window: RingBuffer::new(),
        }
    }

    pub fn check_drift(&mut self, trace: &DecisionTrace) -> DriftStatus {
        // Add to window
        self.confidence_window.push(trace.confidence);

        // Calculate recent average
        let recent_avg = self.calculate_average(&self.confidence_window);

        // Check for drift
        let drift_delta = (recent_avg - self.confidence_baseline).abs();

        if drift_delta > 200.0 {  // 20% drift
            DriftStatus::Alert {
                metric: "confidence".into(),
                baseline: self.confidence_baseline,
                current: recent_avg,
                delta: drift_delta,
            }
        } else if drift_delta > 100.0 {  // 10% drift
            DriftStatus::Warning {
                metric: "confidence".into(),
                delta: drift_delta,
            }
        } else {
            DriftStatus::Ok
        }
    }

    fn calculate_average(&self, buffer: &RingBuffer<u32, 100>) -> f32 {
        let values: Vec<u32> = buffer.iter().cloned().collect();
        if values.is_empty() {
            return 0.0;
        }
        let sum: u32 = values.iter().sum();
        sum as f32 / values.len() as f32
    }
}

#[derive(Debug)]
pub enum DriftStatus {
    Ok,
    Warning {
        metric: String,
        delta: f32,
    },
    Alert {
        metric: String,
        baseline: f32,
        current: f32,
        delta: f32,
    },
}

// Global drift monitor
pub static DRIFT_MONITOR: Mutex<DriftMonitor> = Mutex::new(DriftMonitor::new());
```

**Automatic Actions on Drift:**
```rust
pub fn handle_drift(status: DriftStatus) {
    match status {
        DriftStatus::Ok => {}
        DriftStatus::Warning { metric, delta } => {
            log_kv!("DRIFT", LogLevel::Warn,
                "status" => "warning",
                "metric" => &metric,
                "delta" => &format!("{:.2}", delta)
            );
        }
        DriftStatus::Alert { metric, baseline, current, delta } => {
            log_kv!("DRIFT", LogLevel::Error,
                "status" => "alert",
                "metric" => &metric,
                "baseline" => &format!("{:.2}", baseline),
                "current" => &format!("{:.2}", current),
                "delta" => &format!("{:.2}", delta)
            );

            // Automatic action: Switch to safe mode
            // (Require approval for all actions)
            println!("[DRIFT ALERT] Switching to safe mode");
            // AUTONOMY_SAFE_MODE.store(true, Ordering::Relaxed);
        }
    }
}

// In autonomous_decision_tick()
let drift_status = DRIFT_MONITOR.lock().check_drift(&trace);
handle_drift(drift_status);
```

---

## Phase 7.5: Build System Improvements

### 7.5.1 Clean build.rs Warnings

**File:** `crates/kernel/build.rs`

**Current warnings to fix:**
```
warning: unused import: `std::fs`
warning: unused import: `std::path::Path`
warning: unused variable: `rust_version`
warning: unused variable: `features`
warning: unused variable: `profile`
warning: unused variable: `target`
warning: function `escape_json` is never used
warning: function `escape_rust` is never used
```

**Fix:**
```rust
// Remove unused imports
// use std::fs;  // REMOVE
// use std::path::Path;  // REMOVE

// Prefix unused variables with underscore
let _rust_version = get_rust_version();
let _features = get_enabled_features();
let _profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
let _target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

// Remove unused functions if truly not needed, or mark with #[allow(dead_code)]
#[allow(dead_code)]
fn escape_json(s: &str) -> String { /* ... */ }

#[allow(dead_code)]
fn escape_rust(s: &str) -> String { /* ... */ }
```

### 7.5.2 Gate Bootloader for Non-x86_64

**File:** `crates/kernel/Cargo.toml`

**Current (problematic):**
```toml
[build-dependencies]
bootloader = { version = "0.11.12", default-features = false, features = ["bios"] }
bootloader_api = { version = "0.11.12" }
```

**Fixed (gated):**
```toml
[build-dependencies]
bootloader_api = { version = "0.11.12" }
chrono = "0.4"

[target.'cfg(target_arch = "x86_64")'.build-dependencies]
bootloader = { version = "0.11.12", default-features = false, features = ["bios"] }
```

**Why:** AArch64 builds don't need x86 BIOS bootloader, reduces build warnings

### 7.5.3 Feature Flag for Phase 7

**File:** `crates/kernel/Cargo.toml`

**Add to [features]:**
```toml
[features]
# ... existing features ...

# Phase 7: AI Operations Platform
ai-ops = ["model-lifecycle", "shadow-mode", "otel"]  # Meta-feature
model-lifecycle = []  # Model registry, hot-swap, rollback
shadow-mode = []      # Shadow agent, canary deployment
otel = []             # OpenTelemetry exporter, drift detection
decision-traces = []  # Decision trace buffer and export
```

**Usage:**
```bash
# Enable all Phase 7 features
SIS_FEATURES="llm,crypto-real,chaos,ai-ops" BRINGUP=1 ./scripts/uefi_run.sh build

# Enable specific features
SIS_FEATURES="llm,crypto-real,model-lifecycle,shadow-mode" BRINGUP=1 ./scripts/uefi_run.sh build
```

---

## Testing Strategy

### Unit Tests

**File:** `crates/kernel/src/model/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_load_save() {
        let mut registry = ModelRegistry::new();
        // Test load/save round-trip
    }

    #[test]
    fn test_signature_verification() {
        // Test SHA-256 + Ed25519 verification
    }

    #[test]
    fn test_health_checks() {
        // Test latency, memory, accuracy checks
    }

    #[test]
    fn test_atomic_swap() {
        // Test RCU model swap
    }

    #[test]
    fn test_rollback() {
        // Test rollback to last known good
    }
}
```

### Integration Tests (Shell Scripts)

**File:** `tests/model/test_lifecycle.sh`

```bash
#!/usr/bin/env bash
# Model Lifecycle Integration Test

set -euo pipefail

echo "=== Model Lifecycle Test ==="

# 1. List models
echo "[TEST] Listing models..."
echo "modelctl list" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 2. Load test model
echo "[TEST] Loading model v1.0.0..."
echo "modelctl load v1.0.0" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 3. Health check
echo "[TEST] Running health check..."
echo "modelctl health v1.0.0" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 4. Swap model
echo "[TEST] Swapping to v1.0.0..."
echo "modelctl swap v1.0.0" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 5. Verify active
echo "[TEST] Verifying active model..."
echo "modelctl status" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 6. Rollback
echo "[TEST] Rolling back..."
echo "modelctl rollback" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

echo "=== Model Lifecycle Test PASSED ==="
```

**File:** `tests/shadow/test_canary.sh`

```bash
#!/usr/bin/env bash
# Shadow/Canary Integration Test

set -euo pipefail

echo "=== Shadow Agent Test ==="

# 1. Enable shadow
echo "[TEST] Enabling shadow agent..."
echo "shadowctl enable v1.1.0" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 2. Check status
echo "[TEST] Checking shadow status..."
echo "shadowctl status" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 3. Enable comparison
echo "[TEST] Enabling comparison mode..."
echo "shadowctl mode compare" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 4. Wait for 10 decisions
sleep 5

# 5. Check divergence
echo "[TEST] Checking divergence..."
echo "shadowctl status" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

# 6. If divergence < 10%, promote
echo "[TEST] Promoting shadow to production..."
echo "shadowctl promote" | ./scripts/qmp_input.py --socket $QMP_SOCK send-command

echo "=== Shadow Agent Test PASSED ==="
```

### Automated Test Harness

**File:** `scripts/phase7_tests.sh`

```bash
#!/usr/bin/env bash
# Phase 7 Automated Test Suite

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."

echo "========================================"
echo "Phase 7: AI Operations Platform Tests"
echo "========================================"

# Build with Phase 7 features
echo "[*] Building with ai-ops features..."
cd "$ROOT_DIR"
SIS_FEATURES="llm,crypto-real,chaos,ai-ops" BRINGUP=1 ./scripts/uefi_run.sh build

# Start QEMU with QMP
echo "[*] Starting QEMU with QMP..."
export QMP=1
export QMP_SOCK=/tmp/sis-phase7-qmp.sock
SIS_FEATURES="llm,crypto-real,chaos,ai-ops" BRINGUP=1 ./scripts/uefi_run.sh > /tmp/sis-phase7.log 2>&1 &
QEMU_PID=$!

# Wait for shell
sleep 10

# Run test suites
echo "[*] Running model lifecycle tests..."
./tests/model/test_lifecycle.sh

echo "[*] Running shadow agent tests..."
./tests/shadow/test_canary.sh

echo "[*] Running trace export tests..."
./tests/trace/test_export.sh

# Cleanup
echo "[*] Cleaning up..."
kill $QEMU_PID 2>/dev/null || true
rm -f $QMP_SOCK

echo "========================================"
echo "Phase 7 Tests: ALL PASSED"
echo "========================================"
```

---

## Documentation Deliverables

### 7.1 Implementation Guide

**File:** `docs/PHASE7-AI-OPERATIONS.md` (~800 lines)

Structure:
1. Architecture Overview
2. Model Registry Setup
3. Model Lifecycle Guide (load, swap, rollback)
4. Decision Trace Usage
5. Shadow/Canary Deployment Guide
6. OpenTelemetry Integration
7. Drift Detection Configuration
8. Troubleshooting Guide

### 7.2 Shell Command Reference

**File:** `docs/PHASE7-SHELL-COMMANDS.md`

Commands to document:
- `modelctl` (7 subcommands)
- `shadowctl` (6 subcommands)
- `tracectl` (5 subcommands)

### 7.3 API Reference

**File:** `docs/PHASE7-API-REFERENCE.md`

Rust APIs to document:
- `ModelRegistry`
- `ModelLifecycle`
- `TraceBuffer`
- `ShadowAgent`
- `OTelExporter`
- `DriftMonitor`

---

## Success Criteria

### P0 Acceptance Criteria

1. **Model Lifecycle:**
   - ✅ Load model from ext4 registry
   - ✅ Verify SHA-256 + Ed25519 signature
   - ✅ Run health checks (latency < 1ms, memory < 10MB, accuracy > 95%)
   - ✅ Atomic hot-swap via symlink
   - ✅ Rollback to last known good
   - ✅ Shell commands: `modelctl list/load/swap/rollback/health/status`

2. **Decision Traces:**
   - ✅ Record trace for every autonomous decision
   - ✅ Capture: inputs, processing, outputs, policy checks
   - ✅ Ring buffer with 1024 trace capacity
   - ✅ Export incident bundle to ext4 (JSON format)
   - ✅ Shell commands: `tracectl list/show/export/clear/stats`

3. **Shadow/Canary:**
   - ✅ Run shadow agent in parallel (log-only mode)
   - ✅ Compare shadow vs production predictions
   - ✅ Automatic rollback on divergence threshold (50 divergences or 20% rate)
   - ✅ Canary modes: 10% traffic, 100% traffic
   - ✅ Shell commands: `shadowctl enable/disable/promote/status/threshold/mode`

4. **OpenTelemetry:**
   - ✅ Export decision traces as OTel spans
   - ✅ Batch export to ext4 (JSON format)
   - ✅ Attributes: model version/hash, action, confidence, telemetry
   - ✅ Drift detection with baseline comparison
   - ✅ Automatic safe mode on drift alert (>20% drift)

5. **Build System:**
   - ✅ Clean build.rs warnings (0 warnings)
   - ✅ Gate bootloader for x86_64 only
   - ✅ Feature flags: `ai-ops`, `model-lifecycle`, `shadow-mode`, `otel`, `decision-traces`

### Performance Targets

- Model swap latency: < 100ms
- Health check latency: < 50ms
- Trace recording overhead: < 10μs per decision
- Shadow prediction overhead: < 500μs per decision
- OTel export batch time: < 10ms

### Zero Regressions

- All existing Phase 1-6 features work unchanged
- Autonomous agent continues to function with Phase 7 disabled
- Build succeeds on Rust nightly-2025-01-15 + bootloader 0.11.12

---

## Timeline and Milestones

### Week 1: Model Lifecycle (40-50 hours)
- Day 1-2: Model registry structure and ext4 integration
- Day 3: Signature verification (reuse crypto-real)
- Day 4: Health checks implementation
- Day 5: Atomic swap and rollback
- Day 6-7: Shell commands and testing

**Milestone 1:** `modelctl` commands functional, models can be loaded/swapped/rolled back

### Week 2: Decision Traces (30-40 hours)
- Day 1-2: DecisionTrace structure and ring buffer
- Day 3: Integration with autonomous agent
- Day 4: Incident bundle export
- Day 5: Shell commands
- Day 6-7: Testing and documentation

**Milestone 2:** All decisions traced, incident bundles exportable via `tracectl`

### Week 3: Shadow/Canary (40-50 hours)
- Day 1-2: Shadow agent infrastructure
- Day 3: Comparison logic and divergence detection
- Day 4: Automatic rollback triggers
- Day 5: Canary mode (10% / 100% traffic)
- Day 6-7: Shell commands and testing

**Milestone 3:** Shadow agent operational, canary deployments functional

### Week 4: OTel and Polish (20-30 hours)
- Day 1-2: OpenTelemetry exporter
- Day 3: Drift detection
- Day 4: Build system cleanup
- Day 5-7: Integration testing, documentation, polish

**Milestone 4:** Phase 7 complete, all P0 criteria met

---

## Implementation Notes for AI Agent

### Critical Build Requirements

1. **DO NOT** upgrade Rust toolchain - use rust-toolchain.toml as-is
2. **DO NOT** upgrade bootloader beyond 0.11.12
3. **DO NOT** use `include!()` macro for generated code - use env vars
4. **DO** add `serde` and `serde_json` dependencies with `no_std` + `alloc`
5. **DO** fix all build.rs warnings before submission

### Code Style Guidelines

1. Use existing patterns from Phase 4:
   - Structured logging with `log_kv!()` macro
   - Atomic operations for globals
   - Ring buffers for data structures
   - VFS for all file operations

2. Follow no_std conventions:
   - `alloc::string::String` not `std::string::String`
   - `alloc::vec::Vec` not `std::vec::Vec`
   - `core::fmt` not `std::fmt`

3. Error handling:
   - Use `Result<T>` from `crates/kernel/src/lib/error.rs`
   - Return appropriate `Errno` values
   - Never panic in production code paths

### Testing Requirements

1. Every new module needs unit tests (#[cfg(test)])
2. Every shell command needs integration test (.sh script)
3. Use existing QMP infrastructure for automated testing
4. Test both success and failure paths

### Documentation Requirements

1. Every public function needs /// doc comments
2. Every shell command needs usage examples
3. Every module needs module-level documentation
4. Follow existing doc style from Phase 4

---

## Risks and Mitigations

### Risk 1: Model Deserialization Format

**Risk:** Model binary format not specified
**Mitigation:** Start with simple Vec<f32> format, document extension point for future formats

### Risk 2: VFS Symlink Support

**Risk:** VFS may not support symlinks yet
**Mitigation:** Fall back to direct file path management if symlinks unavailable

### Risk 3: Serde in no_std

**Risk:** serde_json may have issues in no_std
**Mitigation:** Use `serde-json-core` crate if needed, or implement manual JSON serialization

### Risk 4: Memory Overhead

**Risk:** 1024 trace buffer may use too much memory
**Mitigation:** Make buffer size configurable via feature flag, start with 256

### Risk 5: Model Load Time

**Risk:** Model loading may block for too long
**Mitigation:** Implement async loading in future, for P0 accept blocking

---

## Post-P0 Roadmap (P1 and P2)

### P1 Features (4-6 weeks post-P0)
1. Human-in-the-loop override UI
2. Feedback annotation store
3. Adversarial test harness (extend chaos)
4. Long-horizon drift reports
5. SMP bring-up on PSCI targets

### P2 Features (8-12 weeks post-P0)
1. Simulation federation (fast-forward, multi-agent)
2. Compliance evidence generation (EU AI Act mapping)
3. Open research adapters (TensorFlow, RLlib plugins)
4. Micro-VM agent isolation (Firecracker-style)

---

## Appendix: File Checklist

### New Rust Files (Estimated Lines)
- [ ] `crates/kernel/src/model/mod.rs` (50 lines)
- [ ] `crates/kernel/src/model/registry.rs` (400 lines)
- [ ] `crates/kernel/src/model/lifecycle.rs` (300 lines)
- [ ] `crates/kernel/src/model/health.rs` (150 lines)
- [ ] `crates/kernel/src/trace/mod.rs` (50 lines)
- [ ] `crates/kernel/src/trace/decision.rs` (200 lines)
- [ ] `crates/kernel/src/trace/buffer.rs` (250 lines)
- [ ] `crates/kernel/src/trace/export.rs` (200 lines)
- [ ] `crates/kernel/src/shadow/mod.rs` (50 lines)
- [ ] `crates/kernel/src/shadow/agent.rs` (300 lines)
- [ ] `crates/kernel/src/shadow/compare.rs` (200 lines)
- [ ] `crates/kernel/src/shadow/rollback.rs` (150 lines)
- [ ] `crates/kernel/src/otel/mod.rs` (50 lines)
- [ ] `crates/kernel/src/otel/exporter.rs` (350 lines)
- [ ] `crates/kernel/src/otel/drift.rs` (200 lines)
- [ ] `crates/kernel/src/shell/shell_modelctl.rs` (250 lines)
- [ ] `crates/kernel/src/shell/shell_shadowctl.rs` (200 lines)
- [ ] `crates/kernel/src/shell/shell_tracectl.rs` (150 lines)

**Total: ~3,300 lines of new Rust code**

### New Test Files
- [ ] `tests/model/test_lifecycle.sh`
- [ ] `tests/model/test_health.sh`
- [ ] `tests/shadow/test_canary.sh`
- [ ] `tests/shadow/test_rollback.sh`
- [ ] `tests/trace/test_export.sh`
- [ ] `scripts/phase7_tests.sh`

### New Documentation Files
- [ ] `docs/PHASE7-AI-OPERATIONS.md` (800 lines)
- [ ] `docs/PHASE7-SHELL-COMMANDS.md` (300 lines)
- [ ] `docs/PHASE7-API-REFERENCE.md` (400 lines)

### Modified Files
- [ ] `crates/kernel/Cargo.toml` (add serde, feature flags, gate bootloader)
- [ ] `crates/kernel/build.rs` (fix warnings)
- [ ] `crates/kernel/src/autonomy.rs` (integrate traces, shadow, drift)
- [ ] `crates/kernel/src/shell.rs` (add new commands)
- [ ] `README.md` (add Phase 7 section)

---

## End of Plan

**This plan is ready for AI agent implementation.**

**Key success factors:**
1. Strict adherence to build requirements (Rust nightly-2025-01-15, bootloader 0.11.12)
2. Reuse of existing Phase 4 infrastructure (crypto, logging, metrics, chaos)
3. Comprehensive testing at every milestone
4. Clear documentation for all new features

**After implementation:**
1. Create branch: `ai-agent/phase7-ai-operations`
2. Implement according to plan
3. Run `scripts/phase7_tests.sh` to validate
4. Create PR to main with Phase 7 complete

**Branch naming:** `ai-agent/phase7-ai-operations` or similar
**Expected PR size:** +3,300 lines Rust, +1,500 lines docs/tests

---

**Total estimated effort:** 130-170 hours (3.25-4.25 weeks at 40hr/week)
**Confidence level:** HIGH (building on proven Phase 4 patterns)
