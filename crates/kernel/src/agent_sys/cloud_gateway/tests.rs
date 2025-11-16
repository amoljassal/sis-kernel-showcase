//! Integration tests for Cloud Gateway

#![cfg(test)]

use super::*;
use crate::agent_sys::cloud_gateway::{
    CloudGateway, LLMRequest, Provider, FallbackPolicy, RateLimiter
};
use alloc::string::ToString;

#[test]
fn test_cloud_gateway_initialization() {
    init();
    assert!(is_initialized());

    let gateway_guard = CLOUD_GATEWAY.lock();
    assert!(gateway_guard.is_some());
}

#[test]
fn test_llm_request_creation() {
    let req = LLMRequest::new(100, "test prompt".to_string());

    assert_eq!(req.agent_id, 100);
    assert_eq!(req.prompt, "test prompt");
    assert_eq!(req.max_tokens, 1000); // default
    assert_eq!(req.temperature, 0.7); // default
    assert_eq!(req.preferred_provider, None);
}

#[test]
fn test_llm_request_builder() {
    let req = LLMRequest::new(100, "test".to_string())
        .with_max_tokens(2000)
        .with_temperature(0.5)
        .with_provider(Provider::Claude);

    assert_eq!(req.max_tokens, 2000);
    assert_eq!(req.temperature, 0.5);
    assert_eq!(req.preferred_provider, Some(Provider::Claude));
}

#[test]
fn test_provider_cost_tiers() {
    assert!(Provider::LocalFallback.cost_tier() < Provider::GPT4.cost_tier());
    assert!(Provider::GPT4.cost_tier() < Provider::Claude.cost_tier());
}

#[test]
fn test_gateway_basic_routing() {
    let mut gateway = CloudGateway::new();

    // Set to local-only since cloud providers are stubs
    gateway.set_fallback_policy(FallbackPolicy::LocalOnly);

    let req = LLMRequest::new(100, "test prompt".to_string());

    let result = gateway.route_request(&req);

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.provider, Provider::LocalFallback);
}

#[test]
fn test_rate_limiting_enforcement() {
    let mut gateway = CloudGateway::new();
    gateway.set_default_rate_limit(2, 1); // Very low limit for testing
    gateway.set_fallback_policy(FallbackPolicy::LocalOnly);

    let req = LLMRequest::new(100, "test".to_string());

    // First 2 should succeed
    assert!(gateway.route_request(&req).is_ok());
    assert!(gateway.route_request(&req).is_ok());

    // 3rd should fail due to rate limit
    let result = gateway.route_request(&req);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GatewayError::RateLimitExceeded);
}

#[test]
fn test_per_agent_rate_limits() {
    let mut gateway = CloudGateway::new();

    gateway.set_agent_rate_limit(100, 10, 5);
    gateway.set_agent_rate_limit(101, 2, 1);

    assert_eq!(gateway.available_tokens(100), 10);
    assert_eq!(gateway.available_tokens(101), 2);
}

#[test]
fn test_fallback_chain_execution() {
    let mut gateway = CloudGateway::new();

    // Cost-optimized should try local first
    gateway.set_fallback_policy(FallbackPolicy::CostOptimized);

    let req = LLMRequest::new(100, "test".to_string());
    let result = gateway.route_request(&req);

    assert!(result.is_ok());
    let response = result.unwrap();

    // Should use LocalFallback since it's cheapest and available
    assert_eq!(response.provider, Provider::LocalFallback);
}

#[test]
fn test_preferred_provider() {
    let mut gateway = CloudGateway::new();
    gateway.set_fallback_policy(FallbackPolicy::ReliabilityOptimized);

    // Request with preferred provider
    let req = LLMRequest::new(100, "test".to_string())
        .with_provider(Provider::LocalFallback);

    let result = gateway.route_request(&req);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().provider, Provider::LocalFallback);
}

#[test]
fn test_gateway_metrics_tracking() {
    let mut gateway = CloudGateway::new();
    gateway.set_fallback_policy(FallbackPolicy::LocalOnly);

    let req = LLMRequest::new(100, "test".to_string());

    // Make 5 requests
    for _ in 0..5 {
        let _ = gateway.route_request(&req);
    }

    let metrics = gateway.metrics();
    assert_eq!(metrics.total_requests, 5);
    assert_eq!(metrics.successful_requests, 5);
    assert_eq!(metrics.local_successes, 5);
}

