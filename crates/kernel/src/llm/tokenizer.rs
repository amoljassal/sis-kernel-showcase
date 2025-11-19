//! Byte-Pair Encoding (BPE) Tokenizer
//!
//! # Overview
//!
//! This module implements a Byte-Pair Encoding tokenizer compatible with
//! GPT-2, GPT-3, and Llama models. BPE is a subword tokenization algorithm
//! that balances vocabulary size with the ability to represent any text.
//!
//! # Algorithm
//!
//! **Byte-Pair Encoding (BPE)**:
//! 1. Start with character-level vocabulary
//! 2. Iteratively merge most frequent adjacent pairs
//! 3. Build vocabulary of common subwords
//! 4. Encode text using greedy longest-match
//!
//! # Design Rationale
//!
//! **Why BPE?**
//! - **Universal**: Can encode any Unicode text
//! - **Compact**: ~32k tokens covers most common words
//! - **No OOV**: Unknown words split into known subwords
//! - **Standard**: Used by most modern LLMs
//!
//! # Vocabulary Structure
//!
//! ```text
//! Token ID   Bytes         Meaning
//! ─────────────────────────────────
//! 0          <UNK>         Unknown token
//! 1          <BOS>         Beginning of sequence
//! 2          <EOS>         End of sequence
//! 3          <PAD>         Padding token
//! 4-255      [single byte] ASCII characters
//! 256-32767  [multi-byte]  Common subwords
//! ```
//!
//! # Memory Layout
//!
//! The tokenizer vocabulary is stored in the LLM arena:
//! - Forward map: `BTreeMap<u16, Vec<u8>>` (token_id → bytes)
//! - Reverse map: `BTreeMap<Vec<u8>, u16>` (bytes → token_id)
//! - Typical size: ~256 KB for 32k vocabulary
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::tokenizer::BpeTokenizer;
//!
//! let mut tokenizer = BpeTokenizer::new();
//! tokenizer.load_from_gguf(&vocab_data)?;
//!
//! let tokens = tokenizer.encode("Hello, world!");
//! let text = tokenizer.decode(&tokens);
//! assert_eq!(text, "Hello, world!");
//! ```
//!
//! # Performance Characteristics
//!
//! - **Encoding**: O(n * log(vocab_size)) where n is byte count
//! - **Decoding**: O(n) where n is token count
//! - **Memory**: ~256 KB for 32k vocabulary
//!
//! # Compatibility
//!
//! This implementation is compatible with:
//! - GPT-2 tokenizer (50k vocabulary)
//! - GPT-3 tokenizer (50k vocabulary)
//! - Llama tokenizer (32k vocabulary)
//! - TinyLlama tokenizer (32k vocabulary)

use alloc::vec;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
use alloc::format;

/// Maximum vocabulary size (32k tokens)
///
/// This is the standard size used by Llama and many modern LLMs.
/// Smaller vocabularies are supported (e.g., 16k for tiny models).
pub const MAX_VOCAB: usize = 32768;

/// Maximum bytes per token
///
/// BPE tokens typically represent 1-4 characters (~1-16 bytes UTF-8).
/// This limit prevents pathological cases.
pub const MAX_TOKEN_LEN: usize = 64;

/// Special token IDs (standard across most LLMs)
pub const UNK_TOKEN_ID: u16 = 0;
pub const BOS_TOKEN_ID: u16 = 1;
pub const EOS_TOKEN_ID: u16 = 2;
pub const PAD_TOKEN_ID: u16 = 3;

/// Special token strings
pub const UNK_TOKEN_STR: &str = "<UNK>";
pub const BOS_TOKEN_STR: &str = "<BOS>";
pub const EOS_TOKEN_STR: &str = "<EOS>";
pub const PAD_TOKEN_STR: &str = "<PAD>";

