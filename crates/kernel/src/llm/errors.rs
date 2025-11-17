//! LLM Error Types and Handling
//!
//! # Overview
//!
//! Provides comprehensive error types for all LLM operations with:
//! - Detailed error messages
//! - Error codes for programmatic handling
//! - Context information for debugging
//! - Recovery suggestions
//!
//! # Design Philosophy
//!
//! **No Panics in Production**: All fallible operations return `Result<T, LlmError>`
//! instead of panicking. This ensures graceful degradation and debuggability.
//!
//! **Actionable Errors**: Each error includes:
//! - What went wrong
//! - Why it went wrong
//! - How to fix it (when possible)
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::errors::{LlmError, LlmResult};
//!
//! fn load_model(path: &str) -> LlmResult<Model> {
//!     if !file_exists(path) {
//!         return Err(LlmError::ModelNotFound {
//!             path: path.to_string(),
//!         });
//!     }
//!
//!     // ... load model
//!     Ok(model)
//! }
//! ```

use alloc::string::{String, ToString};
use alloc::format;
use core::fmt;

/// Result type for LLM operations
pub type LlmResult<T> = Result<T, LlmError>;

/// Comprehensive LLM error types
#[derive(Debug, Clone)]
pub enum LlmError {
    // Model Loading Errors
    /// Model file not found
    ModelNotFound {
        path: String,
    },

    /// Invalid model format
    InvalidModelFormat {
        path: String,
        reason: String,
    },

    /// Model too large for memory
    ModelTooLarge {
        size: usize,
        available: usize,
    },

    /// Unsupported model version
    UnsupportedVersion {
        version: u32,
        supported: u32,
    },

    // Inference Errors
    /// No model loaded
    NoModelLoaded,

    /// Prompt too long
    PromptTooLong {
        tokens: usize,
        max_tokens: usize,
    },

    /// Context length exceeded
    ContextLengthExceeded {
        requested: usize,
        max_context: usize,
    },

    /// Inference timeout
    InferenceTimeout {
        elapsed_ms: u64,
        timeout_ms: u64,
    },

    /// Inference cancelled
    InferenceCancelled {
        reason: String,
    },

    // Resource Errors
    /// Out of memory
    OutOfMemory {
        requested: usize,
        available: usize,
    },

    /// Too many concurrent inferences
    TooManyConcurrent {
        active: usize,
        max_concurrent: usize,
    },

    /// Token budget exceeded
    TokenBudgetExceeded {
        requested: usize,
        budget: usize,
    },

    // Tokenization Errors
    /// Tokenization failed
    TokenizationFailed {
        text: String,
        reason: String,
    },

    /// Invalid token ID
    InvalidTokenId {
        token_id: u16,
        vocab_size: usize,
    },

    // Backend Errors
    /// Backend not initialized
    BackendNotInitialized,

    /// Backend operation failed
    BackendError {
        operation: String,
        reason: String,
    },

    // Parse Errors
    /// GGUF parse error
    GgufParseError {
        reason: String,
    },

    /// Metadata missing
    MetadataMissing {
        key: String,
    },

    /// Tensor not found
    TensorNotFound {
        name: String,
    },

    // Configuration Errors
    /// Invalid configuration
    InvalidConfig {
        field: String,
        reason: String,
    },

    // Generic Errors
    /// Internal error (should never happen)
    Internal {
        message: String,
    },
}

impl LlmError {
    /// Get error code for programmatic handling
    pub fn code(&self) -> u32 {
        match self {
            LlmError::ModelNotFound { .. } => 1001,
            LlmError::InvalidModelFormat { .. } => 1002,
            LlmError::ModelTooLarge { .. } => 1003,
            LlmError::UnsupportedVersion { .. } => 1004,
            LlmError::NoModelLoaded => 2001,
            LlmError::PromptTooLong { .. } => 2002,
            LlmError::ContextLengthExceeded { .. } => 2003,
            LlmError::InferenceTimeout { .. } => 2004,
            LlmError::InferenceCancelled { .. } => 2005,
            LlmError::OutOfMemory { .. } => 3001,
            LlmError::TooManyConcurrent { .. } => 3002,
            LlmError::TokenBudgetExceeded { .. } => 3003,
            LlmError::TokenizationFailed { .. } => 4001,
            LlmError::InvalidTokenId { .. } => 4002,
            LlmError::BackendNotInitialized => 5001,
            LlmError::BackendError { .. } => 5002,
            LlmError::GgufParseError { .. } => 6001,
            LlmError::MetadataMissing { .. } => 6002,
            LlmError::TensorNotFound { .. } => 6003,
            LlmError::InvalidConfig { .. } => 7001,
            LlmError::Internal { .. } => 9999,
        }
    }

