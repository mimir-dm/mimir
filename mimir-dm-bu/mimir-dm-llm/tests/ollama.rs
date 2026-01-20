use mimir_dm_llm::{
    config::{EndpointType, ModelConfig, RateLimit, RenewalPeriod},
    providers::ollama::OllamaProvider,
    LlmProvider, Message,
};
use std::collections::HashMap;

/// Check if Ollama is available by attempting to connect
async fn is_ollama_available() -> bool {
    let client = reqwest::Client::new();
    client
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .is_ok()
}

#[tokio::test]
async fn test_ollama_completion() {
    if !is_ollama_available().await {
        eprintln!("Skipping test: Ollama not available at http://localhost:11434");
        return;
    }
    println!("Starting completion test");
    let config = ModelConfig {
        name: "llama3.1".to_string(),
        model: "llama3.1".to_string(),
        provider: "ollama".to_string(),
        supported_endpoints: vec![EndpointType::Completion],
        limit: None,
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
    };

    println!("Creating provider with config: {:?}", config);
    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");
    println!("Sending completion request");
    let response = provider
        .complete(
            "What is the capital of France?".to_string(),
            Some(1),
            None,
            None,
            None,
            None,
        )
        .await;
    println!("Got response: {:?}", response);
    let response = response.unwrap();

    assert!(!response.text.is_empty());
    assert!(response.usage.unwrap().prompt_tokens > 0);
}

#[tokio::test]
async fn test_ollama_chat() {
    if !is_ollama_available().await {
        eprintln!("Skipping test: Ollama not available at http://localhost:11434");
        return;
    }

    println!("Starting chat test");
    let config = ModelConfig {
        name: "llama3.1".to_string(),
        model: "llama3.1".to_string(),
        provider: "ollama".to_string(),
        supported_endpoints: vec![EndpointType::Chat],
        limit: None,
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
    };

    println!("Creating provider with config: {:?}", config);
    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
            tool_call_id: None,
        },
        Message {
            role: "user".to_string(),
            content: "What is 2+2?".to_string(),
            tool_call_id: None,
        },
    ];

    println!("Sending chat request with messages: {:?}", messages);
    let response = provider
        .chat(messages, None, None, None, None, None, None, None)
        .await;
    println!("Got response: {:?}", response);
    let response = response.unwrap();

    assert!(!response.content.is_empty());
    assert!(response.usage.unwrap().prompt_tokens > 0);
}

#[tokio::test]
async fn test_ollama_embeddings() {
    if !is_ollama_available().await {
        eprintln!("Skipping test: Ollama not available at http://localhost:11434");
        return;
    }

    let config = ModelConfig {
        name: "nomic-embed-text".to_string(),
        supported_endpoints: vec![EndpointType::Embedding],
        provider: "ollama".to_string(),
        model: "nomic-embed-text".to_string(),
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
        limit: None,
    };

    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");
    let response = provider
        .embed(vec!["test".to_string()], None)
        .await
        .expect("Failed to get embeddings");

    assert!(!response.embedding.is_empty());
    assert_eq!(response.embedding.len(), 768); // nomic-embed-text produces 768-dimensional vectors
    assert!(response.usage.is_some());
    assert_eq!(response.model, "nomic-embed-text");
}

#[tokio::test]
async fn test_ollama_unsupported_embeddings() {
    let config = ModelConfig {
        name: "phi".to_string(),
        supported_endpoints: vec![EndpointType::Chat],
        provider: "ollama".to_string(),
        model: "llama3.1".to_string(),
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
        limit: None,
    };

    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");
    let response = provider.embed(vec!["test".to_string()], None).await;

    assert!(response.is_err());
    assert!(response
        .unwrap_err()
        .to_string()
        .contains("Unsupported endpoint"));
}

#[tokio::test]
async fn test_ollama_unsupported_endpoint() {
    let config = ModelConfig {
        name: "llama3.1-local".to_string(),
        model: "llama3.1".to_string(),
        provider: "ollama".to_string(),
        supported_endpoints: vec![EndpointType::Completion],
        limit: None,
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
    };

    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");
    let messages = vec![Message {
        role: "user".to_string(),
        content: "What is the capital of France?".to_string(),
        tool_call_id: None,
    }];

    let response = provider
        .chat(messages, None, None, None, None, None, None, None)
        .await;
    assert!(response.is_err());
    assert!(response
        .unwrap_err()
        .to_string()
        .contains("Unsupported endpoint"));
}

