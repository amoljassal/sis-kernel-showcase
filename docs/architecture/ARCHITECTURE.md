# SIS Kernel Architecture

This document describes the modular architecture of the SIS kernel, how subsystems interact, and how each can be enabled, disabled, or extended independently. Where useful, it links to source files and lists shell commands bound to each module.

## Design Principles

- Narrow, stable APIs between modules; no hidden cross-dependencies.
- Encapsulated state using `spin::Mutex` or `Atomic*`; no shared `&mut` across modules.
- Event gating via `AUTONOMY_READY` and a periodic virtual timer instead of tight loops.
- Loose coupling via a simple message bus (`agent_bus`) for cross-agent communication.
- Compile-time features (`llm`, `deterministic`, `crypto-real`, `virtio-console`, `arm64-ai`) to drop entire subsystems.
- Runtime shell toggles (`metricsctl`, `neuralctl`, `autoctl`, `metaclassctl`, `mlctl`, `actorctl`) for safe enable/disable.

## Core Runtime

- Shell — `crates/kernel/src/shell.rs`
  - Purpose: Operator control, inspection, demos, and stress scenarios.
  - Feature gates: Commands grouped under `llm`, `deterministic`, `virtio-console`, `arm64-ai`.
  - Notes: Pure orchestrator; calls public APIs of other modules; no back-dependencies.

- IRQ/Timer + Autonomy Gate — `crates/kernel/src/main.rs` (vectors), `crates/kernel/src/autonomy.rs`
  - Purpose: GICv3 PPI 27 virtual timer drives periodic autonomous decisions when `AUTONOMY_READY` is set.
  - Runtime control: `autoctl on|off|status|interval N`.
  - Notes: Timer ISR calls `autonomy::autonomous_decision_tick()`; gate is an `AtomicBool`.

## AI + Control

- Memory Agent — `crates/kernel/src/neural.rs`
  - Purpose: Fixed‑point MLP predicts command success, operator health, and memory health; keeps neural audit logs.
  - Runtime control: `neuralctl ...` (status/reset/infer/update/teach/retrain/learn/config/audit), `nnjson`, `nnact`.
  - Notes: State behind `Mutex`; callable without shell.

- Predictive Memory — `crates/kernel/src/predictive_memory.rs`
  - Purpose: Predict fragmentation, advise compaction, select allocation strategy, and track per-command sizes.
  - Runtime control: `memctl status|predict|stress|strategy <status|test>|learn stats`.
  - Notes: Uses heap stats and meta-agent telemetry getters; no backloops.

- Meta‑Agent — `crates/kernel/src/meta_agent.rs` and Agent Bus — `crates/kernel/src/agent_bus.rs`
  - Purpose: Fuse telemetry (memory/scheduling/command) and issue coordinated directives; publish messages on a bus.
  - Runtime control: `metaclassctl status|force|config --interval/--threshold|on|off`, `metademo`.
  - Coordination: `coordctl process|stats`, `coorddemo`; bus inspection: `agentctl bus|stats|clear`.

- Experience Replay + Advanced ML (inside meta‑agent)
  - Runtime control: `mlctl status|replay N|weights P W L|features --replay on/off --td on/off --topology on/off`.

- Actor‑Critic Policy (inside meta‑agent)
  - Runtime control: `actorctl status|policy|sample|lambda N|natural on/off|kl N|on|off`.

- Prediction Tracking & OOD — `crates/kernel/src/prediction_tracker.rs`
  - Purpose: Track prediction accuracy, detect out‑of‑distribution and drift, adapt learning rate.
  - Runtime control: `learnctl stats|dump|train|feedback good|bad|verybad <decision_id>`, `autoctl oodcheck|driftcheck`.

## Workload + Control Plane

- Stress Test Engine — `crates/kernel/src/stress_test.rs`
  - Purpose: Memory/command/multi/learning/red‑team/chaos/compare/report scenarios; emits metrics.
  - Runtime control: `stresstest memory --duration MS --target-pressure PCT`, `stresstest commands --duration MS --rate RPS`, `stresstest multi|learning|redteam|chaos|compare|report`.

