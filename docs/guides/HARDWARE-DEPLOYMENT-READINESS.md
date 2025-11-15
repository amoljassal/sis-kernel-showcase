# Hardware Deployment Readiness Guide

This guide documents requirements, procedures, and validation steps for deploying the SIS AI-native kernel on real ARM64 hardware.

## Overview

The SIS kernel has been developed and validated in QEMU (AArch64 virt platform). This guide provides the roadmap for transitioning to real hardware deployment.

## Target Hardware Requirements

### Minimum Requirements

**Processor:**
- Architecture: ARMv8-A (AArch64)
- Cores: 1+ (tested with single-core, multi-core ready)
- Features required:
  - NEON SIMD support (for neural network operations)
  - Generic Timer (CNTFRQ_EL0, CNTV_TVAL_EL0)
  - Performance Monitoring Unit (PMU) - optional but recommended

**Memory:**
- RAM: 128 MB minimum, 256 MB recommended
- Heap configured: 100 KiB (adjustable in kernel/src/main.rs)
- Stack: 64 KiB per execution context

**Interrupt Controller:**
- GICv3 (Generic Interrupt Controller version 3)
  - GICD (Distributor) support
  - GICR (Redistributor) support
  - PPI 30 for virtual timer
  - SGIs for inter-processor interrupts (multi-core)

**UART:**
- PL011 (ARM PrimeCell UART) compatible
- Memory-mapped at discoverable address via device tree
- Baud rate: 115200 (configurable)

**Firmware:**
- UEFI firmware with ARM64 support
- Boot services and runtime services
- Simple File System Protocol support
- LoadedImage Protocol support

**Storage:**
- EFI System Partition (ESP)
- FAT32 filesystem
- Minimum 16 MB for kernel + bootloader

### Recommended Hardware

**Development Boards:**
1. **Raspberry Pi 4/5** (BCM2711/BCM2712)
   - 4x Cortex-A72 (Pi 4) or 4x Cortex-A76 (Pi 5)
   - 2-8 GB RAM
   - GICv2 (Pi 4) or GICv3 (Pi 5)
   - UEFI firmware available (EDK2)
   - Well-documented, widely available

2. **NVIDIA Jetson Nano/Xavier**
   - ARM Cortex-A57 (Nano) or Carmel (Xavier)
   - 2-32 GB RAM
   - GICv2/GICv3
   - UEFI support via coreboot/tianocore
   - GPU available for future acceleration

3. **96Boards HiKey 960**
   - 4x Cortex-A73 + 4x Cortex-A53
   - 3 GB RAM
   - GICv3
   - Native UEFI support
   - Good community support

4. **Amazon Graviton-based EC2 Instances**
   - AWS Graviton2/3 (ARM Neoverse cores)
   - Configurable RAM (up to 512 GB)
   - GICv3
   - UEFI boot support
   - Cloud-based validation

**Network (Optional but Recommended):**
- Ethernet or WiFi for remote access
- VirtIO network device support (for virtualized deployment)
- Used for: remote debugging, log aggregation, distributed testing

## Pre-Deployment Checklist

### Code Preparation

**1. Feature Configuration**
```toml
# Review crates/kernel/Cargo.toml
# Recommended features for hardware:
[features]
default = ["bringup", "llm", "crypto-real"]
bringup = []          # Essential for hardware bring-up
llm = []              # AI/ML features
crypto-real = []      # Real cryptographic verification
hw-minimal = []       # Lean build for resource-constrained hardware
```

**2. Build for Hardware**
```bash
# Standard build
cargo build --release --target aarch64-unknown-none

# Minimal build for resource-constrained hardware
cargo build --release --target aarch64-unknown-none --features hw-minimal

# With specific features
SIS_FEATURES="llm,crypto-real" cargo build --release --target aarch64-unknown-none
```

**3. Hardware-Specific Configuration**

Review and adjust in `crates/kernel/src/main.rs`:
- Heap size (currently 100 KiB, increase for production)
- Stack size (currently 64 KiB)
- Timer frequency (currently assumes 62.5 MHz, adjust based on CNTFRQ_EL0)
- UART base address (auto-discovered via device tree)
- Memory layout (adjust if needed for specific hardware)

### Firmware Preparation

**1. UEFI Firmware Installation**

Raspberry Pi 4/5:
```bash
# Download UEFI firmware from github.com/pftf/RPi4
# or github.com/worproject/rpi5-uefi
# Flash to SD card FAT32 partition
```

