# SIS Kernel — Developer Handoff

This document captures the current implementation state and the exact next steps so the work can resume seamlessly in a new context.

## TL;DR

- Shell-first LLM service is complete and working: `llmctl load/budget/status/audit`, `llminfer`, `llmstream`, `llmgraph`, `llmjson`.
- Deterministic integration (CBS/EDF) is wired: budgeting drives scheduler metrics; `llmctl status` summarizes admission/jitter/misses.
- Host control (VirtIO console) is functional but flaky on macOS — sometimes host frames are “sent” but not ACKed or audited. Shell path is reliable.
- Audit is implemented (ring buffer): `llmctl audit` (text) and `llmjson` (JSON). Control-plane LLM frames (load/infer) are audited on allow/deny.
- Tokens: legacy (ctladmin/ctlsubmit) and embedded-rights tokens (upper 8 bits = rights, lower 56 bits secret) are supported.
- Recommended next item: Harden virtconsole multiport to make ACKs/audit reliable on macOS.

## What Works (Implemented)

Feature flags and commands:

- LLM kernel service (`--features llm`)
  - `llmctl load [--wcet-cycles N]`
  - `llmctl budget [--wcet-cycles N] [--period-ns N] [--max-tokens-per-period N]`
  - `llmctl status`
  - `llmctl audit` — prints recent LLM operations with status flags (op/status)
  - `llmjson` — prints audit log as JSON (for post-process)
  - `llminfer "<prompt>" [--max-tokens N]`
  - `llmstream "<prompt>" [--max-tokens N] [--chunk N]` — chunked streaming; emits `llm_stream_chunks`, `llm_stream_chunk_tokens`
  - `llmgraph "<prompt>"` — graph-backed tokenization; emits chunk tensors to an output channel; sink prints `[LLM][GRAPH-OUT] chunk: ...` and `llm_graph_chunk_drop` on enqueue drop

- Deterministic integration (`--features deterministic`)
  - `llmctl budget` converts WCET cycles via `cntfrq_el0` and configures a CBS server
  - Scheduler metrics update per inference: `deterministic_deadline_miss_count`, `deterministic_jitter_p99_ns`, `det_admission_*`

- Control-plane (host, experimental)
  - VirtIO console path with `sis_datactl.py` using `--retries N` (2s ACK timeout)
  - LLM frames: `0x10` LlmLoad (requires ADMIN), `0x11` LlmInferStart (requires SUBMIT or ADMIN)
  - Tokens:
    - Legacy: `ctladmin` / `ctlsubmit` in shell to rotate
    - Embedded rights token: `TOKEN = (RIGHTS << 56) | SECRET` where `RIGHTS` bit0=ADMIN, bit1=SUBMIT; `SECRET` = lower 56 bits of `ctlkey`

## Known Issue (Host ACK/Audit on macOS)

- Symptom: host tool prints “sent”, but no `ACK: b'OK\n'`; `llmjson` prints `[]` because frames didn’t arrive.
- Likely cause: QEMU virtconsole delivery is finicky on macOS; our driver’s RX path is minimal.
- Workaround: Use the shell path for reliable testing; or use TCP chardev fallback (script supports this) — but still may be inconsistent without multiport binding.

## Next Steps (High Priority)

1) Virtconsole multiport hardening (CTRL RX/TX + per-port binding)
   - Status: Implemented feature negotiation for MultiPort, initialized CTRL RX/TX queues, and bound to named port `sis.datactl`.
   - Goal now: Validate delivery on macOS, and if needed route additional data ports; confirm consistent ACKs/audit.
   - Code touched:
     - `crates/kernel/src/virtio_console.rs`
       - Negotiates `MultiPort`, initializes queues 2/3, binds `sis.datactl` on PortName, sends PortOpen, and processes data frames via the primary data queue.
     - `scripts/uefi_run.sh`
       - Binds the virtconsole with `name=sis.datactl` and supports UNIX/TCP chardev.
   - Test steps:
     1) `VIRTIO=1 SIS_FEATURES="llm,virtio-console" BRINGUP=1 ./scripts/uefi_run.sh`
     2) Wait for `VCON: READY` in serial (and `[VCON] BOUND port to sis.datactl` when control events flow)
     3) Host:
        - `./tools/sis_datactl.py --retries 4 --wait-ack llm-load --wcet-cycles 25000`
        - `./tools/sis_datactl.py --retries 4 --wait-ack llm-infer "why op b slow?" --max-tokens 4`
     4) Shell: `llmjson` — expect entries for load/infer

