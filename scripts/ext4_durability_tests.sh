#!/usr/bin/env bash
set -euo pipefail

# Ext4 durability test harness
#  - Creates an ext4 image
#  - Boots the SIS kernel with a second virtio-blk MMIO device pointing at the image
#  - Runs the in-kernel ext4 durability self-test (feature: ext4-durability-test)
#  - Kills QEMU to simulate power loss, then reboots to trigger JBD2 replay
#  - Optionally runs fsck.ext4 on the host to verify integrity

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
IMG="${1:-/tmp/ext4-test.img}"
SIZE_MB="${SIZE_MB:-64}"

mkfs=$(command -v mkfs.ext4 || true)
if [[ -z "$mkfs" ]]; then
  echo "[!] mkfs.ext4 not found. Install e2fsprogs (on macOS: brew install e2fsprogs)" >&2
  exit 1
fi

echo "[*] Creating ext4 image at $IMG (${SIZE_MB}MB)"
rm -f "$IMG"
dd if=/dev/zero of="$IMG" bs=1m count="$SIZE_MB" status=none
"$mkfs" -q "$IMG"

export EXT4_IMG="$IMG"
export VIRTBLK=mmio
export BRINGUP=1
export SIS_FEATURES="ext4-durability-test"

echo "[*] Booting kernel for phase 1 (will simulate power cut)"
"$SCRIPT_DIR/uefi_run.sh" >/tmp/sis-ext4-run1.log 2>&1 &
pid=$!
sleep 5
echo "[*] Simulating power cut (killing QEMU pid=$pid)"
kill -TERM "$pid" || true
sleep 1

echo "[*] Booting kernel for phase 2 (journal replay)"
"$SCRIPT_DIR/uefi_run.sh" >/tmp/sis-ext4-run2.log 2>&1 &
pid2=$!
sleep 5
kill -TERM "$pid2" || true

fsck=$(command -v fsck.ext4 || true)
if [[ -n "$fsck" ]]; then
  echo "[*] Running host fsck.ext4"
  "$fsck" -f -n "$IMG" || true
else
  echo "[i] fsck.ext4 not found; skipping host verification"
fi

echo "[+] Logs:"; echo "  - /tmp/sis-ext4-run1.log"; echo "  - /tmp/sis-ext4-run2.log"
echo "[+] Done"

