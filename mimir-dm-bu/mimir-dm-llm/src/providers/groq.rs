//! # Groq Provider
//!
//! This module provides an implementation of the [`LlmProvider`] trait for the Groq API.
//! Groq provides ultra-fast LLM inference using their custom LPU (Language Processing Unit) architecture.
//!
//! Uses the shared OpenAI-compatible client for API communication.
//!
//! ## Configuration
//!
//! The Groq provider requires an API key in the configuration. The upstream application
//! is responsible for sourcing this key (from environment variables, secure storage, etc.).
//!
//! Example configuration:
//!
//! ```rust
//! use std::collections::HashMap;
//! use mimir_dm_llm::config::{ModelConfig, EndpointType};
//!
//! let mut config_map = HashMap::new();
//! config_map.insert("api_key".to_string(), "gsk_...".to_string());
//! // Optional: custom base URL
//! // config_map.insert("base_url".to_string(), "https://api.groq.com/openai/v1".to_string());
//!
//! let config = ModelConfig {
//!     name: "groq-llama3".to_string(),
//!     model: "llama-3.3-70b-versatile".to_string(),
//!     provider: "groq".to_string(),
//!     supported_endpoints: vec![EndpointType::Chat, EndpointType::Completion],
//!     config: Some(config_map),
//!     limit: None,
//! };
//! ```
//!
//! ## Supported Models
//!
//! - llama-3.3-70b-versatile (recommended)
//! - llama-3.1-8b-instant
//! - llama-3.2-90b-vision-preview
//! - mixtral-8x7b-32768
//! - gemma2-9b-it
//!
//! See https://console.groq.com/docs/models for the current list.

use async_trait::async_trait;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::config::{EndpointType, ModelConfig};
use crate::providers::openai_compat::{OpenAiChatRequest, OpenAiCompatClient, OpenAiMessage};
use crate::traits::{
    ChatResponse, CompletionResponse, EmbeddingResponse, LlmError, LlmProvider, Message,
    ModelInfo, RateLimitState, Tool,
};

// Note: Groq now uses the shared OpenAI-compatible client (OpenAiCompatClient).
// No Groq-specific request/response types needed.

/// Groq provider implementation
///
/// Uses the shared OpenAI-compatible client for all API communication.
pub struct GroqProvider {
    config: ModelConfig,
    rate_limit_state: RateLimitState,
    openai_client: OpenAiCompatClient,
}

impl GroqProvider {
    /// Create a new Groq provider
    pub fn new(config: ModelConfig) -> Result<Self, LlmError> {
        // Get API key from config (required)
        let api_key = config
            .config
            .as_ref()
            .and_then(|c| c.get("api_key"))
            .ok_or_else(|| {
                LlmError::ConfigError(
                    "Missing api_key in config - upstream application must provide API key"
                        .to_string(),
                )
            })?
            .to_string();

        // Get base_url from config or use default
        let base_url = config
            .config
            .as_ref()
            .and_then(|c| c.get("base_url"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string())
            .trim_end_matches('/')
            .to_string();

        let rate_limit_state = config
            .limit
            .as_ref()
            .map_or_else(RateLimitState::default, RateLimitState::new);

        // Create OpenAI-compatible client with API key
        let openai_client = OpenAiCompatClient::new(base_url, Some(api_key), 300)?;

        Ok(Self {
            config,
            rate_limit_state,
            openai_client,
        })
    }
}

#[async_trait]
impl LlmProvider for GroqProvider {
    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn rate_limit_state(&self) -> &RateLimitState {
        &self.rate_limit_state
    }

    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
        _n: Option<u32>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
        _extra_config: Option<HashMap<String, String>>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<ChatResponse, LlmError> {
        if !self.supports_endpoint(EndpointType::Chat) {
            return Err(LlmError::UnsupportedEndpoint("chat".to_string()));
        }

        self.check_rate_limit().await?;

        // Convert messages to OpenAI format
        let openai_messages: Vec<OpenAiMessage> =
            messages.into_iter().map(OpenAiMessage::from).collect();

        let request = OpenAiChatRequest {
            model: self.config.model.clone(),
            messages: openai_messages,
            temperature,
            max_tokens,
            stop,
            tools,
            stream: false,
        };

        debug!(
            "Groq chat request via OpenAI-compat: model={} messages={}",
            request.model,
            request.messages.len()
        );

        // Use the OpenAI-compatible client
        self.openai_client.chat(request, cancellation_token).await
    }

    async fn complete(
        &self,
        prompt: String,
        _n: Option<u32>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        stop: Option<Vec<String>>,
        _extra_config: Option<HashMap<String, String>>,
    ) -> Result<CompletionResponse, LlmError> {
        if !self.supports_endpoint(EndpointType::Completion) {
            return Err(LlmError::UnsupportedEndpoint("completion".to_string()));
        }

        self.check_rate_limit().await?;

        debug!(
            "Groq complete request via OpenAI-compat: model={}",
            self.config.model
        );

        // Use the OpenAI-compatible client (converts to chat format internally)
        self.openai_client
            .complete(
                self.config.model.clone(),
                prompt,
                temperature,
                max_tokens,
                stop,
            )
            .await
    }

