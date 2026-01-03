//! Composable document sections that implement `Renderable`
//!
//! Each section type produces Typst markup and can be appended to a DocumentBuilder.
//! All sections generate Typst directly using shared components from `/_shared/`.
//!
//! ## Available Sections
//!
//! **Character & Entity Sections:**
//! - `CharacterSheetSection` - Full character sheet (single page)
//! - `CompactSheetSection` - 2-page WotC-style character sheet
//! - `CharacterLongFormSection` - Extended character details (personality, background, RP notes)
//! - `EquipmentCardsSection` - Printable equipment cards (2.5in x 3.5in)
//! - `EquipmentDetailSection` - Detailed inventory list
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
mod character_longform;
mod character_summary;
mod compact_sheet;
mod encounter;
mod equipment;
mod equipment_detail;
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
pub use character_longform::CharacterLongFormSection;
pub use character_summary::CharacterSummarySection;
pub use compact_sheet::CompactSheetSection;
pub use encounter::EncounterSection;
pub use equipment::{is_card_worthy, EquipmentCardsSection};
pub use equipment_detail::EquipmentDetailSection;
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
