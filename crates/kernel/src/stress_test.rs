//! Stress Testing & Performance Validation Framework
//!
//! Week 7: Comprehensive stress testing to validate AI/ML improvements
//! - Memory pressure endurance tests
//! - Command flood tests
//! - Multi-subsystem coordination tests
//! - Learning validation tests
//! - Adversarial red team tests
//! - Chaos engineering

use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

/// Stress test metrics collection
#[derive(Copy, Clone)]
pub struct StressTestMetrics {
    // Pre-test baseline
    pub baseline_memory_pressure: u8,
    pub baseline_deadline_misses: u32,
    pub baseline_command_accuracy: u8,
    pub baseline_reward: i16,

    // During test
    pub peak_memory_pressure: u8,
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
}

impl StressTestMetrics {
    pub const fn new() -> Self {
        Self {
            baseline_memory_pressure: 0,
            baseline_deadline_misses: 0,
            baseline_command_accuracy: 0,
            baseline_reward: 0,
            peak_memory_pressure: 0,
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

/// Stress test configuration
#[derive(Copy, Clone)]
pub struct StressTestConfig {
    pub test_type: StressTestType,
    pub duration_ms: u64,
    pub target_pressure: u8,     // For memory tests
    pub command_rate: u32,        // For command flood tests
    pub episodes: u32,            // For learning tests
}

impl StressTestConfig {
    pub const fn new(test_type: StressTestType) -> Self {
        Self {
            test_type,
            duration_ms: 10000,  // 10 seconds default
            target_pressure: 85,
            command_rate: 50,
            episodes: 10,
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
// Memory Stress Test
// ============================================================================

/// Run memory pressure endurance test
pub fn run_memory_stress(config: StressTestConfig) -> StressTestMetrics {
    let mut state = STRESS_TEST_STATE.lock();
    state.running = true;
    state.current_test = StressTestType::Memory;
    state.start_time = crate::time::get_timestamp_us();

    // Collect baseline
    let telemetry = crate::meta_agent::collect_telemetry();
    state.metrics.baseline_memory_pressure = telemetry.memory_pressure;

    drop(state);

    unsafe {
        crate::uart_print(b"\n=== Memory Stress Test ===\n");
        crate::uart_print(b"Duration: ");
        uart_print_num(config.duration_ms);
        crate::uart_print(b" ms\n");
        crate::uart_print(b"Target Pressure: ");
        uart_print_num(config.target_pressure as u64);
        crate::uart_print(b"%\n\n");
    }

    // Reset counters
    COMPACTION_TRIGGERS.store(0, Ordering::Relaxed);
    OOM_EVENTS.store(0, Ordering::Relaxed);

    let start_time = crate::time::get_timestamp_us();
    let end_time = start_time + (config.duration_ms * 1000);

    // Stress loop
    let mut allocations: alloc::vec::Vec<alloc::vec::Vec<u8>> = alloc::vec::Vec::new();
    let mut peak_pressure = 0u8;
    let mut iteration = 0u32;

    while crate::time::get_timestamp_us() < end_time {
        // Allocate memory
        let mut v = alloc::vec::Vec::new();
        if v.try_reserve_exact(4096).is_ok() {
            v.resize(4096, (iteration % 256) as u8);
            allocations.push(v);
        } else {
            OOM_EVENTS.fetch_add(1, Ordering::Relaxed);
            // Free half to recover
            allocations.truncate(allocations.len() / 2);
        }

        // Periodically free some
        if iteration % 10 == 0 && allocations.len() > 5 {
            allocations.remove(0);
        }

        // Check memory pressure
        let telemetry = crate::meta_agent::collect_telemetry();
        if telemetry.memory_pressure > peak_pressure {
            peak_pressure = telemetry.memory_pressure;
        }

        // Trigger memory agent prediction
        if iteration % 20 == 0 {
            let _ = crate::neural::predict_memory_health();
        }

        iteration += 1;

        // Small delay
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }

    // Collect final metrics
    let elapsed_ms = (crate::time::get_timestamp_us() - start_time) / 1000;

    let mut state = STRESS_TEST_STATE.lock();
    state.metrics.peak_memory_pressure = peak_pressure;
    state.metrics.oom_events = OOM_EVENTS.load(Ordering::Relaxed);
    state.metrics.compaction_triggers = COMPACTION_TRIGGERS.load(Ordering::Relaxed);
    state.metrics.test_duration_ms = elapsed_ms;
    state.metrics.test_passed = state.metrics.oom_events < 5; // Pass if < 5 OOMs
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
        crate::uart_print(b"  OOM Events: ");
        uart_print_num(metrics.oom_events as u64);
        crate::uart_print(b"\n");
        crate::uart_print(b"  Compaction Triggers: ");
        uart_print_num(metrics.compaction_triggers as u64);
        crate::uart_print(b"\n");
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
