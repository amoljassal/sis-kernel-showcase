# AI-Native Kernel Enhancement Plan

**Status:** Planning
**Target:** Production-grade AI/ML kernel with demonstrable innovation
**Execution:** Multi-phase implementation via AI agent → Integration & Debug → Validation

## Executive Summary

This plan enhances the SIS kernel from a feature-complete AI-native OS to a production-grade system that demonstrates real-world R&D value. Implementation is prioritized to maximize unique differentiators (AI/ML innovation) while ensuring robustness (testing) and visibility (documentation).

**Scope:** All items except community engagement (r/osdev posting - manual task)

---

## Phase 1: AI/ML Innovation (Weeks 1-4) 🎯 HIGHEST PRIORITY

### Objective
Transform AI components from infrastructure to functional intelligence that provides measurable value.

### 1.1 Predictive Crash Detection via Memory Patterns

**Goal:** Use decision traces and memory metrics to predict kernel panics before they occur.

**Technical Specifications:**

**Location:** `crates/kernel/src/ai_insights/crash_predictor.rs` (new file)

**Core Algorithm:**
```rust
pub struct CrashPredictor {
    // Rolling window of memory allocation patterns
    alloc_history: RingBuffer<AllocMetrics, 100>,
    // Failed allocation counter
    oom_signals: AtomicU32,
    // Fragmentation trend detector
    frag_trend: LinearRegression,
    // Prediction threshold (tunable)
    danger_threshold: f32,
}

struct AllocMetrics {
    timestamp_ms: u64,
    free_pages: usize,
    largest_free_block: usize,
    fragmentation_ratio: f32,
    allocation_failures: u32,
}
```

**Prediction Logic:**
1. Collect metrics every 100ms from buddy allocator
2. Calculate fragmentation trend: `(peak_free - current_free) / peak_free`
3. Detect patterns:
   - Rapid decline in free pages (>20% in 1 second)
   - Fragmentation ratio increasing (>0.7)
   - Repeated allocation failures (>3 in window)
4. Emit warning at 80% confidence, trigger compaction at 90%

**Integration Points:**
- Hook into `crate::mm::buddy::allocate_pages()` for metrics collection
- Call from `crate::ai_insights::mod.rs::update_memory_insight()`
- Trigger `memctl compact` automatically via `crate::control::memory::auto_compact()`

**Shell Command:**
```rust
// crates/kernel/src/shell/shell_crashctl.rs
crashctl status          // Show prediction confidence (0-100%)
crashctl history         // Last 50 predictions with outcomes
crashctl tune <threshold> // Adjust sensitivity (0.0-1.0)
```

**Success Metrics:**
- Predict 80%+ of OOM crashes 5+ seconds before occurrence
- False positive rate <10% during stress tests
- Automatic compaction prevents 50%+ of predicted crashes

---

### 1.2 Transformer-Based Scheduler (Lightweight Attention)

**Goal:** Use attention mechanism to prioritize tasks based on historical performance patterns.

**Technical Specifications:**

**Location:** `crates/kernel/src/sched/transformer_sched.rs` (new file)

**Architecture:**
```rust
// Simplified transformer for embedded use (no full BERT)
pub struct TransformerScheduler {
    // Task embedding: [priority, cpu_time_used, io_wait, cache_affinity]
    embedder: TaskEmbedding,
    // Single-head self-attention (no multi-head for efficiency)
    attention: AttentionLayer,
    // Output: scheduling score (0.0-1.0)
    output_layer: LinearLayer,
    // Training data: past scheduling decisions
    decision_log: CircularBuffer<ScheduleDecision, 1000>,
}

struct TaskEmbedding {
    // 4D embedding per task
    dims: [f32; 4], // [norm_priority, cpu_ratio, io_ratio, cache_score]
}

struct AttentionLayer {
    // Weights: 4x4 matrix (small enough for kernel)
    query_weights: [[f32; 4]; 4],
    key_weights: [[f32; 4]; 4],
    value_weights: [[f32; 4]; 4],
}
```

**Attention Mechanism:**
```
Score(Q, K, V) = softmax(Q * K^T / sqrt(d_k)) * V

Where:
- Q (query): Current task characteristics
- K (keys): Historical task patterns
- V (values): Past scheduling outcomes (success scores)
- d_k = 4 (embedding dimension)
```

**Implementation:**
1. Embed each runnable task into 4D vector
2. Compute attention scores against last 20 scheduled tasks
3. Output scheduling priority (scaled 0-100)
4. Use in existing `deterministic::schedule_decision()`

**Integration Points:**
- Replace/augment `crate::sched::deterministic::calculate_priority()`
- Log decisions to `crate::trace_decision::TRACE_BUFFER`
- Benchmark against baseline scheduler

**Training (Online Learning):**
- Reward: task completion without context switches
- Penalty: excessive wait time, cache misses
- Update weights every 100 scheduling decisions (gradient descent)

**Shell Commands:**
```rust
// crates/kernel/src/shell/shell_schedctl.rs
schedctl transformer on|off     // Toggle transformer scheduler
schedctl stats                  // Show attention weights, accuracy
schedctl reset-weights          // Reinitialize to defaults
```

**Success Metrics:**
- 10-20% reduction in context switch overhead vs baseline
- Task completion time improved by 15%+ in mixed workloads
- Inference latency <50µs (target <100µs)

---

### 1.3 LLM Service Fine-Tuning Hooks (LoRA Stubs)

**Goal:** Enable model fine-tuning via shell commands for on-device adaptation.

**Technical Specifications:**

**Location:** `crates/kernel/src/llm/finetune.rs` (new file)

**LoRA (Low-Rank Adaptation) Stub:**
```rust
pub struct LoRAAdapter {
    // Low-rank matrices for weight adaptation
    // Original weight W ≈ W + A*B (where A: m×r, B: r×n, r << m,n)
    lora_a: Vec<f32>, // Rank r = 4 (keep small)
    lora_b: Vec<f32>,
    rank: usize,
    alpha: f32, // Scaling factor (default 16.0)
}

pub struct FineTuner {
    base_model: Arc<Model>, // Frozen base weights
    adapters: HashMap<String, LoRAAdapter>, // Layer name -> adapter
    training_data: Vec<TrainingExample>,
    learning_rate: f32,
}

struct TrainingExample {
    input_tokens: Vec<u16>,
    expected_output: Vec<u16>,
    loss: f32,
}
```

**Training Process:**
1. Load training data from `/models/finetune-data.json`
2. Freeze base model weights
3. Only update LoRA matrices (A, B)
4. Use simple SGD: `W_new = W_old - lr * gradient`
5. Save adapted model to `/models/<name>-finetuned.safetensors`

**Integration Points:**
- Hook into `crate::model_lifecycle::registry::MODEL_REGISTRY`
- Use existing `crate::llm::infer::infer_stub()` for forward pass
- Persist via `crate::vfs::ext4` (use new write support!)

**Shell Commands:**
```rust
// crates/kernel/src/shell/shell_llmctl.rs
llmctl finetune <data-path> <output-name> [--epochs N] [--lr 0.001]
llmctl finetune-status         // Show training progress
llmctl finetune-cancel         // Stop ongoing training
```

**Data Format (`/models/finetune-data.json`):**
```json
{
  "examples": [
    {
      "prompt": "Optimize memory for high fragmentation",
      "completion": "memctl compact --aggressive"
    },
    {
      "prompt": "Current load: 0.85, predict next action",
      "completion": "schedctl set-priority 50 task_id=42"
    }
  ]
}
```

**Success Metrics:**
- Fine-tune completes in <30 seconds for 100 examples
- Adapted model improves task-specific accuracy by 20%+
- LoRA adapters stored in <1MB (vs full model ~50MB)

---

### 1.4 Real-Time LLM Inference on OS State

**Goal:** Enable natural language queries about system state using LLM.

**Technical Specifications:**

**Location:** `crates/kernel/src/llm/state_inference.rs` (new file)

**State Serialization:**
```rust
pub struct SystemStateEncoder {
    memory_stats: MemorySnapshot,
    sched_stats: SchedulerSnapshot,
    io_stats: IoSnapshot,
    // Encode into tokens for LLM
    tokenizer: SimpleTokenizer,
}

impl SystemStateEncoder {
    pub fn encode_prompt(&self, user_query: &str) -> Vec<u16> {
        // Format: "System State: [mem_free=1024MB, cpu_load=0.45, ...]\nQuery: {user_query}\nAnswer:"
        let state_str = format!(
            "Memory: {} MB free, {} MB used, fragmentation={:.2}\n\
             CPU: load={:.2}, tasks={}\n\
             I/O: reads={}, writes={}\n\
             Query: {}\nAnswer:",
            self.memory_stats.free_mb,
            self.memory_stats.used_mb,
            self.memory_stats.fragmentation,
            self.sched_stats.load_avg,
            self.sched_stats.num_tasks,
            self.io_stats.total_reads,
            self.io_stats.total_writes,
            user_query
        );
        self.tokenizer.encode(&state_str)
    }
}
```

**Inference Pipeline:**
1. Capture current system state snapshot
2. Format as natural language context
3. Append user query
4. Run through LLM (use existing `llm::infer_stub`)
5. Parse output as shell command or explanation

**Integration Points:**
- Call from `crate::shell::shell_llmctl.rs`
- Reuse `crate::ai_insights::gather_memory_stats()` for state capture
- Execute suggested commands via `crate::shell::execute_command()`

**Shell Commands:**
```rust
// Enhanced llmctl
llmctl infer "What's causing high memory usage?"
llmctl infer "Optimize for current workload"
llmctl infer "Predict if I need to compact memory"
llmctl auto-execute on|off  // Auto-run suggested commands
```

**Example Interaction:**
```
> llmctl infer "Memory usage is high, what should I do?"
LLM Output: "Based on 847MB used with 0.73 fragmentation, run 'memctl compact' to free ~200MB"
Suggested command: memctl compact
Execute? [y/N]: y
```

**Success Metrics:**
- 70%+ of queries produce actionable commands
- Inference latency <500ms (target <1s)
- Command execution success rate >80%

---

### 1.5 AI Impact Dashboard (Metrics & Visualization)

**Goal:** Visualize AI decision impact in real-time via autoctl dashboard.

**Technical Specifications:**

**Location:** `crates/kernel/src/control/ai_metrics.rs` (new file)