    /// Get error category
    pub fn category(&self) -> &'static str {
        match self {
            LlmError::ModelNotFound { .. }
            | LlmError::InvalidModelFormat { .. }
            | LlmError::ModelTooLarge { .. }
            | LlmError::UnsupportedVersion { .. } => "Model Loading",

            LlmError::NoModelLoaded
            | LlmError::PromptTooLong { .. }
            | LlmError::ContextLengthExceeded { .. }
            | LlmError::InferenceTimeout { .. }
            | LlmError::InferenceCancelled { .. } => "Inference",

            LlmError::OutOfMemory { .. }
            | LlmError::TooManyConcurrent { .. }
            | LlmError::TokenBudgetExceeded { .. } => "Resources",

            LlmError::TokenizationFailed { .. }
            | LlmError::InvalidTokenId { .. } => "Tokenization",

            LlmError::BackendNotInitialized
            | LlmError::BackendError { .. } => "Backend",

            LlmError::GgufParseError { .. }
            | LlmError::MetadataMissing { .. }
            | LlmError::TensorNotFound { .. } => "Parsing",

            LlmError::InvalidConfig { .. } => "Configuration",

            LlmError::Internal { .. } => "Internal",
        }
    }

    /// Get suggested recovery action
    pub fn recovery_suggestion(&self) -> &'static str {
        match self {
            LlmError::ModelNotFound { .. } => {
                "Check that the model file exists at the specified path"
            }
            LlmError::InvalidModelFormat { .. } => {
                "Ensure the model is in GGUF format version 3"
            }
            LlmError::ModelTooLarge { .. } => {
                "Use a smaller model or increase memory allocation"
            }
            LlmError::UnsupportedVersion { .. } => {
                "Convert model to supported GGUF version"
            }
            LlmError::NoModelLoaded => {
                "Load a model using llmctl load <path>"
            }
            LlmError::PromptTooLong { .. } => {
                "Reduce prompt length or increase max_tokens limit"
            }
            LlmError::ContextLengthExceeded { .. } => {
                "Use a shorter conversation history"
            }
            LlmError::InferenceTimeout { .. } => {
                "Increase timeout or use smaller model"
            }
            LlmError::InferenceCancelled { .. } => {
                "Retry the operation"
            }
            LlmError::OutOfMemory { .. } => {
                "Free memory or use smaller model"
            }
            LlmError::TooManyConcurrent { .. } => {
                "Wait for existing inferences to complete"
            }
            LlmError::TokenBudgetExceeded { .. } => {
                "Increase token budget or wait for reset"
            }
            LlmError::TokenizationFailed { .. } => {
                "Check input text encoding"
            }
            LlmError::InvalidTokenId { .. } => {
                "Use token IDs within vocabulary range"
            }
            LlmError::BackendNotInitialized => {
                "Initialize backend with init_backend()"
            }
            LlmError::BackendError { .. } => {
                "Check backend logs for details"
            }
            LlmError::GgufParseError { .. } => {
                "Verify model file integrity"
            }
            LlmError::MetadataMissing { .. } => {
                "Model may be corrupted, re-download"
            }
            LlmError::TensorNotFound { .. } => {
                "Model architecture mismatch"
            }
            LlmError::InvalidConfig { .. } => {
                "Check configuration parameters"
            }
            LlmError::Internal { .. } => {
                "Report this error to maintainers"
            }
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            LlmError::InferenceTimeout { .. }
            | LlmError::InferenceCancelled { .. }
            | LlmError::TooManyConcurrent { .. }
            | LlmError::TokenBudgetExceeded { .. } => true,

            LlmError::Internal { .. }
            | LlmError::OutOfMemory { .. }
            | LlmError::InvalidModelFormat { .. } => false,

            _ => false,
        }
    }
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::ModelNotFound { path } => {
                write!(f, "Model not found: {}", path)
            }
            LlmError::InvalidModelFormat { path, reason } => {
                write!(f, "Invalid model format '{}': {}", path, reason)
            }
            LlmError::ModelTooLarge { size, available } => {
                write!(
                    f,
                    "Model too large: {} bytes (available: {} bytes)",
                    size, available
                )
            }
            LlmError::UnsupportedVersion { version, supported } => {
                write!(
                    f,
                    "Unsupported version: {} (supported: {})",
                    version, supported
                )
            }
            LlmError::NoModelLoaded => {
                write!(f, "No model loaded")
            }
            LlmError::PromptTooLong { tokens, max_tokens } => {
                write!(
                    f,
                    "Prompt too long: {} tokens (max: {})",
                    tokens, max_tokens
                )
            }
            LlmError::ContextLengthExceeded { requested, max_context } => {
                write!(
                    f,
                    "Context length exceeded: {} (max: {})",
                    requested, max_context
                )
            }
            LlmError::InferenceTimeout { elapsed_ms, timeout_ms } => {
                write!(
                    f,
                    "Inference timeout: {} ms (limit: {} ms)",
                    elapsed_ms, timeout_ms
                )
            }
            LlmError::InferenceCancelled { reason } => {
                write!(f, "Inference cancelled: {}", reason)
            }
            LlmError::OutOfMemory { requested, available } => {
                write!(
                    f,
                    "Out of memory: requested {} bytes, available {} bytes",
                    requested, available
                )
            }
            LlmError::TooManyConcurrent { active, max_concurrent } => {
                write!(
                    f,
                    "Too many concurrent inferences: {} active (max: {})",
                    active, max_concurrent
                )
            }
            LlmError::TokenBudgetExceeded { requested, budget } => {
                write!(
                    f,
                    "Token budget exceeded: {} requested (budget: {})",
                    requested, budget
                )
            }
            LlmError::TokenizationFailed { text, reason } => {
                write!(f, "Tokenization failed for '{}': {}", text, reason)
            }
            LlmError::InvalidTokenId { token_id, vocab_size } => {
                write!(
                    f,
                    "Invalid token ID: {} (vocab size: {})",
                    token_id, vocab_size
                )
            }
            LlmError::BackendNotInitialized => {
                write!(f, "Backend not initialized")
            }
            LlmError::BackendError { operation, reason } => {
                write!(f, "Backend error during '{}': {}", operation, reason)
            }
            LlmError::GgufParseError { reason } => {
                write!(f, "GGUF parse error: {}", reason)
            }
            LlmError::MetadataMissing { key } => {
                write!(f, "Metadata missing: {}", key)
            }
            LlmError::TensorNotFound { name } => {
                write!(f, "Tensor not found: {}", name)
            }
            LlmError::InvalidConfig { field, reason } => {
                write!(f, "Invalid config '{}': {}", field, reason)
            }
            LlmError::Internal { message } => {
                write!(f, "Internal error: {}", message)
            }
        }
    }
}