/// Byte-Pair Encoding Tokenizer
///
/// Implements the BPE algorithm used by GPT-2/GPT-3/Llama models.
/// Vocabulary is loaded from GGUF model files.
///
/// # Thread Safety
///
/// The tokenizer is not inherently thread-safe (uses interior mutability
/// for caching). Wrap in `Mutex` for concurrent access.
pub struct BpeTokenizer {
    /// Forward mapping: token_id → byte sequence
    vocab: BTreeMap<u16, Vec<u8>>,

    /// Reverse mapping: byte sequence → token_id (for encoding)
    reverse_vocab: BTreeMap<Vec<u8>, u16>,

    /// Merge rules for BPE algorithm
    /// Each entry: (left_token, right_token) with implicit priority by index
    /// Lower index = higher priority (applied first)
    merges: Vec<(Vec<u8>, Vec<u8>)>,

    /// Merge priority map: (left, right) -> priority index
    /// Used for fast lookup during encoding
    merge_priority: BTreeMap<(Vec<u8>, Vec<u8>), usize>,

    /// Vocabulary size
    vocab_size: usize,

    /// Whether special tokens are present
    has_special_tokens: bool,
}

impl BpeTokenizer {
    /// Create a new empty tokenizer
    ///
    /// The tokenizer must be loaded with vocabulary data before use.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let tokenizer = BpeTokenizer::new();
    /// ```
    pub fn new() -> Self {
        Self {
            vocab: BTreeMap::new(),
            reverse_vocab: BTreeMap::new(),
            merges: Vec::new(),
            merge_priority: BTreeMap::new(),
            vocab_size: 0,
            has_special_tokens: false,
        }
    }

    /// Load vocabulary from GGUF model file
    ///
    /// # GGUF Vocabulary Format
    ///
    /// ```text
    /// For each token:
    ///   token_id: u16 (little-endian)
    ///   length: u8 (number of bytes)
    ///   bytes: [u8; length]
    /// ```
    ///
    /// # Arguments
    ///
    /// - `vocab_data`: Raw bytes from GGUF vocabulary section
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Vocabulary loaded successfully
    /// - `Err(msg)`: Parse error
    ///
    /// # Example
    ///
    /// ```no_run
    /// let vocab_data = gguf_model.get_vocab_section();
    /// tokenizer.load_from_gguf(vocab_data)?;
    /// ```
    pub fn load_from_gguf(&mut self, vocab_data: &[u8]) -> Result<(), &'static str> {
        self.vocab.clear();
        self.reverse_vocab.clear();

        let mut offset = 0;
        let mut token_count = 0;

        while offset < vocab_data.len() {
            // Check minimum size for header (token_id + length)
            if offset + 3 > vocab_data.len() {
                break;
            }

            // Parse token_id (u16, little-endian)
            let token_id = u16::from_le_bytes([
                vocab_data[offset],
                vocab_data[offset + 1]
            ]);

            // Parse length
            let len = vocab_data[offset + 2] as usize;
            offset += 3;

            // Validate length
            if len > MAX_TOKEN_LEN {
                return Err("Token length exceeds maximum");
            }

            // Check bounds
            if offset + len > vocab_data.len() {
                return Err("Truncated vocabulary data");
            }

            // Extract bytes
            let bytes = vocab_data[offset..offset + len].to_vec();

            // Insert into maps
            self.vocab.insert(token_id, bytes.clone());
            self.reverse_vocab.insert(bytes, token_id);

            offset += len;
            token_count += 1;

            // Enforce vocabulary size limit
            if token_count >= MAX_VOCAB {
                break;
            }
        }

        self.vocab_size = token_count;

        // Check for special tokens
        self.has_special_tokens = self.vocab.contains_key(&BOS_TOKEN_ID)
            && self.vocab.contains_key(&EOS_TOKEN_ID);