#[test]
fn test_gateway_agent_cleanup() {
    let mut gateway = CloudGateway::new();

    gateway.set_agent_rate_limit(100, 10, 5);
    gateway.set_agent_rate_limit(101, 10, 5);

    assert_eq!(gateway.active_agents(), 2);

    gateway.remove_agent(100);

    assert_eq!(gateway.active_agents(), 1);
}

#[test]
fn test_backend_health_reporting() {
    let gateway = CloudGateway::new();

    let health = gateway.backend_health(Provider::LocalFallback);
    assert_eq!(health, Some(1.0)); // LocalFallback always healthy

    let all_health = gateway.all_backend_health();
    assert_eq!(all_health.len(), 4); // All 4 providers
}

#[test]
fn test_rate_limit_reset() {
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
fn test_fallback_policy_switching() {
    let mut gateway = CloudGateway::new();

    // Start with local-only
    gateway.set_fallback_policy(FallbackPolicy::LocalOnly);

    let req = LLMRequest::new(100, "test".to_string());
    let result = gateway.route_request(&req);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().provider, Provider::LocalFallback);

    // Switch to cost-optimized
    gateway.set_fallback_policy(FallbackPolicy::CostOptimized);

    let result2 = gateway.route_request(&req);
    assert!(result2.is_ok());

    // Should still use LocalFallback (cheapest available)
    assert_eq!(result2.unwrap().provider, Provider::LocalFallback);
}

#[test]
fn test_multiple_agents_concurrent() {
    let mut gateway = CloudGateway::new();
    gateway.set_fallback_policy(FallbackPolicy::LocalOnly);

    // Create requests from different agents
    for agent_id in 100..105 {
        let req = LLMRequest::new(agent_id, "test".to_string());
        let result = gateway.route_request(&req);
        assert!(result.is_ok());
    }

    // All 5 agents should have rate limiters
    assert_eq!(gateway.active_agents(), 5);

    // Metrics should show 5 requests
    assert_eq!(gateway.metrics().total_requests, 5);
}

#[test]
fn test_response_includes_metadata() {
    let mut gateway = CloudGateway::new();
    gateway.set_fallback_policy(FallbackPolicy::LocalOnly);

    let req = LLMRequest::new(100, "test prompt".to_string());
    let result = gateway.route_request(&req);

    assert!(result.is_ok());
    let response = result.unwrap();

    // Check response has expected fields
    assert_eq!(response.provider, Provider::LocalFallback);
    assert!(!response.text.is_empty());
    assert!(response.tokens_used > 0);
    assert!(response.duration_us > 0);
}

#[test]
fn test_gateway_handles_invalid_agent() {
    let mut gateway = CloudGateway::new();
    gateway.set_default_rate_limit(0, 0); // No tokens allowed

    let req = LLMRequest::new(999, "test".to_string());

    // Should fail rate limit check
    let result = gateway.route_request(&req);
    assert!(result.is_err());
}

#[test]
fn test_stress_many_requests() {
    let mut gateway = CloudGateway::new();
    gateway.set_fallback_policy(FallbackPolicy::LocalOnly);
    gateway.set_default_rate_limit(1000, 500); // High limits

    let req = LLMRequest::new(100, "stress test".to_string());

    // Make 100 requests
    let mut successes = 0;
    for _ in 0..100 {
        if gateway.route_request(&req).is_ok() {
            successes += 1;
        }
    }

    // Most should succeed (rate limit allows)
    assert!(successes >= 50);
}

#[test]
fn test_provider_name_strings() {
    assert_eq!(Provider::Claude.as_str(), "claude");
    assert_eq!(Provider::GPT4.as_str(), "gpt4");
    assert_eq!(Provider::Gemini.as_str(), "gemini");
    assert_eq!(Provider::LocalFallback.as_str(), "local");
}

#[test]
fn test_gateway_error_messages() {
    assert_eq!(GatewayError::RateLimitExceeded.as_str(), "rate limit exceeded");
    assert_eq!(GatewayError::AllProvidersFailed.as_str(), "all providers failed");
    assert_eq!(GatewayError::UnknownAgent.as_str(), "unknown agent");
}