**Metric Collectors:**
```rust
pub struct AiMetricsCollector {
    // Decision latencies (from trace_decision)
    nn_infer_latencies: RingBuffer<u64, 1000>, // microseconds
    transformer_latencies: RingBuffer<u64, 1000>,

    // Impact measurements
    decisions_made: AtomicU64,
    decisions_followed: AtomicU64,
    crash_predictions: AtomicU64,
    crash_predictions_correct: AtomicU64,

    // Efficiency gains
    baseline_ctx_switch_avg: f32, // Pre-AI average
    ai_ctx_switch_avg: f32,       // With AI scheduler
    memory_saved_by_compaction: AtomicU64, // Bytes
}

pub struct AiMetricsSnapshot {
    pub avg_nn_infer_us: u64,
    pub p50_nn_infer_us: u64,
    pub p99_nn_infer_us: u64,
    pub decision_follow_rate: f32, // %
    pub crash_prediction_accuracy: f32, // %
    pub ctx_switch_improvement: f32, // %
    pub memory_saved_mb: u64,
}
```

**Dashboard Updates:**
```rust
// Integration with existing autonomy/dashboard.rs
impl DashboardState {
    pub fn update_ai_metrics(&mut self, metrics: AiMetricsSnapshot) {
        self.sections.push(DashboardSection {
            title: "AI Performance".into(),
            rows: vec![
                format!("NN Inference: {}µs (p50), {}µs (p99)",
                    metrics.p50_nn_infer_us, metrics.p99_nn_infer_us),
                format!("Decision Follow Rate: {:.1}%", metrics.decision_follow_rate * 100.0),
                format!("Crash Prediction Accuracy: {:.1}%", metrics.crash_prediction_accuracy * 100.0),
                format!("Context Switch Improvement: {:.1}%", metrics.ctx_switch_improvement * 100.0),
                format!("Memory Saved: {} MB", metrics.memory_saved_mb),
            ],
        });
    }
}
```

**GUI Integration (Phase 6 daemon):**
```typescript
// GUI/apps/daemon/src/ai-dashboard.ts
interface AiMetrics {
  nnInferLatency: { p50: number; p99: number };
  decisionFollowRate: number;
  crashPredictionAccuracy: number;
  ctxSwitchImprovement: number;
  memorySavedMb: number;
}

// WebSocket endpoint: /api/ai/metrics
// Poll every 1 second, update charts
```

**Visualization Components:**
1. **Line chart:** NN inference latency over time (last 5 minutes)
2. **Gauge:** Decision follow rate (0-100%)
3. **Bar chart:** Crash prediction accuracy (true positives vs false positives)
4. **Metric card:** Context switch improvement vs baseline
5. **Metric card:** Memory saved by AI-driven compaction

**Integration Points:**
- Collect from `crate::trace_decision::TRACE_BUFFER`
- Update in `crate::control::update_dashboard()`
- Expose via `crate::shell::shell_autoctl.rs::show_dashboard()`

**Shell Commands:**
```rust
autoctl ai-metrics         // Show AI metrics section
autoctl reset-baseline     // Reset baseline measurements
autoctl export-metrics <path> // Export JSON for analysis
```

**Success Metrics:**
- Dashboard updates in real-time (<1s latency)
- Show 20-30% measurable improvements in at least 2 metrics
- Metrics exportable for external analysis

---

## Phase 2: Testing & Validation (Weeks 5-7)

### Objective
Achieve production-grade robustness with comprehensive automated testing.

### 2.1 Automated Ext4 Crash Recovery Testing

**Goal:** Validate ext4 write support with 100% recovery rate across 50+ crash scenarios.

**Technical Specifications:**

**Location:** `scripts/ext4_crash_recovery_harness.sh` (enhance existing `ext4_durability_tests.sh`)

**Test Scenarios:**
```bash
#!/bin/bash
# scripts/ext4_crash_recovery_harness.sh

SCENARIOS=(
    "write_during_allocation"  # Crash during allocate_block()
    "write_during_inode_create" # Crash during allocate_inode()
    "write_during_journal_commit" # Crash during journal write
    "write_during_data_write"  # Crash during write_data_direct()
    "concurrent_writes"        # Multiple writes, random crash
    "directory_operations"     # mkdir/rmdir with crashes
    "truncate_operations"      # O_TRUNC with crashes
)

for scenario in "${SCENARIOS[@]}"; do
    echo "Testing: $scenario"
    # 1. Create fresh ext4 image
    # 2. Boot kernel with EXT4_IMG
    # 3. Run specific workload (via QMP commands)
    # 4. Inject crash at random point (QMP: system_reset)
    # 5. Reboot, verify journal replay
    # 6. Run fsck.ext4, assert clean
    # 7. Verify data integrity (checksum files)
done
```

**QMP Crash Injection:**
```python
# tools/qmp_crasher.py
import qmp
import random
import time

def inject_crash_during_workload(workload_duration_ms):
    """Connect to QEMU via QMP, crash at random point during workload"""
    client = qmp.QMPClient()
    client.connect(('localhost', 4444))

    # Wait random time (10-90% of workload)
    crash_time = random.uniform(0.1, 0.9) * workload_duration_ms / 1000.0
    time.sleep(crash_time)

    # Hard reset (simulates power loss)
    client.execute('system_reset')
```

**Workload Definitions:**
```rust
// crates/testing/src/ext4_stress.rs
#[cfg(feature = "ext4-stress-test")]
pub fn run_workload(scenario: &str) -> Result<()> {
    match scenario {
        "write_during_allocation" => {
            // Create 100 small files rapidly
            for i in 0..100 {
                vfs::create(&format!("/test{}.dat", i), 0o644, OpenFlags::O_CREAT)?;
            }
        }
        "write_during_journal_commit" => {
            // Create file, write data, force sync
            let f = vfs::create("/bigfile.dat", 0o644, OpenFlags::O_CREAT | OpenFlags::O_SYNC)?;
            f.write(&vec![0xAB; 1024 * 1024])?; // 1MB, triggers commit
        }
        "truncate_operations" => {
            // Create, write, truncate, repeat
            for _ in 0..10 {
                let f = vfs::create("/trunctest.dat", 0o644, OpenFlags::O_CREAT | OpenFlags::O_TRUNC)?;
                f.write(&vec![0xFF; 4096])?;
                vfs::unlink("/trunctest.dat")?;
            }
        }
        _ => unimplemented!(),
    }
    Ok(())
}
```

**Verification:**
```bash
# After reboot and journal replay
verify_integrity() {
    local img=$1

    # 1. fsck must be clean
    fsck.ext4 -n "$img" > fsck.log 2>&1
    if ! grep -q "clean" fsck.log; then
        echo "FAIL: Filesystem not clean"
        return 1
    fi

    # 2. Check expected files exist (using debugfs)
    debugfs -R "ls /incidents" "$img" | grep "INC-"

    # 3. Verify checksums (if files contain known data)
    # Compare with pre-crash checksums
}
```

**Integration Points:**
- Use existing `crate::fs::ext4::Ext4FileSystem`
- Trigger via `BRINGUP=1 SIS_FEATURES="ext4-stress-test" ./scripts/uefi_run.sh`
- Store results in `/tmp/ext4-crash-test-results.json`

**Shell Commands:**
```rust
// crates/kernel/src/shell/shell_fsctl.rs (new)
fsctl stress <scenario>   // Run specific stress test
fsctl verify              // Check filesystem integrity
```

**Success Metrics:**
- 100% recovery rate across all 7 scenarios (50 runs each)
- fsck reports clean filesystem in all cases
- Zero data corruption (checksums match)
- Average recovery time <2 seconds

---

### 2.2 VFS Path Lookup Fuzzing

**Goal:** Discover edge cases in VFS path resolution using cargo-fuzz.

**Technical Specifications:**

**Location:** `fuzz/fuzz_targets/vfs_path_lookup.rs` (new directory structure)

**Setup:**
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzzing
cd /Users/amoljassal/sis/sis-kernel
cargo fuzz init
```

**Fuzz Target:**
```rust
// fuzz/fuzz_targets/vfs_path_lookup.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use sis_kernel::vfs::{path_lookup, init_vfs, tmpfs, mount};
use alloc::sync::Arc;

fuzz_target!(|data: &[u8]| {
    // Convert random bytes to string (potential path)
    if let Ok(path_str) = core::str::from_utf8(data) {
        // Initialize VFS (once per process)
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            init_vfs();
            let root = tmpfs::TmpFS::new().root_inode().unwrap();
            mount("tmpfs", root, "/").unwrap();
        });

        let root = get_root().unwrap();

        // Fuzz path_lookup with arbitrary input
        let _ = path_lookup(&root, path_str);
        // Should not panic, only return Err for invalid paths
    }
});
```

**Test Cases (Corpus):**
```
fuzz/corpus/vfs_path_lookup/
├── valid_paths.txt
│   /
│   /foo
│   /foo/bar
│   /foo/bar/baz
├── edge_cases.txt
│   //
│   ///foo
│   /foo//bar
│   /foo/./bar
│   /foo/../bar
│   /foo/.
│   /foo/..
├── invalid_paths.txt
│   (empty string)
│   /\0/foo
│   /foo\nbar
│   /../../../etc/passwd
│   /foo/.../bar
└── long_paths.txt
    (paths with 4096+ chars)
    (paths with 1000+ components)
```

**Run Fuzzing:**
```bash
# Run for 24 hours
cargo fuzz run vfs_path_lookup -- -max_total_time=86400

# Minimize corpus
cargo fuzz cmin vfs_path_lookup

# Reproduce crash
cargo fuzz run vfs_path_lookup fuzz/artifacts/crash-xyz
```

**Additional Fuzz Targets:**
```rust
// fuzz/fuzz_targets/ext4_get_inode_block.rs
fuzz_target!(|data: &[u8]| {
    // Fuzz extent tree traversal
    let (inode_bytes, block_idx) = parse_input(data);
    let inode: Ext4Inode = deserialize(inode_bytes);
    let _ = fs.get_inode_block(&inode, block_idx);
});

