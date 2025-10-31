# Real Hardware Bring‑Up Advisory (SIS Kernel)

Purpose
- Provide a practical guide to port SIS from QEMU/UEFI to real ARM64 hardware without bloat and without losing implemented features.
- Outline refactors and guardrails to keep the kernel small, safe, and maintainable while enabling platform diversity.

Scope
- AArch64 EL1 targets with GICv3 and PL011‑like UARTs (or DT‑described equivalents).
- Assumes no MMU/exceptions virtualization (EL2/EL3) or trusts firmware to hand off to EL1.
- UEFI preferred when available; otherwise, a simple board loader is acceptable.

High‑Level Strategy
- Keep the current AI/autonomy features; isolate platform specifics behind a thin platform layer.
- Ship a “hardware‑minimal” feature preset to reduce surface during early board bring‑up.
- Retain shell‑first workflows and auditability; ensure timers/IRQs work early.

Feature Gating Strategy
- Default build for boards: minimal platform + shell + autonomy gate; optional AI/LLM/drivers behind features.
- Kernel‑worthy mechanisms remain in kernel; analytics/policy/visualization stay at the shell boundary.

Recommended Refactors
- Platform Abstraction (new `platform` module)
  - Create `platform/` or `arch/aarch64/platform/` with a trait exposing addresses and init hooks:
    - `fn plat_uart_init(&self) -> Result<(), Err>`
    - `fn plat_gic_init(&self) -> Result<(), Err>`
    - `fn plat_timer_init(&self) -> Result<(), Err>`
    - `fn plat_mmio_ranges(&self) -> &'static [MmioRange]`
    - `fn plat_ram_ranges(&self) -> &'static [RamRange]`
    - Optional: `fn plat_psci_available(&self) -> bool`
  - Provide `qemu_virt` implementation (current behavior) and a skeleton `board_x` implementation.
  - Source configuration from DT when available; otherwise compile‑time board constants.

- Device Tree (DT) Intake
  - Add a tiny DT parsing path (`fdt`/`libfdt`‑like or minimal custom) to extract:
    - UART base + clock; GICd/GICr bases; RAM regions; timer frequency if provided
  - Pass the UEFI DTB pointer to kernel or embed DTB for board variants.
  - Fail gracefully by falling back to compiled constants if DT is absent.

- GICv3 and Timer Initialization
  - Parameterize GIC distributor/redistributor base addresses; remove QEMU‑hardcoded values.
  - Keep PPI 27 virtual timer path; ensure `cntfrq_el0` is read and used (already done) but validate against board docs.
  - Maintain full context save/restore in ISR; avoid printing in ISRs during early bring‑up (use metrics rings).

- UART Driver Robustness
  - Derive baud divisors from DT clock or expose a board hook to set prescalers.
  - Keep `uart_print` as the lowest‑risk path; avoid deep formatting from ISRs.

- MMU and Memory Map
  - Replace fixed 1 GiB identity map with a builder that consumes RAM + MMIO ranges from the platform layer.
  - Continue with 4 KiB pages; retain device attributes for MMIO.
  - Ensure barriers (`dsb/isb`) around table activation; keep MAIR/TCR config unchanged unless DT dictates.

- Exception Levels & Firmware Hand‑off
  - Detect EL2/EL3 and consider adding a safe EL drop if firmware does not; otherwise document EL requirements.
  - PSCI: add stubs for `cpu_on/cpu_off/system_off` when SMP or power states are needed (future).

- Build Profiles & Presets
  - Add a `hw-minimal` meta feature enabling only: bringup, shell, autonomy gate, core neural (no LLM), no virtio.
  - Provide `board=<name>` build env to pick a platform implementation; default to `qemu_virt`.

- Logging & Metrics Policy
  - Keep UART noise minimal during hardware bring‑up; prefer one‑line banners.
  - Route health and metrics to rings; `metricsctl on` can enable printing once console is stable.

Kernel‑Worthy Features Guidance
- Keep in kernel (mechanism only):
  - Anchored config + quarantine (signed/staged), audit chains, identity beacon, arbitration weights and rationale codes, minimal health summaries, OOD class.
- Push to shell/offboard if it grows: Top‑K rankings beyond on‑demand queries, dashboards, heavy stats or training.

Minimal Bring‑Up Checklist (Per Board)
1) UART online
  - Verify a single print via `uart_print` volatile write.
  - Lock baud via DT clock or known divisor.
2) Generic timer online
  - Read `cntfrq_el0`, set 1s tick, confirm tick increments via a counter.
3) GICv3 online
  - Program distributor + redistributor from platform base; enable PPI 27; confirm timer IRQs.
4) Exception vectors and ISR
  - Use existing EL1 vectors; confirm no re‑entrancy; avoid prints in ISR; record basic metrics.
