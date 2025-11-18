# Hardware Driver Implementation Plan for RPi5

**Target**: Raspberry Pi 5 (BCM2712 SoC)  
**Timeline**: 13-17 weeks  
**Status**: Phase 0 Complete (60% ready)

---

## Critical Path

**Without RP1 PCIe driver, RPi5 has NO USB, NO Ethernet, NO extended GPIO on real hardware.**

```
RP1 PCIe Driver (Phase 1) → BLOCKS EVERYTHING
    ├─> USB XHCI (Phase 2)
    │       ├─> UVC Camera (Phase 5)
    │       └─> USB Audio (Phase 6)
    ├─> I2C Bus (Phase 3)
    ├─> SPI Bus (Phase 7)
    └─> PWM (Phase 4)
```

---

## Phase 1: RP1 PCIe Driver (CRITICAL)

**Duration**: 4-6 weeks  
**Priority**: HIGHEST  
**Files**: `crates/kernel/src/drivers/pcie/`

### What to Implement

1. **PCIe ECAM Configuration Space** (`pcie/ecam.rs`)
   - Memory-mapped config space access
   - Device enumeration (scan bus 0)
   - BAR (Base Address Register) parsing
   - MSI/MSI-X interrupt setup

2. **RP1 I/O Hub Driver** (`pcie/rp1.rs`)
   - RP1 device detection (vendor:device = 0x1de4:0x0001)
   - Initialize RP1 internal buses (I2C, SPI, PWM, GPIO)
   - Route interrupts from RP1 to GICv3
   - Power management

3. **Integration** (`pcie/mod.rs`)
   - Platform init hook in `platform/rpi5.rs`
   - FDT parsing integration (already detects PCIe in `platform/dt.rs`)

### Reference Code
- Linux: `drivers/pci/controller/pcie-brcmstb.c`
- Linux: `drivers/misc/rp1.c` (if exists in RPi kernel tree)
- Existing: `platform/dt.rs:52-58` (PcieInfo struct)

### Testing
```bash
# Debug output should show:
[PCIe] ECAM base: 0x1000_0000
[PCIe] Scanning bus 0...
[PCIe] Found device 00:00.0 - RP1 I/O Hub [1de4:0001]
[RP1] Initializing I/O hub...
[RP1] I2C controllers: 6
[RP1] SPI controllers: 5
[RP1] Ready
```

### Deliverables
- [ ] `crates/kernel/src/drivers/pcie/mod.rs`
- [ ] `crates/kernel/src/drivers/pcie/ecam.rs`
- [ ] `crates/kernel/src/drivers/pcie/rp1.rs`
- [ ] Shell command: `pcie scan` - enumerate devices
- [ ] Shell command: `rp1 status` - show RP1 state

---

## Phase 2: USB XHCI Driver

**Duration**: 3-4 weeks  
**Dependency**: Phase 1 (RP1 PCIe)  
**Files**: `crates/kernel/src/drivers/usb/xhci/`

### What to Implement

1. **XHCI Host Controller** (`usb/xhci/mod.rs`)
   - Capability registers parsing
   - Command/Event/Transfer ring management
   - Device slot allocation
   - USB 3.0 + USB 2.0 fallback

2. **USB Device Enumeration** (`usb/core.rs`)
   - Device detection (port status changes)
   - Descriptor reading (device, config, interface)
   - Driver matching by class/vendor/product

3. **Integration**
   - Detect XHCI via RP1 PCIe enumeration
   - IRQ routing from XHCI → GICv3

### Reference Code
- Linux: `drivers/usb/host/xhci.c`
- Existing: `camera/capture.rs:73-97` (USB enumeration stubs)

### Testing
```bash
# Connect USB keyboard
[USB] Port 0 status change: device connected
[USB] Enumerating device on port 0...
[USB] Device: USB Keyboard [vendor:046d product:c31c]
[USB] Class: HID (Human Interface Device)
```

