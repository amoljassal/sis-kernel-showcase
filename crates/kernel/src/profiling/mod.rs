//! Performance Profiling Framework
//!
//! Sampling-based profiler for identifying performance hotspots in the kernel.
//! Similar to Linux `perf`, captures program counter (PC) samples on timer
//! interrupts to build a statistical profile of where CPU time is spent.
//!
//! # Phase 8 Milestone 5
//!
//! Provides integrated profiling for performance analysis and optimization.
//!
//! ## Architecture
//!
//! ```text
//! Timer Interrupt (every ~1ms)
//!      │
//!      ├──> Sample PC + PID
//!      │
//!      └──> Store in circular buffer
//!
//! User Commands:
//!   profstart  → Enable sampling
//!   profstop   → Disable sampling
//!   profreport → Analyze samples and show hotspots
//! ```
//!
//! ## Example Usage
//!
//! ```text
//! sis> profstart
//! Profiler started (max 10000 samples)
//!
//! sis> stresstest memory --duration 5000
//! [Run workload...]
//!
//! sis> profstop
//! Profiler stopped (captured 4521 samples)
//!
//! sis> profreport
//! === Profiling Report ===
//! Total samples: 4,521
//! Dropped: 0
//!
//! Top 10 Hotspots:
//! Address              Samples    Percent    Symbol
//! 1. 0x0000000040008000 687        15.2%      mm::buddy::allocate
//! 2. 0x0000000040009000 521        11.5%      sched::schedule
//! ...
//! ```

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

/// Maximum number of samples to collect
pub const MAX_SAMPLES: usize = 10_000;

/// Sample granularity (4KB - function/page level)
pub const SAMPLE_GRANULARITY: u64 = 4096;

/// Profiling sample
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    /// Program counter (instruction address)
    pub pc: u64,
    /// Process ID (0 for kernel)
    pub pid: u32,
    /// Timestamp (cycle counter)
    pub timestamp: u64,
}

/// Hotspot in profiling report
#[derive(Debug, Clone)]
pub struct Hotspot {
    /// Base address (aligned to granularity)
    pub address: u64,
    /// Number of samples at this address
    pub samples: u64,
    /// Percentage of total samples
    pub percentage: f64,
    /// Symbol name (if available)
    pub symbol: Option<String>,
}

/// Profiling report
#[derive(Debug)]
pub struct ProfilingReport {
    /// Total samples collected
    pub total_samples: u64,
    /// Samples dropped (buffer full)
    pub dropped_samples: u64,
    /// Top hotspots (sorted by sample count)
    pub hotspots: Vec<Hotspot>,
}

/// Global profiler state
pub struct Profiler {
    /// Is profiling enabled?
    enabled: AtomicBool,
    /// Sample buffer (circular)
    samples: Mutex<Vec<Sample>>,
    /// Next write index in circular buffer
    next_write: AtomicU64,
    /// Total samples collected
    sample_count: AtomicU64,
    /// Samples dropped due to buffer full
    dropped_samples: AtomicU64,
}

impl Profiler {
    /// Create new profiler (disabled by default)
    pub const fn new() -> Self {
        Profiler {
            enabled: AtomicBool::new(false),
            samples: Mutex::new(Vec::new()),
            next_write: AtomicU64::new(0),
            sample_count: AtomicU64::new(0),
            dropped_samples: AtomicU64::new(0),
        }
    }

    /// Start profiling
    ///
    /// Clears existing samples and begins collecting new samples on each
    /// timer interrupt. Sampling continues until `stop()` is called.
    pub fn start(&self) {
        // Reset state
        self.sample_count.store(0, Ordering::Relaxed);
        self.dropped_samples.store(0, Ordering::Relaxed);
        self.next_write.store(0, Ordering::Relaxed);

        // Clear sample buffer
        let mut samples = self.samples.lock();
        samples.clear();
        samples.reserve(MAX_SAMPLES);
        drop(samples);

        // Enable sampling
        self.enabled.store(true, Ordering::Release);

        crate::info!("Profiler started (max {} samples)", MAX_SAMPLES);
    }