- Graph + Control Plane — `crates/kernel/src/graph.rs`, `crates/kernel/src/control.rs`
  - Purpose: Build channels/operators, start runs, compute stats, predictive helpers.
  - Runtime control: `graphctl create|add-channel|add-operator [--in/--out/--prio/--stage/--in-schema/--out-schema]|start <steps>|det <wcet_ns> <period_ns> <deadline_ns>|stats|export-json|predict …|feedback …`.

- Deterministic Scheduling (feature: `deterministic`) — `crates/kernel/src/deterministic.rs`
  - Purpose: CBS+EDF admission/jitter/deadline hooks and LLM budgeting.
  - Runtime control: `det on|off|status|reset`, `llmctl status` (when `llm` is also enabled).

## LLM + Security

- LLM Service (feature: `llm`) — `crates/kernel/src/llm.rs`
  - Purpose: Kernel‑resident inference service with budgets, audit, and graph-backed streaming.
  - Runtime control: `llmctl load|budget|status|audit`, `llminfer`, `llmstream`, `llmgraph`, `llmjson`, `llmstats`, `llmpoll`, `llmcancel`, `llmsummary`, `llmverify`, `llmhash`, `llmkey`.

- Model Security (feature: `crypto-real`) — `crates/kernel/src/model.rs`
  - Purpose: SHA‑256 + Ed25519 verification, permissions (LOAD/EXECUTE/INSPECT/EXPORT/ATTEST), and audit logging.
  - Notes: Enabled with `--features crypto-real` and `SIS_ED25519_PUBKEY` at build time.

## Phase 2: AI Governance

Feature set bundled under the meta‑features `ai-ops` and `llm`:

- Multi-Agent Orchestration (`ai-ops`) — `crates/kernel/src/ai/`:
  - `orchestrator.rs`: Consensus-driven multi-agent decision making (unanimous, majority voting, safety overrides).
  - `conflict.rs`: Conflict resolution strategies (priority-based, voting, synthesis, human escalation).
  - Global instance: `crate::ai::ORCHESTRATOR` — tracks total decisions, unanimous/majority/safety override counts, average latency.
  - Shell: `coordctl status [--json] | conflict-stats [--json] | history [--json] | agents [--json] | priorities [--json]`.

- Deployment Phase Management (`ai-ops`) — `crates/kernel/src/ai/deployment.rs`:
  - Purpose: Progressive rollout across deployment phases (A: Learning, B: Shadow, C: Canary10, D: Canary100, E: Production).
  - Features: auto-advance with health gates, auto-rollback on failure, phase transition tracking.
  - Global instance: `crate::ai::DEPLOYMENT_MANAGER` — tracks current phase, rollout history, health metrics.
  - Shell: `deployctl status [--json] | history [--json] | advance | rollback | config [--auto-advance on|off] [--auto-rollback on|off]`.

- Model Drift Detection (`llm`) — `crates/kernel/src/llm/drift_detector.rs`:
  - Purpose: Monitor AI model performance degradation, detect drift, trigger retraining.
  - Features: ring buffer for predictions, baseline vs current accuracy tracking, warning/critical thresholds, auto-retrain.
  - Global instance: `crate::llm::DRIFT_DETECTOR` — baseline 90% accuracy, 5% warning threshold, 15% critical threshold.
  - Shell: `driftctl status [--json] | history [--json] | retrain | reset-baseline`.

- Adapter Version Control (`llm`) — `crates/kernel/src/llm/version.rs`:
  - Purpose: Git-like versioning for LoRA adapters (commit, rollback, tagging, diffing).
  - Features: version history with metadata, tag management, weight diff computation, rollback to any version.
  - Global instance: `crate::llm::VERSION_CONTROL` — manages adapter version lifecycle.
  - Shell: `versionctl status [--json] | history [--json] | commit <message> | rollback <version_id> | tag <name> <version_id>`.

JSON Output Support:
- All Phase 2 commands support dual output modes: human-readable (default) and JSON (with `--json` flag).
- JSON mode enables programmatic access via API endpoints and external tooling.
- Manual JSON serialization for no_std environment (no serde).

## Phase 7: AI Operations Platform (AI‑Ops)