// fuzz/fuzz_targets/vfs_create.rs
fuzz_target!(|data: &[u8]| {
    // Fuzz file creation with arbitrary names/modes
    let (path, mode) = parse_path_mode(data);
    let _ = vfs::create(path, mode, OpenFlags::O_CREAT);
});
```

**Integration:**
- Add `fuzz/` directory to `.gitignore` (artifacts can be large)
- Document in `docs/guides/FUZZING.md`
- Run in CI for 1 hour per pull request

**Success Metrics:**
- Zero panics after 24-hour fuzzing run
- 80%+ code coverage of VFS path resolution
- All crashes fixed and regression tests added
- Corpus of 1000+ unique test cases

---

### 2.3 Formal Verification with Kani (Memory Safety)

**Goal:** Prove memory safety properties in critical allocator code.

**Technical Specifications:**

**Location:** `crates/kernel/src/mm/buddy.rs` (add Kani harnesses)

**Setup:**
```bash
# Install Kani
cargo install --locked kani-verifier
cargo kani setup
```

**Verification Harness:**
```rust
// crates/kernel/src/mm/buddy.rs
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn verify_allocate_pages_no_overflow() {
        // Prove: allocate_pages never causes integer overflow
        let order: usize = kani::any();
        kani::assume(order <= MAX_ORDER); // Constrain input

        let buddy = BuddyAllocator::new();
        let result = buddy.allocate_pages(order);

        // Property: if successful, address is within valid range
        if let Some(phys_addr) = result {
            kani::assert(phys_addr < PHYS_MEM_SIZE, "Address in bounds");
            kani::assert(phys_addr % PAGE_SIZE == 0, "Page aligned");
        }
    }

    #[kani::proof]
    fn verify_free_pages_double_free() {
        // Prove: double-free is detected
        let addr: usize = kani::any();
        kani::assume(addr % PAGE_SIZE == 0);

        let buddy = BuddyAllocator::new();

        // First free should succeed
        buddy.free_pages(addr, 0);

        // Second free should fail (or detect corruption)
        let result2 = buddy.free_pages(addr, 0);
        kani::assert(result2.is_err(), "Double-free detected");
    }

    #[kani::proof]
    #[kani::unwind(10)] // Limit loop iterations for verification
    fn verify_coalesce_preserves_total_free() {
        // Prove: coalescing doesn't lose pages
        let buddy = BuddyAllocator::new();
        let initial_free = buddy.total_free_pages();

        buddy.coalesce(kani::any()); // Fuzz coalesce logic

        let final_free = buddy.total_free_pages();
        kani::assert(initial_free == final_free, "Free pages preserved");
    }
}
```

**Properties to Verify:**
1. **No integer overflow** in order-to-size calculations
2. **No double-free** - freeing same address twice fails
3. **No memory leaks** - alloc/free preserves total pages
4. **Alignment guarantees** - all allocations page-aligned
5. **Bounds checking** - never access beyond buddy bitmap

**Run Verification:**
```bash
# Verify all harnesses
cargo kani --harness verify_allocate_pages_no_overflow
cargo kani --harness verify_free_pages_double_free

# Generate coverage report
cargo kani --coverage --harness verify_allocate_pages_no_overflow
```

**Integration:**
- Add Kani harnesses to existing files (gated with `#[cfg(kani)]`)
- Document in `docs/guides/FORMAL_VERIFICATION.md`
- Run in CI (may be slow, run on schedule/major PRs only)

**Success Metrics:**
- All 5 properties verified (no counterexamples found)
- Harnesses cover 80%+ of buddy allocator code
- Verification completes in <10 minutes per harness
- Zero memory safety issues in verified code

---

### 2.4 EU AI Act Compliance Testing

**Goal:** Automate compliance scoring (target 92%+) with audit trail generation.

**Technical Specifications:**

**Location:** `crates/testing/src/compliance/eu_ai_act.rs` (new file)

**Compliance Criteria (from README - 92% scoring):**
```rust
pub struct EuAiActCompliance {
    // High-risk AI system requirements (Annex III)
    explainability_score: f32,      // 95% - decision traces
    human_oversight_score: f32,     // 90% - approval workflows
    robustness_score: f32,          // 88% - chaos testing results
    transparency_score: f32,        // 94% - audit logs
    data_governance_score: f32,     // 92% - model provenance
}

impl EuAiActCompliance {
    pub fn calculate_overall_score(&self) -> f32 {
        (self.explainability_score * 0.25 +
         self.human_oversight_score * 0.20 +
         self.robustness_score * 0.20 +
         self.transparency_score * 0.20 +
         self.data_governance_score * 0.15)
    }
}
```

**Test Suite:**
```rust
#[cfg(test)]
mod compliance_tests {
    #[test]
    fn test_explainability_all_decisions_traced() {
        // Requirement: All AI decisions must be traceable
        let decisions = run_ai_workload(100); // 100 AI decisions

        for decision in decisions {
            // Each decision must have trace entry
            let trace = TRACE_BUFFER.find_by_trace_id(decision.trace_id);
            assert!(trace.is_some(), "Decision {} not traced", decision.trace_id);

            // Trace must contain input features
            assert!(!trace.input_features.is_empty());
            // Trace must contain model version
            assert!(!trace.model_version.is_empty());
            // Trace must have confidence score
            assert!(trace.confidence > 0.0);
        }
        // Score: % of decisions with complete traces
    }

    #[test]
    fn test_human_oversight_approval_workflow() {
        // Requirement: High-risk decisions require approval

        // 1. Enable strict approval mode
        set_approval_mode(ApprovalMode::Strict);

        // 2. Trigger high-risk decision (e.g., schedctl set-priority)
        let result = execute_ai_decision(DecisionType::HighRisk);

        // 3. Should be pending approval
        assert_eq!(result.status, DecisionStatus::PendingApproval);

        // 4. Auto-approval in safe mode should be blocked
        let auto_result = try_auto_approve();
        assert!(auto_result.is_err());

        // Score: % of high-risk decisions properly gated
    }

    #[test]
    fn test_robustness_chaos_resilience() {
        // Requirement: System must handle failures gracefully

        // Run chaos scenarios (from Phase 4)
        let scenarios = vec![
            ChaosScenario::DiskFull,
            ChaosScenario::NetworkFail,
            ChaosScenario::MemoryPressure,
        ];

        let mut recovery_count = 0;
        for scenario in scenarios {
            inject_chaos(scenario);
            if system_recovers_within(Duration::from_secs(30)) {
                recovery_count += 1;
            }
        }

        let robustness = recovery_count as f32 / scenarios.len() as f32;
        assert!(robustness >= 0.88, "Robustness score too low: {}", robustness);
    }

    #[test]
    fn test_transparency_audit_logs_complete() {
        // Requirement: All model updates logged

        // 1. Load new model
        modelctl_load("/models/test-v2.safetensors");

        // 2. Check audit log entry exists
        let logs = get_audit_logs();
        let load_entry = logs.iter().find(|e| e.event_type == "model_load");
        assert!(load_entry.is_some());

        // 3. Verify log contains: timestamp, user, model hash, outcome
        let entry = load_entry.unwrap();
        assert!(entry.timestamp > 0);
        assert!(!entry.model_hash.is_empty());
        assert_eq!(entry.outcome, "success");

        // Score: % of events properly logged
    }

    #[test]
    fn test_data_governance_model_provenance() {
        // Requirement: Track model origin and training data

        let model = MODEL_REGISTRY.get_active_model();

        // Must have provenance metadata
        assert!(model.metadata.training_dataset.is_some());
        assert!(model.metadata.training_date.is_some());
        assert!(model.metadata.author.is_some());

        // Must have signature verification
        assert!(model.signature_verified);

        // Score: % of models with complete provenance
    }
}
```

**Auto-Report Generation:**
```rust
// crates/testing/src/compliance/reporter.rs
pub fn generate_compliance_report() -> ComplianceReport {
    let scores = EuAiActCompliance {
        explainability_score: test_explainability(),
        human_oversight_score: test_human_oversight(),
        robustness_score: test_robustness(),
        transparency_score: test_transparency(),
        data_governance_score: test_data_governance(),
    };

    ComplianceReport {
        overall_score: scores.calculate_overall_score(),
        breakdown: scores,
        test_results: run_all_compliance_tests(),
        generated_at: get_timestamp(),
        attestation: "SIS Kernel v1.0 - EU AI Act Assessment",
    }
}
```

**Output Format:**
```json
{
  "compliance_report": {
    "overall_score": 0.92,
    "breakdown": {
      "explainability": 0.95,
      "human_oversight": 0.90,
      "robustness": 0.88,
      "transparency": 0.94,
      "data_governance": 0.92
    },
    "test_results": {
      "total_tests": 47,
      "passed": 43,
      "failed": 4,
      "details": [...]
    },
    "generated_at": "2025-01-15T10:30:00Z",
    "attestation": "SIS Kernel v1.0 - EU AI Act Assessment"
  }
}
```

**Integration:**
- Run via `cargo test --features compliance-testing`
- Export report: `tools/compliance_report.sh > /tmp/eu-ai-act-compliance.json`
- Add to CI/CD pipeline

**Success Metrics:**
- Overall score ≥92%
- All 5 categories ≥88%
- Report auto-generated in <60 seconds
- Zero critical compliance failures

---

### 2.5 Performance Benchmark Suite

**Goal:** Establish baseline vs AI-enhanced metrics for all major subsystems.

**Technical Specifications:**

**Location:** `crates/testing/src/benchmarks/` (new directory)

**Benchmark Harness:**
```rust
// crates/testing/src/benchmarks/scheduler_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_context_switch_baseline(c: &mut Criterion) {
    // Disable AI scheduler
    set_scheduler_mode(SchedulerMode::Baseline);

    c.bench_function("ctx_switch_baseline", |b| {
        b.iter(|| {
            // Force context switch between 2 tasks
            let task1 = create_test_task();
            let task2 = create_test_task();
            switch_to(task1);
            switch_to(task2);
        });
    });
}

fn bench_context_switch_ai(c: &mut Criterion) {
    // Enable AI scheduler (transformer)
    set_scheduler_mode(SchedulerMode::Transformer);

    c.bench_function("ctx_switch_ai", |b| {
        b.iter(|| {
            let task1 = create_test_task();
            let task2 = create_test_task();
            switch_to(task1);
            switch_to(task2);
        });
    });
}

criterion_group!(scheduler_benches, bench_context_switch_baseline, bench_context_switch_ai);
```

**Benchmark Categories:**

1. **Scheduler Performance:**
   - Context switch latency (baseline vs AI)
   - Task selection latency
   - Throughput (tasks/second)

2. **Memory Management:**
   - Allocation latency (buddy allocator)
   - Compaction time
   - Fragmentation ratio over time

3. **AI Inference:**
   - NN inference latency (p50, p99)
   - Transformer attention latency
   - Decision throughput

4. **Filesystem:**
   - Ext4 read/write throughput
   - Journal commit latency
   - fsync durability overhead

**Output Format (JSON):**
```json
{
  "benchmarks": {
    "scheduler": {
      "ctx_switch_baseline_ns": { "mean": 1250, "p50": 1100, "p99": 2800 },
      "ctx_switch_ai_ns": { "mean": 980, "p50": 900, "p99": 2100 },
      "improvement_pct": 21.6
    },
    "memory": {
      "alloc_baseline_ns": { "mean": 28000, "p50": 25000, "p99": 45000 },
      "alloc_ai_ns": { "mean": 22000, "p50": 19000, "p99": 38000 },
      "improvement_pct": 21.4
    },
    "ai_inference": {
      "nn_infer_us": { "mean": 45, "p50": 42, "p99": 89 },
      "transformer_us": { "mean": 38, "p50": 35, "p99": 72 }
    }
  }
}
```

