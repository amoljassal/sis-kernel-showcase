//! LLM (Large Language Model) Integration Module
//!
//! Provides on-device LLM capabilities including:
//! - Fine-tuning with LoRA (Low-Rank Adaptation)
//! - Real-time inference on system state
//! - Natural language query processing
//! - Command suggestion and execution

pub mod finetune;
pub mod state_inference;

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

/// Initialize all LLM subsystems
pub fn init() {
    finetune::init(FineTuneConfig::default());
    state_inference::init();
    crate::info!("llm: all subsystems initialized");
}
