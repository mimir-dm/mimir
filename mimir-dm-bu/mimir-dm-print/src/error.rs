//! Print-specific error types

use thiserror::Error;

/// Errors that can occur during print/PDF generation
#[derive(Error, Debug)]
pub enum PrintError {
    /// Template file not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Failed to read template file
    #[error("Failed to read template: {0}")]
    TemplateReadError(String),

    /// Typst compilation failed
    #[error("Typst compilation failed: {0}")]
    CompilationError(String),

    /// PDF generation failed
    #[error("PDF generation failed: {0}")]
    PdfError(String),

    /// Invalid data provided for template
    #[error("Invalid template data: {0}")]
    InvalidData(String),

    /// Font loading failed
    #[error("Font loading failed: {0}")]
    FontError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PrintError>;
