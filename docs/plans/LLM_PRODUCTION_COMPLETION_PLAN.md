# LLM Production Completion Plan

**Status:** 🚧 Ready to Execute
**Goal:** Enable real transformer inference in QEMU
**Timeline:** 2 weeks (80 hours total)
**Current State:** Full transformer implementation complete, StubBackend active
**Target State:** Production-ready LLM inference with real model weights

---

## Executive Summary

The SIS kernel contains a **complete transformer inference engine** (10,910 lines across 16 modules) that currently runs in stub mode for testing. This plan outlines the precise steps to activate the real transformer backend and achieve production-ready LLM inference.

### What We Have ✅

- ✅ **Complete transformer implementation** (724 lines)
  - Multi-head self-attention
  - Feed-forward networks
  - Layer normalization
  - Residual connections

- ✅ **BPE tokenizer** (644 lines)
  - Vocabulary loading
  - Encode/decode functions
  - Special token handling

- ✅ **Q4_0 quantization** (584 lines)
  - 4-bit weights (8× memory reduction)
  - Fast dequantization
  - SIMD optimizations (ARM NEON)

- ✅ **GGUF model loader** (575 lines)
  - Format parsing
  - Metadata extraction
  - Tensor loading

- ✅ **KV cache** (456 lines)
  - Attention key/value caching
  - Context window management

- ✅ **Generation engine** (502 lines)
  - Autoregressive sampling
  - Temperature/top-k/top-p
  - Repetition penalty

### What We Need 🚧

| Component | Effort | Status | Blocker |
|-----------|--------|--------|---------|
| Backend switch | 4h | ⚠️ Code change | Feature flag |
| VFS integration | 8h | ⚠️ Wiring | File operations |
| Forward pass | 6h | ⚠️ Hookup | Transformer call |
| Timer benchmarks | 3h | ⚠️ Integration | ARM timer |
| Model testing | 8h | ⚠️ Validation | GGUF file |
| Performance tuning | 12h | ⚠️ Optimization | Profiling |

**Total:** 41 hours of focused development

---

## Week 1: Core Implementation (40 hours)

### Day 1-2: Backend Activation (16 hours)

#### Task 1.1: Add TransformerBackend Initialization

**File:** `crates/kernel/src/llm/backend.rs`
**Lines to modify:** 220-240
**Effort:** 4 hours

**Current Code:**
```rust
pub fn init_backend(use_real: bool) {
    let mut backend = BACKEND.lock();

    // Always use stub for testing
    *backend = Some(Box::new(StubBackend::new()));
}
```

**New Code:**
```rust
pub fn init_backend(use_real: bool) {
    let mut backend = BACKEND.lock();

    if use_real {
        // Initialize real transformer backend
        match TransformerBackend::new() {
            Ok(tb) => {
                crate::info!("LLM: Initialized TransformerBackend");
                *backend = Some(Box::new(tb));
            }
            Err(e) => {
                crate::warn!("LLM: TransformerBackend init failed: {}, using stub", e);
                *backend = Some(Box::new(StubBackend::new()));
            }
        }
    } else {
        crate::info!("LLM: Using StubBackend (testing mode)");
        *backend = Some(Box::new(StubBackend::new()));
    }
}
```

**Shell Integration:**

**File:** `crates/kernel/src/shell/commands/llm_commands.rs`
**New command:** `llmctl backend [stub|real]`

```rust
fn cmd_llmctl_backend(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: llmctl backend [stub|real]");
        return;
    }

    let backend_type = args[1];
    match backend_type {
        "stub" => {
            llm::init_backend(false);
            println!("LLM backend: StubBackend");
        }
        "real" => {
            llm::init_backend(true);
            println!("LLM backend: TransformerBackend");
        }
        _ => {
            println!("Error: Unknown backend type '{}'", backend_type);
            println!("Valid types: stub, real");
        }
    }
}
```

**Testing:**
```
sis> llmctl backend stub
LLM backend: StubBackend

sis> llmctl backend real
LLM backend: TransformerBackend

sis> llmctl info
Backend: TransformerBackend
Model: None (use 'llmctl load' to load model)
```

#### Task 1.2: Implement TransformerBackend Structure

**File:** `crates/kernel/src/llm/backend.rs`
**New structure:** Add to existing file (after StubBackend)
**Effort:** 8 hours

```rust
/// Production transformer backend
///
/// This backend performs actual transformer inference using loaded model weights.
pub struct TransformerBackend {
    /// Loaded model (None if not loaded)
    model: Option<LoadedModel>,

    /// Tokenizer instance
    tokenizer: BpeTokenizer,

    /// Generation configuration
    gen_config: GenerationConfig,

    /// Performance statistics
    stats: BackendStats,

    /// Arena allocator for temporary tensors
    arena: &'static Mutex<Arena>,
}

/// Loaded model container
struct LoadedModel {
    /// Model configuration (from GGUF metadata)
    config: TransformerConfig,

    /// Model weights (quantized)
    weights: ModelWeights,

    /// KV cache for attention
    kv_cache: KVCache,

    /// Model metadata
    metadata: ModelMetadata,
}

/// Model weights structure
struct ModelWeights {
    /// Token embeddings: [vocab_size, n_embd]
    token_embeddings: Vec<f32>,

    /// Layer weights (per-layer)
    layers: Vec<LayerWeights>,

    /// Final layer norm
    final_ln_weight: Vec<f32>,
    final_ln_bias: Vec<f32>,

    /// Output projection (language modeling head)
    output_weight: Vec<f32>,
}

/// Per-layer weights
struct LayerWeights {
    // Attention
    attn_q_weight: Vec<u8>,  // Quantized Q4_0
    attn_k_weight: Vec<u8>,
    attn_v_weight: Vec<u8>,
    attn_o_weight: Vec<u8>,

    // Layer norm 1
    ln1_weight: Vec<f32>,
    ln1_bias: Vec<f32>,

    // Feed-forward
    ffn_up_weight: Vec<u8>,
    ffn_down_weight: Vec<u8>,

    // Layer norm 2
    ln2_weight: Vec<f32>,
    ln2_bias: Vec<f32>,
}

impl TransformerBackend {
    /// Create new transformer backend
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            model: None,
            tokenizer: BpeTokenizer::new(),
            gen_config: GenerationConfig::default(),
            stats: BackendStats::new(),
            arena: arena(),
        })
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.model.is_some()
    }

    /// Get model info
    pub fn model_info(&self) -> Option<&ModelMetadata> {
        self.model.as_ref().map(|m| &m.metadata)
    }
}

impl LlmBackend for TransformerBackend {
    fn name(&self) -> &'static str {
        "TransformerBackend"
    }

    fn load_model(&mut self, path: &str) -> Result<(), &'static str> {
        // Implementation in Task 1.3
        todo!("Implement in Task 1.3")
    }

    fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<LlmResult, &'static str> {
        // Implementation in Task 1.4
        todo!("Implement in Task 1.4")
    }

    fn stats(&self) -> BackendStats {
        self.stats
    }
}
```