Feature set bundled under the meta‑feature `ai-ops`:

- Model Lifecycle (`model-lifecycle`) — `crates/kernel/src/model_lifecycle/`:
  - `registry.rs`: ext4‑backed JSONL registry (JSON history at `/models/registry.log`), health metrics, status (Active/Shadow/Rollback).
  - `lifecycle.rs`: atomic hot‑swap (RCU), dry‑swap (load+health without state change), symlink stub updates under `/models/<link>`.
  - `health.rs`: latency P99, memory footprint, accuracy checks (configurable thresholds).
  - Shell: `modelctl dry-swap <ver> | swap <ver> | load <ver> | history [N] | rollback | status`.

- Decision Traces (`decision-traces`) — `crates/kernel/src/trace_decision/`:
  - `decision.rs`: DecisionTrace schema (context, predictions, policy checks, outcome).
  - `buffer.rs`: ring buffer (1024) with `stats`, `get_last_n`, `find_by_trace_id`.
  - `export.rs`: incident bundles to `/incidents/INC-<ts>-NNN.json` (config + system snapshot + traces).
  - Shell: `tracectl demo [N] | list [N] | show <id> | export <id...> | export-divergences [N] | stats`.

- Shadow / Canary (`shadow-mode`) — `crates/kernel/src/shadow/`:
  - `agent.rs`: shadow agent (LogOnly, Compare, Canary), divergence counters, threshold, dry‑run mode.
  - `compare.rs`: comparison result state machine.
  - `rollback.rs`: rollback trigger helpers (rate/threshold checks).
  - `divergence.rs`: ring buffer logging (timestamp, delta, matches, mode, optional trace_id) for incident export.
  - Shell: `shadowctl enable <ver> | disable | status | promote | threshold <N> | mode <log|compare|canary10|canary100> | canary <10|100> | dry-run on|off|status`.

- Observability (`otel`) — `crates/kernel/src/otel/`:
  - `exporter.rs`: convert DecisionTraces into OTel‑approx spans (feature gated, JSON encoding under `alloc`).
  - `drift.rs`: drift monitor (moving average vs baseline), warning/alert callbacks.

SRE integration:
- Bundles under `/incidents/INC-*.json` (atomic writes). Collect on host via RO mount or QMP‑triggered agent.
- History under `/models/registry.log` (JSONL). Viewer: `modelctl history [N]`.

Persistence options:
- Initramfs models (`initramfs-models`): embed a newc CPIO with `/models/...`; unpacked during bring‑up.
- Block image mount overlay: when an extra VirtIO block device is present, a best‑effort read‑only ext2 mount overlays `/models`.
- ext4/JBD2 harness: `scripts/ext4_durability_tests.sh` runs journaled writes, simulated power cut via QMP, replay on reboot, and optional host `fsck.ext4`.

Networking support (Phase C updates):
- DHCP with retry/backoff and static fallback; optional SNTP client (`sntp`) to query a time source.

Graphics support (Phase G):
- VirtIO‑GPU modern path stabilized; framebuffer initialized to XRGB8888; noisy logs gated.

Shell helpers for demos:
- `ls [path]`, `cat <path>` to inspect VFS files (e.g., `/models/registry.log`, `/incidents/INC-*.json`).

Relevant features:
- `ai-ops` (meta): enables `model-lifecycle`, `decision-traces`, `shadow-mode`, `otel`.
- `initramfs-models`: embeds CPIO pointed to by `INITRAMFS_MODELS` env; unpacked during bring‑up.

## Devices + Platform

- NPU Emulation + Driver — `crates/kernel/src/npu.rs`, `crates/kernel/src/npu_driver.rs`
  - Runtime control: `npudemo`, `npudriver`.

- Driver Framework + VirtIO Console (feature: `virtio-console`) — `crates/kernel/src/driver.rs`, `crates/kernel/src/virtio_console.rs`
  - Runtime control: `vconwrite <text>` when feature-enabled.

- Metrics/Tracing — `crates/kernel/src/trace.rs`
  - Runtime control: `metricsctl on|off|status`, `metrics [ctx|mem|real]`.

