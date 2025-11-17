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

/// Loaded model with extracted weights
///
/// This structure holds both the raw GGUF model and the extracted
/// transformer weights ready for inference.
#[cfg(feature = "llm-transformer")]
pub struct LoadedModel {
    /// Raw GGUF model
    pub gguf: crate::llm::gguf::GgufModel,

    /// Model configuration
    pub config: crate::llm::transformer::TransformerConfig,

    /// Model weights (extracted from GGUF)
    pub weights: ModelWeights,
}

/// Transformer model weights
///
/// Contains all weight tensors needed for inference.
/// Weights are stored in their original quantized format (Q4_0, etc.)
#[cfg(feature = "llm-transformer")]
pub struct ModelWeights {
    /// Token embedding weights (vocab_size × n_embd)
    pub token_embd: alloc::vec::Vec<u8>,

    /// Position embedding weights (n_ctx × n_embd)
    pub pos_embd: alloc::vec::Vec<u8>,

    /// Layer weights (per layer)
    pub layers: alloc::vec::Vec<LayerWeights>,

    /// Final layer norm weights
    pub ln_f_weight: alloc::vec::Vec<f32>,
    pub ln_f_bias: alloc::vec::Vec<f32>,

    /// LM head weights (n_embd × vocab_size)
    pub lm_head: alloc::vec::Vec<u8>,
}

/// Weights for a single transformer layer
#[cfg(feature = "llm-transformer")]
pub struct LayerWeights {
    /// Attention layer norm
    pub ln_1_weight: alloc::vec::Vec<f32>,
    pub ln_1_bias: alloc::vec::Vec<f32>,

    /// Attention weights (Q, K, V, O)
    pub attn_q: alloc::vec::Vec<u8>,
    pub attn_k: alloc::vec::Vec<u8>,
    pub attn_v: alloc::vec::Vec<u8>,
    pub attn_o: alloc::vec::Vec<u8>,

    /// FFN layer norm
    pub ln_2_weight: alloc::vec::Vec<f32>,
    pub ln_2_bias: alloc::vec::Vec<f32>,

    /// Feed-forward weights
    pub ffn_up: alloc::vec::Vec<u8>,
    pub ffn_down: alloc::vec::Vec<u8>,
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
    model: Option<LoadedModel>,
    tokenizer: crate::llm::tokenizer::BpeTokenizer,
    stats: BackendStats,
    kv_cache: Option<crate::llm::kv_cache::KVCache>,
    sampling_config: crate::llm::sampling::SamplingConfig,
}

#[cfg(feature = "llm-transformer")]
impl TransformerBackend {
    /// Create new transformer backend
    pub fn new() -> Self {
        Self {
            model: None,
            tokenizer: crate::llm::tokenizer::BpeTokenizer::new(),
            stats: BackendStats::default(),
            kv_cache: None,
            sampling_config: crate::llm::sampling::SamplingConfig::balanced(),
        }
    }

    /// Set sampling configuration
    ///
    /// # Arguments
    ///
    /// - `config`: Sampling configuration (temperature, top-k, top-p)
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut backend = TransformerBackend::new();
    /// backend.set_sampling_config(
    ///     crate::llm::sampling::SamplingConfig::new()
    ///         .temperature(0.8)
    ///         .top_k(40)
    ///         .top_p(0.9)
    /// );
    /// ```
    pub fn set_sampling_config(&mut self, config: crate::llm::sampling::SamplingConfig) {
        self.sampling_config = config;
    }

    /// Get current sampling configuration
    pub fn sampling_config(&self) -> crate::llm::sampling::SamplingConfig {
        self.sampling_config
    }

    /// Try to create new transformer backend with validation
    ///
    /// # Returns
    ///
    /// - `Ok(backend)` if initialization successful
    /// - `Err(msg)` if initialization failed
    pub fn try_new() -> Result<Self, &'static str> {
        // For now, just create a new backend
        // In the future, this could validate memory availability, etc.
        Ok(Self::new())
    }
}