**Comparison Tool:**
```bash
#!/bin/bash
# tools/benchmark_compare.sh

# Run baseline benchmarks
BASELINE_MODE=1 cargo bench --features benchmark > baseline_results.json

# Run AI benchmarks
AI_MODE=1 cargo bench --features benchmark > ai_results.json

# Generate comparison report
python3 tools/bench_compare.py baseline_results.json ai_results.json
```

**Integration:**
- Run in CI on performance-critical PRs
- Store results in `benchmarks/results/` directory
- Track regressions (>5% slowdown = fail CI)

**Success Metrics:**
- AI scheduler shows 10-20% improvement in context switch latency
- Memory allocator p99 <25µs
- NN inference p99 <100µs
- All benchmarks tracked in version control

---

## Phase 3: Performance Optimization (Weeks 8-10)

### Objective
Demonstrate technical excellence through measurable performance improvements.

### 3.1 Slab Allocator for Small Objects

**Goal:** Reduce memory allocation latency for small (<1KB) objects.

**Technical Specifications:**

**Location:** `crates/kernel/src/mm/slab.rs` (new file)

**Architecture:**
```rust
pub struct SlabAllocator {
    // Size classes: 16, 32, 64, 128, 256, 512 bytes
    size_classes: [SlabCache; 6],
    // Fallback to buddy allocator for larger allocations
    buddy: &'static BuddyAllocator,
}

struct SlabCache {
    object_size: usize,
    // Linked list of slabs (each slab is 1 page = 4KB)
    slabs: LinkedList<Slab>,
    // Free object cache (LIFO for cache locality)
    free_list: LinkedList<*mut u8>,
}

struct Slab {
    // Pointer to page memory
    memory: *mut u8,
    // Bitmap of free objects (1 bit per object)
    free_bitmap: u64,
    // Number of free objects
    free_count: usize,
}
```

**Allocation Logic:**
```rust
impl SlabAllocator {
    pub fn alloc(&self, size: usize) -> Option<*mut u8> {
        // 1. Round up to size class
        let size_class = self.round_to_size_class(size);

        if size_class > 512 {
            // Fallback to buddy allocator
            return self.buddy.allocate_pages(pages_for(size));
        }

        // 2. Find slab cache for size class
        let cache = &mut self.size_classes[size_class.trailing_zeros()];

        // 3. Try free list first (fast path)
        if let Some(ptr) = cache.free_list.pop_front() {
            return Some(ptr);
        }

        // 4. Find slab with free objects
        for slab in &mut cache.slabs {
            if slab.free_count > 0 {
                let obj_idx = slab.free_bitmap.trailing_zeros();
                slab.free_bitmap &= !(1 << obj_idx);
                slab.free_count -= 1;
                return Some(unsafe { slab.memory.add(obj_idx * cache.object_size) });
            }
        }

        // 5. Allocate new slab
        self.allocate_new_slab(cache)
    }

    pub fn free(&self, ptr: *mut u8, size: usize) {
        let size_class = self.round_to_size_class(size);

        if size_class > 512 {
            self.buddy.free_pages(ptr as usize, pages_for(size));
            return;
        }

        let cache = &mut self.size_classes[size_class.trailing_zeros()];

        // Add to free list (LIFO for cache warmth)
        cache.free_list.push_front(ptr);
    }
}
```

**Integration Points:**
- Replace `GlobalAllocator` trait implementation in `crates/kernel/src/mm/mod.rs`
- Use slab for kernel structures:
  - Task control blocks (256 bytes)
  - Inode objects (128 bytes)
  - File descriptors (64 bytes)
  - Network packets (512 bytes)

**NEON Optimization (ARM64):**
```rust
#[cfg(target_arch = "aarch64")]
unsafe fn memset_neon(ptr: *mut u8, value: u8, count: usize) {
    // Use NEON SIMD for fast initialization
    use core::arch::aarch64::*;
    let val_vec = vdupq_n_u8(value);

    let mut chunks = count / 16;
    let mut p = ptr;

    while chunks > 0 {
        vst1q_u8(p, val_vec);
        p = p.add(16);
        chunks -= 1;
    }

    // Handle remaining bytes
    let remainder = count % 16;
    for i in 0..remainder {
        *p.add(i) = value;
    }
}
```

**Benchmarks:**
```rust
#[bench]
fn bench_slab_alloc_16b(b: &mut Bencher) {
    b.iter(|| {
        let ptr = SLAB_ALLOCATOR.alloc(16);
        SLAB_ALLOCATOR.free(ptr, 16);
    });
}
// Target: <500ns mean, <1µs p99
```

**Success Metrics:**
- Small allocation (<512B) latency <1µs (vs ~28µs buddy)
- Memory overhead <10% (slab metadata)
- Cache hit rate >80% for common sizes
- NEON optimizations show 2x speedup for initialization

---

### 3.2 VirtIO Performance Tuning

**Goal:** Maximize VirtIO throughput in QEMU (50%+ improvement).

**Technical Specifications:**

**Location:** `crates/kernel/src/drivers/virtio/` (enhance existing drivers)

**Optimization Areas:**

**1. Queue Depth Tuning:**
```rust
// crates/kernel/src/drivers/virtio/block.rs
const QUEUE_SIZE: u16 = 128; // Increase from 16 (default)
// Allows more in-flight requests
```

**2. Descriptor Chaining:**
```rust
impl VirtioBlock {
    pub fn submit_request_batch(&self, requests: &[BlockRequest]) -> Result<()> {
        // Chain multiple descriptors for scatter-gather I/O
        for req in requests {
            self.queue.add_descriptor_chain(&[
                Descriptor::readable(&req.header),
                Descriptor::writable(&req.data),
                Descriptor::readable(&req.status),
            ])?;
        }
        self.queue.notify(); // Single notification for batch
        Ok(())
    }
}
```

**3. Interrupt Coalescing:**
```rust
// Reduce interrupt overhead by batching completions
impl VirtQueue {
    pub fn enable_interrupt_coalescing(&mut self, threshold: u16) {
        // Only trigger interrupt after N completions
        self.used_event = (self.last_used_idx + threshold) % self.size;
        self.avail.flags &= !VIRTQ_AVAIL_F_NO_INTERRUPT;
    }
}
```

**4. Zero-Copy DMA:**
```rust
// crates/kernel/src/drivers/virtio/net.rs
impl VirtioNet {
    pub fn send_packet_zerocopy(&self, packet: &[u8]) -> Result<()> {
        // Avoid copying: use physical address directly
        let phys_addr = virt_to_phys(packet.as_ptr() as usize);

        self.tx_queue.add_descriptor(Descriptor {
            addr: phys_addr,
            len: packet.len() as u32,
            flags: VIRTQ_DESC_F_NEXT,
            next: 0,
        })?;

        self.tx_queue.notify();
        Ok(())
    }
}
```

**5. Polling Mode (for benchmarks):**
```rust
// Disable interrupts and poll for completions (low latency mode)
pub fn enable_polling_mode(&mut self) {
    self.avail.flags |= VIRTQ_AVAIL_F_NO_INTERRUPT;

    // Spin on used ring in tight loop
    loop {
        if self.used.idx != self.last_used_idx {
            self.process_completions();
        }
    }
}
```

**Benchmarks:**
```rust
// crates/testing/src/benchmarks/virtio_bench.rs

#[bench]
fn bench_block_throughput_sequential(b: &mut Bencher) {
    let block_dev = get_virtio_block();
    let data = vec![0u8; 4096];

    b.iter(|| {
        // Measure MB/s for sequential writes
        for block_num in 0..1000 {
            block_dev.write(block_num, &data);
        }
    });
}

#[bench]
fn bench_network_throughput(b: &mut Bencher) {
    let net_dev = get_virtio_net();
    let packet = vec![0u8; 1500]; // MTU size

    b.iter(|| {
        for _ in 0..1000 {
            net_dev.send_packet(&packet);
        }
    });
}
```

**QEMU Configuration (for max performance):**
```bash
# scripts/uefi_run.sh additions
QEMU_OPTS+=" -device virtio-blk-pci,drive=blk0,num-queues=4,queue-size=128"
QEMU_OPTS+=" -device virtio-net-pci,netdev=net0,mq=on,vectors=10"
QEMU_OPTS+=" -object iothread,id=io1"
QEMU_OPTS+=" -device virtio-blk-pci,iothread=io1" # Offload to separate thread
```

**Success Metrics:**
- Block device throughput: 50%+ improvement (target 100 MB/s)
- Network throughput: 50%+ improvement (target 500 Mbps)
- Interrupt rate reduced by 30%+
- Latency reduction: 20%+ for small I/O

---

### 3.3 Energy Estimation (Power-Aware Scheduling)

**Goal:** Predict energy consumption and integrate with scheduling decisions.

**Technical Specifications:**

**Location:** `crates/kernel/src/power/estimator.rs` (new file)

**Power Model:**
```rust
pub struct PowerEstimator {
    // CPU frequency (from CNTFRQ_EL0 register)
    cpu_freq_hz: u64,
    // Power coefficients (calibrated per platform)
    coeff_idle: f32,      // Watts when idle
    coeff_cpu: f32,       // Watts per % CPU utilization
    coeff_mem: f32,       // Watts per MB/s memory bandwidth
    // Historical measurements
    power_history: RingBuffer<PowerSample, 1000>,
}

struct PowerSample {
    timestamp_ms: u64,
    cpu_utilization: f32,    // 0.0-1.0
    mem_bandwidth_mbs: f32,
    estimated_watts: f32,
}

impl PowerEstimator {
    pub fn estimate_current_power(&self) -> f32 {
        // Simple linear model: P = P_idle + C_cpu * util + C_mem * bw
        let util = self.get_cpu_utilization();
        let bw = self.get_memory_bandwidth();

        self.coeff_idle +
        self.coeff_cpu * util +
        self.coeff_mem * bw
    }

    fn get_cpu_utilization(&self) -> f32 {
        // Use PMU counters (from Phase 3)
        let cycles = read_pmu_counter(PMU_CYCLE_COUNTER);
        let total_cycles = self.cpu_freq_hz * 100; // Last 100ms
        cycles as f32 / total_cycles as f32
    }

    fn get_memory_bandwidth(&self) -> f32 {
        // Estimate from allocation rate
        let buddy_stats = BUDDY_ALLOCATOR.stats();
        buddy_stats.allocs_per_sec * PAGE_SIZE as f32 / 1_000_000.0 // MB/s
    }
}
```

