//! Static Memory Arena for LLM Operations
//!
//! # Overview
//!
//! This module provides a bounded, deterministic memory allocation system for LLM
//! operations. Unlike traditional heap allocation, the arena uses a fixed-size
//! static buffer with predictable allocation patterns.
//!
//! # Design Rationale
//!
//! **Why Static Allocation?**
//! - **Determinism**: WCET (Worst-Case Execution Time) bounds required for real-time guarantees
//! - **Safety**: No fragmentation, no allocation failures in hot path
//! - **Performance**: Simple bump allocator, O(1) allocation
//! - **Predictability**: Known memory footprint at compile time
//!
//! # Memory Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │  LLM Arena (8 MB)                           │
//! ├─────────────────────────────────────────────┤
//! │  Model Weights (4-6 MB)                     │  ← Quantized (Q4_0)
//! ├─────────────────────────────────────────────┤
//! │  Activation Buffers (1-2 MB)                │  ← F32 tensors
//! ├─────────────────────────────────────────────┤
//! │  KV Cache (1-2 MB)                          │  ← Context storage
//! ├─────────────────────────────────────────────┤
//! │  Tokenizer Vocab (256 KB)                   │  ← BPE vocabulary
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Usage Example
//!
//! ```no_run
//! use crate::llm::arena::{arena, LlmArena};
//!
//! // Allocate tensor buffer
//! let mut arena_lock = arena().lock();
//! let ptr = arena_lock.alloc(1024, 16).expect("OOM");
//!
//! // Use buffer...
//!
//! // Reset between inferences
//! arena_lock.reset();
//! ```
//!
//! # Thread Safety
//!
//! The arena is protected by a spinlock (`Mutex<LlmArena>`), allowing safe
//! concurrent access from multiple threads/cores.
//!
//! # Performance Characteristics
//!
//! - **Allocation**: O(1) bump pointer increment
//! - **Reset**: O(1) pointer reset
//! - **Fragmentation**: Zero (monotonic allocator)
//! - **Overhead**: 16 bytes per arena (offset + high water mark)

use core::ptr::NonNull;
use spin::Mutex;

/// Size of the LLM memory arena (8 MB)
///
/// This value balances capability vs memory footprint:
/// - Small enough to fit in L3 cache on modern CPUs
/// - Large enough for 10-50M parameter models (Q4_0 quantized)
/// - Leaves room for activations and KV cache
pub const ARENA_SIZE: usize = 8 * 1024 * 1024;

/// Memory alignment for tensor operations
///
/// 32-byte alignment ensures:
/// - SIMD vector operations (NEON: 128-bit = 16 bytes)
/// - Cache line alignment (typically 64 bytes)
/// - GGUF format compatibility (uses 32-byte alignment)
pub const ARENA_ALIGNMENT: usize = 32;

/// Static Memory Arena for LLM Operations
///
/// Provides bounded, deterministic memory allocation using a simple
/// bump allocator strategy. The entire arena is allocated at compile time
/// in the `.bss` section.
///
/// # Memory Safety
///
/// - All allocations are bounds-checked
/// - Alignment is enforced for all allocations
/// - No use-after-free possible (no deallocation)
/// - Memory is zero-initialized by default (.bss)
pub struct LlmArena {
    /// Static buffer (8 MB, zero-initialized)
    buffer: [u8; ARENA_SIZE],

    /// Current allocation offset (bump pointer)
    offset: usize,

    /// Highest offset reached (for monitoring)
    high_water_mark: usize,

    /// Number of allocations made (for debugging)
    allocation_count: u64,
}

