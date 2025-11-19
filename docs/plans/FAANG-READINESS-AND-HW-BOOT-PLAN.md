# FAANG‑Readiness And Real Hardware Boot Plan

Status: Draft (v1)
Owner: Kernel Team
Targets: AArch64 (QEMU + Raspberry Pi 5), with x86_64/riscv64 follow‑on
Timeframe: 3–4 weeks to hardware‑ready profile and CI parity

## 0) Executive Summary

Objective: raise code quality, reliability, and operational maturity to a FAANG‑grade baseline and prepare for first real hardware boot (Raspberry Pi 5). Deliver a minimal, robust HW profile, strong observability, hardened drivers, and CI coverage — without regressing AArch64 QEMU workflows. x86_64/riscv64 continue as parallel, build‑only or smoke jobs, with hardware enablement to follow.

Definition of “FAANG‑readiness” for this phase:
- Deterministic, reproducible builds; strict lints where feasible.
- Clear unsafe boundaries; unified error model; strong logging/metrics.
- FDT‑driven platform bring‑up (no hardcoded MMIO), with early console and panic capture.
- Hardened storage (read‑only first), IRQ/timer correctness, DMA hygiene.
- CI matrix that boots to shell in QEMU and runs a scripted smoke.
- A “hardware‑minimal” feature preset and a documented preflight checklist.

Non‑goals (this phase):
- Full desktop/UI on hardware; high‑perf networking/storage drivers; secure boot chain end‑to‑end.

## 1) Target Environments

- QEMU AArch64 virt + EDK2 UEFI (current runner).
- Raspberry Pi 5 (BCM2712) via EDK2 RPi UEFI (real hardware).
- Optional build/smoke:
  - x86_64 + OVMF (UEFI) — compile and entry smoke (separate plan exists).
  - riscv64 + OpenSBI QEMU — compile/smoke (separate plan exists).

## 2) Workstreams (Actions + Acceptance)

### 2.1 Engineering Foundations & Code Quality
- Actions
  - Add a `strict` feature preset: `#![deny(warnings)]`, `clippy::all` (allow pedantic where too noisy), `deny(unsafe_op_in_unsafe_fn)` where possible.
  - Document unsafe guidelines; add `unsafe_justification` template; audit `drivers/*`, `platform/*` for volatile access and barrier correctness.
  - Unify error model: keep `drivers::error::DriverError`; provide a single mapping module to `lib::error::Errno` for syscall boundaries. Introduce `InitError` enums for major subsystems using `thiserror` to improve diagnostic quality (see code example below).
  - Feature hygiene: presets `hw-minimal`, `debug`, `qemu-smoke`, `full`; review and deprecate overlapping toggles.
- Acceptance
  - `cargo clippy` clean in CI for `qemu-smoke` and `hw-minimal`.
  - Audit file exists; top X unsafe sites carry brief rationale.
  - Errors surfaced consistently in shell/logs; no ad‑hoc errno variants.

Code example (InitError with thiserror):

```rust
#[derive(thiserror::Error, Debug)]
pub enum InitError {
    #[error("device not found: {0}")] NotFound(&'static str),
    #[error("timeout waiting for {0}")] Timeout(&'static str),
    #[error("invalid parameter: {0}")] Invalid(&'static str),
    #[error("mmio mapping failed: base={base:#x} size={size:#x}")]
    MmioMap { base: usize, size: usize },
}

impl From<InitError> for crate::lib::error::Errno {
    fn from(e: InitError) -> Self { match e {
        InitError::NotFound(_) => Self::ENOENT,
        InitError::Timeout(_)  => Self::ETIMEDOUT,
        InitError::Invalid(_)  => Self::EINVAL,
        InitError::MmioMap {..} => Self::EFAULT,
    }}
}
```

