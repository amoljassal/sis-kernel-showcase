# GGUF Model File Format

**Version**: 3 (Current)
**Specification**: Based on llama.cpp GGUF format
**Purpose**: Efficient storage and loading of quantized language models

---

## Table of Contents

1. [Overview](#overview)
2. [File Structure](#file-structure)
3. [Data Types](#data-types)
4. [Metadata Format](#metadata-format)
5. [Tensor Format](#tensor-format)
6. [Quantization Types](#quantization-types)
7. [Example](#example)
8. [Tooling](#tooling)

---

## Overview

GGUF (GPT-Generated Unified Format) is a binary file format designed for storing and loading quantized language models. It is the successor to GGML format and offers several improvements:

- **Self-Describing**: Contains all metadata needed to load the model
- **Quantization-First**: Optimized for Q4, Q8 compressed weights
- **Memory-Mappable**: Can be directly mapped into memory
- **Extensible**: Easy to add new metadata fields
- **Tool Support**: Wide ecosystem (llama.cpp, transformers, etc.)

---

## File Structure

```
┌──────────────────────────────────────────────────────────┐
│  Header                                      20 bytes    │
│  ┌────────────────────────────────────────────────────┐  │
│  │ magic:      u32  (0x46554747 = "GGUF")            │  │
│  │ version:    u32  (3)                               │  │
│  │ n_tensors:  u64  (number of tensors)               │  │
│  │ n_kv:       u64  (number of metadata entries)      │  │
│  └────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────┤
│  Metadata (Key-Value Pairs)                 Variable    │
│  ┌────────────────────────────────────────────────────┐  │
│  │ For each metadata entry:                           │  │
│  │   key_length:  u64                                 │  │
│  │   key:         [u8; key_length]  (UTF-8 string)    │  │
│  │   value_type:  u32                                 │  │
│  │   value:       <varies by type>                    │  │
│  └────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────┤
│  Tensor Info                                 Variable    │
│  ┌────────────────────────────────────────────────────┐  │
│  │ For each tensor:                                   │  │
│  │   name_length: u64                                 │  │
│  │   name:        [u8; name_length]  (UTF-8 string)   │  │
│  │   n_dims:      u32                                 │  │
│  │   dims:        [u64; n_dims]                       │  │
│  │   type:        u32  (Q4_0, Q8_0, F32, etc.)        │  │
│  │   offset:      u64  (byte offset from data start)  │  │
│  └────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────┤
│  Padding (32-byte alignment)                 0-31 bytes │
├──────────────────────────────────────────────────────────┤
│  Tensor Data                                 Variable    │
│  ┌────────────────────────────────────────────────────┐  │
│  │ Raw binary data in format specified by tensor type │  │
│  │ Stored in row-major order                          │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

## Data Types

### Primitive Types

| Type ID | Name | Size | Description |
|---------|------|------|-------------|
| 0 | U8 | 1 | Unsigned 8-bit integer |
| 1 | I8 | 1 | Signed 8-bit integer |
| 2 | U16 | 2 | Unsigned 16-bit integer |
| 3 | I16 | 2 | Signed 16-bit integer |
| 4 | U32 | 4 | Unsigned 32-bit integer |
| 5 | I32 | 4 | Signed 32-bit integer |
| 6 | F32 | 4 | IEEE 754 single precision |
| 7 | BOOL | 1 | Boolean (0 or 1) |
| 8 | STRING | Variable | UTF-8 string (length-prefixed) |
| 9 | ARRAY | Variable | Array of values |
| 10 | U64 | 8 | Unsigned 64-bit integer |
| 11 | I64 | 8 | Signed 64-bit integer |
| 12 | F64 | 8 | IEEE 754 double precision |

### Tensor Types

| Type ID | Name | Block Size | Block Bytes | Compression |
|---------|------|------------|-------------|-------------|
| 0 | F32 | 1 | 4 | 1x |
| 1 | F16 | 1 | 2 | 2x |
| 2 | Q4_0 | 32 | 18 | ~8x |
| 3 | Q4_1 | 32 | 20 | ~7x |
| 6 | Q5_0 | 32 | 22 | ~6x |
| 7 | Q5_1 | 32 | 24 | ~5x |
| 8 | Q8_0 | 32 | 34 | ~4x |
| 9 | Q8_1 | 32 | 40 | ~3x |

---

## Metadata Format

### Common Metadata Keys

| Key | Type | Description |
|-----|------|-------------|
| `general.name` | STRING | Model name |
| `general.author` | STRING | Model author/organization |
| `general.version` | STRING | Model version |
| `general.license` | STRING | Model license |
| `llm.vocab_size` | U32 | Vocabulary size |
| `llm.context_length` | U32 | Maximum context length |
| `llm.embedding_length` | U32 | Embedding dimension |
| `llm.head_count` | U32 | Number of attention heads |
| `llm.layer_count` | U32 | Number of transformer layers |
| `llm.feed_forward_length` | U32 | Feed-forward hidden dimension |
| `tokenizer.ggml.model` | STRING | Tokenizer type (e.g., "gpt2") |
| `tokenizer.ggml.tokens` | ARRAY | Token vocabulary |
| `tokenizer.ggml.scores` | ARRAY | Token scores |
| `tokenizer.ggml.token_type` | ARRAY | Token types |

### Example Metadata

```
llm.vocab_size = 32000 (U32)
llm.context_length = 2048 (U32)
llm.embedding_length = 384 (U32)
llm.head_count = 6 (U32)
llm.layer_count = 6 (U32)
general.name = "tiny-llama-10m" (STRING)
```

---

## Tensor Format

### Tensor Naming Convention

Tensors follow a hierarchical naming convention:

```
<module>.<layer>.<parameter>

Examples:
  token_embd.weight           - Token embedding weights
  blk.0.attn_q.weight         - Layer 0, Q projection
  blk.0.attn_k.weight         - Layer 0, K projection
  blk.0.attn_v.weight         - Layer 0, V projection
  blk.0.attn_output.weight    - Layer 0, output projection
  blk.0.ffn_up.weight         - Layer 0, FFN up projection
  blk.0.ffn_down.weight       - Layer 0, FFN down projection
  blk.0.attn_norm.weight      - Layer 0, attention layer norm
  blk.0.ffn_norm.weight       - Layer 0, FFN layer norm
  output_norm.weight          - Final layer norm
  output.weight               - LM head (language model output)
```

### Tensor Shapes

Tensors are stored in row-major order with dimensions specified in the tensor info.

**Example**:
```
Tensor: "token_embd.weight"
Dims: [32000, 384]
Type: Q4_0
Interpretation: 32000 tokens × 384 embedding dimension
```

---

## Quantization Types

### Q4_0: 4-bit Symmetric Quantization

**Block Structure** (32 values = 18 bytes):
```
┌─────────────┬──────────────────────────────┐
│ Scale (f16) │ Quants (16 bytes = 32×4-bit) │
│   2 bytes   │          16 bytes            │
└─────────────┴──────────────────────────────┘
```

**Encoding**:
```
scale = max(abs(values)) / 7.5
for each value:
    quant = clamp(round(value / scale) + 8, 0, 15)
```

**Decoding**:
```
for i in 0..32:
    byte_idx = i / 2
    nibble = if i % 2 == 0:
        quants[byte_idx] & 0x0F
    else:
        quants[byte_idx] >> 4

    signed = (nibble as i8) - 8
    value = signed * scale
```

**Properties**:
- **Range**: [-8, 7] × scale
- **Precision**: ~0.125 × scale
- **Compression**: 8x
- **Accuracy Loss**: <2% for most models

### Q8_0: 8-bit Symmetric Quantization

**Block Structure** (32 values = 34 bytes):
```
┌─────────────┬──────────────────┐
│ Scale (f16) │ Quants (32×i8)   │
│   2 bytes   │    32 bytes      │
└─────────────┴──────────────────┘
```

**Properties**:
- **Range**: [-128, 127] × scale
- **Precision**: ~0.008 × scale
- **Compression**: 4x
- **Accuracy Loss**: <0.5%

---

## Example

### Sample GGUF File (Tiny Model)

```
┌─────────────────────────────────────────────┐
│ Header                                      │
├─────────────────────────────────────────────┤
│ magic:      0x46554747                      │
│ version:    3                               │
│ n_tensors:  50                              │
│ n_kv:       10                              │
├─────────────────────────────────────────────┤
│ Metadata                                    │
├─────────────────────────────────────────────┤
│ "llm.vocab_size" = 32000 (U32)              │
│ "llm.context_length" = 512 (U32)            │
│ "llm.embedding_length" = 384 (U32)          │
│ "llm.head_count" = 6 (U32)                  │
│ "llm.layer_count" = 6 (U32)                 │
│ ...                                         │
├─────────────────────────────────────────────┤
│ Tensor Info                                 │
├─────────────────────────────────────────────┤
│ "token_embd.weight"                         │
│   dims: [32000, 384]                        │
│   type: Q4_0                                │
│   offset: 0                                 │
│ "blk.0.attn_q.weight"                       │
│   dims: [384, 384]                          │
│   type: Q4_0                                │
│   offset: 2211840                           │
│ ...                                         │
├─────────────────────────────────────────────┤
│ Tensor Data (starts at 32-byte boundary)   │
├─────────────────────────────────────────────┤
│ <binary data>                               │
└─────────────────────────────────────────────┘
```

---

## Tooling

### Converting Models to GGUF

#### From Hugging Face

```bash
# Using llama.cpp converter
python convert.py path/to/model --outfile model.gguf --outtype q4_0

# Options:
#   --outtype f32    - Full precision
#   --outtype f16    - Half precision
#   --outtype q4_0   - 4-bit quantization (default)
#   --outtype q8_0   - 8-bit quantization
```

#### From PyTorch

```python
import gguf

# Create writer
writer = gguf.GGUFWriter("model.gguf", "llama")

# Add metadata
writer.add_name("tiny-model")
writer.add_vocab_size(32000)
writer.add_context_length(512)
writer.add_embedding_length(384)

# Add tensors
writer.add_tensor("token_embd.weight", embedding_weights, gguf.GGMLQuantizationType.Q4_0)
writer.add_tensor("blk.0.attn_q.weight", attn_q_weights, gguf.GGMLQuantizationType.Q4_0)

# Write file
writer.write_header_to_file()
writer.write_kv_data_to_file()
writer.write_tensors_to_file()
writer.close()
```

### Inspecting GGUF Files

```bash
# Using gguf-dump (from llama.cpp)
gguf-dump model.gguf

# Output:
#   Magic: GGUF
#   Version: 3
#   Tensor count: 50
#   Metadata count: 10
#
#   Metadata:
#     llm.vocab_size: 32000
#     llm.context_length: 512
#     ...
#
#   Tensors:
#     token_embd.weight: [32000, 384] Q4_0
#     blk.0.attn_q.weight: [384, 384] Q4_0
#     ...
```

### Loading in SIS Kernel

```rust
use crate::llm::gguf::GgufModel;

// Read from VFS
let data = vfs::read_file("/models/tiny.gguf")?;

// Parse GGUF
let model = GgufModel::from_bytes(&data)?;

// Access metadata
let vocab_size = model.get_u32("llm.vocab_size")?;

// Access tensors
let embedding_weights = model.get_tensor("token_embd.weight")?;
```

---

## Best Practices

### Model Preparation

1. **Choose Quantization Level**:
   - Q4_0: Best compression, minor accuracy loss
   - Q8_0: Better accuracy, moderate compression
   - F16: High accuracy, 2x compression

2. **Validate Model**:
   - Check tensor shapes match expected architecture
   - Verify vocabulary size matches tokenizer
   - Test inference quality after quantization

3. **Metadata Completeness**:
   - Include all required metadata fields
   - Add model card information (author, license)
   - Document quantization settings

### Loading Optimization

1. **Memory Mapping**:
   - Use `mmap()` to avoid loading entire file
   - Map tensor data sections on-demand

2. **Lazy Loading**:
   - Load metadata first
   - Load tensors only when needed

3. **Caching**:
   - Cache frequently used tensors
   - Dequantize weights on first use

---

## References

1. [GGUF Specification (llama.cpp)](https://github.com/ggerganov/llama.cpp/blob/master/docs/gguf.md)
2. [GGML Quantization](https://github.com/ggerganov/llama.cpp/blob/master/docs/quantization.md)
3. [gguf-py Python Library](https://github.com/ggerganov/llama.cpp/tree/master/gguf-py)

---

**Document Status**: ✅ Complete
**Last Updated**: 2025-01-17
**Maintainer**: SIS Kernel Team