/// Convert from static string error
impl From<&'static str> for LlmError {
    fn from(msg: &'static str) -> Self {
        LlmError::Internal {
            message: msg.to_string(),
        }
    }
}

/// Error context builder
pub struct ErrorContext {
    error: LlmError,
}

impl ErrorContext {
    pub fn new(error: LlmError) -> Self {
        Self { error }
    }

    /// Log error with full context
    pub fn log(&self) {
        crate::error!(
            "[LLM Error] Code: {} | Category: {} | {}",
            self.error.code(),
            self.error.category(),
            self.error
        );
        crate::info!("  Suggestion: {}", self.error.recovery_suggestion());
        crate::info!("  Recoverable: {}", self.error.is_recoverable());
    }

    /// Get the underlying error
    pub fn error(self) -> LlmError {
        self.error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = LlmError::ModelNotFound {
            path: "/test".to_string(),
        };
        assert_eq!(err.code(), 1001);

        let err = LlmError::NoModelLoaded;
        assert_eq!(err.code(), 2001);
    }

    #[test]
    fn test_error_categories() {
        let err = LlmError::ModelNotFound {
            path: "/test".to_string(),
        };
        assert_eq!(err.category(), "Model Loading");

        let err = LlmError::OutOfMemory {
            requested: 100,
            available: 50,
        };
        assert_eq!(err.category(), "Resources");
    }

    #[test]
    fn test_error_display() {
        let err = LlmError::PromptTooLong {
            tokens: 100,
            max_tokens: 50,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
    }

    #[test]
    fn test_error_recoverable() {
        let err = LlmError::InferenceTimeout {
            elapsed_ms: 1000,
            timeout_ms: 500,
        };
        assert!(err.is_recoverable());

        let err = LlmError::InvalidModelFormat {
            path: "/test".to_string(),
            reason: "bad".to_string(),
        };
        assert!(!err.is_recoverable());
    }
}