### 2.2 Platform & Boot (AArch64 → RPi5)
- Actions
  - Replace bespoke parts of `platform/dt.rs` with `fdt` crate usage where beneficial (cells, ranges, chosen/stdout-path, interrupt-parent).
  - Early console abstraction: stdout‑path → PL011/16550 → memory ring buffer → GOP framebuffer. Expose ring as `/proc/bootlog`.
  - UEFI → kernel handoff: conserve EFI memory map in UEFI app; mark reserved regions; avoid identity‑mapping firmware post‑ExitBootServices.
  - MMU attr helpers: `mmio_map_device(name, base, size, attrs)`; enforce Device‑nGnRE for MMIO; centralize MAIR/TCR.
  - PSCI abstraction and MADT/FDT driven SMP topology (enable later; see 2.6).
- Acceptance
  - Boot logs visible even if UART mis‑detected (ring fallback); `/proc/bootlog` shows early entries.
  - DT summary printed: UART/GIC/Timer/SDHCI/PCIe nodes with addresses.
  - AArch64 QEMU unchanged performance/behavior.

### 2.3 Drivers & DMA Hygiene
- Actions
  - Create `hal/mmio.rs`: typed register wrappers (read/modify/write), barriers (`dmb ish`, `dsb sy`), explicit endianness. Replace direct volatile in new code and progressively in SDHCI/GIC/PL011.
  - DMA buffer API: `dma_alloc(size, align, coherent)`, cache maintenance hooks, bounce buffers for 32‑bit devices; ownership model.
  - SDHCI hardening: read‑only PIO path (single/multi‑block), error decode (CRC/timeout), capped retries, timeouts; gate writes until durability proven.
  - PL011 backend: RX/TX with IRQ; configurable baud; console trait implementation.
- Acceptance
  - SDHCI self‑tests pass in QEMU (clean ENODEV on virt); unit tests cover command path state machine (via fake MMIO bus).
  - Console abstraction switchable between PL011 and 16550.

Prompt: Provide detailed error/DMA examples in driver docs (SDHCI init state machine, transfer timeouts, cache maintenance for non‑coherent buffers), and document required barriers for each MMIO access path.

### 2.4 Memory Protections & Scheduling
- Actions
  - Enforce RO/NX page protections (.text/.rodata vs .data/.bss) and confirm W^X (no writable+executable).
  - Introduce SMP‑safe `percpu` storage for counters/state; reduce global contention. Provide an API with explicit `get_local()/for_each_cpu()` and examples; test with `loom` (see 2.10).
  - Keep deterministic scheduler behind feature; log admission math; back‑pressure when timers unreliable.
- Acceptance
  - Kernel page tables validated in logs; panic if mapping violates policy.
  - Per‑CPU counters printed in `metricsctl dump`.

Per‑CPU API example:

```rust
pub struct PerCpu<T: Default + Send + 'static>([core::cell::UnsafeCell<T>; MAX_CPUS]);
unsafe impl<T: Default + Send> Sync for PerCpu<T> {}

impl<T: Default + Send> PerCpu<T> {
    pub const fn new() -> Self { Self([UnsafeCell::new(T::default()); MAX_CPUS]) }
    #[inline] pub fn get_local(&self) -> &mut T { unsafe { &mut *self.0[crate::cpu::id()].get() } }
    pub fn for_each<F: FnMut(usize, &T)>(&self, mut f: F) { for i in 0..crate::cpu::count() { unsafe { f(i, &*self.0[i].get()) } } }
}
```

### 2.5 Observability & Diagnostics
- Actions
  - Unify logging macros; add runtime level + per‑module filters.
  - Panic dump: registers, best‑effort stack backtrace, memory map, last N log entries; persist `/var/log/panic-<ts>.json`.
  - Metrics consolidation: stable key set; `metricsctl dump` JSON.
  - OTel JSON sink rotation by size/time; toggle via shell.
- Acceptance
  - Panic dump verified by injecting a test panic; files readable in QEMU.
  - `metricsctl dump` returns JSON consumed by daemon/GUI on QEMU.