**Integration with Scheduler:**
```rust
// crates/kernel/src/sched/power_aware.rs

pub fn schedule_with_power_budget(max_watts: f32) -> TaskId {
    let current_power = POWER_ESTIMATOR.estimate_current_power();
    let budget_remaining = max_watts - current_power;

    if budget_remaining < 1.0 {
        // Low power mode: prefer low-priority tasks
        return select_low_power_task();
    }

    // Normal scheduling with power awareness
    let tasks = get_runnable_tasks();
    tasks.iter()
        .filter(|t| t.estimated_power < budget_remaining)
        .max_by_key(|t| t.priority)
        .map(|t| t.id)
        .unwrap_or_else(|| select_idle_task())
}

fn estimate_task_power(task: &Task) -> f32 {
    // Use historical data from previous runs
    let avg_cpu = task.stats.avg_cpu_utilization;
    let avg_mem = task.stats.avg_memory_accesses;

    POWER_ESTIMATOR.coeff_cpu * avg_cpu +
    POWER_ESTIMATOR.coeff_mem * avg_mem
}
```

**Shell Commands:**
```rust
// crates/kernel/src/shell/shell_powerctl.rs
powerctl status             // Show current power estimate
powerctl history            // Plot power over time
powerctl set-budget <watts> // Enable power-aware scheduling
powerctl calibrate          // Run calibration workload
```

**Calibration Process:**
```rust
pub fn calibrate_power_model() -> PowerCoefficients {
    // 1. Idle measurement
    let idle_watts = measure_power_during_idle(Duration::from_secs(10));

    // 2. CPU stress test
    let cpu_watts = measure_power_during_cpu_stress(Duration::from_secs(10));
    let cpu_coeff = (cpu_watts - idle_watts) / 1.0; // Assume 100% util

    // 3. Memory stress test
    let mem_watts = measure_power_during_mem_stress(Duration::from_secs(10));
    let mem_coeff = (mem_watts - idle_watts) / measured_bandwidth_mbs;

    PowerCoefficients {
        idle: idle_watts,
        cpu: cpu_coeff,
        mem: mem_coeff,
    }
}
```

**QEMU Power Simulation:**
```rust
// Since QEMU doesn't provide real power measurements, simulate based on activity
#[cfg(target_arch = "aarch64")]
fn measure_power_during_idle(duration: Duration) -> f32 {
    // Use timer frequency as proxy for platform capability
    let freq = unsafe { read_cntfrq_el0() };

    // Simulated power model:
    // High-freq platform (>1GHz) = higher idle power
    let base_idle = if freq > 1_000_000_000 {
        5.0 // Watts (server-class)
    } else {
        1.0 // Watts (embedded)
    };

    base_idle
}
```

**Success Metrics:**
- Power estimation accuracy within 15% (vs baseline measurements)
- Power-aware scheduling reduces estimated consumption by 10-20%
- Dashboard shows real-time power graph
- Predictive power warnings before hitting budget

---

### 3.4 Profiling and Hotspot Elimination

**Goal:** Identify and fix top 3 performance bottlenecks using PMU data.

**Technical Specifications:**

**Location:** Use existing `crates/kernel/src/perf/pmu.rs`

**Profiling Methodology:**

**1. Enable Cycle-Level Profiling:**
```rust
// crates/kernel/src/perf/profiler.rs
pub struct KernelProfiler {
    // Sample-based profiling (every 10ms)
    samples: RingBuffer<ProfileSample, 10000>,
    sampling_interval_ms: u64,
}

struct ProfileSample {
    timestamp_ms: u64,
    pc: usize,              // Program counter
    lr: usize,              // Link register (caller)
    cycles: u64,
    instructions: u64,
    cache_misses: u64,
}

impl KernelProfiler {
    pub fn collect_sample(&mut self) {
        let pc = read_pc();
        let lr = read_lr();

        self.samples.push(ProfileSample {
            timestamp_ms: get_uptime_ms(),
            pc,
            lr,
            cycles: read_pmu_counter(PMU_CYCLE_COUNTER),
            instructions: read_pmu_counter(PMU_INST_RETIRED),
            cache_misses: read_pmu_counter(PMU_L1D_CACHE_REFILL),
        });
    }
}
```

**2. Symbol Resolution:**
```rust
// Resolve PC to function name (requires symbol table)
pub fn resolve_symbol(pc: usize) -> Option<&'static str> {
    // Parse kernel symbol table (generated at build time)
    SYMBOL_TABLE.binary_search_by(|sym| sym.addr.cmp(&pc))
        .ok()
        .map(|idx| SYMBOL_TABLE[idx].name)
}
```

**3. Flame Graph Generation:**
```bash
#!/bin/bash
# tools/profile_kernel.sh

# 1. Run kernel with profiling enabled
BRINGUP=1 SIS_FEATURES="perf-profiler" ./scripts/uefi_run.sh &

# 2. Collect samples for 60 seconds
sleep 60

# 3. Extract profile data via QMP
echo '{"execute":"qmp_capabilities"}' | socat - UNIX-CONNECT:/tmp/qemu-qmp.sock
echo '{"execute":"human-monitor-command","arguments":{"command-line":"info registers"}}' | socat - UNIX-CONNECT:/tmp/qemu-qmp.sock

# 4. Generate flame graph
cat profile_samples.txt | flamegraph.pl > kernel_flamegraph.svg
```

**4. Hotspot Analysis:**
```rust
pub fn analyze_hotspots(samples: &[ProfileSample]) -> Vec<Hotspot> {
    // Aggregate samples by function
    let mut function_cycles: HashMap<&str, u64> = HashMap::new();

    for sample in samples {
        if let Some(func_name) = resolve_symbol(sample.pc) {
            *function_cycles.entry(func_name).or_insert(0) += sample.cycles;
        }
    }

    // Sort by total cycles
    let mut hotspots: Vec<_> = function_cycles.into_iter()
        .map(|(name, cycles)| Hotspot { name, cycles })
        .collect();
    hotspots.sort_by_key(|h| h.cycles);
    hotspots.reverse();

    hotspots
}
```

**Target Hotspots (Expected):**

1. **Memory Allocation (`buddy::allocate_pages`):**
   - **Cause:** Linear search through free lists
   - **Fix:** Use bitmask for faster free block finding
   ```rust
   // Before: O(n) scan
   for order in 0..=MAX_ORDER { if free_list[order].is_some() { ... } }

   // After: O(1) bitmap check
   let order = self.free_bitmap.trailing_zeros(); // Find first set bit
   ```

2. **VFS Path Lookup (`vfs::path_lookup`):**
   - **Cause:** String allocations for each component
   - **Fix:** Use stack-allocated buffer
   ```rust
   // Before: allocates for each split
   let components: Vec<&str> = path.split('/').collect();

   // After: stack buffer
   let mut components: [&str; 32] = [""; 32];
   let count = split_path_inplace(path, &mut components);
   ```

3. **Ext4 Block Lookup (`ext4::get_inode_block`):**
   - **Cause:** Extent tree traversal is sequential
   - **Fix:** Cache last accessed extent
   ```rust
   struct Ext4Inode {
       extent_cache: Option<(u64, u64)>, // (logical_block, physical_block)
       // ...
   }

   fn get_inode_block(&self, logical: u64) -> Result<u64> {
       // Check cache first
       if let Some((cached_logical, cached_physical)) = self.extent_cache {
           if logical == cached_logical {
               return Ok(cached_physical);
           }
       }
       // Fall back to extent tree lookup
   }
   ```

**Validation:**
```rust
#[bench]
fn bench_allocate_pages_optimized(b: &mut Bencher) {
    b.iter(|| {
        BUDDY_ALLOCATOR.allocate_pages(0);
    });
}
// Before: ~28µs mean
// After: <15µs mean (target 50% improvement)
```

**Success Metrics:**
- Top 3 hotspots identified via profiling
- Each hotspot optimized with >30% improvement
- Overall kernel performance improved by 15-20%
- Flame graph shows more balanced distribution

---

## Phase 4: Userspace & GUI Enhancement (Weeks 11-13)

### Objective
Add minimal but functional userspace/GUI to showcase AI features.

**SCOPE CONSTRAINT:** Keep minimal - focus on demonstrating AI capabilities, not building full OS.

### 4.1 Basic Process Support (fork/exec stubs)

**Goal:** Enable simple single-threaded userspace processes.

**Technical Specifications:**

**Location:** `crates/kernel/src/proc/` (enhance existing structures)

**Implementation:**
```rust
// crates/kernel/src/syscall/process.rs

pub fn sys_fork() -> Result<Pid> {
    // Simplified fork: copy current task
    let current = current_task();

    // 1. Allocate new PID
    let child_pid = alloc_pid();

    // 2. Clone address space (COW not required for MVP)
    let child_mm = current.mm.shallow_clone();

    // 3. Clone file descriptor table
    let child_fds = current.files.clone();

    // 4. Create child task
    let child = Task {
        pid: child_pid,
        parent: Some(current.pid),
        mm: child_mm,
        files: child_fds,
        state: TaskState::Runnable,
        // Copy registers (child returns 0)
        regs: current.regs.clone_with_retval(0),
        ..current.clone()
    };

    // 5. Add to scheduler
    add_task_to_scheduler(child);

    // Parent returns child PID
    Ok(child_pid)
}

pub fn sys_exec(path: &str, argv: &[&str]) -> Result<!> {
    // Load ELF from path
    let elf_data = vfs::read_file(path)?;
    let elf = parse_elf(&elf_data)?;

    // Replace current address space
    let current = current_task_mut();
    current.mm = create_mm_from_elf(&elf);

    // Set entry point
    current.regs.pc = elf.entry_point;

    // Setup argv on stack
    setup_argv_on_stack(&mut current.mm, argv)?;

    // Jump to entry point (no return)
    jump_to_userspace(current.regs.pc);
}
```

**Minimal ELF Loader:**
```rust
// crates/kernel/src/proc/elf.rs
struct ElfHeader {
    entry_point: u64,
    program_headers: Vec<ProgramHeader>,
}

struct ProgramHeader {
    vaddr: u64,
    file_offset: u64,
    file_size: u64,
    mem_size: u64,
    flags: u32, // Read/Write/Execute
}

fn parse_elf(data: &[u8]) -> Result<ElfHeader> {
    // Parse basic ELF64 header
    // Support only LOAD segments (no dynamic linking)
    // ...
}

fn create_mm_from_elf(elf: &ElfHeader) -> AddressSpace {
    let mm = AddressSpace::new();

    for segment in &elf.program_headers {
        // Allocate pages for segment
        let pages = allocate_pages_for_region(segment.vaddr, segment.mem_size);

        // Copy data from ELF file
        copy_segment_data(pages, segment);

        // Map into address space with correct permissions
        mm.map_region(segment.vaddr, pages, segment.flags);
    }

    mm
}
```

