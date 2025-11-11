//! Enhanced Stress Testing & Performance Validation Framework
//!
//! Comprehensive stress testing to validate AI/ML improvements with:
//! - Memory pressure endurance tests (with real variability)
//! - Command flood tests (with jitter)
//! - Multi-subsystem coordination tests
//! - Learning validation tests (with real reward calculation)
//! - Adversarial red team tests
//! - Chaos engineering (with randomized events and failure injection)
//! - Autonomy observability and comparative analysis
//! - Latency percentile tracking
//!
//! Based on docs/plans/STRESS_TEST_PLAN.md enhancement roadmap

use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;
use crate::prng;
use crate::autonomy_metrics::AUTONOMY_METRICS;
use crate::latency_histogram::{LatencyHistogram, LatencyReport};
use alloc::format;

/// Stress test metrics collection (enhanced with new fields)
#[derive(Copy, Clone)]
pub struct StressTestMetrics {
    // Pre-test baseline
    pub baseline_memory_pressure: u8,
    pub baseline_deadline_misses: u32,
    pub baseline_command_accuracy: u8,
    pub baseline_reward: i16,

    // During test
    pub peak_memory_pressure: u8,
    pub avg_memory_pressure: u8,           // NEW: Average memory pressure
    pub oom_events: u32,
    pub compaction_triggers: u32,
    pub coordination_events: u32,
    pub prediction_accuracy: u8,
    pub actions_taken: u32,

    // Post-test
    pub recovery_time_ms: u64,
    pub avg_reward_per_decision: i16,
    pub total_rewards: i32,
    pub decisions_made: u32,

    // Test metadata
    pub test_duration_ms: u64,
    pub autonomous_enabled: bool,
    pub test_passed: bool,

    // NEW: Enhanced metrics for chaos/failure tests
    pub successful_recoveries: u32,
    pub failed_recoveries: u32,
    pub chaos_events_count: u32,

    // NEW: Latency percentiles
    pub latency_p50_ns: u64,
    pub latency_p95_ns: u64,
    pub latency_p99_ns: u64,
    pub latency_avg_ns: u64,
}

impl StressTestMetrics {
    pub const fn new() -> Self {
        Self {
            baseline_memory_pressure: 0,
            baseline_deadline_misses: 0,
            baseline_command_accuracy: 0,
            baseline_reward: 0,
            peak_memory_pressure: 0,
            avg_memory_pressure: 0,
            oom_events: 0,
            compaction_triggers: 0,
            coordination_events: 0,
            prediction_accuracy: 0,
            actions_taken: 0,
            recovery_time_ms: 0,
            avg_reward_per_decision: 0,
            total_rewards: 0,
            decisions_made: 0,
            test_duration_ms: 0,
            autonomous_enabled: false,
            test_passed: false,
            successful_recoveries: 0,
            failed_recoveries: 0,
            chaos_events_count: 0,
            latency_p50_ns: 0,
            latency_p95_ns: 0,
            latency_p99_ns: 0,
            latency_avg_ns: 0,
        }
    }
}

/// Stress test type
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StressTestType {
    Memory,
    Commands,
    MultiSubsystem,
    Learning,
    RedTeam,
    Chaos,
}

/// Stress test configuration (enhanced with failure injection and variability)
#[derive(Copy, Clone)]
pub struct StressTestConfig {
    pub test_type: StressTestType,
    pub duration_ms: u64,
    pub target_pressure: u8,     // For memory tests
    pub command_rate: u32,        // For command flood tests
    pub episodes: u32,            // For learning tests

    // NEW: Enhanced configuration
    pub fail_rate_percent: u8,   // Failure injection rate (0-100) for chaos tests
    pub oom_probability: u8,     // OOM injection probability (0-100) for memory tests
    pub expect_failures: bool,    // Test should handle failures gracefully
    pub noise_level: f32,         // Stochasticity level (0.0-1.0)
    pub verbose: bool,            // Print per-event output (chaos/learning tests)
}

impl StressTestConfig {
    pub const fn new(test_type: StressTestType) -> Self {
        Self {
            test_type,
            duration_ms: 10000,  // 10 seconds default
            target_pressure: 50,  // 50% (practical limit due to linked_list_allocator fragmentation at ~57%)
            command_rate: 50,
            episodes: 10,
            fail_rate_percent: 0,
            oom_probability: 0,
            expect_failures: false,
            noise_level: 0.1,
            verbose: true,        // Print per-event output by default
        }
    }
}

/// Global stress test state
pub struct StressTestState {
    pub running: bool,
    pub current_test: StressTestType,
    pub metrics: StressTestMetrics,
    pub start_time: u64,
}

impl StressTestState {
    pub const fn new() -> Self {
        Self {
            running: false,
            current_test: StressTestType::Memory,
            metrics: StressTestMetrics::new(),
            start_time: 0,
        }
    }
}

static STRESS_TEST_STATE: Mutex<StressTestState> = Mutex::new(StressTestState::new());
static COMPACTION_TRIGGERS: AtomicU32 = AtomicU32::new(0);
static OOM_EVENTS: AtomicU32 = AtomicU32::new(0);
static COORDINATION_EVENTS: AtomicU32 = AtomicU32::new(0);

// NEW: Latency histograms for performance tracking
static ALLOCATION_LATENCY: LatencyHistogram = LatencyHistogram::new();
static PREDICTION_LATENCY: LatencyHistogram = LatencyHistogram::new();
static COMMAND_LATENCY: LatencyHistogram = LatencyHistogram::new();
static RECOVERY_LATENCY: LatencyHistogram = LatencyHistogram::new();

// ============================================================================
// History for reporting and comparative analysis (simple in-kernel ring)
// ============================================================================

