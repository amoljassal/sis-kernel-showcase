# UART Keyboard Input Issue

## Problem Summary

The SIS kernel boots successfully and reaches the shell prompt, but keyboard input is not being received by the UART driver. The shell hangs in `uart::read_byte_blocking()` waiting for input that never arrives.

## Symptoms

1. Kernel boots completely, prints shell banner and `sis>` prompt
2. Shell enters polling loop waiting for keyboard input
3. Typing on the keyboard produces no response
4. UART registers show correct initialization:
   - CR = 0x00000301 (UARTEN=1, TXE=1, RXE=1) ✅
   - FR = 0x00000090 (RXFE=1 = RX FIFO empty)
5. UART TX works perfectly (all boot messages display correctly)

## Environment

- **Platform**: macOS (Darwin 25.1.0)
- **QEMU**: qemu-system-aarch64 with EDK2 UEFI firmware
- **QEMU Args**: `-display none -serial stdio -monitor none`
- **Firmware**: edk2-aarch64-code.fd (stable202408)
- **Kernel**: SIS kernel (aarch64-unknown-none, debug build)

## Investigation Timeline

### 1. Initial Hypothesis: QEMU Serial Multiplexing

**Theory**: Using `-nographic` multiplexes console with HMP monitor, causing input issues.

**Fix Applied**: Changed from `-nographic` to `-display none -serial stdio -monitor none`

**Result**: ❌ No change - keyboard input still not received

### 2. UART Initialization Check

**Verification**: Added debug output to dump UART registers at shell start:
```
CR=0x00000301 (UARTEN=1, TXE=1, RXE=1) ✅
FR=0x00000090 (RXFE=1 indicates RX FIFO empty)
```

**Conclusion**: UART hardware is correctly initialized and RX is enabled

### 3. Shell IRQ Masking Check

**Verification**: Confirmed IRQs are unmasked after banner print using `daifclr #2`

**Conclusion**: IRQs are enabled, not blocking UART RX

### 4. Polling Mode Verification

**Note**: The UART driver uses polling mode (no interrupts), so should work regardless of IRQ configuration

## Root Cause Analysis

### Likely Causes (in order of probability):

1. **UEFI Firmware Serial Capture**
   - UEFI firmware initializes PL011 UART and claims stdio
   - After ExitBootServices(), QEMU may not properly reconnect stdin to the UART
   - This is a known issue with some UEFI+QEMU configurations

2. **macOS Terminal Mode**
   - macOS Terminal.app may be in cooked mode, buffering input until Enter
   - QEMU's `-serial stdio` should handle terminal mode, but may fail on macOS

3. **QEMU Character Device Backend**
   - QEMU's stdio chardev may have issues with UEFI firmware handoff
   - The serial device might need explicit reconfiguration after UEFI exits

4. **PL011 UART FIFO Issue**
   - FIFO might be disabled or misconfigured
   - FIFO enable bit (LCRH_FEN) is set correctly in code

### Evidence Against Common Theories:

- ❌ **Not an IRQ issue** - UART TX works, IRQs are enabled, polling mode doesn't need interrupts
- ❌ **Not a baud rate issue** - TX works perfectly, same UART config for RX/TX
- ❌ **Not a hardware issue** - QEMU virt machine PL011 is well-tested
- ❌ **Not a platform detection issue** - FDT parsing works, correct UART base address (0x09000000)

## Workarounds and Solutions

### Solution 1: Try Different Terminal Emulator

Instead of macOS Terminal.app, try:
```bash
# Using socat for raw terminal mode
brew install socat
socat UNIX-CONNECT:/tmp/sis-serial.sock -,raw,echo=0

# Then in QEMU args, use:
-serial unix:/tmp/sis-serial.sock,server,nowait
```

### Solution 2: Use QEMU Monitor to Inject Input

```bash
# In another terminal:
echo "help" | nc -U /tmp/qemu-monitor.sock

# QEMU args:
-monitor unix:/tmp/qemu-monitor.sock,server,nowait
```

### Solution 3: Bypass UEFI - Direct Kernel Boot

Skip UEFI firmware entirely and boot kernel directly:
```bash
qemu-system-aarch64 \
  -M virt \
  -cpu cortex-a72 \
  -m 512M \
  -kernel target/aarch64-unknown-none/debug/sis_kernel \
  -serial stdio \
  -display none
```

### Solution 4: Enable UART RX Interrupts

Instead of polling, use interrupt-driven RX:

```rust
// In uart::init()
pub unsafe fn init(&mut self) {
    // ... existing initialization ...

    // Enable RX interrupt (RXIM bit in IMSC register)
    ptr::write_volatile(reg_imsc() as *mut u32, 1 << 4);

    self.initialized = true;
}
```

Then register UART interrupt handler in GIC (IRQ 33 for QEMU virt PL011).

### Solution 5: Alternative Input Method - VirtIO Console

Use VirtIO console device instead of PL011 UART:
```bash
# QEMU args:
-device virtio-serial-device \
-chardev stdio,id=con0 \
-device virtconsole,chardev=con0
```

## Current Workaround

Added polling indicator (dots printed every 10M iterations) to prove shell is alive and polling. This helps debug whether:
- Shell is actually polling (dots appear)
- Keyboard input eventually arrives (after terminal mode adjustment)

## Next Steps

1. Test with `socat` raw terminal (Solution 1)
2. Try direct kernel boot without UEFI (Solution 3)
3. Implement interrupt-driven UART RX (Solution 4)
4. Add VirtIO console support (Solution 5)

## References

- [QEMU Serial Configuration](https://www.qemu.org/docs/master/system/invocation.html#hxtool-5)
- [ARM PL011 TRM](https://developer.arm.com/documentation/ddi0183/latest/)
- [UEFI Serial Console Issues](https://github.com/tianocore/edk2/issues)

## Files Modified

- `crates/kernel/src/main.rs` - Added MMU bounds checking (fixes panic)
- `crates/kernel/src/uart.rs` - Added polling indicator dots
- `crates/kernel/src/shell.rs` - Removed debug messages
- `scripts/uefi_run.sh` - Changed to `-display none -serial stdio -monitor none`