**Validation:**
```rust
#[test]
fn test_transformer_backend_creation() {
    let backend = TransformerBackend::new().expect("Failed to create backend");
    assert_eq!(backend.name(), "TransformerBackend");
    assert!(!backend.is_loaded());
}
```

#### Task 1.3: VFS Model Loading Integration

**File:** `crates/kernel/src/llm/loader.rs`
**Function to enhance:** `load_model_from_vfs()`
**Effort:** 4 hours

**Current Code:**
```rust
pub fn load_model(path: &str) -> Result<LoadedModel, &'static str> {
    // TODO: Integrate with actual VFS
    let model_data = include_bytes!("../../test_data/model.gguf");
    parse_gguf(model_data)
}
```

**New Code:**
```rust
use crate::vfs::{self, VfsError};

pub fn load_model_from_vfs(path: &str) -> Result<LoadedModel, &'static str> {
    crate::info!("LLM: Loading model from VFS: {}", path);

    // Open file from VFS
    let mut file = vfs::open(path)
        .map_err(|e| match e {
            VfsError::NotFound => "Model file not found",
            VfsError::AccessDenied => "Access denied",
            VfsError::InvalidPath => "Invalid path",
            _ => "VFS error",
        })?;

    // Get file size
    let stat = file.stat()
        .map_err(|_| "Failed to stat model file")?;
    let file_size = stat.size;

    crate::info!("LLM: Model file size: {} bytes ({:.1} MB)",
                 file_size, file_size as f64 / 1024.0 / 1024.0);

    // Validate size (sanity check: 1MB - 2GB)
    if file_size < 1024 * 1024 {
        return Err("Model file too small (< 1MB)");
    }
    if file_size > 2 * 1024 * 1024 * 1024 {
        return Err("Model file too large (> 2GB)");
    }

    // Allocate buffer from arena
    let mut arena_guard = arena().lock();
    let buffer_ptr = arena_guard.alloc(file_size, 16)
        .ok_or("Failed to allocate model buffer")?;
    drop(arena_guard);

    // Read file into buffer
    let buffer = unsafe {
        core::slice::from_raw_parts_mut(buffer_ptr, file_size)
    };

    let bytes_read = file.read(buffer)
        .map_err(|_| "Failed to read model file")?;

    if bytes_read != file_size {
        return Err("Incomplete model file read");
    }

    crate::info!("LLM: Model file read complete, parsing GGUF...");

    // Parse GGUF format
    parse_gguf_with_verification(buffer)
}

/// Parse GGUF with optional checksum verification
fn parse_gguf_with_verification(data: &[u8]) -> Result<LoadedModel, &'static str> {
    // Parse GGUF header
    let model = parse_gguf(data)?;

    // Optional: Verify checksum if crypto-real feature enabled
    #[cfg(feature = "crypto-real")]
    {
        if let Some(expected_hash) = model.metadata.sha256_hash.as_ref() {
            crate::info!("LLM: Verifying model checksum...");

            let actual_hash = crypto::sha256(data);
            if actual_hash != expected_hash {
                crate::warn!("LLM: Model checksum mismatch!");
                return Err("Model checksum verification failed");
            }

            crate::info!("LLM: Checksum verified ✓");
        }
    }

    Ok(model)
}
```

**Integration in TransformerBackend:**
```rust
impl LlmBackend for TransformerBackend {
    fn load_model(&mut self, path: &str) -> Result<(), &'static str> {
        // Validate path
        if !path.starts_with("/models/") {
            return Err("Model path must be in /models/ directory");
        }

        if !path.ends_with(".gguf") {
            return Err("Model file must have .gguf extension");
        }

        // Load from VFS
        let loaded = load_model_from_vfs(path)?;

        // Validate model architecture
        if loaded.config.n_layer < 1 || loaded.config.n_layer > 32 {
            return Err("Invalid number of layers");
        }

        if loaded.config.n_embd < 64 || loaded.config.n_embd > 4096 {
            return Err("Invalid embedding dimension");
        }

        crate::info!("LLM: Model loaded successfully");
        crate::info!("  Architecture: {} layers, {} embed dim",
                     loaded.config.n_layer, loaded.config.n_embd);
        crate::info!("  Vocabulary: {} tokens", loaded.config.n_vocab);
        crate::info!("  Context: {} tokens", loaded.config.n_ctx);

        // Initialize KV cache
        let kv_cache = KVCache::new(
            loaded.config.n_layer,
            loaded.config.n_ctx,
            loaded.config.n_embd,
        );

        // Store model
        self.model = Some(LoadedModel {
            config: loaded.config,
            weights: loaded.weights,
            kv_cache,
            metadata: loaded.metadata,
        });

        Ok(())
    }
}
```

**Shell Command:**
```rust
fn cmd_llmctl_load(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: llmctl load <model-path>");
        println!("Example: llmctl load /models/tinyllama-1.1b-q4_0.gguf");
        return;
    }

    let path = args[1];

    println!("Loading model: {}", path);

    let mut backend_guard = llm::get_backend();
    if let Some(backend) = backend_guard.as_mut() {
        match backend.load_model(path) {
            Ok(_) => {
                println!("✓ Model loaded successfully");

                // Print model info
                if let Some(info) = backend.model_info() {
                    println!("\nModel Information:");
                    println!("  Name: {}", info.name);
                    println!("  Architecture: {}", info.architecture);
                    println!("  Parameters: {}", info.parameters);
                    println!("  Quantization: {}", info.quantization);
                }
            }
            Err(e) => {
                println!("✗ Failed to load model: {}", e);
            }
        }
    } else {
        println!("✗ Backend not initialized");
    }
}
```

**Testing:**
```
sis> llmctl load /models/tinyllama-1.1b-q4_0.gguf
Loading model: /models/tinyllama-1.1b-q4_0.gguf
LLM: Loading model from VFS: /models/tinyllama-1.1b-q4_0.gguf
LLM: Model file size: 638976 bytes (0.6 MB)
LLM: Model file read complete, parsing GGUF...
LLM: Model loaded successfully
  Architecture: 22 layers, 2048 embed dim
  Vocabulary: 32000 tokens
  Context: 2048 tokens
✓ Model loaded successfully

Model Information:
  Name: TinyLlama-1.1B
  Architecture: llama
  Parameters: 1.1B
  Quantization: Q4_0
```

### Day 3-4: Transformer Forward Pass (16 hours)

#### Task 1.4: Implement Inference Pipeline

**File:** `crates/kernel/src/llm/backend.rs`
**Function:** `TransformerBackend::infer()`
**Effort:** 12 hours

