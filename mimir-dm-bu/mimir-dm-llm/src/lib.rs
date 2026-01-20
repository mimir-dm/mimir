//! # Mimir LLM Provider Abstraction
//!
//! This crate provides a provider-agnostic abstraction layer for Large Language Models (LLMs).
//! It supports multiple endpoints (chat, completion, embedding) with configurable rate limiting
//! and provider-specific implementations.
//!
//! ## Features
//!
//! - **Provider abstraction**: Unified interface for different LLM providers
//! - **Rate limiting**: Configurable rate limiting with token bucket algorithm
//! - **Multiple endpoints**: Support for chat, completion, and embedding endpoints
//! - **Configuration**: YAML-based configuration system
//! - **Async support**: Full async/await support with tokio
//!
//! ## Quick Start
//!
//! ```rust
//! use mimir_dm_llm::{
//!     config::{ModelConfig, EndpointType},
//!     providers::ollama::OllamaProvider,
//!     traits::{LlmProvider, Message},
//! };
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create configuration
//! let mut config_map = HashMap::new();
//! config_map.insert("base_url".to_string(), "http://localhost:11434".to_string());
//!
//! let config = ModelConfig {
//!     name: "llama3".to_string(),
//!     supported_endpoints: vec![EndpointType::Chat, EndpointType::Embedding],
//!     provider: "ollama".to_string(),
//!     model: "llama3".to_string(),
//!     config: Some(config_map),
//!     limit: None,
//! };
//!
//! // Create provider
//! let provider = OllamaProvider::new(config)?;
//!
//! // Use chat endpoint
//! let messages = vec![Message {
//!     role: "user".to_string(),
//!     content: "Hello, world!".to_string(),
//!     tool_call_id: None,
//! }];
//!
//! let response = provider.chat(messages, None, None, None, None, None, None, None).await?;
//! println!("Response: {}", response.content);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

pub mod config;
/// LLM provider implementations.
pub mod providers;
/// Tool implementations for LLM function calling.
pub mod tools;
pub mod traits;

// Re-export commonly used types from config
pub use config::{
    ConfigError, EndpointType, FileToolsConfig, ModelConfig, RateLimit, RenewalPeriod,
};

// Re-export provider trait and types
pub use traits::provider::{
    ChatResponse, CompletionResponse, EmbeddingResponse, LlmError, LlmProvider, Message, ModelInfo,
    ModelPullProgress, RateLimitState, Timing, Tool, ToolCall, ToolCallFunction, ToolFunction,
    Usage,
};

// Re-export tool trait
pub use traits::ToolTrait;

// Re-export tools
pub use tools::{
    EditFileTool, ListFilesTool, ReadFileTool, TodoItem, TodoListTool, TodoStateManager,
    WriteFileTool,
};
