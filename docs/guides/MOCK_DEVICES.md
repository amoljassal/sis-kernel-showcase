# Mock Device Drivers

**Phase 6 - Production Readiness Plan**

## Overview

The SIS Kernel includes a comprehensive set of mock device drivers that enable isolated testing without real hardware. These drivers implement the same traits as real devices but run entirely in memory, allowing for:

- **Fast testing** - No hardware access delays
- **Deterministic behavior** - Reproducible results
- **Failure injection** - Simulate hardware failures
- **Chaos testing** - Random errors and delays
- **CI/CD friendly** - Run anywhere without hardware

## Architecture

### Device Traits

All devices implement common trait abstractions for polymorphism:

**Location**: `crates/kernel/src/drivers/traits.rs`

#### Available Traits

1. **BlockDevice** - Block storage devices (disks, SSDs)
2. **NetworkDevice** - Network interfaces
3. **CharDevice** - Character devices (serial ports, etc.)
4. **TimerDevice** - Hardware timers
5. **DisplayDevice** - Framebuffers/displays
6. **InputDevice** - Keyboards, mice, touchscreens
7. **RtcDevice** - Real-time clocks
8. **RngDevice** - Random number generators
9. **GpioPin** - GPIO pins

### Mock Implementations

**Location**: `crates/kernel/src/drivers/mock/`

- `mock/block.rs` - Mock block device
- `mock/network.rs` - Mock network device
- `mock/timer.rs` - Mock timer device

## Mock Block Device

### Features

- In-memory storage
- Configurable capacity and block size
- Read-only mode support
- I/O failure injection
- Artificial delay simulation
- Statistics collection

### Usage

```rust
use crate::drivers::mock::MockBlockDevice;
use crate::drivers::traits::BlockDevice;

// Create 1MB device with 512-byte blocks
let device = MockBlockDevice::new("vdisk0", 1024 * 1024, 512);

// Write data
let write_buf = vec![0x42u8; 512];
device.write(0, &write_buf)?;

// Read it back
let mut read_buf = vec![0u8; 512];
device.read(0, &mut read_buf)?;

assert_eq!(read_buf, write_buf);
```

### Failure Injection

```rust
// Set 50% failure rate
device.set_fail_rate(50);

// Set 100μs artificial delay
device.set_delay(100);

// Now I/O operations may fail and will be delayed
device.write(0, &data)?; // May return Err(Errno::EIO)
```

### Read-Only Mode

```rust
let rom_data = vec![0x42u8; 4096];
let rom = MockBlockDevice::new_readonly("rom0", rom_data, 512);

// Writes fail with EROFS
assert_eq!(rom.write(0, &data), Err(Errno::EROFS));
```

### Statistics

```rust
// Access statistics
let reads = device.read_count();
let writes = device.write_count();
let errors = device.error_count();

// Reset statistics
device.reset_stats();
```

## Mock Network Device

### Features

- Packet queues (TX/RX)
- MAC address configuration
- MTU support
- Link state control
- Packet loss simulation
- Network delay simulation
- Statistics collection

### Usage

```rust
use crate::drivers::mock::MockNetworkDevice;
use crate::drivers::traits::NetworkDevice;

// Create network device
let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
let device = MockNetworkDevice::new("eth0", mac);

// Send packet
let packet = vec![0x01, 0x02, 0x03, 0x04];
device.send(&packet)?;

// Receive packet (inject for testing)
device.inject_packet(vec![0xAA, 0xBB, 0xCC, 0xDD]);
let mut rx_buf = vec![0u8; 1500];
let len = device.recv(&mut rx_buf)?;
```

### Failure Injection

```rust
// Simulate link down
device.set_link_up(false);
assert_eq!(device.send(&packet), Err(Errno::ENETDOWN));

// Set 20% packet loss
device.set_packet_loss_rate(20);

// Set 1ms network delay
device.set_delay(1000);
```

### Testing

```rust
// Inject packets for testing receive path
device.inject_packet(vec![/* packet data */]);

// Get transmitted packets for verification
let tx_packets = device.get_tx_packets();
assert_eq!(tx_packets.len(), expected_count);
```

### Statistics

```rust
let stats = device.stats();
println!("RX: {} packets, {} bytes", stats.rx_packets, stats.rx_bytes);
println!("TX: {} packets, {} bytes", stats.tx_packets, stats.tx_bytes);
println!("Errors: RX={}, TX={}", stats.rx_errors, stats.tx_errors);
println!("Dropped: RX={}, TX={}", stats.rx_dropped, stats.tx_dropped);
```

## Mock Timer Device

### Features

- Configurable frequency
- Manual time advancement (for testing)
- Timeout support
- Jitter simulation
- Nanosecond precision

### Usage