```rust
impl LlmBackend for TransformerBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<LlmResult, &'static str> {
        // Check if model loaded
        let model = self.model.as_mut()
            .ok_or("No model loaded. Use 'llmctl load' first.")?;

        // Start timer
        let start_time = timer::get_ticks();

        // Tokenize input
        crate::info!("LLM: Tokenizing prompt...");
        let mut tokens = self.tokenizer.encode(prompt);

        if tokens.is_empty() {
            return Err("Tokenization produced no tokens");
        }

        if tokens.len() > model.config.n_ctx {
            return Err("Prompt exceeds context length");
        }

        crate::info!("LLM: Prompt tokens: {}", tokens.len());

        // Reset KV cache
        model.kv_cache.reset();

        // Generation loop
        let initial_len = tokens.len();
        let mut generated_tokens = 0;

        for step in 0..max_tokens {
            // Run transformer forward pass
            let logits = self.forward_pass(&tokens, &mut model)?;

            // Sample next token
            let next_token = self.sample_token(&logits)?;

            // Append to sequence
            tokens.push(next_token);
            generated_tokens += 1;

            // Check for EOS
            if next_token == self.tokenizer.eos_token_id() {
                crate::info!("LLM: EOS token generated at step {}", step);
                break;
            }

            // Context length check
            if tokens.len() >= model.config.n_ctx {
                crate::info!("LLM: Reached context limit");
                break;
            }
        }

        // Decode output
        let output = self.tokenizer.decode(&tokens[initial_len..]);

        // Calculate latency
        let end_time = timer::get_ticks();
        let latency_us = timer::ticks_to_us(end_time - start_time);

        // Update stats
        self.stats.total_inferences += 1;
        self.stats.total_tokens_generated += generated_tokens as u64;
        self.stats.total_latency_us += latency_us;

        crate::info!("LLM: Generated {} tokens in {} µs ({:.1} tokens/sec)",
                     generated_tokens, latency_us,
                     generated_tokens as f64 / (latency_us as f64 / 1_000_000.0));

        Ok(LlmResult {
            infer_id: self.stats.total_inferences as usize,
            tokens_emitted: generated_tokens,
            output,
            latency_us,
        })
    }
}
```

**Forward Pass Implementation:**
```rust
impl TransformerBackend {
    /// Run transformer forward pass
    fn forward_pass(
        &mut self,
        tokens: &[u16],
        model: &mut LoadedModel,
    ) -> Result<Vec<f32>, &'static str> {
        let config = &model.config;
        let weights = &model.weights;

        // Input token (last token in sequence)
        let token = tokens[tokens.len() - 1] as usize;

        if token >= config.n_vocab {
            return Err("Token ID exceeds vocabulary size");
        }

        // Get token embedding
        let emb_start = token * config.n_embd;
        let emb_end = emb_start + config.n_embd;
        let mut hidden = weights.token_embeddings[emb_start..emb_end].to_vec();

        // Process each transformer layer
        for (layer_idx, layer_weights) in weights.layers.iter().enumerate() {
            hidden = self.forward_layer(
                &hidden,
                layer_weights,
                layer_idx,
                tokens.len() - 1,  // Current position
                config,
                &mut model.kv_cache,
            )?;
        }

        // Final layer norm
        hidden = layer_norm(&hidden, &weights.final_ln_weight, &weights.final_ln_bias);

        // Output projection (language modeling head)
        let logits = matmul_vec(&hidden, &weights.output_weight,
                                config.n_embd, config.n_vocab);

        Ok(logits)
    }

    /// Process single transformer layer
    fn forward_layer(
        &self,
        input: &[f32],
        weights: &LayerWeights,
        layer_idx: usize,
        pos: usize,
        config: &TransformerConfig,
        kv_cache: &mut KVCache,
    ) -> Result<Vec<f32>, &'static str> {
        let n_embd = config.n_embd;

        // Layer norm 1
        let x = layer_norm(input, &weights.ln1_weight, &weights.ln1_bias);

        // Multi-head self-attention
        let attn_out = self.attention(&x, weights, layer_idx, pos, config, kv_cache)?;

        // Residual connection 1
        let x: Vec<f32> = input.iter()
            .zip(attn_out.iter())
            .map(|(a, b)| a + b)
            .collect();

        // Layer norm 2
        let x2 = layer_norm(&x, &weights.ln2_weight, &weights.ln2_bias);

        // Feed-forward network
        let ffn_out = self.feed_forward(&x2, weights, config)?;

        // Residual connection 2
        let output: Vec<f32> = x.iter()
            .zip(ffn_out.iter())
            .map(|(a, b)| a + b)
            .collect();

        Ok(output)
    }

    /// Multi-head self-attention
    fn attention(
        &self,
        input: &[f32],
        weights: &LayerWeights,
        layer_idx: usize,
        pos: usize,
        config: &TransformerConfig,
        kv_cache: &mut KVCache,
    ) -> Result<Vec<f32>, &'static str> {
        let n_embd = config.n_embd;
        let n_head = config.n_head;
        let head_dim = n_embd / n_head;

        // Dequantize Q, K, V weight matrices
        let mut q_weight = vec![0.0f32; n_embd * n_embd];
        let mut k_weight = vec![0.0f32; n_embd * n_embd];
        let mut v_weight = vec![0.0f32; n_embd * n_embd];

        dequantize_q4_0_to_f32(&weights.attn_q_weight, &mut q_weight);
        dequantize_q4_0_to_f32(&weights.attn_k_weight, &mut k_weight);
        dequantize_q4_0_to_f32(&weights.attn_v_weight, &mut v_weight);

        // Compute Q, K, V
        let q = matmul_vec(input, &q_weight, n_embd, n_embd);
        let k = matmul_vec(input, &k_weight, n_embd, n_embd);
        let v = matmul_vec(input, &v_weight, n_embd, n_embd);

        // Update KV cache
        kv_cache.update(layer_idx, k.clone(), v.clone());
        kv_cache.advance();

        // Get all cached K, V
        let (all_k, all_v) = kv_cache.get(layer_idx);
        let seq_len = all_k.len();

        // Multi-head attention computation
        let mut output = vec![0.0f32; n_embd];

        for head in 0..n_head {
            let head_start = head * head_dim;
            let head_end = head_start + head_dim;

            // Extract head-specific Q
            let q_head = &q[head_start..head_end];

            // Compute attention scores
            let mut scores = vec![0.0f32; seq_len];
            for i in 0..seq_len {
                let k_head = &all_k[i][head_start..head_end];
                scores[i] = dot_product(q_head, k_head) / libm::sqrtf(head_dim as f32);
            }

            // Softmax
            let attn_weights = softmax(&scores);

            // Weighted sum of values
            for i in 0..seq_len {
                let v_head = &all_v[i][head_start..head_end];
                for j in 0..head_dim {
                    output[head_start + j] += attn_weights[i] * v_head[j];
                }
            }
        }

        // Output projection
        let mut o_weight = vec![0.0f32; n_embd * n_embd];
        dequantize_q4_0_to_f32(&weights.attn_o_weight, &mut o_weight);

        let attn_out = matmul_vec(&output, &o_weight, n_embd, n_embd);

        Ok(attn_out)
    }

    /// Feed-forward network
    fn feed_forward(
        &self,
        input: &[f32],
        weights: &LayerWeights,
        config: &TransformerConfig,
    ) -> Result<Vec<f32>, &'static str> {
        let n_embd = config.n_embd;
        let n_ff = config.n_ff;

        // Dequantize weights
        let mut up_weight = vec![0.0f32; n_embd * n_ff];
        let mut down_weight = vec![0.0f32; n_ff * n_embd];

        dequantize_q4_0_to_f32(&weights.ffn_up_weight, &mut up_weight);
        dequantize_q4_0_to_f32(&weights.ffn_down_weight, &mut down_weight);

        // Up projection + GELU
        let up = matmul_vec(input, &up_weight, n_embd, n_ff);
        let activated: Vec<f32> = up.iter().map(|&x| gelu(x)).collect();

        // Down projection
        let output = matmul_vec(&activated, &down_weight, n_ff, n_embd);

        Ok(output)
    }

    /// Sample next token from logits
    fn sample_token(&self, logits: &[f32]) -> Result<u16, &'static str> {
        // Apply temperature
        let mut scaled_logits = logits.to_vec();
        if self.gen_config.temperature > 0.0 && self.gen_config.temperature != 1.0 {
            for logit in scaled_logits.iter_mut() {
                *logit /= self.gen_config.temperature;
            }
        }

        // Apply top-k filtering if enabled
        if self.gen_config.top_k > 0 {
            return Ok(sample_top_k(&scaled_logits, self.gen_config.top_k));
        }

        // Apply top-p filtering if enabled
        if self.gen_config.top_p > 0.0 {
            return Ok(sample_top_p(&scaled_logits, self.gen_config.top_p));
        }

        // Greedy decoding (argmax)
        Ok(argmax(&scaled_logits) as u16)
    }
}

/// GELU activation function
fn gelu(x: f32) -> f32 {
    0.5 * x * (1.0 + libm::tanhf(0.7978845608 * (x + 0.044715 * x * x * x)))
}

/// Dot product
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
```

