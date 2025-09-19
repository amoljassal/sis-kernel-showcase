# Production Modes

Recommended feature flag sets and run scripts for common deployment modes.

- Bring-up: minimal boot and shell only
  - Features: `bringup`
  - Run: `./scripts/uefi_run.sh`

- Graph-only: graph runtime and observability, no virtio
  - Features: `bringup,graph-autostats`
  - Shell: `graphctl create`, `graphctl add-channel 64`, `graphctl add-operator ...`, `graphctl start N`

- Deterministic + Perf: deterministic scheduler and perf metrics
  - Features: `bringup,deterministic,perf-verbose,strict`
  - Shell: `graphctl det <wcet_ns> <period_ns> <deadline_ns>`, then `graphctl start N`

- VirtIO control (host-driven):
  - Features: `bringup,virtio-console,strict`
  - Host: `tools/sis_datactl.py --wait-ack --token 0x53535F4354524C21 create|add-channel|add-operator|start|det`

Notes
- Enable `strict` to treat warnings and unsafe ops-in-unsafe as errors at build-time where possible.
- Control-plane frames require a 64-bit capability token as the first 8 bytes of payload.
- The SPSC channel capacity is currently fixed at 64 entries.

