#!/bin/bash
# Prepare bootable USB files for x86_64 MacBook Pro
#
# This script creates the directory structure and copies the necessary files
# to create a bootable USB drive for UEFI systems.
#
# Usage: ./scripts/prepare_usb.sh [output_directory]
#
# The output directory will contain:
#   EFI/BOOT/BOOTX64.EFI  - UEFI bootloader
#   EFI/SIS/KERNEL.ELF    - SIS kernel

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default output directory
OUTPUT_DIR="${1:-usb_files}"

echo -e "${GREEN}=== SIS Kernel USB Preparation ===${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "crates/kernel" ]; then
    echo -e "${RED}Error: Must be run from the sis-kernel root directory${NC}"
    exit 1
fi

echo -e "${YELLOW}Step 1: Building x86_64 UEFI bootloader...${NC}"
cargo build --release --target x86_64-unknown-uefi --manifest-path crates/uefi-boot/Cargo.toml
if [ ! -f "crates/uefi-boot/target/x86_64-unknown-uefi/release/uefi-boot.efi" ]; then
    echo -e "${RED}Error: UEFI bootloader build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ UEFI bootloader built${NC}"

echo ""
echo -e "${YELLOW}Step 2: Building x86_64 kernel...${NC}"
SIS_FEATURES="llm,crypto-real" BRINGUP=1 cargo build --release --target crates/kernel/x86_64-sis.json -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem --manifest-path crates/kernel/Cargo.toml
if [ ! -f "crates/kernel/target/x86_64-sis/release/sis_kernel" ]; then
    echo -e "${RED}Error: Kernel build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Kernel built${NC}"

echo ""
echo -e "${YELLOW}Step 3: Creating USB file structure...${NC}"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/EFI/BOOT"
mkdir -p "$OUTPUT_DIR/EFI/SIS"
echo -e "${GREEN}✓ Directory structure created${NC}"

echo ""
echo -e "${YELLOW}Step 4: Copying files...${NC}"
cp crates/uefi-boot/target/x86_64-unknown-uefi/release/uefi-boot.efi "$OUTPUT_DIR/EFI/BOOT/BOOTX64.EFI"
echo -e "${GREEN}✓ Copied UEFI bootloader to EFI/BOOT/BOOTX64.EFI${NC}"

cp crates/kernel/target/x86_64-sis/release/sis_kernel "$OUTPUT_DIR/EFI/SIS/KERNEL.ELF"
echo -e "${GREEN}✓ Copied kernel to EFI/SIS/KERNEL.ELF${NC}"

echo ""
echo -e "${GREEN}=== USB files prepared successfully! ===${NC}"
echo ""
echo "Files are ready in: $OUTPUT_DIR"
echo ""
echo "Directory structure:"
tree "$OUTPUT_DIR" 2>/dev/null || find "$OUTPUT_DIR" -type f -o -type d

echo ""
echo -e "${YELLOW}File sizes:${NC}"
ls -lh "$OUTPUT_DIR/EFI/BOOT/BOOTX64.EFI"
ls -lh "$OUTPUT_DIR/EFI/SIS/KERNEL.ELF"

echo ""
echo -e "${GREEN}Next steps:${NC}"
echo "1. Insert a USB drive (will be formatted - backup any data!)"
echo "2. Identify the USB drive with: lsblk"
echo "3. Run: sudo ./scripts/create_bootable_usb.sh /dev/sdX"
echo "   (Replace /dev/sdX with your USB drive, e.g., /dev/sdb)"
echo ""
