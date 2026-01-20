//! OpenAI-Compatible Client
//!
//! Provides a shared HTTP client for OpenAI-compatible API endpoints.
//! This client works with any provider that implements the OpenAI chat completions API:
//! - Ollama (via /v1/chat/completions)
//! - Groq (via /openai/v1/chat/completions)
//! - OpenAI (via /v1/chat/completions)
//! - vLLM, LM Studio, and other compatible providers

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, warn};

use crate::traits::{ChatResponse, CompletionResponse, LlmError, Message, Tool, ToolCall, Usage};

/// Configuration for rate limit retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries for rate-limited requests
    pub max_retries: u32,
    /// Base delay for exponential backoff (in milliseconds)
    pub base_delay_ms: u64,
    /// Maximum delay cap (in milliseconds)
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 30000,  // 30 seconds - Groq free tier has aggressive rate limits
            max_delay_ms: 120000,  // 2 minutes max
        }
    }
}

/// OpenAI-compatible chat request
#[derive(Debug, Serialize)]
pub struct OpenAiChatRequest {
    /// Model identifier
    pub model: String,
    /// Conversation messages
    pub messages: Vec<OpenAiMessage>,
    /// Sampling temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Tools available to the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
}

/// OpenAI-compatible message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiMessage {
    /// Message role (system, user, assistant, tool)
    pub role: String,
    /// Message content (may be null when tool_calls present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Tool call ID (required for tool role messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Tool calls made by the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl From<Message> for OpenAiMessage {
    fn from(msg: Message) -> Self {
        Self {
            role: msg.role,
            content: Some(msg.content),
            tool_call_id: msg.tool_call_id,
            tool_calls: None,
        }
    }
}

/// OpenAI-compatible chat response
#[derive(Debug, Deserialize)]
pub struct OpenAiChatResponse {
    /// Response ID
    #[allow(dead_code)]
    pub id: String,
    /// Object type
    #[allow(dead_code)]
    pub object: String,
    /// Creation timestamp
    #[allow(dead_code)]
    pub created: u64,
    /// Model used
    pub model: String,
    /// Response choices
    pub choices: Vec<OpenAiChoice>,
    /// Token usage
    pub usage: OpenAiUsage,
}

/// Response choice
#[derive(Debug, Deserialize)]
pub struct OpenAiChoice {
    /// Choice index
    #[allow(dead_code)]
    pub index: u32,
    /// Generated message
    pub message: OpenAiMessage,
    /// Reason for stopping
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Deserialize)]
pub struct OpenAiUsage {
    /// Tokens in prompt
    pub prompt_tokens: u32,
    /// Tokens in completion
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// OpenAI-compatible error response
#[derive(Debug, Deserialize)]
pub struct OpenAiErrorResponse {
    /// Error details
    pub error: OpenAiErrorDetail,
}

/// Error detail
#[derive(Debug, Deserialize)]
pub struct OpenAiErrorDetail {
    /// Error message
    pub message: String,
    /// Error type
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    /// Error code
    pub code: Option<String>,
}

/// Client for OpenAI-compatible APIs
pub struct OpenAiCompatClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    retry_config: RetryConfig,
}