**Test Userspace Program:**
```c
// userspace/hello.c
#include <stdio.h>

int main(int argc, char** argv) {
    printf("Hello from userspace!\n");
    printf("PID: %d\n", getpid());
    return 0;
}
```

**Compile & Load:**
```bash
# Cross-compile for aarch64
aarch64-linux-gnu-gcc -static -o hello hello.c

# Copy to ext4 image
mkdir /tmp/mnt
sudo mount -o loop ext4.img /tmp/mnt
sudo cp hello /tmp/mnt/bin/
sudo umount /tmp/mnt

# Run in kernel
> exec /bin/hello
Hello from userspace!
PID: 42
```

**Shell Command:**
```rust
// crates/kernel/src/shell/shell_procctl.rs
procctl run <path> [args...]  // Execute userspace program
procctl list                   // Show running processes
procctl kill <pid>             // Terminate process
```

**Success Metrics:**
- Can load and execute static ELF binaries
- fork() creates child process (returns correct PIDs)
- exec() replaces current process image
- At least 1 test program runs successfully (hello world)

**CONSTRAINTS:**
- No dynamic linking (static binaries only)
- No multi-threading (single-threaded processes)
- No shared memory (simplified)
- No signals (use exit codes only)

---

### 4.2 GUI AI Dashboard Interactivity

**Goal:** Make AI dashboard interactive - click to run llminfer, view graphs.

**Technical Specifications:**

**Location:** `GUI/apps/dashboard/src/` (enhance Phase 6 GUI)

**Interactive Elements:**

**1. AI Metrics Dashboard:**
```typescript
// GUI/apps/dashboard/src/components/AiDashboard.tsx
import React, { useState, useEffect } from 'react';
import { Line, Gauge } from 'react-chartjs-2';

interface AiMetrics {
  nnInferLatency: { p50: number; p99: number; history: number[] };
  crashPrediction: { confidence: number; accuracy: number };
  schedulerImprovement: number;
  memorySavedMb: number;
}

export const AiDashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<AiMetrics | null>(null);
  const [inferInput, setInferInput] = useState('');
  const [inferResult, setInferResult] = useState('');

  useEffect(() => {
    // Poll metrics every 1 second
    const interval = setInterval(async () => {
      const response = await fetch('/api/ai/metrics');
      setMetrics(await response.json());
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const runInference = async () => {
    const response = await fetch('/api/llm/infer', {
      method: 'POST',
      body: JSON.stringify({ prompt: inferInput }),
    });
    const result = await response.json();
    setInferResult(result.output);
  };

  if (!metrics) return <div>Loading...</div>;

  return (
    <div className="ai-dashboard">
      <h2>AI Performance</h2>

      {/* NN Inference Latency Chart */}
      <div className="chart-container">
        <h3>NN Inference Latency (µs)</h3>
        <Line
          data={{
            labels: Array.from({ length: metrics.nnInferLatency.history.length }, (_, i) => i),
            datasets: [{
              label: 'Latency',
              data: metrics.nnInferLatency.history,
              borderColor: 'rgb(75, 192, 192)',
            }],
          }}
          options={{
            animation: false,
            scales: { y: { beginAtZero: true } },
          }}
        />
      </div>

      {/* Crash Prediction Gauge */}
      <div className="gauge-container">
        <h3>Crash Prediction Confidence</h3>
        <Gauge
          value={metrics.crashPrediction.confidence}
          max={100}
          color={metrics.crashPrediction.confidence > 80 ? 'red' : 'green'}
        />
        <p>Accuracy: {metrics.crashPrediction.accuracy.toFixed(1)}%</p>
      </div>

      {/* LLM Inference Interface */}
      <div className="llm-interface">
        <h3>LLM State Inference</h3>
        <textarea
          value={inferInput}
          onChange={(e) => setInferInput(e.target.value)}
          placeholder="Ask about system state..."
          rows={3}
        />
        <button onClick={runInference}>Run Inference</button>
        {inferResult && (
          <div className="inference-result">
            <strong>Result:</strong>
            <pre>{inferResult}</pre>
          </div>
        )}
      </div>

      {/* Metrics Cards */}
      <div className="metrics-grid">
        <MetricCard
          title="Scheduler Improvement"
          value={`${metrics.schedulerImprovement.toFixed(1)}%`}
          icon="⚡"
        />
        <MetricCard
          title="Memory Saved"
          value={`${metrics.memorySavedMb} MB`}
          icon="💾"
        />
      </div>
    </div>
  );
};
```

**2. Backend API Endpoints:**
```typescript
// GUI/apps/daemon/src/api/ai.ts
import { Router } from 'express';
import { kernelClient } from '../kernel-client';

const router = Router();

router.get('/api/ai/metrics', async (req, res) => {
  // Fetch from kernel via QMP or serial
  const metrics = await kernelClient.executeCommand('autoctl ai-metrics --json');
  res.json(JSON.parse(metrics));
});

router.post('/api/llm/infer', async (req, res) => {
  const { prompt } = req.body;

  // Execute llmctl infer via kernel
  const result = await kernelClient.executeCommand(
    `llmctl infer "${prompt}"`
  );

  res.json({ output: result });
});

router.post('/api/ai/run-command', async (req, res) => {
  const { command } = req.body;

  // Execute suggested command (with safety checks)
  const result = await kernelClient.executeCommand(command);
  res.json({ result });
});

export default router;
```

**3. Real-Time Graph Updates:**
```typescript
// Use WebSocket for lower latency
import { WebSocketServer } from 'ws';

const wss = new WebSocketServer({ port: 8080 });

wss.on('connection', (ws) => {
  // Send metrics every 500ms
  const interval = setInterval(async () => {
    const metrics = await fetchAiMetrics();
    ws.send(JSON.stringify({ type: 'metrics', data: metrics }));
  }, 500);

  ws.on('close', () => clearInterval(interval));
});
```

**Success Metrics:**
- Dashboard updates in real-time (<1s latency)
- User can run LLM inference from GUI
- Graphs show live data (last 5 minutes)
- Click-to-execute for suggested commands

---

### 4.3 Ext4 File Manager Demo (GUI)

**Goal:** Visual demonstration of ext4 write support with crash/recovery.

**Technical Specifications:**

**Location:** `GUI/apps/filemanager/` (new app)

**UI Components:**
```typescript
// GUI/apps/filemanager/src/FileManager.tsx
import React, { useState, useEffect } from 'react';

interface FileEntry {
  name: string;
  size: number;
  isDirectory: boolean;
}

export const FileManager: React.FC = () => {
  const [currentPath, setCurrentPath] = useState('/incidents');
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string>('');

  useEffect(() => {
    loadDirectory(currentPath);
  }, [currentPath]);

  const loadDirectory = async (path: string) => {
    const response = await fetch(`/api/fs/ls?path=${encodeURIComponent(path)}`);
    const data = await response.json();
    setFiles(data.entries);
  };

  const createFile = async () => {
    const filename = prompt('Enter filename:');
    if (!filename) return;

    await fetch('/api/fs/create', {
      method: 'POST',
      body: JSON.stringify({
        path: `${currentPath}/${filename}`,
        content: 'New file created via GUI',
      }),
    });

    loadDirectory(currentPath);
  };

  const deleteFile = async (filename: string) => {
    if (!confirm(`Delete ${filename}?`)) return;

    await fetch('/api/fs/delete', {
      method: 'POST',
      body: JSON.stringify({ path: `${currentPath}/${filename}` }),
    });

    loadDirectory(currentPath);
  };

  const simulateCrash = async () => {
    if (!confirm('This will crash the kernel! Continue?')) return;

    // Start write operation
    const writePromise = fetch('/api/fs/create', {
      method: 'POST',
      body: JSON.stringify({
        path: `${currentPath}/crash-test.dat`,
        content: 'X'.repeat(100000), // Large file
      }),
    });

    // Crash after 500ms
    setTimeout(async () => {
      await fetch('/api/kernel/crash', { method: 'POST' });
    }, 500);

    // Wait for kernel to reboot
    setTimeout(() => {
      alert('Kernel rebooted. Check if file is intact.');
      loadDirectory(currentPath);
    }, 5000);
  };

  return (
    <div className="file-manager">
      <div className="toolbar">
        <button onClick={createFile}>New File</button>
        <button onClick={simulateCrash} className="danger">
          Simulate Crash During Write
        </button>
      </div>

      <div className="breadcrumb">
        Current: {currentPath}
      </div>

      <div className="file-list">
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Size</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {files.map((file) => (
              <tr key={file.name}>
                <td>
                  {file.isDirectory ? '📁' : '📄'} {file.name}
                </td>
                <td>{file.size} bytes</td>
                <td>
                  <button onClick={() => deleteFile(file.name)}>Delete</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
```

**Backend API:**
```typescript
// GUI/apps/daemon/src/api/filesystem.ts
router.get('/api/fs/ls', async (req, res) => {
  const { path } = req.query;
  const output = await kernelClient.executeCommand(`ls ${path}`);

  // Parse ls output
  const entries = output.split('\n')
    .filter(line => line.trim())
    .map(line => ({
      name: line.split(/\s+/)[0],
      size: parseInt(line.split(/\s+/)[1]) || 0,
      isDirectory: line.includes('DIR'),
    }));

  res.json({ entries });
});

router.post('/api/fs/create', async (req, res) => {
  const { path, content } = req.body;

  // Write file via kernel
  await kernelClient.writeFile(path, content);

  res.json({ success: true });
});

router.post('/api/fs/delete', async (req, res) => {
  const { path } = req.body;

  await kernelClient.executeCommand(`rm ${path}`);

  res.json({ success: true });
});

router.post('/api/kernel/crash', async (req, res) => {
  // Send QMP command to hard reset QEMU
  await kernelClient.qmpCommand('system_reset');
  res.json({ success: true });
});
```

**Demo Scenario:**
1. User creates file "test.json" via GUI
2. File appears in list immediately
3. User clicks "Simulate Crash During Write"
4. Kernel crashes mid-write
5. Kernel auto-reboots (journal replay happens)
6. User refreshes file list
7. File either exists (write completed) or doesn't (write rolled back)
8. **Key:** No corruption, filesystem is clean (validated by fsck)

**Success Metrics:**
- File operations (create/delete) work via GUI
- Crash simulation triggers kernel reset
- 100% filesystem integrity after crash (fsck clean)
- Visual confirmation of journal replay working

---

## Phase 5: Documentation & Polish (Weeks 14-15)

### Objective
Make the project accessible, professional, and ready for external visibility.

### 5.1 Quick-Start Guide (5-Minute Demo)

**Goal:** New user sees AI in action within 5 minutes.

**Technical Specifications:**

**Location:** `docs/guides/QUICKSTART.md` (new file)