2) Optional: Shell helpers to print embedded-rights tokens
   - Status: Implemented `ctlembed admin|submit` to print an embedded-rights token using the current secret.
   - Test steps:
     1) In shell, `ctlkey` to view current secret
     2) `ctlembed admin` → copy token; host: `--token 0xTOKEN llm-load ...` should ACK
     3) `ctlembed submit` → copy token; host: `--token 0xTOKEN llm-infer ...` should ACK

## Current Test Recipes

Shell-first (reliable):

```bash
SIS_FEATURES="llm,deterministic" BRINGUP=1 ./scripts/uefi_run.sh

# In SIS shell
llmctl load --wcet-cycles 25000
llmctl budget --period-ns 1000000000 --max-tokens-per-period 8
llminfer "why was op B slower than op A?" --max-tokens 8
llmstream "why was op B slower than op A?" --max-tokens 8 --chunk 2
llmgraph "why was op B slower than op A?"
llmctl status
llmjson   # or: llmctl audit
```

Host control (experimental; use TCP fallback if UNIX socket refuses):

```bash
# UNIX socket path
VIRTIO=1 SIS_FEATURES="llm,virtio-console" BRINGUP=1 ./scripts/uefi_run.sh
# OR TCP fallback
DATACTL_TCP=1 DATACTL_PORT=7777 VIRTIO=1 SIS_FEATURES="llm,virtio-console" BRINGUP=1 ./scripts/uefi_run.sh

# Host (accept cases)
./tools/sis_datactl.py --retries 4 --wait-ack llm-load --wcet-cycles 25000
./tools/sis_datactl.py --retries 4 --wait-ack llm-infer "why op b slow?" --max-tokens 4

# Host (reject example)
./tools/sis_datactl.py --retries 2 --wait-ack --token 0x1 llm-infer "why op b slow?" --max-tokens 4

# In SIS shell
llmjson
```

## Audit Semantics

- Operation codes: `1=load`, `2=budget`, `3=infer`, `4=stream`
- Status bits: `0b001=ok`, `0b010=reject`, `0b100=deadline_miss`
- Fields: `prompt_len`, `tokens`, `wcet_cycles`, `period_ns`, `ts_ns`
- Privacy: prompt contents are not logged (only lengths/counters).

## Code Map (Relevant Files)

- `crates/kernel/src/llm.rs`
  - LLM service, audit ring, metrics, stream API (`infer_stream`), JSON audit (`audit_print_json`)
- `crates/kernel/src/shell.rs`
  - Shell commands: `llmctl`, `llminfer`, `llmstream`, `llmgraph`, `llmjson`, `ctlkey`, `ctladmin`, `ctlsubmit`
- `crates/kernel/src/control.rs`
  - Control-plane LLM frames; allow/deny with audit; token checks; embedded-rights support
- `crates/kernel/src/graph.rs`
  - `op_llm_run` (producer) and `op_llm_sink` (consumer) for graph-backed streaming
- `crates/kernel/src/virtio_console.rs`
  - VirtIO console driver; data RX, control RX/TX; prints `VCON: READY`
- `scripts/uefi_run.sh`
  - Feature toggles; VirtIO device wiring; UNIX/TCP chardev configuration
- `tools/sis_datactl.py`
  - `--retries` and 2s ACK timeout; UNIX/TCP modes

## Open Items (Nice to Have)

- CI/demo: a headless test that sends load/infer over host and verifies audit via `llmjson`.
- Improve README with a short embedded-rights token example once `ctlembed` is added.
- Optional: export audit to a fixed memory buffer or a host-readable file for post-processing.
