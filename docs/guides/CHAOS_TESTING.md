# Chaos Engineering for SIS Kernel

**Phase 3.1 - Production Readiness Plan**

## Overview

The SIS Kernel includes a chaos engineering framework for testing system resilience and graceful failure handling. Chaos injection allows controlled failure scenarios to verify that the system degrades gracefully rather than panicking or corrupting data.

## Features

- **Compile-time gating**: Chaos features disabled by default
- **Runtime control**: Enable/disable via shell commands
- **Multiple failure modes**: Disk, network, memory, I/O delays
- **Configurable failure rate**: 0-100% injection probability
- **Statistics tracking**: Count injected failures per type
- **Zero overhead when disabled**: Lock-free atomic operations

## Building with Chaos Support

```bash
# Enable chaos feature during build
SIS_FEATURES="llm,crypto-real,chaos" ./scripts/uefi_run.sh

# Or with cargo directly
cargo build --features chaos
```

## Shell Commands

### Enable Chaos Mode

```bash
# Set chaos mode
chaos mode <mode>

# Available modes:
  none            # Disable (default)
  disk_full       # Inject ENOSPC (disk full) errors
  disk_fail       # Inject EIO (I/O error)
  network_fail    # Inject ENETDOWN (network down)
  memory_pressure # Inject ENOMEM (allocation failure)
  random_panic    # Inject random panics
  slow_io         # Inject I/O delays
```

### Configure Failure Rate

```bash
# Set failure injection probability (0-100%)
chaos rate 30    # 30% of operations will fail

# Check current rate
chaos rate
```

### View Statistics

```bash
# Show chaos injection statistics
chaos stats

# Output:
#   Mode:              disk_full
#   Failure rate:      30%
#   Disk full:         42
#   Disk failures:     0
#   Network failures:  0
#   Alloc failures:    0
```

### Reset Statistics

```bash
# Clear all counters
chaos reset
```

## Usage Examples

### Test Disk Full Scenario

```bash
# Start kernel with chaos enabled
SIS_FEATURES="llm,crypto-real,chaos" ./scripts/uefi_run.sh

# In shell:
chaos mode disk_full
chaos rate 50
touch /test.txt    # May fail with ENOSPC
ls /               # Should handle gracefully
chaos mode none    # Disable
```

### Test Network Resilience

```bash
chaos mode network_fail
chaos rate 20
netstat            # May fail with ENETDOWN
# Network code should handle gracefully
chaos mode none
```

### Test Memory Pressure

```bash
chaos mode memory_pressure
chaos rate 30
# Allocations may fail, but kernel shouldn't panic
memstats
chaos mode none
```

## Automated Testing

### Run All Chaos Tests

```bash
./scripts/run_chaos_tests.sh
```

This runs all chaos test scenarios:
- `test_disk_full.sh` - Disk full handling
- `test_network_fail.sh` - Network failure handling
- `test_memory_pressure.sh` - OOM handling
- `test_slow_io.sh` - I/O delay tolerance

### Expected Behavior

✅ **PASS**: System handles failures gracefully:
- Returns appropriate error codes (ENOSPC, EIO, ENETDOWN, ENOMEM)
- No kernel panics
- System remains responsive
- Clean error messages

❌ **FAIL**: System behavior that indicates issues:
- Kernel panic
- Deadlock or hang
- Data corruption
- Missing error handling

## Implementation Details

### Chaos Module (`crates/kernel/src/chaos.rs`)

```rust
// Feature-gated at compile time
#[cfg(feature = "chaos")]
pub mod chaos;

// Runtime mode control
pub fn set_mode(mode: ChaosMode);
pub fn set_failure_rate(rate: u32);

// Injection points
pub fn should_fail_disk_full() -> bool;
pub fn should_fail_disk_io() -> bool;
pub fn should_fail_network() -> bool;
pub fn should_fail_allocation() -> bool;
```

### Adding Injection Points

To add chaos injection to your code:

```rust
use crate::chaos;

pub fn write_block(&self, block: u64, data: &[u8]) -> Result<()> {
    // Inject failure if chaos mode enabled
    #[cfg(feature = "chaos")]
    if chaos::should_fail_disk_io() {
        chaos::record_disk_fail();
        return Err(Errno::EIO);
    }

    // Normal operation
    self.device.write(block, data)
}
```

### Performance Impact

- **Disabled (default)**: Zero overhead - code removed at compile time
- **Enabled, mode=none**: Minimal overhead - single atomic load
- **Enabled, active**: Atomic load + PRNG + conditional logic

## Integration with CI/CD

Add chaos tests to your CI pipeline:

```yaml
# .github/workflows/chaos-test.yml
- name: Run chaos tests
  run: |
    SIS_FEATURES="llm,crypto-real,chaos" ./scripts/run_chaos_tests.sh
```

## Best Practices

1. **Test all failure paths**: Every error return should be tested
2. **Verify error messages**: Ensure clear, actionable errors
3. **Check resource cleanup**: No leaks on failure paths
4. **Test combinations**: Multiple concurrent failures
5. **Measure recovery time**: How long to recover from failure?

## Limitations

- **No real hardware faults**: Tests software-level error handling only
- **Deterministic PRNG**: Not cryptographically secure (intentional)
- **No timing-based failures**: Race conditions not tested
- **Single-threaded chaos**: All cores see same chaos state

## Future Enhancements

- [ ] Per-subsystem failure rates
- [ ] Time-based failure schedules
- [ ] Correlated failure injection
- [ ] Chaos monkey for automated long-running tests
- [ ] Failure injection via external controller

## Troubleshooting

### Chaos command not found

**Problem**: `chaos: command not found`

**Solution**: Rebuild with chaos feature:
```bash
SIS_FEATURES="llm,crypto-real,chaos" ./scripts/uefi_run.sh
```

### No failures injected

**Problem**: Chaos mode set but no failures occur

**Causes**:
1. Failure rate too low (increase with `chaos rate 100`)
2. No injection points in tested code path
3. Feature not enabled at compile time

### Too many failures

**Problem**: System unusable with chaos enabled

**Solution**: Lower failure rate:
```bash
chaos rate 10  # Reduce to 10%
```

## References

- [Principles of Chaos Engineering](https://principlesofchaos.org/)
- [Netflix Chaos Monkey](https://netflix.github.io/chaosmonkey/)
- [Production Readiness Plan](./plans/PRODUCTION-READINESS-PLAN.md)

---

**See Also**:
- [Testing Guide](./TESTING.md)
- [Metrics Export](./METRICS.md)
- [Build Documentation](./BUILD.md)
