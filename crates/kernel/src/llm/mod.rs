//! LLM (Large Language Model) Integration Module
//!
//! Provides on-device LLM capabilities including:
//! - Fine-tuning with LoRA (Low-Rank Adaptation)
//! - Real-time inference on system state
//! - Natural language query processing
//! - Command suggestion and execution
//!
//! ## Phase 2: AI Governance (NEW)
//! - **Drift Detection**: Monitor model performance degradation
//! - **Version Control**: Git-like versioning for LoRA adapters
//!
//! ## Phase 3: Transformer Backend (IN PROGRESS)
//! - **Memory Arena**: Static allocation for deterministic bounds
//! - **Tokenizer**: BPE (Byte-Pair Encoding) implementation
//! - **Quantization**: Q4_0, Q8_0 support for compact models
//! - **Transformer Core**: Multi-layer transformer inference
//! - **GGUF Parser**: Model file format support

pub mod basic;
pub mod finetune;
pub mod state_inference;
pub mod drift_detector;
pub mod version;

// Phase 3: Transformer backend modules
pub mod arena;
pub mod tokenizer;
pub mod quantize;
pub mod transformer;
pub mod gguf;
pub mod backend;
pub mod generate;

// Phase 3: Optimization & Production (M4-M6)
pub mod simd;
pub mod kv_cache;
pub mod sampling;
pub mod errors;
pub mod limits;
pub mod metrics;

// Integration, Testing & Benchmarking
pub mod loader;
pub mod benchmarks;

#[cfg(test)]
pub mod tests;

// Re-export basic LLM functions and types (Phase 0/1 - inference, audit, control)
pub use basic::{
    // Types
    Quantization,
    ModelMeta,
    LlmConfig,
    LlmResult,
    // Functions
    set_pace_scale,
    get_pace_scale,
    set_auto_pace,
    is_auto_pace,
    audit,
    audit_print,
    audit_print_json,
    load_model,
    load_model_meta,
    load_model_with_meta,
    infer,
    stats,
    configure_budget,
    infer_stream,
    ctl_poll,
    ctl_poll_id,
    ctl_cancel,
    ctl_cancel_id,
    ctl_print_sessions,
    ctl_peek_meta,
    verify_demo_model,
    load_model_package,
    demo_hash_for,
};

pub use finetune::{
    LoRAAdapter,
    TrainingExample,
    FineTuneConfig,
    FineTuneStats,
    init as init_finetune,
    add_adapter,
    load_training_data,
    train as finetune_train,
    cancel as finetune_cancel,
    is_training as is_finetuning,
    get_progress as get_finetune_progress,
    get_adapter_size,
};

pub use state_inference::{
    SystemStateSnapshot,
    InferenceResult,
    InferenceStats,
    init as init_state_inference,
    infer_on_state,
    set_auto_execute,
    is_auto_execute,
    get_stats as get_inference_stats,
    record_query,
};

pub use drift_detector::{
    DriftDetector,
    DriftStatus,
    DriftAction,
    DriftMetrics,
    Prediction,
    Trend,
};

pub use version::{
    AdapterVersionControl,
    AdapterVersion,
    VersionMetadata,
    VersionDiff,
    VersionError,
    VersionStats,
    VersionId,
};

// Global instances for Phase 2 LLM Governance components
/// Global drift detector instance (baseline: 90% accuracy)
pub static DRIFT_DETECTOR: DriftDetector = DriftDetector::new_with_default();

/// Global version control instance
pub static VERSION_CONTROL: AdapterVersionControl = AdapterVersionControl::new();

/// Initialize all LLM subsystems
pub fn init() {
    // Phase 0/1: Basic LLM inference and control (no explicit init needed - uses lazy statics)
    // Phase 2: AI Governance components
    finetune::init(FineTuneConfig::default());
    state_inference::init();
    crate::info!("llm: all subsystems initialized");
}
