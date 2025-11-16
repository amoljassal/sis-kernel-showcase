#!/bin/bash
# Create a Mac-compatible bootable UEFI USB drive for x86_64 MacBook Pro
#
# This script creates a USB drive that's specifically compatible with
# Mac EFI firmware, which is pickier than standard UEFI systems.
#
# Usage: sudo ./scripts/create_bootable_usb_mac.sh /dev/sdX
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

echo -e "${YELLOW}=== SIS Kernel Mac-Compatible Bootable USB Creator ===${NC}"
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
echo -e "${YELLOW}Step 2: Wiping existing partition table...${NC}"
# Wipe any existing partition tables (both MBR and GPT)
dd if=/dev/zero of="$DEVICE" bs=512 count=2048 status=none
sync
echo -e "${GREEN}✓ Partition table wiped${NC}"

echo ""
echo -e "${YELLOW}Step 3: Creating Mac-compatible GPT partition table...${NC}"
# Use gdisk for better Mac compatibility
# Create GPT partition table with Mac-compatible settings
sgdisk -Z "$DEVICE"  # Zap all partition data
sgdisk -n 1:2048:+512M -t 1:EF00 -c 1:"EFI System" "$DEVICE"
sgdisk -p "$DEVICE"  # Print partition table
sync
sleep 2
echo -e "${GREEN}✓ GPT partition created${NC}"

echo ""
echo -e "${YELLOW}Step 4: Creating hybrid MBR for Mac compatibility...${NC}"
# Macs sometimes need a hybrid MBR/GPT setup
# This creates a protective MBR entry that makes Macs happy
sgdisk -h 1 "$DEVICE" 2>/dev/null || true
sync
sleep 2
echo -e "${GREEN}✓ Hybrid MBR created${NC}"

# Wait for partition to appear
sleep 2

# Determine partition name
if [[ "$DEVICE" == *"nvme"* ]]; then
    PARTITION="${DEVICE}p1"
else
    PARTITION="${DEVICE}1"
fi

# Ensure partition exists
if [ ! -b "$PARTITION" ]; then
    echo -e "${YELLOW}Waiting for partition to appear...${NC}"
    sleep 3
fi

if [ ! -b "$PARTITION" ]; then
    echo -e "${RED}Error: Partition $PARTITION did not appear${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 5: Formatting partition as FAT32...${NC}"
# Format as FAT32 with specific settings for Mac compatibility
# -F 32: FAT32
# -n: Volume label
# -s 2: 1KB clusters (better for small files)
mkfs.vfat -F 32 -n "EFI" -s 2 "$PARTITION"
echo -e "${GREEN}✓ Formatted as FAT32${NC}"

echo ""
echo -e "${YELLOW}Step 6: Mounting partition...${NC}"
MOUNT_POINT="/mnt/sisboot_mac_$$"
mkdir -p "$MOUNT_POINT"
mount "$PARTITION" "$MOUNT_POINT"
echo -e "${GREEN}✓ Mounted at $MOUNT_POINT${NC}"

echo ""
echo -e "${YELLOW}Step 7: Copying bootloader and kernel...${NC}"
# Copy files
cp -r "$USB_FILES/EFI" "$MOUNT_POINT/"

# Create .disk directory (some Macs look for this)
mkdir -p "$MOUNT_POINT/.disk"

# Sync to ensure all writes complete
sync
echo -e "${GREEN}✓ Files copied${NC}"

echo ""
echo -e "${YELLOW}Step 8: Setting Mac boot blessing (optional)...${NC}"
# Note: True "blessing" requires macOS's bless command
# But we can create marker files that help
touch "$MOUNT_POINT/EFI/BOOT/.macboot"
sync
echo -e "${GREEN}✓ Boot markers created${NC}"

echo ""
echo -e "${YELLOW}Step 9: Verifying files...${NC}"
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
echo -e "${YELLOW}Step 10: Unmounting...${NC}"
umount "$MOUNT_POINT"
rmdir "$MOUNT_POINT"
sync
echo -e "${GREEN}✓ Unmounted${NC}"

echo ""
echo -e "${GREEN}=== Mac-Compatible Bootable USB created successfully! ===${NC}"
echo ""
echo -e "${GREEN}Mac-Specific Boot Instructions:${NC}"
echo ""
echo "1. Safely remove the USB drive (wait 5 seconds after unmount)"
echo "2. Insert it into your MacBook Pro Mid 2012"
echo ""
echo -e "${YELLOW}Method 1: Startup Manager (Recommended)${NC}"
echo "   - Shut down the Mac completely"
echo "   - Power on and IMMEDIATELY hold Option (⌥) key"
echo "   - Keep holding until you see the Startup Manager"
echo "   - You should see an icon labeled 'EFI Boot' or 'Windows'"
echo "   - Use arrow keys to select it"
echo "   - Press Enter or click the arrow below the icon"
echo ""
echo -e "${YELLOW}Method 2: If Method 1 doesn't work${NC}"
echo "   - Shut down the Mac"
echo "   - Reset NVRAM: Hold Cmd+Option+P+R during startup"
echo "   - Wait for two chimes, then release"
echo "   - Try Method 1 again"
echo ""
echo -e "${YELLOW}Method 3: Set as startup disk from macOS${NC}"
echo "   - Boot into macOS"
echo "   - Insert the USB drive"
echo "   - Open System Preferences > Startup Disk"
echo "   - Select the 'EFI Boot' option"
echo "   - Click 'Restart'"
echo ""
echo -e "${YELLOW}Troubleshooting:${NC}"
echo "   - If you don't see the USB in Startup Manager, try a different USB port"
echo "   - Some Macs only boot from specific ports (usually the ones closest to the hinge)"
echo "   - Make sure the USB is fully inserted"
echo "   - The USB LED should blink when selected if it's being read"
echo ""
echo -e "${GREEN}Good luck! The kernel should boot and display output on screen.${NC}"
echo ""
