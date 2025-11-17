//! LLM Performance Benchmarks
//!
//! # Overview
//!
//! Standardized performance benchmarks for LLM subsystem components:
//! - Tokenization speed
//! - Quantization/dequantization throughput
//! - Matrix operations (matmul, dot product)
//! - Transformer layer forward pass
//! - End-to-end inference
//! - Memory allocator performance
//!
//! # Benchmark Methodology
//!
//! **Warm-up**: Each benchmark runs 100 warm-up iterations
//! **Measurement**: 1000 measurement iterations
//! **Timing**: Cycle-accurate measurement (when available)
//! **Statistics**: Min, max, mean, median, stddev
//!
//! # Performance Targets
//!
//! | Benchmark | Target | Unit |
//! |-----------|--------|------|
//! | Tokenization | <10 µs | per 100 chars |
//! | Q4_0 Dequant | <2 µs | per 32 values |
//! | Dot Product (384) | <1 µs | per operation |
//! | MatMul (384×384) | <500 µs | per operation |
//! | Layer Forward | <5 ms | per layer |
//! | Token Generation | <200 ms | per token |
//!
//! # Running Benchmarks
//!
//! ```bash
//! # All benchmarks
//! cargo bench --features llm-transformer
//!
//! # Specific benchmark
//! cargo bench --features llm-transformer tokenization
//!
//! # With SIMD
//! cargo bench --features llm-transformer,simd
//! ```

use crate::llm::{
    tokenizer::BpeTokenizer,
    quantize::{Q4_0Block, dequantize_q4_0, f32_to_f16},
    transformer::{TransformerConfig, TransformerLayer, layer_norm, matmul_vec},
    arena::arena,
    simd::{dot_product_simd, matmul_vec_simd},
    kv_cache::KVCache,
};
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// Benchmark result statistics
#[derive(Debug, Clone, Copy)]
pub struct BenchmarkStats {
    /// Minimum time (cycles or microseconds)
    pub min: u64,

    /// Maximum time
    pub max: u64,

    /// Mean time
    pub mean: f64,

    /// Median time
    pub median: u64,

    /// Standard deviation
    pub stddev: f64,

    /// Number of iterations
    pub iterations: usize,

    /// Throughput (operations per second)
    pub throughput: f64,
}

impl BenchmarkStats {
    /// Calculate statistics from timing samples
    pub fn from_samples(mut samples: Vec<u64>) -> Self {
        let iterations = samples.len();
        samples.sort_unstable();

        let min = samples[0];
        let max = samples[iterations - 1];
        let median = samples[iterations / 2];

        let sum: u64 = samples.iter().sum();
        let mean = sum as f64 / iterations as f64;

        let variance = samples.iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / iterations as f64;

        let stddev = variance.sqrt();

        let throughput = if mean > 0.0 {
            1_000_000.0 / mean  // ops/sec (assuming microseconds)
        } else {
            0.0
        };

        Self {
            min,
            max,
            mean,
            median,
            stddev,
            iterations,
            throughput,
        }
    }

    /// Print results
    pub fn print(&self, name: &str) {
        crate::info!("Benchmark: {}", name);
        crate::info!("  Iterations: {}", self.iterations);
        crate::info!("  Min: {} µs", self.min);
        crate::info!("  Max: {} µs", self.max);
        crate::info!("  Mean: {:.2} µs", self.mean);
        crate::info!("  Median: {} µs", self.median);
        crate::info!("  Stddev: {:.2} µs", self.stddev);
        crate::info!("  Throughput: {:.0} ops/sec", self.throughput);
    }
}

/// Benchmark suite configuration
#[derive(Debug, Clone, Copy)]
pub struct BenchmarkConfig {
    /// Number of warm-up iterations
    pub warmup_iterations: usize,

    /// Number of measurement iterations
    pub measurement_iterations: usize,

    /// Whether to use SIMD (if available)
    pub use_simd: bool,

    /// Print detailed results
    pub verbose: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 100,
            measurement_iterations: 1000,
            use_simd: true,
            verbose: true,
        }
    }
}

/// Benchmark: Tokenization
pub fn bench_tokenization(config: BenchmarkConfig) -> BenchmarkStats {
    let mut tokenizer = BpeTokenizer::new();

    // Load test vocabulary
    let test_vocab = "0\t3c554e4b3e\n4\t48656c6c6f\n5\t576f726c64\n6\t2c\n7\t20";
    tokenizer.load_from_text(test_vocab).expect("Failed to load vocab");

    let text = "Hello World, Hello World, Hello World";

    // Warm-up
    for _ in 0..config.warmup_iterations {
        let _ = tokenizer.encode(text);
    }

    // Measure
    let mut samples = Vec::new();
    for _ in 0..config.measurement_iterations {
        let start = mock_timestamp();
        let _ = tokenizer.encode(text);
        let elapsed = mock_timestamp() - start;
        samples.push(elapsed);
    }

    let stats = BenchmarkStats::from_samples(samples);
    if config.verbose {
        stats.print("Tokenization (100 chars)");
    }
    stats
}