**Content:**
```markdown
# SIS Kernel Quick Start (5 Minutes)

Get the AI-native kernel running and see intelligent scheduling in action.

## Prerequisites

- macOS/Linux with 8GB RAM
- QEMU installed (`brew install qemu` or `apt install qemu-system-aarch64`)

## Step 1: Clone & Build (2 minutes)

```bash
git clone https://github.com/amoljassal/sis-kernel-showcase.git
cd sis-kernel-showcase
./scripts/uefi_run.sh build
```

## Step 2: Boot Kernel (30 seconds)

```bash
BRINGUP=1 SIS_FEATURES="llm,ai-ops" ./scripts/uefi_run.sh
```

Expected output:
```
SIS Kernel v1.0 [AI-Native]
VFS: MOUNT EXT4 AT /models (rw+journal)
AI: NN inference ready (45µs avg)
Shell ready. Type 'help' for commands.
>
```

## Step 3: See AI Scheduling (1 minute)

```bash
# Enable AI scheduler
> schedctl transformer on
Transformer scheduler enabled

# Show real-time metrics
> autoctl ai-metrics
AI Performance:
  NN Inference: 42µs (p50), 89µs (p99)
  Decision Follow Rate: 87.3%
  Context Switch Improvement: 18.5%

# Watch autonomous decisions
> autoctl on
[Autonomy ON - AI making real-time scheduling decisions]
```

## Step 4: Test Crash Prediction (1 minute)

```bash
# Trigger memory stress
> stresstest mem --pressure high

# Watch AI predict crash
> crashctl status
Crash Prediction: 78% confidence (memory fragmentation increasing)
Recommendation: Run 'memctl compact'

# AI auto-triggers compaction
[AI] Executing: memctl compact
Memory compacted: 247MB freed
```

## Step 5: Export Decision Traces (30 seconds)

```bash
# View recent AI decisions
> tracectl list --recent 10

# Export to file
> tracectl export --recent 10 --path /incidents/demo.json
Exported 10 traces to /incidents/demo.json

# Verify file persisted to ext4
> ls /incidents
demo.json  (2.4 KB)
```

## Next Steps

- **Tutorial:** [Understanding AI Scheduling](./AI-SCHEDULING-TUTORIAL.md)
- **Advanced:** [Train Custom Model](./MODEL-TRAINING.md)
- **Testing:** [Run Full Test Suite](./TESTING-GUIDE.md)

## Troubleshooting

**Kernel doesn't boot:**
- Check QEMU version: `qemu-system-aarch64 --version` (need ≥7.0)
- Try without AI: `BRINGUP=1 ./scripts/uefi_run.sh`

**AI metrics show 0%:**
- Wait 30 seconds for warmup
- Run workload: `stresstest cpu --duration 10`
```

**Integration:**
- Add to main README.md (link in "Getting Started" section)
- Include screenshot/GIF of terminal output
- Test with fresh VM to ensure accuracy

**Success Metrics:**
- New user can run demo in <5 minutes
- All commands execute without errors
- AI features are clearly visible (not buried in logs)

---

### 5.2 Tutorial Series

**Goal:** Three in-depth tutorials explaining core AI features.

**Technical Specifications:**

**Tutorial 1: `docs/guides/AI-SCHEDULING-TUTORIAL.md`**
```markdown
# Understanding AI-Native Scheduling

Learn how the transformer-based scheduler makes intelligent task prioritization decisions.

## How Traditional Schedulers Work

Traditional schedulers (CFS, FIFO, etc.) use static algorithms:
```c
priority = base_priority - (cpu_time_used / total_cpu_time)
```

Problems:
- No adaptation to workload patterns
- Can't predict future behavior
- Fixed heuristics

## How SIS AI Scheduler Works

### 1. Task Embedding

Each task is represented as a 4D vector:
```
[normalized_priority, cpu_ratio, io_ratio, cache_score]

Example:
Task A (CPU-intensive): [0.8, 0.95, 0.05, 0.3]
Task B (I/O-intensive):  [0.6, 0.15, 0.85, 0.6]
```

### 2. Attention Mechanism

The scheduler compares current task against historical patterns:

```
Attention(Q, K, V) = softmax(Q·K^T / sqrt(d)) · V

Where:
Q = Current task embedding
K = Past task embeddings
V = Past scheduling outcomes (success scores)
```

If current task looks like past high-priority task → boost priority
If current task looks like past long-running task → deprioritize

### 3. Online Learning

After each scheduling decision:
- Measure outcome (context switches, completion time)
- Update weights using gradient descent
- Converges to optimal policy over time

## Hands-On: Comparing Schedulers

### Experiment 1: CPU-Bound Workload

```bash
# Baseline scheduler
> schedctl transformer off
> stresstest cpu --tasks 10 --duration 30
Result: 342 context switches, 28.5s total time

# AI scheduler
> schedctl transformer on
> stresstest cpu --tasks 10 --duration 30
Result: 271 context switches, 24.1s total time
Improvement: 21% fewer switches, 15% faster
```

### Experiment 2: Mixed Workload

```bash
# Run CPU + I/O tasks simultaneously
> schedctl transformer on
> stresstest mixed --cpu-tasks 5 --io-tasks 5 --duration 60

# Watch AI adapt in real-time
> autoctl ai-metrics
[Graph shows scheduler learning task patterns over 60 seconds]
```

## Tuning the Scheduler

### Adjust Learning Rate

```bash
# Slower learning (more stable)
> schedctl set-lr 0.0001

# Faster learning (adapts quicker but may overshoot)
> schedctl set-lr 0.01
```

### Reset Weights

```bash
# If scheduler performs poorly, reset to defaults
> schedctl reset-weights
Weights reinitialized. Learning from scratch.
```

## Debugging

### View Attention Weights

```bash
> schedctl stats --verbose
Attention Weights:
  Query:  [[0.23, 0.45, 0.12, 0.67], ...]
  Key:    [[0.34, 0.21, 0.78, 0.43], ...]
  Value:  [[0.56, 0.89, 0.23, 0.12], ...]

Top 3 Tasks by Attention Score:
  1. Task 42 (score=0.87) - CPU-bound, high priority
  2. Task 15 (score=0.73) - I/O-bound, medium priority
  3. Task 8  (score=0.61) - Mixed, low priority
```

## Advanced: Custom Scheduling Policies

Extend the scheduler with custom policies:

```rust
// crates/kernel/src/sched/transformer_sched.rs

impl TransformerScheduler {
    pub fn add_custom_policy(&mut self, policy: SchedulingPolicy) {
        // Example: Boost priority for real-time tasks
        if policy.real_time {
            self.priority_boost = 2.0;
        }
    }
}
```

## References

- [Attention Is All You Need (Vaswani et al.)](https://arxiv.org/abs/1706.03762)
- [AI-based OS Scheduling (Survey)](https://arxiv.org/abs/...)
- [LoRA: Low-Rank Adaptation](https://arxiv.org/abs/2106.09685)
```

**Tutorial 2: `docs/guides/DECISION-TRACING-TUTORIAL.md`**
(Similar structure: background → how it works → hands-on → debugging)

**Tutorial 3: `docs/guides/EXT4-JOURNALING-TUTORIAL.md`**
(Focus on crash recovery, journal replay, forensics)

**Success Metrics:**
- Each tutorial is self-contained (15-30 min read)
- Includes code examples and shell commands
- Has "Try It Yourself" sections
- Links to relevant source files

---

### 5.3 GitHub Repository Polish

**Goal:** Professional presentation for external visibility.

**Technical Specifications:**

**1. Badges (README.md top):**
```markdown
# SIS Kernel

[![Build Status](https://github.com/amoljassal/sis-kernel-showcase/workflows/CI/badge.svg)](...)
[![Code Coverage](https://codecov.io/gh/amoljassal/sis-kernel-showcase/branch/main/graph/badge.svg)](...)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](...)

> The first production-grade AI-native kernel with real-time ML inference and autonomous decision-making.
```

**2. Issue Templates:**
```yaml
# .github/ISSUE_TEMPLATE/bug_report.yml
name: Bug Report
description: Report a bug in SIS Kernel
labels: ["bug", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for reporting! Please provide details below.

  - type: input
    id: kernel-version
    attributes:
      label: Kernel Version
      description: Output of `uname -a` in SIS shell
      placeholder: "SIS Kernel v1.0.0-rc1"
    validations:
      required: true

  - type: textarea
    id: reproduction
    attributes:
      label: Steps to Reproduce
      description: Shell commands to reproduce the issue
      placeholder: |
        1. Boot kernel: BRINGUP=1 ./scripts/uefi_run.sh
        2. Run: tracectl export --all
        3. Observe: kernel panic
    validations:
      required: true

  - type: textarea
    id: logs
    attributes:
      label: Kernel Logs
      description: Paste relevant output
      render: shell
```

**3. Pull Request Template:**
```markdown
# .github/pull_request_template.md

## Description

<!-- What does this PR do? -->

## Type of Change

- [ ] Bug fix (non-breaking change)
- [ ] New feature (non-breaking change)
- [ ] Breaking change
- [ ] Documentation update

## Testing

- [ ] Compiled without errors
- [ ] Ran test suite: `cargo test`
- [ ] Tested in QEMU: `./scripts/uefi_run.sh`
- [ ] Added new tests for feature

## Checklist

- [ ] Code follows project style
- [ ] Commit messages follow convention
- [ ] Updated documentation (if applicable)
- [ ] No new compiler warnings

## Related Issues

Fixes #(issue number)
```

**4. Contributing Guide:**
```markdown
# CONTRIBUTING.md

# Contributing to SIS Kernel

We welcome contributions! Here's how to get started.

## Development Setup

1. **Install dependencies:**
   ```bash
   # Rust toolchain
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add aarch64-unknown-none

   # QEMU
   brew install qemu  # macOS
   apt install qemu-system-aarch64  # Linux
   ```

2. **Build and test:**
   ```bash
   cargo build --release
   cargo test
   ./scripts/uefi_run.sh build
   ```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common issues
- Follow existing patterns in codebase

## Commit Messages

Use conventional commits:
```
feat(scheduler): add transformer-based task selection
fix(ext4): resolve deadlock in allocate_block
docs(readme): update quickstart guide
```

## Testing Requirements

- Unit tests for new functions
- Integration tests for new features
- QEMU smoke test for kernel changes

## Pull Request Process

1. Create feature branch: `git checkout -b feature/my-feature`
2. Make changes and commit
3. Push and open PR on GitHub
4. Address review feedback
5. Wait for CI to pass
6. Maintainer will merge

## Areas Needing Help

- [ ] Fuzzing corpus expansion
- [ ] ARM64 real hardware testing
- [ ] GUI accessibility improvements
- [ ] Performance benchmarking on different QEMU configs

## Questions?

Open a discussion on GitHub or ping maintainers.
```

