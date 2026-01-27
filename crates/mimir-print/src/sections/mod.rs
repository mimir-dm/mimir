//! Document sections for PDF assembly
//!
//! Each section type implements `Renderable` to produce Typst markup
//! that can be assembled into a complete PDF document.

pub mod character;
pub mod character_battle_card;
pub mod character_longform;
pub mod equipment_cards;
pub mod equipment_detail;
pub mod map;
pub mod markdown;
pub mod monster_cards;
pub mod spell_cards;
pub mod token_cutouts;
pub mod trap_cards;

pub use character::{CharacterData, CharacterSection, ClassInfo, InventoryItem, Proficiencies, ProficiencyEntry};
pub use character_battle_card::CharacterBattleCardSection;
pub use character_longform::CharacterLongFormSection;
pub use equipment_cards::{is_card_worthy, EquipmentCardsSection};
pub use equipment_detail::EquipmentDetailSection;
pub use map::{MapPreview, TileData, TiledMapSection};
pub use markdown::MarkdownSection;
pub use monster_cards::MonsterCardSection;
pub use spell_cards::SpellCardsSection;
pub use token_cutouts::{CutoutToken, TokenCutoutSection};
pub use trap_cards::TrapCardSection;
