//! Provider settings management and persistence
//!
//! This module handles saving and loading LLM provider configuration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Provider type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Ollama,
    Groq,
}

/// Ollama-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    /// Selected model name (e.g., "gpt-oss:20b", "qwen3:8b", "llama3.2")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: None,
        }
    }
}

/// Groq-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqConfig {
    pub api_key: String,
    /// Selected model name (e.g., "llama-3.3-70b-versatile", "mixtral-8x7b-32768")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Provider settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub provider_type: ProviderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama_config: Option<OllamaConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groq_config: Option<GroqConfig>,
    /// Timeout in seconds for tool confirmation prompts (default: 60)
    #[serde(default = "default_tool_confirmation_timeout")]
    pub tool_confirmation_timeout_secs: u64,
}

/// Default tool confirmation timeout in seconds
fn default_tool_confirmation_timeout() -> u64 {
    60
}

impl Default for ProviderSettings {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Ollama,
            ollama_config: Some(OllamaConfig::default()),
            groq_config: None,
            tool_confirmation_timeout_secs: default_tool_confirmation_timeout(),
        }
    }
}

impl ProviderSettings {
    /// Validate the settings
    pub fn validate(&self) -> Result<()> {
        match self.provider_type {
            ProviderType::Ollama => {
                if self.ollama_config.is_none() {
                    anyhow::bail!("Ollama provider selected but no Ollama configuration provided");
                }
            }
            ProviderType::Groq => {
                if self.groq_config.is_none() {
                    anyhow::bail!("Groq provider selected but no Groq configuration provided");
                }
                let groq_config = self.groq_config.as_ref().unwrap();
                if groq_config.api_key.trim().is_empty() {
                    anyhow::bail!("Groq API key cannot be empty");
                }
            }
        }
        Ok(())
    }

    /// Load provider settings from file
    pub fn load(config_dir: &Path) -> Result<Self> {
        let config_path = config_dir.join("provider_settings.json");

        if !config_path.exists() {
            debug!("Provider settings file not found, using defaults");
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read provider settings from {:?}", config_path))?;

        let settings: ProviderSettings = serde_json::from_str(&contents)
            .with_context(|| "Failed to parse provider settings JSON")?;

        settings.validate()?;

        info!("Loaded provider settings: {:?}", settings.provider_type);
        Ok(settings)
    }

    /// Save provider settings to file
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        self.validate()?;

        let config_path = config_dir.join("provider_settings.json");

        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize provider settings")?;

        fs::write(&config_path, json)
            .with_context(|| format!("Failed to write provider settings to {:?}", config_path))?;

        info!("Saved provider settings: {:?}", self.provider_type);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = ProviderSettings::default();
        assert_eq!(settings.provider_type, ProviderType::Ollama);
        assert!(settings.ollama_config.is_some());
        assert!(settings.groq_config.is_none());
        assert_eq!(settings.tool_confirmation_timeout_secs, 60);
    }

    #[test]
    fn test_validate_ollama_settings() {
        let settings = ProviderSettings {
            provider_type: ProviderType::Ollama,
            ollama_config: Some(OllamaConfig::default()),
            groq_config: None,
            tool_confirmation_timeout_secs: 60,
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_ollama_missing_config() {
        let settings = ProviderSettings {
            provider_type: ProviderType::Ollama,
            ollama_config: None,
            groq_config: None,
            tool_confirmation_timeout_secs: 60,
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_groq_settings() {
        let settings = ProviderSettings {
            provider_type: ProviderType::Groq,
            ollama_config: None,
            groq_config: Some(GroqConfig {
                api_key: "test-key".to_string(),
                model: None,
            }),
            tool_confirmation_timeout_secs: 60,
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_groq_empty_api_key() {
        let settings = ProviderSettings {
            provider_type: ProviderType::Groq,
            ollama_config: None,
            groq_config: Some(GroqConfig {
                api_key: "".to_string(),
                model: None,
            }),
            tool_confirmation_timeout_secs: 60,
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let settings = ProviderSettings {
            provider_type: ProviderType::Groq,
            ollama_config: None,
            groq_config: Some(GroqConfig {
                api_key: "test-api-key".to_string(),
                model: Some("llama-3.3-70b-versatile".to_string()),
            }),
            tool_confirmation_timeout_secs: 90,
        };

        // Save
        settings.save(&config_dir).unwrap();

        // Load
        let loaded = ProviderSettings::load(&config_dir).unwrap();
        assert_eq!(loaded.provider_type, ProviderType::Groq);
        assert!(loaded.groq_config.is_some());
        let groq_config = loaded.groq_config.unwrap();
        assert_eq!(groq_config.api_key, "test-api-key");
        assert_eq!(
            groq_config.model,
            Some("llama-3.3-70b-versatile".to_string())
        );
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let loaded = ProviderSettings::load(&config_dir).unwrap();
        assert_eq!(loaded.provider_type, ProviderType::Ollama);
    }

    #[test]
    fn test_save_and_load_ollama_with_model() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let settings = ProviderSettings {
            provider_type: ProviderType::Ollama,
            ollama_config: Some(OllamaConfig {
                base_url: "http://localhost:11434".to_string(),
                model: Some("gpt-oss:20b".to_string()),
            }),
            groq_config: None,
            tool_confirmation_timeout_secs: 60,
        };

        // Save
        settings.save(&config_dir).unwrap();

        // Load
        let loaded = ProviderSettings::load(&config_dir).unwrap();
        assert_eq!(loaded.provider_type, ProviderType::Ollama);
        assert!(loaded.ollama_config.is_some());
        let ollama_config = loaded.ollama_config.unwrap();
        assert_eq!(ollama_config.base_url, "http://localhost:11434");
        assert_eq!(ollama_config.model, Some("gpt-oss:20b".to_string()));
    }

    #[test]
    fn test_load_legacy_settings_without_model() {
        // Test backwards compatibility with settings files that don't have a model field
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        // Write a legacy settings file without model field
        let legacy_json = r#"{
            "provider_type": "ollama",
            "ollama_config": {
                "base_url": "http://localhost:11434"
            },
            "tool_confirmation_timeout_secs": 60
        }"#;

        let config_path = config_dir.join("provider_settings.json");
        fs::write(&config_path, legacy_json).unwrap();

        // Load should succeed and model should be None
        let loaded = ProviderSettings::load(&config_dir).unwrap();
        assert_eq!(loaded.provider_type, ProviderType::Ollama);
        assert!(loaded.ollama_config.is_some());
        let ollama_config = loaded.ollama_config.unwrap();
        assert_eq!(ollama_config.base_url, "http://localhost:11434");
        assert_eq!(ollama_config.model, None);
    }
}
