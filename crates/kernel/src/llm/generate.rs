//! Autoregressive Text Generation
//!
//! # Overview
//!
//! This module implements autoregressive text generation (inference loop)
//! for transformer models. Given a prompt, it generates continuation text
//! token-by-token.
//!
//! # Autoregressive Generation Algorithm
//!
//! ```text
//! 1. Tokenize prompt: "Hello" → [15496]
//! 2. For i = 0 to max_tokens:
//!    a. Run transformer forward pass on sequence
//!    b. Get logits for next token
//!    c. Sample next token from distribution
//!    d. Append to sequence
//!    e. If EOS token, stop
//! 3. Decode tokens to text
//! ```
//!
//! # Sampling Strategies
//!
//! ## Greedy Decoding
//!
//! Always select most likely token:
//! ```text
//! token = argmax(logits)
//! ```
//!
//! - **Pros**: Deterministic, fast
//! - **Cons**: Repetitive, lacks diversity
//!
//! ## Temperature Sampling
//!
//! Adjust probability distribution:
//! ```text
//! logits' = logits / temperature
//! probs = softmax(logits')
//! token = sample(probs)
//! ```
//!
//! - `temperature < 1`: More peaked (conservative)
//! - `temperature = 1`: Standard sampling
//! - `temperature > 1`: More uniform (creative)
//!
//! ## Top-K Sampling
//!
//! Sample from top K most likely tokens:
//! ```text
//! top_k_logits = top_k(logits, k)
//! probs = softmax(top_k_logits)
//! token = sample(probs)
//! ```
//!
//! ## Top-P (Nucleus) Sampling
//!
//! Sample from smallest set with cumulative probability > P:
//! ```text
//! sorted_probs = sort(softmax(logits))
//! cumsum = cumulative_sum(sorted_probs)
//! nucleus = {tokens where cumsum < P}
//! token = sample(nucleus)
//! ```
//!
//! # Performance Optimizations
//!
//! 1. **KV Cache**: Cache attention keys/values to avoid recomputation
//! 2. **Batching**: Process multiple sequences in parallel
//! 3. **Speculative Decoding**: Predict multiple tokens ahead
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::generate::{Generator, GenerationConfig};
//!
//! let config = GenerationConfig {
//!     max_tokens: 50,
//!     temperature: 0.8,
//!     top_k: 40,
//!     top_p: 0.95,
//!     ..Default::default()
//! };
//!
//! let mut generator = Generator::new(model, tokenizer);
//! let output = generator.generate("Once upon a time", config)?;
//! ```

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use crate::llm::tokenizer::BpeTokenizer;
use crate::llm::transformer::{TransformerConfig, TransformerLayer, softmax, argmax};

/// Text generation configuration
#[derive(Debug, Clone, Copy)]
pub struct GenerationConfig {
    /// Maximum number of tokens to generate
    pub max_tokens: usize,

    /// Sampling temperature (0.0 = greedy, higher = more random)
    pub temperature: f32,

    /// Top-K sampling: sample from top K tokens (0 = disabled)
    pub top_k: usize,

    /// Top-P (nucleus) sampling: sample from top P probability mass
    /// (0.0 = disabled, 1.0 = sample from all tokens)
    pub top_p: f32,

    /// Repetition penalty (1.0 = none, >1.0 = penalize repetition)
    pub repetition_penalty: f32,

    /// Stop on EOS token
    pub stop_on_eos: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 50,
            temperature: 1.0,
            top_k: 0,         // Disabled
            top_p: 0.0,       // Disabled
            repetition_penalty: 1.0,
            stop_on_eos: true,
        }
    }
}

impl GenerationConfig {
    /// Greedy decoding (deterministic)
    pub fn greedy() -> Self {
        Self {
            max_tokens: 50,
            temperature: 0.0,
            top_k: 1,
            top_p: 0.0,
            repetition_penalty: 1.0,
            stop_on_eos: true,
        }
    }

    /// Conservative sampling
    pub fn conservative() -> Self {
        Self {
            max_tokens: 50,
            temperature: 0.7,
            top_k: 40,
            top_p: 0.9,
            repetition_penalty: 1.1,
            stop_on_eos: true,
        }
    }

    /// Creative sampling
    pub fn creative() -> Self {
        Self {
            max_tokens: 100,
            temperature: 1.2,
            top_k: 0,
            top_p: 0.95,
            repetition_penalty: 1.05,
            stop_on_eos: true,
        }
    }
}