### Deliverables
- [ ] `crates/kernel/src/drivers/usb/mod.rs`
- [ ] `crates/kernel/src/drivers/usb/xhci/` (full XHCI stack)
- [ ] `crates/kernel/src/drivers/usb/core.rs` (enumeration)
- [ ] Shell command: `usb list` - show connected devices

---

## Phase 3: I2C Bus Driver

**Duration**: 1-2 weeks  
**Dependency**: Phase 1 (RP1 PCIe)  
**Files**: `crates/kernel/src/drivers/i2c/`

### What to Implement

1. **BCM2712 I2C Controller** (`i2c/bcm2712.rs`)
   - I2C master mode (100kHz, 400kHz)
   - 7-bit addressing
   - Read/write transactions
   - Clock stretching support

2. **I2C Bus Abstraction** (`i2c/mod.rs`)
   - `i2c_read(bus, addr, buf)`
   - `i2c_write(bus, addr, data)`
   - Multiple bus support (RPi5 has 6 I2C controllers)

3. **RP1 Integration**
   - Access I2C controllers via RP1 PCIe mapping
   - IRQ handling for transaction completion

### Reference Code
- Linux: `drivers/i2c/busses/i2c-bcm2835.c`
- Rust crate: `embedded-hal` I2C traits

### Testing
```bash
# Scan I2C bus for devices
i2c scan 1
[I2C] Scanning bus 1...
[I2C] Device found at 0x68 (likely IMU)
[I2C] Device found at 0x76 (likely BME280)
```

### Deliverables
- [ ] `crates/kernel/src/drivers/i2c/mod.rs`
- [ ] `crates/kernel/src/drivers/i2c/bcm2712.rs`
- [ ] Shell command: `i2c scan <bus>` - detect devices
- [ ] Shell command: `i2c read <bus> <addr> <reg>` - read register

---

## Phase 4: PWM Driver

**Duration**: 1 week  
**Dependency**: Phase 1 (RP1 PCIe)  
**Files**: `crates/kernel/src/drivers/pwm/`

### What to Implement

1. **BCM2712 PWM Controller** (`pwm/bcm2712.rs`)
   - Multiple PWM channels (2+ channels)
   - Frequency: 50Hz (servos) to 20kHz (motors)
   - Duty cycle: 0-100% control
   - DMA mode for smooth output

2. **PWM Abstraction** (`pwm/mod.rs`)
   - `pwm_set_frequency(channel, hz)`
   - `pwm_set_duty_cycle(channel, percent)`
   - `pwm_enable(channel)` / `pwm_disable(channel)`

### Reference Code
- Linux: `drivers/pwm/pwm-bcm2835.c`
- Existing: `drivers/gpio/bcm2xxx.rs` (register access pattern)

### Testing
```bash
# Control servo on channel 0
pwm enable 0
pwm freq 0 50        # 50Hz for servo
pwm duty 0 7.5       # Center position (1.5ms pulse)
```

### Deliverables
- [ ] `crates/kernel/src/drivers/pwm/mod.rs`
- [ ] `crates/kernel/src/drivers/pwm/bcm2712.rs`
- [ ] Shell commands: `pwm enable|disable|freq|duty`

---

## Phase 5: USB Video Class (UVC) Driver

**Duration**: 2-3 weeks  
**Dependency**: Phase 2 (USB XHCI)  
**Files**: `crates/kernel/src/drivers/usb/uvc.rs`

### What to Implement

1. **UVC Protocol** (`usb/uvc.rs`)
   - Parse UVC descriptors (video streaming interface)
   - Negotiate format (MJPEG, YUYV)
   - Isochronous transfer setup
   - Frame extraction from USB stream

2. **Camera Integration**
   - Replace stubs in `camera/capture.rs:73-97`
   - Implement `CameraDevice::detect_devices()` with real USB enumeration
   - Connect to existing camera framework

### Reference Code
- Linux: `drivers/media/usb/uvc/`
- Existing: `camera/mod.rs` (framework ready)