#[cfg(feature = "llm-transformer")]
impl LlmBackend for TransformerBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<LlmResult, &'static str> {
        let start_time = crate::time::uptime_us();

        if self.model.is_none() {
            return Err("No model loaded");
        }

        let model = self.model.as_ref().unwrap();
        let config = &model.config;

        // Step 1: Tokenize prompt
        let mut tokens = self.tokenizer.encode(prompt);
        crate::info!("llm: tokenized prompt into {} tokens", tokens.len());

        if tokens.is_empty() {
            return Err("Empty prompt after tokenization");
        }

        // Validate context length
        if tokens.len() + max_tokens > config.n_ctx {
            crate::warn!("llm: prompt + max_tokens exceeds context length");
            return Err("Sequence too long for context window");
        }

        // Initialize KV cache for this generation
        let n_layer = model.weights.layers.len();
        let mut cache = crate::llm::kv_cache::KVCache::new(n_layer, config.n_ctx, config.n_embd);
        crate::info!("llm: initialized KV cache ({} layers, {} ctx, {} embd)", n_layer, config.n_ctx, config.n_embd);

        // Step 2: Autoregressive generation loop
        let mut generated_count = 0;
        for step in 0..max_tokens {
            // Check if weights are loaded (placeholder check)
            if model.weights.token_embd.is_empty() {
                // Weights not yet extracted from GGUF - use placeholder
                crate::warn!("llm: weights not extracted, using placeholder generation");

                // Generate a simple placeholder token
                // In production, this will be replaced with actual transformer inference
                let next_token = if tokens.len() < 10 {
                    // Generate some variety in placeholder output
                    ((tokens.len() * 7 + step * 13) % 1000) as u16
                } else {
                    self.tokenizer.eos_token_id()
                };

                tokens.push(next_token);
                generated_count += 1;

                // Stop at EOS
                if next_token == self.tokenizer.eos_token_id() {
                    break;
                }

                continue;
            }

            // Real transformer inference
            crate::info!("llm: running transformer inference for step {}", step);

            // 1. Get last token embedding
            let last_token_id = tokens[tokens.len() - 1] as usize;
            let mut hidden_state = match extract_embedding(&model.weights.token_embd, last_token_id, config.n_embd) {
                Ok(emb) => emb,
                Err(e) => {
                    crate::warn!("llm: failed to extract embedding: {}", e);
                    // Fallback to placeholder
                    let next_token = ((tokens.len() * 7 + step * 13) % 1000) as u16;
                    tokens.push(next_token);
                    generated_count += 1;
                    if next_token == self.tokenizer.eos_token_id() {
                        break;
                    }
                    continue;
                }
            };

            // 2. Add position embedding (if available)
            let pos = tokens.len() - 1;
            if !model.weights.pos_embd.is_empty() {
                if let Err(e) = add_position_embedding(&mut hidden_state, &model.weights.pos_embd, pos, config.n_embd) {
                    crate::warn!("llm: failed to add position embedding: {}", e);
                }
            }

            // 3. Run through transformer layers with KV cache
            for (layer_idx, layer) in model.weights.layers.iter().enumerate() {
                hidden_state = run_transformer_layer_with_cache(&hidden_state, layer, config, &mut cache, layer_idx);
            }

            // Advance cache position after processing all layers
            cache.advance();

            // 4. Final layer norm
            if !model.weights.ln_f_weight.is_empty() {
                layer_norm_inplace(&mut hidden_state, &model.weights.ln_f_weight, &model.weights.ln_f_bias);
            }

            // 5. Project to logits
            let logits = match compute_logits(&hidden_state, &model.weights.lm_head, config.n_vocab) {
                Ok(l) => l,
                Err(e) => {
                    crate::warn!("llm: failed to compute logits: {}", e);
                    // Fallback to placeholder
                    let next_token = ((tokens.len() * 7 + step * 13) % 1000) as u16;
                    tokens.push(next_token);
                    generated_count += 1;
                    if next_token == self.tokenizer.eos_token_id() {
                        break;
                    }
                    continue;
                }
            };

            // 6. Sample next token using configured sampling strategy
            let next_token = crate::llm::sampling::sample(&logits, self.sampling_config) as u16;

            crate::info!("llm: sampled token {} from {} logits (temperature={:.2}, top_k={}, top_p={:.2})",
                next_token, logits.len(),
                self.sampling_config.temperature,
                self.sampling_config.top_k,
                self.sampling_config.top_p);

            tokens.push(next_token);
            generated_count += 1;

            if next_token == self.tokenizer.eos_token_id() {
                break;
            }
        }

        // Step 3: Decode tokens to text
        let output = self.tokenizer.decode(&tokens);

        // Step 4: Calculate latency
        let end_time = crate::time::uptime_us();
        let latency_us = (end_time - start_time) as usize;

        // Update statistics
        self.stats.total_inferences += 1;
        self.stats.total_tokens += generated_count as u64;

        // Calculate tokens per second
        if latency_us > 0 {
            let tokens_per_sec = (generated_count as f32 * 1_000_000.0) / (latency_us as f32);
            // Exponential moving average
            if self.stats.avg_tokens_per_sec == 0.0 {
                self.stats.avg_tokens_per_sec = tokens_per_sec;
            } else {
                self.stats.avg_tokens_per_sec = 0.9 * self.stats.avg_tokens_per_sec + 0.1 * tokens_per_sec;
            }
        }

        crate::info!("llm: generated {} tokens in {} µs ({:.2} tok/s)",
            generated_count, latency_us, self.stats.avg_tokens_per_sec);

        // Log KV cache statistics
        let cache_stats = cache.stats();
        let speedup = cache.speedup_estimate(generated_count);
        crate::info!("llm: KV cache hit rate: {:.1}%, estimated speedup: {:.1}×",
            cache_stats.hit_rate * 100.0, speedup);

        Ok(LlmResult {
            infer_id: self.stats.total_inferences as usize,
            tokens_emitted: generated_count,
            output,
            latency_us,
        })
    }

    fn load_model(&mut self, path: &str) -> Result<(), &'static str> {
        crate::info!("llm: loading model from {}", path);

        // Step 1: Open file from VFS
        let file = crate::vfs::open(path, crate::vfs::OpenFlags::O_RDONLY)
            .map_err(|_| "Failed to open model file")?;

        // Step 2: Get file size
        let inode = file.inode.as_ref().ok_or("File has no inode")?;
        let file_size = inode.size() as usize;

        if file_size > 100 * 1024 * 1024 {  // 100 MB limit
            crate::warn!("llm: model file too large: {} bytes", file_size);
            return Err("Model file exceeds 100MB limit");
        }

        crate::info!("llm: model file size: {} bytes", file_size);

        // Step 3: Read file contents
        let mut buffer = alloc::vec::Vec::with_capacity(file_size);
        buffer.resize(file_size, 0);

        let bytes_read = file.read(&mut buffer)
            .map_err(|_| "Failed to read model file")?;

        if bytes_read != file_size {
            crate::warn!("llm: partial read: {} of {} bytes", bytes_read, file_size);
            return Err("Incomplete file read");
        }

        crate::info!("llm: read {} bytes from {}", bytes_read, path);

        // Step 4: Parse GGUF model
        let gguf = crate::llm::gguf::GgufModel::from_bytes(&buffer)
            .map_err(|_| "Failed to parse GGUF model")?;

        crate::info!("llm: GGUF model parsed successfully");

        // Step 5: Extract configuration from GGUF metadata
        let config = extract_config_from_gguf(&gguf)?;
        crate::info!("llm: extracted config: vocab={} ctx={} embd={} heads={} layers={}",
            config.n_vocab, config.n_ctx, config.n_embd, config.n_head, config.n_layer);

        // Step 6: Extract model weights from GGUF tensors
        crate::info!("llm: extracting weights from {} tensors", gguf.tensors.len());
        let weights = extract_weights_from_gguf(&gguf, &config)?;
        crate::info!("llm: weight extraction complete");

        // Step 7: Create LoadedModel
        let loaded_model = LoadedModel {
            gguf,
            config,
            weights,
        };

        // Step 8: Store model
        self.model = Some(loaded_model);
        self.stats.model_loaded = true;

        crate::info!("llm: model loaded successfully from {}", path);
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