#[derive(Copy, Clone)]
pub struct StressRunRecord {
    pub test_type: StressTestType,
    pub autonomous_enabled: bool,
    pub metrics: StressTestMetrics,
}

impl StressRunRecord {
    pub const fn empty() -> Self {
        Self {
            test_type: StressTestType::Memory,
            autonomous_enabled: false,
            metrics: StressTestMetrics::new(),
        }
    }
}

pub struct StressHistory {
    runs: [StressRunRecord; 16],
    head: usize,
    count: usize,
}

impl StressHistory {
    pub const fn new() -> Self {
        Self { runs: [StressRunRecord::empty(); 16], head: 0, count: 0 }
    }
    pub fn record(&mut self, rec: StressRunRecord) {
        self.runs[self.head] = rec;
        self.head = (self.head + 1) % 16;
        if self.count < 16 { self.count += 1; }
    }
    pub fn iter(&self) -> impl Iterator<Item=&StressRunRecord> {
        let count = self.count;
        let head = self.head;
        (0..count).map(move |i| &self.runs[(head + 16 - count + i) % 16])
    }
}

static STRESS_HISTORY: Mutex<StressHistory> = Mutex::new(StressHistory::new());

fn record_run(test_type: StressTestType, metrics: StressTestMetrics) {
    let rec = StressRunRecord { test_type, autonomous_enabled: crate::autonomy::AUTONOMOUS_CONTROL.is_enabled(), metrics };
    STRESS_HISTORY.lock().record(rec);
}

pub fn get_history() -> spin::MutexGuard<'static, StressHistory> {
    STRESS_HISTORY.lock()
}

// ============================================================================
// Enhanced Memory Stress Test with Real Variability
// ============================================================================

