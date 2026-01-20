//! Integration tests for LLM service initialization

use mimir_dm_llm::{
    config::{EndpointType, ModelConfig},
    providers::groq::GroqProvider,
    providers::ollama::OllamaProvider,
    LlmProvider,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Test configuration for the required model
fn create_test_config() -> ModelConfig {
    let mut config_map = HashMap::new();
    config_map.insert("base_url".to_string(), "http://localhost:11434".to_string());

    ModelConfig {
        name: "qwen3:30b-dm".to_string(),
        supported_endpoints: vec![
            EndpointType::Chat,
            EndpointType::Completion,
            EndpointType::Embedding,
        ],
        provider: "ollama".to_string(),
        model: "qwen3:30b".to_string(),
        config: Some(config_map),
        limit: None,
    }
}

#[tokio::test]
async fn test_ollama_service_availability() {
    let config = create_test_config();
    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");

    let service_available = provider
        .check_service()
        .await
        .expect("Should be able to check service status");

    if !service_available {
        // Skip test if Ollama isn't running - this is acceptable for CI/CD
        eprintln!("WARNING: Skipping test - Ollama service not running");
        return;
    }

    assert!(service_available, "Ollama service should be available");
}

#[tokio::test]
async fn test_model_availability() {
    let config = create_test_config();
    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");

    // First check if service is available
    if !provider.check_service().await.unwrap_or(false) {
        eprintln!("WARNING: Skipping test - Ollama not running");
        return;
    }

    // Check if model exists - this MUST succeed
    let model_exists = provider
        .model_exists("qwen3:30b")
        .await
        .expect("Should be able to check model existence");

    assert!(
        model_exists,
        "Required model qwen3:30b must be available for tests. Please run: ollama pull qwen3:30b"
    );
}

#[tokio::test]
async fn test_model_ensure_without_download() {
    // This test checks the ensure_model logic without actually downloading
    let config = create_test_config();
    let provider = Arc::new(OllamaProvider::new(config).expect("Failed to create Ollama provider"));

    // First check if service is available
    if !provider.check_service().await.unwrap_or(false) {
        println!("Skipping test - Ollama not running");
        return;
    }

    // Check if we can ensure a small model (for testing)
    // Using tinyllama as it's small if it needs to be downloaded
    let test_config = {
        let mut config_map = HashMap::new();
        config_map.insert("base_url".to_string(), "http://localhost:11434".to_string());

        ModelConfig {
            name: "tinyllama-test".to_string(),
            supported_endpoints: vec![EndpointType::Chat],
            provider: "ollama".to_string(),
            model: "tinyllama".to_string(),
            config: Some(config_map),
            limit: None,
        }
    };

    let test_provider = OllamaProvider::new(test_config).expect("Failed to create test provider");

    match test_provider.model_exists("tinyllama").await {
        Ok(true) => {
            println!("✅ Test model (tinyllama) already exists");
        }
        Ok(false) => {
            println!("⚠️  Test model (tinyllama) would be downloaded");
            // Don't actually download in tests
        }
        Err(e) => {
            println!("Error checking test model: {}", e);
        }
    }
}

#[tokio::test]
async fn test_llm_service_initialization_flow() {
    // This simulates the initialization flow without the Tauri app
    use std::sync::Mutex;

    let config = create_test_config();
    let provider = Arc::new(OllamaProvider::new(config).expect("Failed to create Ollama provider"));

    // Check service - MUST be available
    let service_available = provider
        .check_service()
        .await
        .expect("Should be able to check service");

    if !service_available {
        eprintln!("WARNING: Skipping test - Ollama service not available");
        return;
    }

    assert!(service_available, "Ollama service must be running");

    // Check model - MUST exist
    let model_exists = provider
        .model_exists("qwen3:30b")
        .await
        .expect("Should be able to check model existence");

    assert!(
        model_exists,
        "Model qwen3:30b must exist for initialization flow test"
    );

    // Test progress tracking mechanism (without actual download)
    let progress_events = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = Arc::clone(&progress_events);

    // Simulate progress callback mechanism
    let sample_progress = vec![
        ("pulling manifest", 0, 0),
        ("downloading", 1024, 10240),
        ("downloading", 5120, 10240),
        ("downloading", 10240, 10240),
        ("success", 10240, 10240),
    ];

    for (status, downloaded, total) in sample_progress {
        let percentage = if total > 0 {
            (downloaded as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        if let Ok(mut events) = progress_clone.lock() {
            events.push(format!("{}: {:.1}%", status, percentage));
        }

        // Simulate download delay
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let events = progress_events.lock().unwrap();
    assert_eq!(events.len(), 5, "Should have tracked all 5 progress events");
    assert!(
        events.last().unwrap().contains("success"),
        "Last event should be success"
    );
}

#[tokio::test]
async fn test_list_available_models() {
    let config = create_test_config();
    let provider = OllamaProvider::new(config).expect("Failed to create Ollama provider");

    if !provider.check_service().await.unwrap_or(false) {
        eprintln!("WARNING: Skipping test - Ollama not running");
        return;
    }

    let models = provider
        .list_models()
        .await
        .expect("Should be able to list models when service is running");

    // Should have at least one model
    assert!(
        !models.is_empty(),
        "Ollama should have at least one model installed"
    );

    // Check if our required model is in the list
    let has_required_model = models.iter().any(|m| m.name.starts_with("qwen3:30b"));

    assert!(
        has_required_model,
        "Required model qwen3:30b must be in the available models list. Found: {:?}",
        models.iter().map(|m| &m.name).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_model_download_with_progress() {
    // This test will download a very small model to test the download functionality
    // We use smollm:135m which is a very small model (1/3 the size of qwen:0.5b)
    const TEST_MODEL: &str = "smollm:135m";

    let mut config_map = HashMap::new();
    config_map.insert("base_url".to_string(), "http://localhost:11434".to_string());

    let config = ModelConfig {
        name: "download-test".to_string(),
        supported_endpoints: vec![EndpointType::Chat],
        provider: "ollama".to_string(),
        model: TEST_MODEL.to_string(),
        config: Some(config_map),
        limit: None,
    };

    let provider = Arc::new(OllamaProvider::new(config).expect("Failed to create Ollama provider"));

    // Check service first
    if !provider.check_service().await.unwrap_or(false) {
        eprintln!("WARNING: Skipping test - Ollama not running");
        return;
    }

    // First, check if model already exists and delete it if it does
    // (so we can test the download from scratch)
    if provider.model_exists(TEST_MODEL).await.unwrap_or(false) {
        println!(
            "Model {} already exists, removing it to test download...",
            TEST_MODEL
        );

        let delete_result = tokio::process::Command::new("ollama")
            .arg("rm")
            .arg(TEST_MODEL)
            .output()
            .await;

        match delete_result {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!(
                        "Failed to remove existing model: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    panic!("Cannot proceed with download test - unable to remove existing model");
                }
                println!("Existing model removed, proceeding with download test");
            }
            Err(e) => {
                panic!("Could not run ollama rm command: {}", e);
            }
        }

        // Verify deletion worked
        let still_exists = provider.model_exists(TEST_MODEL).await.unwrap_or(true);
        assert!(
            !still_exists,
            "Model should be deleted before testing download"
        );
    }

    // Track progress updates
    let progress_updates = Arc::new(std::sync::Mutex::new(Vec::new()));
    let progress_clone = Arc::clone(&progress_updates);

    println!("Starting download of test model: {}", TEST_MODEL);

    // Download the model with progress tracking
    let result = provider
        .pull_model_with_progress(TEST_MODEL, move |progress| {
            let percentage = if progress.total > 0 {
                (progress.downloaded as f32 / progress.total as f32) * 100.0
            } else {
                0.0
            };

            println!(
                "Download progress: {} - {}/{} bytes ({:.1}%)",
                progress.status, progress.downloaded, progress.total, percentage
            );

            if let Ok(mut updates) = progress_clone.lock() {
                updates.push((progress.status.clone(), progress.downloaded, progress.total));
            }
        })
        .await;

    // Verify download succeeded
    assert!(
        result.is_ok(),
        "Model download should succeed: {:?}",
        result
    );

    // Verify we got progress updates (scope the lock to avoid holding across await)
    {
        let updates = progress_updates.lock().unwrap();
        assert!(
            !updates.is_empty(),
            "Should have received progress updates during download"
        );

        // Verify the last update indicates success
        let last_update = updates.last().expect("Should have at least one update");
        assert!(
            last_update.0.contains("success") || last_update.0.contains("already exists"),
            "Last update should indicate success, got: {}",
            last_update.0
        );
    }

    // Verify model now exists
    let model_exists = provider
        .model_exists(TEST_MODEL)
        .await
        .expect("Should be able to check model existence");
    assert!(model_exists, "Model should exist after successful download");

    // Clean up: Delete the test model
    println!("Cleaning up: Removing test model {}", TEST_MODEL);
    let cleanup_result = tokio::process::Command::new("ollama")
        .arg("rm")
        .arg(TEST_MODEL)
        .output()
        .await;

    match cleanup_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Test model {} removed successfully", TEST_MODEL);
            } else {
                eprintln!(
                    "Warning: Failed to remove test model: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not run cleanup command: {}", e);
        }
    }

    // Verify cleanup worked
    let model_still_exists = provider.model_exists(TEST_MODEL).await.unwrap_or(true);
    assert!(
        !model_still_exists,
        "Test model should be deleted after test"
    );

    println!("✅ Model download test completed and cleaned up successfully");
}

#[tokio::test]
async fn test_groq_list_models() {
    // Create Groq provider with a dummy API key (list_models doesn't need authentication)
    let mut config_map = HashMap::new();
    config_map.insert("api_key".to_string(), "test-api-key".to_string());

    let config = ModelConfig {
        name: "groq-test".to_string(),
        supported_endpoints: vec![EndpointType::Chat],
        provider: "groq".to_string(),
        model: "llama-3.3-70b-versatile".to_string(),
        config: Some(config_map),
        limit: None,
    };

    let provider = GroqProvider::new(config).expect("Failed to create Groq provider");

    // Groq's list_models returns a static list, so this should always succeed
    let models = provider
        .list_models()
        .await
        .expect("Groq list_models should return static list");

    // Verify we got some models
    assert!(
        !models.is_empty(),
        "Groq should return at least one model"
    );

    // Verify expected models are in the list
    let model_names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();

    assert!(
        model_names.contains(&"llama-3.3-70b-versatile"),
        "Should include llama-3.3-70b-versatile. Found: {:?}",
        model_names
    );

    assert!(
        model_names.contains(&"mixtral-8x7b-32768"),
        "Should include mixtral-8x7b-32768. Found: {:?}",
        model_names
    );

    println!("✅ Groq list_models test passed with {} models", models.len());
}