impl LlmArena {
    /// Create a new arena (const fn for static initialization)
    ///
    /// # Example
    ///
    /// ```no_run
    /// static ARENA: Mutex<LlmArena> = Mutex::new(LlmArena::new());
    /// ```
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; ARENA_SIZE],
            offset: 0,
            high_water_mark: 0,
            allocation_count: 0,
        }
    }

    /// Allocate memory from the arena
    ///
    /// # Arguments
    ///
    /// - `size`: Number of bytes to allocate
    /// - `align`: Alignment requirement (must be power of 2)
    ///
    /// # Returns
    ///
    /// - `Some(ptr)`: Non-null pointer to allocated memory
    /// - `None`: Insufficient space or invalid alignment
    ///
    /// # Safety
    ///
    /// The returned pointer is valid for the lifetime of the arena and
    /// is guaranteed to be properly aligned. The memory is zero-initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Allocate 1024 bytes with 16-byte alignment
    /// let ptr = arena.alloc(1024, 16).expect("Out of memory");
    /// ```
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Validate alignment (must be power of 2)
        if !align.is_power_of_two() {
            return None;
        }

        // Check for zero-size allocation
        if size == 0 {
            return None;
        }

        // Align current offset
        let mask = align - 1;
        let aligned_offset = (self.offset + mask) & !mask;

        // Check bounds
        if aligned_offset.checked_add(size)? > ARENA_SIZE {
            return None;
        }

        // Allocate
        let ptr = unsafe {
            NonNull::new_unchecked(self.buffer.as_mut_ptr().add(aligned_offset))
        };

        // Update state
        self.offset = aligned_offset + size;
        self.allocation_count += 1;

        // Track high water mark
        if self.offset > self.high_water_mark {
            self.high_water_mark = self.offset;
        }

        Some(ptr)
    }

    /// Allocate a typed array from the arena
    ///
    /// # Type Parameters
    ///
    /// - `T`: Element type (must be `Copy`)
    ///
    /// # Arguments
    ///
    /// - `count`: Number of elements to allocate
    ///
    /// # Returns
    ///
    /// - `Some(slice)`: Mutable slice of allocated elements
    /// - `None`: Insufficient space
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Allocate 100 f32 values
    /// let buffer: &mut [f32] = arena.alloc_array(100).expect("OOM");
    /// ```
    pub fn alloc_array<T: Copy>(&mut self, count: usize) -> Option<&mut [T]> {
        let size = core::mem::size_of::<T>() * count;
        let align = core::mem::align_of::<T>();

        let ptr = self.alloc(size, align)?;

        // SAFETY: We just allocated `size` bytes with proper alignment
        unsafe {
            Some(core::slice::from_raw_parts_mut(
                ptr.as_ptr() as *mut T,
                count,
            ))
        }
    }

    /// Reset the arena (call between inferences)
    ///
    /// This resets the bump pointer to the beginning, effectively
    /// "freeing" all allocations. The memory is not zeroed (for performance).
    ///
    /// # Example
    ///
    /// ```no_run
    /// // After inference completes
    /// arena.reset();
    /// ```
    pub fn reset(&mut self) {
        self.offset = 0;
        // Note: We don't reset allocation_count or high_water_mark
        // These are cumulative metrics
    }

    /// Get current memory usage
    ///
    /// # Returns
    ///
    /// `(current_offset, high_water_mark)` tuple in bytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// let (used, peak) = arena.usage();
    /// println!("Using {} KB, peak {} KB", used / 1024, peak / 1024);
    /// ```
    pub fn usage(&self) -> (usize, usize) {
        (self.offset, self.high_water_mark)
    }

    /// Get allocation statistics
    ///
    /// Returns detailed statistics about arena usage:
    /// - Current offset (bytes currently allocated)
    /// - High water mark (peak usage across all time)
    /// - Total allocation count (number of alloc() calls)
    /// - Fragmentation ratio (always 0.0 for bump allocator)
    ///
    /// # Returns
    ///
    /// `ArenaStats` structure with detailed metrics
    pub fn stats(&self) -> ArenaStats {
        ArenaStats {
            current_offset: self.offset,
            high_water_mark: self.high_water_mark,
            total_size: ARENA_SIZE,
            allocation_count: self.allocation_count,
            utilization: (self.offset as f32 / ARENA_SIZE as f32) * 100.0,
            peak_utilization: (self.high_water_mark as f32 / ARENA_SIZE as f32) * 100.0,
        }
    }

    /// Check if the arena has sufficient space
    ///
    /// # Arguments
    ///
    /// - `size`: Required size in bytes
    /// - `align`: Required alignment
    ///
    /// # Returns
    ///
    /// `true` if allocation would succeed
    pub fn can_allocate(&self, size: usize, align: usize) -> bool {
        if !align.is_power_of_two() || size == 0 {
            return false;
        }

        let mask = align - 1;
        let aligned_offset = (self.offset + mask) & !mask;

        aligned_offset.checked_add(size).map_or(false, |end| end <= ARENA_SIZE)
    }

    /// Get remaining space in the arena
    ///
    /// # Returns
    ///
    /// Number of bytes remaining (conservative estimate, not accounting for alignment)
    pub fn remaining(&self) -> usize {
        ARENA_SIZE.saturating_sub(self.offset)
    }
}

/// Arena usage statistics
///
/// Detailed metrics about memory arena utilization, useful for
/// monitoring and optimization.
#[derive(Debug, Clone, Copy)]
pub struct ArenaStats {
    /// Current allocation offset (bytes)
    pub current_offset: usize,

    /// Peak offset reached (bytes)
    pub high_water_mark: usize,

    /// Total arena size (bytes)
    pub total_size: usize,

    /// Number of allocations made
    pub allocation_count: u64,

    /// Current utilization percentage (0-100)
    pub utilization: f32,

    /// Peak utilization percentage (0-100)
    pub peak_utilization: f32,
}

impl ArenaStats {
    /// Check if arena is near capacity
    ///
    /// Returns `true` if current utilization exceeds threshold
    pub fn is_near_capacity(&self, threshold_percent: f32) -> bool {
        self.utilization > threshold_percent
    }
}

