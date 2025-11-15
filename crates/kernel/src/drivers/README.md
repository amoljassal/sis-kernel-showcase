# Device Drivers Subsystem

## What Lives Here

The drivers subsystem provides hardware abstraction for all I/O devices in SIS Kernel. All device drivers integrate through the VirtIO framework, providing a common interface for block devices, network devices, and GPU devices.

**Core Components:**
- `mod.rs` - Driver core and device registration
- `virtio_blk.rs` - VirtIO block device driver (disk I/O)
- `virtio_net.rs` - VirtIO network device driver (Ethernet)
- `virtio_gpu.rs` - VirtIO GPU device driver (graphics acceleration)

**Related:**
- `../virtio/virtqueue.rs` - VirtIO queue management (shared by all drivers)
- `../block/mod.rs` - Block device abstraction layer
- `../net/mod.rs` - Network stack integration
- `../vfs/devfs.rs` - `/dev` filesystem for device nodes

## How to Extend: Adding a New Device Driver

All device drivers integrate through **VirtIO device traits** and the **Device trait**:

```rust
pub trait Device: Send + Sync {
    fn name(&self) -> &str;
    fn device_type(&self) -> DeviceType;
    fn init(&mut self) -> Result<(), DriverError>;
    fn reset(&mut self) -> Result<(), DriverError>;
}

pub trait BlockDevice: Device {
    fn read_blocks(&mut self, start_lba: u64, count: u32, buf: &mut [u8]) -> Result<usize>;
    fn write_blocks(&mut self, start_lba: u64, count: u32, buf: &[u8]) -> Result<usize>;
    fn block_size(&self) -> u32;
    fn total_blocks(&self) -> u64;
}

pub trait NetworkDevice: Device {
    fn send_packet(&mut self, packet: &[u8]) -> Result<()>;
    fn receive_packet(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn mac_address(&self) -> [u8; 6];
}
```

### Steps to Add a New VirtIO Device

1. **Create driver module** (e.g., `virtio_console.rs`)
2. **Implement Device trait** for basic device operations
3. **Implement device-specific trait** (BlockDevice, NetworkDevice, etc.)
4. **Initialize VirtIO queues** using `../virtio/virtqueue.rs`
5. **Register device** in `mod.rs::init_drivers()`
6. **Add interrupt handler** for device events
7. **Test with QEMU** using appropriate VirtIO device

### Example: Minimal VirtIO Device

```rust
use crate::virtio::VirtQueue;

pub struct VirtioConsole {
    base_addr: usize,
    rx_queue: VirtQueue,
    tx_queue: VirtQueue,
}

impl Device for VirtioConsole {
    fn name(&self) -> &str {
        "virtio-console"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Console
    }

    fn init(&mut self) -> Result<(), DriverError> {
        // 1. Reset device
        // 2. Set ACKNOWLEDGE status bit
        // 3. Set DRIVER status bit
        // 4. Negotiate features
        // 5. Set FEATURES_OK status bit
        // 6. Initialize virtqueues
        // 7. Set DRIVER_OK status bit
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DriverError> {
        // Write 0 to device status register
        Ok(())
    }
}
```

## Integration Points and Boundaries

### **Incoming:** What Depends on Drivers
- **VFS layer** (`../vfs/`) - Block devices mount filesystems (ext2, ext4)
- **Network stack** (`../net/smoltcp_iface.rs`) - Network device for TCP/IP
- **Graphics subsystem** (`../graphics/`) - GPU device for rendering
- **Syscalls** (`../syscall/`) - `sys_read`, `sys_write` for device files

### **Outgoing:** What Drivers Depend On
- **VirtIO framework** (`../virtio/`) - Queue management, MMIO access
- **Interrupt handler** (`../interrupts.rs`) - Device interrupt routing
- **Memory management** (`../mm/`) - DMA buffers, page allocation
- **Platform layer** (`../platform/`) - MMIO base addresses, IRQ numbers

### **Boundary Rules**
1. **All hardware access through platform layer** - No hardcoded MMIO addresses
2. **Device lifecycle managed by driver core** - `init() → running → reset()`
3. **Interrupt handlers are minimal** - Defer work to driver main loop
4. **DMA buffers must be physically contiguous** - Use buddy allocator for large buffers
5. **Drivers are Send + Sync** - Must be safe to call from multiple contexts

