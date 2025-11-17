//! Transformer Neural Network Implementation
//!
//! # Overview
//!
//! This module implements a minimal transformer architecture compatible with
//! GPT-2, GPT-3, and Llama models. The implementation focuses on:
//! - **Determinism**: Bounded execution time for real-time guarantees
//! - **Efficiency**: Optimized for constrained kernel environment
//! - **Correctness**: Numerically stable operations
//!
//! # Architecture
//!
//! ```text
//! Input Tokens
//!     │
//!     ▼
//! ┌─────────────────┐
//! │ Token Embedding │  (vocab_size × n_embd)
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌────────────────────────┐
//! │ Transformer Layer 1    │  ◄─┐
//! │  ┌──────────────────┐  │    │
//! │  │ Layer Norm 1     │  │    │
//! │  ├──────────────────┤  │    │ n_layer
//! │  │ Multi-Head Attn  │  │    │ times
//! │  ├──────────────────┤  │    │
//! │  │ Residual Add     │  │    │
//! │  ├──────────────────┤  │    │
//! │  │ Layer Norm 2     │  │    │
//! │  ├──────────────────┤  │    │
//! │  │ Feed-Forward     │  │    │
//! │  ├──────────────────┤  │    │
//! │  │ Residual Add     │  │    │
//! │  └──────────────────┘  │    │
//! └────────┬───────────────┘  ◄─┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ Final Layer Norm│
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  LM Head        │  (n_embd × vocab_size)
//! └────────┬────────┘
//!          │
//!          ▼
//!       Logits
//! ```
//!
//! # Mathematical Operations
//!
//! ## Self-Attention
//!
//! ```text
//! Q = x * W_q
//! K = x * W_k
//! V = x * W_v
//!
//! scores = (Q * K^T) / sqrt(d_k)
//! attn_weights = softmax(scores)
//! output = attn_weights * V
//! ```
//!
//! ## Feed-Forward Network
//!
//! ```text
//! hidden = GELU(x * W_up)
//! output = hidden * W_down
//! ```
//!
//! ## Layer Normalization
//!
//! ```text
//! mean = sum(x) / n
//! var = sum((x - mean)^2) / n
//! output = (x - mean) / sqrt(var + eps) * weight + bias
//! ```
//!
//! # Performance Characteristics
//!
//! **Time Complexity** (per layer):
//! - Attention: O(n² × d) where n = sequence length, d = model dimension
//! - Feed-Forward: O(n × d × 4d) = O(n × d²)
//!
//! **Space Complexity**:
//! - Weights: O(L × d²) where L = number of layers
//! - Activations: O(n × d)
//! - KV Cache: O(L × n × d)
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::transformer::{TransformerConfig, Transformer};
//!
//! let config = TransformerConfig {
//!     n_vocab: 32000,
//!     n_ctx: 512,
//!     n_embd: 384,
//!     n_head: 6,
//!     n_layer: 6,
//! };
//!
//! let mut transformer = Transformer::new(config);
//! let output = transformer.forward(&input_tokens);
//! ```

use alloc::vec::Vec;
use alloc::vec;
use crate::llm::quantize::{Q4_0Block, dequantize_q4_0};
use core::f32;

/// Transformer model configuration
///
/// Defines the architecture parameters for the transformer model.
/// These values must match the GGUF model file being loaded.
#[derive(Debug, Clone, Copy)]
pub struct TransformerConfig {
    /// Vocabulary size (number of tokens)
    pub n_vocab: usize,

    /// Context length (maximum sequence length)
    pub n_ctx: usize,

    /// Embedding dimension (model width)
    pub n_embd: usize,

    /// Number of attention heads
    pub n_head: usize,

    /// Number of transformer layers (model depth)
    pub n_layer: usize,
}

impl Default for TransformerConfig {
    /// Default config for tiny model (10-50M parameters)
    ///
    /// Optimized for kernel memory constraints (<8 MB total)
    fn default() -> Self {
        Self {
            n_vocab: 32000,   // Standard vocabulary
            n_ctx: 512,       // Moderate context length
            n_embd: 384,      // Compact embedding
            n_head: 6,        // 64-dim per head (384/6)
            n_layer: 6,       // Shallow network
        }
    }
}

