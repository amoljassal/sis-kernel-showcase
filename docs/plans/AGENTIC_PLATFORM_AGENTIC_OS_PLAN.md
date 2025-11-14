# SIS Kernel – Agentic OS Technical Plan (LLM-as-Userspace)

Status: Draft (implementation-ready)
Owner: Core kernel + testing maintainers
Scope: QEMU target (aarch64-virt), bringup features enabled
Non-goal: Real hardware port in this phase (separate workstream)

## Objectives

- Turn SIS Kernel into an “agentic platform” where the default user experience is an LLM/agent shell and all capabilities are exposed via safe, policy‑governed kernel interfaces.
- Provide stable agent IPC (AgentSys) with fine‑grained policy control and auditable actions.
- Ship reference agents (music, files, docs) that demonstrate secure capability use.
- Keep changes incremental; preserve current QEMU bring‑up and test harness.

## Deliverables (High-level)

1) AgentSys (Agent System Call) layer in kernel with:
   - Message schema, dispatcher, capability checks, auditing hooks.
   - Asynchronous, multiplexed channels for agent ↔ system services.

2) Agent runtime (“agentd”) as default shell app:
   - LLM orchestrator + prompt router + child agent launcher.
   - Pluggable “tool” adapters mapped to AgentSys messages.

3) Reference agents (apps): music, files, docs (minimal but end‑to‑end).

4) Security/policy module for agents: capabilities, resource scopes, audit trail.

5) Test suite extensions and examples; CI validations for new features.

## Constraints / Current Context

- Target: QEMU aarch64-virt; 8 MiB heap; single‑core; UART shell.
- Existing components to reuse:
  - Control channel framing (`control.rs`) used by `graphctl` (token + framed commands).
  - Shell command framework and helper modules (`crates/kernel/src/shell/…`).
  - Agent bus (`crates/kernel/src/agent_bus.rs`), meta‑agent, and tracing/metrics.
  - Testing harness (crates/testing) and QEMU runtime.

## Architecture Overview

- AgentSys is a kernel control plane over the existing framed control path with a dedicated command range (0x20–0x2F). Agents send AgentSys frames; kernel dispatches to service handlers.
- Each AgentSys message is validated against a per‑agent CapabilityDescriptor (capabilities + resource scopes). On allow, kernel carries out the requested operation and emits audit records.
- The “agentd” runtime provides the LLM loop, maps LLM tools to AgentSys calls, and manages child agents in isolated sandboxes (in this phase: logical isolation + capability constraints; OS namespace/cgroup stubs prepared for later).

## Step-by-Step Implementation Plan

### Step 1 — Harden & Align Kernel APIs for Agentic Operation

Goals
- Provide a secure, async, multiplexed communication path for agents to request OS capabilities.
- Centralize capability checks and auditing.

Tasks
- Introduce AgentSys control space and dispatcher:
  - File: `crates/kernel/src/agent_sys/mod.rs`
  - Define `AgentSysCmd` enum (u8 opcodes) and `AgentSysRequest/Response` (compact headers + payload).
  - Reserve control opcodes 0x20–0x2F in `control.rs` for AgentSys.
  - Add `control::handle_agentsys_frame()` to decode, validate (`agent_policy`), and dispatch.

- Add Agent capability and policy module:
  - File: `crates/kernel/src/security/agent_policy.rs`
  - Structures: `AgentId`, `Capability` (enum), `Scope` (paths/patterns), `AgentToken` (runtime identity), `PolicyEngine`.
  - API: `check(agent_id, capability, scope) -> Result<(), PolicyError>`; `audit(agent_id, action, result)`.

- Multiplexed channels primitives (reuse/control):
  - Provide a thin async request table keyed by `(agent_id, req_id)` with completion callbacks; minimal for now: complete in dispatch thread, later extend with background ops.

Acceptance
- Unit build; shell shows new help for `agentsys` dev command (see Step 2 harness support).

### Step 2 — Design Agent Interface Layer (AgentSys-call)

Goals
- Define stable, minimal operations for core capabilities via messages.

Initial Capability Set and Messages
- Files (capability: `FsBasic`):
  - `FS_LIST { path } -> DirEntries`
  - `FS_READ { path, offset, len } -> Bytes`
  - `FS_WRITE { path, offset, bytes } -> Count`
  - `FS_STAT { path } -> Stat`
  - `FS_CREATE { path, kind } -> Ok`
  - `FS_DELETE { path } -> Ok`

