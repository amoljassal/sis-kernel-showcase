# Kernel Logging System

## Overview

The SIS Kernel includes a comprehensive production-grade logging framework with runtime-configurable log levels. This allows you to control boot verbosity and debug output without recompiling.

## Log Levels

The logging system supports five log levels (from least to most verbose):

| Level | Value | Description | Use Case |
|-------|-------|-------------|----------|
| **ERROR** | 0 | Critical errors only | Production systems with minimal logging |
| **WARN** | 1 | Warnings and errors | Production default (recommended) |
| **INFO** | 2 | Normal operation milestones | Development and testing (default) |
| **DEBUG** | 3 | Detailed component initialization | Debugging specific issues |
| **TRACE** | 4 | Very verbose output | Deep debugging, hardware bringup |

## Default Log Level

The kernel boots with **INFO** level by default, showing major subsystem initialization milestones without overwhelming detail.

## Runtime Control

### Shell Commands

Once the kernel boots and you have a `sis>` prompt, use the `logctl` command:

```bash
# Show current log configuration
logctl status

# Set specific log level
logctl level error   # Only critical errors
logctl level warn    # Warnings and errors
logctl level info    # Normal operation (default)
logctl level debug   # Detailed debugging
logctl level trace   # Maximum verbosity

# Quick policy presets
logctl production    # Sets WARN level (minimal logging)
logctl development   # Sets DEBUG level (detailed logging)
logctl testing       # Sets TRACE level (maximum logging)

# Demo all log levels
logctl demo
```

### Examples

#### Minimal Boot Output (WARN level)
```bash
sis> logctl level warn
```
Shows only warnings and errors. Boot messages for normal initialization are suppressed.

#### Debug GIC Initialization (DEBUG level)
```bash
sis> logctl level debug
```
Shows detailed GIC, PMU, and subsystem initialization steps without overwhelming trace data.

#### Maximum Verbosity (TRACE level)
```bash
sis> logctl level trace
```
Shows every register write, memory allocation, and low-level operation.

## Boot Messages Converted to Log System

The following boot phases now use the logging framework:

### Early Init (`init/phases.rs::early_init`)
- **INFO**: UART ready, Heap ready
- **DEBUG**: Heap initialization, heap tests

### Platform Init (`init/phases.rs::platform_init`)
- **INFO**: Platform detected, PSCI ready
- **DEBUG**: PSCI initialization

### Memory Init (`init/phases.rs::memory_init`)
- **INFO**: Phase announcement, slab ready, page cache ready
- **DEBUG**: Buddy allocator init, slab init, page cache init

### Subsystem Init (`init/phases.rs::subsystem_init`)
- **INFO**: Process table, scheduler, VFS, block devices ready
- **DEBUG**: VFS mounting, filesystem initialization

### Late Init (`init/phases.rs::late_init`)
- **INFO**: GICv3 ready, PMU ready, AI agents ready, autonomy ready
- **DEBUG**: GIC initialization, interrupt setup, SMP bringup

## Code Examples

### Using the Log System in Kernel Code

```rust
// Simple logging
crate::log::error("MODULE", "Critical error occurred");
crate::log::warn("MODULE", "Warning: potential issue");
crate::log::info("MODULE", "Subsystem initialized");
crate::log::debug("MODULE", "Detailed initialization step");
crate::log::trace("MODULE", "Register write: 0xABCD");

// Structured logging with context
crate::log::info_ctx("GPIO", "Pin configured", &[("pin", 27), ("mode", 1)]);
crate::log::warn_ctx("THERMAL", "Temperature high", &[("temp", 75), ("limit", 70)]);

// Driver error logging
let error = crate::drivers::DriverError::InvalidParameter;
crate::log::log_driver_error("DRIVER", "operation", &error);
```

### Checking if a Log Level is Enabled

```rust
// Avoid expensive computation if debug logging is disabled
if crate::log::is_enabled(crate::log::LogLevel::Debug) {
    let detailed_state = expensive_debug_computation();
    crate::log::debug("MODULE", &detailed_state);
}
```

## Benefits

1. **Clean Default Boot**: INFO level shows major milestones without clutter
2. **Debug On Demand**: Switch to DEBUG/TRACE when investigating issues
3. **Production Ready**: WARN level minimizes log overhead in production
4. **No Recompilation**: Change log levels at runtime via shell
5. **Zero Cost**: Disabled log levels compile to no-ops (when using feature gates)

## Implementation Details

- **Location**: `crates/kernel/src/log.rs`
- **Shell Integration**: `crates/kernel/src/shell/logctl_helpers.rs`
- **Atomic Global State**: Lock-free runtime configuration
- **UART Backend**: Direct UART writes for minimal overhead

## Future Enhancements

Potential improvements for the logging system:

1. **Per-Module Log Levels**: Different verbosity for different subsystems
2. **Log Buffering**: Circular buffer for post-mortem debugging
3. **Network Logging**: Send logs to remote syslog server
4. **Timestamp Precision**: Microsecond-resolution timestamps
5. **Compile-Time Filtering**: Feature flags to remove log levels entirely

## Migration from Direct UART Prints

Old style:
```rust
crate::uart_print(b"SUBSYSTEM: INIT\n");
```

New style:
```rust
crate::log::info("SUBSYSTEM", "Initializing");
crate::log::debug("SUBSYSTEM", "Detailed initialization step");
```

The new style provides:
- Runtime control over verbosity
- Consistent formatting with module tags
- Structured logging capabilities
- Production-ready log level filtering
