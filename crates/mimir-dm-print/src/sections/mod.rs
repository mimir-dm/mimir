//! Composable document sections that implement `Renderable`
//!
//! Each section type produces Typst markup and can be appended to a DocumentBuilder.
//! All sections generate Typst directly using shared components from `/_shared/`.
//!
//! ## Available Sections
//!
//! **Character & Entity Sections:**
//! - `CharacterSheetSection` - Full character sheet
//! - `NpcAppendix` - NPC reference cards
//!
//! **Content Sections:**
//! - `MarkdownSection` - Markdown documents with YAML frontmatter
//! - `MonsterAppendix` - Monster stat blocks (simplified for appendix)
//!
//! **Map Sections:**
//! - `MapPreview` - Map fit to single page
//! - `TiledMapSection` - Map at true scale, tiled across pages
//! - `TokenCutoutSheet` - Token standees for cutting

mod character;
mod character_summary;
mod encounter;
mod map;
mod markdown;
mod monster_cards;
mod monster_stat_block;
mod monsters;
mod npc_card;
mod npcs;
mod spell_list;
mod spells;
mod tokens;

pub use character::CharacterSheetSection;
pub use character_summary::CharacterSummarySection;
pub use encounter::EncounterSection;
pub use map::{MapPreview, TileData, TiledMapSection};
pub use markdown::MarkdownSection;
pub use monster_cards::MonsterCardSection;
pub use monster_stat_block::MonsterStatBlockSection;
pub use monsters::MonsterAppendix;
pub use npc_card::NpcIndexCardSection;
pub use npcs::NpcAppendix;
pub use spell_list::SpellListSection;
pub use spells::SpellCardsSection;
pub use tokens::TokenCutoutSheet;
