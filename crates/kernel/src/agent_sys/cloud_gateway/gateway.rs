//! Main Cloud Gateway implementation

use super::backend::{CloudBackend, BackendError, create_all_backends};
use super::types::{Provider, LLMRequest, LLMResponse, GatewayError, GatewayMetrics};
use super::rate_limit::RateLimiter;
use super::fallback::FallbackPolicy;
use crate::agent_sys::AgentId;
use crate::time::get_timestamp_us;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Cloud Gateway orchestrates LLM requests across multiple providers
pub struct CloudGateway {
    /// Available backends by provider
    backends: BTreeMap<Provider, Box<dyn CloudBackend>>,

    /// Rate limiters per agent
    rate_limiters: BTreeMap<AgentId, RateLimiter>,

    /// Fallback policy
    fallback_policy: FallbackPolicy,

    /// Gateway metrics
    metrics: GatewayMetrics,

    /// Default rate limiter for new agents
    default_rate_limit: (u32, u32), // (capacity, refill_rate)
}

impl CloudGateway {
    /// Create a new Cloud Gateway
    pub fn new() -> Self {
        let mut backends = BTreeMap::new();

        // Initialize all backends
        for backend in create_all_backends() {
            let provider = backend.provider();
            backends.insert(provider, backend);
        }

        Self {
            backends,
            rate_limiters: BTreeMap::new(),
            fallback_policy: FallbackPolicy::default(),
            metrics: GatewayMetrics::default(),
            default_rate_limit: (30, 10), // 30 burst, 10/sec sustained
        }
    }

    /// Route an LLM request through the gateway
    pub fn route_request(&mut self, request: &LLMRequest) -> Result<LLMResponse, GatewayError> {
        let start_time = get_timestamp_us();

        // Check rate limit
        if !self.check_rate_limit(request.agent_id) {
            self.metrics.record_rate_limit();
            return Err(GatewayError::RateLimitExceeded);
        }

        // Get fallback chain based on policy and preferred provider
        let mut chain = self.fallback_policy.get_chain(request.preferred_provider);

        // Try each provider in the fallback chain
        let mut last_error = None;
        let mut tried_fallback = false;

        while let Some(provider) = chain.next() {
            // Skip if provider was already tried as primary
            if tried_fallback && request.preferred_provider == Some(provider) {
                continue;
            }

            if let Some(backend) = self.backends.get_mut(&provider) {
                // Check if backend is available
                if !backend.is_available() {
                    continue;
                }

                // Try request
                match backend.execute(request) {
                    Ok(mut response) => {
                        // Mark as fallback if not the first provider
                        if tried_fallback {
                            response = response.with_fallback();
                            self.metrics.record_fallback();
                        }

                        let duration = get_timestamp_us() - start_time;
                        self.metrics.record_success(provider, response.tokens_used, duration);

                        return Ok(response);
                    }
                    Err(e) => {
                        self.metrics.record_failure(provider);
                        last_error = Some(e);
                        tried_fallback = true;

                        // Log fallback attempt
                        crate::uart::print_str(&alloc::format!(
                            "[CloudGateway] {} failed: {}, trying fallback\n",
                            provider.as_str(),
                            e.as_str()
                        ));
                    }
                }
            }
        }

        // All providers failed
        Err(GatewayError::AllProvidersFailed)
    }

    /// Check rate limit for agent
    fn check_rate_limit(&mut self, agent_id: AgentId) -> bool {
        let limiter = self.rate_limiters
            .entry(agent_id)
            .or_insert_with(|| {
                let (cap, rate) = self.default_rate_limit;
                RateLimiter::new(cap, rate)
            });

        limiter.check_and_consume()
    }

    /// Set rate limit for an agent
    pub fn set_agent_rate_limit(&mut self, agent_id: AgentId, capacity: u32, refill_rate: u32) {
        self.rate_limiters.insert(agent_id, RateLimiter::new(capacity, refill_rate));
    }

    /// Set default rate limit for new agents
    pub fn set_default_rate_limit(&mut self, capacity: u32, refill_rate: u32) {
        self.default_rate_limit = (capacity, refill_rate);
    }

    /// Set fallback policy
    pub fn set_fallback_policy(&mut self, policy: FallbackPolicy) {
        self.fallback_policy = policy;
    }

    /// Get gateway metrics
    pub fn metrics(&self) -> &GatewayMetrics {
        &self.metrics
    }