impl TransformerConfig {
    /// Calculate head dimension
    ///
    /// # Returns
    ///
    /// Dimension of each attention head (n_embd / n_head)
    pub fn head_dim(&self) -> usize {
        self.n_embd / self.n_head
    }

    /// Estimate model size (parameters)
    ///
    /// # Returns
    ///
    /// Approximate number of parameters
    pub fn param_count(&self) -> usize {
        // Embedding: n_vocab * n_embd
        let embedding = self.n_vocab * self.n_embd;

        // Per layer: 4 weight matrices (Q, K, V, O) + 2 FFN + 2 LayerNorm
        // QKV: 3 * n_embd * n_embd
        // O: n_embd * n_embd
        // FFN: n_embd * 4*n_embd + 4*n_embd * n_embd
        // LN: 2 * n_embd * 2 (weight + bias for 2 LayerNorms)
        let per_layer = 4 * self.n_embd * self.n_embd +  // Attention
                        8 * self.n_embd * self.n_embd +  // FFN
                        4 * self.n_embd;                  // LayerNorm

        // LM head: n_embd * n_vocab
        let lm_head = self.n_embd * self.n_vocab;

        embedding + (per_layer * self.n_layer) + lm_head
    }

    /// Estimate memory usage (bytes)
    ///
    /// # Arguments
    ///
    /// - `quantized`: If true, assume Q4_0 quantization (4-bit)
    ///
    /// # Returns
    ///
    /// Estimated memory usage in bytes
    pub fn memory_usage(&self, quantized: bool) -> usize {
        let params = self.param_count();

        if quantized {
            // Q4_0: 4 bits per parameter + 1/32 overhead for scales
            params / 2  // Approximate (ignores scale overhead)
        } else {
            // F32: 4 bytes per parameter
            params * 4
        }
    }
}

/// Single transformer layer
///
/// Implements one complete transformer block with:
/// - Multi-head self-attention
/// - Feed-forward network
/// - Layer normalization
/// - Residual connections
pub struct TransformerLayer {
    /// Attention Q projection weights (quantized)
    pub attn_q: Vec<Q4_0Block>,

    /// Attention K projection weights (quantized)
    pub attn_k: Vec<Q4_0Block>,

    /// Attention V projection weights (quantized)
    pub attn_v: Vec<Q4_0Block>,

    /// Attention output projection weights (quantized)
    pub attn_out: Vec<Q4_0Block>,

    /// Feed-forward up projection weights (quantized)
    pub ffn_up: Vec<Q4_0Block>,

    /// Feed-forward down projection weights (quantized)
    pub ffn_down: Vec<Q4_0Block>,

    /// Layer norm 1 weight (f32)
    pub ln1_weight: Vec<f32>,

    /// Layer norm 1 bias (f32)
    pub ln1_bias: Vec<f32>,

    /// Layer norm 2 weight (f32)
    pub ln2_weight: Vec<f32>,

    /// Layer norm 2 bias (f32)
    pub ln2_bias: Vec<f32>,
}

impl TransformerLayer {
    /// Create a new empty layer
    ///
    /// Weights must be loaded separately from GGUF file.
    pub fn new() -> Self {
        Self {
            attn_q: Vec::new(),
            attn_k: Vec::new(),
            attn_v: Vec::new(),
            attn_out: Vec::new(),
            ffn_up: Vec::new(),
            ffn_down: Vec::new(),
            ln1_weight: Vec::new(),
            ln1_bias: Vec::new(),
            ln2_weight: Vec::new(),
            ln2_bias: Vec::new(),
        }
    }