- Music/Audio (capability: `AudioControl`):
  - `AUDIO_PLAY { track_ref } -> Ok`
  - `AUDIO_STOP -> Ok`
  - `AUDIO_VOLUME { level } -> Ok`

- Docs (capability: `DocBasic`):
  - `DOC_NEW { name } -> DocRef`
  - `DOC_EDIT { doc_ref, ops[] } -> Ok`
  - `DOC_SAVE { doc_ref } -> Ok`

- Vision/Audio IO (capabilities: `Capture`, `Screenshot`):
  - `SCREENSHOT -> ImageRef`
  - `AUDIO_RECORD { seconds } -> ClipRef`

Protocol
- Use the kernel’s existing control framing (token + payload).
- Payload is a compact, tagged byte format (QEMU‑friendly). For this phase use minimal structs + small TLV; JSON reserved for shell logging only.

Files
- `crates/kernel/src/agent_sys/protocol.rs` (opcodes, structs)
- `crates/kernel/src/agent_sys/handlers/{fs.rs,audio.rs,docs.rs,io.rs}` (handlers)

Acceptance
- `agentsys` shell helper: `agentsys status|capabilities|call <json>` for quick manual testing.

### Step 3 — Implement LLM/Agent Runtime in Userspace (agentd)

Goals
- Make LLM the default userspace “shell”: parse intent → map to AgentSys calls.

Tasks
- Create `agentd` application:
  - File: `crates/kernel/src/applications/agentd.rs`
  - Pieces: intent parser (uses existing LLM module), tool registry (maps intents to AgentSys calls), child agent manager (logical descriptors).
  - Provide `agentshell` command to enter conversational mode in UART.

- Tool registry initial tools:
  - `file.list`, `file.read`, `file.write`, `music.play`, `music.stop`, `doc.new`, `doc.edit`, `doc.save`, `screenshot.take`.

- Orchestrator:
  - Use existing `llm` module for inference and prompt templates.
  - Route structured actions into AgentSys (serialize request, send via `control::send_frame`).

Acceptance
- `sis>`: `agentshell` starts a REPL that can “list files in /”, “create doc”, “play demo track”.

### Step 4 — Reference Agents (PoC apps)

Goals
- Provide concrete mini‑agents that act as app demos.

Tasks
- File Manager Agent (`files_agent.rs`): list/read/write/stat simple files on tmpfs.
- Music Agent (`music_agent.rs`): no real audio device yet; simulate play/stop/volume and log state.
- Docs Agent (`docs_agent.rs`): in‑memory documents, save to tmpfs.

Acceptance
- All operations route through AgentSys; direct syscalls off‑limits to agents.
- Shell: `agentctl list` shows registered agents and their capabilities.

### Step 5 — Security/Isolation & Auditing

Goals
- Enforce least privilege; keep a full audit trail.

Tasks
- Capability engine gate for all AgentSys handlers (see Step 1).
- Per‑agent policy config (static for phase 1; dynamic later):
  - File: `crates/kernel/src/security/agent_policy.rs`: `PolicyEngine::allow(agent_id, cap, scope)`
- Audit sink:
  - File: `crates/kernel/src/security/agent_audit.rs`
  - Emit `AUDIT agent=<id> action=<op> scope=<...> result=<ok|err>` lines and METRIC counters (e.g., `metric_kv("agent_audit", 1)`).

Acceptance
- Denied operations return `EPERM` with audit line; allowed ops emit `AUDIT`.

### Step 6 — Ergonomics & UX Prototyping

Tasks
- `agentshell` becomes discoverable: `help` lists it; minimal help for prompts.
- Optional: simple GUI stub prints “Agent connected; use shell for natural language”.

Acceptance
- Demo flows executed via natural language mapped to tools + AgentSys.

### Step 7 — Wrapping Legacy/Complex Apps as Agents (Stubs)

Tasks
- Provide adapter trait `ExternalServiceAdapter` with `execute(request) -> response`.
- Implement a stub `external_docs_adapter` that echoes and logs; no real external procs in this phase.

### Step 8 — Iterate, Audit, and Dogfood

Tasks
- Run daily flows exclusively via agent shell (files/docs/music). Log ambiguity; tighten policies.

### Step 9 — Docs, Demos, Distribution

