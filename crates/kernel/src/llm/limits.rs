//! Resource Limits and Enforcement
//!
//! # Overview
//!
//! This module enforces hard limits on LLM resource usage to prevent:
//! - Memory exhaustion
//! - CPU starvation
//! - Denial of service attacks
//! - Runaway inferences
//!
//! # Design Philosophy
//!
//! **Defense in Depth**: Multiple layers of protection:
//! 1. Input validation (reject bad requests early)
//! 2. Resource quotas (limit per-user/per-session usage)
//! 3. Runtime monitoring (detect and stop runaway operations)
//! 4. Graceful degradation (fail safely when limits hit)
//!
//! # Limits Enforced
//!
//! | Resource | Limit | Rationale |
//! |----------|-------|-----------|
//! | Prompt length | 2048 tokens | Prevent memory exhaustion |
//! | Generation length | 512 tokens | Bounded execution time |
//! | Concurrent inferences | 32 | CPU/memory capacity |
//! | Inference timeout | 30 seconds | Prevent hangs |
//! | Token budget (per user) | 10k tokens/hour | Fair usage |
//! | Memory per inference | 2 MB | Arena capacity |
//!
//! # Example Usage
//!
//! ```no_run
//! use crate::llm::limits::{ResourceLimits, check_limits};
//!
//! // Check if inference is allowed
//! check_limits(prompt, max_tokens)?;
//!
//! // Track resource usage
//! ResourceLimits::record_tokens(user_id, tokens);
//! ```

use crate::llm::errors::{LlmError, LlmResult};
use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::string::String;

/// Maximum prompt length in tokens
pub const MAX_PROMPT_TOKENS: usize = 2048;

/// Maximum generation length in tokens
pub const MAX_GENERATION_TOKENS: usize = 512;

/// Maximum concurrent inferences
pub const MAX_CONCURRENT_INFERENCES: usize = 32;

/// Inference timeout in milliseconds
pub const INFERENCE_TIMEOUT_MS: u64 = 30_000; // 30 seconds

/// Token budget per user per hour
pub const TOKEN_BUDGET_PER_HOUR: usize = 10_000;

/// Maximum memory per inference (bytes)
pub const MAX_MEMORY_PER_INFERENCE: usize = 2 * 1024 * 1024; // 2 MB

/// Resource limits configuration
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimitsConfig {
    /// Maximum prompt length (tokens)
    pub max_prompt_tokens: usize,

    /// Maximum generation length (tokens)
    pub max_generation_tokens: usize,

    /// Maximum concurrent inferences
    pub max_concurrent: usize,

    /// Inference timeout (milliseconds)
    pub timeout_ms: u64,

    /// Token budget per user per hour
    pub token_budget_per_hour: usize,

    /// Maximum memory per inference (bytes)
    pub max_memory_per_inference: usize,
}

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            max_prompt_tokens: MAX_PROMPT_TOKENS,
            max_generation_tokens: MAX_GENERATION_TOKENS,
            max_concurrent: MAX_CONCURRENT_INFERENCES,
            timeout_ms: INFERENCE_TIMEOUT_MS,
            token_budget_per_hour: TOKEN_BUDGET_PER_HOUR,
            max_memory_per_inference: MAX_MEMORY_PER_INFERENCE,
        }
    }
}

/// Per-user token usage tracking
struct UserQuota {
    /// Total tokens used
    tokens_used: usize,

    /// Last reset timestamp (cycles)
    last_reset: u64,
}

impl UserQuota {
    fn new() -> Self {
        Self {
            tokens_used: 0,
            last_reset: 0, // TODO: Use actual timestamp
        }
    }
}

/// Global resource limits enforcer
pub struct ResourceLimits {
    /// Configuration
    config: ResourceLimitsConfig,

    /// Active inferences count
    active_inferences: usize,

    /// Per-user quotas
    user_quotas: BTreeMap<String, UserQuota>,

    /// Total inferences run
    total_inferences: u64,

    /// Total tokens generated
    total_tokens: u64,

    /// Total timeouts
    total_timeouts: u64,

    /// Total rejections (limit exceeded)
    total_rejections: u64,
}

impl ResourceLimits {
    /// Create new resource limits enforcer
    pub fn new(config: ResourceLimitsConfig) -> Self {
        Self {
            config,
            active_inferences: 0,
            user_quotas: BTreeMap::new(),
            total_inferences: 0,
            total_tokens: 0,
            total_timeouts: 0,
            total_rejections: 0,
        }
    }