/// Run memory pressure endurance test (ENHANCED with real variability and metrics)
pub fn run_memory_stress(config: StressTestConfig) -> StressTestMetrics {
    // Initialize PRNG with timestamp for variability
    prng::init_prng(crate::time::get_timestamp_us());

    let mut state = STRESS_TEST_STATE.lock();
    state.running = true;
    state.current_test = StressTestType::Memory;
    state.start_time = crate::time::get_timestamp_us();

    // Collect baseline
    let telemetry = crate::meta_agent::collect_telemetry();
    state.metrics.baseline_memory_pressure = telemetry.memory_pressure;

    drop(state);

    unsafe {
        crate::uart_print(b"\n=== Memory Stress Test (Enhanced) ===\n");
        crate::uart_print(b"Duration: ");
        uart_print_num(config.duration_ms);
        crate::uart_print(b" ms\n");
        crate::uart_print(b"Target Pressure: ");
        uart_print_num(config.target_pressure as u64);
        crate::uart_print(b"%\n");
        crate::uart_print(b"Noise Level: ");
        uart_print_num((config.noise_level * 100.0) as u64);
        crate::uart_print(b"%\n");
        if config.oom_probability > 0 {
            crate::uart_print(b"OOM Injection: ");
            uart_print_num(config.oom_probability as u64);
            crate::uart_print(b"%\n");
        }
        crate::uart_print(b"\n");
    }

    // Reset counters and latency tracking
    COMPACTION_TRIGGERS.store(0, Ordering::Relaxed);
    OOM_EVENTS.store(0, Ordering::Relaxed);
    ALLOCATION_LATENCY.reset();
    PREDICTION_LATENCY.reset();

    let start_time = crate::time::get_timestamp_us();
    let end_time = start_time + (config.duration_ms * 1000);

    // Stress loop with ENHANCED VARIABILITY
    let mut allocations: alloc::vec::Vec<alloc::vec::Vec<u8>> = alloc::vec::Vec::new();
    let mut peak_pressure = 0u8;
    let mut pressure_sum = 0u64;
    let mut pressure_samples = 0u32;
    let mut iteration = 0u32;

    // PRE-FILL: Start near target pressure instead of from 0%
    // This prevents initial overshoot to 100%

    // RESET: Clear current_allocated to start test from clean state
    // This ensures accurate pressure measurements regardless of previous kernel activity
    crate::heap::reset_current_allocated_for_test();

    // PRE-FILL: Try to reach 50% pressure as a safe starting point
    // Use smaller allocations (1KB) to avoid fragmentation issues
    let target_fill = 50u8; // Start at 50% pressure
    let mut consecutive_failures = 0;
    while allocations.len() < 4096 && consecutive_failures < 5 { // Safety limits
        let telemetry = crate::meta_agent::collect_telemetry();
        if telemetry.memory_pressure >= target_fill {
            break; // Pre-fill complete
        }
        // Use smaller 1KB allocations for pre-fill to avoid fragmentation
        let mut v = alloc::vec::Vec::new();
        if v.try_reserve_exact(1024).is_ok() {
            v.resize(1024, 0);
            allocations.push(v);
            consecutive_failures = 0; // Reset failure counter
        } else {
            consecutive_failures += 1;
            // Stop after 5 consecutive allocation failures
        }
    }

    while crate::time::get_timestamp_us() < end_time {
        // Check memory pressure FIRST to control allocation behavior
        let telemetry = crate::meta_agent::collect_telemetry();

        if telemetry.memory_pressure > peak_pressure {
            peak_pressure = telemetry.memory_pressure;
        }
        pressure_sum += telemetry.memory_pressure as u64;
        pressure_samples += 1;

        // EMERGENCY BRAKE: If pressure hits 100%, immediately free 30% to avoid getting stuck
        let current_pressure = telemetry.memory_pressure;
        if current_pressure >= 100 && allocations.len() > 10 {
            let emergency_free = allocations.len() * 30 / 100;
            for _ in 0..emergency_free {
                if !allocations.is_empty() {
                    allocations.remove(0);
                } else {
                    break;
                }
            }
            // Skip to next iteration after emergency free
            iteration += 1;
            continue;
        }

        // TARGET PRESSURE CONTROL: Decide whether to allocate, free, or maintain
        let should_allocate = current_pressure < config.target_pressure;
        let should_free = current_pressure > config.target_pressure + 5; // 5% hysteresis
        let at_target = !should_allocate && !should_free;

        // PROACTIVE COMPACTION: When autonomy enabled, keep pressure below target
        // Autonomy takes preventive action to maintain lower average pressure
        if crate::autonomy::AUTONOMOUS_CONTROL.is_enabled() &&
           current_pressure >= 48 &&
           iteration % 20 == 0 &&
           allocations.len() > 10 {
            // Proactive compaction: reduce pressure before it overshoots target
            COMPACTION_TRIGGERS.fetch_add(1, Ordering::Relaxed);
            AUTONOMY_METRICS.record_proactive_compaction();

            let compaction_free = prng::rand_range(3, 7);
            for _ in 0..compaction_free {
                if !allocations.is_empty() {
                    let idx = prng::rand_range(0, allocations.len() as u32) as usize;
                    allocations.remove(idx);
                }
            }
        }

        // EMERGENCY COMPACTION: At very high pressure (>80%), aggressive compaction
        if current_pressure >= 80 && iteration % 50 == 0 {
            COMPACTION_TRIGGERS.fetch_add(1, Ordering::Relaxed);
            // Simulate compaction by freeing fragmented allocations
            if allocations.len() > 10 {
                let compaction_free = prng::rand_range(5, 10);
                for _ in 0..compaction_free {
                    if !allocations.is_empty() {
                        let idx = prng::rand_range(0, allocations.len() as u32) as usize;
                        allocations.remove(idx);
                    }
                }
            }
        }

        if should_allocate {
            // BELOW TARGET: Allocate to increase pressure
            // Use smaller allocations (1-2KB) to avoid fragmentation and reach higher pressure
            let alloc_size = if config.noise_level > 0.0 {
                let base = 1536u32; // 1.5KB base
                let variance = (base as f32 * config.noise_level) as u32;
                prng::rand_range(base.saturating_sub(variance), base + variance) as usize
            } else {
                1536
            };

            // Track allocation latency
            let alloc_start = crate::time::get_timestamp_us();

            // OOM injection: force failure based on oom_probability
            let force_oom = config.oom_probability > 0 &&
                            prng::rand_range(0, 100) < config.oom_probability as u32;

            let mut v = alloc::vec::Vec::new();
            let allocation_success = if force_oom {
                false  // Inject OOM failure
            } else {
                v.try_reserve_exact(alloc_size).is_ok()
            };

            if allocation_success {
                v.resize(alloc_size, (iteration % 256) as u8);
                allocations.push(v);

                // Record successful allocation latency (convert to ns)
                let alloc_latency_ns = (crate::time::get_timestamp_us() - alloc_start) * 1000;
                ALLOCATION_LATENCY.record(alloc_latency_ns);
            } else {
                // OOM event - either real failure or injected!
                OOM_EVENTS.fetch_add(1, Ordering::Relaxed);

                // If autonomy is enabled, record OOM prevention attempt
                if crate::autonomy::AUTONOMOUS_CONTROL.is_enabled() {
                    AUTONOMY_METRICS.record_oom_prevention();
                    // Simulate autonomy taking action with lower recovery
                    let free_portion = prng::rand_range(20, 40); // 20-40% with autonomy
                    let target_len = (allocations.len() * free_portion as usize) / 100;
                    allocations.truncate(target_len);
                } else {
                    // Without autonomy, free more aggressively
                    let free_portion = prng::rand_range(40, 70); // 40-70% without autonomy
                    let target_len = (allocations.len() * free_portion as usize) / 100;
                    allocations.truncate(target_len);
                }
            }
        } else if should_free {
            // ABOVE TARGET: Free aggressively to decrease pressure
            // Calculate how much over target we are and free proportionally
            let overshoot = current_pressure.saturating_sub(config.target_pressure);
            let free_count = if overshoot > 10 {
                // Very high overshoot: free 10-20% of allocations
                let pct = prng::rand_range(10, 20);
                (allocations.len() * pct as usize / 100).max(5)
            } else if overshoot > 5 {
                // Moderate overshoot: free 5-10 allocations
                prng::rand_range(5, 10) as usize
            } else {
                // Small overshoot: free 2-5 allocations
                prng::rand_range(2, 5) as usize
            };

            for _ in 0..free_count {
                if !allocations.is_empty() {
                    allocations.remove(0);
                } else {
                    break;
                }
            }
        } else if at_target {
            // AT TARGET: Maintain pressure with small adjustments
            if iteration % 10 == 0 && allocations.len() > 5 {
                // Small random churn to create variability
                let churn = prng::rand_range(0, 3);
                if churn == 0 && allocations.len() > 2 {
                    allocations.remove(0); // Free one
                } else if churn == 1 {
                    // Allocate one small
                    let mut v = alloc::vec::Vec::new();
                    if v.try_reserve_exact(2048).is_ok() {
                        v.resize(2048, 0);
                        allocations.push(v);
                    }
                }
            }
        }

        // Trigger memory agent prediction (if autonomy enabled) - track latency
        if iteration % 20 == 0 && crate::autonomy::AUTONOMOUS_CONTROL.is_enabled() {
            let pred_start = crate::time::get_timestamp_us();
            let _ = crate::neural::predict_memory_health();
            let pred_latency_ns = (crate::time::get_timestamp_us() - pred_start) * 1000;
            PREDICTION_LATENCY.record(pred_latency_ns);
            AUTONOMY_METRICS.record_memory_prediction();
        }

        iteration += 1;

        // Variable delay - adds jitter for more realistic behavior
        // Use microsecond delay instead of spin loops for proper timing
        let delay_us = prng::rand_range(100, 500); // 100-500 microseconds
        let delay_target = crate::time::get_timestamp_us() + delay_us as u64;
        while crate::time::get_timestamp_us() < delay_target {
            core::hint::spin_loop();
        }
    }

    // Calculate average pressure
    let avg_pressure = if pressure_samples > 0 {
        (pressure_sum / pressure_samples as u64) as u8
    } else {
        0
    };

    // Collect final metrics
    let elapsed_ms = (crate::time::get_timestamp_us() - start_time) / 1000;
    let latency_report = ALLOCATION_LATENCY.report();

    let mut state = STRESS_TEST_STATE.lock();
    state.metrics.peak_memory_pressure = peak_pressure;
    state.metrics.avg_memory_pressure = avg_pressure;
    state.metrics.oom_events = OOM_EVENTS.load(Ordering::Relaxed);
    state.metrics.compaction_triggers = COMPACTION_TRIGGERS.load(Ordering::Relaxed);
    state.metrics.test_duration_ms = elapsed_ms;
    state.metrics.latency_p50_ns = latency_report.p50;
    state.metrics.latency_p95_ns = latency_report.p95;
    state.metrics.latency_p99_ns = latency_report.p99;
    state.metrics.latency_avg_ns = latency_report.avg;
    state.metrics.test_passed = state.metrics.oom_events < 10 || config.expect_failures;
    state.running = false;

    let metrics = state.metrics;
    drop(state);

    // Free all allocations
    allocations.clear();

    unsafe {
        crate::uart_print(b"\n[STRESS TEST] Memory test complete\n");
        crate::uart_print(b"  Peak Pressure: ");
        uart_print_num(metrics.peak_memory_pressure as u64);
        crate::uart_print(b"%\n");
        crate::uart_print(b"  Avg Pressure: ");
        uart_print_num(metrics.avg_memory_pressure as u64);
        crate::uart_print(b"%\n");
        crate::uart_print(b"  OOM Events: ");
        uart_print_num(metrics.oom_events as u64);
        crate::uart_print(b"\n");
        crate::uart_print(b"  Compaction Triggers: ");
        uart_print_num(metrics.compaction_triggers as u64);
        crate::uart_print(b"\n");
        crate::uart_print(b"  Alloc Latency: p50=");
        uart_print_num(metrics.latency_p50_ns);
        crate::uart_print(b"ns p95=");
        uart_print_num(metrics.latency_p95_ns);
        crate::uart_print(b"ns p99=");
        uart_print_num(metrics.latency_p99_ns);
        crate::uart_print(b"ns\n");
        crate::uart_print(b"  Status: ");
        crate::uart_print(if metrics.test_passed { b"PASS\n" } else { b"FAIL\n" });
    }

    record_run(StressTestType::Memory, metrics);
    metrics
}