**Helper Function for Q4_0 Dequantization:**
```rust
/// Dequantize Q4_0 buffer to f32
fn dequantize_q4_0_to_f32(quantized: &[u8], output: &mut [f32]) {
    // Convert quantized bytes to Q4_0Block format
    let num_blocks = quantized.len() / 18;  // Each block is 18 bytes
    let mut blocks = Vec::with_capacity(num_blocks);

    for i in 0..num_blocks {
        let offset = i * 18;
        let scale_bytes = [quantized[offset], quantized[offset + 1]];
        let scale = u16::from_le_bytes(scale_bytes);

        let mut quants = [0u8; 16];
        quants.copy_from_slice(&quantized[offset + 2..offset + 18]);

        blocks.push(Q4_0Block { scale, quants });
    }

    // Dequantize
    dequantize_q4_0(&blocks, output);
}
```

**Testing:**
```
sis> llmctl infer "Once upon a time"
LLM: Tokenizing prompt...
LLM: Prompt tokens: 5
LLM: Generated 10 tokens in 523000 µs (19.1 tokens/sec)
Once upon a time, there was a little girl named Sophie who loved to play outside.

sis> llmctl stats
Backend: TransformerBackend
Total inferences: 1
Total tokens generated: 10
Average latency: 523.0 ms
Average tokens/sec: 19.1
```

### Day 5: Timer Integration & Benchmarking (8 hours)

#### Task 1.5: Integrate ARM Generic Timer

**File:** `crates/kernel/src/timer.rs`
**New functions:** Add precise timing support
**Effort:** 3 hours

```rust
/// Get current tick count from ARM Generic Timer
pub fn get_ticks() -> u64 {
    unsafe {
        let ticks: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) ticks);
        ticks
    }
}

/// Get timer frequency (Hz)
pub fn get_frequency() -> u64 {
    unsafe {
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        freq
    }
}

/// Convert ticks to microseconds
pub fn ticks_to_us(ticks: u64) -> u64 {
    let freq = get_frequency();
    (ticks * 1_000_000) / freq
}

/// Convert ticks to milliseconds
pub fn ticks_to_ms(ticks: u64) -> u64 {
    ticks_to_us(ticks) / 1000
}

/// High-precision sleep (microseconds)
pub fn sleep_us(us: u64) {
    let freq = get_frequency();
    let start = get_ticks();
    let target_ticks = (us * freq) / 1_000_000;

    while get_ticks() - start < target_ticks {
        core::hint::spin_loop();
    }
}
```

**Update benchmark.rs:**
```rust
/// Get actual timestamp from ARM Generic Timer
fn actual_timestamp() -> u64 {
    crate::timer::get_ticks()
}

/// Convert ticks to microseconds
fn ticks_to_us(ticks: u64) -> u64 {
    crate::timer::ticks_to_us(ticks)
}

// Replace mock_timestamp() with actual_timestamp()
pub fn bench_tokenization(config: BenchmarkConfig) -> BenchmarkStats {
    // ... setup ...

    // Measure
    let mut samples = Vec::new();
    for _ in 0..config.measurement_iterations {
        let start = actual_timestamp();  // Changed
        let _ = tokenizer.encode(text);
        let elapsed_ticks = actual_timestamp() - start;  // Changed
        let elapsed_us = ticks_to_us(elapsed_ticks);  // Changed
        samples.push(elapsed_us);
    }

    // ... rest ...
}
```

#### Task 1.6: Performance Profiling

**File:** `crates/kernel/src/llm/profiler.rs` (NEW)
**Effort:** 5 hours

