//! Platform abstraction layer for hardware-neutral bring-up.
//! Provides device descriptors and memory ranges. Default implementation targets QEMU virt.

#![allow(dead_code)]

/// UART descriptor
#[derive(Copy, Clone)]
pub struct UartDesc {
    pub base: usize,
    pub clock_hz: u32,
}

/// GICv3 descriptor
#[derive(Copy, Clone)]
pub struct GicDesc {
    pub gicd: usize,
    pub gicr: usize,
}

/// Generic timer descriptor
#[derive(Copy, Clone)]
pub struct TimerDesc {
    pub freq_hz: u64,
}

/// MMIO range descriptor
#[derive(Copy, Clone)]
pub struct MmioRange {
    pub start: usize,
    pub size: usize,
    pub device: bool,
}

/// RAM range descriptor
#[derive(Copy, Clone)]
pub struct RamRange {
    pub start: usize,
    pub size: usize,
}

/// Platform trait provides device descriptors and ranges.
pub trait Platform {
    fn uart(&self) -> UartDesc;
    fn gic(&self) -> GicDesc;
    fn timer(&self) -> TimerDesc;
    fn mmio_ranges(&self) -> &'static [MmioRange];
    fn ram_ranges(&self) -> &'static [RamRange];
    fn psci_available(&self) -> bool { false }
    /// Optional hint for VirtIO MMIO layout: (base, per-device size, irq_base)
    fn virtio_mmio_hint(&self) -> Option<(usize, usize, u32)> { None }
}

pub mod qemu_virt;
pub mod dt;

/// Return the active platform implementation. For now, default to QEMU virt.
static mut ACTIVE_OVERRIDE: Option<&'static dyn Platform> = None;

#[allow(static_mut_refs)]
pub fn active() -> &'static dyn Platform {
    unsafe { ACTIVE_OVERRIDE.unwrap_or(&qemu_virt::INSTANCE) }
}

/// Try to override the active platform by parsing a provided DTB pointer.
/// Returns true on success. Safe to call multiple times; subsequent calls are ignored once set.
#[allow(static_mut_refs)]
pub unsafe fn override_with_dtb(dtb_ptr: *const u8) -> bool {
    if ACTIVE_OVERRIDE.is_some() { return true; }
    if let Some(p) = dt::from_dtb(dtb_ptr) {
        ACTIVE_OVERRIDE = Some(p);
        true
    } else { false }
}
