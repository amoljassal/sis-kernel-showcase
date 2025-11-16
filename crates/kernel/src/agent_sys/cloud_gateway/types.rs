//! Core types for Cloud Gateway

use crate::agent_sys::AgentId;
use alloc::string::String;
use alloc::vec::Vec;

/// LLM Provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum Provider {
    /// Anthropic Claude API
    Claude,
    /// OpenAI GPT-4 API
    GPT4,
    /// Google Gemini API
    Gemini,
    /// Local kernel-resident LLM (fallback)
    LocalFallback,
}

impl Provider {
    /// Get provider name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Claude => "claude",
            Provider::GPT4 => "gpt4",
            Provider::Gemini => "gemini",
            Provider::LocalFallback => "local",
        }
    }

    /// Get cost tier (lower = cheaper)
    pub fn cost_tier(&self) -> u32 {
        match self {
            Provider::LocalFallback => 0,
            Provider::GPT4 => 1,
            Provider::Claude => 2,
            Provider::Gemini => 3,
        }
    }
}

/// LLM Request from agent
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LLMRequest {
    /// Agent making the request
    pub agent_id: AgentId,

    /// Prompt text
    pub prompt: String,

    /// Maximum tokens in response
    pub max_tokens: u32,

    /// Temperature (0.0 - 1.0)
    pub temperature: f32,

    /// Preferred provider (optional)
    pub preferred_provider: Option<Provider>,

    /// System message (optional)
    pub system_message: Option<String>,

    /// Request timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

impl LLMRequest {
    /// Create a new LLM request with defaults
    pub fn new(agent_id: AgentId, prompt: String) -> Self {
        Self {
            agent_id,
            prompt,
            max_tokens: 1000,
            temperature: 0.7,
            preferred_provider: None,
            system_message: None,
            timeout_ms: Some(30_000), // 30 seconds default
        }
    }

    /// Set maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set preferred provider
    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.preferred_provider = Some(provider);
        self
    }
}

/// LLM Response to agent
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LLMResponse {
    /// Provider that fulfilled the request
    pub provider: Provider,

    /// Response text
    pub text: String,

    /// Tokens used in response
    pub tokens_used: u32,

    /// Request duration in microseconds
    pub duration_us: u64,

    /// Was this a fallback response?
    pub was_fallback: bool,
}

impl LLMResponse {
    /// Create a new response
    pub fn new(provider: Provider, text: String, tokens_used: u32, duration_us: u64) -> Self {
        Self {
            provider,
            text,
            tokens_used,
            duration_us,
            was_fallback: false,
        }
    }

    /// Mark as fallback response
    pub fn with_fallback(mut self) -> Self {
        self.was_fallback = true;
        self
    }
}

/// Gateway metrics
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GatewayMetrics {
    /// Total requests processed
    pub total_requests: u64,

    /// Successful requests
    pub successful_requests: u64,

    /// Failed requests (all providers exhausted)
    pub failed_requests: u64,

    /// Requests that used fallback
    pub fallback_requests: u64,

    /// Requests rate-limited
    pub rate_limited_requests: u64,

    /// Per-provider success counts
    pub claude_successes: u64,
    pub gpt4_successes: u64,
    pub gemini_successes: u64,
    pub local_successes: u64,

    /// Per-provider failure counts
    pub claude_failures: u64,
    pub gpt4_failures: u64,
    pub gemini_failures: u64,
    pub local_failures: u64,

    /// Total tokens processed
    pub total_tokens: u64,

    /// Average response time (microseconds)
    pub avg_response_time_us: u64,
}

impl GatewayMetrics {
    /// Record a successful request
    pub fn record_success(&mut self, provider: Provider, tokens: u32, duration_us: u64) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.total_tokens += tokens as u64;

        // Update average response time
        self.avg_response_time_us =
            (self.avg_response_time_us * (self.successful_requests - 1) + duration_us)
            / self.successful_requests;

        // Update per-provider counters
        match provider {
            Provider::Claude => self.claude_successes += 1,
            Provider::GPT4 => self.gpt4_successes += 1,
            Provider::Gemini => self.gemini_successes += 1,
            Provider::LocalFallback => self.local_successes += 1,
        }
    }

    /// Record a failed request
    pub fn record_failure(&mut self, provider: Provider) {
        self.total_requests += 1;
        self.failed_requests += 1;

        match provider {
            Provider::Claude => self.claude_failures += 1,
            Provider::GPT4 => self.gpt4_failures += 1,
            Provider::Gemini => self.gemini_failures += 1,
            Provider::LocalFallback => self.local_failures += 1,
        }
    }

    /// Record a fallback
    pub fn record_fallback(&mut self) {
        self.fallback_requests += 1;
    }

    /// Record a rate limit hit
    pub fn record_rate_limit(&mut self) {
        self.rate_limited_requests += 1;
    }
}

/// Cloud Gateway error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayError {
    /// Agent not found in registry
    UnknownAgent,

    /// Rate limit exceeded
    RateLimitExceeded,

    /// No provider available
    NoProviderAvailable,

    /// All providers failed
    AllProvidersFailed,

    /// Request timeout
    Timeout,

    /// Network error
    NetworkError,

    /// Invalid request
    InvalidRequest,

    /// Permission denied (missing capability)
    PermissionDenied,
}

impl GatewayError {
    /// Convert to error string
    pub fn as_str(&self) -> &'static str {
        match self {
            GatewayError::UnknownAgent => "unknown agent",
            GatewayError::RateLimitExceeded => "rate limit exceeded",
            GatewayError::NoProviderAvailable => "no provider available",
            GatewayError::AllProvidersFailed => "all providers failed",
            GatewayError::Timeout => "request timeout",
            GatewayError::NetworkError => "network error",
            GatewayError::InvalidRequest => "invalid request",
            GatewayError::PermissionDenied => "permission denied",
        }
    }
}