```rust
//! LLM Performance Profiler
//!
//! Provides detailed performance metrics for transformer inference.

use crate::timer;
use alloc::vec::Vec;
use alloc::string::String;

/// Performance profile for a single inference
#[derive(Debug, Clone)]
pub struct InferenceProfile {
    /// Total inference time (µs)
    pub total_us: u64,

    /// Breakdown by stage
    pub tokenization_us: u64,
    pub embedding_us: u64,
    pub layer_times_us: Vec<u64>,
    pub output_proj_us: u64,
    pub sampling_us: u64,
    pub decoding_us: u64,

    /// Token counts
    pub prompt_tokens: usize,
    pub generated_tokens: usize,

    /// Throughput metrics
    pub tokens_per_second: f64,
    pub prompt_eval_speed: f64,  // tokens/sec for prompt processing
    pub generation_speed: f64,    // tokens/sec for generation
}

impl InferenceProfile {
    /// Print detailed profile
    pub fn print(&self) {
        crate::info!("=== Inference Profile ===");
        crate::info!("Total time: {:.2} ms", self.total_us as f64 / 1000.0);
        crate::info!("");
        crate::info!("Stage Breakdown:");
        crate::info!("  Tokenization:  {:.2} ms ({:.1}%)",
                     self.tokenization_us as f64 / 1000.0,
                     self.tokenization_us as f64 / self.total_us as f64 * 100.0);
        crate::info!("  Embedding:     {:.2} ms ({:.1}%)",
                     self.embedding_us as f64 / 1000.0,
                     self.embedding_us as f64 / self.total_us as f64 * 100.0);

        let total_layer_time: u64 = self.layer_times_us.iter().sum();
        crate::info!("  Layers ({} total): {:.2} ms ({:.1}%)",
                     self.layer_times_us.len(),
                     total_layer_time as f64 / 1000.0,
                     total_layer_time as f64 / self.total_us as f64 * 100.0);

        // Print per-layer times
        for (i, &time) in self.layer_times_us.iter().enumerate() {
            crate::info!("    Layer {}: {:.2} ms", i, time as f64 / 1000.0);
        }

        crate::info!("  Output Proj:   {:.2} ms ({:.1}%)",
                     self.output_proj_us as f64 / 1000.0,
                     self.output_proj_us as f64 / self.total_us as f64 * 100.0);
        crate::info!("  Sampling:      {:.2} ms ({:.1}%)",
                     self.sampling_us as f64 / 1000.0,
                     self.sampling_us as f64 / self.total_us as f64 * 100.0);
        crate::info!("  Decoding:      {:.2} ms ({:.1}%)",
                     self.decoding_us as f64 / 1000.0,
                     self.decoding_us as f64 / self.total_us as f64 * 100.0);
        crate::info!("");
        crate::info!("Throughput:");
        crate::info!("  Overall:       {:.1} tokens/sec", self.tokens_per_second);
        crate::info!("  Prompt eval:   {:.1} tokens/sec", self.prompt_eval_speed);
        crate::info!("  Generation:    {:.1} tokens/sec", self.generation_speed);
    }
}

/// Profiling context (active during inference)
pub struct Profiler {
    start_time: u64,
    stage_times: Vec<(String, u64, u64)>,  // (name, start, end)
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            start_time: timer::get_ticks(),
            stage_times: Vec::new(),
        }
    }

    /// Start timing a stage
    pub fn start_stage(&mut self, name: &str) {
        let start = timer::get_ticks();
        self.stage_times.push((name.to_string(), start, 0));
    }

    /// End timing current stage
    pub fn end_stage(&mut self) {
        let end = timer::get_ticks();
        if let Some(last) = self.stage_times.last_mut() {
            last.2 = end;
        }
    }

    /// Finalize and return profile
    pub fn finalize(
        &self,
        prompt_tokens: usize,
        generated_tokens: usize,
    ) -> InferenceProfile {
        let total_ticks = timer::get_ticks() - self.start_time;
        let total_us = timer::ticks_to_us(total_ticks);

        // Extract stage times
        let mut tokenization_us = 0;
        let mut embedding_us = 0;
        let mut layer_times_us = Vec::new();
        let mut output_proj_us = 0;
        let mut sampling_us = 0;
        let mut decoding_us = 0;

        for (name, start, end) in &self.stage_times {
            let duration = timer::ticks_to_us(end - start);

            match name.as_str() {
                "tokenization" => tokenization_us += duration,
                "embedding" => embedding_us += duration,
                "output_projection" => output_proj_us += duration,
                "sampling" => sampling_us += duration,
                "decoding" => decoding_us += duration,
                _ if name.starts_with("layer_") => layer_times_us.push(duration),
                _ => {}
            }
        }

        // Calculate throughput
        let total_tokens = prompt_tokens + generated_tokens;
        let tokens_per_second = if total_us > 0 {
            (total_tokens as f64 / (total_us as f64 / 1_000_000.0))
        } else {
            0.0
        };

        let prompt_eval_speed = if tokenization_us + embedding_us > 0 {
            (prompt_tokens as f64 / ((tokenization_us + embedding_us) as f64 / 1_000_000.0))
        } else {
            0.0
        };

        let gen_time = total_us - (tokenization_us + embedding_us + decoding_us);
        let generation_speed = if gen_time > 0 && generated_tokens > 0 {
            (generated_tokens as f64 / (gen_time as f64 / 1_000_000.0))
        } else {
            0.0
        };

        InferenceProfile {
            total_us,
            tokenization_us,
            embedding_us,
            layer_times_us,
            output_proj_us,
            sampling_us,
            decoding_us,
            prompt_tokens,
            generated_tokens,
            tokens_per_second,
            prompt_eval_speed,
            generation_speed,
        }
    }
}
```

**Integration in TransformerBackend:**
```rust
use crate::llm::profiler::Profiler;

impl LlmBackend for TransformerBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<LlmResult, &'static str> {
        let mut profiler = Profiler::new();

        // Tokenize
        profiler.start_stage("tokenization");
        let mut tokens = self.tokenizer.encode(prompt);
        profiler.end_stage();

        let initial_len = tokens.len();

        // Generation loop
        for step in 0..max_tokens {
            // Forward pass with profiling
            profiler.start_stage(&format!("forward_step_{}", step));
            let logits = self.forward_pass_profiled(&tokens, &mut profiler)?;
            profiler.end_stage();

            // Sample
            profiler.start_stage("sampling");
            let next_token = self.sample_token(&logits)?;
            profiler.end_stage();

            tokens.push(next_token);

            if next_token == self.tokenizer.eos_token_id() {
                break;
            }
        }

        // Decode
        profiler.start_stage("decoding");
        let output = self.tokenizer.decode(&tokens[initial_len..]);
        profiler.end_stage();

        // Finalize profile
        let profile = profiler.finalize(initial_len, tokens.len() - initial_len);
        profile.print();

        // ... return result ...
    }
}
```

**Shell Command:**
```
sis> llmctl profile on
LLM profiling enabled

sis> llmctl infer "Hello world"
=== Inference Profile ===
Total time: 523.45 ms

Stage Breakdown:
  Tokenization:  1.23 ms (0.2%)
  Embedding:     2.45 ms (0.5%)
  Layers (22 total): 510.34 ms (97.5%)
    Layer 0: 23.12 ms
    Layer 1: 23.45 ms
    ...
  Output Proj:   5.67 ms (1.1%)
  Sampling:      0.89 ms (0.2%)
  Decoding:      2.87 ms (0.5%)

Throughput:
  Overall:       19.1 tokens/sec
  Prompt eval:   1627.2 tokens/sec (2 tokens)
  Generation:    17.3 tokens/sec (9 tokens)
```

---

## Week 2: Testing, Optimization & Documentation (40 hours)

### Day 6-7: Model Testing & Validation (16 hours)

#### Task 2.1: Prepare Test Models

**Effort:** 4 hours

**Models to test:**
1. **TinyLlama-1.1B-Q4_0** (638 MB)
   - Small, fast, good for testing
   - Download: https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF

2. **Phi-2-Q4_0** (1.6 GB)
   - Microsoft's 2.7B parameter model
   - Higher quality, still runnable

