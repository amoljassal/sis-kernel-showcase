#!/usr/bin/env bash
set -euo pipefail

# Prepare USB drive for booting SIS Kernel on Raspberry Pi 5/500+
# Usage: sudo ./prepare_rpi_usb.sh /dev/sdX

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <device>"
  echo "Example: $0 /dev/sda"
  echo ""
  echo "Available USB devices:"
  lsblk -d -o NAME,SIZE,TYPE,TRAN | grep usb || echo "No USB devices found"
  exit 1
fi

DEVICE="$1"

if [[ ! -b "$DEVICE" ]]; then
  echo "[!] $DEVICE is not a block device"
  exit 1
fi

if [[ "$EUID" -ne 0 ]]; then
  echo "[!] Must run as root (use sudo)"
  exit 1
fi

echo "========================================="
echo "   SIS Kernel - RPi USB Boot Setup"
echo "========================================="
lsblk "$DEVICE"
echo ""
echo "WARNING: All data on $DEVICE will be ERASED!"
read -p "Type 'YES' to continue: " confirm
[[ "$confirm" == "YES" ]] || exit 0

echo "[*] Unmounting..."
umount ${DEVICE}* 2>/dev/null || true

echo "[*] Creating FAT32 partition..."
parted -s "$DEVICE" mklabel msdos
parted -s "$DEVICE" mkpart primary fat32 1MiB 512MiB  
parted -s "$DEVICE" set 1 boot on
sleep 2; partprobe "$DEVICE" || true; sleep 1

PART="${DEVICE}1"
[[ -b "$PART" ]] || PART="${DEVICE}p1"

mkfs.vfat -F 32 -n SISBOOT "$PART"

MOUNT="/tmp/sis-$$"
mkdir -p "$MOUNT"
mount "$PART" "$MOUNT"

echo "[*] Downloading RPi UEFI (compatible with RPi 4/5)..."
cd /tmp
[[ -f RPi4_UEFI_Firmware_v1.38.zip ]] || \
  wget https://github.com/pftf/RPi4/releases/download/v1.38/RPi4_UEFI_Firmware_v1.38.zip
rm -rf rpi-uefi; mkdir rpi-uefi; cd rpi-uefi
unzip -q ../RPi4_UEFI_Firmware_v1.38.zip

echo "[*] Copying UEFI firmware..."
cp *.fd *.dtb *.dat *.elf *.bin "$MOUNT/" 2>/dev/null || true
[[ -f config.txt ]] && cp config.txt "$MOUNT/"

# Create config.txt to bypass OS check if not present
if [[ ! -f "$MOUNT/config.txt" ]]; then
  echo "[*] Creating config.txt with os_check=0..."
  cat > "$MOUNT/config.txt" << 'CFGEOF'
# Raspberry Pi 5 UEFI Boot Configuration
# Disable OS compatibility check for custom kernels
os_check=0
CFGEOF
fi

mkdir -p "$MOUNT/EFI/BOOT" "$MOUNT/EFI/SIS"

echo "[*] Building kernel..."
# Get the project root directory (script is in scripts/ subdirectory)
SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname "$SCRIPT_PATH")"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Source cargo env from the actual user's home, not root
REAL_USER=$(logname || echo $SUDO_USER)
REAL_HOME=$(eval echo ~$REAL_USER)
source "$REAL_HOME/.cargo/env"

rustup target add aarch64-unknown-uefi aarch64-unknown-none

cargo build --release --target aarch64-unknown-uefi \
  --manifest-path crates/uefi-boot/Cargo.toml

cargo +nightly build -Z build-std=core,alloc \
  --target aarch64-unknown-none \
  --manifest-path crates/kernel/Cargo.toml \
  --features "bringup"

cp target/aarch64-unknown-uefi/release/uefi-boot.efi "$MOUNT/EFI/BOOT/BOOTAA64.EFI"
cp target/aarch64-unknown-none/debug/sis_kernel "$MOUNT/EFI/SIS/KERNEL.ELF"

echo "[*] Files copied:"
ls -lh "$MOUNT/EFI/BOOT/BOOTAA64.EFI"
ls -lh "$MOUNT/EFI/SIS/KERNEL.ELF"

sync
umount "$MOUNT"
rmdir "$MOUNT"

echo ""
echo "✓ USB drive ready!"
echo ""
echo "Next steps:"
echo "1. Insert USB into Raspberry Pi 500+"
echo "2. Power on (remove SD card if present)"
echo "3. Connect UART: screen /dev/ttyUSB0 115200"
echo "4. Watch for: Platform detected: RaspberryPi5"