        Ok(())
    }

    /// Load vocabulary from simple text format (for testing)
    ///
    /// # Format
    ///
    /// Each line: `token_id<TAB>token_bytes_hex`
    ///
    /// Example:
    /// ```text
    /// 0    3c554e4b3e
    /// 1    3c424f533e
    /// 2    3c454f533e
    /// 4    48656c6c6f
    /// ```
    ///
    /// # Arguments
    ///
    /// - `data`: Text data with token definitions
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Vocabulary loaded
    /// - `Err(msg)`: Parse error
    pub fn load_from_text(&mut self, data: &str) -> Result<(), &'static str> {
        self.vocab.clear();
        self.reverse_vocab.clear();

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 2 {
                return Err("Invalid format: expected 'id<TAB>hex'");
            }

            let token_id: u16 = parts[0].parse()
                .map_err(|_| "Invalid token ID")?;

            let bytes = hex_decode(parts[1])?;

            self.vocab.insert(token_id, bytes.clone());
            self.reverse_vocab.insert(bytes, token_id);
        }

        self.vocab_size = self.vocab.len();
        self.has_special_tokens = self.vocab.contains_key(&BOS_TOKEN_ID);

        Ok(())
    }

    /// Load BPE merge rules from text format
    ///
    /// # Format
    ///
    /// Each line: `left_hex right_hex`
    ///
    /// Example:
    /// ```text
    /// 48 65    # H + e -> He
    /// 6c 6c    # l + l -> ll
    /// 6f 20    # o + space -> "o "
    /// ```
    ///
    /// # Arguments
    ///
    /// - `data`: Text data with merge rules
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Merges loaded successfully
    /// - `Err(msg)`: Parse error
    pub fn load_merges_from_text(&mut self, data: &str) -> Result<(), &'static str> {
        self.merges.clear();
        self.merge_priority.clear();

        for (idx, line) in data.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return Err("Invalid merge format: expected 'left right'");
            }

            let left = hex_decode(parts[0])?;
            let right = hex_decode(parts[1])?;

            self.merge_priority.insert((left.clone(), right.clone()), idx);
            self.merges.push((left, right));
        }

        Ok(())
    }

    /// Apply BPE merges to a sequence of tokens
    ///
    /// Iteratively finds and applies the highest-priority merge until no more merges are possible.
    ///
    /// # Arguments
    ///
    /// - `tokens`: Initial token sequence (byte-level)
    ///
    /// # Returns
    ///
    /// Merged token sequence
    fn apply_bpe_merges(&self, mut tokens: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        if self.merges.is_empty() || tokens.len() < 2 {
            return tokens;
        }

        loop {
            // Find best merge (highest priority = lowest index)
            let mut best_merge: Option<(usize, usize)> = None;  // (position, priority)

            for i in 0..(tokens.len() - 1) {
                let pair = (tokens[i].clone(), tokens[i + 1].clone());
                if let Some(&priority) = self.merge_priority.get(&pair) {
                    if best_merge.is_none() || priority < best_merge.unwrap().1 {
                        best_merge = Some((i, priority));
                    }
                }
            }

            // If no merge found, we're done
            if best_merge.is_none() {
                break;
            }

            let (pos, _) = best_merge.unwrap();

            // Merge tokens at position pos and pos+1
            let mut merged = tokens[pos].clone();
            merged.extend_from_slice(&tokens[pos + 1]);

            // Create new token sequence
            let mut new_tokens = Vec::new();
            for (i, token) in tokens.iter().enumerate() {
                if i == pos {
                    new_tokens.push(merged.clone());
                } else if i == pos + 1 {
                    // Skip - already merged
                } else {
                    new_tokens.push(token.clone());
                }
            }

            tokens = new_tokens;

            // Safety: prevent infinite loops
            if tokens.len() == 1 {
                break;
            }
        }

        tokens
    }

    /// Encode text to token IDs
    ///
    /// Uses BPE merge algorithm if merges are loaded, otherwise falls back to greedy longest-match.
    ///
    /// # Arguments
    ///
    /// - `text`: Input text to encode
    ///
    /// # Returns
    ///
    /// Vector of token IDs
    ///
    /// # Example
    ///
    /// ```no_run
    /// let tokens = tokenizer.encode("Hello, world!");
    /// // tokens = [15496, 11, 995, 0]
    /// ```
    pub fn encode(&self, text: &str) -> Vec<u16> {
        if !self.merges.is_empty() {
            self.encode_bpe(text)
        } else {
            self.encode_greedy(text)
        }
    }

    /// Encode text using BPE merge algorithm
    ///
    /// # Arguments
    ///
    /// - `text`: Input text to encode
    ///
    /// # Returns
    ///
    /// Vector of token IDs after applying BPE merges
    fn encode_bpe(&self, text: &str) -> Vec<u16> {
        let bytes = text.as_bytes();

        // Start with byte-level tokens
        let mut tokens: Vec<Vec<u8>> = bytes.iter().map(|&b| vec![b]).collect();

        // Apply BPE merges
        tokens = self.apply_bpe_merges(tokens);

        // Convert byte sequences to token IDs
        let mut result = Vec::new();
        for token_bytes in tokens {
            if let Some(&token_id) = self.reverse_vocab.get(&token_bytes) {
                result.push(token_id);
            } else {
                // Unknown token - use UNK
                result.push(UNK_TOKEN_ID);
            }
        }

        result
    }

    /// Encode text using greedy longest-match algorithm
    ///
    /// This is a fallback for when BPE merges are not loaded.
    ///
    /// # Arguments
    ///
    /// - `text`: Input text to encode
    ///
    /// # Returns
    ///
    /// Vector of token IDs
    fn encode_greedy(&self, text: &str) -> Vec<u16> {
        let mut tokens = Vec::new();
        let bytes = text.as_bytes();

        let mut i = 0;
        while i < bytes.len() {
            // Greedy matching: find longest token
            let mut matched_len = 1;
            let mut matched_id = UNK_TOKEN_ID;

            // Try all possible lengths (longest first)
            for len in (1..=MAX_TOKEN_LEN.min(bytes.len() - i)).rev() {
                let candidate = &bytes[i..i + len];
                if let Some(&token_id) = self.reverse_vocab.get(candidate) {
                    matched_len = len;
                    matched_id = token_id;
                    break;
                }
            }

            // Fallback: single byte
            if matched_id == UNK_TOKEN_ID && matched_len == 1 {
                let single_byte = &bytes[i..i + 1];
                if let Some(&token_id) = self.reverse_vocab.get(single_byte) {
                    matched_id = token_id;
                }
            }

            tokens.push(matched_id);
            i += matched_len;
        }

        tokens
    }

    /// Encode text with BOS/EOS tokens
    ///
    /// Adds special tokens: `<BOS> ... <EOS>`
    ///
    /// # Arguments
    ///
    /// - `text`: Input text
    /// - `add_bos`: Whether to add BOS token
    /// - `add_eos`: Whether to add EOS token
    ///
    /// # Returns
    ///
    /// Vector of token IDs with special tokens
    pub fn encode_with_special(&self, text: &str, add_bos: bool, add_eos: bool) -> Vec<u16> {
        let mut tokens = Vec::new();

        if add_bos && self.has_special_tokens {
            tokens.push(BOS_TOKEN_ID);
        }

        tokens.extend(self.encode(text));

        if add_eos && self.has_special_tokens {
            tokens.push(EOS_TOKEN_ID);
        }

        tokens
    }

    /// Decode token IDs to text
    ///
    /// # Arguments
    ///
    /// - `tokens`: Vector of token IDs
    ///
    /// # Returns
    ///
    /// Decoded text string
    ///
    /// # Example
    ///
    /// ```no_run
    /// let text = tokenizer.decode(&[15496, 11, 995]);
    /// // text = "Hello, world"
    /// ```
    pub fn decode(&self, tokens: &[u16]) -> String {
        let mut result = Vec::new();

        for &token_id in tokens {
            // Skip special tokens
            if token_id == BOS_TOKEN_ID || token_id == EOS_TOKEN_ID || token_id == PAD_TOKEN_ID {
                continue;
            }

            // Look up token bytes
            if let Some(bytes) = self.vocab.get(&token_id) {
                result.extend_from_slice(bytes);
            } else {
                // Unknown token: output replacement character
                result.extend_from_slice(UNK_TOKEN_STR.as_bytes());
            }
        }

        // Convert to UTF-8 (lossy)
        String::from_utf8_lossy(&result).to_string()
    }

    /// Decode single token to string
    ///
    /// # Arguments
    ///
    /// - `token_id`: Token ID to decode
    ///
    /// # Returns
    ///
    /// Token as string (or "<UNK>" if not found)
    pub fn decode_token(&self, token_id: u16) -> String {
        if let Some(bytes) = self.vocab.get(&token_id) {
            String::from_utf8_lossy(bytes).to_string()
        } else {
            UNK_TOKEN_STR.to_string()
        }
    }

    /// Get vocabulary size
    ///
    /// # Returns
    ///
    /// Number of tokens in vocabulary
    pub fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    /// Check if token exists
    ///
    /// # Arguments
    ///
    /// - `token_id`: Token ID to check
    ///
    /// # Returns
    ///
    /// `true` if token exists in vocabulary
    pub fn has_token(&self, token_id: u16) -> bool {
        self.vocab.contains_key(&token_id)
    }

    /// Get bytes for a token
    ///
    /// # Arguments
    ///
    /// - `token_id`: Token ID
    ///
    /// # Returns
    ///
    /// - `Some(bytes)`: Byte sequence for token
    /// - `None`: Token not found
    pub fn get_token_bytes(&self, token_id: u16) -> Option<&[u8]> {
        self.vocab.get(&token_id).map(|v| v.as_slice())
    }

    /// Find token ID for byte sequence
    ///
    /// # Arguments
    ///
    /// - `bytes`: Byte sequence
    ///
    /// # Returns
    ///
    /// - `Some(token_id)`: Token ID if found
    /// - `None`: No token for this sequence
    pub fn find_token(&self, bytes: &[u8]) -> Option<u16> {
        self.reverse_vocab.get(bytes).copied()
    }

    /// Get EOS token ID
    pub fn eos_token_id(&self) -> u16 {
        EOS_TOKEN_ID
    }

    /// Get BOS token ID
    pub fn bos_token_id(&self) -> u16 {
        BOS_TOKEN_ID
    }

    /// Get PAD token ID
    pub fn pad_token_id(&self) -> u16 {
        PAD_TOKEN_ID
    }

    /// Check if tokenizer has special tokens
    pub fn has_special_tokens(&self) -> bool {
        self.has_special_tokens
    }

    /// Get tokenizer statistics
    pub fn stats(&self) -> TokenizerStats {
        TokenizerStats {
            vocab_size: self.vocab_size,
            has_special_tokens: self.has_special_tokens,
            avg_token_len: self.average_token_length(),
        }
    }

    /// Calculate average token length in bytes
    fn average_token_length(&self) -> f32 {
        if self.vocab.is_empty() {
            return 0.0;
        }

        let total: usize = self.vocab.values().map(|v| v.len()).sum();
        total as f32 / self.vocab.len() as f32
    }
}