    async fn embed(
        &self,
        _input: Vec<String>,
        _extra_config: Option<HashMap<String, String>>,
    ) -> Result<EmbeddingResponse, LlmError> {
        // Groq doesn't currently support embeddings via their API
        Err(LlmError::NotSupported)
    }

    /// List available Groq models (static list)
    ///
    /// Returns a curated list of commonly available Groq models.
    /// See <https://console.groq.com/docs/models> for the current list.
    async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError> {
        // Static list of known Groq models
        // These are the commonly available models as of early 2025
        let models = vec![
            ModelInfo {
                name: "llama-3.3-70b-versatile".to_string(),
            },
            ModelInfo {
                name: "llama-3.1-70b-versatile".to_string(),
            },
            ModelInfo {
                name: "llama-3.1-8b-instant".to_string(),
            },
            ModelInfo {
                name: "llama-3.2-90b-vision-preview".to_string(),
            },
            ModelInfo {
                name: "llama-3.2-11b-vision-preview".to_string(),
            },
            ModelInfo {
                name: "mixtral-8x7b-32768".to_string(),
            },
            ModelInfo {
                name: "gemma2-9b-it".to_string(),
            },
        ];
        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::openai_compat::OpenAiChatResponse;

    #[test]
    fn test_provider_creation_with_api_key() {
        let mut config_map = HashMap::new();
        config_map.insert("api_key".to_string(), "test-api-key".to_string());

        let config = ModelConfig {
            name: "test-groq".to_string(),
            model: "llama-3.3-70b-versatile".to_string(),
            provider: "groq".to_string(),
            supported_endpoints: vec![EndpointType::Chat],
            config: Some(config_map),
            limit: None,
        };

        let provider = GroqProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_creation_fails_without_api_key() {
        let config = ModelConfig {
            name: "test-groq".to_string(),
            model: "llama-3.3-70b-versatile".to_string(),
            provider: "groq".to_string(),
            supported_endpoints: vec![EndpointType::Chat],
            config: Some(HashMap::new()),
            limit: None,
        };

        let provider = GroqProvider::new(config);
        assert!(provider.is_err());

        if let Err(LlmError::ConfigError(msg)) = provider {
            assert!(msg.contains("Missing api_key"));
        } else {
            panic!("Expected ConfigError");
        }
    }

    #[test]
    fn test_custom_base_url() {
        let mut config_map = HashMap::new();
        config_map.insert("api_key".to_string(), "test-key".to_string());
        config_map.insert(
            "base_url".to_string(),
            "https://custom.groq.com/v1".to_string(),
        );

        let config = ModelConfig {
            name: "test-groq".to_string(),
            model: "llama-3.3-70b-versatile".to_string(),
            provider: "groq".to_string(),
            supported_endpoints: vec![EndpointType::Chat],
            config: Some(config_map),
            limit: None,
        };

        // Should succeed with custom base URL
        let provider = GroqProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_parse_response_with_content_only() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "llama-3.3-70b-versatile",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello, world!"
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
            Some("Hello, world!".to_string())
        );
        assert!(response.choices[0].message.tool_calls.is_none());
    }

    #[test]
    fn test_parse_response_with_tool_calls_no_content() {
        let json = r#"{
            "id": "chatcmpl-456",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "llama-3.3-70b-versatile",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "tool_calls": [{
                        "id": "call_abc123",
                        "function": {
                            "name": "get_weather",
                            "arguments": {"location": "San Francisco"}
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
        let tool_calls = response.choices[0].message.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "get_weather");
    }

    #[test]
    fn test_parse_response_with_content_and_tool_calls() {
        let json = r#"{
            "id": "chatcmpl-789",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "llama-3.3-70b-versatile",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "I'll check the weather for you.",
                    "tool_calls": [{
                        "id": "call_def456",
                        "function": {
                            "name": "get_weather",
                            "arguments": {"location": "New York"}
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 25,
                "completion_tokens": 15,
                "total_tokens": 40
            }
        }"#;

        let response: OpenAiChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            response.choices[0].message.content,
            Some("I'll check the weather for you.".to_string())
        );
        assert!(response.choices[0].message.tool_calls.is_some());
    }

    #[test]
    fn test_parse_response_with_null_content() {
        let json = r#"{
            "id": "chatcmpl-999",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "llama-3.3-70b-versatile",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_ghi789",
                        "function": {
                            "name": "create_character",
                            "arguments": {"name": "Barf"}
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 30,
                "completion_tokens": 20,
                "total_tokens": 50
            }
        }"#;

        let response: OpenAiChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices[0].message.content, None);
        assert!(response.choices[0].message.tool_calls.is_some());
    }
}
