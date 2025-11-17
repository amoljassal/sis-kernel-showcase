//! GGUF Model File Format Parser
//!
//! # Overview
//!
//! This module implements a parser for the GGUF (GPT-Generated Unified Format)
//! file format used by llama.cpp and the broader LLM ecosystem. GGUF is designed
//! for efficient storage and loading of quantized language models.
//!
//! # GGUF Format Specification
//!
//! ## File Structure
//!
//! ```text
//! ┌──────────────────────────────────┐
//! │  Header                          │  20 bytes
//! ├──────────────────────────────────┤
//! │  Metadata (Key-Value Pairs)      │  Variable
//! ├──────────────────────────────────┤
//! │  Tensor Info                     │  Variable
//! ├──────────────────────────────────┤
//! │  Padding (32-byte alignment)     │  0-31 bytes
//! ├──────────────────────────────────┤
//! │  Tensor Data                     │  Variable
//! └──────────────────────────────────┘
//! ```
//!
//! ## Header Format
//!
//! ```text
//! Offset  Size  Type    Field
//! ───────────────────────────────
//! 0       4     u32     magic (0x46554747 = "GGUF")
//! 4       4     u32     version (3)
//! 8       8     u64     n_tensors
//! 16      8     u64     n_kv (metadata count)
//! ```
//!
//! ## Metadata Format
//!
//! Each metadata entry:
//! ```text
//! - key_length: u64
//! - key: [u8; key_length]  (UTF-8 string)
//! - value_type: u32
//! - value: <varies by type>
//! ```
//!
//! Value types:
//! - 0: u8
//! - 1: i8
//! - 2: u16
//! - 3: i16
//! - 4: u32
//! - 5: i32
//! - 6: f32
//! - 7: bool
//! - 8: string
//! - 9: array
//! - 10: u64
//! - 11: i64
//! - 12: f64
//!
//! ## Tensor Info Format
//!
//! Each tensor entry:
//! ```text
//! - name_length: u64
//! - name: [u8; name_length]  (UTF-8 string)
//! - n_dims: u32
//! - dims: [u64; n_dims]
//! - type: u32  (Q4_0=2, Q4_1=3, Q8_0=8, F32=0, F16=1)
//! - offset: u64  (byte offset from start of tensor data section)
//! ```
//!
//! ## Tensor Data Format
//!
//! - Aligned to 32-byte boundary
//! - Raw binary data in format specified by tensor type
//! - Stored in row-major order
//!
//! # Design Rationale
//!
//! **Why GGUF?**
//! - **Industry Standard**: Used by llama.cpp, widely supported
//! - **Quantization-First**: Designed for Q4_0, Q8_0, etc.
//! - **Metadata Rich**: Stores model hyperparameters
//! - **Memory Efficient**: Can be memory-mapped
//! - **Tool Support**: Many converters (HuggingFace → GGUF)
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::gguf::GgufModel;
//!
//! // Parse GGUF file
//! let data = /* read from VFS */;
//! let model = GgufModel::from_bytes(&data)?;
//!
//! // Access metadata
//! let vocab_size = model.get_u32("llm.vocab_size")?;
//!
//! // Access tensor
//! let embedding_weights = model.get_tensor("token_embd.weight")?;
//! ```
//!
//! # Safety Considerations
//!
//! - All parsing is bounds-checked
//! - No unsafe code in hot path
//! - Validates magic number and version
//! - Checks alignment requirements

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::format;
use core::convert::TryFrom;

/// GGUF magic number: "GGUF" as little-endian u32
pub const GGUF_MAGIC: u32 = 0x46554747;

/// GGUF format version (current: 3)
pub const GGUF_VERSION: u32 = 3;

/// Tensor data alignment (bytes)
pub const GGUF_ALIGNMENT: usize = 32;

/// GGUF tensor type IDs
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GgufType {
    F32 = 0,
    F16 = 1,
    Q4_0 = 2,
    Q4_1 = 3,
    Q5_0 = 6,
    Q5_1 = 7,
    Q8_0 = 8,
    Q8_1 = 9,
}

impl GgufType {
    /// Create from u32 type ID
    pub fn from_u32(value: u32) -> Result<Self, &'static str> {
        match value {
            0 => Ok(GgufType::F32),
            1 => Ok(GgufType::F16),
            2 => Ok(GgufType::Q4_0),
            3 => Ok(GgufType::Q4_1),
            6 => Ok(GgufType::Q5_0),
            7 => Ok(GgufType::Q5_1),
            8 => Ok(GgufType::Q8_0),
            9 => Ok(GgufType::Q8_1),
            _ => Err("Unknown GGUF type"),
        }
    }

    /// Get element size in bytes
    pub fn element_size(&self) -> usize {
        match self {
            GgufType::F32 => 4,
            GgufType::F16 => 2,
            GgufType::Q4_0 => 18,  // 32 values per block
            GgufType::Q4_1 => 20,
            GgufType::Q5_0 => 22,
            GgufType::Q5_1 => 24,
            GgufType::Q8_0 => 34,  // 32 values per block
            GgufType::Q8_1 => 40,
        }
    }

    /// Get block size (number of values per block for quantized types)
    pub fn block_size(&self) -> usize {
        match self {
            GgufType::F32 => 1,
            GgufType::F16 => 1,
            GgufType::Q4_0 | GgufType::Q4_1 |
            GgufType::Q5_0 | GgufType::Q5_1 |
            GgufType::Q8_0 | GgufType::Q8_1 => 32,
        }
    }
}

