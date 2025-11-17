//! LLM Backend Abstraction Layer
//!
//! # Overview
//!
//! This module provides an abstraction layer between the high-level LLM API
//! (`llm::basic`) and the underlying inference implementation. This allows
//! seamless switching between:
//! - **Stub Backend**: Deterministic placeholder for testing
//! - **Transformer Backend**: Real neural network inference
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  llm::basic (Public API)            │
//! │  - infer(), load_model(), etc.      │
//! └───────────────┬─────────────────────┘
//!                 │
//!                 ▼
//! ┌───────────────────────────────────────┐
//! │  Backend Trait (this module)          │
//! └───────────────┬───────────────────────┘
//!                 │
//!     ┌───────────┴────────────┐
//!     │                        │
//!     ▼                        ▼
//! ┌─────────────┐     ┌──────────────────┐
//! │ StubBackend │     │TransformerBackend│
//! │ (Phase 0/1) │     │   (Phase 3)      │
//! └─────────────┘     └──────────────────┘
//! ```
//!
//! # Design Rationale
//!
//! **Why Abstraction?**
//! - **Backward Compatibility**: Keep existing stub working
//! - **Testing**: Easy to mock backends
//! - **Feature Flags**: Compile-time selection
//! - **Future Extensions**: Easy to add new backends (GPU, NPU)
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::backend::{LlmBackend, get_backend};
//!
//! // Get active backend
//! let mut backend = get_backend();
//!
//! // Run inference
//! let result = backend.infer("Hello", 10)?;
//! println!("Output: {}", result.output);
//! ```

use alloc::string::String;
use alloc::boxed::Box;
use alloc::format;
use spin::Mutex;
use crate::llm::basic::LlmResult;

/// LLM Backend Trait
///
/// Defines the interface that all inference backends must implement.
/// This allows swapping between stub and real transformer implementations.
pub trait LlmBackend: Send {
    /// Run inference on prompt
    ///
    /// # Arguments
    ///
    /// - `prompt`: Input text
    /// - `max_tokens`: Maximum number of tokens to generate
    ///
    /// # Returns
    ///
    /// - `Ok(result)`: Inference successful
    /// - `Err(msg)`: Inference failed
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<LlmResult, &'static str>;

    /// Load model from path
    ///
    /// # Arguments
    ///
    /// - `path`: Model file path (e.g., "/models/tiny.gguf")
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Model loaded successfully
    /// - `Err(msg)`: Load failed
    fn load_model(&mut self, path: &str) -> Result<(), &'static str>;

    /// Check if model is loaded
    fn is_loaded(&self) -> bool;

    /// Get backend name (for debugging)
    fn name(&self) -> &'static str;

    /// Get backend statistics
    fn stats(&self) -> BackendStats;
}

/// Backend statistics
#[derive(Debug, Clone, Copy)]
pub struct BackendStats {
    /// Total inferences run
    pub total_inferences: u64,

    /// Total tokens generated
    pub total_tokens: u64,

    /// Average tokens per second
    pub avg_tokens_per_sec: f32,

    /// Whether model is loaded
    pub model_loaded: bool,
}

impl Default for BackendStats {
    fn default() -> Self {
        Self {
            total_inferences: 0,
            total_tokens: 0,
            avg_tokens_per_sec: 0.0,
            model_loaded: false,
        }
    }
}

/// Stub Backend (Phase 0/1)
///
/// Deterministic placeholder that echoes transformed tokens.
/// Used for testing infrastructure without real inference.
pub struct StubBackend {
    model_loaded: bool,
    stats: BackendStats,
}

impl StubBackend {
    /// Create new stub backend
    pub fn new() -> Self {
        Self {
            model_loaded: false,
            stats: BackendStats::default(),
        }
    }
}

impl LlmBackend for StubBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<LlmResult, &'static str> {
        if !self.model_loaded {
            return Err("No model loaded");
        }

        // Stub implementation: echo transformed prompt
        let output = format!("[STUB] {} ...", prompt);

        self.stats.total_inferences += 1;
        self.stats.total_tokens += max_tokens as u64;

        Ok(LlmResult {
            infer_id: self.stats.total_inferences as usize,
            tokens_emitted: max_tokens,
            output,
            latency_us: 1000, // Stub: 1ms
        })
    }

    fn load_model(&mut self, _path: &str) -> Result<(), &'static str> {
        self.model_loaded = true;
        self.stats.model_loaded = true;
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.model_loaded
    }

    fn name(&self) -> &'static str {
        "StubBackend"
    }

    fn stats(&self) -> BackendStats {
        self.stats
    }
}

