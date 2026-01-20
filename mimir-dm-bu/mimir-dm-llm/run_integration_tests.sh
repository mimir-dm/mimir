#!/bin/bash
set -e

echo "🚀 Running Mimir LLM Integration Tests"
echo "======================================"

# Check if Ollama is running
if ! curl -s http://localhost:11434/api/version > /dev/null; then
    echo "❌ Ollama is not running on localhost:11434"
    echo "   Please start Ollama with: ollama serve"
    echo "   Then install required models:"
    echo "   ollama pull llama3.1"
    echo "   ollama pull nomic-embed-text"
    exit 1
fi

echo "✅ Ollama is running"

# Check if required models are available
echo "🔍 Checking for required models..."

if ! ollama list | grep -q "llama3.1"; then
    echo "❌ Model 'llama3.1' not found"
    echo "   Install with: ollama pull llama3.1"
    exit 1
fi

if ! ollama list | grep -q "nomic-embed-text"; then
    echo "❌ Model 'nomic-embed-text' not found"
    echo "   Install with: ollama pull nomic-embed-text"
    exit 1
fi

echo "✅ Required models found"

# Run the integration tests
echo "🧪 Running integration tests..."
echo ""

if [ "$1" = "--verbose" ] || [ "$1" = "-v" ]; then
    cargo test --test main -- --nocapture
else
    cargo test --test main
fi

echo ""
echo "🎉 Integration tests completed successfully!"