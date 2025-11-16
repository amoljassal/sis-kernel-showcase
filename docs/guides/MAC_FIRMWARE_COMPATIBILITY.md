# MacBook Pro Mid 2012 UEFI Firmware Compatibility Issues

This document describes the compatibility issues encountered when attempting to boot the SIS kernel on a MacBook Pro Mid 2012 (Intel Core i5, 16GB RAM, 480GB SSD) running Ubuntu 24.04.

## Hardware Specifications

- **Model**: MacBook Pro 9,2 (Mid 2012)
- **CPU**: Intel Core i5-3210M (Ivy Bridge)
- **RAM**: 16GB
- **Storage**: 480GB SSD
- **OS**: Ubuntu 24.04 LTS (not macOS)
- **Firmware**: Apple EFI (UEFI-compatible but with Apple-specific behavior)

## Summary

The SIS kernel successfully builds for x86_64 with full driver support (AHCI, PS/2, GOP framebuffer), but encounters boot issues on MacBook Pro firmware. The bootloader appears to hang or crash when executed by Mac firmware, showing only a blank screen.

## What Works ✅

1. **Kernel Build**: x86_64 kernel builds successfully with all features
2. **Driver Implementation**: AHCI/SATA, PS/2 keyboard, GOP framebuffer all implemented
3. **USB Creation**: Mac-compatible bootable USB created with hybrid MBR/GPT
4. **File Placement**: Bootloader and kernel correctly placed on EFI partition
5. **Boot Entry Creation**: efibootmgr successfully creates UEFI boot entries
6. **QEMU Boot**: Kernel boots successfully in QEMU with OVMF (x86_64 UEFI firmware)

## What Doesn't Work ❌

### Issue 1: USB Boot Failure
**Symptom**: Mac firmware freezes at boot selection screen when USB is selected

**What Happens**:
1. Power on Mac while holding Option (⌥) key
2. Startup Manager displays boot options
3. USB drive appears as "EFI Boot" or similar
4. Selecting the USB causes the selection screen to freeze
5. No transition to black screen, no boot attempt, just frozen UI

**Attempted Fixes**:
- ✅ Created GPT partition table (not MBR)
- ✅ Set partition type to EFI System Partition (type EF00)
- ✅ Created hybrid MBR/GPT for Mac compatibility (using sgdisk -h)
- ✅ Formatted as FAT32 with Mac-friendly options
- ✅ Tried multiple USB ports
- ✅ Reset NVRAM (Cmd+Option+P+R)
- ❌ None resolved the freeze

**Partition Details**:
```
Partition Table: GPT + Hybrid MBR
Partition 1:
  Start sector: 2048
  Size: 512MB
  Type: EF00 (EFI System Partition)
  Filesystem: FAT32 (mkfs.vfat -F 32 -n "EFI" -s 2)
  Files:
    /EFI/BOOT/BOOTX64.EFI (45KB)
    /EFI/SIS/KERNEL.ELF (335KB)
```

**Analysis**:
The Mac firmware recognizes the USB as a bootable device (shows in Startup Manager) but refuses to execute the bootloader. This suggests the firmware is validating something about the bootloader before execution and rejecting it.

### Issue 2: Internal EFI Boot Shows Blank Screen
**Symptom**: Booting from internal EFI partition shows blank screen, no output

**What Happens**:
1. Copied BOOTX64.EFI and KERNEL.ELF to `/boot/efi/EFI/SIS/`
2. Created efibootmgr entry: `Boot0001* SIS Kernel Direct`
3. Set as first boot option in boot order
4. On boot, screen goes blank (no output, no error message)
5. System appears to hang (no disk activity, no progress)

**Boot Entry Details**:
```bash
Boot0001* SIS Kernel Direct
HD(1,GPT,cf7ccd87-5c49-451d-8161-56cb3b64a3b3,0x800,0x219800)/File(\EFI\SIS\BOOTX64.EFI)
```

**Analysis**:
The firmware successfully locates and attempts to execute the bootloader (unlike USB boot), but the bootloader crashes or hangs during execution. The blank screen suggests:
- Bootloader starts executing
- Encounters an error or unsupported UEFI call
- Hangs or crashes without displaying an error

### Issue 3: GRUB Chainloading Failures
**Symptom**: GRUB cannot find or chainload the bootloader

**Attempted GRUB Configurations**:

1. **UUID Search (failed)**:
   ```
   search --no-floppy --fs-uuid --set=root CF7CCD87-5C49-451D-8161-56CB3B64A3B3
   chainloader /EFI/SIS/BOOTX64.EFI
   ```
   Error: `no such device: CF7CCD87-5C49-451D-8161-56CB3B64A3B3`

2. **Direct Device (failed)**:
   ```
   set root='hd0,gpt1'
   chainloader /EFI/SIS/BOOTX64.EFI
   ```
   Error: `disk 'hd0,gpt1' not found`

3. **Root Variable (failed)**:
   ```
   chainloader ($root)/EFI/SIS/BOOTX64.EFI
   ```
   Error: `file '/EFI/SIS/BOOTX64.EFI' not found`

