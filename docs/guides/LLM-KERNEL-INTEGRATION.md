# LLM Kernel Integration Plan

Status: Draft (feature-gated via `llm`)

This document describes how to integrate a Large Language Model (LLM) as a first‑class, kernel‑resident service in the SIS kernel. The LLM becomes a privileged operator within the kernel’s dataflow graph, scheduled by the deterministic CBS+EDF runtime, controlled via the existing control plane and shell, protected by capabilities and model signature verification, and observed through the kernel’s metrics pipeline.

## Objectives

- Make the LLM a core kernel service, not a userland add‑on.
- Treat LLM inference as a first‑class graph operator with zero‑copy buffers.
- Schedule LLM work under CBS+EDF with real‑time bounds and quotas.
- Secure model packages with signatures and capability gating.
- Expose a minimal, audited interface for prompts/results via control plane and shell.
- Maintain observability parity (latency, jitter, throughput, stalls) with existing operators.

## Scope

In scope:
- Kernel‑resident LLM “operator” scaffolding and control interfaces.
- Deterministic scheduling (CBS+EDF) integration and admission control.
- Metrics, schema extensions, dashboards, and tests in `sis-testing`.
- Capability and audit integration with model/package security.

Out of scope (initial phases):
- Hosting large, production LLM weights fully in kernel memory.
- Full tokenizer implementation; initial phases may stream raw bytes or use a minimal tokenizer.
- Full-blown hardware offload; start with a stub and optional coprocessor path, then iterate.

## Architecture Overview

### High-Level

1) Kernel‑resident LLM Operator (feature `llm`):
- New module `llm.rs` implements an operator interface (run over input buffers to produce token chunks).
- Integrated with `graph.rs` as a typed operator; inputs are prompt buffers, outputs are token buffers.

2) Dataflow Integration:
- Use existing SPSC ring channels for prompt/token streams.
- Zero‑copy handles via `tensor::BumpArena` for text buffers (typed schema id for TEXT/TOKENS).
- Graph wiring through `graphctl` / shell convenience APIs.

3) Deterministic Scheduling:
- Represent LLM inferences as jobs under CBS servers with EDF ordering.
- Admission control via utilization ppm; budgets per prompt/token burst.
- Enforce deadlines and track jitter/misses, consistent with Phase 2.

4) Control Plane + Shell:
- New control frames for model management and inference (load/start/poll/cancel).
- Shell commands (`llmctl`, `llminfer`) for interactive testing and admin flows.

5) Security:
- Model packages are signed (SHA‑256 + Ed25519), verified on load.
- Capabilities extended with `LLM` object kind and rights: SUBMIT, STREAM, ADMIN.
- Audit logs for submit, load/unload, budget decisions, and rejections.

6) Observability:
- Metrics: latency, tokens/sec, queue depth, rejects, deadline misses.
- Schema additions and dashboard cards in `target/testing` outputs.

7) Coprocessor / Hardware Path:
- For large prompts/models, route to coprocessor (NPU emulation or virtio device) using existing MMIO/virtio plumbing.
- Unify metrics across local and coprocessor backends.

## Interfaces & Data Types

### Operator Interface

- `Ll mOperator` (kernel‑internal): consumes a prompt buffer, emits token chunks.
- Runs inside the graph executor (`graph.run_steps`) with strict no‑alloc sections when in deterministic mode.

### Buffer Schemas (tensor headers)

- `SCHEMA_TEXT` (prompt): UTF‑8 bytes; header fields indicate length and encoding.
- `SCHEMA_TOKENS` (output): token ids or UTF‑8 chunk; header indicates framing (ids or bytes), count, and sequence index.

### Control Plane Frames (V0)

- `LLM_LOAD {token, model_meta, signature}`: load/activate model (cap: ADMIN).
- `LLM_INFER_START {token, prompt_desc, limits}`: enqueue prompt (cap: SUBMIT).
- `LLM_INFER_POLL {token, infer_id}`: non‑blocking fetch of available output tokens.
- `LLM_CANCEL {token, infer_id}`: cancel an in‑flight prompt.

Notes:
- All payloads are prefixed with the existing 64‑bit capability token.
- Frames are deliberately small; prompt bytes are provided via a graph channel or shared buffer handle id.

### Shell Commands

- `llmctl load [--model <id>] [--wcet-cycles N] [--ctx N]`
- `llminfer "<prompt>" [--max-tokens N] [--prio P]`
- `llmstats` (prints queue depth, throughput, deadline misses)

### Optional Syscalls (future)

- Direct userland API can be added after control‑plane stabilizes, using the same capability enforcement and audited entry points.

## Scheduling & Determinism

- Admission: translate `wcet_cycles` to ns and use `AdmissionController::try_admit`.
- CBS Servers: one per LLM service (or per priority class) with configured max tokens per period.
- EDF: per‑inference absolute deadlines (based on prompt class); earliest deadline runs first.
- Budget Enforcement: preempt/stop token generation when budget exhausted; report `llm_deadline_miss_count`.
- Backpressure: enforce limits for in‑flight inferences; reject when queue full; emit `llm_rejects`.

## Security Model

- Capability Rights:
  - `LLM::ADMIN`: load/unload models, set budgets/limits.
  - `LLM::SUBMIT`: submit prompts for inference (quota‑limited).
  - `LLM::STREAM`: read token streams from output channels/control plane.
- Audit Events:
  - Model load/unload, signature result, capability checks.
  - Prompt submission (size, caller id), budget decisions, rejections.
  - Completion/cancel and token counts (no prompt bodies in logs).