## Driver Architecture Patterns

### VirtIO Device Initialization Sequence

```
1. Device Discovery (platform layer provides MMIO base)
   ↓
2. Device Reset (write 0 to status register)
   ↓
3. Feature Negotiation (driver ↔ device)
   ↓
4. VirtQueue Setup (allocate descriptor rings)
   ↓
5. DRIVER_OK (device ready for I/O)
   ↓
6. Register interrupt handler
   ↓
7. Device operational
```

### Interrupt Handling Pattern

```rust
// Minimal interrupt handler
fn virtio_blk_irq_handler() {
    // 1. Read interrupt status
    // 2. Clear interrupt (ACK)
    // 3. Set flag for deferred work
    // 4. Return quickly (no heavy processing)
}

// Deferred work (called from main loop)
fn virtio_blk_process_completions() {
    // 1. Process completed requests
    // 2. Update internal state
    // 3. Wake waiting tasks
}
```

## API Surface

### Block Device API
```rust
// Read/write blocks (512-byte or 4096-byte sectors)
pub fn read_blocks(dev: &mut dyn BlockDevice, lba: u64, count: u32, buf: &mut [u8]) -> Result<usize>;
pub fn write_blocks(dev: &mut dyn BlockDevice, lba: u64, count: u32, buf: &[u8]) -> Result<usize>;

// Device properties
pub fn block_size(dev: &dyn BlockDevice) -> u32;
pub fn total_blocks(dev: &dyn BlockDevice) -> u64;
pub fn capacity_mb(dev: &dyn BlockDevice) -> u64;
```

### Network Device API
```rust
// Packet transmission/reception
pub fn send_packet(dev: &mut dyn NetworkDevice, packet: &[u8]) -> Result<()>;
pub fn receive_packet(dev: &mut dyn NetworkDevice, buf: &mut [u8]) -> Result<usize>;

// Device properties
pub fn mac_address(dev: &dyn NetworkDevice) -> [u8; 6];
pub fn link_status(dev: &dyn NetworkDevice) -> LinkStatus;
```

### Device Registration
```rust
// Register device with driver core
pub fn register_device(device: Box<dyn Device>) -> DeviceId;
pub fn unregister_device(id: DeviceId) -> Result<()>;

// Look up registered devices
pub fn get_device_by_type(device_type: DeviceType) -> Option<&'static mut dyn Device>;
```

## Testing Your Driver

### QEMU Device Configuration

Add VirtIO device to QEMU command line:

```bash
# Block device
-device virtio-blk-device,drive=hd0 \
-drive file=disk.img,if=none,id=hd0,format=raw

# Network device
-device virtio-net-device,netdev=net0 \
-netdev user,id=net0,hostfwd=tcp::8080-:80

# GPU device
-device virtio-gpu-device

# Console device
-device virtio-console-device
```

### Driver Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_init() {
        let mut dev = VirtioBlock::new(0x10000000);
        assert!(dev.init().is_ok());
        assert_eq!(dev.device_type(), DeviceType::Block);
    }

    #[test]
    fn test_block_io() {
        let mut dev = VirtioBlock::new(0x10000000);
        dev.init().unwrap();

        let mut buf = [0u8; 512];
        assert!(dev.read_blocks(0, 1, &mut buf).is_ok());
    }
}
```

### Integration Tests

Test through VFS and syscall layer:

```rust
// Mount filesystem on block device
vfs::mount("/mnt", "ext4", block_device_id)?;

// Read file from mounted filesystem
let fd = vfs_open("/mnt/test.txt", O_RDONLY, 0)?;
let mut buf = [0u8; 256];
vfs_read(fd, &mut buf)?;
```

## Common Patterns and Gotchas

### ✅ Correct: Physically Contiguous DMA Buffers
```rust
let buf_pages = buddy_alloc(order);  // Allocate 2^order contiguous pages
let phys_addr = virt_to_phys(buf_pages);
// Pass phys_addr to device
```

### ❌ Incorrect: Virtual Address to Device
```rust
let buf = vec![0u8; 4096];  // Heap allocation (not physically contiguous)
device.set_dma_addr(buf.as_ptr());  // WRONG - device needs physical address
```

### ✅ Correct: Minimal Interrupt Handler
```rust
fn irq_handler() {
    clear_interrupt();
    set_work_pending_flag();  // Defer heavy work
}
```

### ❌ Incorrect: Heavy Processing in IRQ
```rust
fn irq_handler() {
    process_all_completions();  // DON'T - too slow for interrupt context
    update_network_statistics();
    log_debug_info();
}
```

### ✅ Correct: Device State Machine
```rust
enum DeviceState {
    Uninitialized,
    Initializing,
    Ready,
    Error,
}

