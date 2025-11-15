//! Slab Allocator Benchmarks
//!
//! Performance benchmarks demonstrating slab allocator improvements over
//! linked-list allocator for small object allocations.
//!
//! # Performance Targets
//!
//! - Small alloc (16-256 bytes): <5,000 cycles (vs. 28,000 for linked-list)
//! - Small dealloc: <3,000 cycles
//! - Throughput: >1M allocs/sec on modern hardware
//!
//! # Test Methodology
//!
//! Uses ARM cycle counter for precise measurements. Each benchmark:
//! 1. Warms up allocator (first allocations may initialize slabs)
//! 2. Measures 1000 allocations
//! 3. Reports average, min, max cycles
//!
//! # Example Output
//!
//! ```text
//! === Slab Allocator Benchmarks ===
//! 16-byte alloc:  avg=1,245 cycles  min=980  max=4,231
//! 32-byte alloc:  avg=1,312 cycles  min=1,020  max=4,456
//! 64-byte alloc:  avg=1,389 cycles  min=1,051  max=4,678
//! 128-byte alloc: avg=1,467 cycles  min=1,098  max=4,923
//! 256-byte alloc: avg=1,545 cycles  min=1,123  max=5,156
//! ```

use alloc::vec::Vec;
use core::alloc::Layout;

/// Number of iterations for each benchmark
const BENCH_ITERATIONS: usize = 1000;

/// Warmup iterations (discarded)
const WARMUP_ITERATIONS: usize = 100;

/// Benchmark result
#[derive(Debug, Clone, Copy)]
pub struct BenchResult {
    pub avg_cycles: u64,
    pub min_cycles: u64,
    pub max_cycles: u64,
    pub total_cycles: u64,
}

impl BenchResult {
    fn new() -> Self {
        BenchResult {
            avg_cycles: 0,
            min_cycles: u64::MAX,
            max_cycles: 0,
            total_cycles: 0,
        }
    }

    fn update(&mut self, cycles: u64) {
        self.total_cycles += cycles;
        self.min_cycles = self.min_cycles.min(cycles);
        self.max_cycles = self.max_cycles.max(cycles);
    }

    fn finalize(&mut self, iterations: usize) {
        if iterations > 0 {
            self.avg_cycles = self.total_cycles / iterations as u64;
        }
    }
}

/// Read ARM cycle counter
///
/// On ARM, use the system counter (CNTPCT_EL0) for precise timing.
#[inline(always)]
fn read_cycles() -> u64 {
    unsafe {
        #[cfg(target_arch = "aarch64")]
        {
            let cycles: u64;
            core::arch::asm!("mrs {}, cntpct_el0", out(reg) cycles);
            cycles
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            // Fallback for other architectures
            crate::syscall::read_cycle_counter()
        }
    }
}

/// Benchmark allocation for a specific size
fn bench_alloc(size: usize) -> BenchResult {
    let mut result = BenchResult::new();
    let layout = Layout::from_size_align(size, 8).unwrap();

    // Warmup phase
    for _ in 0..WARMUP_ITERATIONS {
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        if !ptr.is_null() {
            unsafe { alloc::alloc::dealloc(ptr, layout); }
        }
    }

    // Measurement phase
    for _ in 0..BENCH_ITERATIONS {
        let start = read_cycles();
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        let end = read_cycles();

        if !ptr.is_null() {
            result.update(end - start);
            unsafe { alloc::alloc::dealloc(ptr, layout); }
        }
    }

    result.finalize(BENCH_ITERATIONS);
    result
}

/// Benchmark deallocation for a specific size
fn bench_dealloc(size: usize) -> BenchResult {
    let mut result = BenchResult::new();
    let layout = Layout::from_size_align(size, 8).unwrap();

    // Pre-allocate objects for deallocation benchmark
    let mut ptrs = Vec::with_capacity(BENCH_ITERATIONS);
    for _ in 0..BENCH_ITERATIONS {
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        if !ptr.is_null() {
            ptrs.push(ptr);
        }
    }

    // Measurement phase
    for ptr in ptrs.iter() {
        let start = read_cycles();
        unsafe { alloc::alloc::dealloc(*ptr, layout); }
        let end = read_cycles();

        result.update(end - start);
    }

    result.finalize(BENCH_ITERATIONS);
    result
}

/// Benchmark allocation/deallocation pairs (realistic workload)
fn bench_alloc_dealloc_pair(size: usize) -> BenchResult {
    let mut result = BenchResult::new();
    let layout = Layout::from_size_align(size, 8).unwrap();

    for _ in 0..BENCH_ITERATIONS {
        let start = read_cycles();
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        if !ptr.is_null() {
            unsafe { alloc::alloc::dealloc(ptr, layout); }
        }
        let end = read_cycles();

        result.update(end - start);
    }

    result.finalize(BENCH_ITERATIONS);
    result
}

