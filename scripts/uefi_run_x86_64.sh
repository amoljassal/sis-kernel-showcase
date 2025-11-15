#!/usr/bin/env bash
set -euo pipefail

# Build the x86_64 UEFI app and kernel, then launch QEMU with OVMF firmware.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"

ESP_DIR="$SCRIPT_DIR/esp_x86_64"
EFI_BOOT_DIR="$ESP_DIR/EFI/BOOT"
EFI_SIS_DIR="$ESP_DIR/EFI/SIS"

QEMU_BIN="${QEMU_BIN:-qemu-system-x86_64}"
QEMU_MEM="${QEMU_MEM:-1024M}"
QEMU_SMP="${QEMU_SMP:-2}"

resolve_firmware() {
  local var_name="$1"; shift
  local env_value="${!var_name:-}"
  if [[ -n "$env_value" && -f "$env_value" ]]; then
    echo "$env_value"
    return 0
  fi
  for candidate in "$@"; do
    if [[ -f "$candidate" ]]; then
      echo "$candidate"
      return 0
    fi
  done
  return 1
}

echo "[*] Building UEFI app (x86_64-unknown-uefi)..."
rustup target add x86_64-unknown-uefi >/dev/null 2>&1 || true
if [[ -n "${BOOT_FEATURES:-}" ]]; then
  echo "[*] Building uefi-boot with features: ${BOOT_FEATURES}"
  cargo build \
    --manifest-path crates/uefi-boot/Cargo.toml \
    --target-dir "$ROOT_DIR/target" \
    --release \
    --target x86_64-unknown-uefi \
    --features "${BOOT_FEATURES}"
else
  cargo build \
    --manifest-path crates/uefi-boot/Cargo.toml \
    --target-dir "$ROOT_DIR/target" \
    --release \
    --target x86_64-unknown-uefi
fi

UEFI_APP="$ROOT_DIR/target/x86_64-unknown-uefi/release/uefi-boot.efi"
if [[ ! -f "$UEFI_APP" ]]; then
  echo "[!] UEFI app not found: $UEFI_APP" >&2
  exit 1
fi

echo "[*] Preparing ESP at $ESP_DIR ..."
rm -rf "$ESP_DIR"
mkdir -p "$EFI_BOOT_DIR" "$EFI_SIS_DIR"
cp "$UEFI_APP" "$EFI_BOOT_DIR/BOOTX64.EFI"

echo "[*] Building kernel (x86_64-unknown-none)..."
rustup target add x86_64-unknown-none >/dev/null 2>&1 || true

FEATURES=""
if [[ -n "${BRINGUP:-}" ]]; then
  echo "[*] Enabling bringup feature"
  FEATURES="${FEATURES},bringup"
fi
if [[ -n "${AI:-}" ]]; then
  echo "[*] Enabling AI features"
  FEATURES="${FEATURES},arm64-ai,formal-verification,neon-optimized"
fi
if [[ -n "${NEON:-}" ]]; then
  echo "[*] Enabling neon-optimized feature"
  FEATURES="${FEATURES},neon-optimized"
fi
if [[ -n "${GRAPH:-}" ]]; then
  echo "[*] Enabling graph demo"
  FEATURES="${FEATURES},graph-demo"
fi
if [[ -n "${GRAPH_STATS:-}" ]]; then
  echo "[*] Enabling graph stats"
  FEATURES="${FEATURES},graph-autostats"
fi
if [[ -n "${PERF:-}" ]]; then
  echo "[*] Enabling perf-verbose"
  FEATURES="${FEATURES},perf-verbose"
fi
if [[ -n "${VIRTIO:-}" ]]; then
  echo "[*] Enabling virtio-console"
  FEATURES="${FEATURES},virtio-console"
fi
if [[ -n "${SIS_FEATURES:-}" ]]; then
  echo "[*] Adding SIS_FEATURES: ${SIS_FEATURES}"
  FEATURES="${FEATURES},${SIS_FEATURES}"
fi
if [[ "${GRAPHCTL_FRAMED:-1}" != "0" ]]; then
  echo "[*] Enabling graphctl-framed feature"
  FEATURES="${FEATURES},graphctl-framed"
else
  echo "[*] graphctl-framed disabled by env"
fi
FEATURES="${FEATURES#,}"

if [[ -n "$FEATURES" ]]; then
  echo "[*] Building kernel with features: $FEATURES"
  cargo +nightly build \
    --manifest-path crates/kernel/Cargo.toml \
    --target-dir "$ROOT_DIR/target" \
    -Z build-std=core,alloc \
    --target x86_64-unknown-none \
    --features "$FEATURES"
else
  cargo +nightly build \
    --manifest-path crates/kernel/Cargo.toml \
    --target-dir "$ROOT_DIR/target" \
    -Z build-std=core,alloc \
    --target x86_64-unknown-none
fi

KERNEL_ELF="$ROOT_DIR/target/x86_64-unknown-none/debug/sis_kernel"
if [[ ! -f "$KERNEL_ELF" ]]; then
  echo "[!] Kernel ELF not found: $KERNEL_ELF" >&2
  exit 1
fi
cp "$KERNEL_ELF" "$EFI_SIS_DIR/KERNEL.ELF"

echo "[*] ESP contents:"
ls -l "$EFI_BOOT_DIR" "$EFI_SIS_DIR"
if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$EFI_BOOT_DIR/BOOTX64.EFI" "$EFI_SIS_DIR/KERNEL.ELF" | sed 's/^/  /'
fi

