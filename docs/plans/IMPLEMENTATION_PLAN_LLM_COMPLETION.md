# LLM Subsystem Completion Plan

**Status**: Planning → Implementation
**Priority**: P1 - Critical for "AI-Native" kernel claim
**Timeline**: 12-16 weeks (3-4 months)
**Goal**: Complete LLM subsystem from framework to production-ready inference

---

## Executive Summary

The SIS Kernel currently has **excellent LLM infrastructure** but uses a deterministic stub for inference validation. This plan outlines the steps to integrate a real transformer backend while maintaining the kernel's safety, determinism, and real-time properties.

### Current Status (v0.1.0)

**✅ Production-Ready** (8.5/10):
- Model loading and security (SHA-256 + Ed25519)
- Resource budgeting and quota enforcement
- Audit logging for compliance
- LoRA fine-tuning infrastructure
- Streaming inference API
- Session management (32 concurrent)
- Drift detection and version control
- 14 shell commands

**⚠️ Stub Implementation**:
- Inference echoes transformed tokens (deterministic validation)
- No real transformer weights
- No actual language understanding

**✅ Fully Implemented**:
- Transformer-based scheduler (`sched/transformer_sched.rs`)
- Neural agent (fixed-point MLP in `neural.rs`)

### Vision

**"Real LLM inference in kernel space with deterministic bounds and zero userspace overhead"**

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Phase-Based Implementation](#phase-based-implementation)
3. [Milestone 0: Research & Design](#milestone-0-research--design)
4. [Milestone 1: Minimal Transformer Core](#milestone-1-minimal-transformer-core)
5. [Milestone 2: Model Loading](#milestone-2-model-loading)
6. [Milestone 3: Integration](#milestone-3-integration)
7. [Milestone 4: Optimization](#milestone-4-optimization)
8. [Milestone 5: Production Hardening](#milestone-5-production-hardening)
9. [Milestone 6: Testing & Validation](#milestone-6-testing--validation)
10. [Technical Decisions](#technical-decisions)
11. [Risk Mitigation](#risk-mitigation)
12. [Success Metrics](#success-metrics)

---

## Architecture Overview

### Design Constraints

1. **No Dynamic Memory in Hot Path**: Use static allocation or bounded buffers
2. **Deterministic Execution**: WCET bounds required for real-time guarantees
3. **Small Footprint**: Target <5MB memory overhead
4. **No External Dependencies**: Minimize crate dependencies
5. **Safe Rust**: No unsafe unless absolutely necessary
6. **Feature-Gated**: LLM remains optional (`feature = "llm"`)

### Proposed Architecture

```
┌─────────────────────────────────────────────────────┐
│  Shell Commands (llminfer, llmctl, etc.)           │
└─────────────────┬───────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│  llm::basic (Current Interface - KEEP)             │
│  - infer(), load_model(), configure_budget()       │
│  - Audit, metrics, session management              │
└─────────────────┬───────────────────────────────────┘
                  │
      ┌───────────▼───────────┐
      │  Backend Abstraction  │ ← NEW
      └───────────┬───────────┘
                  │
      ┌───────────▼───────────────────────┐
      │  Transformer Backend (NEW)        │
      │  ┌─────────────────────────────┐  │
      │  │ Model Loader                │  │
      │  │ - GGUF/safetensors parser   │  │
      │  │ - Weight quantization       │  │
      │  │ - Model validation          │  │
      │  └─────────────────────────────┘  │
      │  ┌─────────────────────────────┐  │
      │  │ Inference Engine            │  │
      │  │ - Tokenizer                 │  │
      │  │ - Transformer layers        │  │
      │  │ - Attention mechanism       │  │
      │  │ - KV cache                  │  │
      │  └─────────────────────────────┘  │
      │  ┌─────────────────────────────┐  │
      │  │ Quantization                │  │
      │  │ - Q4_0, Q4_1, Q8_0          │  │
      │  │ - SIMD vectorization        │  │
      │  └─────────────────────────────┘  │
      └───────────────────────────────────┘
                  │
      ┌───────────▼───────────┐
      │  Memory Arena         │
      │  - Static 4MB buffer  │
      │  - Bump allocator     │
      └───────────────────────┘
```

---

## Phase-Based Implementation

### Development Phases

```
Phase 0/1 (CURRENT): Stub implementation ✅
  ├─ Infrastructure complete
  └─ Validation framework ready

Phase 2 (COMPLETE): AI Governance ✅
  ├─ Drift detection
  ├─ Version control
  └─ LoRA fine-tuning

Phase 3 (NEW): Minimal Transformer (M1-M2)
  ├─ Tiny model (117M params)
  ├─ Basic tokenizer
  └─ Single-layer transformer

Phase 4 (NEW): Production Backend (M3-M5)
  ├─ Multi-layer transformer
  ├─ Optimized quantization
  └─ Full GGUF support

Phase 5 (FUTURE): Advanced Features
  ├─ LoRA integration with real models
  ├─ Multi-model support
  └─ GPU offload (if available)
```

---

## Milestone 0: Research & Design

**Duration**: 2 weeks
**Goal**: Design decisions and proof of concept

### M0.1: Technology Stack Selection

**Decision Matrix**:

| Backend Option | Pros | Cons | Verdict |
|---------------|------|------|---------|
| **llama.cpp** | Battle-tested, GGUF support | Large C++ codebase, complex | ❌ Too complex |
| **candle-rs** | Pure Rust, clean API | Heap allocations, not no_std | ❌ Needs userspace |
| **Custom minimal** | Full control, kernel-safe | Development time | ✅ **RECOMMENDED** |
| **tinygrad** | Small, Python-like | Python dependency | ❌ Wrong language |

**Decision**: **Build custom minimal transformer** based on:
- llama.cpp quantization algorithms (Q4_0, Q8_0)
- GGUF format for model weights
- Simplified transformer architecture

**Rationale**:
- Full control over memory allocation
- Can guarantee deterministic bounds
- No external runtime dependencies
- Educational value for kernel developers

### M0.2: Model Format Selection

**GGUF (GPT-Generated Unified Format)**:
- ✅ Compact binary format
- ✅ Supports quantization (Q4_0, Q8_0, etc.)
- ✅ Well-documented
- ✅ Used by llama.cpp ecosystem
- ✅ Can convert from Hugging Face models

**Alternative**: Safetensors
- ❌ Larger file size
- ❌ Less compact for quantized models

**Decision**: Use **GGUF** with Q4_0 quantization

### M0.3: Model Size Target

**Constraints**:
- Kernel memory budget: 4-8 MB for model weights
- Context length: 512-1024 tokens (vs 2048+ for full models)
- Vocabulary: 32k tokens (standard BPE)

**Target Models**:

1. **TinyLlama-1.1B** (Quantized Q4_0):
   - Original: 1.1B params ~4.4GB
   - Q4_0: ~550 MB ❌ Too large
   - Q8_0: ~1.1 GB ❌ Still too large

2. **GPT-2 Small** (117M params, Quantized Q4_0):
   - Original: 117M params ~500MB
   - Q4_0: ~60 MB ⚠️ Borderline
   - Q8_0: ~120 MB ❌ Too large

3. **Custom Tiny Model** (10-50M params):
   - Custom architecture
   - Target: <10 MB quantized
   - ✅ **RECOMMENDED for MVP**

**Decision**: Start with **custom 10-50M param model**
- 6 layers (vs 12 for GPT-2)
- 384 hidden dim (vs 768)
- 6 attention heads (vs 12)
- Estimated size: 5-8 MB (Q4_0)

### M0.4: Proof of Concept

**Deliverable**: Standalone Rust program (not kernel yet)

**Tasks**:
1. **Tokenizer PoC** (3 days):
   ```rust
   // Simple BPE tokenizer
   fn tokenize(text: &str) -> Vec<u16> {
       // Byte-pair encoding
       // Target: 32k vocab
   }
   ```

2. **Single Attention Layer** (4 days):
   ```rust
   // Simplified attention
   fn attention(
       query: &[f32],
       key: &[f32],
       value: &[f32],
       dim: usize
   ) -> Vec<f32> {
       // Q * K^T / sqrt(d)
       // softmax
       // * V
   }
   ```

3. **Q4_0 Quantization** (3 days):
   ```rust
   // 4-bit quantization
   struct Q4_0Block {
       scale: f16,      // 2 bytes
       quants: [u8; 16] // 16 nibbles (4-bit values)
   }
   ```

4. **Memory Arena Allocator** (2 days):
   ```rust
   struct LlmArena {
       buffer: [u8; 8 * 1024 * 1024], // 8MB static
       offset: usize,
   }
   ```

### M0 Deliverables

- [ ] Technology stack chosen and documented
- [ ] Model format (GGUF) support PoC
- [ ] Tokenizer working (encode/decode)
- [ ] Single attention layer validated
- [ ] Q4_0 quantization implemented
- [ ] Memory arena allocator tested
- [ ] Design document approved

**Success Criteria**:
- Can load tiny GGUF model
- Can run single inference (100 tokens)
- Memory usage <10 MB
- Inference time <500ms (userspace PoC)

---

## Milestone 1: Minimal Transformer Core

**Duration**: 3 weeks
**Goal**: Working transformer inference in kernel space

### M1.1: Kernel Memory Arena

**File**: `crates/kernel/src/llm/arena.rs` (NEW)

**Implementation**:

```rust
//! Static memory arena for LLM operations
//!
//! Provides bounded, deterministic memory allocation for model weights
//! and activation buffers.

use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

const ARENA_SIZE: usize = 8 * 1024 * 1024; // 8 MB

pub struct LlmArena {
    buffer: [u8; ARENA_SIZE],
    offset: usize,
    high_water_mark: usize,
}

impl LlmArena {
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; ARENA_SIZE],
            offset: 0,
            high_water_mark: 0,
        }
    }

    /// Allocate memory from arena
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        // Align offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);

        // Check bounds
        if aligned_offset + size > ARENA_SIZE {
            return None;
        }

        let ptr = unsafe { self.buffer.as_mut_ptr().add(aligned_offset) };
        self.offset = aligned_offset + size;

        if self.offset > self.high_water_mark {
            self.high_water_mark = self.offset;
        }

        Some(ptr)
    }

    /// Reset arena (call between inferences)
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Get current usage
    pub fn usage(&self) -> (usize, usize) {
        (self.offset, self.high_water_mark)
    }
}

static LLM_ARENA: Mutex<LlmArena> = Mutex::new(LlmArena::new());

pub fn arena() -> &'static Mutex<LlmArena> {
    &LLM_ARENA
}
```

**Tests**:
- [ ] Allocation works
- [ ] Alignment enforced
- [ ] Bounds checking
- [ ] Reset clears offset
- [ ] High water mark tracked

### M1.2: BPE Tokenizer

**File**: `crates/kernel/src/llm/tokenizer.rs` (NEW)

**Implementation**:

```rust
//! Byte-Pair Encoding (BPE) Tokenizer
//!
//! Implements the BPE algorithm used by GPT-2/GPT-3/Llama models.
//! Vocabulary is loaded from model file (GGUF).

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

const MAX_VOCAB: usize = 32768;  // 32k tokens
const MAX_TOKEN_LEN: usize = 64; // Max bytes per token

pub struct BpeTokenizer {
    /// Token ID -> byte sequence
    vocab: BTreeMap<u16, Vec<u8>>,
    /// Byte sequence -> Token ID (for encoding)
    reverse_vocab: BTreeMap<Vec<u8>, u16>,
    /// Merges (for BPE algorithm)
    merges: Vec<(Vec<u8>, Vec<u8>)>,
}

impl BpeTokenizer {
    pub fn new() -> Self {
        Self {
            vocab: BTreeMap::new(),
            reverse_vocab: BTreeMap::new(),
            merges: Vec::new(),
        }
    }

    /// Load vocabulary from GGUF model
    pub fn load_from_gguf(&mut self, vocab_data: &[u8]) -> Result<(), &'static str> {
        // Parse GGUF vocab section
        // Format: token_id (u16) + length (u8) + bytes
        let mut offset = 0;
        while offset < vocab_data.len() {
            if offset + 3 > vocab_data.len() {
                break;
            }

            let token_id = u16::from_le_bytes([
                vocab_data[offset],
                vocab_data[offset + 1]
            ]);
            let len = vocab_data[offset + 2] as usize;
            offset += 3;

            if offset + len > vocab_data.len() {
                return Err("Truncated vocab data");
            }

            let bytes = vocab_data[offset..offset + len].to_vec();
            self.vocab.insert(token_id, bytes.clone());
            self.reverse_vocab.insert(bytes, token_id);
            offset += len;
        }

        Ok(())
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str) -> Vec<u16> {
        let mut tokens = Vec::new();
        let bytes = text.as_bytes();

        let mut i = 0;
        while i < bytes.len() {
            // Greedy matching: find longest token
            let mut matched_len = 1;
            let mut matched_id = 0u16;

            for len in (1..=MAX_TOKEN_LEN.min(bytes.len() - i)).rev() {
                let candidate = &bytes[i..i + len];
                if let Some(&token_id) = self.reverse_vocab.get(candidate) {
                    matched_len = len;
                    matched_id = token_id;
                    break;
                }
            }

            tokens.push(matched_id);
            i += matched_len;
        }

        tokens
    }

    /// Decode token IDs to text
    pub fn decode(&self, tokens: &[u16]) -> String {
        let mut result = Vec::new();

        for &token_id in tokens {
            if let Some(bytes) = self.vocab.get(&token_id) {
                result.extend_from_slice(bytes);
            }
        }

        String::from_utf8_lossy(&result).to_string()
    }
}
```

**Tests**:
- [ ] Load vocab from test data
- [ ] Encode "Hello World" → token IDs
- [ ] Decode token IDs → "Hello World"
- [ ] Round-trip consistency
- [ ] Handle unknown bytes

### M1.3: Q4_0 Quantization

**File**: `crates/kernel/src/llm/quantize.rs` (NEW)

**Implementation**:

```rust
//! 4-bit Quantization (Q4_0)
//!
//! Implements llama.cpp-compatible Q4_0 quantization format.
//! Each block: 1 scale (f16) + 16 nibbles (4-bit values)

use core::f32;

const QK4_0: usize = 32; // Block size

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Q4_0Block {
    /// Scale factor (f16 = 2 bytes)
    pub scale: u16, // Stored as u16, interpreted as f16
    /// 32 values packed into 16 bytes (2 per byte)
    pub quants: [u8; QK4_0 / 2],
}

impl Q4_0Block {
    /// Dequantize a single value at index i
    pub fn dequant(&self, i: usize) -> f32 {
        debug_assert!(i < QK4_0);

        // Extract nibble (4-bit value)
        let byte_idx = i / 2;
        let nibble = if i % 2 == 0 {
            self.quants[byte_idx] & 0x0F
        } else {
            self.quants[byte_idx] >> 4
        };

        // Convert to signed (-8 to 7)
        let signed = (nibble as i8) - 8;

        // Scale: convert u16 to f16 then to f32
        let scale_f32 = half::f16::from_bits(self.scale).to_f32();

        signed as f32 * scale_f32
    }
}

/// Dequantize entire Q4_0 tensor
pub fn dequantize_q4_0(blocks: &[Q4_0Block], output: &mut [f32]) {
    let num_blocks = blocks.len();
    assert_eq!(output.len(), num_blocks * QK4_0);

    for (block_idx, block) in blocks.iter().enumerate() {
        for i in 0..QK4_0 {
            output[block_idx * QK4_0 + i] = block.dequant(i);
        }
    }
}
```

**Dependencies**:
- Add `half` crate for f16 support (no_std compatible)

**Tests**:
- [ ] Dequantize known Q4_0 block
- [ ] Check value range (-8 to 7) * scale
- [ ] Benchmark dequantization speed

### M1.4: Single Transformer Layer

**File**: `crates/kernel/src/llm/transformer.rs` (NEW)

**Implementation** (Simplified):

```rust
//! Minimal transformer layer
//!
//! Implements:
//! - Multi-head self-attention
//! - Feed-forward network
//! - Layer normalization

use crate::llm::arena::arena;
use crate::llm::quantize::{Q4_0Block, dequantize_q4_0};

pub struct TransformerConfig {
    pub n_vocab: usize,      // 32k
    pub n_ctx: usize,        // 512 context length
    pub n_embd: usize,       // 384 embedding dim
    pub n_head: usize,       // 6 attention heads
    pub n_layer: usize,      // 6 layers
}

pub struct TransformerLayer {
    // Attention weights (Q, K, V projections)
    attn_q: Vec<Q4_0Block>,
    attn_k: Vec<Q4_0Block>,
    attn_v: Vec<Q4_0Block>,
    attn_out: Vec<Q4_0Block>,

    // Feed-forward weights
    ffn_up: Vec<Q4_0Block>,
    ffn_down: Vec<Q4_0Block>,

    // Layer norm params
    ln1_weight: Vec<f32>,
    ln1_bias: Vec<f32>,
    ln2_weight: Vec<f32>,
    ln2_bias: Vec<f32>,
}

impl TransformerLayer {
    /// Forward pass through single layer
    pub fn forward(
        &self,
        input: &[f32],     // n_embd
        config: &TransformerConfig
    ) -> Vec<f32> {
        let n_embd = config.n_embd;

        // 1. Layer norm
        let normed = layer_norm(input, &self.ln1_weight, &self.ln1_bias);

        // 2. Attention
        let attn_out = self.attention(&normed, config);

        // 3. Residual connection
        let residual1: Vec<f32> = input.iter()
            .zip(attn_out.iter())
            .map(|(a, b)| a + b)
            .collect();

        // 4. Layer norm 2
        let normed2 = layer_norm(&residual1, &self.ln2_weight, &self.ln2_bias);

        // 5. Feed-forward
        let ffn_out = self.feed_forward(&normed2, config);

        // 6. Residual connection
        residual1.iter()
            .zip(ffn_out.iter())
            .map(|(a, b)| a + b)
            .collect()
    }

    fn attention(&self, input: &[f32], config: &TransformerConfig) -> Vec<f32> {
        // Simplified single-head attention
        let n_embd = config.n_embd;

        // Dequantize Q, K, V matrices
        let mut q_weights = vec![0f32; n_embd * n_embd];
        dequantize_q4_0(&self.attn_q, &mut q_weights);

        // Q = input * W_q (simplified: just matmul)
        let q = matmul(input, &q_weights, n_embd, n_embd);

        // ... (similar for K, V)
        // ... (attention scores, softmax, output)

        vec![0f32; n_embd] // Stub for now
    }

    fn feed_forward(&self, input: &[f32], config: &TransformerConfig) -> Vec<f32> {
        // FFN: up projection -> GELU -> down projection
        vec![0f32; config.n_embd] // Stub
    }
}

fn layer_norm(input: &[f32], weight: &[f32], bias: &[f32]) -> Vec<f32> {
    // LayerNorm: (x - mean) / sqrt(var + eps) * weight + bias
    let mean: f32 = input.iter().sum::<f32>() / input.len() as f32;
    let variance: f32 = input.iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f32>() / input.len() as f32;
    let std = (variance + 1e-5).sqrt();

    input.iter().enumerate()
        .map(|(i, &x)| ((x - mean) / std) * weight[i] + bias[i])
        .collect()
}

fn matmul(a: &[f32], b: &[f32], m: usize, n: usize) -> Vec<f32> {
    // Simple matrix multiply: A (1 x m) * B (m x n) = C (1 x n)
    let mut c = vec![0f32; n];
    for i in 0..n {
        for j in 0..m {
            c[i] += a[j] * b[j * n + i];
        }
    }
    c
}
```

### M1.5: Integration Test (Userspace)

**File**: `crates/llm-poc/src/main.rs` (NEW)

Create standalone test harness:

```rust
//! LLM PoC - userspace test
//! Tests transformer core before kernel integration

use llm_core::*;

fn main() {
    println!("LLM Transformer PoC");

    // 1. Test tokenizer
    let mut tokenizer = BpeTokenizer::new();
    // Load tiny vocab (100 tokens for testing)
    tokenizer.load_from_test_data();

    let text = "Hello World";
    let tokens = tokenizer.encode(text);
    println!("Tokens: {:?}", tokens);

    let decoded = tokenizer.decode(&tokens);
    println!("Decoded: {}", decoded);
    assert_eq!(text, decoded);

    // 2. Test Q4_0 dequantization
    // ... (load test weights)

    // 3. Test single layer forward pass
    let config = TransformerConfig {
        n_vocab: 100,
        n_ctx: 64,
        n_embd: 128,
        n_head: 4,
        n_layer: 2,
    };

    // ... (create layer, run forward pass)

    println!("✅ All tests passed");
}
```

### M1 Deliverables

- [ ] Memory arena working in kernel
- [ ] BPE tokenizer functional
- [ ] Q4_0 dequantization correct
- [ ] Single transformer layer implemented
- [ ] Userspace PoC passing all tests
- [ ] Memory usage <8 MB

---

## Milestone 2: Model Loading

**Duration**: 2 weeks
**Goal**: Load real GGUF models into kernel

### M2.1: GGUF Parser

**File**: `crates/kernel/src/llm/gguf.rs` (NEW)

**GGUF Format** (from llama.cpp):

```
Header:
  magic: u32 (0x46554747 = "GGUF")
  version: u32
  n_tensors: u64
  n_kv: u64

Metadata (key-value pairs):
  key_length: u32
  key: [u8; key_length]
  value_type: u32
  value: <varies>

Tensor Info:
  name_length: u32
  name: [u8; name_length]
  n_dims: u32
  dims: [u64; n_dims]
  type: u32 (Q4_0, Q8_0, F32, etc.)
  offset: u64

Tensor Data:
  <aligned raw bytes>
```

**Implementation**:

```rust
//! GGUF model file parser
//!
//! Parses GGUF format used by llama.cpp ecosystem.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

const GGUF_MAGIC: u32 = 0x46554747; // "GGUF"
const GGUF_VERSION: u32 = 3;

#[repr(u32)]
enum GgufType {
    Q4_0 = 2,
    Q4_1 = 3,
    Q8_0 = 8,
    F32 = 0,
    F16 = 1,
}

pub struct GgufMetadata {
    pub kv: BTreeMap<String, GgufValue>,
}

pub enum GgufValue {
    U32(u32),
    F32(f32),
    String(String),
    // ... other types
}

pub struct GgufTensor {
    pub name: String,
    pub dims: Vec<u64>,
    pub type_id: u32,
    pub offset: u64,
    pub data: Vec<u8>,
}

pub struct GgufModel {
    pub metadata: GgufMetadata,
    pub tensors: BTreeMap<String, GgufTensor>,
}

impl GgufModel {
    /// Parse GGUF file from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        let mut offset = 0;

        // 1. Parse header
        let magic = read_u32(data, &mut offset)?;
        if magic != GGUF_MAGIC {
            return Err("Invalid GGUF magic");
        }

        let version = read_u32(data, &mut offset)?;
        if version != GGUF_VERSION {
            return Err("Unsupported GGUF version");
        }

        let n_tensors = read_u64(data, &mut offset)? as usize;
        let n_kv = read_u64(data, &mut offset)? as usize;

        // 2. Parse metadata
        let mut kv = BTreeMap::new();
        for _ in 0..n_kv {
            let key = read_string(data, &mut offset)?;
            let value = read_value(data, &mut offset)?;
            kv.insert(key, value);
        }

        // 3. Parse tensor info
        let mut tensors = BTreeMap::new();
        for _ in 0..n_tensors {
            let name = read_string(data, &mut offset)?;
            let n_dims = read_u32(data, &mut offset)? as usize;
            let mut dims = Vec::new();
            for _ in 0..n_dims {
                dims.push(read_u64(data, &mut offset)?);
            }
            let type_id = read_u32(data, &mut offset)?;
            let tensor_offset = read_u64(data, &mut offset)?;

            tensors.insert(name.clone(), GgufTensor {
                name,
                dims,
                type_id,
                offset: tensor_offset,
                data: Vec::new(), // Filled later
            });
        }

        // 4. Align to tensor data section
        let alignment = 32; // GGUF uses 32-byte alignment
        offset = (offset + alignment - 1) & !(alignment - 1);

        // 5. Load tensor data
        for tensor in tensors.values_mut() {
            let data_offset = offset + tensor.offset as usize;
            let size = calculate_tensor_size(&tensor.dims, tensor.type_id);

            if data_offset + size > data.len() {
                return Err("Truncated tensor data");
            }

            tensor.data = data[data_offset..data_offset + size].to_vec();
        }

        Ok(Self {
            metadata: GgufMetadata { kv },
            tensors,
        })
    }
}

fn read_u32(data: &[u8], offset: &mut usize) -> Result<u32, &'static str> {
    if *offset + 4 > data.len() {
        return Err("Buffer underflow");
    }
    let value = u32::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
    Ok(value)
}

// Similar for read_u64, read_string, read_value, etc.
```

### M2.2: Model Storage

**Options**:

1. **Embedded in Kernel Binary** (for tiny models):
   ```rust
   const TINY_MODEL: &[u8] = include_bytes!("../models/tiny-10m.gguf");
   ```
   - ✅ Always available
   - ❌ Increases binary size

2. **Load from VFS** (recommended):
   ```rust
   let model_data = vfs::read_file("/models/tiny-10m.gguf")?;
   ```
   - ✅ Flexible
   - ✅ Can swap models
   - ⚠️ Requires VFS mounted

**Decision**: Support both, prefer VFS

### M2.3: Weight Caching

**Challenge**: Model weights (5-8 MB) don't fit in memory multiple times

**Solution**: Memory-map weights, dequantize on-the-fly

```rust
pub struct ModelWeights {
    /// Raw GGUF tensor data (memory-mapped or static)
    raw_data: &'static [u8],

    /// Tensor metadata (offsets, sizes)
    tensors: BTreeMap<String, TensorInfo>,
}

impl ModelWeights {
    /// Get dequantized tensor (temporary allocation)
    pub fn get_tensor(&self, name: &str) -> Vec<f32> {
        let info = &self.tensors[name];
        let raw = &self.raw_data[info.offset..info.offset + info.size];

        // Dequantize on demand
        match info.type_id {
            Q4_0 => dequantize_q4_0(raw),
            F32 => raw.as_f32_slice().to_vec(),
            // ...
        }
    }
}
```

### M2 Deliverables

- [ ] GGUF parser functional
- [ ] Can load tiny model (<10 MB)
- [ ] Weights accessible via VFS
- [ ] Memory-mapped weight access
- [ ] Model metadata extracted

---

## Milestone 3: Integration

**Duration**: 2 weeks
**Goal**: Replace stub inference with real transformer

### M3.1: Backend Abstraction Layer

**File**: `crates/kernel/src/llm/backend.rs` (NEW)

```rust
//! Backend abstraction for LLM inference
//!
//! Allows swapping between stub and real transformer

pub trait LlmBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> LlmResult;
    fn load_model(&mut self, path: &str) -> Result<(), &'static str>;
}

/// Stub backend (current implementation)
pub struct StubBackend;

impl LlmBackend for StubBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> LlmResult {
        // Current stub implementation
        // (echo transformed tokens)
    }
}

/// Transformer backend (new)
pub struct TransformerBackend {
    model: Option<GgufModel>,
    tokenizer: BpeTokenizer,
    config: TransformerConfig,
}

impl LlmBackend for TransformerBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize) -> LlmResult {
        // Real transformer inference
        let tokens = self.tokenizer.encode(prompt);
        let output_tokens = self.generate(&tokens, max_tokens);
        let output_text = self.tokenizer.decode(&output_tokens);

        LlmResult {
            infer_id: get_next_id(),
            tokens_emitted: output_tokens.len(),
            output: output_text,
            latency_us: /* measure */,
        }
    }
}
```

### M3.2: Inference Loop

**File**: `crates/kernel/src/llm/generate.rs` (NEW)

```rust
//! Autoregressive text generation
//!
//! Implements the inference loop:
//! 1. Tokenize prompt
//! 2. For each position:
//!    a. Forward pass through transformer
//!    b. Sample next token
//!    c. Append to sequence
//! 3. Decode to text

impl TransformerBackend {
    pub fn generate(&mut self, prompt_tokens: &[u16], max_tokens: usize) -> Vec<u16> {
        let mut output = prompt_tokens.to_vec();

        for _ in 0..max_tokens {
            // Forward pass
            let logits = self.forward(&output);

            // Sample next token (greedy for now)
            let next_token = argmax(&logits);

            // Append
            output.push(next_token);

            // Stop on EOS token
            if next_token == self.config.eos_token_id {
                break;
            }
        }

        output
    }

    fn forward(&self, tokens: &[u16]) -> Vec<f32> {
        let seq_len = tokens.len();

        // 1. Embedding lookup
        let mut hidden = self.embed(tokens);

        // 2. Transformer layers
        for layer in &self.layers {
            hidden = layer.forward(&hidden, &self.config);
        }

        // 3. Final layer norm
        hidden = layer_norm(&hidden, &self.ln_f_weight, &self.ln_f_bias);

        // 4. Language model head (project to vocab)
        let logits = self.lm_head(&hidden);

        logits
    }
}

fn argmax(logits: &[f32]) -> u16 {
    logits.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx as u16)
        .unwrap_or(0)
}
```

### M3.3: Update `llm::basic`

**Modify**: `crates/kernel/src/llm/basic.rs`

**Changes**:

```rust
// At top of file:
use crate::llm::backend::{LlmBackend, StubBackend, TransformerBackend};

static BACKEND: Mutex<Option<Box<dyn LlmBackend>>> = Mutex::new(None);

pub fn init_backend(use_real: bool) {
    let mut backend = BACKEND.lock();
    *backend = if use_real {
        Some(Box::new(TransformerBackend::new()))
    } else {
        Some(Box::new(StubBackend))
    };
}

// Modify infer() function:
pub fn infer(prompt: &str, max_tokens: Option<usize>) -> LlmResult {
    let mut backend = BACKEND.lock();

    if let Some(ref mut backend) = *backend {
        // Use backend (stub or real)
        return backend.infer(prompt, max_tokens.unwrap_or(64));
    }

    // Fallback to current stub
    // ... (current implementation)
}
```

**Feature Flag**:

Add to `Cargo.toml`:
```toml
[features]
llm = []
llm-transformer = ["llm", "half"]  # NEW
```

Enable real transformer:
```rust
#[cfg(feature = "llm-transformer")]
pub fn init() {
    init_backend(true); // Use real transformer
}

#[cfg(all(feature = "llm", not(feature = "llm-transformer")))]
pub fn init() {
    init_backend(false); // Use stub
}
```

### M3 Deliverables

- [ ] Backend abstraction working
- [ ] Transformer backend integrated
- [ ] Inference loop functional
- [ ] Feature flag toggles backend
- [ ] Backward compatibility (stub still works)

---

## Milestone 4: Optimization

**Duration**: 3 weeks
**Goal**: Production-level performance

### M4.1: SIMD Vectorization

**Target**: ARM NEON intrinsics for aarch64

```rust
//! SIMD-optimized operations

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[cfg(target_arch = "aarch64")]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let len = a.len();

    unsafe {
        let mut sum = vdupq_n_f32(0.0);
        let mut i = 0;

        // Process 4 elements at a time
        while i + 4 <= len {
            let va = vld1q_f32(a.as_ptr().add(i));
            let vb = vld1q_f32(b.as_ptr().add(i));
            let prod = vmulq_f32(va, vb);
            sum = vaddq_f32(sum, prod);
            i += 4;
        }

        // Horizontal sum
        let sum_arr: [f32; 4] = core::mem::transmute(sum);
        let mut total = sum_arr.iter().sum();

        // Handle remainder
        while i < len {
            total += a[i] * b[i];
            i += 1;
        }

        total
    }
}
```

**Performance Target**: 2-4x speedup for matmul

### M4.2: KV Cache

**Problem**: Recomputing attention for all tokens is wasteful

**Solution**: Cache key/value projections

```rust
pub struct KVCache {
    /// Cached keys: [n_layer, n_ctx, n_embd]
    keys: Vec<Vec<Vec<f32>>>,
    /// Cached values: [n_layer, n_ctx, n_embd]
    values: Vec<Vec<Vec<f32>>>,
    /// Current sequence position
    seq_pos: usize,
}

impl KVCache {
    pub fn new(n_layer: usize, n_ctx: usize, n_embd: usize) -> Self {
        let mut keys = Vec::new();
        let mut values = Vec::new();

        for _ in 0..n_layer {
            keys.push(vec![vec![0f32; n_embd]; n_ctx]);
            values.push(vec![vec![0f32; n_embd]; n_ctx]);
        }

        Self { keys, values, seq_pos: 0 }
    }

    pub fn update(&mut self, layer: usize, k: Vec<f32>, v: Vec<f32>) {
        self.keys[layer][self.seq_pos] = k;
        self.values[layer][self.seq_pos] = v;
    }

    pub fn get(&self, layer: usize) -> (&[Vec<f32>], &[Vec<f32>]) {
        let k = &self.keys[layer][..self.seq_pos + 1];
        let v = &self.values[layer][..self.seq_pos + 1];
        (k, v)
    }
}
```

**Memory Cost**: ~2 MB for 512 context length

### M4.3: Optimized Quantization

**Goal**: Faster dequantization

```rust
// Batch dequantization with SIMD
pub fn dequantize_q4_0_fast(blocks: &[Q4_0Block], output: &mut [f32]) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // Use NEON intrinsics
        // Process 4 blocks at a time
        // ~4x faster than scalar
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        // Fallback to scalar
        dequantize_q4_0(blocks, output)
    }
}
```

### M4.4: Streaming Optimizations

**Goal**: Lower latency for first token

```rust
pub fn infer_stream_optimized(
    &mut self,
    prompt: &str,
    max_tokens: usize,
    callback: impl Fn(&str)
) {
    let tokens = self.tokenizer.encode(prompt);
    let mut output = tokens.clone();

    // Process prompt in parallel (if multi-core)
    let prompt_hidden = self.forward_batch(&tokens);

    // Stream tokens one at a time
    for i in 0..max_tokens {
        let logits = self.forward_incremental(&output, i);
        let next_token = argmax(&logits);
        output.push(next_token);

        // Decode and callback immediately
        let token_str = self.tokenizer.decode(&[next_token]);
        callback(&token_str);

        if next_token == EOS_TOKEN {
            break;
        }
    }
}
```

### M4 Deliverables

- [ ] SIMD matmul 2x faster
- [ ] KV cache working
- [ ] Streaming latency <100ms for first token
- [ ] Throughput >10 tokens/sec

---

## Milestone 5: Production Hardening

**Duration**: 2 weeks
**Goal**: Safety, error handling, monitoring

### M5.1: Error Handling

**Current Issues**: Many `unwrap()` calls in PoC code

**Fix**: Replace with proper error handling

```rust
#[derive(Debug)]
pub enum LlmError {
    ModelNotLoaded,
    TokenizationFailed,
    InferenceFailed,
    OutOfMemory,
    InvalidModel,
    // ...
}

impl LlmResult {
    pub fn error(msg: &'static str) -> Self {
        Self {
            infer_id: 0,
            tokens_emitted: 0,
            output: String::from(msg),
            latency_us: 0,
        }
    }
}
```

**Rules**:
- ❌ No `unwrap()` in production code
- ❌ No `expect()` unless truly impossible
- ✅ Return `Result<T, LlmError>`
- ✅ Log errors for debugging

### M5.2: Memory Safety Audit

**Tasks**:
1. Review all `unsafe` blocks
2. Add safety comments
3. Verify bounds checking
4. Test with memory sanitizer

**Example**:
```rust
pub fn get_tensor_data(&self, offset: usize, size: usize) -> &[u8] {
    // SAFETY: Bounds checked above, data is valid for 'static lifetime
    unsafe {
        core::slice::from_raw_parts(
            self.data.as_ptr().add(offset),
            size
        )
    }
}
```

### M5.3: Resource Limits

**Enforce hard limits**:

```rust
pub const MAX_PROMPT_LENGTH: usize = 2048;  // tokens
pub const MAX_GENERATION_LENGTH: usize = 512;
pub const MAX_CONCURRENT_INFERENCES: usize = 32;
pub const INFERENCE_TIMEOUT_MS: u64 = 30_000; // 30 seconds

pub fn infer_with_limits(
    prompt: &str,
    max_tokens: usize
) -> Result<LlmResult, LlmError> {
    // Check prompt length
    let tokens = tokenizer.encode(prompt);
    if tokens.len() > MAX_PROMPT_LENGTH {
        return Err(LlmError::PromptTooLong);
    }

    // Check generation length
    let max_gen = max_tokens.min(MAX_GENERATION_LENGTH);

    // Check concurrent limit
    if active_inferences() >= MAX_CONCURRENT_INFERENCES {
        return Err(LlmError::TooManyConcurrent);
    }

    // Run with timeout
    timeout(INFERENCE_TIMEOUT_MS, || {
        infer_internal(prompt, max_gen)
    })
}
```

### M5.4: Monitoring & Metrics

**Add detailed metrics**:

```rust
pub struct LlmMetrics {
    pub total_inferences: AtomicU64,
    pub total_tokens_generated: AtomicU64,
    pub total_inference_time_us: AtomicU64,
    pub avg_tokens_per_second: f32,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub oom_count: AtomicU64,
    pub timeout_count: AtomicU64,
}

pub fn get_metrics() -> LlmMetrics {
    METRICS.lock().clone()
}
```

**Shell command**: `llmstats` (already exists, enhance it)

### M5 Deliverables

- [ ] All error paths tested
- [ ] No unsafe UB (verified with Miri/ASAN)
- [ ] Resource limits enforced
- [ ] Comprehensive metrics
- [ ] Production-ready error messages

---

## Milestone 6: Testing & Validation

**Duration**: 2 weeks
**Goal**: Comprehensive test coverage

### M6.1: Unit Tests

**File**: `crates/kernel/src/llm/tests.rs` (NEW)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_roundtrip() {
        let tokenizer = BpeTokenizer::new();
        let text = "Hello, world!";
        let tokens = tokenizer.encode(text);
        let decoded = tokenizer.decode(&tokens);
        assert_eq!(text, decoded);
    }

    #[test]
    fn test_q4_0_dequantization() {
        // Test known values
        let block = Q4_0Block {
            scale: half::f16::from_f32(0.5).to_bits(),
            quants: [0x01, 0x23, /* ... */],
        };
        let value = block.dequant(0);
        assert!((value - (-3.5)).abs() < 1e-5);
    }

    #[test]
    fn test_attention_output_shape() {
        let layer = TransformerLayer::new(/* ... */);
        let input = vec![0.1; 384];
        let output = layer.attention(&input, &config);
        assert_eq!(output.len(), 384);
    }
}
```

**Coverage Target**: >80%

### M6.2: Integration Tests

**File**: `crates/testing/src/llm_tests.rs` (NEW)

```rust
//! Integration tests for LLM subsystem
//!
//! Tests full inference pipeline in QEMU

pub struct LlmIntegrationTests {
    kernel_interface: KernelCommandInterface,
}

impl LlmIntegrationTests {
    pub async fn test_basic_inference(&mut self) -> Result<bool, Box<dyn Error>> {
        log::info!("Testing basic LLM inference");

        // Load tiny model
        let output = self.kernel_interface
            .execute_command("llmctl load /models/tiny-10m.gguf")
            .await?;

        assert!(output.raw_output.contains("Model loaded"));

        // Run inference
        let output = self.kernel_interface
            .execute_command("llminfer \"Hello\" --max-tokens 5")
            .await?;

        // Check output contains tokens
        assert!(output.raw_output.len() > 0);

        Ok(true)
    }

    pub async fn test_memory_limits(&mut self) -> Result<bool, Box<dyn Error>> {
        // Test that inference respects memory bounds
        // ...
    }

    pub async fn test_concurrent_inferences(&mut self) -> Result<bool, Box<dyn Error>> {
        // Test multiple concurrent inferences
        // ...
    }
}
```

**Add to Phase 9 test suite**

### M6.3: Benchmarks

**File**: `crates/kernel/src/llm/benches.rs` (NEW)

```rust
//! Performance benchmarks

pub fn bench_tokenization() {
    let text = "The quick brown fox jumps over the lazy dog";
    let start = now_cycles();
    let tokens = tokenizer.encode(text);
    let end = now_cycles();

    log::info!("Tokenization: {} cycles", end - start);
}

pub fn bench_single_layer_forward() {
    let input = vec![0.1; 384];
    let start = now_cycles();
    let output = layer.forward(&input, &config);
    let end = now_cycles();

    log::info!("Layer forward: {} cycles", end - start);
}

pub fn bench_full_inference() {
    let prompt = "Once upon a time";
    let start = now_cycles();
    let result = infer(prompt, Some(10));
    let end = now_cycles();

    log::info!("Full inference (10 tokens): {} cycles", end - start);
    log::info!("Tokens/sec: {}", 10.0 / cycles_to_sec(end - start));
}
```

**Performance Targets**:
- Tokenization: <10µs
- Single layer: <500µs
- 10 tokens: <2 seconds
- Throughput: >5 tokens/sec

### M6.4: Correctness Validation

**Test against reference implementation**:

```python
# Reference: Python + PyTorch
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM

model = AutoModelForCausalLM.from_pretrained("tiny-10m")
tokenizer = AutoTokenizer.from_pretrained("tiny-10m")

prompt = "Hello"
inputs = tokenizer(prompt, return_tensors="pt")
outputs = model.generate(**inputs, max_length=10)
reference_text = tokenizer.decode(outputs[0])

print(f"Reference: {reference_text}")
```

Compare kernel output:
```
sis> llminfer "Hello" --max-tokens 5
Output: <kernel output>
```

**Acceptance**: Tokens should match reference (greedy decoding)

### M6 Deliverables

- [ ] 50+ unit tests passing
- [ ] Integration tests in Phase 9 suite
- [ ] Benchmarks showing >5 tokens/sec
- [ ] Output matches reference implementation

---

## Technical Decisions

### Decision 1: Model Format

**Options Considered**:
1. GGUF (llama.cpp)
2. Safetensors (Hugging Face)
3. Custom binary format

**Decision**: **GGUF**

**Rationale**:
- Widely supported (llama.cpp ecosystem)
- Optimized for quantization
- Well-documented format
- Can convert from HuggingFace with `llama.cpp/convert.py`

### Decision 2: Quantization Scheme

**Options**:
1. Q4_0 (4-bit, block scale)
2. Q8_0 (8-bit, block scale)
3. F16 (half precision)
4. F32 (full precision)

**Decision**: **Q4_0** for model weights, **F32** for activations

**Rationale**:
- Q4_0 gives 8x compression with minimal quality loss
- Enables larger models in same memory budget
- F32 activations maintain numerical stability

### Decision 3: Context Length

**Options**: 128, 256, 512, 1024, 2048

**Decision**: **512 tokens**

**Rationale**:
- Balances capability vs memory
- 512 tokens ~2KB prompt
- KV cache: ~2 MB (manageable)

### Decision 4: Concurrent Inferences

**Decision**: **32 concurrent sessions** (matches current stub)

**Rationale**:
- Realistic for kernel workloads
- Each session: ~64KB metadata
- Total: ~2 MB overhead

---

## Risk Mitigation

### Risk 1: Memory Exhaustion

**Risk**: Model weights + activations exceed available memory

**Mitigation**:
- Use static 8MB arena (bounded)
- Memory-map weights (don't copy)
- Dequantize on-the-fly
- Enforce hard limits
- Monitor high water mark

**Fallback**: Load smaller model (5-10M params instead of 50M)

### Risk 2: Performance Too Slow

**Risk**: Inference too slow for practical use (<1 token/sec)

**Mitigation**:
- SIMD optimization (NEON)
- KV cache
- Batch processing
- Profile hot paths

**Fallback**: Use smaller model, reduce context length

### Risk 3: Numerical Instability

**Risk**: Quantization errors compound, producing garbage

**Mitigation**:
- Validate against reference (PyTorch)
- Use F32 for activations
- Careful handling of softmax (overflow)
- Add epsilon to layer norm

**Fallback**: Use Q8_0 or F16 if Q4_0 unstable

### Risk 4: Integration Complexity

**Risk**: Breaking existing LLM infrastructure

**Mitigation**:
- Backend abstraction (swap stub/real)
- Feature flags (`llm-transformer`)
- Comprehensive tests
- Backward compatibility

**Fallback**: Keep stub as default, real transformer opt-in

### Risk 5: Model Availability

**Risk**: Can't find/train suitable 10-50M param model

**Mitigation**:
- Use GPT-2 small (117M) pruned to 50M
- Train tiny model from scratch
- Use DistilGPT-2 (82M params)

**Fallback**: Document as "framework ready for models"

---

## Success Metrics

### Minimum Viable Product (MVP)

**Must Have** (P0):
- [ ] Load GGUF model <50 MB
- [ ] Tokenize text (BPE)
- [ ] Generate 10 tokens
- [ ] Memory usage <8 MB total
- [ ] No crashes, no UB
- [ ] Feature flag works

**Nice to Have** (P1):
- [ ] Streaming inference
- [ ] KV cache working
- [ ] >5 tokens/sec
- [ ] SIMD optimizations
- [ ] Multiple concurrent inferences

### Performance Targets

| Metric | Target | Stretch |
|--------|--------|---------|
| **Throughput** | >5 tokens/sec | >10 tokens/sec |
| **Latency (first token)** | <500ms | <200ms |
| **Memory (total)** | <8 MB | <6 MB |
| **Concurrent sessions** | 32 | 64 |
| **Context length** | 512 | 1024 |
| **Model size** | 10-50M params | 50-100M params |

### Quality Metrics

| Metric | Target |
|--------|--------|
| **Correctness** | Output matches reference (greedy) |
| **Test coverage** | >80% |
| **Memory safety** | 0 UB (Miri clean) |
| **Error handling** | All paths tested |
| **Documentation** | Every public function |

---

## Timeline Summary

```
Week 1-2:   M0 - Research & Design
Week 3-5:   M1 - Minimal Transformer Core
Week 6-7:   M2 - Model Loading (GGUF)
Week 8-9:   M3 - Integration
Week 10-12: M4 - Optimization
Week 13-14: M5 - Production Hardening
Week 15-16: M6 - Testing & Validation

Total: 16 weeks (4 months)
```

### Critical Path

```
M0 → M1.1 (Arena) → M1.2 (Tokenizer) → M2.1 (GGUF) → M3.2 (Inference Loop) → M6
```

### Parallel Tracks

- **Track 1** (Core): M0 → M1 → M2 → M3
- **Track 2** (Optimization): M4 (starts after M1)
- **Track 3** (Testing): M6 (continuous)

---

## Deliverables

### Code Deliverables

1. **New Files**:
   - `crates/kernel/src/llm/arena.rs`
   - `crates/kernel/src/llm/tokenizer.rs`
   - `crates/kernel/src/llm/quantize.rs`
   - `crates/kernel/src/llm/transformer.rs`
   - `crates/kernel/src/llm/gguf.rs`
   - `crates/kernel/src/llm/backend.rs`
   - `crates/kernel/src/llm/generate.rs`
   - `crates/kernel/src/llm/tests.rs`
   - `crates/kernel/src/llm/benches.rs`
   - `crates/testing/src/llm_tests.rs`

2. **Modified Files**:
   - `crates/kernel/src/llm/mod.rs` (add new modules)
   - `crates/kernel/src/llm/basic.rs` (backend abstraction)
   - `crates/kernel/Cargo.toml` (new dependencies)

3. **New Dependencies**:
   - `half` (f16 support, no_std)
   - `libm` (math functions, no_std)

### Documentation Deliverables

1. **Technical Docs**:
   - `docs/llm/ARCHITECTURE.md`
   - `docs/llm/GGUF_FORMAT.md`
   - `docs/llm/QUANTIZATION.md`
   - `docs/llm/PERFORMANCE.md`

2. **User Docs**:
   - Update `docs/testing/WEEK4_CLOUD_GATEWAY_TESTS.md`
   - Add `docs/llm/USER_GUIDE.md`
   - Add `docs/llm/MODEL_CONVERSION.md`

3. **Wiki Pages** (when wiki created):
   - "LLM Subsystem Overview"
   - "Loading Custom Models"
   - "Performance Tuning"
   - "Troubleshooting"

---

## Next Steps (Immediate)

### Week 1 Action Items

1. **Day 1-2**: Technology stack approval
   - Review this plan with team
   - Approve GGUF + custom transformer approach
   - Set up development branch: `feature/llm-transformer`

2. **Day 3-5**: PoC Development
   - Create `llm-poc` crate (userspace)
   - Implement basic tokenizer
   - Test BPE encoding/decoding

3. **Day 6-7**: GGUF Research
   - Study llama.cpp GGUF implementation
   - Write GGUF parser (userspace)
   - Test loading tiny model

4. **Day 8-10**: Quantization PoC
   - Implement Q4_0 dequantization
   - Benchmark performance
   - Validate correctness

### Decision Points

**End of Week 2** (M0 Complete):
- ✅ GO: PoC works, proceed to M1
- ❌ NO-GO: Adjust approach (use simpler model, different format)

**End of Week 5** (M1 Complete):
- ✅ GO: Transformer layer works, proceed to M2
- ❌ NO-GO: Simplify model architecture

**End of Week 9** (M3 Complete):
- ✅ GO: Integration successful, proceed to optimization
- ❌ NO-GO: Keep as experimental feature

---

## Conclusion

This plan provides a **structured, risk-mitigated path** to completing the LLM subsystem. By following these milestones, SIS Kernel will have:

1. **Real transformer inference** in kernel space
2. **Production-quality implementation** (safe, tested, monitored)
3. **Backward compatibility** (stub still available)
4. **Honest documentation** (framework → production transition)

**Timeline**: 16 weeks (4 months)
**Effort**: 1 FTE (or distributed across contributors)
**Risk**: Medium (well-defined scope, proven algorithms)

**Success**: SIS becomes the **first AI-native OS kernel with real LLM inference in kernel space**.

---

**Last Updated**: 2025-01-17
**Status**: ✅ READY FOR REVIEW
**Next**: Team approval → Start M0
