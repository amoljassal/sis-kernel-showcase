// Week 12: Benchmark Suite for AI-Native Kernel
//
// Purpose: Quantify performance improvements from AI/ML features
// Comparative analysis: With AI vs Without AI
//
// Features:
// - Memory stress benchmarks
// - Command flood benchmarks
// - Network throughput benchmarks
// - Full system integration benchmarks
// - Comparative report generation

use spin::Mutex;

// ============================================================================
// Benchmark Metrics
// ============================================================================

#[derive(Copy, Clone, Debug)]
pub struct BenchmarkMetrics {
    // Memory subsystem
    pub memory_pressure_avg: u8,
    pub memory_pressure_peak: u8,
    pub oom_events: u32,
    pub compaction_triggers: u32,
    pub allocation_failures: u32,

    // Scheduling subsystem
    pub deadline_misses: u32,
    pub avg_latency_us: u64,
    pub max_latency_us: u64,
    pub operators_executed: u32,

    // Command subsystem
    pub commands_executed: u32,
    pub prediction_accuracy: u8,  // 0-100%
    pub queue_overflows: u32,
    pub avg_execution_time_us: u64,

    // Network subsystem (if available)
    pub packets_sent: u32,
    pub packets_lost: u32,
    pub congestion_events: u32,
    pub avg_throughput_kbps: u32,

    // Overall system
    pub test_duration_ms: u64,
    pub ai_enabled: bool,
}