3. **Custom test model** (minimal)
   - 2 layers, 128 embedding dim
   - For unit testing

**Download and prepare:**
```bash
# Create models directory on host
mkdir -p models/

# Download TinyLlama
cd models/
wget https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_0.gguf

# Verify checksum
sha256sum tinyllama-1.1b-chat-v1.0.Q4_0.gguf

# Mount in QEMU (update uefi_run.sh)
# Add to QEMU command: -drive file=models.img,format=raw
```

**Create VirtIO block device for models:**
```bash
# Create disk image
dd if=/dev/zero of=models.img bs=1M count=2048

# Format as ext4
mkfs.ext4 models.img

# Mount and copy models
mkdir -p /tmp/models_mount
sudo mount -o loop models.img /tmp/models_mount
sudo cp models/*.gguf /tmp/models_mount/
sudo umount /tmp/models_mount
```

**Update kernel VFS mount:**
```rust
// In kernel initialization
fn mount_models_disk() -> Result<(), &'static str> {
    // Detect VirtIO block device 1 (models disk)
    let block_dev = virtio_blk::get_device(1)?;

    // Mount as /models
    vfs::mount("/models", Box::new(Ext4Fs::new(block_dev)))?;

    crate::info!("Mounted /models filesystem");

    // List available models
    if let Ok(entries) = vfs::read_dir("/models") {
        crate::info!("Available models:");
        for entry in entries {
            if entry.name.ends_with(".gguf") {
                crate::info!("  - {}", entry.name);
            }
        }
    }

    Ok(())
}
```

#### Task 2.2: Comprehensive Testing

**Effort:** 12 hours

**Test Suite 1: Functionality Tests**

**File:** `crates/kernel/src/llm/tests/integration.rs` (NEW)

```rust
//! LLM Integration Tests

use crate::llm::*;

#[test]
fn test_model_loading() {
    // Initialize backend
    init_backend(true);

    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    // Load model
    let result = backend.load_model("/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf");
    assert!(result.is_ok(), "Failed to load model: {:?}", result);

    // Verify model loaded
    assert!(backend.is_loaded());

    // Check model info
    if let Some(info) = backend.model_info() {
        assert_eq!(info.architecture, "llama");
        assert!(info.parameters > 1_000_000_000); // >1B parameters
    }
}

#[test]
fn test_simple_inference() {
    init_backend(true);
    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    backend.load_model("/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf")
        .expect("Failed to load model");

    // Run inference
    let result = backend.infer("Hello", 5);
    assert!(result.is_ok(), "Inference failed: {:?}", result);

    let output = result.unwrap();
    assert!(output.tokens_emitted > 0);
    assert!(output.output.len() > 0);
    assert!(output.latency_us > 0);
}

#[test]
fn test_empty_prompt() {
    init_backend(true);
    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    backend.load_model("/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf")
        .expect("Failed to load model");

    let result = backend.infer("", 5);
    assert!(result.is_err(), "Should reject empty prompt");
}

#[test]
fn test_long_prompt() {
    init_backend(true);
    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    backend.load_model("/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf")
        .expect("Failed to load model");

    // Create prompt exceeding context length
    let long_prompt = "test ".repeat(3000); // Much longer than 2048 tokens

    let result = backend.infer(&long_prompt, 5);
    assert!(result.is_err(), "Should reject prompt exceeding context");
}

#[test]
fn test_multiple_inferences() {
    init_backend(true);
    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    backend.load_model("/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf")
        .expect("Failed to load model");

    // Run 5 inferences
    for i in 0..5 {
        let prompt = format!("Test {}", i);
        let result = backend.infer(&prompt, 3);
        assert!(result.is_ok(), "Inference {} failed", i);
    }

    // Check stats
    let stats = backend.stats();
    assert_eq!(stats.total_inferences, 5);
    assert!(stats.total_tokens_generated >= 15); // At least 3 per inference
}
```

**Test Suite 2: Quality Tests**

**File:** `crates/kernel/src/llm/tests/quality.rs` (NEW)

```rust
//! LLM Output Quality Tests

#[test]
fn test_output_coherence() {
    init_backend(true);
    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    backend.load_model("/models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf")
        .expect("Failed to load model");

    // Test coherent continuation
    let result = backend.infer("The capital of France is", 5);
    assert!(result.is_ok());

    let output = result.unwrap().output;

    // Output should contain "Paris" or similar
    // (This is a heuristic test, actual output may vary)
    crate::info!("Output: {}", output);
    assert!(output.len() > 0, "Output should not be empty");
}

#[test]
fn test_different_temperatures() {
    init_backend(true);
    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    backend.load_model("/models/tinyllama-1.1b-chat-v1.0.gguf")
        .expect("Failed to load model");

    let prompt = "Once upon a time";

    // Test with temperature 0 (greedy, deterministic)
    // TODO: Add temperature control API
    let result1 = backend.infer(prompt, 10);
    assert!(result1.is_ok());

    let result2 = backend.infer(prompt, 10);
    assert!(result2.is_ok());

    // With temperature 0, outputs should be identical
    // (assuming deterministic sampling)
    assert_eq!(result1.unwrap().output, result2.unwrap().output);
}

#[test]
fn test_max_tokens_respected() {
    init_backend(true);
    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    backend.load_model("/models/tinyllama-1.1b-chat-v1.0.gguf")
        .expect("Failed to load model");

    // Request exactly 5 tokens
    let result = backend.infer("Hello", 5);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.tokens_emitted <= 5,
            "Generated {} tokens, max was 5", output.tokens_emitted);
}
```

**Test Suite 3: Performance Tests**

```rust
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_throughput() {
    init_backend(true);
    let mut backend = get_backend();
    let backend = backend.as_mut().unwrap();

    backend.load_model("/models/tinyllama-1.1b-chat-v1.0.gguf")
        .expect("Failed to load model");

    let result = backend.infer("Hello world", 50);
    assert!(result.is_ok());

    let output = result.unwrap();
    let tokens_per_sec = output.tokens_emitted as f64 /
                         (output.latency_us as f64 / 1_000_000.0);

    crate::info!("Throughput: {:.1} tokens/sec", tokens_per_sec);

    // Minimum acceptable throughput: 5 tokens/sec
    assert!(tokens_per_sec >= 5.0,
            "Throughput too low: {:.1} tokens/sec", tokens_per_sec);
}

#[test]
#[ignore]
fn test_memory_usage() {
    let initial_usage = heap::allocated_bytes();

    {
        init_backend(true);
        let mut backend = get_backend();
        let backend = backend.as_mut().unwrap();

        backend.load_model("/models/tinyllama-1.1b-chat-v1.0.gguf")
            .expect("Failed to load model");

        let model_usage = heap::allocated_bytes() - initial_usage;
        crate::info!("Model memory: {:.1} MB", model_usage as f64 / 1024.0 / 1024.0);

        // TinyLlama Q4_0 should be ~600 MB
        assert!(model_usage < 700 * 1024 * 1024, "Model too large");
    }

    // Check for memory leaks
    let final_usage = heap::allocated_bytes();
    let leaked = final_usage - initial_usage;

    assert!(leaked < 1024 * 1024, "Memory leak detected: {} KB", leaked / 1024);
}
```