    /// Check if inference is allowed
    ///
    /// # Arguments
    ///
    /// - `user_id`: User identifier
    /// - `prompt_tokens`: Prompt length in tokens
    /// - `max_tokens`: Requested generation length
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Inference allowed
    /// - `Err(LlmError)`: Inference rejected with reason
    pub fn check_inference(
        &mut self,
        user_id: &str,
        prompt_tokens: usize,
        max_tokens: usize,
    ) -> LlmResult<()> {
        // Check prompt length
        if prompt_tokens > self.config.max_prompt_tokens {
            self.total_rejections += 1;
            return Err(LlmError::PromptTooLong {
                tokens: prompt_tokens,
                max_tokens: self.config.max_prompt_tokens,
            });
        }

        // Check generation length
        if max_tokens > self.config.max_generation_tokens {
            self.total_rejections += 1;
            return Err(LlmError::ContextLengthExceeded {
                requested: max_tokens,
                max_context: self.config.max_generation_tokens,
            });
        }

        // Check concurrent limit
        if self.active_inferences >= self.config.max_concurrent {
            self.total_rejections += 1;
            return Err(LlmError::TooManyConcurrent {
                active: self.active_inferences,
                max_concurrent: self.config.max_concurrent,
            });
        }

        // Check user token budget
        let quota = self.user_quotas
            .entry(user_id.into())
            .or_insert_with(UserQuota::new);

        // TODO: Reset quota if hour has passed
        // For now, simple check
        if quota.tokens_used + max_tokens > self.config.token_budget_per_hour {
            self.total_rejections += 1;
            return Err(LlmError::TokenBudgetExceeded {
                requested: max_tokens,
                budget: self.config.token_budget_per_hour - quota.tokens_used,
            });
        }

        Ok(())
    }

    /// Start inference (increment active count)
    pub fn start_inference(&mut self) {
        self.active_inferences += 1;
        self.total_inferences += 1;
    }

    /// End inference (decrement active count)
    pub fn end_inference(&mut self) {
        if self.active_inferences > 0 {
            self.active_inferences -= 1;
        }
    }

    /// Record tokens used by user
    pub fn record_tokens(&mut self, user_id: &str, tokens: usize) {
        let quota = self.user_quotas
            .entry(user_id.into())
            .or_insert_with(UserQuota::new);

        quota.tokens_used += tokens;
        self.total_tokens += tokens as u64;
    }

    /// Record timeout
    pub fn record_timeout(&mut self) {
        self.total_timeouts += 1;
    }

    /// Get current active inference count
    pub fn active_count(&self) -> usize {
        self.active_inferences
    }

    /// Get user's remaining token budget
    pub fn remaining_budget(&self, user_id: &str) -> usize {
        self.user_quotas
            .get(user_id)
            .map(|q| self.config.token_budget_per_hour.saturating_sub(q.tokens_used))
            .unwrap_or(self.config.token_budget_per_hour)
    }

    /// Reset user quota (called hourly)
    pub fn reset_user_quota(&mut self, user_id: &str) {
        if let Some(quota) = self.user_quotas.get_mut(user_id) {
            quota.tokens_used = 0;
            quota.last_reset = 0; // TODO: Use actual timestamp
        }
    }

    /// Reset all quotas
    pub fn reset_all_quotas(&mut self) {
        for quota in self.user_quotas.values_mut() {
            quota.tokens_used = 0;
            quota.last_reset = 0;
        }
    }

    /// Get statistics
    pub fn stats(&self) -> ResourceLimitsStats {
        ResourceLimitsStats {
            active_inferences: self.active_inferences,
            max_concurrent: self.config.max_concurrent,
            total_inferences: self.total_inferences,
            total_tokens: self.total_tokens,
            total_timeouts: self.total_timeouts,
            total_rejections: self.total_rejections,
            utilization: (self.active_inferences as f32 / self.config.max_concurrent as f32) * 100.0,
        }
    }
}

/// Resource limits statistics
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimitsStats {
    /// Currently active inferences
    pub active_inferences: usize,

    /// Maximum concurrent inferences
    pub max_concurrent: usize,

    /// Total inferences run
    pub total_inferences: u64,

    /// Total tokens generated
    pub total_tokens: u64,

    /// Total timeouts
    pub total_timeouts: u64,

    /// Total rejections
    pub total_rejections: u64,

    /// Current utilization (percentage)
    pub utilization: f32,
}

impl ResourceLimitsStats {
    /// Pretty-print statistics
    pub fn print(&self) {
        crate::info!("Resource Limits Statistics:");
        crate::info!("  Active: {}/{}", self.active_inferences, self.max_concurrent);
        crate::info!("  Utilization: {:.1}%", self.utilization);
        crate::info!("  Total Inferences: {}", self.total_inferences);
        crate::info!("  Total Tokens: {}", self.total_tokens);
        crate::info!("  Timeouts: {}", self.total_timeouts);
        crate::info!("  Rejections: {}", self.total_rejections);
    }
}

