//! Fallback policies for Cloud Gateway
//!
//! Implements intelligent provider selection and fallback strategies
//! for handling failures and optimizing costs.

use super::types::Provider;
use alloc::vec::Vec;

/// Fallback policy determines how to select providers and handle failures
#[derive(Debug, Clone)]
pub enum FallbackPolicy {
    /// Try providers in order of cost (cheapest first)
    CostOptimized,

    /// Try providers in explicit order
    Explicit(Vec<Provider>),

    /// Use local fallback only (no cloud calls)
    LocalOnly,

    /// Try providers in order of reliability (highest success rate first)
    ReliabilityOptimized,

    /// Round-robin across all providers
    RoundRobin { next_index: usize },
}

impl FallbackPolicy {
    /// Get the fallback chain for this policy
    pub fn get_chain(&self, preferred: Option<Provider>) -> FallbackChain {
        match self {
            FallbackPolicy::CostOptimized => {
                // Order: Local → GPT-4 → Claude → Gemini
                FallbackChain::new(alloc::vec![
                    Provider::LocalFallback,
                    Provider::GPT4,
                    Provider::Claude,
                    Provider::Gemini,
                ])
            }

            FallbackPolicy::Explicit(providers) => {
                FallbackChain::new(providers.clone())
            }

            FallbackPolicy::LocalOnly => {
                FallbackChain::new(alloc::vec![Provider::LocalFallback])
            }

            FallbackPolicy::ReliabilityOptimized => {
                // Order: Claude → GPT-4 → Gemini → Local
                // (Assuming Claude is most reliable in this example)
                FallbackChain::new(alloc::vec![
                    Provider::Claude,
                    Provider::GPT4,
                    Provider::Gemini,
                    Provider::LocalFallback,
                ])
            }

            FallbackPolicy::RoundRobin { .. } => {
                // Try all in round-robin order
                FallbackChain::new(alloc::vec![
                    Provider::Claude,
                    Provider::GPT4,
                    Provider::Gemini,
                    Provider::LocalFallback,
                ])
            }
        }.with_preferred(preferred)
    }

    /// Update round-robin index
    pub fn advance_round_robin(&mut self) {
        if let FallbackPolicy::RoundRobin { ref mut next_index } = self {
            *next_index = (*next_index + 1) % 4; // 4 providers total
        }
    }
}

impl Default for FallbackPolicy {
    fn default() -> Self {
        FallbackPolicy::ReliabilityOptimized
    }
}

/// Fallback chain represents the sequence of providers to try
#[derive(Debug, Clone)]
pub struct FallbackChain {
    providers: Vec<Provider>,
    current_index: usize,
}

impl FallbackChain {
    /// Create a new fallback chain
    pub fn new(providers: Vec<Provider>) -> Self {
        Self {
            providers,
            current_index: 0,
        }
    }

    /// Put preferred provider first (if specified and in chain)
    pub fn with_preferred(mut self, preferred: Option<Provider>) -> Self {
        if let Some(pref) = preferred {
            if let Some(pos) = self.providers.iter().position(|p| *p == pref) {
                // Move preferred to front
                let provider = self.providers.remove(pos);
                self.providers.insert(0, provider);
            }
        }
        self
    }

    /// Get next provider to try
    pub fn next(&mut self) -> Option<Provider> {
        if self.current_index < self.providers.len() {
            let provider = self.providers[self.current_index];
            self.current_index += 1;
            Some(provider)
        } else {
            None
        }
    }

    /// Reset to beginning of chain
    pub fn reset(&mut self) {
        self.current_index = 0;
    }

    /// Check if more providers available
    pub fn has_next(&self) -> bool {
        self.current_index < self.providers.len()
    }

    /// Get all providers in order
    pub fn providers(&self) -> &[Provider] {
        &self.providers
    }

    /// Get number of providers remaining
    pub fn remaining(&self) -> usize {
        self.providers.len().saturating_sub(self.current_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_optimized_chain() {
        let policy = FallbackPolicy::CostOptimized;
        let mut chain = policy.get_chain(None);

        assert_eq!(chain.next(), Some(Provider::LocalFallback));
        assert_eq!(chain.next(), Some(Provider::GPT4));
        assert_eq!(chain.next(), Some(Provider::Claude));
        assert_eq!(chain.next(), Some(Provider::Gemini));
        assert_eq!(chain.next(), None);
    }

    #[test]
    fn test_reliability_optimized_chain() {
        let policy = FallbackPolicy::ReliabilityOptimized;
        let mut chain = policy.get_chain(None);

        assert_eq!(chain.next(), Some(Provider::Claude));
        assert_eq!(chain.next(), Some(Provider::GPT4));
        assert_eq!(chain.next(), Some(Provider::Gemini));
        assert_eq!(chain.next(), Some(Provider::LocalFallback));
    }

    #[test]
    fn test_local_only_chain() {
        let policy = FallbackPolicy::LocalOnly;
        let mut chain = policy.get_chain(None);

        assert_eq!(chain.next(), Some(Provider::LocalFallback));
        assert_eq!(chain.next(), None);
    }

    #[test]
    fn test_explicit_chain() {
        let policy = FallbackPolicy::Explicit(alloc::vec![
            Provider::Gemini,
            Provider::GPT4,
        ]);
        let mut chain = policy.get_chain(None);

        assert_eq!(chain.next(), Some(Provider::Gemini));
        assert_eq!(chain.next(), Some(Provider::GPT4));
        assert_eq!(chain.next(), None);
    }

    #[test]
    fn test_preferred_provider() {
        let policy = FallbackPolicy::ReliabilityOptimized;
        let mut chain = policy.get_chain(Some(Provider::Gemini));

        // Gemini should be first even though reliability policy normally puts Claude first
        assert_eq!(chain.next(), Some(Provider::Gemini));
        assert_eq!(chain.next(), Some(Provider::Claude));
        assert_eq!(chain.next(), Some(Provider::GPT4));
    }

    #[test]
    fn test_chain_reset() {
        let policy = FallbackPolicy::LocalOnly;
        let mut chain = policy.get_chain(None);

        assert_eq!(chain.next(), Some(Provider::LocalFallback));
        assert_eq!(chain.next(), None);

        chain.reset();

        assert_eq!(chain.next(), Some(Provider::LocalFallback));
    }

    #[test]
    fn test_chain_has_next() {
        let policy = FallbackPolicy::LocalOnly;
        let mut chain = policy.get_chain(None);

        assert!(chain.has_next());
        chain.next();
        assert!(!chain.has_next());
    }

    #[test]
    fn test_chain_remaining() {
        let policy = FallbackPolicy::CostOptimized;
        let mut chain = policy.get_chain(None);

        assert_eq!(chain.remaining(), 4);
        chain.next();
        assert_eq!(chain.remaining(), 3);
        chain.next();
        assert_eq!(chain.remaining(), 2);
    }
}
