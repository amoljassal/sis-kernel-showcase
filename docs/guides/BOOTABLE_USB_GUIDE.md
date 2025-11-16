# Creating a Bootable USB Drive for MacBook Pro Mid 2012

This guide walks you through creating a bootable USB drive to test the SIS kernel on real x86_64 hardware (MacBook Pro Mid 2012).

## Prerequisites

- Ubuntu Linux system (or any Linux distribution)
- USB drive (minimum 512MB, will be formatted/erased)
- MacBook Pro Mid 2012 with Intel processor
- Basic familiarity with Linux command line

## Overview

The process involves:
1. **Building** the UEFI bootloader and kernel
2. **Preparing** the USB file structure
3. **Creating** the bootable USB drive
4. **Booting** from the USB on your MacBook Pro

---

## Step 1: Build the UEFI Bootloader and Kernel

Run the preparation script:

```bash
cd ~/projects/sis-kernel
./scripts/prepare_usb.sh
```

This script will:
- Build the x86_64 UEFI bootloader (`BOOTX64.EFI`)
- Build the x86_64 kernel (`KERNEL.ELF`)
- Create the proper directory structure in `usb_files/`

Expected output:
```
=== SIS Kernel USB Preparation ===

Step 1: Building x86_64 UEFI bootloader...
✓ UEFI bootloader built

Step 2: Building x86_64 kernel...
✓ Kernel built

Step 3: Creating USB file structure...
✓ Directory structure created

Step 4: Copying files...
✓ Copied UEFI bootloader to EFI/BOOT/BOOTX64.EFI
✓ Copied kernel to EFI/SIS/KERNEL.ELF

=== USB files prepared successfully! ===
```

---

## Step 2: Identify Your USB Drive

**⚠️ WARNING:** The next steps will **ERASE ALL DATA** on the USB drive!

Insert your USB drive and identify it:

```bash
lsblk
```

Example output:
```
NAME   MAJ:MIN RM   SIZE RO TYPE MOUNTPOINT
sda      8:0    0 238.5G  0 disk
├─sda1   8:1    0   512M  0 part /boot/efi
└─sda2   8:2    0   238G  0 part /
sdb      8:16   1   7.5G  0 disk          ← Your USB drive
└─sdb1   8:17   1   7.5G  0 part /media/usb
```

In this example, the USB drive is `/dev/sdb`. **Make absolutely sure you identify the correct device!**

Common USB device names:
- `/dev/sdb` - Second drive
- `/dev/sdc` - Third drive
- `/dev/nvme0n1` - NVMe drives (not typical for USB)

---

## Step 3: Create the Bootable USB

Run the USB creation script with sudo:

```bash
sudo ./scripts/create_bootable_usb.sh /dev/sdX
```

Replace `/dev/sdX` with your USB drive (e.g., `/dev/sdb`).

Example:
```bash
sudo ./scripts/create_bootable_usb.sh /dev/sdb
```

The script will:
1. Show device information and ask for confirmation
2. Unmount any existing partitions
3. Create a GPT partition table
4. Create an EFI System Partition (ESP)
5. Format it as FAT32 with label "SISBOOT"
6. Copy the bootloader and kernel
7. Verify and unmount

Expected confirmation prompt:
```
=== SIS Kernel Bootable USB Creator ===

WARNING: This will ERASE ALL DATA on /dev/sdb

Device information:
NAME   SIZE MODEL            MOUNTPOINT
sdb    7.5G SanDisk_USB_3.2
└─sdb1 7.5G

Are you sure you want to continue? (yes/no):
```

Type `yes` to proceed.

---

## Step 4: Boot from USB on MacBook Pro

### 4.1 Safely Remove the USB Drive

```bash
sync
# Wait a few seconds, then unplug the USB drive
```

### 4.2 Insert into MacBook Pro

1. Power off your MacBook Pro Mid 2012
2. Insert the USB drive into a USB port
3. Power on while **holding the Option (⌥) key**
4. Keep holding until you see the boot menu

### 4.3 Select the Boot Device

You should see:
- **Macintosh HD** - Your Mac's internal drive
- **EFI Boot** or **SISBOOT** - Your USB drive

Use arrow keys to select **EFI Boot** or **SISBOOT**, then press Enter.

---

## Step 5: Observing the Kernel Boot

The kernel will boot and initialize. Here's what you should see:

### Option A: Display Output (Framebuffer/GOP)

The kernel has UEFI GOP framebuffer support, so you should see output on the MacBook's display:

```
================================================================================
                         SIS Kernel - x86_64 Architecture
================================================================================

[BOOT] Early initialization started
[BOOT] GDT loaded
[BOOT] TSS loaded
[BOOT] IDT loaded
...
```

### Option B: Serial Console (Advanced)

If you have a USB-to-Serial adapter connected:

1. Connect the adapter to another computer
2. Use a terminal program (minicom, screen, or PuTTY)
3. Settings: 115200 baud, 8N1, no flow control
4. You'll see detailed kernel output

```bash
# On Linux:
screen /dev/ttyUSB0 115200

# Or:
minicom -D /dev/ttyUSB0 -b 115200
```

---

## Expected Kernel Output

The kernel will initialize hardware and display:

```
[BOOT] Early initialization started
[BOOT] CPU features enabled
[CPU] Vendor: GenuineIntel
[CPU] Model: Intel(R) Core(TM) i5-3210M CPU @ 2.50GHz
[HPET] High Precision Event Timer initialized
[ACPI] ACPI initialization complete
[PCI] Initializing PCI bus enumeration
[PCI] Found device 00:00.0: vendor=0x8086 ...
[BOOT] AHCI/SATA Controller Detection
[AHCI] Initializing controller at BAR5: ...
[AHCI] Port 0: Device detected (SATA drive)
[BOOT] PS/2 Keyboard Controller Initialization
[PS2] PS/2 keyboard initialized
[BOOT] Kernel initialization complete
[BOOT] PS/2 keyboard is active - press keys to test!
```

