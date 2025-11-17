# VFS Integration for LLM Model Loading

## Overview

The LLM subsystem integrates with the Virtual File System (VFS) to load GGUF model files from persistent storage. This document describes the model loading architecture, security considerations, and usage patterns.

## Architecture

### Model Loader

The `ModelLoader` provides a high-level interface for loading and managing GGUF models:

```rust
use crate::llm::loader::{ModelLoader, LoadConfig, ModelMetadata};

let mut loader = ModelLoader::new();
let config = LoadConfig {
    verify_checksum: true,
    max_model_size: 100 * 1024 * 1024,  // 100 MB
    allow_quantized: true,
    trusted_paths: vec!["/models/".to_string()],
};

loader.load("/models/tinyllama-1.1b-q4_0.gguf", config)?;
```

### Components

#### 1. ModelLoader

**File**: `crates/kernel/src/llm/loader.rs`

**Purpose**: High-level model loading with security validation

**Key Methods**:
- `load(path, config)` - Load model from VFS path
- `unload()` - Unload current model
- `is_loaded()` - Check if model is loaded
- `metadata()` - Get current model metadata
- `get_model()` - Get reference to loaded GGUF model

**Features**:
- Path validation and security checks
- Checksum verification (SHA-256)
- Size limit enforcement
- Quantization format validation
- Metadata extraction

#### 2. LoadConfig

**Purpose**: Configuration for model loading behavior

**Fields**:
```rust
pub struct LoadConfig {
    /// Verify file checksum after loading
    pub verify_checksum: bool,

    /// Maximum allowed model size (bytes)
    pub max_model_size: usize,

    /// Allow loading quantized models
    pub allow_quantized: bool,

    /// Trusted path prefixes
    pub trusted_paths: Vec<String>,
}
```

**Default Configuration**:
- `verify_checksum: true` - Always verify integrity
- `max_model_size: 100 MB` - Reasonable for embedded systems
- `allow_quantized: true` - Enable Q4_0/Q8_0 models
- `trusted_paths: ["/models/"]` - Only load from models directory

#### 3. ModelMetadata

**Purpose**: Extracted model information

**Fields**:
```rust
pub struct ModelMetadata {
    /// Model architecture (e.g., "llama", "gpt2")
    pub architecture: String,

    /// Model name
    pub name: String,

    /// Context length
    pub context_length: usize,

    /// Embedding dimension
    pub embedding_dim: usize,

    /// Number of layers
    pub num_layers: usize,

    /// Number of attention heads
    pub num_heads: usize,

    /// Vocabulary size
    pub vocab_size: usize,

    /// File size in bytes
    pub file_size: usize,

    /// File checksum (SHA-256)
    pub checksum: Option<String>,

    /// Quantization format
    pub quantization: Option<String>,
}
```

## VFS Integration

### File System Access

The model loader integrates with the VFS through the following pattern:

```rust
// In production, this would use actual VFS calls:
// use crate::fs::vfs;
// let file = vfs::open(path)?;
// let data = file.read_all()?;

// For now, stub implementation:
fn load_from_vfs(path: &str) -> Result<Vec<u8>, LlmError> {
    // TODO: Replace with actual VFS integration
    // vfs::open(path)?.read_all()

    // Stub: return empty data or error
    Err(LlmError::ModelNotFound {
        path: path.to_string()
    })
}
```

### Security Considerations

#### 1. Path Validation

All paths are validated before access:

```rust
fn validate_path(&self, path: &str, config: &LoadConfig) -> LlmResult<()> {
    // Check path traversal
    if path.contains("..") {
        return Err(LlmError::InvalidConfig {
            field: "path".to_string(),
            reason: "Path traversal not allowed".to_string(),
        });
    }

    // Check trusted paths
    let is_trusted = config.trusted_paths.iter()
        .any(|prefix| path.starts_with(prefix));

    if !is_trusted {
        return Err(LlmError::InvalidConfig {
            field: "path".to_string(),
            reason: format!("Path not in trusted locations: {:?}",
                          config.trusted_paths),
        });
    }

    Ok(())
}
```

**Protections**:
- Path traversal prevention (`..` blocked)
- Whitelist-based trusted paths
- Absolute path validation

#### 2. Size Limits

Models are size-checked before loading:

```rust
if file_size > config.max_model_size {
    return Err(LlmError::ModelTooLarge {
        size: file_size,
        available: config.max_model_size,
    });
}
```

**Purpose**: Prevent memory exhaustion attacks

#### 3. Checksum Verification

Optional SHA-256 checksum verification:

```rust
if config.verify_checksum {
    let computed = compute_checksum(&data);
    if computed != expected_checksum {
        return Err(LlmError::InvalidModelFormat {
            path: path.to_string(),
            reason: "Checksum mismatch".to_string(),
        });
    }
}
```

**Purpose**: Detect corrupted or tampered files

#### 4. Format Validation

GGUF magic number and version checked:

```rust
fn validate_gguf_format(data: &[u8]) -> LlmResult<()> {
    if data.len() < 4 {
        return Err(LlmError::InvalidModelFormat {
            path: "unknown".to_string(),
            reason: "File too small".to_string(),
        });
    }

    let magic = &data[0..4];
    if magic != b"GGUF" {
        return Err(LlmError::InvalidModelFormat {
            path: "unknown".to_string(),
            reason: format!("Invalid magic: {:?}", magic),
        });
    }

    Ok(())
}
```

## Usage Examples

### Basic Model Loading

