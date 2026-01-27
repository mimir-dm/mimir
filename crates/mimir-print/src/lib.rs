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

pub mod error;
pub mod world;
pub mod service;
pub mod builder;
pub mod markdown;
pub mod map_renderer;
pub mod sections;
pub mod embedded_templates;

#[cfg(feature = "tauri-commands")]
pub mod commands;

pub use error::{PrintError, Result};
pub use world::MimirTypstWorld;
pub use service::{PrintService, TemplateInfo};
pub use builder::{DocumentBuilder, DocumentConfig, Renderable, RenderContext, VirtualFileRegistry, escape_typst_string};
pub use markdown::{ParsedDocument, parse_campaign_document, markdown_to_typst};
pub use sections::MarkdownSection;
pub use sections::{CharacterData, CharacterSection, ClassInfo, InventoryItem};
pub use sections::{CharacterBattleCardSection, CharacterLongFormSection};
pub use sections::{is_card_worthy, EquipmentCardsSection, EquipmentDetailSection};
pub use sections::{MapPreview, TileData, TiledMapSection};
pub use sections::{MonsterCardSection, TrapCardSection};
pub use sections::{SpellCardsSection};
pub use sections::{CutoutToken, TokenCutoutSection};
pub use map_renderer::{MapPrintOptions, RenderMap, RenderToken, RenderedMapForPrint};

#[cfg(feature = "tauri-commands")]
pub use commands::PrintState;