/// Global resource limits instance
static RESOURCE_LIMITS: Mutex<ResourceLimits> = Mutex::new(
    ResourceLimits {
        config: ResourceLimitsConfig {
            max_prompt_tokens: MAX_PROMPT_TOKENS,
            max_generation_tokens: MAX_GENERATION_TOKENS,
            max_concurrent: MAX_CONCURRENT_INFERENCES,
            timeout_ms: INFERENCE_TIMEOUT_MS,
            token_budget_per_hour: TOKEN_BUDGET_PER_HOUR,
            max_memory_per_inference: MAX_MEMORY_PER_INFERENCE,
        },
        active_inferences: 0,
        user_quotas: BTreeMap::new(),
        total_inferences: 0,
        total_tokens: 0,
        total_timeouts: 0,
        total_rejections: 0,
    }
);

/// Initialize resource limits with custom config
pub fn init(config: ResourceLimitsConfig) {
    let mut limits = RESOURCE_LIMITS.lock();
    limits.config = config;
}

/// Check if inference is allowed (global)
pub fn check_inference(user_id: &str, prompt_tokens: usize, max_tokens: usize) -> LlmResult<()> {
    RESOURCE_LIMITS.lock().check_inference(user_id, prompt_tokens, max_tokens)
}

/// Start inference tracking (global)
pub fn start_inference() {
    RESOURCE_LIMITS.lock().start_inference()
}

/// End inference tracking (global)
pub fn end_inference() {
    RESOURCE_LIMITS.lock().end_inference()
}

/// Record tokens used (global)
pub fn record_tokens(user_id: &str, tokens: usize) {
    RESOURCE_LIMITS.lock().record_tokens(user_id, tokens)
}

/// Get active inference count (global)
pub fn active_count() -> usize {
    RESOURCE_LIMITS.lock().active_count()
}

/// Get user's remaining token budget (global)
pub fn remaining_budget(user_id: &str) -> usize {
    RESOURCE_LIMITS.lock().remaining_budget(user_id)
}

/// Get resource limits statistics (global)
pub fn stats() -> ResourceLimitsStats {
    RESOURCE_LIMITS.lock().stats()
}

/// RAII guard for automatic resource tracking
///
/// Automatically calls start_inference() on creation and
/// end_inference() on drop.
pub struct InferenceGuard {
    active: bool,
}

impl InferenceGuard {
    /// Create new inference guard
    pub fn new() -> Self {
        start_inference();
        Self { active: true }
    }

    /// Cancel guard (don't decrement on drop)
    pub fn cancel(mut self) {
        self.active = false;
    }
}

impl Drop for InferenceGuard {
    fn drop(&mut self) {
        if self.active {
            end_inference();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_creation() {
        let limits = ResourceLimits::new(ResourceLimitsConfig::default());
        assert_eq!(limits.active_count(), 0);
    }

    #[test]
    fn test_check_prompt_length() {
        let mut limits = ResourceLimits::new(ResourceLimitsConfig::default());

        // Valid prompt
        assert!(limits.check_inference("user1", 100, 50).is_ok());

        // Too long prompt
        let result = limits.check_inference("user1", 3000, 50);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::PromptTooLong { .. }));
    }

    #[test]
    fn test_concurrent_limit() {
        let mut limits = ResourceLimits::new(ResourceLimitsConfig {
            max_concurrent: 2,
            ..Default::default()
        });

        limits.start_inference();
        limits.start_inference();
        assert_eq!(limits.active_count(), 2);

        // Should reject third
        let result = limits.check_inference("user1", 100, 50);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::TooManyConcurrent { .. }));
    }

    #[test]
    fn test_token_budget() {
        let mut limits = ResourceLimits::new(ResourceLimitsConfig {
            token_budget_per_hour: 100,
            ..Default::default()
        });

        // Use 80 tokens
        limits.record_tokens("user1", 80);
        assert_eq!(limits.remaining_budget("user1"), 20);

        // Try to use 30 more (should fail)
        let result = limits.check_inference("user1", 10, 30);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::TokenBudgetExceeded { .. }));
    }

    #[test]
    fn test_inference_guard() {
        {
            let _guard = InferenceGuard::new();
            assert_eq!(active_count(), 1);
        }
        // Guard dropped
        assert_eq!(active_count(), 0);
    }

    #[test]
    fn test_stats() {
        let mut limits = ResourceLimits::new(ResourceLimitsConfig::default());

        limits.start_inference();
        limits.record_tokens("user1", 100);
        limits.end_inference();

        let stats = limits.stats();
        assert_eq!(stats.total_inferences, 1);
        assert_eq!(stats.total_tokens, 100);
    }
}