**Running Tests:**
```bash
# Unit tests
cargo test -p sis_kernel --features llm -- llm::tests

# Integration tests
cargo test -p sis_kernel --features llm,crypto-real -- llm::tests::integration

# Performance tests
cargo test -p sis_kernel --features llm -- --ignored llm::tests::performance
```

### Day 8-9: Performance Optimization (16 hours)

#### Task 2.3: SIMD Optimizations

**File:** `crates/kernel/src/llm/simd.rs`
**Current:** Basic NEON implementations
**Target:** Optimized critical paths
**Effort:** 8 hours

**Optimizations:**

1. **Vectorized Dequantization**
```rust
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

/// NEON-optimized Q4_0 dequantization
#[cfg(target_arch = "aarch64")]
pub fn dequantize_q4_0_neon(blocks: &[Q4_0Block], output: &mut [f32]) {
    unsafe {
        for (block_idx, block) in blocks.iter().enumerate() {
            let scale = f16_to_f32(block.scale);
            let scale_vec = vdupq_n_f32(scale);

            let out_offset = block_idx * 32;

            // Process 4 values at a time using NEON
            for i in (0..32).step_by(4) {
                // Extract 4-bit values
                let byte_idx = i / 2;
                let q0 = ((block.quants[byte_idx] & 0x0F) as i8 - 8) as f32;
                let q1 = (((block.quants[byte_idx] >> 4) & 0x0F) as i8 - 8) as f32;

                // Load into NEON vector and scale
                let vals = vld1q_f32(&[q0, q1, 0.0, 0.0] as *const f32);
                let scaled = vmulq_f32(vals, scale_vec);

                // Store result
                vst1q_f32(&mut output[out_offset + i] as *mut f32, scaled);
            }
        }
    }
}
```

2. **Vectorized Matrix Multiply**
```rust
/// NEON-optimized matrix-vector multiplication
#[cfg(target_arch = "aarch64")]
pub fn matmul_vec_neon_f32(
    vec: &[f32],
    mat: &[f32],
    rows: usize,
    cols: usize,
) -> Vec<f32> {
    let mut result = vec![0.0f32; rows];

    unsafe {
        for row in 0..rows {
            let row_start = row * cols;
            let mut sum_vec = vdupq_n_f32(0.0);

            // Process 4 elements at a time
            for col in (0..cols).step_by(4) {
                if col + 4 <= cols {
                    let a = vld1q_f32(&vec[col] as *const f32);
                    let b = vld1q_f32(&mat[row_start + col] as *const f32);
                    sum_vec = vfmaq_f32(sum_vec, a, b);  // Fused multiply-add
                }
            }

            // Horizontal sum
            result[row] = vaddvq_f32(sum_vec);

            // Handle remaining elements
            for col in (cols / 4 * 4)..cols {
                result[row] += vec[col] * mat[row_start + col];
            }
        }
    }

    result
}
```

3. **Batch Processing**
```rust
/// Process multiple tokens in parallel (for prompt processing)
pub fn forward_batch(
    tokens: &[u16],
    model: &LoadedModel,
) -> Result<Vec<Vec<f32>>, &'static str> {
    let batch_size = tokens.len();
    let mut all_hidden = Vec::with_capacity(batch_size);

    // Get all embeddings at once
    for &token in tokens {
        let emb_start = token as usize * model.config.n_embd;
        let emb_end = emb_start + model.config.n_embd;
        all_hidden.push(model.weights.token_embeddings[emb_start..emb_end].to_vec());
    }

    // Process through layers (can parallelize later)
    for layer_idx in 0..model.config.n_layer {
        for hidden in &mut all_hidden {
            *hidden = forward_layer(hidden, layer_idx, model)?;
        }
    }

    Ok(all_hidden)
}
```

#### Task 2.4: Memory Optimization

**Effort:** 4 hours

**Optimizations:**

1. **Arena Reset Between Inferences**
```rust
impl LlmBackend for TransformerBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> Result<LlmResult, &'static str> {
        // Reset arena at start
        {
            let mut arena = self.arena.lock();
            arena.reset();
        }

        // ... inference ...

        // Arena automatically resets for next inference
        Ok(result)
    }
}
```

2. **Weight Caching**
```rust
/// Cache for dequantized weights (trade memory for speed)
struct WeightCache {
    attention_weights: HashMap<usize, Vec<f32>>,  // layer_idx -> weights
    ffn_weights: HashMap<usize, Vec<f32>>,
}

impl WeightCache {
    fn get_or_dequantize(&mut self, layer: usize, quantized: &[u8]) -> &[f32] {
        self.attention_weights.entry(layer).or_insert_with(|| {
            let mut weights = vec![0.0f32; quantized.len() * 2];
            dequantize_q4_0_to_f32(quantized, &mut weights);
            weights
        })
    }
}
```

3. **KV Cache Optimization**
```rust
// Use circular buffer instead of growing Vec
pub struct KVCache {
    keys: [[f32; N_EMBD]; N_CTX],    // Fixed-size array
    values: [[f32; N_EMBD]; N_CTX],
    position: usize,  // Current write position
}
```

#### Task 2.5: Benchmark Comparison

**Effort:** 4 hours

**Create benchmark harness:**
```bash
sis> llmctl bench
Running LLM benchmarks...

Prompt processing:
  10 tokens:   15.3 ms  (653 tokens/sec)
  50 tokens:   71.2 ms  (702 tokens/sec)
  100 tokens: 145.8 ms  (686 tokens/sec)

Token generation:
  1 token:    52.4 ms  (19.1 tokens/sec)
  10 tokens: 524.1 ms  (19.1 tokens/sec)
  50 tokens:   2.6 s   (19.2 tokens/sec)

Memory usage:
  Model loaded: 638 MB
  Arena usage:  12 MB peak
  Total:       650 MB
```

### Day 10: Documentation & Finalization (8 hours)

#### Task 2.6: User Documentation

**File:** `docs/llm/USER_GUIDE.md` (NEW)
**Effort:** 4 hours

Create comprehensive user guide covering:
- Loading models
- Running inference
- Configuring generation parameters
- Performance tuning
- Troubleshooting

#### Task 2.7: API Documentation

**File:** `docs/llm/API_REFERENCE.md` (NEW)
**Effort:** 2 hours

Document all public APIs:
- `init_backend()`
- `load_model()`
- `infer()`
- Configuration structures
- Error codes

#### Task 2.8: Final Integration Testing

**Effort:** 2 hours