### Testing
```bash
camera list
[Camera] USB Camera 0: Logitech C920 [046d:0892]
camera capture 0
[Camera] Captured frame: 1280x720 YUYV 184320 bytes
```

### Deliverables
- [ ] `crates/kernel/src/drivers/usb/uvc.rs`
- [ ] Update `camera/capture.rs` with real USB implementation
- [ ] Shell command: `camera list|capture|stream`

---

## Phase 6: USB Audio Class Driver

**Duration**: 1-2 weeks  
**Dependency**: Phase 2 (USB XHCI)  
**Files**: `crates/kernel/src/drivers/usb/audio.rs`

### What to Implement

1. **USB Audio Class** (`usb/audio.rs`)
   - Parse audio descriptors
   - Configure sample rate (16kHz mono for voice)
   - Isochronous transfer for audio stream
   - Buffer management (4KB buffers)

2. **Audio Integration**
   - Replace stubs in `audio/input.rs`, `audio/output.rs`
   - Implement `AudioDevice::detect_devices()`

### Reference Code
- Linux: `sound/usb/`
- Existing: `audio/mod.rs:156-162` (global manager ready)

### Testing
```bash
audio list
[Audio] USB Microphone: Blue Yeti [b58e:9e84]
audio record 5
[Audio] Recording 5 seconds...
[Audio] Captured 80000 samples (16kHz mono)
```

### Deliverables
- [ ] `crates/kernel/src/drivers/usb/audio.rs`
- [ ] Update `audio/input.rs` with USB implementation
- [ ] Shell command: `audio list|record|play`

---

## Phase 7: SPI Bus Driver

**Duration**: 1-2 weeks  
**Dependency**: Phase 1 (RP1 PCIe)  
**Files**: `crates/kernel/src/drivers/spi/`

### What to Implement

1. **BCM2712 SPI Controller** (`spi/bcm2712.rs`)
   - SPI master mode
   - Speed: up to 125MHz
   - Modes: 0, 1, 2, 3 (CPOL/CPHA)
   - Chip select management

2. **SPI Abstraction** (`spi/mod.rs`)
   - `spi_transfer(bus, cs, data_out, data_in)`
   - Multiple bus support

### Reference Code
- Linux: `drivers/spi/spi-bcm2835.c`

### Deliverables
- [ ] `crates/kernel/src/drivers/spi/mod.rs`
- [ ] `crates/kernel/src/drivers/spi/bcm2712.rs`
- [ ] Shell command: `spi transfer <bus> <cs> <hex_data>`

---

## Phase 8: CAN Bus Driver (Optional)

**Duration**: 2 weeks  
**Files**: `crates/kernel/src/drivers/can/`

### What to Implement

1. **SocketCAN Interface** (`can/socketcan.rs`)
   - CAN 2.0B protocol
   - Standard + Extended IDs
   - CAN FD support (optional)
   - Error handling (bus-off, error frames)

### Reference Code
- Linux: `drivers/net/can/`
- SocketCAN documentation

### Deliverables
- [ ] `crates/kernel/src/drivers/can/mod.rs`
- [ ] Shell command: `can send|receive|status`

---

## Phase 9: Sensor Drivers

**Duration**: 2-4 weeks (depends on sensors needed)  
**Dependency**: Phase 3 (I2C)  
**Files**: `crates/kernel/src/drivers/sensors/`

### Common Sensors for Robotics

1. **IMU**: MPU6050, MPU9250, BNO055 (I2C)
2. **Distance**: VL53L0X TOF sensor (I2C)
3. **Environment**: BME280 temp/humidity (I2C)
4. **Current/Voltage**: INA219 (I2C)

### Template per Sensor (`sensors/mpu6050.rs`)
```rust
pub struct Mpu6050 {
    i2c_bus: u8,
    addr: u8,
}

impl Mpu6050 {
    pub fn new(bus: u8) -> Result<Self> {
        // Initialize, read WHO_AM_I register
    }
    
    pub fn read_accel(&self) -> Result<(i16, i16, i16)> {
        // Read accelerometer data
    }
    
    pub fn read_gyro(&self) -> Result<(i16, i16, i16)> {
        // Read gyroscope data
    }
}
```

