# Integration Tests for mimir-dm-llm

This directory contains integration tests for the LLM provider abstraction layer.

## Prerequisites

### Running Ollama

The integration tests require a running Ollama instance on `localhost:11434` with the following models installed:

```bash
# Install Ollama (if not already installed)
curl -fsSL https://ollama.ai/install.sh | sh

# Start Ollama service
ollama serve

# In another terminal, pull required models
ollama pull llama3.1
ollama pull nomic-embed-text
```

### Verify Models are Available

You can verify the models are available by running:

```bash
ollama list
```

You should see both `llama3.1` and `nomic-embed-text` in the list.

## Running the Tests

### Run All Integration Tests

```bash
# From the workspace root
cargo test --package mimir-dm-llm --test main

# Or from the mimir-dm-llm directory
cd crates/mimir-dm-llm
cargo test --test main
```

### Run Specific Test Categories

```bash
# Run only chat tests
cargo test --package mimir-dm-llm --test main test_ollama_chat

# Run only embedding tests  
cargo test --package mimir-dm-llm --test main test_ollama_embedding

# Run only completion tests
cargo test --package mimir-dm-llm --test main test_ollama_completion
```

### Run with Output

```bash
# See test output (useful for debugging)
cargo test --package mimir-dm-llm --test main -- --nocapture
```

## Test Coverage

The integration tests cover:

### Core Functionality
- ✅ **Chat endpoint**: Multi-turn conversations with llama3.1 model
- ✅ **Completion endpoint**: Text completion with llama3.1 model  
- ✅ **Embedding endpoint**: Vector generation with nomic-embed-text model

### Error Handling
- ✅ **Unsupported endpoints**: Proper error when endpoint not configured
- ✅ **Invalid URLs**: Network error handling
- ✅ **Missing models**: Ollama API error handling
- ✅ **Rate limiting**: Token bucket rate limiting enforcement

### Advanced Features
- ✅ **Multi-message chat**: Conversation history handling
- ✅ **Embedding dimensions**: Verify 768-dimensional vectors
- ✅ **Usage tracking**: Token count validation

## Test Models

### llama3.1
- **Purpose**: Chat and completion testing
- **Size**: ~4.7GB
- **Use**: General text generation, conversation

### nomic-embed-text  
- **Purpose**: Embedding generation testing
- **Size**: ~274MB
- **Output**: 768-dimensional vectors
- **Use**: Semantic similarity, vector search

## Troubleshooting

### Ollama Not Running
```
Error: HTTP request failed: error sending request
```
**Solution**: Start Ollama with `ollama serve`

### Model Not Found
```
Error: Ollama API error: 404 Not Found
```
**Solution**: Pull the required model with `ollama pull <model-name>`

### Rate Limiting Test Flaky
The rate limiting test may occasionally fail if run too quickly in succession.
**Solution**: Run tests individually or increase the rate limit window.

### Connection Timeout
If tests timeout, the models may still be loading into memory.
**Solution**: Wait a few moments and retry, or increase timeout in test configuration.

## Configuration

Test configuration is stored in `ollama_config.yaml` and includes:
- Model definitions for different test scenarios  
- Rate limiting configurations
- Base URL settings

## Performance Notes

- **First run**: May be slower as models load into memory
- **Subsequent runs**: Faster as models stay cached
- **Parallel execution**: Tests run sequentially to avoid rate limits
- **Resource usage**: Requires ~4GB RAM for both models loaded