    /// Stop profiling
    ///
    /// Disables sample collection. Existing samples remain in buffer
    /// and can be analyzed with `report()`.
    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Release);

        let total = self.sample_count.load(Ordering::Relaxed);
        let dropped = self.dropped_samples.load(Ordering::Relaxed);

        crate::info!("Profiler stopped (captured {} samples, dropped {})",
                     total, dropped);
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Record a sample (called from timer interrupt)
    ///
    /// This is called on every timer tick when profiling is enabled.
    /// Samples are stored in a circular buffer, oldest samples are
    /// overwritten if buffer is full.
    ///
    /// # Arguments
    /// * `pc` - Program counter (instruction address)
    /// * `pid` - Process ID (0 for kernel code)
    pub fn sample(&self, pc: u64, pid: u32) {
        if !self.enabled.load(Ordering::Acquire) {
            return;
        }

        let sample = Sample {
            pc,
            pid,
            timestamp: unsafe { crate::syscall::read_cycle_counter() },
        };

        let mut samples = self.samples.lock();

        if samples.len() < MAX_SAMPLES {
            // Buffer not full - append
            samples.push(sample);
        } else {
            // Buffer full - circular overwrite
            let idx = self.next_write.fetch_add(1, Ordering::Relaxed) as usize % MAX_SAMPLES;
            samples[idx] = sample;
            self.dropped_samples.fetch_add(1, Ordering::Relaxed);
        }

        self.sample_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Generate profiling report
    ///
    /// Analyzes collected samples and returns a report with top hotspots.
    /// Samples are grouped by address (aligned to SAMPLE_GRANULARITY) and
    /// sorted by frequency.
    ///
    /// # Returns
    /// * ProfilingReport with statistics and top 10 hotspots
    pub fn report(&self) -> ProfilingReport {
        let samples = self.samples.lock();

        // Count samples per address bucket (4KB granularity)
        let mut histogram: BTreeMap<u64, u64> = BTreeMap::new();

        for sample in samples.iter() {
            // Align PC to granularity (4KB boundaries = function/page level)
            let bucket = sample.pc & !(SAMPLE_GRANULARITY - 1);
            *histogram.entry(bucket).or_insert(0) += 1;
        }

        // Sort by sample count (descending)
        let mut sorted: Vec<(u64, u64)> = histogram.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        // Build top 10 hotspots
        let total_samples = samples.len() as u64;
        let hotspots: Vec<Hotspot> = sorted
            .into_iter()
            .take(10)
            .map(|(addr, count)| {
                let percentage = if total_samples > 0 {
                    (count as f64 / total_samples as f64) * 100.0
                } else {
                    0.0
                };

                Hotspot {
                    address: addr,
                    samples: count,
                    percentage,
                    symbol: resolve_symbol(addr),
                }
            })
            .collect();

        ProfilingReport {
            total_samples: self.sample_count.load(Ordering::Relaxed),
            dropped_samples: self.dropped_samples.load(Ordering::Relaxed),
            hotspots,
        }
    }

    /// Get current sample count
    pub fn sample_count(&self) -> u64 {
        self.sample_count.load(Ordering::Relaxed)
    }

    /// Get dropped sample count
    pub fn dropped_count(&self) -> u64 {
        self.dropped_samples.load(Ordering::Relaxed)
    }
}