/// GGUF metadata value types
#[derive(Debug, Clone)]
pub enum GgufValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
    U64(u64),
    I64(i64),
    F64(f64),
    Array(Vec<GgufValue>),
}

impl GgufValue {
    /// Try to extract u32 value
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            GgufValue::U32(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract i32 value
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            GgufValue::I32(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract f32 value
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            GgufValue::F32(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract string value
    pub fn as_string(&self) -> Option<&str> {
        match self {
            GgufValue::String(s) => Some(s),
            _ => None,
        }
    }
}

/// GGUF tensor metadata
#[derive(Debug, Clone)]
pub struct GgufTensor {
    /// Tensor name (e.g., "token_embd.weight")
    pub name: String,

    /// Tensor dimensions [d0, d1, d2, ...]
    pub dims: Vec<u64>,

    /// Tensor data type
    pub tensor_type: GgufType,

    /// Byte offset from start of tensor data section
    pub offset: u64,

    /// Raw tensor data (populated after parsing)
    pub data: Vec<u8>,
}

impl GgufTensor {
    /// Calculate total number of elements
    pub fn element_count(&self) -> usize {
        self.dims.iter().product::<u64>() as usize
    }

    /// Calculate tensor size in bytes
    pub fn byte_size(&self) -> usize {
        let elements = self.element_count();
        let block_size = self.tensor_type.block_size();
        let num_blocks = (elements + block_size - 1) / block_size;
        num_blocks * self.tensor_type.element_size()
    }
}

/// GGUF model container
///
/// Represents a parsed GGUF model file with metadata and tensors.
pub struct GgufModel {
    /// Model metadata (key-value pairs)
    pub metadata: BTreeMap<String, GgufValue>,

    /// Model tensors
    pub tensors: BTreeMap<String, GgufTensor>,

    /// GGUF version
    pub version: u32,
}

impl GgufModel {
    /// Parse GGUF file from bytes
    ///
    /// # Arguments
    ///
    /// - `data`: Raw GGUF file bytes
    ///
    /// # Returns
    ///
    /// - `Ok(model)`: Successfully parsed model
    /// - `Err(msg)`: Parse error
    ///
    /// # Example
    ///
    /// ```no_run
    /// let data = vfs::read_file("/models/tiny.gguf")?;
    /// let model = GgufModel::from_bytes(&data)?;
    /// ```
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        let mut offset = 0;

        // Parse header
        let magic = read_u32(data, &mut offset)?;
        if magic != GGUF_MAGIC {
            return Err("Invalid GGUF magic number");
        }

        let version = read_u32(data, &mut offset)?;
        if version != GGUF_VERSION {
            return Err("Unsupported GGUF version");
        }

        let n_tensors = read_u64(data, &mut offset)? as usize;
        let n_kv = read_u64(data, &mut offset)? as usize;

        // Parse metadata
        let mut metadata = BTreeMap::new();
        for _ in 0..n_kv {
            let (key, value) = parse_metadata_entry(data, &mut offset)?;
            metadata.insert(key, value);
        }

        // Parse tensor info
        let mut tensors = BTreeMap::new();
        for _ in 0..n_tensors {
            let tensor = parse_tensor_info(data, &mut offset)?;
            tensors.insert(tensor.name.clone(), tensor);
        }

        // Align to tensor data section
        let align_mask = GGUF_ALIGNMENT - 1;
        offset = (offset + align_mask) & !align_mask;

        // Load tensor data
        for tensor in tensors.values_mut() {
            let data_offset = offset + tensor.offset as usize;
            let size = tensor.byte_size();

            if data_offset + size > data.len() {
                return Err("Truncated tensor data");
            }

            tensor.data = data[data_offset..data_offset + size].to_vec();
        }

        Ok(Self {
            metadata,
            tensors,
            version,
        })
    }

    /// Get metadata value by key
    ///
    /// # Arguments
    ///
    /// - `key`: Metadata key (e.g., "llm.vocab_size")
    ///
    /// # Returns
    ///
    /// - `Some(value)`: Metadata value if found
    /// - `None`: Key not found
    pub fn get_metadata(&self, key: &str) -> Option<&GgufValue> {
        self.metadata.get(key)
    }

    /// Get u32 metadata value
    pub fn get_u32(&self, key: &str) -> Result<u32, &'static str> {
        self.metadata
            .get(key)
            .and_then(|v| v.as_u32())
            .ok_or("Metadata key not found or wrong type")
    }

    /// Get string metadata value
    pub fn get_string(&self, key: &str) -> Result<&str, &'static str> {
        self.metadata
            .get(key)
            .and_then(|v| v.as_string())
            .ok_or("Metadata key not found or wrong type")
    }