echo "[*] Resolving OVMF firmware..."
OVMF_CODE_PATH="$(resolve_firmware OVMF_CODE \
  "/usr/share/OVMF/OVMF_CODE.fd" \
  "/usr/share/OVMF/OVMF_CODE.secboot.fd" \
  "/usr/share/qemu/OVMF_CODE.fd" \
  "/usr/lib/ovmf/OVMF_CODE.fd" \
  "$SCRIPT_DIR/OVMF_CODE.fd" \
  "$ROOT_DIR/firmware/OVMF_CODE.fd" \
  "$ROOT_DIR/firmware/ovmf-prebuilt/OVMF_CODE.fd" \
)" || {
  cat >&2 <<'EOF'
[!] Could not find OVMF_CODE firmware image.
    Set OVMF_CODE env var to the path of OVMF_CODE.fd, e.g.:
      OVMF_CODE=/usr/share/OVMF/OVMF_CODE.fd ./scripts/uefi_run_x86_64.sh
EOF
  exit 1
}

OVMF_VARS_TEMPLATE="$(resolve_firmware OVMF_VARS \
  "/usr/share/OVMF/OVMF_VARS.fd" \
  "/usr/share/OVMF/OVMF_VARS.secboot.fd" \
  "/usr/share/qemu/OVMF_VARS.fd" \
  "/usr/lib/ovmf/OVMF_VARS.fd" \
  "$SCRIPT_DIR/OVMF_VARS.fd" \
  "$ROOT_DIR/firmware/OVMF_VARS.fd" \
  "$ROOT_DIR/firmware/ovmf-prebuilt/OVMF_VARS.fd" \
)" || true

OVMF_VARS_RUNTIME="$SCRIPT_DIR/OVMF_VARS_X64.fd"
if [[ -n "${OVMF_VARS_TEMPLATE:-}" ]]; then
  cp "$OVMF_VARS_TEMPLATE" "$OVMF_VARS_RUNTIME"
else
  echo "[!] OVMF_VARS template not found; creating blank vars file (state will not persist)."
  dd if=/dev/zero of="$OVMF_VARS_RUNTIME" bs=1M count=2 >/dev/null 2>&1
fi

if ! command -v "$QEMU_BIN" >/dev/null 2>&1; then
  echo "[!] QEMU binary '$QEMU_BIN' not found. Install qemu-system-x86_64 or set QEMU_BIN." >&2
  exit 1
fi

echo "[*] Launching QEMU (x86_64, OVMF)..."
echo "[i] Quit: Ctrl+a, then x"

if [[ -n "${VIRTIO:-}" && -S "/tmp/sis-datactl.sock" ]]; then
  echo "[*] Removing stale /tmp/sis-datactl.sock"
  rm -f /tmp/sis-datactl.sock || true
fi

DEBUG_ARGS=()
if [[ -n "${DEBUG:-}" ]]; then
  DEBUG_LOG="${DEBUG_LOG:-/tmp/qemu-x86-debug.log}"
  DEBUG_ARGS=(-d int,mmio -D "$DEBUG_LOG")
  echo "[*] Debug mode enabled: logging to $DEBUG_LOG"
fi

QEMU_ARGS=(
  -machine q35,accel=tcg
  -cpu "${QEMU_CPU:-qemu64,+sse2,+sse3,+sse4.1,+sse4.2}"
  -m "$QEMU_MEM"
  -smp "$QEMU_SMP"
  -nographic
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE_PATH"
  -drive if=pflash,format=raw,file="$OVMF_VARS_RUNTIME"
  -drive if=none,id=esp,format=raw,file=fat:rw:"$ESP_DIR"
  -device virtio-blk-pci,drive=esp,id=boot-disk,disable-legacy=on
  -device virtio-rng-pci,id=rng0,disable-legacy=on
)

if [[ "${QEMU_NET:-1}" != "0" ]]; then
  QEMU_ARGS+=( -netdev user,id=n0 )
  QEMU_ARGS+=( -device virtio-net-pci,netdev=n0 )
fi

if [[ "${QEMU_GPU:-0}" != "0" ]]; then
  QEMU_ARGS+=( -device virtio-gpu-pci )
fi

if [[ -n "${EXT4_IMG:-}" ]]; then
  if [[ ! -f "$EXT4_IMG" ]]; then
    echo "[!] EXT4_IMG not found: $EXT4_IMG" >&2
    exit 1
  fi
  QEMU_ARGS+=( -drive if=none,id=ext4img,format=raw,file="$EXT4_IMG" )
  QEMU_ARGS+=( -device virtio-blk-pci,drive=ext4img )
fi

if [[ -n "${VIRTIO:-}" ]]; then
  QEMU_ARGS+=( -device virtio-serial-pci )
  if [[ -n "${DATACTL_TCP:-}" ]]; then
    PORT="${DATACTL_PORT:-7777}"
    echo "[*] Using TCP chardev for datactl on 127.0.0.1:${PORT}"
    QEMU_ARGS+=( -chardev socket,id=datactl,host=127.0.0.1,port=${PORT},server=on,wait=off )
  else
    echo "[*] Using UNIX socket chardev for datactl at /tmp/sis-datactl.sock"
    QEMU_ARGS+=( -chardev socket,id=datactl,server=on,wait=off,path=/tmp/sis-datactl.sock )
  fi
  QEMU_ARGS+=( -device virtconsole,chardev=datactl,name=sis.datactl )
fi

QMP_ARGS=()
if [[ -n "${QMP:-}" ]]; then
  QMP_ARGS=( -qmp "unix:${QMP_SOCK:-/tmp/sis-qmp.sock},server,nowait" )
fi

exec "$QEMU_BIN" \
  "${QEMU_ARGS[@]}" \
  "${DEBUG_ARGS[@]}" \
  "${QMP_ARGS[@]}"