### Deliverables
- [ ] `crates/kernel/src/drivers/sensors/mod.rs`
- [ ] One driver per sensor (2-3 days each)
- [ ] Shell commands per sensor

---

## Integration & Testing

### Before Hardware Boot
1. **Unit Tests**: Each driver must have `#[cfg(test)]` tests
2. **QEMU Testing**: Test what's possible in QEMU
3. **Mock Devices**: Create mock USB/I2C devices for testing

### First Hardware Boot Checklist
```bash
# Milestone 1: Serial console works
[✓] UART output visible
[✓] Shell prompt appears

# Milestone 2: PCIe enumeration
[✓] RP1 device detected
[✓] RP1 initialization complete

# Milestone 3: USB works
[✓] USB keyboard detected
[✓] Keyboard input works in shell

# Milestone 4: I2C works
[✓] I2C bus scan finds devices
[✓] Can read sensor data

# Milestone 5: Camera works
[✓] USB camera enumerated
[✓] Can capture frame

# Milestone 6: Full robotics ready
[✓] IMU data streaming
[✓] PWM controlling servo
[✓] Camera capturing at 30fps
```

---

## Branch Submission Guidelines

### Per-Phase Submission

Each phase should be submitted as a separate branch:

```bash
# Phase 1
git checkout -b feature/rp1-pcie-driver
# Implement Phase 1
git commit -m "feat(drivers): add RP1 PCIe driver for RPi5"
git push origin feature/rp1-pcie-driver

# Phase 2
git checkout -b feature/usb-xhci-driver
# Implement Phase 2
git commit -m "feat(drivers): add USB XHCI host controller"
git push origin feature/usb-xhci-driver
```

### Branch Naming Convention
- `feature/rp1-pcie-driver`
- `feature/usb-xhci-driver`
- `feature/i2c-bus-driver`
- `feature/pwm-driver`
- `feature/uvc-camera-driver`
- `feature/usb-audio-driver`
- `feature/spi-bus-driver`
- `feature/can-bus-driver`
- `feature/sensors-<sensor-name>`

### Commit Message Format
```
feat(drivers): add <driver-name>

- Implement <key feature 1>
- Implement <key feature 2>
- Add shell commands for testing
- Add unit tests

Tested: <brief test description>
```

### What to Include in Each Branch

1. **Driver Code**: All `.rs` files in correct location
2. **Shell Commands**: Interactive testing commands
3. **Documentation**: Update relevant `README.md` or add `docs/drivers/<driver>.md`
4. **Cargo.toml**: Add feature flags if needed (e.g., `feature = "hardware"`)
5. **Tests**: Unit tests in `#[cfg(test)]` sections

### Integration Process

When you provide the branch:
1. Share branch name and brief status
2. I will:
   - Pull the branch
   - Review code for integration points
   - Integrate into main codebase
   - Run tests
   - Document any issues found
3. Iterate if needed

---

## Summary

### Minimum Viable Product (MVP)
To boot on real RPi5 hardware:
- **Phase 1** (RP1 PCIe) - MANDATORY
- **Phase 2** (USB XHCI) - MANDATORY (for USB keyboard debug)
- **Phase 3** (I2C) - OPTIONAL but recommended

**MVP Timeline**: 8-12 weeks

### Full Robotics Ready
All phases complete:
- **Timeline**: 17-26 weeks (4-6 months)
- **Test Coverage**: >80% for each driver
- **Documentation**: Each driver documented

### Current Baseline
- ✅ Platform: 100% ready
- ✅ Core kernel: 100% ready
- ✅ Networking (QEMU): 100% ready
- ✅ GPIO: 100% ready
- ❌ Peripheral drivers: 0% (need all phases)

**After all phases**: 100% hardware ready for robotics deployment on RPi5.