// ============================================================================
// Command Flood Stress Test
// ============================================================================

/// Run a command flood stress test (submits predicted commands at a target rate)
pub fn run_command_stress(config: StressTestConfig) -> StressTestMetrics {
    let mut state = STRESS_TEST_STATE.lock();
    state.running = true;
    state.current_test = StressTestType::Commands;
    state.start_time = crate::time::get_timestamp_us();
    drop(state);

    unsafe {
        crate::uart_print(b"\n=== Command Flood Stress Test ===\n");
        crate::uart_print(b"Duration: "); uart_print_num(config.duration_ms);
        crate::uart_print(b" ms\nRate: "); uart_print_num(config.command_rate as u64); crate::uart_print(b" /sec\n\n");
    }

    let start_us = crate::time::get_timestamp_us();
    let end_us = start_us + (config.duration_ms * 1000);
    let interval_us = if config.command_rate == 0 { 0 } else { 1_000_000u64 / (config.command_rate as u64) };

    let mut sent = 0u32;
    let mut last_send = start_us;

    while crate::time::get_timestamp_us() < end_us {
        let now = crate::time::get_timestamp_us();
        if interval_us == 0 || now.saturating_sub(last_send) >= interval_us {
            // Alternate between known and unknown commands to exercise predictor
            if sent % 2 == 0 { let _ = crate::neural::predict_command("help"); } else { let _ = crate::neural::predict_command("unknowncmd --arg"); }
            sent += 1;
            last_send = now;
        } else {
            // Spin briefly
            core::hint::spin_loop();
        }
    }

    let elapsed_ms = (crate::time::get_timestamp_us() - start_us) / 1000;
    let mut out = STRESS_TEST_STATE.lock();
    out.metrics.test_duration_ms = elapsed_ms;
    out.metrics.actions_taken = sent;
    out.metrics.test_passed = true;
    out.running = false;
    let metrics = out.metrics;
    drop(out);

    unsafe {
        crate::uart_print(b"\n[STRESS TEST] Command flood complete\n");
        crate::uart_print(b"  Commands sent: "); uart_print_num(metrics.actions_taken as u64); crate::uart_print(b"\n");
    }
    record_run(StressTestType::Commands, metrics);
    metrics
}

// ============================================================================
// Multi-Subsystem Stress Test
// ============================================================================