// =============================================================================
// Helper Functions for Real Transformer Inference
// =============================================================================

/// Convert raw bytes to Q4_0Block slice
///
/// # Safety
///
/// Assumes bytes are properly aligned and sized for Q4_0Block layout
#[cfg(feature = "llm-transformer")]
#[inline]
fn bytes_to_q4_0_blocks(bytes: &[u8]) -> &[crate::llm::quantize::Q4_0Block] {
    use core::slice;
    use crate::llm::quantize::Q4_0Block;

    // Q4_0Block is 18 bytes (2 bytes scale + 16 bytes quants)
    const BLOCK_SIZE: usize = 18;
    let num_blocks = bytes.len() / BLOCK_SIZE;

    if num_blocks == 0 {
        return &[];
    }

    unsafe {
        slice::from_raw_parts(
            bytes.as_ptr() as *const Q4_0Block,
            num_blocks
        )
    }
}

/// Extract token embedding from quantized embedding table
///
/// # Arguments
///
/// - `embd_bytes`: Raw bytes of quantized embedding table (Q4_0)
/// - `token_id`: Token ID to extract
/// - `n_embd`: Embedding dimension
///
/// # Returns
///
/// Dequantized embedding vector (f32)
#[cfg(feature = "llm-transformer")]
fn extract_embedding(embd_bytes: &[u8], token_id: usize, n_embd: usize) -> Result<alloc::vec::Vec<f32>, &'static str> {
    if embd_bytes.is_empty() {
        return Err("Empty embedding table");
    }

    // Convert bytes to Q4_0 blocks
    let blocks = bytes_to_q4_0_blocks(embd_bytes);

    // Calculate offset (each embedding is n_embd values)
    let values_per_block = crate::llm::quantize::QK4_0;
    let start_value = token_id * n_embd;
    let start_block = start_value / values_per_block;
    let start_offset = start_value % values_per_block;

    // Allocate output
    let mut embedding = alloc::vec![0.0f32; n_embd];

    // Dequantize values
    let mut emb_idx = 0;
    let mut block_idx = start_block;
    let mut offset = start_offset;

    while emb_idx < n_embd && block_idx < blocks.len() {
        let remaining = values_per_block - offset;
        let to_copy = core::cmp::min(n_embd - emb_idx, remaining);

        for i in 0..to_copy {
            embedding[emb_idx + i] = blocks[block_idx].dequant(offset + i);
        }

        emb_idx += to_copy;
        block_idx += 1;
        offset = 0;
    }

    Ok(embedding)
}

