// Test utilities - not all are currently used but kept for future test development
#![allow(dead_code)]

use mimir_dm_llm::{
    config::{EndpointType, ModelConfig, RateLimit, RenewalPeriod},
    providers::ollama::OllamaProvider,
    LlmProvider,
};
use std::collections::HashMap;

/// Create a basic test configuration for Ollama
pub fn create_ollama_config(model: &str, endpoints: Vec<EndpointType>) -> ModelConfig {
    let mut config_map = HashMap::new();
    config_map.insert("base_url".to_string(), get_ollama_base_url());

    ModelConfig {
        name: format!("{}-test", model),
        supported_endpoints: endpoints,
        provider: "ollama".to_string(),
        model: model.to_string(),
        config: Some(config_map),
        limit: None,
    }
}

/// Create a rate-limited test configuration
pub fn create_rate_limited_config(
    model: &str,
    calls_per_period: u32,
    period: RenewalPeriod,
) -> ModelConfig {
    let mut config = create_ollama_config(model, vec![EndpointType::Chat]);
    config.limit = Some(RateLimit {
        renewal_period: period,
        calls: calls_per_period,
    });
    config
}

/// Get the Ollama base URL from environment or use default
pub fn get_ollama_base_url() -> String {
    std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string())
}

/// Check if Ollama service is available
pub async fn is_ollama_available() -> bool {
    let config = create_ollama_config("test", vec![]);
    if let Ok(provider) = OllamaProvider::new(config) {
        provider.check_service().await.unwrap_or(false)
    } else {
        false
    }
}

/// Skip test if Ollama is not available
#[macro_export]
macro_rules! require_ollama {
    () => {
        if !$crate::common::is_ollama_available().await {
            println!("Skipping test - Ollama service not available");
            return;
        }
    };
}

// Note: load_test_config functionality removed - use create_ollama_config directly
// If you need to load from YAML file, consider using ModelConfig::from_yaml_file directly

/// Helper to create a test provider with standard configuration
pub fn create_test_provider(model: &str) -> Result<OllamaProvider, mimir_dm_llm::LlmError> {
    let config = create_ollama_config(
        model,
        vec![
            EndpointType::Chat,
            EndpointType::Completion,
            EndpointType::Embedding,
        ],
    );
    OllamaProvider::new(config).map_err(|e| mimir_dm_llm::LlmError::ConfigError(e.to_string()))
}

/// Test data for common scenarios
pub mod test_data {
    use mimir_dm_llm::Message;

    /// Create a simple chat conversation
    pub fn simple_chat_messages() -> Vec<Message> {
        vec![Message {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
            tool_call_id: None,
        }]
    }

    /// Create a multi-turn conversation
    pub fn multi_turn_conversation() -> Vec<Message> {
        vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
                tool_call_id: None,
            },
            Message {
                role: "user".to_string(),
                content: "What is 2 + 2?".to_string(),
                tool_call_id: None,
            },
            Message {
                role: "assistant".to_string(),
                content: "2 + 2 equals 4.".to_string(),
                tool_call_id: None,
            },
            Message {
                role: "user".to_string(),
                content: "What about 3 + 3?".to_string(),
                tool_call_id: None,
            },
        ]
    }

    /// Test texts for embeddings
    pub fn embedding_test_texts() -> Vec<String> {
        vec![
            "The quick brown fox jumps over the lazy dog".to_string(),
            "Machine learning is a subset of artificial intelligence".to_string(),
            "Rust is a systems programming language".to_string(),
        ]
    }
}

/// Assertion helpers
pub mod assertions {
    use mimir_dm_llm::{ChatResponse, CompletionResponse, EmbeddingResponse};

    /// Assert that a chat response is valid
    pub fn assert_valid_chat_response(response: &ChatResponse) {
        assert!(
            !response.content.is_empty(),
            "Chat response should not be empty"
        );
        assert!(!response.model.is_empty(), "Model name should be specified");

        if let Some(usage) = &response.usage {
            assert!(usage.prompt_tokens > 0, "Should have prompt tokens");
            assert!(
                usage.total_tokens >= usage.prompt_tokens,
                "Total tokens should include prompt tokens"
            );
        }
    }

    /// Assert that a completion response is valid
    pub fn assert_valid_completion_response(response: &CompletionResponse) {
        assert!(
            !response.text.is_empty(),
            "Completion text should not be empty"
        );
        assert!(!response.model.is_empty(), "Model name should be specified");

        if let Some(usage) = &response.usage {
            assert!(usage.prompt_tokens > 0, "Should have prompt tokens");
            assert!(
                usage.total_tokens >= usage.prompt_tokens,
                "Total tokens should include prompt tokens"
            );
        }
    }

    /// Assert that an embedding response is valid
    pub fn assert_valid_embedding_response(
        response: &EmbeddingResponse,
        expected_dim: Option<usize>,
    ) {
        assert!(
            !response.embedding.is_empty(),
            "Embedding should not be empty"
        );
        assert!(!response.model.is_empty(), "Model name should be specified");

        // Check all values are finite
        for value in &response.embedding {
            assert!(value.is_finite(), "Embedding values should be finite");
        }

        // Check expected dimension if provided
        if let Some(dim) = expected_dim {
            assert_eq!(
                response.embedding.len(),
                dim,
                "Embedding dimension mismatch"
            );
        }
    }
}