### 2.6 Testing & CI/CD
- Actions
  - CI matrix: aarch64 (build + QEMU run‑to‑shell), x86_64 (build + entry smoke), riscv64 (build‑only initially).
  - QEMU smoke script: runs `help`, `metricsctl off`, `validate quick`, `selftest all`, `agentsys` artifact checks; parse success tokens.
  - Fuzz: extend `fuzz_targets/vfs_path_lookup.rs`; add parsers (FDT, shell cmd).
  - Coverage for host‑side crates (`daemon`, tests) and quick coverage report.
-  - Verification CI: add optional jobs for Kani (bounded proof harnesses) and Prusti (contracts) on small critical modules (ring buffer, virtqueue, MMIO helpers). Gated, non‑blocking at first.
- Acceptance
  - CI green on matrix; smoke script artifacts (`/otel/spans.json`, `/var/log/rollback.json`, `/tmp/agentsys/*`) verified.
  - Verification CI runs on PRs for whitelisted modules and produces artifacts (reports) even if non‑blocking.

### 2.7 Concurrency Audit (New)
- Actions
  - Add a dedicated concurrency pass for week 2: use `loom` to test `percpu` API, ring buffers, lock‑free queues, and deferred work queues. Model key scheduling invariants under interleavings.
  - Extract minimal reproducer tests for suspected races and regressions; make them part of unit tests behind a `loom-tests` feature.
- Acceptance
  - Loom tests pass under randomized schedules (N seeds × M iters); any discovered race has a tracked issue and a patch.

### 2.8 Security & Supply Chain
- Actions
  - SBOM generation (CycloneDX) in CI; attach to releases.
  - Sign artifacts with `cosign` (detached) for release tags.
  - Boot integrity plan: UEFI app verifies kernel ELF hash (developer key); kernel verifies model bundles (Ed25519) — wire as optional checks.
- Acceptance
  - SBOM available in CI artifacts; signed release flow documented.

### 2.9 Documentation & Operational Readiness
- Actions
  - “Hardware Minimal Profile” doc: feature presets, expected device nodes.
  - RPi5 Bring‑Up Guide: power, UART wiring, known DT quirks, troubleshooting (no UART output, timer stalls, GIC PMR stuck).
  - Developer docs: driver style, FDT integration, MMIO/DMA guidelines.
- Acceptance
  - New docs under `docs/guides/` and README references updated.

### 2.10 Hardware Bring‑Up Preflight (RPi5)
- Actions
  - Feature preset `hw-minimal`: no GUI/LLM/demos/ext4 writes; metrics off by default; PL011 console; watchdog disabled initially.
  - Storage read‑only mount by default; shell opt‑in to enable writes.
  - Timer calibration: log CNTFRQ_EL0 and jitter stats; watchdog for anomalies.
  - GICv3 verification: runtime checks (PMR, IGRP enable, PPI priority) and a soft IRQ self‑test.
  - DT print summary at boot for quick field debug.
- Acceptance
  - Boot on hardware prints DT summary and enters shell; basic commands work; no ext4 writes until manually enabled.

### 2.11 QEMU Hardware Simulation (HW Realism)
- Actions
  - Add QEMU simulations to approximate hardware faults: use QMP to drop/restore IRQs, throttle virtio queues, and simulate timeouts. Where possible, use `-device` knobs (e.g., disable MSI, force legacy paths) to exercise drivers.
  - For Pi 5 preflight: simulate missing FDT properties, invalid regs/interrupts in a crafted DTB; run smoke to ensure graceful degradation.
- Acceptance
  - A scripted “QEMU HW Sim” run produces expected error logs without panics and validates fallback paths.

## 3) Timeline (Indicative)