/// Transformer Backend (Phase 3)
///
/// Real neural network inference using quantized transformer.
///
/// **Status**: Implementation in progress
/// - Tokenizer: ✅ Complete
/// - Quantization: ✅ Complete
/// - Transformer: ✅ Complete
/// - Model Loading: ✅ Complete
/// - Integration: 🚧 In Progress
#[cfg(feature = "llm-transformer")]
pub struct TransformerBackend {
    model: Option<crate::llm::gguf::GgufModel>,
    tokenizer: crate::llm::tokenizer::BpeTokenizer,
    config: crate::llm::transformer::TransformerConfig,
    stats: BackendStats,
}

#[cfg(feature = "llm-transformer")]
impl TransformerBackend {
    /// Create new transformer backend
    pub fn new() -> Self {
        Self {
            model: None,
            tokenizer: crate::llm::tokenizer::BpeTokenizer::new(),
            config: crate::llm::transformer::TransformerConfig::default(),
            stats: BackendStats::default(),
        }
    }
}

#[cfg(feature = "llm-transformer")]
impl LlmBackend for TransformerBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<LlmResult, &'static str> {
        if self.model.is_none() {
            return Err("No model loaded");
        }

        // Tokenize prompt
        let tokens = self.tokenizer.encode(prompt);

        // TODO: Run transformer inference (M3.2)
        // For now, placeholder implementation
        let output = format!("[Transformer] {} tokens", tokens.len());

        self.stats.total_inferences += 1;
        self.stats.total_tokens += max_tokens as u64;

        Ok(LlmResult {
            infer_id: self.stats.total_inferences,
            tokens_emitted: max_tokens,
            output,
            latency_us: 10000, // Placeholder: 10ms
        })
    }

    fn load_model(&mut self, path: &str) -> Result<(), &'static str> {
        // TODO: Load from VFS
        // For now, mark as loaded
        self.stats.model_loaded = true;
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.model.is_some()
    }

    fn name(&self) -> &'static str {
        "TransformerBackend"
    }

    fn stats(&self) -> BackendStats {
        self.stats
    }
}

/// Global backend instance
///
/// Protected by mutex for thread-safe access.
static BACKEND: Mutex<Option<Box<dyn LlmBackend>>> = Mutex::new(None);

/// Initialize backend
///
/// # Arguments
///
/// - `use_transformer`: If true, use transformer backend; otherwise stub
///
/// # Example
///
/// ```no_run
/// // Use stub (default)
/// init_backend(false);
///
/// // Use transformer (requires llm-transformer feature)
/// init_backend(true);
/// ```
pub fn init_backend(use_transformer: bool) {
    let mut backend = BACKEND.lock();

    #[cfg(feature = "llm-transformer")]
    if use_transformer {
        *backend = Some(Box::new(TransformerBackend::new()));
        crate::info!("llm: initialized transformer backend");
        return;
    }

    // Default: stub backend
    *backend = Some(Box::new(StubBackend::new()));
    crate::info!("llm: initialized stub backend");
}

/// Get active backend
///
/// Returns a mutex guard to the active backend.
/// Panics if backend not initialized (call `init_backend()` first).
///
/// # Example
///
/// ```no_run
/// let mut backend_guard = get_backend();
/// if let Some(backend) = backend_guard.as_mut() {
///     backend.infer("Hello", 10)?;
/// }
/// ```
pub fn get_backend() -> spin::MutexGuard<'static, Option<Box<dyn LlmBackend>>> {
    BACKEND.lock()
}

/// Check if backend is initialized
pub fn is_initialized() -> bool {
    BACKEND.lock().is_some()
}

/// Get backend name (for debugging)
pub fn get_backend_name() -> &'static str {
    let backend = BACKEND.lock();
    backend.as_ref()
        .map(|b| b.name())
        .unwrap_or("None")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_backend() {
        let mut backend = StubBackend::new();
        assert_eq!(backend.name(), "StubBackend");
        assert!(!backend.is_loaded());

        // Load model
        backend.load_model("/fake/model").unwrap();
        assert!(backend.is_loaded());

        // Run inference
        let result = backend.infer("Hello", 5).unwrap();
        assert!(result.output.contains("STUB"));
        assert_eq!(result.tokens_emitted, 5);
    }

    #[test]
    fn test_backend_stats() {
        let mut backend = StubBackend::new();
        backend.load_model("/fake/model").unwrap();

        backend.infer("Test", 10).unwrap();
        backend.infer("Test", 10).unwrap();

        let stats = backend.stats();
        assert_eq!(stats.total_inferences, 2);
        assert_eq!(stats.total_tokens, 20);
    }

    #[test]
    fn test_init_backend() {
        init_backend(false);
        assert!(is_initialized());
        assert_eq!(get_backend_name(), "StubBackend");
    }
}