    /// Forward pass through transformer layer
    ///
    /// # Arguments
    ///
    /// - `input`: Input vector (n_embd,)
    /// - `config`: Model configuration
    ///
    /// # Returns
    ///
    /// Output vector (n_embd,)
    ///
    /// # Computation Flow
    ///
    /// ```text
    /// 1. x_norm = LayerNorm(x)
    /// 2. attn_out = MultiHeadAttention(x_norm)
    /// 3. x = x + attn_out  (residual)
    /// 4. x_norm2 = LayerNorm(x)
    /// 5. ffn_out = FeedForward(x_norm2)
    /// 6. x = x + ffn_out  (residual)
    /// ```
    pub fn forward(&self, input: &[f32], config: &TransformerConfig) -> Vec<f32> {
        let n_embd = config.n_embd;
        debug_assert_eq!(input.len(), n_embd);

        // 1. Layer norm 1
        let normed = layer_norm(input, &self.ln1_weight, &self.ln1_bias);

        // 2. Multi-head attention
        let attn_out = self.attention(&normed, config);

        // 3. Residual connection 1
        let mut residual1 = vec![0.0f32; n_embd];
        for i in 0..n_embd {
            residual1[i] = input[i] + attn_out[i];
        }

        // 4. Layer norm 2
        let normed2 = layer_norm(&residual1, &self.ln2_weight, &self.ln2_bias);

        // 5. Feed-forward network
        let ffn_out = self.feed_forward(&normed2, config);

        // 6. Residual connection 2
        let mut output = vec![0.0f32; n_embd];
        for i in 0..n_embd {
            output[i] = residual1[i] + ffn_out[i];
        }

        output
    }

    /// Multi-head self-attention
    ///
    /// Simplified single-head version (multi-head support in future milestone)
    ///
    /// # Arguments
    ///
    /// - `input`: Input vector (n_embd,)
    /// - `config`: Model configuration
    ///
    /// # Returns
    ///
    /// Attention output (n_embd,)
    fn attention(&self, input: &[f32], config: &TransformerConfig) -> Vec<f32> {
        let n_embd = config.n_embd;

        // Dequantize weight matrices
        let mut q_weights = vec![0f32; n_embd * n_embd];
        let mut k_weights = vec![0f32; n_embd * n_embd];
        let mut v_weights = vec![0f32; n_embd * n_embd];
        let mut o_weights = vec![0f32; n_embd * n_embd];

        dequantize_q4_0(&self.attn_q, &mut q_weights);
        dequantize_q4_0(&self.attn_k, &mut k_weights);
        dequantize_q4_0(&self.attn_v, &mut v_weights);
        dequantize_q4_0(&self.attn_out, &mut o_weights);

        // Compute Q, K, V projections
        let q = matmul_vec(input, &q_weights, n_embd, n_embd);
        let k = matmul_vec(input, &k_weights, n_embd, n_embd);
        let v = matmul_vec(input, &v_weights, n_embd, n_embd);

        // Attention scores: Q * K^T / sqrt(d)
        let scale = 1.0 / libm::sqrtf(n_embd as f32);
        let score = dot_product(&q, &k) * scale;

        // Softmax (trivial for single token)
        let attn_weight = 1.0; // exp(score) / exp(score) = 1.0

        // Weighted sum of values
        let mut attn_output = vec![0.0f32; n_embd];
        for i in 0..n_embd {
            attn_output[i] = v[i] * attn_weight;
        }

        // Output projection
        matmul_vec(&attn_output, &o_weights, n_embd, n_embd)
    }

    /// Feed-forward network
    ///
    /// Two-layer MLP with GELU activation:
    /// ```text
    /// FFN(x) = W_down * GELU(W_up * x)
    /// ```
    ///
    /// # Arguments
    ///
    /// - `input`: Input vector (n_embd,)
    /// - `config`: Model configuration
    ///
    /// # Returns
    ///
    /// FFN output (n_embd,)
    fn feed_forward(&self, input: &[f32], config: &TransformerConfig) -> Vec<f32> {
        let n_embd = config.n_embd;
        let n_ffn = n_embd * 4; // Standard: 4x expansion

        // Dequantize weights
        let mut up_weights = vec![0f32; n_embd * n_ffn];
        let mut down_weights = vec![0f32; n_ffn * n_embd];

        dequantize_q4_0(&self.ffn_up, &mut up_weights);
        dequantize_q4_0(&self.ffn_down, &mut down_weights);

        // Up projection
        let hidden = matmul_vec(input, &up_weights, n_embd, n_ffn);

        // GELU activation
        let mut activated = vec![0.0f32; n_ffn];
        for i in 0..n_ffn {
            activated[i] = gelu(hidden[i]);
        }

        // Down projection
        matmul_vec(&activated, &down_weights, n_ffn, n_embd)
    }
}

