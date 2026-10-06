//! mimir-print - PDF generation for Mimir using Typst
//!
//! This crate provides print and PDF generation capabilities for Mimir,
//! using the Typst document compiler.
//!
//! # Architecture
//!
//! The crate is organized into several layers:
//!
//! - **world**: Custom Typst World implementation for file/font resolution
//! - **service**: High-level PrintService for template-based PDF generation
//! - **builder**: Composable DocumentBuilder for assembling multi-section documents
//! - **markdown**: Markdown to Typst conversion with frontmatter support
//! - **sections**: Renderable document sections (markdown, monsters, maps, etc.)
//! - **map_renderer**: Map image rendering with grid, LOS walls, and tokens
//!
//! # Usage
//!
//! ## Simple Template Rendering
//!
//! ```ignore
//! use mimir_print::PrintService;
//!
//! let service = PrintService::new(templates_path);
//! let pdf = service.render_to_pdf("character/sheet.typ", data)?;
//! ```
//!
//! ## Multi-Section Document Assembly
//!
//! ```ignore
//! use mimir_print::{DocumentBuilder, MarkdownSection};
//!
//! let pdf = DocumentBuilder::new("Campaign Guide")
//!     .with_toc(true)
//!     .append(MarkdownSection::from_file(&doc_path)?)
//!     .append(MarkdownSection::from_file(&session_notes)?)
//!     .to_pdf()?;
//! ```

use std::path::PathBuf;

pub mod builder;
pub mod embedded_templates;
pub mod error;
pub mod map_renderer;
pub mod markdown;
pub mod sections;
pub mod service;
pub mod world;

pub use builder::{
    escape_typst_string, DocumentBuilder, DocumentConfig, RenderContext, Renderable,
    VirtualFileRegistry,
};
pub use error::{PrintError, Result};
pub use map_renderer::{MapPrintOptions, RenderMap, RenderToken, RenderedMapForPrint};
pub use markdown::{markdown_to_typst, parse_campaign_document, ParsedDocument};
pub use sections::CharacterBattleCardSection;
pub use sections::MarkdownSection;
pub use sections::SpellCardsSection;
pub use sections::{is_card_worthy, EquipmentCardsSection};
pub use sections::{CharacterData, CharacterSection, ClassInfo, InventoryItem};
pub use sections::{CutoutToken, TokenCutoutSection};
pub use sections::{MapPreview, TileData, TiledMapSection};
pub use sections::{MonsterCardSection, TrapCardSection};
pub use service::{PrintService, TemplateInfo};
pub use world::MimirTypstWorld;

/// State for print functionality, managed by Tauri.
pub struct PrintState {
    /// Path to templates directory
    pub templates_dir: PathBuf,
    /// Path to assets directory
    pub assets_dir: PathBuf,
}

impl PrintState {
    /// Create a new PrintState
    pub fn new(templates_dir: PathBuf, assets_dir: PathBuf) -> Self {
        Self {
            templates_dir,
            assets_dir,
        }
    }
}