NVIDIA Jetson:
```bash
# Use L4T (Linux for Tegra) with UEFI support
# or custom coreboot/tianocore build
```

96Boards HiKey:
```bash
# UEFI firmware typically pre-installed
# Update via board-specific flash tool if needed
```

**2. Create EFI System Partition**

```bash
# Format SD card or storage device
sudo fdiskutil eraseDisk FAT32 SISBOOT MBRFormat /dev/diskX  # macOS
# or
sudo mkfs.vfat -F 32 /dev/sdX1  # Linux

# Create EFI directory structure
mkdir -p /Volumes/SISBOOT/EFI/BOOT
mkdir -p /Volumes/SISBOOT/EFI/SIS

# Copy bootloader
cp target/aarch64-unknown-uefi/release/uefi-boot.efi /Volumes/SISBOOT/EFI/BOOT/BOOTAA64.EFI

# Copy kernel
cp target/aarch64-unknown-none/release/sis-kernel /Volumes/SISBOOT/EFI/SIS/KERNEL.ELF

# Verify
ls -lh /Volumes/SISBOOT/EFI/BOOT/
ls -lh /Volumes/SISBOOT/EFI/SIS/
```

### Device Tree Considerations

**QEMU vs Hardware Differences:**

| Aspect | QEMU | Hardware |
|--------|------|----------|
| UART Address | Fixed (0x09000000) | Device tree discovery |
| GIC Address | Fixed (GICD: 0x08000000) | Device tree discovery |
| Timer Frequency | 62.5 MHz | CNTFRQ_EL0 register |
| Memory Layout | Simplified | Complex, multi-region |
| Devices | Minimal set | Full platform devices |

**Device Tree Parsing:**

The kernel includes device tree parsing in `crates/kernel/src/platform/dt.rs`:
- Automatically discovers UART base address
- Parses memory layout
- Identifies interrupt controller
- Extracts compatible strings

**Enable Device Tree Override (if needed):**
```bash
# Build with device tree override feature
SIS_FEATURES="dt-override" cargo build --release --target aarch64-unknown-none
```

## Deployment Procedure

### Step 1: Initial Bring-Up

**Serial Console Setup:**
```bash
# macOS
screen /dev/tty.usbserial-XXXXX 115200

# Linux
screen /dev/ttyUSB0 115200
# or
minicom -D /dev/ttyUSB0 -b 115200
```

**First Boot Expected Output:**
```
UEFI firmware (version ...)
[... UEFI messages ...]
BOOT-ARM64 (UEFI)
SIS UEFI loader v2 (VERBOSE)
[... loader messages ...]
!KERNEL(U)
STACK OK
VECTORS OK
MMU: MAIR/TCR
MMU: TABLES
MMU: TTBR0
MMU: SCTLR
MMU ON
[... initialization messages ...]
=== SIS Kernel Shell ===
Type 'help' for available commands

sis>
```

**Troubleshooting First Boot:**

If boot hangs at UEFI:
- Verify BOOTAA64.EFI is correctly named and in /EFI/BOOT/
- Check UEFI firmware is ARM64, not ARM32
- Ensure SD card/storage is properly formatted (FAT32, MBR)

If boot hangs at "!KERNEL(U)":
- Check serial console baud rate (115200)
- Verify kernel ELF is correctly copied to /EFI/SIS/KERNEL.ELF
- Check kernel was built for aarch64-unknown-none target

If boot hangs at "STACK OK":
- Stack initialization failed
- Check memory map compatibility
- Review stack allocation in main.rs

If boot hangs at "VECTORS OK":
- Exception vector setup failed
- Check exception vector alignment
- Verify VBAR_EL1 register access

If boot hangs at "MMU: ...":
- MMU initialization failed
- Check page table setup
- Verify memory attributes
- Review MAIR/TCR/TTBR configuration

If boot hangs after "MMU ON":
- Post-MMU initialization failed
- Check UART address discovery
- Verify GIC initialization
- Review timer setup

### Step 2: Hardware Validation Commands

Once shell is available, run validation sequence:

```bash
# 1. Basic system info
help
version

# 2. Check memory allocator
# (should show heap stats without errors)

# 3. Check timer frequency
# Look for: METRIC cntfrq_hz=<value>
# Verify matches hardware specification

# 4. Test neural network
# If 'llm' feature enabled:
imagedemo
# Should see: Top-5 labels without crashes

# 5. Test autonomous mode
autoctl on
autoctl status
# Should show: Mode: ENABLED

# 6. Quick benchmark
benchmark commands 5
# Should complete without crashes

# 7. Check for errors
# Review boot log for any [ERROR] messages
```

