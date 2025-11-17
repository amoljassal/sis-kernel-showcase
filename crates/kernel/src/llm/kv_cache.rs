//! Key-Value Cache for Transformer Attention
//!
//! # Overview
//!
//! The KV cache stores previously computed attention keys and values to avoid
//! recomputation during autoregressive generation. This provides a significant
//! speedup (10-100x) for long sequences.
//!
//! # Why KV Cache?
//!
//! During autoregressive generation, each new token attends to all previous tokens.
//! Without caching:
//! - Token 1: Compute K,V for 1 token
//! - Token 2: Compute K,V for 2 tokens (token 1 recomputed!)
//! - Token 3: Compute K,V for 3 tokens (tokens 1,2 recomputed!)
//! - ...
//! - Token N: Compute K,V for N tokens
//!
//! **Total computations**: O(N²)
//!
//! With KV cache:
//! - Token 1: Compute K,V for 1 token, cache
//! - Token 2: Load K,V for token 1 from cache, compute for token 2
//! - Token 3: Load K,V for tokens 1-2 from cache, compute for token 3
//! - ...
//!
//! **Total computations**: O(N)
//!
//! # Memory Layout
//!
//! ```text
//! For 6 layers, 512 context, 384 embedding dimension:
//!
//! KVCache:
//!   Layer 0: Keys   [512 × 384]  = 768 KB (f32)
//!            Values [512 × 384]  = 768 KB (f32)
//!   Layer 1: Keys   [512 × 384]  = 768 KB (f32)
//!            Values [512 × 384]  = 768 KB (f32)
//!   ...
//!   Layer 5: Keys   [512 × 384]  = 768 KB (f32)
//!            Values [512 × 384]  = 768 KB (f32)
//!
//! Total: 6 layers × 2 (K+V) × 768 KB = ~9 MB
//! ```
//!
//! # Optimization Strategy
//!
//! To fit within 8MB arena:
//! - Use smaller context (256 instead of 512): ~4.5 MB
//! - Or use f16 instead of f32: ~4.5 MB
//! - Or both: ~2.25 MB ✓
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::kv_cache::KVCache;
//!
//! let mut cache = KVCache::new(6, 256, 384);
//!
//! // During generation
//! for pos in 0..max_tokens {
//!     for layer in 0..6 {
//!         let (k, v) = cache.get(layer, pos);
//!         // Use cached values...
//!
//!         // Update cache with new values
//!         cache.update(layer, pos, new_k, new_v);
//!     }
//! }
//! ```

use alloc::vec::Vec;
use alloc::vec;

/// Key-Value Cache for Transformer Attention
///
/// Stores precomputed attention keys and values to avoid recomputation
/// during autoregressive generation.
///
/// # Type Parameters
///
/// - Keys and values stored as f32 for now (future: f16 for memory)
///
/// # Memory Usage
///
/// ```text
/// Memory = n_layer × 2 × n_ctx × n_embd × sizeof(f32)
/// Example: 6 × 2 × 256 × 384 × 4 = 4.7 MB
/// ```
pub struct KVCache {
    /// Cached keys: [n_layer][n_ctx][n_embd]
    keys: Vec<Vec<Vec<f32>>>,

    /// Cached values: [n_layer][n_ctx][n_embd]
    values: Vec<Vec<Vec<f32>>>,

    /// Current sequence position
    seq_pos: usize,

    /// Maximum context length
    n_ctx: usize,

    /// Embedding dimension
    n_embd: usize,

    /// Number of layers
    n_layer: usize,

    /// Cache hit/miss statistics
    hits: u64,
    misses: u64,
}

impl KVCache {
    /// Create a new KV cache
    ///
    /// # Arguments
    ///
    /// - `n_layer`: Number of transformer layers
    /// - `n_ctx`: Maximum context length
    /// - `n_embd`: Embedding dimension
    ///
    /// # Example
    ///
    /// ```no_run
    /// let cache = KVCache::new(6, 256, 384);
    /// ```
    pub fn new(n_layer: usize, n_ctx: usize, n_embd: usize) -> Self {
        let mut keys = Vec::with_capacity(n_layer);
        let mut values = Vec::with_capacity(n_layer);

        for _ in 0..n_layer {
            // Allocate storage for each layer
            let layer_keys = vec![vec![0.0f32; n_embd]; n_ctx];
            let layer_values = vec![vec![0.0f32; n_embd]; n_ctx];

            keys.push(layer_keys);
            values.push(layer_values);
        }

        Self {
            keys,
            values,
            seq_pos: 0,
            n_ctx,
            n_embd,
            n_layer,
            hits: 0,
            misses: 0,
        }
    }

