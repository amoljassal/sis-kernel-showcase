/// AI-powered system insights and predictive features
///
/// This module provides intelligent monitoring and prediction capabilities:
/// - Crash prediction via memory pattern analysis
/// - Automatic memory compaction triggers
/// - System health monitoring

pub mod crash_predictor;

pub use crash_predictor::{
    AllocMetrics, PredictionStatus, PredictionRecord, PredictionOutcome,
    init as init_crash_predictor,
    update_metrics,
    get_status as get_crash_status,
    should_auto_compact,
    get_history as get_crash_history,
    set_threshold as set_crash_threshold,
    reset_peak,
};

/// Initialize all AI insights subsystems
pub fn init() {
    crash_predictor::init();
    crate::info!("ai_insights: all subsystems initialized");
}

/// Gather current memory stats and update crash predictor
pub fn update_memory_insight() {
    // Get current memory stats from buddy allocator
    if let Some(stats) = crate::mm::buddy::get_stats() {
        let timestamp_ms = crate::time::get_uptime_ms();

        // Calculate fragmentation ratio
        // Simple heuristic: ratio of allocated to total
        let fragmentation = if stats.total_pages > 0 {
            stats.allocated_pages as f32 / stats.total_pages as f32
        } else {
            0.0
        };

        // Get largest free block (approximate from buddy allocator)
        // For now, assume it's related to free pages
        let largest_free_block = if stats.free_pages > 0 {
            stats.free_pages / 2 // Rough estimate
        } else {
            0
        };

        let metrics = AllocMetrics {
            timestamp_ms,
            free_pages: stats.free_pages,
            largest_free_block,
            fragmentation_ratio: fragmentation,
            allocation_failures: 0, // Will be tracked separately
        };

        update_metrics(metrics);
    }
}