/// Resolve address to symbol name
///
/// For Phase 8, this is a stub that returns known kernel function names
/// based on address ranges. Phase 9 will implement proper symbol table
/// parsing from kernel ELF.
///
/// # Arguments
/// * `addr` - Address to resolve
///
/// # Returns
/// * Symbol name if known, None otherwise
fn resolve_symbol(addr: u64) -> Option<String> {
    // Known kernel address ranges (approximate - would parse from ELF in production)
    // These are educated guesses based on typical kernel layout

    match addr {
        // Memory management (0x4000_0000 - 0x4000_FFFF)
        0x40000000..=0x40003FFF => Some("mm::buddy::allocate".to_string()),
        0x40004000..=0x40007FFF => Some("mm::slab::allocate".to_string()),
        0x40008000..=0x4000BFFF => Some("mm::page_fault".to_string()),

        // Scheduler (0x4001_0000 - 0x4001_FFFF)
        0x40010000..=0x40013FFF => Some("sched::schedule".to_string()),
        0x40014000..=0x40017FFF => Some("sched::context_switch".to_string()),

        // VirtIO (0x4002_0000 - 0x4002_FFFF)
        0x40020000..=0x40023FFF => Some("virtio::block::read".to_string()),
        0x40024000..=0x40027FFF => Some("virtio::queue::add_buf".to_string()),

        // Syscalls (0x4003_0000 - 0x4003_FFFF)
        0x40030000..=0x40033FFF => Some("syscall::handle".to_string()),

        // Process management (0x4004_0000 - 0x4004_FFFF)
        0x40040000..=0x40043FFF => Some("process::fork".to_string()),
        0x40044000..=0x40047FFF => Some("process::exec".to_string()),

        // Kernel entry/trap (0x4005_0000 - 0x4005_FFFF)
        0x40050000..=0x40053FFF => Some("trap::handler".to_string()),

        _ => None,
    }
}

/// Format number with thousand separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
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

/// Print profiling report to console
///
/// Formats and displays the profiling report in a human-readable format.
pub fn print_report(report: &ProfilingReport) {
    crate::info!("");
    crate::info!("=== Profiling Report ===");
    crate::info!("Total samples: {}", format_number(report.total_samples));

    if report.dropped_samples > 0 {
        crate::warn!("Dropped samples: {} (buffer full)", format_number(report.dropped_samples));
    }

    if report.hotspots.is_empty() {
        crate::info!("No samples collected");
        return;
    }

    crate::info!("");
    crate::info!("Top 10 Hotspots:");
    crate::info!("{:<4} {:<18} {:<10} {:<8} {}",
                 "Rank", "Address", "Samples", "Percent", "Symbol");
    crate::info!("{}", "-".repeat(70));

    for (i, hotspot) in report.hotspots.iter().enumerate() {
        let symbol = hotspot.symbol.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        crate::info!("{:<4} {:#018x} {:<10} {:>6.2}%  {}",
                     i + 1,
                     hotspot.address,
                     format_number(hotspot.samples),
                     hotspot.percentage,
                     symbol);
    }

    crate::info!("");
}

// Global profiler instance
static PROFILER: Profiler = Profiler::new();

/// Get global profiler instance
pub fn get() -> &'static Profiler {
    &PROFILER
}

/// Start profiling (convenience function)
pub fn start() {
    PROFILER.start();
}

/// Stop profiling (convenience function)
pub fn stop() {
    PROFILER.stop();
}

/// Get profiling report (convenience function)
pub fn report() -> ProfilingReport {
    PROFILER.report()
}

/// Check if profiling is enabled (convenience function)
pub fn is_enabled() -> bool {
    PROFILER.is_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_lifecycle() {
        let profiler = Profiler::new();
        assert!(!profiler.is_enabled());

        profiler.start();
        assert!(profiler.is_enabled());

        profiler.sample(0x40000000, 0);
        profiler.sample(0x40000000, 0);
        profiler.sample(0x40004000, 0);

        assert_eq!(profiler.sample_count(), 3);

        profiler.stop();
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
    }

    #[test]
    fn test_symbol_resolution() {
        assert!(resolve_symbol(0x40000000).is_some());
        assert!(resolve_symbol(0xFFFFFFFF).is_none());
    }
}