    /// Update cache with new key and value at current position
    ///
    /// # Arguments
    ///
    /// - `layer`: Layer index (0..n_layer)
    /// - `k`: Key vector (n_embd,)
    /// - `v`: Value vector (n_embd,)
    ///
    /// # Example
    ///
    /// ```no_run
    /// cache.update(0, k, v);
    /// ```
    pub fn update(&mut self, layer: usize, k: Vec<f32>, v: Vec<f32>) {
        debug_assert!(layer < self.n_layer);
        debug_assert_eq!(k.len(), self.n_embd);
        debug_assert_eq!(v.len(), self.n_embd);
        debug_assert!(self.seq_pos < self.n_ctx);

        self.keys[layer][self.seq_pos] = k;
        self.values[layer][self.seq_pos] = v;
    }

    /// Get all cached keys and values for a layer up to current position
    ///
    /// # Arguments
    ///
    /// - `layer`: Layer index
    ///
    /// # Returns
    ///
    /// Tuple of (keys, values) slices containing cached data
    ///
    /// # Example
    ///
    /// ```no_run
    /// let (keys, values) = cache.get(0);
    /// // keys: &[Vec<f32>] with length = seq_pos + 1
    /// ```
    pub fn get(&mut self, layer: usize) -> (&[Vec<f32>], &[Vec<f32>]) {
        debug_assert!(layer < self.n_layer);

        if self.seq_pos > 0 {
            self.hits += 1;
        } else {
            self.misses += 1;
        }

        let end_pos = (self.seq_pos + 1).min(self.n_ctx);

        (
            &self.keys[layer][..end_pos],
            &self.values[layer][..end_pos],
        )
    }

    /// Get key and value at specific position
    ///
    /// # Arguments
    ///
    /// - `layer`: Layer index
    /// - `pos`: Position in sequence
    ///
    /// # Returns
    ///
    /// Tuple of (key, value) references
    pub fn get_at(&self, layer: usize, pos: usize) -> (&[f32], &[f32]) {
        debug_assert!(layer < self.n_layer);
        debug_assert!(pos < self.n_ctx);

        (
            &self.keys[layer][pos],
            &self.values[layer][pos],
        )
    }

    /// Advance sequence position (call after updating all layers)
    pub fn advance(&mut self) {
        if self.seq_pos + 1 < self.n_ctx {
            self.seq_pos += 1;
        }
    }

    /// Reset cache (call at start of new generation)
    pub fn reset(&mut self) {
        self.seq_pos = 0;
        // Note: We don't zero out the cache for performance.
        // Old values will be overwritten as needed.
    }

    /// Get current sequence position
    pub fn position(&self) -> usize {
        self.seq_pos
    }

    /// Get maximum context length
    pub fn capacity(&self) -> usize {
        self.n_ctx
    }

    /// Check if cache is full
    pub fn is_full(&self) -> bool {
        self.seq_pos + 1 >= self.n_ctx
    }

    /// Get cache statistics
    pub fn stats(&self) -> KVCacheStats {
        KVCacheStats {
            n_layer: self.n_layer,
            n_ctx: self.n_ctx,
            n_embd: self.n_embd,
            seq_pos: self.seq_pos,
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f32 / (self.hits + self.misses) as f32
            } else {
                0.0
            },
            memory_usage: self.memory_usage(),
        }
    }

    /// Calculate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        // keys + values: n_layer × 2 × n_ctx × n_embd × sizeof(f32)
        self.n_layer * 2 * self.n_ctx * self.n_embd * core::mem::size_of::<f32>()
    }

    /// Estimate memory savings from using cache
    ///
    /// Compares recomputation cost vs cache storage cost
    ///
    /// # Returns
    ///
    /// Speedup factor (how many times faster)
    pub fn speedup_estimate(&self, seq_len: usize) -> f32 {
        if seq_len <= 1 {
            return 1.0;
        }

        // Without cache: compute K,V for all tokens at each step
        // Cost: 1 + 2 + 3 + ... + N = N(N+1)/2
        let without_cache = (seq_len * (seq_len + 1)) / 2;

        // With cache: compute K,V only for new token
        // Cost: N
        let with_cache = seq_len;

        without_cache as f32 / with_cache as f32
    }
}