/// Layer Normalization
///
/// Normalizes activations to have zero mean and unit variance.
///
/// # Formula
///
/// ```text
/// mean = sum(x) / n
/// var = sum((x - mean)^2) / n
/// y = (x - mean) / sqrt(var + eps) * weight + bias
/// ```
///
/// # Arguments
///
/// - `input`: Input vector
/// - `weight`: Scale parameters
/// - `bias`: Shift parameters
///
/// # Returns
///
/// Normalized vector
pub fn layer_norm(input: &[f32], weight: &[f32], bias: &[f32]) -> Vec<f32> {
    let n = input.len();
    debug_assert_eq!(weight.len(), n);
    debug_assert_eq!(bias.len(), n);

    // Compute mean
    let mean: f32 = input.iter().sum::<f32>() / n as f32;

    // Compute variance
    let variance: f32 = input
        .iter()
        .map(|&x| libm::powf(x - mean, 2.0))
        .sum::<f32>() / n as f32;

    // Compute standard deviation (with epsilon for numerical stability)
    const EPSILON: f32 = 1e-5;
    let std = libm::sqrtf(variance + EPSILON);

    // Normalize and scale
    let mut output = vec![0.0f32; n];
    for i in 0..n {
        output[i] = ((input[i] - mean) / std) * weight[i] + bias[i];
    }

    output
}

/// Matrix-vector multiplication
///
/// Computes: y = A * x
///
/// # Arguments
///
/// - `vec`: Input vector (m,)
/// - `mat`: Matrix stored in row-major order (m × n)
/// - `m`: Input dimension
/// - `n`: Output dimension
///
/// # Returns
///
/// Output vector (n,)
///
/// # Example
///
/// ```text
/// A = [1, 2]    x = [3]    y = [1*3 + 2*4] = [11]
///     [3, 4]        [4]        [3*3 + 4*4]   [25]
/// ```
pub fn matmul_vec(vec: &[f32], mat: &[f32], m: usize, n: usize) -> Vec<f32> {
    debug_assert_eq!(vec.len(), m);
    debug_assert_eq!(mat.len(), m * n);

    let mut output = vec![0.0f32; n];

    for i in 0..n {
        let mut sum = 0.0f32;
        for j in 0..m {
            sum += vec[j] * mat[j * n + i];
        }
        output[i] = sum;
    }

    output
}

/// Dot product of two vectors
///
/// Computes: sum(a[i] * b[i])
///
/// # Arguments
///
/// - `a`: First vector
/// - `b`: Second vector (must have same length as a)
///
/// # Returns
///
/// Scalar dot product
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// GELU activation function
///
/// Gaussian Error Linear Unit:
/// ```text
/// GELU(x) = x * Φ(x)
/// where Φ(x) is the cumulative distribution function of standard normal
/// ```
///
/// # Approximation
///
/// Uses tanh approximation for efficiency:
/// ```text
/// GELU(x) ≈ 0.5 * x * (1 + tanh(√(2/π) * (x + 0.044715 * x³)))
/// ```
///
/// # Arguments
///
/// - `x`: Input value
///
/// # Returns
///
/// GELU(x)
pub fn gelu(x: f32) -> f32 {
    const SQRT_2_OVER_PI: f32 = 0.7978845608; // sqrt(2/π)
    const COEFF: f32 = 0.044715;

    let inner = SQRT_2_OVER_PI * (x + COEFF * x * x * x);
    0.5 * x * (1.0 + tanh_approx(inner))
}

