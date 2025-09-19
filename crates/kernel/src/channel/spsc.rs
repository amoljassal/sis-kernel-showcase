//! Lock-free Single-Producer Single-Consumer ring buffer.
//! Suitable for zero-copy handle passing in Phase 1.

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct Spsc<T: Copy, const N: usize> {
    buf: [MaybeUninit<T>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<T: Copy, const N: usize> Spsc<T, N> {
    pub const fn new() -> Self {
        // const-init workaround: MaybeUninit::uninit_array not const-stable in core for all versions
        Self {
            buf: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize { N }

    #[inline(always)]
    pub fn try_enqueue(&self, v: T) -> Result<(), T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let next = (tail + 1) % N;
        if next == head { return Err(v); }
        unsafe {
            let slot = self.buf.as_ptr().add(tail) as *mut MaybeUninit<T>;
            core::ptr::write((*slot).as_mut_ptr(), v);
        }
        self.tail.store(next, Ordering::Release);
        Ok(())
    }

    #[inline(always)]
    pub fn try_dequeue(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head == tail { return None; }
        let v = unsafe {
            let slot = self.buf.as_ptr().add(head) as *const MaybeUninit<T>;
            core::ptr::read((*slot).as_ptr())
        };
        self.head.store((head + 1) % N, Ordering::Release);
        Some(v)
    }

    #[inline(always)]
    pub fn depth(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        if tail >= head { tail - head } else { N - (head - tail) }
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        ((tail + 1) % N) == head
    }
    
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }
}