/// Benchmark: Q4_0 Dequantization
pub fn bench_q4_0_dequant(config: BenchmarkConfig) -> BenchmarkStats {
    // Create test blocks
    let blocks = vec![Q4_0Block {
        scale: f32_to_f16(0.5),
        quants: [0x88; 16],
    }; 32];  // 1024 values total

    let mut output = vec![0.0f32; 1024];

    // Warm-up
    for _ in 0..config.warmup_iterations {
        dequantize_q4_0(&blocks, &mut output);
    }

    // Measure
    let mut samples = Vec::new();
    for _ in 0..config.measurement_iterations {
        let start = mock_timestamp();
        dequantize_q4_0(&blocks, &mut output);
        let elapsed = mock_timestamp() - start;
        samples.push(elapsed);
    }

    let stats = BenchmarkStats::from_samples(samples);
    if config.verbose {
        stats.print("Q4_0 Dequantization (1024 values)");
    }
    stats
}

/// Benchmark: Dot Product
pub fn bench_dot_product(config: BenchmarkConfig) -> BenchmarkStats {
    let a: Vec<f32> = (0..384).map(|i| i as f32 * 0.1).collect();
    let b: Vec<f32> = (0..384).map(|i| (i as f32 * 0.1) + 0.5).collect();

    // Warm-up
    for _ in 0..config.warmup_iterations {
        if config.use_simd {
            let _ = dot_product_simd(&a, &b);
        } else {
            let _ = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
        }
    }

    // Measure
    let mut samples = Vec::new();
    for _ in 0..config.measurement_iterations {
        let start = mock_timestamp();
        if config.use_simd {
            let _ = dot_product_simd(&a, &b);
        } else {
            let _ = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
        }
        let elapsed = mock_timestamp() - start;
        samples.push(elapsed);
    }

    let stats = BenchmarkStats::from_samples(samples);
    if config.verbose {
        let name = if config.use_simd {
            "Dot Product (384, SIMD)"
        } else {
            "Dot Product (384, Scalar)"
        };
        stats.print(name);
    }
    stats
}

/// Benchmark: Matrix-Vector Multiplication
pub fn bench_matmul(config: BenchmarkConfig) -> BenchmarkStats {
    let vec: Vec<f32> = (0..384).map(|i| i as f32 * 0.1).collect();
    let mat: Vec<f32> = (0..(384 * 384)).map(|i| i as f32 * 0.01).collect();

    // Warm-up
    for _ in 0..config.warmup_iterations {
        if config.use_simd {
            let _ = matmul_vec_simd(&vec, &mat, 384, 384);
        } else {
            let _ = matmul_vec(&vec, &mat, 384, 384);
        }
    }

    // Measure
    let mut samples = Vec::new();
    for _ in 0..config.measurement_iterations {
        let start = mock_timestamp();
        if config.use_simd {
            let _ = matmul_vec_simd(&vec, &mat, 384, 384);
        } else {
            let _ = matmul_vec(&vec, &mat, 384, 384);
        }
        let elapsed = mock_timestamp() - start;
        samples.push(elapsed);
    }

    let stats = BenchmarkStats::from_samples(samples);
    if config.verbose {
        let name = if config.use_simd {
            "MatMul (384×384, SIMD)"
        } else {
            "MatMul (384×384, Scalar)"
        };
        stats.print(name);
    }
    stats
}

/// Benchmark: Layer Normalization
pub fn bench_layer_norm(config: BenchmarkConfig) -> BenchmarkStats {
    let input: Vec<f32> = (0..384).map(|i| i as f32 * 0.1).collect();
    let weight = vec![1.0f32; 384];
    let bias = vec![0.0f32; 384];

    // Warm-up
    for _ in 0..config.warmup_iterations {
        let _ = layer_norm(&input, &weight, &bias);
    }

    // Measure
    let mut samples = Vec::new();
    for _ in 0..config.measurement_iterations {
        let start = mock_timestamp();
        let _ = layer_norm(&input, &weight, &bias);
        let elapsed = mock_timestamp() - start;
        samples.push(elapsed);
    }

    let stats = BenchmarkStats::from_samples(samples);
    if config.verbose {
        stats.print("Layer Normalization (384)");
    }
    stats
}

/// Benchmark: KV Cache Access
pub fn bench_kv_cache(config: BenchmarkConfig) -> BenchmarkStats {
    let mut cache = KVCache::new(6, 256, 384);

    let k = vec![1.0f32; 384];
    let v = vec![1.0f32; 384];

    // Fill cache
    for i in 0..100 {
        for layer in 0..6 {
            cache.update(layer, k.clone(), v.clone());
        }
        cache.advance();
    }

    // Warm-up
    for _ in 0..config.warmup_iterations {
        let _ = cache.get(0);
    }

    // Measure
    let mut samples = Vec::new();
    for _ in 0..config.measurement_iterations {
        let start = mock_timestamp();
        let _ = cache.get(0);
        let elapsed = mock_timestamp() - start;
        samples.push(elapsed);
    }

    let stats = BenchmarkStats::from_samples(samples);
    if config.verbose {
        stats.print("KV Cache Access");
    }
    stats
}