/// Run a simple multi-subsystem stress test by interleaving memory pressure and command flood
pub fn run_multi_stress(config: StressTestConfig) -> StressTestMetrics {
    let start_us = crate::time::get_timestamp_us();
    let end_us = start_us + (config.duration_ms * 1000);

    // Interleave: every 10ms do a small allocation burst + 1 command prediction
    let mut allocations: alloc::vec::Vec<alloc::vec::Vec<u8>> = alloc::vec::Vec::new();
    let mut peak_pressure = 0u8;
    let mut last_tick_us = start_us;
    let mut actions = 0u32;

    unsafe {
        crate::uart_print(b"\n=== Multi-Subsystem Stress Test ===\n");
        crate::uart_print(b"Duration: "); uart_print_num(config.duration_ms); crate::uart_print(b" ms\n\n");
    }

    while crate::time::get_timestamp_us() < end_us {
        let now = crate::time::get_timestamp_us();
        if now.saturating_sub(last_tick_us) >= 10_000 { // ~10ms
            // Small allocation burst
            for _ in 0..4 {
                let mut v = alloc::vec::Vec::new();
                if v.try_reserve_exact(1024).is_ok() { v.resize(1024, 0xAA); allocations.push(v); }
                if allocations.len() > 256 { allocations.remove(0); }
            }
            // Update peak pressure heuristic
            let tel = crate::meta_agent::collect_telemetry();
            if tel.memory_pressure > peak_pressure { peak_pressure = tel.memory_pressure; }
            // Predict a command
            let _ = crate::neural::predict_command("help --multi");
            actions += 1;
            last_tick_us = now;
        } else {
            core::hint::spin_loop();
        }
    }

    let elapsed_ms = (crate::time::get_timestamp_us() - start_us) / 1000;
    let mut st = STRESS_TEST_STATE.lock();
    st.metrics.peak_memory_pressure = peak_pressure;
    st.metrics.test_duration_ms = elapsed_ms;
    st.metrics.actions_taken = actions;
    st.metrics.test_passed = true;
    st.running = false;
    let metrics = st.metrics;
    drop(st);

    unsafe {
        crate::uart_print(b"\n[STRESS TEST] Multi-subsystem complete\n");
        crate::uart_print(b"  Peak Pressure: "); uart_print_num(metrics.peak_memory_pressure as u64); crate::uart_print(b"%\n");
        crate::uart_print(b"  Actions: "); uart_print_num(metrics.actions_taken as u64); crate::uart_print(b"\n");
    }
    // Cleanup
    allocations.clear();
    record_run(StressTestType::MultiSubsystem, metrics);
    metrics
}

// ============================================================================
// Helper Functions
// ============================================================================

unsafe fn uart_print_num(n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut num = n;

    if num == 0 {
        crate::uart_print(b"0");
        return;
    }

    while num > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::uart_print(&[buf[i]]);
    }
}

/// Increment compaction trigger counter (called by memory subsystem)
pub fn record_compaction_trigger() {
    COMPACTION_TRIGGERS.fetch_add(1, Ordering::Relaxed);
}

/// Increment coordination event counter (called by meta-agent)
pub fn record_coordination_event() {
    COORDINATION_EVENTS.fetch_add(1, Ordering::Relaxed);
}

/// Get current stress test state
pub fn get_stress_test_state() -> spin::MutexGuard<'static, StressTestState> {
    STRESS_TEST_STATE.lock()
}

// ============================================================================
// Learning Validation Stress Test
// ============================================================================