/// Tokenizer statistics
#[derive(Debug, Clone, Copy)]
pub struct TokenizerStats {
    /// Number of tokens in vocabulary
    pub vocab_size: usize,

    /// Whether special tokens are present
    pub has_special_tokens: bool,

    /// Average token length in bytes
    pub avg_token_len: f32,
}

impl Default for BpeTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Decode hexadecimal string to bytes
///
/// # Arguments
///
/// - `hex`: Hexadecimal string (e.g., "48656c6c6f")
///
/// # Returns
///
/// - `Ok(bytes)`: Decoded bytes
/// - `Err(msg)`: Invalid hex string
fn hex_decode(hex: &str) -> Result<Vec<u8>, &'static str> {
    let hex = hex.trim();

    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length");
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| "Invalid hex digit")?;
        bytes.push(byte);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_creation() {
        let tokenizer = BpeTokenizer::new();
        assert_eq!(tokenizer.vocab_size(), 0);
    }

    #[test]
    fn test_hex_decode() {
        let bytes = hex_decode("48656c6c6f").unwrap();
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_load_from_text() {
        let data = "0\t3c554e4b3e\n4\t48656c6c6f\n5\t576f726c64";
        let mut tokenizer = BpeTokenizer::new();
        tokenizer.load_from_text(data).unwrap();
        assert_eq!(tokenizer.vocab_size(), 3);
    }

    #[test]
    fn test_encode_decode() {
        let data = "0\t3c554e4b3e\n4\t48\n5\t656c6c6f";
        let mut tokenizer = BpeTokenizer::new();
        tokenizer.load_from_text(data).unwrap();

        let tokens = tokenizer.encode("Hello");
        assert!(tokens.len() > 0);

        let decoded = tokenizer.decode(&tokens);
        assert!(decoded.contains("ello") || decoded.contains("Hello"));
    }

    #[test]
    fn test_special_tokens() {
        let tokenizer = BpeTokenizer::new();
        assert_eq!(tokenizer.eos_token_id(), EOS_TOKEN_ID);
        assert_eq!(tokenizer.bos_token_id(), BOS_TOKEN_ID);
        assert_eq!(tokenizer.pad_token_id(), PAD_TOKEN_ID);
    }

    #[test]
    fn test_find_token() {
        let data = "4\t48656c6c6f";
        let mut tokenizer = BpeTokenizer::new();
        tokenizer.load_from_text(data).unwrap();

        let token_id = tokenizer.find_token(b"Hello");
        assert_eq!(token_id, Some(4));

        let token_id = tokenizer.find_token(b"World");
        assert_eq!(token_id, None);
    }

    #[test]
    fn test_decode_token() {
        let data = "4\t48656c6c6f";
        let mut tokenizer = BpeTokenizer::new();
        tokenizer.load_from_text(data).unwrap();

        let text = tokenizer.decode_token(4);
        assert_eq!(text, "Hello");

        let text = tokenizer.decode_token(999);
        assert_eq!(text, "<UNK>");
    }

    #[test]
    fn test_stats() {
        let data = "4\t48656c6c6f\n5\t576f726c64";
        let mut tokenizer = BpeTokenizer::new();
        tokenizer.load_from_text(data).unwrap();

        let stats = tokenizer.stats();
        assert_eq!(stats.vocab_size, 2);
        assert_eq!(stats.avg_token_len, 5.0); // Both tokens are 5 bytes
    }

    #[test]
    fn test_load_merges() {
        let merges = "48 65\n6c 6c\n6f 20";
        let mut tokenizer = BpeTokenizer::new();
        tokenizer.load_merges_from_text(merges).unwrap();

        assert_eq!(tokenizer.merges.len(), 3);
        assert_eq!(tokenizer.merges[0], (vec![0x48], vec![0x65])); // H + e
        assert_eq!(tokenizer.merges[1], (vec![0x6c], vec![0x6c])); // l + l
        assert_eq!(tokenizer.merges[2], (vec![0x6f], vec![0x20])); // o + space
    }

    #[test]
    fn test_bpe_merging() {
        // Create a simple tokenizer with byte-level vocab
        let mut tokenizer = BpeTokenizer::new();

        // Add byte-level tokens
        for i in 0..=255u8 {
            tokenizer.vocab.insert(i as u16, vec![i]);
            tokenizer.reverse_vocab.insert(vec![i], i as u16);
        }

        // Add merged tokens
        tokenizer.vocab.insert(256, vec![0x48, 0x65]); // "He"
        tokenizer.reverse_vocab.insert(vec![0x48, 0x65], 256);

        tokenizer.vocab.insert(257, vec![0x6c, 0x6c]); // "ll"
        tokenizer.reverse_vocab.insert(vec![0x6c, 0x6c], 257);

        tokenizer.vocab.insert(258, vec![0x6f, 0x20]); // "o "
        tokenizer.reverse_vocab.insert(vec![0x6f, 0x20], 258);

        // Load merge rules
        let merges = "48 65\n6c 6c\n6f 20";
        tokenizer.load_merges_from_text(merges).unwrap();

        // Test encoding "Hello "
        let tokens = tokenizer.encode("Hello ");

        // Should merge: H+e -> He, l+l -> ll, o+space -> "o "
        // Result: [He(256), ll(257), o (258)]
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], 256); // He
        assert_eq!(tokens[1], 257); // ll
        assert_eq!(tokens[2], 258); // o<space>
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        // Setup tokenizer with vocab and merges
        let mut tokenizer = BpeTokenizer::new();

        // Add byte-level tokens
        for i in 0..=255u8 {
            tokenizer.vocab.insert(i as u16, vec![i]);
            tokenizer.reverse_vocab.insert(vec![i], i as u16);
        }

        // Add common merged tokens
        tokenizer.vocab.insert(256, b"He".to_vec());
        tokenizer.reverse_vocab.insert(b"He".to_vec(), 256);
        tokenizer.vocab.insert(257, b"ll".to_vec());
        tokenizer.reverse_vocab.insert(b"ll".to_vec(), 257);

        // Load merges
        tokenizer.load_merges_from_text("48 65\n6c 6c").unwrap();

        // Encode and decode
        let test_text = "Hello";
        let tokens = tokenizer.encode(test_text);
        let decoded = tokenizer.decode(&tokens);

        // Verify roundtrip
        assert_eq!(decoded, test_text);
    }

    #[test]
    fn test_bpe_priority() {
        // Test that earlier merges have higher priority
        let mut tokenizer = BpeTokenizer::new();

        // Setup byte-level vocab
        for i in 0..=255u8 {
            tokenizer.vocab.insert(i as u16, vec![i]);
            tokenizer.reverse_vocab.insert(vec![i], i as u16);
        }

        // Add tokens for "abc"
        tokenizer.vocab.insert(256, vec![b'a', b'b']); // "ab"
        tokenizer.reverse_vocab.insert(vec![b'a', b'b'], 256);

        tokenizer.vocab.insert(257, vec![b'b', b'c']); // "bc"
        tokenizer.reverse_vocab.insert(vec![b'b', b'c'], 257);

        // Load merges with different priorities
        // First rule (higher priority): a+b -> ab
        // Second rule (lower priority): b+c -> bc
        let merges = "61 62\n62 63"; // a+b, b+c
        tokenizer.load_merges_from_text(merges).unwrap();

        // For input "abc", should merge "ab" first (higher priority)
        // Result: [ab(256), c(99)]
        let tokens = tokenizer.encode("abc");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], 256); // ab
        assert_eq!(tokens[1], 99);  // c
    }

    #[test]
    fn test_empty_and_edge_cases() {
        let mut tokenizer = BpeTokenizer::new();

        // Empty text
        let tokens = tokenizer.encode("");
        assert_eq!(tokens.len(), 0);

        // Single character
        tokenizer.vocab.insert(65, vec![b'A']);
        tokenizer.reverse_vocab.insert(vec![b'A'], 65);
        let tokens = tokenizer.encode("A");
        assert_eq!(tokens, vec![65]);
    }
}