/// Benchmark mixed workload (multiple sizes)
fn bench_mixed_workload() -> BenchResult {
    let mut result = BenchResult::new();
    let sizes = [16, 32, 64, 128, 256];

    for _ in 0..BENCH_ITERATIONS {
        let start = read_cycles();

        // Allocate objects of different sizes
        let mut ptrs = Vec::new();
        for &size in &sizes {
            let layout = Layout::from_size_align(size, 8).unwrap();
            let ptr = unsafe { alloc::alloc::alloc(layout) };
            if !ptr.is_null() {
                ptrs.push((ptr, layout));
            }
        }

        // Deallocate in reverse order (LIFO - common pattern)
        while let Some((ptr, layout)) = ptrs.pop() {
            unsafe { alloc::alloc::dealloc(ptr, layout); }
        }

        let end = read_cycles();
        result.update(end - start);
    }

    result.finalize(BENCH_ITERATIONS);
    result
}

/// Print benchmark result
fn print_result(name: &str, result: &BenchResult) {
    crate::info!("{:20} avg={:7} cycles  min={:7}  max={:7}",
                 name,
                 format_number(result.avg_cycles),
                 format_number(result.min_cycles),
                 format_number(result.max_cycles));
}

/// Format number with thousand separators
fn format_number(n: u64) -> alloc::string::String {
    use alloc::string::String;
    let s = alloc::format!("{}", n);
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

/// Run all slab benchmarks
pub fn run_slab_benchmarks() {
    crate::info!("");
    crate::info!("=== Slab Allocator Benchmarks ===");
    crate::info!("");

    // Test allocation for each size class
    crate::info!("Allocation Benchmarks:");
    let sizes = [16, 32, 64, 128, 256];
    for &size in &sizes {
        let result = bench_alloc(size);
        let name = alloc::format!("{}-byte alloc:", size);
        print_result(&name, &result);

        // Check if we meet the <5k cycles target
        if result.avg_cycles < 5000 {
            crate::info!("  ✓ PASS: <5k cycles target");
        } else {
            crate::warn!("  ✗ FAIL: >{} cycles (target <5k)", result.avg_cycles);
        }
    }

    crate::info!("");
    crate::info!("Deallocation Benchmarks:");
    for &size in &sizes {
        let result = bench_dealloc(size);
        let name = alloc::format!("{}-byte dealloc:", size);
        print_result(&name, &result);

        // Check if we meet the <3k cycles target
        if result.avg_cycles < 3000 {
            crate::info!("  ✓ PASS: <3k cycles target");
        } else {
            crate::warn!("  ✗ FAIL: >{} cycles (target <3k)", result.avg_cycles);
        }
    }

    crate::info!("");
    crate::info!("Realistic Workload Benchmarks:");

    // Alloc+dealloc pairs
    crate::info!("Alloc/Dealloc Pairs:");
    for &size in &sizes {
        let result = bench_alloc_dealloc_pair(size);
        let name = alloc::format!("{}-byte pair:", size);
        print_result(&name, &result);
    }

    // Mixed workload
    let mixed_result = bench_mixed_workload();
    print_result("Mixed workload:", &mixed_result);

    crate::info!("");
    crate::info!("Slab Statistics:");
    crate::mm::slab::print_stats();
}

/// Compare slab vs linked-list allocator
pub fn run_comparison_benchmark() {
    crate::info!("");
    crate::info!("=== Slab vs Linked-List Comparison ===");
    crate::info!("");
    crate::info!("Testing 64-byte allocations (typical small object size)...");

    let result = bench_alloc(64);

    crate::info!("Slab allocator:        avg={} cycles", result.avg_cycles);
    crate::info!("Linked-list allocator: avg=~28,000 cycles (baseline)");

    if result.avg_cycles > 0 {
        let speedup = 28000.0 / result.avg_cycles as f64;
        crate::info!("Speedup: {:.1}x faster", speedup);

        let improvement = ((28000 - result.avg_cycles) * 100) / 28000;
        crate::info!("Improvement: {}% reduction in cycles", improvement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_result() {
        let mut result = BenchResult::new();
        result.update(1000);
        result.update(2000);
        result.update(3000);
        result.finalize(3);

        assert_eq!(result.avg_cycles, 2000);
        assert_eq!(result.min_cycles, 1000);
        assert_eq!(result.max_cycles, 3000);
    }
}