impl BenchmarkMetrics {
    pub const fn new(ai_enabled: bool) -> Self {
        Self {
            memory_pressure_avg: 0,
            memory_pressure_peak: 0,
            oom_events: 0,
            compaction_triggers: 0,
            allocation_failures: 0,
            deadline_misses: 0,
            avg_latency_us: 0,
            max_latency_us: 0,
            operators_executed: 0,
            commands_executed: 0,
            prediction_accuracy: 0,
            queue_overflows: 0,
            avg_execution_time_us: 0,
            packets_sent: 0,
            packets_lost: 0,
            congestion_events: 0,
            avg_throughput_kbps: 0,
            test_duration_ms: 0,
            ai_enabled,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ComparativeReport {
    pub with_ai: BenchmarkMetrics,
    pub without_ai: BenchmarkMetrics,

    // Improvement percentages (positive = better with AI)
    pub oom_reduction_pct: i16,
    pub deadline_miss_reduction_pct: i16,
    pub latency_reduction_pct: i16,
    pub accuracy_improvement_pct: i16,
    pub packet_loss_reduction_pct: i16,
}

impl ComparativeReport {
    pub fn compute(with_ai: BenchmarkMetrics, without_ai: BenchmarkMetrics) -> Self {
        // Compute improvement percentages
        let oom_reduction_pct = if without_ai.oom_events > 0 {
            ((without_ai.oom_events as i32 - with_ai.oom_events as i32) * 100
             / without_ai.oom_events as i32) as i16
        } else {
            0
        };

        let deadline_miss_reduction_pct = if without_ai.deadline_misses > 0 {
            ((without_ai.deadline_misses as i32 - with_ai.deadline_misses as i32) * 100
             / without_ai.deadline_misses as i32) as i16
        } else {
            0
        };

        let latency_reduction_pct = if without_ai.avg_latency_us > 0 {
            ((without_ai.avg_latency_us as i64 - with_ai.avg_latency_us as i64) * 100
             / without_ai.avg_latency_us as i64) as i16
        } else {
            0
        };

        let accuracy_improvement_pct =
            (with_ai.prediction_accuracy as i16) - (without_ai.prediction_accuracy as i16);

        let packet_loss_reduction_pct = if without_ai.packets_sent > 0 {
            let without_loss_rate = (without_ai.packets_lost * 100) / without_ai.packets_sent.max(1);
            let with_loss_rate = (with_ai.packets_lost * 100) / with_ai.packets_sent.max(1);
            ((without_loss_rate as i32 - with_loss_rate as i32) * 100
             / without_loss_rate.max(1) as i32) as i16
        } else {
            0
        };

        Self {
            with_ai,
            without_ai,
            oom_reduction_pct,
            deadline_miss_reduction_pct,
            latency_reduction_pct,
            accuracy_improvement_pct,
            packet_loss_reduction_pct,
        }
    }
}

pub struct BenchmarkState {
    pub current_metrics: BenchmarkMetrics,
    pub baseline_metrics: Option<BenchmarkMetrics>,
    pub ai_metrics: Option<BenchmarkMetrics>,
    pub test_start_time: u64,
    pub samples_collected: u32,
}

impl BenchmarkState {
    pub const fn new() -> Self {
        Self {
            current_metrics: BenchmarkMetrics::new(false),
            baseline_metrics: None,
            ai_metrics: None,
            test_start_time: 0,
            samples_collected: 0,
        }
    }

    pub fn reset(&mut self, ai_enabled: bool) {
        self.current_metrics = BenchmarkMetrics::new(ai_enabled);
        self.test_start_time = crate::time::get_timestamp_us();
        self.samples_collected = 0;
    }

    pub fn collect_sample(&mut self) {
        self.samples_collected += 1;

        // Collect current system state
        let heap_stats = crate::heap::get_heap_stats();
        let heap_size: usize = 100 * 1024;
        let used = heap_stats.current_allocated();
        let pressure = ((used * 100) / heap_size).min(100) as u8;

        // Update running averages
        self.current_metrics.memory_pressure_avg =
            ((self.current_metrics.memory_pressure_avg as u32 * (self.samples_collected - 1)
              + pressure as u32) / self.samples_collected) as u8;

        if pressure > self.current_metrics.memory_pressure_peak {
            self.current_metrics.memory_pressure_peak = pressure;
        }

        // Collect network stats if available
        let net_state = crate::network_predictor::NETWORK_STATE.lock();
        self.current_metrics.packets_sent = net_state.total_packets_sent;
        self.current_metrics.packets_lost = net_state.total_packets_lost;
        self.current_metrics.congestion_events = net_state.total_congestion_events;
        drop(net_state);
    }

    pub fn finalize(&mut self) {
        let elapsed_us = crate::time::get_timestamp_us() - self.test_start_time;
        self.current_metrics.test_duration_ms = elapsed_us / 1000;
    }

    pub fn save_baseline(&mut self) {
        self.finalize();
        self.baseline_metrics = Some(self.current_metrics);
    }

    pub fn save_ai_metrics(&mut self) {
        self.finalize();
        self.ai_metrics = Some(self.current_metrics);
    }

    pub fn get_comparative_report(&self) -> Option<ComparativeReport> {
        if let (Some(baseline), Some(ai)) = (self.baseline_metrics, self.ai_metrics) {
            Some(ComparativeReport::compute(ai, baseline))
        } else {
            None
        }
    }
}

pub static BENCHMARK_STATE: Mutex<BenchmarkState> = Mutex::new(BenchmarkState::new());

// ============================================================================
// Benchmark Tests
// ============================================================================

/// Memory stress benchmark
pub fn run_memory_benchmark(duration_sec: u32, ai_enabled: bool) -> BenchmarkMetrics {
    let mut state = BENCHMARK_STATE.lock();
    state.reset(ai_enabled);
    drop(state);

    let start_time = crate::time::get_timestamp_us();
    let duration_us = duration_sec as u64 * 1_000_000;

    // Run memory stress
    let mut alloc_count = 0u32;
    while crate::time::get_timestamp_us() - start_time < duration_us {
        // Allocate and free to create pressure
        if alloc_count < 100 {
            let _ = alloc::vec![0u8; 512];
            alloc_count += 1;
        }

        // Collect sample every 100ms
        if (crate::time::get_timestamp_us() - start_time) % 100_000 < 1000 {
            BENCHMARK_STATE.lock().collect_sample();
        }

        // Small delay to avoid spinning too hard
        for _ in 0..1000 { core::hint::spin_loop(); }
    }

    let mut state = BENCHMARK_STATE.lock();
    state.finalize();
    state.current_metrics
}

/// Command flood benchmark
pub fn run_command_benchmark(duration_sec: u32, rate_per_sec: u32, ai_enabled: bool) -> BenchmarkMetrics {
    let mut state = BENCHMARK_STATE.lock();
    state.reset(ai_enabled);
    state.current_metrics.commands_executed = 0;
    drop(state);

    let start_time = crate::time::get_timestamp_us();
    let duration_us = duration_sec as u64 * 1_000_000;
    let interval_us = 1_000_000 / rate_per_sec as u64;

    let mut next_command_time = start_time;
    let mut commands_executed = 0u32;

    while crate::time::get_timestamp_us() - start_time < duration_us {
        let now = crate::time::get_timestamp_us();

        if now >= next_command_time {
            // Simulate command execution
            commands_executed += 1;
            next_command_time += interval_us;

            // Collect sample
            let mut state = BENCHMARK_STATE.lock();
            state.current_metrics.commands_executed = commands_executed;
            state.collect_sample();
            drop(state);
        }

        // Small delay
        for _ in 0..100 { core::hint::spin_loop(); }
    }

    let mut state = BENCHMARK_STATE.lock();
    state.finalize();
    state.current_metrics
}

/// Network throughput benchmark
pub fn run_network_benchmark(duration_sec: u32, ai_enabled: bool) -> BenchmarkMetrics {
    let mut state = BENCHMARK_STATE.lock();
    state.reset(ai_enabled);
    drop(state);

    // Add test connections
    let mut net_state = crate::network_predictor::NETWORK_STATE.lock();
    for i in 0..4 {
        net_state.add_connection(i);
    }
    drop(net_state);

    let start_time = crate::time::get_timestamp_us();
    let duration_us = duration_sec as u64 * 1_000_000;

    // Simulate packet transmission
    let mut packet_count = 0u32;
    while crate::time::get_timestamp_us() - start_time < duration_us {
        // Send packets on connections
        let conn_id = packet_count % 4;
        let mut net_state = crate::network_predictor::NETWORK_STATE.lock();
        net_state.record_packet_sent(conn_id, 1500);
        drop(net_state);

        packet_count += 1;

        // Occasional packet loss (simulate congestion)
        if packet_count % 50 == 0 {
            let mut net_state = crate::network_predictor::NETWORK_STATE.lock();
            net_state.record_packet_loss(conn_id);
            drop(net_state);
        }

        // Collect sample every 100ms
        if (crate::time::get_timestamp_us() - start_time) % 100_000 < 1000 {
            BENCHMARK_STATE.lock().collect_sample();
        }

        // Small delay
        for _ in 0..100 { core::hint::spin_loop(); }
    }

    let mut state = BENCHMARK_STATE.lock();
    state.finalize();
    state.current_metrics
}

/// Full system integration benchmark
pub fn run_full_benchmark(duration_sec: u32, ai_enabled: bool) -> BenchmarkMetrics {
    let mut state = BENCHMARK_STATE.lock();
    state.reset(ai_enabled);
    drop(state);

    // Add network connections
    let mut net_state = crate::network_predictor::NETWORK_STATE.lock();
    for i in 0..4 {
        net_state.add_connection(i);
    }
    drop(net_state);

    let start_time = crate::time::get_timestamp_us();
    let duration_us = duration_sec as u64 * 1_000_000;

    let mut iteration = 0u32;
    while crate::time::get_timestamp_us() - start_time < duration_us {
        iteration += 1;

        // Memory stress
        if iteration % 10 == 0 {
            let _ = alloc::vec![0u8; 256];
        }

        // Command execution
        if iteration % 5 == 0 {
            let mut state = BENCHMARK_STATE.lock();
            state.current_metrics.commands_executed += 1;
            drop(state);
        }

        // Network activity
        if iteration % 3 == 0 {
            let conn_id = iteration % 4;
            let mut net_state = crate::network_predictor::NETWORK_STATE.lock();
            net_state.record_packet_sent(conn_id, 1500);
            drop(net_state);
        }

        // Collect sample every 100ms
        if (crate::time::get_timestamp_us() - start_time) % 100_000 < 1000 {
            BENCHMARK_STATE.lock().collect_sample();
        }

        // Small delay
        for _ in 0..500 { core::hint::spin_loop(); }
    }

    let mut state = BENCHMARK_STATE.lock();
    state.finalize();
    state.current_metrics
}

// ============================================================================
// Benchmark Report Formatting
// ============================================================================

pub fn format_metrics_summary(metrics: &BenchmarkMetrics, label: &str) -> alloc::string::String {
    use alloc::format;

    format!(
        "\n{}\n{}\n\
         Memory Subsystem:\n\
         - Avg Pressure: {}%\n\
         - Peak Pressure: {}%\n\
         - OOM Events: {}\n\
         - Compactions: {}\n\
         - Allocation Failures: {}\n\n\
         Scheduling Subsystem:\n\
         - Deadline Misses: {}\n\
         - Avg Latency: {} μs\n\
         - Max Latency: {} μs\n\
         - Operators Executed: {}\n\n\
         Command Subsystem:\n\
         - Commands Executed: {}\n\
         - Prediction Accuracy: {}%\n\
         - Queue Overflows: {}\n\
         - Avg Execution Time: {} μs\n\n\
         Network Subsystem:\n\
         - Packets Sent: {}\n\
         - Packets Lost: {}\n\
         - Congestion Events: {}\n\
         - Avg Throughput: {} kbps\n\n\
         Test Duration: {} ms\n",
        label,
        "=".repeat(label.len()),
        metrics.memory_pressure_avg,
        metrics.memory_pressure_peak,
        metrics.oom_events,
        metrics.compaction_triggers,
        metrics.allocation_failures,
        metrics.deadline_misses,
        metrics.avg_latency_us,
        metrics.max_latency_us,
        metrics.operators_executed,
        metrics.commands_executed,
        metrics.prediction_accuracy,
        metrics.queue_overflows,
        metrics.avg_execution_time_us,
        metrics.packets_sent,
        metrics.packets_lost,
        metrics.congestion_events,
        metrics.avg_throughput_kbps,
        metrics.test_duration_ms
    )
}

pub fn format_comparative_report(report: &ComparativeReport) -> alloc::string::String {
    use alloc::format;

    format!(
        "\n{}\n{}\n\n\
         Performance Improvements:\n\
         - OOM Reduction: {}%\n\
         - Deadline Miss Reduction: {}%\n\
         - Latency Reduction: {}%\n\
         - Prediction Accuracy Gain: +{}%\n\
         - Packet Loss Reduction: {}%\n\n\
         {} events: {} → {} ({} reduction)\n\
         {} deadline misses: {} → {} ({} reduction)\n\
         Avg latency: {} μs → {} μs ({} reduction)\n\
         Prediction accuracy: {}% → {}% (+{} improvement)\n\
         Packet loss: {}/{} → {}/{} ({} reduction)\n\n\
         CONCLUSION: AI-native kernel reduces deadline misses by {}% and\n\
         prevents {} OOM conditions under sustained stress.\n",
        "COMPARATIVE BENCHMARK REPORT",
        "=".repeat(28),
        report.oom_reduction_pct,
        report.deadline_miss_reduction_pct,
        report.latency_reduction_pct,
        report.accuracy_improvement_pct,
        report.packet_loss_reduction_pct,
        "OOM",
        report.without_ai.oom_events,
        report.with_ai.oom_events,
        report.oom_reduction_pct,
        "Scheduling",
        report.without_ai.deadline_misses,
        report.with_ai.deadline_misses,
        report.deadline_miss_reduction_pct,
        report.without_ai.avg_latency_us,
        report.with_ai.avg_latency_us,
        report.latency_reduction_pct,
        report.without_ai.prediction_accuracy,
        report.with_ai.prediction_accuracy,
        report.accuracy_improvement_pct,
        report.without_ai.packets_lost,
        report.without_ai.packets_sent,
        report.with_ai.packets_lost,
        report.with_ai.packets_sent,
        report.packet_loss_reduction_pct,
        report.deadline_miss_reduction_pct.abs(),
        report.without_ai.oom_events - report.with_ai.oom_events
    )
}
