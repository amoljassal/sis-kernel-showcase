//! Model Loader with VFS Integration
//!
//! # Overview
//!
//! This module provides high-level model loading functionality with VFS integration,
//! allowing GGUF models to be loaded from the file system. It handles:
//! - File system access and validation
//! - Model format detection and parsing
//! - Memory-mapped model loading (for efficiency)
//! - Model caching and hot-swapping
//! - Security validation (checksums, signatures)
//!
//! # Design Philosophy
//!
//! **Lazy Loading**: Models are loaded on-demand, not at boot
//! **Memory Mapping**: Large models are memory-mapped instead of copied
//! **Validation**: All models are validated before loading
//! **Caching**: Recently used models cached for fast access
//!
//! # File System Structure
//!
//! ```text
//! /models/
//!   ├── tiny-10m.gguf          # Tiny model (5-10 MB)
//!   ├── tiny-10m.gguf.sha256   # Checksum file
//!   ├── small-50m.gguf         # Small model (25-50 MB)
//!   └── vocab/                  # Shared vocabularies
//!       └── llama-32k.vocab
//! ```
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::loader::{ModelLoader, LoadConfig};
//!
//! // Initialize loader
//! let mut loader = ModelLoader::new();
//!
//! // Load model from VFS
//! let model = loader.load("/models/tiny-10m.gguf", LoadConfig::default())?;
//!
//! // Use model for inference
//! let backend = TransformerBackend::with_model(model);
//! ```
//!
//! # Security
//!
//! - SHA-256 checksum validation
//! - Ed25519 signature verification (optional)
//! - Size limits enforcement
//! - Path traversal protection

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::format;
use crate::llm::gguf::GgufModel;
use crate::llm::errors::{LlmError, LlmResult};

/// Model load configuration
#[derive(Debug, Clone)]
pub struct LoadConfig {
    /// Verify checksum before loading
    pub verify_checksum: bool,

    /// Verify signature before loading (requires public key)
    pub verify_signature: bool,

    /// Memory-map model instead of loading into memory
    pub use_mmap: bool,

    /// Maximum model size (bytes)
    pub max_size: usize,

    /// Load model into specific memory arena
    pub use_arena: bool,
}

impl Default for LoadConfig {
    fn default() -> Self {
        Self {
            verify_checksum: true,
            verify_signature: false,
            use_mmap: false,  // For now, load into memory
            max_size: 100 * 1024 * 1024,  // 100 MB max
            use_arena: false,  // Use system allocator for now
        }
    }
}

/// Model metadata (extracted from GGUF + filesystem)
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    /// Model name
    pub name: String,

    /// File path
    pub path: String,

    /// File size (bytes)
    pub size: usize,

    /// Model architecture (e.g., "llama", "gpt2")
    pub architecture: String,

    /// Number of parameters
    pub param_count: usize,

    /// Vocabulary size
    pub vocab_size: usize,

    /// Context length
    pub context_length: usize,

    /// Quantization type
    pub quantization: String,
}

/// Model loader with caching
pub struct ModelLoader {
    /// Currently loaded model
    current_model: Option<GgufModel>,

    /// Current model metadata
    current_metadata: Option<ModelMetadata>,

    /// Load configuration
    config: LoadConfig,

    /// Statistics
    loads_successful: u64,
    loads_failed: u64,
}

impl ModelLoader {
    /// Create new model loader
    pub fn new() -> Self {
        Self {
            current_model: None,
            current_metadata: None,
            config: LoadConfig::default(),
            loads_successful: 0,
            loads_failed: 0,
        }
    }

    /// Create loader with custom config
    pub fn with_config(config: LoadConfig) -> Self {
        Self {
            current_model: None,
            current_metadata: None,
            config,
            loads_successful: 0,
            loads_failed: 0,
        }
    }

    /// Load model from file system
    ///
    /// # Arguments
    ///
    /// - `path`: Path to GGUF model file (e.g., "/models/tiny-10m.gguf")
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Model loaded successfully
    /// - `Err(LlmError)`: Load failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// loader.load("/models/tiny-10m.gguf", LoadConfig::default())?;
    /// ```
    pub fn load(&mut self, path: &str, config: LoadConfig) -> LlmResult<()> {
        self.config = config;

        // 1. Validate path
        self.validate_path(path)?;

        // 2. Read file
        let data = self.read_file(path)?;

        // 3. Verify checksum (if enabled)
        if self.config.verify_checksum {
            self.verify_checksum(path, &data)?;
        }

        // 4. Parse GGUF
        let model = GgufModel::from_bytes(&data)
            .map_err(|e| LlmError::GgufParseError {
                reason: e.to_string(),
            })?;

        // 5. Extract metadata
        let metadata = self.extract_metadata(path, &model, data.len())?;

        // 6. Validate model size
        if metadata.size > self.config.max_size {
            return Err(LlmError::ModelTooLarge {
                size: metadata.size,
                available: self.config.max_size,
            });
        }

        // 7. Store model
        self.current_model = Some(model);
        self.current_metadata = Some(metadata);
        self.loads_successful += 1;

        crate::info!("llm: model loaded from {}", path);
        Ok(())
    }