Week 1 — Foundations (KPIs: 0 warnings in `qemu-smoke`, clippy clean; 85% line coverage in host crates)
- Strict preset + clippy gating, unsafe audit template.
- Error model mapping, feature preset consolidation.
- Early ring buffer + `/proc/bootlog`.
- CI: add clippy, fmt jobs; start matrix skeleton.

Week 2 — Drivers & MMU + Concurrency Audit (KPIs: all loom tests pass; percpu in use in 2 subsystems)
- `hal/mmio.rs`, barrier use in new code.
- PL011 backend; SDHCI read‑only hardening.
- MMU attribute helpers; centralized MAIR/TCR.

Week 3 — Observability & Tests (KPIs: panic dump validated; QEMU smoke <90s; verification CI green for whitelisted modules)
- Panic dump + metrics consolidation + OTel rotation.
- QEMU smoke script; basic coverage report.
- Fake MMIO bus for unit testing SDHCI state machine.

Week 4 — HW Profile & Docs (KPIs: hardware preflight doc complete; README updated; CI matrix stable >95% over week). Symbiosis milestone: open targeted GitHub issues seeking external expert review (drivers/MMIO, FDT, SMP), assign code owners, and tag help‑wanted.

Week 5 — Buffers & Concurrency Hardening (extension)
- Stabilize DMA/coherent buffer handling and barrier usage; widen loom coverage to virtqueue paths; iterate on any found issues.
- `hw-minimal` preset; RPi5 preflight and troubleshooting docs.
- README: updated quickstarts; add “Hardware Minimal” path.
- CI: aarch64 run‑to‑shell; x86_64 build+entry smoke; riscv64 build.

## 4) Deliverables

- Code
  - `hal/mmio.rs`, PL011 driver, SDHCI hardened (RO path), DMA helpers.
  - Early boot ring log; panic dump facility; metrics/OTel refinements.
  - Feature presets (`hw-minimal`, `qemu-smoke`, `strict`).
  - Concurrency: `loom` tests and fixes in percpu, ring buffers, deferred work queue.
- Tooling/CI
  - QEMU smoke script; SBOM; signed release pipeline.
  - CI matrix with clippy/fmt and run‑to‑shell; optional Kani/Prusti verification jobs.
- Documentation
  - Hardware Minimal profile; RPi5 Bring‑Up Guide; MMIO/DMA/FDT guidelines; updated README structure.

## 5) Risks & Mitigations

- DT variance across firmware builds → use robust `fdt` parsing; log raw properties on parse failure.
- GIC PMR/priority oddities in QEMU vs hardware → runtime sanity checks; soft‑IRQ self‑tests.
- Storage corruption risk → read‑only default; enable writes behind explicit shell command.
- Time constraints → prioritize early console, panic dump, SDHCI RO, and CI smoke; defer non‑critical features.

## 6) Success Metrics

- CI stability: >95% pass on aarch64 matrix jobs over a week.
- QEMU smoke duration: <90s; artifacts verified.
- Hardware: consistent boot‑to‑shell with DT summary and no panics; timer jitter within expected bounds for platform.
- Clippy/fmt clean on `qemu-smoke` and `hw-minimal` presets.

## 7) Follow‑On Work (Post‑Boot)

- Storage writes + journaling validation on hardware; ext4 fsck integration.
- Networking (USB XHCI, Ethernet) on RPi5; PCIe scanning.
- x86_64 and riscv64 hardware enablement to parity.
- Verified boot chain and secure console hand‑off.

## 8) References (Current Code)

- Platform & DT: `crates/kernel/src/platform/{mod.rs,dt.rs,rpi5.rs}`
- Drivers: `crates/kernel/src/drivers/{gpio/bcm2xxx.rs,block/sdhci.rs,firmware/mailbox.rs}`
- Shell & tests: `crates/kernel/src/shell/*`, `crates/kernel/src/drivers/selftest.rs`
- OTel/AgentSys/Shadow artifacts: `/otel/spans.json`, `/var/log/rollback.json`, `/tmp/agentsys/*`
