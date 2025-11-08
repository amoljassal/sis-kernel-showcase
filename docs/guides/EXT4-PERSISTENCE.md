# Ext Image Persistence & Journaling

This guide explains two ways to exercise persistence of Phase 7 artifacts (`/models`, `/incidents`) using an attached block image.

## A) Read‑Only Mount Overlay at `/models`

Attach an image as a second VirtIO device and let the kernel attempt a read‑only mount of the first block device at `/models` (ext2 driver):

```
EXT4_IMG=/path/to/image.img SIS_FEATURES="bringup,ai-ops" ./scripts/uefi_run.sh
```

In the shell:

```
autoctl off
ls /models
```

Notes:
- The current VFS mounts ext2 read‑only. Use this to validate mountpoint overlay and readback.
- Write‑side ext4 mounting via VFS is planned; use the durability harness below to exercise journaling today.

## B) ext4/JBD2 Durability Harness (Crash/Replay)

Run the full ext4 journaling cycle (create image → write → crash → replay → fsck):

```
./scripts/ext4_durability_tests.sh /tmp/ext4-test.img
```

The script:
- Creates an ext4 image and attaches it as a second VirtIO device
- Boots the kernel and runs the in‑kernel ext4 test (journaled writes)
- Reboots to replay the journal (verifies JBD2 recovery)
- Optionally runs host `fsck.ext4` to validate the image

Artifacts:
- Logs stored under `/tmp/sis-ext4-run*.log`
- Optional `fsck.ext4` report under `/tmp/sis-ext4-fsck.log`

## Roadmap

- VFS ext4 shim: mount `/models` on ext4 with write support using the in‑tree ext4/JBD2 code
- Switch journaled registry/incident paths to the ext4 mount for full persistence across reboots