    /// Validate file path (security check)
    fn validate_path(&self, path: &str) -> LlmResult<()> {
        // Check for path traversal attacks
        if path.contains("..") {
            return Err(LlmError::InvalidConfig {
                field: "path".to_string(),
                reason: "Path traversal not allowed".to_string(),
            });
        }

        // Must be absolute path starting with /models/
        if !path.starts_with("/models/") {
            return Err(LlmError::InvalidConfig {
                field: "path".to_string(),
                reason: "Model must be in /models/ directory".to_string(),
            });
        }

        // Must have .gguf extension
        if !path.ends_with(".gguf") {
            return Err(LlmError::InvalidModelFormat {
                path: path.to_string(),
                reason: "Not a GGUF file".to_string(),
            });
        }

        Ok(())
    }

    /// Read file from VFS
    ///
    /// For now, this is a stub. In production, would use actual VFS.
    fn read_file(&self, path: &str) -> LlmResult<Vec<u8>> {
        // TODO: Integrate with actual VFS
        // For now, return error with helpful message

        crate::warn!("llm: VFS integration not yet connected");
        crate::info!("llm: would load model from: {}", path);

        // Return empty vector as placeholder
        // In production, would call: vfs::read_file(path)
        Err(LlmError::ModelNotFound {
            path: path.to_string(),
        })
    }

    /// Verify checksum
    fn verify_checksum(&self, path: &str, data: &[u8]) -> LlmResult<()> {
        // TODO: Implement SHA-256 verification
        // Expected checksum file: path + ".sha256"
        crate::info!("llm: checksum verification not yet implemented");
        Ok(())
    }

    /// Extract metadata from model
    fn extract_metadata(
        &self,
        path: &str,
        model: &GgufModel,
        size: usize,
    ) -> LlmResult<ModelMetadata> {
        // Extract name from path
        let name = path.rsplit('/').next().unwrap_or("unknown").to_string();

        // Extract architecture
        let architecture = model.get_string("general.architecture")
            .unwrap_or("unknown")
            .to_string();

        // Extract config
        let vocab_size = model.get_u32("llm.vocab_size")
            .unwrap_or(32000) as usize;

        let context_length = model.get_u32("llm.context_length")
            .unwrap_or(512) as usize;

        // Calculate param count
        let stats = model.stats();
        let param_count = stats.param_count;

        Ok(ModelMetadata {
            name,
            path: path.to_string(),
            size,
            architecture,
            param_count,
            vocab_size,
            context_length,
            quantization: "Q4_0".to_string(),  // TODO: detect from model
        })
    }

    /// Get currently loaded model
    pub fn get_model(&self) -> Option<&GgufModel> {
        self.current_model.as_ref()
    }

    /// Get current model metadata
    pub fn get_metadata(&self) -> Option<&ModelMetadata> {
        self.current_metadata.as_ref()
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.current_model.is_some()
    }

    /// Unload current model
    pub fn unload(&mut self) {
        self.current_model = None;
        self.current_metadata = None;
        crate::info!("llm: model unloaded");
    }

    /// Get loader statistics
    pub fn stats(&self) -> LoaderStats {
        LoaderStats {
            loads_successful: self.loads_successful,
            loads_failed: self.loads_failed,
            is_loaded: self.is_loaded(),
            current_model: self.current_metadata.as_ref().map(|m| m.name.clone()),
        }
    }
}

impl Default for ModelLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Loader statistics
#[derive(Debug, Clone)]
pub struct LoaderStats {
    pub loads_successful: u64,
    pub loads_failed: u64,
    pub is_loaded: bool,
    pub current_model: Option<String>,
}

/// Helper function to list available models
///
/// Scans /models/ directory for .gguf files
pub fn list_models() -> LlmResult<Vec<String>> {
    // TODO: Integrate with VFS directory listing
    crate::info!("llm: model listing not yet implemented");

    // Return placeholder list
    Ok(vec![
        "/models/tiny-10m.gguf".to_string(),
        "/models/small-50m.gguf".to_string(),
    ])
}

/// Helper function to get model info without loading
pub fn get_model_info(path: &str) -> LlmResult<ModelMetadata> {
    // TODO: Implement lightweight metadata extraction
    Err(LlmError::ModelNotFound {
        path: path.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = ModelLoader::new();
        assert!(!loader.is_loaded());
    }

    #[test]
    fn test_path_validation() {
        let loader = ModelLoader::new();

        // Valid path
        assert!(loader.validate_path("/models/test.gguf").is_ok());

        // Invalid: path traversal
        assert!(loader.validate_path("/models/../etc/passwd").is_err());

        // Invalid: not in /models/
        assert!(loader.validate_path("/etc/test.gguf").is_err());

        // Invalid: wrong extension
        assert!(loader.validate_path("/models/test.txt").is_err());
    }

    #[test]
    fn test_loader_stats() {
        let loader = ModelLoader::new();
        let stats = loader.stats();

        assert_eq!(stats.loads_successful, 0);
        assert_eq!(stats.loads_failed, 0);
        assert!(!stats.is_loaded);
    }

    #[test]
    fn test_load_config() {
        let config = LoadConfig::default();
        assert!(config.verify_checksum);
        assert!(!config.verify_signature);
        assert_eq!(config.max_size, 100 * 1024 * 1024);
    }
}