5) Shell prompt
  - Enter full shell on dedicated stack; type `help`.
6) Heap self‑test
  - Run existing heap tests and stress small allocations through shell.
7) Autonomy gate
  - `autoctl status/on/off`; ensure periodic decision tick does not hang or flood.
8) Metrics
  - Enable controlled metrics printing; confirm latency summary and memory alloc snapshots work.

Enabling Advanced Features (After Base Bring‑Up)
- Enable `audit-chain` and verify `nnjson/llmjson chain_ok`.
- Enable `config-quarantine` and test `ctlconfig propose/commit/status` (with signatures under `crypto-real`).
- Enable `meta-weights` and check deterministic changes from weights.
- Enable `health-monitor` at low cadence; confirm ISR budget remains tiny.
- Enable LLM (`llm`) last; ensure memory footprint and deadlines are sane.

Identity & Integrity
- Identity Beacon (`id-beacon` + `crypto-real`): sign `boot_time||build_id` at boot and expose via shell.
- Audit Chains (`audit-chain`): add `prev_hash`/`entry_hash` to audit entries; verify on JSON dump.
- Signed Config (`config-quarantine` + `crypto-real`): require valid signatures for commits in production images.

Performance & ISR Budgets
- ISR: avoid heap, avoid long prints; use metrics rings and single‑byte breadcrumbs only when diagnosing.
- Crypto: keep signatures and hashes on control paths or at boot, never in ISRs.
- Rings: fixed size only; saturate instead of grow.

Safety & Concurrency
- Prefer `spin::Mutex` for short critical sections; avoid taking locks in ISRs unless absolutely bounded.
- Keep IRQ masking windows minimal and audited; you already reduced them—maintain this discipline.
- Rate‑limit autonomous actions with counters and publish clear audit entries.

Testing & Debugging on Hardware
- If JTAG/SWD is available: add small debug stubs (optional feature) to park core on panic.
- Use LED/GPIO (if present) for early boot breadcrumbs when UART is not yet stable.
- Provide a `panic` path that disables IRQs, prints a compact code, and halts.

Portability Pitfalls to Avoid
- Hardcoded MMIO bases (replace via platform layer/DT).
- Assuming UEFI presence on all boards; support a fallback loader if needed.
- Assuming PL011 clock; derive divisors via platform hooks.
- Printing excessively in timers/IRQs; keep it to rings and summary.

Build & Delivery Suggestions
- Create a `scripts/hw_build.sh` that sets `--features hw-minimal` and `BOARD=<name>`.
- Ship board presets under `platform/boards/` with documented addresses and DT bindings.
- Document runtime toggles in README (link to Architecture doc and this advisory).

Refactor Roadmap (Incremental)
- Phase 1: Extract a platform trait; migrate QEMU code to `qemu_virt` module behind trait.
- Phase 2: Introduce DT intake (optional first) and board presets; parametrize GIC/Timer/UART init.
- Phase 3: Introduce `hw-minimal` feature and ensure a clean boot to shell + heap tests on at least one board.
- Phase 4: Re‑enable `audit-chain`, `config-quarantine`, `meta-weights`, `health-monitor`; validate ISR budgets.
- Phase 5: Evaluate LLM (`llm`) footprint and deadlines; only then enable on hardware.

Definition of “No Bloat”
- Mechanism only in kernel; policy, analysis, and heavy printing stay at shell/offboard.
- Bounded memory and CPU: no unbounded growth, no background scans in kernel.
- Feature‑gated modules; default‑off for production images until verified.

Go/No‑Go Criteria for a New Board
- UART reliable + Timer tick stable + GIC PPI 27 interrupts verified
- Shell usable + heap tests pass + autonomy gate toggles
- No ISR stalls; metrics rings show sane latencies
- Platform descriptor finalized (bases/clock/DT) and checked in under `platform/boards/<name>.rs`

Appendix: Suggested Module Moves
- Move hardcoded bases from `main.rs` into `platform/qemu_virt.rs`.
- Add `platform/mod.rs` trait and re‑export the active platform implementation via a `BOARD` cfg.
- Gate `virtio_console` and LLM features in `hw-minimal` builds by default.

# End

---

# HW‑First Refactoring Plan (Do This While Still in QEMU)

Goal
- Restructure the codebase so it targets real hardware as the default design, while remaining fully runnable in QEMU.
- After this refactor, real hardware bring‑up should require only platform‑specific glue (e.g., board DT/power/PSCI), not core code changes.

Deliverables (Summary)
- Platform layer abstraction with a `qemu_virt` implementation and board skeletons
- Device‑tree (DT) intake path (optional fallback to board constants)
- Parameterized GIC/Timer/UART/MMU map builder using platform data
- `hw-minimal` feature preset for lean board images
- Static “no hardcoded MMIO” standard and CI checks (grep‑based advisory initially)

