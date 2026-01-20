//! # LLM Provider Interface
//!
//! This module defines the core interface for LLM providers, including rate limiting,
//! endpoint support, and response types.
//!
//! ## Key Components
//!
//! - `LlmProvider`: The base trait that all providers must implement
//! - `ModelConfig`: Configuration for a specific model, including rate limits
//! - `RateLimitState`: Internal state for tracking rate limiting
//! - Response types: `ChatResponse`, `CompletionResponse`, `EmbeddingResponse`
//!
//! ## Rate Limiting
//!
//! The rate limiting implementation:
//! 1. Tracks the last call time and call count
//! 2. Resets the counter when the period expires
//! 3. When the limit is hit, sleeps for the remaining time in the period
//! 4. Never returns an error, just delays the call

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use crate::config::{EndpointType, ModelConfig, RateLimit, RenewalPeriod};

/// Timing information for LLM responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timing {
    /// Total duration of the request in milliseconds
    pub total_duration_ms: u64,
    /// Time spent loading the model in milliseconds
    pub load_duration_ms: u64,
    /// Time spent evaluating the prompt in milliseconds
    pub prompt_eval_duration_ms: u64,
    /// Time spent generating the completion in milliseconds
    pub completion_duration_ms: u64,
}

/// Tool function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    /// Name of the function
    pub name: String,
    /// Description of what the function does
    pub description: String,
    /// JSON Schema for parameters
    pub parameters: serde_json::Value,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Name of the tool (required for some providers like Groq)
    pub name: String,
    /// Type of tool (usually "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition
    pub function: ToolFunction,
}

/// Tool call in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    /// Function to call
    pub function: ToolCallFunction,
}

/// Tool call function details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    /// Name of the function
    pub name: String,
    /// Arguments as JSON value
    pub arguments: serde_json::Value,
}

/// Response from chat endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The generated message content
    pub content: String,
    /// Token usage information
    pub usage: Option<Usage>,
    /// Timing information for the request
    pub timing: Option<Timing>,
    /// The model that generated the response
    pub model: String,
    /// Tool calls if any
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Response from completion endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The generated text
    pub text: String,
    /// Token usage information
    pub usage: Option<Usage>,
    /// Timing information for the request
    pub timing: Option<Timing>,
    /// The model that generated the response
    pub model: String,
}

/// Response from embedding endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// The generated embedding vector
    pub embedding: Vec<f32>,
    /// Token usage information
    pub usage: Option<Usage>,
    /// Timing information for the request
    pub timing: Option<Timing>,
    /// The model that generated the response
    pub model: String,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total number of tokens
    pub total_tokens: u32,
}

/// Message structure for chat requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role (system, user, assistant, tool)
    pub role: String,
    /// Message content
    pub content: String,
    /// Tool call ID (required for tool role messages in OpenAI-compatible APIs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Error type for LLM operations.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// The requested endpoint is not supported by this provider.
    #[error("Unsupported endpoint: {0}")]
    UnsupportedEndpoint(String),
    /// Rate limit has been exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    /// An error occurred in the provider.
    #[error("Provider error: {0}")]
    ProviderError(String),
    /// The requested feature is not implemented.
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),
    /// The service is not available.
    #[error("Service not available: {0}")]
    ServiceUnavailable(String),
    /// The specified model was not found.
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    /// Failed to pull/download a model.
    #[error("Model pull failed: {0}")]
    ModelPullFailed(String),
    /// Operation is not supported by this provider.
    #[error("Operation not supported")]
    NotSupported,
    /// The request was cancelled.
    #[error("Request was cancelled")]
    Cancelled,
}

/// Basic model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Name/identifier of the model.
    pub name: String,
}

/// Progress information for model downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPullProgress {
    /// Current status message
    pub status: String,
    /// Bytes downloaded
    pub downloaded: u64,
    /// Total bytes to download (0 if unknown)
    pub total: u64,
}

/// State for rate limiting using a token bucket algorithm.
#[derive(Debug)]
pub struct RateLimitState {
    /// Current number of available tokens.
    tokens: AtomicU32,
    /// Maximum number of tokens (bucket capacity).
    max_tokens: u32,
    /// How often tokens are refilled.
    refill_rate: Duration,
    /// When the bucket was last refilled.
    last_refill: Mutex<Instant>,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            tokens: AtomicU32::new(0),
            max_tokens: 0,
            refill_rate: Duration::from_secs(1),
            last_refill: Mutex::new(Instant::now()),
        }
    }
}

