# Phase 4 Add‑Ons from Modular OS Project

This document is a developer‑facing plan to integrate a small set of high‑value, low‑risk features into the SIS kernel. It is written to be reviewed and followed by future contributors. All changes preserve the kernel’s decoupled, feature‑gated architecture and shell‑first ergonomics.

Scope focuses on five additions:
- Anchored runtime config + quarantine (signed/staged config changes)
- Audit chains (hash‑linked logs) for integrity
- Meta‑agent conflict resolution policy table (weights/tiebreakers)
- Lightweight health monitors on timer ticks (IRQ latency/heap/OOD summaries)
- OOD/Drift severity surfacing (classification + clear shell UI)

All items are feature‑gated, incremental, and default‑off to avoid regressions.

---

## Design Goals

- Keep modules independent with narrow APIs; no shell back‑calls
- Prefer compile‑time features + runtime toggles for safe rollout
- Zero hidden coupling: state encapsulated with `Mutex`/atomics; cross‑agent comms via `agent_bus`
- Keep bring‑up unaltered; avoid long work on the boot path
- Favor simple, verifiable primitives over large frameworks

---

## 1) Anchored Runtime Config + Quarantine

Purpose: Allow reconfiguring sensitive parameters (e.g., autonomy interval, meta thresholds) through a staged, auditable process. When `crypto-real` is enabled, require signatures; otherwise allow demo flows in bring‑up only.

Feature flag: `config-quarantine`

Proposed interfaces:
- New control frame (in `crates/kernel/src/control.rs`):
  - `CTRL_CONFIG_PROPOSE` (id TBD): payload contains key/value updates and optional signature
  - `CTRL_CONFIG_COMMIT`: ID reference to pending change
  - `CTRL_CONFIG_STATUS`: dump active and pending configs
- Shell: `ctlconfig propose key=value [key=value ...] [--sig 0xHEX..128] | commit <id> | status`
- Audit: Emit `metric_kv("config_change", 1)` on commit; track a small config history ring

Data model:
- `ConfigVersion { id, timestamp_us, prev_hash, map: [K:V; M], signer: Option<[u8;32]> }`
- Quarantine queue: pending entries awaiting commit
- Hash derivation: `hash(prev_hash || canonical_map_encoding)` using real SHA‑256 under `crypto-real`, fallback demo hash otherwise

Safety:
- With `crypto-real` ON: require a valid signature (Ed25519) on `canonical_map_hash`
- With `crypto-real` OFF: accept unsigned updates only when `bringup` is enabled; otherwise reject

Integration points:
- A thin `config` module owning state + API: `propose()`, `commit()`, `status()`
- `control.rs`: frame decoding to call the `config` APIs
- Shell: `ctlconfig` command binds to the same APIs

Testing:
- Propose invalid map → reject; wrong signature → reject; commit non‑existent → reject
- Happy path: propose → commit → status shows active and chain integrity

---

## 2) Audit Chains (Hash‑Linked Logs)

Purpose: Provide tamper‑evident logs for critical audit rings without heavy storage.

Feature flag: `audit-chain`

Targets:
- Neural audit ring (`neural.rs`)
- LLM audit log (`llm.rs`)
- Autonomy audit (decisions and rationale)

Design:
- Each entry includes `prev_hash` and `entry_hash = H(prev_hash || entry_fields_canonical)`
- Provide `verify_chain()` API: starts at head, walks back until empty/default
- JSON dump (`nnjson`, `llmjson`) includes `chain_ok=true|false`, head hash

Hash function:
- With `crypto-real`: SHA‑256
- Without: existing demo hash helper (documented as non‑security)

Testing:
- Inject a corruption (during tests/dev feature) and verify `chain_ok=false`
- Normal runs return `true`

---

## 3) Meta‑Agent Conflict Resolution Policy Table

Purpose: Make directive arbitration explicit, tunable, and explainable.

Feature flag: `meta-weights` (or integrate directly if minimal)

Data structure:
- `MetaArbitrationConfig { w_mem: u8, w_sched: u8, w_cmd: u8, tiebreaker: enum { MemFirst, SchedFirst, CmdFirst } }`

Behavior:
- `force_meta_decision()` computes a composite score (weighted sum) and applies the tiebreaker when magnitudes conflict
- Explainability: store a concise reason code alongside action (e.g., `ARB_MEM_DOMINANT`)

Shell:
- `metaclassctl config --weights M:S:C [--tiebreaker mem|sched|cmd]`
- `metaclassctl status` prints weights and last arbitration rationale

Testing:
- Vary weights; ensure decisions change deterministically with identical telemetry
- Tiebreakers apply only under ties or near‑ties (define a small epsilon)

---

## 4) Lightweight Health Monitors on Timer Ticks

Purpose: Summarize basic system health periodically without spamming or blocking.

Feature flag: `health-monitor`

