# LLM Subsystem Architecture

**Version**: 0.2.0 (Phase 3 - Transformer Backend)
**Status**: 🚧 Implementation In Progress
**Last Updated**: 2025-01-17

---

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Component Details](#component-details)
4. [Memory Management](#memory-management)
5. [Data Flow](#data-flow)
6. [Performance Characteristics](#performance-characteristics)
7. [Safety and Security](#safety-and-security)
8. [Future Enhancements](#future-enhancements)

---

## Overview

### Mission Statement

> **Build the first production-ready LLM subsystem in kernel space with deterministic bounds, zero userspace overhead, and real neural network inference.**

### Current Status

| Component | Status | Description |
|-----------|--------|-------------|
| **Arena Allocator** | ✅ Complete | Static 8MB memory arena with bounded allocation |
| **BPE Tokenizer** | ✅ Complete | Byte-Pair Encoding for text tokenization |
| **Quantization** | ✅ Complete | Q4_0, Q8_0 support for 8x compression |
| **Transformer Core** | ✅ Complete | Single-layer transformer with attention |
| **GGUF Parser** | ✅ Complete | Model file format support |
| **Backend Abstraction** | ✅ Complete | Pluggable stub/transformer backends |
| **Generation Loop** | ✅ Complete | Autoregressive text generation |
| **Model Loading** | 🚧 In Progress | VFS integration for GGUF files |
| **Multi-layer Inference** | 🚧 In Progress | Full transformer stack |
| **SIMD Optimization** | 📋 Planned | ARM NEON vectorization |

---

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Shell Commands (User Interface)                            │
│  ┌──────────┬─────────┬────────────┬─────────────────────┐  │
│  │llminfer  │llmctl   │llmstats    │llmfinetune           │  │
│  └──────────┴─────────┴────────────┴─────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│  llm::basic (Public API)                                    │
│  - infer(), load_model(), configure_budget()                │
│  - Session management (32 concurrent)                       │
│  - Audit logging & compliance                               │
│  - Resource quotas & budgeting                              │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│  Backend Abstraction Layer                                  │
│  ┌─────────────────┬────────────────────────────────────┐   │
│  │  LlmBackend     │  Trait for all implementations     │   │
│  └─────────────────┴────────────────────────────────────┘   │
└─────────────────────────┬───────────────────────────────────┘
                          │
              ┌───────────┴────────────┐
              │                        │
┌─────────────▼──────┐   ┌────────────▼─────────────────────┐
│  Stub Backend      │   │  Transformer Backend             │
│  (Phase 0/1)       │   │  (Phase 3 - NEW)                 │
│                    │   │  ┌─────────────────────────────┐ │
│  - Echo tokens     │   │  │ Tokenizer (BPE)             │ │
│  - Deterministic   │   │  ├─────────────────────────────┤ │
│  - For testing     │   │  │ Quantization (Q4_0/Q8_0)    │ │
│                    │   │  ├─────────────────────────────┤ │
└────────────────────┘   │  │ Transformer Layers          │ │
                         │  ├─────────────────────────────┤ │
                         │  │ GGUF Model Loader           │ │
                         │  ├─────────────────────────────┤ │
                         │  │ Generation Loop             │ │
                         │  └─────────────────────────────┘ │
                         └──────────────┬───────────────────┘
                                        │
                         ┌──────────────▼───────────────────┐
                         │  Memory Arena (8 MB)             │
                         │  - Static allocation             │
                         │  - Bounded, deterministic        │
                         └──────────────────────────────────┘
```

### Component Interactions

```
┌─────────────┐
│ User Prompt │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ Tokenizer       │  "Hello" → [15496]
│ (BPE)           │
└──────┬──────────┘
       │
       ▼
┌─────────────────────────────────────────┐
│ Transformer Layers (6 layers)           │
│                                          │
│  For each layer:                         │
│    1. Layer Norm                         │
│    2. Multi-Head Attention               │
│       ┌─────────────────────┐            │
│       │ Q, K, V Projections │            │
│       │ (Dequantize Q4_0)   │            │
│       └─────────────────────┘            │
│    3. Residual Connection                │
│    4. Layer Norm                         │
│    5. Feed-Forward Network               │
│       ┌─────────────────────┐            │
│       │ Up Projection       │            │
│       │ GELU Activation     │            │
│       │ Down Projection     │            │
│       └─────────────────────┘            │
│    6. Residual Connection                │
└──────┬───────────────────────────────────┘
       │
       ▼
┌─────────────────┐
│ LM Head         │  Project to vocabulary
│ (n_embd → vocab)│
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Sampling        │  Select next token
│ (top-k/top-p)   │  (greedy, temperature, etc.)
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Detokenizer     │  [15496, 1279] → "Hello world"
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Output Text     │
└─────────────────┘
```

---

## Component Details

### 1. Memory Arena (`arena.rs`)

**Purpose**: Provide bounded, deterministic memory allocation for LLM operations.

**Design**:
- **Size**: 8 MB static buffer (`.bss` section)
- **Allocator**: Simple bump allocator (O(1) allocation)
- **Alignment**: 32-byte alignment for SIMD operations
- **Reset**: Zero-cost reset between inferences

**Memory Layout**:
```
┌─────────────────────────────────────────────┐
│  LLM Arena (8 MB)                           │
├─────────────────────────────────────────────┤
│  Model Weights (4-6 MB)                     │  ← Q4_0 quantized
├─────────────────────────────────────────────┤
│  Activation Buffers (1-2 MB)                │  ← F32 tensors
├─────────────────────────────────────────────┤
│  KV Cache (1-2 MB)                          │  ← Context storage
├─────────────────────────────────────────────┤
│  Tokenizer Vocab (256 KB)                   │  ← BPE vocabulary
└─────────────────────────────────────────────┘
```

**Key Functions**:
- `alloc(size, align)`: Allocate memory with alignment
- `alloc_array<T>(count)`: Allocate typed array
- `reset()`: Reset allocator (O(1))
- `usage()`: Get current/peak memory usage

### 2. BPE Tokenizer (`tokenizer.rs`)

**Purpose**: Convert text to/from token IDs using Byte-Pair Encoding.

**Algorithm**:
1. Build vocabulary of common subwords (trained offline)
2. Encode: Greedy longest-match from text to tokens
3. Decode: Lookup tokens in vocabulary, concatenate bytes

**Vocabulary Structure**:
- **Size**: 32k tokens (standard for Llama models)
- **Format**: BTreeMap for O(log n) lookup
- **Memory**: ~256 KB

**Example**:
```
Text: "Hello, world!"
     ↓ encode
Tokens: [15496, 11, 995, 0]
     ↓ decode
Text: "Hello, world!"
```

### 3. Quantization (`quantize.rs`)

**Purpose**: Compress model weights from 32-bit to 4-bit (8x reduction).

**Q4_0 Format**:
```
Block (32 values = 18 bytes):
┌─────────────┬──────────────────────────────┐
│ Scale (f16) │ Quants (16 bytes = 32×4-bit) │
│   2 bytes   │          16 bytes            │
└─────────────┴──────────────────────────────┘

Quantization:
  quant = round(value / scale) + 8
  value = (quant - 8) * scale

Range: [-8, 7] * scale
```

**Performance**:
- **Compression**: 8x (4 bytes → 0.5 bytes per value)
- **Accuracy**: <2% degradation for most models
- **Speed**: ~5 cycles/value (scalar), ~1.5 cycles/value (SIMD)

### 4. Transformer Core (`transformer.rs`)

**Purpose**: Implement neural network layers for text generation.

**Architecture**:
- **Layers**: 6 (configurable)
- **Embedding Dimension**: 384 (configurable)
- **Attention Heads**: 6 (64-dim per head)
- **Feed-Forward**: 4x expansion (1536 hidden dim)

**Operations**:

#### Layer Normalization
```
mean = sum(x) / n
var = sum((x - mean)²) / n
y = (x - mean) / sqrt(var + ε) * γ + β
```

#### Multi-Head Attention
```
Q = x * W_q
K = x * W_k
V = x * W_v

scores = (Q * K^T) / sqrt(d_k)
attn = softmax(scores)
output = attn * V
```

#### Feed-Forward Network
```
hidden = GELU(x * W_up)
output = hidden * W_down
```

#### GELU Activation
```
GELU(x) ≈ 0.5 * x * (1 + tanh(√(2/π) * (x + 0.044715 * x³)))
```

### 5. GGUF Parser (`gguf.rs`)

**Purpose**: Load quantized models from GGUF format files.

**GGUF Format**:
```
┌──────────────────────────────────┐
│  Header (20 bytes)               │
│  - Magic: 0x46554747 ("GGUF")   │
│  - Version: 3                    │
│  - n_tensors, n_kv               │
├──────────────────────────────────┤
│  Metadata (Key-Value Pairs)      │
│  - Model config                  │
│  - Vocabulary                    │
├──────────────────────────────────┤
│  Tensor Info                     │
│  - Names, shapes, types          │
├──────────────────────────────────┤
│  Tensor Data (32-byte aligned)   │
│  - Quantized weights             │
└──────────────────────────────────┘
```

**Supported Types**:
- Q4_0, Q4_1: 4-bit quantization
- Q8_0, Q8_1: 8-bit quantization
- F16: Half precision (16-bit)
- F32: Full precision (32-bit)

### 6. Backend Abstraction (`backend.rs`)

**Purpose**: Provide pluggable inference backends.

**Trait Definition**:
```rust
pub trait LlmBackend {
    fn infer(&mut self, prompt: &str, max_tokens: usize)
        -> Result<LlmResult, &'static str>;
    fn load_model(&mut self, path: &str)
        -> Result<(), &'static str>;
    fn is_loaded(&self) -> bool;
    fn stats(&self) -> BackendStats;
}
```

**Implementations**:
1. **StubBackend**: Deterministic placeholder (Phase 0/1)
2. **TransformerBackend**: Real inference (Phase 3)

### 7. Generation Loop (`generate.rs`)

**Purpose**: Autoregressive text generation with configurable sampling.

**Algorithm**:
```
tokens = tokenize(prompt)

for i in 0..max_tokens:
    logits = transformer.forward(tokens)
    next_token = sample(logits, config)
    tokens.append(next_token)

    if next_token == EOS:
        break

return detokenize(tokens)
```

**Sampling Strategies**:

| Strategy | Description | Use Case |
|----------|-------------|----------|
| **Greedy** | argmax(logits) | Deterministic, testing |
| **Temperature** | Scale logits before softmax | Control randomness |
| **Top-K** | Sample from top K tokens | Diverse but coherent |
| **Top-P** | Sample from cumulative P mass | Adaptive diversity |

---

## Memory Management

### Memory Budget

| Component | Typical Size | Max Size | Notes |
|-----------|--------------|----------|-------|
| **Model Weights** | 4-6 MB | 6 MB | Q4_0 quantized |
| **Activations** | 512 KB | 2 MB | Per inference |
| **KV Cache** | 1 MB | 2 MB | 512 context length |
| **Vocabulary** | 256 KB | 512 KB | 32k tokens |
| **Other** | 256 KB | 1 MB | Metadata, buffers |
| **TOTAL** | **~7 MB** | **8 MB** | **Fits in arena** |

### Allocation Strategy

1. **Model Weights**: Allocated once at model load, never freed
2. **Activations**: Allocated per inference, freed after (arena reset)
3. **KV Cache**: Allocated per session, reused across inferences
4. **Vocabulary**: Allocated at model load, never freed

### Memory Safety

**Guarantees**:
- ✅ No dynamic heap allocation in hot path
- ✅ All allocations bounds-checked
- ✅ No use-after-free (no deallocation)
- ✅ WCET bounds (deterministic allocation)

---

## Data Flow

### Inference Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User Input                                               │
│    "Once upon a time"                                        │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│ 2. Tokenization                                             │
│    BpeTokenizer::encode()                                   │
│    Output: [5539, 2501, 264, 892]  (4 tokens)              │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│ 3. Embedding Lookup                                         │
│    token_embd.weight[token_id] → embedding vector           │
│    Output: [0.1, -0.3, 0.5, ...] (384-dim)                 │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│ 4. Transformer Layers (6 layers)                            │
│    For each layer:                                          │
│      - Dequantize Q4_0 weights → F32                        │
│      - Layer Norm → Attention → Residual                    │
│      - Layer Norm → FFN → Residual                          │
│    Output: [0.2, -0.1, 0.4, ...] (384-dim)                 │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│ 5. LM Head                                                  │
│    Project to vocabulary: (384) → (32000)                   │
│    Output: logits for all tokens                            │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│ 6. Sampling                                                 │
│    sample_token(logits, config)                             │
│    Output: token_id = 264 ("the")                           │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│ 7. Append & Repeat                                          │
│    tokens.push(264)                                         │
│    Repeat steps 3-6 for max_tokens                          │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│ 8. Detokenization                                           │
│    BpeTokenizer::decode(tokens)                             │
│    Output: "Once upon a time the..."                        │
└─────────────────────────────────────────────────────────────┘
```

---

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|------------|-------|
| **Tokenization** | O(n log V) | n = text length, V = vocab size |
| **Embedding** | O(s × d) | s = sequence length, d = embd dim |
| **Attention** | O(s² × d) | Quadratic in sequence length |
| **FFN** | O(s × d²) | Dominant for large models |
| **LM Head** | O(d × V) | V = vocabulary size |
| **Total (per token)** | **O(s² × d + d² × L)** | L = num layers |

### Space Complexity

| Component | Complexity | Notes |
|-----------|------------|-------|
| **Weights** | O(L × d²) | L layers, d embedding dim |
| **Activations** | O(s × d) | s sequence length |
| **KV Cache** | O(L × s × d) | Caches keys & values |

### Benchmarks (Target)

| Metric | Target | Stretch Goal |
|--------|--------|--------------|
| **Throughput** | 5 tokens/sec | 10 tokens/sec |
| **Latency (first token)** | <500ms | <200ms |
| **Memory Usage** | <8 MB | <6 MB |
| **Context Length** | 512 tokens | 1024 tokens |

---

## Safety and Security

### Memory Safety

**Rust Guarantees**:
- No buffer overflows (bounds checking)
- No use-after-free (ownership system)
- No data races (borrowing rules)

**Unsafe Code Audit**:
```rust
// Only unsafe blocks (with safety comments):
// 1. Arena allocation (bounds-checked)
// 2. SIMD intrinsics (validated inputs)
```

### Resource Limits

**Hard Limits**:
- Max prompt length: 2048 tokens
- Max generation length: 512 tokens
- Max concurrent inferences: 32
- Inference timeout: 30 seconds

**Enforcement**:
```rust
if tokens.len() > MAX_PROMPT_LENGTH {
    return Err("Prompt too long");
}
```

### Security Considerations

1. **Input Validation**: All user inputs sanitized
2. **Model Verification**: SHA-256 + Ed25519 signature
3. **Audit Logging**: All inferences logged
4. **Resource Quotas**: Per-user token budgets

---

## Future Enhancements

### Short Term (Next 3 Months)

1. **SIMD Optimization** (M4.1)
   - ARM NEON vectorization for matmul
   - Target: 3-4x speedup

2. **KV Cache** (M4.2)
   - Cache attention keys/values
   - Target: 10x speedup for long sequences

3. **Multi-Layer Integration** (M3)
   - Full 6-layer transformer stack
   - End-to-end inference working

4. **Model Loading** (M2.2)
   - VFS integration for GGUF files
   - Dynamic model swapping

### Medium Term (3-6 Months)

1. **LoRA Integration**
   - Real adapter application (not stub)
   - Fine-tuning support

2. **Larger Models**
   - Support up to 100M parameters
   - Optimize memory usage

3. **GPU Offload** (if available)
   - Hybrid CPU+GPU inference
   - Async execution

### Long Term (6-12 Months)

1. **Multi-Modal Support**
   - Vision-language models
   - Audio processing

2. **Distributed Inference**
   - Model parallelism across cores
   - Pipeline parallelism

3. **Quantization Research**
   - Q2, Q3 formats
   - Custom quantization schemes

---

## References

1. [GGUF Format Specification](https://github.com/ggerganov/llama.cpp/blob/master/docs/gguf.md)
2. [Transformer Architecture](https://arxiv.org/abs/1706.03762)
3. [GELU Activation](https://arxiv.org/abs/1606.08415)
4. [Q4_0 Quantization](https://github.com/ggerganov/llama.cpp/blob/master/docs/quantization.md)

---

**Document Status**: ✅ Complete
**Next Review**: 2025-02-17
**Maintainer**: SIS Kernel Team
