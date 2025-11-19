//! DMA buffer helpers (scaffold)
//! These helpers provide stubs for coherent and non-coherent DMA buffer
//! allocation patterns. On platforms without an IOMMU, cache maintenance
//! is required for non-coherent devices.

use core::ptr::NonNull;

pub struct DmaBuf {
    pub ptr: NonNull<u8>,
    pub len: usize,
    pub align: usize,
    pub coherent: bool,
}

impl DmaBuf {
    pub fn as_ptr(&self) -> *mut u8 { self.ptr.as_ptr() }
}

/// Allocate a DMA buffer. For now this is a thin wrapper over the global
/// allocator; platforms should replace with page-based allocation and
/// attribute setting.
pub fn dma_alloc(len: usize, align: usize, coherent: bool) -> Option<DmaBuf> {
    // TODO: Replace with page-based allocator + cache attribute control
    let layout = core::alloc::Layout::from_size_align(len, align).ok()?;
    let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
    let nn = NonNull::new(ptr)?;
    Some(DmaBuf { ptr: nn, len, align, coherent })
}

pub unsafe fn dma_free(buf: DmaBuf) {
    if let Ok(layout) = core::alloc::Layout::from_size_align(buf.len, buf.align) {
        alloc::alloc::dealloc(buf.ptr.as_ptr(), layout);
    }
}

/// Perform cache maintenance for a DMA region. No-ops for now; platforms
/// should implement clean/invalidate as required by device coherency.
pub fn dma_sync(_buf: &DmaBuf) { /* platform-specific no-op */ }