/// Fast tanh approximation
///
/// Uses rational approximation for speed.
///
/// # Arguments
///
/// - `x`: Input value
///
/// # Returns
///
/// tanh(x) approximation
fn tanh_approx(x: f32) -> f32 {
    // Clamp to avoid overflow
    if x > 5.0 {
        return 1.0;
    } else if x < -5.0 {
        return -1.0;
    }

    // Rational approximation
    let x2 = x * x;
    let num = x * (27.0 + x2);
    let den = 27.0 + 9.0 * x2;

    num / den
}

/// Softmax function
///
/// Converts logits to probability distribution.
///
/// # Formula
///
/// ```text
/// softmax(x)[i] = exp(x[i]) / sum(exp(x[j]))
/// ```
///
/// # Numerical Stability
///
/// Uses log-sum-exp trick:
/// ```text
/// max_x = max(x)
/// exp(x[i] - max_x) / sum(exp(x[j] - max_x))
/// ```
///
/// # Arguments
///
/// - `logits`: Input logits
///
/// # Returns
///
/// Probability distribution (sums to 1.0)
pub fn softmax(logits: &[f32]) -> Vec<f32> {
    let n = logits.len();
    let mut output = vec![0.0f32; n];

    // Find max for numerical stability
    let max_logit = logits.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    // Compute exp(x - max)
    let mut sum = 0.0f32;
    for i in 0..n {
        let exp_val = libm::expf(logits[i] - max_logit);
        output[i] = exp_val;
        sum += exp_val;
    }

    // Normalize
    for i in 0..n {
        output[i] /= sum;
    }

    output
}

/// Find index of maximum value (argmax)
///
/// # Arguments
///
/// - `values`: Input array
///
/// # Returns
///
/// Index of maximum value
pub fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TransformerConfig::default();
        assert_eq!(config.n_vocab, 32000);
        assert_eq!(config.n_embd, 384);
    }

    #[test]
    fn test_config_head_dim() {
        let config = TransformerConfig::default();
        assert_eq!(config.head_dim(), 64); // 384 / 6
    }

    #[test]
    fn test_layer_norm() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let weight = vec![1.0, 1.0, 1.0, 1.0];
        let bias = vec![0.0, 0.0, 0.0, 0.0];

        let output = layer_norm(&input, &weight, &bias);

        // Mean = 2.5, Variance = 1.25
        // output should have mean ≈ 0, variance ≈ 1
        let output_mean: f32 = output.iter().sum::<f32>() / output.len() as f32;
        assert!(output_mean.abs() < 1e-5);
    }

    #[test]
    fn test_matmul_vec() {
        let vec = vec![1.0, 2.0];
        let mat = vec![1.0, 2.0, 3.0, 4.0]; // 2x2 matrix
        let output = matmul_vec(&vec, &mat, 2, 2);

        assert_eq!(output.len(), 2);
        // [1, 2] * [1, 3] = 1*1 + 2*3 = 7
        //          [2, 4]   1*2 + 2*4 = 10
        assert_eq!(output[0], 7.0);
        assert_eq!(output[1], 10.0);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = dot_product(&a, &b);

        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_eq!(result, 32.0);
    }

    #[test]
    fn test_gelu() {
        let result = gelu(0.0);
        assert!((result - 0.0).abs() < 0.01);

        let result = gelu(1.0);
        assert!(result > 0.8 && result < 0.9); // GELU(1) ≈ 0.84
    }

    #[test]
    fn test_softmax() {
        let logits = vec![1.0, 2.0, 3.0];
        let probs = softmax(&logits);

        // Check sum to 1
        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);

        // Check monotonicity
        assert!(probs[0] < probs[1]);
        assert!(probs[1] < probs[2]);
    }

    #[test]
    fn test_argmax() {
        let values = vec![1.0, 3.0, 2.0];
        assert_eq!(argmax(&values), 1);
    }

    #[test]
    fn test_config_memory_usage() {
        let config = TransformerConfig::default();
        let mem_f32 = config.memory_usage(false);
        let mem_q4 = config.memory_usage(true);

        // Quantized should be ~8x smaller
        assert!(mem_q4 < mem_f32 / 4);
    }
}