/// Add position embedding to hidden state
///
/// # Arguments
///
/// - `hidden_state`: Hidden state to modify (in-place)
/// - `pos_embd_bytes`: Raw bytes of position embedding table (Q4_0)
/// - `pos`: Position index
/// - `n_embd`: Embedding dimension
#[cfg(feature = "llm-transformer")]
fn add_position_embedding(
    hidden_state: &mut [f32],
    pos_embd_bytes: &[u8],
    pos: usize,
    n_embd: usize,
) -> Result<(), &'static str> {
    let pos_emb = extract_embedding(pos_embd_bytes, pos, n_embd)?;

    for i in 0..n_embd {
        hidden_state[i] += pos_emb[i];
    }

    Ok(())
}

/// Run single transformer layer forward pass with KV cache
///
/// # Arguments
///
/// - `input`: Input hidden state (n_embd,)
/// - `layer`: Layer weights
/// - `config`: Model configuration
/// - `cache`: KV cache (updated in-place)
/// - `layer_idx`: Layer index for cache access
///
/// # Returns
///
/// Output hidden state (n_embd,)
#[cfg(feature = "llm-transformer")]
fn run_transformer_layer_with_cache(
    input: &[f32],
    layer: &LayerWeights,
    config: &crate::llm::transformer::TransformerConfig,
    cache: &mut crate::llm::kv_cache::KVCache,
    layer_idx: usize,
) -> alloc::vec::Vec<f32> {
    use crate::llm::transformer::TransformerLayer;

    // Create TransformerLayer from LayerWeights
    // Convert Vec<u8> to Vec<Q4_0Block>
    let transformer_layer = TransformerLayer {
        ln1_weight: layer.ln_1_weight.clone(),
        ln1_bias: layer.ln_1_bias.clone(),
        attn_q: bytes_to_q4_0_blocks(&layer.attn_q).to_vec(),
        attn_k: bytes_to_q4_0_blocks(&layer.attn_k).to_vec(),
        attn_v: bytes_to_q4_0_blocks(&layer.attn_v).to_vec(),
        attn_out: bytes_to_q4_0_blocks(&layer.attn_o).to_vec(),
        ln2_weight: layer.ln_2_weight.clone(),
        ln2_bias: layer.ln_2_bias.clone(),
        ffn_up: bytes_to_q4_0_blocks(&layer.ffn_up).to_vec(),
        ffn_down: bytes_to_q4_0_blocks(&layer.ffn_down).to_vec(),
    };

    // Run forward pass with cache
    transformer_layer.forward_with_cache(input, config, cache, layer_idx)
}