### Step 3: Automated Testing on Hardware

Run standard test suite:

```bash
# From development machine (if network available)
ssh user@hardware-board

# Or via serial console
# Run quick validation
./scripts/run_phase4_tests_expect.sh quick

# If quick validation passes, run standard suite
./scripts/run_phase4_tests_expect.sh standard

# For extended validation
./scripts/run_phase4_tests_expect.sh full
```

**Expected Hardware Performance:**

| Metric | QEMU (typical) | Hardware (expected) |
|--------|---------------|---------------------|
| Boot time | 2-3 seconds | 1-2 seconds |
| Context switch | ~1 µs | 0.5-1 µs |
| Memory alloc | ~25 µs | 10-20 µs |
| NN inference | ~2.3 ms | <100 µs (with optimizations) |
| Command rate | 10K/sec | 20K-50K/sec |
| Network pkts | 1-2 Mpps | 5-10 Mpps |

### Step 4: Hardware-Specific Optimizations

**1. Adjust Timer Frequency**

If timer frequency differs from QEMU (62.5 MHz):
- Kernel auto-detects via CNTFRQ_EL0
- Verify correct detection in boot log
- Adjust decision_interval_ms if needed for autonomous mode

**2. Optimize Memory Configuration**

```rust
// In crates/kernel/src/main.rs
// Increase heap for production:
pub const HEAP_SIZE: usize = 1_048_576;  // 1 MB (was 100 KiB)
```

**3. Enable Performance Features**

```toml
# In crates/kernel/Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"              # Aggressive link-time optimization
codegen-units = 1        # Better optimization, slower build
panic = "abort"          # Smaller binary
strip = true             # Remove debug symbols
```

**4. NEON Optimization**

Ensure NEON is enabled for neural network operations:
```rust
// Already configured in .cargo/config.toml
[target.aarch64-unknown-none]
rustflags = ["-C", "target-cpu=native"]  # Use all available features
```

## Validation Criteria for Hardware

### Functional Validation

**Must Pass:**
- [ ] Boot completes to shell prompt
- [ ] All shell commands respond
- [ ] Memory allocator functional (no OOM on basic operations)
- [ ] Timer interrupts firing (check PMU metrics)
- [ ] Neural network inference completes
- [ ] Autonomous mode enables/disables
- [ ] No kernel panics during 10-minute operation

**Should Pass:**
- [ ] Quick test suite passes (AI + benchmarks)
- [ ] Standard test suite passes (AI + benchmarks + compliance)
- [ ] 1-hour autonomous validation passes
- [ ] Memory stress test passes (10min at 85%)
- [ ] No crashes during 24-hour operation

### Performance Validation

**Target Metrics:**
- Context switch: <1 µs
- Memory allocation: <20 µs
- NN inference: <100 µs (with optimizations)
- Command processing: >20K/sec
- Network throughput: >5 Mpps
- Autonomous decisions: >100/hour
- System stability: 0 crashes in 24 hours

### Compliance Validation

**Must Maintain:**
- EU AI Act compliance: ≥85%
- Safety score: ≥90/100
- Safety checklist: ≥90% complete
- Critical incidents: 0
- Production readiness: YES

## Known Hardware Limitations

### QEMU vs Hardware Differences

**1. Interrupt Latency**
- QEMU: Deterministic, low jitter
- Hardware: Variable, higher jitter possible
- Impact: May need to adjust autonomy decision intervals

**2. Memory Performance**
- QEMU: Simulated, relatively fast
- Hardware: Real DRAM, cache effects matter
- Impact: May need to optimize memory access patterns

**3. Timer Accuracy**
- QEMU: Perfect virtual timer
- Hardware: Crystal oscillator accuracy dependent
- Impact: Long-duration tests may show slight drift

**4. Device Availability**
- QEMU: Minimal VirtIO devices
- Hardware: Full platform devices (GPIO, I2C, SPI, etc.)
- Impact: Additional device drivers may be needed

### Platform-Specific Issues

**Raspberry Pi 4:**
- GICv2 instead of GICv3 (requires GIC code adjustment)
- VideoCore firmware interactions
- USB boot quirks

**Raspberry Pi 5:**
- GICv3 support (compatible)
- RP1 southbridge peculiarities
- PCIe device considerations