**End-to-end test:**
```
sis> llmctl backend real
LLM backend: TransformerBackend

sis> llmctl load /models/tinyllama-1.1b-chat-v1.0.Q4_0.gguf
✓ Model loaded successfully

sis> llmctl infer "Explain operating systems in one sentence"
An operating system is software that manages computer hardware and provides
services for computer programs to run efficiently.

sis> llmctl stats
Backend: TransformerBackend
Model: TinyLlama-1.1B-Chat-v1.0
Total inferences: 1
Total tokens generated: 23
Average latency: 1.2 sec
Average tokens/sec: 19.1

sis> llmctl profile
Last inference profile:
  Tokenization:   1.2 ms
  Forward pass: 1,198.5 ms
  Sampling:       0.8 ms
  Decoding:       2.1 ms
```

---

## Success Criteria

### Minimal Success (MVP)

- ✅ TransformerBackend can be initialized
- ✅ Model loads from VFS successfully
- ✅ Single inference completes without crash
- ✅ Output is non-empty and coherent

### Functional Success

- ✅ Multiple inferences work correctly
- ✅ Different prompts produce different outputs
- ✅ Performance is acceptable (>5 tokens/sec)
- ✅ Memory usage is reasonable (<1GB)
- ✅ All integration tests pass

### Production Success

- ✅ Throughput >10 tokens/sec
- ✅ SIMD optimizations active
- ✅ Comprehensive error handling
- ✅ Full documentation
- ✅ Profiling and benchmarking tools
- ✅ Works with multiple model sizes

---

## Risk Mitigation

### Risk 1: VFS Integration Fails

**Likelihood:** Low
**Impact:** High
**Mitigation:**
- Test VFS file operations independently first
- Fallback to in-memory model loading if needed
- Add detailed error logging

### Risk 2: Performance Below Target

**Likelihood:** Medium
**Impact:** Medium
**Mitigation:**
- Profile early to identify bottlenecks
- SIMD optimizations ready
- Can reduce model size if needed

### Risk 3: Memory Exhaustion

**Likelihood:** Low
**Impact:** High
**Mitigation:**
- Arena allocator prevents fragmentation
- Model size validation before loading
- Monitoring during testing

### Risk 4: Numeric Instability

**Likelihood:** Low
**Impact:** Medium
**Mitigation:**
- Use well-tested quantization code
- Validate outputs against reference implementation
- Add numerical checks (NaN, Inf detection)

---

## Dependencies

### External

- **GGUF Model File:** TinyLlama-1.1B-Q4_0 (~638 MB)
  - Download from HuggingFace
  - Mount via VirtIO block device

### Internal

- ✅ **VFS:** File operations (`open`, `read`, `stat`)
- ✅ **Timer:** ARM Generic Timer access
- ✅ **Allocator:** Arena allocator for tensors
- ✅ **SIMD:** ARM NEON intrinsics
- ✅ **libm:** Math functions (sqrt, exp, tanh)

---

## Timeline

### Week 1: Implementation

| Day | Tasks | Hours | Deliverable |
|-----|-------|-------|-------------|
| 1-2 | Backend activation, VFS integration | 16 | TransformerBackend loads models |
| 3-4 | Forward pass, inference loop | 16 | Basic inference works |
| 5   | Timer integration, benchmarking | 8 | Performance metrics |

### Week 2: Testing & Optimization

| Day | Tasks | Hours | Deliverable |
|-----|-------|-------|-------------|
| 6-7 | Testing, validation | 16 | Comprehensive test suite |
| 8-9 | SIMD optimization, tuning | 16 | >10 tokens/sec throughput |
| 10  | Documentation, finalization | 8 | Production ready |

**Total:** 80 hours (2 weeks)

---

## Validation Plan

### Phase 1: Smoke Tests (Day 1-2)

- [ ] Backend initializes without crash
- [ ] Model file opens from VFS
- [ ] GGUF parsing succeeds
- [ ] Tokenizer encodes/decodes

### Phase 2: Integration Tests (Day 3-5)

- [ ] Forward pass completes
- [ ] Attention mechanism works
- [ ] Feed-forward works
- [ ] Output projection works
- [ ] Full inference end-to-end

### Phase 3: Quality Tests (Day 6-7)

- [ ] Output is coherent
- [ ] Different prompts → different outputs
- [ ] Max tokens respected
- [ ] EOS token handling
- [ ] Error cases handled

### Phase 4: Performance Tests (Day 8-9)

- [ ] Throughput >10 tokens/sec
- [ ] Latency <2 sec for 10 tokens
- [ ] Memory usage <1 GB
- [ ] No memory leaks
- [ ] SIMD optimizations active

### Phase 5: Documentation (Day 10)

- [ ] User guide complete
- [ ] API reference complete
- [ ] Examples working
- [ ] Troubleshooting guide

---

## Monitoring & Metrics

### Build Metrics

```bash
# Code size
wc -l crates/kernel/src/llm/*.rs
# Should be ~11,000 lines

# Binary size
ls -lh target/aarch64-unknown-none/debug/sis_kernel
# Target: <5 MB kernel binary
```

### Runtime Metrics

```
llmctl stats
Backend: TransformerBackend
Model: TinyLlama-1.1B-Chat-v1.0
Status: Loaded
Memory: 638 MB

Performance:
  Total inferences: 42
  Total tokens: 523
  Average latency: 1.2 sec
  Average throughput: 19.1 tokens/sec

Cache stats:
  KV cache size: 12 MB
  Arena peak: 8 MB
```

---

## Rollback Plan

If major issues arise, fallback to StubBackend:

```
sis> llmctl backend stub
LLM backend: StubBackend (safe mode)

sis> llmctl infer "test"
[Stub output: synthetic response]
```

All existing functionality preserved with stub backend.

---

## Next Steps After Completion

Once LLM production is complete:

1. **Hardware Integration** - Deploy on Raspberry Pi 5
2. **Model Variants** - Support different architectures
3. **Streaming Output** - Real-time token generation
4. **Multi-turn Chat** - Conversation history
5. **Function Calling** - Structured output
6. **Fine-tuning** - Custom model adaptation

---

## Appendix: Shell Commands

### LLM Control Commands

```bash
# Backend management
llmctl backend [stub|real]       # Switch backend
llmctl info                      # Backend information

# Model management
llmctl load <path>               # Load model from VFS
llmctl unload                    # Unload current model
llmctl models                    # List available models

# Inference
llmctl infer "<prompt>"          # Run inference
llmctl infer "<prompt>" --max-tokens N
llmctl infer "<prompt>" --temperature 0.7

# Monitoring
llmctl stats                     # Performance statistics
llmctl profile [on|off]          # Toggle profiling
llmctl bench                     # Run benchmarks

# Configuration
llmctl config temperature 0.8    # Set parameter
llmctl config top-k 40
llmctl config top-p 0.95
```

---

**Document Version:** 1.0
**Last Updated:** 2025-11-17
**Status:** 🚧 Ready to Execute
**Estimated Completion:** 2 weeks from start