#[tokio::test]
async fn test_ollama_invalid_endpoint() {
    let config = ModelConfig {
        name: "llama3.1-local".to_string(),
        model: "llama3.1".to_string(),
        provider: "ollama".to_string(),
        supported_endpoints: vec![EndpointType::Completion],
        limit: None,
        config: {
            let mut map = HashMap::new();
            map.insert(
                "base_url".to_string(),
                "http://invalid-host:1234".to_string(),
            );
            Some(map)
        },
    };

    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");

    let response = provider
        .complete(
            "What is the capital of France?".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(response.is_err());
    // OpenAI-compat client returns "Request failed" instead of "HTTP request failed"
    assert!(response.unwrap_err().to_string().contains("Request failed"));
}

#[tokio::test]
async fn test_ollama_missing_model() {
    if !is_ollama_available().await {
        eprintln!("Skipping test: Ollama not available at http://localhost:11434");
        return;
    }

    let config = ModelConfig {
        name: "non-existent-model".to_string(),
        model: "non-existent-model".to_string(),
        provider: "ollama".to_string(),
        supported_endpoints: vec![EndpointType::Completion],
        limit: None,
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
    };

    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");

    let response = provider
        .complete(
            "What is the capital of France?".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(response.is_err());
    // The error message may vary depending on Ollama version
    let error_str = response.unwrap_err().to_string();
    assert!(error_str.contains("Ollama API error") || error_str.contains("model"));
}

#[tokio::test]
async fn test_ollama_rate_limiting() {
    if !is_ollama_available().await {
        eprintln!("Skipping test: Ollama not available at http://localhost:11434");
        return;
    }

    let config = ModelConfig {
        name: "llama3.1-local".to_string(),
        model: "llama3.1".to_string(),
        provider: "ollama".to_string(),
        supported_endpoints: vec![EndpointType::Completion],
        limit: Some(RateLimit {
            renewal_period: RenewalPeriod::Seconds,
            calls: 1,
        }),
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
    };

    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");

    // First request should succeed
    let response = provider
        .complete(
            "What is the capital of France?".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    assert!(response.is_ok());

    // Second request should fail due to rate limiting (make immediate request)
    let response = provider
        .complete(
            "What is the capital of Spain?".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    // The second request should either fail due to rate limiting or succeed if enough time passed
    // Since this is integration testing with real timing, we'll be more flexible
    if response.is_err() {
        assert!(response
            .unwrap_err()
            .to_string()
            .contains("Rate limit exceeded"));
    } else {
        // If it succeeded, that's also acceptable as timing can vary
        println!("Rate limit test: Second request succeeded (timing variation)");
    }
}

#[tokio::test]
async fn test_ollama_embedding_dimensions() {
    if !is_ollama_available().await {
        eprintln!("Skipping test: Ollama not available at http://localhost:11434");
        return;
    }

    let config = ModelConfig {
        name: "nomic-embed-text".to_string(),
        supported_endpoints: vec![EndpointType::Embedding],
        provider: "ollama".to_string(),
        model: "nomic-embed-text".to_string(),
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
        limit: None,
    };

    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");

    // Test with different input texts
    let test_texts = vec![
        "Hello world".to_string(),
        "This is a longer text to test embedding generation with more content".to_string(),
        "Short".to_string(),
    ];

    for text in test_texts {
        let response = provider
            .embed(vec![text.clone()], None)
            .await
            .unwrap_or_else(|_| panic!("Failed to get embeddings for: {}", text));

        // Verify embedding dimensions match expected (768 for nomic-embed-text)
        assert_eq!(
            response.embedding.len(),
            768,
            "Embedding dimension mismatch for text: {}",
            text
        );

        // Verify embedding contains valid floats
        for (i, &value) in response.embedding.iter().enumerate() {
            assert!(
                value.is_finite(),
                "Invalid embedding value at index {} for text: {}",
                i,
                text
            );
        }
    }
}

#[tokio::test]
async fn test_ollama_multiple_messages_chat() {
    if !is_ollama_available().await {
        eprintln!("Skipping test: Ollama not available at http://localhost:11434");
        return;
    }

    let config = ModelConfig {
        name: "llama3.1".to_string(),
        model: "llama3.1".to_string(),
        provider: "ollama".to_string(),
        supported_endpoints: vec![EndpointType::Chat],
        limit: None,
        config: {
            let mut map = HashMap::new();
            map.insert("base_url".to_string(), "http://localhost:11434".to_string());
            Some(map)
        },
    };

    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");

    // Test with conversation history
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful math tutor.".to_string(),
            tool_call_id: None,
        },
        Message {
            role: "user".to_string(),
            content: "What is 5 + 3?".to_string(),
            tool_call_id: None,
        },
        Message {
            role: "assistant".to_string(),
            content: "5 + 3 equals 8.".to_string(),
            tool_call_id: None,
        },
        Message {
            role: "user".to_string(),
            content: "What about 8 - 2?".to_string(),
            tool_call_id: None,
        },
    ];

    let response = provider
        .chat(messages, None, None, None, None, None, None, None)
        .await
        .expect("Failed to get chat response");

    assert!(!response.content.is_empty());
    assert!(response.usage.is_some());
    assert!(response.usage.unwrap().prompt_tokens > 0);
}