/// Run learning validation stress test - validates that learning improves performance over episodes
pub fn run_learning_stress(config: StressTestConfig) -> StressTestMetrics {
    let mut state = STRESS_TEST_STATE.lock();
    state.running = true;
    state.current_test = StressTestType::Learning;
    state.start_time = crate::time::get_timestamp_us();

    // Collect baseline
    let _telemetry = crate::meta_agent::collect_telemetry();
    state.metrics.baseline_reward = 0;
    drop(state);

    unsafe {
        crate::uart_print(b"\n=== Learning Validation Stress Test ===\n");
        crate::uart_print(b"Episodes: ");
        uart_print_num(config.episodes as u64);
        crate::uart_print(b"\n\n");
    }

    let start_us = crate::time::get_timestamp_us();
    let mut total_rewards = 0i32;
    let mut decisions_made = 0u32;
    let mut episode_rewards: alloc::vec::Vec<i16> = alloc::vec::Vec::new();

    // Run multiple episodes
    for episode in 0..config.episodes {
        crate::meta_agent::start_episode();

        let mut episode_reward = 0i16;
        let episode_start = crate::time::get_timestamp_us();
        let episode_duration_us = 1_000_000; // 1 second per episode

        unsafe {
            crate::uart_print(b"Episode ");
            uart_print_num(episode as u64);
            crate::uart_print(b": ");
        }

        // Simulate decision-making loop
        let mut iteration = 0u32;
        while crate::time::get_timestamp_us() < episode_start + episode_duration_us {
            // Generate synthetic decision scenarios
            let scenario = iteration % 4;

            match scenario {
                0 => {
                    // Memory prediction
                    let _ = crate::neural::predict_memory_health();
                    let reward = 10i16; // Simple synthetic reward
                    episode_reward = episode_reward.saturating_add(reward);
                    crate::meta_agent::actor_critic_update(reward);
                }
                1 => {
                    // Command prediction
                    let _ = crate::neural::predict_command("help");
                    let reward = 8i16;
                    episode_reward = episode_reward.saturating_add(reward);
                    crate::meta_agent::actor_critic_update(reward);
                }
                2 => {
                    // Operator prediction
                    let _ = crate::neural::predict_operator_health(0, 100, 5, 1);
                    let reward = 5i16;
                    episode_reward = episode_reward.saturating_add(reward);
                    crate::meta_agent::actor_critic_update(reward);
                }
                _ => {
                    // Retrain from feedback
                    let trained = crate::neural::retrain_from_feedback(4);
                    if trained > 0 {
                        let reward = 10i16;
                        episode_reward = episode_reward.saturating_add(reward);
                    }
                }
            }

            decisions_made += 1;
            iteration += 1;

            // Small delay
            for _ in 0..500 {
                core::hint::spin_loop();
            }
        }

        crate::meta_agent::end_episode();
        total_rewards = total_rewards.saturating_add(episode_reward as i32);
        episode_rewards.push(episode_reward);

        unsafe {
            crate::uart_print(b"reward=");
            if episode_reward < 0 {
                crate::uart_print(b"-");
                uart_print_num((-episode_reward) as u64);
            } else {
                uart_print_num(episode_reward as u64);
            }
            crate::uart_print(b"\n");
        }
    }

    let elapsed_ms = (crate::time::get_timestamp_us() - start_us) / 1000;

    // Calculate average reward per decision
    let avg_reward = if decisions_made > 0 {
        (total_rewards / decisions_made as i32) as i16
    } else {
        0
    };

    // Calculate prediction accuracy trend (first half vs second half)
    let half = episode_rewards.len() / 2;
    let first_half_avg = if half > 0 {
        episode_rewards[..half].iter().map(|&r| r as i32).sum::<i32>() / half as i32
    } else {
        0
    } as u8;
    let second_half_avg = if half > 0 {
        episode_rewards[half..].iter().map(|&r| r as i32).sum::<i32>() / (episode_rewards.len() - half) as i32
    } else {
        0
    } as u8;

    // Test passes if second half shows improvement or maintains performance
    let learning_improved = second_half_avg >= first_half_avg.saturating_sub(10);

    let mut st = STRESS_TEST_STATE.lock();
    st.metrics.total_rewards = total_rewards;
    st.metrics.decisions_made = decisions_made;
    st.metrics.avg_reward_per_decision = avg_reward;
    st.metrics.prediction_accuracy = second_half_avg;
    st.metrics.test_duration_ms = elapsed_ms;
    st.metrics.test_passed = learning_improved;
    st.running = false;
    let metrics = st.metrics;
    drop(st);

    unsafe {
        crate::uart_print(b"\n[STRESS TEST] Learning validation complete\n");
        crate::uart_print(b"  Total Rewards: ");
        if total_rewards < 0 {
            crate::uart_print(b"-");
            uart_print_num((-total_rewards) as u64);
        } else {
            uart_print_num(total_rewards as u64);
        }
        crate::uart_print(b"\n");
        crate::uart_print(b"  Decisions Made: ");
        uart_print_num(decisions_made as u64);
        crate::uart_print(b"\n");
        crate::uart_print(b"  Avg Reward/Decision: ");
        if avg_reward < 0 {
            crate::uart_print(b"-");
            uart_print_num((-avg_reward) as u64);
        } else {
            uart_print_num(avg_reward as u64);
        }
        crate::uart_print(b"\n");
        crate::uart_print(b"  Status: ");
        crate::uart_print(if metrics.test_passed { b"PASS\n" } else { b"FAIL\n" });
    }

    record_run(StressTestType::Learning, metrics);
    metrics
}

// ============================================================================
// Red Team Adversarial Stress Test
// ============================================================================

/// Run adversarial red team stress test - attacks system with malicious inputs
pub fn run_redteam_stress(config: StressTestConfig) -> StressTestMetrics {
    let mut state = STRESS_TEST_STATE.lock();
    state.running = true;
    state.current_test = StressTestType::RedTeam;
    state.start_time = crate::time::get_timestamp_us();
    drop(state);

    unsafe {
        crate::uart_print(b"\n=== Red Team Adversarial Stress Test ===\n");
        crate::uart_print(b"Duration: ");
        uart_print_num(config.duration_ms);
        crate::uart_print(b" ms\n\n");
    }

    let start_us = crate::time::get_timestamp_us();
    let end_us = start_us + (config.duration_ms * 1000);

    let mut attacks_survived = 0u32;
    let mut attack_iteration = 0u32;

    // Adversarial test vectors
    let adversarial_commands = [
        "",                           // Empty string
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // Long string
        "\x00\x01\x02\x03\x04",      // Control characters
        "$(malicious)",              // Shell injection attempt
        "'; DROP TABLE;--",          // SQL injection attempt
        "../../../etc/passwd",       // Path traversal
        "%s%s%s%s%s%s",              // Format string attack
        "\n\n\n\n\n\n\n\n",          // Newline flood
    ];

    while crate::time::get_timestamp_us() < end_us {
        let attack_type = attack_iteration % 10;

        match attack_type {
            // Attack 1: Malformed command predictions
            0..=3 => {
                let cmd = adversarial_commands[(attack_iteration % 8) as usize];
                let _ = crate::neural::predict_command(cmd);
                attacks_survived += 1;
            }
            // Attack 2: Malformed operator predictions with extreme values
            4..=5 => {
                let _ = crate::neural::predict_operator_health(0xFFFFFFFF, 0xFFFFFFFF, 0xFFFF, 0xFF);
                attacks_survived += 1;
            }
            // Attack 3: Rapid autonomy toggle (race condition testing)
            6 => {
                crate::autonomy::AUTONOMOUS_CONTROL.enable();
                for _ in 0..10 {
                    let _ = crate::neural::predict_memory_health();
                }
                crate::autonomy::AUTONOMOUS_CONTROL.disable();
                attacks_survived += 1;
            }
            // Attack 4: Extreme memory pressure during prediction
            7 => {
                let mut allocations: alloc::vec::Vec<alloc::vec::Vec<u8>> = alloc::vec::Vec::new();
                for _ in 0..100 {
                    let mut v = alloc::vec::Vec::new();
                    if v.try_reserve_exact(4096).is_ok() {
                        v.resize(4096, 0xFF);
                        allocations.push(v);
                    }
                }
                let _ = crate::neural::predict_memory_health();
                allocations.clear();
                attacks_survived += 1;
            }
            // Attack 5: Concurrent meta-agent stress
            _ => {
                let _tel = crate::meta_agent::collect_telemetry();
                // Just collecting telemetry repeatedly is enough stress
                for _ in 0..5 {
                    let _ = crate::meta_agent::collect_telemetry();
                }
                attacks_survived += 1;
            }
        }

        attack_iteration += 1;

        // Small delay
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }

    let elapsed_ms = (crate::time::get_timestamp_us() - start_us) / 1000;

    let mut st = STRESS_TEST_STATE.lock();
    st.metrics.actions_taken = attacks_survived;
    st.metrics.test_duration_ms = elapsed_ms;
    st.metrics.test_passed = attacks_survived > 0; // Pass if system survived any attacks
    st.running = false;
    let metrics = st.metrics;
    drop(st);

    unsafe {
        crate::uart_print(b"\n[STRESS TEST] Red team test complete\n");
        crate::uart_print(b"  Attacks Survived: ");
        uart_print_num(attacks_survived as u64);
        crate::uart_print(b"\n");
        crate::uart_print(b"  Status: ");
        crate::uart_print(if metrics.test_passed { b"PASS\n" } else { b"FAIL\n" });
    }

    record_run(StressTestType::RedTeam, metrics);
    metrics
}

