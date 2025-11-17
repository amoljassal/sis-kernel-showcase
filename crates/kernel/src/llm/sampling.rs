//! Advanced Sampling for Language Model Generation
//!
//! # Overview
//!
//! Implements sophisticated sampling strategies for next-token prediction:
//! - **Temperature scaling**: Controls randomness/creativity
//! - **Top-k sampling**: Limits to k most likely tokens
//! - **Top-p (nucleus) sampling**: Limits to tokens whose cumulative probability exceeds p
//! - **Greedy sampling**: Deterministic selection (argmax)
//!
//! # Temperature Scaling
//!
//! Temperature τ controls the sharpness of the probability distribution:
//! - τ = 0: Greedy (always pick most likely token)
//! - τ < 1: Sharper distribution (more conservative)
//! - τ = 1: Unmodified distribution
//! - τ > 1: Flatter distribution (more creative/random)
//!
//! Formula: `P(token) ∝ exp(logit / τ)`
//!
//! # Top-k Sampling
//!
//! Limits sampling to the k most likely tokens:
//! - k = 1: Greedy (same as temperature = 0)
//! - k = 10-50: Common range for balanced creativity
//! - k = vocab_size: No filtering (standard sampling)
//!
//! # Top-p (Nucleus) Sampling
//!
//! Dynamically limits to smallest set of tokens whose cumulative probability exceeds p:
//! - p = 0.0: Greedy
//! - p = 0.9: Common value (90% probability mass)
//! - p = 1.0: No filtering
//!
//! More adaptive than top-k: includes more tokens when distribution is flat,
//! fewer when distribution is peaked.
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::sampling::{SamplingConfig, sample};
//!
//! let logits = vec![2.5, 1.0, 0.5, -1.0];
//!
//! // Greedy sampling
//! let config = SamplingConfig::greedy();
//! let token = sample(&logits, config);
//!
//! // Creative sampling with temperature
//! let config = SamplingConfig::new()
//!     .temperature(1.2)
//!     .top_k(40)
//!     .top_p(0.9);
//! let token = sample(&logits, config);
//! ```
//!
//! # Performance
//!
//! - Temperature scaling: O(n)
//! - Top-k filtering: O(n log k) with partial sort
//! - Top-p filtering: O(n log n) with full sort
//! - Sampling: O(n) linear scan
//!
//! Total: ~1-10 µs for typical vocabulary sizes (1000-50000 tokens)

use alloc::vec::Vec;
use alloc::vec;

/// Sampling configuration
#[derive(Debug, Clone, Copy)]
pub struct SamplingConfig {
    /// Temperature (0.0 = greedy, >1.0 = more random)
    pub temperature: f32,

    /// Top-k filtering (0 = disabled, >0 = keep k most likely)
    pub top_k: usize,

    /// Top-p/nucleus filtering (0.0-1.0, 0.0 = greedy, 1.0 = disabled)
    pub top_p: f32,

    /// Random seed for reproducibility (0 = use time-based)
    pub seed: u64,
}

impl SamplingConfig {
    /// Create new sampling config with defaults
    ///
    /// Defaults:
    /// - temperature: 1.0 (unmodified)
    /// - top_k: 0 (disabled)
    /// - top_p: 1.0 (disabled)
    /// - seed: 0 (time-based)
    pub fn new() -> Self {
        Self {
            temperature: 1.0,
            top_k: 0,
            top_p: 1.0,
            seed: 0,
        }
    }

    /// Greedy sampling (deterministic, always pick most likely)
    pub fn greedy() -> Self {
        Self {
            temperature: 0.0,
            top_k: 1,
            top_p: 0.0,
            seed: 0,
        }
    }

    /// Balanced sampling (good default for most use cases)
    pub fn balanced() -> Self {
        Self {
            temperature: 0.8,
            top_k: 40,
            top_p: 0.9,
            seed: 0,
        }
    }

    /// Creative sampling (more random/diverse)
    pub fn creative() -> Self {
        Self {
            temperature: 1.2,
            top_k: 100,
            top_p: 0.95,
            seed: 0,
        }
    }