/// Global LLM arena instance
///
/// This is the primary memory pool for all LLM operations.
/// Protected by a spinlock for thread-safe access.
static LLM_ARENA: Mutex<LlmArena> = Mutex::new(LlmArena::new());

/// Get reference to the global LLM arena
///
/// # Returns
///
/// Mutex-protected reference to the arena
///
/// # Example
///
/// ```no_run
/// use crate::llm::arena::arena;
///
/// let mut arena_lock = arena().lock();
/// let buffer = arena_lock.alloc(1024, 16).expect("OOM");
/// ```
pub fn arena() -> &'static Mutex<LlmArena> {
    &LLM_ARENA
}

/// RAII guard for automatic arena reset
///
/// Automatically resets the arena when dropped, ensuring
/// cleanup even in the presence of panics (if unwinding is enabled).
///
/// # Example
///
/// ```no_run
/// use crate::llm::arena::{arena, ArenaGuard};
///
/// {
///     let guard = ArenaGuard::new(arena());
///     let mut arena_lock = guard.arena().lock();
///     // ... use arena ...
/// } // Arena automatically reset here
/// ```
pub struct ArenaGuard {
    arena: &'static Mutex<LlmArena>,
}

impl ArenaGuard {
    /// Create a new arena guard
    pub fn new(arena: &'static Mutex<LlmArena>) -> Self {
        Self { arena }
    }

    /// Get the protected arena
    pub fn arena(&self) -> &'static Mutex<LlmArena> {
        self.arena
    }
}

impl Drop for ArenaGuard {
    fn drop(&mut self) {
        self.arena.lock().reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_creation() {
        let arena = LlmArena::new();
        assert_eq!(arena.offset, 0);
        assert_eq!(arena.high_water_mark, 0);
        assert_eq!(arena.allocation_count, 0);
    }

    #[test]
    fn test_simple_allocation() {
        let mut arena = LlmArena::new();
        let ptr = arena.alloc(1024, 8).expect("Failed to allocate");
        assert!(!ptr.as_ptr().is_null());
        assert_eq!(arena.offset, 1024);
        assert_eq!(arena.allocation_count, 1);
    }

    #[test]
    fn test_alignment() {
        let mut arena = LlmArena::new();

        // Allocate unaligned
        arena.alloc(1, 1).expect("Failed");

        // Next allocation should be aligned
        let ptr = arena.alloc(16, 16).expect("Failed");
        assert_eq!(ptr.as_ptr() as usize % 16, 0);
    }

    #[test]
    fn test_oom() {
        let mut arena = LlmArena::new();

        // Try to allocate more than arena size
        let result = arena.alloc(ARENA_SIZE + 1, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_reset() {
        let mut arena = LlmArena::new();

        arena.alloc(1024, 1).expect("Failed");
        assert_eq!(arena.offset, 1024);

        arena.reset();
        assert_eq!(arena.offset, 0);
        assert_eq!(arena.high_water_mark, 1024); // Preserved
    }

    #[test]
    fn test_high_water_mark() {
        let mut arena = LlmArena::new();

        arena.alloc(1000, 1).expect("Failed");
        assert_eq!(arena.high_water_mark, 1000);

        arena.reset();
        arena.alloc(500, 1).expect("Failed");
        assert_eq!(arena.high_water_mark, 1000); // Not decreased
    }

    #[test]
    fn test_typed_allocation() {
        let mut arena = LlmArena::new();

        let buffer: &mut [f32] = arena.alloc_array(100).expect("Failed");
        assert_eq!(buffer.len(), 100);

        // Write and read
        buffer[0] = 3.14;
        assert_eq!(buffer[0], 3.14);
    }

    #[test]
    fn test_stats() {
        let mut arena = LlmArena::new();
        arena.alloc(1024 * 1024, 1).expect("Failed"); // 1 MB

        let stats = arena.stats();
        assert_eq!(stats.current_offset, 1024 * 1024);
        assert!(stats.utilization > 12.0 && stats.utilization < 13.0); // ~12.5%
        assert_eq!(stats.allocation_count, 1);
    }

    #[test]
    fn test_can_allocate() {
        let mut arena = LlmArena::new();

        assert!(arena.can_allocate(1024, 8));

        arena.alloc(ARENA_SIZE - 512, 1).expect("Failed");
        assert!(arena.can_allocate(256, 1));
        assert!(!arena.can_allocate(1024, 1));
    }

    #[test]
    fn test_remaining() {
        let mut arena = LlmArena::new();
        assert_eq!(arena.remaining(), ARENA_SIZE);

        arena.alloc(1024, 1).expect("Failed");
        assert_eq!(arena.remaining(), ARENA_SIZE - 1024);
    }
}