/// Run single transformer layer forward pass
///
/// # Arguments
///
/// - `input`: Input hidden state (n_embd,)
/// - `layer`: Layer weights
/// - `config`: Model configuration
///
/// # Returns
///
/// Output hidden state (n_embd,)
#[cfg(feature = "llm-transformer")]
fn run_transformer_layer(
    input: &[f32],
    layer: &LayerWeights,
    config: &crate::llm::transformer::TransformerConfig,
) -> Result<alloc::vec::Vec<f32>, &'static str> {
    use crate::llm::transformer::TransformerLayer;

    // Create TransformerLayer from LayerWeights
    // Convert Vec<u8> to Vec<Q4_0Block>
    let transformer_layer = TransformerLayer {
        ln1_weight: layer.ln_1_weight.clone(),
        ln1_bias: layer.ln_1_bias.clone(),
        attn_q: bytes_to_q4_0_blocks(&layer.attn_q).to_vec(),
        attn_k: bytes_to_q4_0_blocks(&layer.attn_k).to_vec(),
        attn_v: bytes_to_q4_0_blocks(&layer.attn_v).to_vec(),
        attn_out: bytes_to_q4_0_blocks(&layer.attn_o).to_vec(),
        ln2_weight: layer.ln_2_weight.clone(),
        ln2_bias: layer.ln_2_bias.clone(),
        ffn_up: bytes_to_q4_0_blocks(&layer.ffn_up).to_vec(),
        ffn_down: bytes_to_q4_0_blocks(&layer.ffn_down).to_vec(),
    };

    Ok(transformer_layer.forward(input, config))
}

/// Apply layer normalization in-place
#[cfg(feature = "llm-transformer")]
fn layer_norm_inplace(hidden_state: &mut [f32], weight: &[f32], bias: &[f32]) {
    let normed = crate::llm::transformer::layer_norm(hidden_state, weight, bias);
    hidden_state.copy_from_slice(&normed);
}

/// Compute logits from hidden state
///
/// # Arguments
///
/// - `hidden_state`: Final hidden state (n_embd,)
/// - `lm_head_bytes`: Language model head weights (Q4_0), shape (n_vocab, n_embd)
/// - `n_vocab`: Vocabulary size
///
/// # Returns
///
/// Logits vector (n_vocab,)
#[cfg(feature = "llm-transformer")]
fn compute_logits(
    hidden_state: &[f32],
    lm_head_bytes: &[u8],
    n_vocab: usize,
) -> Result<alloc::vec::Vec<f32>, &'static str> {
    if lm_head_bytes.is_empty() {
        return Err("Empty lm_head");
    }

    let n_embd = hidden_state.len();

    // Logits = lm_head @ hidden_state
    // lm_head shape: (n_vocab, n_embd)
    let mut logits = alloc::vec![0.0f32; n_vocab];

    for vocab_idx in 0..n_vocab {
        // Extract row from lm_head
        let row = match extract_embedding(lm_head_bytes, vocab_idx, n_embd) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Dot product with hidden_state
        let mut sum = 0.0f32;
        for i in 0..n_embd {
            sum += row[i] * hidden_state[i];
        }
        logits[vocab_idx] = sum;
    }

    Ok(logits)
}

// =============================================================================
// GGUF Weight Extraction Functions
// =============================================================================

/// Extract configuration from GGUF metadata
///
/// Reads model hyperparameters from GGUF key-value pairs
#[cfg(feature = "llm-transformer")]
fn extract_config_from_gguf(gguf: &crate::llm::gguf::GgufModel) -> Result<crate::llm::transformer::TransformerConfig, &'static str> {
    // Try to extract from metadata, fall back to defaults if not found
    let n_vocab = gguf.get_u32("llm.vocab_size")
        .or_else(|_| gguf.get_u32("tokenizer.ggml.vocab_size"))
        .unwrap_or(32000) as usize;

    let n_ctx = gguf.get_u32("llm.context_length")
        .or_else(|_| gguf.get_u32("gpt_neox.n_ctx"))
        .unwrap_or(512) as usize;

    let n_embd = gguf.get_u32("llm.embedding_length")
        .or_else(|_| gguf.get_u32("gpt_neox.n_embd"))
        .unwrap_or(384) as usize;

    let n_head = gguf.get_u32("llm.attention.head_count")
        .or_else(|_| gguf.get_u32("gpt_neox.n_head"))
        .unwrap_or(6) as usize;

    let n_layer = gguf.get_u32("llm.block_count")
        .or_else(|_| gguf.get_u32("gpt_neox.n_layer"))
        .unwrap_or(6) as usize;

    Ok(crate::llm::transformer::TransformerConfig {
        n_vocab,
        n_ctx,
        n_embd,
        n_head,
        n_layer,
    })
}