**5. CI/CD Workflow:**
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: aarch64-unknown-none

      - name: Build kernel
        run: cargo build --release

      - name: Run tests
        run: cargo test

      - name: Check formatting
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy -- -D warnings

  qemu-smoke-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install QEMU
        run: sudo apt-get install -y qemu-system-aarch64

      - name: Build and boot
        run: |
          ./scripts/uefi_run.sh build
          timeout 60 ./scripts/uefi_run.sh || true

      - name: Check for boot success
        run: grep "Shell ready" qemu_output.log
```

**Success Metrics:**
- All badges show passing status
- Issue templates guide users to provide needed info
- PR template ensures quality contributions
- CI runs on every PR and catches regressions

---

### 5.4 Performance Comparison Report

**Goal:** Publish benchmark results showing AI improvements.

**Technical Specifications:**

**Location:** `docs/results/PERFORMANCE-COMPARISON.md` (new file)

**Content:**
```markdown
# SIS Kernel Performance Comparison

Benchmarks comparing baseline (traditional) vs AI-enhanced kernel.

**Test Environment:**
- QEMU 8.1.0 (aarch64)
- Host: macOS 14.2, M1 Pro, 16GB RAM
- Kernel: SIS v1.0.0
- Date: 2025-01-15

---

## Scheduler Performance

### Context Switch Latency

| Metric | Baseline | AI (Transformer) | Improvement |
|--------|----------|------------------|-------------|
| Mean   | 1,250 ns | 980 ns           | **21.6%** ↓ |
| P50    | 1,100 ns | 900 ns           | **18.2%** ↓ |
| P99    | 2,800 ns | 2,100 ns         | **25.0%** ↓ |

**Graph:** Context switch latency distribution
![Context Switch Latency](../assets/ctx_switch_latency.png)

### Task Throughput (tasks/sec)

| Workload Type | Baseline | AI (Transformer) | Improvement |
|---------------|----------|------------------|-------------|
| CPU-bound     | 1,450    | 1,720            | **18.6%** ↑ |
| I/O-bound     | 2,100    | 2,380            | **13.3%** ↑ |
| Mixed         | 1,680    | 1,950            | **16.1%** ↑ |

---

## Memory Management

### Allocation Latency

| Allocator | Mean (ns) | P99 (ns) | Improvement |
|-----------|-----------|----------|-------------|
| Buddy (baseline) | 28,000 | 45,000 | - |
| Slab (small obj) | 950 | 1,800 | **96.6%** ↓ |
| Buddy + Slab | 18,500 | 32,000 | **28.9%** ↓ |

**Graph:** Allocation latency by size
![Allocation Latency](../assets/alloc_latency_by_size.png)

### Fragmentation Reduction

| Scenario | Baseline Fragmentation | With AI Compaction | Improvement |
|----------|------------------------|---------------------|-------------|
| After 1 hour | 0.73 | 0.48 | **34.2%** ↓ |
| After 24 hours | 0.89 | 0.61 | **31.5%** ↓ |

---

## AI Inference Performance

### NN Inference Latency

| Metric | Value |
|--------|-------|
| Mean   | 45 µs |
| P50    | 42 µs |
| P99    | 89 µs |

**Target:** <100 µs (✅ achieved)

### Transformer Scheduler Inference

| Metric | Value |
|--------|-------|
| Mean   | 38 µs |
| P50    | 35 µs |
| P99    | 72 µs |

**Target:** <50 µs (✅ achieved for p50)

---

## Filesystem Performance

### Ext4 Throughput

| Operation | Baseline ext2 (ro) | Ext4 (rw+journal) | Delta |
|-----------|--------------------|-------------------|-------|
| Sequential Read | 85 MB/s | 82 MB/s | -3.5% |
| Sequential Write | N/A | 67 MB/s | N/A |
| Random Read | 12 MB/s | 11 MB/s | -8.3% |
| Random Write | N/A | 9 MB/s | N/A |

**Note:** Write overhead is due to journaling (acceptable trade-off for durability).

### Journal Commit Latency

| Metric | Value |
|--------|-------|
| Mean   | 4.2 ms |
| P99    | 12.8 ms |

---

## VirtIO Performance

### Block Device (after tuning)

| Metric | Before Tuning | After Tuning | Improvement |
|--------|---------------|--------------|-------------|
| Throughput | 68 MB/s | 103 MB/s | **51.5%** ↑ |
| Latency (p50) | 2.1 ms | 1.4 ms | **33.3%** ↓ |

**Optimizations:**
- Increased queue size: 16 → 128
- Enabled descriptor chaining
- Interrupt coalescing

### Network Device

| Metric | Before Tuning | After Tuning | Improvement |
|--------|---------------|--------------|-------------|
| Throughput | 320 Mbps | 485 Mbps | **51.6%** ↑ |
| Latency (p50) | 1.8 ms | 1.2 ms | **33.3%** ↓ |

---

## Overall System Performance

### Boot Time

| Phase | Duration |
|-------|----------|
| UEFI → Kernel Entry | 1.2 s |
| Kernel Init | 0.8 s |
| VFS Mount | 0.3 s |
| AI Subsystems Init | 0.5 s |
| **Total** | **2.8 s** |

### Power Estimation

| Workload | Estimated Power (W) |
|----------|---------------------|
| Idle | 1.2 W |
| Light (browsing) | 2.8 W |
| Medium (compilation) | 5.4 W |
| Heavy (AI inference) | 7.9 W |

**Note:** QEMU simulation, not real hardware measurements.

---

## Crash Prediction Accuracy

| Metric | Value |
|--------|-------|
| True Positives | 42 / 50 |
| False Positives | 3 / 50 |
| False Negatives | 5 / 50 |
| **Accuracy** | **84.0%** |

**Success Criteria:** ≥80% accuracy (✅ achieved)

---

## Conclusion

The AI-enhanced SIS kernel demonstrates **measurable improvements** across all major subsystems:

- **Scheduling:** 21.6% reduction in context switch latency
- **Memory:** 34.2% reduction in fragmentation
- **I/O:** 51.5% increase in block throughput
- **Predictability:** 84% crash prediction accuracy

These results validate the AI-native approach to kernel design.

---

## Reproduction

To reproduce these benchmarks:

```bash
# Run full benchmark suite
./scripts/run_benchmarks.sh

# Generate report
python3 tools/generate_report.py benchmarks/results/ > PERFORMANCE-COMPARISON.md
```

Raw data: `benchmarks/results/raw_data.json`
```

**Success Metrics:**
- Report shows ≥15% improvement in at least 3 categories
- All benchmarks are reproducible
- Graphs/visualizations included
- Raw data available for verification

---

## Implementation Roadmap

| Phase | Duration | Deliverables | Priority |
|-------|----------|--------------|----------|
| **Phase 1: AI/ML Innovation** | Weeks 1-4 | Crash prediction, transformer scheduler, LLM fine-tuning, AI dashboard | 🔴 Critical |
| **Phase 2: Testing & Validation** | Weeks 5-7 | Ext4 crash testing, VFS fuzzing, Kani verification, EU compliance | 🟠 High |
| **Phase 3: Performance** | Weeks 8-10 | Slab allocator, VirtIO tuning, power estimation, profiling | 🟡 Medium |
| **Phase 4: Userspace & GUI** | Weeks 11-13 | Basic processes, GUI interactivity, file manager demo | 🟢 Low |
| **Phase 5: Documentation** | Weeks 14-15 | Quickstart, tutorials, GitHub polish, benchmarks | 🔴 Critical |

**Total Duration:** 15 weeks

---

## Success Criteria

### Phase 1 (AI/ML)
- [ ] Crash prediction accuracy ≥80%
- [ ] Transformer scheduler shows ≥10% improvement
- [ ] LLM fine-tuning completes in <30s
- [ ] AI dashboard updates in real-time (<1s)

### Phase 2 (Testing)
- [ ] 100% ext4 crash recovery rate (50 scenarios)
- [ ] Zero VFS panics after 24h fuzzing
- [ ] All Kani proofs pass (5 properties)
- [ ] EU AI Act compliance ≥92%

### Phase 3 (Performance)
- [ ] Slab allocator <1µs for small objects
- [ ] VirtIO throughput +50%
- [ ] Power estimation within 15% accuracy
- [ ] Top 3 hotspots optimized (>30% each)

### Phase 4 (Userspace)
- [ ] Can execute static ELF binaries
- [ ] GUI dashboard interactive
- [ ] File manager demonstrates crash recovery

### Phase 5 (Documentation)
- [ ] New user completes quickstart in <5min
- [ ] All 3 tutorials complete
- [ ] CI shows green badges
- [ ] Performance report published

---

## Technical Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| AI models too large for kernel | High | Use quantized models, LoRA adapters |
| Inference latency exceeds budget | High | Profile and optimize hot paths, use NEON |
| Fuzzing finds critical bugs | Medium | Fix immediately, add regression tests |
| QEMU limitations prevent benchmarks | Medium | Document limitations, test on real hardware later |
| Userspace complexity explodes scope | High | **Keep minimal:** static binaries only, no threading |
| Documentation becomes stale | Low | Auto-generate from tests where possible |

---

## Dependencies

- Rust toolchain 1.75+
- QEMU 7.0+
- cargo-fuzz (for fuzzing)
- Kani verifier (for formal verification)
- Python 3.9+ (for scripting)
- Node.js 18+ (for GUI)

---

## Handoff to AI Agent

**Instructions for Implementation:**

1. **Read this entire plan** before starting any phase
2. **Follow technical specifications exactly** - they include:
   - File locations
   - Function signatures
   - Algorithm details
   - Integration points
3. **Create branch per phase:** `feature/phase-1-ai-innovation`, etc.
4. **Commit frequently** with descriptive messages
5. **Run tests after each major change**
6. **Document any deviations** from the plan in commit messages
7. **Push to GitHub** when phase is complete
8. **Notify for integration** - provide branch link

**Testing Before Handoff:**
- Code compiles without errors
- No new Clippy warnings
- Existing tests still pass
- New features have basic tests

**Questions/Blockers:**
- Document in `docs/implementation/AI_NATIVE_ENHANCEMENTS.md` in branch
- Flag critical blockers that need human decision

---

## Integration Checklist

When integrating completed phases:

- [ ] Review all code changes
- [ ] Run full test suite
- [ ] Boot kernel in QEMU and smoke test
- [ ] Check performance benchmarks (no regressions)
- [ ] Update README if needed
- [ ] Merge to main branch
- [ ] Tag release (if major milestone)

---

**Document Version:** 1.0
**Created:** 2025-01-15
**Last Updated:** 2025-01-15
**Status:** Ready for Implementation
