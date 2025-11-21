# Core Systems Fix & Completion Plan

**Status**: DRAFT
**Created**: 2025-01-21
**Target**: Fix failing and incomplete core kernel systems
**Priority**: HIGH - Production Readiness Blockers

---

## Executive Summary

Analysis of the boot sequence revealed 9 critical systems that are either failing or incomplete. This plan addresses each system with detailed implementation strategies, prioritized by impact on functionality and user experience.

**Quick Stats:**
- 🔴 **3 Critical Failures**: SMP, Fork, SDHCI
- 🟡 **4 Incomplete Stubs**: GOP/Framebuffer, Watchdog, VirtIO Console, Graphics Stack
- 🟠 **2 Minor Issues**: GIC Priority Mask, Autonomy Confidence

---

## Table of Contents

1. [Critical Failures](#1-critical-failures)
   - [1.1 SMP Multi-Core Support](#11-smp-multi-core-support)
   - [1.2 Fork Syscall](#12-fork-syscall)
   - [1.3 SDHCI Block Device](#13-sdhci-block-device)
2. [Incomplete Implementations](#2-incomplete-implementations)
   - [2.1 Framebuffer/GOP Graphics](#21-framebuffergop-graphics)
   - [2.2 Watchdog Timer](#22-watchdog-timer)
   - [2.3 VirtIO Console Driver](#23-virtio-console-driver)
   - [2.4 Graphics Stack Verification](#24-graphics-stack-verification)
3. [Minor Issues](#3-minor-issues)
   - [3.1 GIC Priority Mask](#31-gic-priority-mask)
   - [3.2 Autonomy Low Confidence](#32-autonomy-low-confidence)
4. [Implementation Phases](#4-implementation-phases)
5. [Testing Strategy](#5-testing-strategy)
6. [Success Criteria](#6-success-criteria)

---

## 1. Critical Failures

### 1.1 SMP Multi-Core Support

**Current Status**: ❌ CRITICAL FAILURE
```
[Warn] SMP: Timeout waiting for CPU 1 to come online
[Warn] SMP: Timeout waiting for CPU 2 to come online
[Warn] SMP: Timeout waiting for CPU 3 to come online
[Warn] SMP: Failed to bring up any secondary CPUs
SMP: 1 CPU(S) ONLINE (should be 4)
```

**Impact**: HIGH
- Only 25% CPU utilization (1 of 4 cores)
- No parallel processing capability
- Scheduler cannot distribute load
- Poor performance on multi-threaded workloads

**Root Cause Analysis**:
1. Secondary CPUs not responding to PSCI CPU_ON calls
2. Possible issues with:
   - CPU entry point address calculation
   - Stack allocation for secondary CPUs
   - Synchronization barriers
   - MPIDR register interpretation
   - GIC redistributor initialization per-CPU

**Implementation Plan**:

#### Phase 1: Diagnostic Enhancement
```rust
// File: crates/kernel/src/arch/aarch64/smp.rs

// Add verbose logging for PSCI calls
fn bring_up_cpu(cpu_id: u64) -> Result<()> {
    let entry_point = secondary_cpu_entry as u64;
    let context_id = cpu_id;

    crate::warn!("SMP: Bringing up CPU {} with entry=0x{:x} context=0x{:x}",
                 cpu_id, entry_point, context_id);

    // Log MPIDR affinity levels
    let mpidr = read_mpidr();
    crate::warn!("SMP: MPIDR for CPU {}: Aff0={} Aff1={} Aff2={} Aff3={}",
                 cpu_id,
                 mpidr & 0xFF,
                 (mpidr >> 8) & 0xFF,
                 (mpidr >> 16) & 0xFF,
                 (mpidr >> 32) & 0xFF);

    let result = psci::cpu_on(cpu_id, entry_point, context_id);
    crate::warn!("SMP: CPU_ON returned: {:?}", result);

    // Poll with timeout
    for attempt in 0..1000 {
        if is_cpu_online(cpu_id) {
            crate::warn!("SMP: CPU {} online after {} attempts", cpu_id, attempt);
            return Ok(());
        }
        delay_ms(10);
    }

    Err(Errno::ETIMEDOUT)
}
```

#### Phase 2: Fix Entry Point
```rust
// File: crates/kernel/src/arch/aarch64/smp.rs

#[naked]
#[no_mangle]
pub unsafe extern "C" fn secondary_cpu_entry() -> ! {
    // Save context_id passed in X0
    asm!(
        // Set up temporary stack
        "mov x19, x0",                    // Save context_id
        "adrp x1, __secondary_stack_end", // Load stack top
        "add x1, x1, :lo12:__secondary_stack_end",
        "mov sp, x1",

        // Enable MMU (inherit from primary CPU's page tables)
        "mrs x2, ttbr0_el1",
        "msr ttbr0_el1, x2",

        // Set SCTLR_EL1.M to enable MMU
        "mrs x3, sctlr_el1",
        "orr x3, x3, #1",
        "msr sctlr_el1, x3",
        "isb",

        // Branch to Rust code
        "mov x0, x19",                    // Restore context_id
        "b secondary_cpu_main",
        options(noreturn)
    );
}

#[no_mangle]
pub extern "C" fn secondary_cpu_main(cpu_id: u64) -> ! {
    // Initialize per-CPU structures
    unsafe {
        // Set up GIC redistributor for this CPU
        gicv3::init_redistributor_for_cpu(cpu_id);

        // Initialize per-CPU timer
        timer::init_per_cpu();

        // Mark CPU as online
        mark_cpu_online(cpu_id);

        crate::info!("CPU {} is now online!", cpu_id);
    }

    // Enter scheduler idle loop
    scheduler::enter_idle_loop()
}
```

#### Phase 3: Per-CPU Stack Allocation
```rust
// File: crates/kernel/src/arch/aarch64/mod.rs

// Allocate stacks for secondary CPUs at boot
static SECONDARY_STACKS: Mutex<Option<Vec<u64>>> = Mutex::new(None);

pub fn init_secondary_cpu_stacks(num_cpus: usize) -> Result<()> {
    const STACK_SIZE: usize = 16 * 1024; // 16KB per CPU

    let mut stacks = Vec::new();
    for cpu in 1..num_cpus {
        let stack = mm::alloc_pages(4)?; // 4 pages = 16KB
        let stack_top = stack + STACK_SIZE as u64;
        stacks.push(stack_top);
        crate::info!("SMP: Allocated stack for CPU {}: 0x{:x}-0x{:x}",
                     cpu, stack, stack_top);
    }

    *SECONDARY_STACKS.lock() = Some(stacks);
    Ok(())
}
```

#### Phase 4: PSCI Verification
```rust
// File: crates/kernel/src/arch/aarch64/psci.rs

pub fn verify_cpu_on_support() -> bool {
    // Check PSCI_FEATURES for CPU_ON support
    let features = smc_call(PSCI_FEATURES, PSCI_CPU_ON, 0, 0);

    if features == PSCI_SUCCESS {
        crate::info!("PSCI: CPU_ON is supported");
        true
    } else {
        crate::warn!("PSCI: CPU_ON not supported, return code: {}", features);
        false
    }
}
```

**Testing**:
1. Boot with `-smp 1` to verify single-core still works
2. Boot with `-smp 2` to test basic SMP
3. Boot with `-smp 4` to test full quad-core
4. Run parallel workload to verify all CPUs active
5. Verify GIC interrupts routed to correct CPUs

**Success Criteria**:
- ✅ All 4 CPUs online in QEMU
- ✅ Load distributed across cores
- ✅ No race conditions in scheduler
- ✅ Proper CPU affinity for interrupts

**Estimated Effort**: 3-5 days
**Priority**: 🔴 P0 - Critical

---

### 1.2 Fork Syscall

**Current Status**: ❌ UNIMPLEMENTED
```
[TEST] Testing unimplemented fork syscall...
[TEST] Fork syscall returned unexpected result
```

**Impact**: HIGH
- Cannot create child processes
- No process isolation for user programs
- Shell cannot spawn background tasks
- Blocks Unix-like process model

**Root Cause**: Stub implementation, address space duplication not implemented

**Implementation Plan**:

#### Phase 1: Address Space Duplication
```rust
// File: crates/kernel/src/process/mod.rs

impl Process {
    pub fn fork(&self) -> Result<Arc<Process>> {
        // Create new process structure
        let child_pid = allocate_pid()?;
        let mut child = Process::new(child_pid);

        // Copy address space
        let parent_mm = self.mm.lock();
        let mut child_mm = child.mm.lock();

        // Clone page tables
        for vma in parent_mm.vmas() {
            let child_vma = vma.clone_for_fork()?;

            if vma.is_writable() {
                // Mark both parent and child as COW (copy-on-write)
                vma.mark_copy_on_write()?;
                child_vma.mark_copy_on_write()?;
            } else {
                // Share read-only pages
                child_vma.share_pages_from(vma)?;
            }

            child_mm.add_vma(child_vma);
        }

        // Copy file descriptor table
        child.fd_table = self.fd_table.clone();

        // Copy signal handlers
        child.signals = self.signals.clone();

        // Set parent/child relationship
        child.parent_pid = Some(self.pid);
        self.children.lock().push(child_pid);

        // Copy registers for context
        child.context = self.context.clone();
        child.context.set_return_value(0); // Child returns 0

        crate::info!("FORK: Created child process {} from parent {}",
                     child_pid, self.pid);

        let child_arc = Arc::new(child);
        PROCESS_TABLE.lock().insert(child_pid, child_arc.clone());

        Ok(child_arc)
    }
}
```

#### Phase 2: Copy-on-Write (COW) Page Handling
```rust
// File: crates/kernel/src/mm/vma.rs

impl VirtualMemoryArea {
    pub fn mark_copy_on_write(&mut self) -> Result<()> {
        // Clear writable bit, set COW flag
        let page_table = get_current_page_table();

        for page_addr in self.start..self.end {
            let pte = page_table.get_pte_mut(page_addr)?;

            if pte.is_writable() {
                pte.clear_writable(); // Make read-only
                pte.set_cow_flag();   // Mark as COW

                // Increment reference count for physical page
                let phys_addr = pte.physical_address();
                mm::increment_page_refcount(phys_addr);
            }
        }

        // Flush TLB for this VMA
        tlb_flush_range(self.start, self.end);

        Ok(())
    }

    pub fn handle_cow_fault(&mut self, fault_addr: u64) -> Result<()> {
        // Allocate new physical page
        let new_page = mm::alloc_page()?;

        // Copy contents from old page
        let old_pte = get_current_page_table().get_pte(fault_addr)?;
        let old_phys = old_pte.physical_address();

        unsafe {
            core::ptr::copy_nonoverlapping(
                old_phys as *const u8,
                new_page as *mut u8,
                PAGE_SIZE
            );
        }

        // Update PTE to point to new page, restore writable
        let pte = get_current_page_table().get_pte_mut(fault_addr)?;
        pte.set_physical_address(new_page);
        pte.set_writable();
        pte.clear_cow_flag();

        // Decrement refcount on old page
        mm::decrement_page_refcount(old_phys);

        // Flush TLB entry
        tlb_flush_single(fault_addr);

        crate::debug!("COW: Handled fault at 0x{:x}, old_phys=0x{:x} new_phys=0x{:x}",
                      fault_addr, old_phys, new_page);

        Ok(())
    }
}
```

#### Phase 3: Page Fault Handler Integration
```rust
// File: crates/kernel/src/arch/aarch64/exceptions.rs

fn handle_data_abort(esr: u64, far: u64) {
    let fault_addr = far;
    let current = current_process();

    // Check if this is a COW fault
    if let Some(vma) = current.mm.lock().find_vma(fault_addr) {
        if vma.is_cow_page(fault_addr) {
            match vma.handle_cow_fault(fault_addr) {
                Ok(()) => return, // COW handled successfully
                Err(e) => {
                    crate::error!("COW fault handling failed: {:?}", e);
                }
            }
        }
    }

    // Not a COW fault, handle as regular page fault
    handle_page_fault(fault_addr, esr);
}
```

#### Phase 4: Syscall Implementation
```rust
// File: crates/kernel/src/syscall/process.rs

pub fn sys_fork() -> Result<usize> {
    let current = current_process();

    crate::debug!("FORK: Process {} calling fork()", current.pid);

    // Create child process
    let child = current.fork()?;
    let child_pid = child.pid;

    // Add child to scheduler
    scheduler::add_process(child.clone());

    // Parent returns child PID
    crate::info!("FORK: Parent {} created child {}", current.pid, child_pid);
    Ok(child_pid as usize)
}
```

**Testing**:
```rust
#[test]
fn test_fork() {
    let parent = Process::new(1);
    let child = parent.fork().unwrap();

    assert_ne!(parent.pid, child.pid);
    assert_eq!(child.parent_pid, Some(parent.pid));
    assert_eq!(parent.fd_table.len(), child.fd_table.len());
}

#[test]
fn test_cow() {
    let parent = Process::new(1);
    let addr = 0x10000;

    // Write to parent's memory
    parent.write_memory(addr, &[1, 2, 3, 4]);

    // Fork
    let child = parent.fork().unwrap();

    // Verify pages are shared (same physical address)
    assert_eq!(parent.get_physical_addr(addr), child.get_physical_addr(addr));

    // Write to child's memory (should trigger COW)
    child.write_memory(addr, &[5, 6, 7, 8]);

    // Verify pages are now different
    assert_ne!(parent.get_physical_addr(addr), child.get_physical_addr(addr));

    // Verify contents are different
    assert_eq!(parent.read_memory(addr, 4), &[1, 2, 3, 4]);
    assert_eq!(child.read_memory(addr, 4), &[5, 6, 7, 8]);
}
```

**Success Criteria**:
- ✅ Fork creates child process with unique PID
- ✅ Child has copy of parent's address space
- ✅ COW works correctly (pages shared until write)
- ✅ File descriptors properly duplicated
- ✅ Shell can spawn background processes

**Estimated Effort**: 4-6 days
**Priority**: 🔴 P0 - Critical

---

### 1.3 SDHCI Block Device

**Current Status**: ❌ FAILED
```
[Warn] Block: Failed to initialize SDHCI: ENODEV
```

**Impact**: MEDIUM
- No SD card storage access
- Cannot boot from SD card on real hardware
- Limits storage options in embedded scenarios

**Root Cause**: SDHCI controller not detected in QEMU (QEMU uses VirtIO-blk instead)

**Implementation Plan**:

#### Strategy: Dual Approach
1. **For QEMU**: Use VirtIO-blk (already working)
2. **For Hardware**: Implement SDHCI driver for Raspberry Pi, etc.

#### Phase 1: SDHCI Detection
```rust
// File: crates/kernel/src/drivers/sdhci.rs

pub struct SdhciController {
    base_addr: u64,
    version: u16,
    capabilities: SdhciCapabilities,
    clock_freq: u32,
}

impl SdhciController {
    pub fn probe(base_addr: u64) -> Result<Self> {
        // Read version register
        let version = unsafe {
            ptr::read_volatile((base_addr + SDHCI_HOST_VERSION) as *const u16)
        };

        if version == 0 || version == 0xFFFF {
            return Err(Errno::ENODEV);
        }

        crate::info!("SDHCI: Found controller at 0x{:x}, version 0x{:x}",
                     base_addr, version);

        // Read capabilities
        let caps = Self::read_capabilities(base_addr)?;

        Ok(Self {
            base_addr,
            version,
            capabilities: caps,
            clock_freq: 0, // Will be set during init
        })
    }

    pub fn init(&mut self) -> Result<()> {
        // Reset controller
        self.reset()?;

        // Set up clocks
        self.set_clock(400_000)?; // Start at 400kHz for initialization

        // Configure voltage
        self.set_bus_voltage()?;

        // Enable interrupts
        self.enable_interrupts()?;

        crate::info!("SDHCI: Controller initialized successfully");
        Ok(())
    }
}
```

#### Phase 2: SD Card Initialization
```rust
// File: crates/kernel/src/drivers/sdhci.rs

impl SdhciController {
    pub fn init_card(&mut self) -> Result<SdCard> {
        // Send CMD0 (GO_IDLE_STATE)
        self.send_command(0, 0, ResponseType::None)?;

        // Send CMD8 (SEND_IF_COND) for SD 2.0
        let response = self.send_command(8, 0x1AA, ResponseType::R7)?;
        let sd_version = if response == 0x1AA {
            SdVersion::V2_0
        } else {
            SdVersion::V1_0
        };

        // ACMD41 (SD_SEND_OP_COND) - wait for card to be ready
        let mut retries = 1000;
        loop {
            self.send_command(55, 0, ResponseType::R1)?; // CMD55 (APP_CMD)
            let response = self.send_command(41, 0x40FF8000, ResponseType::R3)?;

            if response & 0x80000000 != 0 {
                // Card is ready
                let card_capacity = if response & 0x40000000 != 0 {
                    CardCapacity::HighCapacity // SDHC/SDXC
                } else {
                    CardCapacity::Standard      // SDSC
                };
                break;
            }

            retries -= 1;
            if retries == 0 {
                return Err(Errno::ETIMEDOUT);
            }
            delay_ms(1);
        }

        // Get CID (Card Identification)
        self.send_command(2, 0, ResponseType::R2)?;

        // Get RCA (Relative Card Address)
        let rca_response = self.send_command(3, 0, ResponseType::R6)?;
        let rca = (rca_response >> 16) as u16;

        // Select card
        self.send_command(7, (rca as u32) << 16, ResponseType::R1b)?;

        // Set bus width to 4-bit
        self.send_command(55, (rca as u32) << 16, ResponseType::R1)?;
        self.send_command(6, 0x02, ResponseType::R1)?; // ACMD6

        // Increase clock to high speed
        self.set_clock(50_000_000)?; // 50 MHz

        crate::info!("SDHCI: SD card initialized, RCA=0x{:x}, type={:?}",
                     rca, card_capacity);

        Ok(SdCard {
            rca,
            capacity: card_capacity,
            version: sd_version,
        })
    }
}
```

#### Phase 3: Block Device Interface
```rust
// File: crates/kernel/src/drivers/sdhci.rs

impl BlockDevice for SdhciController {
    fn read_block(&self, block_num: u64, buffer: &mut [u8]) -> Result<usize> {
        if buffer.len() < 512 {
            return Err(Errno::EINVAL);
        }

        // CMD17 (READ_SINGLE_BLOCK)
        self.send_command(17, block_num as u32, ResponseType::R1)?;

        // Wait for data ready
        self.wait_for_data_ready()?;

        // Read data from buffer
        let data_port = self.base_addr + SDHCI_BUFFER;
        for i in (0..512).step_by(4) {
            let word = unsafe {
                ptr::read_volatile(data_port as *const u32)
            };
            buffer[i..i+4].copy_from_slice(&word.to_le_bytes());
        }

        Ok(512)
    }

    fn write_block(&self, block_num: u64, buffer: &[u8]) -> Result<usize> {
        if buffer.len() < 512 {
            return Err(Errno::EINVAL);
        }

        // CMD24 (WRITE_BLOCK)
        self.send_command(24, block_num as u32, ResponseType::R1)?;

        // Wait for buffer ready
        self.wait_for_buffer_ready()?;

        // Write data to buffer
        let data_port = self.base_addr + SDHCI_BUFFER;
        for i in (0..512).step_by(4) {
            let word = u32::from_le_bytes([
                buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]
            ]);
            unsafe {
                ptr::write_volatile(data_port as *mut u32, word);
            }
        }

        // Wait for transfer complete
        self.wait_for_transfer_complete()?;

        Ok(512)
    }

    fn capacity(&self) -> u64 {
        // Return card capacity in bytes
        // This should be read from CSD register
        0
    }
}
```

#### Phase 4: Conditional Compilation
```rust
// File: crates/kernel/src/drivers/mod.rs

#[cfg(feature = "sdhci")]
pub mod sdhci;

pub fn init_block_devices() {
    // Try VirtIO-blk first (for QEMU)
    match virtio_blk::probe_devices() {
        Ok(count) if count > 0 => {
            crate::info!("BLOCK: Using VirtIO-blk ({} devices)", count);
            return;
        }
        _ => {}
    }

    // Try SDHCI (for real hardware)
    #[cfg(feature = "sdhci")]
    {
        if let Ok(controller) = sdhci::SdhciController::probe(BCM2711_EMMC_BASE) {
            crate::info!("BLOCK: Using SDHCI controller");
            return;
        }
    }

    crate::warn!("BLOCK: No block devices found");
}
```

**Testing**:
1. Test with QEMU (should use VirtIO-blk)
2. Test with Raspberry Pi (should use SDHCI)
3. Read/write tests for both drivers
4. Stress test with large file I/O

**Success Criteria**:
- ✅ SDHCI driver compiles with feature flag
- ✅ SD card detection works on RPi
- ✅ Read/write operations succeed
- ✅ VirtIO-blk still works in QEMU

**Estimated Effort**: 5-7 days
**Priority**: 🟡 P1 - Medium (QEMU works with VirtIO-blk)

---

## 2. Incomplete Implementations

### 2.1 Framebuffer/GOP Graphics

**Current Status**: 🚧 STUB
```
Querying GOP for framebuffer...
GOP protocol not available
```

**Impact**: MEDIUM
- No graphical output to screen
- UART serial console only
- Poor user experience on real hardware

**Root Cause**: QEMU's UEFI firmware doesn't provide GOP in headless mode

**Implementation Plan**:

#### Strategy: Multi-Backend Approach
1. **UEFI GOP** - For machines with UEFI graphics
2. **VirtIO-GPU** - For QEMU with GPU device
3. **BCM2711 MailBox** - For Raspberry Pi 4/5 direct framebuffer

#### Phase 1: GOP Integration (Already Partially Done)
```rust
// File: crates/kernel/src/arch/aarch64/framebuffer.rs
// (Already exists, needs enhancement)

// Add fallback if GOP not available
pub fn init(boot_info: &BootInfo) {
    if boot_info.framebuffer_addr != 0 {
        // UEFI provided framebuffer
        init_from_uefi(boot_info);
    } else {
        // Try VirtIO-GPU
        if let Ok(fb) = virtio_gpu::get_framebuffer() {
            init_from_virtio_gpu(fb);
        } else {
            crate::warn!("Framebuffer: No display available, using serial only");
        }
    }
}
```

#### Phase 2: VirtIO-GPU Backend
```rust
// File: crates/kernel/src/drivers/virtio_gpu.rs

pub struct VirtioGpu {
    transport: Arc<Mutex<VirtIOMMIOTransport>>,
    control_queue: Arc<Mutex<VirtQueue>>,
    cursor_queue: Arc<Mutex<VirtQueue>>,
    framebuffer: Mutex<Option<Framebuffer>>,
}

impl VirtioGpu {
    pub fn init() -> Result<Self> {
        // Probe for VirtIO-GPU device (device ID 16)
        let transport = probe_virtio_device(VirtIODeviceType::Gpu)?;

        // Initialize queues
        let control_queue = VirtQueue::new(0, 256)?;
        let cursor_queue = VirtQueue::new(1, 16)?;

        let gpu = Self {
            transport: Arc::new(Mutex::new(transport)),
            control_queue: Arc::new(Mutex::new(control_queue)),
            cursor_queue: Arc::new(Mutex::new(cursor_queue)),
            framebuffer: Mutex::new(None),
        };

        // Create 2D resource
        gpu.create_2d_resource()?;

        // Get scanout info
        let scanout = gpu.get_display_info()?;

        crate::info!("VirtIO-GPU: Display {}x{}", scanout.width, scanout.height);

        Ok(gpu)
    }

    pub fn get_framebuffer(&self) -> Option<Framebuffer> {
        self.framebuffer.lock().clone()
    }

    fn create_2d_resource(&self) -> Result<()> {
        // Send VIRTIO_GPU_CMD_RESOURCE_CREATE_2D command
        // ...
        Ok(())
    }
}
```

#### Phase 3: Raspberry Pi Mailbox Backend
```rust
// File: crates/kernel/src/arch/aarch64/rpi_mailbox.rs

pub struct RpiMailbox {
    base_addr: u64,
}

impl RpiMailbox {
    const MAILBOX_BASE: u64 = 0xFE00B880; // BCM2711

    pub fn new() -> Self {
        Self { base_addr: Self::MAILBOX_BASE }
    }

    pub fn allocate_framebuffer(&self, width: u32, height: u32, depth: u32)
        -> Result<Framebuffer>
    {
        // Mailbox property interface
        let mut message = [
            35 * 4,                    // Total size
            0,                         // Request code

            // Tag: Set physical display size
            0x00048003,
            8,                         // Value buffer size
            0,                         // Request
            width,
            height,

            // Tag: Set virtual display size
            0x00048004,
            8,
            0,
            width,
            height,

            // Tag: Set depth
            0x00048005,
            4,
            0,
            depth,

            // Tag: Allocate buffer
            0x00040001,
            8,
            0,
            16,                        // Alignment
            0,

            0                          // End tag
        ];

        self.call(8, &mut message)?;

        let fb_addr = message[28] & 0x3FFFFFFF; // Remove VC/ARM address bit
        let fb_size = message[29];

        crate::info!("RPi: Framebuffer allocated at 0x{:x}, size {} bytes",
                     fb_addr, fb_size);

        Ok(Framebuffer {
            addr: fb_addr as u64,
            width,
            height,
            pitch: width * (depth / 8),
            depth,
        })
    }
}
```

**Testing**:
1. Test with QEMU + VirtIO-GPU enabled
2. Test on Raspberry Pi with mailbox
3. Test drawing primitives (lines, rectangles, text)
4. Test scrolling and clearing

**Success Criteria**:
- ✅ Framebuffer available in at least one backend
- ✅ Text rendering works
- ✅ Basic graphics primitives work
- ✅ Graceful fallback to serial if no display

**Estimated Effort**: 3-5 days
**Priority**: 🟡 P1 - Medium

---

### 2.2 Watchdog Timer

**Current Status**: 🚧 STUB
```
WATCHDOG: INIT
WATCHDOG: NONE AVAILABLE
```

**Impact**: LOW
- No automatic recovery from hangs
- System may freeze without detection
- Useful for production reliability

**Implementation Plan**:

#### Phase 1: ARM Generic Watchdog
```rust
// File: crates/kernel/src/drivers/watchdog.rs

pub struct ArmWatchdog {
    enabled: AtomicBool,
    timeout_ms: AtomicU32,
}

impl ArmWatchdog {
    pub fn init() -> Result<Self> {
        // Check if EL1 physical timer is available
        let id_aa64pfr0 = read_id_aa64pfr0_el1();

        if (id_aa64pfr0 >> 16) & 0xF == 0 {
            return Err(Errno::ENODEV);
        }

        crate::info!("Watchdog: Using ARM generic timer");

        Ok(Self {
            enabled: AtomicBool::new(false),
            timeout_ms: AtomicU32::new(5000), // Default 5s
        })
    }

    pub fn start(&self, timeout_ms: u32) {
        self.timeout_ms.store(timeout_ms, Ordering::Relaxed);

        // Set timer
        let ticks = (timeout_ms as u64 * cntfrq_hz()) / 1000;
        write_cntp_tval_el0(ticks as u32);

        // Enable timer interrupt
        write_cntp_ctl_el0(ENABLE | IMASK_OFF);

        self.enabled.store(true, Ordering::Release);

        crate::info!("Watchdog: Started with {}ms timeout", timeout_ms);
    }

    pub fn kick(&self) {
        if !self.enabled.load(Ordering::Acquire) {
            return;
        }

        // Reset timer
        let timeout_ms = self.timeout_ms.load(Ordering::Relaxed);
        let ticks = (timeout_ms as u64 * cntfrq_hz()) / 1000;
        write_cntp_tval_el0(ticks as u32);
    }

    pub fn handle_timeout(&self) {
        crate::error!("WATCHDOG: System hang detected! Rebooting...");

        // Attempt reboot via PSCI
        psci::system_reset();

        // If PSCI fails, spin forever
        loop {
            core::hint::spin_loop();
        }
    }
}
```

**Success Criteria**:
- ✅ Watchdog starts and counts down
- ✅ Kicking watchdog resets timer
- ✅ Timeout triggers reboot
- ✅ Can be disabled for debugging

**Estimated Effort**: 1-2 days
**Priority**: 🟢 P2 - Low

---

### 2.3 VirtIO Console Driver

**Current Status**: 🚧 DISABLED
```
DRIVER FRAMEWORK: SKIPPED (virtio-console feature off)
```

**Impact**: LOW
- No host-guest communication channel
- Cannot send commands from host to kernel
- Useful for automation and testing

**Implementation Plan**:

#### Phase 1: Enable Feature & Basic Driver
```rust
// File: crates/kernel/src/drivers/virtio_console.rs

pub struct VirtioConsole {
    transport: Arc<Mutex<VirtIOMMIOTransport>>,
    rx_queue: Arc<Mutex<VirtQueue>>,
    tx_queue: Arc<Mutex<VirtQueue>>,
    port_name: String,
}

impl VirtioConsole {
    pub fn new(transport: VirtIOMMIOTransport, port_name: String) -> Result<Self> {
        // Similar to virtio-net setup
        let rx_queue = VirtQueue::new(0, 256)?;
        let tx_queue = VirtQueue::new(1, 256)?;

        let console = Self {
            transport: Arc::new(Mutex::new(transport)),
            rx_queue: Arc::new(Mutex::new(rx_queue)),
            tx_queue: Arc::new(Mutex::new(tx_queue)),
            port_name,
        };

        // Pre-fill RX buffers
        console.refill_rx_buffers()?;

        crate::info!("VirtIO-Console: Port '{}' ready", port_name);

        Ok(console)
    }

    pub fn send(&self, data: &[u8]) -> Result<()> {
        // Similar to virtio-net TX
        Ok(())
    }

    pub fn receive(&self) -> Option<Vec<u8>> {
        // Similar to virtio-net RX
        None
    }
}
```

**Success Criteria**:
- ✅ Console device detected
- ✅ Can send/receive data from host
- ✅ Useful for automated testing

**Estimated Effort**: 2-3 days
**Priority**: 🟢 P2 - Low

---

### 2.4 Graphics Stack Verification

**Current Status**: ❓ UNCLEAR
```
GRAPHICS: TEST PASSED
WM: TEST PASSED
UI: TEST PASSED
```

**Impact**: UNKNOWN
- Tests pass but actual functionality unclear
- May be mock/stub implementations
- Need to verify real graphics capability

**Investigation Needed**:
1. Check if graphics tests are stubs or real
2. Verify window manager functionality
3. Test UI toolkit with actual rendering
4. Determine if GPU acceleration works

**Action Items**:
```bash
# Investigate graphics test implementation
grep -r "GRAPHICS: TEST" crates/kernel/src/
grep -r "WM: TEST" crates/kernel/src/
grep -r "UI: TEST" crates/kernel/src/

# Check for actual GPU/framebuffer usage
grep -r "virtio_gpu" crates/kernel/src/
grep -r "framebuffer" crates/kernel/src/
```

**Priority**: 🟡 P1 - Need Investigation

---

## 3. Minor Issues

### 3.1 GIC Priority Mask

**Current Status**: ⚠️ WORKING WITH LIMITATION
```
[GIC] ICC_PMR_EL1 final: 248 (WARNING: Stuck at 0xF8! Will allow priorities 0-0xF7)
```

**Impact**: LOW
- May miss very low priority interrupts (248-255)
- Functional for most use cases
- Not a blocker

**Investigation**:
```rust
// File: crates/kernel/src/arch/aarch64/gicv3.rs

// Try writing with different patterns
pub fn debug_icc_pmr() {
    let patterns = [0xFF, 0xF0, 0xE0, 0x00, 0x80, 0xA0];

    for &value in &patterns {
        unsafe {
            asm!("msr ICC_PMR_EL1, {}", in(reg) value);
            asm!("isb");

            let readback: u64;
            asm!("mrs {}, ICC_PMR_EL1", out(reg) readback);

            crate::warn!("ICC_PMR_EL1: wrote 0x{:x}, read 0x{:x}", value, readback);
        }
    }
}
```

**Priority**: 🟢 P3 - Low

---

### 3.2 Autonomy Low Confidence

**Current Status**: ⚠️ EXPECTED BEHAVIOR
```
[AUTONOMY] Low confidence (0 < 600), deferring action: Model still initializing (< 10 decisions recorded)
```

**Impact**: NONE
- This is normal during boot
- Confidence increases over time
- Not a bug

**No Action Required**

---

## 4. Implementation Phases

### Phase 1: Critical Blockers (Week 1-2)
**Goal**: Fix systems preventing core functionality

1. **SMP Multi-Core** (5 days)
   - Day 1-2: Diagnostic enhancement
   - Day 3-4: Fix entry point and per-CPU initialization
   - Day 5: Testing and validation

2. **Fork Syscall** (6 days)
   - Day 1-2: Address space duplication
   - Day 3-4: COW implementation
   - Day 5-6: Testing and integration

**Deliverables**:
- All 4 CPUs online in QEMU
- Fork syscall working with COW
- Shell can spawn child processes

---

### Phase 2: Storage & Display (Week 3)
**Goal**: Improve I/O and user experience

3. **Framebuffer/Display** (5 days)
   - Day 1-2: VirtIO-GPU backend
   - Day 3-4: RPi mailbox backend
   - Day 5: Testing and integration

4. **SDHCI Driver** (3 days)
   - Day 1-2: Basic SDHCI implementation
   - Day 3: Testing on hardware

**Deliverables**:
- Graphics output working (QEMU or RPi)
- SD card support on real hardware

---

### Phase 3: Polish & Reliability (Week 4)
**Goal**: Add production features

5. **Watchdog Timer** (2 days)
6. **VirtIO Console** (2 days)
7. **GIC Priority Mask Investigation** (1 day)
8. **Graphics Stack Verification** (2 days)

**Deliverables**:
- Watchdog protecting against hangs
- Host communication via console
- All systems validated

---

## 5. Testing Strategy

### Unit Tests
```rust
// Per-system unit tests
#[cfg(test)]
mod tests {
    #[test]
    fn test_smp_initialization() { }

    #[test]
    fn test_fork_creates_child() { }

    #[test]
    fn test_cow_on_write() { }

    #[test]
    fn test_sdhci_detection() { }
}
```

### Integration Tests
```rust
// Cross-system integration tests
#[test]
fn test_multicore_scheduling() {
    // Spawn tasks on all cores
    // Verify they run in parallel
}

#[test]
fn test_fork_and_exec() {
    // Fork then exec new program
    // Verify child runs independently
}
```

### Hardware Tests
- Boot on Raspberry Pi 4/5
- Verify all 4 cores online
- Test SD card read/write
- Test display output

### Stress Tests
- Fork bomb protection
- Multi-core load test
- Disk I/O stress
- Graphics rendering stress

---

## 6. Success Criteria

### Critical Success Metrics

#### SMP
- [ ] All 4 CPUs online in QEMU
- [ ] Scheduler distributes load across cores
- [ ] No race conditions under stress
- [ ] GIC interrupts routed correctly
- [ ] Per-CPU data structures working

#### Fork/COW
- [ ] Fork creates child process
- [ ] Child has independent address space
- [ ] COW triggers on write
- [ ] Page sharing works correctly
- [ ] File descriptors duplicated

#### Storage
- [ ] VirtIO-blk works in QEMU
- [ ] SDHCI works on Raspberry Pi
- [ ] Read/write operations succeed
- [ ] File system operations work

#### Display
- [ ] Framebuffer initialized
- [ ] Text rendering works
- [ ] Graphics primitives work
- [ ] Window manager usable

### Non-Functional Metrics
- [ ] Boot time < 5 seconds
- [ ] Fork latency < 10ms
- [ ] Context switch latency < 2µs (per-core)
- [ ] No memory leaks in stress tests
- [ ] System stable under load

---

## 7. Risk Assessment

### High Risk
🔴 **SMP Initialization**
- Risk: Complex synchronization issues
- Mitigation: Extensive logging, staged rollout
- Fallback: Disable SMP if unstable

🔴 **COW Implementation**
- Risk: Page table corruption
- Mitigation: Comprehensive testing
- Fallback: Eager copy without COW

### Medium Risk
🟡 **Hardware-Specific Drivers**
- Risk: No hardware for testing
- Mitigation: Emulator + community testing
- Fallback: VirtIO-only for now

### Low Risk
🟢 **Display Backends**
- Risk: Multiple implementations to maintain
- Mitigation: Common abstraction layer
- Fallback: Serial console always works

---

## 8. Dependencies

### External Dependencies
- QEMU 8.0+ with SMP and VirtIO support
- Raspberry Pi 4/5 firmware for testing
- UEFI firmware with GOP support

### Internal Dependencies
- Page allocator must handle COW refcounts
- Scheduler must support multi-core
- VFS must work with SDHCI driver
- GIC must route interrupts per-CPU

---

## 9. Rollout Plan

### Stage 1: Development (Weeks 1-3)
- Implement features in feature branches
- Run unit tests continuously
- Integrate to main branch incrementally

### Stage 2: Testing (Week 4)
- Run full test suite
- Hardware validation
- Stress testing
- Performance benchmarking

### Stage 3: Documentation (Week 4-5)
- Update README with new capabilities
- Document known issues
- Create user guide for features
- Write developer notes

### Stage 4: Release (Week 5)
- Tag release version
- Update changelog
- Announce new features
- Gather community feedback

---

## 10. Open Questions

1. **SMP**: Should we support CPU hotplug?
2. **Fork**: Do we need vfork() for performance?
3. **Storage**: Priority for NVMe driver?
4. **Display**: Hardware acceleration needed?
5. **Watchdog**: Should timeout be configurable at runtime?

---

## Appendix A: Code Locations

### Files to Create
- `crates/kernel/src/process/fork.rs` - Fork implementation
- `crates/kernel/src/mm/cow.rs` - Copy-on-write logic
- `crates/kernel/src/drivers/sdhci.rs` - SDHCI driver
- `crates/kernel/src/drivers/virtio_gpu.rs` - VirtIO-GPU driver
- `crates/kernel/src/arch/aarch64/rpi_mailbox.rs` - RPi framebuffer
- `crates/kernel/src/drivers/watchdog.rs` - Watchdog driver

### Files to Modify
- `crates/kernel/src/arch/aarch64/smp.rs` - SMP fixes
- `crates/kernel/src/arch/aarch64/gicv3.rs` - Per-CPU GIC init
- `crates/kernel/src/mm/page_table.rs` - COW support
- `crates/kernel/src/syscall/process.rs` - Fork syscall
- `crates/kernel/src/drivers/mod.rs` - Block device fallback

---

## Appendix B: Resources

### Documentation
- [ARM Architecture Reference Manual (ARMv8)](https://developer.arm.com/documentation/ddi0487/latest)
- [VirtIO Specification v1.1](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html)
- [SDHCI Simplified Specification](https://www.sdcard.org/downloads/pls/)
- [Raspberry Pi BCM2711 Datasheet](https://datasheets.raspberrypi.com/bcm2711/bcm2711-peripherals.pdf)

### Reference Implementations
- Linux kernel: `arch/arm64/kernel/smp.c`
- Linux kernel: `kernel/fork.c`
- Linux kernel: `drivers/mmc/host/sdhci.c`
- Redox OS: `kernel/src/context/switch.rs`

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-01-21 | Claude Code | Initial draft |

---

**Next Steps**: Review this plan and prioritize which system to tackle first. Recommend starting with **SMP multi-core** as it has the highest impact on performance and enables true parallelism.
