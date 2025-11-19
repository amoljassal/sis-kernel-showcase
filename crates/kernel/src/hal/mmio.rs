//! Typed MMIO accessors and memory barrier helpers

#[inline(always)]
pub unsafe fn read32(addr: *const u32) -> u32 {
    core::ptr::read_volatile(addr)
}

#[inline(always)]
pub unsafe fn write32(addr: *mut u32, val: u32) {
    core::ptr::write_volatile(addr, val)
}

#[inline(always)]
pub unsafe fn read16(addr: *const u16) -> u16 { core::ptr::read_volatile(addr) }

#[inline(always)]
pub unsafe fn write16(addr: *mut u16, val: u16) { core::ptr::write_volatile(addr, val) }

#[inline(always)]
pub unsafe fn read8(addr: *const u8) -> u8 { core::ptr::read_volatile(addr) }

#[inline(always)]
pub unsafe fn write8(addr: *mut u8, val: u8) { core::ptr::write_volatile(addr, val) }

/// Data memory barrier (inner shareable)
#[inline(always)]
pub fn dmb_ish() {
    #[cfg(target_arch = "aarch64")]
    unsafe { core::arch::asm!("dmb ish", options(nostack, preserves_flags)) }
}

/// Data synchronization barrier (full system)
#[inline(always)]
pub fn dsb_sy() {
    #[cfg(target_arch = "aarch64")]
    unsafe { core::arch::asm!("dsb sy", options(nostack, preserves_flags)) }
}