Tasks
- Update README (QEMU scope), add quickstart for `agentshell`.
- Record terminal demo scripts under `docs/demos/`.

### Step 10 — Scaling & Extensibility (Design Notes)

Tasks (design only for now)
- Define a plug‑in registry: `agentd` tool providers discovered via a registry.
- Prepare network/remote agent execution (future): reserve `AgentNet` capability.

## Detailed Work Items (Backlog)

1) Control Plane Reservation
- Update `crates/kernel/src/control.rs`: reserve opcodes 0x20–0x2F for AgentSys; forward frames to `agent_sys::handle_frame`.

2) AgentSys Core
- `crates/kernel/src/agent_sys/mod.rs` (new): dispatcher + request/response types.
- `crates/kernel/src/agent_sys/protocol.rs` (new): opcodes/structs.
- `crates/kernel/src/agent_sys/handlers/{fs.rs,audio.rs,docs.rs,io.rs}` (new): service handlers.

3) Security Modules
- `crates/kernel/src/security/agent_policy.rs` (new): capability checks + scopes.
- `crates/kernel/src/security/agent_audit.rs` (new): audit sink.

4) Agent Runtime (agentd)
- `crates/kernel/src/applications/agentd.rs` (new): orchestrator, tool registry, conversation loop.
- Shell helpers: `crates/kernel/src/shell/agentctl_helpers.rs` (extend): `agentctl list|status`.

5) Tests & Harness
- Add Phase 9 “Agentic Platform” tests:
  - Path: `crates/testing/src/phase9_agentic/` with modules: `agentsys_api.rs`, `agentd_runtime.rs`, `policy_enforcement.rs`.
  - Add to `crates/testing/src/lib.rs` suite init + results aggregation.
- Serial log metrics:
  - Ensure AgentSys emits METRIC counters (e.g., `agentsys_calls_total`, `agent_audit_events`).

6) Documentation
- This plan (docs/plans/AGENTIC_PLATFORM_AGENTIC_OS_PLAN.md)
- Developer HOWTO: `docs/guides/AGENTSYS-DEV-GUIDE.md` (API, examples, message layout, audit expectations)

## Message & Data Formats (Initial)

Header (8 bytes)
- `u8 opcode` (e.g., 0x20 FS_LIST, 0x21 FS_READ, …)
- `u8 flags` (async=1, stream=2, …)
- `u16 req_id`
- `u32 len` (payload length)

Payload (TLV)
- T=1 path (UTF‑8), T=2 offset (u64), T=3 len (u32), T=4 bytes ([]), …

Response
- Header mirrors request; same `req_id` and `opcode`, `flags` indicate error; payload holds result.

## Security Model (Initial)

- Capabilities (non‑exhaustive): `FsBasic`, `AudioControl`, `DocBasic`, `Capture`, `Screenshot`.
- Scope examples: `{ path_prefix: "/" }`, `{ path_prefix: "/tmp/docs/" }`.
- Policy: static table mapping `AgentId -> Capabilities + Scopes` for this phase.
- Audit: one line per operation + METRIC counters.

## Acceptance Tests (Minimal)

- Agentsys smoke:
  - `agentsys call {"op":"FS_LIST","path":"/"}` returns entries; audit line present; METRIC `agentsys_calls_total` increments.
- Agentd shell:
  - From `agentshell`: “Create a document called plan.md with ‘hello’” → DOC_NEW/DOC_EDIT/DOC_SAVE issued; files on tmpfs.
- Policy enforcement:
  - Agent without `FsBasic` denied on FS_READ; audit shows denied.

## Rollout & Compatibility

- QEMU only; no real devices required.
- No change to existing features unless `AgentSys` invoked.

## Risks & Mitigations

- Kernel memory pressure: keep message payloads small (4–8 KiB). Stream large content in chunks.
- Security gaps: start with deny‑by‑default; explicit allow in policy.
- Test flakiness: ensure METRIC and AUDIT lines include `[QEMU-OUT]` prefix stripping in harness parsers (already implemented).

## Milestones

M1 (AgentSys core + FS only)
- Dispatcher + FS handlers + `agentsys` shell test; policy allow for a test AgentId.

M2 (Agentd + tools)
- `agentd` shell with `file.*`, `doc.*`, `music.*` mapped tools; simple docs/music agents.

M3 (Security/Audit + Tests)
- Policy enforcement + audit + Phase 9 tests green under QEMU.