impl OpenAiCompatClient {
    /// Create a new OpenAI-compatible client
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the API (e.g., "http://localhost:11434/v1")
    /// * `api_key` - Optional API key for authentication
    /// * `timeout_secs` - Request timeout in seconds
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        timeout_secs: u64,
    ) -> Result<Self, LlmError> {
        Self::with_retry_config(base_url, api_key, timeout_secs, RetryConfig::default())
    }

    /// Create a new OpenAI-compatible client with custom retry configuration
    pub fn with_retry_config(
        base_url: String,
        api_key: Option<String>,
        timeout_secs: u64,
        retry_config: RetryConfig,
    ) -> Result<Self, LlmError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| LlmError::ProviderError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            retry_config,
        })
    }

    /// Parse retry delay from rate limit error message
    /// Looks for patterns like "Please try again in 970ms" or "retry after 2 seconds"
    fn parse_retry_delay(error_message: &str) -> Option<Duration> {
        // Pattern: "try again in Xms"
        let ms_re = Regex::new(r"try again in (\d+)ms").ok()?;
        if let Some(caps) = ms_re.captures(error_message) {
            if let Ok(ms) = caps[1].parse::<u64>() {
                return Some(Duration::from_millis(ms));
            }
        }

        // Pattern: "try again in X.Xs" or "try again in X seconds"
        let sec_re = Regex::new(r"try again in (\d+(?:\.\d+)?)\s*(?:s|seconds?)").ok()?;
        if let Some(caps) = sec_re.captures(error_message) {
            if let Ok(secs) = caps[1].parse::<f64>() {
                return Some(Duration::from_millis((secs * 1000.0) as u64));
            }
        }

        // Pattern: "Retry-After: X"
        let retry_after_re = Regex::new(r"[Rr]etry-?[Aa]fter:?\s*(\d+)").ok()?;
        if let Some(caps) = retry_after_re.captures(error_message) {
            if let Ok(secs) = caps[1].parse::<u64>() {
                return Some(Duration::from_secs(secs));
            }
        }

        None
    }

    /// Check if an error is a rate limit error
    fn is_rate_limit_error(status: reqwest::StatusCode, error_text: &str) -> bool {
        status == reqwest::StatusCode::TOO_MANY_REQUESTS
            || error_text.to_lowercase().contains("rate limit")
            || error_text.to_lowercase().contains("too many requests")
    }

    /// Calculate backoff delay with exponential increase
    fn calculate_backoff(&self, attempt: u32, suggested_delay: Option<Duration>) -> Duration {
        // If the API suggested a delay, use it (with a small buffer)
        if let Some(delay) = suggested_delay {
            let delay_ms = delay.as_millis() as u64 + 100; // Add 100ms buffer
            return Duration::from_millis(delay_ms.min(self.retry_config.max_delay_ms));
        }

        // Exponential backoff: base_delay * 2^attempt
        let delay_ms = self.retry_config.base_delay_ms * (1 << attempt);
        Duration::from_millis(delay_ms.min(self.retry_config.max_delay_ms))
    }

    /// Send a chat completion request with automatic retry on rate limits
    pub async fn chat(
        &self,
        request: OpenAiChatRequest,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut attempt = 0;

        loop {
            debug!(
                "OpenAI-compat request to {}: model={} messages={} (attempt {})",
                url,
                request.model,
                request.messages.len(),
                attempt + 1
            );

            // Build the request (need to rebuild for each attempt)
            let request_json = serde_json::to_string(&request)
                .map_err(|e| LlmError::ProviderError(format!("Failed to serialize request: {}", e)))?;

            let mut req_builder = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(request_json);

            // Add authorization if API key is present
            if let Some(ref key) = self.api_key {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
            }

            // Execute with optional cancellation
            let response = if let Some(ref token) = cancellation_token {
                tokio::select! {
                    result = req_builder.send() => {
                        result.map_err(|e| LlmError::ProviderError(format!("Request failed: {}", e)))?
                    }
                    _ = token.cancelled() => {
                        debug!("Chat request cancelled");
                        return Err(LlmError::Cancelled);
                    }
                }
            } else {
                req_builder
                    .send()
                    .await
                    .map_err(|e| LlmError::ProviderError(format!("Request failed: {}", e)))?
            };

            let status = response.status();

            if !status.is_success() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                // Check if this is a rate limit error and we can retry
                if Self::is_rate_limit_error(status, &error_text) {
                    if attempt < self.retry_config.max_retries {
                        let suggested_delay = Self::parse_retry_delay(&error_text);
                        let backoff = self.calculate_backoff(attempt, suggested_delay);

                        warn!(
                            "Rate limited (attempt {}/{}). Waiting {:?} before retry...",
                            attempt + 1,
                            self.retry_config.max_retries + 1,
                            backoff
                        );

                        // Wait with cancellation support
                        if let Some(ref token) = cancellation_token {
                            tokio::select! {
                                _ = tokio::time::sleep(backoff) => {}
                                _ = token.cancelled() => {
                                    debug!("Rate limit wait cancelled");
                                    return Err(LlmError::Cancelled);
                                }
                            }
                        } else {
                            tokio::time::sleep(backoff).await;
                        }

                        attempt += 1;
                        continue;
                    }

                    // Exhausted retries
                    error!(
                        "Rate limit exceeded after {} attempts: {}",
                        attempt + 1,
                        error_text
                    );
                    return Err(LlmError::RateLimitExceeded);
                }

                // Try to parse as OpenAI error format
                if let Ok(error_response) = serde_json::from_str::<OpenAiErrorResponse>(&error_text)
                {
                    error!("OpenAI-compat API error: {}", error_response.error.message);
                    return Err(LlmError::ProviderError(format!(
                        "API error: {}",
                        error_response.error.message
                    )));
                }

                error!(
                    "OpenAI-compat API error (status {}): {}",
                    status, error_text
                );
                return Err(LlmError::ProviderError(format!(
                    "API error (status {}): {}",
                    status, error_text
                )));
            }

            // Read response with optional cancellation
            let response_text = if let Some(ref token) = cancellation_token {
                tokio::select! {
                    result = response.text() => {
                        result.map_err(|e| LlmError::ProviderError(format!("Failed to read response: {}", e)))?
                    }
                    _ = token.cancelled() => {
                        debug!("Response reading cancelled");
                        return Err(LlmError::Cancelled);
                    }
                }
            } else {
                response
                    .text()
                    .await
                    .map_err(|e| LlmError::ProviderError(format!("Failed to read response: {}", e)))?
            };

            debug!("Response size: {} bytes", response_text.len());

            let api_response: OpenAiChatResponse =
                serde_json::from_str(&response_text).map_err(|e| {
                    error!("Failed to parse response: {}", e);
                    if response_text.len() > 500 {
                        error!("Response preview: {}...", &response_text[..500]);
                    } else {
                        error!("Full response: {}", response_text);
                    }
                    LlmError::ProviderError(format!("Failed to parse response: {}", e))
                })?;

            // Extract first choice
            let choice = api_response
                .choices
                .first()
                .ok_or_else(|| LlmError::ProviderError("No choices in response".to_string()))?;

            return Ok(ChatResponse {
                content: choice.message.content.clone().unwrap_or_default(),
                usage: Some(Usage {
                    prompt_tokens: api_response.usage.prompt_tokens,
                    completion_tokens: api_response.usage.completion_tokens,
                    total_tokens: api_response.usage.total_tokens,
                }),
                timing: None, // OpenAI format doesn't include detailed timing
                model: api_response.model,
                tool_calls: choice.message.tool_calls.clone(),
            });
        }
    }

    /// Send a completion request (uses chat endpoint with single user message)
    pub async fn complete(
        &self,
        model: String,
        prompt: String,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
    ) -> Result<CompletionResponse, LlmError> {
        // Convert to chat format
        let messages = vec![OpenAiMessage {
            role: "user".to_string(),
            content: Some(prompt),
            tool_call_id: None,
            tool_calls: None,
        }];

        let request = OpenAiChatRequest {
            model: model.clone(),
            messages,
            temperature,
            max_tokens,
            stop,
            tools: None,
            stream: false,
        };

        let chat_response = self.chat(request, None).await?;

        Ok(CompletionResponse {
            text: chat_response.content,
            usage: chat_response.usage,
            timing: chat_response.timing,
            model: chat_response.model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_conversion() {
        let msg = Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_call_id: None,
        };

        let openai_msg: OpenAiMessage = msg.into();
        assert_eq!(openai_msg.role, "user");
        assert_eq!(openai_msg.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_chat_response() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "llama3",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }"#;

        let response: OpenAiChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            response.choices[0].message.content,
            Some("Hello!".to_string())
        );
        assert_eq!(response.usage.total_tokens, 15);
    }

    #[test]
    fn test_parse_response_with_tool_calls() {
        let json = r#"{
            "id": "chatcmpl-456",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "llama3",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "tool_calls": [{
                        "id": "call_abc",
                        "function": {
                            "name": "get_weather",
                            "arguments": {"location": "NYC"}
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 20,
                "completion_tokens": 10,
                "total_tokens": 30
            }
        }"#;

        let response: OpenAiChatResponse = serde_json::from_str(json).unwrap();
        assert!(response.choices[0].message.content.is_none());
        assert!(response.choices[0].message.tool_calls.is_some());
    }

    #[test]
    fn test_parse_retry_delay_ms() {
        let msg = "Rate limit reached. Please try again in 970ms.";
        let delay = OpenAiCompatClient::parse_retry_delay(msg);
        assert_eq!(delay, Some(std::time::Duration::from_millis(970)));
    }

    #[test]
    fn test_parse_retry_delay_seconds() {
        let msg = "Too many requests. Please try again in 2.5 seconds.";
        let delay = OpenAiCompatClient::parse_retry_delay(msg);
        assert_eq!(delay, Some(std::time::Duration::from_millis(2500)));
    }

    #[test]
    fn test_parse_retry_delay_retry_after() {
        let msg = "Retry-After: 5";
        let delay = OpenAiCompatClient::parse_retry_delay(msg);
        assert_eq!(delay, Some(std::time::Duration::from_secs(5)));
    }

    #[test]
    fn test_parse_retry_delay_no_match() {
        let msg = "Some other error message";
        let delay = OpenAiCompatClient::parse_retry_delay(msg);
        assert_eq!(delay, None);
    }

    #[test]
    fn test_is_rate_limit_error() {
        assert!(OpenAiCompatClient::is_rate_limit_error(
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            "any message"
        ));
        assert!(OpenAiCompatClient::is_rate_limit_error(
            reqwest::StatusCode::BAD_REQUEST,
            "Rate limit exceeded"
        ));
        assert!(OpenAiCompatClient::is_rate_limit_error(
            reqwest::StatusCode::BAD_REQUEST,
            "Too many requests"
        ));
        assert!(!OpenAiCompatClient::is_rate_limit_error(
            reqwest::StatusCode::BAD_REQUEST,
            "Invalid request"
        ));
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 30000); // 30 seconds - Groq free tier has aggressive rate limits
        assert_eq!(config.max_delay_ms, 120000); // 2 minutes max
    }
}