    /// Get tensor by name
    ///
    /// # Arguments
    ///
    /// - `name`: Tensor name (e.g., "token_embd.weight")
    ///
    /// # Returns
    ///
    /// - `Some(tensor)`: Tensor if found
    /// - `None`: Tensor not found
    pub fn get_tensor(&self, name: &str) -> Option<&GgufTensor> {
        self.tensors.get(name)
    }

    /// List all tensor names
    pub fn tensor_names(&self) -> Vec<String> {
        self.tensors.keys().cloned().collect()
    }

    /// Get model statistics
    pub fn stats(&self) -> GgufStats {
        let param_count: usize = self.tensors.values()
            .map(|t| t.element_count())
            .sum();

        let total_size: usize = self.tensors.values()
            .map(|t| t.data.len())
            .sum();

        GgufStats {
            version: self.version,
            n_tensors: self.tensors.len(),
            n_metadata: self.metadata.len(),
            param_count,
            total_size,
        }
    }
}

/// GGUF model statistics
#[derive(Debug, Clone, Copy)]
pub struct GgufStats {
    pub version: u32,
    pub n_tensors: usize,
    pub n_metadata: usize,
    pub param_count: usize,
    pub total_size: usize,
}

/// Read u32 from buffer (little-endian)
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

/// Read u64 from buffer (little-endian)
fn read_u64(data: &[u8], offset: &mut usize) -> Result<u64, &'static str> {
    if *offset + 8 > data.len() {
        return Err("Buffer underflow");
    }

    let value = u64::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
        data[*offset + 4],
        data[*offset + 5],
        data[*offset + 6],
        data[*offset + 7],
    ]);

    *offset += 8;
    Ok(value)
}

/// Read f32 from buffer (little-endian)
fn read_f32(data: &[u8], offset: &mut usize) -> Result<f32, &'static str> {
    let bits = read_u32(data, offset)?;
    Ok(f32::from_bits(bits))
}

/// Read string from buffer
fn read_string(data: &[u8], offset: &mut usize) -> Result<String, &'static str> {
    let len = read_u64(data, offset)? as usize;

    if *offset + len > data.len() {
        return Err("Buffer underflow");
    }

    let bytes = &data[*offset..*offset + len];
    *offset += len;

    String::from_utf8(bytes.to_vec())
        .map_err(|_| "Invalid UTF-8 in string")
}

/// Parse metadata entry (key-value pair)
fn parse_metadata_entry(data: &[u8], offset: &mut usize) -> Result<(String, GgufValue), &'static str> {
    let key = read_string(data, offset)?;
    let value_type = read_u32(data, offset)?;

    let value = match value_type {
        4 => GgufValue::U32(read_u32(data, offset)?),
        5 => GgufValue::I32(read_u32(data, offset)? as i32),
        6 => GgufValue::F32(read_f32(data, offset)?),
        8 => GgufValue::String(read_string(data, offset)?),
        _ => return Err("Unsupported metadata type"),
    };

    Ok((key, value))
}

/// Parse tensor info entry
fn parse_tensor_info(data: &[u8], offset: &mut usize) -> Result<GgufTensor, &'static str> {
    let name = read_string(data, offset)?;
    let n_dims = read_u32(data, offset)? as usize;

    let mut dims = Vec::with_capacity(n_dims);
    for _ in 0..n_dims {
        dims.push(read_u64(data, offset)?);
    }

    let type_id = read_u32(data, offset)?;
    let tensor_type = GgufType::from_u32(type_id)?;

    let tensor_offset = read_u64(data, offset)?;

    Ok(GgufTensor {
        name,
        dims,
        tensor_type,
        offset: tensor_offset,
        data: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gguf_type_sizes() {
        assert_eq!(GgufType::F32.element_size(), 4);
        assert_eq!(GgufType::Q4_0.element_size(), 18);
        assert_eq!(GgufType::Q8_0.element_size(), 34);
    }

    #[test]
    fn test_gguf_type_block_sizes() {
        assert_eq!(GgufType::F32.block_size(), 1);
        assert_eq!(GgufType::Q4_0.block_size(), 32);
    }

    #[test]
    fn test_read_u32() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut offset = 0;
        let value = read_u32(&data, &mut offset).unwrap();
        assert_eq!(value, 0x04030201); // Little-endian
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_read_u64() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut offset = 0;
        let value = read_u64(&data, &mut offset).unwrap();
        assert_eq!(value, 0x0807060504030201); // Little-endian
        assert_eq!(offset, 8);
    }

    #[test]
    fn test_gguf_value() {
        let val = GgufValue::U32(42);
        assert_eq!(val.as_u32(), Some(42));
        assert_eq!(val.as_string(), None);
    }
}
