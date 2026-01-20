# mimir-dm-llm

## Purpose & Boundaries

The `mimir-dm-llm` crate provides a clean abstraction layer for integrating Large Language Models (LLMs) into the Mimir D&D Campaign Assistant. It focuses on providing a provider-agnostic interface while initially supporting Ollama for local LLM inference.

### Responsibilities

- **Provider Abstraction**: Unified interface for different LLM providers
- **Ollama Integration**: Full support for local Ollama deployments
- **Rate Limiting**: Token bucket algorithm for API rate management
- **Multiple Endpoints**: Chat, completion, and embedding support
- **Configuration Management**: YAML-based model configuration
- **Error Handling**: Comprehensive error types for LLM operations
- **Response Streaming**: Support for streaming responses (future)

### What This Crate Does NOT Do

- No model training or fine-tuning
- No model hosting (relies on external providers)
- No prompt engineering or templates (that's application logic)
- No conversation management or memory
- No D&D-specific logic (remains domain-agnostic)

## Architecture

### Provider System
The crate uses a trait-based provider system where each LLM provider implements the `LlmProvider` trait. This allows for easy addition of new providers while maintaining a consistent interface.

### Supported Endpoints
1. **Chat**: Multi-turn conversation with message history
2. **Completion**: Single-turn text generation
3. **Embedding**: Text-to-vector conversion for semantic search

## Layout

```
src/
├── lib.rs           # Crate root with public API and re-exports
├── config.rs        # Configuration structures and YAML parsing
├── embeddings.rs    # Embedding-specific utilities
├── traits/          # Core trait definitions
│   ├── mod.rs      # Trait module exports
│   ├── provider.rs  # LlmProvider trait and response types
│   ├── context.rs   # Context management traits
│   └── tool.rs      # Tool integration trait (ToolTrait)
└── providers/       # Provider implementations
    ├── mod.rs      # Provider module exports
    └── ollama.rs    # Ollama provider implementation
```

## Key Features

### Provider Abstraction
```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: Vec<Message>, ...) -> Result<ChatResponse>;
    async fn complete(&self, prompt: &str, ...) -> Result<CompletionResponse>;
    async fn embed(&self, text: &str) -> Result<EmbeddingResponse>;
    // ... configuration and capability methods
}
```

### Rate Limiting
- Token bucket algorithm implementation
- Configurable limits per model
- Automatic request throttling
- Never fails, only delays

### Configuration System
- YAML-based configuration files
- Per-model settings
- Provider-specific parameters
- Runtime reconfiguration support

## Configuration

Example configuration for Ollama:
```yaml
models:
  - name: "llama3"
    provider: "ollama"
    model: "llama3:latest"
    supported_endpoints:
      - chat
      - completion
      - embedding
    config:
      base_url: "http://localhost:11434"
    limit:
      max_calls: 10
      renewal_period: 
        seconds: 60

  - name: "nomic-embed"
    provider: "ollama"
    model: "nomic-embed-text"
    supported_endpoints:
      - embedding
    config:
      base_url: "http://localhost:11434"
```

## Usage

### Basic Setup
```rust
use mimir_dm_llm::{
    config::{ModelConfig, EndpointType},
    providers::ollama::OllamaProvider,
    provider::{LlmProvider, Message},
};

// Create configuration
let config = ModelConfig {
    name: "llama3".to_string(),
    provider: "ollama".to_string(),
    model: "llama3:latest".to_string(),
    supported_endpoints: vec![EndpointType::Chat],
    config: Some(config_map),
    limit: None,
};

// Create provider
let provider = OllamaProvider::new(config)?;
```

### Chat Completion
```rust
let messages = vec![
    Message {
        role: "system".to_string(),
        content: "You are a helpful D&D dungeon master.".to_string(),
    },
    Message {
        role: "user".to_string(),
        content: "Describe a mysterious tavern.".to_string(),
    },
];

let response = provider.chat(
    messages,
    Some(0.7),  // temperature
    None,       // max_tokens
    None,       // top_p
    None,       // frequency_penalty
    None,       // seed
).await?;

println!("DM says: {}", response.content);
```

### Text Embedding
```rust
let text = "The ancient dragon sleeps in its lair";
let response = provider.embed(text).await?;
let embedding = response.embedding; // Vec<f32>
```

## Provider Implementations

### Ollama (Current)
- Local LLM inference
- No API keys required
- Support for all Ollama models
- HTTP-based communication
- Automatic model pulling (optional)

### Future Providers
- OpenAI API
- Anthropic Claude
- Google Gemini
- AWS Bedrock
- Azure OpenAI

## Error Handling

The crate provides comprehensive error types:
```rust
pub enum LlmError {
    Configuration(String),
    Connection(String),
    RateLimit(String),
    InvalidResponse(String),
    UnsupportedEndpoint(String),
    ProviderError(String),
}
```

## Dependencies

- `tokio` - Async runtime
- `async-trait` - Async trait support
- `reqwest` - HTTP client for API calls
- `serde` & `serde_json` - Serialization
- `serde_yaml` - YAML configuration parsing
- `anyhow` & `thiserror` - Error handling

## Testing

Run tests with:
```bash
cargo test -p mimir-dm-llm
```

Integration tests require a running Ollama instance:
```bash
ollama serve
cargo test -p mimir-dm-llm --features integration-tests
```

## Design Principles

1. **Provider Agnostic**: Easy to add new LLM providers
2. **Configuration Driven**: Flexible runtime configuration
3. **Fail Gracefully**: Comprehensive error handling
4. **Performance First**: Efficient rate limiting and connection pooling
5. **Type Safe**: Strong typing for all API interactions
6. **Async Native**: Built on tokio for concurrent operations

## Current Capabilities

- **Provider Abstraction** - Unified interface supporting multiple LLM providers
- **Rate Limiting** - Token bucket algorithm with configurable limits
- **Multiple Endpoints** - Chat, completion, and embedding support
- **Ollama Integration** - Complete support for local Ollama deployments
- **Tool Support** - ToolTrait for function calling integration
- **Error Handling** - Comprehensive error types and graceful failure
- **Configuration** - YAML-based model and provider configuration
- **Async Design** - Full tokio async/await support