```rust
use crate::drivers::mock::MockTimerDevice;
use crate::drivers::traits::TimerDevice;

// Create 1MHz timer
let timer = MockTimerDevice::new("timer0", 1_000_000);

// Read current time
let ticks = timer.read();
let nanos = timer.nanos();
let millis = timer.millis();

// Advance time (for testing)
timer.advance(1000); // Advance by 1000 ticks
timer.advance_millis(1); // Advance by 1ms
```

### Timeouts

```rust
// Set timeout for 1000 ticks
timer.set_timeout(1000)?;

// Check if timeout occurred
if timer.timeout_occurred() {
    println!("Timeout!");
}

// Cancel timeout
timer.cancel_timeout()?;
```

### Jitter Simulation

```rust
// Set ±100μs jitter
timer.set_jitter(100);

// Now timer.read() will vary by ±100μs randomly
```

## Integration with Real Drivers

### Trait-Based Polymorphism

Real and mock devices share the same traits:

```rust
// Works with both real and mock devices
fn test_block_device(device: &dyn BlockDevice) {
    let mut buf = vec![0u8; device.block_size()];
    device.read(0, &mut buf).unwrap();
    device.write(0, &buf).unwrap();
    device.flush().unwrap();
}

// Test with mock
let mock_device = MockBlockDevice::new("mock", 4096, 512);
test_block_device(&mock_device);

// Use with real device
let real_device = VirtioBlkDevice::new(/* ... */);
test_block_device(&real_device);
```

### Device Selection

```rust
// Select device at compile time or runtime
pub fn create_block_device() -> Box<dyn BlockDevice> {
    #[cfg(feature = "mock-devices")]
    {
        Box::new(MockBlockDevice::new("mock", 1024 * 1024, 512))
    }

    #[cfg(not(feature = "mock-devices"))]
    {
        Box::new(VirtioBlkDevice::probe().expect("No block device"))
    }
}
```

## Testing Strategies

### Unit Testing

```rust
#[test]
fn test_filesystem_with_mock_device() {
    let device = MockBlockDevice::new("test", 1024 * 1024, 512);
    let fs = Ext4Filesystem::mount(Box::new(device))?;

    // Test filesystem operations
    fs.create_file("/test.txt")?;
    fs.write_file("/test.txt", b"Hello")?;

    let data = fs.read_file("/test.txt")?;
    assert_eq!(data, b"Hello");
}
```

### Failure Testing

```rust
#[test]
fn test_disk_full_handling() {
    let device = MockBlockDevice::new("test", 4096, 512); // Only 8 blocks
    device.set_fail_rate(50); // 50% I/O failure rate

    // Filesystem should handle failures gracefully
    let fs = Ext4Filesystem::mount(Box::new(device))?;

    loop {
        match fs.create_file(&format!("/file{}.txt", i)) {
            Ok(_) => i += 1,
            Err(Errno::ENOSPC) => break, // Expected
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
```

### Performance Testing

```rust
#[test]
fn test_network_throughput() {
    let device = MockNetworkDevice::new("test", [0; 6]);
    device.set_delay(0); // No artificial delay

    let start = timer.read();

    for _ in 0..1000 {
        device.send(&vec![0u8; 1500])?;
    }

    let elapsed = timer.read() - start;
    let throughput = (1000 * 1500 * 8) / (elapsed / timer.frequency());
    println!("Throughput: {} Mbps", throughput / 1_000_000);
}
```

### Deterministic Testing

```rust
#[test]
fn test_deterministic_behavior() {
    let timer = MockTimerDevice::new("test", 1_000_000);
    timer.set_jitter(0); // No jitter

    // Exact time control
    timer.set_ticks(0);
    assert_eq!(timer.read(), 0);

    timer.advance(1000);
    assert_eq!(timer.read(), 1000);

    // Behavior is fully deterministic
}
```

## Build Configuration

### Enable Mock Devices

```bash
# Build with mock devices
SIS_FEATURES="mock-devices" ./scripts/uefi_run.sh

# Or in Cargo.toml
cargo build --features mock-devices
```

### Feature Flag

The `mock-devices` feature flag controls compilation:

```toml
[features]
mock-devices = []  # Mock device drivers for isolated testing
```

### Conditional Compilation

```rust
#[cfg(feature = "mock-devices")]
use crate::drivers::mock::MockBlockDevice;

#[cfg(not(feature = "mock-devices"))]
use crate::drivers::virtio_blk::VirtioBlkDevice;
```

## Benefits

### 1. Fast Iteration

- No QEMU startup time
- No hardware initialization
- Instant feedback

### 2. Reproducibility

- Deterministic behavior
- No hardware flakiness
- Same results every time

### 3. Failure Injection