    /// Set temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set top-k
    pub fn top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Set top-p
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    /// Set random seed
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample next token from logits
///
/// # Arguments
///
/// - `logits`: Unnormalized log probabilities for each token
/// - `config`: Sampling configuration
///
/// # Returns
///
/// Index of sampled token
///
/// # Example
///
/// ```no_run
/// let logits = vec![2.5, 1.0, 0.5, -1.0];
/// let config = SamplingConfig::balanced();
/// let token = sample(&logits, config);
/// ```
pub fn sample(logits: &[f32], config: SamplingConfig) -> usize {
    debug_assert!(!logits.is_empty());

    // Fast path: greedy sampling
    if config.temperature <= 0.0 || config.top_k == 1 {
        return argmax(logits);
    }

    // 1. Apply temperature scaling
    let mut scaled_logits = apply_temperature(logits, config.temperature);

    // 2. Convert to probabilities (softmax)
    let mut probs = softmax(&scaled_logits);

    // 3. Apply top-k filtering
    if config.top_k > 0 && config.top_k < probs.len() {
        apply_top_k(&mut probs, config.top_k);
    }

    // 4. Apply top-p filtering
    if config.top_p < 1.0 && config.top_p > 0.0 {
        apply_top_p(&mut probs, config.top_p);
    }

    // 5. Renormalize after filtering
    renormalize(&mut probs);

    // 6. Sample from distribution
    sample_from_distribution(&probs, config.seed)
}

/// Apply temperature scaling to logits
///
/// Formula: `logit_scaled = logit / temperature`
///
/// # Arguments
///
/// - `logits`: Original logits
/// - `temperature`: Temperature value (> 0)
///
/// # Returns
///
/// Temperature-scaled logits
fn apply_temperature(logits: &[f32], temperature: f32) -> Vec<f32> {
    debug_assert!(temperature > 0.0);

    logits.iter()
        .map(|&x| x / temperature)
        .collect()
}

/// Compute softmax: exp(logits) / sum(exp(logits))
///
/// Uses numerically stable implementation with max subtraction
///
/// # Arguments
///
/// - `logits`: Input logits
///
/// # Returns
///
/// Probability distribution
fn softmax(logits: &[f32]) -> Vec<f32> {
    // Find max for numerical stability
    let max_logit = logits.iter()
        .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    // Compute exp(logit - max)
    let mut exp_values: Vec<f32> = logits.iter()
        .map(|&x| libm::expf(x - max_logit))
        .collect();

    // Compute sum
    let sum: f32 = exp_values.iter().sum();

    // Normalize
    for val in exp_values.iter_mut() {
        *val /= sum;
    }

    exp_values
}

/// Apply top-k filtering: keep only k highest probability tokens
///
/// # Arguments
///
/// - `probs`: Probability distribution (modified in-place)
/// - `k`: Number of tokens to keep
fn apply_top_k(probs: &mut [f32], k: usize) {
    debug_assert!(k > 0);
    debug_assert!(k <= probs.len());

    if k >= probs.len() {
        return; // No filtering needed
    }

    // Create indexed pairs
    let mut indexed: Vec<(usize, f32)> = probs.iter()
        .enumerate()
        .map(|(i, &p)| (i, p))
        .collect();

    // Partial sort to find k-th largest (descending order)
    indexed.select_nth_unstable_by(k, |a, b| b.1.partial_cmp(&a.1).unwrap());

    // Zero out probabilities below k-th
    let threshold = indexed[k].1;
    for (i, p) in probs.iter_mut().enumerate() {
        if *p < threshold {
            *p = 0.0;
        }
    }
}

/// Apply top-p (nucleus) filtering: keep smallest set of tokens with cumulative prob >= p
///
/// # Arguments
///
/// - `probs`: Probability distribution (modified in-place)
/// - `p`: Cumulative probability threshold (0.0-1.0)
fn apply_top_p(probs: &mut [f32], p: f32) {
    debug_assert!(p >= 0.0 && p <= 1.0);

    if p >= 1.0 {
        return; // No filtering needed
    }

    // Create indexed pairs and sort by probability (descending)
    let mut indexed: Vec<(usize, f32)> = probs.iter()
        .enumerate()
        .map(|(i, &p)| (i, p))
        .collect();

    indexed.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Find cumulative probability threshold
    let mut cumsum = 0.0f32;
    let mut cutoff_idx = indexed.len();

    for (i, (_idx, prob)) in indexed.iter().enumerate() {
        cumsum += prob;
        if cumsum >= p {
            cutoff_idx = i + 1; // Keep i+1 tokens (0..=i)
            break;
        }
    }

    // Zero out probabilities outside nucleus
    for i in cutoff_idx..indexed.len() {
        let idx = indexed[i].0;
        probs[idx] = 0.0;
    }
}

/// Renormalize probability distribution after filtering
///
/// # Arguments
///
/// - `probs`: Probability distribution (modified in-place)
fn renormalize(probs: &mut [f32]) {
    let sum: f32 = probs.iter().sum();

    if sum > 0.0 {
        for p in probs.iter_mut() {
            *p /= sum;
        }
    }
}

/// Sample from probability distribution
///
/// # Arguments
///
/// - `probs`: Probability distribution
/// - `seed`: Random seed (0 = use time-based)
///
/// # Returns
///
/// Index of sampled token
fn sample_from_distribution(probs: &[f32], seed: u64) -> usize {
    // Generate random value in [0, 1)
    let rand_val = if seed == 0 {
        // Use time-based randomness
        random_f32()
    } else {
        // Use seeded randomness for reproducibility
        seeded_random_f32(seed)
    };

    // Linear scan to find cumulative probability bin
    let mut cumsum = 0.0f32;
    for (i, &p) in probs.iter().enumerate() {
        cumsum += p;
        if rand_val < cumsum {
            return i;
        }
    }

    // Fallback: return last non-zero probability
    for i in (0..probs.len()).rev() {
        if probs[i] > 0.0 {
            return i;
        }
    }

    // Final fallback: return 0
    0
}

/// Generate random f32 in [0, 1) using time-based seed
fn random_f32() -> f32 {
    // Use uptime microseconds as entropy source
    let uptime_us = crate::time::uptime_us();

    // Simple LCG (Linear Congruential Generator)
    // X_{n+1} = (a * X_n + c) mod m
    let a: u64 = 1664525;
    let c: u64 = 1013904223;
    let m: u64 = 1u64 << 32;

    let x = ((a * uptime_us + c) % m) as u32;

    // Convert to [0, 1)
    (x as f32) / (m as f32)
}

/// Generate reproducible random f32 in [0, 1) from seed
fn seeded_random_f32(seed: u64) -> f32 {
    // Use the seed directly with LCG
    let a: u64 = 1664525;
    let c: u64 = 1013904223;
    let m: u64 = 1u64 << 32;

    let x = ((a * seed + c) % m) as u32;

    // Convert to [0, 1)
    (x as f32) / (m as f32)
}

/// Find index of maximum value (greedy sampling)
///
/// # Arguments
///
/// - `values`: Array of values
///
/// # Returns
///
/// Index of maximum value
pub fn argmax(values: &[f32]) -> usize {
    debug_assert!(!values.is_empty());

    let mut max_idx = 0;
    let mut max_val = values[0];

    for (i, &val) in values.iter().enumerate().skip(1) {
        if val > max_val {
            max_val = val;
            max_idx = i;
        }
    }

    max_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argmax() {
        let values = vec![1.0, 3.0, 2.0, -1.0];
        assert_eq!(argmax(&values), 1);

        let values = vec![-5.0, -1.0, -3.0];
        assert_eq!(argmax(&values), 1);
    }

    #[test]
    fn test_softmax() {
        let logits = vec![1.0, 2.0, 3.0];
        let probs = softmax(&logits);

        // Check sum = 1.0
        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);

        // Check monotonic
        assert!(probs[0] < probs[1]);
        assert!(probs[1] < probs[2]);
    }

