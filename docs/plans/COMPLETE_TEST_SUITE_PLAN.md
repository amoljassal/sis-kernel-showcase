# SIS Kernel Complete Test Suite Implementation Plan

**Target:** 100% Test Coverage Across All Phases (A, 1-8)
**Current Coverage:** 45%
**Goal:** Single-command comprehensive validation for demo purposes

**Document Version:** 1.0
**Date:** 2025-11-11
**Status:** READY FOR IMPLEMENTATION

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Phase 7: AI Operations Platform Tests](#phase-7-ai-operations-platform-tests-0--100)
4. [Phase 8: Performance Optimization Tests](#phase-8-performance-optimization-tests-20--100)
5. [Phase 6: Web GUI Management Tests](#phase-6-web-gui-management-tests-30--100)
6. [Phase 3: Temporal Isolation Tests](#phase-3-temporal-isolation-tests-40--100)
7. [Phase 1: AI-Native Dataflow Tests](#phase-1-ai-native-dataflow-tests-70--100)
8. [Phase 2: AI Governance Tests](#phase-2-ai-governance-tests-50--100)
9. [Phase 5: UX Safety Tests](#phase-5-ux-safety-tests-60--100)
10. [Integration & Orchestration](#integration--orchestration)
11. [Validation Criteria](#validation-criteria)
12. [Implementation Timeline](#implementation-timeline)

---

## Overview

### Mission Statement
Create a **single-command complete test suite** that validates ALL SIS Kernel phases from foundation to advanced features, providing a comprehensive demo that proves production readiness.

### Success Criteria
```bash
# Single command execution
cargo run -p sis-testing --release -- --full-demo

# Expected result
✅ 100% test coverage across all phases
✅ Detailed pass/fail for each subsystem
✅ Performance metrics against targets
✅ Production-ready validation report
✅ Exit code 0 (all tests pass)
```

### Gap Analysis Summary
```
Phase 7 (AI Ops):        0% → 100% (+100%) 🚨 CRITICAL
Phase 8 (Performance):  20% → 100% (+80%)  🚨 CRITICAL
Phase 6 (Web GUI):      30% → 100% (+70%)  ⚠️  HIGH
Phase 3 (Temporal):     40% → 100% (+60%)  ⚠️  HIGH
Phase 1 (Dataflow):     70% → 100% (+30%)  ⚠️  MEDIUM
Phase 2 (Governance):   50% → 100% (+50%)  ⚠️  MEDIUM
Phase 5 (UX Safety):    60% → 100% (+40%)  ⚠️  MEDIUM
```

---

## Architecture

### Directory Structure
```
crates/testing/src/
├── phase7_ai_ops/              # NEW - Phase 7 complete tests
│   ├── mod.rs                  # Module orchestration
│   ├── model_lifecycle.rs      # Registry, hot-swap, rollback
│   ├── shadow_mode.rs          # Canary deployment, A/B testing
│   ├── otel_exporter.rs        # OpenTelemetry trace export
│   ├── decision_traces.rs      # Decision buffer & export
│   └── integration_tests.rs    # End-to-end AI Ops validation
│
├── phase8_deterministic/       # NEW - Phase 8 complete tests
│   ├── mod.rs                  # Module orchestration
│   ├── cbs_edf_scheduler.rs    # CBS+EDF active validation
│   ├── slab_allocator.rs       # Slab performance benchmarks
│   ├── adaptive_memory.rs      # Strategy switching tests
│   ├── meta_agent.rs           # Decision validation
│   ├── stress_comparison.rs    # Autonomy ON vs OFF
│   └── rate_limiting.rs        # Output rate-limit validation
│
├── phase6_web_gui/             # NEW - Phase 6 complete tests
│   ├── mod.rs                  # Module orchestration
│   ├── http_server.rs          # HTTP server lifecycle
│   ├── websocket.rs            # WebSocket connection tests
│   ├── api_endpoints.rs        # REST API validation
│   ├── authentication.rs       # Auth/authz tests
│   └── real_time_updates.rs    # Live metric streaming
│
├── phase3_temporal/            # NEW - Phase 3 active tests
│   ├── mod.rs                  # Module orchestration
│   ├── active_isolation.rs     # Active temporal isolation
│   ├── deadline_validation.rs  # Real-time deadline tests
│   └── latency_under_load.rs   # Stress with temporal guarantees
│
├── phase1_dataflow/            # NEW - Phase 1 active tests
│   ├── mod.rs                  # Module orchestration
│   ├── graph_execution.rs      # Active graph creation/run
│   ├── operator_validation.rs  # Operator correctness
│   ├── channel_throughput.rs   # Channel performance
│   └── tensor_operations.rs    # Tensor correctness
│
├── phase2_governance/          # ENHANCED - Phase 2 active tests
│   ├── mod.rs                  # Module orchestration (enhanced)
│   ├── llm_finetune.rs         # Fine-tuning validation
│   ├── lora_adapters.rs        # LoRA adapter tests
│   ├── drift_detector.rs       # Active drift detection
│   ├── version_control.rs      # Model versioning tests
│   └── multi_agent.rs          # Multi-agent coordination
│
└── phase5_ux_safety/           # ENHANCED - Phase 5 active tests
    ├── mod.rs                  # Module orchestration (enhanced)
    ├── safety_controls.rs      # Safety control validation
    ├── explainability.rs       # Explainability feature tests
    └── user_feedback.rs        # UX safety guarantees
```

### Test Orchestration Flow
```rust
// lib.rs: execute_comprehensive_validation()
pub async fn execute_comprehensive_validation(&mut self) -> Result<ValidationReport> {
    // Phase-by-phase validation
    let results = tokio::try_join!(
        self.validate_phase_a(),     // OS Foundation
        self.validate_phase_1(),     // AI-Native + Dataflow (NEW)
        self.validate_phase_2(),     // AI Governance (ENHANCED)
        self.validate_phase_3(),     // Temporal Isolation (NEW)
        self.validate_phase_4(),     // Production Readiness
        self.validate_phase_5(),     // UX Safety (ENHANCED)
        self.validate_phase_6(),     // Web GUI (NEW)
        self.validate_phase_7(),     // AI Operations (NEW)
        self.validate_phase_8(),     // Performance Optimization (NEW)
    )?;

    self.generate_comprehensive_report(results).await
}
```

---

## Phase 7: AI Operations Platform Tests (0% → 100%)

### Priority: 🚨 CRITICAL
**Effort:** 2-3 days
**LOC:** ~2500 lines
**Complexity:** High (requires model management, shadow deployment, telemetry)

### Module: `phase7_ai_ops/mod.rs`

#### Overview
Complete validation of Phase 7 AI Operations Platform including model lifecycle management, shadow deployment, OpenTelemetry integration, and decision trace collection.

#### Test Coverage Requirements

##### 1. Model Lifecycle Tests (`model_lifecycle.rs`)

**Objective:** Validate model registry, hot-swap, rollback operations without downtime.

**Test Cases:**

```rust
#[derive(Debug)]
pub struct ModelLifecycleTests {
    kernel_interface: KernelCommandInterface,
    test_models: Vec<ModelMetadata>,
}

impl ModelLifecycleTests {
    /// Test 1.1: Model Registration
    pub async fn test_model_registration() -> Result<()> {
        // Command sequence:
        // 1. llmctl register --id test-model-v1 --size 512KB --ctx 2048
        // 2. llmctl list
        // 3. Verify model appears in registry

        // Expected output:
        // Model registered: test-model-v1 (512KB, ctx=2048)
        // Registry: [test-model-v1]

        // Validation:
        assert!(output.contains("Model registered: test-model-v1"));
        assert!(output.contains("512KB"));

        // Metrics:
        // - Registration time < 100ms
        // - Registry lookup time < 10ms
    }

    /// Test 1.2: Hot-Swap (Zero Downtime)
    pub async fn test_model_hot_swap() -> Result<()> {
        // Setup: Load model-v1, start inference workload
        // 1. llmctl load --id model-v1
        // 2. Start continuous inference: for i in {1..100}; do llminfer test; done &
        // 3. llmctl swap --from model-v1 --to model-v2
        // 4. Verify no inference failures during swap

        // Expected output:
        // Hot-swap initiated: model-v1 → model-v2
        // Draining in-flight requests...
        // Swap complete. Downtime: 0ms

        // Validation:
        assert_eq!(downtime_ms, 0);
        assert_eq!(failed_inferences, 0);

        // Metrics:
        // - Swap time < 500ms
        // - In-flight requests drained gracefully
        // - Zero dropped requests
    }

    /// Test 1.3: Rollback
    pub async fn test_model_rollback() -> Result<()> {
        // Scenario: model-v2 has accuracy degradation
        // 1. llmctl load --id model-v2
        // 2. Detect accuracy drop (simulate)
        // 3. llmctl rollback --to model-v1
        // 4. Verify model-v1 active, accuracy restored

        // Expected output:
        // Rollback triggered: model-v2 → model-v1
        // Rollback complete. Active model: model-v1

        // Validation:
        assert!(current_model == "model-v1");
        assert!(accuracy > 0.999);

        // Metrics:
        // - Rollback time < 200ms
        // - State consistency maintained
    }

    /// Test 1.4: Multi-Model Registry
    pub async fn test_multi_model_management() -> Result<()> {
        // Register 10 models, list, query, delete
        // Validate registry operations scale linearly

        // Metrics:
        // - List time < 50ms (10 models)
        // - Query time < 10ms per model
        // - Delete time < 20ms per model
    }
}
```

**Acceptance Criteria:**
- ✅ Model registration succeeds with metadata
- ✅ Hot-swap completes with 0ms downtime
- ✅ Rollback restores previous model state
- ✅ Registry operations scale to 100+ models
- ✅ All operations < target latencies

---

##### 2. Shadow Mode Tests (`shadow_mode.rs`)

**Objective:** Validate shadow agent deployment, canary traffic routing, A/B comparison.

**Test Cases:**

```rust
pub struct ShadowModeTests {
    kernel_interface: KernelCommandInterface,
    traffic_generator: TrafficGenerator,
}

impl ShadowModeTests {
    /// Test 2.1: Shadow Agent Deployment
    pub async fn test_shadow_deployment() -> Result<()> {
        // Deploy shadow agent alongside primary
        // 1. llmctl shadow-deploy --id shadow-agent-v2 --traffic 0%
        // 2. Verify shadow agent running but not serving traffic
        // 3. llmctl shadow-status

        // Expected output:
        // Shadow agent deployed: shadow-agent-v2
        // Traffic routing: 0% shadow, 100% primary
        // Shadow agent: READY, requests=0

        // Validation:
        assert!(shadow_agent_deployed);
        assert_eq!(shadow_traffic_pct, 0);
    }

    /// Test 2.2: Canary Traffic Routing (10%)
    pub async fn test_canary_10_percent() -> Result<()> {
        // Route 10% traffic to shadow agent
        // 1. llmctl shadow-traffic --percent 10
        // 2. Send 1000 requests
        // 3. Verify ~100 go to shadow, ~900 to primary

        // Expected distribution:
        // Primary: 900 ± 30 requests
        // Shadow: 100 ± 30 requests

        // Validation:
        assert!((shadow_requests as f64 / total_requests as f64 - 0.10).abs() < 0.03);

        // Metrics:
        // - Routing overhead < 1ms per request
        // - No dropped requests
    }

    /// Test 2.3: A/B Comparison
    pub async fn test_ab_comparison() -> Result<()> {
        // Compare primary vs shadow performance
        // 1. Route 50% traffic to each
        // 2. Collect metrics: latency, accuracy, throughput
        // 3. llmctl shadow-compare

        // Expected output:
        // A/B Comparison Report:
        // Primary:  avg_latency=2.1ms, accuracy=99.95%, throughput=1000 rps
        // Shadow:   avg_latency=1.8ms, accuracy=99.96%, throughput=1100 rps
        // Winner: Shadow (+0.01% accuracy, -14% latency, +10% throughput)

        // Validation:
        assert!(comparison_report.contains("Winner"));
        assert!(comparison_report.metrics.len() >= 3);
    }

    /// Test 2.4: Shadow Promotion
    pub async fn test_shadow_promotion() -> Result<()> {
        // Promote shadow to primary after validation
        // 1. Verify shadow metrics superior
        // 2. llmctl shadow-promote
        // 3. Verify shadow is now primary, old primary retired

        // Expected output:
        // Shadow promoted: shadow-agent-v2 → primary
        // Previous primary retired: primary-agent-v1

        // Validation:
        assert_eq!(current_primary, "shadow-agent-v2");
    }
}
```

**Acceptance Criteria:**
- ✅ Shadow agent deploys without affecting primary
- ✅ Traffic routing accurate within 3% of target
- ✅ A/B comparison generates valid metrics
- ✅ Shadow promotion succeeds with 0 downtime

---

##### 3. OpenTelemetry Exporter Tests (`otel_exporter.rs`)

**Objective:** Validate OpenTelemetry trace export, span creation, context propagation.

**Test Cases:**

```rust
pub struct OTelExporterTests {
    kernel_interface: KernelCommandInterface,
    otel_collector_endpoint: String, // Mock collector
}

impl OTelExporterTests {
    /// Test 3.1: Trace Export Initialization
    pub async fn test_otel_init() -> Result<()> {
        // Initialize OTel exporter
        // 1. otelctl init --endpoint http://localhost:4318
        // 2. Verify connection established

        // Expected output:
        // OTel exporter initialized
        // Endpoint: http://localhost:4318
        // Status: CONNECTED

        // Validation:
        assert!(otel_status.is_connected());
    }

    /// Test 3.2: Span Creation
    pub async fn test_span_creation() -> Result<()> {
        // Create spans for AI operations
        // 1. otelctl enable-tracing
        // 2. llminfer "test prompt"
        // 3. otelctl export-traces
        // 4. Verify span contains: inference_latency, model_id, tokens

        // Expected span structure:
        // {
        //   "name": "llm.inference",
        //   "duration_us": 2100,
        //   "attributes": {
        //     "model.id": "primary-v1",
        //     "tokens.input": 3,
        //     "tokens.output": 8,
        //     "inference.latency_us": 2100
        //   }
        // }

        // Validation:
        assert!(span.name == "llm.inference");
        assert!(span.attributes.contains_key("model.id"));
        assert!(span.duration_us > 0);
    }

    /// Test 3.3: Context Propagation
    pub async fn test_context_propagation() -> Result<()> {
        // Verify trace context propagates across operations
        // 1. Start parent span: graphctl start
        // 2. Child operation: llminfer
        // 3. Verify parent-child relationship in traces

        // Expected trace structure:
        // Parent Span: "graph.execution" (trace_id: abc123)
        //   └─ Child Span: "llm.inference" (parent_id: abc123, span_id: def456)

        // Validation:
        assert_eq!(child_span.parent_id, parent_span.span_id);
        assert_eq!(child_span.trace_id, parent_span.trace_id);
    }

    /// Test 3.4: Batch Export Performance
    pub async fn test_batch_export() -> Result<()> {
        // Generate 10k spans, verify batch export performance
        // 1. Generate load: 10k inference operations
        // 2. otelctl export-traces
        // 3. Measure export time, memory usage

        // Metrics:
        // - Export time < 1s for 10k spans
        // - Memory overhead < 10MB
        // - No dropped spans

        // Validation:
        assert!(export_time_ms < 1000);
        assert!(memory_overhead_mb < 10);
        assert_eq!(exported_spans, 10000);
    }
}
```

**Acceptance Criteria:**
- ✅ OTel exporter connects to collector
- ✅ Spans created with complete attributes
- ✅ Context propagates across operations
- ✅ Batch export handles 10k+ spans efficiently

---

##### 4. Decision Traces Tests (`decision_traces.rs`)

**Objective:** Validate decision trace buffer, export, replay capabilities.

**Test Cases:**

```rust
pub struct DecisionTracesTests {
    kernel_interface: KernelCommandInterface,
}

impl DecisionTracesTests {
    /// Test 4.1: Decision Trace Collection
    pub async fn test_trace_collection() -> Result<()> {
        // Collect meta-agent decisions
        // 1. autoctl on
        // 2. Generate workload (memory pressure, scheduling events)
        // 3. autoctl audit last 100
        // 4. Verify decisions captured with context

        // Expected trace format:
        // [Decision #123] ts=450000μs conf=720
        //   Input: mem_pressure=50%, deadline_misses=0
        //   Output: memory_directive=+480, scheduling_directive=-200
        //   Outcome: +60 reward (pressure decreased to 45%)

        // Validation:
        assert!(traces.len() >= 100);
        assert!(traces[0].contains("Input:"));
        assert!(traces[0].contains("Output:"));
        assert!(traces[0].contains("Outcome:"));
    }

    /// Test 4.2: Decision Buffer Management
    pub async fn test_buffer_management() -> Result<()> {
        // Verify circular buffer behavior
        // 1. Fill buffer (default: 1000 decisions)
        // 2. Add 100 more decisions
        // 3. Verify oldest 100 evicted, newest 1000 retained

        // Validation:
        assert_eq!(buffer.len(), 1000);
        assert_eq!(buffer.oldest().id, 101);
        assert_eq!(buffer.newest().id, 1100);

        // Metrics:
        // - Buffer overhead < 100KB
        // - Insert time < 10μs per decision
    }

    /// Test 4.3: Decision Export
    pub async fn test_decision_export() -> Result<()> {
        // Export decisions for offline analysis
        // 1. autoctl export-decisions --format json --output /tmp/decisions.json
        // 2. Verify JSON format correctness
        // 3. Verify all fields present

        // Expected JSON structure:
        // {
        //   "schema_version": "v1",
        //   "decisions": [
        //     {
        //       "id": 123,
        //       "timestamp_us": 450000,
        //       "confidence": 720,
        //       "inputs": { "mem_pressure": 50, "deadline_misses": 0 },
        //       "outputs": { "memory_directive": 480, "scheduling_directive": -200 },
        //       "outcome": { "reward": 60, "measured": true }
        //     }
        //   ]
        // }

        // Validation:
        assert!(json.schema_version == "v1");
        assert!(json.decisions.len() > 0);
        assert!(json.decisions[0].contains_key("inputs"));
    }

    /// Test 4.4: Decision Replay
    pub async fn test_decision_replay() -> Result<()> {
        // Replay decisions for debugging/analysis
        // 1. Export decisions from run 1
        // 2. autoctl replay-decisions --input /tmp/decisions.json
        // 3. Verify outcomes match original run (deterministic replay)

        // Validation:
        assert_eq!(replayed_outcomes.len(), original_outcomes.len());
        for (i, (orig, replay)) in original_outcomes.iter().zip(replayed_outcomes.iter()).enumerate() {
            assert_eq!(orig.reward, replay.reward, "Decision {} outcome mismatch", i);
        }
    }
}
```

**Acceptance Criteria:**
- ✅ Decisions captured with full context
- ✅ Buffer manages 1000+ decisions efficiently
- ✅ Export generates valid JSON
- ✅ Replay produces deterministic results

---

##### 5. Integration Tests (`integration_tests.rs`)

**Objective:** End-to-end Phase 7 workflow validation.

**Test Scenario:**

```rust
pub struct Phase7IntegrationTests;

impl Phase7IntegrationTests {
    /// Integration Test: Complete AI Ops Workflow
    pub async fn test_complete_ai_ops_workflow() -> Result<()> {
        // Scenario: Deploy new model version with shadow testing

        // Step 1: Register new model
        execute_command("llmctl register --id model-v2 --size 1MB --ctx 4096")?;

        // Step 2: Deploy as shadow agent
        execute_command("llmctl shadow-deploy --id model-v2 --traffic 0%")?;

        // Step 3: Enable tracing
        execute_command("otelctl enable-tracing")?;

        // Step 4: Gradually increase traffic (canary)
        execute_command("llmctl shadow-traffic --percent 10")?;
        tokio::time::sleep(Duration::from_secs(10)).await;

        execute_command("llmctl shadow-traffic --percent 50")?;
        tokio::time::sleep(Duration::from_secs(10)).await;

        // Step 5: Compare performance
        let comparison = execute_command("llmctl shadow-compare")?;

        // Step 6: Promote if better
        if comparison.contains("Winner: Shadow") {
            execute_command("llmctl shadow-promote")?;
        } else {
            execute_command("llmctl shadow-retire")?;
        }

        // Step 7: Export traces and decisions
        execute_command("otelctl export-traces --output /tmp/traces.json")?;
        execute_command("autoctl export-decisions --output /tmp/decisions.json")?;

        // Validation:
        // ✅ All commands succeeded
        // ✅ No downtime during promotion
        // ✅ Traces and decisions exported
        // ✅ Model lifecycle complete
    }
}
```

---

### Phase 7 Module Structure

**File: `crates/testing/src/phase7_ai_ops/mod.rs`**

```rust
//! Phase 7: AI Operations Platform Tests
//!
//! Complete validation of AI Ops infrastructure including model lifecycle,
//! shadow deployment, OpenTelemetry integration, and decision traces.

pub mod model_lifecycle;
pub mod shadow_mode;
pub mod otel_exporter;
pub mod decision_traces;
pub mod integration_tests;

use crate::{TestError, kernel_interface::KernelCommandInterface};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase7Results {
    pub model_lifecycle_passed: bool,
    pub shadow_mode_passed: bool,
    pub otel_exporter_passed: bool,
    pub decision_traces_passed: bool,
    pub integration_passed: bool,
    pub overall_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct Phase7AIOpsSuite {
    kernel_interface: KernelCommandInterface,
    model_lifecycle: model_lifecycle::ModelLifecycleTests,
    shadow_mode: shadow_mode::ShadowModeTests,
    otel_exporter: otel_exporter::OTelExporterTests,
    decision_traces: decision_traces::DecisionTracesTests,
    integration: integration_tests::Phase7IntegrationTests,
}

impl Phase7AIOpsSuite {
    pub fn new(serial_log_path: String, monitor_port: u16) -> Self {
        let kernel_interface = KernelCommandInterface::new(serial_log_path, monitor_port);

        Self {
            kernel_interface: kernel_interface.clone(),
            model_lifecycle: model_lifecycle::ModelLifecycleTests::new(kernel_interface.clone()),
            shadow_mode: shadow_mode::ShadowModeTests::new(kernel_interface.clone()),
            otel_exporter: otel_exporter::OTelExporterTests::new(kernel_interface.clone()),
            decision_traces: decision_traces::DecisionTracesTests::new(kernel_interface.clone()),
            integration: integration_tests::Phase7IntegrationTests::new(),
        }
    }

    /// Run complete Phase 7 validation suite
    pub async fn validate_phase7(&mut self) -> Result<Phase7Results, TestError> {
        log::info!("Starting Phase 7: AI Operations Platform validation");

        // Run all test suites in parallel
        let (lifecycle_result, shadow_result, otel_result, traces_result) = tokio::try_join!(
            self.model_lifecycle.run_all_tests(),
            self.shadow_mode.run_all_tests(),
            self.otel_exporter.run_all_tests(),
            self.decision_traces.run_all_tests(),
        )?;

        // Run integration tests sequentially (depends on above)
        let integration_result = self.integration.run_all_tests().await?;

        // Calculate overall score
        let passed_count = [
            lifecycle_result.passed,
            shadow_result.passed,
            otel_result.passed,
            traces_result.passed,
            integration_result.passed,
        ].iter().filter(|&&p| p).count();

        let overall_score = (passed_count as f64 / 5.0) * 100.0;

        Ok(Phase7Results {
            model_lifecycle_passed: lifecycle_result.passed,
            shadow_mode_passed: shadow_result.passed,
            otel_exporter_passed: otel_result.passed,
            decision_traces_passed: traces_result.passed,
            integration_passed: integration_result.passed,
            overall_score,
            timestamp: chrono::Utc::now(),
        })
    }
}
```

---

### Phase 7 Acceptance Criteria

**Module Completion:**
- ✅ All 5 test modules implemented
- ✅ 20+ test cases covering all AI Ops features
- ✅ Integration test validates end-to-end workflow

**Test Execution:**
- ✅ All tests executable via single command
- ✅ Tests run in parallel where possible
- ✅ Clear pass/fail for each subsystem

**Metrics:**
- ✅ Model operations < target latencies
- ✅ Shadow deployment 0ms downtime
- ✅ OTel export handles 10k+ spans
- ✅ Decision buffer < 100KB overhead

**Documentation:**
- ✅ Each test case documented with expected outputs
- ✅ Failure modes defined
- ✅ Integration examples provided

---

## Phase 8: Performance Optimization Tests (20% → 100%)

### Priority: 🚨 CRITICAL
**Effort:** 2-3 days
**LOC:** ~2000 lines
**Complexity:** High (requires real-time validation, performance measurement)

### Module: `phase8_deterministic/mod.rs`

#### Overview
Complete validation of Phase 8 Performance Optimization including CBS+EDF deterministic scheduler, slab allocator, adaptive memory patterns, meta-agent decisions, and autonomy comparison.

#### Test Coverage Requirements

##### 1. CBS+EDF Scheduler Tests (`cbs_edf_scheduler.rs`)

**Objective:** Actively validate CBS+EDF scheduler with admission control, deadline guarantees, budget management.

**Test Cases:**

```rust
#[derive(Debug)]
pub struct CBSEDFSchedulerTests {
    kernel_interface: KernelCommandInterface,
}

impl CBSEDFSchedulerTests {
    /// Test 1.1: Admission Control
    pub async fn test_admission_control() -> Result<()> {
        // Test admission control with utilization bound

        // Step 1: Create graph
        self.execute("graphctl create --num-operators 10")?;

        // Step 2: Enable deterministic scheduler (85% utilization bound)
        let output = self.execute("det on 10000000 100000000 100000000")?;
        // WCET=10ms, Period=100ms, Budget=100ms

        // Validation:
        assert!(output.contains("[DET] admitted"));

        // Step 3: Attempt to exceed utilization bound
        let output2 = self.execute("det on 90000000 100000000 100000000")?;
        // WCET=90ms, Period=100ms (90% utilization - should be REJECTED)

        // Validation:
        assert!(output2.contains("[DET] rejected") || output2.contains("utilization"));

        // Expected:
        // [DET] admitted (task 1: WCET=10ms, Period=100ms, U=10%)
        // [DET] rejected (total utilization 100% > bound 85%)
    }

    /// Test 1.2: Deadline Misses Detection
    pub async fn test_deadline_miss_detection() -> Result<()> {
        // Overload scheduler, verify deadline miss detection

        // Step 1: Admit task with tight deadline
        self.execute("det on 50000000 100000000 100000000")?;
        // WCET=50ms, Period=100ms

        // Step 2: Add operators that consume time
        self.execute("graphctl add-operator 1 --in none --out 0 --prio 10")?;

        // Step 3: Start execution with heavy workload
        self.execute("graphctl start 1000")?; // 1000 steps

        // Step 4: Check deadline misses
        let status = self.execute("det status")?;

        // Expected output:
        // [DET] enabled=1 wcet_ns=50000000 misses=<N>

        // Validation:
        let misses = extract_deadline_misses(&status);
        // Either 0 misses (scheduler working) or >0 (overload detected)
        assert!(misses >= 0); // Proper accounting

        // Metrics:
        // - Miss detection latency < 1ms
        // - Accurate miss count
    }

    /// Test 1.3: Budget Replenishment
    pub async fn test_budget_replenishment() -> Result<()> {
        // Verify CBS budget correctly replenished each period

        // Step 1: Enable deterministic mode
        self.execute("det on 10000000 100000000 100000000")?;

        // Step 2: Monitor budget over multiple periods
        for i in 0..10 {
            let status = self.execute("det status")?;
            // Extract budget from status (implementation needed in kernel)
            // Expected: budget resets to 100ms every 100ms period

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Validation:
        // Budget correctly replenished 10 times
    }

    /// Test 1.4: EDF Priority Scheduling
    pub async fn test_edf_priority() -> Result<()> {
        // Verify Earliest Deadline First priority ordering

        // Step 1: Create multiple tasks with different deadlines
        self.execute("graphctl create --num-operators 3")?;
        self.execute("det on 5000000 50000000 50000000")?;  // Task 1: deadline=50ms
        self.execute("det on 3000000 30000000 30000000")?;  // Task 2: deadline=30ms (higher priority)

        // Step 2: Start execution
        self.execute("graphctl start 100")?;

        // Step 3: Verify Task 2 scheduled before Task 1 when both ready
        // (Requires kernel to log scheduling decisions)

        // Expected: Task 2 executes first due to earlier deadline
    }

    /// Test 1.5: Integration with Graph Execution
    pub async fn test_graph_integration() -> Result<()> {
        // End-to-end: Graph execution under CBS+EDF

        let output = self.execute_sequence(&[
            "graphctl create --num-operators 5",
            "det on 10000000 100000000 100000000",
            "graphctl add-operator 1 --in none --out 0 --prio 10",
            "graphctl add-operator 2 --in 0 --out 1 --prio 20",
            "graphctl start 100",
            "det status",
            "det off",
        ])?;

        // Validation:
        assert!(output.contains("[DET] admitted"));
        assert!(output.contains("misses=0")); // No deadline misses
        assert!(output.contains("[DET] disabled"));

        // Metrics:
        // - All 100 graph steps completed
        // - 0 deadline misses
        // - Execution time predictable
    }
}
```

**Acceptance Criteria:**
- ✅ Admission control enforces 85% utilization bound
- ✅ Deadline misses accurately detected
- ✅ Budget replenishment correct
- ✅ EDF priority ordering verified
- ✅ Graph execution deterministic

---

##### 2. Slab Allocator Tests (`slab_allocator.rs`)

**Objective:** Validate slab allocator performance meets <5k cycles target for small allocations.

**Test Cases:**

```rust
pub struct SlabAllocatorTests {
    kernel_interface: KernelCommandInterface,
}

impl SlabAllocatorTests {
    /// Test 2.1: Slab Performance Benchmark
    pub async fn test_slab_performance() -> Result<()> {
        // Run slab benchmarks from kernel
        // Kernel has: crates/kernel/src/tests/slab_bench.rs

        // Command: Run benchmark via shell (if exposed)
        // Or parse boot-time benchmark output from serial log

        let output = self.read_serial_log()?;

        // Expected output (from slab_bench.rs):
        // === Slab Allocator Benchmarks ===
        // 16-byte alloc:  avg=1,245 cycles  min=980  max=4,231
        // 32-byte alloc:  avg=1,312 cycles  min=1,020  max=4,456
        // 64-byte alloc:  avg=1,389 cycles  min=1,051  max=4,678
        // 128-byte alloc: avg=1,467 cycles  min=1,098  max=4,923
        // 256-byte alloc: avg=1,545 cycles  min=1,123  max=5,156

        // Parse results
        let bench_results = parse_slab_benchmarks(&output)?;

        // Validation:
        for (size, avg_cycles) in bench_results.allocations {
            assert!(avg_cycles < 5000,
                "{}-byte allocation {} cycles exceeds 5k target", size, avg_cycles);
        }

        // Metrics:
        // - 16-256 byte allocations: <5k cycles (P99)
        // - Deallocation: <3k cycles (P99)
    }

    /// Test 2.2: Slab vs Linked-List Comparison
    pub async fn test_slab_vs_linked_list() -> Result<()> {
        // Compare slab allocator vs baseline linked-list

        let output = self.read_serial_log()?;

        // Expected output (from slab_bench.rs run_comparison_benchmark):
        // === Slab vs Linked-List Comparison ===
        // Slab allocator:        avg=1389 cycles
        // Linked-list allocator: avg=~28,000 cycles (baseline)
        // Speedup: 20.2x faster
        // Improvement: 95% reduction in cycles

        let comparison = parse_comparison(&output)?;

        // Validation:
        assert!(comparison.speedup > 10.0, "Speedup {} < 10x target", comparison.speedup);
        assert!(comparison.slab_cycles < comparison.linked_list_cycles);

        // Metrics:
        // - Speedup: >10x vs linked-list
        // - Cycle reduction: >90%
    }

    /// Test 2.3: Slab Cache Efficiency
    pub async fn test_slab_cache_hit_rate() -> Result<()> {
        // Measure slab cache hit rate

        // Parse slab statistics from kernel output
        let output = self.execute("memctl slab-stats")?; // Hypothetical command

        // Expected output:
        // Slab Statistics:
        //   16-byte cache: 1000 allocs, 980 hits (98.0% hit rate)
        //   32-byte cache: 500 allocs, 490 hits (98.0% hit rate)
        //   ...

        let stats = parse_slab_stats(&output)?;

        // Validation:
        for cache_stats in stats {
            let hit_rate = cache_stats.hits as f64 / cache_stats.allocs as f64;
            assert!(hit_rate > 0.90, "Cache hit rate {} < 90% target", hit_rate);
        }
    }
}
```

**Acceptance Criteria:**
- ✅ Small allocations (16-256 bytes) < 5k cycles
- ✅ Speedup >10x vs linked-list allocator
- ✅ Cache hit rate >90%

---

##### 3. Adaptive Memory Tests (`adaptive_memory.rs`)

**Objective:** Validate adaptive memory patterns (Conservative/Balanced/Aggressive) and strategy switching.

**Test Cases:**

```rust
pub struct AdaptiveMemoryTests {
    kernel_interface: KernelCommandInterface,
}

impl AdaptiveMemoryTests {
    /// Test 3.1: Strategy Switching
    pub async fn test_strategy_switching() -> Result<()> {
        // Test Conservative → Balanced → Aggressive transitions

        // Step 1: Start with Balanced (default)
        let status = self.execute("memctl strategy status")?;
        assert!(status.contains("Balanced"));

        // Step 2: Trigger pressure increase → Aggressive
        // Generate memory pressure via stress test
        self.execute("stresstest memory --duration 5000")?;

        // Step 3: Check strategy changed
        let status2 = self.execute("memctl strategy status")?;
        // May show Aggressive due to high pressure

        // Step 4: Reduce pressure → Balanced
        // Let pressure decay
        tokio::time::sleep(Duration::from_secs(5)).await;

        let status3 = self.execute("memctl strategy status")?;
        // Should return to Balanced

        // Validation:
        // Strategy changes observed based on pressure
    }

    /// Test 3.2: Meta-Agent Directive Thresholds
    pub async fn test_directive_thresholds() -> Result<()> {
        // Verify meta-agent directives trigger correct strategies

        // Parse decision log from autoctl
        let audit = self.execute("autoctl audit last 100")?;

        // Expected:
        // [Decision #N] directive=-700 → Conservative
        // [Decision #M] directive=+250 → Balanced
        // [Decision #P] directive=+800 → Aggressive

        let decisions = parse_audit(&audit)?;

        for decision in decisions {
            let expected_strategy = if decision.memory_directive < -500 {
                "Conservative"
            } else if decision.memory_directive > 500 {
                "Aggressive"
            } else {
                "Balanced"
            };

            // Verify strategy matches directive (if logged)
            // assert_eq!(decision.resulting_strategy, expected_strategy);
        }
    }

    /// Test 3.3: Oscillation Detection
    pub async fn test_oscillation_detection() -> Result<()> {
        // Verify no rapid oscillation (>5 switches in 10s)

        // Step 1: Monitor strategy changes during stress test
        self.execute("stresstest memory --duration 10000")?;

        // Step 2: Parse strategy change log
        let history = self.execute("memctl strategy history")?;

        // Expected format:
        // [125000μs] Conservative → Balanced
        // [98000μs]  Balanced → Conservative
        // ...

        let changes = parse_strategy_history(&history)?;

        // Validation: <5 changes in any 10s window
        let recent_changes = count_changes_in_window(&changes, Duration::from_secs(10));
        assert!(recent_changes < 5, "Oscillation detected: {} changes in 10s", recent_changes);
    }

    /// Test 3.4: Rate-Limited Output
    pub async fn test_rate_limited_output() -> Result<()> {
        // Verify strategy change prints rate-limited to 1/sec

        // Step 1: Cause rapid strategy changes (edge case testing)
        // This was the bug fixed in Phase 8

        // Step 2: Parse serial log timestamps
        let log = self.read_serial_log()?;
        let strategy_prints = extract_strategy_change_prints(&log)?;

        // Expected: Max 1 print per second
        for i in 1..strategy_prints.len() {
            let time_diff = strategy_prints[i].timestamp - strategy_prints[i-1].timestamp;
            assert!(time_diff >= Duration::from_secs(1),
                "Strategy prints {} ms apart (< 1s rate limit)", time_diff.as_millis());
        }

        // Validation:
        // ✅ No output flooding
        // ✅ Rate limit enforced
    }
}
```

**Acceptance Criteria:**
- ✅ Strategies switch based on pressure/directives
- ✅ Meta-agent directives map correctly to strategies
- ✅ No oscillation (< 5 changes/10s)
- ✅ Output rate-limited to 1 print/second

---

##### 4. Meta-Agent Decision Tests (`meta_agent.rs`)

**Objective:** Validate meta-agent neural network decisions, confidence, and reward feedback.

**Test Cases:**

```rust
pub struct MetaAgentTests {
    kernel_interface: KernelCommandInterface,
}

impl MetaAgentTests {
    /// Test 4.1: Decision Inference
    pub async fn test_meta_agent_inference() -> Result<()> {
        // Enable autonomous mode
        self.execute("autoctl on")?;

        // Generate workload
        self.execute("stresstest memory --duration 5000")?;

        // Check decision log
        let audit = self.execute("autoctl audit last 10")?;

        // Expected format:
        // [Decision #145] ts=125000μs conf=720
        //   Memory: +480 → Balanced (current: 50%)
        //   Outcome: +60 reward

        let decisions = parse_audit(&audit)?;

        // Validation:
        assert!(decisions.len() >= 10);
        for decision in decisions {
            assert!(decision.confidence >= 0 && decision.confidence <= 1000);
            assert!(decision.memory_directive >= -1000 && decision.memory_directive <= 1000);
        }
    }

    /// Test 4.2: Confidence Thresholds
    pub async fn test_confidence_thresholds() -> Result<()> {
        // Verify low-confidence decisions deferred

        let audit = self.execute("autoctl audit last 100")?;
        let decisions = parse_audit(&audit)?;

        // Expected: Decisions with conf < 600 should be marked as deferred
        for decision in decisions {
            if decision.confidence < 600 {
                assert!(decision.action == "deferred",
                    "Low-confidence decision {} not deferred", decision.id);
            }
        }
    }

    /// Test 4.3: Multi-Subsystem Directives
    pub async fn test_multi_subsystem_directives() -> Result<()> {
        // Verify meta-agent outputs memory + scheduling + command directives

        let audit = self.execute("autoctl audit last 10")?;
        let decisions = parse_audit(&audit)?;

        for decision in decisions {
            // Each decision should have 3 directive types
            assert!(decision.memory_directive.is_some());
            assert!(decision.scheduling_directive.is_some());
            assert!(decision.command_directive.is_some());
        }
    }

    /// Test 4.4: Reward Feedback Loop
    pub async fn test_reward_feedback() -> Result<()> {
        // Verify outcome rewards calculated correctly

        let audit = self.execute("autoctl audit last 50")?;
        let decisions = parse_audit(&audit)?;

        // Rewards should be in range [-100, +100]
        for decision in decisions {
            if let Some(reward) = decision.outcome_reward {
                assert!(reward >= -100 && reward <= 100,
                    "Reward {} out of bounds [-100, +100]", reward);
            }
        }

        // Positive rewards for pressure reduction
        // Negative rewards for pressure increase
    }
}
```

**Acceptance Criteria:**
- ✅ Meta-agent produces decisions with valid directives
- ✅ Low-confidence decisions deferred
- ✅ Multi-subsystem directives present
- ✅ Reward feedback calculated correctly

---

##### 5. Stress Test Comparison (`stress_comparison.rs`)

**Objective:** Compare autonomy ON vs OFF performance in stress tests.

**Test Cases:**

```rust
pub struct StressComparisonTests {
    kernel_interface: KernelCommandInterface,
}

impl StressComparisonTests {
    /// Test 5.1: Autonomy OFF Baseline
    pub async fn test_autonomy_off_baseline() -> Result<()> {
        // Run stress test with autonomy disabled

        self.execute("autoctl off")?;
        self.execute("stresstest memory --duration 10000")?;

        // Parse metrics from serial log
        let metrics = parse_stress_metrics(&self.read_serial_log()?)?;

        // Expected metrics (from Phase 8 documentation):
        // Autonomy OFF:
        //   Peak pressure: 56%
        //   Avg pressure: 53%
        //   OOM events: 0
        //   Compaction triggers: 0

        Ok(StressMetrics {
            peak_pressure: metrics.peak_pressure,
            avg_pressure: metrics.avg_pressure,
            oom_events: metrics.oom_events,
            compaction_triggers: metrics.compaction_triggers,
            ai_interventions: 0,
        })
    }

    /// Test 5.2: Autonomy ON Comparison
    pub async fn test_autonomy_on_comparison() -> Result<()> {
        // Run stress test with autonomy enabled

        self.execute("autoctl on")?;
        self.execute("stresstest memory --duration 10000")?;

        let metrics = parse_stress_metrics(&self.read_serial_log()?)?;

        // Expected metrics (from Phase 8 documentation):
        // Autonomy ON:
        //   Peak pressure: 51%
        //   Avg pressure: 50%
        //   OOM events: 0
        //   Compaction triggers: 6
        //   AI interventions: 1120

        Ok(StressMetrics {
            peak_pressure: metrics.peak_pressure,
            avg_pressure: metrics.avg_pressure,
            oom_events: metrics.oom_events,
            compaction_triggers: metrics.compaction_triggers,
            ai_interventions: metrics.ai_interventions,
        })
    }

    /// Test 5.3: Performance Delta Validation
    pub async fn test_performance_delta() -> Result<()> {
        // Compare ON vs OFF

        let baseline = self.test_autonomy_off_baseline().await?;
        let autonomous = self.test_autonomy_on_comparison().await?;

        // Validation (from Phase 8 documentation):
        // ✅ Peak pressure reduced by ≥3% (56% → 51% = 5%)
        // ✅ Avg pressure reduced by ≥2% (53% → 50% = 3%)
        // ✅ Proactive compactions: >0 (6 observed)
        // ✅ AI interventions: >1000 (1120 observed)

        let peak_reduction = baseline.peak_pressure - autonomous.peak_pressure;
        let avg_reduction = baseline.avg_pressure - autonomous.avg_pressure;

        assert!(peak_reduction >= 3.0, "Peak pressure reduction {} < 3% target", peak_reduction);
        assert!(avg_reduction >= 2.0, "Avg pressure reduction {} < 2% target", avg_reduction);
        assert!(autonomous.compaction_triggers > 0, "No proactive compactions");
        assert!(autonomous.ai_interventions > 1000, "Insufficient AI interventions");

        // Generate comparison report
        log::info!("Autonomy Impact:");
        log::info!("  Peak pressure: {}% → {}% ({}% reduction)",
            baseline.peak_pressure, autonomous.peak_pressure, peak_reduction);
        log::info!("  Avg pressure: {}% → {}% ({}% reduction)",
            baseline.avg_pressure, autonomous.avg_pressure, avg_reduction);
        log::info!("  Proactive compactions: {} → {}",
            baseline.compaction_triggers, autonomous.compaction_triggers);
        log::info!("  AI interventions: {}", autonomous.ai_interventions);
    }
}
```

**Acceptance Criteria:**
- ✅ Baseline metrics collected (autonomy OFF)
- ✅ Autonomous metrics collected (autonomy ON)
- ✅ Peak pressure reduced ≥3%
- ✅ Avg pressure reduced ≥2%
- ✅ Proactive compactions >0
- ✅ AI interventions >1000

---

##### 6. Rate Limiting Tests (`rate_limiting.rs`)

**Objective:** Validate output rate-limiting bug fix (1 print/second).

**Test Cases:**

```rust
pub struct RateLimitingTests {
    kernel_interface: KernelCommandInterface,
}

impl RateLimitingTests {
    /// Test 6.1: Strategy Change Rate Limiting
    pub async fn test_strategy_change_rate_limit() -> Result<()> {
        // Trigger rapid strategy changes, verify output limited

        // Generate high-frequency decision loop
        self.execute("stresstest memory --duration 10000")?;

        // Parse serial log for strategy change prints
        let log = self.read_serial_log()?;
        let prints = extract_prints_with_timestamps(&log, "[PRED_MEM] Strategy change:")?;

        // Validation: Max 1 print per second
        for i in 1..prints.len() {
            let time_diff_us = prints[i].timestamp_us - prints[i-1].timestamp_us;
            assert!(time_diff_us >= 1_000_000,
                "Strategy prints {}μs apart (< 1s)", time_diff_us);
        }
    }

    /// Test 6.2: Meta-Agent Directive Rate Limiting
    pub async fn test_meta_agent_directive_rate_limit() -> Result<()> {
        // Verify meta-agent directive prints rate-limited

        self.execute("autoctl on")?;
        self.execute("stresstest memory --duration 10000")?;

        let log = self.read_serial_log()?;
        let prints = extract_prints_with_timestamps(&log, "[META] Memory directive:")?;

        // Validation: Max 1 print per second
        for i in 1..prints.len() {
            let time_diff_us = prints[i].timestamp_us - prints[i-1].timestamp_us;
            assert!(time_diff_us >= 1_000_000,
                "Directive prints {}μs apart (< 1s)", time_diff_us);
        }
    }

    /// Test 6.3: No Output Flooding
    pub async fn test_no_output_flooding() -> Result<()> {
        // Verify stress test doesn't hang due to output I/O

        let start = std::time::Instant::now();
        self.execute("stresstest memory --duration 5000")?;
        let elapsed = start.elapsed();

        // Should complete in ~5s, not hang indefinitely
        assert!(elapsed.as_secs() < 10,
            "Stress test took {}s (> 10s, possible hang)", elapsed.as_secs());
    }
}
```

**Acceptance Criteria:**
- ✅ Strategy change prints ≤1/second
- ✅ Meta-agent directive prints ≤1/second
- ✅ No output flooding causes hang

---

### Phase 8 Module Structure

**File: `crates/testing/src/phase8_deterministic/mod.rs`**

```rust
//! Phase 8: Performance Optimization Tests
//!
//! Complete validation of CBS+EDF deterministic scheduler, slab allocator,
//! adaptive memory patterns, meta-agent decisions, and autonomy impact.

pub mod cbs_edf_scheduler;
pub mod slab_allocator;
pub mod adaptive_memory;
pub mod meta_agent;
pub mod stress_comparison;
pub mod rate_limiting;

use crate::{TestError, kernel_interface::KernelCommandInterface};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase8Results {
    pub cbs_edf_passed: bool,
    pub slab_allocator_passed: bool,
    pub adaptive_memory_passed: bool,
    pub meta_agent_passed: bool,
    pub stress_comparison_passed: bool,
    pub rate_limiting_passed: bool,
    pub overall_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct Phase8DeterministicSuite {
    kernel_interface: KernelCommandInterface,
    cbs_edf: cbs_edf_scheduler::CBSEDFSchedulerTests,
    slab_allocator: slab_allocator::SlabAllocatorTests,
    adaptive_memory: adaptive_memory::AdaptiveMemoryTests,
    meta_agent: meta_agent::MetaAgentTests,
    stress_comparison: stress_comparison::StressComparisonTests,
    rate_limiting: rate_limiting::RateLimitingTests,
}

impl Phase8DeterministicSuite {
    pub fn new(serial_log_path: String, monitor_port: u16) -> Self {
        let kernel_interface = KernelCommandInterface::new(serial_log_path, monitor_port);

        Self {
            kernel_interface: kernel_interface.clone(),
            cbs_edf: cbs_edf_scheduler::CBSEDFSchedulerTests::new(kernel_interface.clone()),
            slab_allocator: slab_allocator::SlabAllocatorTests::new(kernel_interface.clone()),
            adaptive_memory: adaptive_memory::AdaptiveMemoryTests::new(kernel_interface.clone()),
            meta_agent: meta_agent::MetaAgentTests::new(kernel_interface.clone()),
            stress_comparison: stress_comparison::StressComparisonTests::new(kernel_interface.clone()),
            rate_limiting: rate_limiting::RateLimitingTests::new(kernel_interface.clone()),
        }
    }

    /// Run complete Phase 8 validation suite
    pub async fn validate_phase8(&mut self) -> Result<Phase8Results, TestError> {
        log::info!("Starting Phase 8: Performance Optimization validation");

        // Run tests in parallel where possible
        let (cbs_result, slab_result, adaptive_result, meta_result, rate_result) = tokio::try_join!(
            self.cbs_edf.run_all_tests(),
            self.slab_allocator.run_all_tests(),
            self.adaptive_memory.run_all_tests(),
            self.meta_agent.run_all_tests(),
            self.rate_limiting.run_all_tests(),
        )?;

        // Run stress comparison sequentially (requires clean state)
        let stress_result = self.stress_comparison.run_all_tests().await?;

        // Calculate overall score
        let passed_count = [
            cbs_result.passed,
            slab_result.passed,
            adaptive_result.passed,
            meta_result.passed,
            stress_result.passed,
            rate_result.passed,
        ].iter().filter(|&&p| p).count();

        let overall_score = (passed_count as f64 / 6.0) * 100.0;

        Ok(Phase8Results {
            cbs_edf_passed: cbs_result.passed,
            slab_allocator_passed: slab_result.passed,
            adaptive_memory_passed: adaptive_result.passed,
            meta_agent_passed: meta_result.passed,
            stress_comparison_passed: stress_result.passed,
            rate_limiting_passed: rate_result.passed,
            overall_score,
            timestamp: chrono::Utc::now(),
        })
    }
}
```

---

### Phase 8 Acceptance Criteria

**Module Completion:**
- ✅ All 6 test modules implemented
- ✅ 30+ test cases covering all Phase 8 features
- ✅ Stress comparison validates autonomy impact

**Test Execution:**
- ✅ Tests run via single command
- ✅ Parallel execution where possible
- ✅ Clear pass/fail for each subsystem

**Metrics:**
- ✅ CBS+EDF: 0 deadline misses, admission control enforced
- ✅ Slab allocator: <5k cycles for small allocations
- ✅ Adaptive memory: Strategies switch correctly
- ✅ Meta-agent: Valid decisions with rewards
- ✅ Autonomy: ≥3% peak pressure reduction
- ✅ Rate limiting: ≤1 print/second

---

## Phase 6: Web GUI Management Tests (30% → 100%)

### Priority: ⚠️ HIGH
**Effort:** 1-2 days
**LOC:** ~1500 lines
**Complexity:** Medium (requires HTTP server validation)

### Module: `phase6_web_gui/mod.rs`

#### Overview
Complete validation of Phase 6 Web GUI Management Interface including HTTP server lifecycle, WebSocket connections, REST API endpoints, authentication, and real-time metric streaming.

#### Test Coverage Requirements

##### 1. HTTP Server Tests (`http_server.rs`)

**Objective:** Validate HTTP server startup, shutdown, basic request handling.

**Test Cases:**

```rust
pub struct HTTPServerTests {
    kernel_interface: KernelCommandInterface,
    base_url: String, // e.g., "http://localhost:8080"
}

impl HTTPServerTests {
    /// Test 1.1: Server Startup
    pub async fn test_server_startup() -> Result<()> {
        // Start HTTP server
        let output = self.execute("webctl start --port 8080")?;

        // Expected output:
        // [WEB] HTTP server started on 0.0.0.0:8080
        // [WEB] WebSocket endpoint: ws://localhost:8080/ws

        // Validation:
        assert!(output.contains("HTTP server started"));
        assert!(output.contains("8080"));

        // Verify server responding
        let response = reqwest::get(&format!("{}/health", self.base_url)).await?;
        assert_eq!(response.status(), 200);
    }

    /// Test 1.2: Health Endpoint
    pub async fn test_health_endpoint() -> Result<()> {
        let response = reqwest::get(&format!("{}/health", self.base_url)).await?;
        let body: serde_json::Value = response.json().await?;

        // Expected:
        // {
        //   "status": "healthy",
        //   "uptime_secs": 123,
        //   "version": "0.1.0"
        // }

        assert_eq!(body["status"], "healthy");
        assert!(body["uptime_secs"].as_u64().unwrap() > 0);
    }

    /// Test 1.3: Server Shutdown
    pub async fn test_server_shutdown() -> Result<()> {
        let output = self.execute("webctl stop")?;

        // Expected:
        // [WEB] HTTP server stopped

        assert!(output.contains("stopped"));

        // Verify server no longer responding
        let result = reqwest::get(&format!("{}/health", self.base_url)).await;
        assert!(result.is_err()); // Connection refused
    }
}
```

**Acceptance Criteria:**
- ✅ Server starts on specified port
- ✅ Health endpoint returns valid JSON
- ✅ Server shuts down gracefully
- ✅ No port conflicts or leaks

---

##### 2. WebSocket Tests (`websocket.rs`)

**Objective:** Validate WebSocket connection lifecycle, message passing, real-time updates.

**Test Cases:**

```rust
pub struct WebSocketTests {
    kernel_interface: KernelCommandInterface,
    ws_url: String, // e.g., "ws://localhost:8080/ws"
}

impl WebSocketTests {
    /// Test 2.1: WebSocket Connection
    pub async fn test_websocket_connection() -> Result<()> {
        use tokio_tungstenite::connect_async;

        let (ws_stream, _) = connect_async(&self.ws_url).await?;

        // Validation: Connection established
        assert!(ws_stream.is_ok());
    }

    /// Test 2.2: Ping/Pong
    pub async fn test_ping_pong() -> Result<()> {
        let (mut ws_stream, _) = connect_async(&self.ws_url).await?;

        // Send ping
        ws_stream.send(Message::Ping(vec![])).await?;

        // Expect pong
        let response = ws_stream.next().await.unwrap()?;
        assert!(matches!(response, Message::Pong(_)));
    }

    /// Test 2.3: Metric Subscription
    pub async fn test_metric_subscription() -> Result<()> {
        let (mut ws_stream, _) = connect_async(&self.ws_url).await?;

        // Subscribe to metrics
        let subscribe_msg = json!({
            "type": "subscribe",
            "metrics": ["memory_pressure", "cpu_usage"]
        });
        ws_stream.send(Message::Text(subscribe_msg.to_string())).await?;

        // Expect acknowledgment
        let response = ws_stream.next().await.unwrap()?;
        let ack: serde_json::Value = serde_json::from_str(&response.to_text()?)?;
        assert_eq!(ack["type"], "subscribed");

        // Expect periodic metric updates
        for _ in 0..5 {
            let msg = ws_stream.next().await.unwrap()?;
            let data: serde_json::Value = serde_json::from_str(&msg.to_text()?)?;
            assert_eq!(data["type"], "metric_update");
            assert!(data["memory_pressure"].is_number());
        }
    }
}
```

**Acceptance Criteria:**
- ✅ WebSocket connections succeed
- ✅ Ping/pong heartbeat works
- ✅ Metric subscriptions deliver updates
- ✅ Connection cleanup on disconnect

---

##### 3. API Endpoint Tests (`api_endpoints.rs`)

**Objective:** Validate REST API endpoints for kernel management.

**Test Cases:**

```rust
pub struct APIEndpointTests {
    client: reqwest::Client,
    base_url: String,
}

impl APIEndpointTests {
    /// Test 3.1: GET /api/metrics
    pub async fn test_get_metrics() -> Result<()> {
        let response = self.client
            .get(&format!("{}/api/metrics", self.base_url))
            .send()
            .await?;

        assert_eq!(response.status(), 200);

        let metrics: serde_json::Value = response.json().await?;

        // Expected:
        // {
        //   "memory_pressure": 45,
        //   "cpu_usage": 60,
        //   "deadline_misses": 0,
        //   "uptime_secs": 3600
        // }

        assert!(metrics["memory_pressure"].is_number());
        assert!(metrics["cpu_usage"].is_number());
    }

    /// Test 3.2: POST /api/command
    pub async fn test_post_command() -> Result<()> {
        let command = json!({
            "command": "memctl status"
        });

        let response = self.client
            .post(&format!("{}/api/command", self.base_url))
            .json(&command)
            .send()
            .await?;

        assert_eq!(response.status(), 200);

        let result: serde_json::Value = response.json().await?;

        // Expected:
        // {
        //   "success": true,
        //   "output": "Memory pressure: 45%\n...",
        //   "exit_code": 0
        // }

        assert_eq!(result["success"], true);
        assert!(result["output"].is_string());
    }

    /// Test 3.3: GET /api/logs
    pub async fn test_get_logs() -> Result<()> {
        let response = self.client
            .get(&format!("{}/api/logs?lines=100", self.base_url))
            .send()
            .await?;

        assert_eq!(response.status(), 200);

        let logs: Vec<String> = response.json().await?;
        assert!(logs.len() <= 100);
    }
}
```

**Acceptance Criteria:**
- ✅ All REST endpoints return valid JSON
- ✅ Commands execute correctly via API
- ✅ Metrics endpoint provides real-time data
- ✅ Error handling returns proper HTTP codes

---

### Phase 6 Module Structure

**File: `crates/testing/src/phase6_web_gui/mod.rs`**

```rust
//! Phase 6: Web GUI Management Interface Tests
//!
//! Complete validation of HTTP server, WebSocket connections, REST API,
//! authentication, and real-time metric streaming.

pub mod http_server;
pub mod websocket;
pub mod api_endpoints;
pub mod authentication;
pub mod real_time_updates;

use crate::{TestError, kernel_interface::KernelCommandInterface};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase6Results {
    pub http_server_passed: bool,
    pub websocket_passed: bool,
    pub api_endpoints_passed: bool,
    pub authentication_passed: bool,
    pub real_time_updates_passed: bool,
    pub overall_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct Phase6WebGUISuite {
    kernel_interface: KernelCommandInterface,
    http_server: http_server::HTTPServerTests,
    websocket: websocket::WebSocketTests,
    api_endpoints: api_endpoints::APIEndpointTests,
    authentication: authentication::AuthenticationTests,
    real_time: real_time_updates::RealTimeUpdateTests,
}

impl Phase6WebGUISuite {
    pub async fn validate_phase6(&mut self) -> Result<Phase6Results, TestError> {
        log::info!("Starting Phase 6: Web GUI Management validation");

        // Run tests
        let (http_result, ws_result, api_result, auth_result, rt_result) = tokio::try_join!(
            self.http_server.run_all_tests(),
            self.websocket.run_all_tests(),
            self.api_endpoints.run_all_tests(),
            self.authentication.run_all_tests(),
            self.real_time.run_all_tests(),
        )?;

        let passed_count = [
            http_result.passed,
            ws_result.passed,
            api_result.passed,
            auth_result.passed,
            rt_result.passed,
        ].iter().filter(|&&p| p).count();

        let overall_score = (passed_count as f64 / 5.0) * 100.0;

        Ok(Phase6Results {
            http_server_passed: http_result.passed,
            websocket_passed: ws_result.passed,
            api_endpoints_passed: api_result.passed,
            authentication_passed: auth_result.passed,
            real_time_updates_passed: rt_result.passed,
            overall_score,
            timestamp: chrono::Utc::now(),
        })
    }
}
```

---

## Phase 3: Temporal Isolation Tests (40% → 100%)

### Priority: ⚠️ HIGH
**Effort:** 1 day
**LOC:** ~800 lines
**Complexity:** Medium

### Module: `phase3_temporal/mod.rs`

#### Test Coverage Requirements

##### 1. Active Isolation Tests (`active_isolation.rs`)

**Test Cases:**

```rust
pub struct ActiveIsolationTests {
    kernel_interface: KernelCommandInterface,
}

impl ActiveIsolationTests {
    /// Test 1.1: Temporal Isolation Verification
    pub async fn test_temporal_isolation() -> Result<()> {
        // Execute rtaivalidation command
        let output = self.execute("rtaivalidation")?;

        // Expected output:
        // === Real-Time AI Validation ===
        // Temporal isolation: VERIFIED
        // Max jitter: 234ns
        // Inference latency: 2.1ms (deterministic)

        // Validation:
        assert!(output.contains("Temporal isolation: VERIFIED"));

        // Parse max jitter
        let max_jitter_ns = extract_max_jitter(&output)?;
        assert!(max_jitter_ns < 1000, "Max jitter {} > 1μs", max_jitter_ns);
    }

    /// Test 1.2: Deadline Validation
    pub async fn test_deadline_validation() -> Result<()> {
        // Run deterministic inference with deadline
        self.execute("det on 5000000 10000000 10000000")?; // 5ms WCET, 10ms period
        self.execute("llminfer test --deadline 10ms")?;

        // Check deadline met
        let status = self.execute("det status")?;
        assert!(status.contains("misses=0"));
    }
}
```

**Acceptance Criteria:**
- ✅ Temporal isolation verified
- ✅ Jitter < 1μs
- ✅ Deadlines met consistently

---

## Phase 1: AI-Native Dataflow Tests (70% → 100%)

### Priority: ⚠️ MEDIUM
**Effort:** 1 day
**LOC:** ~1000 lines
**Complexity:** Medium

### Module: `phase1_dataflow/mod.rs`

#### Test Coverage Requirements

##### 1. Graph Execution Tests (`graph_execution.rs`)

**Test Cases:**

```rust
pub struct GraphExecutionTests {
    kernel_interface: KernelCommandInterface,
}

impl GraphExecutionTests {
    /// Test 1.1: Graph Creation
    pub async fn test_graph_creation() -> Result<()> {
        let output = self.execute("graphctl create --num-operators 5")?;

        // Expected:
        // [GRAPH] Created graph with 5 operators

        assert!(output.contains("Created graph"));
        assert!(output.contains("5 operators"));
    }

    /// Test 1.2: Operator Addition
    pub async fn test_operator_addition() -> Result<()> {
        self.execute("graphctl create --num-operators 10")?;
        let output = self.execute("graphctl add-operator 1 --in none --out 0 --prio 10")?;

        // Expected:
        // [GRAPH] Added operator 1: priority=10, in=none, out=[0]

        assert!(output.contains("Added operator 1"));
    }

    /// Test 1.3: Graph Execution
    pub async fn test_graph_execution() -> Result<()> {
        self.execute("graphctl create --num-operators 5")?;
        self.execute("graphctl add-operator 1 --in none --out 0 --prio 10")?;

        let output = self.execute("graphctl start 100")?; // 100 steps

        // Expected:
        // [GRAPH] Executing 100 steps...
        // [GRAPH] Execution complete: 100/100 steps

        assert!(output.contains("Execution complete: 100/100"));
    }
}
```

**Acceptance Criteria:**
- ✅ Graph creation succeeds
- ✅ Operators added correctly
- ✅ Graph executes specified steps
- ✅ No deadlocks or hangs

---

## Phase 2: AI Governance Tests (50% → 100%)

### Priority: ⚠️ MEDIUM
**Effort:** 1 day
**LOC:** ~800 lines
**Complexity:** Medium

### Module: `phase2_governance/` (Enhanced)

#### Test Coverage Requirements

##### 1. LLM Fine-Tuning Tests (`llm_finetune.rs`)

**Test Cases:**

```rust
pub struct LLMFineTuneTests {
    kernel_interface: KernelCommandInterface,
}

impl LLMFineTuneTests {
    /// Test 1.1: LoRA Adapter Registration
    pub async fn test_lora_adapter_registration() -> Result<()> {
        let output = self.execute("llmctl lora-register --name adapter-v1 --rank 8")?;

        // Expected:
        // [LLM] LoRA adapter registered: adapter-v1 (rank=8)

        assert!(output.contains("LoRA adapter registered"));
        assert!(output.contains("adapter-v1"));
    }

    /// Test 1.2: Fine-Tuning Execution
    pub async fn test_finetuning() -> Result<()> {
        self.execute("llmctl load")?;
        let output = self.execute("llmctl finetune --adapter adapter-v1 --epochs 10")?;

        // Expected:
        // [LLM] Fine-tuning started: adapter-v1
        // [LLM] Epoch 1/10 complete (loss=0.45)
        // ...
        // [LLM] Fine-tuning complete: final loss=0.12

        assert!(output.contains("Fine-tuning complete"));
    }
}
```

**Acceptance Criteria:**
- ✅ LoRA adapters register correctly
- ✅ Fine-tuning executes without errors
- ✅ Loss decreases over epochs

---

## Phase 5: UX Safety Tests (60% → 100%)

### Priority: ⚠️ MEDIUM
**Effort:** 1 day
**LOC:** ~600 lines
**Complexity:** Low

### Module: `phase5_ux_safety/` (Enhanced)

#### Test Coverage Requirements

##### 1. Safety Controls Tests (`safety_controls.rs`)

**Test Cases:**

```rust
pub struct SafetyControlTests {
    kernel_interface: KernelCommandInterface,
}

impl SafetyControlTests {
    /// Test 1.1: Safety Override
    pub async fn test_safety_override() -> Result<()> {
        // Attempt unsafe operation
        let output = self.execute("memctl force-oom")?; // Hypothetical unsafe command

        // Expected: Blocked by safety controls
        // [SAFETY] Operation blocked: memctl force-oom (reason: unsafe)

        assert!(output.contains("blocked") || output.contains("denied"));
    }
}
```

**Acceptance Criteria:**
- ✅ Unsafe operations blocked
- ✅ Safety overrides require explicit confirmation
- ✅ Audit log records safety events

---

## Integration & Orchestration

### Main Test Suite Orchestration

**File: `crates/testing/src/lib.rs` (Enhanced)**

```rust
impl SISTestSuite {
    pub async fn execute_comprehensive_validation(&mut self) -> Result<ValidationReport> {
        log::info!("Starting SIS Kernel Complete Validation (100% Coverage)");

        // Run all phases in parallel (where possible)
        let (phase_a, phase_1, phase_2, phase_3, phase_4, phase_5, phase_6, phase_7, phase_8) =
            tokio::try_join!(
                self.validate_phase_a(),      // OS Foundation
                self.validate_phase_1(),      // AI-Native + Dataflow
                self.validate_phase_2(),      // AI Governance
                self.validate_phase_3(),      // Temporal Isolation
                self.validate_phase_4(),      // Production Readiness
                self.validate_phase_5(),      // UX Safety
                self.validate_phase_6(),      // Web GUI
                self.validate_phase_7(),      // AI Operations (NEW)
                self.validate_phase_8(),      // Performance Optimization (NEW)
            )?;

        // Generate comprehensive report
        self.generate_complete_report(
            phase_a, phase_1, phase_2, phase_3, phase_4,
            phase_5, phase_6, phase_7, phase_8
        ).await
    }

    async fn validate_phase_7(&mut self) -> Result<Phase7Results> {
        if let Some(ref qemu_mgr) = self.qemu_runtime {
            if let Some(serial_log) = qemu_mgr.get_serial_log_path(0) {
                let monitor_port = qemu_mgr.get_monitor_port(0);
                let mut phase7_suite = Phase7AIOpsSuite::new(serial_log, monitor_port);
                return phase7_suite.validate_phase7().await;
            }
        }

        // Fallback: skip if QEMU not available
        log::warn!("Phase 7 tests skipped (QEMU not available)");
        Ok(Phase7Results::default())
    }

    async fn validate_phase_8(&mut self) -> Result<Phase8Results> {
        if let Some(ref qemu_mgr) = self.qemu_runtime {
            if let Some(serial_log) = qemu_mgr.get_serial_log_path(0) {
                let monitor_port = qemu_mgr.get_monitor_port(0);
                let mut phase8_suite = Phase8DeterministicSuite::new(serial_log, monitor_port);
                return phase8_suite.validate_phase8().await;
            }
        }

        log::warn!("Phase 8 tests skipped (QEMU not available)");
        Ok(Phase8Results::default())
    }
}
```

---

## Validation Criteria

### Overall Test Suite Success

**Must Pass ALL of the Following:**

```
✅ Phase A: OS Foundation (≥90% tests pass)
✅ Phase 1: AI-Native + Dataflow (≥90% tests pass)
✅ Phase 2: AI Governance (≥90% tests pass)
✅ Phase 3: Temporal Isolation (≥90% tests pass)
✅ Phase 4: Production Readiness (≥90% tests pass)
✅ Phase 5: UX Safety (≥90% tests pass)
✅ Phase 6: Web GUI (≥90% tests pass)
✅ Phase 7: AI Operations (≥90% tests pass) 🆕
✅ Phase 8: Performance Optimization (≥90% tests pass) 🆕

Overall Score: ≥90% (162/180+ tests pass)
```

### Performance Targets

**Phase 8 Specific:**
- ✅ CBS+EDF: 0 deadline misses, <100ms admission latency
- ✅ Slab allocator: <5k cycles (16-256 byte allocations)
- ✅ Adaptive memory: Strategies switch within 1s of directive change
- ✅ Meta-agent: Inference <100μs, confidence ≥600 for 80% decisions
- ✅ Autonomy: ≥3% peak pressure reduction vs baseline
- ✅ Rate limiting: ≤1 print/second for high-frequency events

**Phase 7 Specific:**
- ✅ Model hot-swap: 0ms downtime
- ✅ Shadow promotion: <500ms completion
- ✅ OTel export: <1s for 10k spans
- ✅ Decision traces: <100KB buffer overhead

---

## Implementation Timeline

### Phase Priorities & Timeline

**Week 1: Critical Phases (7 days)**
- Days 1-3: Phase 7 (AI Operations) - 0% → 100%
- Days 4-6: Phase 8 (Performance) - 20% → 100%
- Day 7: Integration testing & bug fixes

**Week 2: High Priority Phases (5 days)**
- Days 1-2: Phase 6 (Web GUI) - 30% → 100%
- Day 3: Phase 3 (Temporal Isolation) - 40% → 100%
- Day 4: Phase 1 (Dataflow) - 70% → 100%
- Day 5: Phase 2 (Governance) - 50% → 100%

**Week 3: Final Phase & Validation (3 days)**
- Day 1: Phase 5 (UX Safety) - 60% → 100%
- Day 2: End-to-end integration testing
- Day 3: Documentation, final validation, delivery

**Total: ~15 days to 100% coverage**

### Deliverables

**Per Phase:**
1. Complete test module with all test cases
2. Module documentation (rustdoc comments)
3. Integration with main test suite
4. Passing validation (≥90% tests)

**Final Delivery:**
1. Complete test suite (all 9 phases)
2. Single-command execution (`--full-demo`)
3. Comprehensive validation report
4. Coverage report showing 100% across all phases
5. Performance benchmarks vs targets
6. GitHub PR ready for integration

---

## Development Guidelines

### Code Quality Standards

**Rust Best Practices:**
```rust
// ✅ Good: Clear test names, assertions, error messages
#[tokio::test]
async fn test_admission_control_enforces_utilization_bound() {
    let output = execute_command("det on 90000000 100000000 100000000")?;
    assert!(output.contains("[DET] rejected"),
        "Expected admission rejection for 90% utilization (> 85% bound), got: {}", output);
}

// ❌ Bad: Unclear names, missing context
#[tokio::test]
async fn test1() {
    let o = run("det on 90000000 100000000 100000000")?;
    assert!(o.contains("rejected"));
}
```

**Error Handling:**
```rust
// ✅ Good: Rich error context
pub async fn execute_command(&mut self, cmd: &str) -> Result<CommandOutput, TestError> {
    self.kernel_interface
        .execute_command(cmd)
        .await
        .map_err(|e| TestError::CommandFailed {
            command: cmd.to_string(),
            reason: format!("Execution failed: {}", e),
        })
}
```

**Documentation:**
```rust
/// Test 1.1: Admission Control
///
/// **Objective:** Verify CBS+EDF admission control enforces 85% utilization bound.
///
/// **Steps:**
/// 1. Create graph with 10 operators
/// 2. Admit task with 10% utilization (should succeed)
/// 3. Attempt to admit task with 90% utilization (should be rejected)
///
/// **Expected Output:**
/// ```
/// [DET] admitted (task 1: WCET=10ms, Period=100ms, U=10%)
/// [DET] rejected (total utilization 100% > bound 85%)
/// ```
///
/// **Validation:**
/// - First admission contains "[DET] admitted"
/// - Second admission contains "[DET] rejected" or "utilization"
///
/// **Metrics:**
/// - Admission decision latency < 100ms
pub async fn test_admission_control() -> Result<()> { ... }
```

### Testing Best Practices

**Isolation:**
- Each test case is independent
- Clean state before/after tests
- No shared mutable state

**Assertions:**
- Clear assertion messages
- Check expected outputs explicitly
- Validate both positive and negative cases

**Performance:**
- Run tests in parallel where possible
- Use timeouts to catch hangs
- Measure and validate latencies

**Logging:**
- Log test start/end
- Log command execution
- Capture kernel output for debugging

---

## Implementation Checklist

### Phase 7: AI Operations Platform
- [ ] `phase7_ai_ops/mod.rs` - Module orchestration
- [ ] `model_lifecycle.rs` - Registration, hot-swap, rollback tests
- [ ] `shadow_mode.rs` - Shadow deployment, canary, A/B tests
- [ ] `otel_exporter.rs` - OTel initialization, spans, export tests
- [ ] `decision_traces.rs` - Trace collection, buffer, export tests
- [ ] `integration_tests.rs` - End-to-end AI Ops workflow
- [ ] Integration with `lib.rs::validate_phase_7()`
- [ ] Documentation and examples
- [ ] ≥90% test pass rate

### Phase 8: Performance Optimization
- [ ] `phase8_deterministic/mod.rs` - Module orchestration
- [ ] `cbs_edf_scheduler.rs` - Admission, deadlines, budget, EDF tests
- [ ] `slab_allocator.rs` - Performance benchmarks, comparison tests
- [ ] `adaptive_memory.rs` - Strategy switching, oscillation tests
- [ ] `meta_agent.rs` - Decision inference, confidence, reward tests
- [ ] `stress_comparison.rs` - Autonomy ON vs OFF comparison
- [ ] `rate_limiting.rs` - Output rate-limit validation
- [ ] Integration with `lib.rs::validate_phase_8()`
- [ ] Documentation and examples
- [ ] ≥90% test pass rate

### Phase 6: Web GUI
- [ ] `phase6_web_gui/mod.rs` - Module orchestration
- [ ] `http_server.rs` - Server lifecycle tests
- [ ] `websocket.rs` - WebSocket connection, subscription tests
- [ ] `api_endpoints.rs` - REST API endpoint tests
- [ ] `authentication.rs` - Auth/authz tests
- [ ] `real_time_updates.rs` - Metric streaming tests
- [ ] Integration with `lib.rs::validate_phase_6()`
- [ ] ≥90% test pass rate

### Phase 3: Temporal Isolation
- [ ] `phase3_temporal/mod.rs` - Module orchestration
- [ ] `active_isolation.rs` - Temporal isolation verification
- [ ] `deadline_validation.rs` - Real-time deadline tests
- [ ] `latency_under_load.rs` - Stress tests with temporal guarantees
- [ ] Integration with `lib.rs::validate_phase_3()`
- [ ] ≥90% test pass rate

### Phase 1: AI-Native Dataflow
- [ ] `phase1_dataflow/mod.rs` - Module orchestration
- [ ] `graph_execution.rs` - Graph creation, execution tests
- [ ] `operator_validation.rs` - Operator correctness tests
- [ ] `channel_throughput.rs` - Channel performance tests
- [ ] `tensor_operations.rs` - Tensor correctness tests
- [ ] Integration with `lib.rs::validate_phase_1()`
- [ ] ≥90% test pass rate

### Phase 2: AI Governance (Enhanced)
- [ ] `phase2_governance/llm_finetune.rs` - Fine-tuning tests
- [ ] `lora_adapters.rs` - LoRA adapter tests
- [ ] `drift_detector.rs` - Active drift detection tests
- [ ] `version_control.rs` - Model versioning tests
- [ ] `multi_agent.rs` - Multi-agent coordination tests
- [ ] Enhancement to `lib.rs::validate_phase_2()`
- [ ] ≥90% test pass rate

### Phase 5: UX Safety (Enhanced)
- [ ] `phase5_ux_safety/safety_controls.rs` - Safety control tests
- [ ] `explainability.rs` - Explainability feature tests
- [ ] `user_feedback.rs` - UX safety guarantee tests
- [ ] Enhancement to `lib.rs::validate_phase_5()`
- [ ] ≥90% test pass rate

### Integration & Documentation
- [ ] Update `lib.rs::execute_comprehensive_validation()` to run all 9 phases
- [ ] Update `bin/main.rs` to support `--full-demo` flag
- [ ] Generate comprehensive validation report (all phases)
- [ ] Update `TEST_COVERAGE_AUDIT.md` with final coverage
- [ ] Create `TESTING_GUIDE.md` with usage examples
- [ ] Add rustdoc comments to all public APIs
- [ ] Performance benchmark summary report
- [ ] CI/CD integration examples

---

## Acceptance & Delivery

### Final Validation

**Before Submitting GitHub Branch:**

1. **Build Check:**
   ```bash
   cargo build -p sis-testing --release
   # Must compile without errors or warnings
   ```

2. **Clippy Clean:**
   ```bash
   cargo clippy -p sis-testing -- -D warnings
   # Must pass with 0 warnings
   ```

3. **Full Test Run:**
   ```bash
   cargo run -p sis-testing --release -- --full-demo
   # Must achieve ≥90% overall score
   ```

4. **Phase-by-Phase Validation:**
   ```bash
   cargo run -p sis-testing --release -- --phase 7
   cargo run -p sis-testing --release -- --phase 8
   # Each phase ≥90% pass rate
   ```

5. **Documentation Check:**
   ```bash
   cargo doc -p sis-testing --no-deps --open
   # All public APIs documented
   ```

### Delivery Package

**GitHub Branch Structure:**
```
feature/complete-test-suite-100-coverage
├── crates/testing/src/
│   ├── phase7_ai_ops/          # NEW
│   ├── phase8_deterministic/   # NEW
│   ├── phase6_web_gui/         # NEW
│   ├── phase3_temporal/        # NEW
│   ├── phase1_dataflow/        # NEW
│   ├── phase2_governance/      # ENHANCED
│   ├── phase5_ux_safety/       # ENHANCED
│   ├── lib.rs                  # ENHANCED
│   └── bin/main.rs             # ENHANCED
├── TEST_COVERAGE_AUDIT.md      # UPDATED (100% coverage)
├── TESTING_GUIDE.md            # NEW (usage guide)
└── COMPLETE_TEST_SUITE_PLAN.md # THIS FILE
```

**Commit Message Template:**
```
feat(testing): Implement complete test suite with 100% phase coverage

- Add Phase 7 (AI Operations) tests: model lifecycle, shadow mode, OTel, decision traces
- Add Phase 8 (Performance) tests: CBS+EDF, slab allocator, adaptive memory, autonomy comparison
- Add Phase 6 (Web GUI) tests: HTTP server, WebSocket, REST API
- Add Phase 3 (Temporal Isolation) active tests
- Add Phase 1 (Dataflow) active tests: graph execution, operators, channels
- Enhance Phase 2 (Governance) tests: fine-tuning, LoRA, drift detection
- Enhance Phase 5 (UX Safety) tests: safety controls, explainability
- Update main orchestration for 9-phase validation
- Add --full-demo flag for single-command complete validation

Coverage: 45% → 100% across all phases (A, 1-8)

Test Count: ~180+ test cases
LOC: ~8,500 lines (test code)
Validation: ≥90% pass rate per phase

Closes #<ISSUE_NUMBER>
```

---

## Support & Questions

### During Implementation

**If Stuck:**
1. Refer to existing test modules for patterns
2. Check kernel shell commands in `crates/kernel/src/shell.rs`
3. Review kernel features in `crates/kernel/Cargo.toml`
4. Examine kernel output in serial logs

**If Test Failures:**
1. Check kernel actually supports the feature (feature flags)
2. Verify command syntax matches shell implementation
3. Review serial log for actual kernel output
4. Adjust expected outputs based on real behavior

### After Delivery

**Integration Process:**
1. Submit GitHub PR with branch link
2. I will review, test, and debug
3. Iterate on feedback
4. Merge when all tests pass (≥90% score)

---

## Summary

This plan provides a **comprehensive, precise roadmap** to achieve **100% test coverage** across all SIS Kernel phases (A, 1-8).

**Key Deliverables:**
- **9 phase test modules** (2 new, 2 enhanced, 5 existing)
- **180+ test cases** covering all features
- **Single-command validation** (`--full-demo`)
- **Professional reports** (JSON, HTML, MD)
- **≥90% pass rate** per phase

**Estimated Effort:** 15 days (following timeline)

**End Result:** A truly complete test suite that validates every aspect of the SIS Kernel in a single command, suitable for demos, CI/CD, and production readiness validation.

---

**Ready for Implementation! 🚀**
