use crate::common::create_ollama_config;
use mimir_dm_llm::{
    config::{EndpointType, ModelConfig},
    providers::ollama::OllamaProvider,
    LlmProvider, ModelPullProgress,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test_check_service() {
    let config = create_ollama_config("llama3.1", vec![EndpointType::Chat]);
    let provider = OllamaProvider::new(config).expect("Failed to create provider");

    // This test will pass if Ollama is running, fail if not
    let result = provider.check_service().await;

    // We can't assert a specific value since it depends on whether Ollama is running
    // But we can assert that the method doesn't panic
    match result {
        Ok(true) => println!("Ollama service is running"),
        Ok(false) => println!("Ollama service is not running"),
        Err(e) => println!("Error checking service: {}", e),
    }
}

#[tokio::test]
async fn test_list_models() {
    let config = create_ollama_config("llama3.1", vec![EndpointType::Chat]);
    let provider = OllamaProvider::new(config).expect("Failed to create provider");

    match provider.list_models().await {
        Ok(models) => {
            println!("Found {} models", models.len());
            for model in &models {
                println!("  - {}", model.name);
            }
            // Verify the result is a valid list (even if empty)
            assert!(models.is_empty() || models.iter().all(|m| !m.name.is_empty()));
        }
        Err(e) => {
            // If Ollama isn't running, this is expected
            println!("Could not list models (Ollama may not be running): {}", e);
        }
    }
}

#[tokio::test]
async fn test_model_exists() {
    let config = create_ollama_config("llama3.1", vec![EndpointType::Chat]);
    let provider = OllamaProvider::new(config).expect("Failed to create provider");

    // Test with a model that likely exists if Ollama is running
    match provider.model_exists("llama3.1").await {
        Ok(exists) => {
            println!("Model llama3.1 exists: {}", exists);
        }
        Err(e) => {
            println!("Could not check model existence: {}", e);
        }
    }

    // Test with a model that definitely doesn't exist
    match provider
        .model_exists("definitely-not-a-real-model-xyz123")
        .await
    {
        Ok(exists) => {
            assert!(!exists, "Non-existent model should not exist");
        }
        Err(_) => {
            // If Ollama isn't running, this is expected
            println!("Could not check model (Ollama may not be running)");
        }
    }
}

#[tokio::test]
async fn test_ensure_model() {
    let config = create_ollama_config("llama3.1", vec![EndpointType::Chat]);
    let provider = OllamaProvider::new(config).expect("Failed to create provider");

    // First check if service is available
    match provider.check_service().await {
        Ok(true) => {
            // Service is running, try to ensure a model
            match provider.ensure_model("llama3.1").await {
                Ok(()) => println!("Model ensured successfully"),
                Err(e) => println!("Could not ensure model: {}", e),
            }
        }
        Ok(false) | Err(_) => {
            // Service not available, ensure_model should fail
            let result = provider.ensure_model("llama3.1").await;
            assert!(result.is_err());
            if let Err(e) = result {
                assert!(e.to_string().contains("Service"));
            }
        }
    }
}

#[tokio::test]
async fn test_pull_model_with_progress() {
    let config = create_ollama_config("tinyllama", vec![EndpointType::Chat]); // Using a small model for testing
    let provider = OllamaProvider::new(config).expect("Failed to create provider");

    // Check if service is available first
    match provider.check_service().await {
        Ok(true) => {
            // Track progress updates
            let progress_updates = Arc::new(Mutex::new(Vec::new()));
            let progress_clone = progress_updates.clone();

            let result = provider
                .pull_model_with_progress("tinyllama", move |progress: ModelPullProgress| {
                    println!(
                        "Progress: {} - {}/{} bytes",
                        progress.status, progress.downloaded, progress.total
                    );
                    progress_clone.lock().unwrap().push(progress);
                })
                .await;

            match result {
                Ok(()) => {
                    let updates = progress_updates.lock().unwrap();
                    println!("Received {} progress updates", updates.len());

                    // If model was pulled, we should have received at least one update
                    if !updates.is_empty() {
                        // Check that we got meaningful progress updates
                        assert!(updates.iter().any(|p| !p.status.is_empty()));
                    }
                }
                Err(e) => {
                    // Model might already exist or other error
                    println!("Pull model result: {}", e);
                }
            }
        }
        Ok(false) | Err(_) => {
            println!("Skipping pull test - Ollama service not available");
        }
    }
}

#[tokio::test]
async fn test_model_management_with_invalid_service() {
    let mut config_map = HashMap::new();
    config_map.insert("base_url".to_string(), "http://localhost:65000".to_string()); // Likely unused port

    let config = ModelConfig {
        name: "test".to_string(),
        supported_endpoints: vec![EndpointType::Chat],
        provider: "ollama".to_string(),
        model: "test".to_string(),
        config: Some(config_map),
        limit: None,
    };

    let provider = OllamaProvider::new(config).expect("Failed to create provider");

    // All operations should fail gracefully with invalid service
    assert!(!provider.check_service().await.unwrap_or(true));
    assert!(provider.list_models().await.is_err());
    assert!(provider.model_exists("any").await.is_err());
    assert!(provider.pull_model("any").await.is_err());
    assert!(provider.ensure_model("any").await.is_err());
}

#[tokio::test]
async fn test_model_name_matching() {
    let config = create_ollama_config("llama3.1", vec![EndpointType::Chat]);
    let provider = OllamaProvider::new(config).expect("Failed to create provider");

    // The implementation should handle partial model names
    // e.g., "llama3.1" should match "llama3.1:latest" or "llama3.1-instruct"
    match provider.check_service().await {
        Ok(true) => {
            // Create a list of test cases
            let test_cases = vec![
                ("llama3.1", "Should handle base model name"),
                ("llama3.1:latest", "Should handle tagged versions"),
            ];

            for (model_name, description) in test_cases {
                match provider.model_exists(model_name).await {
                    Ok(exists) => {
                        println!("{}: {} exists: {}", description, model_name, exists);
                    }
                    Err(e) => {
                        println!("{}: Error checking {}: {}", description, model_name, e);
                    }
                }
            }
        }
        _ => {
            println!("Skipping model name matching test - Ollama not available");
        }
    }
}
