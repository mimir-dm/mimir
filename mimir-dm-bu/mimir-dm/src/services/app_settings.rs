//! Application settings management and persistence
//!
//! This module handles saving and loading general application preferences.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Application settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Whether the AI assistant feature is enabled (default: false - opt-in)
    #[serde(default)]
    pub ai_assistant_enabled: bool,

    /// Whether the MCP server is enabled for Claude Code/Desktop integration (default: false)
    #[serde(default)]
    pub mcp_server_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Default to disabled per acceptance criteria - fully opt-in
            ai_assistant_enabled: false,
            mcp_server_enabled: false,
        }
    }
}

impl AppSettings {
    /// Load app settings from file
    pub fn load(config_dir: &Path) -> Result<Self> {
        let config_path = config_dir.join("app_settings.json");

        if !config_path.exists() {
            debug!("App settings file not found, using defaults");
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read app settings from {:?}", config_path))?;

        let settings: AppSettings = serde_json::from_str(&contents)
            .with_context(|| "Failed to parse app settings JSON")?;

        info!(
            "Loaded app settings: ai_assistant_enabled={}, mcp_server_enabled={}",
            settings.ai_assistant_enabled, settings.mcp_server_enabled
        );
        Ok(settings)
    }

    /// Save app settings to file
    pub fn save(&self, config_dir: &Path) -> Result<()> {
        let config_path = config_dir.join("app_settings.json");

        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize app settings")?;

        fs::write(&config_path, json)
            .with_context(|| format!("Failed to write app settings to {:?}", config_path))?;

        info!(
            "Saved app settings: ai_assistant_enabled={}, mcp_server_enabled={}",
            self.ai_assistant_enabled, self.mcp_server_enabled
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert!(!settings.ai_assistant_enabled);
        assert!(!settings.mcp_server_enabled);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let settings = AppSettings {
            ai_assistant_enabled: true,
            mcp_server_enabled: true,
        };

        // Save
        settings.save(&config_dir).unwrap();

        // Load
        let loaded = AppSettings::load(&config_dir).unwrap();
        assert!(loaded.ai_assistant_enabled);
        assert!(loaded.mcp_server_enabled);
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let loaded = AppSettings::load(&config_dir).unwrap();
        assert!(!loaded.ai_assistant_enabled);
        assert!(!loaded.mcp_server_enabled);
    }
}
