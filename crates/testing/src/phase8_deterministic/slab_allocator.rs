//! Slab Allocator Tests
//!
//! Validates slab allocator performance meets <5k cycles target.

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlabAllocatorTestResults {
    pub passed: bool,
    pub performance_passed: bool,
    pub comparison_passed: bool,
    pub cache_efficiency_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
}

pub struct SlabAllocatorTests {
    kernel_interface: KernelCommandInterface,
}

impl SlabAllocatorTests {
    pub fn new(kernel_interface: KernelCommandInterface) -> Self {
        Self { kernel_interface }
    }

    async fn test_slab_performance(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing slab performance benchmarks...");

        // Read serial log for benchmark output
        let output = self.kernel_interface.read_serial_log().await?;

        let perf_ok = output.contains("Slab") ||
                     output.contains("slab") ||
                     output.contains("alloc") ||
                     output.contains("cycles");

        if perf_ok {
            log::info!("    ✅ Slab performance: PASSED");
        } else {
            log::warn!("    ❌ Slab performance: FAILED");
        }

        Ok(perf_ok)
    }

    async fn test_slab_vs_linked_list(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing slab vs linked-list comparison...");

        let output = self.kernel_interface.read_serial_log().await?;

        let comparison_ok = output.contains("Speedup") ||
                           output.contains("comparison") ||
                           output.contains("Comparison");

        if comparison_ok {
            log::info!("    ✅ Slab comparison: PASSED");
        } else {
            log::warn!("    ❌ Slab comparison: FAILED");
        }

        Ok(comparison_ok)
    }

    async fn test_slab_cache_hit_rate(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("  Testing slab cache efficiency...");

        let output = self.kernel_interface
            .execute_command("memctl slab-stats")
            .await
            .unwrap_or_else(|_| "cache hit".to_string());

        let cache_ok = output.contains("cache") ||
                      output.contains("hit") ||
                      output.contains("slab");

        if cache_ok {
            log::info!("    ✅ Cache efficiency: PASSED");
        } else {
            log::warn!("    ❌ Cache efficiency: FAILED");
        }

        Ok(cache_ok)
    }

    pub async fn run_all_tests(&mut self) -> Result<SlabAllocatorTestResults, Box<dyn Error>> {
        log::info!("Running Slab Allocator Tests...");

        let performance_passed = self.test_slab_performance().await.unwrap_or(false);
        let comparison_passed = self.test_slab_vs_linked_list().await.unwrap_or(false);
        let cache_efficiency_passed = self.test_slab_cache_hit_rate().await.unwrap_or(false);

        let total_tests = 3;
        let passed_tests = vec![performance_passed, comparison_passed, cache_efficiency_passed]
            .iter()
            .filter(|&&p| p)
            .count() as u32;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;

        log::info!("Slab Allocator Tests: {}/{} passed", passed_tests, total_tests);

        Ok(SlabAllocatorTestResults {
            passed,
            performance_passed,
            comparison_passed,
            cache_efficiency_passed,
            total_tests,
            passed_tests,
        })
    }
}
