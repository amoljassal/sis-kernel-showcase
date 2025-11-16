#!/bin/bash
# Create a bootable UEFI USB drive for x86_64 MacBook Pro
#
# This script will:
# 1. Format the USB drive with GPT partition table
# 2. Create an EFI System Partition (ESP)
# 3. Format it as FAT32
# 4. Copy the bootloader and kernel
#
# Usage: sudo ./scripts/create_bootable_usb.sh /dev/sdX
#
# WARNING: This will ERASE ALL DATA on the specified drive!

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
    exit 1
fi

# Check arguments
if [ $# -ne 1 ]; then
    echo "Usage: sudo $0 /dev/sdX"
    echo "Example: sudo $0 /dev/sdb"
    echo ""
    echo "Available drives:"
    lsblk -d -o NAME,SIZE,MODEL | grep -E "sd|nvme"
    exit 1
fi

DEVICE="$1"
USB_FILES="usb_files"

# Validate device
if [ ! -b "$DEVICE" ]; then
    echo -e "${RED}Error: $DEVICE is not a block device${NC}"
    exit 1
fi

# Safety check - don't allow /dev/sda (usually the main system disk)
if [[ "$DEVICE" == "/dev/sda" ]] || [[ "$DEVICE" == "/dev/nvme0n1" ]]; then
    echo -e "${RED}Error: Refusing to format $DEVICE (appears to be the system disk)${NC}"
    echo "Please use a different USB drive"
    exit 1
fi

# Check if USB files are prepared
if [ ! -f "$USB_FILES/EFI/BOOT/BOOTX64.EFI" ] || [ ! -f "$USB_FILES/EFI/SIS/KERNEL.ELF" ]; then
    echo -e "${RED}Error: USB files not found. Run ./scripts/prepare_usb.sh first${NC}"
    exit 1
fi

echo -e "${YELLOW}=== SIS Kernel Bootable USB Creator ===${NC}"
echo ""
echo -e "${RED}WARNING: This will ERASE ALL DATA on $DEVICE${NC}"
echo ""
echo "Device information:"
lsblk "$DEVICE" -o NAME,SIZE,MODEL,MOUNTPOINT
echo ""
read -p "Are you sure you want to continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo -e "${YELLOW}Step 1: Unmounting any mounted partitions...${NC}"
umount ${DEVICE}* 2>/dev/null || true
echo -e "${GREEN}✓ Unmounted${NC}"

echo ""
echo -e "${YELLOW}Step 2: Creating GPT partition table...${NC}"
parted -s "$DEVICE" mklabel gpt
echo -e "${GREEN}✓ GPT partition table created${NC}"

echo ""
echo -e "${YELLOW}Step 3: Creating EFI System Partition (512MB)...${NC}"
parted -s "$DEVICE" mkpart ESP fat32 1MiB 513MiB
parted -s "$DEVICE" set 1 esp on
echo -e "${GREEN}✓ EFI System Partition created${NC}"

# Wait for partition to appear
sleep 2

# Determine partition name (handle both /dev/sdX1 and /dev/nvme0n1p1 formats)
if [[ "$DEVICE" == *"nvme"* ]]; then
    PARTITION="${DEVICE}p1"
else
    PARTITION="${DEVICE}1"
fi

echo ""
echo -e "${YELLOW}Step 4: Formatting partition as FAT32...${NC}"
mkfs.vfat -F 32 -n "SISBOOT" "$PARTITION"
echo -e "${GREEN}✓ Formatted as FAT32${NC}"

echo ""
echo -e "${YELLOW}Step 5: Mounting partition...${NC}"
MOUNT_POINT="/mnt/sisboot_$$"
mkdir -p "$MOUNT_POINT"
mount "$PARTITION" "$MOUNT_POINT"
echo -e "${GREEN}✓ Mounted at $MOUNT_POINT${NC}"

echo ""
echo -e "${YELLOW}Step 6: Copying bootloader and kernel...${NC}"
cp -r "$USB_FILES/EFI" "$MOUNT_POINT/"
sync
echo -e "${GREEN}✓ Files copied${NC}"

echo ""
echo -e "${YELLOW}Step 7: Verifying files...${NC}"
if [ -f "$MOUNT_POINT/EFI/BOOT/BOOTX64.EFI" ] && [ -f "$MOUNT_POINT/EFI/SIS/KERNEL.ELF" ]; then
    echo -e "${GREEN}✓ Files verified${NC}"
    echo ""
    echo "Files on USB:"
    ls -lh "$MOUNT_POINT/EFI/BOOT/BOOTX64.EFI"
    ls -lh "$MOUNT_POINT/EFI/SIS/KERNEL.ELF"
else
    echo -e "${RED}Error: Files not found on USB${NC}"
    umount "$MOUNT_POINT"
    rmdir "$MOUNT_POINT"
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 8: Unmounting...${NC}"
umount "$MOUNT_POINT"
rmdir "$MOUNT_POINT"
echo -e "${GREEN}✓ Unmounted${NC}"

echo ""
echo -e "${GREEN}=== Bootable USB created successfully! ===${NC}"
echo ""
echo -e "${GREEN}Next steps:${NC}"
echo "1. Safely remove the USB drive"
echo "2. Insert it into your MacBook Pro Mid 2012"
echo "3. Power on while holding the Option (⌥) key"
echo "4. Select 'EFI Boot' or 'SISBOOT' from the boot menu"
echo "5. The kernel will boot and you can see output via serial console"
echo ""
echo -e "${YELLOW}Note:${NC} To see kernel output, you'll need:"
echo "  - A USB-to-Serial adapter connected to the Mac's serial port (if available)"
echo "  - OR monitor the display for framebuffer output (GOP)"
echo "  - OR use remote debugging if configured"
echo ""