    /// Get backend health for a provider
    pub fn backend_health(&self, provider: Provider) -> Option<f32> {
        self.backends.get(&provider).map(|b| b.health())
    }

    /// Get all backend health statuses
    pub fn all_backend_health(&self) -> BTreeMap<Provider, f32> {
        self.backends.iter()
            .map(|(p, b)| (*p, b.health()))
            .collect()
    }

    /// Reset rate limiter for an agent
    pub fn reset_rate_limit(&mut self, agent_id: AgentId) {
        if let Some(limiter) = self.rate_limiters.get_mut(&agent_id) {
            limiter.reset();
        }
    }

    /// Get available tokens for an agent
    pub fn available_tokens(&mut self, agent_id: AgentId) -> u32 {
        self.rate_limiters
            .get_mut(&agent_id)
            .map(|l| l.available_tokens())
            .unwrap_or(0)
    }

    /// Remove agent rate limiter (cleanup on agent exit)
    pub fn remove_agent(&mut self, agent_id: AgentId) {
        self.rate_limiters.remove(&agent_id);
    }

    /// Get number of active rate limiters
    pub fn active_agents(&self) -> usize {
        self.rate_limiters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_creation() {
        let gateway = CloudGateway::new();
        assert_eq!(gateway.backends.len(), 4); // All 4 providers
        assert_eq!(gateway.active_agents(), 0);
    }

    #[test]
    fn test_rate_limiting() {
        let mut gateway = CloudGateway::new();
        gateway.set_default_rate_limit(2, 1); // Very restrictive: 2 burst, 1/sec

        let req = LLMRequest::new(100, "test".to_string());

        // First 2 should succeed (rate limit check only)
        assert!(gateway.check_rate_limit(100));
        assert!(gateway.check_rate_limit(100));

        // 3rd should fail
        assert!(!gateway.check_rate_limit(100));
    }

    #[test]
    fn test_agent_specific_rate_limit() {
        let mut gateway = CloudGateway::new();

        gateway.set_agent_rate_limit(100, 10, 5);
        gateway.set_agent_rate_limit(101, 2, 1);

        // Agent 100 should have more tokens
        let tokens_100 = gateway.available_tokens(100);
        let tokens_101 = gateway.available_tokens(101);

        assert_eq!(tokens_100, 10);
        assert_eq!(tokens_101, 2);
    }

    #[test]
    fn test_reset_rate_limit() {
        let mut gateway = CloudGateway::new();
        gateway.set_agent_rate_limit(100, 5, 1);

        // Consume all tokens
        for _ in 0..5 {
            gateway.check_rate_limit(100);
        }

        assert_eq!(gateway.available_tokens(100), 0);

        // Reset
        gateway.reset_rate_limit(100);

        assert_eq!(gateway.available_tokens(100), 5);
    }

    #[test]
    fn test_remove_agent() {
        let mut gateway = CloudGateway::new();
        gateway.set_agent_rate_limit(100, 10, 5);

        assert_eq!(gateway.active_agents(), 1);

        gateway.remove_agent(100);

        assert_eq!(gateway.active_agents(), 0);
    }

    #[test]
    fn test_backend_health() {
        let gateway = CloudGateway::new();

        // LocalFallback should always be healthy
        let health = gateway.backend_health(Provider::LocalFallback);
        assert_eq!(health, Some(1.0));
    }

    #[test]
    fn test_all_backend_health() {
        let gateway = CloudGateway::new();
        let health_map = gateway.all_backend_health();

        assert_eq!(health_map.len(), 4);
        assert!(health_map.contains_key(&Provider::LocalFallback));
        assert!(health_map.contains_key(&Provider::Claude));
        assert!(health_map.contains_key(&Provider::GPT4));
        assert!(health_map.contains_key(&Provider::Gemini));
    }

    #[test]
    fn test_fallback_chain_used() {
        let mut gateway = CloudGateway::new();
        gateway.set_fallback_policy(FallbackPolicy::LocalOnly);

        let req = LLMRequest::new(100, "test prompt".to_string());

        // Should fall back to local since cloud providers are not available (stub)
        let result = gateway.route_request(&req);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.provider, Provider::LocalFallback);
    }

    #[test]
    fn test_metrics_tracking() {
        let mut gateway = CloudGateway::new();
        gateway.set_fallback_policy(FallbackPolicy::LocalOnly);

        let req = LLMRequest::new(100, "test".to_string());

        // Make a request
        let _ = gateway.route_request(&req);

        let metrics = gateway.metrics();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.local_successes, 1);
    }
}