```rust
use crate::llm::loader::{ModelLoader, LoadConfig};

// Create loader
let mut loader = ModelLoader::new();

// Configure loading
let config = LoadConfig::default();

// Load model
loader.load("/models/tinyllama-1.1b-q4_0.gguf", config)?;

// Check metadata
let metadata = loader.metadata().expect("No metadata");
println!("Loaded model: {}", metadata.name);
println!("Context length: {}", metadata.context_length);
println!("Quantization: {:?}", metadata.quantization);

// Get model for inference
let model = loader.get_model().expect("No model");
// ... use model for inference
```

### Custom Configuration

```rust
use crate::llm::loader::{ModelLoader, LoadConfig};

// Strict security configuration
let config = LoadConfig {
    verify_checksum: true,             // Always verify
    max_model_size: 50 * 1024 * 1024,  // 50 MB max
    allow_quantized: true,             // Only Q4/Q8
    trusted_paths: vec![
        "/models/approved/".to_string(),
    ],
};

let mut loader = ModelLoader::new();
loader.load("/models/approved/model.gguf", config)?;
```

### Handling Load Errors

```rust
use crate::llm::loader::{ModelLoader, LoadConfig};
use crate::llm::errors::LlmError;

let mut loader = ModelLoader::new();
let config = LoadConfig::default();

match loader.load("/models/model.gguf", config) {
    Ok(()) => {
        println!("Model loaded successfully");
    }
    Err(LlmError::ModelNotFound { path }) => {
        eprintln!("Model not found: {}", path);
    }
    Err(LlmError::ModelTooLarge { size, available }) => {
        eprintln!("Model too large: {} bytes (max: {})", size, available);
    }
    Err(LlmError::InvalidModelFormat { path, reason }) => {
        eprintln!("Invalid model format '{}': {}", path, reason);
    }
    Err(e) => {
        eprintln!("Load error: {}", e);
    }
}
```

### Model Metadata Inspection

```rust
use crate::llm::loader::ModelLoader;

let mut loader = ModelLoader::new();
loader.load("/models/model.gguf", LoadConfig::default())?;

if let Some(meta) = loader.metadata() {
    println!("=== Model Information ===");
    println!("Name: {}", meta.name);
    println!("Architecture: {}", meta.architecture);
    println!("Context Length: {}", meta.context_length);
    println!("Embedding Dim: {}", meta.embedding_dim);
    println!("Layers: {}", meta.num_layers);
    println!("Heads: {}", meta.num_heads);
    println!("Vocab Size: {}", meta.vocab_size);
    println!("File Size: {} MB", meta.file_size / 1024 / 1024);

    if let Some(quant) = &meta.quantization {
        println!("Quantization: {}", quant);
    }

    if let Some(checksum) = &meta.checksum {
        println!("Checksum: {}", checksum);
    }
}
```

## Performance Considerations

### Memory Usage

**Loading Phase**:
- File buffer: Size of model file (e.g., 50-100 MB for quantized models)
- GGUF parsing: ~1 MB temporary structures
- Metadata: ~1 KB

**Post-Loading**:
- Model data retained in memory
- Metadata cache: ~1 KB

**Optimization**: Use arena allocator for temporary structures during parsing

### Loading Time

**Expected Times** (for 100 MB model):
- VFS read: ~100-500 ms (depends on storage)
- GGUF parse: ~50-100 ms
- Checksum verify: ~100-200 ms (if enabled)
- **Total**: ~250-800 ms

**Optimization**: Disable checksum verification for trusted models

## Integration with Backend

The model loader integrates with the LLM backend:

```rust
use crate::llm::backend::{init_backend, get_backend};
use crate::llm::loader::{ModelLoader, LoadConfig};

// Initialize backend
init_backend(true);  // Use real transformer

// Load model
let mut loader = ModelLoader::new();
loader.load("/models/model.gguf", LoadConfig::default())?;

// Pass to backend
let mut backend = get_backend();
let backend = backend.as_mut().expect("Backend not initialized");

// Backend uses loaded model for inference
let result = backend.infer("Hello, world!", 50)?;
```

## Future Enhancements

### Planned Features

1. **Streaming Loading**: Load model incrementally to reduce memory spikes
2. **Model Caching**: Keep frequently-used models in memory
3. **Lazy Tensor Loading**: Load tensors on-demand during inference
4. **Compressed Models**: Support gzip/zstd compressed GGUF files
5. **Remote Loading**: HTTP/HTTPS model downloads
6. **Model Hot-Swapping**: Switch models without backend restart

### VFS Integration Roadmap

**Phase 1** (Current): Stub implementation with security framework
**Phase 2**: Integration with actual VFS layer
**Phase 3**: Advanced features (streaming, caching, compression)

## Testing

See `TESTING_GUIDE.md` for comprehensive testing documentation.

**Key Tests**:
- Path validation security
- Size limit enforcement
- Checksum verification
- Format validation
- Metadata extraction
- Error handling

## Security Checklist

Before deploying in production:

- [ ] Enable checksum verification
- [ ] Configure appropriate size limits
- [ ] Whitelist trusted model paths
- [ ] Review path validation logic
- [ ] Test with malformed GGUF files
- [ ] Verify error handling coverage
- [ ] Audit VFS integration code

## References

- GGUF Format: `docs/llm/GGUF_FORMAT.md`
- Error Handling: `crates/kernel/src/llm/errors.rs`
- Backend Integration: `docs/llm/ARCHITECTURE.md`
- Testing Guide: `docs/llm/TESTING_GUIDE.md`
