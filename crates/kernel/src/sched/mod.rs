//! Intelligent Scheduling Subsystem
//!
//! This module provides AI-powered scheduling capabilities including:
//! - Transformer-based priority calculation with attention mechanism
//! - Historical pattern learning for task optimization
//! - Real-time performance metrics and tuning

pub mod transformer_sched;

pub use transformer_sched::{
    TaskEmbedding,
    SchedulerMetrics,
    init as init_transformer,
    set_enabled as set_transformer_enabled,
    is_enabled as is_transformer_enabled,
    compute_priority as transformer_compute_priority,
    update_outcome as transformer_update_outcome,
    get_metrics as get_transformer_metrics,
    reset as reset_transformer,
};

/// Initialize all scheduler subsystems
pub fn init() {
    transformer_sched::init();
    crate::info!("sched: all subsystems initialized");
}
