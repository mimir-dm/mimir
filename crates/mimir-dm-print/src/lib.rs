//! mimir-dm-print - PDF generation for Mimir using Typst
//!
//! This crate provides print and PDF generation capabilities for Mimir,
//! using the Typst document compiler. It supports:
//!
//! - Template-based document generation
//! - Data injection from JSON
//! - System font support
//! - Multiple output formats (PDF)
//!
//! # Architecture
//!
//! The crate is organized around a few key components:
//!
//! - [`PrintService`] - Main service for rendering templates to PDF
//! - [`MimirTypstWorld`] - Custom Typst World implementation for Mimir
//! - [`commands`] - Tauri commands (when `tauri-commands` feature enabled)
//!
//! # Example
//!
//! ```ignore
//! use mimir_dm_print::PrintService;
//! use serde_json::json;
//!
//! let service = PrintService::new("/path/to/templates".into());
//!
//! let data = json!({
//!     "character_name": "Gandalf",
//!     "level": 20,
//!     "class": "Wizard"
//! });
//!
//! let pdf_bytes = service.render_to_pdf("character/sheet.typ", data)?;
//! std::fs::write("character_sheet.pdf", pdf_bytes)?;
//! ```

mod campaign;
pub mod commands;
pub mod error;
pub mod map_renderer;
pub mod markdown;
pub mod service;
pub mod world;

pub use error::{PrintError, Result};
pub use map_renderer::{
    render_map, render_map_for_print, MapPrintOptions, RenderMap, RenderToken, RenderedMap,
    RenderedMapForPrint,
};
pub use markdown::{markdown_to_typst, parse_campaign_document, ParsedDocument};
pub use service::{PrintService, TemplateInfo};
pub use world::MimirTypstWorld;

#[cfg(feature = "tauri-commands")]
pub use commands::PrintState;