impl VirtioDevice {
    fn transition_state(&mut self, new_state: DeviceState) {
        log_debug!("Device state: {:?} -> {:?}", self.state, new_state);
        self.state = new_state;
    }
}
```

## VirtIO Specification Compliance

SIS Kernel drivers implement **VirtIO 1.0** specification:

- **MMIO transport** (no PCI support yet)
- **Split virtqueues** (descriptor table + available ring + used ring)
- **Feature negotiation** via DEVICE_FEATURES and DRIVER_FEATURES registers
- **Status register state machine** (ACKNOWLEDGE → DRIVER → FEATURES_OK → DRIVER_OK)

### VirtIO Register Layout (MMIO)

| Offset | Register | Access | Description |
|--------|----------|--------|-------------|
| 0x000  | MagicValue | R | VirtIO magic (0x74726976) |
| 0x004  | Version | R | VirtIO version (2 for v1.0) |
| 0x008  | DeviceID | R | Device type (1=net, 2=blk, 16=gpu) |
| 0x010  | DeviceFeatures | R | Device feature bits |
| 0x020  | DriverFeatures | W | Driver feature bits |
| 0x034  | QueueSel | W | Select virtqueue |
| 0x038  | QueueNumMax | R | Maximum queue size |
| 0x03C  | QueueNum | W | Actual queue size |
| 0x050  | QueueReady | RW | Queue enable (1=ready) |
| 0x064  | QueueNotify | W | Kick queue (write queue index) |
| 0x070  | Status | RW | Device status |

## Key Files and Lines of Code

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | ~200 | Driver core and device registration |
| `virtio_blk.rs` | ~400 | Block device driver |
| `virtio_net.rs` | ~500 | Network device driver |
| `virtio_gpu.rs` | ~600 | GPU device driver |
| `../virtio/virtqueue.rs` | ~300 | VirtQueue management (shared) |
| `../virtio_console.rs` | ~350 | Console device (top-level module) |

## Mock Devices (Phase 4)

For testing without hardware dependencies:

```rust
// Phase 4 mock device traits
pub trait MockBlockDevice: BlockDevice {
    fn inject_error(&mut self, error_type: ErrorType);
    fn set_latency(&mut self, latency_ms: u64);
}

pub trait MockNetworkDevice: NetworkDevice {
    fn drop_next_packet(&mut self);
    fn set_packet_loss_rate(&mut self, rate: f32);
}
```

See `../kernel/README.md` (once created) for mock device documentation.

## Performance Considerations

1. **Interrupt coalescing** - Batch multiple completions per interrupt
2. **DMA alignment** - Align buffers to cache line boundaries (64 bytes)
3. **Ring buffer size** - Larger rings reduce notification overhead
4. **Zero-copy I/O** - Use DMA directly from/to user buffers when possible

## Future Work / TODOs

- [ ] PCI transport support (in addition to MMIO)
- [ ] Packed virtqueue layout (VirtIO 1.1)
- [ ] VirtIO-SCSI for advanced block features
- [ ] VirtIO-input for keyboard/mouse
- [ ] VirtIO-sound for audio devices
- [ ] MSI/MSI-X interrupt support
- [ ] Driver hot-plug and hot-unplug
- [ ] Power management (device sleep/wake)

## Related Documentation

- **Phase A.4 Device Drivers in main README** - High-level overview
- `../virtio/README.md` - VirtIO framework details (TODO)
- `../platform/README.md` - Platform layer and MMIO (TODO)
- [VirtIO 1.0 Specification](https://docs.oasis-open.org/virtio/virtio/v1.0/virtio-v1.0.html)

## Contact / Maintainers

Device drivers are critical for system stability. Changes to driver core or VirtIO framework should be reviewed carefully.

For questions about adding new devices or VirtIO implementation, refer to this README or the VirtIO specification.
