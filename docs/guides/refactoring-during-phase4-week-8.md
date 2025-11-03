# Refactoring Plan — Phase 4 / Week 8 (HW‑First, QEMU‑Validated)

Purpose
- Restructure the SIS kernel so development in QEMU produces code that is ready for real ARM64 hardware without further core refactors.
- Keep the kernel’s unique identity (ML gating, signed/staged config, audit chains, timer/IRQ autonomy, rationales) intact while improving portability and maintainability.

Outcomes
- Platform layer abstraction for UART/GIC/Timer/MMU maps, with a default `qemu_virt` implementation.
- Bring‑up and MMU configured from platform/DT descriptors (no hardcoded QEMU bases outside platform code).
- Shell modularized into thin command adapters; demos gated.
- Feature presets for hardware‑minimal builds; QEMU path remains green throughout.

References
- Architecture: `docs/ARCHITECTURE.md`
- Real hardware bring‑up advisory: `docs/real-hardware-bringup-advisory.md`
- Phase 4 add‑ons plan: `docs/phase-4-add-ons-from-modular-OS-project.md`
- HW‑First standards (must‑follow): see README “HW‑First Standards”

What Does Not Change
- Kernel mechanisms remain: ML/OOD/drift classifiers, autonomy gate + timers/IRQs, audit chains, signed/staged config, rationale codes, metrics rings.
- QEMU behavior remains functional; `qemu_virt` platform supplies the same addresses/clock values as today.

Scope
- Whole codebase structure: bring‑up, platform/DT, MMU map, UART/GIC/Timer init, shell structure, feature presets, and basic CI guardrails.

---

## HW‑First Standards (Summary)
- No hardcoded MMIO or device bases outside `platform/*`; device params (clock/base) via platform or DT.
- Mechanism in kernel; policy/analytics/formatting in shell.
- Bounded ISR (no heap/long prints); fixed rings; minimal masking.
- Feature‑gated subsystems; `hw-minimal` preset for hardware builds.
- Communicate via narrow APIs/`agent_bus`; avoid hidden globals.

---

## Phased Plan

Phase 0 — Guardrails & Standards (0.5 day)
- Add HW‑First standards to README (done) and team agreement.
- Add CI/advisory grep to flag direct usage of `0x0900_0000`, `0x0800_0000`, or `GICD/GICR` outside `platform/*`.
- Deliverable: Standards doc + advisory CI script.

Phase 1 — Platform Layer Extraction (1–2 days)
- Add `crates/kernel/src/platform/mod.rs` trait with descriptors:
  - `UartDesc { base, clock_hz }`, `GicDesc { gicd, gicr }`, `TimerDesc { freq_hz }`
  - `mmio_ranges() -> &'static [MmioRange]`, `ram_ranges() -> &'static [RamRange]`, optional `psci_available()`
- Implement `platform/qemu_virt.rs` populating current QEMU values.
- Replace bring‑up call sites in `main.rs`/init with platform getters (functional no‑op under QEMU).
- Acceptance: Builds and boots identical in QEMU; no constants remain in bring‑up code.

Phase 2 — MMU Map Builder (1 day)
- Introduce a small builder that programs L1 entries from `ram_ranges()` + `mmio_ranges()`.
- Keep MAIR/TCR config; device attributes for MMIO; remove fixed 1GiB map from non‑platform code.
- Acceptance: QEMU boots; heap tests pass; UART still works.

Phase 3 — DT Intake (1–2 days; optional initially)
- Add a tiny FDT walker to extract UART base/clock, GICd/GICr, RAM ranges, timer freq.
- Allow UEFI to pass a DTB pointer; platform impl uses DT if present; otherwise fall back to constants.
- Acceptance: DT path can be exercised in QEMU (UEFI DTB); fallback path still works.

Phase 4 — Timer & GIC Parameterization (1 day)
- Ensure timer freq from `timer().freq_hz` or `cntfrq_el0` fallback; remove remaining constants.
- Parameterize GIC init from `gic()`; validate PPI 27.
- Acceptance: Timer IRQ latency bench runs; periodic autonomous ticks work.

Phase 5 — UART Parameterization (0.5 day)
- Compute IBRD/FBRD from `uart().clock_hz`; retain PL011 behavior.
- Acceptance: UART prints unchanged; shell prompt works.

Phase 6 — Shell Modularization (2–3 days)
- Split `shell.rs` into `shell/autoctl.rs`, `shell/neuralctl.rs`, `shell/memctl.rs`, `shell/graphctl.rs`, `shell/llmctl.rs`, `shell/metricsctl.rs`.
- Move demo commands (aidemo/graphdemo/imagedemo/detdemo/mladvdemo/actorcriticdemo) to `shell/demos/` behind a `demos` feature.
- Ensure handlers only parse args and call module APIs; reduce inline logic.
- Acceptance: All commands still function; build time + code navigation improve.

Phase 7 — Feature Presets & Build Script (0.5 day)
- Add `hw-minimal` preset enabling: `bringup,meta-weights,audit-chain,config-quarantine` and disabling `llm,virtio-console,perf-verbose`.
- Add `BOARD=<name>` env to select platform impl (default: `qemu_virt`); optional `scripts/hw_build.sh`.
- Acceptance: `hw-minimal` boots to shell in QEMU with timer/GIC/heap working.

Phase 8 — Acceptance/Regression (0.5–1 day)
- Confirm: no QEMU constants outside platform; platform path used for MMU/UART/GIC/Timer; DT path verified.
- Run: heap tests, IRQ latency bench, metrics off/on; enable `audit-chain`, `config-quarantine`, `meta-weights`, `health-monitor` sequentially.

---

## Module‑Level Changes (Overview)
- `main.rs` bring‑up: switch to platform getters; keep ISR and safety prints minimal.
- `uart.rs`: add `init_with(UartDesc)` path or internal getter from platform.
- GIC/timer init: read addresses/freq from `platform` instead of constants.
- MMU builder: new module; called from bring‑up using platform ranges.
- `shell.rs`: split into per‑domain files; demos gated; handlers call module APIs.
- `graph/control`: prefer framed API for create/add/start; keep direct helpers in tests/demos only.
- `llm.rs`: expose “LLM‑core” path for HW‑minimal (budget checks + tiny stepping) under feature composition.
- `Cargo.toml`: add `hw-minimal`, `demos` features; keep defaults conservative.

---

## CI & Guardrails
- Advisory grep to find MMIO constants outside `platform/*`.
- QEMU acceptance job: boot logs must include bring‑up banners, shell, heap tests pass.
- Optional: quick unit tests for MMU builder mapping and DT walker extraction on canned blobs.

---

## Risks & Mitigations
- Hidden constants: grep + staged replacements + code review.
- DT complexity: keep walker minimal; always provide fallback constants.
- ISR regression: rerun IRQ latency bench; keep ISR code mechanical; avoid prints.
- Shell split churn: structural only; preserve command behavior and outputs.

---

## Rollback Strategy
- Each phase lands behind existing behavior (`qemu_virt` defaults); easy to revert a phase if instability appears.
- DT intake is optional; can disable while keeping platform layer.

---

## Timeline (Estimate)
- Total 7–10 developer‑days, delivered in small PRs that keep the build green.

---

## Exit Criteria (HW‑Ready in QEMU)
- No hardcoded device constants outside platform code.
- Bring‑up uses platform/DT for UART/GIC/Timer/MMU; DT path validated.
- `hw-minimal` boots in QEMU with shell + heap/context tests.
- Advanced features re‑enabled sequentially without destabilizing bring‑up.

