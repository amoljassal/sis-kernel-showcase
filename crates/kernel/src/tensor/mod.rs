//! Zero-copy friendly tensor handle and simple allocator hooks.
//! Phase 1 keeps this minimal; per-graph arenas arrive later.

use core::ptr::NonNull;
use core::alloc::Layout;

#[repr(C, align(64))]
pub struct TensorHeader {
    pub version: u32,
    pub dtype: u32,
    pub dims: [u64; 4],
    pub strides: [u64; 4],
    pub data_offset: u64,
    // Phase 1.5 typed DataTensor fields (for schema/quality/lineage guardrails)
    pub schema_id: u32,
    pub records: u32,
    pub quality: u16,
    pub _pad: u16,
    pub lineage: u64,
}

#[derive(Copy, Clone)]
pub struct TensorHandle {
    pub ptr: *mut u8,
    pub len: usize,
}

impl TensorHandle {
    #[inline(always)]
    pub fn null() -> Self { Self { ptr: core::ptr::null_mut(), len: 0 } }
    #[inline(always)]
    pub fn is_null(&self) -> bool { self.ptr.is_null() }
    #[inline(always)]
    pub unsafe fn header_mut(&self) -> Option<&'static mut TensorHeader> {
        if self.ptr.is_null() || self.len < core::mem::size_of::<TensorHeader>() { return None; }
        Some(&mut *(self.ptr as *mut TensorHeader))
    }
    #[inline(always)]
    pub unsafe fn header(&self) -> Option<&'static TensorHeader> {
        if self.ptr.is_null() || self.len < core::mem::size_of::<TensorHeader>() { return None; }
        Some(&*(self.ptr as *const TensorHeader))
    }
}

pub struct TensorAlloc;

impl TensorAlloc {
    /// Allocate an uninitialized tensor buffer of `len` bytes.
    pub unsafe fn alloc_uninit(len: usize, align: usize) -> Option<TensorHandle> {
        let layout = Layout::from_size_align(len, align).ok()?;
        let ptr = alloc::alloc::alloc(layout);
        NonNull::new(ptr).map(|nn| TensorHandle { ptr: nn.as_ptr(), len })
    }

    /// Deallocate a previously allocated buffer.
    pub unsafe fn dealloc(h: TensorHandle, align: usize) {
        if !h.ptr.is_null() {
            if let Ok(layout) = Layout::from_size_align(h.len, align) {
                alloc::alloc::dealloc(h.ptr, layout);
            }
        }
    }
}

/// Simple per-graph bump arena for predictable allocation in Phase 1.
#[repr(align(64))]
pub struct Aligned<const N: usize>(pub [u8; N]);

pub struct BumpArena<const N: usize> {
    buf: Aligned<N>,
    off: usize,
}

impl<const N: usize> BumpArena<N> {
    pub const fn new() -> Self { Self { buf: Aligned([0; N]), off: 0 } }

    #[inline(always)]
    pub fn remaining(&self) -> usize { N.saturating_sub(self.off) }

    #[inline(always)]
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<TensorHandle> {
        let base = self.buf.0.as_ptr() as usize;
        let cur = base + self.off;
        let aligned = (cur + (align - 1)) & !(align - 1);
        let end = aligned.checked_add(size)?;
        if end > base + N { return None; }
        self.off = end - base;
        Some(TensorHandle { ptr: aligned as *mut u8, len: size })
    }
}