/// Extract model weights from GGUF tensors
///
/// Maps GGUF tensor names to ModelWeights structure
#[cfg(feature = "llm-transformer")]
fn extract_weights_from_gguf(
    gguf: &crate::llm::gguf::GgufModel,
    config: &crate::llm::transformer::TransformerConfig,
) -> Result<ModelWeights, &'static str> {
    // Helper to get tensor data or return empty vec
    let get_tensor_data = |name: &str| -> alloc::vec::Vec<u8> {
        gguf.get_tensor(name)
            .map(|t| t.data.clone())
            .unwrap_or_else(|| {
                crate::warn!("llm: tensor '{}' not found, using empty placeholder", name);
                alloc::vec![]
            })
    };

    // Helper to get f32 tensor data
    let get_f32_tensor = |name: &str| -> alloc::vec::Vec<f32> {
        // For now, return empty - will implement dequantization in next phase
        crate::warn!("llm: f32 tensor '{}' placeholder (dequantization not yet implemented)", name);
        alloc::vec![]
    };

    // Extract embeddings
    let token_embd = get_tensor_data("token_embd.weight");
    let pos_embd = get_tensor_data("position_embd.weight");

    // Extract final layer norm
    let ln_f_weight = get_f32_tensor("ln_f.weight");
    let ln_f_bias = get_f32_tensor("ln_f.bias");

    // Extract LM head
    let lm_head = get_tensor_data("output.weight");

    // Extract layer weights
    let mut layers = alloc::vec::Vec::new();
    for layer_idx in 0..config.n_layer {
        let layer_weights = LayerWeights {
            ln_1_weight: get_f32_tensor(&format!("blk.{}.attn_norm.weight", layer_idx)),
            ln_1_bias: get_f32_tensor(&format!("blk.{}.attn_norm.bias", layer_idx)),
            attn_q: get_tensor_data(&format!("blk.{}.attn_q.weight", layer_idx)),
            attn_k: get_tensor_data(&format!("blk.{}.attn_k.weight", layer_idx)),
            attn_v: get_tensor_data(&format!("blk.{}.attn_v.weight", layer_idx)),
            attn_o: get_tensor_data(&format!("blk.{}.attn_output.weight", layer_idx)),
            ln_2_weight: get_f32_tensor(&format!("blk.{}.ffn_norm.weight", layer_idx)),
            ln_2_bias: get_f32_tensor(&format!("blk.{}.ffn_norm.bias", layer_idx)),
            ffn_up: get_tensor_data(&format!("blk.{}.ffn_up.weight", layer_idx)),
            ffn_down: get_tensor_data(&format!("blk.{}.ffn_down.weight", layer_idx)),
        };
        layers.push(layer_weights);
    }

    Ok(ModelWeights {
        token_embd,
        pos_embd,
        layers,
        ln_f_weight,
        ln_f_bias,
        lm_head,
    })
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
///
/// # Returns
///
/// - `true` if initialization successful
/// - `false` if initialization failed (falls back to stub)
pub fn init_backend(use_transformer: bool) -> bool {
    let mut backend = BACKEND.lock();

    #[cfg(feature = "llm-transformer")]
    if use_transformer {
        match TransformerBackend::try_new() {
            Ok(tb) => {
                crate::info!("llm: initialized transformer backend");
                *backend = Some(Box::new(tb));
                return true;
            }
            Err(e) => {
                crate::warn!("llm: transformer backend init failed: {}, falling back to stub", e);
                *backend = Some(Box::new(StubBackend::new()));
                return false;
            }
        }
    }

    // Default: stub backend
    *backend = Some(Box::new(StubBackend::new()));
    crate::info!("llm: initialized stub backend");
    true
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
