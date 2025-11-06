// AArch64 architecture-specific code
// Phase A0 - Exception handling and syscall infrastructure

pub mod trap;
pub mod timer;

// TODO: Add in later phases:
// pub mod psci;    // SMP CPU bring-up (Phase E)
// pub mod gicv3;   // Interrupt controller (Phase E)
// pub mod mmu;     // Page tables, TLB (Phase A1)

pub use trap::*;

/// CPU context for context switching
/// Contains callee-saved registers that must be preserved across function calls
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,  // FP
    pub x30: u64,  // LR
    pub sp: u64,
}

impl CpuContext {
    /// Create a new empty context
    pub const fn new() -> Self {
        Self {
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            x29: 0,
            x30: 0,
            sp: 0,
        }
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

extern "C" {
    /// Context switch function (implemented in switch.S)
    /// Saves current context to prev, restores from next
    pub fn switch_to(prev: *mut CpuContext, next: *const CpuContext);
}

/// Set ELR_EL1 (Exception Link Register) - return address for ERET
#[inline]
pub fn set_elr_el1(pc: u64) {
    unsafe {
        core::arch::asm!(
            "msr elr_el1, {pc}",
            pc = in(reg) pc,
            options(nostack, preserves_flags)
        );
    }
}

/// Set SPSR_EL1 (Saved Program Status Register) - processor state for ERET
#[inline]
pub fn set_spsr_el1(pstate: u64) {
    unsafe {
        core::arch::asm!(
            "msr spsr_el1, {pstate}",
            pstate = in(reg) pstate,
            options(nostack, preserves_flags)
        );
    }
}

/// Set SP_EL0 (Stack Pointer for EL0)
#[inline]
pub fn set_sp_el0(sp: u64) {
    unsafe {
        core::arch::asm!(
            "msr sp_el0, {sp}",
            sp = in(reg) sp,
            options(nostack, preserves_flags)
        );
    }
}

/// Flush instruction cache for entire system
/// Required after writing executable code to memory
#[inline]
pub fn flush_icache_all() {
    unsafe {
        core::arch::asm!(
            "dsb ish",      // Data Synchronization Barrier
            "ic iallu",     // Invalidate all instruction caches to PoU
            "dsb ish",      // Ensure completion
            "isb",          // Instruction Synchronization Barrier
            options(nostack, preserves_flags)
        );
    }
}

/// Flush instruction cache for a specific address range
/// addr: starting virtual address
/// len: length in bytes
#[inline]
pub fn flush_icache_range(addr: u64, len: usize) {
    // IC IVAU operates on cache lines (typically 64 bytes)
    const CACHE_LINE_SIZE: usize = 64;

    let start = addr & !(CACHE_LINE_SIZE as u64 - 1); // Align down
    let end = (addr + len as u64 + CACHE_LINE_SIZE as u64 - 1) & !(CACHE_LINE_SIZE as u64 - 1);

    unsafe {
        core::arch::asm!("dsb ish", options(nostack, preserves_flags));

        let mut current = start;
        while current < end {
            core::arch::asm!(
                "ic ivau, {addr}",  // Invalidate instruction cache by VA to PoU
                addr = in(reg) current,
                options(nostack, preserves_flags)
            );
            current += CACHE_LINE_SIZE as u64;
        }

        core::arch::asm!(
            "dsb ish",
            "isb",
            options(nostack, preserves_flags)
        );
    }
}

/// Initialize VirtIO block devices (Phase B)
///
/// Probes QEMU virt machine VirtIO MMIO addresses for block devices
pub fn init_virtio_blk() {
    use crate::virtio::{VirtIOMMIOTransport, VirtIODeviceType};
    use crate::drivers::virtio_blk::register_virtio_blk;

    // QEMU virt machine VirtIO MMIO address range
    // Base: 0x0a000000, size: 0x200 (512 bytes), IRQ: 16+N
    const VIRTIO_MMIO_BASE: u64 = 0x0a000000;
    const VIRTIO_MMIO_SIZE: u64 = 0x200;
    const VIRTIO_MMIO_COUNT: usize = 32; // QEMU supports up to 32 VirtIO devices

    let mut blk_count = 0;

    for i in 0..VIRTIO_MMIO_COUNT {
        let base = VIRTIO_MMIO_BASE + (i as u64 * VIRTIO_MMIO_SIZE);
        let irq = Some(16 + i as u32);

        // Try to probe this address
        match VirtIOMMIOTransport::new(base, VIRTIO_MMIO_SIZE, irq) {
            Ok(transport) => {
                // Check if it's a block device
                if transport.device_type() == VirtIODeviceType::BlockDevice {
                    crate::info!("virtio-blk: Found block device at 0x{:x}", base);

                    // Register the block device
                    let name = alloc::format!("vd{}", (b'a' + blk_count as u8) as char);
                    match register_virtio_blk(transport, name.clone()) {
                        Ok(dev) => {
                            crate::info!("virtio-blk: Registered {} ({} MB)",
                                       name, dev.capacity_bytes() / 1024 / 1024);
                            blk_count += 1;
                        }
                        Err(e) => {
                            crate::warn!("virtio-blk: Failed to initialize {}: {:?}", name, e);
                        }
                    }
                }
            }
            Err(_) => {
                // Not a valid VirtIO device at this address, continue
                continue;
            }
        }
    }

    if blk_count == 0 {
        crate::info!("virtio-blk: No block devices found");
    } else {
        crate::info!("virtio-blk: Initialized {} block device(s)", blk_count);
    }
}

/// Initialize virtio-net devices (Phase C)
pub fn init_virtio_net() {
    use crate::virtio::{VirtIOMMIOTransport, VirtIODeviceType};
    use crate::drivers::virtio_net::register_virtio_net;

    // QEMU virt machine VirtIO MMIO address range
    const VIRTIO_MMIO_BASE: u64 = 0x0a000000;
    const VIRTIO_MMIO_SIZE: u64 = 0x200;
    const VIRTIO_MMIO_COUNT: usize = 32;

    let mut net_count = 0;

    for i in 0..VIRTIO_MMIO_COUNT {
        let base = VIRTIO_MMIO_BASE + (i as u64 * VIRTIO_MMIO_SIZE);
        let irq = Some(16 + i as u32);

        // Try to probe this address
        match VirtIOMMIOTransport::new(base, VIRTIO_MMIO_SIZE, irq) {
            Ok(transport) => {
                // Check if it's a network device
                if transport.device_type() == VirtIODeviceType::NetworkCard {
                    crate::info!("virtio-net: Found network device at 0x{:x}", base);

                    // Register the network device
                    let name = alloc::format!("eth{}", net_count);
                    match register_virtio_net(transport, name.clone()) {
                        Ok(dev) => {
                            let mac = dev.mac_address();
                            crate::info!("virtio-net: Registered {} (MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x})",
                                       name, mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
                            net_count += 1;
                        }
                        Err(e) => {
                            crate::warn!("virtio-net: Failed to initialize {}: {:?}", name, e);
                        }
                    }
                }
            }
            Err(_) => {
                // Not a valid VirtIO device at this address, continue
                continue;
            }
        }
    }

    if net_count == 0 {
        crate::info!("virtio-net: No network devices found");
    } else {
        crate::info!("virtio-net: Initialized {} network device(s)", net_count);
    }
}