- PMU/Time/Heap — `crates/kernel/src/pmu.rs`, `crates/kernel/src/time.rs`, `crates/kernel/src/heap.rs`
  - Notes: Boot-time heap self-tests; `pmu` and `mem` info commands.

- UART/Boot/Interrupts — `crates/kernel/src/uart.rs`, `crates/kernel/src/main.rs`
  - Notes: Low-level I/O, vector table, GICv3 init, timer setup. Higher modules do not directly touch MMIO.

## Interaction Model

- Timer tick (PPI 27) drives periodic `autonomy::autonomous_decision_tick()` when `AUTONOMY_READY` is set.
- Shell invokes module APIs; modules never call back into the shell.
- Cross-agent coordination occurs through `agent_bus` plus `meta_agent::collect_telemetry()`/`force_meta_decision()`.
- Metrics are emitted via `trace::metric_kv` and optionally summarized by shell.

## Enabling/Disabling Modules

- Compile-time features: `llm`, `ai-ops`, `deterministic`, `crypto-real`, `virtio-console`, `arm64-ai`, `perf-verbose`, `graph-demo`, `graph-autostats`.
  - **Phase 2 AI Governance**: Requires `ai-ops` (orchestration, deployment, conflicts) and/or `llm` (drift detection, version control).
- Runtime toggles: `metricsctl`, `neuralctl`, `autoctl`, `metaclassctl`, `mlctl`, `actorctl`.

## Shell Command Map (by module)

| Module | Commands |
|---|---|
| Shell core | `help`, `clear`, `mem`, `regs`, `pmu` |
| Metrics/Tracing | `metricsctl on|off|status`, `metrics [ctx|mem|real]` |
| Autonomy | `autoctl on|off|status|interval N|limits|audit last N|rewards --breakdown|anomalies|verify|explain ID|dashboard|checkpoints|saveckpt|restoreckpt N|restorebest|tick|oodcheck|driftcheck` |
| Memory Agent | `neuralctl status|reset|infer|update|teach|retrain N|selftest|learn on|off [limit]|tick|dump|load <in hid out>|demo-metrics N|config --confidence/--boost/--max-boosts|audit`, `nnjson`, `nnact` |
| Predictive Memory | `memctl status|predict|stress [N]|strategy <status|test>|learn stats` |
| Meta‑Agent | `metaclassctl status|force|config --interval N --threshold N|on|off`, `metademo` |
| **Phase 2: Multi-Agent Orchestration** | `coordctl status [--json]|conflict-stats [--json]|history [--json]|agents [--json]|priorities [--json]` |
| **Phase 2: Deployment Management** | `deployctl status [--json]|history [--json]|advance|rollback|config [--auto-advance on|off] [--auto-rollback on|off]` |
| **Phase 2: Model Drift Detection** | `driftctl status [--json]|history [--json]|retrain|reset-baseline` |
| **Phase 2: Adapter Version Control** | `versionctl status [--json]|history [--json]|commit <message>|rollback <version_id>|tag <name> <version_id>` |
| Coordination/Bus (Legacy) | `coordctl process|stats`, `coorddemo`, `agentctl bus|stats|clear` |
| Actor‑Critic | `actorctl status|policy|sample|lambda N|natural on/off|kl N|on|off` |
| Advanced ML | `mlctl status|replay N|weights P W L|features --replay on/off --td on/off --topology on/off` |
| Stress Engine | `stresstest memory --duration MS --target-pressure PCT`, `stresstest commands --duration MS --rate RPS`, `stresstest multi|learning|redteam|chaos|compare|report` |
| Graph/Control Plane | `graphctl create|add-channel <cap>|add-operator <op_id> [--in/--out/--prio/--stage/--in-schema/--out-schema]|start <steps>|det <wcet_ns> <period_ns> <deadline_ns>|stats|export-json|predict …|feedback …` |
| Deterministic (feature) | `det on <wcet_ns> <period_ns> <deadline_ns>|off|status|reset` |
| LLM (feature) | `llmctl load …|budget …|status|audit`, `llminfer`, `llmstream`, `llmgraph`, `llmjson`, `llmstats`, `llmpoll`, `llmcancel`, `llmsummary`, `llmverify`, `llmhash`, `llmkey` |
| NPU | `npudemo`, `npudriver` |
| VirtIO Console (feature) | `vconwrite <text>` |

