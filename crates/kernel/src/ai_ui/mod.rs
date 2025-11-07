/// AI Integration UI - Phase G.4
///
/// Visualization and control widgets for kernel AI subsystems

pub mod decision_log;
pub mod decision_viewer;
pub mod memory_predictor_viewer;
pub mod scheduling_viewer;
pub mod ai_controls;

pub use decision_log::{
    DecisionEntry, DecisionType, DecisionLog, AIStats,
    log_ai_decision, get_recent_decisions, get_ai_stats, update_ai_outcome
};
pub use decision_viewer::DecisionViewer;
pub use memory_predictor_viewer::MemoryPredictorViewer;
pub use scheduling_viewer::SchedulingViewer;
pub use ai_controls::AIControls;
