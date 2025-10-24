#!/usr/bin/env bash
set -euo pipefail

# Build the UEFI boot app and run it under QEMU with edk2-aarch64 firmware.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"
ESP_DIR="$SCRIPT_DIR/esp"
EFI_BOOT_DIR="$ESP_DIR/EFI/BOOT"
EFI_SIS_DIR="$ESP_DIR/EFI/SIS"

echo "[*] Building UEFI app (aarch64-unknown-uefi)..."
rustup target add aarch64-unknown-uefi >/dev/null 2>&1 || true
cargo build -p uefi-boot --release --target aarch64-unknown-uefi

UEFI_APP="$ROOT_DIR/target/aarch64-unknown-uefi/release/uefi-boot.efi"
if [[ ! -f "$UEFI_APP" ]]; then
  echo "[!] UEFI app not found: $UEFI_APP" >&2
  exit 1
fi

echo "[*] Preparing ESP at $ESP_DIR ..."
rm -rf "$ESP_DIR"
mkdir -p "$EFI_BOOT_DIR" "$EFI_SIS_DIR"
cp "$UEFI_APP" "$EFI_BOOT_DIR/BOOTAA64.EFI"

echo "[*] Building kernel (aarch64-unknown-none)..."
rustup target add aarch64-unknown-none >/dev/null 2>&1 || true
export RUSTFLAGS="-C link-arg=-T$ROOT_DIR/crates/kernel/src/arch/aarch64/aarch64-qemu.ld"

# Build features based on environment variables
FEATURES=""
if [[ "${BRINGUP:-}" != "" ]]; then
  echo "[*] Enabling bringup feature (STACK/VECTORS/MMU)"
  FEATURES="${FEATURES},bringup"
fi
if [[ "${AI:-}" != "" ]]; then
  echo "[*] Enabling AI features (NEON, formal-verification)"
  FEATURES="${FEATURES},arm64-ai,formal-verification,neon-optimized"
fi
if [[ "${NEON:-}" != "" ]]; then
  echo "[*] Forcing NEON-optimized benchmarks"
  FEATURES="${FEATURES},neon-optimized"
fi

# Optional toggles for demos and verbose perf
if [[ "${GRAPH:-}" != "" ]]; then
  echo "[*] Enabling graph demo"
  FEATURES="${FEATURES},graph-demo"
fi
if [[ "${GRAPH_STATS:-}" != "" ]]; then
  echo "[*] Enabling baseline graph stats"
  FEATURES="${FEATURES},graph-autostats"
fi
if [[ "${PERF:-}" != "" ]]; then
  echo "[*] Enabling perf-verbose"
  FEATURES="${FEATURES},perf-verbose"
fi

# Optional toggle for VirtIO console driver path (off by default)
if [[ "${VIRTIO:-}" != "" ]]; then
  echo "[*] Enabling virtio-console feature"
  FEATURES="${FEATURES},virtio-console"
fi

# Allow caller to pass arbitrary additional features via SIS_FEATURES
if [[ -n "${SIS_FEATURES:-}" ]]; then
  echo "[*] Adding SIS_FEATURES: ${SIS_FEATURES}"
  FEATURES="${FEATURES},${SIS_FEATURES}"
fi

# Remove leading comma if present
FEATURES="${FEATURES#,}"

if [[ -n "$FEATURES" ]]; then
  echo "[*] Building with features: $FEATURES"
  cargo +nightly build -p sis_kernel -Z build-std=core,alloc --target aarch64-unknown-none --features "$FEATURES"
else
  cargo +nightly build -p sis_kernel -Z build-std=core,alloc --target aarch64-unknown-none
fi
KERNEL_ELF="$ROOT_DIR/target/aarch64-unknown-none/debug/sis_kernel"
if [[ ! -f "$KERNEL_ELF" ]]; then
  echo "[!] Kernel ELF not found: $KERNEL_ELF" >&2
  exit 1
fi
cp "$KERNEL_ELF" "$EFI_SIS_DIR/KERNEL.ELF"

echo "[*] ESP contents:"
ls -l "$EFI_BOOT_DIR" "$EFI_SIS_DIR"
if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$EFI_BOOT_DIR/BOOTAA64.EFI" "$EFI_SIS_DIR/KERNEL.ELF" | sed 's/^/  /'
fi

FIRMWARE="/opt/homebrew/share/qemu/edk2-aarch64-code.fd"
if [[ ! -f "$FIRMWARE" ]]; then
  echo "[!] EDK2 firmware not found at $FIRMWARE"
  echo "    Install with: brew install qemu (or edk2-aarch64)"
  exit 1
fi

echo "[*] Launching QEMU (UEFI) with GICv3, highmem, and VirtIO devices ..."
echo "[i] Quit: Ctrl+a, then x (monitor on stdio)"
# Clean up stale control socket to avoid bind/connect issues
if [[ -n "${VIRTIO:-}" ]]; then
  if [[ -S "/tmp/sis-datactl.sock" ]]; then
    echo "[*] Removing stale /tmp/sis-datactl.sock"
    rm -f /tmp/sis-datactl.sock || true
  fi
fi
# Add debugging for VirtIO discovery if DEBUG env var is set
DEBUG_FLAGS=""
if [[ "${DEBUG:-}" != "" ]]; then
  DEBUG_FLAGS="-d int,mmio -D /tmp/qemu-debug.log"
  echo "[*] Debug mode enabled: logging to /tmp/qemu-debug.log"
fi

QEMU_DEVICES=(
  -M virt,gic-version=3,highmem=on,secure=off
  -cpu cortex-a72,pmu=on
  -m 512M
  -nographic
  -bios "$FIRMWARE"
  -drive if=none,id=esp,format=raw,file=fat:rw:"$ESP_DIR"
  -device virtio-blk-pci,drive=esp,id=boot-disk,disable-legacy=on
  -device virtio-rng-pci,id=rng0,disable-legacy=on
  -device virtio-net-pci,netdev=net0,id=net0,disable-legacy=on
  -netdev user,id=net0
)

# Add virtio-serial only if VIRTIO=1
if [[ "${VIRTIO:-}" != "" ]]; then
  QEMU_DEVICES+=( -device virtio-serial-device )
  if [[ -n "${DATACTL_TCP:-}" ]]; then
    PORT="${DATACTL_PORT:-7777}"
    echo "[*] Using TCP chardev for datactl on 127.0.0.1:${PORT}"
    # Use socket backend in TCP server mode with correct key=value syntax
    QEMU_DEVICES+=( -chardev socket,id=datactl,host=127.0.0.1,port=${PORT},server=on,wait=off )
  else
    echo "[*] Using UNIX socket chardev for datactl at /tmp/sis-datactl.sock"
    QEMU_DEVICES+=( -chardev socket,id=datactl,server=on,wait=off,path=/tmp/sis-datactl.sock )
  fi
  # Bind a single primary virtconsole port for control-plane with a stable name for multiport
  # The name propagates via the control channel (PortName), allowing the guest to bind reliably.
  QEMU_DEVICES+=( -device virtconsole,chardev=datactl,name=sis.datactl )
fi

qemu-system-aarch64 \
  "${QEMU_DEVICES[@]}" \
  -rtc base=utc \
  -no-reboot \
  -smp 2 \
  $DEBUG_FLAGS
