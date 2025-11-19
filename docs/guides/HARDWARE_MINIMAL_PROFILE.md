# Hardware Minimal Profile

Status: Draft

The Hardware Minimal profile is a conservative feature preset and runtime configuration intended for first boots on physical hardware (e.g., Raspberry Pi 5). It prioritizes robustness, clear logs, and read-only storage.

## Goals
- Boot to shell with clear DT summary and early boot logs.
- Avoid heavy subsystems (GUI/LLM demos) and noisy metrics.
- Keep storage read-only until explicitly enabled.

## Build Profile
- Features: `bringup` (and optionally `strict` once stable)
- Avoid: `demos`, heavy GUI features, audio/camera.

Example:
```
BRINGUP=1 SIS_FEATURES="bringup" ./scripts/uefi_run.sh
```

## Runtime Settings
- Disable metrics streaming: `metricsctl off`
- Timer/autonomy optional: `autoctl off`

## Logs & Artifacts
- Early boot logs: `cat /proc/bootlog`
- Panic dumps (if any): `/var/log/panic-<ts>.json`
- OTel spans (QEMU): `/otel/spans.json`
- Shadow rollback: `/var/log/rollback.json`

## Storage
- Treat any block device read-only initially.
- Enable writes only when validated.

## Troubleshooting
- No UART output: check `/proc/bootlog` and DT UART node.
- Timer anomalies: log CNTFRQ_EL0 and jitter; disable autonomy.
- GIC: verify PMR and PPI priority; run IRQ self-test if available.

