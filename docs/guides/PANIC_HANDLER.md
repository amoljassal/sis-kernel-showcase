# Enhanced Panic Handler

**Phase 3.2 - Production Readiness Plan**

## Overview

The SIS Kernel features an enhanced panic handler that provides comprehensive diagnostic information when a kernel panic occurs. This assists with debugging and forensic analysis in both development and production environments.

## Features

### 1. Register Dump

Full register state capture at panic time:

**AArch64 (ARM64)**:
- All general-purpose registers (x0-x30)
- Stack pointer (sp)
- Program counter (pc)
- Frame pointer (x29)
- Link register (x30)

**x86_64**:
- General-purpose registers (rax, rbx, rcx, rdx, rsi, rdi, rbp, rsp)
- Extended registers (r8-r15)
- Instruction pointer (rip)

**RISC-V**:
- Placeholder (implementation pending)

### 2. System State Information

- **Uptime**: System runtime in seconds and milliseconds
- **Heap Statistics**:
  - Current allocation (MB)
  - Peak allocation (MB)
  - Allocation/deallocation counts
  - Active allocations
  - Failure count
- **Build Information**: Git commit, branch, build timestamp

### 3. Stack Trace

Basic stack unwinding using frame pointers (when available):
- Displays up to 10 stack frames
- Shows return addresses for each frame
- Requires `-C force-frame-pointers=yes` RUSTFLAGS

### 4. Debugging Guidance

Automatic suggestions for:
- Common panic causes (null pointer, bounds check, heap exhaustion, etc.)
- Steps to take for debugging
- Information to examine in register dump

### 5. Structured Logging

Panic events emitted in JSON format (with `structured-logging` feature):
```json
{
  "ts": 1234567890,
  "subsystem": "PANIC",
  "status": "kernel_panic",
  "level": "FATAL",
  "location": "kernel/src/foo.rs:42:5",
  "message": "assertion failed: x > 0"
}
```

### 6. Recursive Panic Protection

- Detects and handles recursive panics gracefully
- Minimal output for recursive panics to prevent infinite loops
- Tracks panic count for debugging

### 7. Crash Dump Support

With the `crash-dump` feature flag (optional):
- Crash state can be written to disk
- Includes all diagnostic information
- Enables post-mortem analysis

## Usage

### Normal Operation

The panic handler is invoked automatically when:
- A `panic!()` macro is called
- An assertion fails
- An unrecoverable error occurs

Example output:
```
================================================================================
!!!                        KERNEL PANIC                                      !!!
================================================================================

PANIC INFORMATION:
------------------
  Location: kernel/src/mm/heap.rs:156:9
  Message:  allocation error: out of memory

REGISTER DUMP:
--------------
  x0:  0000000000000000  x1:  0000000040080000  x2:  0000000000001000  x3:  0000000000000001
  x4:  0000000040100000  x5:  0000000000000020  x6:  0000000000000000  x7:  0000000000000000
  x8:  0000000000000001  x9:  0000000000000000  x10: 0000000000000000  x11: 0000000000000000
  x12: 0000000000000000  x13: 0000000000000000  x14: 0000000000000000  x15: 0000000000000000
  x16: 0000000000000000  x17: 0000000000000000  x18: 0000000000000000  x19: 0000000040200000
  x20: 0000000000000001  x21: 0000000000000000  x22: 0000000000000000  x23: 0000000000000000
  x24: 0000000000000000  x25: 0000000000000000  x26: 0000000000000000  x27: 0000000000000000
  x28: 0000000000000000  x29: 0000000040080000  x30: 0000000040012345
  sp:  0000000040080000  pc:  0000000040012340

SYSTEM STATE:
-------------
  Uptime:       125 seconds (125234 ms)
  Heap usage:   7 MB current, 8 MB peak
                  Allocations: 1024 allocs, 1020 deallocs, 4 active
                  Failures:    1
  Version:      SIS Kernel 7be18b2 (main) built 2025-11-07

RECENT LOGS:
------------
  [Log buffer not yet implemented]

STACK TRACE:
------------
  [Stack unwinding requires frame pointers]
  [Build with RUSTFLAGS="-C force-frame-pointers=yes"]
  #0: 0000000040012345
  #1: 0000000040015678
  #2: 0000000040018abc

DEBUGGING STEPS:
----------------
  1. Check panic location and message above
  2. Examine register values for invalid pointers
  3. Check heap usage for memory exhaustion
  4. Review recent logs for error patterns
  5. If stack trace available, identify call chain
  6. Check system uptime for timing-related issues

COMMON CAUSES:
--------------
  - Null or invalid pointer dereference
  - Array out of bounds access
  - Heap corruption or exhaustion
  - Stack overflow
  - Assertion failure
  - Unhandled error condition

================================================================================
System halted.
================================================================================
```

### Enable Stack Traces

For better stack traces, build with frame pointers:

```bash
export RUSTFLAGS="-C force-frame-pointers=yes"
./scripts/uefi_run.sh
```

### Enable Structured Logging

Build with the `structured-logging` feature:

```bash
SIS_FEATURES="llm,crypto-real,structured-logging" ./scripts/uefi_run.sh
```

### Enable Crash Dumps

Build with the `crash-dump` feature:

```bash
SIS_FEATURES="llm,crypto-real,crash-dump" ./scripts/uefi_run.sh
```

## Testing

### Trigger Test Panic

You can test the panic handler using a shell command:

```bash
# In kernel shell
panic_test
```

Or programmatically:

```rust
#[cfg(test)]
fn test_panic_handler() {
    panic!("Test panic message");
}
```

