//! mimir-dm-print - PDF generation for Mimir using Typst
//!
//! This crate provides print and PDF generation capabilities for Mimir,
//! using the Typst document compiler.
//!
//! # Two Rendering Approaches
//!
//! ## 1. Standalone Template Rendering
//!
//! For high-fidelity single documents (character sheets, spell cards, etc.):
//!
//! ```ignore
//! use mimir_dm_print::PrintService;
//! use serde_json::json;
//!
//! let service = PrintService::new("/path/to/templates".into());
//! let data = json!({"character_name": "Gandalf", "level": 20});
//! let pdf_bytes = service.render_to_pdf("character/sheet.typ", data)?;
//! ```
//!
//! ## 2. Composed Document Building
//!
//! For combining multiple content types (campaign exports, module references):
//!
//! ```ignore
//! use mimir_dm_print::{DocumentBuilder, MarkdownSection, MonsterAppendix};
//!
//! let pdf = DocumentBuilder::new("Campaign Guide")
//!     .with_toc(true)
//!     .append(MarkdownSection::from_file(&doc_path)?)
//!     .append(MonsterAppendix::new(monsters_json))
//!     .to_pdf()?;
//! ```

pub mod builder;
pub mod campaign;
pub mod character;
pub mod commands;
pub mod error;
pub mod map_renderer;
pub mod maps;
pub mod markdown;
pub mod sections;
pub mod service;
pub mod world;

pub use campaign::{build_campaign_pdf, build_single_document_pdf, CampaignExportData, ExportOptions};
pub use character::{export_character_pdf, generate_character_sheet_pdf, CharacterExportOptions};
pub use maps::{generate_map_pdf, slice_map_into_tiles, MapPdfOptions};

pub use builder::{DocumentBuilder, DocumentConfig, RenderContext, Renderable, VirtualFileRegistry};
pub use error::{PrintError, Result};
pub use sections::{
    CharacterLongFormSection, CharacterSheetSection, CharacterSummarySection, CompactSheetSection,
    EncounterSection, EquipmentDetailSection, MapPreview, MarkdownSection, MonsterAppendix,
    MonsterCardSection, MonsterStatBlockSection, NpcAppendix, NpcIndexCardSection, SpellCardsSection,
    SpellListSection, TileData, TiledMapSection, TokenCutoutSheet,
};
pub use map_renderer::{
    render_map, render_map_for_print, MapPrintOptions, RenderMap, RenderToken, RenderedMap,
    RenderedMapForPrint,
};
pub use markdown::{markdown_to_typst, parse_campaign_document, ParsedDocument};
pub use service::{PrintService, TemplateInfo};
pub use world::MimirTypstWorld;

#[cfg(feature = "tauri-commands")]
pub use commands::PrintState;
