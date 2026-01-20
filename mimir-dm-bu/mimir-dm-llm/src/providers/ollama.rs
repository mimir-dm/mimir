//! # Ollama Provider
//!
//! This module provides an implementation of the [`LlmProvider`] trait for the Ollama API.
//! Uses OpenAI-compatible endpoints for chat and completion, with Ollama-specific endpoints
//! for model management (list, pull, check).
//!
//! ## Configuration
//!
//! The Ollama provider requires the following configuration:
//!
//! ```yaml
//! name: "llama3"
//! model: "llama3"
//! provider: "ollama"
//! supported_endpoints: ["chat", "completion", "embedding"]
//! config:
//!   base_url: "http://localhost:11434"
//! limit:
//!   renewal_period: "minutes"
//!   calls: 60
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use url::Url;

use crate::config::{EndpointType, ModelConfig};
use crate::providers::openai_compat::{OpenAiChatRequest, OpenAiCompatClient, OpenAiMessage};
use crate::traits::{
    ChatResponse, CompletionResponse, EmbeddingResponse, LlmError, LlmProvider, Message, ModelInfo,
    ModelPullProgress, RateLimitState, Tool, Usage,
};

// Note: Chat and completion now use OpenAI-compatible endpoint via OpenAiCompatClient.
// The following types are only used for Ollama-specific endpoints (embeddings, model management).

/// Ollama embedding request (Ollama-specific /api/embeddings endpoint)
#[derive(Debug, Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

/// Ollama embedding response
#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

/// Response from Ollama's /api/tags endpoint
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

/// Represents a model available in Ollama
#[derive(Debug, Clone, Deserialize)]
struct OllamaModel {
    name: String,
    #[allow(dead_code)]
    digest: String,
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    modified_at: String,
}

/// Request to pull a model
#[derive(Debug, Serialize)]
struct OllamaPullRequest {
    name: String,
    stream: bool,
}

/// Response from model pull (streaming)
#[derive(Debug, Deserialize)]
struct OllamaPullStreamResponse {
    status: String,
    #[serde(default)]
    #[allow(dead_code)]
    digest: String,
    #[serde(default)]
    total: u64,
    #[serde(default)]
    completed: u64,
}

