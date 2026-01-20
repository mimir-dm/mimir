//! # Configuration Module
//!
//! This module handles the configuration of LLM providers and models.
//!
//! ## Overview
//!
//! The configuration system supports:
//! - Multiple endpoint types (chat, completion, embedding)
//! - Rate limiting with configurable periods
//! - Provider-specific settings
//! - YAML-based configuration files
//!
//! ## Configuration File Format
//!
//! ```yaml
//! name: llama3-config
//! supported_endpoints:
//!   - chat
//!   - completion
//!   - embedding
//! provider: ollama
//! model: llama3
//! config:
//!   base_url: "http://localhost:11434"
//!   temperature: "0.7"
//!   max_tokens: "1000"
//! limit:
//!   renewal_period: minutes
//!   calls: 60
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Supported endpoint types for models
///
/// Each endpoint type represents a different way of interacting with the LLM:
/// - `Chat`: For conversational interactions
/// - `Completion`: For text completion tasks
/// - `Embedding`: For generating vector embeddings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EndpointType {
    /// Generate vector embeddings from text
    Embedding,
    /// Conversational chat interface
    Chat,
    /// Text completion interface
    Completion,
}

/// Rate limit renewal period
///
/// Defines the time period after which the rate limit counter resets.
/// This can be specified in seconds, minutes, or hours.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RenewalPeriod {
    /// Reset counter every second
    Seconds,
    /// Reset counter every minute
    Minutes,
    /// Reset counter every hour
    Hours,
}

/// Rate limiting configuration
///
/// Controls how many API calls can be made within a given time period.
/// For example, a rate limit of 60 calls per minute would be:
/// ```rust
/// # use mimir_dm_llm::config::{RateLimit, RenewalPeriod};
/// RateLimit {
///     renewal_period: RenewalPeriod::Minutes,
///     calls: 60,
/// }
/// # ;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Time period for rate limit renewal
    pub renewal_period: RenewalPeriod,
    /// Number of allowed calls per renewal period
    pub calls: u32,
}

/// Base configuration that all models will use
///
/// This struct contains all the necessary configuration for a model,
/// including supported endpoints, provider information, and rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Unique identifier for this configuration
    pub name: String,
    /// List of supported endpoint types
    pub supported_endpoints: Vec<EndpointType>,
    /// Name of the provider (e.g., "ollama", "openai")
    pub provider: String,
    /// Name of the model (e.g., "llama3", "gpt-4")
    pub model: String,
    /// Optional model-specific configuration
    ///
    /// This can include parameters like temperature, max_tokens, etc.
    /// The exact parameters depend on the provider and model.
    pub config: Option<HashMap<String, String>>,
    /// Optional rate limiting configuration
    ///
    /// If not specified, no rate limiting will be applied.
    pub limit: Option<RateLimit>,
}

/// Errors that can occur when loading configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// IO error when reading config file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Error parsing YAML configuration.
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

impl ModelConfig {
    /// Load a model configuration from a YAML file
    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        Self::from_yaml_str(&contents)
    }

    /// Load a model configuration from a YAML string
    pub fn from_yaml_str(yaml: &str) -> Result<Self, ConfigError> {
        let config: Self = serde_yaml::from_str(yaml)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_valid_config() -> String {
        r#"
name: test-model
supported_endpoints:
  - chat
  - embedding
provider: ollama
model: llama3
config:
  base_url: "http://localhost:11434"
  temperature: "0.7"
  max_tokens: "1000"
limit:
  renewal_period: minutes
  calls: 60
"#
        .to_string()
    }

    #[test]
    fn test_valid_config() {
        let config = ModelConfig::from_yaml_str(&create_valid_config()).unwrap();
        assert_eq!(config.name, "test-model");
        assert_eq!(config.provider, "ollama");
        assert_eq!(config.model, "llama3");
        assert_eq!(config.supported_endpoints.len(), 2);
        assert!(config.supported_endpoints.contains(&EndpointType::Chat));
        assert!(config
            .supported_endpoints
            .contains(&EndpointType::Embedding));
        assert!(config.config.is_some());
        assert!(config.limit.is_some());
    }

    #[test]
    fn test_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let yaml = create_valid_config();
        temp_file.write_all(yaml.as_bytes()).unwrap();

        let result = ModelConfig::from_yaml_file(temp_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_endpoint_type() {
        let invalid_yaml = r#"
name: test-model
supported_endpoints:
  - invalid
provider: ollama
model: llama3
"#;
        let result = ModelConfig::from_yaml_str(invalid_yaml);
        assert!(result.is_err());
    }
}

/// Simple configuration for file tools - just manages allowed directories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileToolsConfig {
    /// Directories that file tools can access
    pub allowed_directories: Vec<PathBuf>,
}

impl FileToolsConfig {
    /// Create a new config with the specified allowed directories
    pub fn new(allowed_directories: Vec<PathBuf>) -> Self {
        Self {
            allowed_directories,
        }
    }

    /// Create a config with a single root directory
    pub fn with_root(root_directory: PathBuf) -> Self {
        Self {
            allowed_directories: vec![root_directory],
        }
    }
}