/// KV Cache Statistics
#[derive(Debug, Clone, Copy)]
pub struct KVCacheStats {
    /// Number of layers
    pub n_layer: usize,

    /// Context length
    pub n_ctx: usize,

    /// Embedding dimension
    pub n_embd: usize,

    /// Current sequence position
    pub seq_pos: usize,

    /// Number of cache hits
    pub hits: u64,

    /// Number of cache misses
    pub misses: u64,

    /// Hit rate (0.0-1.0)
    pub hit_rate: f32,

    /// Memory usage in bytes
    pub memory_usage: usize,
}

impl KVCacheStats {
    /// Pretty-print statistics
    pub fn print(&self) {
        crate::info!("KV Cache Statistics:");
        crate::info!("  Layers: {}", self.n_layer);
        crate::info!("  Context: {}/{}", self.seq_pos, self.n_ctx);
        crate::info!("  Embedding: {}", self.n_embd);
        crate::info!("  Hit Rate: {:.2}%", self.hit_rate * 100.0);
        crate::info!("  Memory: {} KB", self.memory_usage / 1024);
    }
}

/// Compact KV Cache using f16 (future optimization)
///
/// Uses half-precision floats to reduce memory by 2x:
/// - Original: 6 layers × 512 ctx × 384 embd × 4 bytes = 9 MB
/// - With f16: 6 layers × 512 ctx × 384 embd × 2 bytes = 4.5 MB
///
/// Trade-off: Slight accuracy loss (~0.1%) for 2x memory savings
pub struct CompactKVCache {
    // TODO: Implement f16 version in future milestone
    // For now, use standard KVCache
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_cache_creation() {
        let cache = KVCache::new(6, 256, 384);
        assert_eq!(cache.position(), 0);
        assert_eq!(cache.capacity(), 256);
        assert!(!cache.is_full());
    }

    #[test]
    fn test_kv_cache_update_get() {
        let mut cache = KVCache::new(2, 10, 4);

        // Update layer 0
        let k = vec![1.0, 2.0, 3.0, 4.0];
        let v = vec![5.0, 6.0, 7.0, 8.0];
        cache.update(0, k.clone(), v.clone());

        // Get cached values
        let (keys, values) = cache.get(0);
        assert_eq!(keys.len(), 1);
        assert_eq!(values.len(), 1);
        assert_eq!(keys[0], k);
        assert_eq!(values[0], v);
    }

    #[test]
    fn test_kv_cache_advance() {
        let mut cache = KVCache::new(1, 10, 4);
        assert_eq!(cache.position(), 0);

        cache.advance();
        assert_eq!(cache.position(), 1);

        cache.advance();
        assert_eq!(cache.position(), 2);
    }

    #[test]
    fn test_kv_cache_reset() {
        let mut cache = KVCache::new(1, 10, 4);

        cache.advance();
        cache.advance();
        assert_eq!(cache.position(), 2);

        cache.reset();
        assert_eq!(cache.position(), 0);
    }

    #[test]
    fn test_kv_cache_full() {
        let mut cache = KVCache::new(1, 3, 4);

        assert!(!cache.is_full());
        cache.advance();
        assert!(!cache.is_full());
        cache.advance();
        assert!(cache.is_full());
    }

    #[test]
    fn test_kv_cache_memory_usage() {
        let cache = KVCache::new(6, 256, 384);
        let expected = 6 * 2 * 256 * 384 * 4; // n_layer × 2 × n_ctx × n_embd × 4 bytes
        assert_eq!(cache.memory_usage(), expected);
    }

    #[test]
    fn test_kv_cache_speedup() {
        let cache = KVCache::new(6, 256, 384);

        // For 10 tokens: without cache = 55, with cache = 10, speedup = 5.5x
        let speedup = cache.speedup_estimate(10);
        assert!((speedup - 5.5).abs() < 0.1);

        // For 100 tokens: without cache = 5050, with cache = 100, speedup = 50.5x
        let speedup = cache.speedup_estimate(100);
        assert!((speedup - 50.5).abs() < 0.1);
    }

    #[test]
    fn test_kv_cache_stats() {
        let mut cache = KVCache::new(6, 256, 384);

        cache.get(0); // miss
        cache.advance();
        cache.get(0); // hit
        cache.get(0); // hit

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.666).abs() < 0.01);
    }
}