- Test error paths
- Verify error handling
- Chaos testing

### 4. CI/CD Integration

- Run anywhere
- No special hardware
- Parallel testing

### 5. Debugging

- Direct memory access
- No timing issues
- Easy state inspection

## Limitations

### What Mock Devices Don't Do

1. **Real Hardware Behavior** - Subtle hardware bugs won't be caught
2. **Performance Testing** - Can't measure real I/O performance
3. **DMA/MMIO** - No real memory-mapped I/O
4. **Interrupts** - No real interrupt handling
5. **Concurrency** - May not expose real race conditions

### When to Use Real Hardware

- Final integration testing
- Performance benchmarking
- Production validation
- Hardware-specific bugs
- End-to-end testing

## Examples

### Example 1: Filesystem Testing

```rust
#[test]
fn test_ext4_journal_recovery() {
    // Create mock device
    let device = MockBlockDevice::new("test", 10 * 1024 * 1024, 4096);

    // Mount filesystem
    let fs = Ext4Filesystem::mount(Box::new(device))?;

    // Write some files
    fs.create_file("/test1.txt")?;
    fs.write_file("/test1.txt", b"data1")?;

    // Simulate crash (no flush)
    device.set_fail_rate(100);
    fs.create_file("/test2.txt").unwrap_err(); // Fails

    // Remount and recover
    device.set_fail_rate(0);
    let fs = Ext4Filesystem::mount(Box::new(device))?;

    // Verify recovery
    assert!(fs.file_exists("/test1.txt"));
    assert!(!fs.file_exists("/test2.txt"));
}
```

### Example 2: Network Stack Testing

```rust
#[test]
fn test_tcp_retransmission() {
    let device = MockNetworkDevice::new("test", [0; 6]);
    device.set_packet_loss_rate(50); // 50% packet loss

    let tcp = TcpStack::new(Box::new(device));

    // Send data - should retry on packet loss
    tcp.send(&data)?;

    // Verify retransmission occurred
    let tx_packets = device.get_tx_packets();
    assert!(tx_packets.len() > 1); // Retransmitted
}
```

### Example 3: Timer Accuracy Testing

```rust
#[test]
fn test_scheduler_timing() {
    let timer = MockTimerDevice::new("test", 1_000_000);
    timer.set_jitter(0); // No jitter for accurate testing

    let scheduler = Scheduler::new(&timer);

    // Schedule task for 10ms in future
    scheduler.schedule_after(10_000, || { /* task */ });

    // Advance time by 9ms - should not run
    timer.advance_millis(9);
    assert!(!scheduler.has_ready_tasks());

    // Advance by 1ms more - should run
    timer.advance_millis(1);
    assert!(scheduler.has_ready_tasks());
}
```

## Future Enhancements

### Additional Mock Devices (TODO)

- **MockDisplayDevice** - Framebuffer testing
- **MockInputDevice** - Keyboard/mouse simulation
- **MockRtcDevice** - Real-time clock testing
- **MockRngDevice** - Deterministic random numbers
- **MockGpioPin** - GPIO testing

### Advanced Features (TODO)

- **Record/Replay** - Record real device traffic and replay
- **Differential Testing** - Compare mock vs real device behavior
- **Coverage Tracking** - Track which code paths are tested
- **Property-Based Testing** - QuickCheck integration

## References

- [Device Traits](../crates/kernel/src/drivers/traits.rs)
- [Mock Block Device](../crates/kernel/src/drivers/mock/block.rs)
- [Mock Network Device](../crates/kernel/src/drivers/mock/network.rs)
- [Mock Timer Device](../crates/kernel/src/drivers/mock/timer.rs)
- [Production Readiness Plan](./plans/PRODUCTION-READINESS-PLAN.md) - Phase 6

## Best Practices

### 1. Test with Mocks First

Start with mock devices for rapid iteration, then validate with real hardware.

### 2. Inject Failures

Always test error paths with failure injection.

### 3. Use Deterministic Timers

Disable jitter for reproducible timing tests.

### 4. Collect Statistics

Use device statistics to verify expected behavior.

### 5. Reset Between Tests

Always reset device state between tests.

```rust
#[test]
fn test_something() {
    let device = MockBlockDevice::new("test", 4096, 512);
    device.reset_stats(); // Start clean

    // Test code...

    // Verify statistics
    assert_eq!(device.read_count(), expected_reads);
}
```

---

**Last Updated**: 2025-11-07
**Version**: 1.0
**Status**: Phase 6 Complete

**See Also**:
- [Production Readiness Plan](./plans/PRODUCTION-READINESS-PLAN.md)
- [Testing Guide](./TESTING.md)
- [Development Guide](./DEVELOPMENT.md)
- [Security & Fuzzing](./SECURITY.md)