**NVIDIA Jetson:**
- Tegra-specific power management
- GPU/CUDA integration opportunities
- Complex device tree

**96Boards:**
- Standardized boot process (easiest)
- Consistent UEFI implementation
- Good hardware documentation

## Production Deployment

### Pre-Production Checklist

**Code Quality:**
- [ ] All automated tests passing
- [ ] Extended duration tests completed (1hr, 4hr, 24hr)
- [ ] Memory stress tests passed (95% pressure)
- [ ] Compliance validation passed (92%+)
- [ ] Code review completed
- [ ] Security audit completed

**Documentation:**
- [ ] Hardware compatibility documented
- [ ] Deployment procedures documented
- [ ] Troubleshooting guide available
- [ ] API documentation complete
- [ ] Integration guide available

**Testing:**
- [ ] Hardware validation completed
- [ ] Performance baseline established
- [ ] Stability validated (24hr+ uptime)
- [ ] Failure recovery tested
- [ ] Rollback procedure tested

**Operational Readiness:**
- [ ] Monitoring infrastructure setup
- [ ] Logging aggregation configured
- [ ] Alert thresholds defined
- [ ] Incident response procedures documented
- [ ] Rollback procedure tested

### Production Monitoring

**Key Metrics to Monitor:**
```
# System Health
- Uptime
- Memory utilization
- CPU utilization
- Interrupt rate

# AI/ML Performance
- Neural network inference count
- Autonomous decision count
- Watchdog trigger rate
- Prediction accuracy

# Safety Metrics
- Critical incidents
- OOM events
- Watchdog triggers
- Hard limit violations

# Performance Metrics
- Context switch latency
- Memory allocation latency
- Command processing rate
- Network throughput
```

**Alert Thresholds:**
- Critical incident: >0 (immediate alert)
- OOM event rate: >1/hour (warning)
- Watchdog trigger rate: >1%/hour (warning)
- System uptime: <99.9% (warning)

## Hardware Deployment Timeline

### Phase 1: Initial Bring-Up (Week 1)
- Day 1-2: Hardware acquisition and firmware setup
- Day 3-4: Initial kernel boot and serial console validation
- Day 5: Shell command validation
- Day 6-7: Quick test suite validation

### Phase 2: Validation (Week 2)
- Day 1-2: Standard test suite validation
- Day 3-4: Extended benchmark validation (1hr)
- Day 5-6: Memory stress testing
- Day 7: 24-hour stability test initiation

### Phase 3: Optimization (Week 3)
- Day 1-2: Performance tuning
- Day 3-4: Hardware-specific optimizations
- Day 5-6: Re-validation
- Day 7: Documentation updates

### Phase 4: Production Ready (Week 4)
- Day 1-2: Final validation
- Day 3-4: Production deployment procedures
- Day 5-6: Monitoring setup
- Day 7: Production readiness sign-off

## Support and Troubleshooting

### Getting Help

**Community Resources:**
- GitHub Issues: [repository]/issues
- Documentation: docs/guides/
- Test Results: docs/results/

**Debug Information to Collect:**
- Serial console log (full boot sequence)
- Kernel version and build features
- Hardware platform details
- UEFI firmware version
- Test results and logs

### Common Issues and Solutions

**Issue: Slow Performance**
- Check timer frequency (CNTFRQ_EL0)
- Verify NEON is enabled
- Review memory configuration
- Check for excessive interrupts

**Issue: Instability**
- Review memory allocation patterns
- Check for race conditions
- Verify interrupt handling
- Review autonomous decision rate

**Issue: Compliance Failures**
- Verify all safety features enabled
- Check audit log integrity
- Review watchdog configuration
- Validate hard limits

## Next Steps

After successful hardware deployment:

1. **Performance Baseline:** Establish hardware-specific performance baselines
2. **Long-Term Validation:** Run 7-day, 30-day stability tests
3. **Production Deployment:** Deploy to production environment with monitoring
4. **Continuous Improvement:** Iterate based on operational data

## References

- [Automated Testing Guide](AUTOMATED-TESTING-EXPECT.md)
- [Extended Testing Guide](EXTENDED-TESTING.md)
- [Phase 4 Testing Guide](PHASE4-TESTING-GUIDE.md)
- [Manual Testing Checklist](MANUAL-TESTING-CHECKLIST.md)

---

**Last Updated:** November 4, 2025
**Document Version:** 1.0
**Project Phase:** Phase 4 Week 2 - Hardware Readiness
