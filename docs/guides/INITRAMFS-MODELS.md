# Embedded Models via Initramfs

Embed a small `/models` tree in the kernel image for easy Phase 7 testing.

## Create the Archive (newc CPIO)

```
mkdir -p /tmp/models_root/models/v1
dd if=/dev/zero of=/tmp/models_root/models/v1/model.bin bs=1k count=4
cd /tmp/models_root && find . -print | cpio -o -H newc > /tmp/models.cpio
```

Notes:
- The kernel initramfs unpacker normalizes leading `./` in CPIO names; your files appear under `/models/...` at runtime.
- Keep the archive uncompressed (raw newc).

## Build with Embedded Archive

```
INITRAMFS_MODELS=/tmp/models.cpio SIS_FEATURES="bringup,ai-ops,initramfs-models" ./scripts/uefi_run.sh build
```

## Verify in the Shell

```
autoctl off
ls /models
ls /models/v1
modelctl dry-swap v1
```

Troubleshooting:
- If `/models` doesn’t show entries, recheck the archive with `cpio -tv < /tmp/models.cpio` and rebuild.