### Testing Keyboard Input

Once you see `[BOOT] PS/2 keyboard is active - press keys to test!`, try pressing keys:

```
[KEYBOARD] Key pressed: 'h' (ASCII: 104)
[KEYBOARD] Key pressed: 'e' (ASCII: 101)
[KEYBOARD] Key pressed: 'l' (ASCII: 108)
[KEYBOARD] Key pressed: 'l' (ASCII: 108)
[KEYBOARD] Key pressed: 'o' (ASCII: 111)
```

---

## Troubleshooting

### USB Drive Not Appearing in Boot Menu

1. **Try a different USB port** - Some Macs are picky about which ports work for booting
2. **Hold Option key earlier** - Start holding immediately after power button press
3. **Reset NVRAM** - Hold Cmd+Option+P+R at startup until you hear the chime twice
4. **Check BIOS/EFI settings** - Ensure USB booting is enabled

### Kernel Doesn't Boot / Black Screen

1. **Verify USB files**:
   ```bash
   sudo mount /dev/sdb1 /mnt
   ls -la /mnt/EFI/BOOT/BOOTX64.EFI
   ls -la /mnt/EFI/SIS/KERNEL.ELF
   sudo umount /mnt
   ```

2. **Check file sizes**:
   - `BOOTX64.EFI` should be ~100-500 KB
   - `KERNEL.ELF` should be ~10-50 MB

3. **Rebuild with verbose output**:
   ```bash
   ./scripts/prepare_usb.sh 2>&1 | tee build.log
   ```

### Serial Console Shows Garbage

- Check baud rate: Must be 115200
- Check connection: RX/TX might be swapped
- Try different terminal settings

### Kernel Panics or Crashes

1. Note the error message
2. Check if it's a hardware compatibility issue
3. The kernel has detailed panic handlers - the error will show the failing component

---

## What Hardware Gets Detected

On a successful boot, the kernel will detect:

✅ **CPU**: Intel Core i5/i7 (Sandy Bridge/Ivy Bridge)
✅ **ACPI Tables**: Real hardware ACPI (not QEMU)
✅ **PCI Devices**: All PCI/PCIe devices
✅ **AHCI/SATA**: Internal SATA controller and drives
✅ **PS/2 Keyboard**: Built-in keyboard
✅ **Framebuffer**: Display via UEFI GOP
✅ **Serial Port**: If available via adapter

---

## Next Steps After Successful Boot

Once the kernel boots successfully on real hardware:

1. **Test keyboard input** - Verify PS/2 driver works
2. **Check AHCI detection** - Verify SSD/HDD is detected
3. **Monitor stability** - Let it run for a few minutes
4. **Test power management** - Try ACPI shutdown commands
5. **Document hardware quirks** - Note any Mac-specific issues

---

## Safety Notes

- ⚠️ **Backup your data** before creating the USB
- ⚠️ **Double-check device names** before formatting
- ⚠️ **Keep macOS bootable** - Don't touch your Mac's internal drive
- ⚠️ **Have a recovery plan** - Keep a macOS install USB handy

---

## File Structure on USB

The USB drive will have this structure:

```
/
├── EFI/
│   ├── BOOT/
│   │   └── BOOTX64.EFI    (UEFI bootloader - ~200KB)
│   └── SIS/
│       └── KERNEL.ELF     (Kernel binary - ~20MB)
```

This follows the UEFI specification for removable media boot.

---

## Technical Details

### Partition Scheme

- **Partition Table**: GPT (GUID Partition Table)
- **Partition 1**: EFI System Partition (ESP)
  - Type: `C12A7328-F81F-11D2-BA4B-00A0C93EC93B`
  - Filesystem: FAT32
  - Size: 512 MB
  - Label: SISBOOT
  - ESP flag: ON

### UEFI Boot Process

1. **Mac firmware** initializes hardware
2. **Option key** boot menu displays available devices
3. **UEFI looks** for `EFI/BOOT/BOOTX64.EFI` on the ESP
4. **Bootloader executes**, reads kernel configuration
5. **Bootloader loads** `EFI/SIS/KERNEL.ELF` into memory
6. **Bootloader exits** boot services and jumps to kernel
7. **Kernel starts** at entry point in long mode (64-bit)

### Kernel Boot Sequence

1. Early init (GDT, IDT, TSS)
2. CPU feature detection
3. Serial console init
4. Interrupt setup (PIC, APIC)
5. ACPI parsing
6. PCI enumeration
7. Driver initialization (AHCI, PS/2, etc.)
8. Idle loop

---

## Support & Debugging

If you encounter issues:

1. **Check build logs**: `./scripts/prepare_usb.sh 2>&1 | tee build.log`
2. **Verify file integrity**: Check file sizes and SHA sums
3. **Test in QEMU first**: Ensure it works in virtualization
4. **Hardware-specific issues**: Some Macs have quirks with UEFI
5. **Report issues**: Document error messages and hardware details

---

## Summary

Creating a bootable USB:

```bash
# 1. Build everything
./scripts/prepare_usb.sh

# 2. Identify USB drive
lsblk

# 3. Create bootable USB
sudo ./scripts/create_bootable_usb.sh /dev/sdX

# 4. Boot on Mac
# Hold Option (⌥) key during power-on
# Select "EFI Boot" from menu
```

Good luck testing on real hardware! 🚀