/// Ollama provider implementation
///
/// Uses OpenAI-compatible endpoints for chat and completion (/v1/chat/completions),
/// with Ollama-specific endpoints for model management (/api/tags, /api/pull).
pub struct OllamaProvider {
    config: ModelConfig,
    rate_limit_state: RateLimitState,
    /// OpenAI-compatible client for chat/completion
    openai_client: OpenAiCompatClient,
    /// HTTP client for Ollama-specific endpoints (model management, embeddings)
    client: reqwest::Client,
    /// Base URL for Ollama API (e.g., "http://localhost:11434")
    base_url: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(config: ModelConfig) -> Result<Self, LlmError> {
        // Get base_url from config and normalize it (remove trailing slash)
        let base_url = config
            .config
            .as_ref()
            .and_then(|c| c.get("base_url"))
            .ok_or_else(|| LlmError::ConfigError("Missing base_url in config".to_string()))?
            .trim_end_matches('/')
            .to_string();

        // Validate URL
        let _url = Url::parse(&base_url)
            .map_err(|e| LlmError::ConfigError(format!("Invalid base_url: {}", e)))?;

        let rate_limit_state = config
            .limit
            .as_ref()
            .map_or_else(RateLimitState::default, RateLimitState::new);

        // Create OpenAI-compatible client for chat/completion
        // Ollama's OpenAI-compatible endpoint is at /v1
        let openai_base_url = format!("{}/v1", base_url);
        let openai_client = OpenAiCompatClient::new(openai_base_url, None, 300)?;

        // Create standard HTTP client for Ollama-specific endpoints
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(|e| LlmError::ProviderError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            rate_limit_state,
            openai_client,
            client,
            base_url,
        })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
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
            "Ollama chat request via OpenAI-compat: model={} messages={}",
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
            "Ollama complete request via OpenAI-compat: model={}",
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
        input: Vec<String>,
        _extra_config: Option<HashMap<String, String>>,
    ) -> Result<EmbeddingResponse, LlmError> {
        if !self.supports_endpoint(EndpointType::Embedding) {
            return Err(LlmError::UnsupportedEndpoint("embedding".to_string()));
        }

        self.check_rate_limit().await?;

        // For multiple inputs, we'll concatenate them with spaces
        // This is a simplification - in production you might want to handle this differently
        let text = input.join(" ");

        let request = OllamaEmbeddingRequest {
            model: self.config.model.clone(),
            prompt: text.clone(),
        };

        let response = self
            .client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::ProviderError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(LlmError::ProviderError(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ProviderError(format!("JSON parsing failed: {}", e)))?;

        // Estimate token usage based on text length (rough approximation)
        let estimated_tokens = (text.len() as f32 / 4.0).ceil() as u32;

        let usage = Some(Usage {
            prompt_tokens: estimated_tokens,
            completion_tokens: 0,
            total_tokens: estimated_tokens,
        });

        Ok(EmbeddingResponse {
            embedding: ollama_response.embedding,
            usage,
            timing: None, // Ollama embeddings don't return timing info
            model: self.config.model.clone(),
        })
    }

    // Model Management implementations

    /// Check if the Ollama service is available and responding
    ///
    /// Returns `Ok(true)` if the service is running and accessible,
    /// `Ok(false)` if the service is not responding.
    ///
    /// # Example
    /// ```no_run
    /// # use mimir_dm_llm::{providers::ollama::OllamaProvider, LlmProvider};
    /// # async fn example(provider: OllamaProvider) -> Result<(), Box<dyn std::error::Error>> {
    /// if provider.check_service().await? {
    ///     println!("Ollama is running");
    /// } else {
    ///     println!("Ollama is not available");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn check_service(&self) -> Result<bool, LlmError> {
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false), // Service not running
        }
    }

    /// List all models available in the Ollama service
    ///
    /// Returns a vector of `ModelInfo` containing the names of all locally available models.
    ///
    /// # Errors
    /// Returns an error if the service is not available or if the response cannot be parsed.
    ///
    /// # Example
    /// ```no_run
    /// # use mimir_dm_llm::{providers::ollama::OllamaProvider, LlmProvider};
    /// # async fn example(provider: OllamaProvider) -> Result<(), Box<dyn std::error::Error>> {
    /// let models = provider.list_models().await?;
    /// for model in models {
    ///     println!("Available model: {}", model.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn list_models(&self) -> Result<Vec<ModelInfo>, LlmError> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            LlmError::ProviderError(format!(
                "Failed to list models from {}: {}",
                self.base_url, e
            ))
        })?;

        if !response.status().is_success() {
            return Err(LlmError::ServiceUnavailable(format!(
                "Ollama service at {} returned status: {}",
                self.base_url,
                response.status()
            )));
        }

        let tags_response: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Failed to parse model list: {}", e)))?;

        Ok(tags_response
            .models
            .into_iter()
            .map(|m| ModelInfo { name: m.name })
            .collect())
    }

    /// Check if a specific model exists locally in Ollama
    ///
    /// This method checks if a model with the given name (or starting with the given name)
    /// exists in the local Ollama installation. It handles partial matches, so "llama3.1"
    /// will match "llama3.1:latest" or "llama3.1-instruct".
    ///
    /// # Arguments
    /// * `model_name` - The name of the model to check for
    ///
    /// # Example
    /// ```no_run
    /// # use mimir_dm_llm::{providers::ollama::OllamaProvider, LlmProvider};
    /// # async fn example(provider: OllamaProvider) -> Result<(), Box<dyn std::error::Error>> {
    /// if provider.model_exists("llama3.1").await? {
    ///     println!("Model is available locally");
    /// } else {
    ///     println!("Model needs to be pulled");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn model_exists(&self, model_name: &str) -> Result<bool, LlmError> {
        let models = self.list_models().await?;
        // Check if any model name starts with the requested model
        // This handles cases like "qwen2:8b" matching "qwen2:8b-instruct-q4_0"
        Ok(models.iter().any(|m| m.name.starts_with(model_name)))
    }

    /// Pull (download) a model from the Ollama library
    ///
    /// This method downloads a model from the Ollama library if it's not already available locally.
    /// The download happens synchronously (blocking until complete).
    ///
    /// # Arguments
    /// * `model_name` - The name of the model to pull (e.g., "llama3.1", "mistral:latest")
    ///
    /// # Errors
    /// Returns an error if the model cannot be found or if the download fails.
    ///
    /// # Example
    /// ```no_run
    /// # use mimir_dm_llm::{providers::ollama::OllamaProvider, LlmProvider};
    /// # async fn example(provider: OllamaProvider) -> Result<(), Box<dyn std::error::Error>> {
    /// provider.pull_model("tinyllama").await?;
    /// println!("Model downloaded successfully");
    /// # Ok(())
    /// # }
    /// ```
    async fn pull_model(&self, model_name: &str) -> Result<(), LlmError> {
        let url = format!("{}/api/pull", self.base_url);

        let request = OllamaPullRequest {
            name: model_name.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                LlmError::ProviderError(format!(
                    "Failed to pull model '{}' from {}: {}",
                    model_name, self.base_url, e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ModelPullFailed(format!(
                "Failed to pull model '{}': HTTP {} - {}",
                model_name, status, error_text
            )));
        }

        // Wait for the response to complete (non-streaming)
        let _body = response
            .text()
            .await
            .map_err(|e| LlmError::ProviderError(format!("Failed to read pull response: {}", e)))?;

        Ok(())
    }

    /// Pull (download) a model with progress updates
    ///
    /// This method downloads a model from the Ollama library and provides progress updates
    /// through a callback function. This is useful for showing download progress to users.
    ///
    /// # Arguments
    /// * `model_name` - The name of the model to pull
    /// * `progress_callback` - A callback function that receives progress updates
    ///
    /// # Example
    /// ```no_run
    /// # use mimir_dm_llm::{providers::ollama::OllamaProvider, LlmProvider, ModelPullProgress};
    /// # async fn example(provider: OllamaProvider) -> Result<(), Box<dyn std::error::Error>> {
    /// provider.pull_model_with_progress("llama3.1", |progress: ModelPullProgress| {
    ///     println!("{}: {}/{} bytes", progress.status, progress.downloaded, progress.total);
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn pull_model_with_progress<F>(
        &self,
        model_name: &str,
        progress_callback: F,
    ) -> Result<(), LlmError>
    where
        F: Fn(ModelPullProgress) + Send + 'static,
    {
        let url = format!("{}/api/pull", self.base_url);

        let request = OllamaPullRequest {
            name: model_name.to_string(),
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                LlmError::ProviderError(format!(
                    "Failed to initiate pull for model '{}' from {}: {}",
                    model_name, self.base_url, e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(LlmError::ModelPullFailed(format!(
                "Failed to pull model '{}': HTTP {} from {}",
                model_name, status, self.base_url
            )));
        }

        // Process streaming response
        use futures::StreamExt;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|e| LlmError::ProviderError(format!("Stream error: {}", e)))?;

            // Parse each line as JSON (Ollama sends newline-delimited JSON)
            for line in chunk.split(|&b| b == b'\n') {
                if line.is_empty() {
                    continue;
                }

                match serde_json::from_slice::<OllamaPullStreamResponse>(line) {
                    Ok(progress_data) => {
                        let progress = ModelPullProgress {
                            status: progress_data.status.clone(),
                            downloaded: progress_data.completed,
                            total: progress_data.total,
                        };

                        progress_callback(progress);

                        // Check if done
                        if progress_data.status.contains("success")
                            || progress_data.status.contains("already exists")
                        {
                            return Ok(());
                        }
                    }
                    Err(_) => {
                        // Ignore parse errors for individual lines
                        // Ollama sends various status messages we don't need to parse
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EndpointType, ModelConfig};
    use std::collections::HashMap;

    fn create_test_config() -> ModelConfig {
        let mut config_map = HashMap::new();
        config_map.insert("base_url".to_string(), "http://localhost:11434".to_string());

        ModelConfig {
            name: "test-ollama".to_string(),
            supported_endpoints: vec![
                EndpointType::Chat,
                EndpointType::Completion,
                EndpointType::Embedding,
            ],
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            config: Some(config_map),
            limit: None,
        }
    }

    #[test]
    fn test_ollama_provider_creation() {
        let config = create_test_config();
        let provider = OllamaProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_ollama_provider_invalid_url() {
        let mut config = create_test_config();
        config
            .config
            .as_mut()
            .unwrap()
            .insert("base_url".to_string(), "invalid-url".to_string());

        let provider = OllamaProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_ollama_provider_missing_base_url() {
        let mut config = create_test_config();
        config.config = None;

        let provider = OllamaProvider::new(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_supported_endpoints() {
        let config = create_test_config();
        let provider = OllamaProvider::new(config).unwrap();

        assert!(provider.supports_endpoint(EndpointType::Chat));
        assert!(provider.supports_endpoint(EndpointType::Completion));
        assert!(provider.supports_endpoint(EndpointType::Embedding));
    }
}