- Quotas & Limits:
  - Max prompt bytes, max tokens, per‑tenant rate limits, queue capacity per class.

## Metrics & Observability

Per‑inference samples (arrays, summarized in `summary`):
- `llm_infer_us`: inference latency per request (us)
- `llm_tokens_out`: tokens produced per request
- `llm_deadline_miss_count`: misses per interval
- `llm_queue_depth_max`: peak queue depth
- `llm_rejects`: number of rejected submissions due to quotas/backpressure
- Optional PMU: cycles and cache refills (QEMU limitations apply)

Schema Additions (docs/schemas/sis-metrics-v1.schema.json):
- Arrays: `llm_infer_us`.
- Scalars: `llm_tokens_out`, `llm_deadline_miss_count`, `llm_queue_depth_max`, `llm_rejects`.
- Summary: `llm_infer_p50_us`, `llm_infer_p95_us`, `llm_infer_p99_us`, `llm_tokens_per_sec`, `llm_jitter_p99_ns`.

Dashboard:
- Add an LLM card: current queue depth, tokens/sec, p95 latency, misses.

## Phased Implementation Plan

Phase 0 — Feature Gate & Skeleton (low risk)
- Add `feature = "llm"` to kernel crate.
- New module `crates/kernel/src/llm.rs` with operator trait, buffer schemas, and metrics stubs.
- Extend `cap.rs` with `CapObjectKind::LLM` and rights.
- Shell stubs: `llmctl`, `llminfer`, `llmstats` (no real backend yet).

Phase 1 — Functional Stub Operator
- Implement a deterministic summarizer stub (byte‑heuristic) to exercise the full path: control‑plane → graph → scheduler → metrics.
- Add a tiny “LLM graph” constructor in `graph.rs` to wire channels and operator.
- Add tests in `sis-testing` to submit a prompt and validate metrics + schema.

Phase 2 — Scheduler Integration & Quotas
- Introduce an `LLMInference` job type (or reuse `AiTaskSpec`), create CBS server(s) and admission logic.
- Implement budgets and per‑tenant quotas with rejection metrics.
- Integrate deadline/jitter counters with existing `det_*` metrics.

Phase 3 — Streaming & Tokenization
- Support streaming token chunks (fixed size) over SPSC; control‑plane polling for partial reads.
- Minimal tokenizer (or raw‑byte streaming initially); ensure bounded work per chunk.

Phase 4 — Model Packaging & Security
- Extend `model.rs` to recognize LLM packages (metadata: ctx len, vocab size, quant scheme, wcet_cycles).
- Signed load path with audit; reserve arenas per model; refuse oversize models.

Phase 5 — Backends & Coprocessor Path
- Kernel micro‑LLM backend for privileged summaries.
- Coprocessor/virtio path for larger models; unify metrics and admission across backends.

## File & Module Touchpoints

- `crates/kernel/src/llm.rs` (new): operator, schemas, metrics, backend stub.
- `crates/kernel/src/graph.rs`: add `op_llm_run`, text/token schemas, helper to construct a small LLM graph.
- `crates/kernel/src/deterministic.rs`: add LLM job/server helpers; admission integration.
- `crates/kernel/src/control.rs`: add LLM frames; direct helpers similar to existing graph ops.
- `crates/kernel/src/shell.rs`: `llmctl`, `llminfer`, `llmstats` commands.
- `crates/kernel/src/cap.rs`, `crates/kernel/src/model.rs`: new capability kind and model package metadata.
- `docs/schemas/sis-metrics-v1.schema.json`: add LLM fields.
- `crates/testing`: tests to submit a prompt and validate artifacts; dashboard updates.

## Testing & Validation

- Quick mode (`--quick`): run stub operator, submit prompt via control‑plane, validate `llm_*` metrics and schema.
- QEMU mode: run with `SIS_FEATURES="llm,deterministic"`, verify deadlines, queue behavior, and rejection paths.
- Negative tests: oversize prompts, missing caps, budget exhaustion.

## Security Considerations

- Enforce capability checks on all LLM commands; default deny.
- Audit prompt sizes and token counts, not prompt contents.
- Bound all work per token/chunk; never unbounded loops in deterministic sections.
- Quotas per caller to prevent starvation/denial.

## Open Questions / Risks

- Tokenization in no_std: start with byte streaming or a minimal tokenizer; revisit once interfaces stabilize.
- Memory pressure: ensure arenas and per‑model reservations don’t threaten kernel stability.
- QEMU PMU limitations: guard non‑cycle counters; keep metrics meaningful.
- Backcompat: ensure `llm` is feature‑gated and off by default to preserve current demos.

## Example Flow (Initial Stub)

1) Build with features: `SIS_FEATURES="llm,deterministic" BRINGUP=1 ./scripts/uefi_run.sh`
2) Shell: `llmctl load --model 1 --wcet-cycles 25000`
3) Shell: `llminfer "why was op B slower than op A?" --max-tokens 64`
4) Observe:
   - METRICs: `llm_infer_us`, `llm_tokens_out`, `llm_queue_depth_max`, `llm_deadline_miss_count`.
   - Dashboard card shows p95 latency and tokens/sec.

---

Authoring Notes:
- Keep kernel changes minimal and feature‑gated.
- Validate before expanding scope or adding heavy dependencies.
- Prefer direct shell/control‑plane paths until VirtIO becomes robust.

