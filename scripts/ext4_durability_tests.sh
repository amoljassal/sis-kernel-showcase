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
export QMP=1
export QMP_SOCK="/tmp/sis-ext4-qmp.sock"

echo "[*] Booting kernel for phase 1 (will simulate power cut)"
rm -f "$QMP_SOCK"
"$SCRIPT_DIR/uefi_run.sh" >/tmp/sis-ext4-run1.log 2>&1 &
pid=$!

# Wait for the self-test marker then issue QMP quit
echo "[*] Waiting for in-kernel self-test marker..."
for _ in $(seq 1 100); do
  if grep -q "\[EXT4TEST\] Operations done" /tmp/sis-ext4-run1.log; then
    break
  fi
  sleep 0.2
done

echo "[*] Simulating power cut via QMP"
python3 - "$QMP_SOCK" <<'PY'
import os, sys, socket, json, time
path = sys.argv[1]
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(path)
_ = s.recv(4096)  # greeting
s.sendall(b'{"execute": "qmp_capabilities"}\n')
time.sleep(0.1)
s.sendall(b'{"execute": "quit"}\n')
s.close()
PY

sleep 1

echo "[*] Booting kernel for phase 2 (journal replay)"
rm -f "$QMP_SOCK"
"$SCRIPT_DIR/uefi_run.sh" >/tmp/sis-ext4-run2.log 2>&1 &
pid2=$!
for _ in $(seq 1 100); do
  if grep -q "JBD2: Journal replay complete" /tmp/sis-ext4-run2.log; then
    break
  fi
  sleep 0.2
done
python3 - "$QMP_SOCK" <<'PY'
import os, sys, socket, json, time
path = sys.argv[1]
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(path)
_ = s.recv(4096)
s.sendall(b'{"execute": "qmp_capabilities"}\n')
time.sleep(0.1)
s.sendall(b'{"execute": "quit"}\n')
s.close()
PY

fsck=$(command -v fsck.ext4 || true)
if [[ -n "$fsck" ]]; then
  echo "[*] Running host fsck.ext4"
  "$fsck" -f -n "$IMG" > /tmp/sis-ext4-fsck.log 2>&1 || true
else
  echo "[i] fsck.ext4 not found; skipping host verification"
fi

# Summaries
REPLAY_SUM=$(grep -E "JBD2: Journal replay complete" /tmp/sis-ext4-run2.log | tail -n1 | sed -E 's/.*complete \(([^)]*)\).*/\1/')
FSCK_SUM="N/A"
if [[ -f /tmp/sis-ext4-fsck.log ]]; then
  if grep -q "clean" /tmp/sis-ext4-fsck.log; then
    FSCK_SUM="clean"
  elif grep -qiE "error|corrupt|repair" /tmp/sis-ext4-fsck.log; then
    FSCK_SUM="issues detected"
  else
    FSCK_SUM="no summary"
  fi
fi

echo "[+] ext4/JBD2 replay: ${REPLAY_SUM:-not found}"
echo "[+] fsck summary: $FSCK_SUM"
echo "[+] Logs:"; echo "  - /tmp/sis-ext4-run1.log"; echo "  - /tmp/sis-ext4-run2.log"; [[ -f /tmp/sis-ext4-fsck.log ]] && echo "  - /tmp/sis-ext4-fsck.log"
echo "[+] Done"
