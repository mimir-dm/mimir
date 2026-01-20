//! Tauri commands for print functionality
//!
//! These commands are only available when the `tauri-commands` feature is enabled.

#[cfg(feature = "tauri-commands")]
use tauri::State;

use crate::error::Result;
use crate::service::{PrintService, TemplateInfo};

/// State wrapper for PrintService in Tauri
#[cfg(feature = "tauri-commands")]
pub struct PrintState(pub PrintService);

/// Generate a PDF from a template
///
/// # Arguments
/// * `template` - Path to template relative to templates root
/// * `data` - JSON data to inject into template
///
/// # Returns
/// PDF file as base64-encoded string
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn generate_pdf(
    state: State<'_, PrintState>,
    template: String,
    data: serde_json::Value,
) -> std::result::Result<String, String> {
    let pdf_bytes = state
        .0
        .render_to_pdf(&template, data)
        .map_err(|e| e.to_string())?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&pdf_bytes))
}

/// List all available templates
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn list_templates(
    state: State<'_, PrintState>,
) -> std::result::Result<Vec<TemplateInfo>, String> {
    state.0.list_templates().map_err(|e| e.to_string())
}

/// Save PDF to a file
///
/// # Arguments
/// * `path` - Destination file path
/// * `pdf_base64` - PDF content as base64-encoded string
#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn save_pdf(path: String, pdf_base64: String) -> std::result::Result<(), String> {
    use base64::Engine;

    let pdf_bytes = base64::engine::general_purpose::STANDARD
        .decode(&pdf_base64)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    std::fs::write(&path, &pdf_bytes).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Non-Tauri versions for direct use
pub mod direct {
    use super::*;
    use std::path::PathBuf;

    /// Generate PDF directly (without Tauri)
    pub fn generate_pdf(
        service: &PrintService,
        template: &str,
        data: serde_json::Value,
    ) -> Result<Vec<u8>> {
        service.render_to_pdf(template, data)
    }

    /// List templates directly (without Tauri)
    pub fn list_templates(service: &PrintService) -> Result<Vec<TemplateInfo>> {
        service.list_templates()
    }

    /// Save PDF directly (without Tauri)
    pub fn save_pdf(path: &PathBuf, pdf_bytes: &[u8]) -> Result<()> {
        std::fs::write(path, pdf_bytes)?;
        Ok(())
    }
}