### Verify Output

Check that the panic handler:
1. ✅ Prints panic header and location
2. ✅ Shows register dump
3. ✅ Displays system state
4. ✅ Provides debugging guidance
5. ✅ Emits structured log (if feature enabled)
6. ✅ Halts system gracefully

## Implementation Details

### Architecture Support

| Architecture | Register Dump | Stack Trace | Interrupt Disable |
|--------------|---------------|-------------|-------------------|
| AArch64      | ✅ Full       | ✅ Basic    | ✅ DAIF          |
| x86_64       | ✅ Full       | ✅ Basic    | ✅ CLI           |
| RISC-V       | ⏸️ Pending    | ⏸️ Pending  | ✅ CSR           |

### Code Location

- **Implementation**: `crates/kernel/src/lib/panic.rs`
- **Integration**: `crates/kernel/src/main.rs` (panic_handler)
- **Module**: `crates/kernel/src/lib/mod.rs`

### Panic Flow

```
1. Panic occurs
   ↓
2. Check for recursive panic
   ↓
3. Disable interrupts
   ↓
4. Print panic header
   ↓
5. Print location and message
   ↓
6. Dump registers (arch-specific)
   ↓
7. Print system state
   ↓
8. Print recent logs (if available)
   ↓
9. Print stack trace (if available)
   ↓
10. Print debugging guidance
    ↓
11. Write crash dump (if feature enabled)
    ↓
12. Emit structured log
    ↓
13. Halt system
```

### Safety Considerations

1. **Minimal Allocation**: Uses UART directly to avoid heap allocation during panic
2. **Recursive Panic Protection**: Detects and handles recursive panics
3. **Interrupt Safety**: Disables interrupts before panic handling
4. **Atomic Operations**: Uses lock-free atomics for panic state

### Performance Impact

- **Normal Operation**: Zero overhead (panic handler not in hot path)
- **During Panic**: ~10-50ms for full diagnostic output
- **Memory**: <1KB static data (panic counters, flags)

## Future Enhancements

### Phase 1: Recent Logs (TODO)

Implement circular log buffer to capture last N log entries:

```rust
// crates/kernel/src/lib/panic.rs
static LOG_BUFFER: Mutex<CircularBuffer<LogEntry, 100>> = ...;
```

### Phase 2: Crash Dump Writing (TODO)

Write crash state to virtio-blk device:

```rust
#[cfg(feature = "crash-dump")]
fn write_crash_dump(info: &PanicInfo) {
    if let Some(blk) = get_block_device(0) {
        let dump = serialize_crash_state(info);
        blk.write(CRASH_DUMP_BLOCK, dump.as_bytes());
    }
}
```

### Phase 3: Symbol Resolution

Add symbol table support for better stack traces:

```rust
// Resolve addresses to function names
fn resolve_symbol(addr: u64) -> &'static str {
    SYMBOL_TABLE.lookup(addr)
        .map(|s| s.name)
        .unwrap_or("<unknown>")
}
```

### Phase 4: Crash Analytics

Aggregate crash data for pattern detection:

```rust
// Track crash patterns
struct CrashAnalytics {
    locations: HashMap<&'static str, u32>,
    types: HashMap<&'static str, u32>,
}
```

## Troubleshooting

### No Stack Trace

**Problem**: Stack trace shows "[Stack unwinding requires frame pointers]"

**Solution**: Rebuild with frame pointers:
```bash
export RUSTFLAGS="-C force-frame-pointers=yes"
```

### Invalid Register Values

**Problem**: Register dump shows unexpected values

**Causes**:
1. Compiler optimizations moved data
2. Register clobber by panic handling code
3. Stack corruption before panic

**Solution**: Use `volatile` operations and examine surrounding code

### Missing System State

**Problem**: Heap stats or uptime missing

**Causes**:
1. Heap not initialized yet (early boot panic)
2. Time subsystem not available

**Solution**: Check boot sequence and initialization order

### Recursive Panic

**Problem**: "RECURSIVE PANIC" message appears

**Causes**:
1. Panic handler itself triggered a panic
2. Logging/allocation during panic
3. Invalid memory access during diagnostics

**Solution**: Review panic handler code for potential failures

## References

- [Production Readiness Plan](./plans/PRODUCTION-READINESS-PLAN.md) - Phase 3.2
- [Chaos Testing](./CHAOS_TESTING.md) - Related reliability features
- [Build Documentation](./BUILD.md) - Compilation flags
- [Testing Guide](./TESTING.md) - Test procedures

## Best Practices

### For Developers

1. **Add Context**: Include meaningful panic messages
   ```rust
   panic!("Invalid state: expected {} but got {}", expected, actual);
   ```

2. **Use Assertions**: Prefer assertions over panics for invariant checks
   ```rust
   debug_assert!(x > 0, "x must be positive");
   ```

3. **Handle Errors**: Don't panic for recoverable errors
   ```rust
   match result {
       Ok(val) => val,
       Err(e) => return Err(e),  // Don't panic!
   }
   ```

### For Operations

1. **Capture Full Output**: Save entire panic output for analysis
2. **Check Build Info**: Note exact commit and features
3. **Analyze Patterns**: Look for recurring panic locations
4. **Test Fixes**: Use chaos testing to verify fixes

---

**See Also**:
- [Panic Handler Implementation](../crates/kernel/src/lib/panic.rs)
- [Main Integration](../crates/kernel/src/main.rs)
- [Chaos Testing Guide](./CHAOS_TESTING.md)