> Notes:
> - Commands under “feature” rows require the corresponding Cargo feature at build time.
> - Most commands print their own usage lines if invoked without required parameters.

## How To Extend A Module (Template)

This checklist shows how to add a new subsystem that is decoupled, feature‑gated, and shell‑controlled.

1) Define the module API and state

```rust
// crates/kernel/src/my_module.rs
use spin::Mutex;

pub struct Config { pub enabled: bool, pub level: u8 }
static STATE: Mutex<Config> = Mutex::new(Config { enabled: false, level: 1 });

pub fn get_config() -> Config { STATE.lock().clone() }
pub fn set_config(c: Config) { *STATE.lock() = c; }

pub fn run_once() -> usize {
    let c = STATE.lock().clone();
    if !c.enabled { return 0; }
    // do work; return a metricable result
    42
}
```

Expose it from `main.rs` behind a feature:

```rust
// crates/kernel/src/main.rs
#[cfg(feature = "my-module")] pub mod my_module;
```

Add a Cargo feature (optional):

```toml
# crates/kernel/Cargo.toml
[features]
my-module = []
```

2) Wire into bring‑up (optional)

- Keep bring‑up minimal; call your module only after UART/heap/IRQ are ready.
- Avoid long blocking work on the boot path. Prefer shell‑triggered entry points.

3) Bind shell commands

Add a dispatcher arm and a handler method in `shell.rs`:

```rust
// match parts[0] { …, "mymodctl" => { self.cmd_mymodctl(&parts[1..]); true }, … }

fn cmd_mymodctl(&self, args: &[&str]) {
    #[cfg(not(feature = "my-module"))]
    unsafe { crate::uart_print(b"[MYMOD] feature not enabled\n"); return; }

    #[cfg(feature = "my-module")]
    {
        if args.is_empty() { unsafe { crate::uart_print(b"Usage: mymodctl <on|off|status|level N|run>\n"); } return; }
        match args[0] {
            "on" => { let mut c = crate::my_module::get_config(); c.enabled = true; crate::my_module::set_config(c); unsafe { crate::uart_print(b"[MYMOD] ON\n"); } }
            "off" => { let mut c = crate::my_module::get_config(); c.enabled = false; crate::my_module::set_config(c); unsafe { crate::uart_print(b"[MYMOD] OFF\n"); } }
            "status" => { let c = crate::my_module::get_config(); unsafe { crate::uart_print(b"[MYMOD] "); crate::uart_print(if c.enabled { b"ENABLED\n" } else { b"DISABLED\n" }); } }
            "level" => { if args.len()<2 { unsafe { crate::uart_print(b"Usage: mymodctl level <0-255>\n"); } return; } let v=args[1].parse::<u8>().unwrap_or(1); let mut c=crate::my_module::get_config(); c.level=v; crate::my_module::set_config(c); }
            "run" => { let n = crate::my_module::run_once(); unsafe { crate::uart_print(b"[MYMOD] result="); self.print_number_simple(n as u64); crate::uart_print(b"\n"); } }
            _ => unsafe { crate::uart_print(b"Usage: mymodctl <on|off|status|level N|run>\n"); }
        }
    }
}
```

4) Keep it decoupled

- No calls back into shell from the module.
- Encapsulate state in `Mutex` or atomics; avoid global `static mut` references.
- Expose a narrow API (`get_config/set_config`, `run_once`, etc.).
- If periodic work is required, trigger from the timer tick via a single `autonomy` hook (guarded by a gate), not via busy loops.

5) Telemetry and metrics

- Emit metrics with `crate::trace::metric_kv("my_mod_result", value)`; keep UART noise optional (behind `metricsctl`).
- For cross‑agent interaction, publish messages via `agent_bus` instead of direct calls.

6) Validation tips

- Add a `stresstest` sub‑mode or reuse the engine to exercise your module.
- Prefer bounded loops and timeouts; avoid heap growth during IRQ or bring‑up windows.
- Gate new code behind a feature until stabilized.