Checks (quick, bounded):
- IRQ latency summary (mean/min/max over last N samples)
- Heap churn/fragmentation snapshot (via `heap::get_heap_stats()`)
- OOD/drift snapshot from `prediction_tracker`

Operation:
- Hook into virtual timer ISR path only when no latency benchmark is running and `AUTONOMY_READY` is true
- Compute at low cadence (e.g., every 1–5 seconds), amortized and bounded
- Publish a compact message on `agent_bus`; emit a single METRIC summary line per interval

Shell:
- `healthctl status` — print last summary
- `healthctl run-now` — compute once on demand
- `healthctl config --interval-ms N` (optional)

Testing:
- Confirm cadence and that ISR remains bounded (no long loops)
- Toggle on/off via feature flag

---

## 5) OOD/Drift Severity Surfacing

Purpose: Turn raw OOD/drift metrics into actionable status with simple classes: OK/WARNING/ALERT.

Feature flag: piggyback on `health-monitor` or a small `drift-ui` (dev‑facing)

Design:
- Classify current distance vs threshold and trend of distribution snapshots
- `prediction_tracker` returns `(class, distance, threshold, sample_count)` where `class ∈ {0=OK,1=WARNING,2=ALERT}`
- Shell (`autoctl driftcheck`) prints classification, reasons, and last N deltas

Testing:
- Force synthetic states; verify classification logic boundaries

---

## 6) Useful With Adaptation (Additional Add‑Ons)

These improvements are optional and low‑risk, and can be integrated behind small feature flags or folded into existing modules.

### 6.1 Identity Beacons (Signed Pings)

Purpose: Emit a verifiable kernel “identity” for audit streams and multi‑host traceability.

Feature flag: `id-beacon` (enabled only alongside `crypto-real` for signing)

Design:
- Compute a beacon over `boot_time || build_id` (and optionally a monotonic counter) and sign under `crypto-real` using the build public key.
- Emit once at boot (as a METRIC) and expose on demand via shell.

Shell:
- `ctlkey` and `verify` show whether a beacon/signature is present and valid.

Integration points:
- Small helper in a new `id_beacon.rs` or within `time.rs`/`trace.rs` under feature flag.
- Store the beacon signature in a static once‑cell; do not recompute frequently.

Testing:
- With `crypto-real` ON and a valid key: verification succeeds.
- With `crypto-real` OFF: do not expose signing; optionally print an unsigned stamp in bring‑up only.

### 6.2 Strategic Scrolls (Explanations/Rationales)

Purpose: Enrich autonomy audit entries with compact rationale codes and optional short explanations for post‑mortem clarity.

Feature flag: `rationales` (or integrate into existing autonomy feature set)

Design:
- Add a small “reason code registry” (enum → short bytes) mapped to known conditions (e.g., MEM_HIGH_FRAG, OOD_ALERT, WATCHDOG_LOW_REWARD).
- Autonomy audit entries include `explanation_code` (already present) plus an optional short string when available.

Shell:
- `autoctl audit last N` prints rationale alongside actions.

Integration points:
- Extend autonomy audit record and printer; keep strings short and bounded.

Testing:
- Verify rationale codes match conditions across known scenarios (memory stress, OOD alert).

### 6.3 Allocation Prediction History (Top‑K Summary)

Purpose: Demonstrate value of predictive memory by surfacing the most accurate command types.

Feature flag: reuse `predictive_memory` (no new flag)

Design:
- Compute a simple Top‑K list of commands with most accurate size predictions (within 20%).
- Show counts and accuracy percent for each of Top‑K.

Shell:
- `memctl learn stats` prints Top‑K summary after the main stats block.

Integration points:
- Add a helper in `predictive_memory.rs` to assemble Top‑K from existing predictors.

Testing:
- Seed predictors with synthetic data and check ranking and accuracy calculation.

### 6.4 Agent Registry (Producers/Consumers + Liveness)

Purpose: Increase observability without tight coupling by registering agents and tracking liveness.

Feature flag: `agent-registry` (tiny; can be folded into `agent_bus`)

Design:
- Extend `agent_bus` with a registry of producers/consumers and a lightweight heartbeat counter per agent.
- Agents call `register(name, role)` once and `heartbeat(name)` periodically (best‑effort, low cadence).

Shell:
- `agentctl bus|stats` prints the registry table (name, role, last_seen, counters).

Integration points:
- `agent_bus.rs`: registry data structure + APIs (`register`, `heartbeat`, `get_registry`).
- Do not make scheduling depend on registry presence.

Testing:
- Register a few agents (memory/meta/actor) and verify they appear with updated liveness.

---

## Cross‑Cutting Concerns

- Feature flags (Cargo): `config-quarantine`, `audit-chain`, `health-monitor`, `meta-weights`
- Runtime toggles: minimal; avoid expanding global state surface area
- Performance: all new work is O(1) per event or amortized; no heap growth from ISR
- Security: signature requirement under `crypto-real` for config; log hashes never reveal secrets
- Observability: single‑line METRICs and shell status views; JSON dumps provide full detail on demand
- Platform neutrality: No direct MMIO or hardcoded bases; all add‑ons must be platform‑agnostic and use only module APIs and atomics/Mutex. Do not reach into UART/GIC from add‑ons.