**Analysis**:
GRUB's file path resolution appears to have issues finding files on the EFI partition, even though the files physically exist at `/boot/efi/EFI/SIS/`. This might be a GRUB configuration issue specific to the Mac's EFI partition layout.

## Bootloader Analysis

### File Structure Validation
```bash
$ file /boot/efi/EFI/SIS/BOOTX64.EFI
PE32+ executable (EFI application) x86-64, for MS Windows, 4 sections

$ objdump -p /boot/efi/EFI/SIS/BOOTX64.EFI | head -30
Characteristics 0x22
    executable
    large address aware
Magic            020b    (PE32+)
Subsystem        0000000a    (EFI application)
ImageBase        0000000140000000
AddressOfEntryPoint    00000000000042b0
```

**Validation**: The bootloader is a properly formatted PE32+ UEFI application with correct subsystem type (0xa = EFI application).

### Size Comparison
```
SIS BOOTX64.EFI:     45 KB
Ubuntu shimx64.efi:  945 KB (21x larger)
Ubuntu grubx64.efi:  2.6 MB (59x larger)
```

**Analysis**: Our bootloader is significantly smaller, which is expected (it's a minimal loader), but the size difference suggests it may lack features or error handling that larger bootloaders have.

### Dependencies
```bash
$ objdump -p /boot/efi/EFI/SIS/BOOTX64.EFI | grep -i "dll\|import"
Entry 1 0000000000000000 00000000 Import Directory [parts of .idata]
```

**Analysis**: No DLL imports (normal for UEFI applications which use UEFI Boot Services directly).

## Suspected Root Causes

### Hypothesis 1: Mac Firmware UEFI Service Incompatibility
**Likelihood**: High

Mac firmware implements UEFI but has Apple-specific quirks and limitations. The bootloader may be:
- Calling UEFI services in a way Mac firmware doesn't support
- Using features that work in OVMF but not in Apple's EFI implementation
- Missing error handling for Mac-specific return codes

**Evidence**:
- Bootloader works in QEMU/OVMF
- Mac firmware shows blank screen (starts executing but crashes)
- No error messages (suggests crash, not graceful error)

### Hypothesis 2: Missing Mac-Specific Boot Protocols
**Likelihood**: Medium

Macs may expect certain protocols or data structures that standard UEFI doesn't require:
- Apple's own GOP (Graphics Output Protocol) extensions
- Specific memory allocation patterns
- Boot service call ordering requirements

**Evidence**:
- Mac firmware is more restrictive than standard UEFI
- Commercial bootloaders (GRUB, rEFInd) have Mac-specific code paths

### Hypothesis 3: Bootloader Code Path Issues
**Likelihood**: Medium

The bootloader may have a code path that:
- Enters an infinite loop
- Dereferences a null pointer
- Allocates memory incorrectly
- Fails to initialize required UEFI services

**Evidence**:
- Blank screen suggests execution started but didn't complete
- No output means GOP initialization might be failing
- Similar symptoms to infinite loops or crashes

### Hypothesis 4: Firmware Security / Code Signing
**Likelihood**: Low

Mac firmware might be rejecting unsigned UEFI applications.

**Evidence Against**:
- MacBook Pro 2012 predates Apple's T2 security chip
- Firmware shows the USB in boot menu (recognizes it)
- If signature was the issue, firmware would show an error

## Tested Workarounds

### ✅ Successful (Partial)
1. **QEMU Validation**: Kernel boots successfully in QEMU with OVMF firmware
2. **efibootmgr Entry**: Successfully creates boot entry in NVRAM
3. **File Placement**: Files correctly placed and accessible on EFI partition
4. **USB Creation**: Proper GPT/MBR hybrid partitioning for Macs

### ❌ Unsuccessful
1. **USB Boot**: Multiple partition schemes, all freeze at selection
2. **Internal EFI Boot**: Blank screen, no output
3. **GRUB Chainload**: File path resolution failures
4. **NVRAM Reset**: Doesn't change boot behavior

## Recommended Next Steps

### Short Term (Debugging)

1. **Add Serial Debug Output to Bootloader**
   - Add UART serial output before GOP initialization
   - Log each major step: entry point, UEFI services, GOP, file loading
   - Use serial console to see where it's failing

2. **Test Minimal Bootloader**
   - Create absolute minimal UEFI app that just prints "Hello" via serial
   - Test if Mac firmware can execute ANY custom UEFI code
   - Incrementally add features until it breaks

3. **Compare with Working Bootloaders**
   - Study rEFInd source code for Mac-specific workarounds
   - Check how GRUB handles Mac firmware differences
   - Identify Mac-specific UEFI service call patterns

4. **Test on Standard UEFI Hardware**
   - Test on non-Mac x86_64 UEFI system
   - Isolate if issue is Mac-specific or general bootloader bug
   - Validate bootloader works on standard UEFI

### Medium Term (Alternative Approaches)

1. **Use rEFInd as Primary Bootloader**
   - Install rEFInd (known to work on Macs)
   - Configure rEFInd to chainload our kernel
   - Bypass our custom UEFI bootloader entirely

2. **Use GRUB with Manual Boot**
   - Add manual GRUB entry to load kernel as ELF
   - Use GRUB's multiboot2 or Linux boot protocol
   - Avoid UEFI bootloader altogether

3. **Create Mac-Specific Bootloader**
   - Fork our bootloader
   - Add Mac-specific error handling
   - Implement conservative UEFI service usage

### Long Term (Proper Solution)

1. **Implement Mac Firmware Compatibility Layer**
   - Add Mac detection in bootloader
   - Use Mac-specific code paths for GOP, file I/O
   - Handle Mac firmware quirks gracefully

2. **Support Multiple Bootloader Backends**
   - Support both custom UEFI bootloader and GRUB
   - Allow user to choose bootloader at build time
   - Provide Mac-specific bootloader variant

3. **Comprehensive Mac Testing**
   - Test on multiple Mac models (2012-2020)
   - Document firmware differences across generations
   - Build Mac firmware compatibility matrix

## Comparison: QEMU vs Real Mac

| Feature | QEMU (OVMF) | MacBook Pro 2012 |
|---------|-------------|------------------|
| **USB Boot** | ✅ Works | ❌ Freezes at selection |
| **Internal EFI Boot** | ✅ Would work | ❌ Blank screen |
| **UEFI Services** | Standard UEFI | Apple-modified UEFI |
| **Error Messages** | Shows errors | Silent failure |
| **Boot Entry Creation** | ✅ Works | ✅ Works (but boot fails) |
| **Bootloader Execution** | ✅ Successful | ❌ Hangs/crashes |

## Technical Details

### Bootloader Source Location
```
crates/uefi-boot/src/main.rs
```

### Build Command
```bash
cargo build --release --target x86_64-unknown-uefi \
  --manifest-path crates/uefi-boot/Cargo.toml
```

### Output Location
```
crates/uefi-boot/target/x86_64-unknown-uefi/release/uefi-boot.efi
```

### USB Creation Script
```
scripts/create_bootable_usb_mac.sh
```

### Files Required on EFI Partition
```
/EFI/BOOT/BOOTX64.EFI    # UEFI bootloader
/EFI/SIS/KERNEL.ELF      # Kernel binary
```

## References

### Successful Boot on QEMU
```bash
qemu-system-x86_64 \
    -bios /usr/share/ovmf/OVMF.fd \
    -drive file=/dev/sdb,format=raw,if=ide \
    -m 512M \
    -serial stdio
```

### Mac Boot Sequence
1. Power on + Option (⌥) → Startup Manager
2. Select boot device → Firmware loads bootloader
3. **[FAILS HERE]** → Bootloader should initialize UEFI services
4. Bootloader loads kernel → Kernel initializes

### Ubuntu Boot Sequence (Working)
1. Firmware loads shimx64.efi (945KB)
2. Shim loads grubx64.efi (2.6MB)
3. GRUB loads Linux kernel
4. Kernel boots successfully

## Lessons Learned

1. **Mac Firmware is Not Standard UEFI**
   - Works differently than OVMF/standard UEFI
   - Requires Mac-specific testing and code paths
   - Cannot assume OVMF compatibility = Mac compatibility

2. **Bootloader Size Matters**
   - Minimal bootloaders may lack necessary error handling
   - Larger bootloaders (GRUB, rEFInd) handle edge cases
   - More robust error handling = better compatibility

3. **Silent Failures are Common**
   - Mac firmware doesn't always show error messages
   - Serial output is essential for debugging
   - Blank screen doesn't mean "nothing happened"

4. **USB Boot is More Restrictive**
   - Mac firmware is pickier about USB boot
   - Internal EFI boot has different code path
   - USB boot may have additional validation

5. **Multiple Boot Methods Needed**
   - USB boot, internal boot, GRUB chainload all useful
   - Different methods fail in different ways
   - Having alternatives helps isolate issues

## Conclusion

The SIS kernel is fully functional on x86_64 when booted via QEMU/OVMF, with working AHCI, PS/2, and GOP framebuffer drivers. However, the custom UEFI bootloader has compatibility issues with MacBook Pro 2012 firmware, causing hangs or crashes during bootloader execution.

The issue is **not with the kernel itself**, but with the **UEFI bootloader's interaction with Mac firmware**. The kernel can likely boot successfully on Mac hardware if loaded via an alternative bootloader (rEFInd, GRUB) or if the custom bootloader is modified to handle Mac firmware quirks.

**Immediate Recommendation**: Use rEFInd or GRUB to boot the SIS kernel on Mac hardware while debugging the custom bootloader's Mac compatibility issues.

---

**Document Version**: 1.0
**Last Updated**: 2025-11-16
**Tested On**: MacBook Pro 9,2 (Mid 2012), Ubuntu 24.04 LTS
**Status**: Active Investigation