Phases & Tasks

Phase 0: Ground Rules & CI Guards
- Add coding standards (documented in README, see HW‑First Standards) prohibiting direct MMIO constants in modules outside the platform layer.
- Add a basic CI grep/advisory script to flag common QEMU constants (e.g., `0x0900_0000`, `0x0800_0000`) in non‑platform code.
- Outcome: Guardrails in place; team aligned on constraints.

Phase 1: Platform Layer Extraction
- Add `crates/kernel/src/platform/mod.rs` trait:
  - `uart() -> UartDesc { base, clock_hz }`
  - `gic() -> GicDesc { gicd, gicr }`
  - `timer() -> TimerDesc { freq_hz }`
  - `mmio_ranges() -> &'static [MmioRange]`
  - `ram_ranges() -> &'static [RamRange]`
  - Optional: `psci_available() -> bool`
- Implement `platform/qemu_virt.rs` returning current QEMU virt addresses.
- Replace hardcoded bases in bring‑up (GIC/UART/MMU) with calls into the platform layer.
- Outcome: QEMU boots identically via the platform API.

Phase 2: MMU Map Builder
- Introduce a small builder that consumes `ram_ranges()` + `mmio_ranges()` and programs L1 entries.
- Keep MAIR/TCR/4KiB pages unchanged; ensure device attributes for MMIO.
- Outcome: MMU config is data‑driven from platform descriptors.

Phase 3: DT Intake (Optional but Preferred)
- Add a minimal DT parser (tiny FDT walker) that can extract UART clock/base, GICd/GICr, RAM ranges, and optionally timer frequency.
- Allow bootloader to pass DTB pointer; if present, the platform implementation populates descriptors from DT; otherwise fall back to constants.
- Outcome: Same kernel image adapts to boards that ship DTs, still boots in QEMU.

Phase 4: Timer & GIC Parameterization
- Refactor GIC distributor/redistributor init to use `platform.gic()` addresses and wake sequences.
- Validate PPI 27 programming and virtual timer setup using `timer().freq_hz` or `cntfrq_el0` fallback.
- Outcome: No QEMU‑specific constants in interrupt/timer code.

Phase 5: UART Parameterization
- Use `uart().clock_hz` to compute IBRD/FBRD; keep PL011 path, but generalize for DT‑described variants.
- Outcome: UART init works across boards with different clocks.

Phase 6: Shell Thinning & Module Boundaries (Structure, Not Behavior)
- Split `shell.rs` into domain adapters (autoctl/neuralctl/memctl/graphctl/llmctl/metricsctl) that only parse and call module APIs.
- Move demo‑heavy commands to `shell/demos/` behind a `demos` feature.
- Ensure all work is done in module APIs; shell prints minimal output.
- Outcome: Smaller blast radius for platform changes; policy/analytics at the edge.

Phase 7: Feature Presets
- Add `hw-minimal` feature preset enabling: `bringup, meta-weights, audit-chain, config-quarantine` and disabling `llm, virtio-console, perf-verbose` by default.
- Provide `BOARD=<name>` env to select platform implementation (default `qemu_virt`).
- Outcome: One command builds lean images for hardware and dev images for QEMU.

Phase 8: Acceptance & Regression Tests (in QEMU)
- UART prints minimal banners; full shell works on dedicated stack.
- Timer tick + GIC PPI 27 verified; context switch/heap tests pass.
- No direct MMIO constants outside platform layer (grep check passes).
- Enabling `audit-chain`, `config-quarantine`, `meta-weights`, `health-monitor` does not change bring‑up stability.

What Remains for Real Hardware (Non‑QEMU or Board‑Specific)
- PSCI services (cpu_on/off/system_off) where needed for SMP/power
- Board power/reset/clock controller nuances not modeled in QEMU
- Secure monitor/EL transitions for firmware that does not hand off to EL1
- Board DTB packaging/distribution

QEMU‑Testable Proxies
- DT intake: embed a small DTB for QEMU runs or pass a DTB via UEFI to exercise the DT path.
- Multiple platform descriptors: provide `qemu_virt_alt` with different (fake) bases to prove no hardcoding remains.
- ISR budget: run latency bench with `health-monitor` on to ensure bounded ISR work.

Exit Criteria (HW‑Ready in QEMU)
- No QEMU hardcodes outside platform layer.
- Boots with platform‑derived MMU/GIC/Timer/UART; DT intake path validated.
- `hw-minimal` image boots to shell and passes heap/context tests.
- Advanced features re‑enabled incrementally without destabilizing bring‑up.