    #[test]
    fn test_temperature_scaling() {
        let logits = vec![1.0, 2.0, 3.0];

        // Temperature = 1.0 (no change)
        let scaled = apply_temperature(&logits, 1.0);
        assert_eq!(scaled, logits);

        // Temperature = 2.0 (flatten)
        let scaled = apply_temperature(&logits, 2.0);
        assert_eq!(scaled, vec![0.5, 1.0, 1.5]);

        // Temperature = 0.5 (sharpen)
        let scaled = apply_temperature(&logits, 0.5);
        assert_eq!(scaled, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_top_k() {
        let mut probs = vec![0.1, 0.4, 0.3, 0.2];

        apply_top_k(&mut probs, 2);

        // Should keep top 2: indices 1 (0.4) and 2 (0.3)
        assert!(probs[0] == 0.0); // 0.1 filtered out
        assert!(probs[1] > 0.0);  // 0.4 kept
        assert!(probs[2] > 0.0);  // 0.3 kept
        assert!(probs[3] == 0.0); // 0.2 filtered out
    }

    #[test]
    fn test_top_p() {
        let mut probs = vec![0.5, 0.3, 0.15, 0.05];

        apply_top_p(&mut probs, 0.8);

        // Cumulative: 0.5, 0.8, 0.95, 1.0
        // Should keep first 2 to reach 0.8
        assert!(probs[0] > 0.0);  // 0.5 kept
        assert!(probs[1] > 0.0);  // 0.3 kept (cumsum = 0.8)
        assert!(probs[2] == 0.0); // 0.15 filtered out
        assert!(probs[3] == 0.0); // 0.05 filtered out
    }

    #[test]
    fn test_renormalize() {
        let mut probs = vec![0.5, 0.0, 0.3, 0.0];
        renormalize(&mut probs);

        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);

        assert!((probs[0] - 0.625).abs() < 1e-6); // 0.5 / 0.8
        assert!(probs[1] == 0.0);
        assert!((probs[2] - 0.375).abs() < 1e-6); // 0.3 / 0.8
        assert!(probs[3] == 0.0);
    }

    #[test]
    fn test_greedy_sampling() {
        let logits = vec![1.0, 3.0, 2.0];
        let config = SamplingConfig::greedy();
        let token = sample(&logits, config);

        assert_eq!(token, 1); // Always picks max
    }

    #[test]
    fn test_seeded_sampling_reproducible() {
        let logits = vec![1.0, 2.0, 3.0, 4.0];
        let config = SamplingConfig::new().seed(12345);

        let token1 = sample(&logits, config);
        let token2 = sample(&logits, config);

        // Same seed should give same result
        assert_eq!(token1, token2);
    }
}
