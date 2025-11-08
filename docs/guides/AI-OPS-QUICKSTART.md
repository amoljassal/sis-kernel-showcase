# Phase 7 AI‑Ops Quickstart

This guide shows a fast end‑to‑end run of the Phase 7 features: model lifecycle, trace/incident export, shadow/canary, and simple persistence demos.

## Build and Boot

- Minimal build (QEMU AArch64 UEFI):

```
SIS_FEATURES="bringup,ai-ops" ./scripts/uefi_run.sh build
```

- With embedded models initramfs (recommended for modelctl demos):

```
# 1) Pack models as newc CPIO
mkdir -p /tmp/models_root/models/v1
dd if=/dev/zero of=/tmp/models_root/models/v1/model.bin bs=1k count=4
cd /tmp/models_root && find . -print | cpio -o -H newc > /tmp/models.cpio

# 2) Build with embedded archive
INITRAMFS_MODELS=/tmp/models.cpio SIS_FEATURES="bringup,ai-ops,initramfs-models" ./scripts/uefi_run.sh build
```

Tip: `INITRAMFS_MODELS` is detected by `build.rs` and embedded only if the file exists.

## Shell Warm‑Up

- Quiet logs for clean output:

```
autoctl off
```

- Verify models from initramfs:

```
ls /models
ls /models/v1
```

## Model Lifecycle (modelctl)

- Dry‑swap (load + health, no state change):

```
modelctl dry-swap v1
```

- Swap and view history (JSONL):

```
modelctl swap v1
modelctl history 10
```

- Inspect history file:

```
cat /models/registry.log
```

## Decision Traces & Incidents (tracectl)

- Generate demo traces and list them:

```
tracectl demo 25
tracectl list 10
tracectl show 1000
```

- Export a bundle containing specific traces:

```
tracectl export 1000 1001 1002
ls /incidents
cat /incidents/INC-<id>.json
```

- Export recent shadow divergences (see Shadow section):

```
tracectl export-divergences 10
```

## Shadow / Canary (shadowctl)

- Enable shadow with a model and switch modes:

```
shadowctl enable v1
shadowctl status
shadowctl mode compare
```

- Optional canary alias:

```
shadowctl canary 10     # alias for mode canary10
```

- Dry‑run mode (log divergences only, no counters/rollback):

```
shadowctl dry-run on|off|status
```

## Persistence and SRE Entry Points

- Incident bundles are written atomically under `/incidents/INC-*.json`.
- History entries are appended as JSONL under `/models/registry.log`.
- Use `ls` and `cat` to inspect both in the shell; collect on the host by mounting the image read‑only or via a QMP signal.

## Ext4 Journaling (Durability Harness)

For an ext4 crash/replay test, use:

```
./scripts/ext4_durability_tests.sh /tmp/ext4-test.img
```

This script creates an ext4 image, boots the kernel with a second virtio‑blk device, runs the in‑kernel ext4/JBD2 test, reboots to replay the journal, and (optionally) runs host `fsck.ext4`.