/// Text generator
///
/// Orchestrates the generation loop with configurable sampling strategies.
pub struct Generator {
    /// Tokenizer for encoding/decoding
    tokenizer: BpeTokenizer,

    /// Model configuration
    config: TransformerConfig,

    /// Generation history (for repetition penalty)
    history: Vec<u16>,
}

impl Generator {
    /// Create new generator
    ///
    /// # Arguments
    ///
    /// - `tokenizer`: Tokenizer instance
    /// - `config`: Model configuration
    pub fn new(tokenizer: BpeTokenizer, config: TransformerConfig) -> Self {
        Self {
            tokenizer,
            config,
            history: Vec::new(),
        }
    }

    /// Generate text from prompt
    ///
    /// # Arguments
    ///
    /// - `prompt`: Input text
    /// - `gen_config`: Generation configuration
    ///
    /// # Returns
    ///
    /// Generated text (prompt + continuation)
    pub fn generate(&mut self, prompt: &str, gen_config: GenerationConfig) -> Result<String, &'static str> {
        // Tokenize prompt
        let mut tokens = self.tokenizer.encode(prompt);
        self.history.clear();
        self.history.extend_from_slice(&tokens);

        // Generate tokens
        for _ in 0..gen_config.max_tokens {
            // Get next token
            let next_token = self.sample_next_token(&tokens, &gen_config)?;

            // Append to sequence
            tokens.push(next_token);
            self.history.push(next_token);

            // Check for EOS
            if gen_config.stop_on_eos && next_token == self.tokenizer.eos_token_id() {
                break;
            }

            // Safety: prevent infinite loops
            if tokens.len() > self.config.n_ctx {
                return Err("Context length exceeded");
            }
        }

        // Decode to text
        Ok(self.tokenizer.decode(&tokens))
    }

    /// Generate text with streaming callback
    ///
    /// Calls the callback function with each generated token.
    ///
    /// # Arguments
    ///
    /// - `prompt`: Input text
    /// - `gen_config`: Generation configuration
    /// - `callback`: Function called with each generated token
    pub fn generate_stream<F>(
        &mut self,
        prompt: &str,
        gen_config: GenerationConfig,
        mut callback: F,
    ) -> Result<String, &'static str>
    where
        F: FnMut(&str),
    {
        // Tokenize prompt
        let mut tokens = self.tokenizer.encode(prompt);
        self.history.clear();
        self.history.extend_from_slice(&tokens);

        // Output prompt
        callback(prompt);

        // Generate tokens
        for _ in 0..gen_config.max_tokens {
            // Get next token
            let next_token = self.sample_next_token(&tokens, &gen_config)?;

            // Decode and callback
            let token_str = self.tokenizer.decode_token(next_token);
            callback(&token_str);

            // Append to sequence
            tokens.push(next_token);
            self.history.push(next_token);

            // Check for EOS
            if gen_config.stop_on_eos && next_token == self.tokenizer.eos_token_id() {
                break;
            }

            // Safety: prevent infinite loops
            if tokens.len() > self.config.n_ctx {
                return Err("Context length exceeded");
            }
        }

        // Return full text
        Ok(self.tokenizer.decode(&tokens))
    }

    /// Sample next token from logits
    ///
    /// Applies temperature, top-k, top-p, and repetition penalty.
    ///
    /// # Arguments
    ///
    /// - `tokens`: Current token sequence
    /// - `gen_config`: Generation configuration
    ///
    /// # Returns
    ///
    /// Next token ID
    fn sample_next_token(&self, tokens: &[u16], gen_config: &GenerationConfig) -> Result<u16, &'static str> {
        // TODO: Run transformer forward pass to get logits
        // For now, placeholder implementation
        let vocab_size = self.config.n_vocab;
        let mut logits = vec![0.0f32; vocab_size];

        // Placeholder: uniform distribution
        for i in 0..vocab_size {
            logits[i] = 1.0;
        }

        // Apply repetition penalty
        if gen_config.repetition_penalty != 1.0 {
            self.apply_repetition_penalty(&mut logits, gen_config.repetition_penalty);
        }

        // Apply temperature
        if gen_config.temperature > 0.0 && gen_config.temperature != 1.0 {
            apply_temperature(&mut logits, gen_config.temperature);
        }

        // Sample token
        let token = if gen_config.temperature == 0.0 || gen_config.top_k == 1 {
            // Greedy sampling
            argmax(&logits) as u16
        } else if gen_config.top_k > 0 {
            // Top-K sampling
            sample_top_k(&logits, gen_config.top_k)
        } else if gen_config.top_p > 0.0 {
            // Top-P sampling
            sample_top_p(&logits, gen_config.top_p)
        } else {
            // Standard sampling
            sample_categorical(&logits) as u16
        };

        Ok(token)
    }

    /// Apply repetition penalty to logits
    ///
    /// Reduces probability of tokens that have already appeared.
    fn apply_repetition_penalty(&self, logits: &mut [f32], penalty: f32) {
        for &token in &self.history {
            let token_idx = token as usize;
            if token_idx < logits.len() {
                if logits[token_idx] >= 0.0 {
                    logits[token_idx] /= penalty;
                } else {
                    logits[token_idx] *= penalty;
                }
            }
        }
    }
}