impl RateLimitState {
    /// Creates a new rate limit state from a rate limit configuration.
    pub fn new(limit: &RateLimit) -> Self {
        let max_tokens = limit.calls;
        let refill_rate = match limit.renewal_period {
            RenewalPeriod::Seconds => Duration::from_secs(1),
            RenewalPeriod::Minutes => Duration::from_secs(60),
            RenewalPeriod::Hours => Duration::from_secs(3600),
        };

        Self {
            tokens: AtomicU32::new(max_tokens),
            max_tokens,
            refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    fn refill(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.lock().unwrap();
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= self.refill_rate {
            // Reset tokens to max and update last_refill time
            self.tokens.store(self.max_tokens, Ordering::SeqCst);
            *last_refill = now;
        }
    }

    fn acquire(&self) -> bool {
        self.refill();

        // Try to acquire a token
        let mut current = self.tokens.load(Ordering::SeqCst);
        loop {
            if current == 0 {
                return false;
            }

            match self.tokens.compare_exchange_weak(
                current,
                current - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(new_current) => current = new_current,
            }
        }
    }
}

/// Base trait for all LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get a reference to the provider's config
    fn config(&self) -> &ModelConfig;

    /// Get a reference to the rate limit state
    fn rate_limit_state(&self) -> &RateLimitState;

    /// Check if an endpoint is supported
    fn supports_endpoint(&self, endpoint: EndpointType) -> bool {
        self.config().supported_endpoints.contains(&endpoint)
    }

    /// Default rate limiting implementation
    async fn check_rate_limit(&self) -> Result<(), LlmError> {
        // Get rate limit config
        let _limit = match &self.config().limit {
            Some(limit) => limit,
            None => return Ok(()), // No rate limiting configured
        };

        // Get or initialize rate limit state
        let state = self.rate_limit_state();

        // Try to acquire a token
        if !state.acquire() {
            return Err(LlmError::RateLimitExceeded);
        }

        Ok(())
    }

    /// Chat endpoint with default "not supported" implementation
    #[allow(clippy::too_many_arguments)]
    async fn chat(
        &self,
        _messages: Vec<Message>,
        _tools: Option<Vec<Tool>>,
        _n: Option<u32>,
        _temperature: Option<f32>,
        _max_tokens: Option<u32>,
        _stop: Option<Vec<String>>,
        _extra_config: Option<HashMap<String, String>>,
        _cancellation_token: Option<CancellationToken>,
    ) -> Result<ChatResponse, LlmError> {
        if !self.supports_endpoint(EndpointType::Chat) {
            return Err(LlmError::UnsupportedEndpoint("chat".to_string()));
        }
        self.check_rate_limit().await?;
        Err(LlmError::NotImplemented("chat".to_string()))
    }

    /// Completion endpoint with default "not supported" implementation
    async fn complete(
        &self,
        _prompt: String,
        _n: Option<u32>,
        _temperature: Option<f32>,
        _max_tokens: Option<u32>,
        _stop: Option<Vec<String>>,
        _extra_config: Option<HashMap<String, String>>,
    ) -> Result<CompletionResponse, LlmError> {
        if !self.supports_endpoint(EndpointType::Completion) {
            return Err(LlmError::UnsupportedEndpoint("completion".to_string()));
        }
        self.check_rate_limit().await?;
        Err(LlmError::NotImplemented("completion".to_string()))
    }

    /// Embedding endpoint with default "not supported" implementation
    async fn embed(
        &self,
        _input: Vec<String>,
        _extra_config: Option<HashMap<String, String>>,
    ) -> Result<EmbeddingResponse, LlmError> {
        if !self.supports_endpoint(EndpointType::Embedding) {
            return Err(LlmError::UnsupportedEndpoint("embedding".to_string()));
        }
        self.check_rate_limit().await?;
        Err(LlmError::NotImplemented("embedding".to_string()))
    }

    // Model Management Methods (with default "not supported" implementations)

    /// Check if the service is available and running
    async fn check_service(&self) -> Result<bool, LlmError> {
        Err(LlmError::NotSupported)
    }

    /// List all available models
    async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError> {
        Err(LlmError::NotSupported)
    }

    /// Check if a specific model exists
    async fn model_exists(&self, _model_name: &str) -> Result<bool, LlmError> {
        Err(LlmError::NotSupported)
    }

    /// Pull/download a model
    async fn pull_model(&self, _model_name: &str) -> Result<(), LlmError> {
        Err(LlmError::NotSupported)
    }

    /// Pull/download a model with progress callback
    async fn pull_model_with_progress<F>(
        &self,
        _model_name: &str,
        _progress_callback: F,
    ) -> Result<(), LlmError>
    where
        F: Fn(ModelPullProgress) + Send + 'static,
    {
        Err(LlmError::NotSupported)
    }

    /// Ensure a model is available (download if necessary)
    async fn ensure_model(&self, model_name: &str) -> Result<(), LlmError> {
        // Default implementation that works for providers that support model management
        if !self.check_service().await? {
            return Err(LlmError::ServiceUnavailable(
                "Service is not running".to_string(),
            ));
        }

        if !self.model_exists(model_name).await? {
            self.pull_model(model_name).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EndpointType, ModelConfig};

    // Mock provider for testing
    struct MockProvider {
        config: ModelConfig,
        rate_limit_state: RateLimitState,
    }

    impl MockProvider {
        fn new(config: ModelConfig) -> Self {
            let rate_limit_state = if let Some(limit) = &config.limit {
                RateLimitState::new(limit)
            } else {
                RateLimitState::default()
            };

            Self {
                config,
                rate_limit_state,
            }
        }
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        fn config(&self) -> &ModelConfig {
            &self.config
        }

        fn rate_limit_state(&self) -> &RateLimitState {
            &self.rate_limit_state
        }
    }

    #[tokio::test]
    async fn test_rate_limit_without_config() {
        let config = ModelConfig {
            name: "test".to_string(),
            supported_endpoints: vec![],
            provider: "test".to_string(),
            model: "test".to_string(),
            config: None,
            limit: None,
        };

        let provider = MockProvider::new(config);

        // Should not block or error without rate limit config
        for _ in 0..10 {
            assert!(provider.check_rate_limit().await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_unsupported_endpoint() {
        let config = ModelConfig {
            name: "test".to_string(),
            supported_endpoints: vec![EndpointType::Chat],
            provider: "test".to_string(),
            model: "test".to_string(),
            config: None,
            limit: None,
        };

        let provider = MockProvider::new(config);

        // Chat should be supported
        assert!(provider.supports_endpoint(EndpointType::Chat));

        // Completion should not be supported
        assert!(!provider.supports_endpoint(EndpointType::Completion));

        // Trying to use unsupported endpoint should return error
        let result = provider
            .complete("test".to_string(), None, None, None, None, None)
            .await;
        assert!(matches!(result, Err(LlmError::UnsupportedEndpoint(_))));
    }
}