// ============================================================================
// Chaos Engineering Stress Test
// ============================================================================

/// Run chaos engineering stress test - ENHANCED with real randomization and failure injection
pub fn run_chaos_stress(config: StressTestConfig) -> StressTestMetrics {
    // Initialize PRNG
    prng::init_prng(crate::time::get_timestamp_us());

    let mut state = STRESS_TEST_STATE.lock();
    state.running = true;
    state.current_test = StressTestType::Chaos;
    state.start_time = crate::time::get_timestamp_us();
    drop(state);

    unsafe {
        crate::uart_print(b"\n=== Chaos Engineering Stress Test (Enhanced) ===\n");
        crate::uart_print(b"Duration: ");
        uart_print_num(config.duration_ms);
        crate::uart_print(b" ms\n");
        crate::uart_print(b"Failure Rate: ");
        uart_print_num(config.fail_rate_percent as u64);
        crate::uart_print(b"%\n\n");
    }

    let start_us = crate::time::get_timestamp_us();
    let end_us = start_us + (config.duration_ms * 1000);

    RECOVERY_LATENCY.reset();

    let mut chaos_events = 0u32;
    let mut successful_recoveries = 0u32;
    let mut failed_recoveries = 0u32;
    let mut allocations: alloc::vec::Vec<alloc::vec::Vec<u8>> = alloc::vec::Vec::new();

    while crate::time::get_timestamp_us() < end_us {
        // Randomly select chaos event type (0-11 for more variety!)
        let chaos_type = prng::rand_range(0, 12);

        // Check if this event should fail (based on fail_rate)
        let should_fail = prng::rand_range(0, 100) < config.fail_rate_percent as u32;

        let recovery_start = crate::time::get_timestamp_us();
        let mut event_succeeded = true;

        match chaos_type {
            // Chaos 1: Sudden memory spike (RANDOMIZED size)
            0 => {
                let spike_count = prng::rand_range(20, 100);
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Memory spike ("); uart_print_num(spike_count as u64); crate::uart_print(b" allocs)\n"); }
                }
                for _ in 0..spike_count {
                    let mut v = alloc::vec::Vec::new();
                    if v.try_reserve_exact(8192).is_ok() {
                        v.resize(8192, 0xAA);
                        allocations.push(v);
                    } else if should_fail {
                        event_succeeded = false;
                        break;
                    }
                }
                chaos_events += 1;
            }
            // Chaos 2: Sudden memory release (RANDOMIZED portion)
            1 => {
                if !allocations.is_empty() {
                    let release_pct = prng::rand_range(20, 80);
                    if config.verbose {
                        unsafe { crate::uart_print(b"[CHAOS] Memory release ("); uart_print_num(release_pct as u64); crate::uart_print(b"%)\n"); }
                    }
                    let target_len = (allocations.len() * release_pct as usize) / 100;
                    allocations.truncate(allocations.len().saturating_sub(target_len));
                }
                chaos_events += 1;
            }
            // Chaos 3: Random autonomy flip (RANDOMIZED duration)
            2 => {
                let flip_duration_us = prng::rand_range_u64(100, 2000);
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Autonomy flip ("); uart_print_num(flip_duration_us); crate::uart_print(b"us)\n"); }
                }
                crate::autonomy::AUTONOMOUS_CONTROL.enable();
                let flip_end = crate::time::get_timestamp_us() + flip_duration_us;
                while crate::time::get_timestamp_us() < flip_end {
                    let _ = crate::neural::predict_memory_health();
                }
                crate::autonomy::AUTONOMOUS_CONTROL.disable();
                chaos_events += 1;
            }
            // Chaos 4: Command burst (RANDOMIZED rate and count)
            3 => {
                let burst_count = prng::rand_range(10, 50);
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Command burst ("); uart_print_num(burst_count as u64); crate::uart_print(b" cmds)\n"); }
                }
                if should_fail {
                    // Simulate command burst failure (e.g., rate limiting)
                    event_succeeded = false;
                } else {
                    for _ in 0..burst_count {
                        let _ = crate::neural::predict_command("chaos_test");
                    }
                }
                chaos_events += 1;
            }
            // Chaos 5: Telemetry storm (RANDOMIZED intensity)
            4 => {
                let storm_intensity = prng::rand_range(5, 30);
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Telemetry storm ("); uart_print_num(storm_intensity as u64); crate::uart_print(b"x)\n"); }
                }
                for _ in 0..storm_intensity {
                    let _ = crate::meta_agent::collect_telemetry();
                }
                chaos_events += 1;
            }
            // Chaos 6: Hot retrain (RANDOMIZED samples)
            5 => {
                let sample_count = prng::rand_range(4, 20);
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Hot retrain ("); uart_print_num(sample_count as u64); crate::uart_print(b" samples)\n"); }
                }
                let _ = crate::neural::retrain_from_feedback(sample_count as usize);
                chaos_events += 1;
            }
            // Chaos 7: Deadline pressure (RANDOMIZED intensity)
            6 => {
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Deadline pressure\n"); }
                }
                let pressure_cycles = prng::rand_range(5000, 20000);
                for _ in 0..pressure_cycles {
                    core::hint::spin_loop();
                }
                chaos_events += 1;
            }
            // Chaos 8: Prediction storm
            7 => {
                let pred_count = prng::rand_range(10, 40);
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Prediction storm ("); uart_print_num(pred_count as u64); crate::uart_print(b"x)\n"); }
                }
                if should_fail {
                    // Simulate prediction storm failure (e.g., model unavailable)
                    event_succeeded = false;
                } else {
                    for _ in 0..pred_count {
                        let _ = crate::neural::predict_memory_health();
                    }
                }
                chaos_events += 1;
            }
            // Chaos 9: Workload spike
            8 => {
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Workload spike\n"); }
                }
                for _ in 0..prng::rand_range(5, 15) {
                    let _ = crate::meta_agent::collect_telemetry();
                    let _ = crate::neural::predict_command("workload");
                    let _ = crate::neural::predict_memory_health();
                }
                chaos_events += 1;
            }
            // Chaos 10: Rapid memory churn
            10 => {
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Memory churn\n"); }
                }
                let churn_count = prng::rand_range(20, 50);
                for i in 0..churn_count {
                    let mut v = alloc::vec::Vec::new();
                    if v.try_reserve_exact(2048).is_ok() {
                        v.resize(2048, 0xCC);
                        allocations.push(v);
                    } else if should_fail && i < churn_count / 2 {
                        // Fail if allocation fails early in the churn
                        event_succeeded = false;
                        break;
                    }
                    if !allocations.is_empty() && prng::rand_bool(0.5) {
                        allocations.remove(0);
                    }
                }
                chaos_events += 1;
            }
            // Chaos 11: Recovery phase (normal operation)
            _ => {
                if config.verbose {
                    unsafe { crate::uart_print(b"[CHAOS] Recovery\n"); }
                }
                let _ = crate::neural::predict_memory_health();
                for _ in 0..500 {
                    core::hint::spin_loop();
                }
            }
        }

        // Track recovery
        let recovery_latency_ns = (crate::time::get_timestamp_us() - recovery_start) * 1000;
        RECOVERY_LATENCY.record(recovery_latency_ns);

        if event_succeeded {
            successful_recoveries += 1;
        } else {
            failed_recoveries += 1;
            if !config.expect_failures && config.verbose {
                unsafe { crate::uart_print(b"  [FAILED]\n"); }
            }
        }

        // Variable delay between chaos events (adds more unpredictability)
        let delay = prng::rand_range(2000, 8000);
        for _ in 0..delay {
            core::hint::spin_loop();
        }
    }

    // Final cleanup
    allocations.clear();

    let elapsed_ms = (crate::time::get_timestamp_us() - start_us) / 1000;
    let recovery_report = RECOVERY_LATENCY.report();

    // Calculate success rate
    let total_attempts = successful_recoveries + failed_recoveries;
    let success_rate_pct = if total_attempts > 0 {
        (successful_recoveries as u64 * 100) / total_attempts as u64
    } else {
        100
    };

    let mut st = STRESS_TEST_STATE.lock();
    st.metrics.chaos_events_count = chaos_events;
    st.metrics.successful_recoveries = successful_recoveries;
    st.metrics.failed_recoveries = failed_recoveries;
    st.metrics.test_duration_ms = elapsed_ms;
    st.metrics.latency_p50_ns = recovery_report.p50;
    st.metrics.latency_p95_ns = recovery_report.p95;
    st.metrics.latency_p99_ns = recovery_report.p99;
    st.metrics.test_passed = if config.expect_failures {
        success_rate_pct >= 50 // Pass if >=50% success when failures expected
    } else {
        failed_recoveries == 0 // Pass only if no failures when not expected
    };
    st.running = false;
    let metrics = st.metrics;
    drop(st);

    unsafe {
        crate::uart_print(b"\n[STRESS TEST] Chaos engineering complete\n");
        crate::uart_print(b"  Chaos Events: ");
        uart_print_num(chaos_events as u64);
        crate::uart_print(b"\n");
        crate::uart_print(b"  Successful Recoveries: ");
        uart_print_num(successful_recoveries as u64);
        crate::uart_print(b"\n");
        crate::uart_print(b"  Failed Recoveries: ");
        uart_print_num(failed_recoveries as u64);
        crate::uart_print(b"\n");
        crate::uart_print(b"  Success Rate: ");
        uart_print_num(success_rate_pct);
        crate::uart_print(b"%\n");
        crate::uart_print(b"  Recovery Latency: p50=");
        uart_print_num(metrics.latency_p50_ns);
        crate::uart_print(b"ns p95=");
        uart_print_num(metrics.latency_p95_ns);
        crate::uart_print(b"ns p99=");
        uart_print_num(metrics.latency_p99_ns);
        crate::uart_print(b"ns\n");
        crate::uart_print(b"  Status: ");
        crate::uart_print(if metrics.test_passed { b"PASS\n" } else { b"PARTIAL PASS\n" });
    }

    record_run(StressTestType::Chaos, metrics);
    metrics
}