/// Apply temperature scaling to logits
///
/// # Arguments
///
/// - `logits`: Logits to scale (modified in-place)
/// - `temperature`: Temperature parameter (>0)
fn apply_temperature(logits: &mut [f32], temperature: f32) {
    for logit in logits.iter_mut() {
        *logit /= temperature;
    }
}

/// Sample token using top-K sampling
///
/// # Arguments
///
/// - `logits`: Token logits
/// - `k`: Number of top tokens to consider
///
/// # Returns
///
/// Sampled token ID
fn sample_top_k(logits: &[f32], k: usize) -> u16 {
    let k = k.min(logits.len());

    // Get top K indices
    let mut indexed_logits: Vec<(usize, f32)> = logits.iter()
        .enumerate()
        .map(|(i, &logit)| (i, logit))
        .collect();

    // Partial sort to get top K
    indexed_logits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    indexed_logits.truncate(k);

    // Extract top K logits
    let top_k_logits: Vec<f32> = indexed_logits.iter().map(|(_, logit)| *logit).collect();

    // Sample from top K
    let probs = softmax(&top_k_logits);
    let idx = sample_categorical(&probs);

    indexed_logits[idx].0 as u16
}

/// Sample token using top-P (nucleus) sampling
///
/// # Arguments
///
/// - `logits`: Token logits
/// - `p`: Cumulative probability threshold
///
/// # Returns
///
/// Sampled token ID
fn sample_top_p(logits: &[f32], p: f32) -> u16 {
    // Convert to probabilities
    let probs = softmax(logits);

    // Sort by probability (descending)
    let mut indexed_probs: Vec<(usize, f32)> = probs.iter()
        .enumerate()
        .map(|(i, &prob)| (i, prob))
        .collect();

    indexed_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Find nucleus (smallest set with cumulative prob > p)
    let mut cumsum = 0.0;
    let mut nucleus_size = 0;

    for (_, prob) in &indexed_probs {
        cumsum += prob;
        nucleus_size += 1;
        if cumsum >= p {
            break;
        }
    }

    // Sample from nucleus
    indexed_probs.truncate(nucleus_size);
    let nucleus_probs: Vec<f32> = indexed_probs.iter().map(|(_, prob)| *prob).collect();

    // Renormalize
    let sum: f32 = nucleus_probs.iter().sum();
    let normalized: Vec<f32> = nucleus_probs.iter().map(|p| p / sum).collect();

    let idx = sample_categorical(&normalized);
    indexed_probs[idx].0 as u16
}

/// Sample token from categorical distribution
///
/// # Arguments
///
/// - `probs`: Probability distribution (must sum to ~1.0)
///
/// # Returns
///
/// Sampled index
fn sample_categorical(probs: &[f32]) -> usize {
    // TODO: Use proper RNG
    // For now, return argmax (greedy)
    argmax(probs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_config_default() {
        let config = GenerationConfig::default();
        assert_eq!(config.max_tokens, 50);
        assert_eq!(config.temperature, 1.0);
    }

    #[test]
    fn test_generation_config_greedy() {
        let config = GenerationConfig::greedy();
        assert_eq!(config.temperature, 0.0);
        assert_eq!(config.top_k, 1);
    }

    #[test]
    fn test_apply_temperature() {
        let mut logits = vec![1.0, 2.0, 3.0];
        apply_temperature(&mut logits, 2.0);

        assert_eq!(logits[0], 0.5);
        assert_eq!(logits[1], 1.0);
        assert_eq!(logits[2], 1.5);
    }

    #[test]
    fn test_sample_top_k() {
        let logits = vec![1.0, 5.0, 3.0, 2.0];
        let token = sample_top_k(&logits, 2);

        // Should sample from top 2: indices 1 (5.0) and 2 (3.0)
        // With argmax fallback, should be 1
        assert_eq!(token, 1);
    }
}