/// Benchmark: Arena Allocation
pub fn bench_arena_allocation(config: BenchmarkConfig) -> BenchmarkStats {
    // Warm-up
    for _ in 0..config.warmup_iterations {
        let mut arena = arena().lock();
        let _ = arena.alloc(1024, 16);
        arena.reset();
    }

    // Measure
    let mut samples = Vec::new();
    for _ in 0..config.measurement_iterations {
        let start = mock_timestamp();
        {
            let mut arena = arena().lock();
            let _ = arena.alloc(1024, 16);
            arena.reset();
        }
        let elapsed = mock_timestamp() - start;
        samples.push(elapsed);
    }

    let stats = BenchmarkStats::from_samples(samples);
    if config.verbose {
        stats.print("Arena Allocation (1KB)");
    }
    stats
}

/// Run full benchmark suite
pub fn run_benchmark_suite(config: BenchmarkConfig) -> BenchmarkSuiteResults {
    crate::info!("=== Running LLM Benchmark Suite ===");
    crate::info!("Config: {} warmup, {} measurement iterations",
                 config.warmup_iterations, config.measurement_iterations);

    let results = BenchmarkSuiteResults {
        tokenization: bench_tokenization(config),
        q4_0_dequant: bench_q4_0_dequant(config),
        dot_product: bench_dot_product(config),
        matmul: bench_matmul(config),
        layer_norm: bench_layer_norm(config),
        kv_cache: bench_kv_cache(config),
        arena_alloc: bench_arena_allocation(config),
    };

    crate::info!("=== Benchmark Suite Complete ===");
    results
}

/// Benchmark suite results
#[derive(Debug, Clone)]
pub struct BenchmarkSuiteResults {
    pub tokenization: BenchmarkStats,
    pub q4_0_dequant: BenchmarkStats,
    pub dot_product: BenchmarkStats,
    pub matmul: BenchmarkStats,
    pub layer_norm: BenchmarkStats,
    pub kv_cache: BenchmarkStats,
    pub arena_alloc: BenchmarkStats,
}

impl BenchmarkSuiteResults {
    /// Print summary
    pub fn print_summary(&self) {
        crate::info!("\n=== Benchmark Summary ===");
        crate::info!("Tokenization:     {:.2} µs", self.tokenization.mean);
        crate::info!("Q4_0 Dequant:     {:.2} µs", self.q4_0_dequant.mean);
        crate::info!("Dot Product:      {:.2} µs", self.dot_product.mean);
        crate::info!("MatMul:           {:.2} µs", self.matmul.mean);
        crate::info!("Layer Norm:       {:.2} µs", self.layer_norm.mean);
        crate::info!("KV Cache:         {:.2} µs", self.kv_cache.mean);
        crate::info!("Arena Alloc:      {:.2} µs", self.arena_alloc.mean);
    }

    /// Check if all benchmarks meet targets
    pub fn meets_targets(&self) -> bool {
        self.tokenization.mean < 10.0
            && self.q4_0_dequant.mean < 2.0
            && self.dot_product.mean < 1.0
            && self.matmul.mean < 500.0
            && self.layer_norm.mean < 10.0
            && self.kv_cache.mean < 1.0
            && self.arena_alloc.mean < 1.0
    }
}

/// Mock timestamp function (replace with actual cycle counter)
///
/// TODO: Integrate with actual timer hardware
fn mock_timestamp() -> u64 {
    // In production, would use:
    // - ARM: Read PMCCNTR_EL0 (cycle counter)
    // - RISC-V: Read CYCLE CSR
    // - x86: Read TSC (rdtsc)
    static mut COUNTER: u64 = 0;
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_stats() {
        let samples = vec![10, 20, 30, 40, 50];
        let stats = BenchmarkStats::from_samples(samples);

        assert_eq!(stats.min, 10);
        assert_eq!(stats.max, 50);
        assert_eq!(stats.median, 30);
        assert_eq!(stats.mean, 30.0);
        assert_eq!(stats.iterations, 5);
    }

    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.warmup_iterations, 100);
        assert_eq!(config.measurement_iterations, 1000);
    }

    #[test]
    #[ignore]  // Run with: cargo test --features llm-transformer -- --ignored
    fn test_run_benchmarks() {
        let config = BenchmarkConfig {
            warmup_iterations: 10,
            measurement_iterations: 100,
            use_simd: false,
            verbose: false,
        };

        let results = run_benchmark_suite(config);
        results.print_summary();
    }
}