---

## API & Code Touchpoints (Minimal)

- `crates/kernel/src/control.rs`: add config frames; helper to dispatch to config APIs
- `crates/kernel/src/config.rs` (new): in‑memory config versions, propose/commit/status
- `crates/kernel/src/neural.rs`, `crates/kernel/src/llm.rs`, `crates/kernel/src/autonomy.rs`: add `prev_hash`/`entry_hash` fields under `audit-chain`
- `crates/kernel/src/meta_agent.rs`: apply arbitration weights; store rationale code
- `crates/kernel/src/agent_bus.rs`: add a small “health” message type (id + payload)
- `crates/kernel/src/health.rs` (new): summaries + state for last health snapshot
- `crates/kernel/src/shell.rs`: add `ctlconfig` and `healthctl` commands; extend `metaclassctl`
- `crates/kernel/Cargo.toml`: new feature flags; keep default empty

---

## Implementation Plan (Phased)

Phase A — Foundations (1–2 days)
- Add feature flags (no code executed by default)
- Create `config.rs` skeleton and wire `control.rs` frame IDs (behind `config-quarantine`)
- Add `health.rs` skeleton and agent_bus message enum extension (behind `health-monitor`)

Phase B — Audit Chains (1–2 days)
- Implement hash linking for neural, LLM, autonomy audit rings (behind `audit-chain`)
- Add `verify_chain()` and surface `chain_ok` in JSON dumps
- Metrics: `metric_kv("audit_chain_ok", 0|1)` on verification

Phase C — Meta Arbitration (1 day)
- Introduce `MetaArbitrationConfig` + `metaclassctl --weights …` (behind `meta-weights`)
- Apply weights in `force_meta_decision()`; record rationale code

Phase D — Config Quarantine (2–3 days)
- Implement propose/commit/status; Ed25519 verification under `crypto-real`
- Shell `ctlconfig`; control frame support
- Add small ring of committed versions with `prev_hash` chain

Phase E — Health Monitor + OOD UI (1–2 days)
- Compute periodic summaries (bounded) and publish on `agent_bus`
- `healthctl status|run-now`; optional cadence config
- Extend `autoctl driftcheck` with classification

Phase F — Docs/Tests/Hardening (1–2 days)
- Update `docs/ARCHITECTURE.md` with new features
- Add unit‑style tests where feasible (hash link, config verify)
- Bench ISR budget for health checks; tune cadence

Optional Phase G — Useful‑With‑Adaptation Add‑Ons (1–2 days each, independent)
- Identity Beacons (`id-beacon` + `crypto-real`): sign/emit boot identity; shell shows validity
- Strategic Scrolls (`rationales`): enrich autonomy audit with rationale strings
- Allocation Top‑K (inside `predictive_memory`): add Top‑K summary to `memctl learn stats`
- Agent Registry (`agent-registry`): registry + liveness in `agent_bus`; expose via `agentctl`

---

## Acceptance Criteria

- Build: All features compile behind flags; default build unchanged; bring‑up path unaffected
- Audit chains: `nnjson`/`llmjson` show `chain_ok` and head hash when enabled
- Meta weights: `metaclassctl status` prints weights; decisions change predictably with weights
- Config quarantine: `ctlconfig propose/commit/status` works; signature enforced under `crypto-real`
- Health monitor: `healthctl status` prints recent summary; no ISR stalls; metrics show periodic health
- Docs: This plan + architecture doc updated; shell help shows new commands and usage

---

## Risks & Mitigations

- ISR budget: Keep health calculations tiny; compute pieces across ticks if needed; guard with feature and cadence
- Config spoofing: Enforce signature only when `crypto-real` is ON; otherwise reject in production builds; allow demo under `bringup`
- Hash link false confidence: Clearly label demo hash as non‑security; prefer SHA‑256 with `crypto-real`
- Scope creep: Features are independent; deliver in phases; default‑off until tested

---

## Rollback Plan

- Feature‑gated: disable flags to remove functionality entirely
- Minimal code paths in core modules; no invasive type/ABI changes
- Audit chain and health monitor are additive; can be dropped without affecting others

---

## Developer Notes

- Follow existing patterns: `spin::Mutex`, `metric_kv`, small rings, `agent_bus`
- Keep shell bindings thin; do validation in module APIs
- Use integer‑only logic; avoid heap in ISRs
- Prefer explicit print banners for new commands; conform to existing help style

---

## Tracking & Ownership

- DRI: Kernel subsystem maintainer
- Reviewers: Security (config signatures), Performance (ISR), AI/ML (meta weights, OOD UI)
- Milestones align to Phases A–F above; ship features independently once verified
